//! 武典（wudian）宿主技能的主动发动面。

use super::super::*;

impl HostSkillBehavior {
    pub(super) fn activate_wudian(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
    ) -> Option<Box<dyn crate::catalog::SkillMachine>> {
        match self.behavior {
            // 屯田·DIY：有装备牌（手/场）才可发动
            HostBehavior::WudianDiyjuntun => {
                let has = !ctx.state.player(owner).equips.is_empty()
                    || ctx
                        .state
                        .player(owner)
                        .hand
                        .iter()
                        .any(|c| ctx.catalog.card_slot_of(ctx.state, *c).is_some());
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
            HostBehavior::WudianPaotai => {
                // 炮台（U05 ow v123）：手牌有杀且等级 <3 才可发动
                if !ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|c| ctx.catalog.card_kind(*c).is_sha_like())
                    || ctx.state.player(owner).paotai_lv >= 3
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
                    stage: HostStage::Act,
                }))
            }
            HostBehavior::WudianShouhu => {
                // 守护（U07 ow v121）：手牌有杀才可发动
                if !ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|c| ctx.catalog.card_kind(*c).is_sha_like())
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
                    stage: HostStage::Act,
                }))
            }
            HostBehavior::WudianBaoleiSwitch => {
                // 堡垒/侦察切换（U05 ow v124）：无前置（上游 filter 仅
                // zhencha2 互斥锁——原生以机器推进天然互斥）
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
            HostBehavior::WudianKangyin => {
                // 抗yin（U07 批四 v114）：存在其他存活且 he 非空目标
                if !ctx.state.alive_players().any(|p| {
                    p != owner
                        && (!ctx.state.player(p).hand.is_empty()
                            || !ctx.state.player(p).equips.is_empty())
                }) {
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
            HostBehavior::WudianJuedao => {
                // 绝道（U07 批六 v116）：未横置才可发动（上游 filter
                // isLinked()===false 同口径）
                if ctx.state.player(owner).linked {
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
            HostBehavior::WudianMujing => {
                // 目镜（U07 ow v118）：手牌有黑色牌才可发动（上游
                // filterCard 黑色 + viewAsFilter 同口径）
                if !ctx.state.player(owner).hand.iter().any(|c| {
                    matches!(
                        ctx.catalog.cards[*c as usize].suit,
                        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                    )
                }) {
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
            HostBehavior::WudianGuangshu => {
                // 光书（U07 ow v119）：he 有牌才可发动
                if ctx.state.player(owner).hand.is_empty()
                    && ctx.state.player(owner).equips.is_empty()
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
                    stage: HostStage::Act,
                }))
            }
            HostBehavior::WudianZiyu => {
                // 自愈（U07 ow v120）：触发技经 OptionalGate 确认
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
            HostBehavior::WudianKuoyin => {
                let t = &ctx.state.turn;
                if t.yuedong_eff > 0 && t.yuedong_num > 0 {
                    return None;
                }
                let p = ctx.state.player(owner);
                if p.hand.is_empty() && p.equips.is_empty() {
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
            HostBehavior::WudianHuhuan => {
                let p = ctx.state.player(owner);
                if p.hand.len() + p.equips.len() < 2 || ctx.state.turn.yuedong_recover {
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
            HostBehavior::WudianBaoxue => {
                if ctx.state.player(owner).baoxue_used {
                    return None;
                }
                let p = ctx.state.player(owner);
                let black = p
                    .hand
                    .iter()
                    .chain(p.equips.iter())
                    .filter(|c| {
                        matches!(
                            ctx.catalog.cards[**c as usize].suit,
                            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                        )
                    })
                    .count();
                if black == 0 {
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
            HostBehavior::WudianLiangou => {
                let p = ctx.state.player(owner);
                if p.hand.is_empty() && p.equips.is_empty() {
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
            HostBehavior::WudianYinbo => {
                let p = ctx.state.player(owner);
                let has_spade = p.hand.iter().chain(p.equips.iter()).any(|c| {
                    matches!(
                        ctx.catalog.cards[*c as usize].suit,
                        crate::catalog::Suit::Spade
                    )
                });
                if !has_spade {
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
            HostBehavior::WudianQianggu => {
                let p = ctx.state.player(owner);
                if p.hand.len() + p.equips.len() < 2 {
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
            HostBehavior::WudianFeitiao => {
                let p = ctx.state.player(owner);
                if p.hand.is_empty() && p.equips.is_empty() {
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
            HostBehavior::WudianChoudu => {
                let p = ctx.state.player(owner);
                if p.hand.is_empty() && p.equips.is_empty() {
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
