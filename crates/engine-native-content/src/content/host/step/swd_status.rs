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
                run_swd_anlianying(ctx, self.owner);
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
            // 破茧·齐面：牌堆→弃堆寻首张装备牌装备之（table 暂存 → Equipped 帧，不扰动手牌区）
            HostBehavior::SwdPojian => {
                run_swd_pojian(ctx, self.owner);
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
            // 极谋·姬良面：回合外受伤后摸 1，结算后可选使用一张手牌（character/swd.js:7072）
            HostBehavior::SwdJqimou => match self.stage {
                HostStage::Act => {
                    if !ctx.core.player(self.owner).alive {
                        return Flow::Done;
                    }
                    self.stage = HostStage::DrawSettle;
                    ctx.start_draw(self.owner, 1);
                    Flow::Yield
                }
                HostStage::DrawSettle => {
                    let Some(spec) = ask_swd_jqimou(
                        HostStage::SelectCard,
                        self.owner,
                        None,
                        ctx.core,
                        ctx.catalog,
                    ) else {
                        return Flow::Done;
                    };
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec,
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).hand.contains(&c) {
                        return Flow::Done;
                    }
                    let Some((kind, auto_ready, legal)) =
                        super::swd_card_effects::swd_playable_card_targets(
                            ctx.core,
                            ctx.catalog,
                            self.owner,
                            c,
                            true,
                        )
                    else {
                        return Flow::Done;
                    };
                    if auto_ready {
                        let use_as = (!kind.is_equip()).then_some(kind);
                        ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                            c,
                            self.owner,
                            legal,
                            use_as,
                        )));
                        return Flow::Done;
                    }
                    self.card = Some(c);
                    let Some(spec) = ask_swd_jqimou(
                        HostStage::TargetChoice,
                        self.owner,
                        Some(c),
                        ctx.core,
                        ctx.catalog,
                    ) else {
                        return Flow::Done;
                    };
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec,
                    }
                }
                HostStage::TargetChoice => {
                    let Some(c) = self.card.take() else {
                        return Flow::Done;
                    };
                    let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(self.owner).hand.contains(&c) {
                        return Flow::Done;
                    }
                    let Some((kind, _, legal)) =
                        super::swd_card_effects::swd_playable_card_targets(
                            ctx.core,
                            ctx.catalog,
                            self.owner,
                            c,
                            true,
                        )
                    else {
                        return Flow::Done;
                    };
                    if !legal.contains(&t) {
                        return Flow::Done;
                    }
                    let use_as = (!kind.is_equip()).then_some(kind);
                    ctx.emit(EventData::CardUsed(crate::machine::card_use_as(
                        c,
                        self.owner,
                        vec![t],
                        use_as,
                    )));
                    Flow::Done
                }
                _ => Flow::Done,
            }
            // 幻形·移除面：移除幻形授予的技能实例（在位旗标=内部状态技能
            // 实例；按其他玩家将技能全表尝试移除——登记差异：不追踪具体
            // 授予集）
            // 幻形·移除面：自己受伤结算后清除本次授予技能、swd_huanxing_prev_gid 与在位标记
            HostBehavior::SwdHuanxing2 => {
                super::swd_card_targets::revoke_huanxing_granted(ctx, self.owner, true);
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
            // 流虹：使用杀后摸 1
            HostBehavior::SwdLiuhong => {
                ctx.start_draw(self.owner, 1);
                Flow::Done
            }
            // 道法：来源弃置一张“he”牌（手牌优先，否则装备区）
            HostBehavior::SwdDaofa => {
                if let Some(src) = self.target {
                    if let Some(c) = ctx.core.player(src).hand.first().copied() {
                        ctx.discard_from_hand(src, c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: src,
                            cards: vec![c],
                        });
                    } else if let Some(c) = ctx.core.player(src).equips.first().copied() {
                        super::swd_u05_7d::discard_owned_card_once(ctx, src, c);
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
                if let Some(actor) = self.target.filter(|&a| a != self.owner) {
                    ctx.use_virtual_card(self.owner, CardKind::Sha, vec![actor]);
                    ctx.start_draw(self.owner, 1);
                }
                Flow::Done
            }
            // 幻音：判定——黑桃取消该牌、红桃摸 1、其余无事
            HostBehavior::SwdHuanyin => {
                if !ctx.core.deck.is_empty() || !ctx.core.discard.is_empty() {
                    let judged = ctx.draw_judge_card_with_elo(
                        self.owner,
                        engine_native_kernel::kernel::EloJudgePref::Huanyin,
                    );
                    let (suit, _) = ctx.inline_judge_suit_rank(self.owner, judged);
                    match suit {
                        crate::catalog::Suit::Spade => {
                            ctx.current_card_use_mut().cancelled = true;
                        }
                        crate::catalog::Suit::Heart => {
                            ctx.start_draw(self.owner, 1);
                        }
                        _ => {}
                    }
                    let _ = ctx.settle_inline_judge_card(self.owner, suit, judged);
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
                // 随机获得使用者一张 he 牌（hand + equips 联合均匀随机）
                let h_len = ctx.core.player(attacker).hand.len();
                let e_len = ctx.core.player(attacker).equips.len();
                let total = h_len + e_len;
                let stolen = if total == 0 {
                    None
                } else {
                    let j = ctx.core.rng.gen_range(total as u32) as usize;
                    if j < h_len {
                        Some(ctx.core.player(attacker).hand[j])
                    } else {
                        Some(ctx.core.player(attacker).equips[j - h_len])
                    }
                };
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
            // 陷阱·取回：受伤后随机取回一张陷阱（rng.gen_range）
            HostBehavior::SwdSxianjingDamage => {
                let n = ctx.core.player(self.owner).sxianjing_traps.len();
                if n > 0 {
                    let j = ctx.core.rng.gen_range(n as u32) as usize;
                    let trap = ctx.core.player(self.owner).sxianjing_traps[j];
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
            // 破舟·封印到期（swd/skill.js:3463）：拥有者下一回合结束（TurnPhaseAfter）恢复技能并移除自身
            HostBehavior::SwdPozhouSealed => {
                for ((p, _), inst) in ctx.core.skills.iter_mut() {
                    if *p == self.owner {
                        inst.enabled = true;
                    }
                }
                if let Some(def) = ctx.catalog.skill_by_name(SWD_POZHOU_SEALED_KEY) {
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

pub(in crate::content::host::step) fn run_swd_anlianying(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if ctx.core.player(owner).alive {
        ctx.start_draw(owner, 2);
    }
}

pub(in crate::content::host::step) fn run_swd_pojian(ctx: &mut Ctx<'_>, owner: PlayerId) {
    if !ctx.core.player(owner).alive {
        return;
    }
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
        return;
    };
    if ctx.core.deck.contains(&id) {
        ctx.core.deck.retain(|x| *x != id);
    } else {
        ctx.core.discard.retain(|x| *x != id);
    }
    ctx.core.table.push(id);
    ctx.emit(EventData::Equipped {
        player: owner,
        card: id,
        replaced: None,
    });
}

pub(in crate::content::host::step) fn host_on_hand_card_removed_swd(
    ctx: &mut Ctx<'_>,
    player: PlayerId,
) {
    if !ctx.core.player(player).alive
        || !ctx.core.player(player).hand.is_empty()
        || ctx.catalog.is_tieji_sealed(ctx.core, player)
    {
        return;
    }
    let (do_anlianying, do_pojian) = {
        let has_beh = |beh: HostBehavior| -> bool {
            ctx.core.skills.iter().any(|(&(p, d), i)| {
                p == player
                    && i.enabled
                    && ctx.catalog.skills.get(d as usize).is_some_and(|s| {
                        s.behavior.as_host_behavior() == Some(beh)
                            && !ctx.catalog.is_rechanyuan_sealed_name(ctx.core, player, s.name)
                    })
            })
        };
        (
            has_beh(HostBehavior::SwdAnlianying),
            has_beh(HostBehavior::SwdPojian),
        )
    };
    if do_anlianying {
        run_swd_anlianying(ctx, player);
    }
    if do_pojian {
        run_swd_pojian(ctx, player);
    }
}

pub(in crate::content::host) fn ask_swd_jqimou(
    stage: HostStage,
    owner: PlayerId,
    card: Option<CardId>,
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
) -> Option<ChoiceSpec> {
    if !core.player(owner).alive {
        return None;
    }
    match stage {
        HostStage::SelectCard => {
            let options: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    super::swd_card_effects::swd_playable_card_targets(core, catalog, owner, c, true)
                        .is_some()
                })
                .map(ChoiceOption::Card)
                .collect();
            (!options.is_empty()).then_some(ChoiceSpec {
                kind: ChoiceKind::SelectCards,
                prompt: "swd_jqimou_use",
                options,
                min: 0,
                max: 1,
                allow_cancel: true,
            })
        }
        HostStage::TargetChoice => {
            let c = card?;
            if !core.player(owner).hand.contains(&c) {
                return None;
            }
            let (_, auto_ready, legal) =
                super::swd_card_effects::swd_playable_card_targets(core, catalog, owner, c, true)?;
            if auto_ready || legal.is_empty() {
                return None;
            }
            Some(ChoiceSpec {
                kind: ChoiceKind::SelectPlayers,
                prompt: "swd_jqimou_target",
                options: legal.into_iter().map(ChoiceOption::Player).collect(),
                min: 1,
                max: 1,
                allow_cancel: true,
            })
        }
        _ => None,
    }
}
