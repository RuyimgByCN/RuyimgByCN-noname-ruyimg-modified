//! G1 旧包转换生成器（ADR 0776）：把两仓 cardpile 扩展转换为
//! `discover_extensions` 可发现的真实包（manifest.json + package.npack.json
//! + assets/pile.json），写入仓库 `packages/extensions/`。
//!
//! 数据来源（逐字节核对自上游工作区）：
//! - noname      `apps/core/extension/cardpile/extension.js` @ 5c02a745
//! - noname-liuli `noname/extension/cardpile/extension.js`  @ 8485dbb2
//! 两仓 data 表语义一致（仅 JS 语法差异），各保留独立来源身份。
//!
//! 运行：`cargo run --offline -p content-runtime --example gen_cardpile_packages`

use blake3::hash;
use content_schema::{
    ContentPackage, PackageManifest,
};
use resource_schema::{AssetEntry, AssetKind, AssetManifest, PreloadPriority};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// 上游 cardpile 比例表（两仓一致；牌名 = 上游 data 键）。
const ENTRIES: &[(&str, &[(&str, u32)])] = &[
    ("sha", &[("diamond", 6), ("club", 14), ("heart", 3), ("spade", 7)]),
    ("huosha", &[("diamond", 2), ("heart", 3)]),
    ("leisha", &[("spade", 5), ("club", 4)]),
    ("shan", &[("heart", 6), ("diamond", 18)]),
    ("jiu", &[("diamond", 1), ("spade", 2), ("club", 2)]),
    ("tao", &[("heart", 9), ("diamond", 3)]),
    ("wanjian", &[("heart", 1)]),
    ("nanman", &[("spade", 2), ("club", 1)]),
    ("guohe", &[("spade", 3), ("club", 2), ("heart", 1)]),
    ("shunshou", &[("spade", 3), ("diamond", 2)]),
    ("wuxie", &[("heart", 2), ("diamond", 1), ("spade", 2), ("club", 2)]),
    ("tiesuo", &[("spade", 2), ("club", 4)]),
];
const TOTAL: u32 = 160;

fn pile_json(source: &str, upstream_path: &str, upstream_commit: &str) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!("  \"source\": \"{source}\",\n"));
    out.push_str(&format!("  \"upstream_path\": \"{upstream_path}\",\n"));
    out.push_str(&format!("  \"upstream_commit\": \"{upstream_commit}\",\n"));
    out.push_str(&format!("  \"total\": {TOTAL},\n"));
    out.push_str("  \"entries\": [\n");
    for (idx, (card, counts)) in ENTRIES.iter().enumerate() {
        let mut line = String::from("    {\"card\": \"");
        line.push_str(card);
        line.push_str("\", \"counts\": {");
        for (j, (suit, n)) in counts.iter().enumerate() {
            if j > 0 {
                line.push_str(", ");
            }
            line.push_str(&format!("\"{suit}\": {n}"));
        }
        line.push_str("}}");
        if idx + 1 < ENTRIES.len() {
            line.push(',');
        }
        out.push_str(&line);
        out.push('\n');
    }
    out.push_str("  ]\n}\n");
    out
}

fn build_package(namespace: &str, source: &str, upstream_path: &str, upstream_commit: &str) -> ContentPackage {
    let mut package = ContentPackage {
        manifest: PackageManifest::new(namespace, "cardpile", "1.0.0"),
        modules: vec![],
        skill_index: vec![],
        character_index: vec![],
        skin_index: Default::default(),
        card_index: vec![],
        trigger_index: vec![],
        modifier_index: vec![],
        active_index: vec![],
        view_as_index: vec![],
        translations: BTreeMap::from([(
            "zh-CN".to_string(),
            BTreeMap::from([
                ("jiu".to_string(), "酒".to_string()),
                ("leisha".to_string(), "雷杀".to_string()),
                ("huosha".to_string(), "火杀".to_string()),
                ("cardpile".to_string(), "牌堆".to_string()),
            ]),
        )]),
        asset_manifest: None,
        asset_paths: BTreeMap::new(),
        compiler_version: "gen-1".into(),
    };
    let pile = pile_json(source, upstream_path, upstream_commit);
    let mut assets = AssetManifest::new("cardpile");
    assets.add_entry(AssetEntry {
        asset_id: "cardpile:pile".into(),
        source_hash: *hash(pile.as_bytes()).as_bytes(),
        kind: AssetKind::Data,
        image_variants: vec![],
        audio_variants: vec![],
        priority: PreloadPriority::Lazy,
        depends_on: vec![],
        platforms: vec![],
    });
    package.asset_manifest = Some(assets);
    package
        .asset_paths
        .insert("cardpile:pile".into(), "assets/pile.json".into());
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package
}

/// 欢乐卡牌（G1 第二样本）的卡牌与宿主技能声明
///（核对自 `apps/core/extension/欢乐卡牌/card/{list,card,skill}.js` @ 5c02a745；
/// monkey/xuelunyang/mianju 行为需事件取消/技能窃取/翻面原语，暂缓并登记）。
const HUANLE_CARDS: &[(&str, &str, u32, (&str, u8))] = &[
    // (卡名, 原生牌种, 武器射程, (花色, 点数))
    ("kuwu", "weapon", 2, ("spade", 6)), // attackFrom -1 = 攻击范围 +1
    ("shoulijian", "shoulijian", 0, ("diamond", 4)),
    ("shoulijian", "shoulijian", 0, ("spade", 4)),
    // 猴子（E03）：equip5 宝物，猴子偷桃原语的独立载荷
    ("monkey", "monkey", 0, ("heart", 8)),
    // 写轮眼（E03）：equip5 宝物，技能窃取原语的独立载荷（上游 heart 2）
    ("xuelunyang", "xuelunyang", 0, ("heart", 2)),
    // 漩涡面具（E03）：equip5 宝物，翻面取消锁定技的独立载荷
    ("mianju", "mianju", 0, ("spade", 1)),
];
const HUANLE_SKILLS: &[(&str, &str)] = &[
    ("huanle_kuwu", "sha_damage_force_discard"),
    ("huanle_jiuwei", "phase_end_recover_or_draw"),
];

fn huanle_cards_json() -> String {
    let mut out = String::from("{\n");
    out.push_str("  \"source\": \"noname\",\n");
    out.push_str("  \"upstream_path\": \"apps/core/extension/欢乐卡牌/\",\n");
    out.push_str("  \"upstream_commit\": \"5c02a745eb33b61b2e61e1167e1b71ca163911f6\",\n");
    out.push_str("  \"pack\": { \"name\": \"huanle-upstream-noname\", \"version\": 1, \"description\": \"G1 欢乐卡牌\", \"card_prefix\": \"hnl_\" },\n");
    out.push_str("  \"cards\": [\n");
    for (idx, (name, kind, range, (suit, rank))) in HUANLE_CARDS.iter().enumerate() {
        let mut line = String::from("    {\"name\": \"");
        line.push_str(name);
        line.push_str("\", \"kind\": \"");
        line.push_str(kind);
        line.push_str("\"");
        if *range > 0 {
            // 仅武器栏牌种声明射程（装载器按通用武器建模）
            line.push_str(&format!(", \"range\": {range}"));
        }
        if *name == "kuwu" {
            // 苦无声明装备技能（G1 生命周期：装备时授予、离场撤销）
            line.push_str(", \"equip_skills\": [\"huanle_kuwu\"]");
        }
        line.push_str(&format!(", \"suit\": \"{suit}\", \"rank\": {rank}}}"));
        if idx + 1 < HUANLE_CARDS.len() {
            line.push(',');
        }
        out.push_str(&line);
        out.push('\n');
    }
    out.push_str("  ],\n");
    out.push_str("  \"skills\": [\n");
    for (idx, (name, behavior)) in HUANLE_SKILLS.iter().enumerate() {
        let mut line = String::from("    {\"name\": \"");
        line.push_str(name);
        line.push_str("\", \"mapping\": \"host:");
        line.push_str(behavior);
        line.push_str("\"}");
        if idx + 1 < HUANLE_SKILLS.len() {
            line.push(',');
        }
        out.push_str(&line);
        out.push('\n');
    }
    out.push_str("  ]\n}\n");
    out
}

fn build_huanle_package() -> ContentPackage {
    let mut package = ContentPackage {
        manifest: PackageManifest::new("upstream-noname", "huanle-cards", "1.0.0"),
        modules: vec![],
        skill_index: vec![],
        character_index: vec![],
        skin_index: Default::default(),
        card_index: vec![],
        trigger_index: vec![],
        modifier_index: vec![],
        active_index: vec![],
        view_as_index: vec![],
        translations: BTreeMap::from([(
            "zh-CN".to_string(),
            BTreeMap::from([
                ("kuwu".to_string(), "苦无".to_string()),
                ("shoulijian".to_string(), "手里剑".to_string()),
                ("jiuwei".to_string(), "酒尾".to_string()),
                ("huanle".to_string(), "欢乐卡牌".to_string()),
            ]),
        )]),
        asset_manifest: None,
        asset_paths: BTreeMap::new(),
        compiler_version: "gen-1".into(),
    };
    let cards_json = huanle_cards_json();
    let mut assets = AssetManifest::new("huanle-cards");
    assets.add_entry(AssetEntry {
        asset_id: "huanle-cards:cards".into(),
        source_hash: *hash(cards_json.as_bytes()).as_bytes(),
        kind: AssetKind::Data,
        image_variants: vec![],
        audio_variants: vec![],
        priority: PreloadPriority::Lazy,
        depends_on: vec![],
        platforms: vec![],
    });
    package.asset_manifest = Some(assets);
    package
        .asset_paths
        .insert("huanle-cards:cards".into(), "assets/cards.json".into());
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package
}

/// wuxing（G1 第三样本，liuli 来源）配置数据
///（核对自 `noname/extension/wuxing/extension.js` @ 8485dbb2：
/// 每角色随机属性；牌堆 1/3 概率随机属性；五行盘 equip5 @ 黑桃5）。
const WUXING_CONFIG_JSON: &str = r#"{
  "source": "noname-liuli",
  "upstream_path": "noname/extension/wuxing/extension.js",
  "upstream_commit": "8485dbb29ff16f9b258a766b1a17ef75f883aade",
  "nature_fraction": { "numerator": 1, "denominator": 3 },
  "natures": ["metal", "wood", "water", "fire", "soil"],
  "wuxingpan": { "suit": "spade", "rank": 5 }
}
"#;

/// wuxing 卡牌装载文件（ADR 0778 自描述）：五行盘 + 生克/盘属性改写
/// 两个宿主技能声明 + 开局随机属性钩子。
const WUXING_CARDS_JSON: &str = r#"{
  "source": "noname-liuli",
  "upstream_path": "noname/extension/wuxing/extension.js",
  "upstream_commit": "8485dbb29ff16f9b258a766b1a17ef75f883aade",
  "pack": { "name": "wuxing-upstream-liuli", "version": 1, "description": "G1 wuxing：五行属性与生克", "card_prefix": "wxl_", "init_hook": "wuxing_natures" },
  "cards": [
    { "name": "wuxingpan", "kind": "wuxingpan", "suit": "spade", "rank": 5 }
  ],
  "skills": [
    { "name": "wuxing_shengke", "mapping": "host:shengke_check" },
    { "name": "wuxingpan_skill", "mapping": "host:change_card_nature" }
  ]
}
"#;

fn build_wuxing_package() -> ContentPackage {
    let mut package = ContentPackage {
        manifest: PackageManifest::new("upstream-liuli", "wuxing", "1.0.0"),
        modules: vec![],
        skill_index: vec![],
        character_index: vec![],
        skin_index: Default::default(),
        card_index: vec![],
        trigger_index: vec![],
        modifier_index: vec![],
        active_index: vec![],
        view_as_index: vec![],
        translations: BTreeMap::from([(
            "zh-CN".to_string(),
            BTreeMap::from([
                ("metal".to_string(), "金".to_string()),
                ("wood".to_string(), "木".to_string()),
                ("water".to_string(), "水".to_string()),
                ("fire".to_string(), "火".to_string()),
                ("soil".to_string(), "土".to_string()),
                ("wuxingpan".to_string(), "五行盘".to_string()),
                ("wuxingpan_skill".to_string(), "五行".to_string()),
            ]),
        )]),
        asset_manifest: None,
        asset_paths: BTreeMap::new(),
        compiler_version: "gen-1".into(),
    };
    let mut assets = AssetManifest::new("wuxing");
    assets.add_entry(AssetEntry {
        asset_id: "wuxing:config".into(),
        source_hash: *hash(WUXING_CONFIG_JSON.as_bytes()).as_bytes(),
        kind: AssetKind::Data,
        image_variants: vec![],
        audio_variants: vec![],
        priority: PreloadPriority::Lazy,
        depends_on: vec![],
        platforms: vec![],
    });
    assets.add_entry(AssetEntry {
        asset_id: "wuxing:cards".into(),
        source_hash: *hash(WUXING_CARDS_JSON.as_bytes()).as_bytes(),
        kind: AssetKind::Data,
        image_variants: vec![],
        audio_variants: vec![],
        priority: PreloadPriority::Lazy,
        depends_on: vec![],
        platforms: vec![],
    });
    package.asset_manifest = Some(assets);
    package
        .asset_paths
        .insert("wuxing:config".into(), "assets/wuxing.json".into());
    package
        .asset_paths
        .insert("wuxing:cards".into(), "assets/cards.json".into());
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package
}

/// 3D精选（G1 第四样本，noname 来源）角色数据切片
///（核对自 `apps/core/extension/3D精选/character/{character,translate}.js` @ 5c02a745；
/// 刘冲：群/4血/男，技能 精彀+末焱。精彀映射为宿主原语 max_range_weapon_damage；
/// 末焱（展示牌+获得+存标记+伤害增幅）需更多原语，登记暂缓）。
const D3_CHARACTER_JSON: &str = r#"{
  "source": "noname",
  "upstream_path": "apps/core/extension/3D精选/",
  "upstream_commit": "5c02a745eb33b61b2e61e1167e1b71ca163911f6",
  "pack": { "name": "d3-upstream-noname", "version": 1, "description": "G1 3D精选", "card_prefix": "ddd_" },
  "generals": [
    {
      "id": "ddd_liuchong",
      "name": "ddd刘冲",
      "faction": "qun",
      "hp": 4,
      "gender": "male",
      "skills": [
        { "name": "dddjinggou", "title": "精彀", "mapping": "host:max_range_weapon_damage" },
        { "name": "dddmoyan", "title": "末焱", "mapping": null }
      ]
    }
  ]
}
"#;

fn build_d3_package() -> ContentPackage {
    let mut package = ContentPackage {
        manifest: PackageManifest::new("upstream-noname", "d3-select", "1.0.0"),
        modules: vec![],
        skill_index: vec![],
        character_index: vec![],
        skin_index: Default::default(),
        card_index: vec![],
        trigger_index: vec![],
        modifier_index: vec![],
        active_index: vec![],
        view_as_index: vec![],
        translations: BTreeMap::from([(
            "zh-CN".to_string(),
            BTreeMap::from([
                ("ddd_liuchong".to_string(), "ddd刘冲".to_string()),
                ("dddjinggou".to_string(), "精彀".to_string()),
                ("dddmoyan".to_string(), "末焱".to_string()),
            ]),
        )]),
        asset_manifest: None,
        asset_paths: BTreeMap::new(),
        compiler_version: "gen-1".into(),
    };
    let mut assets = AssetManifest::new("d3-select");
    assets.add_entry(AssetEntry {
        asset_id: "d3-select:characters".into(),
        source_hash: *hash(D3_CHARACTER_JSON.as_bytes()).as_bytes(),
        kind: AssetKind::Data,
        image_variants: vec![],
        audio_variants: vec![],
        priority: PreloadPriority::Lazy,
        depends_on: vec![],
        platforms: vec![],
    });
    package.asset_manifest = Some(assets);
    package
        .asset_paths
        .insert("d3-select:characters".into(), "assets/character.json".into());
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package
}

/// 玩点论杀（G1 第五样本，noname 来源）卡牌切片
///（核对自 `apps/core/extension/玩点论杀/card/{list,card}.js` @ 5c02a745）。
/// 已转换 14 张：基础牌（杀×2/闪×3/桃×2）+ 望梅止渴×2 + 诱敌深入×3 +
/// 三尖×1（射程 3，装备技 wd_sanjian）+ 铁索×1；暂缓：调兵遣将×2
///（多人多步方向选择）。`pack` 段为 C0 装载器读取的包元数据（ADR 0778）。
const WD_CARDS_JSON: &str = r#"{
  "source": "noname",
  "upstream_path": "apps/core/extension/玩点论杀/",
  "upstream_commit": "5c02a745eb33b61b2e61e1167e1b71ca163911f6",
  "pack": { "name": "wudian-upstream-noname", "version": 2, "description": "G1 玩点论杀", "card_prefix": "wd_" },
  "cards": [
    { "name": "sha", "kind": "sha", "suit": "diamond", "rank": 9 },
    { "name": "sha", "kind": "sha", "suit": "spade", "rank": 6 },
    { "name": "shan", "kind": "shan", "suit": "heart", "rank": 6 },
    { "name": "shan", "kind": "shan", "suit": "diamond", "rank": 3 },
    { "name": "shan", "kind": "shan", "suit": "diamond", "rank": 11 },
    { "name": "tao", "kind": "tao", "suit": "heart", "rank": 5 },
    { "name": "tao", "kind": "tao", "suit": "heart", "rank": 7 },
    { "name": "wangmeizhike", "kind": "wangmeizhike", "suit": "club", "rank": 8 },
    { "name": "wangmeizhike", "kind": "wangmeizhike", "suit": "club", "rank": 11 },
    { "name": "youdishenru", "kind": "youdishenru", "suit": "club", "rank": 9 },
    { "name": "youdishenru", "kind": "youdishenru", "suit": "spade", "rank": 11 },
    { "name": "youdishenru", "kind": "youdishenru", "suit": "spade", "rank": 12 },
    { "name": "sanjian", "kind": "sanjian", "suit": "club", "rank": 5, "range": 3, "equip_skills": ["wd_sanjian"] },
    { "name": "tiesuo", "kind": "tiesuo", "suit": "spade", "rank": 4 }
  ],
  "deferred": [
    { "name": "diaobingqianjiang", "count": 2, "reason": "多人方向选择多步结算" }
  ],
  "skills": [
    { "name": "wd_wangmeizhike", "card": "wangmeizhike", "mapping": "host:wangmeizhike_card" },
    { "name": "wd_sanjian", "card": "sanjian", "mapping": "host:sanjian_weapon" },
    { "name": "moshou", "general": "wdb_feishi", "mapping": "host:moshou_immunity" }
  ],
  "generals": [
    {
      "name": "费诗", "faction": "shu", "hp": 3, "gender": "male",
      "skills": [
        { "name": "nsshuaiyan", "mapping": null, "reason": "多目标牌局联动原语" },
        { "name": "moshou", "mapping": null, "reason": "mod 免疫类原语" }
      ]
    },
    {
      "name": "韩龙", "faction": "wei", "hp": 4, "gender": "male",
      "skills": [
        { "name": "siji", "mapping": "host:siji_discard_draw" },
        { "name": "ciqiu", "mapping": null, "reason": "伤害增幅+自戕子技能" }
      ]
    },
    {
      "name": "刘馥", "faction": "wei", "hp": 3, "gender": "male",
      "skills": [
        { "name": "zhucheng", "mapping": null, "reason": "多步筑城原语" },
        { "name": "duoqi", "mapping": null, "reason": "全局弃牌遍历原语" }
      ]
    },
    {
      "name": "刘焉", "faction": "qun", "hp": 3, "gender": "male",
      "skills": [
        { "name": "juedao", "mapping": null, "reason": "距离/决斗联动原语" },
        { "name": "geju", "mapping": null, "reason": "割据标记原语" }
      ]
    },
    {
      "name": "留赞", "faction": "wu", "hp": 4, "gender": "male",
      "skills": [
        { "name": "kangyin", "mapping": null, "reason": "抗yin 多步原语" }
      ]
    },
    {
      "name": "田豫", "faction": "wei", "hp": 4, "gender": "male",
      "skills": [
        { "name": "chezhen", "mapping": null, "reason": "车阵标记原语" },
        { "name": "youzhan", "mapping": null, "reason": "诱战多步原语" }
      ]
    }
  ]
}
"#;

fn build_wd_package() -> ContentPackage {
    let mut package = ContentPackage {
        manifest: PackageManifest::new("upstream-noname", "wudian", "1.0.0"),
        modules: vec![],
        skill_index: vec![],
        character_index: vec![],
        skin_index: Default::default(),
        card_index: vec![],
        trigger_index: vec![],
        modifier_index: vec![],
        active_index: vec![],
        view_as_index: vec![],
        translations: BTreeMap::from([(
            "zh-CN".to_string(),
            BTreeMap::from([
                ("wangmeizhike".to_string(), "望梅止渴".to_string()),
                ("diaobingqianjiang".to_string(), "调兵遣将".to_string()),
                ("youdishenru".to_string(), "诱敌深入".to_string()),
                ("sanjian".to_string(), "三尖".to_string()),
            ]),
        )]),
        asset_manifest: None,
        asset_paths: BTreeMap::new(),
        compiler_version: "gen-1".into(),
    };
    let mut assets = AssetManifest::new("wudian");
    assets.add_entry(AssetEntry {
        asset_id: "wudian:cards".into(),
        source_hash: *hash(WD_CARDS_JSON.as_bytes()).as_bytes(),
        kind: AssetKind::Data,
        image_variants: vec![],
        audio_variants: vec![],
        priority: PreloadPriority::Lazy,
        depends_on: vec![],
        platforms: vec![],
    });
    package.asset_manifest = Some(assets);
    package
        .asset_paths
        .insert("wudian:cards".into(), "assets/cards.json".into());
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package
}

/// 杀海拾遗·运筹帷幄子包（G1 第六样本，noname 来源）
///（核对自 `apps/core/extension/杀海拾遗/card/yunchou/{list,card}.js`）。
/// 切片：4/22 张经既有牌种直转（火杀 1 + 过河拆桥 3）+ 水攻（新原语：
/// 多目标强制弃装备）；其余 17 张按 reason 登记（拼点比较、特殊锦囊、
/// 装备技、舌战群儒群体系）。
const YUNCHOU_CARDS_JSON: &str = r#"{
  "source": "noname",
  "upstream_path": "apps/core/extension/杀海拾遗/card/yunchou/",
  "upstream_commit": "5c02a745eb33b61b2e61e1167e1b71ca163911f6",
  "pack": { "name": "shahai-yunchou-upstream-noname", "version": 2, "description": "G1 杀海拾遗·运筹帷幄切片", "card_prefix": "yc_" },
  "cards": [
    { "name": "huoshan", "kind": "huosha", "suit": "heart", "rank": 6 },
    { "name": "guohe", "kind": "guohe", "suit": "diamond", "rank": 3 },
    { "name": "guohe", "kind": "guohe", "suit": "club", "rank": 8 },
    { "name": "guohe", "kind": "guohe", "suit": "spade", "rank": 13 },
    { "name": "shuigong", "kind": "shuigong", "suit": "club", "rank": 7 },
    { "name": "shuigong", "kind": "shuigong", "suit": "club", "rank": 7 },
    { "name": "shuigong", "kind": "shuigong", "suit": "spade", "rank": 9 },
    { "name": "fudichouxin", "kind": "fudichouxin", "suit": "diamond", "rank": 4 },
    { "name": "fudichouxin", "kind": "fudichouxin", "suit": "club", "rank": 6 },
    { "name": "fudichouxin", "kind": "fudichouxin", "suit": "spade", "rank": 1 },
    { "name": "chenhuodajie", "kind": "chenhuodajie", "suit": "diamond", "rank": 6 },
    { "name": "chenhuodajie", "kind": "chenhuodajie", "suit": "diamond", "rank": 9 },
    { "name": "chenhuodajie", "kind": "chenhuodajie", "suit": "club", "rank": 3 },
    { "name": "suolianjia", "kind": "suolianjia", "suit": "club", "rank": 13, "link_on_equip": true },
    { "name": "hongshui", "kind": "hongshui", "suit": "club", "rank": 7 },
    { "name": "toulianghuanzhu", "kind": "toulianghuanzhu", "suit": "heart", "rank": 9 },
    { "name": "toulianghuanzhu", "kind": "toulianghuanzhu", "suit": "club", "rank": 10 },
    { "name": "geanguanhuo", "kind": "geanguanhuo", "suit": "heart", "rank": 1 },
    { "name": "geanguanhuo", "kind": "geanguanhuo", "suit": "spade", "rank": 6 },
    { "name": "caochuanjiejian", "kind": "caochuanjiejian", "suit": "spade", "rank": 7 },
    { "name": "caochuanjiejian", "kind": "caochuanjiejian", "suit": "spade", "rank": 7 },
    { "name": "shezhanqunru", "kind": "shezhanqunru", "suit": "heart", "rank": 4 },
    { "name": "shezhanqunru", "kind": "shezhanqunru", "suit": "club", "rank": 8 }
  ],
  "deferred": []
}
"#;

/// 杀海拾遗·古剑子包切片（G1 第八样本，noname 来源）
///（核对自 `apps/core/extension/杀海拾遗/card/gujian/{list,card}.js`）。
/// 已转换 9/40 张：舒筋散 2（目标弃任意张手牌摸等量——可选弃置原语）、
/// 面里金针 1（目标摸 1 后受 1 点伤害——登记差异：失去体力简化为伤害）、
/// 飞镖 2（目标弃一张黑色手牌或受 1 点伤害——可选弃置+登记差异同前；
/// 上游毒属性未建模）、金莲烛 2（伤害前置事件取消——持有者打出防止
/// 此伤害、来源摸 1；登记差异：chooseToUse 简化为确认后确定性弃置）、
/// 食物 7（摊黄冻/有更/青团[回合内首杀摸 1]/元宝肉[额外一张杀]/
/// 咕咚羹[受杀伤害 -1 一次性]/麻婆豆腐[结束阶段弃牌，登记差异：随机
/// 改确定性]/春饼[手牌上限 +1，5 回合]——计时状态原语，rounds 倒计时
/// 效果触发点扣减）。
/// 余簇：紫阳丹 2[+3 护甲，结束阶段 -1 衰减；登记差异：上游每阶段
/// 开始衰减]/玉女元神 1[濒死一次性回复标记]/石会粉 3[需闪否则禁用
/// 对其他角色用牌至其回合结束；登记差异：上游为他人出牌阶段响应
/// 使用，原生为主动使用]/六风散 2[获得两张实体闪；登记差异：虚拟牌
/// 生成改实体牌获得，牌堆无闪则无事]。
/// 最后一批 5/40：砂糖[火伤+护甲，emit/Yield 两段式]/黑龙鳞片[+1 护
/// 甲；登记差异：防御距离临时技暂缓]/硫头阁 2[自己摸 1/他人弃首张；
/// 登记差异：随机效果池简化为确定性]/木头面具[view-as 装备技]。
/// 帝女玄霜/玉衡已转换（暂缓登记差异：帝女玄霜濒死救援弃摸骑乘
/// 暂缓按桃结算；玉衡升级机制+随机敌目标暂缓，确定性目标+首张）。
/// 古剑 40/40 全卡转换收官。食物/装备的伴随技能由引擎牌种表登记，
/// 包数据只含 `pack` 元数据与牌（ADR 0778）。
const GUJIAN_CARDS_JSON: &str = r#"{
  "source": "noname",
  "upstream_path": "apps/core/extension/杀海拾遗/card/gujian/",
  "upstream_commit": "5c02a745eb33b61b2e61e1167e1b71ca163911f6",
  "pack": { "name": "shahai-gujian-upstream-noname", "version": 1, "description": "G1 杀海拾遗·古剑切片", "card_prefix": "gj_" },
  "cards": [
    { "name": "shujinsan", "kind": "shujinsan", "suit": "diamond", "rank": 6 },
    { "name": "shujinsan", "kind": "shujinsan", "suit": "spade", "rank": 2 },
    { "name": "mianlijinzhen", "kind": "mianlijinzhen", "suit": "spade", "rank": 1 },
    { "name": "feibiao", "kind": "feibiao", "suit": "club", "rank": 8 },
    { "name": "feibiao", "kind": "feibiao", "suit": "diamond", "rank": 9 },
    { "name": "jinlianzhu", "kind": "jinlianzhu", "suit": "heart", "rank": 9 },
    { "name": "jinlianzhu", "kind": "jinlianzhu", "suit": "spade", "rank": 7 },
    { "name": "tanhuadong", "kind": "tanhuadong", "suit": "spade", "rank": 2 },
    { "name": "yougeng", "kind": "yougeng", "suit": "heart", "rank": 12 },
    { "name": "qingtuan", "kind": "qingtuan", "suit": "spade", "rank": 3 },
    { "name": "yuanbaorou", "kind": "yuanbaorou", "suit": "spade", "rank": 8 },
    { "name": "gudonggeng", "kind": "gudonggeng", "suit": "heart", "rank": 8 },
    { "name": "mapodoufu", "kind": "mapodoufu", "suit": "heart", "rank": 4 },
    { "name": "chunbing", "kind": "chunbing", "suit": "club", "rank": 3 },
    { "name": "liyutang", "kind": "liyutang", "suit": "heart", "rank": 1 },
    { "name": "molicha", "kind": "molicha", "suit": "club", "rank": 1 },
    { "name": "xiajiao", "kind": "xiajiao", "suit": "diamond", "rank": 6 },
    { "name": "mizhilianou", "kind": "mizhilianou", "suit": "diamond", "rank": 4 },
    { "name": "luyugeng", "kind": "luyugeng", "suit": "club", "rank": 11 },
    { "name": "bingpotong", "kind": "bingpotong", "suit": "spade", "rank": 3 },
    { "name": "bingpotong", "kind": "bingpotong", "suit": "club", "rank": 12 },
    { "name": "qiankunbiao", "kind": "qiankunbiao", "suit": "spade", "rank": 9 },
    { "name": "qiankunbiao", "kind": "qiankunbiao", "suit": "club", "rank": 13 },
    { "name": "shenhuofeiya", "kind": "shenhuofeiya", "suit": "diamond", "rank": 9 },
    { "name": "longxugou", "kind": "longxugou", "suit": "spade", "rank": 7 },
    { "name": "ziyangdan", "kind": "ziyangdan", "suit": "diamond", "rank": 6 },
    { "name": "ziyangdan", "kind": "ziyangdan", "suit": "heart", "rank": 1 },
    { "name": "yunvyuanshen", "kind": "yunvyuanshen", "suit": "heart", "rank": 13 },
    { "name": "shihuifen", "kind": "shihuifen", "suit": "club", "rank": 5 },
    { "name": "shihuifen", "kind": "shihuifen", "suit": "club", "rank": 1 },
    { "name": "shihuifen", "kind": "shihuifen", "suit": "spade", "rank": 13 },
    { "name": "liufengsan", "kind": "liufengsan", "suit": "club", "rank": 6 },
    { "name": "liufengsan", "kind": "liufengsan", "suit": "club", "rank": 3 },
    { "name": "shatang", "kind": "shatang", "suit": "heart", "rank": 13 },
    { "name": "heilonglinpian", "kind": "heilonglinpian", "suit": "spade", "rank": 2 },
    { "name": "liutouge", "kind": "liutouge", "suit": "heart", "rank": 6 },
    { "name": "liutouge", "kind": "liutouge", "suit": "club", "rank": 6 },
    { "name": "mutoumianju", "kind": "mutoumianju", "suit": "club", "rank": 4 },
    { "name": "dinvxuanshuang", "kind": "dinvxuanshuang", "suit": "heart", "rank": 9 },
    { "name": "gjyuheng", "kind": "gjyuheng", "suit": "spade", "rank": 7 }
  ],
  "deferred": [





















  ]
}
"#;

/// 构建杀海拾遗·古剑子包的 npack 清单。
fn build_gujian_package() -> ContentPackage {
    let mut package = ContentPackage {
        manifest: PackageManifest::new("upstream-noname", "shahai-gujian", "1.0.0"),
        modules: vec![],
        skill_index: vec![],
        character_index: vec![],
        skin_index: Default::default(),
        card_index: vec![],
        trigger_index: vec![],
        modifier_index: vec![],
        active_index: vec![],
        view_as_index: vec![],
        translations: BTreeMap::from([(
            "zh-CN".to_string(),
            BTreeMap::from([
                ("shujinsan".to_string(), "舒筋散".to_string()),
                ("mianlijinzhen".to_string(), "面里金针".to_string()),
                ("feibiao".to_string(), "飞镖".to_string()),
                ("jinlianzhu".to_string(), "金莲烛".to_string()),
                ("tanhuadong".to_string(), "摊黄冻".to_string()),
                ("yougeng".to_string(), "有更".to_string()),
                ("qingtuan".to_string(), "青团".to_string()),
                ("yuanbaorou".to_string(), "元宝肉".to_string()),
                ("gudonggeng".to_string(), "咕咚羹".to_string()),
                ("mapodoufu".to_string(), "麻婆豆腐".to_string()),
                ("chunbing".to_string(), "春饼".to_string()),
                ("liyutang".to_string(), "鲤鱼汤".to_string()),
                ("molicha".to_string(), "麻茶".to_string()),
                ("xiajiao".to_string(), "菱角".to_string()),
                ("mizhilianou".to_string(), "蜜链藕".to_string()),
                ("luyugeng".to_string(), "卤玉羹".to_string()),
                ("bingpotong".to_string(), "冰魄通".to_string()),
                ("qiankunbiao".to_string(), "乾坤镖".to_string()),
                ("shenhuofeiya".to_string(), "神火飞鸦".to_string()),
                ("longxugou".to_string(), "龙须钩".to_string()),
                ("ziyangdan".to_string(), "紫阳丹".to_string()),
                ("yunvyuanshen".to_string(), "玉女元神".to_string()),
                ("shihuifen".to_string(), "石会粉".to_string()),
                ("liufengsan".to_string(), "六风散".to_string()),
                ("shatang".to_string(), "砂糖".to_string()),
                ("heilonglinpian".to_string(), "黑龙鳞片".to_string()),
                ("liutouge".to_string(), "硫头阁".to_string()),
                ("mutoumianju".to_string(), "木头面具".to_string()),
                ("dinvxuanshuang".to_string(), "帝女玄霜".to_string()),
                ("gjyuheng".to_string(), "玉衡".to_string()),
                ("gujian".to_string(), "古剑奇谭".to_string()),
            ]),
        )]),
        asset_manifest: None,
        asset_paths: BTreeMap::new(),
        compiler_version: "gen-1".into(),
    };
    let mut assets = AssetManifest::new("shahai-gujian");
    assets.add_entry(AssetEntry {
        asset_id: "shahai-gujian:cards".into(),
        source_hash: *hash(GUJIAN_CARDS_JSON.as_bytes()).as_bytes(),
        kind: AssetKind::Data,
        image_variants: vec![],
        audio_variants: vec![],
        priority: PreloadPriority::Lazy,
        depends_on: vec![],
        platforms: vec![],
    });
    package.asset_manifest = Some(assets);
    package
        .asset_paths
        .insert("shahai-gujian:cards".into(), "assets/cards.json".into());
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package
}

/// 杀海拾遗·gwent 子包首切片（G1 第九样本，noname 来源）
///（核对自 `apps/core/extension/杀海拾遗/card/gwent/{list,card}.js`）。
/// 已转换 9/21 张：燕子药水 2（最小手牌数摸 2 否则 1）/奥祖之雷 2
/// （1 雷伤后摸 1）/瘟疫 1（弃首张或受伤；登记差异：随机改首张、
/// 失去体力简化为伤害）/置换军 2（反复失去体力上限）/暴雪药水 2
/// （弃前两张摸 1；登记差异：确定性前两张）。
/// 天气状态群 6/21 转换（v58）：倾盆大雨 2[手牌上限 -1 至弃牌阶段
/// 结束，手牌上限钩子读取]/碧日浓雾 2[不能使用杀至出牌阶段结束，
/// 杀次数校验钩子读取]/刺骨寒霜 2[摸牌阶段结束弃首张，登记差异：
/// 等价净 -1]——目标+距离 1 邻座，授予机跳过已持有者。
/// 收官 4/21（v59）：昆恩法印[防止普通伤害，按防止次数计数，初始
/// min(5, 存活数)；登记差异：上游按回合计数]/山崩术[弃其他角色前
/// 两件装备；登记差异：随机改座位序前二]/破晓 2[分支清除天气+判定
/// 区牌或获得牌堆首张天气法术；登记差异：随机改牌堆序首张]。
/// 祖墨手铠 2[封印：目标技能失效至其阶段结束；登记差异：原生无
/// 锁定技区分，全部技能失效]——gwent 21/21 全卡转换收官。
const GWENT_CARDS_JSON: &str = r#"{
  "source": "noname",
  "upstream_path": "apps/core/extension/杀海拾遗/card/gwent/",
  "upstream_commit": "5c02a745eb33b61b2e61e1167e1b71ca163911f6",
  "pack": { "name": "shahai-gwent-upstream-noname", "version": 1, "description": "G1 杀海拾遗·gwent 切片", "card_prefix": "gw_" },
  "cards": [
    { "name": "yanziyaoshui", "kind": "yanziyaoshui", "suit": "heart", "rank": 8 },
    { "name": "yanziyaoshui", "kind": "yanziyaoshui", "suit": "club", "rank": 7 },
    { "name": "aozuzhilei", "kind": "aozuzhilei", "suit": "spade", "rank": 9 },
    { "name": "aozuzhilei", "kind": "aozuzhilei", "suit": "club", "rank": 3 },
    { "name": "wenyi", "kind": "wenyi", "suit": "club", "rank": 5 },
    { "name": "zhihuanjun", "kind": "zhihuanjun", "suit": "club", "rank": 1 },
    { "name": "zhihuanjun", "kind": "zhihuanjun", "suit": "spade", "rank": 3 },
    { "name": "baoxueyaoshui", "kind": "baoxueyaoshui", "suit": "heart", "rank": 4 },
    { "name": "baoxueyaoshui", "kind": "baoxueyaoshui", "suit": "spade", "rank": 8 },
    { "name": "qinpendayu", "kind": "qinpendayu", "suit": "diamond", "rank": 5 },
    { "name": "qinpendayu", "kind": "qinpendayu", "suit": "club", "rank": 7 },
    { "name": "birinongwu", "kind": "birinongwu", "suit": "spade", "rank": 9 },
    { "name": "birinongwu", "kind": "birinongwu", "suit": "heart", "rank": 13 },
    { "name": "ciguhanshuang", "kind": "ciguhanshuang", "suit": "diamond", "rank": 11 },
    { "name": "ciguhanshuang", "kind": "ciguhanshuang", "suit": "club", "rank": 7 },
    { "name": "kunenfayin", "kind": "kunenfayin", "suit": "spade", "rank": 11 },
    { "name": "shanbengshu", "kind": "shanbengshu", "suit": "spade", "rank": 8 },
    { "name": "poxiao", "kind": "poxiao", "suit": "heart", "rank": 7 },
    { "name": "poxiao", "kind": "poxiao", "suit": "diamond", "rank": 4 },
    { "name": "zumoshoukao", "kind": "zumoshoukao", "suit": "spade", "rank": 2 },
    { "name": "zumoshoukao", "kind": "zumoshoukao", "suit": "club", "rank": 5 }
  ],
  "deferred": []
}
"#;

fn build_gwent_package() -> ContentPackage {
    let mut package = ContentPackage {
        manifest: PackageManifest::new("upstream-noname", "shahai-gwent", "1.0.0"),
        modules: vec![],
        skill_index: vec![],
        character_index: vec![],
        skin_index: Default::default(),
        card_index: vec![],
        trigger_index: vec![],
        modifier_index: vec![],
        active_index: vec![],
        view_as_index: vec![],
        translations: BTreeMap::from([(
            "zh-CN".to_string(),
            BTreeMap::from([
                ("yanziyaoshui".to_string(), "燕子药水".to_string()),
                ("aozuzhilei".to_string(), "奥祖之雷".to_string()),
                ("wenyi".to_string(), "瘟疫".to_string()),
                ("zhihuanjun".to_string(), "置换军".to_string()),
                ("baoxueyaoshui".to_string(), "暴雪药水".to_string()),
                ("qinpendayu".to_string(), "倾盆大雨".to_string()),
                ("birinongwu".to_string(), "碧日浓雾".to_string()),
                ("ciguhanshuang".to_string(), "刺骨寒霜".to_string()),
                ("kunenfayin".to_string(), "昆恩法印".to_string()),
                ("shanbengshu".to_string(), "山崩术".to_string()),
                ("poxiao".to_string(), "破晓".to_string()),
                ("zumoshoukao".to_string(), "祖墨手铠".to_string()),
                ("gwent".to_string(), "gwent".to_string()),
            ]),
        )]),
        asset_manifest: None,
        asset_paths: BTreeMap::new(),
        compiler_version: "gen-1".into(),
    };
    let mut assets = AssetManifest::new("shahai-gwent");
    assets.add_entry(AssetEntry {
        asset_id: "shahai-gwent:cards".into(),
        source_hash: *hash(GWENT_CARDS_JSON.as_bytes()).as_bytes(),
        kind: AssetKind::Data,
        image_variants: vec![],
        audio_variants: vec![],
        priority: PreloadPriority::Lazy,
        depends_on: vec![],
        platforms: vec![],
    });
    package.asset_manifest = Some(assets);
    package
        .asset_paths
        .insert("shahai-gwent:cards".into(), "assets/cards.json".into());
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package
}

fn build_yunchou_package() -> ContentPackage {
    let mut package = ContentPackage {
        manifest: PackageManifest::new("upstream-noname", "shahai-yunchou", "1.0.0"),
        modules: vec![],
        skill_index: vec![],
        character_index: vec![],
        skin_index: Default::default(),
        card_index: vec![],
        trigger_index: vec![],
        modifier_index: vec![],
        active_index: vec![],
        view_as_index: vec![],
        translations: BTreeMap::from([(
            "zh-CN".to_string(),
            BTreeMap::from([
                ("huoshan".to_string(), "火杀".to_string()),
                ("shuigong".to_string(), "水攻".to_string()),
                ("fudichouxin".to_string(), "釜底抽薪".to_string()),
                ("yunchou".to_string(), "运筹帷幄".to_string()),
            ]),
        )]),
        asset_manifest: None,
        asset_paths: BTreeMap::new(),
        compiler_version: "gen-1".into(),
    };
    let mut assets = AssetManifest::new("shahai-yunchou");
    assets.add_entry(AssetEntry {
        asset_id: "shahai-yunchou:cards".into(),
        source_hash: *hash(YUNCHOU_CARDS_JSON.as_bytes()).as_bytes(),
        kind: AssetKind::Data,
        image_variants: vec![],
        audio_variants: vec![],
        priority: PreloadPriority::Lazy,
        depends_on: vec![],
        platforms: vec![],
    });
    package.asset_manifest = Some(assets);
    package
        .asset_paths
        .insert("shahai-yunchou:cards".into(), "assets/cards.json".into());
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package
}

/// 杀海拾遗·炉石传说子包切片（G1 第七样本，noname 来源）
///（核对自 `apps/core/extension/杀海拾遗/card/hearth/{list,card}.js`）。
/// 已转换 7/17 张：灵魂之火 1（火焰伤害+自弃手牌）、照明弹 2（弃目标
/// 手牌+双方摸牌）、急耗卡 1（自摸）、闪电剑 2（雷伤+弃牌链，简化为
/// 雷杀语义——登记差异）、神恩术 1（简化：摸 2；上游为随机基本牌池）。
/// 余量 9/17 转换（v55）：盾牌格挡[+1 护甲摸 1 弃 1，登记差异：确定性
/// 首张]/治疗波 2[判定红回 1 否则+1 护甲]/元素毁灭[逐目标弃 1–2 每少
/// 弃一张受 1 雷伤]/星界沟通[体力上限+1 回 1 弃全部手牌]/传送门 2[获
/// 得牌堆顶，登记差异：立即使用链暂缓]/坦蛇至人[座位序链式弃首张，
/// 登记差异：随机改确定性]。暂缓 1/17：世界树[牌堆装备搜索+穿戴机]。
const HEARTH_CARDS_JSON: &str = r#"{
  "source": "noname",
  "upstream_path": "apps/core/extension/杀海拾遗/card/hearth/",
  "upstream_commit": "5c02a745eb33b61b2e61e1167e1b71ca163911f6",
  "pack": { "name": "shahai-hearth-upstream-noname", "version": 1, "description": "G1 炉石传说切片", "card_prefix": "ht_" },
  "cards": [
    { "name": "linghunzhihuo", "kind": "linghunzhihuo", "suit": "diamond", "rank": 2 },
    { "name": "zhaomingdan", "kind": "zhaomingdan", "suit": "diamond", "rank": 5 },
    { "name": "zhaomingdan", "kind": "zhaomingdan", "suit": "heart", "rank": 10 },
    { "name": "jihuocard", "kind": "jihuocard", "suit": "diamond", "rank": 2 },
    { "name": "shandianjian", "kind": "leisha", "suit": "club", "rank": 3 },
    { "name": "shandianjian", "kind": "leisha", "suit": "spade", "rank": 1 },
    { "name": "shenenshu", "kind": "jihuocard", "suit": "heart", "rank": 2 },
    { "name": "shenenshu", "kind": "jihuocard", "suit": "diamond", "rank": 12 },
    { "name": "dunpaigedang", "kind": "dunpaigedang", "suit": "club", "rank": 3 },
    { "name": "zhiliaobo", "kind": "zhiliaobo", "suit": "club", "rank": 7 },
    { "name": "zhiliaobo", "kind": "zhiliaobo", "suit": "spade", "rank": 1 },
    { "name": "yuansuhuimie", "kind": "yuansuhuimie", "suit": "spade", "rank": 13 },
    { "name": "xingjiegoutong", "kind": "xingjiegoutong", "suit": "spade", "rank": 13 },
    { "name": "chuansongmen", "kind": "chuansongmen", "suit": "diamond", "rank": 2 },
    { "name": "chuansongmen", "kind": "chuansongmen", "suit": "heart", "rank": 2 },
    { "name": "tanshezhiren", "kind": "tanshezhiren", "suit": "diamond", "rank": 2 },
    { "name": "shijieshu", "kind": "shijieshu", "suit": "spade", "rank": 7 }
  ],
  "deferred": []
}
"#;

fn build_hearth_package() -> ContentPackage {
    let mut package = ContentPackage {
        manifest: PackageManifest::new("upstream-noname", "shahai-hearth", "1.0.0"),
        modules: vec![],
        skill_index: vec![],
        character_index: vec![],
        skin_index: Default::default(),
        card_index: vec![],
        trigger_index: vec![],
        modifier_index: vec![],
        active_index: vec![],
        view_as_index: vec![],
        translations: BTreeMap::from([(
            "zh-CN".to_string(),
            BTreeMap::from([
                ("linghunzhihuo".to_string(), "灵魂之火".to_string()),
                ("zhaomingdan".to_string(), "照明弹".to_string()),
                ("jihuocard".to_string(), "急耗卡".to_string()),
                ("shandianjian".to_string(), "闪电剑".to_string()),
                ("shenenshu".to_string(), "神恩术".to_string()),
                ("dunpaigedang".to_string(), "盾牌格挡".to_string()),
                ("zhiliaobo".to_string(), "治疗波".to_string()),
                ("yuansuhuimie".to_string(), "元素毁灭".to_string()),
                ("xingjiegoutong".to_string(), "星界沟通".to_string()),
                ("chuansongmen".to_string(), "传送门".to_string()),
                ("tanshezhiren".to_string(), "坦蛇至人".to_string()),
                ("hearth".to_string(), "炉石传说".to_string()),
            ]),
        )]),
        asset_manifest: None,
        asset_paths: BTreeMap::new(),
        compiler_version: "gen-1".into(),
    };
    let mut assets = AssetManifest::new("shahai-hearth");
    assets.add_entry(AssetEntry {
        asset_id: "shahai-hearth:cards".into(),
        source_hash: *hash(HEARTH_CARDS_JSON.as_bytes()).as_bytes(),
        kind: AssetKind::Data,
        image_variants: vec![],
        audio_variants: vec![],
        priority: PreloadPriority::Lazy,
        depends_on: vec![],
        platforms: vec![],
    });
    package.asset_manifest = Some(assets);
    package
        .asset_paths
        .insert("shahai-hearth:cards".into(), "assets/cards.json".into());
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package
}

fn main() {
    let out_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions");

    // 杀海拾遗·gwent（G1 第九样本：第四子包）
    {
        let dir = out_root.join("shahai-gwent-upstream-noname");
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        let manifest = format!(
            "{{\n  \"schema_version\": 1,\n  \"enabled\": true,\n  \"package\": \"package.npack.json\"\n}}\n"
        );
        std::fs::write(dir.join("manifest.json"), manifest).unwrap();
        std::fs::write(dir.join("assets/cards.json"), GWENT_CARDS_JSON).unwrap();
        let package = build_gwent_package();
        std::fs::write(
            dir.join("package.npack.json"),
            package.to_json_bytes().unwrap(),
        )
        .unwrap();
        println!("written {}", dir.display());
    }

    // 杀海拾遗·古剑（G1 第八样本：第三子包）
    {
        let dir = out_root.join("shahai-gujian-upstream-noname");
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        let manifest = format!(
            "{{\n  \"schema_version\": 1,\n  \"enabled\": true,\n  \"package\": \"package.npack.json\"\n}}\n"
        );
        std::fs::write(dir.join("manifest.json"), manifest).unwrap();
        std::fs::write(dir.join("assets/cards.json"), GUJIAN_CARDS_JSON).unwrap();
        let package = build_gujian_package();
        std::fs::write(
            dir.join("package.npack.json"),
            package.to_json_bytes().unwrap(),
        )
        .unwrap();
        println!("written {}", dir.display());
    }

    // 杀海拾遗·炉石传说（G1 第七样本：第二子包）
    {
        let dir = out_root.join("shahai-hearth-upstream-noname");
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        let manifest = format!(
            "{{\n  \"schema_version\": 1,\n  \"enabled\": true,\n  \"package\": \"package.npack.json\"\n}}\n"
        );
        std::fs::write(dir.join("manifest.json"), manifest).unwrap();
        std::fs::write(dir.join("assets/cards.json"), HEARTH_CARDS_JSON).unwrap();
        let package = build_hearth_package();
        std::fs::write(
            dir.join("package.npack.json"),
            package.to_json_bytes().unwrap(),
        )
        .unwrap();
        println!("written {}", dir.display());
    }

    // 杀海拾遗·运筹帷幄（G1 第六样本：首个子包）
    {
        let dir = out_root.join("shahai-yunchou-upstream-noname");
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        let manifest = format!(
            "{{\n  \"schema_version\": 1,\n  \"enabled\": true,\n  \"package\": \"package.npack.json\"\n}}\n"
        );
        std::fs::write(dir.join("manifest.json"), manifest).unwrap();
        std::fs::write(dir.join("assets/cards.json"), YUNCHOU_CARDS_JSON).unwrap();
        let package = build_yunchou_package();
        std::fs::write(
            dir.join("package.npack.json"),
            package.to_json_bytes().unwrap(),
        )
        .unwrap();
        println!("written {}", dir.display());
    }

    // 玩点论杀（G1 第五样本，noname 来源）
    {
        let dir = out_root.join("wudian-upstream-noname");
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        let manifest = format!(
            "{{\n  \"schema_version\": 1,\n  \"enabled\": true,\n  \"package\": \"package.npack.json\"\n}}\n"
        );
        std::fs::write(dir.join("manifest.json"), manifest).unwrap();
        std::fs::write(dir.join("assets/cards.json"), WD_CARDS_JSON).unwrap();
        let package = build_wd_package();
        std::fs::write(
            dir.join("package.npack.json"),
            package.to_json_bytes().unwrap(),
        )
        .unwrap();
        println!("written {}", dir.display());
    }

    // 3D精选（G1 第四样本，noname 来源）
    {
        let dir = out_root.join("d3-upstream-noname");
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        let manifest = format!(
            "{{\n  \"schema_version\": 1,\n  \"enabled\": true,\n  \"package\": \"package.npack.json\"\n}}\n"
        );
        std::fs::write(dir.join("manifest.json"), manifest).unwrap();
        std::fs::write(dir.join("assets/character.json"), D3_CHARACTER_JSON).unwrap();
        let package = build_d3_package();
        std::fs::write(
            dir.join("package.npack.json"),
            package.to_json_bytes().unwrap(),
        )
        .unwrap();
        println!("written {}", dir.display());
    }

    // wuxing（G1 第三样本，liuli 来源）
    {
        let dir = out_root.join("wuxing-upstream-liuli");
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        let manifest = format!(
            "{{\n  \"schema_version\": 1,\n  \"enabled\": true,\n  \"package\": \"package.npack.json\"\n}}\n"
        );
        std::fs::write(dir.join("manifest.json"), manifest).unwrap();
        std::fs::write(dir.join("assets/wuxing.json"), WUXING_CONFIG_JSON).unwrap();
        std::fs::write(dir.join("assets/cards.json"), WUXING_CARDS_JSON).unwrap();
        let package = build_wuxing_package();
        std::fs::write(
            dir.join("package.npack.json"),
            package.to_json_bytes().unwrap(),
        )
        .unwrap();
        println!("written {}", dir.display());
    }

    // 欢乐卡牌（G1 第二样本）
    {
        let dir = out_root.join("huanle-upstream-noname");
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        let manifest = format!(
            "{{\n  \"schema_version\": 1,\n  \"enabled\": true,\n  \"package\": \"package.npack.json\"\n}}\n"
        );
        std::fs::write(dir.join("manifest.json"), manifest).unwrap();
        std::fs::write(dir.join("assets/cards.json"), huanle_cards_json()).unwrap();
        let package = build_huanle_package();
        std::fs::write(
            dir.join("package.npack.json"),
            package.to_json_bytes().unwrap(),
        )
        .unwrap();
        println!("written {}", dir.display());
    }

    let out_root = out_root;
    let sources: &[(&str, &str, &str, &str)] = &[
        (
            "upstream-noname",
            "noname",
            "apps/core/extension/cardpile/extension.js",
            "5c02a745eb33b61b2e61e1167e1b71ca163911f6",
        ),
        (
            "upstream-liuli",
            "noname-liuli",
            "noname/extension/cardpile/extension.js",
            "8485dbb29ff16f9b258a766b1a17ef75f883aade",
        ),
    ];
    for (namespace, source, path, commit) in sources {
        let dir = out_root.join(format!("cardpile-{namespace}"));
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        let manifest = format!(
            "{{\n  \"schema_version\": 1,\n  \"enabled\": true,\n  \"package\": \"package.npack.json\"\n}}\n"
        );
        std::fs::write(dir.join("manifest.json"), manifest).unwrap();
        let pile = pile_json(source, path, commit);
        std::fs::write(dir.join("assets/pile.json"), &pile).unwrap();
        let package = build_package(namespace, source, path, commit);
        std::fs::write(
            dir.join("package.npack.json"),
            package.to_json_bytes().unwrap(),
        )
        .unwrap();
        println!("written {}", dir.display());
    }
}
