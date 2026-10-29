//! ADR 0786 J1（裁决第 13 项，规则 v328）：VM 提交期选择窗挂起。
//!
//! 修复前反例：`discard_hand_equip_count(N>0)` 在 Halted 落账时被内联为
//! 「确定性首 N」近似——**不开任何选择窗**，玩家无权决定弃哪几张；旧线
//! 语义为「开窗选恰好 N 张并弃置」。本测试族以真实 DSL 编译包验收：
//!
//! - 真实选择：开窗（恰好 N、手牌+装备、不可取消）→ 按所选弃置（反例：
//!   选非前 N 张，确定性近似不可能保留它们）→ 队列剩余效果继续落账；
//! - 不足封顶与空过：可用 < N 时窗口恰好=可用；无牌空过不阻塞后续效果；
//! - 存读档：开窗挂起中存档→归档往返→恢复哈希一致→应答完成同一效果
//!   （GT-011 口径：提交期挂起是可恢复状态）；
//! - N=0 首发拒绝照旧（方案①：不新增原语）。
//!
//! 注意时序：`card_targeted` 映射 CardUsed，其 AfterTriggers（VM 机落账开窗）
//! 先于用牌固有流程（目标闪询问）——窗口检查/应答必须在 `drain` 之前。

mod nc08_common;

use engine_native::content::extension::ExtensionPack;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};

// dedup 计数器必须与 nc08_common 共用（advance_to_play 已消费序号；自建
// 计数器会与引擎去重高水位冲突 → DuplicateCommand）。

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: nc08_common::next_dedup(),
        action,
    })
}

fn answer(e: &mut Engine, selection: Vec<ChoiceOption>) {
    let c = e.open_choice().cloned().expect("choice open");
    let out = submit(
        e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection,
        },
    );
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
}

fn pack_for(source: &str, id: &str) -> ExtensionPack {
    let pkg = content_compiler::compile_source_named(id, source).expect("编译 .skill");
    let out = content_native_loader::bridge_vm_skill_decls(&pkg).expect("正式桥接");
    let mut pack = ExtensionPack {
        def: engine_native::content::extension::ExtensionDef {
            name: engine_native::content::names::intern(id.to_string()),
            version: 1,
            description: engine_native::content::names::intern("j1".to_string()),
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills: out.decls,
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    pack.vm_skills.truncate(1);
    pack
}

const PROBE: &str = "skill j1_probe(991031) {\n  trigger card_targeted always {\n    effect discard_hand_equip_count(event_player, 2);\n    effect draw_cards(event_player, 1);\n  }\n}\n";

/// 反例反转：真实选牌窗 + 按所选弃置（非首 N）+ 队列继续（随后摸 1）。
#[test]
fn j1_real_choice_window_discards_picked_cards_and_resumes_queue() {
    let pack = pack_for(PROBE, "j1-probe");
    let name = pack.vm_skills[0].name.clone();
    let mut e = Engine::new_with_extensions(990_001, 4, &[&pack]).expect("建局");
    nc08_common::advance_to_play(&mut e, 0);
    e.debug_grant(0, &name);
    e.debug_clear_hand(1);

    // 布牌：0 号手牌固定为 4 张可区分实体（打出杀后余 3，开窗选 2 留 1）。
    e.debug_clear_hand(0);
    // debug_place 从牌堆取出实体（区唯一——put_hand 的目录 id 会与牌堆
    // 重复，恢复期 card_in_multiple_zones 校验拒绝）。
    let c_sha = e.debug_place(0, "sha");
    let c_shan = e.debug_place(0, "shan");
    let c_tao = e.debug_place(0, "tao");
    let c_wuxie = e.debug_place(0, "nullify");
    let (keep, pick) = (c_wuxie, [c_shan, c_tao]);

    // 出杀指定 1 号触发 card_targeted；VM 机落账遇开窗效果（先于闪询问）。
    let out = submit(&mut e, 0, GameAction::PlayCard { card: c_sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));

    // 反例断言（修复前红）：真实开窗——恰好 2 张、候选含全部手牌、不可取消。
    let window = e
        .open_choice()
        .cloned()
        .expect("提交期开窗（修复前为确定性首 N，无窗口）");
    assert_eq!((window.observer, window.spec.prompt), (0, "rvm_discard_pick"));
    assert!(matches!(
        window.spec.kind,
        engine_native::choice::ChoiceKind::SelectCards
    ));
    assert_eq!((window.spec.min, window.spec.max), (2, 2));
    assert!(!window.spec.allow_cancel);
    assert!(window.spec.options.contains(&ChoiceOption::Card(c_shan)));
    assert!(window.spec.options.contains(&ChoiceOption::Card(c_tao)));

    // 玩家选**非前 N**两张（确定性首 N 必弃 c_sha、c_shan）。
    answer(&mut e, vec![ChoiceOption::Card(pick[0]), ChoiceOption::Card(pick[1])]);
    // 收尾目标闪询问（1 号无闪→Pass→伤害）等。
    nc08_common::drain(&mut e);

    let hand = e.state_core().player(0).hand.clone();
    assert!(hand.contains(&keep), "未选中的牌必须保留（确定性首 N 会弃它）");
    assert!(!hand.contains(&pick[0]) && !hand.contains(&pick[1]), "只弃所选");
    // 队列剩余效果继续落账：draw_cards(event_player, 1)。
    assert_eq!(hand.len(), 2, "保留 1 张 + 后续摸 1 张");
    assert!(e.open_choice().is_none(), "锁定技窗口答毕无残留");
    let discard = e.state_core().discard.clone();
    assert!(discard.contains(&pick[0]) && discard.contains(&pick[1]), "所选入弃堆");
}

/// 不足封顶（可用 1 < N 2 → 恰好 1）与空过（无牌 → 效果跳过、后续照常）。
#[test]
fn j1_window_caps_to_available_and_skips_when_empty() {
    let pack = pack_for(PROBE, "j1-probe");
    let name = pack.vm_skills[0].name.clone();

    // ① 可用 1 张（打出杀后仅剩闪）：窗口恰好 1。
    let mut e = Engine::new_with_extensions(990_002, 4, &[&pack]).expect("建局");
    nc08_common::advance_to_play(&mut e, 0);
    e.debug_grant(0, &name);
    e.debug_clear_hand(1);
    e.debug_clear_hand(0);
    let c_sha = e.debug_place(0, "sha");
    let c_shan = e.debug_place(0, "shan");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: c_sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    let window = e.open_choice().cloned().expect("不足封顶仍开窗");
    assert_eq!(window.spec.prompt, "rvm_discard_pick");
    assert_eq!((window.spec.min, window.spec.max), (1, 1), "可用 1 < N 2 → 恰好 1");
    answer(&mut e, vec![ChoiceOption::Card(c_shan)]);
    nc08_common::drain(&mut e);
    let hand = e.state_core().player(0).hand.clone();
    assert_eq!(hand.len(), 1, "唯一可用牌被弃 + 队列后续 draw 1");
    assert!(!hand.contains(&c_shan), "被弃的闪不回手");

    // ② 无牌（打出唯一一张杀后手牌空）：开窗效果空过、不阻塞后续 draw。
    let mut e = Engine::new_with_extensions(990_003, 4, &[&pack]).expect("建局");
    nc08_common::advance_to_play(&mut e, 0);
    e.debug_grant(0, &name);
    e.debug_clear_hand(1);
    e.debug_clear_hand(0);
    let c_sha = e.debug_place(0, "sha");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: c_sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    let first = e.open_choice().cloned();
    assert!(
        first.as_ref().is_none_or(|c| c.spec.prompt != "rvm_discard_pick"),
        "无牌空过：不开窗（其后最多是目标闪询问）"
    );
    nc08_common::drain(&mut e);
    let hand = e.state_core().player(0).hand.clone();
    assert_eq!(hand.len(), 1, "空过后 draw_cards 仍落账（打出杀后手牌 0 + 摸 1）");
    assert!(!hand.contains(&c_sha), "打出的杀不回手");
}

/// GT-011 口径：开窗挂起中存档→归档往返→恢复哈希一致→应答完成同一效果，
/// 与未中断路径的终态一致。
#[test]
fn j1_commit_window_saves_and_restores_across_archive() {
    let pack = pack_for(PROBE, "j1-probe");
    let name = pack.vm_skills[0].name.clone();
    let mut e = Engine::new_with_extensions(990_004, 4, &[&pack]).expect("建局");
    nc08_common::advance_to_play(&mut e, 0);
    e.debug_grant(0, &name);
    e.debug_clear_hand(1);
    e.debug_clear_hand(0);
    // debug_place 从牌堆取出实体（区唯一——put_hand 的目录 id 会与牌堆
    // 重复，恢复期 card_in_multiple_zones 校验拒绝）。
    let c_sha = e.debug_place(0, "sha");
    let c_shan = e.debug_place(0, "shan");
    let c_tao = e.debug_place(0, "tao");
    let c_wuxie = e.debug_place(0, "nullify");
    let (keep, pick) = (c_wuxie, [c_shan, c_tao]);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: c_sha, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    let window = e.open_choice().cloned().expect("提交期开窗");
    assert_eq!(window.spec.prompt, "rvm_discard_pick");

    let snap = e.save();
    let bytes = engine_native::archive::encode_archive(&snap);
    let decoded = engine_native::archive::decode_archive(&bytes).unwrap();
    let mut restored = Engine::restore_with_extensions(&decoded, &[&pack])
        .expect("提交期挂起档可恢复");
    assert_eq!(restored.state_hash(), e.state_hash(), "恢复哈希一致");
    let rwindow = restored.open_choice().cloned().expect("恢复后派生同一开窗");
    assert_eq!((rwindow.observer, rwindow.spec.prompt), (0, "rvm_discard_pick"));
    assert_eq!((rwindow.spec.min, rwindow.spec.max), (2, 2));

    for g in [&mut restored, &mut e] {
        answer(g, vec![ChoiceOption::Card(pick[0]), ChoiceOption::Card(pick[1])]);
        nc08_common::drain(g);
        let hand = g.state_core().player(0).hand.clone();
        assert!(hand.contains(&keep) && hand.len() == 2);
        assert!(g.open_choice().is_none());
    }
    assert_eq!(restored.state_hash(), e.state_hash(), "应答并收尾后终态哈希一致");
}

/// 方案①边界：N=0（VM 首发开选牌）不在 J1 范围——DSL 编译期即拒
/// （typecheck「discard count must be a positive integer」）；rvm_ops accept
/// 期的 N=0 拒绝是对**手写字节码**的防御层（既有行为，照旧）。
#[test]
fn j1_zero_count_first_launch_remains_rejected() {
    let source = "skill j1_zero(991032) {\n  trigger card_targeted always {\n    effect discard_hand_equip_count(event_player, 0);\n  }\n}\n";
    let err = content_compiler::compile_source_named("j1-zero", source)
        .expect_err("N=0 字面量在 DSL 编译期拒绝");
    assert!(
        format!("{err:?}").contains("positive"),
        "编译期拒绝原因可定位: {err:?}"
    );
}
