use super::super::*;

impl HostMachine {
    pub(super) fn step_wudian_status(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 复仇：目录锚（取消逻辑在 ShaResolveMachine 内联）
            HostBehavior::WudianFuchou => Flow::Done,
            // 坚壁：锁定查询面（AOE 目标段查询），无独立效果步
            HostBehavior::WudianJianbi => Flow::Done,
            // 理断：同栏已有装备 → 其交首张手牌（登记差异：上游自选）；
            // 否则直接装备该牌（不走 Equipped 帧防双 grant）
            HostBehavior::WudianLiduan => {
                let to = match self.target {
                    Some(t) => t,
                    None => return Flow::Done,
                };
                let Some(card) = self.card else {
                    return Flow::Done;
                };
                let Some(slot) = ctx.catalog.card_slot_of(ctx.core, card) else {
                    return Flow::Done;
                };
                let same_slot = ctx
                    .core
                    .player(to)
                    .equips
                    .iter()
                    .any(|e| ctx.catalog.card_slot_of(ctx.core, *e) == Some(slot));
                if same_slot {
                    // 交首张手牌给持有者
                    if let Some(c) = ctx.core.player(to).hand.first().copied() {
                        ctx.core.player_mut(to).hand.retain(|x| *x != c);
                        ctx.core.player_mut(self.owner).hand.push(c);
                        ctx.emit(EventData::CardsMoved {
                            cards: vec![c],
                            from: to,
                            to: self.owner,
                        });
                        ctx.log(LogEvent::CardsGiven {
                            from: to,
                            to: self.owner,
                            cards: vec![c],
                        });
                    }
                } else {
                    ctx.core.player_mut(to).hand.retain(|x| *x != card);
                    let old_equip = ctx
                        .core
                        .player(to)
                        .equips
                        .iter()
                        .copied()
                        .find(|e| ctx.catalog.card_slot_of(ctx.core, *e) == Some(slot));
                    if let Some(old) = old_equip {
                        ctx.core.player_mut(to).equips.retain(|x| *x != old);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, to, old);
                        ctx.to_discard(old);
                    }
                    ctx.core.player_mut(to).equips.push(card);
                    crate::kernel::grant_equip_skills(ctx, to, card);
                    ctx.log(LogEvent::Equipped {
                        player: to,
                        card,
                        replaced: old_equip,
                    });
                }
                Flow::Done
            }
            // 复仇·清算：标记来源逐个摸 1+虚拟杀强制指定（死亡移除）
            HostBehavior::WudianFuchou2 => {
                let mut marked = std::mem::take(&mut ctx.core.player_mut(self.owner).fuchou_marked);
                marked.retain(|src| ctx.core.player(*src).alive);
                for src in marked {
                    ctx.draw_cards(self.owner, 1);
                    ctx.use_virtual_card(self.owner, crate::catalog::CardKind::Sha, vec![src]);
                }
                Flow::Done
            }
            // 良策·余获：锁定查询面（五谷机器内联查询），无独立效果步
            HostBehavior::WudianLiangceRemain => Flow::Done,
            // 归汉：目录锚（无独立效果步）
            HostBehavior::WudianGuihan => Flow::Done,
            // 禁言：查询锚（PlayCard 门），无独立效果步
            HostBehavior::WudianJinyan => Flow::Done,
            HostBehavior::WudianCiqiu => {
                // 刺酋（玩点论杀/韩龙，规则 v111）：锁定——你的杀对满血目标
                // 伤害 +1；加成后达到目标体力则其本回合濒死即死、本技能移除。
                // 上游 damageBegin1；效果在 commit_damage（kernel.rs 142 行）
                // 与濒死处理（kernel.rs 3568 行），宿主无机器步骤。
                Flow::Done
            }
            HostBehavior::WudianZhucheng => match self.stage {
                // 筑城开关（U07 批三 v113）：确认（OptionalGate）后——无
                // 标记 → 牌堆顶取 max(1, maxHp-体力) 张存标记；有标记 →
                // 收回入手并清空。
                HostStage::Act => {
                    if ctx.core.player(self.owner).zhucheng.is_empty() {
                        let n = 1.max(
                            ctx.core.player(self.owner).max_hp as i32
                                - ctx.core.player(self.owner).hp,
                        ) as usize;
                        for _ in 0..n {
                            if ctx.core.deck.is_empty() {
                                break;
                            }
                            let c = ctx.core.deck.remove(ctx.core.deck.len() - 1);
                            ctx.core.player_mut(self.owner).zhucheng.push(c);
                        }
                    } else {
                        let cards = std::mem::take(&mut ctx.core.player_mut(self.owner).zhucheng);
                        ctx.core.player_mut(self.owner).hand.extend(cards);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianChezhen => {
                // 车阵（玩点论杀/田豫，规则 v115）：锁定——持有者装备区有牌
                // 时其到他人距离 -1；目标装备区为空时其被算到 +1。上游仅
                // mod.globalFrom/globalTo；效果在距离计算（state.rs 700 行），
                // 宿主无机器步骤。
                Flow::Done
            }
            HostBehavior::WudianJuedao => {
                // 绝道（U07 批六 v116）：出牌阶段横置（自连环，已横置
                // 不可发动=activate 层互斥）；横置距离双向 +1 由
                // computed_distance 直读 linked（无机器步）。
                if !ctx.core.player(self.owner).linked {
                    ctx.core.player_mut(self.owner).linked = true;
                    ctx.log(LogEvent::Linked {
                        player: self.owner,
                        linked: true,
                    });
                }
                Flow::Done
            }
            HostBehavior::WudianGeju => {
                // 割据（U07 批七 v117）：摸「攻击范围外其他势力种数」张
                // （势力=effective_faction；unknown/无主将排除；杀范围内
                // 他势力不计入——上游 geju 同口径）
                let mut factions: Vec<u8> = Vec::new();
                for q in ctx.core.alive_players() {
                    if q == self.owner {
                        continue;
                    }
                    if let Some(f) = ctx.core.effective_faction(ctx.catalog, q) {
                        let fb = f as u8;
                        if !factions.contains(&fb) {
                            factions.push(fb);
                        }
                    }
                }
                let range = ctx.core.attack_range(ctx.catalog, self.owner);
                for q in ctx.core.alive_players() {
                    if q == self.owner {
                        continue;
                    }
                    let in_range = ctx.core.computed_distance(ctx.catalog, q, self.owner) <= range;
                    if in_range {
                        if let Some(f) = ctx.core.effective_faction(ctx.catalog, q) {
                            factions.retain(|fb| *fb != f as u8);
                        }
                    }
                }
                if !factions.is_empty() {
                    ctx.draw_cards(self.owner, factions.len());
                }
                Flow::Done
            }
            HostBehavior::WudianTuji => {
                // 突击（ow/士兵76，规则 v118）：锁定——持有者回合内每已使用
                // 一张牌，其进攻距离 -1。上游仅 mod.globalFrom；效果在距离
                // 计算（state.rs 663 行），宿主无机器步骤。
                Flow::Done
            }
            HostBehavior::WudianGuangshuDiamond => {
                // ♦光书：造成伤害后摸 2 张牌（一次性移除实例）。
                ctx.draw_cards(self.owner, 2);
                if let Some(def) = ctx.catalog.skill_by_name(WUDIAN_GUANGSHU_DIAMOND_KEY) {
                    ctx.core.skills.remove(&(self.owner, def));
                }
                Flow::Done
            }

            HostBehavior::WudianGuangshuClub => {
                // ♣光井：mod 钩在杀门控（下阶段结束移除——PhaseEnd 触发臂）
                let self_def = ctx
                    .catalog
                    .skill_by_name(WUDIAN_GUANGSHU_CLUB_KEY)
                    .expect("光井在目录");
                ctx.core.skills.remove(&(self.owner, self_def));
                Flow::Done
            }
            HostBehavior::WudianGuangshuSpade => {
                // 光塔·♠（ow，规则 v119）：锁定——持有者于下一个阶段结束时受
                // 1 点无来源雷伤并移除本子技。上游 guangshu_spade trigger
                // phaseEnd；效果在 commit_phase 的阶段结束直推（kernel.rs 768
                // 行），宿主无机器步骤（TriggerMeta 仍注册 PhaseEnd 供过滤）。
                Flow::Done
            }
            HostBehavior::WudianDuwen => {
                // 毒吻（ow/黑白合，规则 v122）：锁定——造成伤害时若来源与受
                // 伤害角色手牌数相等，伤害 +1。上游 duwen damageBegin；效果
                // 在 commit_damage（kernel.rs 167 行），宿主无机器步骤。
                Flow::Done
            }
            HostBehavior::WudianBaoleiSwitch => match self.stage {
                // 堡垒/侦察切换（U05 ow，规则 v124）：三态翻转——0→1
                //（哨卫）、1→2（侦察）、2→1（回哨卫；上游双技各 phaseEnd
                // 确认切换的互斥语义合并为一个确认式翻转）。
                HostStage::Act => {
                    let cur = ctx.core.player(self.owner).baolei_mode;
                    let next = match cur {
                        2 => 1,
                        _ => cur + 1,
                    };
                    ctx.core.player_mut(self.owner).baolei_mode = next;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianFeiren => {
                // 飞刃（ow/源氏，规则 v129）：锁定——黑桃杀伤害 +1；杀无视
                // 距离；梅花杀可多指定一个目标。上游 feiren damageBegin 与
                // mod.targetInRange/selectTarget；效果在 commit_damage
                // （kernel.rs 121 行）与出杀目标数/距离校验（engine.rs 2377、
                // 2447 行），宿主无机器步骤。
                Flow::Done
            }
            HostBehavior::WudianShenqiang => match self.stage {
                // 神枪（U05 ow 麦克雷，规则 v132）：杀使用计数回退——
                // 本回合可再出一张杀（上游 getStat().card.sha-- 同口径；
                // 地图效果加成杀的 land_sha_used 不回退，登记）。
                HostStage::Act => {
                    ctx.core.turn.sha_used_play_phase = false;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianShihuo => match self.stage {
                // 拾火（U05 ow 狂鼠，规则 v133）：摸一张牌（每次火伤各
                // 触发；铁索传导的独立火伤事件照常）。
                HostStage::Act => {
                    ctx.draw_cards(self.owner, 1);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianMengji => match self.stage {
                // 猛击（U05 ow 莱因哈特，规则 v134）：伤害 +1（前置窗口
                // 直写帧量——伊籍改目标同机制；上游 trigger.num++ 同口径）。
                HostStage::Act => {
                    if !ctx.modify_damage_amount(1) { return Flow::Done; }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianOwliyong => match self.stage {
                // 应用（U05 ow 查莉娅，规则 v138）：摸牌阶段额外摸上回合
                // 屏障次数张。
                HostStage::Act => {
                    let n = ctx.core.player(self.owner).pingzhang_count;
                    if n > 0 {
                        ctx.draw_cards(self.owner, n as usize);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianYinshen => match self.stage {
                // 隐匿（U05 ow 黑影，规则 v139）：置潜行（不能成为其他角
                // 色的卡牌目标；自己回合判定阶段开始解除）。
                HostStage::Act => {
                    ctx.core.player_mut(self.owner).yinshen_active = true;
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::WudianXiandan => {
                // 霰弹（ow/死神，规则 v140）：出杀时可弃一张 he——红牌使该杀
                // 不可闪避，黑牌使该杀伤害 +1。上游 shaBegin direct 与
                // xiandan2 damageBegin；效果在 ShaResolve 内联询问（kernel.rs
                // 3152 行）与黑牌加伤折算（kernel.rs 2222 行），宿主无机器
                // 步骤。
                Flow::Done
            }
            HostBehavior::WudianJijia => {
                // 机甲（ow/D.Va，规则 v141）：初始 4 点机甲血，受伤 1:1 抵扣；
                // 机甲血 >0 时手牌上限 +4。上游 jijia changeHp 拦截与
                // mod.maxHandcard；效果在 commit_damage（kernel.rs 197 行）与
                // 弃牌阶段手牌上限（kernel.rs 2006 行），宿主无机器步骤。
                Flow::Done
            }
            HostBehavior::WudianLongxiang => {
                // 龙翔（xianjian/温慧，规则 v146）：你使用杀时可令目标随机弃至
                // 与你手牌数相同。上游 shaBegin（有 check，非强制）；原生在
                // ShaResolve 内联、不询问（v146 已登记口径，kernel.rs 3047
                // 行），宿主无机器步骤。
                Flow::Done
            }
            HostBehavior::WudianLueying => {
                // 掠影（xianjian/慕长兰，规则 v154）：你使用杀时获得目标 he 随机
                // 一张；若另有其他有牌角色，目标另选一人供你随机弃其一张。
                // 上游 shaBegin usable:1（可选发动）；效果在 ShaResolve 内联
                // （kernel.rs 3087 行），宿主无机器步骤。差异：上游要求另有
                // 其他有牌角色且可选发动，原生自动获得一张——登记见 T11。
                Flow::Done
            }
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
