//! Golden coverage for 界吕蒙「勤学」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const RE_LVMENG: u32 = 2_276_140_348;
const KEJI: u32 = 31;
const QINXUE: u32 = 2_693_472_089;
const REBOTU: u32 = 2_231_312_450;
const GONGXIN: u32 = 3_100_453_257;
const DECADEZHENJUN: u32 = 2_634_318_954;
const YUJIN_YUJIN: u32 = 3_873_317_599;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/qinxue.skill");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const EXTRA: &[u8] = include_bytes!("../../../content/original/ported/extra/extra.npack.json");
const MODULE_CODE: [u8; 57] = [
    83, 81, 4, 16, 1, 0, 0, 0, 48, 83, 81, 5, 52, 65, 40, 0, 87, 80, 15, 83, 16, 255, 255, 255,
    255, 80, 5, 83, 16, 2, 0, 0, 0, 16, 1, 0, 0, 0, 16, 0, 0, 0, 0, 80, 39, 83, 16, 137, 41, 205,
    184, 80, 11, 64, 0, 0, 1,
];
const PREPARE_LOGIC_HASH: [u8; 32] = [
    164, 225, 152, 211, 188, 230, 193, 167, 115, 85, 35, 65, 9, 110, 106, 149, 51, 171, 123, 246,
    101, 34, 118, 94, 56, 61, 145, 191, 26, 211, 225, 31,
];
const END_LOGIC_HASH: [u8; 32] = [
    46, 91, 242, 236, 249, 215, 166, 53, 71, 1, 71, 194, 208, 230, 45, 183, 115, 182, 199, 12, 228,
    241, 136, 237, 123, 7, 85, 7, 144, 209, 148, 138,
];

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
}

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32, suit: Suit) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn has_skill(state: &GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == skill_def)
}

fn setup(seed: u64, hp: i32, max_hp: u32, hand_count: u32) -> Setup {
    let qinxue = content_compiler::compile_source_named("qinxue-test", SOURCE).unwrap();
    let extra = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&qinxue).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&extra).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(QINXUE),
    });
    run_effects(&mut tx, effects).unwrap();
    {
        let participant = tx.state_mut().players.get_mut(owner).unwrap();
        participant.hp = hp;
        participant.max_hp = max_hp;
    }
    for index in 0..hand_count {
        inject_hand(tx.state_mut(), owner, 99_900 + index, Suit::Spade);
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target,
    }
}

fn fire_phase(state: GameState, owner: PlayerId, phase: TurnPhaseTag) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_re_lvmeng_and_keeps_refresh_on_abi_298() {
    let compiled = content_compiler::compile_source_named("qinxue", SOURCE).unwrap();
    assert_eq!(compiled.trigger_index.len(), 2);
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == QINXUE && skill.locked));
    assert!(compiled
        .trigger_index
        .iter()
        .all(|trigger| { !trigger.optional && trigger.filter == "owner_is_target" }));
    assert!(compiled
        .trigger_index
        .iter()
        .any(|trigger| trigger.event == "phase_prepare"));
    assert!(compiled
        .trigger_index
        .iter()
        .any(|trigger| trigger.event == "phase_end"));
    let prepare = compiled
        .modules
        .iter()
        .find(|module| module.name == "qinxue_phase_prepare")
        .unwrap();
    assert_eq!(prepare.code, MODULE_CODE);
    assert_eq!(prepare.logic_hash, PREPARE_LOGIC_HASH);
    let end = compiled
        .modules
        .iter()
        .find(|module| module.name == "qinxue_phase_end")
        .unwrap();
    assert_eq!(end.code, MODULE_CODE);
    assert_eq!(end.logic_hash, END_LOGIC_HASH);

    let package = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 298);
    assert!(package
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "extra"));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_LVMENG)
        .unwrap();
    assert_eq!(character.name_key, "re_lvmeng");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [KEJI, QINXUE, REBOTU]);
    assert!(compiled.modules.iter().all(|expected| {
        package
            .modules
            .iter()
            .any(|actual| actual.name == expected.name && actual.logic_hash == expected.logic_hash)
    }));

    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == DECADEZHENJUN));
    assert!(package
        .character_index
        .iter()
        .any(|character| character.id == YUJIN_YUJIN && character.skills == [DECADEZHENJUN]));
}

#[test]
fn exact_threshold_fires_at_hp_plus_two_on_both_phase_boundaries_and_only_once() {
    let mut setup = setup(20_260_730_01, 3, 4, 4);
    setup.state = fire_phase(setup.state, setup.owner, TurnPhaseTag::Prepare);
    assert!(has_skill(&setup.state, setup.owner, QINXUE));
    assert_eq!(setup.state.players.get(setup.owner).unwrap().max_hp, 4);

    inject_hand(&mut setup.state, setup.owner, 99_910, Suit::Spade);
    setup.state = fire_phase(setup.state, setup.owner, TurnPhaseTag::End);
    assert!(!has_skill(&setup.state, setup.owner, QINXUE));
    assert!(has_skill(&setup.state, setup.owner, GONGXIN));
    assert_eq!(setup.state.players.get(setup.owner).unwrap().max_hp, 3);
    assert_eq!(setup.state.hand_count(setup.owner), 7);

    let before_hp = setup.state.players.get(setup.owner).unwrap().hp;
    let before_max_hp = setup.state.players.get(setup.owner).unwrap().max_hp;
    let before_hand = setup.state.hand_count(setup.owner);
    let after = fire_phase(setup.state, setup.owner, TurnPhaseTag::Prepare);
    assert_eq!(after.players.get(setup.owner).unwrap().hp, before_hp);
    assert_eq!(
        after.players.get(setup.owner).unwrap().max_hp,
        before_max_hp
    );
    assert_eq!(after.hand_count(setup.owner), before_hand);
    assert!(!has_skill(&after, setup.owner, QINXUE));
}

#[test]
fn full_health_loses_max_hp_then_automatically_draws_two() {
    let setup = setup(20_260_730_02, 4, 4, 6);
    let state = fire_phase(setup.state, setup.owner, TurnPhaseTag::Prepare);
    let owner = state.players.get(setup.owner).unwrap();
    assert_eq!((owner.hp, owner.max_hp), (3, 3));
    assert_eq!(state.hand_count(setup.owner), 8);
    assert!(!state.has_pending_choice());
    assert!(!has_skill(&state, setup.owner, QINXUE));
    assert!(has_skill(&state, setup.owner, GONGXIN));
}

#[test]
fn wounded_owner_gets_a_forced_choice_and_gongxin_is_live_after_resolution() {
    let setup = setup(20_260_730_03, 2, 4, 4);
    let mut tx = Transaction::begin(setup.state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    let state = committed.state;
    assert!(!has_skill(&state, setup.owner, QINXUE));
    assert!(!has_skill(&state, setup.owner, GONGXIN));
    assert_eq!(state.players.get(setup.owner).unwrap().max_hp, 3);
    let choice = committed
        .output
        .choice
        .expect("draw-or-recover choice must remain pending");
    assert_eq!(choice.kind, ChoiceKind::Custom);
    assert_eq!(choice.min_count, 1);
    assert_eq!(choice.max_count, 1);
    assert!(!choice.allow_cancel);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );

    let (unchanged, cancelled) = resolve(state.clone(), setup.owner, Vec::new(), true, 1);
    assert!(matches!(cancelled.result, CommandResult::Rejected { .. }));
    assert_eq!(unchanged, state);

    let (mut state, recovered) = resolve(state, setup.owner, vec![2], false, 2);
    assert!(matches!(recovered.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 3);
    assert!(has_skill(&state, setup.owner, GONGXIN));

    let heart = inject_hand(&mut state, setup.target, 99_920, Suit::Heart);
    state.turn.current = setup.owner;
    state.turn.phase = Phase::Play;
    let gongxin = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == setup.owner && skill.def.raw() == GONGXIN)
        .unwrap()
        .id;
    let revision = state.revision;
    let (state, activated) = apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ActivateSkill {
                skill: gongxin,
                targets: vec![setup.target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let choice = activated.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [heart.raw()]
    );
    assert!(activated.events.iter().any(|event| {
        matches!(event, DomainEvent::CardsRevealed { cards, to }
            if cards == &[heart] && to == &[setup.owner])
    }));
    assert!(state.has_pending_choice());
}
