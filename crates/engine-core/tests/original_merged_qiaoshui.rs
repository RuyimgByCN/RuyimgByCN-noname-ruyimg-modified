//! O2 全量合并第二批（2026-10-14）：家族 qiaoshui 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_qiaoshui_family {
//! Golden coverage for legacy 简雍/OL界简雍「巧说」rewrites.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const QIAOSHUI: u32 = 3_955_213_386;
const QIAOSHUI2: u32 = 2_280_240_537;
const QIAOSHUI3: u32 = 2_260_792_673;
const OLQIAOSHUI: u32 = 2_828_774_708;
const OLQIAOSHUI_USED: u32 = 2_540_060_947;
const JYZONGSHI: u32 = 2_485_328_360;
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const ONLYOL: &[u8] = include_bytes!("../../../content/original/ported/onlyOL/onlyOL.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
}

fn setup(seed: u64, skill_def: u32, onlyol: bool) -> Setup {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&yijiang).unwrap();
    if onlyol {
        let onlyol = content_schema::ContentPackage::from_json_bytes(ONLYOL).unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&onlyol).unwrap();
    }
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target: players[1],
    }
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, rank: u8) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> Option<engine_api::SkillId> {
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
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    )
}

fn open_play_phase(state: GameState) -> GameState {
    let current = state.turn.current;
    let mut tx = Transaction::begin(state);
    match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase: TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Ok(effects) => assert_eq!(
            run_effects(&mut tx, effects).unwrap_err(),
            EffectError::ChoiceRequested
        ),
        Err(error) => assert_eq!(error, EffectError::ChoiceRequested),
    }
    tx.commit().state
}

fn end_turn(state: GameState, current: PlayerId) -> GameState {
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
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

#[test]
fn packages_publish_both_complete_jianyong_variants() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    let onlyol = content_schema::ContentPackage::from_json_bytes(ONLYOL).unwrap();
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    assert_eq!(onlyol.manifest.rule_vm_abi_version, 758);
    assert_eq!(yijiang.skill_index.len(), 281);
    assert_eq!(onlyol.skill_index.len(), 193);
    assert_eq!(
        yijiang
            .character_index
            .iter()
            .find(|character| character.id == 2_747_947_994)
            .unwrap()
            .skills,
        [QIAOSHUI, JYZONGSHI]
    );
    assert_eq!(
        onlyol
            .character_index
            .iter()
            .find(|character| character.id == 3_268_954_958)
            .unwrap()
            .skills,
        [OLQIAOSHUI, JYZONGSHI]
    );
}

#[test]
fn qiaoshui_phase_start_compare_grants_the_strict_result_helper() {
    let mut setup = setup(2_026_072_211, QIAOSHUI, false);
    let owner_card = inject(&mut setup.state, setup.owner, 99_211, 13);
    let target_card = inject(&mut setup.state, setup.target, 99_212, 1);
    let state = open_play_phase(setup.state);
    let (state, confirmed) = resolve(state, setup.owner, vec![1], 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseHandCompareTarget
    );
    let (state, target_chosen) = resolve(state, setup.owner, vec![setup.target.raw()], 2);
    assert!(matches!(
        target_chosen.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseHandComparePlayerCard
    );
    let (state, owner_chosen) = resolve(state, setup.owner, vec![owner_card.raw()], 3);
    assert!(matches!(
        owner_chosen.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseCompareTargetHandCard
    );
    let (state, compared) = resolve(state, setup.target, vec![target_card.raw()], 4);
    assert!(matches!(compared.result, CommandResult::Accepted { .. }));
    assert!(skill_id(&state, setup.owner, QIAOSHUI3).is_some());
    assert!(skill_id(&state, setup.owner, QIAOSHUI2).is_none());
}

#[test]
fn qiaoshui_nonwin_blocks_tricks_only_until_turn_end() {
    let mut setup = setup(2_026_072_213, QIAOSHUI, false);
    let owner_card = inject(&mut setup.state, setup.owner, 99_216, 1);
    let target_card = inject(&mut setup.state, setup.target, 99_217, 13);
    let state = open_play_phase(setup.state);
    let (state, _) = resolve(state, setup.owner, vec![1], 20);
    let (state, _) = resolve(state, setup.owner, vec![setup.target.raw()], 21);
    let (state, _) = resolve(state, setup.owner, vec![owner_card.raw()], 22);
    let (state, compared) = resolve(state, setup.target, vec![target_card.raw()], 23);
    assert!(matches!(compared.result, CommandResult::Accepted { .. }));
    assert!(skill_id(&state, setup.owner, QIAOSHUI2).is_some());
    let state = end_turn(state, setup.owner);
    assert!(skill_id(&state, setup.owner, QIAOSHUI2).is_none());
}

#[test]
fn olqiaoshui_failure_bans_itself_and_tricks_only_until_turn_end() {
    let mut setup = setup(2_026_072_212, OLQIAOSHUI, true);
    let owner_card = inject(&mut setup.state, setup.owner, 99_213, 1);
    let target_card = inject(&mut setup.state, setup.target, 99_214, 13);
    let active = skill_id(&setup.state, setup.owner, OLQIAOSHUI).unwrap();
    let (state, activated) = apply_command(
        setup.state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(10),
            kind: CommandKind::ActivateSkill {
                skill: active,
                targets: vec![setup.target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let (state, _) = resolve(state, setup.owner, vec![owner_card.raw()], 11);
    let (state, compared) = resolve(state, setup.target, vec![target_card.raw()], 12);
    assert!(matches!(compared.result, CommandResult::Accepted { .. }));
    assert!(skill_id(&state, setup.owner, OLQIAOSHUI).is_none());
    assert!(skill_id(&state, setup.owner, OLQIAOSHUI_USED).is_some());

    let mut checked = state.clone();
    let trick = inject(&mut checked, setup.owner, 99_215, 7);
    checked.cards.get_mut(trick).unwrap().kind = CardKind::TrickWuzhong;
    assert!(engine_core::query::card_use_cancelled_for_card(
        &checked,
        setup.owner,
        trick
    ));

    let state = end_turn(state, setup.owner);
    assert!(skill_id(&state, setup.owner, OLQIAOSHUI).is_some());
    assert!(skill_id(&state, setup.owner, OLQIAOSHUI_USED).is_none());
}
}
