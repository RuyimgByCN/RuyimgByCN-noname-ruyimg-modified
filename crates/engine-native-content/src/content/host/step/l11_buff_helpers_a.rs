//! L11（太虚幻境）Batch L11-3 衍生/被动辅助技能（`L11BuffHelpersA` tag 735）：
//!
//! 承载 10 项 `extension_skill.js:3280..3422` 衍生技能：
//! - `txhj_kapaikeshi`（洞察/明镜临时查看目标手牌）
//! - `txhj_wenhejibei`（延时锦囊不能指定自己为目标 — `d34_flags.rs` + `judge.rs`）
//! - `txhj_yueyingdezhihui`（锦囊牌无距离限制 — `d34_flags.rs`）
//! - `txhj_changbingguangren`（攻击范围 +3 — `d34_flags.rs`）
//! - `txhj_xiongziyingfa`（手牌上限基数 = `max_hp` — `host.rs::hand_limit_base`）
//! - `txhj_wuxiangdexinxue`（手牌上限 +3 — `d34_flags.rs`）
//! - `txhj_pozhenzhifeng`（出牌阶段持武器对敌无视防具 — `d34_flags.rs`）
//! - `txhj_wuweizhile`（本局手牌上限 -1/标记 — `d34_l11_mods.rs`）
//! - `txhj_yuanduandilue`（兵粮寸断无距离限制 — `d34_l11_mods.rs`）
//! - `txhj_huangtiannizhuan`（奇数标记反转闪电判定 — `d34_l11_buff_mods.rs` + `judge.rs`）

use super::*;
use crate::catalog::{QueryCtx, SkillLimits, SkillMachine, TriggerMeta};
use crate::event::EventData;
use crate::ids::PlayerId;
use crate::machine::{Ctx, Flow};

pub(in crate::content::host) fn trigger_meta_l11_buff_helpers_a(
    _name: &str,
) -> Option<TriggerMeta> {
    None
}

pub(in crate::content::host) fn limits_l11_buff_helpers_a(_name: &str) -> SkillLimits {
    SkillLimits::default()
}

pub(in crate::content::host) fn activate_l11_buff_helpers_a(
    _skill: &HostSkillBehavior,
    _ctx: &QueryCtx<'_>,
    _owner: PlayerId,
) -> Option<Box<dyn SkillMachine>> {
    None
}

pub(in crate::content::host) fn filter_l11_buff_helpers_a(
    _skill: &HostSkillBehavior,
    _ctx: &QueryCtx<'_>,
    _owner: PlayerId,
    _event: &EventData,
) -> bool {
    false
}

pub(in crate::content::host) fn machine_l11_buff_helpers_a(
    skill: &HostSkillBehavior,
    owner: PlayerId,
    _event: &EventData,
) -> HostMachine {
    HostMachine {
        def_key: skill.name.clone(),
        behavior: HostBehavior::L11BuffHelpersA,
        owner,
        target: None,
        card: None,
        cards: Vec::new(),
        targets: Vec::new(),
        stage: HostStage::Act,
    }
}

pub(in crate::content::host) fn handle_l11_buff_helpers_a(
    _m: &mut HostMachine,
    _ctx: &mut Ctx<'_>,
) -> Flow {
    Flow::Done
}
