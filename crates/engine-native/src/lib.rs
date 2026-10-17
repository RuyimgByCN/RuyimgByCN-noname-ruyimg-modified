//! 干净原生内核原型：只读规则目录 + 紧凑对局状态 + 显式事件栈 +
//! 类型化技能暂停恢复。
//!
//! 本 crate 是重写第一阶段（原生内核）的落地验证场，不以兼容旧
//! `engine-core` 为目标；语义参照见 `docs/adr/0703-*`。

// 穷尽匹配中的重复臂（牌种路由重复、快照解码表重复 tag）会遮蔽
// 真实分支，按错误处理。
#![deny(unreachable_patterns)]

pub mod catalog;
pub mod choice;
pub mod ai;
pub mod archive;
pub mod content;
pub mod view;
pub mod engine;
pub mod event;
pub mod ids;
pub mod kernel;
pub mod machine;
pub mod presentation;
pub mod rvm_bridge;
pub mod rvm_ops;
pub mod rvm_named;
pub mod resolution;
pub mod rng;
pub mod snap;
pub mod state;

pub use catalog::{CardKind, Catalog, EquipSlot, SharedCatalog, SkillBehavior, SkillDef, SkillMachine};
pub use choice::{ChoiceKind, ChoiceOption, ChoiceSpec, OpenChoice};
pub use engine::{AuthToken, CommandOutcome, Engine, GameAction, GameCommand, RejectReason};
pub use event::{DamageData, EventData, EventKind, Phase, Stage, Timing};
pub use state::{
    BossStageKind, BossStagePending, BossStageState, GameMode, Outcome, Role, Winner,
};
pub use view::{ChoiceView, EventView, HandView, PlayerCellView, PlayerView};
pub use ids::{CardId, EventId, PlayerId, SkillDefId};
pub use machine::{Ctx, Flow, LogEvent};
pub use resolution::Resolution;
pub use snap::{MachineSnapshot, ParkSnapshot, ResolutionSnapshot, SnapshotError, StateSnapshot};


// ---------------------------------------------------------------------------
// A0 lib 单元测试（T17，2026-10-11 重建）：10-10 事故恢复中丢失的 5 个
// lib 测试中可由公有 API 枚举的两个。其余三个（CardKind 编号往返/键双
// 射/伴随一致）依赖 archive 模块私有枚举函数，重建于 archive.rs 测试
// 模组。冻结表以 2026-10-11 实序为准（v60 原表 46 名为其前缀，事故中
// 与表一同丢失）。
// ---------------------------------------------------------------------------
#[cfg(test)]
mod a0_lib_tests {
    use crate::content::host::HostBehavior;
    use crate::content::standard::standard_catalog;

    /// HostBehavior 数据键双射 + 唯一（ALL 全量 226，ADR 0778 键表）。
    #[test]
    fn host_behavior_keys_bijective() {
        let mut keys = std::collections::BTreeMap::new();
        for b in HostBehavior::ALL {
            let k = b.key();
            assert!(keys.insert(k, b).is_none(), "宿主行为数据键重复: {k}");
            assert_eq!(HostBehavior::from_key(k), Some(b), "键双射失败: {k}");
        }
        assert_eq!(keys.len(), 478, "HostBehavior 数量应 478（2026-10-17 陷阱/孤行/灵矢/锦鳞/hlss 族/潜龙/涅槃族/清正/灵武/神焰/入梦/义山/守印/魅惑/集控 tag 445-481）；新增行为须同步键表");
    }

    /// standard 技能名序只追加：目录技能名序列与冻结表逐名相等——中间
    /// 插入/重排/删除即失败；尾部追加须更新冻结表并递增规则版本登记。
    #[test]
    fn standard_skill_ids_append_only() {
        const FROZEN: &[&str] = &["散财", "移伤", "碎盾", "肃清", "授艺", "反馈", "奸雄", "遗计", "刚烈", "突袭", "裸衣", "观星", "连营", "空城", "谦逊", "仁德", "激将", "马术", "倾国", "奇才", "救援", "急救", "护驾", "流离", "结姻", "反间", "离间", "铁骑", "咆哮", "奇袭", "无双", "集智", "克己", "国色", "武圣", "龙胆", "闭月", "洛神", "苦肉", "制衡", "英姿", "枭姬", "青囊", "鬼才", "鬼道", "天妒", "据守", "骁果", "狂斧", "耀武", "淑慎", "神智", "恂恂", "忘隙", "义从", "机捷", "急援", "妄尊", "妄尊印", "同疾", "丈八蛇矛", "凶兽", "无脏", "相德", "隐贼", "逐厄", "覆泰", "延妒", "贪欲", "藏目", "积财", "拟态", "乱常", "冥顽", "催毒", "中毒", "妖兽", "兽九", "毒妒", "邪酬", "蛇毒", "兵先", "虚失", "巨源", "招祸", "红莲", "炎狱", "封冻", "巡游", "司圃", "神裔", "雷击", "狂笑", "焚天", "灵渠", "神恩", "星囚", "不臣", "回火", "复燃", "星夏", "绝红", "庆祝", "夹击", "急叮", "造异", "造异异变", "自润", "年瑞", "猛态", "形态轮换", "忍性", "睿智", "守翼", "胆术", "精觉", "魔道", "魔剑", "御兽", "魔炎", "爆敛", "强征", "醉酒", "枭首", "鬼计", "太平", "索命", "吸行", "炼狱", "绝策", "酒池", "鬼谋", "远策", "奇佐", "绝情", "无心", "伤逝", "枭雄", "章武", "凌波", "娇孝", "芳华", "踏水", "火神", "红莲·周瑜", "仙音", "天道", "雷击·张角", "极点", "枳枉", "公正", "血池", "黑绳", "胜负", "雷舟", "雷斧", "雷诛", "服毒", "苦酒", "热恼", "轮回", "往生", "反噬", "拾忧", "忘魂", "往事", "铁面", "铡刀", "诛心", "锁足", "阿鼻", "平等", "热闷", "炙焚", "火刑", "不佛", "无量", "谛听", "帷幕", "冰封", "判官", "拘魂", "望乡", "天狱", "栖梧", "青木", "青木·转阶", "青木3", "青木3·转阶", "白芒", "白芒·转阶", "赤炎", "赤炎·转阶", "玄麟", "玄麟·转阶", "白芒3", "白芒3·转阶", "赤炎3", "赤炎3·转阶", "玄麟3", "玄麟3·转阶", "后援3", "后援3·转阶", "后援4", "后援4·转阶", "还人", "还人·转关", "青翼", "赤翼", "赤仪2", "白翼", "白翼·雷", "涅槃", "浴火", "神威",];
        let cat = standard_catalog();
        let seq: Vec<&str> = cat.skills.iter().map(|d| d.name).collect();
        assert_eq!(seq.len(), FROZEN.len(), "standard 技能数变化（冻结表 {}，实得 {}）", FROZEN.len(), seq.len());
        assert_eq!(seq.as_slice(), FROZEN, "standard 技能名序与冻结表不符：只允许尾部追加");
    }
}
