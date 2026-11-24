//! Bridge output and error types shared across loader submodules.

use engine_native::content::extension::{ExtensionGeneral, ExtensionPack, VmSkillDecl};
use engine_native::content::host::HostSkillDecl;

use crate::BridgedDefinitionIds;

/// 装载错误：坏包、未知键与宿主 API 版本不符给出明确原因。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BridgeError {
    /// JSON 解析失败。
    BadJson(String),
    /// 缺少必需字段。
    MissingField(&'static str),
    /// 字段类型或取值非法。
    InvalidField { field: &'static str, reason: String },
    /// `kind` 不是 engine-native 的牌种键。
    UnknownCardKind(String),
    /// 未知花色。
    UnknownSuit(String),
    /// `mapping` 不是 engine-native 的宿主行为键。
    UnknownBehaviorKey(String),
    /// 未知势力。
    UnknownFaction(String),
    /// 未知性别。
    UnknownGender(String),
    /// 未知开局钩子键。
    UnknownInitHook(String),
    /// 装备技既不是本包 `skills` 映射，也不是牌种伴随技能。
    UnknownEquipSkill(String),
    /// 包内宿主技能重名。
    DuplicateSkill(String),
    /// cards.json 声明的引擎 API 版本与当前运行时不符。
    HostVersionMismatch { declared: u64, current: u32 },
    /// 名称符号表已达总量上限（ADR 0779）：拒绝装载而非无界增长。
    SymbolTableLimit { limit: usize },
    /// 包声明了原生桥接尚不承载的 DSL 滤镜（PA12 批①：装载期拒绝——
    /// 此前静默跳过会让技能「装上但不触发」）。
    UnsupportedVmFilters { filters: Vec<String> },
}

impl std::fmt::Display for BridgeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BridgeError::BadJson(e) => write!(f, "包数据解析失败: {e}"),
            BridgeError::MissingField(fld) => write!(f, "包数据缺少字段 {fld}"),
            BridgeError::InvalidField { field, reason } => {
                write!(f, "包数据字段 {field} 非法: {reason}")
            }
            BridgeError::UnknownCardKind(k) => write!(f, "未知牌种键 {k}"),
            BridgeError::UnknownSuit(s) => write!(f, "未知花色 {s}"),
            BridgeError::UnknownBehaviorKey(k) => write!(f, "未知宿主行为键 {k}"),
            BridgeError::UnknownFaction(s) => write!(f, "未知势力 {s}"),
            BridgeError::UnknownGender(s) => write!(f, "未知性别 {s}"),
            BridgeError::UnknownInitHook(s) => write!(f, "未知开局钩子 {s}"),
            BridgeError::UnknownEquipSkill(s) => write!(f, "装备技 {s} 未在本包登记"),
            BridgeError::DuplicateSkill(s) => write!(f, "包内宿主技能重名 {s}"),
            BridgeError::HostVersionMismatch { declared, current } => write!(
                f,
                "包声明的引擎 API 版本 {declared} 与运行时 {current} 不符"
            ),
            BridgeError::UnsupportedVmFilters { filters } => {
                write!(
                    f,
                    "VM 技能滤镜原生不承载（整包拒装）: {}",
                    filters.join(", ")
                )
            }
            BridgeError::SymbolTableLimit { limit } => {
                write!(f, "名称符号表已达总量上限 {limit}，拒绝装载（ADR 0779）")
            }
        }
    }
}

impl std::error::Error for BridgeError {}

/// 装载产物：扩展包 + 未映射（暂缓）牌清单。
pub struct BridgedPack {
    pub pack: ExtensionPack,
    /// cards.json `deferred` 数组的牌名（登记暂缓，不阻断装载）。
    pub deferred: Vec<String>,
    /// cards.json 声明的 definition_id。
    pub definition_ids: BridgedDefinitionIds,
}

/// 武将技能装载产物。
pub struct BridgedGenerals {
    pub generals: Vec<ExtensionGeneral>,
    /// 命中宿主行为的技能声明（按名去重，并入 ExtensionPack 用）。
    pub host_skills: Vec<HostSkillDecl>,
    /// 命中宿主行为的技能键（逐武将出现次序；内容锁/断言用）。
    pub mapped_skill_keys: Vec<&'static str>,
    /// 未映射（暂缓）的技能计数。
    pub unmapped_skills: usize,
    /// generals[].definition_id 与 generals[].skills[].definition_id。
    pub definition_ids: BridgedDefinitionIds,
}

pub(crate) fn invalid(field: &'static str, reason: impl Into<String>) -> BridgeError {
    BridgeError::InvalidField {
        field,
        reason: reason.into(),
    }
}

/// T8 rule-vm trigger outcome (`npack` -> engine-native `VmSkillDecl`).
#[derive(Debug)]
pub struct VmBridgeOutcome {
    pub decls: Vec<VmSkillDecl>,
}
