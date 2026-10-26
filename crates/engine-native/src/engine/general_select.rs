//! NC05 产品入口：**正式选将/装配**（不使用 `debug_grant`）。
//!
//! 背景：此前只有建局时的随机装配与 `debug_*` 助手；NC05 要求独立扩展经
//! 「正式选将/开局」进入对局，而不是靠调试助手授技。本模块提供产品入口：
//!
//! - [`Engine::assign_general`]：把目录中的武将装配到座位（清旧技 → 挂新将
//!   → 授予其技能集与体力上限），并校验武将存在、座位合法、对局未结束；
//! - [`Engine::general_candidates`]：该座位当前可装配的武将候选（含扩展包
//!   武将），供选将 UI/AI 使用。
//!
//! 与 `debug_*` 的区别：入口做**产品级校验**并返回拒绝原因；调试助手仍仅用于
//! 测试构造。未建模的部分（赛前大厅阶段、多武将明置选择）登记为待办。

use crate::engine::{Engine, RejectReason};
use crate::ids::PlayerId;

impl Engine {
    /// 该座位当前可装配的武将候选（目录全量：标准 + Boss + 扩展包）。
    ///
    /// 顺序稳定（目录定义序），供选将 UI/AI 使用；不改变引擎状态。
    pub fn general_candidates(&self) -> Vec<u16> {
        (0..self.catalog().general_count() as u16)
            .filter(|def| self.catalog().general_meta(*def).is_some())
            .collect()
    }

    /// 产品入口：把武将 `general` 装配到 `seat`（正式选将）。
    ///
    /// 校验：座位在范围内、武将存在于目录、对局未结束。装配语义与建局一致
    /// （撤销旧武将技能 → 写入主将槽 → 按武将技能集与体力上限重装）。
    /// 返回 `Err(RejectReason)` 表示不可装配（不改变状态）。
    pub fn assign_general(&mut self, seat: PlayerId, general: u16) -> Result<(), RejectReason> {
        if self.outcome().is_some() {
            return Err(RejectReason::MatchOver);
        }
        if seat as usize >= self.state_core().players.len() {
            return Err(RejectReason::UnknownActor);
        }
        if self.catalog().general_meta(general).is_none() {
            return Err(RejectReason::NoSuchGeneral);
        }
        // Only revoke the old generals' skills: extension-global and equipment grants survive selection.
        let removed: Vec<crate::ids::SkillDefId> = self
            .state_core()
            .player(seat)
            .general_slots
            .iter()
            .flatten()
            .filter_map(|def| self.catalog().general_meta(*def))
            .flat_map(|meta| meta.2)
            .filter_map(|name| self.catalog().skill_by_name(name))
            .collect();
        for def in removed {
            let name = self.catalog().skills[def as usize].name;
            let still_provided = self.catalog().extension_global_skills.contains(&def)
                || self.state_core().player(seat).equips.iter().any(|card| {
                    self.catalog().cards[*card as usize]
                        .equip_skills
                        .contains(&name)
                });
            if !still_provided {
                self.state_core_mut().skills.remove(&(seat, def));
            }
        }
        self.state_core_mut().player_mut(seat).general_slots[0] = Some(general);
        self.state_core_mut().player_mut(seat).general_slots[1] = None;
        self.state_core_mut().player_mut(seat).revealed_generals = [true, false];
        let catalog = self.catalog().clone();
        crate::engine::apply_general_assembly(self.state_core_mut(), &catalog, seat, 0);
        let lord = self
            .state_core()
            .identity_roles()
            .and_then(|roles| roles.get(seat as usize))
            == Some(&crate::state::Role::Lord);
        let p = self.state_core_mut().player_mut(seat);
        if lord && p.max_hp != crate::state::INFINITE_HP {
            p.max_hp += 1;
        }
        p.hp = p.max_hp as i32;
        p.alive = true;
        p.dying = false;
        Ok(())
    }

    /// 按**名字**装配（便捷入口；名字不在目录时返回 `NoSuchGeneral`）。
    pub fn assign_general_named(&mut self, seat: PlayerId, name: &str) -> Result<(), RejectReason> {
        let def = (0..self.catalog().general_count() as u16)
            .find(|d| {
                self.catalog()
                    .general_meta(*d)
                    .map(|m| m.0 == name)
                    .unwrap_or(false)
            })
            .ok_or(RejectReason::NoSuchGeneral)?;
        self.assign_general(seat, def)
    }
}
