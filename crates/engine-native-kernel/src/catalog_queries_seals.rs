//! Seal and card-lock state queries split from `catalog_queries.rs` (`impl Catalog`).

use super::{CardId, CardKind, Catalog, SkillDefId};
use crate::ids::PlayerId;
use crate::state::GameStateCore;

impl Catalog {
    /// L20 / U05-ow (`fz_new_tieji` / `sbtieji` / `ow_qinru_fengyin` / `retieji` / `rewansha`):
    /// whether `p`'s non-locked skills are sealed.
    pub fn is_tieji_sealed(&self, core: &GameStateCore, p: PlayerId) -> bool {
        core.vm_aux
            .flags
            .get(&(p, "fz_new_tieji_target_sealed".to_string()))
            .copied()
            .unwrap_or(0)
            > 0
            || core
                .vm_aux
                .flags
                .get(&(p, "sbtieji_target_sealed".to_string()))
                .copied()
                .unwrap_or(0)
                > 0
            || self
                .skill_by_name("txhj_feilong_skill_blank")
                .is_some_and(|def| core.has_skill(p, def))
            || self
                .skill_by_name("txtanbei_effect1")
                .is_some_and(|def| core.has_skill(p, def))
            || core
                .vm_aux
                .flags
                .get(&(p, "ow_qinru_fengyin".to_string()))
                .copied()
                .unwrap_or(0)
                > 0
            || core
                .vm_aux
                .flags
                .get(&(p, "q4_retieji_target_sealed".to_string()))
                .copied()
                .unwrap_or(0)
                > 0
            || core
                .vm_aux
                .flags
                .get(&(p, "hs_jinzhou_sealed".to_string()))
                .copied()
                .unwrap_or(0)
                > 0
            || core
                .vm_aux
                .flags
                .get(&(p, "hs_fengyin_xuetu".to_string()))
                .copied()
                .unwrap_or(0)
                > 0
            || core
                .vm_aux
                .flags
                .get(&(p, "q4_rewansha_sealed".to_string()))
                .copied()
                .unwrap_or(0)
                > 0
    }

    /// Hand-card lock (`position == "h"` cannot use or respond; equips unaffected).
    /// Includes `swd_yijue2_hand_locked` (`yijue2` until turn end), `dddtongyu`, and `mianzhen2`.
    pub fn blocks_hand_cards(&self, core: &GameStateCore, p: PlayerId) -> bool {
        if self.blocks_all_cards(core, p)
            || crate::rvm_bridge::modifiers::d34_flags::is_dddtongyu_hand_locked(core, p)
            || core
                .vm_aux
                .flags
                .get(&(p, "swd_yijue2_hand_locked".to_string()))
                .copied()
                .unwrap_or(0)
                > 0
            || self.skill_by_name("mianzhen2").is_some_and(|d| {
                self.skills[d as usize].behavior.as_host_behavior()
                    == Some(crate::vocabulary::host::HostBehavior::U05OwBatchB)
                    && core.has_skill(p, d)
            })
        {
            return true;
        }
        core.skills.iter().any(|((o, def), inst)| {
            *o == p && inst.enabled && self.skills[*def as usize].behavior.blocks_hand_cards()
        })
    }

    /// Card-use & dying-save lock (`cardEnabled` + `cardSavable`, excluding `cardRespondable`).
    pub fn blocks_card_use_and_save(&self, core: &GameStateCore, p: PlayerId) -> bool {
        self.blocks_all_cards(core, p)
            || self.skill_by_name("yihun2").is_some_and(|d| {
                self.skills[d as usize].behavior.as_host_behavior()
                    == Some(crate::vocabulary::host::HostBehavior::U05OwBatchC)
                    && core.has_skill(p, d)
            })
    }

    /// Generic `targetEnabled` check (`yihun2`/`sheng2`/`jidong2`/`yinshen`/`yinmi`/`jileng` + VM).
    #[allow(clippy::too_many_arguments)]
    pub fn ow_target_enabled(
        &self,
        core: &GameStateCore,
        actor: PlayerId,
        target: PlayerId,
        card: Option<CardId>,
        kind: CardKind,
        nature: crate::event::DamageNature,
    ) -> bool {
        if !core.player(target).alive {
            return false;
        }
        let has_behavior = |name: &str, beh: crate::vocabulary::host::HostBehavior| {
            self.skill_by_name(name).is_some_and(|d| {
                self.skills[d as usize].behavior.as_host_behavior() == Some(beh)
                    && core.has_skill(target, d)
            })
        };
        if has_behavior("yihun2", crate::vocabulary::host::HostBehavior::U05OwBatchC) {
            return false;
        }
        if target != actor
            && (core.player(target).jidong_active
                || core.player(target).yinshen_active
                || core.player(target).jileng_active
                || core.player(target).yinmi_active
                || has_behavior("sheng2", crate::vocabulary::host::HostBehavior::U05OwBatchD))
        {
            return false;
        }
        !crate::rvm_bridge::modifiers::eval_card_target_canceled_opt(
            core, self, actor, target, card, kind, nature,
        )
    }

    pub fn is_swd_yijue_fengyin_sealed(&self, core: &GameStateCore, p: PlayerId) -> bool {
        core.vm_aux
            .flags
            .get(&(p, "swd_yijue_fengyin_sealed".to_string()))
            .copied()
            .unwrap_or(0)
            > 0
    }

    /// Whether `p` currently has `fengyin` debuff active (disables non-locked skills).
    pub fn is_xj_fengyin_sealed(&self, core: &GameStateCore, p: PlayerId) -> bool {
        core.vm_aux
            .flags
            .get(&(p, "xj_fengyin_sealed".to_string()))
            .copied()
            .unwrap_or(0)
            > 0
            || self.is_swd_yijue_fengyin_sealed(core, p)
    }

    /// Whether `def` is blocked under `fengyin` (`!charlotte && !forever && !locked && !equipSkill`).
    pub fn is_xj_fengyin_blocked(&self, def: SkillDefId) -> bool {
        let Some(skill) = self.skills.get(def as usize) else {
            return false;
        };
        !self.is_charlotte_skill(def)
            && !self.is_forever_skill(def)
            && !Self::is_locked_skill_name(skill.name, skill.trigger.as_ref())
            && !self.cards.iter().any(|c| c.equip_skills.contains(&skill.name))
    }

    /// Returns the active skill-blocker flag prefix if `def` should be inserted/retained disabled.
    pub fn skill_disable_flag_for(
        &self,
        core: &GameStateCore,
        player: PlayerId,
        def: SkillDefId,
    ) -> Option<&'static str> {
        if !self.is_charlotte_skill(def) && !self.is_forever_skill(def) {
            if crate::kernel::mbcmfangzhu_baiban(core, player) {
                return Some("mbcmfangzhu_disabled");
            }
            if self.is_swd_yijue_fengyin_sealed(core, player) && self.is_xj_fengyin_blocked(def) {
                return Some("swd_yijue_fengyin_disabled");
            }
            if self.is_xj_fengyin_sealed(core, player) && self.is_xj_fengyin_blocked(def) {
                return Some("xj_fengyin_disabled");
            }
        }
        None
    }
}
