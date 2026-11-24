//! P3-E1 L11 (`taixu`) 跨包 NPC 与疑点 VM 技能全量迁移宿主模块（`HostBehavior::L11VmFixA`，tag 1389）。
//!
//! 迁移范围（共 31 技，`xiangle` 复用 `U01DddBatchD`，`jsrgbahu` 保留 `VM` 验证）：
//! - 主动出牌：`wurong`, `yeyan`, `qiangwu`, `drlt_xiongluan`, `mingce`, `nsyangwu`
//! - 阶段/回合：`miji`, `fulin`, `langmie`, `yingjian`, `kunfen`, `lianpo`, `fenming`, `guzheng`,
//!   `gzyinghun`, `chunlao`, `nzry_longnu`
//! - 伤害/用牌/失牌/摸牌/濒死：`fuji`, `refankui`, `fenwei`, `kuangfu`, `wushen`, `qiaomeng`,
//!   `shenxian`, `jiezi`, `bihuo`, `fenyin`, `jianying`, `kaikang`, `lihuo`, `manji`

mod active;
mod asks;
mod combat_triggers;
mod helpers;
mod hooks;
mod phase_triggers;
mod routing;

pub(in crate::content::host) use asks::ask;
pub(in crate::content::host) use hooks::{
    dying_save_skills, on_card_use_after, on_phase_use_end, on_player_died, on_round_start,
    resolve_dying_save,
};
pub(in crate::content::host) use routing::{
    activate, filter_with_frames, has_phase_use, limits, machine, trigger_meta,
};

use super::super::*;

pub(in crate::content::host) fn handle(m: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match m.def_key.as_str() {
        "wurong" | "yeyan" | "qiangwu" | "drlt_xiongluan" | "mingce" | "nsyangwu" => {
            active::step(m, ctx)
        }
        "miji" | "fulin" | "langmie" | "yingjian" | "kunfen" | "lianpo" | "fenming"
        | "guzheng" | "gzyinghun" | "chunlao" | "chunlao_save" | "nzry_longnu" => {
            phase_triggers::step(m, ctx)
        }
        "fuji" | "refankui" | "fenwei" | "kuangfu" | "wushen" | "qiaomeng" | "shenxian"
        | "jiezi" | "bihuo" | "fenyin" | "jianying" | "kaikang" | "lihuo" | "manji" => {
            combat_triggers::step(m, ctx)
        }
        _ => Flow::Done,
    }
}
