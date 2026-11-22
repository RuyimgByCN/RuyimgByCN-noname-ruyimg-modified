//! L01 AI优化（noname-liuli）的可移植决策。
//!
//! 上游是纯 **AI 调参**扩展（不改规则）：技能威胁度表（`cf`）、武将权重表
//! （`qz`）与一组策略开关。本模块把「参数 + 结算口径」抽成可装载数据：
//! - 威胁度补充模式（`applyCf`：不补充/自动补充/手动补充）；
//! - 权重补充来源（`ckQz`：不设置/评级（线性映射到 `[0.8,1.95]`）/威胁度）；
//! - 权重参与效益结算（`qzCf`：开启后「权重 × 结果」）；
//! - 查询面：`threaten(skill)` / `weight(general, rating)` / `benefit(...)`。
//!
//! 原生 AI（`engine-ai`）为独立策略引擎——本模块只提供**参数与口径**，
//! 消费面由宿主注入（登记差异）。本模块不读写文件、不请求网络、不 eval。

use engine_native::ai::{GeneralRarity, L01AiOptions, L01StrategyMode, NhFriendsMode};
use serde_json::Value;
use std::collections::BTreeMap;

/// L01 `AI优化` 全部 69 项技能 ID（与 `AI优化/extension.js` 对齐）。
pub const L01_ALL_69_SKILLS: [&str; 69] = [
    "_aiyh_firstKs",
    "_aiyh_nhFriends",
    "_aiyh_meks",
    "_aiyh_neiKey",
    "_aiyh_fixEQz",
    "_aiyh_fixECf",
    "_aiyh_fake_prohibited",
    "_aiyh_fixWj",
    "_findZhong",
    "_mjAiSkill",
    "aiMangju",
    "_attzhi",
    "attcard",
    "attdamage",
    "attlosehp",
    "attrecover",
    "attgain",
    "attlose",
    "attturnover",
    "attbo",
    "attmingbian",
    "attmingbian1",
    "attfanzhuang",
    "attbaolu",
    "atttiaofan",
    "_shyg_qjAi",
    "_aiyh_reserved_shan",
    "aiyh_disabled_shan",
    "_shyg_wuxie",
    "_shyg_wuxie_jiedao",
    "_shyg_wuxie_lei",
    "shyg_wuxie_fy",
    "_shyg_wugu",
    "shyg_wugu_ss_pai",
    "shyg_wugu_wss_pai",
    "_shyg_Discard",
    "shyg_Discard_key",
    "_shyg_Discard_du",
    "shyg_Discard_du_useful",
    "_fenghuyunlong",
    "shyg_mz",
    "_tianxiayitong",
    "shyg_smz",
    "jlnj",
    "tfattzhu",
    "tfattzhong",
    "atttf",
    "_yingshilanggu",
    "shyg_tz",
    "shyg_tf",
    "_wanjunpiyi",
    "chaofeng_common",
    "chaofeng_rare",
    "chaofeng_epic",
    "chaofeng_legend",
    "_shyg",
    "gjcx_zhuAi",
    "gjcx_neiAi",
    "gjcx_neiZhong",
    "gjcx_neiFan",
    "gjcx_neiJiang",
    "_aiyh_lianhe",
    "_aiyh_wqjz",
    "xinfu_limu",
    "zhenlie_lose",
    "twzhian_ai",
    "dcwumei_all",
    "aiyh_gjcx_qj",
    "_gjcx_correct",
];

/// 内奸亮明身份模式（上游 `neiKey`：`off` / `can` / `must`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NeiKeyMode {
    Off,
    Can,
    Must,
}

/// 威胁度补充模式（上游 `applyCf`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CfMode {
    /// `off`：不补充。
    Off,
    /// `pj`：进入游戏时按武将评级自动补充。
    Auto,
    /// `sd`：游戏开始/隐匿武将展示时手动补充。
    Manual,
}

/// 武将权重补充来源（上游 `ckQz`）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QzMode {
    /// `off`：不设置。
    Off,
    /// `pj`：按武将评级线性映射到 `[0.8, 1.95]`。
    Rating,
    /// `cf`：以武将威胁度作为权重。
    Threat,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AiTuningConfig {
    pub source: String,
    pub upstream_path: String,
    pub upstream_commit: String,
    /// 技能威胁度表（`cf`：技能名 → 威胁度）。
    pub cf: BTreeMap<String, f64>,
    /// 武将权重表（`qz`：武将名 → 权重）。
    pub qz: BTreeMap<String, f64>,
    /// 威胁度补充模式。
    pub apply_cf: CfMode,
    /// 武将登场补充权重（`applyQz`）。
    pub apply_qz: bool,
    /// 第二权重参考来源（`ckQz`）。
    pub ck_qz: QzMode,
    /// 权重参与效益结算（`qzCf`）。
    pub qz_cf: bool,
    /// 显示好感度（`viewAtt`）。
    pub view_att: bool,
    /// 不砍低血量队友（`nhFriends` 布尔摘要；细粒度阈值见 `nh_friends_mode`）。
    pub nh_friends: bool,
    /// 南蛮/万箭不受 `nhFriends` 保护限制（`ntAoe`）。
    pub nt_aoe: bool,
    /// 禁用千幻（`banQh`）。
    pub ban_qh: bool,
    /// 本体 AI 策略（`btAi`：`shyg` / `gjcx` / `off`）。
    pub bt_ai: L01StrategyMode,
    /// 身份局 AI 策略（`sfjAi`：`shyg` / `gjcx` / `off`）。
    pub sfj_ai: L01StrategyMode,
    /// 装备 AI 优化（`zbAi`）。
    pub zb_ai: bool,
    /// 武将技能 AI 优化（`wjAi`）。
    pub wj_ai: bool,
    /// 细粒度不砍队友阈值（`nhFriends`：`off` / `1..=4` / `hp`）。
    pub nh_friends_mode: NhFriendsMode,
    /// 盲狙激进 AI（`mjAi`）。
    pub mj_ai: bool,
    /// 闲云野鹤（`xyyh`：跳过部分小游戏）。
    pub xyyh: bool,
    /// 本体展示与拼点优化（`hsyh`）。
    pub hsyh: bool,
    /// 慧眼识忠（`findZhong`）。
    pub find_zhong: bool,
    /// 内奸出牌阶段亮明身份模式（`neiKey`）。
    pub nei_key: NeiKeyMode,
    /// 态度测试与伪装追踪（`attceshi`）。
    pub attceshi: bool,
    /// 四人身份局修改（`fixFour`）。
    pub fix_four: bool,
    /// 伪禁开关（`Wj`）。
    pub wj: bool,
    /// 伪禁候选武将数（`wjs`：`None` = `same`，`Some(n)` = `1..=10`）。
    pub wjs: Option<u8>,
    /// 全局伪禁列表（`wj`）。
    pub ban_all: Vec<String>,
    /// 主公禁将列表（`banzhu` / `zhu`）。
    pub ban_zhu: Vec<String>,
    /// 忠臣禁将列表（`banzhong` / `zhong`）。
    pub ban_zhong: Vec<String>,
    /// 反贼禁将列表（`banfan` / `fan`）。
    pub ban_fan: Vec<String>,
    /// 内奸禁将列表（`bannei` / `nei`）。
    pub ban_nei: Vec<String>,
    /// 地主禁将列表（`bandizhu` / `dizhu`）。
    pub ban_dizhu: Vec<String>,
    /// 农民禁将列表（`bannongmin` / `nongmin`）。
    pub ban_nongmin: Vec<String>,
    /// 出牌可修改武将权重（`fixEQz`）。
    pub fix_eqz: bool,
    /// 出牌可修改技能威胁度（`fixECf`）。
    pub fix_ecf: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiTuningError {
    BadField(String),
    MissingField(String),
}

impl AiTuningError {
    pub fn message(&self) -> String {
        match self {
            AiTuningError::BadField(f) => format!("配置字段非法：{f}"),
            AiTuningError::MissingField(f) => format!("配置字段缺失：{f}"),
        }
    }
}

fn opt_str(v: &Value, key: &str) -> Result<String, AiTuningError> {
    v.get(key)
        .and_then(|x| x.as_str())
        .map(str::to_string)
        .ok_or_else(|| AiTuningError::MissingField(key.to_string()))
}

fn opt_bool(v: &Value, key: &str, default: bool) -> bool {
    v.get(key).and_then(|x| x.as_bool()).unwrap_or(default)
}

fn number_table(v: &Value, key: &str) -> Result<BTreeMap<String, f64>, AiTuningError> {
    let mut out = BTreeMap::new();
    let Some(map) = v.get(key) else {
        return Ok(out);
    };
    let Some(map) = map.as_object() else {
        return Err(AiTuningError::BadField(key.to_string()));
    };
    for (k, val) in map {
        let Some(n) = val.as_f64() else {
            return Err(AiTuningError::BadField(format!("{key}.{k}")));
        };
        if !(0.0..=100.0).contains(&n) {
            return Err(AiTuningError::BadField(format!("{key}.{k}")));
        }
        out.insert(k.clone(), n);
    }
    Ok(out)
}

fn string_list(v: &Value, keys: &[&str]) -> Result<Vec<String>, AiTuningError> {
    for &key in keys {
        let Some(raw) = v.get(key) else { continue };
        let Some(arr) = raw.as_array() else {
            return Err(AiTuningError::BadField(key.to_string()));
        };
        let mut out = Vec::with_capacity(arr.len());
        for item in arr {
            let Some(s) = item.as_str() else {
                return Err(AiTuningError::BadField(key.to_string()));
            };
            if !s.is_empty() && !out.iter().any(|x| x == s) {
                out.push(s.to_string());
            }
        }
        return Ok(out);
    }
    Ok(Vec::new())
}

fn parse_strategy_mode(v: &Value, key: &str, default: L01StrategyMode) -> Result<L01StrategyMode, AiTuningError> {
    let Some(raw) = v.get(key) else {
        return Ok(default);
    };
    match raw.as_str() {
        Some("shyg") => Ok(L01StrategyMode::Shyg),
        Some("gjcx") => Ok(L01StrategyMode::Gjcx),
        Some("off") => Ok(L01StrategyMode::Off),
        _ => Err(AiTuningError::BadField(key.to_string())),
    }
}

fn parse_nh_friends(v: &Value) -> Result<(bool, NhFriendsMode), AiTuningError> {
    let Some(raw) = v.get("nhFriends") else {
        return Ok((false, NhFriendsMode::Off));
    };
    if let Some(b) = raw.as_bool() {
        return Ok((b, if b { NhFriendsMode::SelfHp } else { NhFriendsMode::Off }));
    }
    match raw.as_str() {
        Some("off") => Ok((false, NhFriendsMode::Off)),
        Some("1") => Ok((true, NhFriendsMode::MaxHp(1))),
        Some("2") => Ok((true, NhFriendsMode::MaxHp(2))),
        Some("3") => Ok((true, NhFriendsMode::MaxHp(3))),
        Some("4") => Ok((true, NhFriendsMode::MaxHp(4))),
        Some("hp") => Ok((true, NhFriendsMode::SelfHp)),
        _ => Err(AiTuningError::BadField("nhFriends".to_string())),
    }
}

/// 解析转换包内的 `assets/config.json`（兼容旧 8 项与完整 27 项含规则 L01 配置）。
pub fn parse_ai_tuning_config(json: &str) -> Result<AiTuningConfig, AiTuningError> {
    let v: Value = serde_json::from_str(json).map_err(|e| AiTuningError::BadField(e.to_string()))?;
    let apply_cf = match v.get("applyCf").and_then(|x| x.as_str()).unwrap_or("off") {
        "off" => CfMode::Off,
        "pj" => CfMode::Auto,
        "sd" => CfMode::Manual,
        other => return Err(AiTuningError::BadField(format!("applyCf={other}"))),
    };
    let ck_qz = match v.get("ckQz").and_then(|x| x.as_str()).unwrap_or("cf") {
        "off" => QzMode::Off,
        "pj" => QzMode::Rating,
        "cf" => QzMode::Threat,
        other => return Err(AiTuningError::BadField(format!("ckQz={other}"))),
    };
    let nei_key = match v.get("neiKey").and_then(|x| x.as_str()).unwrap_or("off") {
        "off" => NeiKeyMode::Off,
        "can" => NeiKeyMode::Can,
        "must" => NeiKeyMode::Must,
        other => return Err(AiTuningError::BadField(format!("neiKey={other}"))),
    };
    let wjs = match v.get("wjs") {
        None => None,
        Some(val) if val.as_str() == Some("same") => None,
        Some(val) => {
            let parsed = val
                .as_str()
                .and_then(|s| s.parse::<u8>().ok())
                .or_else(|| val.as_u64().and_then(|n| u8::try_from(n).ok()));
            match parsed {
                Some(n @ 1..=10) => Some(n),
                _ => return Err(AiTuningError::BadField("wjs".to_string())),
            }
        }
    };
    let (nh_friends, nh_friends_mode) = parse_nh_friends(&v)?;
    Ok(AiTuningConfig {
        source: opt_str(&v, "source")?,
        upstream_path: opt_str(&v, "upstream_path")?,
        upstream_commit: opt_str(&v, "upstream_commit")?,
        cf: number_table(&v, "cf")?,
        qz: number_table(&v, "qz")?,
        apply_cf,
        apply_qz: opt_bool(&v, "applyQz", false),
        ck_qz,
        qz_cf: opt_bool(&v, "qzCf", true),
        view_att: opt_bool(&v, "viewAtt", false),
        nh_friends,
        nt_aoe: opt_bool(&v, "ntAoe", false),
        ban_qh: opt_bool(&v, "banQh", false),
        bt_ai: parse_strategy_mode(&v, "btAi", L01StrategyMode::Shyg)?,
        sfj_ai: parse_strategy_mode(&v, "sfjAi", L01StrategyMode::Shyg)?,
        zb_ai: opt_bool(&v, "zbAi", true),
        wj_ai: opt_bool(&v, "wjAi", true),
        nh_friends_mode,
        mj_ai: opt_bool(&v, "mjAi", false),
        xyyh: opt_bool(&v, "xyyh", false),
        hsyh: opt_bool(&v, "hsyh", true),
        find_zhong: opt_bool(&v, "findZhong", false),
        nei_key,
        attceshi: opt_bool(&v, "attceshi", true),
        fix_four: opt_bool(&v, "fixFour", false),
        wj: opt_bool(&v, "Wj", false),
        wjs,
        ban_all: string_list(&v, &["wj"])?,
        ban_zhu: string_list(&v, &["banzhu", "zhu"])?,
        ban_zhong: string_list(&v, &["banzhong", "zhong"])?,
        ban_fan: string_list(&v, &["banfan", "fan"])?,
        ban_nei: string_list(&v, &["bannei", "nei"])?,
        ban_dizhu: string_list(&v, &["bandizhu", "dizhu"])?,
        ban_nongmin: string_list(&v, &["bannongmin", "nongmin"])?,
        fix_eqz: opt_bool(&v, "fixEQz", false),
        fix_ecf: opt_bool(&v, "fixECf", false),
    })
}

/// 从公开玩家技能/表权重推导出武将稀有度（严守 N10：仅用于已公开槽位的武将）。
fn infer_visible_rarity(
    cfg: &AiTuningConfig,
    skills: &[&str],
    general_name: &str,
) -> GeneralRarity {
    if skills.contains(&"chaofeng_legend") {
        return GeneralRarity::Legend;
    }
    if skills.contains(&"chaofeng_epic") {
        return GeneralRarity::Epic;
    }
    if skills.contains(&"chaofeng_rare") {
        return GeneralRarity::Rare;
    }
    if skills.contains(&"chaofeng_junk") {
        return GeneralRarity::Junk;
    }
    if skills.contains(&"chaofeng_common") {
        return GeneralRarity::Common;
    }
    if let Some(&w) = cfg.qz.get(general_name) {
        return if w >= 2.1 {
            GeneralRarity::Legend
        } else if w >= 1.65 {
            GeneralRarity::Epic
        } else if w >= 1.15 {
            GeneralRarity::Rare
        } else if w < 0.9 {
            GeneralRarity::Junk
        } else {
            GeneralRarity::Common
        };
    }
    let max_cf = skills
        .iter()
        .filter_map(|s| cfg.cf.get(*s).copied())
        .fold(None, |acc: Option<f64>, v| Some(acc.map_or(v, |a| a.max(v))));
    match max_cf {
        Some(v) if v >= 2.1 => GeneralRarity::Legend,
        Some(v) if v >= 1.65 => GeneralRarity::Epic,
        Some(v) if v >= 1.15 => GeneralRarity::Rare,
        Some(v) if v < 0.9 => GeneralRarity::Junk,
        _ => GeneralRarity::Common,
    }
}

/// 将 L01 配置转换为原生规划器使用的公开策略配置。
pub fn to_l01_ai_options(
    cfg: &AiTuningConfig,
    seat: u8,
    engine: &engine_native::Engine,
) -> L01AiOptions {
    let core = engine.state_core();
    let view = engine.view_for(seat);
    let lord_known_loyalist = core
        .vm_aux
        .flags
        .get(&(seat, "l01_find_zhong_target_plus1".to_string()))
        .copied()
        .and_then(|v| (v > 0).then(|| (v - 1) as u8));
    let mut public_attitudes = Vec::new();
    for p in 0..core.players.len() as u8 {
        if let Some(&raw) = core.vm_aux.flags.get(&(0, format!("l01_pub_att:{p}"))) {
            if raw != 0 {
                let decoded = (i64::from(raw) - 1_000_000) as i32;
                if decoded != 0 {
                    public_attitudes.push((p, decoded));
                }
            }
        }
    }
    let mut general_rarities: Vec<(String, GeneralRarity)> = Vec::new();
    for p in &view.players {
        for g in p.generals.iter().flatten() {
            if !general_rarities.iter().any(|(n, _)| n == g.name) {
                general_rarities.push((
                    g.name.to_string(),
                    infer_visible_rarity(cfg, &p.skills, g.name),
                ));
            }
        }
    }
    let general_weights: Vec<(String, f64)> =
        cfg.qz.iter().map(|(k, v)| (k.clone(), *v)).collect();
    let skill_threats: Vec<(String, f64)> =
        cfg.cf.iter().map(|(k, v)| (k.clone(), *v)).collect();
    let mut opts = L01AiOptions {
        bt_ai: cfg.bt_ai,
        sfj_ai: cfg.sfj_ai,
        zb_ai: cfg.zb_ai,
        wj_ai: cfg.wj_ai,
        nh_friends: cfg.nh_friends_mode,
        nt_aoe: cfg.nt_aoe,
        mj_ai: cfg.mj_ai,
        hsyh: cfg.hsyh,
        xyyh: cfg.xyyh,
        find_zhong: cfg.find_zhong,
        attceshi: cfg.attceshi,
        qz_cf: cfg.qz_cf,
        apply_cf_auto: cfg.apply_cf == CfMode::Auto,
        ck_qz_mode: match cfg.ck_qz {
            QzMode::Off => 0,
            QzMode::Rating => 1,
            QzMode::Threat => 2,
        },
        lord_known_loyalist,
        public_attitudes,
        general_rarities,
        twzhian_active_enemy: false,
        dcwumei_active: view.players.iter().any(|p| {
            p.alive
                && p.skills
                    .iter()
                    .any(|&s| s == "dcwumei_wake" || s == "dcwumei_all")
        }),
        general_weights,
        skill_threats,
        zhenlie_lose_seats: Vec::new(),
        jiu_active: false,
        respond_shan_is_sha: false,
        respond_shan_fire: false,
        respond_shan_attacker: None,
        attacker_sha_usable: false,
    };
    opts.twzhian_active_enemy = view.players.iter().any(|p| {
        p.alive
            && p.seat != seat
            && engine_native::ai::l01_public_attitude(&view, Some(&opts), seat, p.seat) <= 0
            && p.skills
                .iter()
                .any(|&s| s == "twzhian" || s == "twzhian_ai")
    });
    engine_native::ai::l01_policy::sync_l01_public_runtime(engine, &mut opts);
    opts
}

/// 将 L01 含规则开关与禁将列表注入对局权威状态，并在开局阶段即时应用。
pub fn apply_l01_config_to_engine(cfg: &AiTuningConfig, engine: &mut engine_native::Engine) {
    let catalog = engine.catalog().clone();
    let core = engine.state_core_mut();
    let set = |core: &mut engine_native::state::GameStateCore, k: &str, v: u32| {
        core.vm_aux.flags.insert((0, k.to_string()), v);
    };
    set(core, "l01_cfg:fix_four", u32::from(cfg.fix_four));
    set(core, "l01_cfg:find_zhong", u32::from(cfg.find_zhong));
    set(
        core,
        "l01_cfg:neikey_mode",
        match cfg.nei_key {
            NeiKeyMode::Off => 0,
            NeiKeyMode::Can => 1,
            NeiKeyMode::Must => 2,
        },
    );
    set(core, "l01_cfg:fake_prohibited", u32::from(cfg.wj));
    set(core, "l01_cfg:wjs", u32::from(cfg.wjs.unwrap_or(6)));
    set(core, "l01_cfg:attceshi", u32::from(cfg.attceshi));
    set(core, "l01_cfg:xyyh", u32::from(cfg.xyyh));
    set(core, "l01_cfg:fix_eqz", u32::from(cfg.fix_eqz));
    set(core, "l01_cfg:fix_ecf", u32::from(cfg.fix_ecf));
    for (prefix, list) in [
        ("l01_ban:all", &cfg.ban_all),
        ("l01_ban:zhu", &cfg.ban_zhu),
        ("l01_ban:zhong", &cfg.ban_zhong),
        ("l01_ban:fan", &cfg.ban_fan),
        ("l01_ban:nei", &cfg.ban_nei),
        ("l01_ban:dizhu", &cfg.ban_dizhu),
        ("l01_ban:nongmin", &cfg.ban_nongmin),
    ] {
        for gen_id in list {
            core.vm_aux.flags.insert((0, format!("{prefix}:{gen_id}")), 1);
        }
    }
    engine_native::content::host::step_l01_on_game_start(core, &catalog);
}

/// 将 L01 `applyCf = Auto (pj)` 威胁度公式与原生核心对齐。
pub fn apply_cf_rating_threat(
    rarity: GeneralRarity,
    skill_count: usize,
    is_maixie_defend: bool,
) -> f64 {
    engine_native::ai::l01_auto_skill_threat(rarity, skill_count, is_maixie_defend)
}

/// 技能威胁度查询：表内取值；未配置返回 `None`（上游未覆盖即不改动）。
pub fn threaten(cfg: &AiTuningConfig, skill: &str) -> Option<f64> {
    cfg.cf.get(skill).copied()
}

/// 武将权重查询。
///
/// - 表内有值 → 该值；
/// - 否则按 `ckQz`：`Rating` → 评级线性映射到 `[0.8, 1.95]`（评级 0–5 钳制）；
///   `Threat` → 该武将技能威胁度的**最大值**（无记录则 1.0）；`Off` → 1.0。
pub fn weight(cfg: &AiTuningConfig, general: &str, rating: Option<f64>, skills: &[&str]) -> f64 {
    if let Some(w) = cfg.qz.get(general) {
        return *w;
    }
    match cfg.ck_qz {
        QzMode::Off => 1.0,
        QzMode::Rating => rating.map(rating_to_weight).unwrap_or(1.0),
        QzMode::Threat => skills
            .iter()
            .filter_map(|s| cfg.cf.get(*s).copied())
            .fold(None, |acc: Option<f64>, v| Some(acc.map_or(v, |a| a.max(v))))
            .unwrap_or(1.0),
    }
}

/// 评级 → 权重（上游：正相关映射到 `[0.8, 1.95]`；评级 0–5 钳制）。
pub fn rating_to_weight(rating: f64) -> f64 {
    let r = rating.clamp(0.0, 5.0);
    0.8 + (1.95 - 0.8) * (r / 5.0)
}

/// 效益结算：`qzCf` 开启时「权重 × 结果」，否则原结果。
pub fn benefit(cfg: &AiTuningConfig, w: f64, value: f64) -> f64 {
    if cfg.qz_cf {
        w * value
    } else {
        value
    }
}

/// L01 `AI优化` (`archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9216-9219`):
/// `lib.card.yiyi.selectTarget`: `guozhan` 或 `versus` 模式返回 `(-1, -1)`（全体友方），其余模式返回 `(1, 3)`。
pub fn yiyi_select_target_range(mode: &str) -> (i32, i32) {
    if mode == "guozhan" || mode == "versus" {
        (-1, -1)
    } else {
        (1, 3)
    }
}

/// L01 `AI优化` (`archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9225-9227`):
/// `lib.card.yiyi.ai.wuxie`: 固定返回 `0`（不对以逸待劳使用无懈可击）。
pub fn yiyi_wuxie_weight() -> i32 {
    0
}

/// L01 `AI优化` (`archive/upstream/noname-liuli/noname/extension/AI优化/extension.js:9234-9253`):
/// `lib.card.yiyi.ai.result.target` 收益评估公式：
/// `add + Math.sqrt(i / 3.6 + tars) / 2`，其中：
/// - `tars = 0.01 * selected_targets_len`
/// - 若 `target == player`，`i` 为扣除待打出牌（或首张 `yiyi`）后的手牌数；否则 `i` 为目标的 `he` 牌数；
/// - 若目标拥有 `noh` 标签，`add = 1.0`，否则 `0.0`。
pub fn evaluate_yiyi_target_benefit(
    is_self: bool,
    hand_count: usize,
    he_count: usize,
    selected_cards_in_hand: usize,
    has_yiyi_in_hand: bool,
    selected_targets_len: usize,
    has_noh_tag: bool,
) -> f64 {
    let tars = 0.01 * (selected_targets_len as f64);
    let i = if is_self {
        if selected_cards_in_hand > 0 {
            hand_count.saturating_sub(selected_cards_in_hand)
        } else if has_yiyi_in_hand {
            hand_count.saturating_sub(1)
        } else {
            hand_count
        }
    } else {
        he_count
    };
    let add = if has_noh_tag { 1.0 } else { 0.0 };
    add + ((i as f64) / 3.6 + tars).sqrt() / 2.0
}

