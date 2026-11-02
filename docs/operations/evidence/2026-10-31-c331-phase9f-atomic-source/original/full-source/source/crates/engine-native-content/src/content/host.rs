//! G1 规则宿主原型（ADR 0776）：扩展包以**数据声明行为**，宿主以
//! 有界原语执行——扩展机器载荷的独立装载与显式续行状态。
//!（2026-10-10 事故重建：臂库自 db.part 重放+挖掘恢复；部分行为臂
//! 为占位待重写——见 file-loss-recovery-20261010 记忆）

use crate::catalog::{CardKind, QueryCtx, SkillBehavior};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::EventData;
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::snap::MachineSnapshot;

mod activation;
mod asks;
mod asks_mtg;
mod asks_wudian;
mod intrinsic;
pub(crate) use asks::active_ask_host;
pub(crate) use intrinsic::card_intrinsic;
mod boss;
mod constants;
mod factory;
mod filtering;
mod step;
mod support;

pub use crate::vocabulary::host::*;
use boss::*;
use constants::*;
use support::*;
pub use support::{
    jifeng_single_target_trick, sbzhaxiang_active, shengong_borrow_options,
    shengong_borrowed_owner, yangpijuan_class,
};
pub(crate) use support::{niepan_effect, sync_land_skill};

/// 机关牌名目标规则（按上游 filterTarget 与引擎 PlayCard 校验同口径）。
enum CyqiaoxieRule {
    SelfTarget,
    OneOther,
    AllOthers,
}

/// 宿主技能续执行机器：显式类型化状态（挂起/恢复载荷）。
#[derive(Debug, Clone)]
pub struct HostMachine {
    /// 定义键（目录技能名；String 便于快照编解码，无泄漏风险）。
    pub def_key: String,
    pub behavior: HostBehavior,
    pub owner: PlayerId,
    pub target: Option<PlayerId>,
    /// 关联卡牌（手里剑：伤害归因）。
    pub card: Option<CardId>,
    /// 关联卡牌组（思继：本次弃置的牌）。
    pub cards: Vec<CardId>,
    /// 多目标行为的目标列表（水攻）。
    pub targets: Vec<PlayerId>,
    /// 显式续行阶段（入快照；恢复后从该阶段继续）。
    pub stage: HostStage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostStage {
    /// 执行行为效果。
    Act,
    /// 手里剑：等待目标选择（弃装备 / 受伤）。
    TargetChoice,
    /// 五行盘：等待选择要改属性的手牌。
    SelectCard,
    /// 五行盘：等待选择新属性（PickTag 0..=4）。
    SelectNature,
    /// 拼点：等待目标选牌。
    CompareTargetPick,
    /// 拼点：等待己方选牌。
    ComparePlayerPick,
    /// 丈八蛇矛：等待选择两张手牌。
    CardsAsk,
    /// 分舵（习嫣 jiefen）：等待选交牌目标。
    JiefenTarget,
    /// 分舵：等待目标选交出的手牌。
    JiefenTargetDiscard,
    /// 分舵：等待自己选交出的手牌。
    JiefenOwnerDiscard,
    /// 神秘果窗口（卡牌面窗口机批）：载体用牌结算后——原体上桌面重发射。
    ShenmiguoRestage,
    /// 鼠槌（equip5）：等待目标选择（攻击范围内）。
    ShuchuiTarget,
    /// 鼠槌（equip5）：目标已选定——逐张用杀循环（凤略 Yield 同型）。
    ShuchuiTargetLoop,
    /// 还人：等待选下一关武将（PickTag 下标）。
    HuanrenPick,
}

impl HostStage {
    pub const fn tag(self) -> u8 {
        match self {
            HostStage::Act => 0,
            HostStage::TargetChoice => 1,
            HostStage::SelectCard => 2,
            HostStage::SelectNature => 3,
            HostStage::CompareTargetPick => 4,
            HostStage::ComparePlayerPick => 5,
            HostStage::CardsAsk => 6,
            HostStage::JiefenTarget => 7,
            HostStage::JiefenTargetDiscard => 8,
            HostStage::JiefenOwnerDiscard => 9,
            HostStage::ShenmiguoRestage => 10,
            HostStage::ShuchuiTargetLoop => 11,
            HostStage::ShuchuiTarget => 12,
            HostStage::HuanrenPick => 13,
        }
    }
    pub fn from_tag(t: u8) -> Option<Self> {
        Some(match t {
            0 => HostStage::Act,
            1 => HostStage::TargetChoice,
            2 => HostStage::SelectCard,
            3 => HostStage::SelectNature,
            4 => HostStage::CompareTargetPick,
            5 => HostStage::ComparePlayerPick,
            6 => HostStage::CardsAsk,
            7 => HostStage::JiefenTarget,
            8 => HostStage::JiefenTargetDiscard,
            9 => HostStage::JiefenOwnerDiscard,
            10 => HostStage::ShenmiguoRestage,
            11 => HostStage::ShuchuiTargetLoop,
            12 => HostStage::ShuchuiTarget,
            13 => HostStage::HuanrenPick,
            _ => return None,
        })
    }
}

/// 宿主行为实现：打包为 SkillDef 的行为载体。
pub struct HostSkillBehavior {
    pub name: String,
    pub behavior: HostBehavior,
}

impl SkillBehavior for HostSkillBehavior {
    fn native_damage_effect(
        &self,
        kind: crate::event::EventKind,
    ) -> Option<crate::event::NativeDamageEffect> {
        crate::event::NativeDamageEffect::from_host(self.behavior)
            .filter(|e| e.window().event_kind() == kind)
    }
    fn filter_in_window(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
        event: &EventData,
        kind: crate::event::EventKind,
    ) -> bool {
        if let Some(effect) = self.native_damage_effect(kind) {
            return matches!(event, EventData::Damage(d) if effect.matches(owner,d,ctx,None,None));
        }
        self.filter(ctx, owner, event)
    }
    fn start_in_window(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
        event: &EventData,
        kind: crate::event::EventKind,
    ) -> Box<dyn crate::catalog::SkillMachine> {
        if let Some(effect) = self.native_damage_effect(kind) {
            return Box::new(crate::kernel::NativeDamageSkill {
                owner,
                def: ctx
                    .catalog
                    .skill_by_name(&self.name)
                    .expect("host declaration"),
                window: effect.window(),
            });
        }
        self.start(owner, event)
    }

    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        self.filter_behavior(ctx, owner, event)
    }
    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn crate::catalog::SkillMachine> {
        Box::new(self.machine(owner, event))
    }
    fn activate(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
    ) -> Option<Box<dyn crate::catalog::SkillMachine>> {
        self.activate_behavior(ctx, owner)
    }
    fn as_host_behavior(&self) -> Option<HostBehavior> {
        Some(self.behavior)
    }
    fn blocks_delayed_trick_target(&self) -> bool {
        self.blocks_delayed_trick()
    }
    fn blocks_all_cards(&self) -> bool {
        matches!(self.behavior, HostBehavior::Bingfeng2)
    }
    /// 止息（L27 批 2）：**计数标记 ≥ 体力** 或 **已使用过锦囊**（记录标记
    /// 非零）→ 封锁出牌/响应。两个标记分别以 `new_zhixi` 与
    /// `new_zhixi_mark2` 的技能定义为键存于 `vm_aux.marks`。
    fn blocks_all_cards_when(
        &self,
        catalog: &crate::catalog::Catalog,
        core: &crate::state::GameStateCore,
        p: crate::ids::PlayerId,
    ) -> bool {
        if self.behavior != HostBehavior::NewZhixi {
            return self.blocks_all_cards();
        }
        let mark_of = |name: &str| -> i32 {
            catalog
                .skill_by_name(name)
                .and_then(|d| core.vm_aux.marks.get(&(p, u32::from(d))).copied())
                .unwrap_or(0)
        };
        let count = mark_of(L27_NEW_ZHIXI_KEY);
        let trick = mark_of(L27_NEW_ZHIXI_MARK2_KEY) > 0;
        trick || count >= core.player(p).hp.max(1)
    }
}

impl crate::catalog::SkillMachine for HostMachine {
    fn fork(&self) -> Box<dyn crate::catalog::SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::Host {
            def_key: self.def_key.clone(),
            behavior: self.behavior.tag(),
            stage: self.stage.tag(),
            owner: self.owner,
            target: self.target,
            card: self.card,
            cards: self.cards.clone(),
            targets: self.targets.clone(),
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        self.step_behavior(ctx)
    }
}

/// 目录侧注册：行为 → 触发元数据 + 宿主行为实现（打包为 SkillDef）。
pub fn host_skill_def(name: String, behavior: HostBehavior) -> crate::catalog::SkillDef {
    // 目录技能名为 'static：经符号表 intern，同名名称全局只分配一次
    // （重复开局/启停循环内存恒定，见 content::names）。
    let name: &'static str = crate::content::names::intern(name);
    let limits = match behavior {
        HostBehavior::GujianFoodQingtuan | HostBehavior::GujianFoodLuyugeng => {
            crate::catalog::SkillLimits {
                once_per_turn: true,
                ..Default::default()
            }
        }
        // swd 装备主动技 usable:1（潜偷 v92 补执行/御夫录 v96）：出牌
        // 阶段限一次（上游 usable:1 同口径）
        // 革新印（批 135）：每回合限一次（上游 usable:1）
        HostBehavior::Gxianyin
        | HostBehavior::SwdShentouEquipSkill
        | HostBehavior::SwdYufuluEquipSkill
        | HostBehavior::SwdLangeguaiyiEquipSkill
        | HostBehavior::MtgYixialanRecast
        | HostBehavior::SwdShuchuiUse
        | HostBehavior::SwdDuishi
        | HostBehavior::SwdZhanlu
        // 解纷（第 1007 轮补）：上游 `enable:'phaseUse'` + `usable: 1`
        | HostBehavior::SwdJiefen
        // 魔焰（第 1010 轮补）：上游 `enable:'phaseUse'` + `usable: 1`
        | HostBehavior::Moyan
        // 攻心（第 1011 轮补）：上游 `enable:'phaseUse'` + `usable: 1`
        | HostBehavior::Gongxin
        | HostBehavior::SwdXianghui
        | HostBehavior::SwdSusheng
        | HostBehavior::SwdSenluo
        | HostBehavior::SwdDuanyi
        | HostBehavior::SwdFengming
        | HostBehavior::SwdXiaomoyu
        | HostBehavior::SwdBingjian
        | HostBehavior::SwdShending
        | HostBehavior::SwdGuiyan
        | HostBehavior::SwdXingzhui
        | HostBehavior::SwdHuangyu
        | HostBehavior::SwdTianhuo
        | HostBehavior::SwdYunchou
        | HostBehavior::SwdLuanji
        | HostBehavior::SwdZhanxing
        | HostBehavior::SwdZhenjiu
        | HostBehavior::SwdLmazui
        | HostBehavior::SwdHuodan
        | HostBehavior::SwdYijue
        | HostBehavior::SwdDangping
        | HostBehavior::SwdMingfu
        | HostBehavior::SwdCyxianjiang
        | HostBehavior::SwdJianji
        | HostBehavior::SwdAojian
        | HostBehavior::SwdHuajin
        | HostBehavior::SwdXuanzhou
        | HostBehavior::SwdFuniao
        | HostBehavior::SwdHeihuo
        | HostBehavior::SwdToudan
        | HostBehavior::SwdNlianji
        | HostBehavior::SwdShehun
        | HostBehavior::SwdJuxi
        | HostBehavior::SwdPozhou2
        | HostBehavior::SwdSxianjing
        | HostBehavior::SwdGuxing => crate::catalog::SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        HostBehavior::MtgLinzhongjiantaStrike
        | HostBehavior::MtgYoulinDiscard
        | HostBehavior::WudianKangyin
        | HostBehavior::WudianLiangou
        | HostBehavior::WudianQianggu
        | HostBehavior::WudianYinbo
        | HostBehavior::XianjianGuiyuan
        | HostBehavior::SwdDuanyue
        | HostBehavior::SwdXingdian
        | HostBehavior::SwdLianyaohuShouna
        | HostBehavior::SwdLianyaohuLianhua
        | HostBehavior::SwdShennongdingCard
        | HostBehavior::SwdKunlunjingcCard
        | HostBehavior::SwdSifeizhenmianCard => crate::catalog::SkillLimits {
            once_per_turn: true,
            ..Default::default()
        },
        _ => crate::catalog::SkillLimits::default(),
    };
    crate::catalog::SkillDef {
        name,
        trigger: crate::event::NativeDamageEffect::from_host(behavior)
            .map(|e| e.meta())
            .or_else(|| behavior.trigger_meta()),
        limits,
        behavior: Box::new(HostSkillBehavior {
            name: name.to_string(),
            behavior,
        }),
    }
}

pub use crate::vocabulary::card_skills::{card_kind_equip_skills, card_kind_host_skills};
