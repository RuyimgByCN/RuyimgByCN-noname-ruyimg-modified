use super::*;

/// 枳枉（十殿阎罗五官王，规则 v265）：其他角色获得牌结算后（CardsMoved
/// 面——摸牌阶段摸牌走 CardsDrawn 天然隔离=上游排除 draw 同口径），其
/// 随机弃置一张手牌（随机→首张确定性近似登记；planetarian 彩蛋分支不
/// 落登记）
pub(super) fn commit_cards_moved(ctx: &mut Ctx<'_>, to: PlayerId) {
    if let Some(zwdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_ZHIWANG)
    {
        let holders: Vec<PlayerId> = ctx
            .core
            .skills
            .iter()
            .filter(|((_, dd), i)| *dd == zwdef && i.enabled)
            .map(|((o, _), _)| *o)
            .collect();
        if let Some(holder) = holders.first() {
            if to != *holder && ctx.core.player(to).alive {
                let pick = ctx.core.player(to).hand.first().copied();
                if let Some(c) = pick {
                    ctx.discard_from_hand(to, c);
                }
            }
        }
    }
}

/// 连环切换提交：翻转横置状态并写回帧载荷。
pub(super) fn commit_linked(ctx: &mut Ctx<'_>, frame_id: EventId, player: PlayerId) {
    let next = !ctx.core.player(player).linked;
    ctx.core.player_mut(player).linked = next;
    if let Some(f) = ctx.res.frame_mut(frame_id) {
        if let EventData::Linked { linked, .. } = &mut f.data {
            *linked = next;
        }
    }
    ctx.log(LogEvent::Linked {
        player,
        linked: next,
    });
}

/// 出牌提交：牌离开手牌区。杀置桌面等待结算；桃立即回复。
/// 猴子偷桃（E03）取消的使用的牌已在确认机器中移走，跳过结算。
pub(super) fn commit_card_used(ctx: &mut Ctx<'_>, u: crate::event::CardUseData) {
    // 奇佐副本载体槽（Boss 郭嘉，规则 v259）：任一非载体卡牌提交即清
    //（副本用牌自身的提交保留槽位，供其 AfterTriggers 过滤拒递归）
    if ctx.core.turn.qizuo_copy != Some(u.card) {
        ctx.core.turn.qizuo_copy = None;
    }
    if u.cancelled {
        // 取消痕迹已在猴子机器的 CardsGiven/CardsDiscarded 中观测。
        return;
    }
    if ctx.core.player(u.player).hand.contains(&u.card) {
        ctx.take_hand_card(u.player, u.card);
    } else if ctx.core.table.contains(&u.card) {
        // 神秘果再结算（swd 卡牌面窗口机批）：原体自弃堆直上桌面（处理
        // 区暂存）后重发射——提交段自桌面取，不经手牌（AfterTriggers 窗
        // 口内手牌数无扰动）；随后按牌种臂照常置桌面/弃置。
        ctx.core.table.retain(|c| *c != u.card);
    } else {
        // 保持原 panic 语义（调用方先验证）。
        ctx.take_hand_card(u.player, u.card);
    }
    // 全局用牌计数（U05 xianjian 小蛮，规则 v163 灵泉门控；上游
    // getHistory("useCard") 同口径——未被取消的使用全计）
    ctx.core.player_mut(u.player).total_used += 1;
    // 沐风（swd 桓远之，批㉑）：本回合已使用基本牌（TurnState 旗，换回合清）
    if ctx.catalog.card_kind(u.card).is_basic_kind() {
        ctx.core.turn.mufeng_used_basic = true;
    }
    // 邪酬（Boss 相柳，规则 v223）：激活态下其他角色使用【桃】/【酒】
    // → 相柳获 1 蛇毒标记（上游 boss_echou global useCard 同口径）
    let effective_kind = ctx.catalog.card_kind(u.card);
    if matches!(
        u.use_as.unwrap_or(effective_kind),
        CardKind::Tao | CardKind::Jiu
    ) {
        if let Some(edef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_ECHOU)
        {
            let holders: Vec<PlayerId> = ctx
                .core
                .skills
                .iter()
                .filter(|((_, dd), i)| *dd == edef && i.enabled)
                .map(|((o, _), _)| *o)
                .collect();
            for holder in holders {
                if holder != u.player && ctx.core.player(holder).alive {
                    let p = ctx.core.player_mut(holder);
                    p.shedu = p.shedu.saturating_add(1);
                }
            }
        }
    }
    // 服毒（十殿阎罗泰山王，规则 v268）：其他角色使用【桃】时，随机另
    // 一名其他角色失去 1 点体力（上游 global useCard boss_fudu；随机→
    // 座位序首个确定性近似登记；候选=全体减持有者减使用者）
    {
        let kind = u.use_as.unwrap_or(effective_kind);
        if kind == CardKind::Tao {
            if let Some(fddef) = ctx
                .catalog
                .skill_by_name(crate::vocabulary::names::BOSS_FUDU)
            {
                let seats = ctx.core.players.len() as PlayerId;
                if seats > 2 {
                    let holders: Vec<PlayerId> = ctx
                        .core
                        .skills
                        .iter()
                        .filter(|((_, dd), i)| *dd == fddef && i.enabled)
                        .map(|((o, _), _)| *o)
                        .collect();
                    if let Some(t) = (0..seats).find(|&t| {
                        t != u.player && !holders.contains(&t) && ctx.core.player(t).alive
                    }) {
                        if holders
                            .iter()
                            .any(|h| *h != u.player && ctx.core.player(*h).alive)
                        {
                            ctx.emit(EventData::LoseHp {
                                player: t,
                                amount: 1,
                            });
                        }
                    }
                }
            }
        }
    }
    // 冥顽（Boss 梼杌，规则 v219）：回合内锁定集已激活（非空）时每用
    // 一张牌摸一张（上游 boss_minwan_draw useCard 同口径）
    if ctx.core.turn.current == u.player && !ctx.core.turn.minwan_targets.is_empty() {
        if let Some(mdef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_MINWAN)
        {
            if ctx.core.has_skill(u.player, mdef) {
                ctx.draw_cards(u.player, 1);
            }
        }
    }
    // 栖梧（群雄乱入 Boss 庞统，规则 v277）：使用梅花牌后若已受伤回复
    // 1 点体力（上游 useCard club+hp<maxHp recover 同口径；事件面与娇孝
    // 臂同族——机器直发 useCard 同覆盖）
    if let Some(qwdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_QIWU)
    {
        let club = matches!(
            ctx.catalog.cards[u.card as usize].suit,
            crate::catalog::Suit::Club
        );
        if club
            && ctx.core.has_skill(u.player, qwdef)
            && ctx.core.player(u.player).alive
            && ctx.core.player(u.player).hp > 0
            && ctx.core.player(u.player).hp < ctx.core.player(u.player).max_hp as i32
        {
            ctx.recover(u.player, 1, None);
        }
    }
    // 娇孝（Boss 甄姬，规则 v262）：成为红色牌的目标后摸 1（上游 target
    // useCardToTargeted frequent 同口径；红色=实体花色 Heart/Diamond；
    // 含延时自置目标）
    if let Some(jxdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_JIAOXIA)
    {
        let holders: Vec<PlayerId> = ctx
            .core
            .skills
            .iter()
            .filter(|((_, dd), i)| *dd == jxdef && i.enabled)
            .map(|((o, _), _)| *o)
            .filter(|o| ctx.core.player(*o).alive)
            .collect();
        if let Some(holder) = holders.first() {
            let red = matches!(
                ctx.catalog.cards[u.card as usize].suit,
                crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
            );
            // 显式目标面：targets 含持有者；闪电自置：targets 空（隐式
            // 自目标，上游 useCardToTargeted target=使用者同口径）
            let is_self_shandian =
                u.targets.is_empty() && u.player == *holder && effective_kind == CardKind::ShanDian;
            if red && (u.targets.contains(holder) || is_self_shandian) {
                ctx.draw_cards(*holder, 1);
            }
        }
    }
    ctx.log(LogEvent::CardPlayed {
        card: u.card,
        player: u.player,
        targets: u.targets.clone(),
    });
    // view-as 声明持久化：延时锦囊转化（国色）需跨阶段保存声明牌种，
    // 判定生效时按声明牌种执行（目录牌种不再可靠）。
    if let Some(as_kind) = u.use_as {
        ctx.core.declared_kinds.insert(u.card, as_kind);
    }
    // view-as 转化（武圣/龙胆）：以声明牌种结算，而非目录牌种
    let effective = u.use_as.unwrap_or_else(|| ctx.catalog.card_kind(u.card));
    match effective {
        CardKind::Tao => {
            ctx.to_discard(u.card);
            ctx.recover(u.player, 1, Some(u.card));
        }
        CardKind::Zhufangshenshi => {
            // 贴房绅视（U05 swd 第十批，规则 v94）：本回合无视距离指定
            // 该目标（sha 距离校验豁免）并摸一张牌。登记差异：上游为
            // 锦囊可被无懈（AI 权重 0 实战不戒），原生简化为直接结算。
            let t = *u.targets.first().expect("zhufangshenshi has one target");
            ctx.core.turn.zhufangshenshi_target = Some(t);
            ctx.to_discard(u.card);
            ctx.draw_cards(u.player, 1);
        }
        CardKind::Caoyao => {
            // 草药（U05 swd 第三批，规则 v87）：为受伤目标回复 1 点体力
            //（目标合法性在 PlayCard 门控：距离 1 + hp<maxHp）
            ctx.to_discard(u.card);
            let t = *u.targets.first().expect("caoyao has one target");
            ctx.recover(t, 1, Some(u.card));
        }
        CardKind::Xuejibingbao => {
            // 雪霁冰雹（swd 卡牌面补切批②）：目标标记 2 个摸牌阶段摸牌
            // 数 +1（Draw 钩消费；重标记在 PlayCard 门控拒绝）
            ctx.to_discard(u.card);
            let t = *u.targets.first().expect("xuejibingbao has one target");
            ctx.core.player_mut(t).xuejibingbao = 2;
        }
        CardKind::Fengyinzhidan => {
            // 封印之丹（swd 卡牌面补切批②）：随机使用两张普通锦囊牌——
            // 登记差异：原生确定性窄池{五谷→全体其他存活者, 决斗→座位序
            // 首个其他存活者}×2（炮台 v108 随机锦囊最小闭环同口径）
            ctx.to_discard(u.card);
            let others: Vec<PlayerId> = (0..ctx.core.players.len())
                .map(|i| i as PlayerId)
                .filter(|p| *p != u.player && ctx.core.player(*p).alive)
                .collect();
            if others.is_empty() {
                return;
            }
            ctx.use_virtual_card(u.player, CardKind::Wugu, others.clone());
            let others2: Vec<PlayerId> = (0..ctx.core.players.len())
                .map(|i| i as PlayerId)
                .filter(|p| *p != u.player && ctx.core.player(*p).alive)
                .collect();
            if let Some(t) = others2.first() {
                ctx.use_virtual_card(u.player, CardKind::Duel, vec![*t]);
            }
        }
        CardKind::Sha | CardKind::FireSha | CardKind::LeiSha => {
            // 出牌阶段限用一次（普通/火/雷杀共享）；龙颅山峰地图效果
            //（mtg 首切片，规则 v100）下第二张杀走额外标记——若本次为
            // 加成额外杀则置 land_sha_used，否则置常规已用
            // 破月（拓跋玉儿，swd 武将面批 28）：红杀不计入限一次（上
            // 游 stat 回退同口径）
            let poyue_red_free = matches!(
                ctx.catalog.cards[u.card as usize].suit,
                crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
            ) && ctx
                .catalog
                .skill_by_name("poyue")
                .map(|def| ctx.core.has_skill(u.player, def))
                .unwrap_or(false);
            // 愚钝·福面（swd 武将面批 82）：双锦囊当杀不计入出杀次数
            //（上游 yudun_count useCard 后 stat.card.sha-- 同口径）。
            // 判据=声明杀且实体非杀族（PlayCard 常态出杀也带 use_as=
            // 声明位，须按实体牌种区分——批 15 良策勘误同口径）
            let yudun_free = u.use_as == Some(crate::catalog::CardKind::Sha)
                && !ctx.catalog.card_kind(u.card).is_sha_like()
                && ctx
                    .catalog
                    .skill_by_name("yudun")
                    .map(|def| ctx.core.has_skill(u.player, def))
                    .unwrap_or(false);
            if ctx.core.turn.sha_used_play_phase {
                if ctx.core.current_land == Some(crate::catalog::CardKind::Longlushanfeng) {
                    ctx.core.turn.land_sha_used = true;
                }
            } else if !poyue_red_free && !yudun_free {
                ctx.core.turn.sha_used_play_phase = true;
            }
            // 冰河要塞地图效果（mtg 第八批，规则 v107）：锁定全局——使
            // 用杀时若有牌须弃一张（上游 chooseToDiscard 可取消+玩家选；
            // 原生 RNG 代抽不可取消，登记差异）。当前弃牌阶段的水没计数
            // 不受影响（非弃牌阶段）。
            if ctx.core.current_land == Some(crate::catalog::CardKind::Bingheyaosai) {
                let p = ctx.core.player(u.player);
                let has_cards = !p.hand.is_empty() || !p.equips.is_empty();
                if has_cards {
                    let from_hand = if p.hand.is_empty() {
                        false
                    } else if p.equips.is_empty() {
                        true
                    } else {
                        ctx.core.rng.gen_range(2u32) == 0
                    };
                    if from_hand {
                        let n = ctx.core.player(u.player).hand.len() as u32;
                        let j = ctx.core.rng.gen_range(n) as usize;
                        let c = ctx.core.player_mut(u.player).hand.remove(j);
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: u.player,
                            cards: vec![c],
                        });
                    } else {
                        let n = ctx.core.player(u.player).equips.len() as u32;
                        let j = ctx.core.rng.gen_range(n) as usize;
                        let c = ctx.core.player_mut(u.player).equips.remove(j);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, u.player, c);
                        ctx.to_discard(c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: u.player,
                            cards: vec![c],
                        });
                    }
                }
            }
            ctx.core.table.push(u.card);
        }
        CardKind::Jiu | CardKind::Tianxianjiu => {
            // 酒/天仙酒：命令边界已校验每回合一次——置本回合已用 + 待
            // 消耗效果（天仙酒为酒的 swd 变体，规则 v89；登记差异：与酒
            // 共享每回合一次计数）
            ctx.to_discard(u.card);
            ctx.core.turn.jiu_used = true;
            ctx.core.turn.jiu_active = true;
        }
        CardKind::Shenmiguo => {
            // 神秘果（swd 卡牌面窗口机批）：载体牌直接弃置（不置桌面）
            //——再结算由 CardUsed after 窗口行为发射（mufeng 等基本牌
            // 计数已在前置段按真实牌种覆盖）。
            ctx.to_discard(u.card);
        }
        // Le/Bing/ShanDian：先置桌面，经无懈窗口（使用时开窗，对齐旧线）
        // 由 PlaceDelayedMachine 放入判定区或弃置。
        _ => {
            ctx.core.table.push(u.card);
        }
    }
    // 突击计数（U07 ow，规则 v118）：出牌阶段使用任意牌 +1（取消不计；
    // 等值牌种分支已各自置 table，此处只累计）
    ctx.core.turn.cards_used_play_phase += 1;
    // 本回合已用牌数（swd 灵武批 120，规则 v307；上游 `countUsed()` 无参
    // 口径=本回合全部使用，含出牌阶段外）
    ctx.core.turn.cards_used_turn += 1;
    // 基本/锦囊/装备三类计数（T8 增量 14）：按结算牌种判类（view-as 转
    // 化后），地/机关牌异类不计
    if effective.is_basic_trick_or_equip() {
        ctx.core.turn.bte_used_play_phase += 1;
    }
}

/// E03 翻面提交：取消位为真则跳过翻转（漩涡面具等锁定技）；
/// 否则翻转武将牌朝向并发出完成事件。
/// 雷击（Boss 青龙，规则 v234）：打出/使用闪后触发——座位序下一名其
/// 他存活角色确定性判定（铁骑内联口径，无鬼道/天妒改判窗；目标选择以
/// 上游 chooseTarget→座位序首名确定性近似登记）：黑桃 → 其受 2 点雷伤；
/// 梅花 → 持有者回复 1、其受 1 点雷伤；其他无事。
pub(super) fn releiji_on_shan(ctx: &mut Ctx<'_>, holder: PlayerId) {
    let releiji = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_RELEIJI)
        .map(|d| ctx.core.has_skill(holder, d))
        .unwrap_or(false);
    if !releiji {
        return;
    }
    let tgt = ctx.core.next_alive(holder);
    if tgt == holder || !ctx.core.player(tgt).alive {
        return;
    }
    let judged = ctx.draw_judge_card();
    let suit = ctx.catalog.cards[judged as usize].suit;
    ctx.to_discard(judged);
    match suit {
        crate::catalog::Suit::Spade => {
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                None,
                tgt,
                2,
                crate::event::DamageNature::Thunder,
                None,
            )));
        }
        crate::catalog::Suit::Club => {
            ctx.recover(holder, 1, None);
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                None,
                tgt,
                1,
                crate::event::DamageNature::Thunder,
                None,
            )));
        }
        _ => {}
    }
}

pub(super) fn commit_turn_over_before(ctx: &mut Ctx<'_>, player: PlayerId, cancelled: bool) {
    if cancelled {
        return;
    }
    // 凶兽（Boss 模式共有技，规则 v208）：你不能被翻面（上游
    // boss_xiongshou_turn turnOverBefore priority 20 trigger.cancel 同
    // 口径——入口直读跳过，死亡取消型基建同款）
    if let Some(def) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_XIONGSHOU)
    {
        if ctx
            .core
            .skills
            .get(&(player, def))
            .map(|i| i.enabled)
            .unwrap_or(false)
        {
            return;
        }
    }
    // 神裔（Boss 青龙共有技，规则 v233）：武将牌不能翻至背面（上游
    // boss_shenyi turnOverBefore filter !isTurnedOver——仅正→背取消，
    // 背→正照常翻回）
    if let Some(def) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_SHENYI)
    {
        if ctx
            .core
            .skills
            .get(&(player, def))
            .map(|i| i.enabled)
            .unwrap_or(false)
            && !ctx.core.player(player).face_down
        {
            return;
        }
    }
    let face_down = !ctx.core.player(player).face_down;
    ctx.core.player_mut(player).face_down = face_down;
    // 急冻冰封解除（U05 ow 美，规则 v127）：翻回正面即冰封结束——上游
    // jidong2 至 turnOverAfter 到期同口径（含翻面跳回合的翻回；急冻
    // 自身翻至背面为内联翻转不经此处）
    if !face_down {
        ctx.core.player_mut(player).jidong_active = false;
        // 虚失（Boss 朱厌，规则 v225）：武将牌翻回正面时对所有其他角色
        // 各造成 1 点伤害（上游 [1,2] 随机——恒 1 登记差异；激活态查询）
        if let Some(xdef) = ctx
            .catalog
            .skill_by_name(crate::vocabulary::names::BOSS_XUSHI)
        {
            if ctx.core.has_skill(player, xdef) {
                for t in ctx.core.seat_order_from(player) {
                    if t != player && ctx.core.player(t).alive {
                        ctx.emit(EventData::Damage(crate::machine::damage_data(
                            Some(player),
                            t,
                            1,
                        )));
                    }
                }
            }
        }
    }
    ctx.emit(EventData::TurnOvered { player, face_down });
}

/// 胆术/精觉（Boss 夜叉/精觉年兽，规则 v252）：玩家失去手牌后（上游
/// loseAfter+loseAsync 族的 CardsDiscarded 面；gain/addJudge/equip 等
/// 其余卡牌移动面登记差异）——非当前回合且已受伤 → 判定红色回 1（铁骑
/// 内联确定性口径，无鬼才/天妒改判窗）。
pub(super) fn commit_cards_discarded(ctx: &mut Ctx<'_>, player: PlayerId) {
    // 望尘（swd 古月圣，批㉑）：自己弃牌阶段弃置过基本牌（TurnState 旗，
    // 换回合清；判定区/装备区弃置路径亦计入——上游 discardEnd cards 同为
    // 实际弃置集合）
    if let Some(f) = ctx.res.frame(ctx.current_frame_id()) {
        if let EventData::CardsDiscarded { cards, .. } = &f.data {
            if ctx.core.turn.current == player
                && ctx.core.turn.phase == crate::event::Phase::Discard
                && cards
                    .iter()
                    .any(|c| ctx.catalog.card_kind(*c).is_basic_kind())
            {
                ctx.core.turn.wangchen_discard_basic = true;
            }
        }
    }
    let names = [
        crate::vocabulary::names::BOSS_DANSHU,
        crate::vocabulary::names::BOSS_JINGJUE,
    ];
    for name in names {
        let Some(def) = ctx.catalog.skill_by_name(name) else {
            continue;
        };
        if !ctx.core.has_skill(player, def) {
            continue;
        }
        // 非当前回合 + 已受伤
        if ctx.core.turn.current == player || !ctx.core.player(player).alive {
            continue;
        }
        let p = ctx.core.player(player);
        if p.hp >= p.max_hp as i32 {
            continue;
        }
        let judged = ctx.draw_judge_card();
        let red = matches!(
            ctx.catalog.cards[judged as usize].suit,
            crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
        );
        ctx.to_discard(judged);
        if red {
            ctx.recover(player, 1, None);
        }
        break; // 胆术/精觉同内容不叠加
    }
    // 魔炎（Boss 罗刹，规则 v254）：非当前回合失去牌后判定红色 → 对座
    // 位序首个其他存活角色造成 2 点火焰伤害（目标自选→确定性近似登记；
    // loseEnd 全牌损失面→CardsDiscarded 面登记差异）
    if let Some(mydef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_MOYANY)
    {
        let holders: Vec<PlayerId> = ctx
            .core
            .skills
            .iter()
            .filter(|((_, dd), i)| *dd == mydef && i.enabled)
            .map(|((o, _), _)| *o)
            .filter(|o| ctx.core.player(*o).alive && ctx.core.turn.current != *o)
            .collect();
        if let Some(holder) = holders.first() {
            let judged = ctx.draw_judge_card();
            let red = matches!(
                ctx.catalog.cards[judged as usize].suit,
                crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
            );
            ctx.to_discard(judged);
            if red {
                let seats = ctx.core.players.len() as PlayerId;
                if let Some(t) = (0..seats).find(|&t| {
                    t != *holder
                        && ctx.core.player(t).alive
                        && ctx.core.is_boss_side(t) != ctx.core.is_boss_side(*holder)
                }) {
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(*holder),
                        t,
                        2,
                        crate::event::DamageNature::Fire,
                        None,
                    )));
                }
            }
        }
    }
    // 绝策（Boss 马面，规则 v258，一将包李儒旧绝策）：自己回合内其他角
    // 色失去最后一手牌（弃后空手）→ 对其造成 1 点伤害（chooseBool AI→
    // 恒发动确定性近似登记；lose/equip/gain 全事件族→CardsDiscarded 面
    // 登记差异）
    if let Some(jcdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_JUECE)
    {
        let holders: Vec<PlayerId> = ctx
            .core
            .skills
            .iter()
            .filter(|((_, dd), i)| *dd == jcdef && i.enabled)
            .map(|((o, _), _)| *o)
            .filter(|o| ctx.core.player(*o).alive && ctx.core.turn.current == *o && *o != player)
            .collect();
        for holder in holders {
            if ctx.core.player(player).hand.is_empty() {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(holder),
                    player,
                    1,
                )));
            }
        }
    }
    // 仙音（Boss 周瑜，规则 v263）：非当前回合失去牌后判定红 → 座位序
    // 首个其他存活角色失去 1 点体力（上游 loseEnd frequent judge 红 →
    // chooseTarget loseHp 同口径；chooseTarget AI 态度面→首个确定性近似
    // 登记）
    if let Some(xydef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_XIANYIN)
    {
        let holders: Vec<PlayerId> = ctx
            .core
            .skills
            .iter()
            .filter(|((_, dd), i)| *dd == xydef && i.enabled)
            .map(|((o, _), _)| *o)
            .filter(|o| ctx.core.player(*o).alive && ctx.core.turn.current != *o)
            .collect();
        if let Some(holder) = holders.first() {
            let judged = ctx.draw_judge_card();
            let red = matches!(
                ctx.catalog.cards[judged as usize].suit,
                crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
            );
            ctx.to_discard(judged);
            if red {
                let seats = ctx.core.players.len() as PlayerId;
                if let Some(t) = (0..seats).find(|&t| t != *holder && ctx.core.player(t).alive) {
                    ctx.emit(EventData::LoseHp {
                        player: t,
                        amount: 1,
                    });
                }
            }
        }
    }
    // 伤逝（规则 v260）：失去牌后补牌
    shangshix_draw(ctx, player);
}

/// 造异异变 def 查询（运行时授予技）。
pub(super) fn zhp_def(ctx: &Ctx<'_>) -> crate::ids::SkillDefId {
    ctx.catalog
        .skill_by_name(crate::vocabulary::names::BOSS_ZAOYI_HP)
        .expect("造异异变入目录")
}

/// 回合收束：到期技能移除，然后交给下一位存活玩家并开启判定阶段。
pub(super) fn record_vm_history(ctx: &mut Ctx<'_>, data: &EventData) {
    // 标准局面没有 VM 技能时不记。空 VmAux 不写摘要，v295 golden 保持。
    if !ctx.catalog.vm_card_history {
        return;
    }
    match data {
        EventData::CardsDiscarded { player, cards } => {
            ctx.core
                .vm_aux
                .lost
                .entry(*player)
                .or_default()
                .extend(cards.iter().copied());
        }
        EventData::CardsMoved { cards, from, to } if from != to => {
            ctx.core
                .vm_aux
                .lost
                .entry(*from)
                .or_default()
                .extend(cards.iter().copied());
        }
        EventData::CardsDrawn { player, cards } => {
            *ctx.core.vm_aux.drawn.entry(*player).or_default() += cards.len() as u32;
        }
        _ => {}
    }
}

pub(super) fn commit_turn_ended(ctx: &mut Ctx<'_>, player: PlayerId) {
    ctx.core.vm_aux.clear_turn();
    ctx.log(LogEvent::TurnEnded { player });
    ctx.expire_temporary_skills(player);
    for state in &mut ctx.core.players {
        state.zlfanshi_terra = false;
    }
    // 清残留跳阶段旗标（正常流程已被消费）
    ctx.core.turn.skip_draw_phase = false;
    ctx.core.turn.skip_play_phase = false;
    // 还侠跟踪清残留（正常流程已被阶段末回收消费）
    ctx.core.turn.huanxia_track.clear();
    // 神秘果/鼠槌窗口旗清残留（正常流程已被窗口机消费；防御换回合残留）
    ctx.core.turn.shenmiguo_used = false;
    ctx.core.turn.shuchui_armed = false;
    ctx.core.turn.shuchui_hit = false;
    ctx.core.turn.lianda_used = false;
    // Boss 模式（P2/N08 模式入口批三-a，规则 v82）：死亡座位不跳过——
    // 轮到其回合时执行重整 tick（上游 bossPhaseLoop）：计数 +1、hp<上限
    // 则 +1（否则手牌 <4 摸 1），计数达阈值复活；连续死亡座位逐个 tick
    // 后落在下一存活座位。
    if let Some((_, threshold)) = ctx.core.boss_mode() {
        let seats = ctx.core.players.len() as u8;
        let mut next = (player + 1) % seats;
        loop {
            if ctx.core.player(next).alive {
                break;
            }
            if threshold == 0 {
                // 上游 game.bossinfo.chongzheng 未设：无复活模式，跳过 tick
                next = (next + 1) % seats;
                if next == (player + 1) % seats {
                    break;
                }
                continue;
            }
            // 重整 tick
            let cnt = ctx.core.player(next).boss_chongzheng + 1;
            ctx.core.player_mut(next).boss_chongzheng = cnt;
            if (ctx.core.player(next).hp as u32) < ctx.core.player(next).max_hp {
                ctx.core.player_mut(next).hp += 1;
            } else if ctx.core.player(next).hand.len() < 4 {
                ctx.draw_cards(next, 1);
            }
            ctx.log(LogEvent::BossChongzhengTick {
                player: next,
                count: cnt,
            });
            if cnt >= threshold {
                let p = ctx.core.player_mut(next);
                p.alive = true;
                p.dying = false;
                p.boss_chongzheng = 0;
                ctx.log(LogEvent::BossRevived { player: next });
            }
            next = (next + 1) % seats;
            if next == (player + 1) % seats {
                break; // 全场死亡（终局判负已在死亡结算处理）
            }
        }
        if next <= player {
            ctx.core.turn.round += 1;
            ctx.core.vm_aux.clear_round();
        }
        ctx.core.turn.current = next;
        ctx.core.turn.phase = Phase::Judge;
        ctx.core.turn.sha_used_play_phase = false;
        ctx.core.turn.land_used = false;
        ctx.core.turn.land_sha_used = false;
        ctx.core.turn.turn_opened = false;
        // 回合级存量清零（v128 修复 v126 遗留：TurnState 换回合为原地
        // 改字段而非重建——乐动三角/链钩必须显式复位）
        ctx.core.turn.yuedong_num = 0;
        ctx.core.turn.yuedong_eff = 0;
        ctx.core.turn.yuedong_recover = false;
        ctx.core.turn.liangou_hook = None;
        ctx.core.turn.liangou_bonus_used = false;
        ctx.core.turn.feitiao_target = None;
        ctx.core.turn.fanling_used = false;
        ctx.core.turn.xjyufeng_used = 0;
        ctx.core.turn.diewu_draw_used = false;
        ctx.core.turn.lueying_used = false;
        // 冥顽锁定集（Boss 梼杌，规则 v219）：回合更替清零（上游 phaseAfter
        // clear 同窗）
        ctx.core.turn.minwan_targets.clear();
        ctx.core.turn.yanyu_count = 0;
        ctx.core.turn.txianqu_used = false;
        ctx.core.turn.jtjubao_used = false;
        ctx.emit(EventData::Phase {
            player: next,
            phase: Phase::Judge,
            begin: true,
        });
        return;
    }
    let next = ctx.core.next_alive(player);
    if next <= player {
        ctx.core.turn.round += 1;
        ctx.core.vm_aux.clear_round();
    }
    ctx.core.turn.current = next;
    ctx.core.turn.phase = Phase::Judge;
    ctx.core.turn.sha_used_play_phase = false;
    ctx.core.turn.land_used = false;
    ctx.core.turn.land_sha_used = false;
    ctx.core.turn.turn_opened = false;
    // 回合级存量清零（v128 修复 v126 遗留：TurnState 换回合为原地
    // 改字段而非重建——乐动三角/链钩必须显式复位）
    ctx.core.turn.yuedong_num = 0;
    ctx.core.turn.yuedong_eff = 0;
    ctx.core.turn.yuedong_recover = false;
    ctx.core.turn.liangou_hook = None;
    ctx.core.turn.liangou_bonus_used = false;
    ctx.core.turn.feitiao_target = None;
    ctx.core.turn.fanling_used = false;
    ctx.core.turn.xjyufeng_used = 0;
    ctx.core.turn.diewu_draw_used = false;
    ctx.core.turn.lueying_used = false;
    // 冥顽锁定集（Boss 梼杌，规则 v219）：回合更替清零（上游 phaseAfter
    // clear 同窗）
    ctx.core.turn.minwan_targets.clear();
    ctx.core.turn.yanyu_count = 0;
    ctx.core.turn.txianqu_used = false;
    ctx.core.turn.jtjubao_used = false;
    ctx.emit(EventData::Phase {
        player: next,
        phase: Phase::Judge,
        begin: true,
    });
}

/// 积财（Boss 饕餮，规则 v216）：回复结算后——回复者==持有者 → 持有
/// 者摸 2；否则持有者与回复者各摸 1（上游 boss_jicai recoverAfter 同
/// 口径；仅实际回复量>0 的 Recover 事件进入）。
pub(super) fn commit_recover_skills(ctx: &mut Ctx<'_>, recovered: PlayerId) {
    // 忍性（年兽形态，规则 v249）：global recoverEnd——持有者非当前回
    // 合角色时，任意角色回复后其摸 1（上游 boss_renxing recoverEnd 同口
    // 径；damageEnd 半面见 commit_damage 忍性臂）
    if let Some(rxdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_RENXING)
    {
        let holders: Vec<PlayerId> = ctx
            .core
            .skills
            .iter()
            .filter(|((_, dd), i)| *dd == rxdef && i.enabled)
            .map(|((o, _), _)| *o)
            .filter(|o| ctx.core.player(*o).alive && ctx.core.turn.current != *o)
            .collect();
        for holder in holders {
            ctx.draw_cards(holder, 1);
        }
    }
    let Some(jdef) = ctx
        .catalog
        .skill_by_name(crate::vocabulary::names::BOSS_JICAI)
    else {
        return;
    };
    let holders: Vec<PlayerId> = ctx
        .core
        .skills
        .iter()
        .filter(|((_, dd), i)| *dd == jdef && i.enabled)
        .map(|((o, _), _)| *o)
        .filter(|o| ctx.core.player(*o).alive)
        .collect();
    for holder in holders {
        if recovered == holder {
            ctx.draw_cards(holder, 2);
        } else {
            ctx.draw_cards(holder, 1);
            if ctx.core.player(recovered).alive {
                ctx.draw_cards(recovered, 1);
            }
        }
    }
    // 伤逝（规则 v260）：体力变化（回复）后补牌
    shangshix_draw(ctx, recovered);
}
