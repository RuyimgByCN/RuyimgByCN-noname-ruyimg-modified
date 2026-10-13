//! O2 全量合并第二批（2026-10-14）：家族 multi 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_multi_target_sha {
//! Golden coverage for legacy skills that increase Sha target count.

use engine_api::{ChoiceId,
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
const SP2: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");
const DCBENSHI: u32 = 3_177_558_288;
const DCWANGLIE2: u32 = 2_388_106_243;
const SHENJI: u32 = 3_515_984_232;
const TIANYI2: u32 = 4_028_521_539;
const WUSHUANG: u32 = 18;

fn start(seed: u64) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let mut engine = Engine::new();
    for bytes in [STANDARD, SHENHUA, SP2] {
        let package = content_schema::ContentPackage::from_json_bytes(bytes).unwrap();
        engine.load_package(&package).unwrap();
    }
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let actor = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let actor_index = seats.iter().position(|player| *player == actor).unwrap();
    let ordered = (1..seats.len())
        .map(|offset| seats[(actor_index + offset) % seats.len()])
        .collect();
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    for player in seats {
        let skills = tx.state().players.get(player).unwrap().skills.clone();
        for skill in skills {
            run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
        }
        clear_hand(&mut tx, player);
        let max_hp = tx.state().players.get(player).unwrap().max_hp as i32;
        tx.state_mut().players.get_mut(player).unwrap().hp = max_hp;
    }
    (tx, actor, ordered)
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return;
    }
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
}

fn add_skill(tx: &mut Transaction, player: PlayerId, skill_def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> engine_api::CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(9_800 + kind.tag() as u32 * 20 + rank as u32),
        kind,
        suit: Suit::Heart,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    };
    let card = tx.state_mut().cards.alloc(card);
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn play(actor: PlayerId, card: engine_api::CardId, targets: Vec<PlayerId>) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(1),
        kind: CommandKind::PlayCard {
            card,
            targets,
            card_targets: vec![],
        },
    }
}

#[test]
fn shenji_and_tianyi2_restore_legacy_sha_modifiers() {
    let (mut tx, actor, others) = start(20_260_714);
    let sha = inject(&mut tx, actor, CardKind::Sha, 7);
    let adjacent = vec![others[0], others[2]];
    let (_, base_multi) = apply_command(tx.state().clone(), play(actor, sha, adjacent.clone()));
    assert!(matches!(base_multi.result, CommandResult::Rejected { .. }));

    add_skill(&mut tx, actor, SHENJI);
    assert_eq!(
        engine_core::query::sha_target_limit_for(tx.state(), actor),
        3
    );
    assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), actor), 2);
    let (_, with_shenji) = apply_command(tx.state().clone(), play(actor, sha, adjacent));
    assert!(matches!(with_shenji.result, CommandResult::Accepted { .. }));

    let (mut tx, actor, others) = start(20_260_715);
    let sha = inject(&mut tx, actor, CardKind::Sha, 8);
    let far = others[1];
    let (_, base_far) = apply_command(tx.state().clone(), play(actor, sha, vec![far]));
    assert!(matches!(base_far.result, CommandResult::Rejected { .. }));

    add_skill(&mut tx, actor, TIANYI2);
    assert_eq!(
        engine_core::query::sha_target_limit_for(tx.state(), actor),
        2
    );
    assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), actor), 2);
    assert!(engine_core::query::sha_distance_unlimited_for(
        tx.state(),
        actor
    ));
    let (_, with_tianyi) = apply_command(tx.state().clone(), play(actor, sha, vec![far]));
    assert!(matches!(with_tianyi.result, CommandResult::Accepted { .. }));
}

#[test]
fn multi_target_sha_resolves_each_targets_required_shan_in_order() {
    let (mut tx, actor, others) = start(20_260_716);
    let targets = vec![others[0], others[2]];
    add_skill(&mut tx, actor, SHENJI);
    add_skill(&mut tx, actor, WUSHUANG);
    let sha = inject(&mut tx, actor, CardKind::Sha, 9);
    let shan1 = inject(&mut tx, targets[0], CardKind::Shan, 2);
    let shan2 = inject(&mut tx, targets[0], CardKind::Shan, 3);
    let first_hp = tx.state().players.get(targets[0]).unwrap().hp;
    let second_hp = tx.state().players.get(targets[1]).unwrap().hp;

    let (state, result) = apply_command(tx.state().clone(), play(actor, sha, targets.clone()));
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let first_choice = result.choice.unwrap();
    assert_eq!(first_choice.observer, targets[0]);

    let (state, result) = apply_command(
        state,
        Command {
            actor: targets[0],
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: first_choice.choice_id,
                selections: vec![shan1.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let second_choice = result.choice.unwrap();
    assert_eq!(second_choice.observer, targets[0]);

    let (state, result) = apply_command(
        state,
        Command {
            actor: targets[0],
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ResolveChoice {
                choice_id: second_choice.choice_id,
                selections: vec![shan2.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(targets[0]).unwrap().hp, first_hp);
    assert_eq!(state.players.get(targets[1]).unwrap().hp, second_hp - 1);
    assert_eq!(state.resolution.continuation_kind, ContinuationKind::None);
    assert!(!state.has_pending_choice());
}

#[test]
fn dcbenshi_requires_every_in_range_sha_target() {
    let (mut tx, actor, _) = start(20_260_717);
    add_skill(&mut tx, actor, DCBENSHI);
    let sha = inject(&mut tx, actor, CardKind::Sha, 10);
    let targets: Vec<PlayerId> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != actor)
        .collect();
    assert_eq!(targets.len(), 3);
    assert!(engine_core::query::sha_targets_all_for(tx.state(), actor));

    let (_, partial) = apply_command(tx.state().clone(), play(actor, sha, targets[..2].to_vec()));
    assert!(matches!(partial.result, CommandResult::Rejected { .. }));

    let hp: Vec<i32> = targets
        .iter()
        .map(|target| tx.state().players.get(*target).unwrap().hp)
        .collect();
    let (state, result) = apply_command(tx.state().clone(), play(actor, sha, targets.clone()));
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    for (target, before) in targets.iter().zip(hp) {
        assert_eq!(state.players.get(*target).unwrap().hp, before - 1);
    }
}

#[test]
fn self_only_target_policy_covers_explicit_and_implicit_targets() {
    let (mut tx, actor, others) = start(20_260_718);
    add_skill(&mut tx, actor, DCWANGLIE2);
    assert!(engine_core::query::card_targets_self_only_for(
        tx.state(),
        actor
    ));

    let guohe = inject(&mut tx, actor, CardKind::TrickGuohe, 4);
    inject(&mut tx, others[0], CardKind::Shan, 5);
    let (_, explicit_other) =
        apply_command(tx.state().clone(), play(actor, guohe, vec![others[0]]));
    assert!(matches!(
        explicit_other.result,
        CommandResult::Rejected { .. }
    ));

    let wugu = inject(&mut tx, actor, CardKind::TrickWugu, 6);
    let before: Vec<usize> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .map(|player| tx.state().hand_cards(player).len())
        .collect();
    let (mut state, mut result) = apply_command(tx.state().clone(), play(actor, wugu, vec![]));
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    // 独立锦囊队列（0615 起）逐目标排队取牌 choice；self-only wugu 的取牌也排队，
    // 显式解析后用户获得揭示牌。
    let mut key = 9_000;
    while let Some(choice) = result.choice.clone() {
        let (s, p) = apply_command(
            state.clone(),
            Command {
                actor: choice.observer,
                expected_revision: ExpectedRevision(state.revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: ChoiceId::new(choice.choice_id.raw()),
                    selections: vec![choice.options[0].token],
                    cancelled: false,
                },
            },
        );
        state = s;
        result = p;
        key += 1;
    }
    let after: Vec<usize> = state
        .players
        .alive_by_seat()
        .into_iter()
        .map(|player| state.hand_cards(player).len())
        .collect();
    assert_eq!(after, before, "only the user draws after consuming 五谷");

    let nanman = inject(&mut tx, actor, CardKind::TrickNanman, 7);
    let (_, implicit_other) = apply_command(tx.state().clone(), play(actor, nanman, vec![]));
    assert!(matches!(
        implicit_other.result,
        CommandResult::Rejected { .. }
    ));
}
}
