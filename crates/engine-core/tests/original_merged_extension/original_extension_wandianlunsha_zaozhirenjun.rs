//! Golden for 玩点论杀 枣祗任峻【粮策/坚壁/军屯】、【五谷丰登】与【望梅止渴】（ABI 616）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LIANGCE: u32 = 3_473_735_644;
const JIANBI: u32 = 3_452_664_776;
const DIYJUNTUN: u32 = 2_307_382_183;
const SOURCE: &str =
    include_str!("../../../../content/original/ported/wandianlunsha/zaozhirenjun.skill");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package =
        content_compiler::compile_source_named("wandianlunsha-zaozhirenjun", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 616);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend(
        [LIANGCE, JIANBI, DIYJUNTUN].map(|skill_def| EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, players)
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    serial: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_000 + serial),
        kind,
        suit: Suit::Club,
        rank: 8,
        weapon_range: if kind == CardKind::Weapon { 3 } else { 0 },
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == def)
        })
        .unwrap()
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn choose(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

#[test]
fn package_exposes_exact_abi_and_three_skill_surfaces() {
    let package =
        content_compiler::compile_source_named("wandianlunsha-zaozhirenjun", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 616);
    assert_eq!(package.skill_index.len(), 3);
    assert_eq!(package.trigger_index.len(), 2);
    assert_eq!(package.active_index.len(), 1);
    assert_eq!(package.view_as_index.len(), 1);
    assert_eq!(package.view_as_index[0].as_kind, "wugu");
}

#[test]
fn diyjuntun_recasts_an_equipment_from_the_equip_zone() {
    let (mut state, players) = setup();
    let owner = players[0];
    let equipment = inject(&mut state, owner, ZoneKind::Equip, CardKind::Weapon, 1);
    let replacement = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Sha,
        2,
    );
    let diyjuntun = skill(&state, owner, DIYJUNTUN);
    let (state, result) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill: diyjuntun,
            targets: Vec::new(),
            card_targets: vec![equipment],
        },
        10,
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert!(state.hand_cards(owner).contains(&replacement));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state.zones.get(discard).unwrap().cards.contains(&equipment));
}

#[test]
fn jianbi_excludes_one_selected_tiesuo_target() {
    let (mut state, players) = setup();
    let owner = players[0];
    let actor = players[1];
    let other = players[2];
    state.turn.current = actor;
    let card = inject(&mut state, actor, ZoneKind::Hand, CardKind::TrickTiesuo, 10);
    let (state, result) = submit(
        state,
        actor,
        CommandKind::PlayCard {
            card,
            targets: vec![owner, other],
            card_targets: Vec::new(),
        },
        20,
    );
    assert_eq!(result.choice.as_ref().unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![1], false, 21);
    assert_eq!(result.choice.as_ref().unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![other.raw()], false, 22);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    let chained = |player| {
        state
            .players
            .get(player)
            .unwrap()
            .marks
            .iter()
            .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0)
    };
    assert!(chained(owner));
    assert!(!chained(other));
}

#[test]
fn wugu_reveals_one_per_target_and_each_target_takes_one() {
    let (mut state, players) = setup();
    let actor = players[1];
    state.turn.current = actor;
    let card = inject(&mut state, actor, ZoneKind::Hand, CardKind::TrickWugu, 20);
    for serial in 21..=23 {
        inject(
            &mut state,
            PlayerId::NONE,
            ZoneKind::DrawPile,
            CardKind::Sha,
            serial,
        );
    }
    let (mut state, mut result) = submit(
        state,
        actor,
        CommandKind::PlayCard {
            card,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        30,
    );
    let mut key = 31;
    loop {
        let Some(choice) = result.choice.clone() else {
            break;
        };
        if choice.kind == engine_api::ChoiceKind::SelectCards {
            (state, result) = choose(
                state,
                choice.observer,
                vec![choice.options[0].token],
                false,
                key,
            );
        } else {
            // Do not activate 坚壁 in this test.
            (state, result) = choose(state, choice.observer, Vec::new(), true, key);
        }
        key += 1;
    }
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert!(players.iter().all(|player| state.hand_count(*player) == 1));
}

#[test]
fn jianbi_excludes_a_wugu_target_without_reducing_the_revealed_count() {
    let (mut state, players) = setup();
    let owner = players[0];
    let actor = players[1];
    let excluded = players[2];
    state.turn.current = actor;
    let card = inject(&mut state, actor, ZoneKind::Hand, CardKind::TrickWugu, 24);
    for serial in 25..=27 {
        inject(
            &mut state,
            PlayerId::NONE,
            ZoneKind::DrawPile,
            CardKind::Sha,
            serial,
        );
    }
    let (state, result) = submit(
        state,
        actor,
        CommandKind::PlayCard {
            card,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        35,
    );
    assert_eq!(result.choice.as_ref().unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![1], false, 36);
    assert!(result.choice.is_some(), "{result:?}");
    assert_eq!(
        result.choice.as_ref().unwrap().kind,
        engine_api::ChoiceKind::SelectTargets
    );
    let (mut state, mut result) = choose(state, owner, vec![excluded.raw()], false, 37);
    let mut key = 38;
    loop {
        let Some(choice) = result.choice.clone() else {
            break;
        };
        if choice.kind == engine_api::ChoiceKind::SelectCards {
            (state, result) = choose(
                state,
                choice.observer,
                vec![choice.options[0].token],
                false,
                key,
            );
        } else {
            // Decline 粮策 after the single remaining revealed card reaches discard.
            (state, result) = choose(state, choice.observer, Vec::new(), true, key);
        }
        key += 1;
    }
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.hand_count(owner), 1);
    assert_eq!(state.hand_count(actor), 1);
    assert_eq!(state.hand_count(excluded), 0);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(
        state
            .zones
            .get(discard)
            .unwrap()
            .cards
            .iter()
            .filter(|card| state.cards.get(**card).unwrap().def.raw() >= 99_025)
            .count(),
        1
    );
}

#[test]
fn liangce_can_give_all_wugu_remainders_still_in_discard() {
    let (mut state, players) = setup();
    let owner = players[0];
    let recipient = players[2];
    let left = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::Processing,
        CardKind::Sha,
        30,
    );
    let right = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::Processing,
        CardKind::Shan,
        31,
    );
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::ContinueWugu {
            action: 2,
            actor: players[1],
            source_card: CardId::NONE,
            targets: Vec::new(),
            excluded: Vec::new(),
            revealed: vec![left, right],
        }],
    );
    assert!(matches!(result, Err(EffectError::ChoiceRequested)));
    let committed = tx.commit();
    assert_eq!(committed.output.choice.unwrap().observer, owner);
    let (state, result) = choose(committed.state, owner, vec![1], false, 40);
    assert_eq!(result.choice.as_ref().unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![recipient.raw()], false, 41);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert!(state.hand_cards(recipient).contains(&left));
    assert!(state.hand_cards(recipient).contains(&right));
}

#[test]
fn wangmeizhike_recomputes_min_hand_after_recovery() {
    let (mut state, players) = setup();
    let actor = players[1];
    let target = players[2];
    state.turn.current = actor;
    state.players.get_mut(target).unwrap().hp = 2;
    state.players.get_mut(target).unwrap().max_hp = 3;
    for player in [players[0], actor] {
        inject(
            &mut state,
            player,
            ZoneKind::Hand,
            CardKind::Other,
            40 + player.raw(),
        );
    }
    let card = inject(
        &mut state,
        actor,
        ZoneKind::Hand,
        CardKind::TrickWangmeizhike,
        50,
    );
    let drawn = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Sha,
        51,
    );
    let (state, result) = submit(
        state,
        actor,
        CommandKind::PlayCard {
            card,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        50,
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    assert_eq!(state.hand_count(target), 1);
    assert!(state.hand_cards(target).contains(&drawn));
}

#[test]
fn wangmeizhike_draws_two_when_it_does_not_recover() {
    let (mut state, players) = setup();
    let actor = players[1];
    let target = players[2];
    state.turn.current = actor;
    state.players.get_mut(target).unwrap().hp = 3;
    state.players.get_mut(target).unwrap().max_hp = 3;
    for player in [players[0], actor] {
        inject(
            &mut state,
            player,
            ZoneKind::Hand,
            CardKind::Other,
            60 + player.raw(),
        );
    }
    let card = inject(
        &mut state,
        actor,
        ZoneKind::Hand,
        CardKind::TrickWangmeizhike,
        70,
    );
    let first = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Sha,
        71,
    );
    let second = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Shan,
        72,
    );
    let (state, result) = submit(
        state,
        actor,
        CommandKind::PlayCard {
            card,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        60,
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    assert_eq!(state.hand_count(target), 2);
    assert!(state.hand_cards(target).contains(&first));
    assert!(state.hand_cards(target).contains(&second));
}

#[test]
fn wuxie_cancels_wangmeizhike_before_recovery_and_draw() {
    let (mut state, players) = setup();
    let responder = players[0];
    let actor = players[1];
    let target = players[2];
    state.turn.current = actor;
    state.players.get_mut(target).unwrap().hp = 2;
    state.players.get_mut(target).unwrap().max_hp = 3;
    let wuxie = inject(
        &mut state,
        responder,
        ZoneKind::Hand,
        CardKind::TrickWuxie,
        80,
    );
    let card = inject(
        &mut state,
        actor,
        ZoneKind::Hand,
        CardKind::TrickWangmeizhike,
        81,
    );
    inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Sha,
        82,
    );
    let (state, result) = submit(
        state,
        actor,
        CommandKind::PlayCard {
            card,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        70,
    );
    assert_eq!(result.choice.as_ref().unwrap().observer, responder);
    let (state, result) = choose(state, responder, vec![wuxie.raw()], false, 71);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.players.get(target).unwrap().hp, 2);
    assert_eq!(state.hand_count(target), 0);
}
