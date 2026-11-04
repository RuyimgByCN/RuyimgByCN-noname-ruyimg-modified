//! R05-F2 批：闭合 F 批（真包接入）报告的三个语义缺口，先红后绿——每项
//! 断言**上游语义**而非原生现状，转绿只许改实现。
//!
//! · 缺口1（Current 消费）：dcbihuo 的 Current 槽在 D 机 Begin2 的内核消费
//!   点——上游 dcbihuo_sha `player:"phaseDrawBegin2"` filter
//!   `hasStorage && !numFixed` → `trigger.num += num`（有符号），
//!   sp2/skill.js:7828-7838。原生现状：Begin2 只消费 Combs，Current 从未
//!   进 pending（真包「额定摸牌 ±1」不生效）。
//! · 缺口2（跳过终窗）：摸牌阶段跳过路径补发恒发射终窗 After——上游 tiqi
//!   trigger `global:["phaseDrawEnd","phaseDrawSkipped","phaseDrawCancelled"]`
//!   （xianding/skill.js:41912），跳过面须有触发载体。Cancelled 面：原生无
//!   摸牌阶段取消的生产者（DamageOutcome::Cancelled 属伤害族；D 帧
//!   DrawOutcome::Cancelled 无发射点），登记差异、面留待有生产者时接。
//! · 缺口3（owner 角色门）：dcbihuo 伤害两面按触发面区分持有者——上游
//!   trigger `{player:"damageEnd", source:"damageSource"}`＋filter
//!   `event.source && event.player != event.source`（sp2/skill.js:7740-7743）
//!   ：damageEnd 面只问**受害者**持有者（+1）、damageSource 面只问**来源**
//!   持有者（−1）。原生现状：任意存活持有者两面都被问（旁观者也被问）。

use engine_native::content::extension::ExtensionPack;
use engine_native::{ChoiceOption as O, CommandOutcome, Engine, GameAction, GameCommand, OpenChoice, Phase};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn must(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    let outcome = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    });
    assert!(
        !matches!(outcome, CommandOutcome::Rejected(_)),
        "命令意外被拒: {outcome:?}"
    );
    outcome
}

fn default_answer(open: &OpenChoice) -> Vec<O> {
    if open.spec.options.contains(&O::Pass) {
        vec![O::Pass]
    } else if open.spec.allow_cancel {
        vec![]
    } else if open.spec.options.len() >= open.spec.min {
        open.spec.options.iter().take(open.spec.min).copied().collect()
    } else {
        panic!("无法默认应答: {:?}", open.spec)
    }
}

/// 引擎泵：有开放询问先按脚本（rvm_* 前缀必须命中，否则按默认应答），无询问
/// 推进一次阶段。
fn pump<F>(e: &mut Engine, mut script: F)
where
    F: FnMut(&OpenChoice) -> Option<Vec<O>>,
{
    if let Some(open) = e.open_choice().cloned() {
        let selection = if open.spec.prompt.starts_with("rvm_") {
            script(&open).unwrap_or_else(|| panic!("未脚本化的 rvm 询问: {}", open.spec.prompt))
        } else {
            default_answer(&open)
        };
        must(e, open.observer, GameAction::Answer {
            choice_id: open.id,
            selection,
        });
        return;
    }
    let actor = e.turn().current;
    must(e, actor, GameAction::NextPhase);
}

fn run_until<F, P>(e: &mut Engine, script: F, pred: P)
where
    F: FnMut(&OpenChoice) -> Option<Vec<O>>,
    P: Fn(&Engine) -> bool,
{
    let mut script = script;
    for _ in 0..256 {
        if pred(e) {
            return;
        }
        pump(e, |c| script(c));
    }
    panic!(
        "run_until 超预算: turn={:?}/{:?} open={:?}",
        e.turn().current,
        e.turn().phase,
        e.open_choice().map(|c| c.spec.prompt.to_string())
    );
}

fn at_phase(e: &Engine, player: u8, phase: Phase) -> bool {
    e.turn().current == player && e.turn().phase == phase
}

fn mark(e: &Engine, seat: u8, def: u32) -> Option<i32> {
    e.state_core().vm_aux.marks.get(&(seat, def)).copied()
}

fn helper_def(e: &Engine, name: &str) -> u32 {
    e.catalog().skill_by_name(name).unwrap_or_else(|| panic!("派生 helper 入目录: {name}")) as u32
}

fn pkg_root(dir: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(dir)
}

/// 生产装载路径（与 r05f_real_packages 同口径）：npack＋characters.json。
fn load_real_pack(dir: &str) -> ExtensionPack {
    let npack_json = std::fs::read_to_string(pkg_root(dir).join("package.npack.json"))
        .unwrap_or_else(|e| panic!("读 {dir}/package.npack.json 失败: {e}"));
    let npack: content_schema::ContentPackage = serde_json::from_str(&npack_json).expect("npack 反序列化");
    let decls = content_native_loader::bridge_vm_skill_decls(&npack)
        .unwrap_or_else(|e| panic!("桥接 {dir} 失败: {e:?}"))
        .decls;
    let chars = std::fs::read_to_string(pkg_root(dir).join("assets/characters.json"))
        .expect("读 characters.json");
    let generals = content_native_loader::bridge_generals_with_vm_package(&chars, &npack)
        .unwrap_or_else(|e| panic!("{dir} 武将装载失败: {e:?}"))
        .generals;
    let meta: serde_json::Value = serde_json::from_str(&chars).expect("characters.json 解析");
    let pack_name = meta["pack"]["name"].as_str().expect("pack.name");
    ExtensionPack {
        def: engine_native::content::extension::ExtensionDef {
            name: engine_native::content::names::intern(pack_name.to_string()),
            version: 1,
            description: engine_native::content::names::intern(
                meta["pack"]["description"].as_str().unwrap_or("").to_string(),
            ),
        },
        generals,
        cards: vec![],
        host_skills: vec![],
        vm_skills: decls,
        init_hook: Default::default(),
        enabled: true,
    }
}

/// seat 的摸牌阶段跨程手牌增量（判定阶段起算 → 出牌阶段读值）：隔离本 D
/// 的实际摸牌数（无其他摸牌源的标准局）。
fn drawn_across_own_draw_phase(e: &mut Engine, seat: u8) -> usize {
    run_until(e, |_| None, |e| at_phase(e, seat, Phase::Judge));
    let before = e.state_core().player(seat).hand.len();
    run_until(e, |_| None, |e| at_phase(e, seat, Phase::Play));
    e.state_core().player(seat).hand.len() - before
}

fn sha_once(e: &mut Engine) {
    let sha = e.debug_place(0, "sha");
    e.debug_clear_hand(1);
    must(e, 0, GameAction::PlayCard {
        card: sha,
        targets: vec![1],
    });
}

// ---- 缺口1：Current 槽在 Begin2 被内核消费（有符号） ----

/// Current=−1 → 额定摸牌 2−1=1，且消费后清槽与 helper 实例。
/// 红（修前）：无消费点——摸 2、条目残留。
#[test]
fn r05f2_current_minus_consumed_at_begin2_signed() {
    let pack = load_real_pack("sp2-upstream-noname");
    let mut e = Engine::new_with_extensions(34905, 3, &[&pack]).unwrap();
    let current = helper_def(&e, "dcbihuo#r05:current");
    run_until(&mut e, |_| None, |e| at_phase(e, 1, Phase::Judge));
    e.state_core_mut().vm_aux.marks.insert((1, current), -1);
    let drawn = drawn_across_own_draw_phase(&mut e, 1);
    assert_eq!(drawn, 1, "Current=−1：额定摸牌 2−1=1（Begin2 有符号消费）");
    assert_eq!(mark(&e, 1, current), None, "消费后条目清除");
    assert!(
        !e.state_core().skills.contains_key(&(1, current as u16)),
        "消费后 helper 实例清理"
    );
}

/// Current=+1 → 额定摸牌 2+1=3，消费后清槽。
/// 红（修前）：无消费点——摸 2、条目残留。
#[test]
fn r05f2_current_plus_consumed_at_begin2() {
    let pack = load_real_pack("sp2-upstream-noname");
    let mut e = Engine::new_with_extensions(34906, 3, &[&pack]).unwrap();
    let current = helper_def(&e, "dcbihuo#r05:current");
    run_until(&mut e, |_| None, |e| at_phase(e, 1, Phase::Judge));
    e.state_core_mut().vm_aux.marks.insert((1, current), 1);
    let drawn = drawn_across_own_draw_phase(&mut e, 1);
    assert_eq!(drawn, 3, "Current=+1：额定摸牌 2+1=3");
    assert_eq!(mark(&e, 1, current), None, "消费后条目清除");
}

/// num_fixed=true 的 D：Begin2 整窗跳过——不消费、条目保留（上游 filter
/// `!event.numFixed` 同门）。守卫面：修前无任何消费点时自然通过，修复后
/// 防过度消费回归；生产者（固定张数摸牌）随后续批次接入，经
/// `debug_start_draw_phase_fixed` 以真实 D 帧承载。
#[test]
fn r05f2_current_not_consumed_when_num_fixed() {
    let pack = load_real_pack("sp2-upstream-noname");
    let mut e = Engine::new_with_extensions(34907, 3, &[&pack]).unwrap();
    let current = helper_def(&e, "dcbihuo#r05:current");
    run_until(&mut e, |_| None, |e| at_phase(e, 0, Phase::Judge));
    e.state_core_mut().vm_aux.marks.insert((1, current), -1);
    let before = e.state_core().player(1).hand.len();
    e.debug_start_draw_phase_fixed(1, 2);
    assert_eq!(
        e.state_core().player(1).hand.len() - before,
        2,
        "num_fixed：按固定张数摸 2"
    );
    assert_eq!(mark(&e, 1, current), Some(-1), "num_fixed：Current 不消费保留");
}

// ---- 缺口2：摸牌阶段跳过路径的恒发射终窗 After ----

/// 兵粮寸断（非梅花判定）跳过 seat1 的摸牌阶段：终窗 After 仍发射，tiqi
/// 持有者（seat0，非自己 D）收确认→符号两问（n=0≠2 → d=2），目标未摸牌。
/// 红（修前）：跳过路径 D 帧直接 Intrinsic→Done 不发窗——无任何 tiqi 询问。
#[test]
fn r05f2_skipped_draw_phase_emits_after_and_tiqi_asks() {
    let pack = load_real_pack("xianding-upstream-noname");
    let mut e = Engine::new_with_extensions(34908, 3, &[&pack]).unwrap();
    e.assign_general_named(0, "xnd冯妤").unwrap();
    // 首轮 seat0 出牌阶段布兵粮寸断＋非梅花判定牌：seat1 的首轮判定阶段
    // 判定（非梅花 → turn.skip_draw_phase=true，kernel/judge.rs CardKind::Bing
    // 分支＝跳过条件的真实生产者）；期间无摸牌，判定翻牌即所置顶牌。
    run_until(&mut e, |_| None, |e| at_phase(e, 0, Phase::Play));
    e.debug_place_judge(1, "bing");
    e.debug_set_deck_top_by(|c| c.suit == engine_native::catalog::Suit::Heart);
    let hand_before = e.state_core().player(1).hand.len();
    let mut asked: Vec<&'static str> = Vec::new();
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_tiqi_confirm" => {
                asked.push("confirm");
                Some(vec![O::Confirm])
            }
            "rvm_tiqi_sign" => {
                asked.push("sign");
                Some(vec![O::Tag(1)])
            }
            other => panic!("意外询问 {other}"),
        },
        // 跳过链在一个命令内可级联直落到弃牌询问（无选择休息点）：到达
        // 「摸牌阶段已过」的首个可观测状态即停（Play/Discard/End 任一）。
        |e| {
            e.turn().current == 1
                && matches!(
                    e.turn().phase,
                    Phase::Play | Phase::Discard | Phase::End
                )
        },
    );
    assert_eq!(asked, vec!["confirm", "sign"], "跳过 D 的终窗 After 触发 tiqi 两问");
    assert_eq!(
        e.state_core().player(1).hand.len(),
        hand_before,
        "跳过＝目标未摸牌"
    );
    let def = helper_def(&e, "tiqi#r05:limit");
    assert_eq!(mark(&e, 1, def), Some(2), "n=0 → d=2 → +2 写目标 Limit");
}

// ---- 缺口3：dcbihuo 伤害两面的 owner 角色门 ----

/// 旁观持有者（非受害者非来源）两面都不被问。
/// 红（修前）：任意存活持有者都被问——旁观者 seat1（持 sp2冯方）收两问。
#[test]
fn r05f2_dcbihuo_bystander_holder_not_asked() {
    let pack = load_real_pack("sp2-upstream-noname");
    let mut e = Engine::new_with_extensions(34909, 3, &[&pack]).unwrap();
    e.assign_general_named(2, "sp2冯方").unwrap();
    run_until(&mut e, |_| None, |e| at_phase(e, 0, Phase::Play));
    sha_once(&mut e);
    run_until(
        &mut e,
        |c| panic!("旁观持有者（非受害者/来源）不应被问: {}", c.spec.prompt),
        |e| !at_phase(e, 0, Phase::Play),
    );
    let next = helper_def(&e, "dcbihuo#r05:next");
    for seat in 0..3 {
        assert_eq!(mark(&e, seat, next), None, "旁观持有者不写任何 Next（seat{seat}）");
    }
}

/// 受害者持有者：仅 End 面被问（+1 方向），Source 面不问（其非来源）。
/// 红（修前）：同一持有者两面都被问（2 问）。
#[test]
fn r05f2_dcbihuo_victim_holder_end_face_plus_only() {
    let pack = load_real_pack("sp2-upstream-noname");
    let mut e = Engine::new_with_extensions(34910, 3, &[&pack]).unwrap();
    e.assign_general_named(1, "sp2冯方").unwrap();
    run_until(&mut e, |_| None, |e| at_phase(e, 0, Phase::Play));
    sha_once(&mut e);
    let mut observers: Vec<u8> = Vec::new();
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_dcbihuo_target" => {
                observers.push(c.observer);
                Some(vec![O::Player(1)])
            }
            other => panic!("意外询问 {other}"),
        },
        |e| !at_phase(e, 0, Phase::Play),
    );
    assert_eq!(observers, vec![1], "仅 End 面问受害者持有者（一次）");
    let next = helper_def(&e, "dcbihuo#r05:next");
    assert_eq!(mark(&e, 1, next), Some(1), "End 面 +1 写目标 Next");
    // take → Current=+1 → 本回合额定摸牌 2+1=3。
    run_until(&mut e, |_| None, |e| at_phase(e, 1, Phase::Judge));
    let current = helper_def(&e, "dcbihuo#r05:current");
    assert_eq!(mark(&e, 1, current), Some(1), "take 转 Current=+1");
    let drawn = drawn_across_own_draw_phase(&mut e, 1);
    assert_eq!(drawn, 3, "End 面 +1：额定摸牌 3");
}

/// 来源持有者：仅 Source 面被问（−1 方向），End 面不问（其非受害者）。
/// 红（修前）：同一持有者两面都被问（2 问）。
#[test]
fn r05f2_dcbihuo_source_holder_source_face_minus_only() {
    let pack = load_real_pack("sp2-upstream-noname");
    let mut e = Engine::new_with_extensions(34911, 3, &[&pack]).unwrap();
    e.assign_general_named(0, "sp2冯方").unwrap();
    run_until(&mut e, |_| None, |e| at_phase(e, 0, Phase::Play));
    sha_once(&mut e);
    let mut observers: Vec<u8> = Vec::new();
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_dcbihuo_target" => {
                observers.push(c.observer);
                Some(vec![O::Player(1)])
            }
            other => panic!("意外询问 {other}"),
        },
        |e| !at_phase(e, 0, Phase::Play),
    );
    assert_eq!(observers, vec![0], "仅 Source 面问来源持有者（一次）");
    let next = helper_def(&e, "dcbihuo#r05:next");
    assert_eq!(mark(&e, 1, next), Some(-1), "Source 面 −1 写目标 Next");
    run_until(&mut e, |_| None, |e| at_phase(e, 1, Phase::Judge));
    let current = helper_def(&e, "dcbihuo#r05:current");
    assert_eq!(mark(&e, 1, current), Some(-1), "take 转 Current=−1");
    let drawn = drawn_across_own_draw_phase(&mut e, 1);
    assert_eq!(drawn, 1, "Source 面 −1：额定摸牌 2−1=1");
}

// ---- R05-D：伤害凭据、窗口序、source 死亡清除与护甲/无限体力（合同 v3 §7 & §9 场景 7/8） ----

/// 缺 source（无来源伤害）、自伤（source == target）、0 点伤害与绝情转体力流失：
/// 均不满足 dcbihuo 的 committed_damage + 存活非自伤 source 凭据，Source/End 两面都不问。
#[test]
fn r05d_dcbihuo_missing_source_self_zero_and_lose_hp_do_not_trigger() {
    let pack = load_real_pack("sp2-upstream-noname");
    let mut e = Engine::new_with_extensions(34912, 3, &[&pack]).unwrap();
    e.assign_general_named(0, "sp2冯方").unwrap();
    e.assign_general_named(1, "sp2冯方").unwrap();
    run_until(&mut e, |_| None, |e| at_phase(e, 0, Phase::Play));

    // 1) 无来源伤害（source = None）
    let out = e.debug_deal_damage(None, 1, 1, engine_native::event::DamageNature::Normal);
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    assert!(e.open_choice().is_none(), "无来源伤害不应触发 dcbihuo");

    // 2) 自伤（source == target == 0）
    let out = e.debug_deal_damage(Some(0), 0, 1, engine_native::event::DamageNature::Normal);
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    assert!(e.open_choice().is_none(), "自伤不应触发 dcbihuo");

    // 3) 0 点伤害（amount = 0 → Zero，committed_damage = None）
    let out = e.debug_deal_damage(Some(0), 1, 0, engine_native::event::DamageNature::Normal);
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    assert!(e.open_choice().is_none(), "0 点伤害未提交 committed_damage，不应触发 dcbihuo");

    // 4) 绝情（绝情）将伤害转为失去体力（ReplacedByLoseHp，committed_damage = None）
    e.debug_grant(0, "绝情");
    let hp_before = e.debug_hp(1);
    let out = e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    assert_eq!(e.debug_hp(1), hp_before - 1, "绝情转为 1 点体力流失");
    assert!(e.open_choice().is_none(), "绝情转失去体力不应触发 dcbihuo");
}

/// 护甲全吸收与无限体力：体力实际未扣减（hp_loss = 0），但伤害已正数提交
/// （committed_damage = Some(1)），Source 先于 End 各触发一次；支持取消或选自己/他人。
#[test]
fn r05d_dcbihuo_armor_full_absorption_and_infinite_hp_still_trigger_source_then_end() {
    let pack = load_real_pack("sp2-upstream-noname");
    let mut e = Engine::new_with_extensions(34913, 3, &[&pack]).unwrap();
    e.assign_general_named(0, "sp2冯方").unwrap();
    e.assign_general_named(1, "sp2冯方").unwrap();
    run_until(&mut e, |_| None, |e| at_phase(e, 0, Phase::Play));

    // A) 护甲全吸收：seat1 护甲 2，受 1 点伤害 → 护甲剩 1、体力不变，但 Source(0)→End(1) 照常触发
    e.debug_set_hujia(1, 2);
    let hp_before = e.debug_hp(1);
    let out = e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    let mut observers = Vec::new();
    while let Some(open) = e.open_choice().cloned() {
        assert_eq!(open.spec.prompt, "rvm_dcbihuo_target");
        observers.push(open.observer);
        // Source(0) 选自己(0) 写 -1；End(1) 选他人(2) 写 +1
        let target = if open.observer == 0 { O::Player(0) } else { O::Player(2) };
        must(&mut e, open.observer, GameAction::Answer {
            choice_id: open.id,
            selection: vec![target],
        });
    }
    assert_eq!(observers, vec![0, 1], "护甲全吸收仍按 Source(0) → End(1) 各触发一次");
    assert_eq!(e.debug_hp(1), hp_before, "护甲全吸收体力未减");
    assert_eq!(e.state_core().player(1).hujia, 1, "护甲消耗 1");
    let next = helper_def(&e, "dcbihuo#r05:next");
    assert_eq!(mark(&e, 0, next), Some(-1), "Source 选自己写 -1");
    assert_eq!(mark(&e, 2, next), Some(1), "End 选他人写 +1");

    // B) 无限体力：seat1 设 INFINITE_HP，受 1 点伤害 → hp_loss=0，Source(0)→End(1) 仍各触发一次
    e.debug_set_max_hp(1, engine_native::state::INFINITE_HP);
    e.debug_set_hp(1, engine_native::state::INFINITE_HP as i32);
    let out = e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    let mut observers_inf = Vec::new();
    while let Some(open) = e.open_choice().cloned() {
        assert_eq!(open.spec.prompt, "rvm_dcbihuo_target");
        observers_inf.push(open.observer);
        // 两面均取消
        must(&mut e, open.observer, GameAction::Answer {
            choice_id: open.id,
            selection: vec![],
        });
    }
    assert_eq!(observers_inf, vec![0, 1], "无限体力仍按 Source(0) → End(1) 各触发一次");
}

/// 受害者濒死阵亡后：存活来源（seat0）在 Source 窗照常触发 dcbihuo（§7：victim 死亡不取消存活 source 资格）；
/// 阵亡受害者（seat1）不再触发 End 窗。
#[test]
fn r05d_dcbihuo_victim_dies_alive_source_still_triggers_source_face() {
    let pack = load_real_pack("sp2-upstream-noname");
    let mut e = Engine::new_with_extensions(34914, 3, &[&pack]).unwrap();
    // 设为 主-反-反 避免 seat1 阵亡直接终局
    e.debug_set_identity_roles(vec![
        engine_native::state::Role::Lord,
        engine_native::state::Role::Rebel,
        engine_native::state::Role::Rebel,
    ]);
    e.assign_general_named(0, "sp2冯方").unwrap();
    e.assign_general_named(1, "sp2冯方").unwrap();
    run_until(&mut e, |_| None, |e| at_phase(e, 0, Phase::Play));
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_set_hp(1, 1);

    let out = e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    let mut dcbihuo_observers = Vec::new();
    while let Some(open) = e.open_choice().cloned() {
        let sel = if open.spec.prompt == "rvm_dcbihuo_target" {
            dcbihuo_observers.push(open.observer);
            vec![O::Player(2)]
        } else {
            default_answer(&open)
        };
        must(&mut e, open.observer, GameAction::Answer {
            choice_id: open.id,
            selection: sel,
        });
    }
    assert!(!e.debug_alive(1), "受害者 seat1 已阵亡");
    assert_eq!(
        dcbihuo_observers,
        vec![0],
        "victim 死亡不取消存活 source 的 Source 窗资格，且阵亡 victim 不再触发 End 窗"
    );
    let next = helper_def(&e, "dcbihuo#r05:next");
    assert_eq!(mark(&e, 2, next), Some(-1), "存活 source 在 Source 窗写 seat2 Next = -1");
}

/// 来源在 damage（TargetDamage）子事件中阵亡后：外层伤害帧的 source 被清除为 None，
/// 后续 Source/End 窗不再触发（§7：死亡 source 清 None，End 不补回原 source）。
#[test]
fn r05d_dcbihuo_source_dies_in_damage_subevent_clears_source_before_end() {
    let pack = load_real_pack("sp2-upstream-noname");
    let mut e = Engine::new_with_extensions(34915, 3, &[&pack]).unwrap();
    e.debug_set_identity_roles(vec![
        engine_native::state::Role::Rebel,
        engine_native::state::Role::Lord,
        engine_native::state::Role::Rebel,
    ]);
    e.assign_general_named(0, "sp2冯方").unwrap();
    e.assign_general_named(1, "sp2冯方").unwrap();
    e.debug_grant(1, "刚烈");
    run_until(&mut e, |_| None, |e| at_phase(e, 0, Phase::Play));
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_set_hp(0, 1);
    // 刚烈判定非红桃（黑桃）→ seat0 无手牌不能弃 2 张 → 受 1 点反伤阵亡
    e.debug_set_deck_top_by(|c| c.suit == engine_native::catalog::Suit::Spade);

    let out = e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    let mut dcbihuo_observers = Vec::new();
    while let Some(open) = e.open_choice().cloned() {
        let sel = if open.spec.prompt == "rvm_dcbihuo_target" {
            dcbihuo_observers.push(open.observer);
            vec![O::Player(2)]
        } else if open.spec.options.contains(&O::Confirm) {
            // 发动刚烈
            vec![O::Confirm]
        } else {
            default_answer(&open)
        };
        must(&mut e, open.observer, GameAction::Answer {
            choice_id: open.id,
            selection: sel,
        });
    }
    assert!(!e.debug_alive(0), "来源 seat0 在刚烈反伤子事件中阵亡");
    // 内层刚烈反伤（1→0）：seat1 作为存活来源在 Source 面（−1）被问 1 次；
    // 外层伤害（0→1）：因来源 seat0 已阵亡被清为 None，外层 Source(0) 与外层 End(1,+1) 均不触发！
    assert_eq!(
        dcbihuo_observers,
        vec![1],
        "仅内层反伤（1→0）的 Source 面问 seat1 一次；外层伤害（0→1）因 source 阵亡清 None，End 面不再触发"
    );
    let next = helper_def(&e, "dcbihuo#r05:next");
    assert_eq!(
        mark(&e, 2, next),
        Some(-1),
        "仅内层 Source 写 −1（若外层 End 误触发则会抵消为 0）"
    );
}

/// 铁索连环传导：每帧独立结算，首帧（0→1）触发 Source(0)→End(1)，传导帧（0→2）触发 Source(0)→End(2)。
#[test]
fn r05d_dcbihuo_chain_damage_frames_independent() {
    let pack = load_real_pack("sp2-upstream-noname");
    let mut e = Engine::new_with_extensions(34916, 3, &[&pack]).unwrap();
    e.assign_general_named(0, "sp2冯方").unwrap();
    e.assign_general_named(1, "sp2冯方").unwrap();
    e.assign_general_named(2, "sp2冯方").unwrap();
    run_until(&mut e, |_| None, |e| at_phase(e, 0, Phase::Play));
    e.state_core_mut().player_mut(1).linked = true;
    e.state_core_mut().player_mut(2).linked = true;

    let out = e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Fire);
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    let mut observers = Vec::new();
    while let Some(open) = e.open_choice().cloned() {
        assert_eq!(open.spec.prompt, "rvm_dcbihuo_target");
        observers.push(open.observer);
        must(&mut e, open.observer, GameAction::Answer {
            choice_id: open.id,
            selection: vec![O::Player(open.observer)],
        });
    }
    assert_eq!(
        observers,
        vec![0, 1, 0, 2],
        "铁索传导每帧独立按 Source → End 触发"
    );
    let next = helper_def(&e, "dcbihuo#r05:next");
    assert_eq!(mark(&e, 0, next), Some(-2), "seat0 两帧 Source 各写 -1");
    assert_eq!(mark(&e, 1, next), Some(1), "seat1 首帧 End 写 +1");
    assert_eq!(mark(&e, 2, next), Some(1), "seat2 传导帧 End 写 +1");
}


