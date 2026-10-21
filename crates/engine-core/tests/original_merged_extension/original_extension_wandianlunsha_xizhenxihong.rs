//! Golden for archived extension 玩点论杀 习珍习宏【负仇/噤言】（ABI 614）。

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceRequest, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const FUCHOU: u32 = 3_890_431_595;
const JINYAN: u32 = 3_563_375_486;
const SOURCE: &str =
    include_str!("../../../../content/original/ported/wandianlunsha/xizhenxihong.skill");

fn setup() -> (GameState, [PlayerId; 4]) {
    let package =
        content_compiler::compile_source_named("wandianlunsha-xizhenxihong", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 614);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_803, "identity-standard").unwrap();
    let players: [PlayerId; 4] = tx.state().players.alive_by_seat().try_into().unwrap();
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
            skill_def: SkillDefId::new(FUCHOU),
        },
        EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(JINYAN),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[1];
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, players)
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    serial: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(97_000 + serial),
        kind,
        suit,
        rank: 7,
        weapon_range: if kind == CardKind::Weapon { 3 } else { 0 },
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
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

fn fuchou_skill(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
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
                .is_some_and(|skill| skill.def.raw() == FUCHOU)
        })
        .unwrap()
}

fn jinyan_skill(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
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
                .is_some_and(|skill| skill.def.raw() == JINYAN)
        })
        .unwrap()
}

fn record_fuchou(
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    gift: CardId,
    incoming: CardId,
    key: u64,
) -> GameState {
    let (state, result) = submit(
        state,
        source,
        CommandKind::PlayCard {
            card: incoming,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        key,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(result.choice.unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![1], false, key + 1);
    assert_eq!(result.choice.unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![gift.raw()], false, key + 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn end_turn(state: GameState, current: PlayerId) -> (GameState, Option<ChoiceRequest>) {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current,
            phase: TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let effect_result = run_effects(&mut tx, effects);
    assert!(matches!(
        effect_result,
        Ok(_) | Err(EffectError::ChoiceRequested)
    ));
    let committed = tx.commit();
    (committed.state, committed.output.choice)
}

#[test]
fn package_exposes_exact_filters_modifiers_and_view_as_contract() {
    let package =
        content_compiler::compile_source_named("wandianlunsha-xizhenxihong", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 614);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == FUCHOU && trigger.filter == "owner_alive_wdb_fuchou_pending_living"
    }));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == JINYAN
            && modifier.target == "card_response_cancel"
            && modifier.selector.as_deref() == Some("black_trick")
    }));
    assert_eq!(
        package
            .view_as_index
            .iter()
            .filter(
                |view| view.skill_def_id == JINYAN && view.filter == "owner_hp_le_two_black_trick"
            )
            .count(),
        2
    );
}

#[test]
fn jinyan_blocks_the_original_black_trick_but_can_use_it_as_sha_at_two_hp() {
    let (mut state, players) = setup();
    let owner = players[0];
    let target = players[1];
    state.turn.current = owner;
    state.players.get_mut(owner).unwrap().hp = 2;
    let card = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        Suit::Spade,
        1,
    );
    let (state, rejected) = submit(
        state,
        owner,
        CommandKind::PlayCard {
            card,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        10,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    let hp = state.players.get(target).unwrap().hp;
    let skill = jinyan_skill(&state, owner);
    let (state, result) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![card],
        },
        11,
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn jinyan_responds_with_black_wuxie_as_sha_instead_of_wuxie() {
    let (mut state, players) = setup();
    let owner = players[0];
    let source = players[1];
    state.players.get_mut(owner).unwrap().hp = 2;
    let black_wuxie = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::TrickWuxie,
        Suit::Club,
        61,
    );
    let duel = inject(
        &mut state,
        source,
        ZoneKind::Hand,
        CardKind::TrickJuedou,
        Suit::Heart,
        62,
    );
    let source_hp = state.players.get(source).unwrap().hp;
    let (state, result) = submit(
        state,
        source,
        CommandKind::PlayCard {
            card: duel,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        12,
    );
    let choice = result.choice.unwrap();
    assert_eq!(choice.observer, owner);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == black_wuxie.raw()));
    let (state, result) = choose(state, owner, vec![black_wuxie.raw()], false, 13);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.players.get(source).unwrap().hp, source_hp - 1);
}

#[test]
fn fuchou_gives_the_card_cancels_sha_and_records_each_source_once() {
    let (mut state, players) = setup();
    let owner = players[0];
    let source = players[1];
    let gift = inject(
        &mut state,
        owner,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Heart,
        2,
    );
    let incoming = inject(
        &mut state,
        source,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Club,
        3,
    );
    let hp = state.players.get(owner).unwrap().hp;
    let mut state = record_fuchou(state, owner, source, gift, incoming, 20);
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    assert!(state.hand_cards(source).contains(&gift));
    state
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 0);
    state
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
    let second_gift = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        Suit::Diamond,
        63,
    );
    let second_sha = inject(
        &mut state,
        source,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        64,
    );
    let state = record_fuchou(state, owner, source, second_gift, second_sha, 23);
    assert!(state.hand_cards(source).contains(&second_gift));
    assert_eq!(
        state
            .skills
            .get(fuchou_skill(&state, owner))
            .unwrap()
            .state_blob
            .len(),
        8
    );
}

#[test]
fn fuchou_draws_then_loses_hp_when_no_sha_is_available() {
    let (mut state, players) = setup();
    let owner = players[0];
    let source = players[1];
    let gift = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Heart,
        4,
    );
    let incoming = inject(
        &mut state,
        source,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Club,
        5,
    );
    let state = record_fuchou(state, owner, source, gift, incoming, 30);
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let mut state = state;
    state.zones.get_mut(draw).unwrap().cards.clear();
    inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::TrickWuzhong,
        Suit::Heart,
        50,
    );
    let hp = state.players.get(owner).unwrap().hp;
    let hand = state.hand_count(owner);
    let (state, result) = end_turn(state, source);
    assert!(result.is_none());
    assert_eq!(state.hand_count(owner), hand + 1);
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
    assert_eq!(
        state
            .skills
            .get(fuchou_skill(&state, owner))
            .unwrap()
            .state_blob,
        b"WFU1"
    );
}

#[test]
fn fuchou_can_use_sha_at_distance_two_and_continues_after_it_resolves() {
    let (mut state, players) = setup();
    let owner = players[0];
    let source = players[2];
    state.turn.current = source;
    state
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 0);
    state
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
    inject(
        &mut state,
        source,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Heart,
        60,
    );
    let gift = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        Suit::Heart,
        6,
    );
    let retaliation = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Diamond,
        7,
    );
    let incoming = inject(
        &mut state,
        source,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Club,
        8,
    );
    let state = record_fuchou(state, owner, source, gift, incoming, 40);
    let hp = state.players.get(source).unwrap().hp;
    let (state, result) = end_turn(state, source);
    let choice = result.unwrap();
    assert_eq!(choice.observer, owner);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == retaliation.raw()));
    let (state, result) = choose(state, owner, vec![retaliation.raw()], false, 43);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.players.get(source).unwrap().hp, hp - 1);
    assert!(result.choice.is_none());
}
