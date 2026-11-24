//! Cross-mode team/friend/hostility predicates on [`GameStateCore`].

use super::*;

impl GameStateCore {
    /// 跨模式己方/友方关系判断（供 U02/L03 boss/jiange 等同阵营判定共享）。
    pub fn is_same_team_or_friend(
        &self,
        catalog: &crate::catalog::Catalog,
        owner: PlayerId,
        other: PlayerId,
    ) -> bool {
        if owner == other {
            return true;
        }
        match &self.mode {
            GameMode::Identity { roles } => {
                let r_owner = roles.get(owner as usize);
                let r_other = roles.get(other as usize);
                matches!(
                    (r_owner, r_other),
                    (Some(Role::Lord | Role::Loyalist), Some(Role::Lord | Role::Loyalist))
                        | (Some(Role::Rebel), Some(Role::Rebel))
                )
            }
            GameMode::Versus { teams, .. } => {
                teams.iter().any(|t| t.contains(&owner) && t.contains(&other))
            }
            GameMode::Boss { boss, fellows, .. } => {
                let o_side = owner == *boss || fellows.contains(&owner);
                let t_side = other == *boss || fellows.contains(&other);
                o_side == t_side
            }
            GameMode::Doudizhu { lord } => (owner == *lord) == (other == *lord),
            GameMode::Kingdoms => self.is_friend_of(catalog, owner, other),
            GameMode::Prototype | GameMode::Kof { .. } => false,
        }
    }

    /// 跨模式敌对判断（非己方存活/同阵营则视为敌对）。
    pub fn is_hostile_to(
        &self,
        catalog: &crate::catalog::Catalog,
        owner: PlayerId,
        other: PlayerId,
    ) -> bool {
        owner != other && !self.is_same_team_or_friend(catalog, owner, other)
    }

    /// 对局内可见势力：国战取明置势力（暗置不出势力）；其余模式
    /// （身份/原型）主将势力直接可见——标准玩法无暗置概念，护驾/
    /// 激将等势力资格按此判定。
    pub fn effective_faction(
        &self,
        catalog: &crate::catalog::Catalog,
        p: PlayerId,
    ) -> Option<crate::catalog::Faction> {
        match self.mode {
            GameMode::Kingdoms => self.declared_faction(catalog, p),
            _ => {
                match self
                    .vm_aux
                    .flags
                    .get(&(p, "l11_faction_override".to_string()))
                    .copied()
                    .unwrap_or(0)
                {
                    1 => return Some(crate::catalog::Faction::Wei),
                    2 => return Some(crate::catalog::Faction::Shu),
                    3 => return Some(crate::catalog::Faction::Wu),
                    4 => return Some(crate::catalog::Faction::Qun),
                    _ => {}
                }
                let pl = self.player(p);
                pl.general_slots[0].and_then(|g| catalog.general_meta(g).map(|(_, f, ..)| f))
            }
        }
    }

    /// 大势力判定（zhenfa 地载阵首卡，规则 v110；上游国战 isMajor 同
    /// 口径简化）：effective_faction 非空者按势力统计存活数，人数最大
    /// 的势力（并列同算）为大势力；未明置/无主将者不属大势力。上游
    /// forceMajor 技能标特例未建模（登记）。
    pub fn is_major_faction(&self, catalog: &crate::catalog::Catalog, p: PlayerId) -> bool {
        let mut counts: std::collections::BTreeMap<u8, u32> = Default::default();
        for q in self.alive_players() {
            if let Some(f) = self.effective_faction(catalog, q) {
                *counts.entry(f as u8).or_insert(0) += 1;
            }
        }
        let Some(max) = counts.values().copied().max() else {
            return false;
        };
        match self.effective_faction(catalog, p) {
            Some(f) => counts.get(&(f as u8)) == Some(&max) && max >= 1,
            None => false,
        }
    }
}
