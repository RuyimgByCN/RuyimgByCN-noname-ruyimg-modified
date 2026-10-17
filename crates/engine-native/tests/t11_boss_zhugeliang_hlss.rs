//! T11 Boss 诸葛亮族（hlss 星系统批 115）：禳星 + 隐月 + 隐月·判。
//!
//! 上游定义在 liuli `character/extra.js`（`hlss_xiangxing` / `hlss_yueyin` /
//! `hlss_yueyin2`，Boss `boss_zhugeliang` 四技之二），配合无限体力哨兵构成
//! Boss 诸葛亮规则面。本文件用测试包装载三技的行为映射并逐面回归。

use engine_native::event::DamageNature;
use engine_native::state::INFINITE_HP;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};

fn test_pack_json() -> String {
    r#"{
  "source": "test",
  "upstream_path": "test/extension.js",
  "pack": {"name": "hlss-test", "version": 1, "description": "hlss 星系统样例", "card_prefix": "t_", "init_hook": null},
  "cards": [],
  "generals": [{
    "name": "boss_zhugeliang",
    "faction": "shu",
    "gender": "male",
    "hp": 4,
    "skills": [
      {"name": "hlss_xiangxing", "mapping": "host:hlss_xiangxing"},
      {"name": "hlss_yueyin", "mapping": "host:hlss_yueyin"},
      {"name": "hlss_yueyin2", "mapping": "host:hlss_yueyin2"}
    ]
  }],
  "skills": []
}"#
    .to_string()
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

/// 建局并把 `boss_zhugeliang` 装到 0 号（3 星、4 体力、隐月在位）。
fn setup(seed: u64) -> Engine {
    let json = test_pack_json();
    let bridged = content_native_loader::bridge_pack(&json).expect("装载 hlss 测试包");
    let pack = bridged.pack;
    let mut e = Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局");
    let def = general_def_of(&e, "boss_zhugeliang");
    e.debug_assemble_general_def(0, def);
    e
}

/// 应答所有开放选择（取首 min 项；可取消取空）直到静止。
fn drain(e: &mut Engine) {
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel: Vec<ChoiceOption> = if c.spec.min == 0 {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: 5000 + c.id,
            action: GameAction::Answer { choice_id: c.id, selection: sel },
        });
    }
}

/// 装配初始化：3 星、计数 0、无受伤记录；隐月 +1 与禳星光回满并存。
#[test]
fn hlss_assembly_initializes_stars_and_light_refill() {
    let mut e = setup(92001);
    assert_eq!(e.state_core().player(0).hlss_stars, 3, "装配 3 星");
    assert_eq!(e.state_core().player(0).hlss_count, 0);
    assert_eq!(e.state_core().player(0).hlss_damage, 0);
    let xd = e
        .catalog()
        .skill_by_name("hlss_xiangxing")
        .expect("禳星在目录");
    let yd = e
        .catalog()
        .skill_by_name("hlss_yueyin")
        .expect("隐月在目录");
    assert!(e.state_core().skills.contains_key(&(0, xd)), "禳星实例");
    assert!(e.state_core().skills.contains_key(&(0, yd)), "隐月实例");

    // 隐月：1 点伤害 +1 = 2 点；禳星光：体力回满（受 2 点仍 4/4）。
    let out = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(0), 4, "光：受 2 点伤害后回满");
    assert_eq!(e.state_core().player(0).hlss_count, 2, "计数按结算值累加（1+隐月 1）");
    assert_eq!(e.state_core().player(0).hlss_stars, 3, "未满 3 不失星");

    // 再受 1 点 → 计数 4 ≥ 3 → 失星（3→2）并分配 3 点雷伤给其他角色。
    let hp_before: Vec<i32> = (1..4).map(|p| e.debug_hp(p)).collect();
    let out = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hlss_stars, 2, "失一星");
    assert_eq!(e.state_core().player(0).hlss_count, 0, "失星清零计数");
    let hp_after: Vec<i32> = (1..4).map(|p| e.debug_hp(p)).collect();
    let dealt: i32 = hp_before
        .iter()
        .zip(hp_after.iter())
        .map(|(a, b)| a - b)
        .sum();
    assert_eq!(dealt, 3, "雷伤分配共 3 点（随机 1 + 随机敌 2）");
    assert!(hp_after.iter().zip(hp_before.iter()).any(|(a, b)| a < b), "至少一人受伤");
    assert_eq!(e.debug_hp(0), 4, "禳星持有者不吃自己的雷伤");
}

/// 隐月免除：当前星数对应伤害类型已在记录中 → 不失星、不放雷。
#[test]
fn hlss_yueyin_skip_prevents_star_loss() {
    let mut e = setup(92002);
    // 先受雷伤：记录雷位；计数 2（1 + 隐月 1）
    let out = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Thunder);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(
        e.state_core().player(0).hlss_damage & 1,
        1,
        "记录雷属性伤害"
    );
    assert_eq!(e.state_core().player(0).hlss_count, 2);
    // 再受普通伤害：计数 4 ≥ 3 → 三星等级要求雷记录 → 免除
    let hp_before: Vec<i32> = (1..4).map(|p| e.debug_hp(p)).collect();
    let out = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hlss_stars, 3, "免除：不失星");
    assert_eq!(e.state_core().player(0).hlss_count, 4, "免除：计数保留");
    let hp_after: Vec<i32> = (1..4).map(|p| e.debug_hp(p)).collect();
    assert_eq!(hp_before, hp_after, "免除：不放雷");
    // 无隐月时不免除（移除隐月实例后同样流程照常失星）。
    let yd = e.catalog().skill_by_name("hlss_yueyin").expect("隐月");
    e.state_core_mut().skills.remove(&(0, yd));
    let out = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hlss_stars, 2, "无隐月：照常失星");
}

/// 星归零后觉醒：不再回满体力，伤害可致死；记录位图在失星时清空。
#[test]
fn hlss_zero_stars_awakens_and_stops_refill() {
    let mut e = setup(92003);
    // 三星耗尽：每次 3 点（1 + 隐月 1 + 2 点补足）
    for _ in 0..3 {
        // 普通伤害：三星等级要求雷记录、二星要求火、一星要求锦囊——普通
        // 伤害不记入任何一位，故三次都照常失星。
        let out = e.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
        assert!(
            matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
            "{out:?}"
        );
        drain(&mut e);
    }
    assert_eq!(e.state_core().player(0).hlss_stars, 0, "三星耗尽");
    assert_eq!(e.state_core().player(0).hlss_damage, 0, "失星清空记录");
    let xd = e
        .catalog()
        .skill_by_name("hlss_xiangxing")
        .expect("禳星");
    assert!(
        !e.state_core()
            .skills
            .get(&(0, xd))
            .map(|i| i.enabled)
            .unwrap_or(false),
        "觉醒：禳星禁用"
    );
    // 觉醒后不再回满：2 点伤害 + 隐月 1 = 3 → 体力 4→1
    let out = e.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(0), 1, "觉醒后伤害照常（不再回满）");
}

/// 隐月·判：不能被翻面；判定区的牌对持有者不生效（乐不思蜀不跳阶段）。
#[test]
fn hlss_yueyin2_blocks_turnover_and_voids_delayed_trick() {
    let mut e = setup(92004);
    // 翻面被取消：debug_flip 返回是否发生翻面
    assert!(!e.debug_flip(0), "隐月·判：翻面被取消");
    assert!(!e.state_core().player(0).face_down, "未翻面");
    // 移除隐月·判后照常翻面（对照）
    let y2 = e
        .catalog()
        .skill_by_name("hlss_yueyin2")
        .expect("隐月·判");
    e.state_core_mut().skills.remove(&(0, y2));
    assert!(e.debug_flip(0), "对照：无隐月·判照常翻面");
    e.debug_set_face_down(0, false);
    // 复位隐月·判实例并验证判定区牌不生效
    e.debug_grant(0, "hlss_yueyin2");
    e.debug_place_judge(0, "le");
    // 判定牌设为黑桃（非红桃 → 照常会跳过错开阶段）
    e.debug_set_deck_top_by(|d| d.suit == engine_native::catalog::Suit::Spade);
    // 进入 0 号回合的判定阶段：把回合交给 3 号并结束其回合，NextPhase
    // 使回合转回 0 号 → 回合开始 → Judge 阶段结算判定区延时锦囊。
    e.debug_set_turn(3, Phase::End, e.turn().round);
    drain(&mut e);
    let out = e.submit(GameCommand {
        actor: 3,
        expected_revision: e.revision(),
        dedup_id: 40001,
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert!(
        e.debug_judge(0).is_empty(),
        "判定区延时锦囊已结算（隐月·判：判定不生效）"
    );
    assert!(
        !e.state_core().turn.skip_play_phase,
        "隐月·判：乐不思蜀判定不生效（不跳阶段）"
    );
    // 对照：移除隐月·判后同一判定照常跳阶段
    let y2 = e.catalog().skill_by_name("hlss_yueyin2").expect("隐月·判");
    e.state_core_mut().skills.remove(&(0, y2));
    e.debug_place_judge(0, "le");
    e.debug_set_deck_top_by(|d| d.suit == engine_native::catalog::Suit::Spade);
    e.debug_set_turn(3, Phase::End, e.turn().round);
    drain(&mut e);
    let out = e.submit(GameCommand {
        actor: 3,
        expected_revision: e.revision(),
        dedup_id: 40002,
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert!(
        e.debug_judge(0).is_empty(),
        "对照：判定区延时锦囊已结算"
    );
    assert!(
        e.state_core().turn.skip_play_phase,
        "对照：无隐月·判照常跳阶段"
    );
}

/// 无限体力哨兵与星系统并存（Boss 诸葛亮族完整形态）：体力不变、星照常掉。
#[test]
fn hlss_with_infinite_hp_keeps_stars() {
    let mut e = setup(92005);
    {
        let core = e.state_core_mut();
        core.players[0].max_hp = INFINITE_HP;
        core.players[0].hp = INFINITE_HP as i32;
    }
    for _ in 0..3 {
        let out = e.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
        assert!(
            matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
            "{out:?}"
        );
        drain(&mut e);
    }
    assert_eq!(e.debug_hp(0), INFINITE_HP as i32, "无限体力不变");
    assert_eq!(e.state_core().player(0).hlss_stars, 0, "星照常耗尽");
}
