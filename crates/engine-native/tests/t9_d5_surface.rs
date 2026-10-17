//! T9 / D5：Boss 诸葛亮与 ylyg* 不能装成“已支持的空操作”。
//!
//! 上游体力是 Infinity（2026-10-17 起装载器接受为无限体力哨兵
//! `INFINITE_HP`；其余非整数/越界值仍拒绝），四技与 ylyg* 没有宿主行为键：
//! mapping 缺省的技能计为暂缓，而不是写进武将技能表。

use content_native_loader::{bridge_generals, BridgeError};
use engine_native::state::INFINITE_HP;

fn general(hp: &str, skills: &str) -> String {
    format!(
        r#"{{
  "generals": [{{
    "name": "boss_zhugeliang",
    "faction": "shu",
    "gender": "male",
    "hp": {hp},
    "skills": [{skills}]
  }}]
}}"#
    )
}

fn skill(name: &str, mapping: &str) -> String {
    format!(r#"{{"name":"{name}","mapping":{mapping}}}"#)
}

/// `"Infinity"` 接受为无限体力哨兵（D5 前置原语）。
#[test]
fn t9_infinity_hp_maps_to_sentinel() {
    let json = general(r#""Infinity""#, &skill("xiangxing", "null"));
    let out = bridge_generals(&json).expect("Infinity 应被接受");
    assert_eq!(out.generals.len(), 1);
    assert_eq!(out.generals[0].max_hp, INFINITE_HP, "无限体力哨兵");
    // 数字 2147483647 也走同一哨兵（避免包数据两种写法分叉）。
    let numeric = general("2147483647", &skill("xiangxing", "null"));
    assert_eq!(
        bridge_generals(&numeric).expect("哨兵数值").generals[0].max_hp,
        INFINITE_HP
    );
}

#[test]
fn t9_invalid_hp_values_are_rejected() {
    for hp in ["null", "0", "0.5", "256", r#""infinity""#, r#""∞""#] {
        let json = general(hp, &skill("xiangxing", "null"));
        let err = match bridge_generals(&json) {
            Ok(_) => panic!("{hp} 不应被接受"),
            Err(e) => e,
        };
        match err {
            BridgeError::InvalidField { field, .. } | BridgeError::MissingField(field) => {
                assert!(
                    field.contains("hp"),
                    "{hp} 应落在体力字段，实际 {err}"
                );
            }
            other => panic!("{hp} 不应被接受: {other}"),
        }
    }
}

#[test]
fn t9_zhuge_and_ylyg_skills_stay_unmapped() {
    let skills = [
        "xiangxing",
        "yueyin",
        "fengqi",
        "gaiming",
        "ylygxiangxing",
        "ylyglianhuan",
        "ylygniepan",
        "ylyg_jiang",
        "ylygguhuo",
        "ylygtianxiang",
    ]
    .into_iter()
    .map(|name| skill(name, "null"))
    .collect::<Vec<_>>()
    .join(",");
    let bridged = bridge_generals(&general("4", &skills)).expect("整数体力可装载");
    assert!(bridged.host_skills.is_empty(), "暂缓技能不得变成宿主行为");
    assert_eq!(bridged.unmapped_skills, 10);
    assert!(bridged.generals[0].skills.is_empty(), "暂缓技能不得写入武将技能表");
    assert_eq!(bridged.generals[0].max_hp, 4);
}

#[test]
fn t9_unknown_mapping_is_not_a_noop() {
    let json = general("4", &skill("xiangxing", r#""xiangxing""#));
    let err = match bridge_generals(&json) {
        Ok(_) => panic!("未知行为键不应被接受"),
        Err(e) => e,
    };
    assert!(
        matches!(err, BridgeError::UnknownBehaviorKey(ref k) if k == "xiangxing"),
        "{err}"
    );
}

// ---------------------------------------------------------------------------
// 无限体力（上游 Infinity）内核语义：装载 → 目录 → 对局。
// ---------------------------------------------------------------------------

/// 扩展武将 def 索引（标准表之后为扩展段）。
fn general_def_of(e: &engine_native::Engine, name: &str) -> u16 {
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

/// 无限体力：伤害/回复都不改变体力（hp 恒等于 max_hp，已受伤判据恒假），
/// 永不进入濒死；普通角色同场景照常受伤阵亡（对照）。
#[test]
fn infinite_hp_survives_damage_and_recovery() {
    use engine_native::event::DamageNature;
    use engine_native::{CommandOutcome, Engine};

    let json = format!(
        r#"{{
  "source": "test",
  "upstream_path": "test/extension.js",
  "pack": {{"name": "d5-infinite-hp", "version": 1, "description": "无限体力样例", "card_prefix": "t_", "init_hook": null}},
  "cards": [],
  "generals": [{{"name": "boss_zhugeliang", "faction": "shu", "gender": "male", "hp": "Infinity", "skills": []}}],
  "skills": []
}}"#
    );
    let bridged = content_native_loader::bridge_pack(&json).expect("装载无限体力武将");
    let pack = bridged.pack;
    let mut e = Engine::new_with_extensions(91001, 4, &[&pack]).expect("建局");
    let def = general_def_of(&e, "boss_zhugeliang");
    assert_eq!(
        e.catalog().general_meta(def).map(|g| g.3),
        Some(INFINITE_HP),
        "目录体力=哨兵"
    );
    e.debug_set_general_def(0, def);
    {
        let core = e.state_core_mut();
        core.players[0].max_hp = INFINITE_HP;
        core.players[0].hp = INFINITE_HP as i32;
    }
    assert!(e.state_core().player(0).infinite_hp(), "哨兵判定");
    assert_eq!(
        e.state_core().hand_limit(e.catalog(), 0) > 1_000_000,
        true,
        "手牌上限按哨兵基数（等效无上限）"
    );

    // 伤害照常结算但不改体力；不进入濒死。
    let out = e.debug_deal_damage(Some(1), 0, 3, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    assert_eq!(e.debug_hp(0), INFINITE_HP as i32, "无限体力不受伤害");
    assert_eq!(
        e.state_core().player(0).max_hp,
        INFINITE_HP,
        "上限不变（已受伤判据恒假）"
    );
    assert!(e.debug_alive(0) && !e.state_core().player(0).dying, "不进入濒死");

    // 回复也不改体力。
    let _ = e.debug_recover(0, 5);
    assert_eq!(e.debug_hp(0), INFINITE_HP as i32, "无限体力不回血");

    // 对照：普通角色（体力上限 4）同场景照常受伤阵亡。
    e.debug_set_max_hp(1, 4);
    e.debug_set_hp(1, 1);
    let out = e.debug_deal_damage(Some(0), 1, 3, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    for _ in 0..8 {
        if e.open_choice().is_none() {
            break;
        }
        let c = e.open_choice().cloned().unwrap();
        let sel: Vec<engine_native::ChoiceOption> = if c.spec.min == 0 {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = e.submit(engine_native::GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: 10_000 + c.id,
            action: engine_native::GameAction::Answer { choice_id: c.id, selection: sel },
        });
    }
    assert!(!e.debug_alive(1), "对照：普通角色受伤阵亡");

    // 视图暴露哨兵（宿主映射为 ∞）；存档往返保持。
    let view = e.view_for(0);
    assert_eq!(view.players[0].hp, INFINITE_HP as i32, "投影体力=哨兵");
    assert_eq!(view.players[0].max_hp, INFINITE_HP, "投影上限=哨兵");
    let snap = e.save();
    assert_eq!(snap.core.players[0].max_hp, INFINITE_HP);
    let restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    assert_eq!(restored.debug_hp(0), INFINITE_HP as i32, "恢复后仍无限");
    assert!(restored.state_core().player(0).infinite_hp());

    // 规划器（AI）面对哨兵体力不溢出、结果确定。
    let planner = engine_native::ai::BoundedPlanner::new(16);
    let plan = restored.plan_action(0, &planner);
    let again = restored.plan_action(0, &planner);
    assert_eq!(format!("{:?}", plan.action), format!("{:?}", again.action));
    assert!(!plan.cancelled);
}
