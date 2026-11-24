use super::*;

impl ShaResolveMachine {
    /// Returns whether the attacker ignores the target's armor (Qinggang, Boss Zhadao, etc.),
    /// unless prevented by rewritten armor like Lanyinjia.
    pub(super) fn qinggang_pierces(&self, ctx: &Ctx<'_>) -> bool {
        !crate::rvm_bridge::modifiers::d34_l28_mods::has_rewritten_armor(
            ctx.core,
            ctx.catalog,
            self.target,
            "lanyinjia",
        ) && ((!crate::kernel::mbcmfangzhu_baiban(ctx.core, self.source)
            && ctx
                .core
                .has_active_equip_kind(ctx.catalog, self.source, CardKind::Qinggang))
            || ctx
                .catalog
                .skill_by_name(crate::vocabulary::names::BOSS_ZHADAO)
                .map(|def| ctx.core.has_skill(self.source, def))
                .unwrap_or(false)
            || (self.baolei_sentinel(ctx)
                && ctx
                    .catalog
                    .skill_by_name(crate::vocabulary::host::WUDIAN_BAOLEI_KEY)
                    .is_some_and(|def| {
                        ctx.catalog.skills[def as usize]
                            .behavior
                            .as_host_behavior()
                            == Some(crate::vocabulary::host::HostBehavior::U05OwBatchA)
                    }))
            || ctx
                .core
                .ignores_target_armor(ctx.catalog, self.source, self.target, self.card, self.nature))
    }

    /// Runs the once-per-Sha Bagua Shield judge when `can_bagua` holds and returns `Some(Flow::Done)`
    /// if the judge succeeds.
    pub(super) fn try_bagua_dodge(&mut self, ctx: &mut Ctx<'_>, can_bagua: bool) -> Option<Flow> {
        let has_equipped_armor = ctx
            .core
            .player(self.target)
            .equips
            .iter()
            .any(|c| ctx.catalog.card_slot_of(ctx.core, *c) == Some(crate::catalog::EquipSlot::Armor));
        let has_bazhen = !has_equipped_armor
            && ["bazhen", "八阵"]
                .into_iter()
                .filter_map(|n| ctx.catalog.skill_by_name(n))
                .any(|def| ctx.core.has_skill(self.target, def));
        let has_armor = can_bagua
            && !crate::kernel::mbcmfangzhu_baiban(ctx.core, self.target)
            && ctx.core.player(self.target).alive
            && (has_bazhen
                || ctx
                    .core
                    .player(self.target)
                    .equips
                    .iter()
                    .any(|c| {
                        !ctx.core.is_gw_dieyi_masked(ctx.catalog, self.target, *c)
                            && crate::rvm_bridge::modifiers::d34_l28_mods::is_bagua_armor(
                                ctx.core,
                                ctx.catalog,
                                *c,
                            )
                    })
                || crate::rvm_bridge::modifiers::eval_virtual_bagua_if_armor_slot_empty(
                    ctx.core,
                    ctx.catalog,
                    self.target,
                ));
        if !has_armor {
            return None;
        }
        let pref = if crate::rvm_bridge::modifiers::d34_l28_mods::rewrite_bagua_judge_ok(
            ctx.core,
            ctx.catalog,
            self.target,
            crate::catalog::Suit::Heart,
        )
        .is_some()
        {
            crate::kernel::EloJudgePref::NonSpadePositive
        } else {
            crate::kernel::EloJudgePref::HeartPositive
        };
        if ctx.core.deck.is_empty() && ctx.core.discard.is_empty() {
            return None;
        }
        let judged = ctx.draw_judge_card_with_elo(self.target, pref);
        let eff_suit = ctx.catalog.effective_card_suit(ctx.core, self.target, judged);
        let raw_rank = ctx.catalog.cards[judged as usize].rank;
        let (j_suit, _) = if ctx.catalog.vm_card_history {
            crate::rvm_bridge::modifiers::apply_huangtian_suit_rank(
                ctx.core,
                ctx.catalog,
                self.target,
                eff_suit,
                raw_rank,
            )
        } else {
            (eff_suit, raw_rank)
        };
        let success = crate::rvm_bridge::modifiers::d34_l28_mods::rewrite_bagua_judge_ok(
            ctx.core,
            ctx.catalog,
            self.target,
            j_suit,
        )
        .unwrap_or_else(|| j_suit.is_heart());
        ctx.log(LogEvent::ArmorJudged {
            player: self.target,
            card: judged,
            success,
        });
        // 判定牌判定后即弃置（八卦阵判定无获得路径）
        let settled_spawned = ctx.settle_inline_judge_card(self.target, j_suit, judged);
        if success && !self.shanxie_excludes_shan(ctx, None) {
            if let Some(flow) = self.try_open_sadeng_on_miss(ctx) {
                return Some(flow);
            }
            return Some(self.after_sha_dodged(ctx));
        }
        if settled_spawned {
            return Some(Flow::Yield);
        }
        None
    }

    /// Returns whether Renwang Shield (or Boss Tiemian) negates this Sha.
    pub(super) fn renwang_blocks(&self, ctx: &Ctx<'_>, qinggang_pierce: bool) -> bool {
        if qinggang_pierce {
            return false;
        }
        let eff_suit = self
            .card
            .and_then(|c| crate::kernel::effective_sha_suit(ctx.core, ctx.catalog, self.source, c));
        let black = matches!(
            eff_suit,
            Some(crate::catalog::Suit::Spade | crate::catalog::Suit::Club)
        );
        let rw_blocks = crate::rvm_bridge::modifiers::d34_l28_mods::rewrite_renwang_blocks(
            ctx.core,
            ctx.catalog,
            self.target,
            eff_suit,
        );
        rw_blocks
            || (black && {
                let equips = &ctx.core.player(self.target).equips;
                let has_renwang = !crate::kernel::mbcmfangzhu_baiban(ctx.core, self.target)
                    && ctx
                        .core
                        .has_active_equip_kind(ctx.catalog, self.target, CardKind::Renwang);
                let tiemian_active = ctx
                    .catalog
                    .skill_by_name(crate::vocabulary::names::BOSS_TIEMIAN)
                    .map(|def| ctx.core.has_skill(self.target, def))
                    .unwrap_or(false)
                    && !equips.iter().any(|c| {
                        ctx.catalog.card_slot_of(ctx.core, *c)
                            == Some(crate::catalog::EquipSlot::Armor)
                    });
                has_renwang || tiemian_active
            })
    }
}
