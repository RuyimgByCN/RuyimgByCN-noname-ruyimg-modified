//! L07 全能搜索：公开目录索引。
//!
//! 索引只接受目录定义（武将、技能、牌种、扩展 cards.json 的牌名与译名）。
//! 不接收对局状态，因此没有手牌、牌堆顺序或暗置区。查询若等于调用方
//! 标明的暗牌实例号，直接拒绝，不返回该牌。
//!
//! 匹配与上游 `findCharacter` 相同：译名子串，或标识以后缀相接。
//! 没有译名的纯 ASCII 标识不进入索引。`debug_execute` 不执行脚本。

use std::collections::{BTreeMap, BTreeSet};

use engine_native::catalog::Catalog;
use engine_native::content::standard::Gender;
use serde_json::Value;

/// 公开条目类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntryKind {
    /// 武将。
    General,
    /// 技能。
    Skill,
    /// 卡牌定义（不是对局中的某一张实例）。
    Card,
}

impl EntryKind {
    fn label(self) -> &'static str {
        match self {
            EntryKind::General => "general",
            EntryKind::Skill => "skill",
            EntryKind::Card => "card",
        }
    }
}

/// 一条公开目录记录。字段只含体力、势力、牌种等定义信息。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PublicEntry {
    /// 类别。
    pub kind: EntryKind,
    /// 来源包。标准目录为 `standard`。
    pub package_id: String,
    /// 稳定标识（武将名、技能名或 cards.json 的 `name`）。
    pub id: String,
    /// 展示名。
    pub display_name: String,
    /// 公开字段。键限于体力、技能名、牌种、花色点数、装备技能与映射。
    pub fields: BTreeMap<String, String>,
}

/// 查询失败。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SearchError {
    /// 空查询。
    EmptyQuery,
    /// 查询指向一张暗牌实例。
    HiddenCard,
    /// 调用方提交了要执行的脚本。
    EvalRefused,
    /// cards.json 或译名无法构成公开索引。
    BadCatalog(String),
}

/// 已安装扩展的公开索引。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PublicIndex {
    entries: Vec<PublicEntry>,
}

/// 上游配置里允许的代码高亮样式。只保存名称，不加载样式文件。
const CSS_THEMES: &[&str] = &[
    "a11y-dark.min.css",
    "an-old-hope.min.css",
    "atom-one-dark.min.css",
    "felipec.min.css",
    "gml.min.css",
    "vs2015.min.css",
];

/// 上游配置里允许的背景名。只保存名称，不加载图片。
const BACKGROUNDS: &[&str] = &[
    "相爱相杀",
    "才颜双绝",
    "新杀春日",
    "青铜龙纹",
    "随机",
    "none",
];

/// 有固定公开译名的标准牌种。未列入的牌种不进索引。
const STANDARD_CARD_NAMES: &[(&str, &str)] = &[
    ("sha", "杀"),
    ("shan", "闪"),
    ("tao", "桃"),
    ("jiu", "酒"),
    ("juedou", "决斗"),
    ("nanman", "南蛮入侵"),
    ("wanjian", "万箭齐发"),
    ("wuxie", "无懈可击"),
    ("lebu", "乐不思蜀"),
    ("bingliang", "兵粮寸断"),
    ("shandian", "闪电"),
    ("guohe", "过河拆桥"),
    ("shunshou", "顺手牵羊"),
    ("tiesuo", "铁索连环"),
    ("huosha", "火杀"),
    ("leisha", "雷杀"),
    ("horse_plus", "+1马"),
    ("horse_minus", "-1马"),
];

/// 全能搜索保存的配置。主题与背景只作枚举值。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchConfig {
    /// 高亮样式文件名。
    pub css: String,
    /// 背景名。
    pub background: String,
}

impl PublicIndex {
    /// 空索引。
    pub fn new() -> Self {
        Self::default()
    }

    /// 当前条目数。
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 是否为空。
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 全部条目（定义序）。
    pub fn entries(&self) -> &[PublicEntry] {
        &self.entries
    }

    /// 纳入标准目录里已有译名的武将、技能和基础牌种。
    pub fn insert_standard(&mut self, catalog: &Catalog) {
        let mut seen_general = BTreeSet::new();
        for index in 0..catalog.general_count() {
            let Some((name, faction, skills, hp, gender)) = catalog.general_meta(index as u16)
            else {
                continue;
            };
            if !seen_general.insert(name) {
                continue;
            }
            let mut fields = BTreeMap::new();
            fields.insert("faction".into(), faction_name(faction).into());
            fields.insert("hp".into(), hp.to_string());
            fields.insert("gender".into(), gender_name(gender).into());
            fields.insert("skills".into(), skills.join(","));
            self.push(PublicEntry {
                kind: EntryKind::General,
                package_id: "standard".into(),
                id: name.into(),
                display_name: name.into(),
                fields,
            });
        }
        let mut seen_skill = BTreeSet::new();
        for skill in &catalog.skills {
            if skill.name.is_empty() || !seen_skill.insert(skill.name) {
                continue;
            }
            self.push(PublicEntry {
                kind: EntryKind::Skill,
                package_id: "standard".into(),
                id: skill.name.into(),
                display_name: skill.name.into(),
                fields: BTreeMap::new(),
            });
        }

        let mut seen_kind = BTreeSet::new();
        for card in &catalog.cards {
            let key = card.kind.key();
            if !seen_kind.insert(key) {
                continue;
            }
            let Some(display) = standard_card_name(key) else {
                continue;
            };
            let mut fields = BTreeMap::new();
            fields.insert("kind".into(), key.into());
            self.push(PublicEntry {
                kind: EntryKind::Card,
                package_id: "standard".into(),
                id: key.into(),
                display_name: display.into(),
                fields,
            });
        }
    }

    /// 从一张 cards.json 和包内 zh-CN 译名纳入公开条目。
    ///
    /// 同一 `name` 的多张实体牌合并为一条，`copies` 只记张数。
    /// 标识使用 cards.json 的 `name`，不使用入局后的 `前缀_序号`。
    pub fn insert_cards_json(
        &mut self,
        package_id: &str,
        cards_json: &str,
        translations: &BTreeMap<String, String>,
    ) -> Result<(), SearchError> {
        if package_id.is_empty() || package_id.contains('/') || package_id.contains("..") {
            return Err(SearchError::BadCatalog(format!(
                "invalid package_id {package_id:?}"
            )));
        }
        let parsed: Value = serde_json::from_str(cards_json)
            .map_err(|err| SearchError::BadCatalog(format!("cards.json: {err}")))?;
        let root = parsed
            .as_object()
            .ok_or_else(|| SearchError::BadCatalog("cards.json 须为对象".into()))?;

        if let Some(cards) = root.get("cards") {
            let cards = cards
                .as_array()
                .ok_or_else(|| SearchError::BadCatalog("cards 须为数组".into()))?;
            let mut grouped: BTreeMap<String, CardAcc> = BTreeMap::new();
            for card in cards {
                let obj = card
                    .as_object()
                    .ok_or_else(|| SearchError::BadCatalog("cards[] 须为对象".into()))?;
                let name = obj
                    .get("name")
                    .and_then(Value::as_str)
                    .ok_or_else(|| SearchError::BadCatalog("cards[].name 缺失".into()))?
                    .to_string();
                let kind = obj
                    .get("kind")
                    .and_then(Value::as_str)
                    .ok_or_else(|| SearchError::BadCatalog("cards[].kind 缺失".into()))?
                    .to_string();
                let acc = grouped.entry(name).or_insert_with(|| CardAcc {
                    kind,
                    suit: obj
                        .get("suit")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                    rank: json_number(obj.get("rank")),
                    range: obj.get("range").map(|value| json_number(Some(value))),
                    equip: BTreeSet::new(),
                    copies: 0,
                });
                acc.copies += 1;
                if let Some(skills) = obj.get("equip_skills").and_then(Value::as_array) {
                    for skill in skills {
                        if let Some(name) = skill.as_str() {
                            acc.equip.insert(name.to_string());
                        }
                    }
                }
            }
            let mappings = skill_mappings(root.get("skills"))?;
            for (name, acc) in grouped {
                let Some(display) = display_name(&name, translations) else {
                    continue;
                };
                let mut fields = BTreeMap::new();
                fields.insert("kind".into(), acc.kind);
                if !acc.suit.is_empty() {
                    fields.insert("suit".into(), acc.suit);
                }
                if !acc.rank.is_empty() {
                    fields.insert("rank".into(), acc.rank);
                }
                if let Some(range) = acc.range {
                    fields.insert("range".into(), range);
                }
                if !acc.equip.is_empty() {
                    let joined = acc.equip.iter().cloned().collect::<Vec<_>>().join(",");
                    fields.insert("equip_skills".into(), joined);
                    let mapped: Vec<_> = acc
                        .equip
                        .iter()
                        .filter_map(|skill| {
                            mappings
                                .get(skill)
                                .map(|mapping| format!("{skill}={mapping}"))
                        })
                        .collect();
                    if !mapped.is_empty() {
                        fields.insert("mapping".into(), mapped.join(","));
                    }
                }
                fields.insert("copies".into(), acc.copies.to_string());
                self.push(PublicEntry {
                    kind: EntryKind::Card,
                    package_id: package_id.into(),
                    id: name,
                    display_name: display,
                    fields,
                });
            }
        }

        if let Some(skills) = root.get("skills").and_then(Value::as_array) {
            for skill in skills {
                let Some(name) = skill.get("name").and_then(Value::as_str) else {
                    continue;
                };
                let Some(display) = display_name(name, translations) else {
                    continue;
                };
                let mut fields = BTreeMap::new();
                if let Some(mapping) = skill.get("mapping").and_then(Value::as_str) {
                    fields.insert("mapping".into(), mapping.into());
                }
                self.push(PublicEntry {
                    kind: EntryKind::Skill,
                    package_id: package_id.into(),
                    id: name.into(),
                    display_name: display,
                    fields,
                });
            }
        }

        if let Some(generals) = root.get("generals").and_then(Value::as_array) {
            for general in generals {
                let Some(name) = general.get("name").and_then(Value::as_str) else {
                    continue;
                };
                let Some(display) = display_name(name, translations) else {
                    continue;
                };
                let mut fields = BTreeMap::new();
                if let Some(hp) = general.get("hp") {
                    fields.insert("hp".into(), json_number(Some(hp)));
                }
                if let Some(faction) = general.get("faction").and_then(Value::as_str) {
                    fields.insert("faction".into(), faction.into());
                }
                if let Some(skills) = general.get("skills").and_then(Value::as_array) {
                    let names: Vec<_> = skills
                        .iter()
                        .filter_map(|skill| skill.get("name").and_then(Value::as_str))
                        .map(str::to_string)
                        .collect();
                    if !names.is_empty() {
                        fields.insert("skills".into(), names.join(","));
                    }
                }
                self.push(PublicEntry {
                    kind: EntryKind::General,
                    package_id: package_id.into(),
                    id: name.into(),
                    display_name: display,
                    fields,
                });
            }
        }
        Ok(())
    }

    /// 禁用或卸载后刷新：丢掉该包的全部条目。
    pub fn drop_package(&mut self, package_id: &str) {
        self.entries.retain(|entry| entry.package_id != package_id);
    }

    /// 公开查询。
    ///
    /// `hidden_instance_ids` 是调用方已知道、但搜索不得取回的牌实例号
    /// （对手手牌、牌堆）。查询等于其中任一十进制号时返回 [`SearchError::HiddenCard`]。
    pub fn search(
        &self,
        query: &str,
        hidden_instance_ids: &[u32],
    ) -> Result<Vec<PublicEntry>, SearchError> {
        let query = query.trim();
        if query.is_empty() {
            return Err(SearchError::EmptyQuery);
        }
        if query_names_hidden(query, hidden_instance_ids) {
            return Err(SearchError::HiddenCard);
        }
        let mut hits: Vec<_> = self
            .entries
            .iter()
            .filter(|entry| !entry.display_name.is_empty())
            .filter(|entry| entry.display_name.contains(query) || entry.id.ends_with(query))
            .filter(|entry| !numeric_id_is_hidden(&entry.id, hidden_instance_ids))
            .cloned()
            .collect();
        hits.sort_by(|a, b| (&a.package_id, a.kind, &a.id).cmp(&(&b.package_id, b.kind, &b.id)));
        Ok(hits)
    }

    /// 按包与标识取详情。没有则 `None`。
    pub fn detail(&self, package_id: &str, id: &str) -> Option<&PublicEntry> {
        self.entries
            .iter()
            .find(|entry| entry.package_id == package_id && entry.id == id)
    }

    fn push(&mut self, entry: PublicEntry) {
        if self.entries.iter().any(|existing| {
            existing.package_id == entry.package_id
                && existing.kind == entry.kind
                && existing.id == entry.id
        }) {
            return;
        }
        self.entries.push(entry);
    }
}

struct CardAcc {
    kind: String,
    suit: String,
    rank: String,
    range: Option<String>,
    equip: BTreeSet<String>,
    copies: u32,
}

/// 解析安装包里的搜索配置。未知样式或背景拒绝，不回落到默认值。
pub fn parse_config(json: &str) -> Result<SearchConfig, String> {
    let value: Value = serde_json::from_str(json).map_err(|err| format!("config: {err}"))?;
    let css = value
        .get("css")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing css".to_string())?;
    let background = value
        .get("background")
        .and_then(Value::as_str)
        .ok_or_else(|| "missing background".to_string())?;
    if !CSS_THEMES.contains(&css) {
        return Err(format!("unknown css {css}"));
    }
    if !BACKGROUNDS.contains(&background) {
        return Err(format!("unknown background {background}"));
    }
    Ok(SearchConfig {
        css: css.into(),
        background: background.into(),
    })
}

/// 结构化查看一条公开记录。不包含脚本，也不读取对局。
pub fn debug_inspect(entry: &PublicEntry) -> String {
    let mut lines = vec![
        format!("kind={}", entry.kind.label()),
        format!("package={}", entry.package_id),
        format!("id={}", entry.id),
        format!("name={}", entry.display_name),
    ];
    for (key, value) in &entry.fields {
        lines.push(format!("{key}={value}"));
    }
    lines.join("\n")
}

/// 调试入口不执行调用方提交的脚本。上游用 `eval` 播放阵亡语音，这里一律拒绝。
pub fn debug_execute(source: &str) -> Result<String, SearchError> {
    let _ = source;
    Err(SearchError::EvalRefused)
}

fn display_name(id: &str, translations: &BTreeMap<String, String>) -> Option<String> {
    if let Some(text) = translations.get(id) {
        if !text.is_empty() {
            return Some(text.clone());
        }
    }
    if id.chars().any(|ch| !ch.is_ascii()) {
        Some(id.to_string())
    } else {
        None
    }
}

fn standard_card_name(key: &str) -> Option<&'static str> {
    STANDARD_CARD_NAMES
        .iter()
        .find(|(kind, _)| *kind == key)
        .map(|(_, name)| *name)
}

fn faction_name(faction: engine_native::catalog::Faction) -> &'static str {
    use engine_native::catalog::Faction;
    match faction {
        Faction::Wei => "wei",
        Faction::Shu => "shu",
        Faction::Wu => "wu",
        Faction::Qun => "qun",
    }
}

fn gender_name(gender: Gender) -> &'static str {
    match gender {
        Gender::Male => "male",
        Gender::Female => "female",
    }
}

fn json_number(value: Option<&Value>) -> String {
    match value {
        Some(Value::Number(number)) => number.to_string(),
        Some(Value::String(text)) => text.clone(),
        _ => String::new(),
    }
}

fn skill_mappings(skills: Option<&Value>) -> Result<BTreeMap<String, String>, SearchError> {
    let mut out = BTreeMap::new();
    let Some(skills) = skills else {
        return Ok(out);
    };
    let skills = skills
        .as_array()
        .ok_or_else(|| SearchError::BadCatalog("skills 须为数组".into()))?;
    for skill in skills {
        let Some(name) = skill.get("name").and_then(Value::as_str) else {
            continue;
        };
        if let Some(mapping) = skill.get("mapping").and_then(Value::as_str) {
            out.insert(name.to_string(), mapping.to_string());
        }
    }
    Ok(out)
}

fn query_names_hidden(query: &str, hidden_instance_ids: &[u32]) -> bool {
    let numeric = query
        .strip_prefix("card:")
        .unwrap_or(query)
        .parse::<u32>()
        .ok();
    numeric.is_some_and(|id| hidden_instance_ids.contains(&id))
}

fn numeric_id_is_hidden(id: &str, hidden_instance_ids: &[u32]) -> bool {
    id.parse::<u32>()
        .ok()
        .is_some_and(|parsed| hidden_instance_ids.contains(&parsed))
}
