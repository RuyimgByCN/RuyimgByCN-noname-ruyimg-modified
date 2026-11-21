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
                .player(self.source)
                .equips
                .iter()
                .any(|c| ctx.catalog.card_kind(*c) == CardKind::Qinggang))
            || ctx
                .catalog
                .skill_by_name(crate::vocabulary::names::BOSS_ZHADAO)
                .map(|def| ctx.core.has_skill(self.source, def))
                .unwrap_or(false)
            || ctx
                .core
                .ignores_target_armor(ctx.catalog, self.source, self.target, self.card, self.nature))
    }

    /// Runs the once-per-Sha Bagua Shield judge when `can_bagua` holds and returns `Some(Flow::Done)`
    /// if the judge succeeds.
    pub(super) fn try_bagua_dodge(&mut self, ctx: &mut Ctx<'_>, can_bagua: bool) -> Option<Flow> {
        let has_armor = can_bagua
            && !crate::kernel::mbcmfangzhu_baiban(ctx.core, self.target)
            && ctx.core.player(self.target).alive
            && ctx
                .core
                .player(self.target)
                .equips
                .iter()
                .any(|c| crate::rvm_bridge::modifiers::d34_l28_mods::is_bagua_armor(ctx.catalog, *c));
        if !has_armor {
            return None;
        }
        let judged = ctx.draw_judge_card();
        let j_suit = ctx.catalog.cards[judged as usize].suit;
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
        ctx.to_discard(judged);
        if success {
            ctx.emit(EventData::ShaDodged {
                card: self.card,
                source: self.source,
                target: self.target,
            });
            return Some(self.finish_or_defer_sha_end(ctx, false));
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
            .map(|c| ctx.catalog.effective_card_suit(ctx.core, self.source, c));
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
                    && equips
                        .iter()
                        .any(|c| ctx.catalog.card_kind(*c) == CardKind::Renwang);
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
