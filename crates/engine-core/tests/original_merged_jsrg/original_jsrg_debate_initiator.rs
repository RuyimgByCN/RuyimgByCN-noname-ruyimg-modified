//! Golden coverage for the first initiator-side debate consumer (ADR 0644 P2):
//! jsrgshelun consumes start_debate_all_others (ABI 655) + result-branch filters.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SHELUN: u32 = 3_401_330_142;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/jsrg/jsrg.npack.json");
const SHELUN_SRC: &str = include_str!("../../../../content/original/ported/jsrg/jsrgshelun.skill");

fn inject_hand(tx: &mut Transaction, player: PlayerId, suit: Suit, n: usize) {
    for _ in 0..n {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(333_101),
            kind: CardKind::Shan,
            suit,
            rank: 6,
            weapon_range: 0,
            zone: hand,
            owner: player,
            face_up: true,
        });
        tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    }
}

fn bootstrap() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("shelun", SHELUN_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 31, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let mut others = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|p| *p != owner);
    let a = others.next().unwrap();
    let b = others.next().unwrap();
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    for player in tx.state().players.alive_by_seat() {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SHELUN),
        }],
    )
    .unwrap();
    (tx, owner, a, b)
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}

#[test]
fn shelun_npack_consumes_655() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == SHELUN));
    for filter in [
        "debate_owner_is_initiator_result_red",
        "debate_owner_is_initiator_result_black",
    ] {
        assert!(package.trigger_index.iter().any(|t| {
            t.skill_def_id == SHELUN && t.event == "debate_resolved" && t.filter == filter
        }));
    }
}

#[test]
fn shelun_black_branch_damages_on_black_result() {
    let (mut tx, owner, a, b) = bootstrap();
    // 参与者 a/b 全黑 → 黑结果；owner 未亮牌（无手牌）为发起者。
    inject_hand(&mut tx, a, Suit::Spade, 1);
    inject_hand(&mut tx, b, Suit::Club, 1);
    let hp_b = tx.state().players.get(b).unwrap().hp;
    // 主动技驱动：ActivateSkill 打开 start_debate_all_others（同步发事件），
    // 随后黑分支触发链开确认 → 选 b 伤害。
    let skill_id = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == SHELUN)
        .unwrap()
        .id;
    let (state, pipe) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: skill_id,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    let mut state = state;
    let mut key = 10;
    let mut step = 0;
    // 链固定两步：optional confirm（[1]）→ choose_other_damage 选目标（b）。
    while state.resolution.pending_choice_id != 0 && step < 4 {
        let actor = engine_api::PlayerId::new(state.resolution.pending_observer);
        let sel = if step == 0 {
            vec![1]
        } else {
            vec![u32::from(b.raw())]
        };
        state = resolve(state, actor, sel, key);
        key += 1;
        step += 1;
    }
    assert_eq!(
        state.players.get(b).unwrap().hp,
        hp_b - 2,
        "black branch: damage 2"
    );
}
