//! U05-7d 杀海拾遗 swd 补充主动技激活检查（`activation/swd_u05_7d.rs`）。

use super::super::*;
use super::super::step::swd_u05_7d::same_suit_hand_candidates;

impl HostSkillBehavior {
    pub(super) fn activate_swd_u05_7d(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
    ) -> Option<Box<dyn crate::catalog::SkillMachine>> {
        if !matches!(ctx.state.turn.phase, crate::event::Phase::Play)
            || ctx.state.turn.current != owner
        {
            return None;
        }
        let ok = match self.behavior {
            HostBehavior::SwdDuijue => {
                ctx.state
                    .vm_aux
                    .flags
                    .get(&(owner, "duijue_used".to_string()))
                    .copied()
                    .unwrap_or(0)
                    == 0
                    && ctx.state.alive_players().any(|p| {
                        p != owner
                            && ctx.state.player(p).hp > 1
                            && !ctx.catalog.has_undist(ctx.state, p)
                    })
            }
            HostBehavior::SwdTanlin => {
                !ctx.state.player(owner).hand.is_empty()
                    && ctx.state.alive_players().any(|p| {
                        p != owner
                            && !ctx.state.player(p).hand.is_empty()
                            && !ctx.catalog.has_undist(ctx.state, p)
                    })
            }
            HostBehavior::SwdOldLiaoyuan | HostBehavior::SwdOldLiaoyuan2 => {
                same_suit_hand_candidates(ctx.catalog, ctx.state, owner).len() >= 2
                    && ctx
                        .state
                        .alive_players()
                        .any(|p| p != owner && !ctx.catalog.has_undist(ctx.state, p))
            }
            _ => false,
        };
        if !ok {
            return None;
        }
        Some(Box::new(HostMachine {
            def_key: self.name.clone(),
            behavior: self.behavior,
            owner,
            target: None,
            card: None,
            cards: Vec::new(),
            targets: Vec::new(),
            stage: HostStage::Act,
        }))
    }

    /// 剑系补充技能尾段激活检查（自 `activation/swd_skills.rs` 拆出以保持单文件行数上限）。
    pub(super) fn activate_swd_tail(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
    ) -> Option<Box<dyn crate::catalog::SkillMachine>> {
        match self.behavior {
            // 分舵：存在手牌多于自己的存活角色才可发动
            HostBehavior::SwdJiefen => {
                let my = ctx.state.player(owner).hand.len();
                let has = ctx
                    .state
                    .alive_players()
                    .any(|p| p != owner && ctx.state.player(p).hand.len() > my);
                if !has {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::JiefenTarget,
                }))
            }
            // 断月：有装备牌（手/装备区）且存在其他存活角色
            HostBehavior::SwdDuanyue => {
                let has = !ctx.state.player(owner).equips.is_empty()
                    || ctx
                        .state
                        .player(owner)
                        .hand
                        .iter()
                        .any(|c| ctx.catalog.card_slot_of(ctx.state, *c).is_some());
                let others = ctx.state.alive_players().any(|p| p != owner);
                if !has || !others {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            }
            // 星点：有手牌且存在其他有牌角色
            HostBehavior::SwdXingdian => {
                let has = !ctx.state.player(owner).hand.is_empty();
                let others = ctx.state.alive_players().any(|p| {
                    p != owner
                        && (!ctx.state.player(p).hand.is_empty()
                            || !ctx.state.player(p).equips.is_empty())
                });
                if !has || !others {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::SelectCard,
                }))
            }
            // 雪煌：limited 未发动、手牌全红且非空
            HostBehavior::SwdXuehuang => {
                let p = ctx.state.player(owner);
                let all_red = !p.hand.is_empty()
                    && p.hand.iter().all(|c| {
                        matches!(
                            ctx.catalog.cards[*c as usize].suit,
                            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                        )
                    });
                if p.xuehuang_used || !all_red {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: None,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            }
            // 悲歌：确认门由 optional 承担（activate 恒供机）
            HostBehavior::SwdTiebi | HostBehavior::SwdDatong => Some(Box::new(HostMachine {
                def_key: self.name.clone(),
                behavior: self.behavior,
                owner,
                target: None,
                card: None,
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::Act,
            })),
            _ => unreachable!("behavior routed to the wrong activate module"),
        }
    }
}
