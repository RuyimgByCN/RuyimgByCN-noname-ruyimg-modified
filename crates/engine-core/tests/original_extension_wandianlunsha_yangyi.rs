//! Golden for archived extension 玩点论杀 杨仪【筹度/立断】与【调兵遣将】（ABI 615）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const CHOUDU: u32 = 4_166_492_495;
const LIDUAN: u32 = 2_934_618_094;
const SOURCE: &str = include_str!("../../../content/original/ported/wandianlunsha/yangyi.skill");

fn setup(players_count: u8) -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("wandianlunsha-yangyi", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 615);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, players_count, 20_260_803, "identity-standard").unwrap();
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
    effects.extend([
        EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(CHOUDU),
        },
        EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(LIDUAN),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[2.min(players.len() - 1)];
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
        def: CardDefId::new(98_000 + serial),
        kind,
        suit: Suit::Heart,
        rank: 7,
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

fn gain_one(
    state: GameState,
    target: PlayerId,
    card: CardId,
) -> (GameState, Option<engine_api::ChoiceRequest>) {
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let from = tx.state().cards.get(card).unwrap().zone;
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::MoveCardsWithReason {
            cards: vec![card],
            to: hand,
            from: Some(from),
            reason: engine_api::CardMoveReason::Unspecified,
        }],
    );
    assert!(matches!(result, Err(EffectError::ChoiceRequested)));
    let committed = tx.commit();
    (committed.state, committed.output.choice)
}

#[test]
fn package_exposes_exact_abi_active_and_gain_filter() {
    let package = content_compiler::compile_source_named("wandianlunsha-yangyi", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 615);
    assert_eq!(package.active_index.len(), 1);
    assert_eq!(package.trigger_index.len(), 1);
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_alive_other_gained_single_equip_to_hand_outside_turn_usable"
    );
}

#[test]
fn liduan_recipient_can_give_any_hand_card_to_yangyi() {
    let (mut state, players) = setup(3);
    let owner = players[0];
    let recipient = players[1];
    let gift = inject(&mut state, recipient, ZoneKind::Hand, CardKind::Sha, 1);
    let equipment = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Weapon,
        2,
    );
    let (state, choice) = gain_one(state, recipient, equipment);
    assert_eq!(choice.unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![1], false, 10);
    assert_eq!(result.choice.unwrap().observer, recipient);
    let (state, result) = choose(state, recipient, vec![gift.raw()], false, 11);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(owner).contains(&gift));
    assert!(state.hand_cards(recipient).contains(&equipment));
}

#[test]
fn liduan_cancel_forces_the_gained_equipment_to_be_used() {
    let (mut state, players) = setup(3);
    let owner = players[0];
    let recipient = players[1];
    let equipment = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Armor,
        3,
    );
    let (state, choice) = gain_one(state, recipient, equipment);
    assert_eq!(choice.unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![1], false, 20);
    assert_eq!(result.choice.unwrap().observer, recipient);
    let (state, result) = choose(state, recipient, Vec::new(), true, 21);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    let equip = state.zones.find(ZoneKind::Equip, recipient).unwrap();
    assert!(state.zones.get(equip).unwrap().cards.contains(&equipment));
}

#[test]
fn diaobing_swaps_revealed_card_then_preserves_chosen_top_order() {
    let (mut state, players) = setup(3);
    let actor = players[2];
    let target = players[1];
    state.turn.current = actor;
    let card = inject(
        &mut state,
        actor,
        ZoneKind::Hand,
        CardKind::TrickDiaobingqianjiang,
        10,
    );
    let replacement = inject(&mut state, target, ZoneKind::Hand, CardKind::Sha, 11);
    let pool_first = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Armor,
        12,
    );
    let pool_second = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::TrickWuzhong,
        13,
    );
    let drawn = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Shan,
        14,
    );
    let (state, result) = submit(
        state,
        actor,
        CommandKind::PlayCard {
            card,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        30,
    );
    assert_eq!(result.choice.unwrap().observer, actor);
    assert!(state.hand_cards(actor).contains(&drawn));
    let (state, result) = choose(state, actor, Vec::new(), true, 31);
    assert_eq!(result.choice.unwrap().observer, target);
    let (state, result) = choose(state, target, vec![pool_second.raw()], false, 32);
    assert_eq!(result.choice.unwrap().observer, target);
    let (state, result) = choose(state, target, vec![replacement.raw()], false, 33);
    assert_eq!(result.choice.unwrap().observer, actor);
    let (state, result) = choose(state, actor, vec![replacement.raw()], false, 34);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert!(state.hand_cards(target).contains(&pool_second));
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(
        state.zones.get(draw).unwrap().cards.last(),
        Some(&replacement)
    );
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state
        .zones
        .get(discard)
        .unwrap()
        .cards
        .contains(&pool_first));
}

#[test]
fn diaobing_opens_an_independent_wuxie_window_for_each_target() {
    let (mut state, players) = setup(3);
    let actor = players[2];
    let responder = players[0];
    state.turn.current = actor;
    let card = inject(
        &mut state,
        actor,
        ZoneKind::Hand,
        CardKind::TrickDiaobingqianjiang,
        15,
    );
    let wuxie = inject(
        &mut state,
        responder,
        ZoneKind::Hand,
        CardKind::TrickWuxie,
        16,
    );
    inject(&mut state, responder, ZoneKind::Hand, CardKind::Other, 19);
    inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Armor,
        17,
    );
    inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Sha,
        18,
    );
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
    assert_eq!(result.choice.unwrap().observer, responder);
    let (mut state, mut result) = choose(state, responder, vec![wuxie.raw()], false, 36);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    let mut key = 37;
    let choice = loop {
        let choice = result.choice.unwrap();
        if choice.kind == engine_api::ChoiceKind::SelectCards {
            break choice;
        }
        (state, result) = choose(state, choice.observer, Vec::new(), true, key);
        key += 1;
    };
    assert_eq!(choice.observer, responder);
    assert_eq!(choice.kind, engine_api::ChoiceKind::SelectCards);
}

#[test]
fn choudu_discards_cost_and_makes_the_chosen_player_use_diaobing() {
    let (mut state, players) = setup(3);
    let owner = players[0];
    let target = players[1];
    state.turn.current = owner;
    let cost = inject(&mut state, owner, ZoneKind::Hand, CardKind::Sha, 20);
    let drawn = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Shan,
        21,
    );
    let choudu = skill(&state, owner, CHOUDU);
    let (state, result) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill: choudu,
            targets: Vec::new(),
            card_targets: vec![cost],
        },
        40,
    );
    assert_eq!(result.choice.unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![target.raw()], false, 41);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(result.choice.unwrap().observer, target);
    assert!(!state.hand_cards(owner).contains(&cost));
    assert!(state.hand_cards(target).contains(&drawn));
}
