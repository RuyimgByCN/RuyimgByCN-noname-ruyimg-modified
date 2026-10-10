//! Golden for MOBILE 兀突骨「悍勇」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const MBHANYONG: u32 = 2_639_258_372;
const MBRANSHANG: u32 = 3_354_404_150;
const MBRANSHANG_MARK: u32 = 1_206_920_502;
const MB_WUTUGU: u32 = 4_165_377_298;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/mbhanyong.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn started(seed: u64, wounded: bool) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let targets = players[1..].to_vec();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in &players {
        effects.push(EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        });
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(MBHANYONG),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let draw_cards = tx.state().zones.get(draw).unwrap().cards.clone();
    for card in draw_cards {
        tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
    }
    let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
    owner_state.max_hp = 15;
    owner_state.hp = if wounded { 14 } else { 15 };
    (tx.commit().state, owner, targets)
}

fn play(state: GameState, owner: PlayerId, card: CardId, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    )
}

fn resolve(
    state: GameState,
    pipeline: PipelineResult,
    owner: PlayerId,
    selection: u32,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice = pipeline.choice.expect("expected choice");
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice.choice_id.raw()),
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

fn activate(
    state: GameState,
    pipeline: PipelineResult,
    owner: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, pipeline) = resolve(state, pipeline, owner, 1, key);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark
    );
    assert_eq!(
        pipeline
            .choice
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );
    (state, pipeline)
}

fn mark_count(state: &GameState, owner: PlayerId) -> i32 {
    state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find_map(|(mark, count)| (*mark == MBRANSHANG_MARK).then_some(*count))
        .unwrap_or(0)
}

#[test]
fn package_publishes_abi114_skill_and_mobile_wutugu_closure() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert_eq!(
        content_compiler::compile_source_named("mbhanyong", SOURCE)
            .unwrap()
            .manifest
            .rule_vm_abi_version,
        114
    );
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MBHANYONG)
        .unwrap();
    assert_eq!(trigger.event, "card_used");
    assert_eq!(trigger.filter, "owner_alive_wounded_card_targets_gt_one");
    assert!(trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == MB_WUTUGU)
        .unwrap();
    assert_eq!(character.max_hp, 15);
    assert_eq!(character.skills, [MBRANSHANG, MBHANYONG]);
}

#[test]
fn three_branches_draw_increase_damage_and_backwater_exactly() {
    for (branch, expected_hand, expected_damage, expected_mark) in
        [(1, 2, 1, 0), (2, 0, 2, 0), (3, 2, 2, 1)]
    {
        let (mut state, owner, targets) = started(20_260_724 + branch as u64, true);
        let wanjian = inject(&mut state, owner, CardKind::TrickWanjian, 99_100 + branch);
        let before: Vec<_> = targets
            .iter()
            .map(|target| state.players.get(*target).unwrap().hp)
            .collect();
        let (state, pipeline) = play(state, owner, wanjian, 10 + branch as u64);
        let (state, pipeline) = activate(state, pipeline, owner, 20 + branch as u64);
        let (state, pipeline) = resolve(state, pipeline, owner, branch, 30 + branch as u64);
        assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
        assert!(!state.has_pending_choice());
        assert_eq!(state.hand_count(owner), expected_hand);
        assert_eq!(mark_count(&state, owner), expected_mark);
        for (target, hp) in targets.iter().zip(before.iter()) {
            assert_eq!(state.players.get(*target).unwrap().hp, hp - expected_damage);
        }
    }
}

#[test]
fn decline_full_health_and_non_damage_multi_target_preserve_legacy_gates() {
    let (mut state, owner, targets) = started(20_260_728, true);
    let wanjian = inject(&mut state, owner, CardKind::TrickWanjian, 99_201);
    let before = state.players.get(targets[0]).unwrap().hp;
    let (state, pipeline) = play(state, owner, wanjian, 41);
    let (state, pipeline) = resolve(state, pipeline, owner, 0, 42);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(targets[0]).unwrap().hp, before - 1);
    assert_eq!(state.hand_count(owner), 0);
    assert_eq!(mark_count(&state, owner), 0);

    let (mut state, owner, targets) = started(20_260_729, false);
    let wanjian = inject(&mut state, owner, CardKind::TrickWanjian, 99_202);
    let before = state.players.get(targets[0]).unwrap().hp;
    let (state, pipeline) = play(state, owner, wanjian, 43);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(pipeline.choice.is_none());
    assert_eq!(state.players.get(targets[0]).unwrap().hp, before - 1);

    let (mut state, owner, _) = started(20_260_730, true);
    let wugu = inject(&mut state, owner, CardKind::TrickWugu, 99_203);
    let (mut state, mut pipeline) = play(state, owner, wugu, 44);
    let (next_state, next_pipeline) = activate(state, pipeline, owner, 45);
    state = next_state;
    pipeline = next_pipeline;
    let (next_state, next_pipeline) = resolve(state, pipeline, owner, 3, 46);
    state = next_state;
    pipeline = next_pipeline;
    // 独立锦囊队列（0615 起）逐目标排队取牌 choice；每个玩家（含 owner）各取一张。
    let mut key = 47;
    while let Some(choice) = pipeline.choice.clone() {
        let (s, p) = resolve(state, pipeline, choice.observer, choice.options[0].token, key);
        state = s;
        pipeline = p;
        key += 1;
    }
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_ne!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDrawOrIncreaseCurrentCardDamageOrBothMark
    );
    assert_eq!(state.hand_count(owner), 3);
    assert_eq!(mark_count(&state, owner), 1);
}
