use super::super::*;

impl HostSkillBehavior {
    pub(super) fn activate_swd_skills(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
    ) -> Option<Box<dyn crate::catalog::SkillMachine>> {
        match self.behavior {
            // 枕流·临月面（武将面批 83）：有红色手牌且有其他存活角色
            HostBehavior::SwdZhenjiu => {
                let red = ctx.state.player(owner).hand.iter().any(|c| {
                    matches!(
                        ctx.catalog.cards[*c as usize].suit,
                        crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                    )
                });
                let others = ctx
                    .state
                    .alive_players()
                    .any(|p| p != owner && ctx.state.player(p).off_court.is_empty());
                if !red || !others {
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
            // 麻醉·临月面（武将面批 83）：有黑色手牌且有其他未挂附存活角色
            HostBehavior::SwdLmazui => {
                let black = ctx.state.player(owner).hand.iter().any(|c| {
                    matches!(
                        ctx.catalog.cards[*c as usize].suit,
                        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                    )
                });
                let others = ctx
                    .state
                    .alive_players()
                    .any(|p| p != owner && ctx.state.player(p).off_court.is_empty());
                if !black || !others {
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
            // 剑击·少微面（武将面批 91）：有装备牌且射程内有目标
            HostBehavior::SwdJianji => {
                let has_equip = ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .chain(ctx.state.player(owner).equips.iter())
                    .any(|c| ctx.catalog.card_kind(*c).is_equip());
                let in_range = ctx.state.alive_players().any(|p| {
                    p != owner
                        && ctx.state.computed_distance(ctx.catalog, owner, p)
                            <= ctx.state.attack_range(ctx.catalog, owner)
                });
                if !has_equip || !in_range {
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
            // 奥剑（deferred 消解批）：已损失体力≥1 且有手牌
            HostBehavior::SwdAojian => {
                let x = (ctx.state.player(owner).max_hp as i32 - ctx.state.player(owner).hp).max(0);
                if x == 0 || ctx.state.player(owner).hand.is_empty() {
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
            // 化金（deferred 消解批）：he 有牌
            HostBehavior::SwdHuajin => {
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
            // 拼点（deferred 消解批）：手牌有牌
            HostBehavior::SwdNlianji => {
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
                    stage: HostStage::Act,
                }))
            }
            // 蛇魂（deferred 消解批）：手牌有牌
            HostBehavior::SwdShehun => {
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
                    stage: HostStage::Act,
                }))
            }
            // 聚悉（deferred 消解批）：计数 >= 存活数
            HostBehavior::SwdJuxi => {
                let alive = ctx.state.alive_players().count() as u8;
                if ctx.state.player(owner).juxi_count < alive || alive == 0 {
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
            // 破舟·封印（deferred 消解批）：计数 >0 且存在未封印的其他角色
            HostBehavior::SwdPozhou2 => {
                let has_target = ctx.state.alive_players().any(|p| {
                    if p == owner {
                        return false;
                    }
                    let sealed = ctx
                        .catalog
                        .skill_by_name(SWD_FENGYIN_STATE_KEY)
                        .map(|d| ctx.state.has_skill(p, d))
                        .unwrap_or(false);
                    !sealed
                });
                if ctx.state.player(owner).pozhou_count == 0 || !has_target {
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
            // 陷阱（deferred 消解批）：有花色未在陷阱表中的手牌
            HostBehavior::SwdSxianjing => {
                let traps: Vec<crate::catalog::Suit> = ctx
                    .state
                    .player(owner)
                    .sxianjing_traps
                    .iter()
                    .map(|c| ctx.catalog.cards[*c as usize].suit)
                    .collect();
                let has = ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .any(|c| !traps.contains(&ctx.catalog.cards[*c as usize].suit));
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
            // 锦鳞（deferred 消解批 114）：limited 一次性——已发动则无机器
            //（技能实例仍在，门在 activate；上游 awakenSkill 同口径）
            HostBehavior::SwdJinlin => {
                if ctx.state.player(owner).jinlin_used {
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
            // 天书（swd 批 142）：出牌阶段主动（限定——实例启用时）
            HostBehavior::SwdTianshu => {
                if !matches!(ctx.state.turn.phase, crate::event::Phase::Play)
                    || ctx.state.turn.current != owner
                {
                    return None;
                }
                let enabled = ctx
                    .catalog
                    .skill_by_name(SWD_TIANSHU_KEY)
                    .and_then(|d| ctx.state.skills.get(&(owner, d)).map(|i| i.enabled))
                    .unwrap_or(false);
                let has_trick = ctx.state.player(owner).hand.iter().any(|c| {
                    let kind = ctx.catalog.card_kind(*c);
                    !kind.is_basic_kind() && !kind.is_equip()
                });
                if !enabled || !has_trick {
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
            // 林韵（swd 批 127）：出牌阶段主动——两张牌（手/装备）当杀
            HostBehavior::SwdLinyun => {
                if !matches!(ctx.state.turn.phase, crate::event::Phase::Play)
                    || ctx.state.turn.current != owner
                {
                    return None;
                }
                if ctx.state.player(owner).hand.len() + ctx.state.player(owner).equips.len() < 2 {
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
            // 孤行（deferred 消解批）：手牌数 <= max(1, 已损失体力) 且 >0
            HostBehavior::SwdGuxing => {
                let p = ctx.state.player(owner);
                let lost = (p.max_hp as i32 - p.hp).max(0) as usize;
                let limit = lost.max(1);
                if p.hand.is_empty() || p.hand.len() > limit {
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
            // 玄咒（deferred 消解批）：手牌有普通锦囊
            HostBehavior::SwdXuanzhou => {
                let has = ctx.state.player(owner).hand.iter().any(|c| {
                    let k = ctx.catalog.card_kind(*c);
                    !k.is_basic_kind() && k.equip_slot().is_none() && k != CardKind::Jiguan
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
            // 孵鸟（deferred 消解批）：有手牌
            HostBehavior::SwdFuniao => {
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
                    stage: HostStage::Act,
                }))
            }
            // 黑火（deferred 消解批）：he 有装备牌
            HostBehavior::SwdHeihuo => {
                let has = ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .chain(ctx.state.player(owner).equips.iter())
                    .any(|c| ctx.catalog.card_kind(*c).is_equip());
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
            // 头弹·麦星面（武将面批 91）：有黑桃 he 且有其他存活
            HostBehavior::SwdToudan => {
                let spade = ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .chain(ctx.state.player(owner).equips.iter())
                    .any(|c| {
                        matches!(
                            ctx.catalog.cards[*c as usize].suit,
                            crate::catalog::Suit::Spade
                        )
                    });
                let others = ctx.state.alive_players().any(|p| p != owner);
                if !spade || !others {
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
            // 玄影·端蒙面（武将面批 90）：未横置且有存活其他角色（sha 面）
            HostBehavior::SwdXuanying => {
                let unlinked = !ctx.state.player(owner).linked;
                let others = ctx.state.alive_players().any(|p| p != owner);
                if !unlinked || !others {
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
            // 命符·姜子牙面（武将面批 88）：有梅花手牌
            HostBehavior::SwdMingfu => {
                let club = ctx.state.player(owner).hand.iter().any(|c| {
                    matches!(
                        ctx.catalog.cards[*c as usize].suit,
                        crate::catalog::Suit::Club
                    )
                });
                if !club {
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
            // 义绝（武将面批 86）：自己有手牌且有手牌的其他角色
            HostBehavior::SwdYijue => {
                let has_target = ctx
                    .state
                    .alive_players()
                    .any(|p| p != owner && !ctx.state.player(p).hand.is_empty());
                if ctx.state.player(owner).hand.is_empty() || !has_target {
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
            // 活弹·舒莞儿面（武将面批 85）：有红色手牌且存活其他角色≥1
            HostBehavior::SwdHuodan => {
                let red = ctx.state.player(owner).hand.iter().any(|c| {
                    matches!(
                        ctx.catalog.cards[*c as usize].suit,
                        crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                    )
                });
                let others = ctx.state.alive_players().any(|p| p != owner);
                if !red || !others {
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
            // 愚钝·福面（武将面批 82）：两张锦囊手牌且在攻击范围内有目标
            HostBehavior::SwdYudun => {
                let tricks = ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .filter(|c| {
                        let k = ctx.catalog.card_kind(**c);
                        !k.is_basic_kind()
                            && k.equip_slot().is_none()
                            && k.is_basic_trick_or_equip()
                    })
                    .count();
                let in_range = ctx.state.alive_players().any(|p| {
                    p != owner
                        && ctx.state.computed_distance(ctx.catalog, owner, p)
                            <= ctx.state.attack_range(ctx.catalog, owner)
                });
                if tricks < 2 || !in_range {
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
            // 占星·肯迪面（武将面批 82）：有手牌
            HostBehavior::SwdZhanxing => {
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
                    stage: HostStage::Act,
                }))
            }
            // 乱击·多鹏面（武将面批 81）：手牌存在同花色两卡
            HostBehavior::SwdLuanji => {
                let hand: Vec<CardId> = ctx.state.player(owner).hand.iter().copied().collect();
                let has_pair = hand.iter().enumerate().any(|(i, a)| {
                    let sa = ctx.catalog.cards[*a as usize].suit;
                    hand.iter()
                        .skip(i + 1)
                        .any(|b| ctx.catalog.cards[*b as usize].suit == sa)
                });
                if !has_pair {
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
            // 运筹·姬亮面（武将面批㊳）：自己有手牌
            HostBehavior::SwdYunchou => {
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
                    stage: HostStage::Act,
                }))
            }
            // 天火·秦壬面（武将面批㊲）：任一其他角色判定区有牌
            HostBehavior::SwdTianhuo => {
                let has = ctx
                    .state
                    .alive_players()
                    .any(|p| p != owner && !ctx.state.player(p).judge.is_empty());
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
            // 焰羽·少微面（武将面批㊱）：红色牌 ≥2
            HostBehavior::SwdHuangyu => {
                let reds = ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .chain(ctx.state.player(owner).equips.iter())
                    .filter(|c| {
                        matches!(
                            ctx.catalog.cards[**c as usize].suit,
                            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                        )
                    })
                    .count();
                if reds < 2 {
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
            // 摄引（武将面批㊱）：任一其他角色判定区有牌
            HostBehavior::SwdXianyin => {
                let has = ctx
                    .state
                    .alive_players()
                    .any(|p| p != owner && !ctx.state.player(p).judge.is_empty());
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
            // 星坠·慕月面（武将面批㉟）：he 有牌
            HostBehavior::SwdXingzhui => {
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
            // 冰箭（武将面批㉞ 海大胖）：手牌有黑色杀
            HostBehavior::SwdBingjian => {
                let has = ctx.state.player(owner).hand.iter().any(|c| {
                    ctx.catalog.card_kind(*c).is_sha_like()
                        && matches!(
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
                    stage: HostStage::Act,
                }))
            }
            // 神鼎·麦星面（武将面批㉞）：恒可发动（判定分支自带无效果面）
            HostBehavior::SwdShending => Some(Box::new(HostMachine {
                def_key: self.name.clone(),
                behavior: self.behavior,
                owner,
                target: None,
                card: None,
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::Act,
            })),
            // 鬼眼（武将面批㉞ 冯天凌）：任一其他角色手牌有梅花
            HostBehavior::SwdGuiyan => {
                let has = ctx.state.alive_players().any(|p| {
                    p != owner
                        && ctx.state.player(p).hand.iter().any(|c| {
                            matches!(
                                ctx.catalog.cards[*c as usize].suit,
                                crate::catalog::Suit::Club
                            )
                        })
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
            // 鸣凤（武将面批㉝ 胡月）：有装备牌可弃
            HostBehavior::SwdFengming => {
                let has_equip = ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .chain(ctx.state.player(owner).equips.iter())
                    .any(|c| ctx.catalog.card_kind(*c).is_equip());
                if !has_equip {
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
            // 断义（武将面批㉘ 慕容诗）：手牌中杀多于 1（上游 countCards
            // "sha" > 1 同口径）
            HostBehavior::SwdDuanyi => {
                let sha_n = ctx
                    .state
                    .player(owner)
                    .hand
                    .iter()
                    .filter(|c| ctx.catalog.card_kind(**c).is_sha_like())
                    .count();
                if sha_n <= 1 {
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
            // 森罗（武将面批㉖ 莫耶）：自己手牌数为全场严格最少
            HostBehavior::SwdSenluo => {
                let my = ctx.state.player(owner).hand.len();
                let is_min = ctx
                    .state
                    .alive_players()
                    .all(|p| p == owner || ctx.state.player(p).hand.len() >= my);
                if !is_min {
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
            // 湛卢（武将面批㉕ 屠维）：有黑桃牌且场上有已受伤角色
            HostBehavior::SwdZhanlu => {
                let has_spade = ctx.state.player(owner).hand.iter().any(|c| {
                    matches!(
                        ctx.catalog.cards[*c as usize].suit,
                        crate::catalog::Suit::Spade
                    )
                });
                let any_damaged = ctx
                    .state
                    .alive_players()
                    .any(|p| ctx.state.player(p).hp < ctx.state.player(p).max_hp as i32);
                if !has_spade || !any_damaged {
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
            // 相惠（武将面批㉕ 夏柔）：有红色手牌且体力最少者已受伤
            HostBehavior::SwdXianghui => {
                let has_red = ctx.state.player(owner).hand.iter().any(|c| {
                    matches!(
                        ctx.catalog.cards[*c as usize].suit,
                        crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                    )
                });
                let min_hp = ctx
                    .state
                    .alive_players()
                    .map(|p| ctx.state.player(p).hp)
                    .min()
                    .unwrap_or(0);
                let min_damaged = ctx.state.alive_players().any(|p| {
                    let pl = ctx.state.player(p);
                    pl.hp == min_hp && pl.hp < pl.max_hp as i32
                });
                if !has_red || !min_damaged {
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
            // 对诗（武将面批㉒ 王思月）：手牌非空即发动（usable:1 走
            // limits；上游 duishi2/3 临时技面由限一次收敛）
            HostBehavior::SwdDuishi => {
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
                    stage: HostStage::Act,
                }))
            }
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
            HostBehavior::SwdTiebi => Some(Box::new(HostMachine {
                def_key: self.name.clone(),
                behavior: self.behavior,
                owner,
                target: None,
                card: None,
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::Act,
            })),
            HostBehavior::SwdDatong => Some(Box::new(HostMachine {
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
