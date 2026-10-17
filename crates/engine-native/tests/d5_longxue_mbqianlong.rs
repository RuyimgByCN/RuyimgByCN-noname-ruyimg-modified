//! D5 龙血玄黄批 117：潜龙道心值（`mbqianlong`）+ 卫统（`mbweitong`）。
//!
//! 真实包 `packages/extensions/longxue-upstream-liuli`（来源 liuli
//! `extension/龙血玄黄/extension.js`，宿主武将手杀曹髦 `mb_caomao`）。
//! 道心值：开局 20（另有魏势力持卫统者 60）；获得牌 +5、受伤 +10×伤害、
//! 造成伤害 +15×伤害（上限 99）；阈值 25/50/75/99 解锁清正/酒诗/放逐/绝进
//! （外部技能存在才授予，缺目录按上游守卫跳过）。

use engine_native::event::DamageNature;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn repo_extensions_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn general_def_of(e: &Engine, name: &str) -> u16 {
    for def in 0..=u16::MAX {
        if e.catalog().general_meta(def).map(|g| g.0) == Some(name) {
            return def;
        }
        if e.catalog().general_meta(def).is_none() && def > 64 {
            break;
        }
    }
    panic!("目录中无武将 {name}");
}

fn bridge_pack(name: &str) -> engine_native::content::extension::ExtensionPack {
    let json = std::fs::read_to_string(
        repo_extensions_root().join(format!("{name}/assets/cards.json")),
    )
    .expect("读包数据");
    content_native_loader::bridge_pack(&json).expect("桥接包").pack
}

/// 建局并把 `mb_caomao` 装到 0 号（未推进阶段：开局面由首次阶段开始触发）。
fn setup(seed: u64) -> Engine {
    setup_seat(seed, 0)
}

/// 建局并把 `mb_caomao` 装到指定座位（非当前回合者不受首个摸牌阶段影响，
/// 便于隔离开局面 +20）。
fn setup_seat(seed: u64, seat: u8) -> Engine {
    let pack = bridge_pack("longxue-upstream-liuli");
    let mut e = Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局");
    let def = general_def_of(&e, "mb_caomao");
    e.debug_assemble_general_def(seat, def);
    e
}

fn drain(e: &mut Engine) {
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else { break };
        let has_pass = c.spec.options.contains(&engine_native::ChoiceOption::Pass);
        let sel: Vec<engine_native::ChoiceOption> = if has_pass {
            vec![engine_native::ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: sel },
        });
    }
}

/// 推进到下一个阶段开始（触发开局面等阶段触发）。
fn next_phase(e: &mut Engine) {
    let actor = e.turn().current;
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(e);
}

/// 包装载与依赖登记：曹髦两技映射，四个 mbcm* 外部依赖保持未映射。
#[test]
fn longxue_package_maps_host_and_defers_external() {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("longxue-upstream-liuli/assets/cards.json"),
    )
    .unwrap();
    let bridged = content_native_loader::bridge_generals(&json).expect("武将可装载");
    assert_eq!(bridged.generals.len(), 1);
    assert_eq!(bridged.generals[0].name, "mb_caomao");
    assert_eq!(bridged.generals[0].max_hp, 3);
    assert_eq!(bridged.mapped_skill_keys, vec!["mbqianlong", "mbweitong"]);
    // 四个外部技能写在包级 `skills` 段（mapping null，登记用途），不计入
    // 武将未映射计数；关键断言是它们未入目录（下方）。
    assert_eq!(bridged.unmapped_skills, 0);
    let e = setup(94001);
    assert!(e.catalog().skill_by_name("mbqianlong").is_some());
    assert!(e.catalog().skill_by_name("mbweitong").is_some());
    assert!(
        e.catalog().skill_by_name("mbcmqingzheng").is_some(),
        "清正已转换入目录（批 119）"
    );
    for name in ["mbcmjiushi", "mbcmfangzhu", "mbjuejin"] {
        assert!(
            e.catalog().skill_by_name(name).is_none(),
            "{name} 外部依赖未入目录（不虚构空操作）"
        );
    }
}

/// 开局面：首次阶段开始 +20；有另一魏势力持卫统者 +60（非当前回合座位，
/// 隔离首个摸牌阶段的 +5）。
#[test]
fn longxue_opening_marks_twenty_or_sixty() {
    let mut e = setup_seat(94002, 1);
    assert_eq!(e.state_core().player(1).mbqianlong_marks, 0, "装配未推进：0");
    next_phase(&mut e);
    assert_eq!(
        e.state_core().player(1).mbqianlong_marks,
        20,
        "首次阶段开始 +20（非当前回合座位不摸牌）"
    );

    // 另建一局：2 号位也装曹髦（魏、持卫统）→ 1 号开局面 +60
    let pack = bridge_pack("longxue-upstream-liuli");
    let mut e = Engine::new_with_extensions(94003, 4, &[&pack]).expect("建局");
    let def = general_def_of(&e, "mb_caomao");
    e.debug_assemble_general_def(1, def);
    e.debug_assemble_general_def(2, def);
    next_phase(&mut e);
    assert_eq!(
        e.state_core().player(1).mbqianlong_marks,
        60,
        "另一魏势力持卫统 → 60"
    );
    assert_eq!(
        e.state_core().player(2).mbqianlong_marks,
        60,
        "2 号同条件同样 60"
    );
}

/// 累加面：获得牌 +5、受伤 +10×、造成伤害 +15×；上限 99。
#[test]
fn longxue_marks_accumulate_and_cap() {
    let mut e = setup(94004);
    next_phase(&mut e);
    // 0 号为当前回合：开局面 +20 与首个摸牌阶段摸 2 张的 gainEnd +5 = 25
    assert_eq!(e.state_core().player(0).mbqianlong_marks, 25, "20 + 摸牌 5");
    let base = 25;

    // 获得牌（交牌）：+5
    let shan = e.debug_place(1, "shan");
    e.debug_give_hand_card(1, 0, shan);
    drain(&mut e);
    assert_eq!(
        e.state_core().player(0).mbqianlong_marks,
        base + 5,
        "获得牌 +5"
    );
    // 阈值 25 已过：清正已转换 → 授予（批 119）
    let qz = e.catalog().skill_by_name("mbcmqingzheng").expect("清正入目录");
    assert!(e.state_core().skills.contains_key(&(0, qz)), "25 阈值授予清正");

    // 受伤 2 点：+20
    let out = e.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(
        e.state_core().player(0).mbqianlong_marks,
        base + 25,
        "受伤 2 点 +20"
    );

    // 造成伤害 3 点：+45
    let out = e.debug_deal_damage(Some(0), 1, 3, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(
        e.state_core().player(0).mbqianlong_marks,
        base + 70,
        "造成 3 点伤害 +45"
    );

    // 上限 99（先回满体力：受伤致死者不触发受击面）
    e.debug_set_hp(0, 3);
    let out = e.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.state_core().player(0).mbqianlong_marks, 99, "上限 99");
    // 满值后再受伤不再增长（filter marks<99）
    let out = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.state_core().player(0).mbqianlong_marks, 99, "满值不再增长");
}

/// 阈值授予：目录存在外部技能时按 25/50/75/99 授予（测试包把清正映射到
/// 无效果旗标行为以便观察）。
#[test]
fn longxue_threshold_grants_when_skill_exists() {
    let mut json = std::fs::read_to_string(
        repo_extensions_root().join("longxue-upstream-liuli/assets/cards.json"),
    )
    .unwrap();
    // 清正改挂旗标行为（host:lingshi 为锁定旗标，效果臂按技能名 "lingshi"
    // 查询——本包技能名为 mbcmqingzheng，故无实际效果）
    let marker = "\"name\": \"mbcmqingzheng\"";
    let idx = json.find(marker).expect("清正条目");
    let tail = &json[idx..];
    let m = tail.find("\"mapping\": null").expect("清正 mapping");
    let mut patched = String::new();
    patched.push_str(&json[..idx + m]);
    patched.push_str("\"mapping\": \"host:lingshi\"");
    patched.push_str(&tail[m + "\"mapping\": null".len()..]);
    json = patched;
    assert!(json.contains("host:lingshi"));
    let bridged = content_native_loader::bridge_pack(&json).expect("测试包桥接");
    let pack = bridged.pack;
    let mut e = Engine::new_with_extensions(94005, 4, &[&pack]).expect("建局");
    let def = general_def_of(&e, "mb_caomao");
    e.debug_assemble_general_def(1, def);
    let qz = e.catalog().skill_by_name("mbcmqingzheng").expect("清正入目录");
    next_phase(&mut e);
    assert_eq!(e.state_core().player(1).mbqianlong_marks, 20);
    assert!(!e.state_core().skills.contains_key(&(1, qz)), "20 < 25 未授予");
    let shan = e.debug_place(0, "shan");
    e.debug_give_hand_card(0, 1, shan);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).mbqianlong_marks, 25);
    assert!(
        e.state_core().skills.contains_key(&(1, qz)),
        "25 阈值授予清正"
    );
}

/// 卫统：开局面登记（无机器效果）；开局面只触发一次；存档往返保持道心值。
#[test]
fn longxue_weitong_registration_only() {
    let pack = bridge_pack("longxue-upstream-liuli");
    let mut e = Engine::new_with_extensions(94006, 4, &[&pack]).expect("建局");
    let def = general_def_of(&e, "mb_caomao");
    e.debug_assemble_general_def(1, def);
    let wt = e.catalog().skill_by_name("mbweitong").expect("卫统");
    assert!(e.state_core().skills.contains_key(&(1, wt)));
    next_phase(&mut e);
    assert_eq!(e.state_core().player(1).mbqianlong_marks, 20);
    assert_eq!(e.state_core().player(0).mbqianlong_marks, 0, "无潜龙者 0");
    // 存档往返保持道心值
    let snap = e.save();
    assert_eq!(snap.core.players[1].mbqianlong_marks, 20);
    let mut restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    assert_eq!(restored.state_core().player(1).mbqianlong_marks, 20);
    // 阶段推进不重复开局面（phase_number != 1）
    for _ in 0..3 {
        next_phase(&mut restored);
    }
    assert_eq!(
        restored.state_core().player(1).mbqianlong_marks,
        20,
        "开局面只触发一次"
    );
}

/// 清正（道心值 25 阈值解锁的手杀包技能）：弃自己一种花色全部手牌 → 令目标
/// 弃其一种花色全部手牌 → 其弃牌数少于自己时受 1 点伤害（无来源）。
#[test]
fn qingzheng_discards_both_sides_and_punishes_fewer() {
    let pack = bridge_pack("longxue-upstream-liuli");
    let mut e = Engine::new_with_extensions(94007, 4, &[&pack]).expect("建局");
    let def = general_def_of(&e, "mb_caomao");
    e.debug_assemble_general_def(0, def);
    let qz = e.catalog().skill_by_name("mbcmqingzheng").expect("清正在目录");
    next_phase(&mut e);
    assert_eq!(e.state_core().player(0).mbqianlong_marks, 25);
    assert!(
        e.state_core().skills.contains_key(&(0, qz)),
        "25 阈值授予清正"
    );
    // 布阵：0 号两张黑桃 + 一张红桃；1 号一张梅花
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_place_suit(0, engine_native::catalog::Suit::Spade);
    e.debug_place_suit(0, engine_native::catalog::Suit::Spade);
    e.debug_place_suit(0, engine_native::catalog::Suit::Heart);
    e.debug_place_suit(1, engine_native::catalog::Suit::Club);
    assert_eq!(e.debug_hand(0).len(), 3);
    assert_eq!(e.debug_hand(1).len(), 1);
    assert!(
        advance_to_prompt(&mut e, "trigger_confirm"),
        "清正确认门（出牌阶段开始）"
    );
    let gate = e.open_choice().cloned().expect("清正确认门");
    assert_eq!(gate.spec.prompt, "trigger_confirm");
    submit_answer(&mut e, gate.id, vec![engine_native::ChoiceOption::Confirm]);
    let suit = e.open_choice().cloned().expect("花色选择窗");
    assert_eq!(suit.spec.prompt, "mbcmqingzheng_suit");
    submit_answer(&mut e, suit.id, vec![engine_native::ChoiceOption::Tag(0)]);
    let tgt = e.open_choice().cloned().expect("目标选择窗");
    assert_eq!(tgt.spec.prompt, "mbcmqingzheng_target");
    submit_answer(&mut e, tgt.id, vec![engine_native::ChoiceOption::Player(1)]);
    let tsuit = e.open_choice().cloned().expect("目标花色窗");
    assert_eq!(tsuit.spec.prompt, "mbcmqingzheng_target_suit");
    let hp1_before = e.debug_hp(1);
    submit_answer(&mut e, tsuit.id, vec![engine_native::ChoiceOption::Tag(2)]);
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), 1, "0 号弃 2 张黑桃，剩红桃");
    assert_eq!(e.debug_hand(1).len(), 0, "1 号弃 1 张梅花");
    assert_eq!(e.debug_hp(1), hp1_before - 1, "目标少弃 → 受 1 点伤害");
}

/// 清正对照：目标弃牌数不少于自己时不受伤害。
#[test]
fn qingzheng_no_damage_when_target_discards_enough() {
    let pack = bridge_pack("longxue-upstream-liuli");
    let mut e = Engine::new_with_extensions(94008, 4, &[&pack]).expect("建局");
    let def = general_def_of(&e, "mb_caomao");
    e.debug_assemble_general_def(0, def);
    e.debug_grant(0, "mbcmqingzheng");
    let qz = e.catalog().skill_by_name("mbcmqingzheng").expect("清正");
    assert!(e.state_core().skills.contains_key(&(0, qz)));
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_place_suit(0, engine_native::catalog::Suit::Spade);
    e.debug_place_suit(1, engine_native::catalog::Suit::Club);
    e.debug_place_suit(1, engine_native::catalog::Suit::Club);
    assert!(
        advance_to_prompt(&mut e, "trigger_confirm"),
        "清正确认门（出牌阶段开始）"
    );
    let gate = e.open_choice().cloned().expect("清正确认门");
    submit_answer(&mut e, gate.id, vec![engine_native::ChoiceOption::Confirm]);
    let suit = e.open_choice().cloned().expect("花色选择窗");
    submit_answer(&mut e, suit.id, vec![engine_native::ChoiceOption::Tag(0)]);
    let tgt = e.open_choice().cloned().expect("目标选择窗");
    submit_answer(&mut e, tgt.id, vec![engine_native::ChoiceOption::Player(1)]);
    let tsuit = e.open_choice().cloned().expect("目标花色窗");
    let hp1 = e.debug_hp(1);
    submit_answer(&mut e, tsuit.id, vec![engine_native::ChoiceOption::Tag(2)]);
    drain(&mut e);
    assert_eq!(e.debug_hand(1).len(), 0, "1 号弃 2 张梅花");
    assert_eq!(e.debug_hp(1), hp1, "弃牌数不少于自己 → 不受伤害");
}

/// 推进阶段直到出现指定 prompt 的选择窗（其余选择按可选放弃处理；不吞目标窗）。
fn advance_to_prompt(e: &mut Engine, prompt: &str) -> bool {
    for _ in 0..12 {
        if let Some(c) = e.open_choice() {
            if c.spec.prompt == prompt {
                return true;
            }
            drain(e);
            continue;
        }
        let actor = e.turn().current;
        let out = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        if !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ) {
            return false;
        }
    }
    e.open_choice()
        .map(|c| c.spec.prompt == prompt)
        .unwrap_or(false)
}

/// 应答指定选择（断言被接受）。
fn submit_answer(
    e: &mut Engine,
    choice_id: u64,
    selection: Vec<engine_native::ChoiceOption>,
) {
    let out = e.submit(GameCommand {
        actor: e.open_choice().map(|c| c.observer).unwrap_or(0),
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer { choice_id, selection },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
}
