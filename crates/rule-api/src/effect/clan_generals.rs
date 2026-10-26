//! Clan generals effect definitions.

macro_rules! variants {
    (ClanshenjunReveal [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 神君 (clanshenjun): reveal and gaintag every hand card sharing the event
    /// card's effective kind. Implements upstream 同名牌展示 + addGaintag.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ClanshenjunReveal {
        /// Skill owner.
        player: PlayerId,
        /// The card whose kind is matched (the trigger's used card).
        event_card: CardId,
        /// Skill instance whose definition id becomes the tag.
        skill: SkillId,
    },
    /// 神君 (clanshenjun) phase-end use: open the tagged-card + target choice
    /// and play the selected card as itself.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ClanshenjunUse {
        /// Skill owner.
        player: PlayerId,
        /// Skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ClanyuzhiReveal [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 迂志 round-start: the owner chooses one hand card to reveal; the resolver reveals +
    /// gaintags it, draws its name length (tagging the draws), and records round draw counts.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ClanyuzhiReveal {
        /// Skill owner.
        player: PlayerId,
        /// 迂志 skill instance.
        skill: SkillId,
    },
    /// 迂志 round-end: discard every gaintagged hand card, then if this round's 迂志 draws
    /// exceeded the previous round's (both positive) or the round's used-card count, the owner
    /// loses 保族 or takes 1 thunder damage.
    ///
    /// Appended so existing serialized effect variant ordinals remain stable.
    ClanyuzhiRoundEnd {
        /// Skill owner.
        player: PlayerId,
        /// 迂志 skill instance.
        skill: SkillId,
    },
        ] }
    };
}
pub(super) use variants;
