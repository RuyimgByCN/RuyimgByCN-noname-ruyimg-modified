use super::*;

impl crate::catalog::SkillMachine for ShaResolveMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> crate::snap::MachineSnapshot {
        crate::snap::MachineSnapshot::Sha {
            card: self.card,
            source: self.source,
            target: self.target,
            nature: self.nature,
            asked: self.asked,
            armor_judged: self.armor_judged,
            dodges_used: self.dodges_used,
            tieji_red: self.tieji_red,
            liuli_done: self.liuli_done,
            liuli_stage: self.liuli_stage,
            liuli_card: self.liuli_card.map(|c| c as u32),
            hujia_queue: self.hujia_queue.clone(),
            hujia_holder: self.hujia_holder,
            jiu_bonus: self.jiu_bonus,
            asking_qilin: self.asking_qilin,
            qilin_horse: self.qilin_horse,
            asking_hanbing: self.asking_hanbing,
            asking_fengxueren: self.asking_fengxueren,
            asking_guanshi: self.asking_guanshi,
            guanshi_cards: self.guanshi_cards.iter().copied().collect(),
            asking_qinglong: self.asking_qinglong,
            asking_cixiong: self.asking_cixiong,
            cixiong_done: self.cixiong_done,
            extra_targets: self
                .extra_targets
                .iter()
                .copied()
                .map(|c| c as u32)
                .collect(),
            hanbing_pending: self.hanbing_pending,
            asking_xiandan: self.asking_xiandan,
            xiandan_done: self.xiandan_done,
            xiandan_bonus: self.xiandan_bonus,
            asking_lueying: self.asking_lueying,
            lueying_victim: self.lueying_victim.map(|c| c as u32),
            lueying_declined: self.lueying_declined,
            asking_fuchou: self.asking_fuchou,
            fuchou_card: self.fuchou_card.map(|c| c as u32),
            asking_sadeng: self.asking_sadeng,
            asking_sadeng_shan: self.asking_sadeng_shan,
            shoulie_direct: self.shoulie_direct,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if self.has_sha_trigger_park(ctx) {
            return Flow::Yield;
        }
        if let Some(flow) = self.consume_u01_pending_dodge(ctx) {
            return flow;
        }
        if let Some(flow) = self.resume_pending_answer(ctx) {
            return flow;
        }
        if let Some(flow) = self.handle_u01_respond_park(ctx) {
            return flow;
        }
        // 青釭剑（P2/N08 批③-b）：锁定——无视目标防具（来源装备青釭时
        // 八卦阵判定与仁王盾免疫均不生效）。铡刀（Boss 阎罗王，规则
        // v271）：上游 inherit qinggang_skill——来源持有者即穿透。
        let qinggang_pierce = self.qinggang_pierces(ctx);
        // 八卦阵：受击者装备防具时，出闪询问前先做防具判定（每杀至多一次）
        // ——红桃视为已出闪（伤害免除），否则照常进入出闪询问。
        // 焚天（Boss 朱雀，规则 v236）：红色杀 directHit 无视防具
        //（上游 boss_fentian2 directHit 全场同口径——原生防具=八卦阵）
        let fentian_direct_hit = {
            let red = self.card.is_some_and(|c| {
                matches!(
                    ctx.catalog.resolved_card_suit(ctx.core, c),
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                )
            });
            red && ctx
                .catalog
                .skill_by_name(crate::vocabulary::names::BOSS_FENTIAN)
                .map(|def| ctx.core.has_skill(self.source, def))
                .unwrap_or(false)
        };
        // 破月 poyue2（拓跋玉儿，swd 武将面批 28）：红杀不可被闪避
        //（forced directHit；跳过防具判定与出闪询问）
        // 狩猎应答消费（哨兵在位=刚恢复；asked 门外的首个到达点）：
        // 弃一张→置不可闪避旗；消费后重置 asked 重入门（防应答被闪窗
        // 误读——青龙再杀同款 Yield 重入）
        if self.hujia_holder == Some(self.source) && !self.shoulie_direct {
            self.hujia_holder = None;
            if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                if ctx.core.player(self.source).hand.contains(&c) {
                    ctx.discard_from_hand(self.source, c);
                    self.shoulie_direct = true;
                }
            }
            self.asked = false;
            return Flow::Yield;
        }
        if self.take_sha_end_settled(ctx) {
            return self.advance_target(ctx);
        }
        if let Some(is_damage) = self.take_sha_pending_end(ctx) {
            return self.finish_target_sha_end(ctx, is_damage);
        }
        if ctx
            .core
            .vm_aux
            .flags
            .remove(&(self.target, "l10_reguhuo_sha_post_dodge".to_string()))
            .is_some()
        {
            return self.post_dodge_step(ctx);
        }
        if ctx
            .core
            .vm_aux
            .flags
            .remove(&(self.target, "l10_reguhuo_sha_resume".to_string()))
            .is_some()
        {
            if let Some(shan) = (ctx.catalog.hooks.reguhuo_consume_resp)(ctx.core, self.target) {
                return self.resolve_shan_card(ctx, shan);
            }
            return self.ask_respond_shan_or_hit(ctx, false);
        }
        if !self.asked && !self.armor_judged && self.hujia_queue.is_none() {
            if let Some(fid) = self.current_sha_frame_id(ctx) {
                if ctx
                    .core
                    .vm_aux
                    .flags
                    .remove(&(self.target, format!("sha_cancelled_target:{fid}")))
                    .is_some()
                {
                    return self.advance_target(ctx);
                }
            }
            if let Some(flow) = self.check_sha_begin_hook(ctx, true) {
                return flow;
            }
            if let Some(fid) = self.current_sha_frame_id(ctx) {
                if ctx
                    .core
                    .vm_aux
                    .flags
                    .remove(&(self.target, format!("sha_cancelled_target:{fid}")))
                    .is_some()
                {
                    return self.advance_target(ctx);
                }
            }
            if ctx
                .catalog
                .skill_by_name("cike")
                .is_some_and(|d| ctx.core.has_skill(self.source, d))
            {
                if self.renwang_blocks(ctx, qinggang_pierce) {
                    return self.advance_target(ctx);
                }
                if let Some(flow) = self.check_sha_begin_hook(ctx, false) {
                    return flow;
                }
            } else if !self.renwang_blocks(ctx, qinggang_pierce) {
                let fid = self.current_sha_frame_id(ctx).unwrap_or(0);
                let ow_ran = ctx.core.vm_aux.flags.contains_key(&(
                    self.source,
                    format!("sha_ow_begin_done:{fid}:{}", self.target),
                ));
                if !ow_ran {
                    if let Some(flow) = self.check_sha_begin_hook(ctx, false) {
                        return flow;
                    }
                }
            }
        }
        // 狩猎（swd deferred 消解批 95）：弃牌确认后本杀不可闪避（整杀
        // 有效=机器旗标）
        let shoulie_hit = self.shoulie_direct;
        // 灵矢（swd 计亭，swd deferred 消解批 113）：锁定——装备区有武
        // 器牌或防具牌时【杀】不可闪避（上游 lingshi_hit shaBegin forced
        // directHit 同口径；跳过防具判定与出闪询问）
        let lingshi_direct_hit = ctx
            .catalog
            .skill_by_name("lingshi")
            .map(|def| ctx.core.has_skill(self.source, def))
            .unwrap_or(false)
            && ctx.core.player(self.source).equips.iter().any(|c| {
                matches!(
                    ctx.catalog.card_slot_of(ctx.core, *c),
                    Some(crate::catalog::EquipSlot::Weapon)
                        | Some(crate::catalog::EquipSlot::Armor)
                )
            });
        // 诈降（L27 批 4，`sbzhaxiang`）：本阶段用牌数 ≤ 已损失体力时，所用
        // 【杀】不可被闪避（上游 useCard forced directHit 全场同口径；原生
        // 直伤面仅【杀】有闪避窗口）
        // 反击·红杀直伤（U08 批 1，`yxs_fanji2`）：持技者使用的**红色**【杀】
        // 不可被闪避（上游 yxs_fanji2 shaBegin → trigger.directHit 同口径）
        let yxs_fanji2_direct_hit = {
            let red = self.card.is_some_and(|c| {
                matches!(
                    ctx.catalog.resolved_card_suit(ctx.core, c),
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                )
            });
            red && ctx
                .catalog
                .skill_by_name(crate::vocabulary::host::U08_YXS_FANJI2_KEY)
                .map(|def| ctx.core.has_skill(self.source, def))
                .unwrap_or(false)
        };
        let sbzhaxiang_direct_hit = ctx
            .catalog
            .skill_by_name(crate::vocabulary::host::L27_SBZHAXIANG_KEY)
            .map(|def| ctx.core.has_skill(self.source, def))
            .unwrap_or(false)
            && {
                let lost = (ctx.core.player(self.source).max_hp as i64
                    - ctx.core.player(self.source).hp as i64)
                    .max(0) as u32;
                ctx.core.turn.cards_used_play_phase <= lost
            };
        let poyue_direct_hit = {
            let red = self.card.is_some_and(|c| {
                matches!(
                    ctx.catalog.resolved_card_suit(ctx.core, c),
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                )
            });
            red && (ctx
                    .catalog
                    .skill_by_name("poyue")
                    .map(|def| ctx.core.has_skill(self.source, def))
                    .unwrap_or(false)
                    // 连舞（尚章，swd 武将面批㉒）：锁定——红杀不可被闪避
                    //（上游 shaBegin forced directHit 同口径）
                    || ctx
                        .catalog
                        .skill_by_name("lianwu")
                        .map(|def| ctx.core.has_skill(self.source, def))
                        .unwrap_or(false))
        };
        if !self.armor_judged {
            self.armor_judged = true;
            let guilong_direct_hit =
                crate::rvm_bridge::modifiers::d34_l28_mods::is_guilong_red_sha(
                    ctx.core,
                    ctx.catalog,
                    self.source,
                    self.card,
                );
            let jianwu_direct_hit = ctx
                .catalog
                .skill_by_name("jianwu")
                .is_some_and(|d| {
                    ctx.core.has_skill(self.source, d)
                        && !ctx.catalog.is_tieji_sealed(ctx.core, self.source)
                })
                && ctx.core.computed_distance(ctx.catalog, self.target, self.source)
                    > ctx.core.attack_range(ctx.catalog, self.target);
            let bfengshi_direct = self.current_sha_frame_id(ctx).is_some_and(|fid| {
                ctx.core
                    .vm_aux
                    .flags
                    .get(&(
                        self.source,
                        format!("sha_ow_bfengshi_direct:{fid}:{}", self.target),
                    ))
                    .copied()
                    .unwrap_or(0)
                    != 0
            });
            let can_bagua = !qinggang_pierce
                && !fentian_direct_hit
                && !yxs_fanji2_direct_hit
                && !sbzhaxiang_direct_hit
                && !poyue_direct_hit
                && !shoulie_hit
                && !lingshi_direct_hit
                && !guilong_direct_hit
                && !jianwu_direct_hit
                && !bfengshi_direct
                && !self.is_sha_direct_hit(ctx);
            if let Some(flow) = self.try_bagua_dodge(ctx, can_bagua) {
                return flow;
            }
        }
        if let Some(flow) = self.step_liuli(ctx) {
            return flow;
        }
        // 护驾（曹操主公技，ADR 0776 重构对齐上游）：目标需出闪时，
        // 先按座位顺序逐人询问魏势力角色「是否替其打出一张闪」——
        // 不以暗手牌预筛（询问资格公开、应答私密，不向攻击者泄漏
        // 谁持闪）；拒绝继续问下一位，任一人确认即代出一张闪
        // （计入无双需求），全部拒绝后回到目标本人询问。
        // 燎原（swd 批 145，规则 v319）：额外闪需求并入总需求
        let needed = self.needed_shan_count(ctx);
        if self.dodges_used >= needed {
            // 燎原结束清零（上游 liaoyuan3 shaEnd 同口径）
            ctx.core.turn.liaoyuan_count = 0;
            return self.finish_target_sha_end(ctx, false);
        }
        if self.hujia_active(ctx) {
            // 本响应窗口首次进入：构建逐人询问队列
            if self.hujia_queue.is_none() {
                self.hujia_queue = Some(self.build_hujia_queue(ctx));
            }
            // 消费当前被询问者应答（若有）
            if let Some(holder) = self.hujia_holder.take() {
                // v204 对齐上游 chooseToRespond：持牌者自选一张闪打出
                //（含倾国转化：黑手牌可代出）；取消/空选 = 拒绝，问下一位
                if let Some(ChoiceOption::Card(shan)) = ctx.answer().first().copied() {
                    if ctx.core.player(holder).hand.contains(&shan) {
                        ctx.discard_from_hand(holder, shan);
                        super::respond::note_longdan_sha_source(ctx, holder, self.source);
                        crate::kernel::emit_card_responded_as(ctx, holder, shan, CardKind::Shan);
                        self.dodges_used += 1;
                        if self.dodges_used >= needed {
                            // Upstream `hujia` (`standard/skill.js:287-305`): helper's
                            // `CardResponded` triggers (`dddfushi`, `tuqiang`, etc.) must settle
                            // before `ShaDodged` (`qianglue`, `shaEnd`).
                            if let Some(fid) = self.current_sha_frame_id(ctx) {
                                ctx.core.vm_aux.flags.insert(
                                    (self.target, format!("sha_u01_hujia_dodged:{fid}")),
                                    1,
                                );
                                return Flow::Yield;
                            }
                            ctx.core.turn.liaoyuan_count = 0;
                            return self.after_sha_dodged(ctx);
                        }
                        // 无双还需第二张：新响应窗口——重开逐人询问
                        //（含目标本人的再次询问）
                        self.hujia_queue = Some(self.build_hujia_queue(ctx));
                        self.asked = false;
                        return Flow::Yield;
                    }
                    // 请求期间牌已不在手（并发消耗防御）：视为拒绝
                }
            }
            // 逐人询问下一位（Pass 后继续）
            let queue = self.hujia_queue.as_mut().expect("queue built above");
            if let Some(pos) = queue.iter().position(|p| ctx.core.player(*p).alive) {
                let next = queue.remove(pos);
                self.hujia_holder = Some(next);
                // 候选=自己手牌中的真闪 + 倾国转化（黑手牌）；不含他人牌
                let qingguo_next = ctx
                    .catalog
                    .skill_by_name(crate::vocabulary::names::QING_GUO)
                    .map(|d| ctx.core.has_skill(next, d))
                    .unwrap_or(false);
                let rw_lanyin_next = crate::rvm_bridge::modifiers::d34_l28_mods::has_rewritten_lanyinjia(
                    ctx.core,
                    ctx.catalog,
                    next,
                );
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .player(next)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| {
                        let kind = ctx.catalog.resolved_card_kind(ctx.core, *c);
                        !crate::rvm_bridge::modifiers::d34_flags::is_ducai3_forbidden(
                            ctx.core,
                            ctx.catalog,
                            next,
                            *c,
                        ) && (rw_lanyin_next
                            || kind == CardKind::Shan
                            || (qingguo_next
                                && matches!(
                                    ctx.catalog.resolved_card_suit(ctx.core, *c),
                                    crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                                )))
                    })
                    .map(|c| ChoiceOption::Card(c))
                    .collect();
                return Flow::WaitChoice {
                    observer: next,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "hujia_respond",
                        options,
                        min: 0,
                        max: 1,
                        allow_cancel: false, // 空选=拒绝本位继续问下一位（非取消）
                    },
                };
            }
            // 队列问完（全部拒绝或无人合格）：Some(空) 固定，
            // 进入目标本人询问；后续恢复重入不再重建。
        }
        if !self.asked {
            self.asked = true;
            if self.virtual_sha_canceled(ctx) {
                return self.advance_target(ctx);
            }
            // 天弦（U05 xianjian 柳梦璃，规则 v184）：锁定——杀对每目标
            // 结算前目标判定（上游 useCardToBefore 同窗口，priority 5.5
            // 先于结算技），黑色则取消对该目标的结算（不进入闪询问；每
            // 目标一次=asked 每目标复位）。**登记差异**：同铁骑内联判定
            // 口径=确定性判定，无鬼道/天妒改判窗。
            if let Some(flow) = self.resolve_tianxian(ctx) {
                return flow;
            }
            // 铁骑（马超）：锁定式判定——判定牌为红色则目标不能出闪响应
            if self.tieji_red.is_none() {
                if let Some(flow) = self.resolve_tieji(ctx) {
                    return flow;
                }
            }
            // 仁王盾（P2/N08 批③-a）：锁定——黑色【杀】对装备者无效
            //（先于八卦阵判定；红杀/无盾照常进入判定与出闪询问）。
            // 铁面（Boss 阎罗王，规则 v271）：防具区没有牌时视为装备
            // 仁王盾（防具槽空=不被青釭/铡刀外的牌区分；青釭穿透已在
            // !qinggang_pierce 门内天然覆盖铁面）。
            if self.renwang_blocks(ctx, qinggang_pierce) {
                return self.advance_target(ctx);
            }
            // 堡垒哨卫（U05 ow，规则 v125）：哨卫模式的杀不可闪避/无视
            // 防具——直接结算伤害（上游 bshaowei_sha directHit 同口径）
            if self.baolei_sentinel(ctx) && ctx.core.player(self.target).alive {
                return self.emit_sha_damage(ctx);
            }
            // 霰弹（U05 ow 死神，规则 v140）：shaBegin direct——来源可弃
            // 1 张 he：红=直击（跳过闪询问）；黑=本杀伤害 +1；取消=照常
            //（he 空不询问；每杀至多一次——上游 shaBegin 每杀事件同口径）。
            // 龙翔（U05 xianjian 温慧，规则 v146）：shaBegin forced——
            // 目标手牌多于自己时随机弃差值张（上游 randomGets(diff) 同
            // 口径；无询问零字段）。
            {
                let fid = self.current_sha_frame_id(ctx).unwrap_or(0);
                let lx_key = (self.source, format!("sha_longxiang_done:{fid}"));
                let has_longxiang = !ctx.core.vm_aux.flags.contains_key(&lx_key)
                    && ctx
                        .catalog
                        .skill_by_name(crate::vocabulary::host::WUDIAN_LONGXIANG_KEY)
                        .map(|d| ctx.core.has_skill(self.source, d))
                        .unwrap_or(false);
                if has_longxiang
                    && ctx.core.player(self.target).alive
                    && ctx.core.player(self.target).hand.len()
                        > ctx.core.player(self.source).hand.len()
                {
                    ctx.core.vm_aux.flags.insert(lx_key, 1);
                    let diff = ctx.core.player(self.target).hand.len()
                        - ctx.core.player(self.source).hand.len();
                    let hs: Vec<crate::ids::CardId> =
                        ctx.core.player(self.target).hand.iter().copied().collect();
                    let mut pool = hs;
                    for _ in 0..diff {
                        if pool.is_empty() {
                            break;
                        }
                        let k = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                        let dc = pool.swap_remove(k);
                        let t = self.target;
                        ctx.take_owned_card(t, dc);
                        ctx.to_discard(dc);
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![dc],
                        });
                    }
                }
            }
            // 掠影（U05 xianjian 慕长兰，规则 v154；v194 对齐上游）：
            // shaBegin usable:1，非强制——filter 要求目标 he 有牌**且**
            // 另有一名其他有牌角色（缺第三者则整个技能不可发动，连获得
            // 也不触发）。确认后获得目标 he 随机一张；仍有其他有牌者→
            // 目标选人、来源随机弃其一张（lueying_used 确认后消耗，放弃
            // 不计数，同上游 usable 口径）。
            {
                let has_lueying = ctx
                    .catalog
                    .skill_by_name(crate::vocabulary::host::WUDIAN_LUEYING_KEY)
                    .map(|d| ctx.core.has_skill(self.source, d))
                    .unwrap_or(false);
                let t_he = !ctx.core.player(self.target).hand.is_empty()
                    || !ctx.core.player(self.target).equips.is_empty();
                let others: Vec<PlayerId> = ctx
                    .core
                    .alive_players()
                    .filter(|p| {
                        *p != self.source
                            && *p != self.target
                            && (!ctx.core.player(*p).hand.is_empty()
                                || !ctx.core.player(*p).equips.is_empty())
                    })
                    .collect();
                if has_lueying
                    && !ctx.core.turn.lueying_used
                    && !self.lueying_declined
                    && ctx.core.player(self.source).alive
                    && ctx.core.player(self.target).alive
                    && t_he
                    && !others.is_empty()
                {
                    // 确认相：复用 asking_lueying 挂起（恢复侧按应答形状
                    // 区分相——Confirm=确认发动，Player=受害者已选定）
                    self.asking_lueying = true;
                    return Flow::WaitChoice {
                        observer: self.source,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::Confirm,
                            prompt: "lueying_confirm",
                            options: vec![ChoiceOption::Confirm],
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    };
                }
            }
            let has_xiandan = ctx
                .catalog
                .skill_by_name(crate::vocabulary::host::WUDIAN_XIANDAN_KEY)
                .map(|d| {
                    (
                        ctx.core.has_skill(self.source, d),
                        ctx.catalog.skills[d as usize].behavior.as_host_behavior()
                            == Some(crate::vocabulary::host::HostBehavior::U05OwBatchC),
                    )
                });
            if let Some((true, is_ow_v3)) = has_xiandan {
                let p = ctx.core.player(self.source);
                let has_cards = if is_ow_v3 {
                    !p.hand.is_empty()
                } else {
                    !p.hand.is_empty() || !p.equips.is_empty()
                };
                if !self.xiandan_done
                    && ctx.core.player(self.source).alive
                    && ctx.core.player(self.target).alive
                    && has_cards
                {
                    self.xiandan_done = true;
                    self.asking_xiandan = true;
                    let mut options: Vec<ChoiceOption> =
                        p.hand.iter().copied().map(ChoiceOption::Card).collect();
                    if !is_ow_v3 {
                        options.extend(p.equips.iter().copied().map(ChoiceOption::Card));
                    }
                    return Flow::WaitChoice {
                        observer: self.source,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "xiandan_discard",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    };
                }
            }
            // 雌雄双股剑（批③-b 第八把）：杀指定异性目标后（性别有槽取
            // 武将性别、无槽默认男），目标选择：弃一张手牌（自选，逐技
            // 对照批一 v75 对齐上游 chooseToDiscard）或取消令使用者摸 1；
            // 无手牌则仅可令摸（上游同：!hasCards("h") 直接 draw）。
            let has_cixiong_override = ctx
                .catalog
                .skill_by_name("cixiong_skill")
                .is_some_and(|d| ctx.core.has_skill(self.source, d));
            if !self.cixiong_done
                && !self.asking_cixiong
                && !has_cixiong_override
                && !crate::kernel::mbcmfangzhu_baiban(ctx.core, self.source)
                && ctx
                    .core
                    .player(self.source)
                    .equips
                    .iter()
                    .any(|c| ctx.catalog.equip_card_has_kind(ctx.core, *c, CardKind::Cixiong))
                && ctx.core.player(self.target).alive
                && ctx
                    .core
                    .different_gender_from(ctx.catalog, self.source, self.target)
            {
                self.cixiong_done = true;
                let need = crate::kernel::sha::resume::cixiong_need(ctx, self.source);
                if ctx.core.player(self.target).hand.len() >= need {
                    self.asking_cixiong = true;
                    // 上游 chooseToDiscard：目标自选（拂尘荡魔时需 2 张）手牌，
                    // 空应答（取消）= 令使用者摸 1（逐技对照批一 v75）。
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.target)
                        .hand
                        .iter()
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    return Flow::WaitChoice {
                        observer: self.target,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "cixiong_discard_choose",
                            options,
                            min: if need > 1 { need } else { 0 },
                            max: need,
                            allow_cancel: true,
                        },
                    };
                }
                // 无手牌：唯一可选 = 令使用者摸 1（R05：真实 R 帧）。
                ctx.start_draw(self.source, 1);
            }
            // 狩猎（swd deferred 消解批 子达书）：可选——弃一张手牌令此
            // 杀不可闪避（整杀有效；每杀一次=!asked 门+机器旗标）
            if !self.shoulie_direct
                && ctx
                    .catalog
                    .skill_by_name("shoulie")
                    .map(|d| ctx.core.has_skill(self.source, d))
                    .unwrap_or(false)
                && ctx.core.player(self.source).alive
                && !ctx.core.player(self.source).hand.is_empty()
            {
                self.shoulie_direct = false; // 待应答后置位
                self.hujia_holder = Some(self.source); // 哨兵=shoulie 待应答
                let opts: Vec<ChoiceOption> = ctx
                    .core
                    .player(self.source)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                return Flow::WaitChoice {
                    observer: self.source,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "shoulie_discard",
                        options: opts,
                        min: 0,
                        max: 1,
                        allow_cancel: true,
                    },
                };
            }
            // 渔人（喀玛，swd 武将面批㉙）：锁定——杀指定目标后判定（内联
            // 确定性口径同铁骑，登记差异无改判窗）：黑→弃目标 he 随机一张
            //（上游 discardPlayerCard forced random 同口径）；红→摸 1（登记
            // 差异：上游收回此杀实体——实体回流与在结机器持有冲突改摸 1）。
            // 每杀一次=!asked 门内。
            if let Some(flow) = self.resolve_yueren(ctx) {
                return flow;
            }
            // 玲月（司徒强，swd 武将面批㉗）：锁定——杀指定目标后目标弃
            // 置一张牌（he 确定性首张，登记差异上游目标自选 chooseToDiscard
            // forced；每杀每目标一次=!asked 门内）
            {
                let has_lingyue = ctx
                    .catalog
                    .skill_by_name("lingyue")
                    .map(|d| ctx.core.has_skill(self.source, d))
                    .unwrap_or(false);
                if has_lingyue && ctx.core.player(self.target).alive {
                    let hand_first = ctx.core.player(self.target).hand.first().copied();
                    let eq_first = ctx.core.player(self.target).equips.first().copied();
                    if let Some(c) = hand_first.or(eq_first) {
                        let in_hand = ctx.core.player(self.target).hand.contains(&c);
                        if in_hand {
                            ctx.discard_from_hand(self.target, c);
                        } else {
                            ctx.core.player_mut(self.target).equips.retain(|x| *x != c);
                            if ctx.catalog.resolved_card_kind(ctx.core, c).is_equip() {
                                crate::kernel::revoke_equip_skills(
                                    ctx.core,
                                    ctx.catalog,
                                    self.target,
                                    c,
                                );
                            }
                            ctx.to_discard(c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: self.target,
                                cards: vec![c],
                            });
                        }
                    }
                }
            }
            let jianwu_hit = ctx
                .catalog
                .skill_by_name("jianwu")
                .is_some_and(|d| {
                    ctx.core.has_skill(self.source, d)
                        && !ctx.catalog.is_tieji_sealed(ctx.core, self.source)
                })
                && ctx.core.computed_distance(ctx.catalog, self.target, self.source)
                    > ctx.core.attack_range(ctx.catalog, self.target);
            let bfengshi_direct = self.current_sha_frame_id(ctx).is_some_and(|fid| {
                ctx.core
                    .vm_aux
                    .flags
                    .get(&(
                        self.source,
                        format!("sha_ow_bfengshi_direct:{fid}:{}", self.target),
                    ))
                    .copied()
                    .unwrap_or(0)
                    != 0
            });
            return self.ask_respond_shan_or_hit(
                ctx,
                poyue_direct_hit
                    || shoulie_hit
                    || lingshi_direct_hit
                    || sbzhaxiang_direct_hit
                    || yxs_fanji2_direct_hit
                    || jianwu_hit
                    || bfengshi_direct
                    || self.is_sha_direct_hit(ctx),
            );
        }
        // 已恢复：读取应答
        let answer = ctx.answer_first();
        if let ChoiceOption::Skill(rdef) = answer {
            if !ctx.catalog.blocks_card_use_and_save(ctx.core, self.target)
                && super::super::host_response_skills(ctx.core, ctx.catalog, self.target, 1)
                    .contains(&rdef)
            {
                self.asked = false;
                ctx.core
                    .vm_aux
                    .flags
                    .insert((self.target, "l10_reguhuo_sha_resume".to_string()), 1);
                (ctx.catalog.hooks.reguhuo_spawn_response)(
                    ctx,
                    self.target,
                    1,
                    0,
                    rdef,
                    self.source,
                );
                return Flow::Yield;
            }
            if let Some(flow) = self.try_handle_u01_respond_skill(ctx, rdef) {
                return flow;
            }
        }
        if let ChoiceOption::Card(shan) = answer {
            return self.resolve_shan_card(ctx, shan);
        } else if ctx.core.player(self.target).alive {
            if self.hanbing_can_fire(ctx) {
                self.asking_hanbing = true;
                return Flow::WaitChoice {
                    observer: self.source,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::Confirm,
                        prompt: "hanbing_prevent_confirm",
                        options: vec![ChoiceOption::Confirm],
                        min: 0,
                        max: 1,
                        allow_cancel: true,
                    },
                };
            }
            let horses = self.qilin_horses(ctx);
            if !horses.is_empty() {
                self.asking_qilin = true;
                return Flow::WaitChoice {
                    observer: self.source,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "qilin_discard_horse",
                        options: horses,
                        min: 0,
                        max: 1,
                        allow_cancel: true,
                    },
                };
            }
            return self.emit_sha_damage(ctx);
        }
        // 燎原（swd 批 145）：仍有未满足的额外闪需求 → 未闪避部分视为命中
        if self.dodges_used < ctx.core.turn.liaoyuan_count {
            return self.emit_sha_damage(ctx);
        }
        // 杀被闪避（swd 批 141，规则 v316 同批）：发射 ShaDodged（上游
        // shaMiss 触发窗；青龙/贯石斧等窗口之后——登记差异：窗口次序）
        self.after_sha_dodged(ctx)
    }
}
