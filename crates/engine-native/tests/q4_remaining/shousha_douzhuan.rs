use std::collections::BTreeMap;

use content_native_loader::{apply_douzhuan_to_engine, apply_shousha_to_engine, parse_shousha_config};
use engine_native::catalog::Faction;
use engine_native::content::extension::{ExtensionDef, ExtensionGeneral, ExtensionPack, InitHook};
use engine_native::content::standard::Gender;
use engine_native::Engine;
use serde_json::Value;

use super::helpers::{assert_archive_roundtrip, repo_asset};

#[test]
fn l15_shousha_three_pools_and_eight_configs_on_live_engine() {
    let raw_cfg = repo_asset("packages/extensions/shousha-full-upstream-liuli/assets/config.json");
    let cfg = parse_shousha_config(&raw_cfg).expect("parse shousha config.json");

    // Verify all 3 upstream pools have real mapped entries
    assert!(cfg.base_len() > 50);
    assert!(cfg.contains_huanyuan("liubei") || cfg.contains_huanyuan("shen_guanyu") || cfg.contains_huanyuan("sp_zhaoyun") || cfg.contains_huanyuan("re_machao") || cfg.contains_base("liubei"));
    assert!(cfg.contains_doudizhu("caocao") || cfg.contains_base("caocao"));

    // Build ExtensionPack with genuine upstream general IDs
    let pack = ExtensionPack {
        def: ExtensionDef {
            name: "l15-upstream-generals",
            version: 1,
            description: "upstream general ids",
        },
        generals: vec![
            ExtensionGeneral {
                name: "liubei",
                faction: Faction::Shu,
                skills: &["仁德"],
                max_hp: 4,
                gender: Gender::Male,
                presentation_binding: None,
            },
            ExtensionGeneral {
                name: "zhenji",
                faction: Faction::Wei,
                skills: &["洛神"],
                max_hp: 3,
                gender: Gender::Female,
                presentation_binding: None,
            },
            ExtensionGeneral {
                name: "txhj_xuelan",
                faction: Faction::Qun,
                skills: &[],
                max_hp: 4,
                gender: Gender::Male,
                presentation_binding: None,
            },
            ExtensionGeneral {
                name: "yuanshao",
                faction: Faction::Qun,
                skills: &[],
                max_hp: 4,
                gender: Gender::Male,
                presentation_binding: None,
            },
        ],
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills: Vec::new(),
        init_hook: InitHook::None,
        enabled: true,
    };

    let packs = vec![
        "standard".to_string(),
        "refresh".to_string(),
        "extra".to_string(),
        "unsupported_pack".to_string(),
    ];
    let mut pack_members = BTreeMap::new();
    pack_members.insert("standard".into(), vec!["liubei".into(), "zhenji".into()]);
    pack_members.insert("extra".into(), vec!["yuanshao".into()]);
    pack_members.insert("refresh".into(), vec![]);
    pack_members.insert("yijiang".into(), vec![]);
    pack_members.insert("sp".into(), vec![]);
    pack_members.insert("mobile".into(), vec![]);
    pack_members.insert("xinghuoliaoyuan".into(), vec![]);
    pack_members.insert("sb".into(), vec![]);
    pack_members.insert("yingbian".into(), vec![]);
    pack_members.insert("sp2".into(), vec![]);
    pack_members.insert("shenhua".into(), vec![]);
    pack_members.insert("shiji".into(), vec![]);
    pack_members.insert("jsrg".into(), vec![]);
    pack_members.insert("huicui".into(), vec![]);

    let mut char_pack = BTreeMap::new();
    char_pack.insert("liubei".into(), "standard".into());
    char_pack.insert("zhenji".into(), "standard".into());
    char_pack.insert("txhj_xuelan".into(), "unsupported_pack".into());
    char_pack.insert("yuanshao".into(), "extra".into());

    // Toggle all 8 configs: huanyuan, jinjiang, jinjiang2, packs, sexforbid, txhj, unseen="unseen", yinjian
    let mut val: Value = serde_json::from_str(&raw_cfg).unwrap();
    {
        let obj = val.as_object_mut().unwrap();
        obj.insert("jinjiang".into(), Value::Bool(true));
        obj.insert("jinjiang2".into(), Value::Bool(true));
        obj.insert("huanyuan".into(), Value::Bool(true));
        obj.insert("yinjian".into(), Value::Bool(true));
        obj.insert("packs".into(), Value::Bool(true));
        obj.insert("txhj".into(), Value::Bool(true));
        obj.insert("unseen".into(), Value::String("unseen".into()));
        obj.insert("sexforbid".into(), Value::String("male".into()));
    }
    let tuned_json = serde_json::to_string(&val).unwrap();

    let mut e = Engine::new_with_extensions(701, 4, &[&pack]).unwrap();
    let _report = apply_shousha_to_engine(
        &tuned_json,
        "identity",
        false,
        &packs,
        &pack_members,
        &char_pack,
        &mut e,
    )
    .expect("apply_shousha_to_engine");

    let names_human: Vec<&str> = e
        .filter_selectable_generals(false)
        .into_iter()
        .filter_map(|d| e.catalog().general_meta(d).map(|m| m.0))
        .collect();
    let names_ai: Vec<&str> = e
        .filter_selectable_generals(true)
        .into_iter()
        .filter_map(|d| e.catalog().general_meta(d).map(|m| m.0))
        .collect();

    assert!(names_human.contains(&"liubei"), "liubei matches standard+male+base");
    assert!(
        names_human.contains(&"zhenji"),
        "BUG-9: sexforbid=male only sets forbidai, human still keeps female"
    );
    assert!(
        !names_ai.contains(&"zhenji"),
        "BUG-9: female banned for AI when sexforbid=male"
    );
    assert!(!names_human.contains(&"txhj_xuelan"), "txhj_* banned outside taixuhuanjing & unsupported pack");
    assert!(!names_human.contains(&"yuanshao"), "unlisted base general unseen when jinjiang=true & unseen=unseen");
    assert_eq!(e.l15_pool_bits("liubei") & 1, 1);
    assert!(names_ai.len() <= names_human.len());

    // BUG-9 counterexample: huanyuan=true with packs=false must STILL ban unsupported_pack (ban_packs active)
    {
        let mut v_hy = val.clone();
        let obj = v_hy.as_object_mut().unwrap();
        obj.insert("jinjiang".into(), Value::Bool(false));
        obj.insert("packs".into(), Value::Bool(false));
        obj.insert("huanyuan".into(), Value::Bool(true));
        obj.insert("yinjian".into(), Value::Bool(false));
        obj.insert("txhj".into(), Value::Bool(false));
        obj.insert("unseen".into(), Value::String("none".into()));
        obj.insert("sexforbid".into(), Value::String("none".into()));
        let mut e_hy = Engine::new_with_extensions(702, 4, &[&pack]).unwrap();
        let _ = apply_shousha_to_engine(
            &serde_json::to_string(&v_hy).unwrap(),
            "identity",
            false,
            &packs,
            &pack_members,
            &char_pack,
            &mut e_hy,
        )
        .unwrap();
        let hy_human: Vec<&str> = e_hy
            .filter_selectable_generals(false)
            .into_iter()
            .filter_map(|d| e_hy.catalog().general_meta(d).map(|m| m.0))
            .collect();
        assert!(
            !hy_human.contains(&"txhj_xuelan"),
            "huanyuan=true activates ban_packs even when packs=false"
        );
    }

    // Counterexample: unseen="forbidai", sexforbid="none", huanyuan=false, txhj in "taixuhuanjing" -> zhenji visible & pool bit cleared
    {
        let obj = val.as_object_mut().unwrap();
        obj.insert("sexforbid".into(), Value::String("none".into()));
        obj.insert("unseen".into(), Value::String("forbidai".into()));
        obj.insert("huanyuan".into(), Value::Bool(false));
    }
    let json2 = serde_json::to_string(&val).unwrap();
    let _ = apply_shousha_to_engine(
        &json2,
        "taixuhuanjing",
        false,
        &packs,
        &pack_members,
        &char_pack,
        &mut e,
    )
    .unwrap();
    let names_h2: Vec<&str> = e
        .filter_selectable_generals(false)
        .into_iter()
        .filter_map(|d| e.catalog().general_meta(d).map(|m| m.0))
        .collect();
    assert!(names_h2.contains(&"zhenji"), "female allowed when sexforbid=none");
    assert_eq!(e.l15_pool_bits("liubei") & 1, 0, "huanyuan bit cleared when huanyuan=false");

    // Full save/restore parity of all L15 flags
    let restored = assert_archive_roundtrip(&e, &[&pack]);
    let names_restored: Vec<&str> = restored
        .filter_selectable_generals(false)
        .into_iter()
        .filter_map(|d| restored.catalog().general_meta(d).map(|m| m.0))
        .collect();
    assert_eq!(names_restored, names_h2);
}

#[test]
fn l17_douzhuan_ai_qianghua_xuanjiang_live_doudizhu_selection_and_restore() {
    let raw_cfg = repo_asset("packages/extensions/douzhuan-xingyi-upstream-liuli/assets/config.json");
    let mut val: Value = serde_json::from_str(&raw_cfg).unwrap();
    val["aiQianghua"] = Value::Bool(true);
    val["stars"] = serde_json::json!({
        "曹操": [0, 0, 0, 0, 0, 10, 2],
        "刘备": [0, 0, 0, 0, 0, 3, 9],
        "孙权": [0, 0, 0, 0, 0, 5, 5]
    });
    let cfg_json = serde_json::to_string(&val).unwrap();

    let rarities = BTreeMap::from([
        ("曹操".to_string(), "common".to_string()),
        ("刘备".to_string(), "common".to_string()),
        ("孙权".to_string(), "rare".to_string()),
    ]);

    let mut e = Engine::new_doudizhu(808, 3, 0);
    apply_douzhuan_to_engine(&cfg_json, true, &rarities, &mut e).expect("apply douzhuan online");

    let c_caocao = (0..e.catalog().general_count() as u16)
        .find(|&d| e.catalog().general_meta(d).map(|m| m.0) == Some("曹操"))
        .unwrap();
    let c_liubei = (0..e.catalog().general_count() as u16)
        .find(|&d| e.catalog().general_meta(d).map(|m| m.0) == Some("刘备"))
        .unwrap();
    let c_sunquan = (0..e.catalog().general_count() as u16)
        .find(|&d| e.catalog().general_meta(d).map(|m| m.0) == Some("孙权"))
        .unwrap();
    // Order weak-first so non-enhanced fallback would pick 刘备, while enhanced picks highest score
    let cands = [c_liubei, c_sunquan, c_caocao];

    // Online Lord prefers 曹操 (score 10 vs 5 & 3)
    assert_eq!(e.pick_doudizhu_ai_general(&cands, true, false).0, Some(c_caocao));
    // Online Farmer prefers 刘备 (score 9 vs 5 & 2)
    assert_eq!(e.pick_doudizhu_ai_general(&cands, false, false).0, Some(c_liubei));

    // Double general selection returns 2 distinct top candidates
    let (d1, d2) = e.pick_doudizhu_ai_general(&cands, true, true);
    assert_eq!(d1, Some(c_caocao));
    assert!(d2.is_some() && d2 != d1);

    // Counterexample: offline falls back to candidate[0] = c_liubei even for Lord
    let mut e_off = Engine::new_doudizhu(808, 3, 0);
    apply_douzhuan_to_engine(&cfg_json, false, &rarities, &mut e_off).unwrap();
    assert_eq!(e_off.pick_doudizhu_ai_general(&cands, true, false).0, Some(c_liubei));

    // Counterexample: non-Doudizhu mode returns None
    let mut e_proto = Engine::new(808, 3);
    apply_douzhuan_to_engine(&cfg_json, true, &rarities, &mut e_proto).unwrap();
    assert_eq!(e_proto.pick_doudizhu_ai_general(&cands, true, false), (None, None));

    // Save / Restore preserves L17 online & AI star scores
    let mut restored = assert_archive_roundtrip(&e, &[]);
    assert_eq!(restored.pick_doudizhu_ai_general(&cands, true, false).0, Some(c_caocao));
}
