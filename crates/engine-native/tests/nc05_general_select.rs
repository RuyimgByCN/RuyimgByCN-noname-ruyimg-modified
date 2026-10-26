//! NC05 产品入口：正式选将/装配（不依赖 `debug_grant`）。
//!
//! 验收：扩展包武将经 `assign_general` 装配后，其**技能集自动在册**（来自
//! 武将表而非调试授技）、体力上限按武将设定；校验拒绝非法输入且不改状态；
//! 候选列表含扩展武将；装配后对局可继续推进。

use engine_native::{Engine, GameAction, GameCommand, RejectReason};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn repo_extensions_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn swd_pack() -> engine_native::content::extension::ExtensionPack {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读 swd 包");
    content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack
}

/// 扩展包武将装配：技能集来自武将表（不是 debug_grant）。
#[test]
fn assign_general_grants_pack_skills() {
    let pack = swd_pack();
    let mut e = Engine::new_with_extensions(20261051, 4, &[&pack]).expect("建局");

    // 目录里找一个带技能的扩展武将
    let (def, name, skills) = (0..e.catalog().general_count() as u16)
        .filter_map(|d| e.catalog().general_meta(d))
        .map(|m| (m.0, m.2))
        .enumerate()
        .find(|(_, (_, skills))| !skills.is_empty())
        .map(|(i, (name, skills))| (i as u16, name, skills))
        .expect("目录含带技能武将");

    // 装配前：该武将技能不在座位的技能表（除非随机装配命中）
    e.assign_general(0, def).expect("正式选将应成功");
    let granted: Vec<bool> = skills
        .iter()
        .map(|s| {
            e.catalog()
                .skill_by_name(s)
                .map(|d| e.state_core().skills.contains_key(&(0, d)))
                .unwrap_or(false)
        })
        .collect();
    assert!(
        granted.iter().all(|g| *g),
        "武将 {name} 的技能集应全部在册: {skills:?} -> {granted:?}"
    );
    assert_eq!(
        e.state_core().player(0).max_hp as usize,
        e.catalog().general_meta(def).unwrap().3 as usize,
        "体力上限按武将设定"
    );
    assert_eq!(
        e.debug_hp(0),
        e.state_core().player(0).max_hp as i32,
        "装配后回满"
    );
}

/// 校验：未知武将/越界座位/终局后装配均被拒且不改状态。
#[test]
fn assign_general_validates_inputs() {
    let mut e = Engine::new(20261052, 4);
    let hash = e.state_hash();
    assert_eq!(
        e.assign_general(0, u16::MAX),
        Err(RejectReason::NoSuchGeneral),
        "未知武将拒绝"
    );
    assert_eq!(
        e.assign_general(99, 0),
        Err(RejectReason::UnknownActor),
        "越界座位拒绝"
    );
    assert_eq!(
        e.assign_general_named(0, "不存在的武将"),
        Err(RejectReason::NoSuchGeneral),
        "未知名拒绝"
    );
    assert_eq!(e.state_hash(), hash, "拒绝不改变状态");
}

/// 候选列表：含扩展包武将；按名字装配可选到扩展武将。
#[test]
fn general_candidates_include_pack_generals() {
    let pack = swd_pack();
    let mut e = Engine::new_with_extensions(20261053, 4, &[&pack]).expect("建局");
    let candidates = e.general_candidates();
    assert!(candidates.len() > 20, "候选含标准与扩展武将");
    let pack_names: Vec<&str> = candidates
        .iter()
        .filter_map(|d| e.catalog().general_meta(*d).map(|m| m.0))
        .filter(|n| n.starts_with("swd_") || n.contains("swd"))
        .collect();
    assert!(!pack_names.is_empty(), "候选含扩展包武将: {pack_names:?}");
    let target = pack_names[0].to_string();
    e.assign_general_named(0, &target)
        .expect("按名装配扩展武将");
    assert_eq!(
        e.catalog()
            .general_meta(e.state_core().player(0).general_slots[0].unwrap())
            .unwrap()
            .0,
        target,
        "装配结果与请求一致"
    );
}

/// 装配后对局可继续推进（装配不破坏结算栈）。
#[test]
fn match_continues_after_selection() {
    let pack = swd_pack();
    let mut e = Engine::new_with_extensions(20261054, 4, &[&pack]).expect("建局");
    e.assign_general_named(0, "swd_lingwu")
        .or_else(|_| e.assign_general(0, 0))
        .expect("装配");
    let actor = e.turn().current;
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(
            out,
            engine_native::CommandOutcome::Accepted { .. }
                | engine_native::CommandOutcome::Pending { .. }
        ),
        "装配后仍可推进: {out:?}"
    );
}

fn shared_skill_pack(global: bool) -> engine_native::content::extension::ExtensionPack {
    let mut data = serde_json::json!({
        "pack": {"name": "shared-skill-select", "version": 1, "card_prefix": "shared_"},
        "cards": [{"name":"equipment", "kind":"wuxingpan", "suit":"spade", "rank":5, "equip_skills":["shared_skill"]}],
        "skills": [{"name":"shared_skill", "mapping": if global { "host:shengke_check" } else { "host:change_card_nature" }}],
        "generals": [
            {"name":"shared-general", "faction":"qun", "gender":"male", "hp":4,
             "skills":[{"name":"shared_skill", "mapping": if global { "host:shengke_check" } else { "host:change_card_nature" }}]},
            {"name":"plain-general", "faction":"qun", "gender":"male", "hp":4, "skills":[]}
        ]
    });
    if global {
        data["pack"]["init_hook"] = serde_json::json!("wuxing_natures");
    }
    content_native_loader::bridge_pack(&data.to_string())
        .unwrap()
        .pack
}

#[test]
fn reselect_keeps_global_skill_shared_with_old_general_and_restore() {
    let pack = shared_skill_pack(true);
    let mut e = Engine::new_with_extensions(42, 4, &[&pack]).unwrap();
    let def = e.catalog().skill_by_name("shared_skill").unwrap();
    assert!(e.state_core().skills.contains_key(&(0, def)), "global init");
    e.assign_general_named(0, "shared-general").unwrap();
    let mut restored = Engine::restore_with_extensions(&e.save(), &[&pack]).unwrap();
    for engine in [&mut e, &mut restored] {
        engine.assign_general_named(0, "plain-general").unwrap();
        assert!(
            engine.state_core().skills.contains_key(&(0, def)),
            "global source survives old general removal, including restore"
        );
    }
}

#[test]
fn reselect_keeps_equipment_skill_shared_with_old_general() {
    let pack = shared_skill_pack(false);
    let mut e = Engine::new_with_extensions(42, 4, &[&pack]).unwrap();
    e.assign_general_named(0, "shared-general").unwrap();
    let card = e.debug_move_named_card_to_hand(0, "shared_equipment_0");
    e.debug_set_turn(0, engine_native::event::Phase::Play, 1);
    let result = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
            targets: vec![],
        },
    });
    assert!(
        matches!(result, engine_native::CommandOutcome::Accepted { .. }),
        "real equipment command: {result:?}"
    );
    assert!(e.debug_equips(0).contains(&card));
    e.assign_general_named(0, "plain-general").unwrap();
    let def = e.catalog().skill_by_name("shared_skill").unwrap();
    assert!(e.debug_equips(0).contains(&card));
    assert!(
        e.state_core().skills.contains_key(&(0, def)),
        "equipped source survives old general removal"
    );
    // Without a surviving global/equipment source, the former general's skill is revoked.
    e.debug_remove_equip(0, card);
    assert!(!e.state_core().skills.contains_key(&(0, def)));
}

#[test]
fn identity_lord_selection_preserves_infinite_hp_sentinel() {
    let mut pack = shared_skill_pack(false);
    pack.generals[0].max_hp = engine_native::state::INFINITE_HP;
    let mut e = Engine::try_new_identity_with_extensions(42, 4, &[&pack]).unwrap();
    e.assign_general_named(0, "shared-general").unwrap();
    assert_eq!(e.state_core().player(0).max_hp, engine_native::state::INFINITE_HP);
    assert_eq!(e.state_core().player(0).hp, i32::MAX);
}
