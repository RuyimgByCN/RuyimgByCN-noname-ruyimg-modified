//! 宿主技能的主动发动面（通用、史诗、轩辕剑技能、古剑、运筹帷幄与其他包）。

use super::super::*;

impl HostSkillBehavior {
    pub(super) fn activate_common(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
    ) -> Option<Box<dyn crate::catalog::SkillMachine>> {
        match self.behavior {
            // 轩辕（swd 批 146）：结束阶段触发（无主动面）
            HostBehavior::Xuanyuan => None,
            // 苦肉（L20 批 2）：出牌阶段主动
            HostBehavior::Kurou => {
                if !matches!(ctx.state.turn.phase, crate::event::Phase::Play)
                    || ctx.state.turn.current != owner
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
            // 集舍（L27 批 5）：出牌阶段主动（手牌上限 > 0）
            HostBehavior::Jishe => {
                if !matches!(ctx.state.turn.phase, crate::event::Phase::Play)
                    || ctx.state.turn.current != owner
                    || ctx.state.hand_limit(ctx.catalog, owner) == 0
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
            // 蛊惑（史诗卡牌批 1）：出牌阶段主动（每回合一次；手牌非空）
            HostBehavior::Reguhuo => {
                if !matches!(ctx.state.turn.phase, crate::event::Phase::Play)
                    || ctx.state.turn.current != owner
                    || ctx.state.player(owner).hand.is_empty()
                {
                    return None;
                }
                // 蛊惑：**每回合一次**（第 942 轮接线：`vm_aux.once` scope 1 = 本回合）
                if ctx
                    .state
                    .vm_aux
                    .once
                    .contains(&(owner, self.name.clone(), 1))
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
            // 攻心（swd 批 155）：出牌阶段主动（每回合一次；存在有手牌的其他角色）
            HostBehavior::Gongxin => {
                if !matches!(ctx.state.turn.phase, crate::event::Phase::Play)
                    || ctx.state.turn.current != owner
                    || !ctx
                        .state
                        .alive_players()
                        .any(|p| p != owner && !ctx.state.player(p).hand.is_empty())
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
            // 魔焰（swd 批 151）：出牌阶段主动（每回合一次；体力未满且有红牌）
            HostBehavior::Moyan => {
                if !matches!(ctx.state.turn.phase, crate::event::Phase::Play)
                    || ctx.state.turn.current != owner
                {
                    return None;
                }
                let p = ctx.state.player(owner);
                let has_red = p.hand.iter().any(|c| {
                    matches!(
                        ctx.catalog.cards[*c as usize].suit,
                        crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                    )
                });
                if p.hp >= p.max_hp as i32 || !has_red {
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
            // 极略（swd 批 136）：出牌阶段主动（每阶段 3 次）
            HostBehavior::Jilve => {
                if !matches!(ctx.state.turn.phase, crate::event::Phase::Play)
                    || ctx.state.turn.current != owner
                    || ctx.state.player(owner).jilve_used >= 3
                    || ctx.state.deck.is_empty()
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
            // 革新印（swd 批 135）：出牌阶段主动（手牌非空）
            HostBehavior::Gxianyin => {
                if !matches!(ctx.state.turn.phase, crate::event::Phase::Play)
                    || ctx.state.turn.current != owner
                    || ctx.state.player(owner).hand.is_empty()
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
            // 冰封（swd 批 134）：出牌阶段主动（限定——实例启用时）
            HostBehavior::Bingfeng => {
                if !matches!(ctx.state.turn.phase, crate::event::Phase::Play)
                    || ctx.state.turn.current != owner
                {
                    return None;
                }
                let enabled = ctx
                    .catalog
                    .skill_by_name(SWD_BINGFENG_KEY)
                    .and_then(|d| ctx.state.skills.get(&(owner, d)).map(|i| i.enabled))
                    .unwrap_or(false);
                if !enabled
                    || !ctx
                        .state
                        .alive_players()
                        .any(|p| p != owner && !ctx.state.player(p).face_down)
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
            // 魂殉志（swd 批 132）：出牌阶段主动（限定——实例启用时）
            HostBehavior::Hxunzhi => {
                if !matches!(ctx.state.turn.phase, crate::event::Phase::Play)
                    || ctx.state.turn.current != owner
                {
                    return None;
                }
                let enabled = ctx
                    .catalog
                    .skill_by_name(SWD_HXUNZHI_KEY)
                    .and_then(|d| ctx.state.skills.get(&(owner, d)).map(|i| i.enabled))
                    .unwrap_or(false);
                if !enabled {
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
            // 颂词（swd 批 129）：出牌阶段主动——选未记录角色（含自己）
            HostBehavior::Songci => {
                if !matches!(ctx.state.turn.phase, crate::event::Phase::Play)
                    || ctx.state.turn.current != owner
                {
                    return None;
                }
                let recorded = &ctx.state.player(owner).songci_targets;
                if !ctx.state.alive_players().any(|p| !recorded.contains(&p)) {
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
            // 涅槃（T11 Boss 庞统族批 118）：出牌阶段主动面——限定技未发动
            // 时可用（濒死面走触发路径；两面共用收束）。
            HostBehavior::Niepan => {
                if ctx.state.player(owner).niepan_used {
                    return None;
                }
                if !matches!(ctx.state.turn.phase, crate::event::Phase::Play)
                    || ctx.state.turn.current != owner
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
            // 五行盘：出牌阶段限一次（阶段门控在命令边界），需手牌非空
            HostBehavior::ChangeCardNature => {
                if ctx.state.player(owner).hand.is_empty() {
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
            HostBehavior::GujianFoodLuyugeng => {
                let has_basic = ctx.state.player(owner).hand.iter().any(|c| {
                    matches!(
                        ctx.catalog.card_kind(*c),
                        CardKind::Sha | CardKind::Shan | CardKind::Tao | CardKind::Jiu
                    )
                });
                if !has_basic {
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
            HostBehavior::GujianYuhengEquipSkill => {
                let has = ctx.state.alive_players().any(|p| {
                    p != owner
                        && ctx.state.player(p).hand.iter().any(|c| {
                            matches!(
                                ctx.catalog.cards[*c as usize].suit,
                                crate::catalog::Suit::Spade
                            )
                        })
                });
                if !has {
                    return None;
                }
                // 潜偷同款校验已在 SwdShentouEquipSkill 臂单独实现
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
            HostBehavior::YunchouBeige => Some(Box::new(HostMachine {
                def_key: self.name.clone(),
                behavior: self.behavior,
                owner,
                target: None,
                card: None,
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::Act,
            })),
            // 隙焰：有红手牌才可发动
            HostBehavior::YunchouJieyan => {
                let has = ctx.state.player(owner).hand.iter().any(|c| {
                    matches!(
                        ctx.catalog.cards[*c as usize].suit,
                        crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                    )
                });
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
                    stage: HostStage::Act,
                }))
            }
            // 野升：有黑手牌才可发动（自选多张）
            HostBehavior::YunchouYiesheng => {
                let has = ctx.state.player(owner).hand.iter().any(|c| {
                    matches!(
                        ctx.catalog.cards[*c as usize].suit,
                        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                    )
                });
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
            HostBehavior::ZhangbaSha => {
                // 丈八蛇矛：两张手牌当杀的主动技（机器见 HostMachine）。
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
            HostBehavior::MtgYixialanRecast => {
                // 依夏兰重铸（mtg 第五批 v104）：地附主动技——仅当地图
                // 在场（current_land=依夏兰，全局技同口径）且手牌有基本
                // 牌（上游 filterCard basic + canRecast；canRecast 为引
                // 擎内置归档不可核验，原生取手牌基本牌并登记）
                if ctx.state.current_land != Some(CardKind::Yixialan) {
                    return None;
                }
                let has_basic = ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|c| yangpijuan_class(ctx.catalog.card_kind(*c)) == Some(0));
                if !has_basic {
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
            HostBehavior::MtgYoulinDiscard => {
                // 幽林地附技（mtg 第九批 v108）：current_land=幽林 + 手
                // 牌有锦囊（yangpijuan_class==1）
                if ctx.state.current_land != Some(CardKind::Youlin) {
                    return None;
                }
                let has_trick = ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|c| yangpijuan_class(ctx.catalog.card_kind(*c)) == Some(1));
                if !has_trick {
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
            HostBehavior::MtgLinzhongjiantaStrike => {
                // 塔击（mtg 第七批 v106）：地附主动技——current_land=林
                // 中尖塔 + 装备区有武器（上游 viewAsFilter getEquip(1)
                // 同口径）+ 手牌有基本牌
                if ctx.state.current_land != Some(CardKind::Linzhongjianta) {
                    return None;
                }
                let has_weapon = ctx.state.player(owner).equips.iter().any(|c| {
                    ctx.catalog.card_slot_of(ctx.state, *c)
                        == Some(crate::catalog::EquipSlot::Weapon)
                });
                let has_basic = ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|c| yangpijuan_class(ctx.catalog.card_kind(*c)) == Some(0));
                if !has_weapon || !has_basic {
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
            HostBehavior::DddXiaxing => {
                // 侠行·视为使用【杀】（3D精选/character/skill.js:163-208）：
                // 持有「侠」标记（跨轮保留直至被消费）且本阶段尚未使用杀，移去「侠」并视为使用【杀】（0 卡转化）。
                let has_xia = ctx
                    .state
                    .vm_aux
                    .flags
                    .get(&(owner, "dddxiaxing_xia".to_string()))
                    .copied()
                    .unwrap_or(0)
                    > 0;
                if !has_xia
                    || ctx.state.turn.sha_used_play_phase
                    || !ctx.state.alive_players().any(|p| p != owner)
                {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: Some(2),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            }
            HostBehavior::MbcmJiushi => {
                if ctx.state.player(owner).face_down || ctx.state.turn.jiu_used {
                    return None;
                }
                Some(Box::new(HostMachine {
                    def_key: self.name.clone(),
                    behavior: self.behavior,
                    owner,
                    target: None,
                    card: Some(0),
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::Act,
                }))
            }
            HostBehavior::MbcmFangzhu => {
                let last_t = ctx
                    .state
                    .vm_aux
                    .flags
                    .get(&(owner, "mbcmfangzhu_last_target".to_string()))
                    .copied();
                if !ctx
                    .state
                    .alive_players()
                    .any(|p| p != owner && Some(u32::from(p)) != last_t)
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
            HostBehavior::MbJuejin => {
                if ctx
                    .state
                    .vm_aux
                    .flags
                    .get(&(owner, "mbjuejin_used".to_string()))
                    .copied()
                    .unwrap_or(0)
                    > 0
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
            _ => unreachable!("behavior routed to the wrong activate module"),
        }
    }
}
