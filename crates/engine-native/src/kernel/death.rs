use super::*;

/// 死亡结算：清空牌区（手牌/装备/判定区）、移除技能实例、
/// 身份局奖惩与胜负判定。
///
/// 奖惩（身份局，ADR 0705）：杀死反贼摸三张；主公杀死忠臣弃掉全部手牌与
/// 装备。奖惩先于胜负判定（先结算再判终局，口径挂账见清单）。
/// 胜负（身份局正式规则）：主公死亡 → 内奸是唯一存活则内奸胜，否则反贼胜；
/// 反贼与内奸全灭 → 主公方胜。原型模式维持存活数 ≤ 1。
#[derive(Clone)]
pub struct DeathMachine {
    pub(super) player: PlayerId,
    pub(super) killer: Option<PlayerId>,
}

impl DeathMachine {
    pub(crate) fn new(player: PlayerId, killer: Option<PlayerId>) -> Self {
        Self { player, killer }
    }
}

impl crate::catalog::SkillMachine for DeathMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Death {
            player: self.player,
            killer: self.killer,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 死亡拦截（规则 v171 死亡取消型基建）：标记置位则跳过死亡执
        // 行（存活态保持——前置窗口技已重置体力；上游 dieBefore
        // trigger.cancel 同口径）。
        // 天书移除（swd 批 142，规则 v317）：来源角色阵亡 → 移除所获技能
        // （上游 swdtianshu_remove dieAfter 同口径）
        {
            let dying = self.player;
            for p in 0..ctx.core.players.len() as PlayerId {
                if ctx.core.player(p).swdtianshu_from != Some(dying) {
                    continue;
                }
                let granted = ctx.core.player(p).swdtianshu_granted;
                if let Some(def) = granted.map(|d| d as crate::ids::SkillDefId) {
                    ctx.core.skills.remove(&(p, def));
                    ctx.log(LogEvent::SkillToggled {
                        player: p,
                        skill: ctx.catalog.skill_name(def),
                        enabled: false,
                    });
                }
                let pl = ctx.core.player_mut(p);
                pl.swdtianshu_from = None;
                pl.swdtianshu_granted = None;
            }
        }
        // 护天归还（swd 批 131，规则 v314）：持有者死亡前置（上游 hutian3
        // dieBegin forced）——把其寄存于目标处的牌归还目标并清理双方状态
        hutian_return_deposit(ctx, self.player);
        if ctx.core.player(self.player).death_cancelled {
            ctx.core.player_mut(self.player).death_cancelled = false;
            // 取消死亡=脱离濒死（空洞印批⑥：hp 由前置窗口技重置；不清
            // dying 旗会令后续致死伤害不再进濒死流程）
            ctx.core.player_mut(self.player).dying = false;
            return Flow::Done;
        }
        // versus 替补（规则 v206）：死亡座位所在队替补池非空 → 弹队首
        // 换将复活（上游 revive+uninit/init+摸 4+濒死链取消同口径：死亡
        // 执行被替补替代——本座位不进入死亡结算，奖惩/终局判定不发生；
        // 登记差异：上游 dieAfter 展示与「出场」日志不入原生日志，替补
        // 出场为新将满血开局）。
        {
            let team = ctx.core.versus_team_of(self.player);
            if let Some(ti) = team {
                let has_sub = ctx
                    .core
                    .versus_substitutes()
                    .and_then(|pools| pools.get(ti))
                    .is_some_and(|pool| !pool.is_empty());
                if has_sub {
                    let g = match &mut ctx.core.mode {
                        crate::state::GameMode::Versus { substitutes, .. } => {
                            substitutes[ti].remove(0)
                        }
                        _ => unreachable!("versus_team_of 只在 Versus 模式"),
                    };
                    // 换将前清本座位技能与区域（上游 uninit/init 重建面：
                    // 手牌/装备/判定区离场入弃堆，替补从零开始）
                    let removed: Vec<crate::ids::SkillDefId> = ctx
                        .core
                        .skills
                        .keys()
                        .filter(|(o, _)| *o == self.player)
                        .map(|(_, d)| *d)
                        .collect();
                    for def in removed {
                        ctx.core.skills.remove(&(self.player, def));
                    }
                    let leftover = {
                        let p = ctx.core.player_mut(self.player);
                        let mut leftover = std::mem::take(&mut p.hand);
                        leftover.extend(std::mem::take(&mut p.equips));
                        leftover.extend(std::mem::take(&mut p.judge));
                        p.alive = true;
                        p.dying = false;
                        p.death_cancelled = false;
                        p.general_slots = [Some(g), None];
                        leftover
                    };
                    for c in leftover {
                        ctx.to_discard(c);
                    }
                    crate::state::apply_general_assembly(ctx.core, ctx.catalog, self.player, 0);
                    let p = ctx.core.player_mut(self.player);
                    p.hp = p.max_hp as i32;
                    ctx.draw_cards(self.player, 4);
                    return Flow::Done;
                }
            }
        }
        // 回火（Boss 焰灵，规则 v241）：死亡反噬标记——技能实例随死亡
        // 结算移除，故在移除前捕获
        let had_huihuo = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_HUIHUO)
            .map(|d| ctx.core.skills.contains_key(&(self.player, d)))
            .unwrap_or(false);
        let p = ctx.core.player_mut(self.player);
        p.alive = false;
        p.dying = false;
        p.hp = 0;
        let mut leftover = std::mem::take(&mut p.hand);
        leftover.extend(std::mem::take(&mut p.equips));
        leftover.extend(std::mem::take(&mut p.judge));
        for c in leftover {
            ctx.to_discard(c);
        }
        let removed: Vec<crate::ids::SkillDefId> = ctx
            .core
            .skills
            .keys()
            .filter(|(o, _)| *o == self.player)
            .map(|(_, d)| *d)
            .collect();
        for def in removed {
            ctx.core.skills.remove(&(self.player, def));
            ctx.log(LogEvent::SkillToggled {
                player: self.player,
                skill: ctx.catalog.skill_name(def),
                enabled: false,
            });
        }
        ctx.log(LogEvent::PlayerDied {
            player: self.player,
            killer: self.killer,
        });
        // 造异（Boss 颛顼，规则 v246）：玄冥/共工死亡且场上无存活同名
        // 将 → 造异持有者摸 4 并获「造异异变」（运行时授予；技能实例已
        // 移除，故以死亡者武将名判定）
        {
            let dead_name = (|| {
                let g = ctx.core.player(self.player).general_slots[0];
                let g = g?;
                let base = (ctx.catalog.general_count()
                    - crate::vocabulary::generals::BOSS_GENERALS.len())
                    as u16;
                if g < base {
                    return None;
                }
                ctx.catalog.general_meta(g).map(|meta| meta.0.to_string())
            })();
            if dead_name.as_deref() == Some("共工") || dead_name.as_deref() == Some("玄冥") {
                let base = (ctx.catalog.general_count()
                    - crate::vocabulary::generals::BOSS_GENERALS.len())
                    as u16;
                let named_alive = (base..ctx.catalog.general_count() as u16)
                    .filter(|&d| {
                        matches!(
                            ctx.catalog.general_meta(d).map(|m| m.0),
                            Some(n) if n == "共工" || n == "玄冥"
                        )
                    })
                    .any(|gd| {
                        (0..ctx.core.players.len() as u8).any(|t| {
                            ctx.core.player(t).alive
                                && ctx.core.player(t).general_slots[0] == Some(gd)
                        })
                    });
                if !named_alive {
                    let zdef = ctx
                        .catalog
                        .skill_by_name(crate::vocabulary::names::BOSS_ZAOYI);
                    if let Some(zdef) = zdef {
                        let holders: Vec<PlayerId> = ctx
                            .core
                            .skills
                            .iter()
                            .filter(|((_, dd), i)| *dd == zdef && i.enabled)
                            .map(|((o, _), _)| *o)
                            .filter(|o| ctx.core.player(*o).alive)
                            .collect();
                        for holder in holders {
                            ctx.draw_cards(holder, 4);
                            ctx.core.skills.insert(
                                (holder, zhp_def(ctx)),
                                crate::state::SkillInstance {
                                    owner: holder,
                                    enabled: true,
                                    once_used: false,
                                    expires_end_of_turn: false,
                                    rounds: 0,
                                },
                            );
                            ctx.log(LogEvent::SkillToggled {
                                player: holder,
                                skill: ctx.catalog.skill_name(zhp_def(ctx)),
                                enabled: true,
                            });
                        }
                    }
                }
            }
        }
        // 回火（Boss 焰灵，规则 v241）：死亡结算完成后其敌方全体各受
        // 3 点无来源火伤（上游 boss_huihuo2 global dieAfter 同口径）
        if had_huihuo {
            let seats = ctx.core.players.len() as PlayerId;
            for t in 0..seats {
                if !ctx.core.player(t).alive {
                    continue;
                }
                if ctx.core.is_boss_side(t) != ctx.core.is_boss_side(self.player) {
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        None,
                        t,
                        3,
                        crate::event::DamageNature::Fire,
                        None,
                    )));
                }
            }
        }
        // 身份局奖惩：先于胜负判定
        if let Some(roles) = ctx.core.identity_roles() {
            let dead_role = roles[self.player as usize];
            if let Some(k) = self.killer {
                if k != self.player && ctx.core.player(k).alive {
                    match dead_role {
                        crate::state::Role::Rebel => {
                            ctx.draw_cards(k, 3);
                        }
                        crate::state::Role::Loyalist
                            if roles[k as usize] == crate::state::Role::Lord =>
                        {
                            // 主公杀忠臣：弃掉全部手牌与装备
                            let lord = ctx.core.player_mut(k);
                            let mut cards = std::mem::take(&mut lord.hand);
                            let lord_equips = std::mem::take(&mut lord.equips);
                            for c in &lord_equips {
                                revoke_equip_skills(ctx.core, ctx.catalog, k, *c);
                            }
                            cards.extend(lord_equips);
                            ctx.core.discard.extend(cards.clone());
                            if !cards.is_empty() {
                                ctx.log(LogEvent::CardDiscarded { player: k, cards });
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        // 对决 2v2 奖惩（P2/N08 模式入口批一，规则 v80）：先于胜负判定——
        // 杀死敌方摸三张；杀死友方弃掉自己全部手牌与装备（上游 versus
        // 2v2 基线；替补机制未建模）。
        if let Some(teams) = ctx.core.versus_teams().map(|t| t.to_vec()) {
            if let Some(k) = self.killer {
                if k != self.player && ctx.core.player(k).alive {
                    let dead_team = teams.iter().position(|t| t.contains(&self.player));
                    let killer_team = ctx.core.versus_team_of(k);
                    if let (Some(dt), Some(kt)) = (dead_team, killer_team) {
                        if dt != kt {
                            ctx.draw_cards(k, 3);
                        } else {
                            // 杀友方：弃掉自己全部手牌与装备
                            let killer = ctx.core.player_mut(k);
                            let mut cards = std::mem::take(&mut killer.hand);
                            let killer_equips = std::mem::take(&mut killer.equips);
                            for c in &killer_equips {
                                revoke_equip_skills(ctx.core, ctx.catalog, k, *c);
                            }
                            cards.extend(killer_equips);
                            ctx.core.discard.extend(cards.clone());
                            if !cards.is_empty() {
                                ctx.log(LogEvent::CardDiscarded { player: k, cards });
                            }
                        }
                    }
                }
            }
        }
        // 胜负判定
        match ctx.core.mode.clone() {
            crate::state::GameMode::Prototype => {
                let last = ctx.core.alive_players().next();
                if ctx.core.alive_count() <= 1 {
                    if let Some(w) = last {
                        ctx.core.outcome = Some(crate::state::Outcome {
                            winner: crate::state::Winner::Player(w),
                        });
                    }
                }
            }
            crate::state::GameMode::Identity { roles } => {
                let lord = roles
                    .iter()
                    .position(|r| *r == crate::state::Role::Lord)
                    .expect("identity mode has a lord");
                let lord_alive = ctx.core.player(lord as PlayerId).alive;
                if !lord_alive {
                    // 主公死亡：内奸是唯一存活 → 内奸胜；否则反贼胜
                    let traitor_sole = ctx.core.alive_count() == 1
                        && roles.iter().enumerate().any(|(i, r)| {
                            *r == crate::state::Role::Traitor
                                && ctx.core.player(i as PlayerId).alive
                        });
                    ctx.core.outcome = Some(crate::state::Outcome {
                        winner: if traitor_sole {
                            crate::state::Winner::Traitor
                        } else {
                            crate::state::Winner::Rebels
                        },
                    });
                } else {
                    // 反贼与内奸全灭 → 主公方胜
                    let rebels_gone = !roles.iter().enumerate().any(|(i, r)| {
                        matches!(r, crate::state::Role::Rebel | crate::state::Role::Traitor)
                            && ctx.core.player(i as PlayerId).alive
                    });
                    if rebels_gone {
                        ctx.core.outcome = Some(crate::state::Outcome {
                            winner: crate::state::Winner::LordSide,
                        });
                    }
                }
            }
            crate::state::GameMode::Kingdoms => {
                // 国战（N11 M3 门控，ADR 0756/0757）：存活角色全部同一
                // **已声明**势力（主将明置优先，主暗副明按副将）→ 该势力胜；
                // 任一存活角色势力未定（双暗置/槽空）→ 不判负，对局继续。
                let mut factions: Vec<crate::catalog::Faction> = Vec::new();
                for p in ctx.core.alive_players() {
                    let Some(f) = ctx.core.declared_faction(ctx.catalog, p) else {
                        return Flow::Done;
                    };
                    if !factions.contains(&f) {
                        factions.push(f);
                    }
                }
                if factions.len() == 1 {
                    ctx.core.outcome = Some(crate::state::Outcome {
                        winner: crate::state::Winner::Faction(factions[0]),
                    });
                }
            }
            crate::state::GameMode::Versus { teams, .. } => {
                // 对决（P2/N08 模式入口批一）：任一队伍全灭 → 对方队伍胜
                for (i, t) in teams.iter().enumerate() {
                    if t.iter().all(|p| !ctx.core.player(*p).alive) {
                        ctx.core.outcome = Some(crate::state::Outcome {
                            winner: crate::state::Winner::Versus(1 - i),
                        });
                        break;
                    }
                }
            }
            crate::state::GameMode::Doudizhu { lord } => {
                // 斗地主（P2/N08 模式入口批二）：地主死亡 → 农民胜；
                // 农民全灭 → 地主胜（上游 game.over(me===game.zhu) 语义；
                // 无击杀奖惩——dieAfter 仅 UI+判负）
                let lord_alive = ctx.core.player(lord).alive;
                if !lord_alive {
                    ctx.core.outcome = Some(crate::state::Outcome {
                        winner: crate::state::Winner::Doudizhu { lord_won: false },
                    });
                } else {
                    let farmers_gone = ctx.core.alive_players().all(|p| p == lord);
                    if farmers_gone {
                        ctx.core.outcome = Some(crate::state::Outcome {
                            winner: crate::state::Winner::Doudizhu { lord_won: true },
                        });
                    }
                }
            }
            crate::state::GameMode::Kof {
                mut pools,
                mut wins,
                games,
            } => {
                // KOF 引擎层（P2/N08 模式入口批四，规则 v83）：2 人局，
                // 唯一存活者胜并推进会话——败者队首淘汰、胜者队首轮转
                //（移至队尾）、比分/局数 +1。无存活者（理论不可达）不推进。
                let last = ctx.core.alive_players().next();
                if let Some(w) = last {
                    ctx.core.outcome = Some(crate::state::Outcome {
                        winner: crate::state::Winner::Player(w),
                    });
                    let wi = w as usize;
                    let li = 1 - wi;
                    if !pools[li].is_empty() {
                        pools[li].remove(0);
                    }
                    if !pools[wi].is_empty() {
                        let kept = pools[wi].remove(0);
                        pools[wi].push(kept);
                    }
                    wins[wi] += 1;
                    // games 经可变借用直接更新
                    ctx.core.mode = crate::state::GameMode::Kof {
                        pools,
                        wins,
                        games: games + 1,
                    };
                }
            }
            crate::state::GameMode::Boss { boss, fellows, .. } => {
                // Boss（P2/N08 模式入口批三-a）：boss 死亡 → 挑战者胜；
                // 挑战者全灭 → boss 胜（上游 checkResult 同构）。
                let boss_alive = ctx.core.player(boss).alive;
                if !boss_alive {
                    ctx.core.outcome = Some(crate::state::Outcome {
                        winner: crate::state::Winner::Boss { boss_won: false },
                    });
                } else {
                    let challengers_gone = ctx
                        .core
                        .alive_players()
                        .all(|p| p == boss || fellows.contains(&p));
                    if challengers_gone {
                        ctx.core.outcome = Some(crate::state::Outcome {
                            winner: crate::state::Winner::Boss { boss_won: true },
                        });
                    }
                }
            }
        }
        Flow::Done
    }
}
