use super::super::*;

impl HostSkillBehavior {
    pub(super) fn filter_common(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
        event: &EventData,
    ) -> bool {
        match self.behavior {
            // 厉焰·链杀：持有者以水攻用牌（使用者=持有者；kind 按**结算
            // 种**（use_as 转化声明优先，目录种兜底）——厉焰转化的牌目录
            // 种非水攻）
            // 厉焰·链杀：持有者以水攻用牌（结算种 use_as 优先；语义集
            // 中在 helper）
            HostBehavior::YunchouLuweiyan2 => u_matches_lw2(event, owner, ctx),
            HostBehavior::YunchouJieyan => false,
            HostBehavior::YunchouYiesheng => false,
            HostBehavior::WudianDiyjuntun => false,
            HostBehavior::WudianJianbi => false,
            // 复仇·清算：自己 End 阶段结束（登记差异：上游 global
            // phaseAfter，原生自己回合结束）
            HostBehavior::WudianFuchou2 => matches!(event,
                // A2 H72：上游 `{global:'phaseAfter'}`＝任意角色整回合结束后
                //（每轮 N 次）＋filter「标记里有存活者」（第 986 轮补非空条件）
                EventData::TurnEndWindow { after: true, .. }
                if ctx.state.player(owner)
                    .fuchou_marked
                    .iter()
                    .any(|&t| ctx.state.player(t).alive)),
            HostBehavior::WudianFuchou => false,
            HostBehavior::WudianLiangceRemain => false,
            HostBehavior::WudianGuihan => false,
            // 禳星：自己受到未防止伤害 / 自己失去体力
            HostBehavior::HlssXiangxing => match event {
                EventData::Damage(d) => d.target == owner && !d.prevented,
                EventData::LoseHp { player, .. } => *player == owner,
                _ => false,
            },
            // 隐月：自己受到未防止伤害（前置窗）
            HostBehavior::HlssYueyin => matches!(event,
                EventData::Damage(d) if d.target == owner && !d.prevented),
            // 起风（A2 H02）：上游 player phaseZhunbeiBegin＝自己准备阶段
            // 开始（原锚自己判定阶段开始）
            HostBehavior::HlssFengqi => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Prepare, begin: true }
                if *player == owner),
            HostBehavior::HlssFengqi2 => false,
            HostBehavior::HlssGaiming => false,
            // 潜龙：开局首阶段（原生 phase_number 在触发前已 +1，故 ==1
            // 等价上游 game.phaseNumber == 0）/ 自己获得牌 / 自己受伤 /
            // 自己造成伤害（点数未满 99 才计）
            HostBehavior::MbQianlong => match event {
                // 开局面：游戏首个阶段帧（原生准备阶段开始在建局时隐式提交，
                // 首个阶段帧为摸牌阶段开始——登记差异：时点后移一个阶段）
                EventData::Phase { begin: true, .. } => ctx.state.turn.phase_number == 1,
                EventData::CardsDrawn { player, .. } => {
                    // 开局摸牌不是上游 gain 事件（gameDraw 不触发 gainEnd）：
                    // 首个阶段帧之前不计（登记差异）
                    ctx.state.turn.phase_number > 0
                        && *player == owner
                        && ctx.state.player(owner).mbqianlong_marks < 99
                }
                EventData::CardsMoved { to, .. } => {
                    ctx.state.turn.phase_number > 0
                        && *to == owner
                        && ctx.state.player(owner).mbqianlong_marks < 99
                }
                EventData::Damage(d) => {
                    ctx.state.player(owner).mbqianlong_marks < 99
                        && !d.prevented
                        && d.amount > 0
                        && (d.target == owner || d.source == Some(owner))
                }
                _ => false,
            },
            // 涅槃：自己濒死且未发动
            HostBehavior::Niepan => matches!(event,
                EventData::Dying { player, .. }
                    if *player == owner && !ctx.state.player(owner).niepan_used),
            HostBehavior::Yuhuo => false,
            HostBehavior::Shenwei => false,
            // 集控：自己阶段开始（任意阶段）/ 自己失去手牌且手牌已空
            HostBehavior::Hxunzhi => false,
            HostBehavior::Hxunzhi2 => false,
            HostBehavior::Gxianyin => false,
            HostBehavior::Jilve => false,
            HostBehavior::Gongxin => false,
            // 每步：他人出牌阶段开始，自己有牌，且其在攻击范围内能及自己
            HostBehavior::NewMeibu => match event {
                EventData::Phase {
                    player,
                    phase: crate::event::Phase::Play,
                    begin: true,
                } => {
                    *player != owner
                        && ctx.state.player(*player).alive
                        && (!ctx.state.player(owner).hand.is_empty()
                            || !ctx.state.player(owner).equips.is_empty())
                        && ctx.state.computed_distance(ctx.catalog, *player, owner)
                            <= ctx.state.attack_range(ctx.catalog, *player)
                }
                _ => false,
            },
            HostBehavior::NewMeibuRange => false,
            HostBehavior::SbZhaXiang => false,
            // 反击：自己为伤害承受者、手牌有【杀】、伤害载体为【杀】或【决斗】
            HostBehavior::YxsFanji => match event {
                EventData::Damage(d) if d.target == owner => {
                    let has_sha = ctx
                        .state
                        .player(owner)
                        .hand
                        .iter()
                        .any(|c| ctx.catalog.card_kind(*c).is_sha_like());
                    let by_sha_or_duel = d.card.is_some_and(|c| {
                        let k = ctx.catalog.card_kind(c);
                        k.is_sha_like() || k == crate::catalog::CardKind::Duel
                    });
                    has_sha && by_sha_or_duel
                }
                _ => false,
            },
            HostBehavior::YxsFanji2 => false,
            // 门神（A2 H51）：自己整回合结束且存活角色 >1（上游 player phaseEnd）
            HostBehavior::YxsMenshen => {
                matches!(event,
                EventData::TurnEndWindow { player, after: false } if *player == owner)
                    && ctx.state.alive_players().count() > 1
            }
            // 门神·清除（A2 H06/H51 配套）：上游 trigger `player:["phaseBegin",
            // "dieBegin"]`＝**门神持有者自己**的回合开始＋本人濒死——标记从
            // 其整回合结束授予存活续到其下一回合开始（原「任意阶段开始＋
            // 任意死亡」清会在 H51 迁整回合结束窗后立即清掉刚授予的标记）。
            HostBehavior::YxsMenshen3 => {
                let has_marker = ctx.state.alive_players().any(|p| {
                    ctx.catalog
                        .skill_by_name(U08_YXS_MENSHEN2_KEY)
                        .map(|def| ctx.state.skills.contains_key(&(p, def)))
                        .unwrap_or(false)
                });
                has_marker
                    && match event {
                        // A2 H06：phaseBegin＝持有者自己回合开始（每回合一次）。
                        EventData::TurnBegin { player } => *player == owner,
                        // A2 H06：dieBegin＝持有者本人濒死（原任意死亡过宽）。
                        EventData::Death { player, .. } => *player == owner,
                        _ => false,
                    }
            }
            HostBehavior::YxsMenshen2 => false,
            HostBehavior::XiguaMopaiFanbei => false,
            HostBehavior::Kurou => false,
            HostBehavior::FzLiegong => false,
            // 狂骨：自己为伤害来源、距离 ≤1 且伤害 >0
            HostBehavior::FzXinKuanggu => match event {
                EventData::Damage(d) if d.source == Some(owner) && d.amount > 0 => {
                    let dist = ctx.state.computed_distance(ctx.catalog, owner, d.target);
                    dist <= 1
                }
                _ => false,
            },
            // 主亡忠继：死亡者为主公且自己为忠臣
            HostBehavior::ZhuWanzhongJi => match event {
                EventData::Death { player, .. } => {
                    let roles = ctx.state.identity_roles().unwrap_or_default();
                    roles.get(*player as usize) == Some(&crate::state::Role::Lord)
                        && roles.get(owner as usize) == Some(&crate::state::Role::Loyalist)
                }
                _ => false,
            },
            // 血量翻倍：自己首次阶段开始（标记未置位）
            HostBehavior::XiguaXueliangFanbei => match event {
                EventData::Phase {
                    player,
                    begin: true,
                    ..
                } if *player == owner => {
                    ctx.state
                        .vm_aux
                        .marks
                        .get(&(
                            owner,
                            ctx.catalog
                                .skill_by_name(L20_XUELIANG_KEY)
                                .map(u32::from)
                                .unwrap_or_default(),
                        ))
                        .copied()
                        .unwrap_or(0)
                        == 0
                }
                _ => false,
            },
            // 灾变：自己的出牌阶段开始
            HostBehavior::Zaibian => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Play, begin: true }
                    if *player == owner),
            // 誓师：自己为击杀者
            HostBehavior::Shishi => matches!(event,
                EventData::Death { killer, .. } if *killer == Some(owner)),
            // 循梦：自己为伤害来源、伤害载体为【杀】且非铁索传导
            HostBehavior::Xunmeng => matches!(event,
                EventData::Damage(d) if d.source == Some(owner)
                    && !d.from_chain
                    && d.card.is_some_and(|c| ctx.catalog.card_kind(c).is_sha_like())),
            HostBehavior::Jishe => false,
            // 止息：自己使用牌
            HostBehavior::NewZhixi => matches!(event,
                EventData::CardUsed(u) if u.player == owner),
            // 撷芳：死亡事件（任意角色）或**自己的**摸牌阶段结束
            HostBehavior::Xiefang => match event {
                EventData::Death { .. } => true,
                EventData::Phase {
                    player,
                    phase: crate::event::Phase::Draw,
                    begin: false,
                } => *player == owner,
                _ => false,
            },
            // 让节：自己为该伤害承受者且未防止
            HostBehavior::Rangjie => matches!(event,
                EventData::Damage(d) if d.target == owner && !d.prevented),
            HostBehavior::Reguhuo => false,
            // 魔化：自己进入濒死
            HostBehavior::Mohua2 => matches!(event,
                EventData::Dying { player, .. } if *player == owner),
            // 修罗（A2 H09）：上游 player phaseZhunbeiBegin＝自己准备阶段
            // 开始、判定区有牌且手牌非空（原锚自己判定阶段开始）
            HostBehavior::Xiuluo => {
                matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Prepare, begin: true }
                    if *player == owner)
                    && !ctx.state.player(owner).judge.is_empty()
                    && !ctx.state.player(owner).hand.is_empty()
            }
            // 灭道·弃：自己弃牌阶段开始且体力未满
            HostBehavior::Miedao => {
                matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Discard, begin: true }
                    if *player == owner)
                    && ctx.state.player(owner).hp < ctx.state.player(owner).max_hp as i32
            }
            HostBehavior::Moyan => false,
            // 疾风：自己使用【杀】或单目标锦囊且目标数 >1、未取消
            HostBehavior::Jifeng => match event {
                EventData::CardUsed(u) => {
                    if u.player != owner || u.cancelled || u.targets.len() <= 1 {
                        return false;
                    }
                    let kind = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
                    kind.is_sha_like() || jifeng_single_target_trick(kind)
                }
                _ => false,
            },
            // 幻镜（A2 H11）：快照＝本人整回合结束后（phaseAfter 段）；
            // 比对＝本人回合开始（TurnBegin 窗）且体力低于快照值。
            HostBehavior::Kunlunjing => match event {
                EventData::TurnEndWindow { player, after: true } => *player == owner,
                EventData::TurnBegin { player } => {
                    *player == owner
                        && !ctx.state.turn.kunlunjing_snapshot.is_empty()
                        && ctx.state.player(owner).hp < ctx.state.turn.kunlunjing_hp
                }
                _ => false,
            },
            HostBehavior::Shengong => false,
            // 轩辕剑·伤增：自己是伤害来源且未横置伤害
            HostBehavior::XuanyuanjianDamage => matches!(event,
                EventData::Damage(d)
                    if d.source == Some(owner) && !d.prevented && !d.from_chain),
            // 轩辕剑·反噬：自己是伤害来源且本次加伤生效（标记位）
            HostBehavior::XuanyuanjianBacklash => matches!(event,
                EventData::Damage(d) if d.source == Some(owner) && ctx.state.turn.xuanyuanjian_marked),
            // 轩辕（A2 H53）：**自己回合**的整回合结束、有黑桃牌且弃牌堆有
            //【轩辕剑】（上游 trigger `{ player: 'phaseEnd' }`＝每回合一次）
            HostBehavior::Xuanyuan => {
                matches!(event,
                    EventData::TurnEndWindow { player, after: false } if *player == owner)
                    && ctx
                        .state
                        .player(owner)
                        .hand
                        .iter()
                        .chain(ctx.state.player(owner).equips.iter())
                        .any(|c| ctx.catalog.cards[*c as usize].suit == crate::catalog::Suit::Spade)
                    && ctx.state.discard.iter().any(|c| {
                        ctx.catalog.cards[*c as usize].kind
                            == crate::catalog::CardKind::Xuanyuanjian
                    })
            }
            // 燎原：自己是【杀】的使用者且持有与该杀同花色的牌
            HostBehavior::Liaoyuan => match event {
                EventData::CardUsed(u) => {
                    if u.player != owner || u.cancelled {
                        return false;
                    }
                    let kind = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
                    if !kind.is_sha_like() {
                        return false;
                    }
                    let Some(suit) = u.suit(ctx.catalog) else {
                        return false;
                    };
                    ctx.state
                        .player(owner)
                        .hand
                        .iter()
                        .chain(ctx.state.player(owner).equips.iter())
                        .any(|c| ctx.catalog.cards[*c as usize].suit == suit)
                }
                _ => false,
            },
            // 改造：自己装备牌入槽前，且该牌固有栏位已被自己占用（有空格）
            HostBehavior::Gaizao => match event {
                EventData::Equipped { player, card, .. } => {
                    if *player != owner {
                        return false;
                    }
                    let Some(kind_slot) = ctx.catalog.card_kind(*card).equip_slot() else {
                        return false;
                    };
                    let occupied = |slot: crate::catalog::EquipSlot| {
                        ctx.state
                            .player(owner)
                            .equips
                            .iter()
                            .any(|c| ctx.catalog.card_slot_of(ctx.state, *c) == Some(slot))
                    };
                    occupied(kind_slot)
                        && [
                            crate::catalog::EquipSlot::Weapon,
                            crate::catalog::EquipSlot::Armor,
                            crate::catalog::EquipSlot::HorsePlus,
                            crate::catalog::EquipSlot::HorseMinus,
                            crate::catalog::EquipSlot::Treasure,
                        ]
                        .iter()
                        .any(|s| !occupied(*s))
                }
                _ => false,
            },
            // 李香龙：自己是该杀的使用者且闪避者存活
            HostBehavior::Lxianglong => matches!(event,
                EventData::ShaDodged { source, target, .. }
                    if *source == owner && ctx.state.player(*target).alive),
            // 移花：他人使用的**单目标**牌以自己为目标、且自己手牌 ≥2
            HostBehavior::Yihua => match event {
                EventData::CardUsed(u) => {
                    !u.cancelled
                        && u.player != owner
                        && u.targets.len() == 1
                        && u.targets[0] == owner
                        && ctx.state.player(owner).hand.len() >= 2
                }
                _ => false,
            },
            // 贞烈：他人使用【杀】或锦囊牌指定自己为目标且未取消
            HostBehavior::Zhenlie => match event {
                EventData::CardUsed(u) => {
                    if u.player == owner || u.cancelled || !u.targets.contains(&owner) {
                        return false;
                    }
                    let kind = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
                    kind.is_sha_like() || (!kind.is_basic_kind() && !kind.is_equip())
                }
                _ => false,
            },
            HostBehavior::Tianlun => false,
            // 脉轮（A2 H12）：上游 player phaseBegin＝自己回合开始（每回合
            // 一次；原「任意自己阶段开始」＝每回合 6 次）
            HostBehavior::Mailun => matches!(event,
                EventData::TurnBegin { player } if *player == owner),

            // 冰封·封：自己已翻回正面（移除自身）
            HostBehavior::Bingfeng2 => {
                matches!(event,
                EventData::TurnOvered { player, .. } if *player == owner)
                    && !ctx.state.player(owner).face_down
            }
            HostBehavior::Bingfeng => false,
            // 此桥邪：自己失去的牌中含装备区来源
            HostBehavior::Cyqiaoxie => match event {
                EventData::CardsLost {
                    player, origins, ..
                } => {
                    *player == owner
                        && origins
                            .iter()
                            .any(|o| *o == crate::event::LostOrigin::Equip)
                }
                _ => false,
            },
            // 护天（A2 H54）：**自己回合**的整回合结束、手牌非空且当前无寄存
            //（上游 trigger `{ player: 'phaseEnd' }`＝每回合一次；寄存主面在此，
            //  取回面在内核阶段收束，跨一个阶段边界）
            HostBehavior::Hutian => {
                matches!(event,
                EventData::TurnEndWindow { player, after: false } if *player == owner)
                    && !ctx.state.player(owner).hand.is_empty()
                    && ctx.state.player(owner).hutian_deposit.is_none()
            }
            // 天香：自己受到未防止伤害且手中有红桃牌
            HostBehavior::Tianxiang => matches!(event,
            EventData::Damage(d)
            if d.target == owner
                && !d.prevented
                && d.amount > 0
                && ctx.state.player(owner).hand.iter().any(|c| {
                    ctx.catalog.cards[*c as usize].suit == crate::catalog::Suit::Heart
                })),
            // 天香·受：自己是该伤害的承受者
            HostBehavior::Tianxiang2 => matches!(event,
                EventData::Damage(d) if d.target == owner && !d.prevented),
            HostBehavior::Songci => false,
            // 清正：自己出牌阶段开始且手牌非空
            HostBehavior::MbcmQingzheng => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Play, begin: true }
                if *player == owner && !ctx.state.player(owner).hand.is_empty()),
            // 卫统：开局首阶段 + 自己持潜龙 + 另有魏势力持本技者
            HostBehavior::MbWeitong => match event {
                EventData::Phase { begin: true, .. } => {
                    ctx.state.turn.phase_number == 1
                        && ctx
                            .catalog
                            .skill_by_name(HLSS_MBQIANLONG_KEY)
                            .map(|d| ctx.state.has_skill(owner, d))
                            .unwrap_or(false)
                        && ctx.state.alive_players().any(|p| {
                            p != owner
                                && ctx.state.player(p).general_slots.iter().any(|g| {
                                    g.map(|g| {
                                        ctx.catalog.general_meta(g).map(|m| m.1)
                                            == Some(crate::catalog::Faction::Wei)
                                    })
                                    .unwrap_or(false)
                                })
                                && ctx
                                    .catalog
                                    .skill_by_name(HLSS_MBWEITONG_KEY)
                                    .map(|d| ctx.state.has_skill(p, d))
                                    .unwrap_or(false)
                        })
                }
                _ => false,
            },
            // 隐月·判：自己未被翻面时取消翻面（上游 filter !isTurnedOver）
            HostBehavior::HlssYueyin2 => matches!(event,
                EventData::TurnOverBefore { player, .. }
                    if *player == owner && !ctx.state.player(owner).face_down),
            // 鬼蛇旗/仙露回：无触发面（CardUsed 内建直派）
            HostBehavior::GuisheqiCard => false,
            HostBehavior::XianluhuiCard => false,
            // 悲歌：伤害由杀造成、受伤者在场、持有者有牌
            HostBehavior::YunchouBeige => matches!(event,
                EventData::Damage(d)
                if !d.prevented && d.amount > 0
                    && d.card.map(|c| ctx.catalog.card_kind(c).is_sha_like()).unwrap_or(false)
                    && ctx.state.player(d.target).alive
                    && (!ctx.state.player(owner).hand.is_empty()
                        || !ctx.state.player(owner).equips.is_empty())),
            HostBehavior::WudianLiduan => matches!(event,
                EventData::CardsMoved { cards, from, to }
                if *to != owner
                    && from != to
                    && cards.len() == 1
                    && ctx.state.player(*to).alive
                    && ctx.state.turn.current != *to
                    && ctx.state.player(*to).hand.contains(&cards[0])
                    && ctx.catalog.card_slot_of(ctx.state, cards[0]).is_some()),
            // 蝉缘·变体：有来源的死亡
            HostBehavior::YunchouDiychanyuan => matches!(
                event,
                EventData::Death {
                    killer: Some(_),
                    ..
                }
            ),
            // 蛊惑·变体（A2 H17）：上游 player phaseBegin＝自己回合开始且
            // 区域有牌（原锚自己判定阶段开始）
            HostBehavior::YunchouDiyguhuo => matches!(event,
                EventData::TurnBegin { player }
                if *player == owner
                    && (!ctx.state.player(owner).hand.is_empty()
                        || !ctx.state.player(owner).equips.is_empty()
                        || !ctx.state.player(owner).judge.is_empty())),
            // 狡黠：成为红色牌的目标（使用者不限）
            HostBehavior::YunchouJiaoxia => matches!(event,
            EventData::CardUsed(u)
            if u.targets.contains(&owner)
                && u.suit(ctx.catalog).is_some_and(|s|matches!(s,crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond))),
            // 红莲：受到来源为其他角色的伤害且来源有红牌
            HostBehavior::YunchouHonglian => matches!(event,
            EventData::Damage(d)
            if d.target == owner && !d.prevented && d.amount > 0
                && d.source.is_some() && d.source != Some(owner)
                && {
                    let src = ctx.state.player(d.source.unwrap());
                    src.hand.iter().chain(src.equips.iter()).any(|c| {
                        matches!(
                            ctx.catalog.cards[*c as usize].suit,
                            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                        )
                    })
                }),
            // 纵火：自己造成的非火伤害（前置窗）
            HostBehavior::YunchouZonghuo => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && d.nature != crate::event::DamageNature::Fire),
            // 烧营：自己造成的火伤害
            HostBehavior::YunchouShaoying => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.nature == crate::event::DamageNature::Fire),
            // 雄姿：自己摸牌阶段开始（固定摸 2 后补摸）
            HostBehavior::YunchouXiongzi => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Draw, begin: true }
                if *player == owner),
            // 雅量：自己造成或受到伤害，双方存活且异人
            HostBehavior::YunchouYaliang => matches!(event,
                EventData::Damage(d)
                if !d.prevented && d.amount > 0
                    && (d.source == Some(owner) || d.target == owner)
                    && d.source.is_some()
                    && d.source != Some(d.target)
                    && ctx.state.player(d.source.unwrap()).alive
                    && ctx.state.player(d.target).alive),
            // 戏才：自己受到有牌伤害（可选确认门走 trigger_meta optional）
            HostBehavior::YunchouXicai => matches!(event,
                EventData::Damage(d)
                if d.target == owner && !d.prevented && d.amount > 0 && d.card.is_some()),
            // 狂骨·变体：自己造成伤害；满血时要求目标攻击距离 >1（受伤
            // 恒真——回复分支在效果内自检）
            HostBehavior::YunchouDiykuanggu => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner) && !d.prevented && d.amount > 0 && d.target != owner
                    && (ctx.state.player(owner).hp < ctx.state.player(owner).max_hp as i32
                        || ctx.state.seat_distance(d.target, owner) > 1)),
            HostBehavior::ShaDamageForceDiscard => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && !d.from_chain
                    && d.card.is_some_and(|c| ctx.catalog.card_kind(c).is_sha_like())
                    && ctx.state.player(d.target).alive
                    && (!ctx.state.player(d.target).hand.is_empty()
                        || !ctx.state.player(d.target).equips.is_empty())),
            HostBehavior::PhaseEndRecoverOrDraw => matches!(event,
                // A2 H50：整回合结束窗 phaseEnd 段（上游 player phaseEnd）。
                EventData::TurnEndWindow { player, after: false } if *player == owner),
            HostBehavior::TargetDiscardEquipOrDamage => false,
            HostBehavior::ShengkeCheck => matches!(event,
                EventData::CardUsed(u)
                if u.targets.contains(&owner)
                    && u.card.is_some_and(|c|ctx.state.card_natures.contains_key(&c))
                    && ctx.state.player(owner).wunature.is_some()),
            HostBehavior::ChangeCardNature => false,
            HostBehavior::WangmeiZhikeCard => false,
            HostBehavior::ShuigongCard => false,
            HostBehavior::FudichouxinCompare => false,
            HostBehavior::ChenhuodajieCard => chenhuodajie_damage_gate(ctx, owner, event),
            HostBehavior::TouliangSwap => false,
            HostBehavior::LinghunzhihuoCard => false,
            HostBehavior::ZhaomingdanCard => false,
            HostBehavior::JihuocardDraw => false,
            HostBehavior::GeanguanhuoCompare => false,
            HostBehavior::CaochuanCard => false,
            HostBehavior::ShezhanqunruCompare => false,
            HostBehavior::GujianShujinsanCard => false,
            HostBehavior::GujianMianliCard => false,
            HostBehavior::GujianFeibiaoCard => false,
            HostBehavior::JinlianzhuCancel => matches!(event,
                EventData::Damage(d)
                if !d.prevented && d.amount > 0
                    && ctx.state.player(owner).hand.iter()
                        .any(|c| ctx.catalog.card_kind(*c) == CardKind::Jinlianzhu)),
            HostBehavior::MoshouImmunity => false,
            HostBehavior::SanjianWeaponDamage => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner) && !d.prevented && d.amount > 0
                    && d.card.map(|c| ctx.catalog.card_kind(c).is_sha_like()).unwrap_or(false)
                    && !ctx.state.player(owner).hand.is_empty()),
            HostBehavior::SijiDiscardDraw => matches!(event,
                EventData::CardsDiscarded { player, cards }
                if *player == owner
                    && ctx.state.turn.current == owner
                    && ctx.state.turn.phase == crate::event::Phase::Discard
                    && cards.iter().any(|c| ctx.catalog.card_kind(*c).is_sha_like())),
            HostBehavior::MaxRangeWeaponDamage => matches!(event,
            EventData::Equipped { player, card, .. }
            if *player == owner
                && ctx.catalog.card_kind(*card) == CardKind::Weapon
                && {
                    // 攻击范围全场最大（严格大于其余所有存活角色）
                    let my = ctx.state.attack_range(ctx.catalog, owner);
                    ctx.state.alive_players().filter(|p| *p != owner).all(|p| {
                        ctx.state.attack_range(ctx.catalog, p) < my
                    })
                }),
            HostBehavior::WudianZhenchaDraw => matches!(event,
                EventData::CardUsed(u)
                if u.player == owner
                    && !u.cancelled
                    && u.use_as.unwrap_or(u.effective_kind(ctx.catalog)) == CardKind::Sha
                    && ctx.state.player(owner).baolei_mode == 2),
            HostBehavior::WudianZiyu => matches!(event,
                // A2 H75：任意玩家整回合结束＋phase_number%4 门（上游 global
                // phaseEnd；phaseNumber 口径差异登记规则基准）
                EventData::TurnEndWindow { after: false, .. }
                if ctx.state.turn.phase_number % 4 == 0),
            HostBehavior::WudianGuangshuSpade => matches!(event,
                // A2 H74：自己整回合结束（上游 guangshu_spade player phaseEnd；
                // 雷伤+移除在内核 K13 直推，宿主无机器步骤）
                EventData::TurnEndWindow { player, after: false } if *player == owner),
            HostBehavior::GujianFoodGudonggeng => matches!(event,
                EventData::Damage(d)
                if d.target == owner && !d.prevented && d.amount > 0
                    && d.card.map(|c| ctx.catalog.card_kind(c).is_sha_like()).unwrap_or(false)),
            HostBehavior::GujianFoodXiajiao => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Draw, begin: true }
                if *player == owner),
            HostBehavior::GujianFoodLiyutang => matches!(event,
                // A2 H57：整回合结束窗 phaseEnd 段（上游 liyutang player
                // phaseEnd＝每回合一次；第 991 轮登记）
                EventData::TurnEndWindow { player, after: false }
                if *player == owner),
            HostBehavior::GujianFoodMapodoufu => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: true }
                if *player == owner),
            HostBehavior::GujianFoodQingtuan => matches!(event,
                EventData::CardUsed(u)
                if u.player == owner
                    && u.effective_kind(ctx.catalog).is_sha_like()
                    && ctx.state.turn.current == owner
                    && ctx.state.turn.phase == crate::event::Phase::Play),
            HostBehavior::GujianFoodTanhuadong => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Play, begin: false }
                if *player == owner),
            // 酉羹（A2 H19）：上游 player phaseBegin＝自己回合开始（原锚
            // 自己判定阶段开始）
            HostBehavior::GujianFoodYougeng => matches!(event,
                EventData::TurnBegin { player }
                if *player == owner),
            // 紫阳丹（A2 H22）：上游 player phaseBegin＝自己回合开始（原锚
            // 自己结束阶段开始）
            HostBehavior::GujianFoodZiyangdan => matches!(event,
                EventData::TurnBegin { player }
                if *player == owner),
            HostBehavior::GujianYunvyuanshen => matches!(event,
                EventData::Dying { player, .. } if *player == owner),
            HostBehavior::GwentBirinState => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Play, begin: false }
                if *player == owner),
            HostBehavior::GwentCiguState => matches!(event,
                // 上游 `phaseDrawBegin`：摸牌阶段**开始**（原为 begin:false，晚一步）
                EventData::Phase { player, phase: crate::event::Phase::Draw, begin: true }
                if *player == owner),
            HostBehavior::GwentKunenState => matches!(event,
                EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.nature == crate::event::DamageNature::Normal),
            HostBehavior::GwentQinpenState => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Discard, begin: false }
                if *player == owner),
            HostBehavior::HuanleMonkeyCard => matches!(event,
            EventData::CardUsed(u)
            if u.player != owner
                && !u.cancelled
                && u.card.is_some()
                && u.effective_kind(ctx.catalog) == crate::catalog::CardKind::Tao
                && ctx.state.player(owner).equips.iter().any(|c| {
                    ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Monkey
                })),
            // 雪轮羊（A2 H24）：上游 player phaseBegin＝自己回合开始且
            // 装备雪轮羊＋场上有其他持技角色（原锚自己判定阶段开始）
            HostBehavior::HuanleXuelunyangCard => {
                let armed_and_targets = matches!(event,
                EventData::TurnBegin { player }
                if *player == owner
                    && ctx.state.player(owner).equips.iter().any(|c| {
                        ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Xuelunyang
                    }))
                    && ctx.state.players.iter().enumerate().any(|(i, p)| {
                        i as PlayerId != owner
                            && p.alive
                            && ctx
                                .state
                                .skills
                                .range((i as PlayerId, 0)..(i as PlayerId, u16::MAX))
                                .next()
                                .is_some()
                    });
                armed_and_targets
            }
            // 双刃·殁：自己死亡帧,
            // 阻魔手铐·封印（A2 H60）：本人整回合结束后到期（原「任意
            // 阶段结束」在准备阶段加入后每回合 6 次）。
            HostBehavior::GwentFengyinState => matches!(event,
                EventData::TurnEndWindow { player, after: true } if *player == owner),
            HostBehavior::HuanleMianjuCard => matches!(event,
            EventData::TurnOverBefore { player, .. }
            if *player == owner
                && ctx.state.player(owner).equips.iter().any(|c| {
                    ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Mianju
                })),
            HostBehavior::ZhangbaSha => false,
            HostBehavior::WudianShuaiyan => matches!(event,
                EventData::Recover { player, .. }
                if *player != owner
                    && ctx.state.turn.current != owner
                    && ctx.state.player(*player).alive),
            // 割据（A2 H45）：上游 player phaseBegin＝自己回合开始（原锚
            // 自己判定阶段开始）
            HostBehavior::WudianGeju => matches!(event,
                EventData::TurnBegin { player }
                if *player == owner),
            HostBehavior::WudianFeidan => matches!(event,
            EventData::Damage(d)
            if d.source == Some(owner)
                && !d.prevented
                && d.amount > 0
                && d.card.map(|c| ctx.catalog.card_kind(c).is_sha_like()).unwrap_or(false)
                && ctx.state.player(d.target).alive
                && ctx.state.alive_players().into_iter().any(|p| {
                    p != d.target
                        && ctx.state.computed_distance(ctx.catalog, d.target, p) <= 1
                })),
            HostBehavior::WudianGuangshuDiamond => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner) && !d.prevented && d.amount > 0),
            HostBehavior::WudianGuangshuHeart => matches!(event,
                EventData::Damage(d)
                if d.target == owner && !d.prevented && d.amount > 0),
            HostBehavior::WudianJidong => matches!(event,
                // A2 H78：任意玩家整回合结束（上游 global phaseEnd）
                EventData::TurnEndWindow { after: false, .. }
                if ctx.state.player(owner).hp == 1
                    && !ctx.state.player(owner).face_down),
            HostBehavior::WudianMengji => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && ctx.state.player(owner).hujia == 0
                    && !d.prevented
                    && !d.from_chain
                    && d.card.map(|c| ctx.catalog.card_kind(c).is_sha_like()).unwrap_or(false)),
            HostBehavior::WudianOwliyong => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Draw, begin: true }
                if *player == owner && ctx.state.player(owner).pingzhang_count > 0),
            HostBehavior::WudianPingzhang => matches!(event,
            EventData::Damage(d)
            if !d.prevented && d.amount > 0
                && ctx.state.player(owner).alive
                && (if d.target == owner {
                    !ctx.state.player(owner).pingzhang_self_used
                        && ctx.state.player(owner).hand.iter().chain(ctx.state.player(owner).equips.iter())
                            .any(|c| matches!(ctx.catalog.cards[*c as usize].suit, crate::catalog::Suit::Heart))
                } else {
                    !ctx.state.player(owner).pingzhang_other_used
                        && ctx.state.player(owner).hand.iter().chain(ctx.state.player(owner).equips.iter())
                            .any(|c| matches!(ctx.catalog.cards[*c as usize].suit, crate::catalog::Suit::Spade))
                })),
            // 应用：自己摸牌阶段开始+上回合有屏障次数,
            HostBehavior::WudianQiangguGuard => matches!(event,
                EventData::CardUsed(u)
                if !u.cancelled
                    && {
                        let k = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
                        k == CardKind::Sha
                    }
                    && u.targets.contains(&owner)
                    && ctx.state.player(owner).qianggu_active),
            HostBehavior::WudianShenqiang => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.card.map(|c| ctx.catalog.card_kind(c).is_sha_like()).unwrap_or(false)
                    && ctx.state.turn.current == owner),
            HostBehavior::WudianShihuo => matches!(event,
                EventData::Damage(d)
                if d.nature == crate::event::DamageNature::Fire
                    && !d.prevented
                    && d.amount > 0),
            // 猛击：你无护甲时的杀伤害（非传导；上游 filter storage.
            // zhongdun && !hujia && sha && notLink 同口径——zhongdun 置
            // 位以持技等价）,
            HostBehavior::WudianXiyang => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: false }
                if *player == owner
                    && (ctx.state.player(owner).hp as u32) < ctx.state.player(owner).max_hp
                    && !ctx.state.player(owner).face_down),
            HostBehavior::WudianYuedong => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Play, begin: false }
                if *player == owner),
            HostBehavior::WudianZhiyuan => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner) && !d.prevented && d.amount > 0),
            HostBehavior::WudianZhucheng => matches!(event,
                // 筑城（A2 H73，第 934 轮）：K14 在整回合结束窗 phaseEnd 段
                // 直推确认门；上游 filter 还要「无标记」
                //（`!storage.zhucheng || !storage.zhucheng.length`）
                EventData::TurnEndWindow { player, after: false }
                if *player == owner && ctx.state.player(owner).zhucheng.is_empty()),
            HostBehavior::WudianZhuchengGuard => matches!(event,
                EventData::CardUsed(u)
                if !u.cancelled
                    && {
                        let k = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
                        k == CardKind::Sha
                    }
                    && u.targets.contains(&owner)
                    && !ctx.state.player(owner).zhucheng.is_empty()),
            // 颂：自己造成伤害（未防止有量）
            HostBehavior::WudianYinshen => matches!(event,
            EventData::CardsDiscarded { player, cards }
            if *player == owner
                && !ctx.state.player(owner).yinshen_active
                && cards.iter().any(|c| {
                    matches!(
                        ctx.catalog.card_kind(*c),
                        CardKind::Sha | CardKind::Shan | CardKind::Tao | CardKind::Jiu
                    )
                })
                && !ctx.state.player(owner).hand.iter().any(|c| {
                    matches!(
                        ctx.catalog.card_kind(*c),
                        CardKind::Sha | CardKind::Shan | CardKind::Tao | CardKind::Jiu
                    )
                })),
            HostBehavior::WudianPaotaiFire => matches!(event,
                // A2 H76：自己整回合结束（上游 paotai2 player phaseEnd）
                EventData::TurnEndWindow { player, after: false }
                if *player == owner && ctx.state.player(owner).paotai_lv > 0),
            // 四象隐藏链 2/3 + 后援3/4：本人死亡且本人是 Boss
            HostBehavior::BossQingmu2
            | HostBehavior::BossQingmu3
            | HostBehavior::BossBaimang2
            | HostBehavior::BossChiyan2
            | HostBehavior::BossXuanlin2
            | HostBehavior::BossBaimang3
            | HostBehavior::BossChiyan3
            | HostBehavior::BossXuanlin3
            | HostBehavior::BossBianshen3
            | HostBehavior::BossBianshen4 => matches!(event,
                EventData::Death { player, .. }
                if *player == owner
                    && ctx.state.boss_mode().map(|(b, _)| b == owner).unwrap_or(false)),
            // 四象隐藏链·转阶 + 后援3/4·转阶：同上（技能仍在 DeathMachine 前）
            HostBehavior::BossQingmu2x
            | HostBehavior::BossQingmu3x
            | HostBehavior::BossBaimang2x
            | HostBehavior::BossChiyan2x
            | HostBehavior::BossXuanlin2x
            | HostBehavior::BossBaimang3x
            | HostBehavior::BossChiyan3x
            | HostBehavior::BossXuanlin3x
            | HostBehavior::BossBianshen3x
            | HostBehavior::BossBianshen4x => matches!(event,
                EventData::Death { player, .. }
                if *player == owner
                    && ctx.state.boss_mode().map(|(b, _)| b == owner).unwrap_or(false)),
            // 还人开局：关卡尚未进入十殿，且持有者是 Boss
            HostBehavior::BossHuanren => {
                ctx.state
                    .boss_mode()
                    .map(|(b, _)| b == owner)
                    .unwrap_or(false)
                    && matches!(
                        ctx.state.boss_stage().map(|s| s.kind),
                        Some(crate::state::BossStageKind::None)
                    )
            }
            // 还人转关：十殿链且等级未终（0..2 还有下一关）
            HostBehavior::BossHuanrenDie => matches!(event,
            EventData::Death { player, .. }
            if *player == owner
                && ctx.state.boss_mode().map(|(b, _)| b == owner).unwrap_or(false)
                && ctx.state.boss_stage().is_some_and(|s| {
                    s.kind == crate::state::BossStageKind::ShidianYanluo && s.level < 3
                })),
            HostBehavior::BossQingyi => wing_round_gate(ctx, owner, event, 0),
            HostBehavior::BossChiyi => wing_round_gate(ctx, owner, event, 3),
            HostBehavior::BossChiyiMark => matches!(event,
                EventData::Damage(d)
                if d.target == owner && !d.prevented && d.amount > 0),
            HostBehavior::BossBaiyi => {
                // phase: crate::event::Phase::Judge, begin: true
                wing_bai_gate(ctx, owner, event)
            }
            HostBehavior::BossBaiyiThunder => matches!(event,
                EventData::Damage(d)
                if d.target == owner
                    && d.nature == crate::event::DamageNature::Thunder
                    && !d.prevented
                    && d.amount > 0
                    && ctx.state.turn.round < 7
                    && ctx.state.boss_mode().map(|(b, _)| b == owner).unwrap_or(false)),
            HostBehavior::D3Moyan => moyan_filter(ctx, owner, event),
            // 穿云（太虚幻境·extension_skill.js:3806）：本人结束阶段开始，且有体力 > 自己的其他角色
            HostBehavior::BossChuanyun => {
                if !matches!(
                    event,
                    EventData::Phase {
                        player,
                        phase: crate::event::Phase::End,
                        begin: true,
                    } if *player == owner
                ) {
                    return false;
                }
                let owner_hp = ctx.state.player(owner).hp;
                ctx.state
                    .alive_players()
                    .any(|p| p != owner && ctx.state.player(p).hp > owner_hp)
            }
            // 天陨（太虚幻境·extension_skill.js:4085）：本人结束阶段开始，且有其他存活角色
            HostBehavior::BossTianyun => {
                matches!(
                    event,
                    EventData::Phase {
                        player,
                        phase: crate::event::Phase::End,
                        begin: true,
                    } if *player == owner
                ) && ctx.state.alive_players().any(|p| p != owner)
            }
            // 侠行（3D精选·character/skill.js:59）：每轮限一次（!hasSkill("dddxiaxing_used")）且 he 区有牌，
            // 任意角色准备阶段开始或濒死时触发（持有「侠」标记不阻碍触发，标记跨轮保留直至被转化消费）
            HostBehavior::DddXiaxing => {
                let round_tag = ctx.state.turn.round + 1000;
                let rec_round = ctx
                    .state
                    .vm_aux
                    .flags
                    .get(&(owner, "dddxiaxing_round".to_string()))
                    .copied()
                    .unwrap_or(0);
                let used_this_round = rec_round == round_tag;
                if used_this_round
                    || (ctx.state.player(owner).hand.is_empty()
                        && ctx.state.player(owner).equips.is_empty())
                {
                    return false;
                }
                match event {
                    EventData::Phase {
                        player,
                        phase: crate::event::Phase::Prepare,
                        begin: true,
                    } => ctx.state.player(*player).alive,
                    EventData::Dying { player, .. } => {
                        ctx.state.player(*player).alive && ctx.state.player(*player).hp <= 0
                    }
                    _ => false,
                }
            }
            HostBehavior::WudianDuoqi => matches!(
                event,
                EventData::CardsDiscarded { cards, .. }
                if !cards.is_empty()
                    && ctx.state.turn.current != owner
                    && ctx.state.turn.phase == crate::event::Phase::Play
                    && !ctx.state.player(owner).zhucheng.is_empty()
            ),
            HostBehavior::WudianYouzhan => match event {
                EventData::CardUsed(u) => {
                    if u.cancelled
                        || !u.effective_kind(ctx.catalog).is_sha_like()
                        || u.targets.is_empty()
                        || !ctx.state.player(owner).alive
                    {
                        return false;
                    }
                    let t0 = u.targets[0];
                    if !ctx.state.player(t0).alive {
                        return false;
                    }
                    let in_range =
                        t0 == owner || ctx.state.computed_distance(ctx.catalog, owner, t0) <= 1;
                    let has_equip = !ctx.state.player(owner).equips.is_empty()
                        || ctx
                            .state
                            .player(owner)
                            .hand
                            .iter()
                            .any(|c| ctx.catalog.card_kind(*c).is_equip());
                    in_range && has_equip
                }
                _ => false,
            },
            HostBehavior::WudianChoudu | HostBehavior::WudianDiaobingCard => false,
            _ => unreachable!("behavior routed to the wrong filter module"),
        }
    }
}
