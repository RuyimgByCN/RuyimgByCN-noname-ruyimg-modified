//! 第二百七十七批：meiyong 姝勇状态机行为测试（ABI 726 / tag 750 /
//! Cont 324）。链路：使用/打出【杀】→ 选一名 hej 区有牌的其他角色（可取消）
//! → owner 选其区域一张牌（强制）→ 移入 owner 手牌 + 目标摸 1。

use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const MEIYONG: u32 = 2_290_758_444;
const MEIYONG_SRC: &str =
    include_str!("../../../content/original/ported/mobile/meiyong.skill");
const MOBILE: &[u8] =
    include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    other: PlayerId,
    sha: CardId,
    judge: CardId,
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_700 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    let compiled = content_compiler::compile_source_named("meiyong-test", MEIYONG_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target, other) = (players[0], players[1], players[2]);
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
        skill_def: SkillDefId::new(MEIYONG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let sha = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, Suit::Spade, 7);
    // target：一张手牌 + 一张判定区牌（乐不思蜀），覆盖 hej 三区之二。
    inject(&mut tx, target, ZoneKind::Hand, CardKind::Other, Suit::Heart, 5);
    let judge = inject(
        &mut tx,
        target,
        ZoneKind::Judge,
        CardKind::TrickLe,
        Suit::Club,
        6,
    );
    // other：一张手牌。
    inject(&mut tx, other, ZoneKind::Hand, CardKind::Other, Suit::Diamond, 4);
    Setup {
        state: tx.commit().state,
        owner,
        target,
        other,
        sha,
        judge,
    }
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

fn hand_of(state: &GameState, player: PlayerId) -> Vec<CardId> {
    state.hand_cards(player)
}

#[test]
fn package_publishes_meiyong_dual_triggers() {
    let package = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 734);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == MEIYONG)
        .collect();
    assert_eq!(triggers.len(), 2, "card_used + card_responded arms");
    let events: Vec<_> = triggers.iter().map(|t| t.event.as_str()).collect();
    assert!(events.contains(&"card_used"));
    assert!(events.contains(&"card_responded"));
}

#[test]
fn sha_use_takes_card_from_target_area_and_target_draws() {
    let setup = setup(20_260_914_001);
    let target_hand_before = hand_of(&setup.state, setup.target).len();
    let owner_hand_before = hand_of(&setup.state, setup.owner).len();

    // owner 打出【杀】（作为响应）→ 触发姝勇：选目标。
    let (state, target_choice) = submit(
        setup.state.clone(),
        setup.owner,
        CommandKind::PlayCard {
            card: setup.sha,
            targets: vec![setup.other],
            card_targets: vec![],
        },
        1,
    );
    let target_choice = target_choice.choice.unwrap();
    assert_eq!(target_choice.observer, setup.owner);
    assert_eq!(target_choice.kind, ChoiceKind::SelectTargets);
    // 候选含 target 与 other（各有牌）。
    assert_eq!(target_choice.options.len(), 2);

    // 选 target → owner 从其 hej 选一张（手牌 1 + 判定 1 = 2 个候选）。
    let (state, card_choice) = submit(
        state,
        setup.owner,
        CommandKind::ResolveChoice {
            choice_id: target_choice.choice_id,
            selections: vec![setup.target.raw()],
            cancelled: false,
        },
        2,
    );
    let card_choice = card_choice.choice.unwrap();
    assert_eq!(card_choice.kind, ChoiceKind::SelectCards);
    assert_eq!(card_choice.options.len(), 2, "hand + judge card");
    assert!(
        card_choice
            .options
            .iter()
            .any(|option| option.token == setup.judge.raw()),
        "judge-zone card is gainable"
    );

    // 取判定区牌 → 移入 owner 手牌，target 摸 1。
    let (state, done) = submit(
        state,
        setup.owner,
        CommandKind::ResolveChoice {
            choice_id: card_choice.choice_id,
            selections: vec![setup.judge.raw()],
            cancelled: false,
        },
        3,
    );
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert!(
        hand_of(&state, setup.owner).contains(&setup.judge),
        "judge card moved to owner hand"
    );
    // target 手牌不动，摸 1：1 → 2。
    assert_eq!(hand_of(&state, setup.target).len(), target_hand_before + 1);
    // owner：打出【杀】-1、获得判定牌 +1，总量不变。
    assert_eq!(hand_of(&state, setup.owner).len(), owner_hand_before);
}

#[test]
fn cancel_leaves_everything_unchanged() {
    let setup = setup(20_260_914_002);
    let target_hand_before = hand_of(&setup.state, setup.target).len();
    let owner_hand_before = hand_of(&setup.state, setup.owner).len();

    let (state, target_choice) = submit(
        setup.state.clone(),
        setup.owner,
        CommandKind::PlayCard {
            card: setup.sha,
            targets: vec![setup.other],
            card_targets: vec![],
        },
        1,
    );
    let target_choice = target_choice.choice.unwrap();
    let (mut state, mut outcome) = submit(
        state,
        setup.owner,
        CommandKind::ResolveChoice {
            choice_id: target_choice.choice_id,
            selections: vec![],
            cancelled: true,
        },
        2,
    );
    // 取消后残留的响应/结算询问逐个取消（与批 271 同款处理）。
    for key in 3..8 {
        match outcome.choice.clone() {
            Some(choice) => {
                let (next, next_outcome) = submit(
                    state,
                    choice.observer,
                    CommandKind::ResolveChoice {
                        choice_id: choice.choice_id,
                        selections: vec![],
                        cancelled: true,
                    },
                    key,
                );
                state = next;
                outcome = next_outcome;
            }
            None => break,
        }
    }
    assert!(!state.has_pending_choice());
    assert_eq!(hand_of(&state, setup.target).len(), target_hand_before);
    assert_eq!(hand_of(&state, setup.owner).len(), owner_hand_before - 1);
}
