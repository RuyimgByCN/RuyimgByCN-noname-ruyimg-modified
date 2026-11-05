use super::super::*;

impl HostMachine {
    pub(super) fn step_xianjian_status(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            HostBehavior::XianjianLingxue => match self.stage {
                // 灵雪（U05 xianjian 星璇，规则 v142）：+1 护甲（change_
                // hujia 原语；上游 changeHujia() 同口径）。
                HostStage::Act => {
                    ctx.change_hujia(self.owner, 1);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianBinxin => match self.stage {
                // 冰心（U05 xianjian 沈欺霜，规则 v143）：体力 1 的阶段
                // 结束者 +1 护甲（change_hujia 原语；上游 changeHujia()
                // 同口径；事件者带入 target）。
                HostStage::Act => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).alive && ctx.core.player(t).hp == 1 {
                        ctx.change_hujia(t, 1);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianHuimeng => match self.stage {
                // 回梦（U05 xianjian 赵灵儿，规则 v144）：摸两张牌。
                HostStage::Act => {
                    ctx.start_draw(self.owner, 2);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianTianshe => match self.stage {
                // 蛇蜕（U05 xianjian 赵灵儿，规则 v144）：属性伤害无效
                //（prevented 直写=碎盾/急冻先例）。
                HostStage::Act => {
                    ctx.current_damage_mut().prevented = true;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianTannang => {
                // 探囊（xianjian/韩菱纱，规则 v148）：每回合限一次，梅花手牌当
                // 顺手牵羊使用；此顺手牵羊无距离限制。上游 viewAs 与
                // mod.targetInRange；效果在出牌转化（engine.rs 2165 行）与距离
                // 豁免（engine.rs 2623 行），usable 走命令层 once_used，宿主无
                // 机器步骤。
                Flow::Done
            }
            HostBehavior::XianjianFeixu => match self.stage {
                // 废墟（U05 xianjian 王蓬絮，规则 v149）：使用者/打出者
                // 摸一张牌（事件者带入 target）。
                HostStage::Act => {
                    if let Some(t) = self.target {
                        if ctx.core.player(t).alive {
                            ctx.start_draw(t, 1);
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianFeng => match self.stage {
                // 风（U05 xianjian 雷元戈，规则 v150）：计数累计至 2 后
                // 本次额外摸 1 并清零（上游 drawBegin num++ 的 after 补
                // 摸等价，登记差异）。
                HostStage::Act => {
                    let c = ctx.core.player(self.owner).feng_count;
                    if c < 2 {
                        ctx.core.player_mut(self.owner).feng_count = c + 1;
                    } else {
                        ctx.core.player_mut(self.owner).feng_count = 0;
                        ctx.start_draw(self.owner, 1);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianYa => match self.stage {
                // 雅（U05 xianjian 雷元戈，规则 v150）：计数累计至 2 后
                // 本次受伤 -1 并清零（前置窗口直写帧量=猛击同机制）。
                HostStage::Act => {
                    let c = ctx.core.player(self.owner).ya_count;
                    if c < 2 {
                        ctx.core.player_mut(self.owner).ya_count = c + 1;
                    } else {
                        ctx.core.player_mut(self.owner).ya_count = 0;
                        let d = ctx.current_damage_mut();
                        d.amount = d.amount.saturating_sub(1);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianSong => match self.stage {
                // 颂（U05 xianjian 雷元戈，规则 v150）：计数累计至 2 后
                // 本次伤害 +1 并清零（前置窗口直写帧量）。
                HostStage::Act => {
                    let c = ctx.core.player(self.owner).song_count;
                    if c < 2 {
                        ctx.core.player_mut(self.owner).song_count = c + 1;
                    } else {
                        ctx.core.player_mut(self.owner).song_count = 0;
                        if !ctx.modify_damage_amount(1) {
                            return Flow::Done;
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianXjyufeng => match self.stage {
                // 御风（U05 xianjian 李逍遥，规则 v151）：摸（2-手牌数）
                // 张+置本回合已发动次数。
                HostStage::Act => {
                    let used = ctx.core.turn.xjyufeng_used;
                    ctx.core.turn.xjyufeng_used = used + 1;
                    let hand = ctx.core.player(self.owner).hand.len();
                    if hand < 2 {
                        ctx.start_draw(self.owner, (2 - hand) as u32);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianXuanyan2 => match self.stage {
                // 玄炎·燃（U05 xianjian 玄霄，规则 v152）：火伤 +1（前置
                // 窗口直写帧量=猛击同机制）。
                HostStage::Act => {
                    if !ctx.modify_damage_amount(1) {
                        return Flow::Done;
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianXuanyan3 => match self.stage {
                // 玄炎·烬（U05 xianjian 玄霄，规则 v152）：失去 1 点体力
                //（LoseHp 帧独立事件=返灵同位）。
                HostStage::Act => {
                    let p = ctx.core.player(self.owner);
                    if p.alive {
                        let hp = p.hp;
                        let _ = hp;
                        let amount = 1u32;
                        // LoseHp 事件（经 commit_lose_hp；可触发返灵等）
                        ctx.emit(EventData::LoseHp {
                            player: self.owner,
                            amount,
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianXuanyan => {
                // 玄炎（xianjian/玄霄，规则 v152）：锁定——你造成火属性伤害时
                // +1；受到雷属性伤害后回复 1。上游仅 group:[xuanyan2,
                // xuanyan3]；子技效果在 XianjianXuanyan2/3（同文件 10930、
                // 10939 行），本壳分支无机器步骤。
                Flow::Done
            }
            HostBehavior::XianjianAnwugu => match self.stage {
                // 蛊舞（U05 xianjian 阿奴，规则 v154）：受害者获蛊 3 阶段
                //（上游 addSkill anwugu2+init storage=3 同口径）。
                HostStage::Act => {
                    if let Some(t) = self.target {
                        if ctx.core.player(t).alive {
                            ctx.core.player_mut(t).anwugu_count = 3;
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianAnwugu2 => match self.stage {
                // 蛊舞·缚（无机器——三钩直读：PlayCard 出牌上限/弃牌阶段
                // limit/自己阶段结束递减）。
                HostStage::Act => Flow::Done,
                _ => Flow::Done,
            },
            HostBehavior::XianjianFenshi => match self.stage {
                // 焚逝（U05 xianjian 姜城，规则 v157）：觉醒——+2 护甲、
                // 摸两张、获得龙火（上游 changeHujia(2)/draw(2)/
                // addSkill("longhuo") 同口径；置位兼一次性门控）。
                HostStage::Act => {
                    ctx.core.player_mut(self.owner).jiangcheng_longhuo = true;
                    ctx.change_hujia(self.owner, 2);
                    ctx.start_draw(self.owner, 2);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::XianjianJtjubao => {
                // 聚宝（U05 xianjian 景天，规则 v158）：随机回收一张非基
                // 本弃牌（候选=filter 已验；取牌此刻离弃堆，羊皮卷同款）
                // +置本回合已回收（上游 jtjubao2 临时技等价）。
                ctx.core.turn.jtjubao_used = true;
                let cands: Vec<CardId> = self
                    .cards
                    .iter()
                    .copied()
                    .filter(|c| {
                        yangpijuan_class(ctx.catalog.card_kind(*c)) != Some(0)
                            && ctx.core.discard.contains(c)
                    })
                    .collect();
                if !cands.is_empty() {
                    let i = ctx.core.rng.gen_range(cands.len() as u32) as usize;
                    let picked = cands[i];
                    let Some(pos) = ctx.core.discard.iter().position(|c| *c == picked) else {
                        return Flow::Done;
                    };
                    ctx.core.discard.remove(pos);
                    ctx.core.player_mut(self.owner).hand.push(picked);
                    ctx.log(LogEvent::CardsDrawn {
                        player: self.owner,
                        cards: vec![picked],
                    });
                }
                Flow::Done
            }
            HostBehavior::XianjianWangyou2 => {
                // 忘忧·标（U05 xianjian 紫萱，规则 v162）：受伤者获本回
                // 合戳（wangyou3_round=当前轮；轮次推进自然过期）。
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        ctx.core.player_mut(t).wangyou3_round = ctx.core.turn.round;
                    }
                }
                Flow::Done
            }
            HostBehavior::XianjianXjzhimeng2 => {
                // 织梦·梦（U05 xianjian 柳梦璃，规则 v167）：被使用同类型
                // 牌时摸 1（frequent silent 自动；暗牌保留=上游 storage
                // 不清除同口径）。
                ctx.start_draw(self.owner, 1);
                Flow::Done
            }
            HostBehavior::XianjianSheling => {
                // 摄灵（U05 xianjian 南宫煌，规则 v168）：回收事件牌中在
                // 弃堆者（羊皮卷同款离堆入手）+回合次数 +1（每回合 3 次
                // =上游 usable:3 同口径）。
                ctx.core.turn.sheling_used += 1;
                for c in self.cards.clone() {
                    if let Some(pos) = ctx.core.discard.iter().position(|x| *x == c) {
                        ctx.core.discard.remove(pos);
                        ctx.core.player_mut(self.owner).hand.push(c);
                        ctx.log(LogEvent::CardsDrawn {
                            player: self.owner,
                            cards: vec![c],
                        });
                    }
                }
                Flow::Done
            }
            HostBehavior::XianjianChangnian2 => {
                // 长念·思（U05 xianjian 柳梦璃，规则 v170）：结束阶段摸
                // 1（forced；runtime 授技实例持有者）。
                ctx.start_draw(self.owner, 1);
                Flow::Done
            }
            HostBehavior::XianjianShiying => {
                // 适盈（U05 xianjian 月琦，规则 v171）：死亡取消型首例——
                // 写拦截标记（DeathMachine 直读跳过死亡执行）+双方体力
                // 上限/体力置 3+clearSkills 双方+弃 hej 全部+各摸 3。
                ctx.core.player_mut(self.owner).shiying_used = true;
                let Some(dying) = self.target else {
                    return Flow::Done;
                };
                ctx.core.player_mut(dying).death_cancelled = true;
                for p in [self.owner, dying] {
                    if !ctx.core.player(p).alive && p != dying {
                        continue;
                    }
                    let pl = ctx.core.player_mut(p);
                    pl.max_hp = 3;
                    pl.hp = 3;
                    pl.dying = false;
                }
                // clearSkills 双方（上游 clearSkills 同口径；shiying 实例
                // 自身也被清=上游同款）
                for owner in [self.owner, dying] {
                    let removed: Vec<crate::ids::SkillDefId> = ctx
                        .core
                        .skills
                        .keys()
                        .filter(|(o, _)| *o == owner)
                        .map(|(_, d)| *d)
                        .collect();
                    for def in removed {
                        ctx.remove_skill(owner, def);
                        ctx.log(LogEvent::SkillToggled {
                            player: owner,
                            skill: ctx.catalog.skill_name(def),
                            enabled: false,
                        });
                    }
                    // 弃 hej 全部
                    let hand: Vec<CardId> = ctx.core.player(owner).hand.clone();
                    for c in hand {
                        ctx.take_hand_card(owner, c);
                        ctx.to_discard(c);
                    }
                    let equips: Vec<CardId> = ctx.core.player(owner).equips.clone();
                    for c in equips {
                        ctx.core.player_mut(owner).equips.retain(|x| *x != c);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, owner, c);
                        ctx.to_discard(c);
                    }
                    let judge: Vec<CardId> = ctx.core.player(owner).judge.clone();
                    for c in judge {
                        ctx.core.player_mut(owner).judge.retain(|x| *x != c);
                        ctx.to_discard(c);
                    }
                    ctx.start_draw(owner, 3);
                }
                Flow::Done
            }
            HostBehavior::XianjianSheying => {
                // 射影（U05 xianjian 苏媚，规则 v175）：判定黑色→获得伤
                // 害牌（direct 无询问；每回合 1 次=TurnState.sheying_used）。
                ctx.core.turn.sheying_used = true;
                let judged = ctx.draw_judge_card();
                let suit = ctx.catalog.cards[judged as usize].suit;
                ctx.log(LogEvent::Judged {
                    player: self.owner,
                    card: judged,
                });
                ctx.to_discard(judged);
                let is_black = matches!(
                    suit,
                    crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                );
                if is_black {
                    // target 未用——伤害牌从 cards[0] 取
                    if let Some(c) = self.cards.first().copied() {
                        if let Some(pos) = ctx.core.discard.iter().position(|x| *x == c) {
                            ctx.core.discard.remove(pos);
                            ctx.core.player_mut(self.owner).hand.push(c);
                            ctx.log(LogEvent::CardsDrawn {
                                player: self.owner,
                                cards: vec![c],
                            });
                        }
                    }
                }
                Flow::Done
            }
            HostBehavior::XianjianLiefeng => {
                // 疾风（U05 xianjian 姜云凡，规则 v178）：第 3/4 张使用
                // 牌时 runtime 授天剑/御风实例（第 2 张授炎杀 yanzhan
                // viewAs 暂缓不授；上游 xjyufeng 授予时补牌由御风自身
                // filter 兜底，此处仅授技）。
                let n = ctx.core.turn.cards_used_play_phase;
                let skill_name = match n {
                    3 => Some("tianjian"),
                    4 => Some("xjyufeng"),
                    _ => None,
                };
                if let Some(name) = skill_name {
                    if let Some(def) = ctx.catalog.skill_by_name(name) {
                        if !ctx.core.skills.contains_key(&(self.owner, def)) {
                            ctx.insert_skill(
                                (self.owner, def),
                                crate::state::SkillInstance {
                                    owner: self.owner,
                                    enabled: true,
                                    once_used: false,
                                    expires_end_of_turn: false,
                                    rounds: 0,
                                },
                            );
                        }
                    }
                }
                Flow::Done
            }
            HostBehavior::XianjianXshuangren => {
                // 双刃（U05 xianjian 雪见，规则 v180）：弃牌中的武器从弃
                // 堆回收置离场区+grant 该武器技能（上游 addAdditionalSkill
                // 同口径；**登记差异**：仅弃置路径触发）。
                for c in self.cards.clone() {
                    let is_weapon = ctx.catalog.card_slot_of(ctx.core, c)
                        == Some(crate::catalog::EquipSlot::Weapon);
                    let Some(pos) = ctx.core.discard.iter().position(|x| *x == c) else {
                        continue;
                    };
                    if is_weapon {
                        ctx.core.discard.remove(pos);
                        ctx.core.player_mut(self.owner).off_court.push(c);
                        crate::kernel::grant_equip_skills(ctx, self.owner, c);
                    }
                }
                Flow::Done
            }
            HostBehavior::XianjianXshuangren2 => {
                // 双刃·殁（U05 xianjian 雪见，规则 v180）：死亡时弃置离
                // 场区全部牌（通用区清空=登记差异）。
                let oc: Vec<CardId> = ctx.core.player(self.owner).off_court.clone();
                for c in oc {
                    ctx.core
                        .player_mut(self.owner)
                        .off_court
                        .retain(|x| *x != c);
                    ctx.to_discard(c);
                }
                Flow::Done
            }
            HostBehavior::XianjianXunying => {
                // 迅影（U05 xianjian 姜云凡，规则 v183）：杀结算后自动
                // 再出一张手牌杀对同一目标（**登记差异**：上游
                // chooseToUse 玩家选牌，原生自动取首张杀；酒状态保留不
                // 迁移；sha 限次不适用=CardUsed 直发不经 PlayCard 校验）。
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if !ctx.core.player(t).alive {
                    return Flow::Done;
                }
                let Some(sha) = ctx
                    .core
                    .player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .find(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Sha)
                else {
                    return Flow::Done;
                };
                ctx.emit(EventData::CardUsed(crate::event::CardUseData {
                    card: Some(sha),
                    player: self.owner,
                    targets: vec![t],
                    nature: crate::event::DamageNature::Normal,
                    use_as: None,
                    cancelled: false,
                }));
                Flow::Done
            }
            HostBehavior::XianjianTianxian => {
                // 天弦（锁定旗标行为）：无机器步
                Flow::Done
            }
            HostBehavior::XianjianXjyingfeng => {
                // 影锋（U05 xianjian 越今朝，规则 v185）：随机候选虚拟
                // 杀（use_virtual_card=牌堆底载体完整管线；虚拟杀
                // use_as=Some(Sha) 不再命中本 filter=防递归）。
                // 影锋虚拟杀结算后窗拒绝再触发：emit 前置链挡旗标（上游
                // parent.name!=="xjyingfeng" 的登记差异等价——同回合多张
                // 真实杀仅首次触发，Judge begin 清）
                ctx.core.turn.yingfeng_chain = true;
                let exclude: Vec<PlayerId> = match ctx.current_card_use() {
                    Ok(u) => u.targets.clone(),
                    Err(_) => Vec::new(),
                };
                if !exclude.is_empty() {
                    let candidates: Vec<PlayerId> = ctx
                        .core
                        .seat_order_from(self.owner)
                        .into_iter()
                        .filter(|t| {
                            *t != self.owner
                                && !exclude.contains(t)
                                && ctx.core.player(*t).alive
                                && ctx.core.computed_distance(ctx.catalog, self.owner, *t)
                                    <= ctx.core.attack_range(ctx.catalog, self.owner)
                        })
                        .collect();
                    if let Some(&pick) =
                        candidates.get(ctx.core.rng.gen_range(candidates.len() as u32) as usize)
                    {
                        ctx.use_virtual_card(self.owner, crate::catalog::CardKind::Sha, vec![pick]);
                    }
                }
                Flow::Done
            }
            HostBehavior::XianjianCtianfu3 => {
                // 天符·清挂（ctianfu3，规则 v188）：挂附者死亡——挂附闪
                // 入弃堆（**登记差异**：上游凭空消失）+清挂附+清两实例。
                ctianfu_clear(ctx, self.owner);
                for sub in ["ctianfu2", "ctianfu3"] {
                    if let Some(def) = ctx.catalog.skill_by_name(sub) {
                        ctx.core.skills.remove(&(self.owner, def));
                    }
                }
                Flow::Done
            }
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
