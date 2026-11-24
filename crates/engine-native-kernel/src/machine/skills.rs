//! `Ctx` 技能实例增删、启用与回合临时生命周期方法（自 `machine.rs` 按职责拆分）。

use super::*;

impl Ctx<'_> {
    /// Preserve native instance overwrite semantics and register a genuinely new subscription.
    pub fn insert_skill(
        &mut self,
        key: (PlayerId, crate::ids::SkillDefId),
        mut instance: crate::state::SkillInstance,
    ) -> Option<crate::state::SkillInstance> {
        let block_prefix = instance
            .enabled
            .then(|| self.catalog.skill_disable_flag_for(self.core, key.0, key.1))
            .flatten();
        if block_prefix.is_some() {
            instance.enabled = false;
        }
        let old = self.core.skills.insert(key, instance);
        if let Some(prefix) = block_prefix {
            self.core
                .vm_aux
                .flags
                .insert((key.0, format!("{prefix}:{}", key.1)), 1);
        }
        if old.is_none() {
            if self.catalog.skill_name(key.1) == "ssr_luoyi" {
                let discarded = crate::kernel::discard_armor_for_ssr_luoyi(
                    self.core,
                    self.catalog,
                    key.0,
                );
                if !discarded.is_empty() {
                    note_cards_lost(self.core, self.catalog, key.0, &discarded);
                    self.log(LogEvent::CardDiscarded {
                        player: key.0,
                        cards: discarded,
                    });
                }
            }
            crate::resolution::register_added(self.core, self.res, self.catalog, key.0, key.1);
            (self.catalog.hooks.on_skill_granted)(self.core, self.catalog, key.0, key.1);
        }
        old
    }

    /// Insert only if absent; existing instances retain their limits and lifetime.
    pub fn ensure_skill(
        &mut self,
        key: (PlayerId, crate::ids::SkillDefId),
        instance: crate::state::SkillInstance,
    ) {
        if !self.core.skills.contains_key(&key) {
            self.insert_skill(key, instance);
        }
    }

    /// R05-J（§6）：移除座位技能实例，若为 R05 helper 则一并清理对应 mark。
    fn finish_removed_skill(&mut self, owner: PlayerId, name: &str) {
        let returned =
            (self.catalog.hooks.on_skill_removed)(self.core, self.catalog, owner, name);
        if !returned.is_empty() {
            for &c in &returned {
                self.to_discard(c);
            }
            (self.catalog.hooks.on_cards_to_discard)(self, owner, &returned, false);
        }
    }

    pub fn remove_skill(
        &mut self,
        owner: PlayerId,
        def: SkillDefId,
    ) -> Option<crate::state::SkillInstance> {
        let old = self.core.remove_skill_and_r05_mark(self.catalog, owner, def);
        self.core
            .vm_aux
            .flags
            .remove(&(owner, format!("mbcmfangzhu_disabled:{def}")));
        self.core
            .vm_aux
            .flags
            .remove(&(owner, format!("xj_fengyin_disabled:{def}")));
        self.core
            .vm_aux
            .flags
            .remove(&(owner, format!("swd_yijue_fengyin_disabled:{def}")));
        if old.is_some() {
            let name = self.catalog.skill_name(def);
            self.finish_removed_skill(owner, name);
        }
        old
    }

    pub fn skill_enabled(&self, owner: PlayerId, def: SkillDefId) -> bool {
        self.core.has_skill(owner, def)
    }

    pub fn set_skill_enabled(&mut self, owner: PlayerId, def: SkillDefId, enabled: bool) {
        let key = (owner, def);
        if let Some(inst) = self.core.skills.get_mut(&key) {
            if inst.enabled != enabled {
                inst.enabled = enabled;
                let name = self.catalog.skill_name(def);
                self.emit(EventData::SkillToggled {
                    player: owner,
                    skill: def,
                    enabled,
                    temporary: false,
                });
                self.out.push(LogEvent::SkillToggled {
                    player: owner,
                    skill: name,
                    enabled,
                });
            }
        }
    }

    /// 动态获得技能，至拥有者回合结束移除。
    pub fn add_skill_until_turn_end(&mut self, owner: PlayerId, def: SkillDefId) {
        let name = self.catalog.skill_name(def);
        let key = (owner, def);
        let block_prefix = self.catalog.skill_disable_flag_for(self.core, owner, def);
        if let Some(inst) = self.core.skills.get_mut(&key) {
            // 永久拥有（含已觉醒/禁用）技能不可被临时授技覆写或重新启用（上游 player.js:11267,11288）；
            // 仅已到期的 `异殇` 墓碑或已为回合末临时技实例允许刷新至回合结束。
            if !inst.expires_end_of_turn && name != crate::vocabulary::names::YI_SHANG {
                return;
            }
            inst.enabled = block_prefix.is_none();
            inst.expires_end_of_turn = true;
            if let Some(prefix) = block_prefix {
                self.core
                    .vm_aux
                    .flags
                    .insert((owner, format!("{prefix}:{def}")), 1);
            }
        } else {
            self.insert_skill(
                key,
                crate::state::SkillInstance {
                    owner,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: true,
                    rounds: 0,
                },
            );
        }
        self.emit(EventData::SkillToggled {
            player: owner,
            skill: def,
            enabled: true,
            temporary: true,
        });
        self.out.push(LogEvent::SkillToggled {
            player: owner,
            skill: name,
            enabled: true,
        });
    }

    /// 移除到期临时技能（回合结束提交时调用；即使回合内被忘魂/封印禁用也须过期清理）。
    pub fn expire_temporary_skills(&mut self, owner: PlayerId) {
        let expired: Vec<(SkillDefId, bool)> = self
            .core
            .skills
            .iter()
            .filter(|((o, _), i)| *o == owner && i.expires_end_of_turn)
            .map(|((_, d), i)| (*d, i.enabled))
            .collect();
        for (def, was_enabled) in expired {
            let name = self.catalog.skill_name(def);
            let force_remove = self
                .core
                .vm_aux
                .flags
                .remove(&(owner, format!("xuelunyang_temp:{def}")))
                .is_some();
            if force_remove || name != crate::vocabulary::names::YI_SHANG {
                if self.core.skills.remove(&(owner, def)).is_some() {
                    self.finish_removed_skill(owner, name);
                }
            } else if let Some(inst) = self.core.skills.get_mut(&(owner, def)) {
                inst.enabled = false;
                inst.expires_end_of_turn = false;
            }
            if was_enabled {
                self.emit(EventData::SkillToggled {
                    player: owner,
                    skill: def,
                    enabled: false,
                    temporary: true,
                });
                self.out.push(LogEvent::SkillToggled {
                    player: owner,
                    skill: name,
                    enabled: false,
                });
            }
        }
    }

    /// 重置 once_per_turn 标记（拥有者回合开始时调用；`dddjiexing` 为限定技不由回合开始重置）。
    pub fn reset_turn_limits(&mut self, owner: PlayerId) {
        for ((o, def), inst) in self.core.skills.iter_mut() {
            if *o == owner && self.catalog.skill_name(*def) != "dddjiexing" {
                inst.once_used = false;
            }
        }
        self.core
            .vm_aux
            .flags
            .remove(&(owner, "gw_guaiwuchaoxue_used_play".to_string()));
        self.core
            .vm_aux
            .flags
            .remove(&(owner, "gw_xianzumaijiu_used_play".to_string()));
    }
}
