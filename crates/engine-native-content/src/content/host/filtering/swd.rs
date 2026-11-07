use super::super::*;

impl HostSkillBehavior {
    pub(super) fn filter_swd(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
        event: &EventData,
    ) -> bool {
        match self.behavior {
            HostBehavior::SwdGTiandao => false,
            HostBehavior::SwdShehun => false,
            // 魂身：自己使用/打出的牌为闪
            HostBehavior::SwdHyunshen => {
                let is_shan = |card: crate::ids::CardId| {
                    ctx.catalog.card_kind(card) == crate::catalog::CardKind::Shan
                };
                match event {
                    EventData::CardUsed(u) => {
                        u.player == owner
                            && u.effective_kind(ctx.catalog) == crate::catalog::CardKind::Shan
                    }
                    EventData::CardResponded { card, player } => *player == owner && is_shan(*card),
                    _ => false,
                }
            }
            // 魂身·收束（A2 H25）：上游 player phaseBegin＝自己回合开始且
            // 计数 >0（原锚自己判定阶段开始）
            HostBehavior::SwdHyunshen2 => matches!(event,
                EventData::TurnBegin { player }
                if *player == owner && ctx.state.player(owner).hyunshen_count > 0),
            // 待行：自己结束阶段且有 he 牌
            HostBehavior::SwdDaixing => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: false }
                if *player == owner
                    && (!ctx.state.player(owner).hand.is_empty()
                        || !ctx.state.player(owner).equips.is_empty())),
            // 待行·清甲（A2 H26）：上游 player phaseBegin＝自己回合开始且
            // 计数 >0（原锚自己判定阶段开始）
            HostBehavior::SwdDaixing2 => matches!(event,
                EventData::TurnBegin { player }
                if *player == owner && ctx.state.player(owner).daixing_count > 0),
            // 待行·吸收：自己受伤后且计数 >0
            HostBehavior::SwdDaixing3 => matches!(event,
                EventData::Damage(d) if d.target == owner && !d.prevented
                    && ctx.state.player(owner).daixing_count > 0),
            HostBehavior::SwdJuxi => false,
            // 聚悉·计数：弃牌者非当前回合玩家
            HostBehavior::SwdJuxiCount => matches!(event,
                EventData::CardsDiscarded { player, .. }
                if *player != ctx.state.turn.current),
            // 破舟：自己受伤后
            HostBehavior::SwdPozhou => matches!(event,
                EventData::Damage(d) if d.target == owner && !d.prevented),
            HostBehavior::SwdPozhou2 => false,
            // 陷阱：主动技
            HostBehavior::SwdSxianjing => false,
            // 陷阱·触发：他人以同花色牌指定你，且该花色在陷阱表中
            HostBehavior::SwdSxianjingGain => matches!(event,
            EventData::CardUsed(u)
            if u.player != owner
                && u.targets.contains(&owner)
                && !u.cancelled
                && ctx.state.player(owner).sxianjing_traps.iter().any(|c| {
                    Some(ctx.catalog.cards[*c as usize].suit) == u.suit(ctx.catalog)
                })),
            // 陷阱·取回：自己受伤后且陷阱表非空
            HostBehavior::SwdSxianjingDamage => matches!(event,
                EventData::Damage(d) if d.target == owner && !d.prevented
                    && !ctx.state.player(owner).sxianjing_traps.is_empty()),
            HostBehavior::SwdGuxing => false,
            // 灵矢：锁定旗标（无触发；四臂内联）
            HostBehavior::SwdLingshi => false,
            HostBehavior::SwdJinlin => false,
            // 锦鳞·退（A2 H30）：上游 player phaseBegin＝持有者回合开始
            //（原锚任意自己阶段开始＝每回合 5→6 次）
            HostBehavior::SwdJinlin2 => matches!(event,
                EventData::TurnBegin { player } if *player == owner),
            HostBehavior::SwdTianshu => false,
            // 诛害：他人结束阶段开始、其本回合造成过伤害、自己有杀且在射程内
            HostBehavior::SwdZhuhai => match event {
                EventData::Phase {
                    player,
                    phase: crate::event::Phase::End,
                    begin: true,
                } => {
                    let t = *player;
                    t != owner
                        && ctx.state.player(t).alive
                        && ctx.state.player(t).turn_damage > 0
                        && ctx
                            .state
                            .player(owner)
                            .hand
                            .iter()
                            .any(|c| ctx.catalog.cards[*c as usize].kind.is_sha_like())
                        && ctx.state.computed_distance(ctx.catalog, owner, t)
                            <= ctx.state.attack_range(ctx.catalog, owner)
                }
                _ => false,
            },
            HostBehavior::SwdLinyun => false,
            // 集控（A2 H28）：上游 player phaseBegin＝自己回合开始 或
            // 自己失去手牌（原锚任意自己阶段开始）
            HostBehavior::SwdJikong => {
                // 每回合限一次（上游 usable:1）：**使用时**消费——取消目标窗
                // 不消耗（上游 counttrigger-- 返还同口径）
                let available = ctx
                    .catalog
                    .skill_by_name(SWD_JIKONG_KEY)
                    .and_then(|d| ctx.state.skills.get(&(owner, d)).map(|i| !i.once_used))
                    .unwrap_or(false);
                available
                    && match event {
                        EventData::TurnBegin { player } => *player == owner,
                        EventData::CardsLost {
                            player, origins, ..
                        } => {
                            *player == owner
                                && origins.iter().any(|o| *o == crate::event::LostOrigin::Hand)
                                && ctx.state.player(owner).hand.is_empty()
                        }
                        _ => false,
                    }
            }
            // 魅惑：自己失去装备牌（来源区=装备）且装备区已空
            HostBehavior::SwdMeihuo => match event {
                EventData::CardsLost {
                    player, origins, ..
                } => {
                    *player == owner
                        && origins
                            .iter()
                            .any(|o| *o == crate::event::LostOrigin::Equip)
                        && ctx.state.player(owner).equips.is_empty()
                }
                _ => false,
            },
            // 守印：任意角色濒死、自己未发动且未翻面
            HostBehavior::SwdShouyin => {
                matches!(event, EventData::Dying { .. })
                    && !ctx.state.player(owner).shouyin_used
                    && !ctx.state.player(owner).face_down
            }
            // 义山：自己受伤且有「仍在他处」的已失去牌
            HostBehavior::SwdYishan => match event {
                EventData::Damage(d) if d.target == owner && !d.prevented => ctx
                    .state
                    .player(owner)
                    .yishan_lost
                    .iter()
                    .any(|c| yishan_recoverable_state(ctx.state, owner, *c)),
                _ => false,
            },
            // 入梦：他人出牌阶段开始且自己手/装备中有非基本牌
            HostBehavior::SwdRumeng => match event {
                EventData::Phase {
                    player,
                    phase: crate::event::Phase::Play,
                    begin: true,
                } => {
                    *player != owner
                        && ctx.state.player(owner).alive
                        && ctx
                            .state
                            .player(owner)
                            .hand
                            .iter()
                            .chain(ctx.state.player(owner).equips.iter())
                            .any(|c| !ctx.catalog.cards[*c as usize].kind.is_basic_kind())
                }
                _ => false,
            },
            // 神焰：前置面=自己造成的未防止火伤且未发动；结算面=已发动且
            // 有待蔓延目标（同一伤害事件的两面按状态区分）
            HostBehavior::SwdShenyan => match event {
                EventData::Damage(d) if d.source == Some(owner) && !d.prevented => {
                    if !ctx.state.player(owner).shenyan_used {
                        d.nature == crate::event::DamageNature::Fire
                    } else {
                        !ctx.state.player(owner).shenyan_pending.is_empty()
                    }
                }
                _ => false,
            },
            HostBehavior::SwdLingwu => false,
            // 孤行·计数：孤行杀窗口内自己造成伤害
            HostBehavior::SwdGuxing3 => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && ctx.state.turn.guxing_armed),
            HostBehavior::SwdNlianji => false,
            // 热血：阶段玩家不是自己，且在杀范围内，自己手牌有杀
            HostBehavior::SwdRexue => {
                // A2 H42：上游 global phaseBegin＝他人回合开始（每回合一次）。
                if let EventData::TurnBegin { player } = event {
                    let actor = *player;
                    actor != owner
                        && ctx
                            .state
                            .player(owner)
                            .hand
                            .iter()
                            .any(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Sha)
                        && ctx.state.computed_distance(ctx.catalog, owner, actor)
                            <= ctx.state.attack_range(ctx.catalog, owner)
                } else {
                    false
                }
            }
            // 幻音：他人对你使用牌且未取消
            HostBehavior::SwdHuanyin => matches!(event,
                EventData::CardUsed(u)
                if u.player != owner && u.targets.contains(&owner) && !u.cancelled),
            // 灵弦：自己使用/打出牌，且非自己回合，且存在攻击距离 >1 的其他存活者
            HostBehavior::SwdLingxian => {
                let by_owner = match event {
                    EventData::CardUsed(u) => u.player == owner,
                    EventData::CardResponded { player, .. } => *player == owner,
                    _ => false,
                };
                by_owner
                    && ctx.state.turn.current != owner
                    && (0..ctx.state.players.len() as u8).any(|t| {
                        t != owner
                            && ctx.state.player(t).alive
                            && ctx.state.computed_distance(ctx.catalog, owner, t) > 1
                    })
            }
            // 铁壁：杀指定了与持有者距离 ≤1 的目标，持有者有黑牌
            HostBehavior::SwdTiebi => {
                let hit = matches!(event,
                EventData::CardUsed(u)
                if u.effective_kind(ctx.catalog).is_sha_like()
                    && u.targets.iter().any(|&t| {
                        ctx.state.player(t).alive
                            && ctx.state.player(t).hujia == 0
                            && ctx.state.seat_distance(t, owner) <= 1
                    })
                    && ctx.state.player(owner).hand.iter().any(|c| {
                        matches!(
                            ctx.catalog.cards[*c as usize].suit,
                            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                        )
                    }));
                hit
            }
            // 大同（A2 H62）：任意玩家整回合结束（global phaseEnd）全场极差 ≤1
            HostBehavior::SwdDatong => matches!(event,
            EventData::TurnEndWindow { after: false, .. }
            if {
                let counts: Vec<usize> = ctx.state.players.iter()
                    .filter(|p| p.alive)
                    .map(|p| p.hand.len())
                    .collect();
                let max = counts.iter().copied().max().unwrap_or(0);
                let min = counts.iter().copied().min().unwrap_or(0);
                max - min <= 1
            }),
            // 承剑：其他角色受到杀伤害（来源≠持有者）
            HostBehavior::SwdChengjian => matches!(event,
                EventData::Damage(d)
                if !d.prevented && d.amount > 0
                    && d.card.map(|c| ctx.catalog.card_kind(c).is_sha_like()).unwrap_or(false)
                    && d.source.is_some()
                    && d.source != Some(owner)
                    && ctx.state.player(d.source.unwrap()).alive),
            // 缚斧：持有者对目标用杀伤害前置（有武器）
            HostBehavior::SwdJufu => matches!(event,
            EventData::Damage(d)
            if d.source == Some(owner) && !d.prevented && d.amount > 0
                && d.card.map(|c| ctx.catalog.card_kind(c).is_sha_like()).unwrap_or(false)
                && ctx.state.player(owner).equips.iter().any(|c| {
                    ctx.catalog.card_slot_of(ctx.state, *c)
                        == Some(crate::catalog::EquipSlot::Weapon)
                })),
            // 花镜：持有者造成的锦囊牌伤害
            HostBehavior::SwdHuajing => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner) && !d.prevented && d.amount > 0
                    && d.card.map(|c| {
                        let k = ctx.catalog.card_kind(c);
                        !k.is_basic_kind()
                            && k.equip_slot().is_none()
                            && k.is_basic_trick_or_equip()
                    }).unwrap_or(false)),
            // 护盾：持有者造成伤害后自己无护甲
            HostBehavior::SwdHudun => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner) && !d.prevented && d.amount > 0
                    && ctx.state.player(owner).hujia == 0),
            // 破星：持有者对体力高于自己的目标造成伤害
            HostBehavior::SwdPoxing => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner) && !d.prevented && d.amount > 0
                    && d.target != owner
                    && ctx.state.player(d.target).hp > ctx.state.player(owner).hp),
            // 流虹：持有者使用杀
            HostBehavior::SwdLiuhong => matches!(event,
                EventData::CardUsed(u)
                if u.player == owner
                    && u.effective_kind(ctx.catalog).is_sha_like()),
            // 墨剑：持有者使用杀且指定了目标（上游 shaBegin=自己用杀）
            HostBehavior::SwdMojian => matches!(event,
                EventData::CardUsed(u)
                if u.player == owner
                    && u.effective_kind(ctx.catalog).is_sha_like()
                    && !u.targets.is_empty()),
            // 道法：其他角色受伤且来源有牌
            HostBehavior::SwdDaofa => matches!(event,
                EventData::Damage(d)
                if !d.prevented && d.amount > 0
                    && d.source.is_some()
                    && d.source != Some(owner)
                    && !ctx.state.player(d.source.unwrap()).hand.is_empty()),
            // 玉鳞：持有者即将受伤且手/装备区有装备牌
            HostBehavior::SwdYulin => matches!(event,
            EventData::Damage(d)
            if d.target == owner && !d.prevented && d.amount > 0
                && (!ctx.state.player(owner).equips.is_empty()
                    || ctx.state.player(owner).hand.iter().any(|c| {
                        ctx.catalog.card_slot_of(ctx.state, *c).is_some()
                    }))),
            // 平恤：无触发面
            HostBehavior::SwdPingxu => false,
            // 还灵（A2 H63）：自己整回合结束（上游 player phaseEnd＝每回合一次）
            HostBehavior::SwdHuanling => matches!(event,
                EventData::TurnEndWindow { player, after: false }
                if *player == owner),
            // 屠阵：自己伤害他人结算后且其有非基本手牌
            HostBehavior::SwdTuzhen => matches!(event,
            EventData::Damage(d)
            if d.source == Some(owner) && !d.prevented && d.amount > 0
                && d.target != owner
                && ctx.state.player(d.target).alive
                && ctx.state.player(d.target).hand.iter().any(|c| {
                    !ctx.catalog.card_kind(*c).is_basic_kind()
                })),
            // 协律：自己弃牌阶段结束（End 入场）且手牌≥2 同色
            HostBehavior::SwdXielv => matches!(event,
            EventData::Phase { player, phase, begin }
            if *player == owner
                && *phase == crate::event::Phase::End
                && *begin
                && {
                    let hand = &ctx.state.player(owner).hand;
                    hand.len() >= 2
                        && {
                            let first =
                                ctx.catalog.cards[hand[0] as usize].suit;
                            hand.iter().all(|c| {
                                let red = |su| {
                                    matches!(
                                        su,
                                        crate::catalog::Suit::Heart
                                            | crate::catalog::Suit::Diamond
                                    )
                                };
                                red(ctx.catalog.cards[*c as usize].suit)
                                    == red(first)
                            })
                        }
                }),
            // 雪煌：无触发面
            HostBehavior::SwdXuehuang => false,
            // 还侠：无触发面
            HostBehavior::SwdHuanxia => false,
            // 破月：无触发面
            HostBehavior::SwdPoyue => false,
            // 女娲石：濒死者 hp≤0 且自己 hp>1（上游 global dying 同口径；
            // 含自身濒死时 owner hp>1 恒否自然排除）
            HostBehavior::SwdNvwashiGuard => matches!(event,
                EventData::Dying { player, .. }
                if ctx.state.player(*player).hp <= 0
                    && ctx.state.player(owner).hp > 1),
            // 收纳/炼化/神农鼎：无触发面
            HostBehavior::SwdLianyaohuShouna => false,
            HostBehavior::SwdLianyaohuLianhua => false,
            HostBehavior::SwdShennongdingCard => false,
            // 昆仑镜：无触发面
            HostBehavior::SwdKunlunjingcCard => false,
            // 手持门控 helper（四象五件共用）：持有者手牌含对应器族实体
            // 青龙之圭（A2 H32）：上游 player phaseBegin＝自己回合开始且
            // 手持（原锚自己判定阶段开始）
            HostBehavior::SwdQinglongzhiguiHeld => matches!(event,
                EventData::TurnBegin { player }
                if *player == owner
                    && hand_has_kind(ctx, owner, CardKind::Qinglongzhigui)),
            // 逆攻积攒：自己受到真实伤害
            HostBehavior::SwdNigongAccumulate => matches!(event,
                EventData::Damage(d)
                if d.target == owner && !d.prevented && d.amount > 0),
            // 逆攻发动：无触发面
            HostBehavior::SwdNigongStrike => false,
            // 四非珍面：无触发面
            HostBehavior::SwdSifeizhenmianCard => false,
            // 朱雀之璋：受他人伤害结算后且手持（来源存活且非自己）
            HostBehavior::SwdZhuquezhizhangHeld => matches!(event,
                EventData::Damage(d)
                if d.target == owner && !d.prevented && d.amount > 0
                    && d.source.is_some()
                    && d.source != Some(owner)
                    && ctx.state.player(d.source.unwrap()).alive
                    && hand_has_kind(ctx, owner, CardKind::Zhuquezhizhang)),
            // 白兽之琥：自己弃牌阶段结束且手持且有红牌
            HostBehavior::SwdBaishouzhihuHeld => matches!(event,
                // 上游 `g_baishouzhihu`：`{player:'discardEnd'}` + `countCards('h','baishouzhihu') > 0`
                //（第 935 轮：原臂写成「任意阶段开始 + 手里有红桃/方块」，两处都不对）
                EventData::Phase { player, phase: crate::event::Phase::Discard, begin: false, .. }
                if *player == owner
                    && hand_has_kind(ctx, owner, CardKind::Baishouzhihu)),
            // 玄武之璜：自己造成伤害结算后且手持且受伤
            HostBehavior::SwdXuanwuzhihuangHeld => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner) && !d.prevented && d.amount > 0
                    && ctx.state.player(owner).hp < ctx.state.player(owner).max_hp as i32
                    && hand_has_kind(ctx, owner, CardKind::Xuanwuzhihuang)),
            // 玉蝉换形：自己用牌（非玉蝉本体）结算后且手牌有玉蝉
            HostBehavior::SwdYuchanSwap => matches!(event,
            EventData::CardUsed(u)
            if u.player == owner
                && !u.cancelled
                && ctx.state.player(owner).hand.iter().any(|c| {
                    ctx.catalog.card_kind(*c).yuchan_auto_view_as().is_some()
                })),
            // 疾影：无触发面
            HostBehavior::SwdJiyingHeld => false,
            // 沐风（A2 H66）：自己整回合结束且本回合用过基本牌
            HostBehavior::SwdMufengHeld => matches!(event,
                EventData::TurnEndWindow { player, after: false }
                if *player == owner
                    && ctx.state.turn.mufeng_used_basic),
            // 望尘：弃牌阶段结束（End 入场）且弃置过基本牌
            HostBehavior::SwdWangchenHeld => matches!(event,
                EventData::Phase { player, phase, begin }
                if *player == owner
                    && *phase == crate::event::Phase::End
                    && *begin
                    && ctx.state.turn.wangchen_discard_basic),
            // 神草结：锦囊伤害且自己是受伤者或来源、手持神草结
            HostBehavior::SwdShencaojieGuard => matches!(event,
                EventData::Damage(d)
                if !d.prevented && d.amount > 0
                    && d.card.map(|c| {
                        let k = ctx.catalog.card_kind(c);
                        !k.is_basic_kind() && k.equip_slot().is_none()
                    })
                    .unwrap_or(false)
                    && (d.target == owner || d.source == Some(owner))
                    && hand_has_kind(ctx, owner, CardKind::Shencaojie)),
            // 黄麟之琮（A2 H33）：上游 player phaseBegin＝自己回合开始、
            // 无护甲且手持（原锚自己判定阶段开始）
            HostBehavior::SwdHuanglinzhicongHeld => matches!(event,
                EventData::TurnBegin { player }
                if *player == owner
                    && ctx.state.player(owner).hujia == 0
                    && hand_has_kind(ctx, owner, CardKind::Huanglinzhicong)),
            // 空洞印：自己死亡（maxHp>0 gate 恒真登记）
            HostBehavior::SwdKongdongyinGuard => matches!(event,
                EventData::Death { player, .. } if *player == owner),
            // 神秘果窗口：自己出牌阶段用出基本/普通锦囊结算后、手持神秘
            // 果、每阶段限一次、目标全存活。登记差异：view-as 转化用牌、
            // 五谷（Remained 面未实现）、机关异类与延时锦囊不进再结算
            // 白名单（上游 complexTarget/exclusion 同口径收窄）。
            HostBehavior::SwdShenmiguoWindow => matches!(event,
                EventData::CardUsed(u)
                if u.player == owner
                    && u.card.is_some_and(|c|u.use_as.is_none_or(|k|k==ctx.catalog.card_kind(c)))
                    && ctx.state.turn.phase == crate::event::Phase::Play
                    && ctx.state.turn.current == owner
                    && !ctx.state.turn.shenmiguo_used
                    && {
                        let k = u.effective_kind(ctx.catalog);
                        if matches!(
                            k,
                            CardKind::Shenmiguo
                                | CardKind::Wugu
                                | CardKind::Jiguan
                                | CardKind::Le
                                | CardKind::Bing
                                | CardKind::ShanDian
                        ) {
                            false
                        } else {
                            k.is_basic_kind() || k.equip_slot().is_none()
                        }
                    }
                    && hand_has_kind(ctx, owner, CardKind::Shenmiguo)
                    && u.targets.iter().all(|t| ctx.state.player(*t).alive)),
            // 浴尘：回合外使用/打出黑色牌结算后（双事件；he 有牌的目标
            // 在机器步求取，filter 只验事件形状）
            HostBehavior::SwdYuchen => {
                matches!(event,
                    EventData::CardUsed(u)
                    if u.player == owner
                        && ctx.state.turn.current != owner
                        && u.suit(ctx.catalog).is_some_and(|s|is_black_suit(&s))
                ) || matches!(event,
                EventData::CardResponded { card, player }
                if *player == owner
                    && ctx.state.turn.current != owner
                    && is_black_suit(&ctx.catalog.cards[*card as usize].suit))
            }
            // 挟雷：使用/打出杀结算后（双事件；>2 人局；目标候选在机器
            // 步求取——use 排除原目标、respond 排除来源）
            HostBehavior::SwdXielei => {
                matches!(event,
                    EventData::CardUsed(u)
                    if u.player == owner
                        && ctx.state.players.len() > 2
                        && u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog)).is_sha_like()
                ) || matches!(event,
                EventData::CardResponded { card, player }
                if *player == owner
                    && ctx.state.players.len() > 2
                    && ctx.catalog.card_kind(*card).is_sha_like())
            }
            // 连打：自己杀结算完毕、每回合未链过、he 有牌、目标存活
            HostBehavior::SwdLianda => matches!(event,
                EventData::CardUsed(u)
                if u.player == owner
                    && !ctx.state.turn.lianda_used
                    && u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog)).is_sha_like()
                    && (!ctx.state.player(owner).hand.is_empty()
                        || !ctx.state.player(owner).equips.is_empty())
                    && u.targets.iter().all(|t| ctx.state.player(*t).alive)),
            // 隳延：自己造成伤害、对方有装备
            HostBehavior::SwdSuiyan => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && !ctx.state.player(d.target).equips.is_empty()
                    && (!ctx.state.player(owner).hand.is_empty()
                        || !ctx.state.player(owner).equips.is_empty())),
            // 断行：自己装备牌入装（"未强化"面登记差异：原生无强化语义）
            HostBehavior::SwdDuanxing => matches!(event,
                EventData::Equipped { player, .. } if *player == owner),
            // 破浪：自己造成伤害、对方有装备
            HostBehavior::SwdPolang => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && !ctx.state.player(d.target).equips.is_empty()),
            // 诱引：其他角色弃置过含非基本牌且自己手牌<5（地/机关异类
            // 按非基本近似——登记差异同 is_basic_kind 保守集）
            HostBehavior::SwdYouyin => matches!(event,
            EventData::CardsDiscarded { player, cards }
            if *player != owner
                && ctx.state.player(owner).hand.len() < 5
                && cards.iter().any(|c| {
                    let k = ctx.catalog.card_kind(*c);
                    !k.is_basic_kind() && k.equip_slot().is_none()
                })),
            // 暗莲映：弃牌结算后自己手牌为空（含自己弃牌——上游 loseEnd
            // 手牌丢失面≈弃置面登记差异）→ frequent 摸 2
            HostBehavior::SwdAnlianying => matches!(event,
                EventData::CardsDiscarded { player, cards }
                if *player == owner
                    && !cards.is_empty()
                    && ctx.state.player(owner).hand.is_empty()),
            // 乐跃：任意角色回复结算后（候选面在机器步分流）
            HostBehavior::SwdYinyue => matches!(event,
                EventData::Recover { player, .. }
                    if ctx.state.player(*player).alive),
            // 灵心（A2 H67）：自己整回合结束（上游 player phaseEnd）
            HostBehavior::SwdLingxin => matches!(event,
                EventData::TurnEndWindow { player, after: false }
                if *player == owner),
            // 虚影：自己受到伤害前置（DamageBefore 帧共用 Damage 载荷——
            // 取消+失去体力在机器步写回）
            HostBehavior::SwdXuying => matches!(event,
                EventData::Damage(d)
                if d.target == owner && !d.prevented),
            // 乖戾：自己杀伤害（非铁索传导）——Before 面 +1 / After 面弃两张
            HostBehavior::SwdGuaili => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && d.card.map(|c| ctx.catalog.card_kind(c).is_sha_like()).unwrap_or(false)
                    && !d.from_chain
                    && !d.prevented),
            // 敷衍·护甲面：自己受伤结算后（无护甲目标候选在机器步求取）
            HostBehavior::SwdFuyan => matches!(event,
                EventData::Damage(d)
                if d.target == owner && !d.prevented && d.amount > 0),
            // 凌波·恒爱面：闪的响应/使用结算（双事件；frequent 摸 2）
            HostBehavior::SwdHlingbo => {
                matches!(event,
                    EventData::CardResponded { card, player }
                    if *player == owner && ctx.catalog.card_kind(*card) == CardKind::Shan
                ) || matches!(event,
                EventData::CardUsed(u)
                if u.player == owner
                    && u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog)) == CardKind::Shan)
            }
            // 血裔·Miles 面：自己受到伤害前置
            HostBehavior::SwdMilesXueyi => matches!(event,
                EventData::Damage(d)
                if d.target == owner && !d.prevented),
            // 小莫雨·山小小面：自己造成伤害结算后（每回合一次经 inst 检查）
            HostBehavior::SwdXiaomoyu => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner) && !d.prevented && d.amount > 0),
            // 破茧·齐面：自己弃牌后手牌为空（上游 loseEnd≈弃置面登记差异）
            HostBehavior::SwdPojian => matches!(event,
            EventData::CardsDiscarded { player, cards }
            if *player == owner
                && !cards.is_empty()
                && ctx.state.player(owner).hand.is_empty()
                && {
                    // 牌堆/弃堆有装备牌才可发（否则无效果）
                    ctx.state.deck.iter().any(|c| ctx.catalog.card_kind(*c).is_equip())
                        || ctx.state.discard.iter().any(|c| ctx.catalog.card_kind(*c).is_equip())
                }),
            // 运筹·姬亮面：出牌主动技（ActivateSkill 通道），无触发面
            HostBehavior::SwdYunchou => matches!(event,
                EventData::Phase { player, begin, .. } if *player == owner && false),
            // 破晓·张烈面：锁定 marker（三查询点），无触发面
            HostBehavior::SwdPoxiao => matches!(event,
                EventData::Phase { player, begin, .. } if *player == owner && false),
            // 疾风·司空宇面：锁定 marker（engine 杀距离校验臂查询），无触发面
            HostBehavior::SwdSliufeng => matches!(event,
                EventData::Phase { player, begin, .. } if *player == owner && false),
            // 乱击·多鹏面：出牌主动技（activate 通道），无触发 filter
            HostBehavior::SwdLuanji => matches!(event,
                EventData::Phase { player, begin, .. } if *player == owner && false),
            // 聒噪·疾鹏面：距离 1 以内角色受损（来源非自己；距离与手牌
            // 候选在机器步求取；受损者含自己——上游 filter 只排除来源）
            HostBehavior::SwdGuozao => {
                let pl = ctx.state.player(owner);
                matches!(event,
                    EventData::Damage(d)
                    if d.amount > 0
                        && !d.prevented
                        && d.source != Some(owner)
                        && ctx
                            .state
                            .computed_distance(ctx.catalog, owner, d.target)
                            <= 1
                        && pl.alive)
            }
            // 英姿·刷新面：自己摸牌阶段开始（额外摸 1；手牌上限查询在同位）
            HostBehavior::SwdReyingzi => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Draw, begin: true }
                if *player == owner),
            // 愚钝·福面：锁定 marker（engine PlayCard 禁锦囊臂+双卡 viewAs
            // 臂查询），无触发 filter
            HostBehavior::SwdYudun => matches!(event,
                EventData::Phase { player, begin, .. } if *player == owner && false),
            // 占星·肯迪面：出牌主动技（activate 通道），无触发 filter
            HostBehavior::SwdZhanxing => matches!(event,
                EventData::Phase { player, begin, .. } if *player == owner && false),
            // 乐学：任一角色回合开始（授技在机器步按当前回合角色分流：
            // 自己回合=回合内池，他人回合=回合外池；上游 lexue1/lexue2
            // 分别挂 phaseBegin/phaseEnd 的双池面收敛为回合开始单窗，
            // PhaseBegin 授至本回合结束由 expires_end_of_turn 承担）
            // A2 H38：上游 lexue1 player phaseBegin＝回合开始（原锚判定
            // 开始；lexue2 phaseEnd 双池收敛语义保持，见上注释）。
            HostBehavior::SwdLexue => matches!(event,
                EventData::TurnBegin { .. } if ctx.state.player(owner).alive),
            // 枕流触发面（A2 H39）：上游 player phaseBegin＝自己回合开始且
            // off_court 有牌（原锚自己判定阶段开始）
            HostBehavior::SwdZhenjiu2 => matches!(event,
                EventData::TurnBegin { player }
                if *player == owner && !ctx.state.player(owner).off_court.is_empty()),
            // 麻醉触发面：自己造成伤害前置（source 面——上游
            // trigger: {source: "damageBegin"} 同口径）且 off_court 有牌
            HostBehavior::SwdLmazui2 => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner) && !d.prevented
                    && !ctx.state.player(owner).off_court.is_empty()),
            // 圣手·淳于恒面：锁定 marker（engine viewAs 链查询），无触发
            HostBehavior::SwdShengshou => matches!(event,
                EventData::Phase { player, begin, .. } if *player == owner && false),
            // 活弹·舒莞儿面：出牌主动技（activate 通道），无触发 filter
            HostBehavior::SwdHuodan => matches!(event,
                EventData::Phase { player, begin, .. } if *player == owner && false),
            // 义绝：出牌主动技（activate 通道），无触发 filter
            HostBehavior::SwdYijue => matches!(event,
                EventData::Phase { player, begin, .. } if *player == owner && false),
            // 荡平：自己造成伤害（真实、有受害存活）
            HostBehavior::SwdDangping => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && ctx.state.player(d.target).alive),
            // 幻形（A2 H40）：上游 player phaseBegin＝自己回合开始（可弃牌
            // 与男性其他存活门在步内；原锚自己判定阶段开始）
            HostBehavior::SwdHuanxing => matches!(event,
                EventData::TurnBegin { player }
                if *player == owner),
            // 命符·姜子牙面：出牌主动技（activate 通道），无触发 filter
            HostBehavior::SwdMingfu => matches!(event,
                EventData::Phase { player, begin, .. } if *player == owner && false),
            // 玄影父技只有主动面（未横置出杀）；伤害面是 group 子技。
            HostBehavior::SwdXuanying => false,
            // 玄影伤害子技：自己受伤（真实）、已横置、来源存活（父技授予才可达）
            HostBehavior::SwdXuanyingDamage => matches!(event,
                EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && ctx.state.player(owner).linked
                    && d.source.map(|s| ctx.state.player(s).alive).unwrap_or(false)),
            // 极谋·姬良面：自己受伤（真实）且回合非自己
            HostBehavior::SwdJqimou => matches!(event,
                EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && ctx.state.turn.current != owner
                    && ctx.state.player(owner).alive),
            // 剑击·少微面：出牌主动技（activate 通道），无触发 filter
            HostBehavior::SwdJianji => matches!(event,
                EventData::Phase { player, begin, .. } if *player == owner && false),
            // 头弹·麦星面：出牌主动技（activate 通道），无触发 filter
            HostBehavior::SwdToudan => matches!(event,
                EventData::Phase { player, begin, .. } if *player == owner && false),
            // 真威：其他角色响应结算（响应者非自己、自己回合内、牌已入弃堆）
            HostBehavior::SwdFzhenwei => matches!(event,
                EventData::CardResponded { card, player }
                if *player != owner
                    && ctx.state.turn.current == owner
                    && ctx.state.discard.contains(card)),
            // 遁行：成为其他角色卡牌目标（he 有牌可弃）
            HostBehavior::SwdDunxing => matches!(event,
                EventData::CardUsed(u)
                if u.player != owner
                    && u.targets.contains(&owner)
                    && !u.cancelled
                    && (!ctx.state.player(owner).hand.is_empty()
                        || !ctx.state.player(owner).equips.is_empty())),
            // 黑火：出牌主动技（activate 通道），无触发 filter
            HostBehavior::SwdHeihuo => matches!(event,
                EventData::Phase { player, begin, .. } if *player == owner && false),
            // 凝弦：自己受伤结算后（he 有黑色牌）
            HostBehavior::SwdNingxian => matches!(event,
            EventData::Damage(d)
            if d.target == owner
                && !d.prevented
                && d.amount > 0
                && (ctx.state.player(owner).hand.iter().any(|c| {
                    matches!(
                        ctx.catalog.cards[*c as usize].suit,
                        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                    )
                }) || ctx.state.player(owner).equips.iter().any(|c| {
                    matches!(
                        ctx.catalog.cards[*c as usize].suit,
                        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                    )
                }))),
            // 孵鸟/妖童：出牌主动技/marker（activate/viewAs 通道），无触发 filter
            HostBehavior::SwdFuniao => matches!(event,
                EventData::Phase { player, begin, .. } if *player == owner && false),
            HostBehavior::SwdYaotong => matches!(event,
                EventData::Phase { player, begin, .. } if *player == owner && false),
            // 破阵：自己受伤结算后（有来源、双方存活、手牌数不同）
            HostBehavior::SwdPozhen => matches!(event,
                EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.source.map(|src| {
                        ctx.state.player(src).alive
                            && ctx.state.player(src).hand.len()
                                != ctx.state.player(owner).hand.len()
                    }).unwrap_or(false)),
            // 鬼眼·复活面：自己濒死且存活
            HostBehavior::SwdGuiyan2 => matches!(event,
                EventData::Dying { player, .. }
                if *player == owner && ctx.state.player(owner).alive),
            // 狩猎：Sha 机器内联（无触发 filter）
            HostBehavior::SwdShoulie => matches!(event,
                EventData::Phase { player, begin, .. } if *player == owner && false),
            // 不死：自己濒死且存活
            HostBehavior::SwdBusi => matches!(event,
                EventData::Dying { player, .. }
                if *player == owner && ctx.state.player(owner).alive),
            // 唤魂：任意角色濒死（hp≤0 面）且自己 he 有牌
            HostBehavior::SwdHuanhun => matches!(event,
                EventData::Dying { player, .. }
                if ctx.state.player(*player).hp <= 0
                    && (!ctx.state.player(owner).hand.is_empty()
                        || !ctx.state.player(owner).equips.is_empty())),
            // 倾城（A2 H70）：自己整回合结束（上游 player phaseEnd）
            HostBehavior::SwdLqingcheng => matches!(event,
                EventData::TurnEndWindow { player, after: false }
                if *player == owner),
            // 因果：他人受到有来源伤害前置（来源≠自己、自己 he 有牌）
            HostBehavior::SwdYinguo => matches!(event,
                EventData::Damage(d)
                if d.target != owner
                    && !d.prevented
                    && d.source.map(|src| src != owner).unwrap_or(false)
                    && (!ctx.state.player(owner).hand.is_empty()
                        || !ctx.state.player(owner).equips.is_empty())),
            // 拷贝栏/奥剑：内联/主动通道，无触发 filter
            HostBehavior::SwdKbolan => matches!(event,
                EventData::Phase { player, begin, .. } if *player == owner && false),
            HostBehavior::SwdAojian => matches!(event,
                EventData::Phase { player, begin, .. } if *player == owner && false),
            // 化金：出牌主动技，无触发 filter
            HostBehavior::SwdHuajin => matches!(event,
                EventData::Phase { player, begin, .. } if *player == owner && false),
            // 化金触发面（A2 H41）：自己造成伤害前置，或自己回合开始
            //（自清；原锚自己判定阶段开始）
            HostBehavior::SwdHuajin2 => {
                matches!(event,
                    EventData::Damage(d)
                    if d.source == Some(owner) && !d.prevented
                ) || matches!(event,
                EventData::TurnBegin { player }
                if *player == owner)
            }
            // 明镜：自己受伤前置 + 无防具（1/3 在机器步掷；登记差异：上游
            // 在 filter 掷点，原生 filter 只读故效果步掷——同净效）
            HostBehavior::SwdMinjing => matches!(event,
            EventData::Damage(d)
            if d.target == owner
                && !d.prevented
                && d.amount > 0
                && !ctx.state.player(owner).equips.iter().any(|c| {
                    ctx.catalog.card_slot_of(ctx.state, *c)
                        == Some(crate::catalog::EquipSlot::Armor)
                })),
            // 兰芷：自己使用的牌花色为梅花（未取消）
            HostBehavior::SwdLanzhi => matches!(event,
                EventData::CardUsed(u)
                if u.player == owner
                    && !u.cancelled
                    && u.suit(ctx.catalog)==Some(crate::catalog::Suit::Club)),
            // 烈刃：自己以杀造成真实伤害且受害者存活有牌
            HostBehavior::SwdLieren => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && d.card
                        .map(|c| ctx.catalog.card_kind(c).is_sha_like())
                        .unwrap_or(false)
                    && ctx.state.player(d.target).alive
                    && (!ctx.state.player(d.target).hand.is_empty()
                        || !ctx.state.player(d.target).equips.is_empty())),
            // 仙匠·车芸面：自己使用牌且唯一目标为他人、目标装备区有
            // 自己未持有（按目录牌种）的牌
            HostBehavior::SwdCyxianjiang => {
                matches!(event,
                EventData::CardUsed(u)
                if u.player == owner
                    && u.targets.len() == 1
                    && u.targets[0] != owner
                    // 上游 filter 还有「该目标未被记录过（每目标一次）」：
                    // `player.storage.cyxianjiang.includes(event.target)` ⇒ 用整局 once 记账
                    && !ctx.state.vm_aux.once.contains(&(
                        u.targets[0],
                        format!("cyxianjiang:{owner}"),
                        3,
                    ))
                    && ctx
                        .state
                        .player(u.targets[0])
                        .equips
                        .iter()
                        .any(|c| {
                            // 上游 `countCards("e", card => !player.countCards("he", card.name))`
                            // 比的是**牌名**；第 1001 轮前误用 `card_kind`（装备都归为 Weapon
                            // ⇒ 手里有任意武器就永不触发，比上游更严）
                            let name = ctx.catalog.cards[*c as usize].name;
                            !ctx.state
                                .player(owner)
                                .hand
                                .iter()
                                .chain(ctx.state.player(owner).equips.iter())
                                .any(|o| ctx.catalog.cards[*o as usize].name == name)
                        }))
            }
            // 幻形·移除面：自己受伤结算后（在位旗标=状态技能实例）
            HostBehavior::SwdHuanxing2 => {
                let in_place = ctx
                    .catalog
                    .skill_by_name(SWD_HUANXING_STATE_KEY)
                    .map(|d| ctx.state.skills.contains_key(&(owner, d)))
                    .unwrap_or(false);
                matches!(event,
                    EventData::Damage(d)
                    if d.target == owner && !d.prevented && in_place)
            }
            // 激昂·凤天凌面：①自己使用红杀/决斗（指定唯一目标）②成为
            // 红杀/决斗的目标（他人使用）——上游双面并集；登记差异：
            // 目标面按 CardUsed+targets 表达（useCardToTargeted 等价），
            // 来源面按 targets.len()==1 收窄
            HostBehavior::SwdJiang => {
                let red_sha_or_duel = |c: CardId| {
                    let k = ctx.catalog.card_kind(c);
                    (k == CardKind::Duel)
                        || (k.is_sha_like()
                            && matches!(
                                ctx.catalog.cards[c as usize].suit,
                                crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                            ))
                };
                match event {
                    EventData::CardUsed(u) => {
                        // 自己使用面：红杀/决斗且唯一目标
                        (u.player == owner
                            && u.targets.len() == 1
                            && (u.effective_kind(ctx.catalog)==CardKind::Duel||u.card.is_some_and(red_sha_or_duel)))
                        // 成为目标面：他人使用红杀/决斗且自己在其目标中
                        || (u.player != owner
                            && u.targets.contains(&owner)
                            && (u.effective_kind(ctx.catalog)==CardKind::Duel||u.card.is_some_and(red_sha_or_duel)))
                    }
                    _ => false,
                }
            }
            // 横刀·慧彦面：回合外用牌/响应结算（当前回合角色 he 有牌）
            HostBehavior::SwdHwendao => {
                let cur = ctx.state.turn.current;
                let he_has = {
                    let p = ctx.state.player(cur);
                    !p.hand.is_empty() || !p.equips.is_empty()
                };
                matches!(event,
                    EventData::CardUsed(u)
                    if u.player == owner && cur != owner && he_has
                ) || matches!(event,
                    EventData::CardResponded { card, player }
                    if *player == owner && cur != owner && he_has
                )
            }
            // 凌锋（A2 H69）：自己整回合结束（上游 player phaseEnd；原钉
            // 出牌阶段结束＝错位族随迁；出牌数门在机器步求取）
            HostBehavior::SwdLingfeng => matches!(event,
                EventData::TurnEndWindow { player, after: false }
                if *player == owner),
            // 冰箭/神鼎/鬼眼：出牌主动技（ActivateSkill 通道），无触发面
            // 袭西·凤钰面：自己出牌阶段开始且 he 有牌（候选面机器步求取）
            HostBehavior::SwdShangxi => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Play, begin }
                if *player == owner && *begin
                    && (!ctx.state.player(owner).hand.is_empty()
                        || !ctx.state.player(owner).equips.is_empty())),
            // 花间·剑仙面：自己出牌阶段结束且 he 有牌
            HostBehavior::SwdHuajian => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Play, begin }
                if *player == owner && !*begin
                    && (!ctx.state.player(owner).hand.is_empty()
                        || !ctx.state.player(owner).equips.is_empty())),
            // 观虎：自己杀伤害结算后、对方有牌（手/装备任一）
            HostBehavior::SwdGuanhu => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && d.card.map(|c| ctx.catalog.card_kind(c).is_sha_like()).unwrap_or(false)
                    && (!ctx.state.player(d.target).hand.is_empty()
                        || !ctx.state.player(d.target).equips.is_empty())),
            // 苏生：任意角色死亡前置、自己有手牌（每回合一次走 limits 在
            // OptionalGate 消费；目标体力重置在机器步写回）
            HostBehavior::SwdSusheng => matches!(event,
                EventData::Death { .. }
                if !ctx.state.player(owner).hand.is_empty()),
            // 挥琪：自己受伤结算后（hp<maxHp 门在 filter——X=损失值）
            HostBehavior::SwdHuiqi => matches!(event,
                EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && ctx.state.player(owner).hp < ctx.state.player(owner).max_hp as i32
                    && ctx.state.alive_players().any(|p| p != owner)),

            // 东皇钟（A2 H64）：自己整回合结束且手牌有红牌（蝶舞口径）
            HostBehavior::SwdDonghuangzhongCard => matches!(event,
            EventData::TurnEndWindow { player, after: false }
            if *player == owner
                && ctx.state.player(owner).hand.iter().any(|c| {
                    matches!(
                        ctx.catalog.cards[*c as usize].suit,
                        crate::catalog::Suit::Heart
                            | crate::catalog::Suit::Diamond
                    )
                })),
            // 壶境（A2 H35）：上游 player phaseBegin＝自己回合开始入场且
            // 弃牌堆有炼妖壶（原锚自己判定阶段开始）
            HostBehavior::SwdHujing => matches!(event,
            EventData::TurnBegin { player }
            if *player == owner
                && ctx.state.discard.iter().any(|c| {
                    ctx.catalog.card_kind(*c) == CardKind::Lianyaohu
                })),
            // 朱羽：横置角色受到非传导伤害且持有者有红牌（手/装备区）
            HostBehavior::SwdZhuyu => matches!(event,
            EventData::Damage(d)
            if !d.prevented && d.amount > 0
                && !d.from_chain
                && ctx.state.player(d.target).linked
                && ctx.state.player(owner).hand.iter()
                    .chain(ctx.state.player(owner).equips.iter())
                    .any(|c| {
                        matches!(
                            ctx.catalog.cards[*c as usize].suit,
                            crate::catalog::Suit::Heart
                                | crate::catalog::Suit::Diamond
                        )
                    })),
            // 落梅：持有者使用或打出梅花牌
            HostBehavior::SwdLuomei => match event {
                EventData::CardUsed(u) => {
                    u.player == owner
                        && u.suit(ctx.catalog)
                            .is_some_and(|s| matches!(s, crate::catalog::Suit::Club))
                }
                EventData::CardResponded { card, player } => {
                    *player == owner
                        && matches!(
                            ctx.catalog.cards[*card as usize].suit,
                            crate::catalog::Suit::Club
                        )
                }
                _ => false,
            },
            // 凝霜：成为黑色牌的目标（使用者≠持有者、未「横置且翻面」、
            // 持有者手/装备区有黑牌）
            HostBehavior::SwdNingshuang => matches!(event,
                EventData::CardUsed(u)
                if u.player != owner
                    && u.targets.contains(&owner)
                    && !u.cancelled
                    && u.suit(ctx.catalog).is_some_and(|s|matches!(s,crate::catalog::Suit::Spade | crate::catalog::Suit::Club))
                    && !(ctx.state.player(u.player).linked
                        && ctx.state.player(u.player).face_down)
                    && ctx.state.player(owner).hand.iter()
                        .chain(ctx.state.player(owner).equips.iter())
                        .any(|c| matches!(ctx.catalog.cards[*c as usize].suit,
                            crate::catalog::Suit::Spade | crate::catalog::Suit::Club))),
            // 工神：其他角色使用基本/锦囊牌且有目标（持有者手/装备区有
            // 装备牌；地/机关等异类种不计）
            HostBehavior::SwdGongshen => matches!(event,
            EventData::CardUsed(u)
            if u.player != owner
                && !u.cancelled
                && !u.targets.is_empty()
                && {
                    let k = u.effective_kind(ctx.catalog);
                    k.is_basic_kind()
                        || (k.is_basic_trick_or_equip() && k.equip_slot().is_none())
                }
                && (!ctx.state.player(owner).equips.is_empty()
                    || ctx.state.player(owner).hand.iter().any(|c| {
                        ctx.catalog.card_slot_of(ctx.state, *c).is_some()
                    }))),
            // 机关凤封印（A2 H43）：拥有者下一回合开始解除（上游
            // addTempSkill {player:'phaseBegin'}＝回合开始；原锚任意自己
            // 阶段开始）
            HostBehavior::SwdFengyinState => matches!(event,
                EventData::TurnBegin { player } if *player == owner),
            HostBehavior::SwdJiguanyuanGain => matches!(event,
                // A2 H71：自己整回合结束（上游 jiguanyuan player phaseEnd）
                EventData::TurnEndWindow { player, after: false }
                    if *player == owner && ctx.state.player(owner).alive
                        && !ctx.state.player(owner).jiguanyuan_cards.is_empty()),
            HostBehavior::SwdJiguanfengCard => false,
            HostBehavior::SwdJiguanyuanCard => false,
            HostBehavior::SwdJiguanyaoshuCard => false,
            HostBehavior::SwdJiguanyaoState => false,
            HostBehavior::SwdJiguanshuCard => false,
            HostBehavior::SwdTianfuzhenCard => false,
            HostBehavior::SwdQixingzhenCard => false,
            HostBehavior::SwdNiaoxiangzhenCard => false,
            HostBehavior::SwdChangshezhenCard => false,
            HostBehavior::SwdPozhenjueCard => false,
            // 机关妖术补偿：非自己回合、弃牌含装备 Kind 牌（以弃装备牌近
            // 似"失去装备区牌"，登记差异：替换/获得离去路径不触发）
            HostBehavior::SwdJiguanyaoGain => matches!(event,
            EventData::CardsDiscarded { player, cards }
                if *player != ctx.state.turn.current
                    && ctx.state.player(*player).alive
                    && cards.iter().any(|c| {
                        matches!(
                            ctx.catalog.card_slot_of(ctx.state, *c),
                            Some(_)
                        )
                    })),
            HostBehavior::SwdXuanfengzhirenCard => false,
            HostBehavior::SwdZhongxinghujiaCard => false,
            HostBehavior::SwdXingtigaizaoCard => false,
            HostBehavior::SwdShijianhuisuCard => false,
            HostBehavior::SwdJinjilengdongCard => false,
            HostBehavior::SwdShengxiuhaojiaoCard => false,
            HostBehavior::SwdYinmilichangCard => false,
            // 勾魂锣·倒计时（A2 H44）：上游 global phaseBegin 定向＝仅来源
            // player 的回合开始（原锚任意阶段开始）
            HostBehavior::SwdGouhunluoSub => matches!(event,
                EventData::TurnBegin { player }
                if ctx.state.player(owner).gouhunluo_source == Some(*player)),
            _ => unreachable!("behavior routed to the wrong filter module"),
        }
    }
}
