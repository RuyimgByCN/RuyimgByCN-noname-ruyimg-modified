use super::super::*;

impl HostMachine {
    pub(super) fn step_gwent(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            HostBehavior::GwentYanzhiCard => {
                // 目标为手牌数最少者（之一）→ 摸 2；否则摸 1
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if ctx.core.player(t).alive {
                    let min_hand = ctx
                        .core
                        .alive_players()
                        .all(|p| ctx.core.player(p).hand.len() >= ctx.core.player(t).hand.len());
                    let n = if min_hand { 2 } else { 1 };
                    ctx.start_draw(t, n as u32);
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::GwentAozuCard => match self.stage {
                HostStage::Act => {
                    // 1 点雷电伤害，待伤害结算后再判存活摸 1（LIFO：Damage 先弹栈）
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if ctx.core.player(t).alive {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Thunder,
                            self.card,
                        )));
                        self.stage = HostStage::DrawSettle;
                        return Flow::Yield;
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                HostStage::DrawSettle => {
                    if let Some(t) = self.target {
                        if ctx.core.player(t).alive {
                            ctx.start_draw(t, 1);
                        }
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::GwentWenyiCard => {
                // 瘟疫（`card/gwent/card.js:1814`）：所有体力最少者（`isMinHp()`）
                // 按座位序随机弃 1 张手牌；无手牌则失去 1 点体力。
                let min_hp = ctx
                    .core
                    .alive_players()
                    .map(|p| ctx.core.player(p).hp)
                    .min();
                if let Some(min_hp) = min_hp {
                    let targets: Vec<PlayerId> = ctx
                        .core
                        .seat_order_from(self.owner)
                        .into_iter()
                        .filter(|&p| ctx.core.player(p).alive && ctx.core.player(p).hp == min_hp)
                        .collect();
                    for t in targets {
                        if !ctx.core.player(t).alive {
                            continue;
                        }
                        let hand_len = ctx.core.player(t).hand.len();
                        if hand_len > 0 {
                            let i = ctx.core.rng.gen_range(hand_len as u32) as usize;
                            let c = ctx.core.player(t).hand[i];
                            ctx.discard_from_hand(t, c);
                        } else {
                            ctx.emit(EventData::LoseHp {
                                player: t,
                                amount: 1,
                            });
                        }
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::GwentZhihuanCard => {
                // 致幻菌（`card/gwent/card.js:1585`）：反复经 `lose_max_hp` 失去 1 点体力上限，
                // 直至未受伤或手牌数 >= 体力上限。
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                while ctx.core.player(t).alive {
                    let p = ctx.core.player(t);
                    let damaged = p.hp < p.max_hp as i32;
                    if !(damaged && (p.hand.len() as u32) < p.max_hp) {
                        break;
                    }
                    ctx.lose_max_hp(t, 1);
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::GwentBaoxueCard => match self.stage {
                HostStage::SelectCard => {
                    // 上游 chooseToDiscard("h",2,true)：目标自选两张手牌
                    //（不足则全选，逐技对照批五 v79 消除「前两张」登记差异）
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let n = ctx.core.player(t).hand.len().min(2);
                    if n == 0 || !ctx.core.player(t).alive {
                        // 无手牌：跳过弃牌；上游 step1 draw 恒执行
                        if ctx.core.player(t).alive {
                            ctx.start_draw(t, 1);
                        }
                        if let Some(c) = self.card {
                            ctx.to_discard(c);
                            ctx.core.table.retain(|x| *x != c);
                        }
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    self.stage = HostStage::Act;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: GW_BAOXUE_DISCARD,
                            options,
                            min: n,
                            max: n,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::Act => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let picks: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    let mut discarded = Vec::with_capacity(picks.len());
                    for c in picks {
                        if ctx.core.player(t).hand.contains(&c) {
                            ctx.discard_from_hand(t, c);
                            discarded.push(c);
                        }
                    }
                    if !discarded.is_empty() {
                        ctx.emit(EventData::CardsDiscarded {
                            player: t,
                            cards: discarded,
                        });
                    }
                    if ctx.core.player(t).alive {
                        self.stage = HostStage::DrawSettle;
                        return Flow::Yield;
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                HostStage::DrawSettle => {
                    if let Some(t) = self.target {
                        if ctx.core.player(t).alive {
                            ctx.start_draw(t, 1);
                        }
                    }
                    if let Some(c) = self.card {
                        ctx.to_discard(c);
                        ctx.core.table.retain(|x| *x != c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::GwentQinpenState | HostBehavior::GwentBirinState => {
                // 天气到期移除（效果为被动钩子，无步骤）
                food_remove(ctx, self.owner, &self.def_key);
                Flow::Done
            }
            HostBehavior::GwentCiguState => {
        // 上游 `gw_ciguhanshuang` content: `trigger.num--; player.removeSkill(...)`
        // ⇒ 摸牌数 -1 由 `kernel/phase/draw.rs` 计算，这里**只移除状态**
        //（第 938 轮修正：原实现是「弃首张手牌」，效果不对）
        food_remove(ctx, self.owner, &self.def_key);
        Flow::Done
    }
    HostBehavior::GwentWeatherGrant => {
                // 对 targets（目标 + 左右存活邻座）逐个安装对应天气状态
                //（已持有者跳过，对齐上游 filterTarget）
                let kind = self.card.map(|c| ctx.catalog.card_kind(c));
                let (listener, alt_name) = match (kind, self.def_key.as_str()) {
                    (Some(CardKind::Qinpendayu), _) | (_, GW_QINPEN_CARD_KEY) => {
                        (Some(GW_QINPEN_STATE_KEY), Some("gw_qinpendayu"))
                    }
                    (Some(CardKind::Birinongwu), _) | (_, GW_BIRIN_CARD_KEY) => {
                        (Some(GW_BIRIN_STATE_KEY), Some("gw_birinongwu"))
                    }
                    (Some(CardKind::Ciguhanshuang), _) | (_, GW_CIGU_CARD_KEY) => {
                        (Some(GW_CIGU_STATE_KEY), Some("gw_ciguhanshuang"))
                    }
                    _ => (None, None),
                };
                let (Some(listener), Some(alt)) = (listener, alt_name) else {
                    return Flow::Done;
                };
                let def = ctx
                    .catalog
                    .skill_by_name(listener)
                    .or_else(|| ctx.catalog.skill_by_name(alt));
                let Some(def) = def else {
                    return Flow::Done;
                };
                let alt_def = ctx.catalog.skill_by_name(alt);
                let targets = self.targets.clone();
                for t in targets {
                    if !ctx.core.player(t).alive
                        || ctx.core.skills.contains_key(&(t, def))
                        || alt_def.is_some_and(|ad| ctx.core.skills.contains_key(&(t, ad)))
                    {
                        continue;
                    }
                    ctx.insert_skill(
                        (t, def),
                        crate::state::SkillInstance {
                            owner: t,
                            enabled: true,
                            once_used: false,
                            expires_end_of_turn: false,
                            rounds: 0,
                        },
                    );
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::GwentKunenState => {
                let is_turn_end = ctx
                    .res
                    .frame(ctx.current_event_frame_id())
                    .is_some_and(|f| matches!(f.data, EventData::TurnEndWindow { after: false, .. }));
                if is_turn_end {
                    // `gw_kunenfayin_count` (`card/gwent/skill.js:64`): 全局 phaseEnd 递减计数，归零移除
                    food_tick(ctx, self.owner, &self.def_key);
                } else {
                    // `gw_kunenfayin` (`card/gwent/skill.js:45`): 仅防止所有非属性伤害
                    ctx.current_damage_mut().prevented = true;
                }
                Flow::Done
            }
            HostBehavior::GwentKunenCard => {
                // 授予昆恩状态：rounds = min(5, 存活数)
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                let def = ctx
                    .catalog
                    .skill_by_name(GW_KUNEN_STATE_KEY)
                    .or_else(|| ctx.catalog.skill_by_name("gw_kunenfayin"));
                let Some(def) = def else {
                    return Flow::Done;
                };
                let rounds = (ctx.core.alive_count() as u8).min(5);
                ctx.insert_skill(
                    (t, def),
                    crate::state::SkillInstance {
                        owner: t,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds,
                    },
                );
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::GwentShanbengCard => {
                // 山崩术（`card/gwent/card.js:1850`）：所有敌方装备区随机取至多 2 张弃置
                let pool: Vec<(crate::ids::PlayerId, CardId)> = ctx
                    .core
                    .alive_players()
                    .filter(|v| {
                        *v != self.owner && !ctx.core.is_friend_of(ctx.catalog, self.owner, *v)
                    })
                    .flat_map(|v| {
                        ctx.core
                            .player(v)
                            .equips
                            .iter()
                            .copied()
                            .map(move |c| (v, c))
                            .collect::<Vec<_>>()
                    })
                    .collect();
                let picked: Vec<(crate::ids::PlayerId, CardId)> = if pool.len() >= 2 {
                    let n = pool.len() as u32;
                    let i = ctx.core.rng.gen_range(n) as usize;
                    let mut j = ctx.core.rng.gen_range(n - 1) as usize;
                    if j >= i {
                        j += 1;
                    }
                    vec![pool[i], pool[j]]
                } else {
                    pool
                };
                for (v, c) in picked {
                    ctx.core.player_mut(v).equips.retain(|x| *x != c);
                    crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, v, c);
                    ctx.to_discard(c);
                    ctx.note_cards_lost(v, &[c]);
                    ctx.emit(EventData::CardsLost {
                        player: v,
                        cards: vec![c],
                        origins: vec![crate::event::LostOrigin::Equip],
                    });
                    ctx.log(LogEvent::CardDiscarded {
                        player: v,
                        cards: vec![c],
                    });
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::GwentPoxiaoCard => super::u05_b5a_gwent::step_poxiao_card(self, ctx),
            HostBehavior::GwentFengyinState => {
                // 祖墨手铐·封印解除（`card/gwent/card.js:1625`, `skill.js:1702`）：
                // 恢复封印期间禁用的非锁定/非持恒/非 charlotte 技能，并移除封印实例。
                let prefix = "gw_fengyin_dis:";
                let keys: Vec<(u16, String)> = ctx
                    .core
                    .vm_aux
                    .flags
                    .keys()
                    .filter(|(p, k)| *p == self.owner && k.starts_with(prefix))
                    .filter_map(|(_p, k)| {
                        k.strip_prefix(prefix)
                            .and_then(|s| s.parse::<u16>().ok())
                            .map(|def| (def, k.clone()))
                    })
                    .collect();
                for (def, flag_key) in keys {
                    ctx.core.vm_aux.flags.remove(&(self.owner, flag_key));
                    if let Some(inst) = ctx.core.skills.get_mut(&(self.owner, def)) {
                        inst.enabled = true;
                    }
                }
                if let Some(def) = ctx.catalog.skill_by_name(GW_FENGYIN_STATE_KEY) {
                    ctx.core.skills.remove(&(self.owner, def));
                }
                Flow::Done
            }
            HostBehavior::GwentZumoCard => {
                // 祖墨手铐（`card/gwent/card.js:1616`）：禁用目标全部非锁定技
                //（豁免锁定技、持恒技与 charlotte 技，含天气状态），直至其整回合结束。
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                let Some(fdef) = ctx.catalog.skill_by_name(GW_FENGYIN_STATE_KEY) else {
                    return Flow::Done;
                };
                let defs: Vec<crate::ids::SkillDefId> = ctx
                    .core
                    .skills
                    .iter()
                    .filter_map(|((p, d), inst)| (*p == t && *d != fdef && inst.enabled).then_some(*d))
                    .collect();
                for def in defs {
                    let skill = &ctx.catalog.skills[def as usize];
                    let weather_non_locked = matches!(
                        skill.name,
                        GW_QINPEN_STATE_KEY | "gw_qinpendayu" | GW_BIRIN_STATE_KEY | "gw_birinongwu"
                    );
                    let eff_meta = if weather_non_locked {
                        None
                    } else {
                        skill.trigger.as_ref()
                    };
                    let exempt = ctx.catalog.is_forever_skill(def)
                        || ctx.catalog.is_charlotte_skill(def)
                        || crate::catalog::Catalog::is_locked_skill_name(skill.name, eff_meta);
                    if exempt {
                        continue;
                    }
                    if let Some(inst) = ctx.core.skills.get_mut(&(t, def)) {
                        inst.enabled = false;
                        ctx.core
                            .vm_aux
                            .flags
                            .insert((t, format!("gw_fengyin_dis:{def}")), 1);
                    }
                }
                ctx.insert_skill(
                    (t, fdef),
                    crate::state::SkillInstance {
                        owner: t,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: false,
                        rounds: 0,
                    },
                );
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
