use super::super::*;

impl HostMachine {
    pub(super) fn step_swd_status(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 平恤：无效果步（computed_distance 字段消费）
            HostBehavior::SwdPingxu => Flow::Done,
            // 还侠：无效果步（PlayCard 转化臂+kernel 跟踪钩承载）
            HostBehavior::SwdHuanxia => Flow::Done,
            // 破月：无效果步
            HostBehavior::SwdPoyue => Flow::Done,
            // 森罗：activate 门已验手牌最少——对其他有手牌角色各：弃 2
            //（确定性首两张，登记差异上游自选）+摸 1
            HostBehavior::SwdSenluo => {
                let targets: Vec<PlayerId> = ctx
                    .core
                    .alive_players()
                    .filter(|p| *p != self.owner && !ctx.core.player(*p).hand.is_empty())
                    .collect();
                for t in targets {
                    let hand: Vec<CardId> = ctx.core.player(t).hand.iter().copied().collect();
                    let n = hand.len().min(2);
                    for c in hand.into_iter().take(n) {
                        ctx.discard_from_hand(t, c);
                    }
                    if ctx.core.player(t).alive {
                        ctx.start_draw(t, 1);
                    }
                }
                Flow::Done
            }
            // 诱引：frequent 锁定式——直摸 1（无确认窗）
            HostBehavior::SwdYouyin => {
                if ctx.core.player(self.owner).alive {
                    ctx.start_draw(self.owner, 1);
                }
                Flow::Done
            }
            // 连舞：锁定 marker（engine max_targets 链+kernel 红杀 directHit
            // 臂查询），无独立效果步
            HostBehavior::SwdLianwu => Flow::Done,
            // 万钧：锁定 marker（engine PlayCard viewAs 链查询），无效果步
            HostBehavior::SwdWanjun => Flow::Done,
            // 火扑：锁定 marker（engine PlayCard viewAs 链查询），无效果步
            HostBehavior::SwdHuopu => Flow::Done,
            // 夜影：锁定 marker（engine PlayCard viewAs 链查询），无效果步
            HostBehavior::SwdYeying => Flow::Done,
            // 归影：锁定 marker（engine PlayCard viewAs 链查询），无效果步
            HostBehavior::SwdGuiying => Flow::Done,
            // 凤泽：锁定 marker（engine PlayCard viewAs 链查询），无效果步
            HostBehavior::SwdFengze => Flow::Done,
            // 玲月：锁定（sha 机器内联臂查询），无独立效果步
            HostBehavior::SwdLingyue => Flow::Done,
            // 暗莲映：frequent 锁定式——直摸 2
            HostBehavior::SwdAnlianying => {
                if ctx.core.player(self.owner).alive {
                    ctx.start_draw(self.owner, 2);
                }
                Flow::Done
            }
            // 疾风：锁定 marker（engine max_targets 链+state computed_distance
            // 链查询），无独立效果步
            HostBehavior::SwdLjifeng => Flow::Done,
            // 摄引：座位序逐判定区有牌者——弃其判定牌 + 其交一张手牌（循环
            // 面一次机器内完成，确定性座位序登记差异上游 chooseTarget 循环）
            HostBehavior::SwdXianyin => {
                let victims: Vec<PlayerId> = ctx
                    .core
                    .seat_order_from(self.owner)
                    .into_iter()
                    .filter(|p| *p != self.owner && !ctx.core.player(*p).judge.is_empty())
                    .collect();
                for v in victims {
                    let judge_cards: Vec<CardId> =
                        ctx.core.player(v).judge.iter().copied().collect();
                    for c in judge_cards {
                        ctx.core.player_mut(v).judge.retain(|x| *x != c);
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: v,
                            cards: vec![c],
                        });
                    }
                    // 交一张手牌（确定性首张，登记差异上游目标自选）
                    if let Some(c) = ctx.core.player(v).hand.first().copied() {
                        ctx.take_hand_card(v, c);
                        ctx.core.player_mut(self.owner).hand.push(c);
                        ctx.log(LogEvent::CardsGiven {
                            from: v,
                            to: self.owner,
                            cards: vec![c],
                        });
                    }
                }
                Flow::Done
            }
            // 破茧·齐面：牌堆→弃堆寻首张装备牌装备之（Equipped 帧+授技走
            // 正式 PlayCard 装备提交路径）
            HostBehavior::SwdPojian => {
                let found = ctx
                    .core
                    .deck
                    .iter()
                    .copied()
                    .find(|c| ctx.catalog.card_kind(*c).is_equip())
                    .or_else(|| {
                        ctx.core
                            .discard
                            .iter()
                            .copied()
                            .find(|c| ctx.catalog.card_kind(*c).is_equip())
                    });
                let Some(id) = found else {
                    return Flow::Done;
                };
                let in_deck = ctx.core.deck.contains(&id);
                if in_deck {
                    ctx.core.deck.retain(|x| *x != id);
                } else {
                    ctx.core.discard.retain(|x| *x != id);
                }
                // 走正式装备提交：实体暂入手牌 → Equipped 帧 → commit_equip
                // （take_hand_card + 同栏替换 + grant_equip_skills 全托管）
                ctx.core.player_mut(self.owner).hand.push(id);
                ctx.emit(EventData::Equipped {
                    player: self.owner,
                    card: id,
                    replaced: None,
                });
                Flow::Done
            }
            // 破晓·张烈面：锁定 marker（viewAs 链+距离链+目标上限链三查询）
            HostBehavior::SwdPoxiao => Flow::Done,
            // 疾风·司空宇面：锁定 marker（engine 杀距离校验臂查询），无效果步
            HostBehavior::SwdSliufeng => Flow::Done,
            // 英姿·刷新面：自己摸牌阶段额外摸 1（手牌上限查询在
            // state.hand_limit；此处只补摸牌面）
            HostBehavior::SwdReyingzi => {
                ctx.start_draw(self.owner, 1);
                Flow::Done
            }
            // 乐学·久悠面：回合开始授技——自己回合授{制衡/集智}之一，
            // 他人回合授{遗计/急救/鬼道/反馈}之一（RNG 选择；
            // expires_end_of_turn 到期移除。登记差异：上游池含缔盟/驱虎，
            // 原生未实现故回合内池取已实现子集）
            HostBehavior::SwdLexue => {
                let in_turn = ctx.core.turn.current == self.owner;
                let pool: &[&str] = if in_turn {
                    &["制衡", "集智"]
                } else {
                    &["遗计", "急救", "鬼道", "反馈"]
                };
                let defs: Vec<crate::ids::SkillDefId> = pool
                    .iter()
                    .filter_map(|n| ctx.catalog.skill_by_name(n))
                    .collect();
                if defs.is_empty() {
                    return Flow::Done;
                }
                let j = ctx.core.rng.gen_range(defs.len() as u32) as usize;
                let chosen = defs[j];
                ctx.add_skill_until_turn_end(self.owner, chosen);
                Flow::Done
            }
            // 圣手·淳于恒面：锁定 marker（engine viewAs 链查询），无效果步
            HostBehavior::SwdShengshou => Flow::Done,
            // 拷贝栏：锁定（Draw 内联钩），无独立效果步
            HostBehavior::SwdKbolan => Flow::Done,
            // 明镜：1/3 伤害 -1（等同光纱天衣；无防具门在 filter）
            HostBehavior::SwdMinjing => {
                if ctx.core.rng.gen_range(3u32) == 0 {
                    let d = ctx.current_damage_mut();
                    d.amount = d.amount.saturating_sub(1);
                }
                Flow::Done
            }
            // 倾城：frequent——判定循环至多 3 次：红判定牌入手，非红弃置
            // 终止（确定性顺序判定，内联口径同铁骑）
            HostBehavior::SwdLqingcheng => {
                if !ctx.core.player(self.owner).alive {
                    return Flow::Done;
                }
                for _ in 0..3 {
                    if ctx.core.deck.is_empty() && ctx.core.discard.is_empty() {
                        break;
                    }
                    if ctx.core.deck.is_empty() {
                        let mut cards = std::mem::take(&mut ctx.core.discard);
                        ctx.core.rng.shuffle(&mut cards);
                        ctx.core.deck = cards;
                        ctx.log(LogEvent::DeckReshuffled);
                    }
                    let c = ctx
                        .core
                        .deck
                        .pop()
                        .expect("lqingcheng: deck or discard non-empty");
                    let red = matches!(
                        ctx.catalog.cards[c as usize].suit,
                        crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                    );
                    if red {
                        ctx.core.player_mut(self.owner).hand.push(c);
                        ctx.log(LogEvent::CardsDrawn {
                            player: self.owner,
                            cards: vec![c],
                        });
                    } else {
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![c],
                        });
                        break;
                    }
                }
                Flow::Done
            }
            // 狩猎：Sha 机器内联臂承载，无独立效果步
            HostBehavior::SwdShoulie => Flow::Done,
            // 妖童：锁定 marker（engine viewAs 链查询），无效果步
            HostBehavior::SwdYaotong => Flow::Done,
            // 极谋·姬良面：回合外受伤后摸 1（上游 jqimou draw 同口径；
            // 使用牌续段=主动选择面登记差异不收）
            HostBehavior::SwdJqimou => {
                ctx.start_draw(self.owner, 1);
                Flow::Done
            }
            // 幻形·移除面：移除幻形授予的技能实例（在位旗标=内部状态技能
            // 实例；按其他玩家将技能全表尝试移除——登记差异：不追踪具体
            // 授予集）
            HostBehavior::SwdHuanxing2 => {
                // 移除幻形授予的技能实例：按 all 玩家（除自己）将技能名
                // 全表扫描，跳过自己本将技能（登记差异：不追踪具体授予
                // 集，同名他将技能亦会被移除的边界已收窄为本将排除）
                let own: Vec<&'static str> = {
                    let meta = ctx.catalog.general_meta(
                        ctx.core.player(self.owner).general_slots[0].unwrap_or(u16::MAX),
                    );
                    meta.map(|(_, _, skills, _, _)| skills).unwrap_or_default()
                };
                let mut candidates: Vec<&'static str> = Vec::new();
                for i in 0..ctx.core.players.len() {
                    let pid = i as PlayerId;
                    if pid == self.owner {
                        continue;
                    }
                    let meta = ctx
                        .catalog
                        .general_meta(ctx.core.player(pid).general_slots[0].unwrap_or(u16::MAX));
                    if let Some((_, _, skills, _, _)) = meta {
                        for s in skills {
                            if !own.contains(&s) && !candidates.contains(&s) {
                                candidates.push(s);
                            }
                        }
                    }
                }
                for n in candidates {
                    if let Some(def) = ctx.catalog.skill_by_name(n) {
                        ctx.core.skills.remove(&(self.owner, def));
                    }
                }
                if let Some(sdef) = ctx.catalog.skill_by_name(SWD_HUANXING_STATE_KEY) {
                    ctx.core.skills.remove(&(self.owner, sdef));
                }
                Flow::Done
            }
            // 激昂·凤天凌面：摸 1（frequent 非 optional，无效果步以外的
            // 状态变化）
            HostBehavior::SwdJiang => {
                ctx.start_draw(self.owner, 1);
                Flow::Done
            }
            // 凌波·恒爱面：frequent——闪响应/使用后摸 2
            HostBehavior::SwdHlingbo => {
                if ctx.core.player(self.owner).alive {
                    ctx.start_draw(self.owner, 2);
                }
                Flow::Done
            }
            // 灵心：frequent——牌堆顶三张（pop×3）：红桃入手，其余弃置
            HostBehavior::SwdLingxin => {
                if !ctx.core.player(self.owner).alive {
                    return Flow::Done;
                }
                for _ in 0..3 {
                    if ctx.core.deck.is_empty() && ctx.core.discard.is_empty() {
                        break;
                    }
                    if ctx.core.deck.is_empty() {
                        let mut cards = std::mem::take(&mut ctx.core.discard);
                        ctx.core.rng.shuffle(&mut cards);
                        ctx.core.deck = cards;
                        ctx.log(LogEvent::DeckReshuffled);
                    }
                    let c = ctx
                        .core
                        .deck
                        .pop()
                        .expect("lingxin: deck or discard non-empty");
                    if matches!(
                        ctx.catalog.cards[c as usize].suit,
                        crate::catalog::Suit::Heart
                    ) {
                        ctx.core.player_mut(self.owner).hand.push(c);
                        ctx.log(LogEvent::CardsDrawn {
                            player: self.owner,
                            cards: vec![c],
                        });
                    } else {
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![c],
                        });
                    }
                }
                Flow::Done
            }
            // 渔人：锁定（sha 机器内联臂查询），无独立效果步
            HostBehavior::SwdYueren => Flow::Done,
            // 无邪：锁定 marker（engine 延时校验臂查询），无独立效果步
            HostBehavior::SwdWuxie => Flow::Done,
            // 幻剑：锁定 marker（engine PlayCard viewAs 链查询），无效果步
            HostBehavior::SwdHuanjian => Flow::Done,
            // 乐跃：TargetChoice-free 分流——回复者非自己且其手牌≥自己则
            // 各摸 1，否则回复者摸 1
            HostBehavior::SwdYinyue => {
                let Some(r) = ctx
                    .res
                    .frame(ctx.current_event_frame_id())
                    .map(|f| &f.data)
                    .and_then(|data| match data {
                        crate::event::EventData::Recover { player, .. } => Some(*player),
                        _ => None,
                    })
                else {
                    return Flow::Done;
                };
                if r != self.owner
                    && ctx.core.player(r).hand.len() >= ctx.core.player(self.owner).hand.len()
                {
                    ctx.start_draw(r, 1);
                    if ctx.core.player(self.owner).alive {
                        ctx.start_draw(self.owner, 1);
                    }
                } else if ctx.core.player(r).alive {
                    ctx.start_draw(r, 1);
                }
                Flow::Done
            }
            // 铁壁：弃首张黑牌 → 目标 +1 护甲（登记差异：上游玩家选弃牌）
            HostBehavior::SwdTiebi => {
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                let black = ctx.core.player(self.owner).hand.iter().copied().find(|c| {
                    matches!(
                        ctx.catalog.cards[*c as usize].suit,
                        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                    )
                });
                if let Some(c) = black {
                    ctx.discard_from_hand(self.owner, c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                    ctx.change_hujia(t, 1);
                }
                Flow::Done
            }
            // 流虹：使用杀后摸 1
            HostBehavior::SwdLiuhong => {
                ctx.start_draw(self.owner, 1);
                Flow::Done
            }
            // 道法：来源弃首张手牌（登记差异：上游玩家自选）
            HostBehavior::SwdDaofa => {
                if let Some(src) = self.target {
                    if let Some(c) = ctx.core.player(src).hand.first().copied() {
                        ctx.discard_from_hand(src, c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: src,
                            cards: vec![c],
                        });
                    }
                }
                Flow::Done
            }
            // 屠阵：弃置受伤者全部非基本手牌（确定性整组）
            HostBehavior::SwdTuzhen => {
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                let drop: Vec<CardId> = ctx
                    .core
                    .player(t)
                    .hand
                    .iter()
                    .filter(|c| !ctx.catalog.card_kind(**c).is_basic_kind())
                    .copied()
                    .collect();
                for c in drop {
                    ctx.discard_from_hand(t, c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: t,
                        cards: vec![c],
                    });
                }
                Flow::Done
            }
            // 落梅：使用或打出梅花牌摸 1
            HostBehavior::SwdLuomei => {
                ctx.start_draw(self.owner, 1);
                Flow::Done
            }
            // 雪煌：limited 置位 → 弃全部手牌 → 逐张视为对（至多）两名
            // 其他存活角色使用火杀（座位序确定性；登记差异上游随机 2 名
            // 敌人；目标中途死亡即终止该张）
            HostBehavior::SwdXuehuang => {
                ctx.core.player_mut(self.owner).xuehuang_used = true;
                let cards: Vec<CardId> = ctx.core.player(self.owner).hand.iter().copied().collect();
                let num = cards.len();
                for c in cards {
                    ctx.discard_from_hand(self.owner, c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![c],
                    });
                }
                for _ in 0..num {
                    let targets: Vec<PlayerId> = (0..ctx.core.players.len())
                        .filter(|&i| {
                            i as PlayerId != self.owner && ctx.core.player(i as PlayerId).alive
                        })
                        .map(|i| i as PlayerId)
                        .take(2)
                        .collect();
                    if targets.is_empty() {
                        break;
                    }
                    ctx.use_virtual_card(self.owner, crate::catalog::CardKind::FireSha, targets);
                }
                Flow::Done
            }
            // 大同：全场极差 ≤1 → 摸 2
            HostBehavior::SwdDatong => {
                ctx.start_draw(self.owner, 2);
                Flow::Done
            }
            // 承剑：来源摸 1
            HostBehavior::SwdChengjian => {
                if let Some(src) = self.target {
                    if ctx.core.player(src).alive {
                        ctx.start_draw(src, 1);
                    }
                }
                Flow::Done
            }
            // 护盾：自己无护甲 → +1
            HostBehavior::SwdHudun => {
                if ctx.core.player(self.owner).hujia == 0 {
                    ctx.change_hujia(self.owner, 1);
                }
                Flow::Done
            }
            HostBehavior::SwdGTiandao => Flow::Done,
            // 热血：对阶段玩家视为使用一张杀，然后摸 1
            // （登记差异：虚拟杀不消耗实体杀；恒发动近似）
            HostBehavior::SwdRexue => {
                if let Some(actor) = self.target {
                    ctx.use_virtual_card(self.owner, CardKind::Sha, vec![actor]);
                    ctx.start_draw(self.owner, 1);
                }
                Flow::Done
            }
            // 幻音：判定——黑桃取消该牌、红桃摸 1、其余无事
            HostBehavior::SwdHuanyin => {
                let judged = ctx.draw_judge_card();
                let suit = ctx.catalog.cards[judged as usize].suit;
                ctx.to_discard(judged);
                match suit {
                    crate::catalog::Suit::Spade => {
                        ctx.current_card_use_mut().cancelled = true;
                    }
                    crate::catalog::Suit::Heart => {
                        ctx.start_draw(self.owner, 1);
                    }
                    _ => {}
                }
                Flow::Done
            }
            // 灵弦：与座位序首个攻击距离 >1 的其他存活者各摸 1
            // （登记差异：自选目标→首个确定性近似；可放弃→恒发动近似）
            HostBehavior::SwdLingxian => {
                let n = ctx.core.players.len() as u8;
                if let Some(t) = (0..n).find(|t| {
                    *t != self.owner
                        && ctx.core.player(*t).alive
                        && ctx.core.computed_distance(ctx.catalog, self.owner, *t) > 1
                }) {
                    ctx.start_draw(self.owner, 1);
                    ctx.start_draw(t, 1);
                }
                Flow::Done
            }
            // 魂身（deferred 消解批 胡月）：使用/打出闪后计数 +1
            HostBehavior::SwdHyunshen => {
                ctx.core.player_mut(self.owner).hyunshen_count =
                    ctx.core.player(self.owner).hyunshen_count.saturating_add(1);
                Flow::Done
            }
            // 魂身·收束：准备阶段摸等量并清零
            HostBehavior::SwdHyunshen2 => {
                let n = ctx.core.player(self.owner).hyunshen_count;
                if n > 0 {
                    ctx.start_draw(self.owner, n as u32);
                    ctx.core.player_mut(self.owner).hyunshen_count = 0;
                }
                Flow::Done
            }
            // 待行·清甲：准备阶段按剩余计数移除护甲
            HostBehavior::SwdDaixing2 => {
                let n = ctx.core.player(self.owner).daixing_count;
                if n > 0 {
                    ctx.change_hujia(self.owner, -(n as i32));
                    ctx.core.player_mut(self.owner).daixing_count = 0;
                }
                Flow::Done
            }
            // 待行·吸收：受伤后递减计数（近似：min(计数, 伤害)）
            HostBehavior::SwdDaixing3 => {
                let dmg = ctx.current_damage_mut().amount as u8;
                let cur = ctx.core.player(self.owner).daixing_count;
                ctx.core.player_mut(self.owner).daixing_count = cur.saturating_sub(cur.min(dmg));
                Flow::Done
            }
            // 聚悉·计数：非当前回合角色弃牌后计数 +1
            HostBehavior::SwdJuxiCount => {
                ctx.core.player_mut(self.owner).juxi_count =
                    ctx.core.player(self.owner).juxi_count.saturating_add(1);
                Flow::Done
            }
            // 破舟：受伤后计数累加伤害值
            HostBehavior::SwdPozhou => {
                let dmg = ctx.current_damage_mut().amount as u8;
                ctx.core.player_mut(self.owner).pozhou_count =
                    ctx.core.player(self.owner).pozhou_count.saturating_add(dmg);
                Flow::Done
            }
            // 陷阱·触发：同花色陷阱弃置 + 随机获得使用者一张牌（首个 he）
            HostBehavior::SwdSxianjingGain => {
                let Some(attacker) = self.target else {
                    return Flow::Done;
                };
                let Some(card) = self.card else {
                    return Flow::Done;
                };
                let suit = ctx.catalog.cards[card as usize].suit;
                let trap = ctx
                    .core
                    .player(self.owner)
                    .sxianjing_traps
                    .iter()
                    .copied()
                    .find(|c| ctx.catalog.cards[*c as usize].suit == suit);
                if let Some(trap) = trap {
                    ctx.core
                        .player_mut(self.owner)
                        .sxianjing_traps
                        .retain(|x| *x != trap);
                    ctx.to_discard(trap);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![trap],
                    });
                }
                // 随机获得使用者一张 he 牌（确定性：首个 he）
                let stolen = ctx
                    .core
                    .player(attacker)
                    .hand
                    .first()
                    .copied()
                    .or_else(|| ctx.core.player(attacker).equips.first().copied());
                if let Some(c) = stolen {
                    ctx.take_owned_card(attacker, c);
                    ctx.core.player_mut(self.owner).hand.push(c);
                    ctx.log(LogEvent::CardsGiven {
                        from: attacker,
                        to: self.owner,
                        cards: vec![c],
                    });
                }
                Flow::Done
            }
            // 陷阱·取回：受伤后随机取回一张陷阱（确定性：首个）
            HostBehavior::SwdSxianjingDamage => {
                if let Some(trap) = ctx.core.player(self.owner).sxianjing_traps.first().copied() {
                    ctx.core
                        .player_mut(self.owner)
                        .sxianjing_traps
                        .retain(|x| *x != trap);
                    ctx.core.player_mut(self.owner).hand.push(trap);
                }
                Flow::Done
            }
            // 孤行·计数：孤行杀造成伤害 +1
            HostBehavior::SwdGuxing3 => {
                ctx.core.turn.guxing_count = ctx.core.turn.guxing_count.saturating_add(1);
                Flow::Done
            }
            // 灵矢：锁定旗标行为（无机器步；距离/杀限次/直中/摸牌内联）
            HostBehavior::SwdLingshi => Flow::Done,
            // 义山：取回最近失去且仍在他处的至多 2 张（新在前）
            HostBehavior::SwdYishan => {
                let list = ctx.core.player(self.owner).yishan_lost.clone();
                let mut taken = 0usize;
                for c in list {
                    if taken >= 2 {
                        break;
                    }
                    if !yishan_recoverable(ctx, self.owner, c) {
                        continue;
                    }
                    if !yishan_take(ctx, self.owner, c) {
                        continue;
                    }
                    taken += 1;
                }
                if taken > 0 {
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "义山",
                        owner: self.owner, presentation: None });
                }
                Flow::Done
            }
            HostBehavior::SwdLingwu => Flow::Done,
            // 锦鳞·退：阶段开始 -1 护甲与计数；护甲归零或计数归零移除
            HostBehavior::SwdJinlin2 => {
                if ctx.core.player(self.owner).hujia > 0 {
                    ctx.change_hujia(self.owner, -1);
                }
                let def = ctx.catalog.skill_by_name(SWD_JINLIN2_KEY);
                let hujia = ctx.core.player(self.owner).hujia;
                let remove = match def {
                    Some(def) => match ctx.core.skills.get_mut(&(self.owner, def)) {
                        Some(inst) => {
                            inst.rounds = inst.rounds.saturating_sub(1);
                            hujia == 0 || inst.rounds == 0
                        }
                        None => false,
                    },
                    None => false,
                };
                if remove {
                    if let Some(def) = def {
                        ctx.core.skills.remove(&(self.owner, def));
                    }
                }
                Flow::Done
            }
            HostBehavior::SwdGouhunluoSub => {
                // 勾魂锣·倒计时（规则 v191）：来源回合开始-1；归 0=
                // 失去 1 体力+弃全部手牌（真实弃置事件）+清挂附+清实例。
                let Some(src) = ctx.core.player(self.owner).gouhunluo_source else {
                    return Flow::Done;
                };
                if !ctx.core.player(src).alive {
                    return Flow::Done;
                }
                let rounds = ctx.core.player(self.owner).gouhunluo_rounds;
                if rounds > 1 {
                    ctx.core.player_mut(self.owner).gouhunluo_rounds = rounds - 1;
                    return Flow::Done;
                }
                // 归 0：失去 1 体力
                ctx.emit(EventData::LoseHp {
                    player: self.owner,
                    amount: 1,
                });
                // 弃全部手牌（真实弃置事件=械磐等可触发）
                let hand: Vec<CardId> = ctx.core.player(self.owner).hand.iter().copied().collect();
                if !hand.is_empty() {
                    for c in &hand {
                        ctx.take_hand_card(self.owner, *c);
                        ctx.to_discard(*c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: hand,
                    });
                }
                // 清挂附+清实例
                let pl = ctx.core.player_mut(self.owner);
                pl.gouhunluo_rounds = 0;
                pl.gouhunluo_source = None;
                if let Some(def) = ctx.catalog.skill_by_name(SWD_GOUHUNLUO_SUB_KEY) {
                    ctx.core.skills.remove(&(self.owner, def));
                }
                Flow::Done
            }
            // 机关凤封印状态（规则 v195）：拥有者任一阶段开始→全部技能
            // 实例重新启用并移除自身（上游 addTempSkill {player:
            // 'phaseBegin'} 移除同口径）
            HostBehavior::SwdFengyinState => {
                for ((p, _), inst) in ctx.core.skills.iter_mut() {
                    if *p == self.owner {
                        inst.enabled = true;
                    }
                }
                if let Some(def) = ctx.catalog.skill_by_name(SWD_FENGYIN_STATE_KEY) {
                    ctx.core.skills.remove(&(self.owner, def));
                }
                Flow::Done
            }
            // 机关怨待获得（规则 v196）：拥有者结束阶段——离场区待获得牌
            // 移入手牌并移除自身（上游 jiguanyuan 技 phaseEnd forced 同口径）
            HostBehavior::SwdJiguanyuanGain => {
                let pending = ctx.core.player(self.owner).jiguanyuan_cards.clone();
                if !pending.is_empty() {
                    let off = &mut ctx.core.player_mut(self.owner).off_court;
                    let mut gained = Vec::new();
                    for c in &pending {
                        if let Some(pos) = off.iter().position(|x| x == c) {
                            gained.push(off.remove(pos));
                        }
                    }
                    ctx.core.player_mut(self.owner).hand.extend(gained);
                    ctx.log(LogEvent::CardsDrawn {
                        player: self.owner,
                        cards: pending.clone(),
                    });
                    ctx.core.player_mut(self.owner).jiguanyuan_cards.clear();
                }
                if let Some(def) = ctx.catalog.skill_by_name(SWD_JIGUANYUAN_GAIN_KEY) {
                    ctx.core.skills.remove(&(self.owner, def));
                }
                Flow::Done
            }
            // 机关妖术补偿（规则 v198）：非自己回合弃牌含装备 Kind 牌（近
            // 似失去装备区牌，登记差异：替换/获得离去不触发）→ 随机补获
            // 等量灵剑（牌堆+弃堆发现；无则不补，登记差异：上游全定义池
            // 凭空造）。状态不移除（持久防重面不变）。
            HostBehavior::SwdJiguanyaoGain => {
                // 触发帧即弃牌事件帧（栈顶）：读取其 CardsDiscarded 载荷
                let lost: Vec<CardId> =
                    match ctx.res.frame(ctx.current_event_frame_id()).map(|f| &f.data) {
                        Some(EventData::CardsDiscarded { cards, .. }) => cards.clone(),
                        _ => Vec::new(),
                    };
                let num = lost
                    .iter()
                    .filter(|c| matches!(ctx.catalog.card_slot_of(ctx.core, **c), Some(_)))
                    .count();
                if num > 0 {
                    let mut pool: Vec<CardId> = ctx
                        .core
                        .deck
                        .iter()
                        .chain(ctx.core.discard.iter())
                        .copied()
                        .filter(|x| ctx.catalog.card_kind(*x) == CardKind::Hslingjian)
                        .collect();
                    let n = pool.len();
                    for i in (1..n).rev() {
                        let j = ctx.core.rng.gen_range(i as u32 + 1) as usize;
                        pool.swap(i, j);
                    }
                    let take = num.min(pool.len());
                    let mut got = Vec::new();
                    for x in pool.into_iter().take(take) {
                        if let Some(pos) = ctx.core.deck.iter().position(|y| *y == x) {
                            ctx.core.deck.remove(pos);
                        } else if let Some(pos) = ctx.core.discard.iter().position(|y| *y == x) {
                            ctx.core.discard.remove(pos);
                        }
                        ctx.core.player_mut(self.owner).hand.push(x);
                        got.push(x);
                    }
                    if !got.is_empty() {
                        ctx.log(LogEvent::CardsDrawn {
                            player: self.owner,
                            cards: got,
                        });
                    }
                }
                Flow::Done
            }
            // 机关妖术防重标记：纯状态无触发面（补偿触发暂缓）
            HostBehavior::SwdJiguanyaoState => Flow::Done,
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
