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
        if let Some(flow) = self.resume_pending_answer(ctx) {
            return flow;
        }
        // 青釭剑（P2/N08 批③-b）：锁定——无视目标防具（来源装备青釭时
        // 八卦阵判定与仁王盾免疫均不生效）。铡刀（Boss 阎罗王，规则
        // v271）：上游 inherit qinggang_skill——来源持有者即穿透。
        let qinggang_pierce = ctx
            .core
            .player(self.source)
            .equips
            .iter()
            .any(|c| ctx.catalog.card_kind(*c) == CardKind::Qinggang)
            || ctx
                .catalog
                .skill_by_name(crate::vocabulary::names::BOSS_ZHADAO)
                .map(|def| ctx.core.has_skill(self.source, def))
                .unwrap_or(false);
        // 八卦阵：受击者装备防具时，出闪询问前先做防具判定（每杀至多一次）
        // ——红桃视为已出闪（伤害免除），否则照常进入出闪询问。
        // 焚天（Boss 朱雀，规则 v236）：红色杀 directHit 无视防具
        //（上游 boss_fentian2 directHit 全场同口径——原生防具=八卦阵）
        let fentian_direct_hit = {
            let red = matches!(
                ctx.catalog.cards[self.card as usize].suit,
                crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
            );
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
        // 烈弓（L20 批 3，`fz_liegong`）：持技者对目标出【杀】时，若目标手牌数
        // ≥ 自己体力 或 ≤ 自己攻击范围 → 该目标不可被闪避（上游
        // useCardToPlayered → directHit.push(target) 同口径）
        let fz_liegong_direct_hit = ctx
            .catalog
            .skill_by_name(crate::vocabulary::host::L20_FZ_LIEGONG_KEY)
            .map(|def| ctx.core.has_skill(self.source, def))
            .unwrap_or(false)
            && {
                let hand = ctx.core.player(self.target).hand.len() as i32;
                let hp = ctx.core.player(self.source).hp;
                let range = ctx.core.attack_range(ctx.catalog, self.source) as i32;
                hand >= hp || hand <= range
            };
        // 反击·红杀直伤（U08 批 1，`yxs_fanji2`）：持技者使用的**红色**【杀】
        // 不可被闪避（上游 yxs_fanji2 shaBegin → trigger.directHit 同口径）
        let yxs_fanji2_direct_hit = {
            let red = matches!(
                ctx.catalog.cards[self.card as usize].suit,
                crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
            );
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
            let red = matches!(
                ctx.catalog.cards[self.card as usize].suit,
                crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
            );
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
            let has_armor = !qinggang_pierce
                && !fentian_direct_hit
                && !fz_liegong_direct_hit
                && !yxs_fanji2_direct_hit
                && !sbzhaxiang_direct_hit
                && !poyue_direct_hit
                && !shoulie_hit
                && !lingshi_direct_hit
                && ctx.core.player(self.target).alive
                && ctx
                    .core
                    .player(self.target)
                    .equips
                    .iter()
                    .any(|c| ctx.catalog.card_kind(*c) == CardKind::Armor);
            if has_armor {
                let judged = ctx.draw_judge_card();
                let success = matches!(
                    ctx.catalog.cards[judged as usize].suit,
                    crate::catalog::Suit::Heart
                );
                ctx.log(LogEvent::ArmorJudged {
                    player: self.target,
                    card: judged,
                    success,
                });
                // 判定牌判定后即弃置（八卦阵判定无获得路径）
                ctx.to_discard(judged);
                if success {
                    let card = self.card;
                    ctx.to_discard(card);
                    ctx.core.table.retain(|c| *c != card);
                    return Flow::Done;
                }
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
        let needed: u8 =
            if self.source_has_wushuang(ctx) { 2 } else { 1 } + ctx.core.turn.liaoyuan_count;
        if self.dodges_used >= needed {
            let card = self.card;
            ctx.to_discard(card);
            ctx.core.table.retain(|c| *c != card);
            // 燎原结束清零（上游 liaoyuan3 shaEnd 同口径）
            ctx.core.turn.liaoyuan_count = 0;
            return Flow::Done;
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
                        ctx.emit(EventData::CardResponded {
                            card: shan,
                            player: holder,
                        });
                        self.dodges_used += 1;
                        if self.dodges_used >= needed {
                            let card = self.card;
                            ctx.to_discard(card);
                            ctx.core.table.retain(|c| *c != card);
                            return Flow::Done;
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
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .player(next)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| {
                        let kind = ctx.catalog.card_kind(*c);
                        kind == CardKind::Shan
                            || (qingguo_next
                                && matches!(
                                    ctx.catalog.cards[*c as usize].suit,
                                    crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                                ))
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
            // 天弦（U05 xianjian 柳梦璃，规则 v184）：锁定——杀对每目标
            // 结算前目标判定（上游 useCardToBefore 同窗口，priority 5.5
            // 先于结算技），黑色则取消对该目标的结算（不进入闪询问；每
            // 目标一次=asked 每目标复位）。**登记差异**：同铁骑内联判定
            // 口径=确定性判定，无鬼道/天妒改判窗。
            {
                let tianxian = ctx
                    .catalog
                    .skill_by_name(crate::vocabulary::host::WUDIAN_XJ_TIANXIAN_KEY)
                    .map(|d| ctx.core.has_skill(self.source, d))
                    .unwrap_or(false);
                if tianxian {
                    let judged = ctx.draw_judge_card();
                    let black = matches!(
                        ctx.catalog.cards[judged as usize].suit,
                        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                    );
                    ctx.to_discard(judged);
                    if black {
                        return self.advance_target(ctx);
                    }
                }
            }
            // 铁骑（马超）：锁定式判定——判定牌为红色则目标不能出闪响应
            let tieji = ctx
                .catalog
                .skill_by_name(crate::vocabulary::names::TIE_JI)
                .map(|d| ctx.core.has_skill(self.source, d))
                .unwrap_or(false);
            if tieji && self.tieji_red.is_none() {
                let judged = ctx.draw_judge_card();
                let red = matches!(
                    ctx.catalog.cards[judged as usize].suit,
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                );
                ctx.to_discard(judged);
                self.tieji_red = Some(red);
            }
            // 仁王盾（P2/N08 批③-a）：锁定——黑色【杀】对装备者无效
            //（先于八卦阵判定；红杀/无盾照常进入判定与出闪询问）。
            // 铁面（Boss 阎罗王，规则 v271）：防具区没有牌时视为装备
            // 仁王盾（防具槽空=不被青釭/铡刀外的牌区分；青釭穿透已在
            // !qinggang_pierce 门内天然覆盖铁面）。
            let renwang_blocks = !qinggang_pierce && {
                let black = matches!(
                    ctx.catalog.cards[self.card as usize].suit,
                    crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                );
                black && {
                    let equips = &ctx.core.player(self.target).equips;
                    let has_renwang = equips
                        .iter()
                        .any(|c| ctx.catalog.card_kind(*c) == CardKind::Renwang);
                    let tiemian_active = ctx
                        .catalog
                        .skill_by_name(crate::vocabulary::names::BOSS_TIEMIAN)
                        .map(|def| ctx.core.has_skill(self.target, def))
                        .unwrap_or(false)
                        && !equips.iter().any(|c| {
                            ctx.catalog.card_slot_of(ctx.core, *c)
                                == Some(crate::catalog::EquipSlot::Armor)
                        });
                    has_renwang || tiemian_active
                }
            };
            if renwang_blocks {
                let card = self.card;
                ctx.to_discard(card);
                ctx.core.table.retain(|c| *c != card);
                return Flow::Done;
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
                let has_longxiang = ctx
                    .catalog
                    .skill_by_name(crate::vocabulary::host::WUDIAN_LONGXIANG_KEY)
                    .map(|d| ctx.core.has_skill(self.source, d))
                    .unwrap_or(false);
                if has_longxiang
                    && ctx.core.player(self.target).alive
                    && ctx.core.player(self.target).hand.len()
                        > ctx.core.player(self.source).hand.len()
                {
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
                .map(|d| ctx.core.has_skill(self.source, d))
                .unwrap_or(false);
            if has_xiandan
                && !self.xiandan_done
                && ctx.core.player(self.source).alive
                && ctx.core.player(self.target).alive
                && (!ctx.core.player(self.source).hand.is_empty()
                    || !ctx.core.player(self.source).equips.is_empty())
            {
                self.xiandan_done = true;
                self.asking_xiandan = true;
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .player(self.source)
                    .hand
                    .iter()
                    .copied()
                    .chain(ctx.core.player(self.source).equips.iter().copied())
                    .map(ChoiceOption::Card)
                    .collect();
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
            // 雌雄双股剑（批③-b 第八把）：杀指定异性目标后（性别有槽取
            // 武将性别、无槽默认男），目标选择：弃一张手牌（自选，逐技
            // 对照批一 v75 对齐上游 chooseToDiscard）或取消令使用者摸 1；
            // 无手牌则仅可令摸（上游同：!hasCards("h") 直接 draw）。
            if !self.cixiong_done
                && !self.asking_cixiong
                && ctx
                    .core
                    .player(self.source)
                    .equips
                    .iter()
                    .any(|c| ctx.catalog.card_kind(*c) == CardKind::Cixiong)
                && ctx.core.player(self.target).alive
                && ctx.core.effective_gender(self.source) != ctx.core.effective_gender(self.target)
            {
                self.cixiong_done = true;
                let has_hand = !ctx.core.player(self.target).hand.is_empty();
                if has_hand {
                    self.asking_cixiong = true;
                    // 上游 chooseToDiscard：目标自选一张手牌，空应答（取消）
                    // = 令使用者摸 1（逐技对照批一 v75）。
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
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    };
                }
                // 无手牌：唯一可选 = 令使用者摸 1。
                ctx.draw_cards(self.source, 1);
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
            {
                let has_yueren = ctx
                    .catalog
                    .skill_by_name("yueren")
                    .map(|d| ctx.core.has_skill(self.source, d))
                    .unwrap_or(false);
                if has_yueren && ctx.core.player(self.target).alive {
                    let judged = ctx.draw_judge_card();
                    let black = matches!(
                        ctx.catalog.cards[judged as usize].suit,
                        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                    );
                    ctx.to_discard(judged);
                    if black {
                        let mut pool: Vec<crate::ids::CardId> =
                            ctx.core.player(self.target).hand.iter().copied().collect();
                        pool.extend(ctx.core.player(self.target).equips.iter().copied());
                        if !pool.is_empty() {
                            let j = ctx.core.rng.gen_range(pool.len() as u32) as usize;
                            let c = pool[j];
                            let in_hand = ctx.core.player(self.target).hand.contains(&c);
                            if in_hand {
                                ctx.take_hand_card(self.target, c);
                            } else {
                                ctx.core.player_mut(self.target).equips.retain(|x| *x != c);
                                if ctx.catalog.card_kind(c).is_equip() {
                                    crate::kernel::revoke_equip_skills(
                                        ctx.core,
                                        ctx.catalog,
                                        self.target,
                                        c,
                                    );
                                }
                            }
                            ctx.to_discard(c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: self.target,
                                cards: vec![c],
                            });
                        }
                    } else if ctx.core.player(self.source).alive {
                        ctx.draw_cards(self.source, 1);
                    }
                }
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
                            if ctx.catalog.card_kind(c).is_equip() {
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
            // 龙胆：杀可当闪打出——响应选项并入目标手牌中的杀
            let longdan = ctx
                .catalog
                .skill_by_name(crate::vocabulary::names::LONG_DAN)
                .map(|d| ctx.core.has_skill(self.target, d))
                .unwrap_or(false);
            let tieji_blocks = self.tieji_red == Some(true);
            // 倾国（甄姬，v202 对齐上游 position "hs"）：黑色手牌可当闪打出——并入响应选项
            let qingguo = ctx
                .catalog
                .skill_by_name(crate::vocabulary::names::QING_GUO)
                .map(|d| ctx.core.has_skill(self.target, d))
                .unwrap_or(false);
            // 堡垒哨卫（U05 ow，规则 v125）目标侧：持有者不能闪避杀
            let target_sentinel = ctx
                .catalog
                .skill_by_name(crate::vocabulary::host::WUDIAN_BAOLEI_KEY)
                .map(|def| {
                    ctx.core.has_skill(self.target, def)
                        && ctx.core.player(self.target).baolei_mode == 1
                })
                .unwrap_or(false);
            let mut shans: Vec<ChoiceOption> = ctx
                .core
                .player(self.target)
                .hand
                .iter()
                .filter(|c| {
                    !tieji_blocks && !target_sentinel && {
                        let kind = ctx.catalog.card_kind(**c);
                        kind == CardKind::Shan || (longdan && kind == CardKind::Sha)
                    }
                })
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            // 诱敌深入（G1 玩点论杀）：打出后杀无效，来源受 1 点反噬
            //（登记差异：上游为强制来源连用杀，原生简化为单次反噬）
            for c in ctx.core.player(self.target).hand.iter().copied() {
                if ctx.catalog.card_kind(c) == CardKind::Youdishenru {
                    shans.push(ChoiceOption::Card(c));
                    break;
                }
            }
            if qingguo && !tieji_blocks {
                for c in ctx.core.player(self.target).hand.iter().copied() {
                    let kind = ctx.catalog.card_kind(c);
                    let already = kind == CardKind::Shan || (longdan && kind == CardKind::Sha);
                    let black = matches!(
                        ctx.catalog.cards[c as usize].suit,
                        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                    );
                    if black && !already {
                        shans.push(ChoiceOption::Card(c));
                    }
                }
            }

            // 神弓（swd 批 148）：借用他人防具也视为「可响应」——须在
            // 「无闪直接命中」判定之前纳入
            let shengong_borrow = (ctx.catalog.hooks.borrow_options)(ctx, self.target, true);
            if (shans.is_empty() && shengong_borrow.is_empty())
                || !ctx.core.player(self.target).alive
            {
                // 风雪刃（U05 swd 第十一批，规则 v95）：命中后可选择——
                // 防止伤害，目标翻面并摸一张牌（上游 shaHit → unhurt +
                // turnOver + draw；Confirm 可放弃则照常伤害）
                if !self.asking_fengxueren
                    && ctx.core.player(self.target).alive
                    && !ctx.core.player(self.target).face_down
                    && ctx
                        .core
                        .player(self.source)
                        .equips
                        .iter()
                        .any(|c| ctx.catalog.card_kind(*c) == CardKind::Fengxueren)
                {
                    self.asking_fengxueren = true;
                    return Flow::WaitChoice {
                        observer: self.source,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::Confirm,
                            prompt: "fengxueren_flip_confirm",
                            options: vec![ChoiceOption::Confirm],
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    };
                }
                // 寒冰剑：目标有牌时来源可防止伤害，改为目标自选弃两张
                //（逐技对照批一 v75 对齐上游 discardPlayerCard）。
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
                // 麒麟弓：伤害结算前来源可弃目标一匹坐骑。
                let horses = self.qilin_horses(ctx);
                if !horses.is_empty() && ctx.core.player(self.target).alive {
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
            // 破月 poyue2（拓跋玉儿，swd 武将面批 28）：红杀 directHit
            // ——整段跳过闪询问直接结算伤害
            if poyue_direct_hit
                || shoulie_hit
                || lingshi_direct_hit
                || sbzhaxiang_direct_hit
                || yxs_fanji2_direct_hit
                || fz_liegong_direct_hit
            {
                return self.emit_sha_damage(ctx);
            }
            // 司圃（Boss 应昭，规则 v232）：应昭出牌阶段已用牌数≤2 期间
            // 其他人无法打出牌（上游 cardRespondable 同口径）——出闪选项
            // 仅保留 Pass（目标必受伤害）
            let sipu_ban_respond = ctx
                .catalog
                .skill_by_name(crate::vocabulary::names::BOSS_SIPU)
                .map(|def| {
                    ctx.core.has_skill(self.source, def)
                        && ctx.core.turn.current == self.source
                        && ctx.core.turn.cards_used_play_phase < 3
                })
                .unwrap_or(false);
            let mut options = vec![ChoiceOption::Pass];
            // 冰封·封（swd 批 134）：出牌封锁者不能打出闪（选项为空=必中）
            if !sipu_ban_respond
                && !poyue_direct_hit
                && !shoulie_hit
                && !lingshi_direct_hit
                && !ctx.catalog.blocks_all_cards(ctx.core, self.target)
            {
                options.extend(shans);
                // 神弓（swd 批 148）：借用他人防具作为闪
                options.extend(shengong_borrow);
            }
            return Flow::WaitChoice {
                observer: self.target,
                spec: ChoiceSpec {
                    kind: ChoiceKind::RespondCard,
                    prompt: "respond_shan",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            };
        }
        // 已恢复：读取应答
        let answer = ctx.answer_first();
        // 诱敌深入应答：杀无效（弃置），来源受 1 点反噬伤害
        if let ChoiceOption::Card(yd) = answer {
            if ctx.catalog.card_kind(yd) == CardKind::Youdishenru {
                let in_hand = ctx.core.player(self.target).hand.contains(&yd);
                if in_hand {
                    ctx.discard_from_hand(self.target, yd);
                }
                ctx.emit(EventData::CardResponded {
                    card: yd,
                    player: self.target,
                });
                if ctx.core.player(self.source).alive {
                    ctx.emit(EventData::Damage(damage_data_with(
                        Some(self.target),
                        self.source,
                        1,
                        crate::event::DamageNature::Normal,
                        Some(yd),
                    )));
                }
                let card = self.card;
                ctx.to_discard(card);
                ctx.core.table.retain(|c| *c != card);
                return Flow::Done;
            }
        }
        if let ChoiceOption::Card(shan) = answer {
            // 倾国应答可能来自装备区：按实际所在区移除
            let in_hand = ctx.core.player(self.target).hand.contains(&shan);
            if in_hand {
                ctx.discard_from_hand(self.target, shan);
            } else if ctx.core.player(self.target).equips.contains(&shan) {
                ctx.core
                    .player_mut(self.target)
                    .equips
                    .retain(|x| *x != shan);
                ctx.to_discard(shan);
            } else if let Some(owner) = (ctx.catalog.hooks.borrowed_owner)(ctx, self.target, shan) {
                // 神弓借用（swd 批 148）：从所有者装备区取走并撤技
                ctx.core.player_mut(owner).equips.retain(|x| *x != shan);
                revoke_equip_skills(ctx.core, ctx.catalog, owner, shan);
                ctx.to_discard(shan);
                ctx.log(LogEvent::SkillTriggered {
                    skill: "神弓",
                    owner: self.target,
                });
            } else {
                ctx.to_discard(shan);
            }
            ctx.emit(EventData::CardResponded {
                card: shan,
                player: self.target,
            });
            // 雷击（Boss 青龙，规则 v234）：响应闪后判定雷击（无双第二
            // 闪重入本臂=每张闪各触发一次，上游同口径）
            releiji_on_shan(ctx, self.target);
            self.dodges_used += 1;
            // 无双：还需第二张闪 → 新响应窗口（重开护驾逐人询问 +
            // 目标本人询问；不弃杀、不受伤）
            let needed: u8 =
                if self.source_has_wushuang(ctx) { 2 } else { 1 } + ctx.core.turn.liaoyuan_count;
            if self.dodges_used < needed {
                self.hujia_queue = None;
                self.hujia_holder = None;
                self.asked = false;
                return Flow::Yield;
            }
            // 青龙偃月刀（批③-b 第六把）：杀被闪后，来源可立即对相同
            // 目标再使用一张【杀】（从手牌选择；放弃则不再追问——每杀
            // 至多追问一次，防止连环杀循环）。
            let has_qinglong = ctx
                .core
                .player(self.source)
                .equips
                .iter()
                .any(|c| ctx.catalog.card_kind(*c) == CardKind::Qinglong);
            let hand_shas: Vec<ChoiceOption> = ctx
                .core
                .player(self.source)
                .hand
                .iter()
                .filter(|c| {
                    matches!(
                        ctx.catalog.card_kind(**c),
                        CardKind::Sha | CardKind::FireSha | CardKind::LeiSha
                    )
                })
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            if has_qinglong && !hand_shas.is_empty() && !self.asking_qinglong {
                self.asking_qinglong = true;
                return Flow::WaitChoice {
                    observer: self.source,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "qinglong_replay_sha",
                        options: hand_shas,
                        min: 0,
                        max: 1,
                        allow_cancel: true,
                    },
                };
            }
            // 贯石斧（批③-b 第五把）：杀被闪后，来源可弃两张牌强制伤害
            //（登记差异：可弃牌池不含贯石斧本体；不足两张则无法发动）。
            let has_guanshi = ctx
                .core
                .player(self.source)
                .equips
                .iter()
                .any(|c| ctx.catalog.card_kind(*c) == CardKind::Guanshi);
            if has_guanshi && ctx.core.player(self.source).alive {
                let mut pool: Vec<ChoiceOption> = ctx
                    .core
                    .player(self.source)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                pool.extend(
                    ctx.core
                        .player(self.source)
                        .equips
                        .iter()
                        .filter(|c| ctx.catalog.card_kind(**c) != CardKind::Guanshi)
                        .copied()
                        .map(ChoiceOption::Card),
                );
                if pool.len() >= 2 {
                    self.asking_guanshi = true;
                    return Flow::WaitChoice {
                        observer: self.source,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "guanshi_force_damage",
                            options: pool,
                            min: 2,
                            max: 2,
                            allow_cancel: true,
                        },
                    };
                }
            }
            // 萨登荆环（swd 卡牌面窗口机批）：被闪后可选判定窗（青龙
            // →贯石斧→荆环确定性次序，登记差异：上游同点触发技按
            // 目标序）。三窗并发时青龙先改杀重启、贯石斧先强伤。
            let has_sadeng = ctx
                .core
                .player(self.source)
                .equips
                .iter()
                .any(|c| ctx.catalog.card_kind(*c) == CardKind::Sadengjinhuan);
            if has_sadeng && ctx.core.player(self.source).alive && !self.asking_sadeng {
                self.asking_sadeng = true;
                return Flow::WaitChoice {
                    observer: self.source,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::Confirm,
                        prompt: "sadengjinhuan_judge_confirm",
                        options: vec![ChoiceOption::Confirm],
                        min: 0,
                        max: 1,
                        allow_cancel: true,
                    },
                };
            }
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
        // 燎原（swd 批 145，规则 v319）：仍有未满足的额外闪需求 → 未闪避
        // 部分视为命中（上游 `trigger.untrigger(); directHit=true` 同口径；
        // 加伤=燎原计数，上游 liaoyuan2 damageBegin `trigger.num += storage`
        // 同口径）
        if self.dodges_used < ctx.core.turn.liaoyuan_count {
            // 加伤与清零由 emit_sha_damage 统一承担
            return self.emit_sha_damage(ctx);
        }
        // 杀被闪避（swd 批 141，规则 v316 同批）：发射 ShaDodged（上游
        // shaMiss 触发窗；青龙/贯石斧等窗口之后——登记差异：窗口次序）
        ctx.emit(EventData::ShaDodged {
            card: self.card,
            source: self.source,
            target: self.target,
        });
        // 目标已死等无伤害路径：推进队列/弃置杀本体。
        return self.advance_target(ctx);
    }
}
