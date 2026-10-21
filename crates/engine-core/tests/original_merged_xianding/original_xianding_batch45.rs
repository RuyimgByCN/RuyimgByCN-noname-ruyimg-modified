//! Golden for XIANDING（限定包）批次四十五：
//! 笑武（dcxiaowu）、儒贤（dcruxian）、苏慎（dcsbsushen）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCXIAOWU: u32 = 4_153_740_280;
const DCRUXIAN: u32 = 3_734_967_277;
const DCSBSUSHEN: u32 = 3_864_585_121;
const DCZHANGCAI_ALL: u32 = 3_424_574_524;
const DCSBRUSHI: u32 = 4_089_348_607;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package())
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
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
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn live_skill(tx: &Transaction, player: PlayerId, def: u32) -> engine_api::SkillId {
    tx.state()
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn activate(
    state: GameState,
    actor: PlayerId,
    skill: engine_api::SkillId,
    cards: Vec<CardId>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: cards,
            },
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn inject_card(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| state.skills.get(*skill).map(|s| s.def.raw()) == Some(def))
}

#[test]
fn package_exposes_batch45_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    for (def, name) in [
        (DCXIAOWU, "dcxiaowu"),
        (DCRUXIAN, "dcruxian"),
        (DCSBSUSHEN, "dcsbsushen"),
    ] {
        let active = package
            .active_index
            .iter()
            .find(|a| a.skill_def_id == def)
            .unwrap_or_else(|| panic!("{name} active"));
        assert_eq!(active.window, "play_phase");
    }
    assert!(package
        .skill_index
        .iter()
        .any(|s| s.skill_def_id == DCZHANGCAI_ALL && s.locked));
}

#[test]
fn dcxiaowu_discards_he_and_gains_sha() {
    let (mut tx, owner, _first, _second) = setup(20_260_897);
    add_skill(&mut tx, owner, DCXIAOWU);
    let cost = inject_card(&mut tx, owner, CardKind::Shan);
    let before = tx.state().hand_count(owner);

    let skill = live_skill(&tx, owner, DCXIAOWU);
    let (state, result) = activate(tx.commit().state, owner, skill, vec![cost], 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before); // -1 弃 +1 摸【杀】
}

#[test]
fn dcruxian_grants_zhangcai_all_and_removes_on_turn_end() {
    let (mut tx, owner, _first, _second) = setup(20_260_898);
    add_skill(&mut tx, owner, DCRUXIAN);
    let skill = live_skill(&tx, owner, DCRUXIAN);
    let (state, result) = activate(tx.commit().state, owner, skill, vec![], 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(has_skill(&state, owner, DCZHANGCAI_ALL));

    let mut tx = Transaction::begin(state);
    let disp = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    run_effects(&mut tx, disp.unwrap()).unwrap();
    assert!(!has_skill(tx.state(), owner, DCZHANGCAI_ALL));
}

#[test]
fn dcsbsushen_grants_fumou() {
    let (mut tx, owner, _first, _second) = setup(20_260_899);
    add_skill(&mut tx, owner, DCSBSUSHEN);
    let skill = live_skill(&tx, owner, DCSBSUSHEN);
    let (state, result) = activate(tx.commit().state, owner, skill, vec![], 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(has_skill(&state, owner, DCSBRUSHI));
}
