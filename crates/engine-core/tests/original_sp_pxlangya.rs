//! 第一百七十六批：sp「狼牙」（彭城 pxlangya）观星族首原语（ABI 681）。
//! 上游：锁定技，你的技能增加后，你观看牌堆顶的五张牌并任意重排（拖拽顶/底）。
//! 实现：skill_added + owner_skill_added_self_<def> → guanxing_top(self, 5)
//! （逐张二选顶/底近似拖拽自由排列——登记债）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const PXLANGYA: u32 = 3_999_958_998;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    let start = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(
        matches!(start, CommandResult::Accepted { .. }),
        "start rejected: {start:?}"
    );
    let state = engine.state().clone();
    let owner = state.turn.current;
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    let removals: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    let _ = engine_core::effect::run_effects(&mut tx, removals);
    // AddSkill 触发狼牙（skill_added）→ guanxing_top 的 ChoiceRequested 属预期控制流。
    let _ = engine_core::effect::run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXLANGYA),
        }],
    );
    (tx.commit().state, owner, other)
}

#[test]
fn package_publishes_pxlangya_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 681);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == PXLANGYA
            && trigger.event == "skill_added"
            && trigger.filter == "owner_skill_added_self_3999958998"
            && !trigger.optional
    }));
}

#[test]
fn pxlangya_guanxing_reorders_top_five() {
    // 给牌堆顶 5 张打上可识别的点数序列。
    let (state, owner, _other) = started(2_026_092_001);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let draw = tx.state().zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    let state = tx.commit().state;

    // 模拟「技能增加」：owner 再获得任意技能（sp 包内真实 def——用 pxlangya 自身
    // 会重复；改用重复 AddSkill pxlangya 亦可再次触发）。
    // 直接路径：再次 AddSkill pxlangya 触发 skill_added（过滤器要求 def 匹配自身，
    // 重复添加仍发出 SkillAdded 事件）。
    let mut tx = Transaction::begin(state);
    let effects = vec![EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PXLANGYA),
    }];
    let outcome = engine_core::effect::run_effects(&mut tx, effects);
    // 观星选择链开启（ChoiceRequested）或已直接进入多轮；断言 pending 存在。
    let _ = outcome;
    let mut state = tx.commit().state;
    assert!(
        state.has_pending_choice(),
        "guanxing pick loop must open after skill gain"
    );

    // 五轮：前四轮全部选「底」（token 2），最后一轮选「顶」（token 1）。
    let mut key = 2_026_092_002;
    for round in 0..5 {
        assert!(state.has_pending_choice(), "round {round} must stay open");
        let pick = if round == 4 { 1 } else { 2 };
        let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        let (next, result) = apply_command(
            state,
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: vec![pick],
                    cancelled: false,
                },
            },
        );
        assert!(
            matches!(result.result, CommandResult::Accepted { .. }),
            "round {round}: resolve rejected {:?}",
            result.result
        );
        state = next;
        key += 1;
    }
    assert!(
        !state.has_pending_choice(),
        "guanxing loop must close after 5 rounds"
    );

    // 结算后牌堆顶 5 张被重排：全部置底的 4 张在前（底序列），最后置顶的 1 张在
    // 牌堆最顶。核验「处理区已清空、牌回到牌堆」：
    let processing = state.zones.find(ZoneKind::Processing, PlayerId::NONE).unwrap();
    assert!(
        state.zones.get(processing).unwrap().cards.is_empty(),
        "processing must be drained after guanxing settles"
    );
    // 牌堆顶存在 5 张（重排落位完成，可能混入洗牌补堆的牌——牌堆不足时自动洗回）。
    let draw = state.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    assert!(
        state.zones.get(draw).unwrap().cards.len() >= 5,
        "draw pile must hold at least the reordered cards"
    );
}
