//! E01–E03 扩展运行时基础（ADR 0776）。
//!
//! 扩展包 = 额外武将（元数据）+ 额外卡牌（可入局）。
//! 生命周期：安装（校验冲突）→ 启用（合并入目录并绑定内容锁版本）→
//! 禁用（后续开局不再合并）→ 卸载。
//! 内容锁：启用扩展集合参与目录 `ruleset_version` 派生——存档/恢复
//! 按同一扩展集合校验，集合不同即 `RuleVersionMismatch` 拒绝。
//! 冲突一律返回可处理错误，不再 panic。

use crate::catalog::CardDef;
use crate::catalog::Faction;
use crate::content::standard::{Gender, STANDARD_GENERALS};

/// 扩展包元数据。
#[derive(Debug, Clone)]
pub struct ExtensionDef {
    pub name: &'static str,
    pub version: u32,
    pub description: &'static str,
}

/// 扩展生命周期/内容错误（可处理，不再 panic）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExtensionError {
    /// 启用/禁用/卸载未安装的扩展。
    NotInstalled { name: &'static str },
    /// 重复安装同名扩展（先卸载再安装，或走升级入口）。
    AlreadyInstalled { name: &'static str },
    /// 包内/包间重复卡牌名。
    DuplicateCard { name: &'static str },
    /// 包内/包间重复武将名。
    DuplicateGeneral { name: &'static str },
    /// 包内/包间重复技能名。
    DuplicateSkill { name: String },
    /// VM 模块装载期校验失败（PA06：畸形字节码在建目录时拒绝）。
    InvalidVmModule { name: String },
}

pub use crate::vocabulary::generals::ExtensionGeneral;

/// 开局初始化钩子（G1：扩展声明的对局初始化行为）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InitHook {
    #[default]
    None,
    /// wuxing：开局为每名角色随机五行属性，牌堆按比例随机获得属性。
    WuxingNatures,
    /// swd 器族（卡牌面补切批⑩）：开局为每名角色授予四象五件手持触发技
    /// 实例（效果由手持 filter 门控）。
    SwdJiqiHeld,
}

impl InitHook {
    /// 扩展包数据 `init_hook` 键（宿主 ABI，只增不改）。
    pub const SWD_JIQI_HELD_KEY: &str = "swd_jiqi_held";
    pub const fn key(self) -> &'static str {
        match self {
            InitHook::None => "none",
            InitHook::WuxingNatures => "wuxing_natures",
            InitHook::SwdJiqiHeld => Self::SWD_JIQI_HELD_KEY,
        }
    }

    pub fn grants_global_skill(self, behavior: crate::content::host::HostBehavior) -> bool {
        use crate::content::host::HostBehavior;
        match self {
            Self::None => false,
            Self::WuxingNatures => behavior == HostBehavior::ShengkeCheck,
            Self::SwdJiqiHeld => matches!(behavior,
                HostBehavior::SwdQinglongzhiguiHeld
                    | HostBehavior::SwdZhuquezhizhangHeld
                    | HostBehavior::SwdBaishouzhihuHeld
                    | HostBehavior::SwdXuanwuzhihuangHeld
                    | HostBehavior::SwdHuanglinzhicongHeld
                    | HostBehavior::SwdShencaojieGuard
                    | HostBehavior::SwdYuchanSwap
                    | HostBehavior::SwdShenmiguoWindow
            ),
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        [InitHook::None, InitHook::WuxingNatures, InitHook::SwdJiqiHeld]
            .into_iter()
            .find(|h| h.key() == key)
    }
}

/// T8 增量 1：包声明的 rule-vm 技能（数据驱动；字节码经装载/编译产
/// 物携带，`logic_hash` 为内容锁锚）。
#[derive(Debug, Clone)]
pub struct VmSkillDecl {
    /// Validated schema5 package declaration identity; never an execution permission.
    pub declaration_hash: Option<[u8; 32]>,
    /// 技能名（目录内唯一；intern 进全局符号表）。
    pub name: String,
    /// 已验证字节码模块。
    pub module: rule_vm::BytecodeModule,
    /// 触发元数据（事件门由行为总线承载）。
    pub trigger: crate::catalog::TriggerMeta,
    /// DSL filter 映射（三基语义；具名组合 filter 装载器跳过）。
    pub filter: crate::rvm_bridge::VmFilterPolicy,
}

/// 扩展包：元数据 + 额外内容。
pub struct ExtensionPack {
    pub def: ExtensionDef,
    pub generals: Vec<ExtensionGeneral>,
    pub cards: Vec<CardDef>,
    /// G1 规则宿主：包声明的宿主原语技能（数据 → 目录 SkillDef）。
    pub host_skills: Vec<crate::content::host::HostSkillDecl>,
    /// T8 增量 1：包声明的 rule-vm 技能（数据 → 目录 SkillDef，触发时
    /// 经 rvm_bridge 执行字节码）。
    pub vm_skills: Vec<VmSkillDecl>,
    /// 开局初始化钩子（影响对局随机流，进入内容锁语义）。
    pub init_hook: InitHook,
    pub enabled: bool,
}

impl ExtensionPack {
    /// 合并后目录中的额外武将数。
    pub fn general_count(&self) -> usize {
        self.generals.len()
    }

    /// 合并后目录中的额外卡牌数。
    pub fn card_count(&self) -> usize {
        self.cards.len()
    }
}

/// 扩展注册表：管理已安装扩展的启用/禁用状态。
#[derive(Debug, Clone, Default)]
pub struct ExtensionRegistry {
    /// 已安装扩展（按安装序）。
    pub installed: Vec<ExtensionDef>,
    /// 已启用扩展名集合。
    pub enabled: std::collections::BTreeSet<&'static str>,
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// 安装扩展（默认启用）。重复安装同名扩展返回错误（升级须显式
    /// 卸载后安装，不静默覆盖）。
    pub fn install(&mut self, def: ExtensionDef) -> Result<(), ExtensionError> {
        if self.installed.iter().any(|d| d.name == def.name) {
            return Err(ExtensionError::AlreadyInstalled { name: def.name });
        }
        self.enabled.insert(def.name);
        self.installed.push(def);
        Ok(())
    }

    /// 禁用扩展（未安装返回错误——拒绝启用/操作未安装包）。
    pub fn disable(&mut self, name: &'static str) -> Result<(), ExtensionError> {
        if !self.is_installed(name) {
            return Err(ExtensionError::NotInstalled { name });
        }
        self.enabled.remove(name);
        Ok(())
    }

    /// 启用扩展（未安装返回错误——此前可启用未安装名，已修复）。
    pub fn enable(&mut self, name: &'static str) -> Result<(), ExtensionError> {
        if !self.is_installed(name) {
            return Err(ExtensionError::NotInstalled { name });
        }
        self.enabled.insert(name);
        Ok(())
    }

    /// 查询扩展是否启用。
    pub fn is_enabled(&self, name: &'static str) -> bool {
        self.is_installed(name) && self.enabled.contains(name)
    }

    /// 查询扩展是否已安装。
    pub fn is_installed(&self, name: &'static str) -> bool {
        self.installed.iter().any(|d| d.name == name)
    }

    /// 卸载扩展（未安装返回错误）。
    pub fn uninstall(&mut self, name: &'static str) -> Result<(), ExtensionError> {
        if !self.is_installed(name) {
            return Err(ExtensionError::NotInstalled { name });
        }
        self.installed.retain(|d| d.name != name);
        self.enabled.remove(name);
        Ok(())
    }
}

/// Content lock for direct extensions, including ordered VM handlers and source trigger metadata.
/// Empty extensions retain the standard catalog identity.
pub fn content_lock_id(packs: &[&ExtensionPack]) -> u64 {
    let mut enabled: Vec<_> = packs.iter().copied().filter(|p| p.enabled).collect();
    enabled.sort_by_key(|p| (p.def.name, p.def.version));
    if enabled.is_empty() { return 0; }
    let mut h = blake3::Hasher::new();
    h.update(b"native-extension-lock/331");
    for pack in enabled {
        let mut value = |bytes: &[u8]| { h.update(&(bytes.len() as u64).to_le_bytes()); h.update(bytes); };
        value(pack.def.name.as_bytes()); value(&pack.def.version.to_le_bytes()); value(pack.init_hook.key().as_bytes());
        h.update(&(pack.host_skills.len() as u64).to_le_bytes());
        for skill in &pack.host_skills {
            h.update(&(skill.name.len() as u64).to_le_bytes()); h.update(skill.name.as_bytes()); h.update(&skill.behavior.tag().to_le_bytes());
            if let Some(meta) = skill.behavior.trigger_meta() { crate::rvm_bridge::hash_trigger_meta(&mut h, &meta); }
        }
        h.update(&(pack.vm_skills.len() as u64).to_le_bytes());
        for skill in &pack.vm_skills {
            h.update(&(skill.name.len() as u64).to_le_bytes()); h.update(skill.name.as_bytes());
            let module = skill.module.to_bytes(); h.update(&(module.len() as u64).to_le_bytes()); h.update(&module);
            crate::rvm_bridge::hash_trigger_meta(&mut h, &skill.trigger); skill.filter.hash_semantics(&mut h);
            if let Some(hash) = skill.declaration_hash { h.update(b"native-extension-mechanisms/1"); h.update(&hash); }
        }
    }
    u64::from_le_bytes(h.finalize().as_bytes()[..8].try_into().expect("eight bytes"))
}

/// 将扩展包内容合并到基础目录数据中。
/// 冲突检测：扩展卡牌与基础目录、以及扩展**包内**重复 → 可处理错误。
pub fn merge_extension_cards(
    base_cards: Vec<CardDef>,
    extension_cards: &[CardDef],
) -> Result<Vec<CardDef>, ExtensionError> {
    let base_names: std::collections::BTreeSet<&str> =
        base_cards.iter().map(|c| c.name).collect();
    let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for ext in extension_cards {
        if base_names.contains(ext.name) {
            return Err(ExtensionError::DuplicateCard { name: ext.name });
        }
        if !seen.insert(ext.name) {
            return Err(ExtensionError::DuplicateCard { name: ext.name });
        }
    }
    let mut all = base_cards;
    all.extend(extension_cards.iter().cloned());
    Ok(all)
}

/// 冲突检测：扩展武将与基础武将表、以及扩展**包内**重复 → 可处理错误。
pub fn check_general_conflicts(
    extension_generals: &[ExtensionGeneral],
) -> Result<(), ExtensionError> {
    let base_names: std::collections::BTreeSet<&str> =
        STANDARD_GENERALS.iter().map(|(g, _, _, _, _)| *g).collect();
    let mut seen: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
    for ext in extension_generals {
        if base_names.contains(ext.name) {
            return Err(ExtensionError::DuplicateGeneral { name: ext.name });
        }
        if !seen.insert(ext.name) {
            return Err(ExtensionError::DuplicateGeneral { name: ext.name });
        }
    }
    Ok(())
}

/// 将扩展武将条目追加到标准武将表（运行时查询用）。
pub fn extension_general_count(extensions: &[&ExtensionPack]) -> usize {
    extensions.iter().filter(|e| e.enabled).map(|e| e.general_count()).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_registry_lifecycle() {
        let mut reg = ExtensionRegistry::new();
        let def = ExtensionDef {
            name: "test_ext",
            version: 1,
            description: "测试扩展",
        };
        assert!(!reg.is_installed("test_ext"));
        reg.install(def).expect("安装");
        assert!(reg.is_installed("test_ext"));
        assert!(reg.is_enabled("test_ext"));
        reg.disable("test_ext").expect("禁用");
        assert!(!reg.is_enabled("test_ext"));
        reg.enable("test_ext").expect("启用");
        assert!(reg.is_enabled("test_ext"));
        reg.uninstall("test_ext").expect("卸载");
        assert!(!reg.is_installed("test_ext"));
    }
}

/// 示例扩展包：追加一张「闪电·赤」（红桃闪电，伤害改为火焰 2 点）
/// 与两名扩展武将（带技能集引用基础目录已有技能）。
pub fn example_extension() -> ExtensionPack {
    ExtensionPack {
        def: ExtensionDef {
            name: "example_ext",
            version: 1,
            description: "示例扩展：赤闪电 + 扩展武将",
        },
        generals: vec![
            ExtensionGeneral {
                name: "样例·赤将",
                faction: Faction::Wei,
                skills: &["奸雄"],
                max_hp: 4,
                gender: Gender::Male,
            },
            ExtensionGeneral {
                name: "样例·青将",
                faction: Faction::Wu,
                skills: &["克己"],
                max_hp: 3,
                gender: Gender::Female,
            },
        ],
        cards: vec![
            CardDef::basic("shandian_red", crate::catalog::CardKind::ShanDian)
                .with_suit_rank(crate::catalog::Suit::Heart, 5),
        ],
        host_skills: Vec::new(),
        vm_skills: Vec::new(),
        init_hook: InitHook::None,
        enabled: true,
    }
}
