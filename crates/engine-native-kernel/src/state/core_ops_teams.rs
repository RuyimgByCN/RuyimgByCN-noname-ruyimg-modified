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
}
