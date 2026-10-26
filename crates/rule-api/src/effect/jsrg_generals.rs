//! Jsrg generals effect definitions.

macro_rules! variants {
    (ResolveJsrgchiying [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 驰应（江山如故高翔）：目标攻击范围内除技能拥有者外的角色依座次各弃一张牌；
    /// 若其中基本牌数不大于目标体力，目标获得仍在弃牌堆的这些基本牌。ABI 501。
    ResolveJsrgchiying {
        /// Skill owner.
        player: PlayerId,
        /// Active-skill target whose range determines the forced discarders.
        target: PlayerId,
        /// Whether the fixed seat-ordered discarder list has already been captured.
        started: bool,
        /// Remaining forced discarders, captured before the first discard.
        remaining: Vec<PlayerId>,
        /// Cards discarded by this resolution.
        accum: Vec<CardId>,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveJsrgdaoren [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 专属状态机 (批 240): jsrgdaoren 蹈仁 — give one hand card to another
    /// player, then deal 1 damage to every player in both attack ranges.
    ResolveJsrgdaoren {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
    (ResolveJsrgzuozhan [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 专属状态机 (批 242): jsrgzuozhan 坐瞻 — at game start choose up to two
    /// other players; gain an attack-range mark equal to min(highest HP, 5).
    ResolveJsrgzuozhan {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 专属状态机 (批 243): jsrgzuozhan gain group face — after a marked ally
    /// dies, one surviving 坐瞻 ally gains up to X distinct-name basic cards
    /// from the discard pile.
    ResolveJsrgzuozhanGain {
        /// Owning player.
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
        /// Dying player bound to the death event.
        event_player: PlayerId,
    },
        ] }
    };
    (ResolveJsrgchaozheng [$($tail:tt)*] [$($variants:tt)*]) => {
        collect_variants! { [$($tail)*] [
            $($variants)*
    /// 朝争议事结算（ABI 653，ADR 0644 P1）：全参与者亮首张手牌分组多数决。
    ResolveJsrgchaozheng {
        /// 发起者。
        player: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
    /// 设居双人议事结算（ABI 654，ADR 0644 P1）：黑多数则双方各减 1 体力上限。
    ResolveJsrgsheju {
        /// 发动者。
        player: PlayerId,
        /// 议事对方。
        counterpart: PlayerId,
        /// Originating skill instance.
        skill: SkillId,
    },
        ] }
    };
}
pub(super) use variants;
