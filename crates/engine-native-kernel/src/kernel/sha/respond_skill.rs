use super::*;
use crate::ids::CardId;

impl ShaResolveMachine {
    /// Upstream `shanxie_exclude` (`shiji.js:1404`): a Shan (or virtual Shan) whose rank is 0 or <= 2 * owner.attack_range is excluded.
    pub(super) fn shanxie_excludes_shan(&self, ctx: &mut Ctx<'_>, shan_card: Option<CardId>) -> bool {
        let Some(sdef) = ctx.catalog.skill_by_name("shanxie") else {
            return false;
        };
        if ctx.catalog.skills[sdef as usize].behavior.as_host_behavior()
            != Some(crate::vocabulary::host::HostBehavior::L11VmFixB)
        {
            return false;
        }
        let ex_def = ctx.catalog.skill_by_name("shanxie_exclude");
        let active = ctx.core.has_skill(self.source, sdef)
            || ex_def.is_some_and(|ed| ctx.core.has_skill(self.source, ed));
        if !active
            || !ctx.core.player(self.source).alive
            || ctx.catalog.is_chanyuan_sealed(ctx.core, self.source)
            || ctx.catalog.is_tieji_sealed(ctx.core, self.source)
            || ctx.catalog.is_xj_fengyin_sealed(ctx.core, self.source)
        {
            return false;
        }
        let rank = shan_card
            .and_then(|c| ctx.catalog.resolved_card_def(ctx.core, c).map(|d| d.rank as i32))
            .unwrap_or(0);
        let atk = ctx.core.attack_range(ctx.catalog, self.source);
        if rank == 0 || u32::try_from(rank).is_ok_and(|r| r <= atk.saturating_mul(2)) {
            ctx.log(LogEvent::SkillTriggered {
                skill: ctx
                    .catalog
                    .skill_by_name("shanxie_exclude")
                    .map_or("shanxie_exclude", |d| ctx.catalog.skills[d as usize].name),
                owner: self.source,
                presentation: None,
            });
            return true;
        }
        false
    }

    pub(super) fn complete_shan_dodge(&mut self, ctx: &mut Ctx<'_>, shan_card: Option<CardId>) -> Flow {
        // 雷击（Boss 青龙，规则 v234）：响应闪后判定雷击（无双第二闪重入本臂=每张闪各触发一次）
        super::super::releiji_on_shan(ctx, self.target);
        if !self.shanxie_excludes_shan(ctx, shan_card) {
            self.dodges_used += 1;
        }
        self.post_dodge_step(ctx)
    }

    /// Consumes a yielded post-`CardResponded` dodge transition (`Hujia` or target `Shan`)
    /// after `CardResponded` has finished settling.
    pub(super) fn consume_u01_pending_dodge(&mut self, ctx: &mut Ctx<'_>) -> Option<Flow> {
        let fid = self.current_sha_frame_id(ctx)?;
        if ctx
            .core
            .vm_aux
            .flags
            .remove(&(self.target, format!("sha_u01_hujia_dodged:{fid}")))
            .is_some()
        {
            ctx.core.turn.liaoyuan_count = 0;
            if let Some(flow) = self.try_open_sadeng_on_miss(ctx) {
                return Some(flow);
            }
            ctx.emit(EventData::ShaDodged {
                card: self.card,
                source: self.source,
                target: self.target,
            });
            return Some(self.finish_or_defer_sha_end(ctx, false));
        }
        if ctx
            .core
            .vm_aux
            .flags
            .remove(&(self.target, format!("sha_u01_shan_pending:{fid}")))
            .is_some()
        {
            let shan_card = ctx
                .core
                .vm_aux
                .flags
                .remove(&(self.target, format!("sha_u01_shan_pending_card:{fid}")))
                .and_then(|v| v.checked_sub(1));
            return Some(self.complete_shan_dodge(ctx, shan_card));
        }
        None
    }

    /// Consumes a parked U01 response-skill outcome and advances resolution.
    pub(super) fn handle_u01_respond_park(&mut self, ctx: &mut Ctx<'_>) -> Option<Flow> {
        if let Some(flow) = self.consume_u01_pending_dodge(ctx) {
            return Some(flow);
        }
        let ok = crate::kernel::take_u01_respond_result(ctx, self.target)?;
        if ok {
            ctx.core
                .vm_aux
                .flags
                .remove(&(self.target, "dddfusi_window_refused".to_string()));
            let emit_use = ctx
                .core
                .vm_aux
                .flags
                .remove(&(self.target, "u01_respond_emit_use".to_string()))
                .is_some();
            if emit_use {
                let fid = self.current_sha_frame_id(ctx).unwrap_or(0);
                let card_ovr = ctx
                    .core
                    .vm_aux
                    .flags
                    .remove(&(self.target, "u01_respond_card_override".to_string()))
                    .and_then(|v| v.checked_sub(1));
                let fusi_owner = ctx
                    .core
                    .vm_aux
                    .flags
                    .remove(&(self.target, "u01_respond_fusi_owner".to_string()))
                    .unwrap_or(0);
                let new_fid = ctx.res.next_event_id;
                ctx.core.vm_aux.flags.insert(
                    (self.target, format!("sha_u01_shan_pending:{fid}")),
                    1,
                );
                ctx.core.vm_aux.flags.insert(
                    (self.target, format!("sha_u01_shan_pending_card:{fid}")),
                    card_ovr.map_or(0, |c| c.saturating_add(1)),
                );
                ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                    card: card_ovr,
                    player: self.target,
                    targets: Vec::new(),
                    nature: crate::event::DamageNature::Normal,
                    use_as: Some(CardKind::Shan),
                    cancelled: false,
                }));
                if fusi_owner > 0 {
                    ctx.core.vm_aux.flags.insert(
                        (self.target, format!("dddfusi_used_owner:{new_fid}")),
                        fusi_owner,
                    );
                }
                return Some(Flow::Yield);
            }
            Some(self.complete_shan_dodge(ctx, None))
        } else {
            self.asked = false;
            None
        }
    }

    /// Collects U01 respond-skill options (`dddxiaxing`, `dddbingjian`, `dddshilie`) when `Shan` can be played.
    pub(super) fn collect_u01_respond_skill_options(
        &self,
        ctx: &Ctx<'_>,
        target_can_play: bool,
    ) -> Vec<ChoiceOption> {
        if target_can_play {
            (ctx.catalog.hooks.respond_skill_options)(
                ctx.core,
                ctx.catalog,
                self.target,
                CardKind::Shan,
                true,
            )
        } else {
            Vec::new()
        }
    }

    /// Dispatches a selected `ChoiceOption::Skill` during `Sha` dodge asking.
    pub(super) fn try_handle_u01_respond_skill(
        &mut self,
        ctx: &mut Ctx<'_>,
        def: crate::ids::SkillDefId,
    ) -> Option<Flow> {
        let fid = self.current_sha_frame_id(ctx).unwrap_or(0);
        let immediate = (ctx.catalog.hooks.try_activate_respond_skill)(
            ctx,
            self.target,
            def,
            CardKind::Shan,
            fid,
        )?;
        if immediate {
            self.asked = false;
            let fid = self.current_sha_frame_id(ctx).unwrap_or(0);
            ctx.core
                .vm_aux
                .flags
                .remove(&(self.target, "dddfusi_window_refused".to_string()));
            ctx.core
                .vm_aux
                .flags
                .remove(&(self.target, "u01_respond_emit_use".to_string()));
            let card_ovr = ctx
                .core
                .vm_aux
                .flags
                .remove(&(self.target, "u01_respond_card_override".to_string()))
                .and_then(|v| v.checked_sub(1));
            let fusi_owner = ctx
                .core
                .vm_aux
                .flags
                .remove(&(self.target, "u01_respond_fusi_owner".to_string()))
                .unwrap_or(0);
            let new_fid = ctx.res.next_event_id;
            ctx.core.vm_aux.flags.insert(
                (self.target, format!("sha_u01_shan_pending:{fid}")),
                1,
            );
            ctx.core.vm_aux.flags.insert(
                (self.target, format!("sha_u01_shan_pending_card:{fid}")),
                card_ovr.map_or(0, |c| c.saturating_add(1)),
            );
            ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                card: card_ovr,
                player: self.target,
                targets: Vec::new(),
                nature: crate::event::DamageNature::Normal,
                use_as: Some(CardKind::Shan),
                cancelled: false,
            }));
            if fusi_owner > 0 {
                ctx.core.vm_aux.flags.insert(
                    (self.target, format!("dddfusi_used_owner:{new_fid}")),
                    fusi_owner,
                );
            }
            return Some(Flow::Yield);
        }
        self.asked = false;
        Some(Flow::Yield)
    }
}
