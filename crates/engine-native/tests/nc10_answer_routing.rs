//! 同一栈深多台挂起机器：已校验应答只交给**提出询问的那台**。
//!
//! 反例（2026-11-01 `park-1`，NC10 七个浏览器案例）：读档件在同一栈深停着
//! 两台机器——已确认的「遗计」`OptionalGate`（内层 `JiJi` 在等自己的询问）
//! 与「机捷」`JiJie`（正等 `jijie_give_target` 应答）。修复前 `run` 按栈深
//! 顺序取第一台，把机捷的应答交给遗计内层；机捷随后读 `ctx.answer()` 命中
//! `expect("answer present on resume")`，WASM 侧报 `RuntimeError unreachable`
//! （`engine_native_kernel::machine::Ctx::answer`）。
//!
//! 修复：`Park::awaiting_answer`（运行时标记，不入快照/哈希；读档由
//! `snap::active_ask` 与开放选择逐字段唯一匹配重建）。有应答时只唤醒提问
//! 者，无应答时保持「同深度按挂起顺序恢复」原语义。

use engine_native::ai::BoundedPlanner;
use engine_native::resolution::{Slot, NO_FRAME};
use engine_native::snap::JiJieStageSnap;
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, MachineSnapshot, ParkSnapshot, Phase,
    StateSnapshot,
};

fn roundtrip(snapshot: &StateSnapshot) -> Engine {
    let bytes = engine_native::archive::encode_archive(snapshot);
    let decoded = engine_native::archive::decode_archive(&bytes).unwrap();
    assert_eq!(&decoded, snapshot, "字节往返必须逐字段一致");
    let mut expected = snapshot.clone();
    expected.auth_generation += 1; // 恢复按既有例外只轮换代次
    let engine = Engine::restore(&decoded).unwrap();
    assert_eq!(engine.save(), expected);
    engine
}

/// 与 `NativeSession::auto_step` 同款：开放选择由规划器投影成 `Answer`。
fn answer_open_choice(engine: &mut Engine, seat: u8) -> (CommandOutcome, Vec<ChoiceOption>) {
    let plan = engine.plan_action(seat, &BoundedPlanner::new(64));
    let GameAction::Answer { selection, .. } = &plan.action else {
        panic!("开放选择必须投影成应答，得到 {:?}", plan.action);
    };
    let selection = selection.clone();
    (engine.apply_plan(&plan).unwrap(), selection)
}

/// 「遗计确认门 + 同深度追加机捷挂起」的恢复件（修复前的 park-1 形态）。
fn same_depth_pair() -> Engine {
    let mut engine = Engine::new(2306, 3);
    engine.debug_set_turn(0, Phase::Play, 1);
    for seat in 0..3 {
        engine.debug_clear_hand(seat);
    }
    engine.debug_grant(1, "遗计");
    let dealt = engine.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    assert!(matches!(dealt, CommandOutcome::Pending { .. }), "{dealt:?}");
    let mut snapshot = engine.save();
    let depth = snapshot.resolution.stack.len();
    let open = snapshot.resolution.open_choice.as_ref().unwrap();
    assert_eq!(open.spec.prompt, "trigger_confirm");
    assert_eq!(snapshot.resolution.parks.len(), 1);
    assert_eq!(snapshot.resolution.parks[0].resume_depth, depth);
    // 追加第二台同深度机器：机捷尚未提问（修复前它会被当作第一台消费应答
    // 的候选位置之外的空转机器，修复后它与遗计内层互不串应答）。
    snapshot.resolution.parks.push(ParkSnapshot {
        resume_depth: depth,
        slot: Slot::Trigger { frame: NO_FRAME },
        machine: MachineSnapshot::JiJie {
            owner: 0,
            card: None,
            stage: JiJieStageSnap::Ask,
        },
    });
    roundtrip(&snapshot)
}

/// 机捷已取入手中的牌（机器持牌：不在任何牌区）。
fn jijie_held(engine: &Engine) -> u32 {
    engine
        .save()
        .resolution
        .parks
        .iter()
        .find_map(|park| match &park.machine {
            MachineSnapshot::JiJie {
                card: Some(card),
                stage: engine_native::snap::JiJieStageSnap::Read,
                ..
            } => Some(*card),
            _ => None,
        })
        .expect("机捷提问后持牌")
}

#[test]
fn restored_same_depth_answer_reaches_the_asking_machine() {
    let faults = engine_native::engine::legal_actions::probe_faults();
    let mut engine = same_depth_pair();

    // ① 确认门应答：同深度两台机器里只有遗计门在提问。
    let (outcome, _) = answer_open_choice(&mut engine, 1);
    assert!(
        matches!(outcome, CommandOutcome::Pending { .. }),
        "{outcome:?}"
    );
    let open = engine.open_choice().expect("机捷开窗").clone();
    assert_eq!(open.spec.prompt, "jijie_give_target");
    assert_eq!(open.observer, 0);
    let held = jijie_held(&engine);

    // ② 存档→恢复：反例位置（旧实现恢复后继续即 panic）。
    let mut engine = roundtrip(&engine.save());
    assert_eq!(
        engine.open_choice().unwrap().spec.prompt,
        "jijie_give_target"
    );
    assert_eq!(jijie_held(&engine), held);

    // ③ 应答机捷询问：必须由机捷消费，而不是栈深相同、排在前面的遗计内层。
    let plan = engine.plan_action(0, &BoundedPlanner::new(64));
    let GameAction::Answer { selection, .. } = &plan.action else {
        panic!("开放选择必须投影成应答，得到 {:?}", plan.action);
    };
    let target = match selection.as_slice() {
        [ChoiceOption::Player(player)] => *player,
        other => panic!("机捷询问应只含单目标，得到 {other:?}"),
    };
    let outcome = engine.apply_plan(&plan).unwrap();
    assert!(
        matches!(outcome, CommandOutcome::Pending { .. }),
        "{outcome:?}"
    );

    // 机捷把牌堆底牌交给所选角色，随后自己离场。
    assert!(
        engine.state_core().player(target).hand.contains(&held),
        "机捷取入的牌 {held} 必须交给所选角色 {target}"
    );
    assert!(
        !engine
            .save()
            .resolution
            .parks
            .iter()
            .any(|park| matches!(park.machine, MachineSnapshot::JiJie { .. })),
        "机捷应答后应完结"
    );

    // 遗计内层没有被别人的应答污染：它在自己的阶段重新提问。
    let next = engine.open_choice().expect("遗计继续提问").clone();
    assert_eq!(next.observer, 1);
    assert!(
        next.spec.prompt.starts_with("jiji_"),
        "下一询问应来自遗计内层，得到 {}",
        next.spec.prompt
    );

    // 内核不变量：探测不再捕获 panic，合法动作仍可枚举。
    assert_eq!(
        engine_native::engine::legal_actions::probe_faults(),
        faults,
        "应答错配会让候选探测触发内核 panic"
    );
    assert!(
        !engine.legal_actions(1).is_empty(),
        "恢复后继续必须仍能枚举合法动作"
    );

    // 状态自洽：可再次读档。
    let again = roundtrip(&engine.save());
    assert_eq!(again.state_hash(), engine.state_hash());
}

#[test]
fn answer_without_open_ask_is_rejected_and_keeps_the_archive_atomic() {
    let engine = same_depth_pair();
    // 无询问、无挂起、却留着一条已校验应答：应答无人认领（读档当作坏档）。
    // 合法静止点不会留下这种态——应答与询问在同一段内被消费。
    let mut broken = engine.save();
    broken.resolution.open_choice = None;
    broken.resolution.stack.clear();
    broken.resolution.parks.clear();
    broken.resolution.answer = Some(vec![ChoiceOption::Confirm]);
    let bytes = engine_native::archive::encode_archive(&broken);
    // 字节层自洽（可解码），语义层被校验拒绝。
    assert_eq!(
        engine_native::archive::decode_archive(&bytes).unwrap(),
        broken
    );
    assert!(
        Engine::restore(&broken).is_err(),
        "有应答、无承接询问的档必须被拒（应答无人认领）"
    );
    // 原子性：原引擎与其存档不受影响。
    let before = engine.save();
    assert_eq!(
        engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&before))
            .unwrap(),
        before
    );
    assert_eq!(engine.save(), before);
}
