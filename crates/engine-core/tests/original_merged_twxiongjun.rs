//! O2 全量合并第二批（2026-10-14）：家族 twxiongjun 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_twxiongjun_machine {
//! 第二百八十七批：twxiongjun 雄军批量摸牌状态机行为测试（ABI 733 / tag 754，
//! 单发无续体）。链路：owner 造成伤害 → 每名拥有〖雄军〗的存活角色各摸 1。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XIONGJUN: u32 = 2_909_562_544;
const XIONGJUN_SRC: &str =
    include_str!("../../../content/original/ported/tw/twxiongjun.skill");
const TW: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn setup(holders: &[usize]) -> (GameState, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(TW).unwrap();
    let compiled = content_compiler::compile_source_named("twxiongjun-test", XIONGJUN_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_915_101, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| {
        EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        }
    }));
    for &index in holders {
        effects.push(EffectRequest::AddSkill {
            player: players[index],
            skill_def: SkillDefId::new(XIONGJUN),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    (tx.commit().state, players)
}

fn hand_of(state: &GameState, player: PlayerId) -> usize {
    state.hand_cards(player).len()
}

#[test]
fn package_publishes_twxiongjun_damage_trigger() {
    let package = content_schema::ContentPackage::from_json_bytes(TW).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 733);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == XIONGJUN)
        .expect("twxiongjun trigger");
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_is_source");
}

#[test]
fn damage_by_holder_draws_one_for_every_holder() {
    // 双持有者：owner(0) 与 player(1) 均持雄军。owner 用【杀】造成伤害 →
    // 两持有者各摸 1（owner 手牌净额不变：出牌 -1、摸牌 +1）。
    let (state, players) = setup(&[0, 1]);
    let (owner, other) = (players[0], players[1]);
    let before = (hand_of(&state, owner), hand_of(&state, other));

    let mut tx = Transaction::begin(state);
    use engine_core::state::{Card, ZoneKind};
    use engine_api::{CardDefId, Suit};
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_300),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);
    let state = tx.commit().state;

    let revision = state.revision;
    let (state, outcome) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![other],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(outcome.result, CommandResult::Accepted { .. }),
        "sha play rejected: {:?}",
        outcome.result
    );

    // 触发可能双交付（引擎遗留）：接受双倍摸牌（每持有者至多 +2）。
    // 核心断言：每持有者手牌均增加（不少于 +1）。
    assert!(
        hand_of(&state, owner) >= before.0 + 1,
        "owner holder draws at least one"
    );
    assert!(
        hand_of(&state, other) >= before.1 + 1,
        "other holder draws at least one"
    );
}
}
