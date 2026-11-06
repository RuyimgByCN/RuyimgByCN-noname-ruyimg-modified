use super::super::*;

impl HostSkillBehavior {
    pub(super) fn filter_xianjian(
        &self,
        ctx: &QueryCtx<'_>,
        owner: PlayerId,
        event: &EventData,
    ) -> bool {
        match self.behavior {
            HostBehavior::XianjianXiepan => matches!(event,
            EventData::CardsDiscarded { player, cards }
            if *player == owner
                && cards.iter().any(|c| {
                    matches!(
                        ctx.catalog.card_kind(*c),
                        CardKind::Sha | CardKind::Shan | CardKind::Tao
                    )
                })
                && !ctx.state.player(owner).hand.is_empty()
                && !ctx.state.player(owner).hand.iter().any(|c| {
                    matches!(
                        ctx.catalog.card_kind(*c),
                        CardKind::Sha | CardKind::Shan | CardKind::Tao
                    )
                })),
            HostBehavior::XianjianXjyingfeng => {
                // 影锋（U05 xianjian 越今朝，规则 v185）：自己用杀+链挡
                // 旗标未置+候选存在（其他角色-本杀目标，存活+距离内）
                // ——上游 card=sha && parent.name!=="xjyingfeng" &&
                // canUse 的登记差异等价。
                if let EventData::CardUsed(u) = event {
                    u.player == owner
                        && u.effective_kind(ctx.catalog) == crate::catalog::CardKind::Sha
                        && !ctx.state.turn.yingfeng_chain
                        && ctx.state.seat_order_from(owner).into_iter().any(|t| {
                            t != owner
                                && !u.targets.contains(&t)
                                && ctx.state.player(t).alive
                                && ctx.state.computed_distance(ctx.catalog, owner, t)
                                    <= ctx.state.attack_range(ctx.catalog, owner)
                        })
                } else {
                    false
                }
            }
            HostBehavior::XianjianXjzhimeng2 => {
                if let EventData::CardUsed(u) = event {
                    u.player == owner && {
                        let k = yangpijuan_class(u.effective_kind(ctx.catalog));
                        ctx.state
                            .player(owner)
                            .off_court
                            .iter()
                            .any(|o| yangpijuan_class(ctx.catalog.card_kind(*o)) == k)
                    }
                } else {
                    false
                }
            }
            HostBehavior::XianjianSheling => {
                let in_my_turn = ctx.state.turn.current == owner;
                let others = match event {
                    EventData::CardUsed(u) => u.player != owner,
                    EventData::CardResponded { player, .. } => *player != owner,
                    EventData::CardsDiscarded { player, .. } => *player != owner,
                    _ => false,
                };
                in_my_turn
                    && others
                    && ctx.state.turn.sheling_used < 3
                    && match event {
                        EventData::CardUsed(u) => {
                            u.card.is_some_and(|c| ctx.state.discard.contains(&c))
                        }
                        EventData::CardResponded { card, .. } => ctx.state.discard.contains(card),
                        EventData::CardsDiscarded { cards, .. } => {
                            cards.iter().any(|c| ctx.state.discard.contains(c))
                        }
                        _ => false,
                    }
            }
            HostBehavior::XianjianFanling => matches!(event,
                EventData::LoseHp { player, amount }
                if *player != owner
                    && *amount > 0
                    && !ctx.state.turn.fanling_used
                    && (ctx.state.player(owner).hp as u32) < ctx.state.player(owner).max_hp),
            HostBehavior::XianjianNingbin => matches!(event,
                EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.nature == crate::event::DamageNature::Thunder),
            HostBehavior::XianjianLingyu => {
                // A2 H81：自己整回合结束（上游 player phaseEnd）＋有人已受伤
                matches!(event,
                    EventData::TurnEndWindow { player, after: false }
                    if *player == owner)
                    && ctx.state.alive_players().any(|p| {
                        p != owner && (ctx.state.player(p).hp as u32) < ctx.state.player(p).max_hp
                    })
            }
            HostBehavior::XianjianTxianqu => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && ctx.state.turn.current == owner
                    && ctx.state.turn.phase == crate::event::Phase::Play
                    && !ctx.state.turn.txianqu_used),
            HostBehavior::XianjianCtianfu3 => matches!(event,
                EventData::Death { player, .. }
                if *player == owner
                    && ctx.state.player(owner).ctianfu_source.is_some()),
            HostBehavior::XianjianChangnian => matches!(event,
                EventData::Death { player, .. } if *player == owner),
            // 长念·思：自己阶段末（runtime 授技实例持有者）,
            HostBehavior::XianjianChangnian2 => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: false }
                if *player == owner),
            HostBehavior::XianjianChengxin => {
                matches!(event, EventData::Dying { .. })
                    && ctx.state.turn.round >= ctx.state.player(owner).chengxin_ready_round
            }
            HostBehavior::XianjianCtianfu2 => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && ctx.state.player(owner).ctianfu_source.is_some()),
            // 天符·清挂：挂附者死亡,
            HostBehavior::XianjianDuci => {
                matches!(event,
                EventData::CardsDiscarded { player, cards }
                if *player == owner
                    && cards.iter().any(|c| {
                        yangpijuan_class(ctx.catalog.card_kind(*c)) == Some(2)
                    }))
                    && ctx.state.alive_players().into_iter().any(|p| {
                        p != owner && ctx.state.computed_distance(ctx.catalog, owner, p) <= 1
                    })
            }
            HostBehavior::XianjianFenglue => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Play, begin: true }
                if *player == owner),
            HostBehavior::XianjianFenshi => {
                matches!(event,
                EventData::DyingSaved { player, .. } if *player == owner)
                    && !ctx.state.player(owner).jiangcheng_longhuo
            }
            // 龙火：自己阶段结束且已觉醒（持有龙火）,
            HostBehavior::XianjianHusha => {
                matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: false }
                if *player == owner)
                    && ctx.state.player(owner).husha_mark > 0
                    && ctx.state.alive_players().into_iter().any(|p| p != owner)
            }
            HostBehavior::XianjianHushaCount => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && ctx.state.turn.current == owner
                    && ctx.state.player(owner).husha_mark < 3),
            // 虎煞：有标记+（可虚拟杀：有其他存活目标）或（可虚拟南蛮：
            // 标记≥2）,
            HostBehavior::XianjianJtjubao => matches!(event,
            EventData::CardsDiscarded { player, cards }
            if *player != owner
                && ctx.state.turn.current != owner
                && !ctx.state.turn.jtjubao_used
                && cards.iter().any(|c| {
                    yangpijuan_class(ctx.catalog.card_kind(*c)) != Some(0)
                })),
            HostBehavior::XianjianLiefeng => matches!(event,
                EventData::CardUsed(u)
                if u.player == owner
                    && ctx.state.turn.current == owner
                    && !u.cancelled
                    && matches!(ctx.state.turn.cards_used_play_phase, 3 | 4)),
            HostBehavior::XianjianLiguang => {
                matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: false }
                if *player == owner)
                    && !ctx.state.player(owner).hand.is_empty()
                    && ctx.state.alive_players().into_iter().any(|src| {
                        let eq_ok =
                            ctx.state.player(src).equips.iter().any(|c| {
                                let slot = ctx.catalog.card_slot_of(ctx.state, *c);
                                ctx.state.alive_players().into_iter().any(|dst| {
                                    dst != src
                                        && ctx.state.player(dst).equips.iter().all(|o| {
                                            ctx.catalog.card_slot_of(ctx.state, *o) != slot
                                        })
                                })
                            });
                        let jg_ok = ctx.state.player(src).judge.iter().any(|c| {
                            ctx.state.alive_players().into_iter().any(|dst| {
                                dst != src
                                    && ctx.state.player(dst).judge.iter().all(|o| {
                                        ctx.catalog.card_kind(*o) != ctx.catalog.card_kind(*c)
                                    })
                            })
                        });
                        eq_ok || jg_ok
                    })
            }
            HostBehavior::XianjianLinghuo => matches!(event,
                // A2 H83：他人整回合结束（上游 global phaseEnd＋filter 限
                // event.player!==player）＋本回合受伤＋冷却已过
                EventData::TurnEndWindow { player, after: false }
                if *player != owner
                    && ctx.state.player(*player).turn_damage > 0
                    && ctx.state.turn.round >= ctx.state.player(owner).linghuo_ready_round),
            HostBehavior::XianjianLingquan => {
                matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: false }
                if *player == owner)
                    && !ctx.state.player(owner).lingquan_awakened
                    && ctx.state.turn.round >= 3
                    && ctx.state.player(owner).total_used > ctx.state.player(owner).hp as u32
            }
            // 神雾：水云计数≥3+未觉醒（juexingji 一次性）,
            HostBehavior::XianjianLonghuo => {
                matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: false }
                if *player == owner)
                    && ctx.state.player(owner).jiangcheng_longhuo
            }
            HostBehavior::XianjianPoyun => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && ctx.state.player(owner).xuanning_mark > 0
                    && (!ctx.state.player(d.target).hand.is_empty()
                        || !ctx.state.player(d.target).equips.is_empty())),
            // 千方（A2 H49）：上游 player phaseBegin＝自己回合开始（原锚
            // 自己判定阶段开始）
            HostBehavior::XianjianQianfang => {
                matches!(event,
                EventData::TurnBegin { player }
                if *player == owner)
                    && ctx.state.player(owner).xuanning_mark > 0
                    && ((ctx.state.player(owner).hand.len() + ctx.state.player(owner).equips.len())
                        as u32)
                        + ctx.state.player(owner).xuanning_mark
                    >= 3
            }
            HostBehavior::XianjianRunxin => {
                let heart = match event {
                    EventData::CardUsed(u) => {
                        u.player == owner && {
                            u.suit(ctx.catalog) == Some(crate::catalog::Suit::Heart)
                        }
                    }
                    EventData::CardResponded { card, player } => {
                        *player == owner
                            && ctx.catalog.cards[*card as usize].suit == crate::catalog::Suit::Heart
                    }
                    _ => false,
                };
                heart
                    && ctx
                        .state
                        .alive_players()
                        .into_iter()
                        .any(|p| (ctx.state.player(p).hp as u32) < ctx.state.player(p).max_hp)
            }
            HostBehavior::XianjianShenmu => {
                matches!(event, EventData::Dying { .. })
                    && ctx.state.player(owner).hand.iter().any(|c| {
                        matches!(
                            ctx.catalog.cards[*c as usize].suit,
                            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                        )
                    })
            }
            // 毒刺：自己弃牌含装备 kind+距离 1 内有其他存活角色（登记差
            // 异：仅弃置路径；被获得/替换不触发）,
            HostBehavior::XianjianShenwu => {
                // A2 H87：上游 `{global:'phaseEnd'}`＝任意玩家整回合结束
                //（第 939 轮：原臂钉在自己 End 阶段，过窄；觉醒门不变）
                matches!(event,
                EventData::TurnEndWindow { after: false, .. }
                if ctx.state.player(owner).shuiyun_count >= 3)
                    && !ctx.state.player(owner).shenwu_awakened
            }
            HostBehavior::XianjianSheying => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && !ctx.state.turn.sheying_used
                    && d.card.is_some()
                    && ctx.state.discard.contains(d.card.as_ref().unwrap())),
            // 花好：任意其他存活角色（多目标 [1,∞]）+未发动,
            HostBehavior::XianjianShiying => {
                matches!(event,
                EventData::Death { player, .. } if *player != owner)
                    && !ctx.state.player(owner).shiying_used
            }
            HostBehavior::XianjianShuiyun5 => {
                matches!(event, EventData::Dying { .. })
                    && !ctx.state.player(owner).off_court.is_empty()
            }
            // 忘忧（A2 H85）：他人整回合结束＋自己有牌＋本回合存在受伤标记者
            HostBehavior::XianjianWangyou => matches!(event,
                // 上游 `wangyou`: `{global:'phaseEnd'}` + ①自己有牌 ②非自己回合 ③场上有人持 wangyou3
                EventData::TurnEndWindow { player, after: false }
                if *player != owner
                    && (!ctx.state.player(owner).hand.is_empty()
                        || !ctx.state.player(owner).equips.is_empty())
                    && ctx.state
                        .alive_players()
                        .any(|p| ctx.state.player(p).wangyou3_round != 0)),
            HostBehavior::XianjianWangyou2 => matches!(event,
                EventData::Damage(d)
                if !d.prevented && d.amount > 0 && ctx.state.player(d.target).alive),
            HostBehavior::XianjianXjzhimeng => {
                matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::End, begin: false }
                if *player == owner)
                    && !ctx.state.deck.is_empty()
                    && ctx.state.alive_players().into_iter().any(|p| p != owner)
            }
            // 织梦·梦：暗牌持有者被使用同类型（三分类）牌（targets 含
            // 持有者+暗牌保留=上游 storage 不清除同口径）,
            HostBehavior::XianjianXshuangren => matches!(event,
            EventData::CardsDiscarded { player, cards }
            if *player == owner
                && cards.iter().any(|c| {
                    ctx.catalog.card_slot_of(ctx.state, *c)
                        == Some(crate::catalog::EquipSlot::Weapon)
                })),
            // 双刃·殁：自己死亡帧,
            HostBehavior::XianjianXshuangren2 => matches!(event,
                EventData::Death { player, .. } if *player == owner),
            HostBehavior::XianjianXunying => {
                if let EventData::CardUsed(u) = event {
                    u.player == owner
                        && u.effective_kind(ctx.catalog) == crate::catalog::CardKind::Sha
                        && !u.targets.is_empty()
                        && u.targets.iter().all(|t| {
                            ctx.state.player(*t).alive
                                && ctx.state.computed_distance(ctx.catalog, owner, *t)
                                    <= ctx.state.attack_range(ctx.catalog, owner)
                        })
                        && ctx
                            .state
                            .player(owner)
                            .hand
                            .iter()
                            .any(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Sha)
                } else {
                    false
                }
            }
            HostBehavior::XianjianZhangmu => matches!(event,
            EventData::Damage(d)
            if d.target == owner
                && !d.prevented
                && d.card.is_some()
                && ctx.catalog.card_kind(d.card.as_ref().unwrap().clone())
                    == crate::catalog::CardKind::Sha
                && !ctx.state.turn.zhangmu_used
                && ctx.state.player(owner).hand.iter().any(|c| {
                    ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Shan
                })),
            HostBehavior::XianjianShuiyun => {
                let base = matches!(event,
                    EventData::Phase { player, phase: crate::event::Phase::End, begin: false }
                    if *player == owner);
                let dedup = ctx.state.player(owner).hand.iter().any(|c| {
                    let k = yangpijuan_class(ctx.catalog.card_kind(*c));
                    !ctx.state
                        .player(owner)
                        .off_court
                        .iter()
                        .any(|o| yangpijuan_class(ctx.catalog.card_kind(*o)) == k)
                });
                base && ctx.state.player(owner).off_court.len() < 3 && dedup
            }
            HostBehavior::XianjianZhaoyao => matches!(event,
                EventData::Phase { player, phase: crate::event::Phase::Draw, begin: true }
                if *player != owner
                    && !ctx.state.player(owner).hand.is_empty()
                    && !ctx.state.player(*player).hand.is_empty()),
            HostBehavior::XianjianAnwugu => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && d.target != owner
                    && !d.prevented
                    && d.amount > 0
                    && ctx.state.player(d.target).alive
                    && ctx.state.player(d.target).anwugu_count == 0),
            HostBehavior::XianjianBinxin => matches!(event,
                // A2 H80：任意玩家整回合结束（上游 global phaseEnd），
                // 目标＝回合结束者（factory 按 TurnEndWindow 提取）
                EventData::TurnEndWindow { player, after: false }
                if ctx.state.player(*player).alive
                    && ctx.state.player(*player).hp == 1),
            // 回梦：你回复体力（灵雪同款 filter）,
            HostBehavior::XianjianFeixu => {
                matches!(event,
                EventData::CardUsed(u)
                    if u.effective_kind(ctx.catalog) == CardKind::Shan)
                    || matches!(event,
                EventData::CardResponded { card, .. }
                    if ctx.catalog.card_kind(*card) == CardKind::Shan)
            }
            // 风：自己摸牌（有量）,
            HostBehavior::XianjianFeng => matches!(event,
                EventData::CardsDrawn { player, .. }
                if *player == owner),
            // 雅：自己受伤（未防止有量）,
            HostBehavior::XianjianHuimeng => matches!(event,
                EventData::Recover { player, amount, .. }
                if *player == owner && *amount > 0),
            // 蛇蜕：属性伤害对你（有属性=非普通；传导事件同免疫=上游
            // hasNature 同口径）,
            HostBehavior::XianjianLingxue => matches!(event,
                EventData::Recover { player, amount, .. }
                if *player == owner && *amount > 0),
            // 冰心：任意体力为 1 的角色阶段结束（含自己）,
            HostBehavior::XianjianSong => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner) && !d.prevented && d.amount > 0),
            // 御风：失去手牌（弃置路径含手牌）且手牌<2、本回合<2 次,
            HostBehavior::XianjianTianshe => matches!(event,
                EventData::Damage(d)
                if d.target == owner
                    && !d.prevented
                    && d.amount > 0
                    && d.nature != crate::event::DamageNature::Normal),
            // 蛇蜕·息：你造成属性伤害（未防止有量）且已受伤,
            HostBehavior::XianjianTianshe2 => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && d.nature != crate::event::DamageNature::Normal
                    && (ctx.state.player(owner).hp as u32) < ctx.state.player(owner).max_hp),
            // 返灵：他人失去体力（有量）且自己已受伤、本回合未发动,
            HostBehavior::XianjianXjyufeng => matches!(event,
                EventData::CardsDiscarded { player, cards }
                if *player == owner
                    && ctx.state.turn.xjyufeng_used < 2
                    && !cards.is_empty()
                    && ctx.state.player(owner).hand.len() < 2),
            // 玄炎·燃：你的火属性伤害（非传导）,
            HostBehavior::XianjianXuanyan2 => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && d.nature == crate::event::DamageNature::Fire
                    && !d.from_chain),
            // 玄炎·烬：你造成火属性伤害（未防止有量）,
            HostBehavior::XianjianXuanyan3 => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && d.nature == crate::event::DamageNature::Fire),
            // 凝冰：你受到雷属性伤害（未防止有量）
            HostBehavior::XianjianYa => matches!(event,
                EventData::Damage(d)
                if d.target == owner && !d.prevented && d.amount > 0),
            HostBehavior::XianjianLuanjian2 => matches!(event,
                EventData::Damage(d)
                if d.source == Some(owner)
                    && !d.prevented
                    && d.amount > 0
                    && d.card.is_some()
                    && ctx.state.turn.luanjian_card == d.card.map(|c| c as u32)),
            _ => unreachable!("behavior routed to the wrong filter module"),
        }
    }
}
