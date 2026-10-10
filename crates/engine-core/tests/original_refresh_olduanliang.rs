//! 第一百八十五批：refresh「断粮」（OL徐晃 olduanliang）零新原语重写。
//! 上游：将一张黑色非锦囊牌当做【兵粮寸断】使用；若当前回合未造成过伤害则无
//! 距离限制。实现：view_as bingliang play_phase black_basic_equip from hand_equip
//! （shenhua duanliang 同款源过滤器——非锦囊=基本+装备精确匹配）。
//! 近似（登记债）：条件距离 mod（本回合未造成伤害时）未建模。

use engine_api::PlayerId;
use engine_core::state::GameState;

const OLDUANLIANG: u32 = 3_305_084_926;
const OL_XUHUANG: u32 = 3_406_665_963;
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

#[test]
fn package_publishes_olduanliang_view_as_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 681);
    let view_as = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == OLDUANLIANG)
        .expect("olduanliang view_as contract");
    assert_eq!(view_as.as_kind, "bingliang");
    assert_eq!(view_as.window, "play_phase");
    assert_eq!(view_as.filter, "black_basic_equip");
    assert_eq!(view_as.source_zone.as_deref(), Some("hand_equip"));
    assert!(
        !package.trigger_index.iter().any(|t| t.skill_def_id == OLDUANLIANG),
        "olduanliang is a pure view_as skill"
    );
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == OL_XUHUANG)
        .expect("ol_xuhuang in roster");
    assert!(character.skills.contains(&OLDUANLIANG));
}

#[test]
fn contract_only_placeholder_compiles_without_triggers() {
    // 断粮无触发面：包内该技能不应产生 modules 条目（view_as 不进 trigger/modules）。
    let package = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 681);
    assert!(
        package
            .modules
            .iter()
            .all(|module| !module.name.contains("olduanliang")),
        "view_as skill must not emit trigger modules"
    );
    let _ = PlayerId::NONE;
    let _: GameState = GameState::empty();
}
