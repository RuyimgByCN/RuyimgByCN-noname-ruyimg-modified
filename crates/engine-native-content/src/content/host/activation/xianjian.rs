//! 仙剑（xianjian）宿主技能的主动发动面。

use super::super::*;

impl HostSkillBehavior {
    pub(super) fn activate_xianjian(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
    ) -> Option<Box<dyn crate::catalog::SkillMachine>> {
        match self.behavior {
            HostBehavior::XianjianXtanxi => {
                if ctx.state.player(owner).hand.is_empty()
                    || !ctx
                        .state
                        .alive_players()
                        .any(|t| t != owner && !ctx.state.player(t).hand.is_empty())
                {
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
            HostBehavior::XianjianHuahu => {
                if ctx.state.player(owner).huahu_used {
                    return None;
                }
                let has_other = ctx.state.alive_players().any(|t| t != owner);
                if !has_other {
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
                    stage: HostStage::TargetChoice,
                }))
            }
            HostBehavior::XianjianLuanjian => {
                let sha_count = ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .filter(|c| ctx.catalog.card_kind(**c) == crate::catalog::CardKind::Sha)
                    .count();
                if sha_count < 2 {
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
            HostBehavior::XianjianCtianfu => {
                let has_shan = ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Shan);
                let Some(ctianfu2) = ctx.catalog.skill_by_name("ctianfu2") else {
                    return None;
                };
                let has_target = ctx
                    .state
                    .alive_players()
                    .any(|t| t != owner && !ctx.state.skills.contains_key(&(t, ctianfu2)));
                if !has_shan || !has_target {
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
            HostBehavior::XianjianTianjian => {
                let has_sha = ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Sha);
                if !has_sha {
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
            HostBehavior::XianjianSajin => {
                let p = ctx.state.player(owner);
                let wounded = ctx
                    .state
                    .alive_players()
                    .any(|t| (ctx.state.player(t).hp as u32) < ctx.state.player(t).max_hp);
                if p.hand.is_empty() || !wounded {
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
            HostBehavior::XianjianMinsha => {
                let enough =
                    ctx.state.player(owner).hand.len() + ctx.state.player(owner).equips.len() >= 2;
                let has_target = ctx
                    .state
                    .alive_players()
                    .any(|t| t != owner && ctx.state.player(t).hp > 1);
                if !enough || !has_target {
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
            HostBehavior::XianjianZhuyue => {
                let has_nonbasic = ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|c| yangpijuan_class(ctx.catalog.card_kind(*c)) != Some(0));
                let has_target = ctx.state.alive_players().any(|t| {
                    t != owner
                        && (!ctx.state.player(t).hand.is_empty()
                            || !ctx.state.player(t).equips.is_empty())
                });
                if !has_nonbasic || !has_target {
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
            HostBehavior::XianjianGuanri => {
                if ctx.state.player(owner).guanri_used {
                    return None;
                }
                let reds = ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .filter(|c| {
                        matches!(
                            ctx.catalog.cards[**c as usize].suit,
                            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                        )
                    })
                    .count();
                let has_target = ctx
                    .state
                    .alive_players()
                    .any(|t| t != owner && ctx.state.player(t).hp >= ctx.state.player(owner).hp);
                if reds < 2 || !has_target {
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
            HostBehavior::XianjianDiewu => {
                let has_sha = ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|c| ctx.catalog.card_kind(*c).is_sha_like());
                if !has_sha {
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
            HostBehavior::XianjianGuiyuan => {
                let p = ctx.state.player(owner);
                let has_sha = p
                    .hand
                    .iter()
                    .chain(p.equips.iter())
                    .any(|c| ctx.catalog.card_kind(*c).is_sha_like());
                if !has_sha || (p.hp as u32) >= p.max_hp {
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
            HostBehavior::XianjianGuijin => {
                if ctx.state.turn.round < ctx.state.player(owner).guijin_ready_round {
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
            // 玄墨（U05 xianjian 夏侯瑾轩，规则 v174）：手牌中有三类牌
            //（usable:1 走命令层 once_used）
            HostBehavior::XianjianXuanmo => {
                let has = ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|c| yangpijuan_class(ctx.catalog.card_kind(*c)).is_some());
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
                    stage: HostStage::SelectCard,
                }))
            }
            HostBehavior::XianjianYuexing => {
                let has_other = ctx.state.alive_players().any(|t| t != owner);
                if !has_other {
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
            _ => unreachable!("behavior routed to the wrong activate module"),
        }
    }
}
