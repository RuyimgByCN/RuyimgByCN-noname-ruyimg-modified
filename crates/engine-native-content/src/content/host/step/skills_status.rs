use super::super::*;

impl HostMachine {
    pub(super) fn step_skills_status(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 每步·距离：无效果步（computed_distance 字段消费）
            HostBehavior::NewMeibuRange => Flow::Done,
            // 反击·红杀直伤：无效果步（杀机器内联消费）
            HostBehavior::YxsFanji2 => Flow::Done,
            // 烈弓：无效果步（杀机器内联消费）
            HostBehavior::FzLiegong => Flow::Done,
            // 门神·标记持有：无效果步（主体 70 KB 效果待批次）
            HostBehavior::YxsMenshen2 => Flow::Done,
            // 摸牌翻倍：无效果步（摸牌收束内联消费）
            HostBehavior::XiguaMopaiFanbei => Flow::Done,
            // 诈降：无效果步（出牌校验/杀机器/摸牌收束内联消费）
            HostBehavior::SbZhaXiang => Flow::Done,
            // 苦肉：失去 1 点体力并摸 2 张
            HostBehavior::Kurou => {
                ctx.log(LogEvent::SkillTriggered {
                    skill: "苦肉",
                    owner: self.owner,
                });
                ctx.emit(EventData::LoseHp {
                    player: self.owner,
                    amount: 1,
                });
                ctx.start_draw(self.owner, 2);
                Flow::Done
            }
            // 主亡忠继：忠臣继位为主公（身份互换）
            HostBehavior::ZhuWanzhongJi => {
                let Some(EventData::Death { player, .. }) = ctx
                    .res
                    .frame(ctx.current_event_frame_id())
                    .map(|f| f.data.clone())
                else {
                    return Flow::Done;
                };
                if let crate::state::GameMode::Identity { roles } = &mut ctx.core.mode {
                    if let Some(slot) = roles.get_mut(self.owner as usize) {
                        *slot = crate::state::Role::Lord;
                    }
                    if let Some(slot) = roles.get_mut(player as usize) {
                        *slot = crate::state::Role::Loyalist;
                    }
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "主亡忠继",
                    owner: self.owner,
                });
                Flow::Done
            }
            // 门神·清除：移除全场门神实例与标记
            HostBehavior::YxsMenshen3 => {
                let Some(def) = ctx.catalog.skill_by_name(U08_YXS_MENSHEN2_KEY) else {
                    return Flow::Done;
                };
                let seats: Vec<PlayerId> = (0..ctx.core.players.len() as PlayerId)
                    .filter(|p| ctx.core.skills.contains_key(&(*p, def)))
                    .collect();
                if seats.is_empty() {
                    return Flow::Done;
                }
                for p in seats {
                    ctx.core.skills.remove(&(p, def));
                    ctx.core.vm_aux.marks.remove(&(p, u32::from(def)));
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "门神·清除",
                    owner: self.owner,
                });
                Flow::Done
            }
            // 灾变：摸「忠 + 2 - 反 - 内」（下限 0）张
            HostBehavior::Zaibian => {
                let (mut loyal, mut rebel, mut renegade) = (0usize, 0usize, 0usize);
                let roles = ctx
                    .core
                    .identity_roles()
                    .map(|r| r.to_vec())
                    .unwrap_or_default();
                for p in 0..ctx.core.players.len() as PlayerId {
                    if !ctx.core.player(p).alive {
                        continue;
                    }
                    match roles.get(p as usize) {
                        Some(crate::state::Role::Lord) | Some(crate::state::Role::Loyalist) => {
                            loyal += 1;
                        }
                        Some(crate::state::Role::Rebel) => rebel += 1,
                        Some(crate::state::Role::Traitor) => renegade += 1,
                        _ => {}
                    }
                }
                let n = (loyal as i64 + 2 - rebel as i64 - renegade as i64).max(0) as usize;
                if n > 0 {
                    ctx.start_draw(self.owner, n as u32);
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "灾变",
                        owner: self.owner,
                    });
                }
                Flow::Done
            }
            // 循梦：伤害 +1；体力 >1 时失去 1 点体力
            HostBehavior::Xunmeng => {
                if !ctx.modify_damage_amount(1) {
                    return Flow::Done;
                }
                if ctx.core.player(self.owner).hp > 1 {
                    // 失去体力（上游 loseHp）：无来源、不可防止
                    ctx.emit(EventData::LoseHp {
                        player: self.owner,
                        amount: 1,
                    });
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "循梦",
                    owner: self.owner,
                });
                Flow::Done
            }
            // 集舍：摸 1 + 累计标记（上游 jishe2 的手牌上限削减不生效）
            HostBehavior::Jishe => {
                ctx.start_draw(self.owner, 1);
                let def = ctx
                    .catalog
                    .skill_by_name(L27_JISHE_KEY)
                    .map(u32::from)
                    .unwrap_or_default();
                let entry = ctx.core.vm_aux.marks.entry((self.owner, def)).or_insert(0);
                *entry += 1;
                ctx.log(LogEvent::SkillTriggered {
                    skill: "集舍",
                    owner: self.owner,
                });
                Flow::Done
            }
            // 鬼蛇旗：目标 +1 护甲（change_hujia=受伤抵 1 伤语义）
            HostBehavior::GuisheqiCard => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        ctx.change_hujia(t, 1);
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            // 仙露回：全体受伤目标各 +1 护甲（targets 已过滤受伤）
            HostBehavior::XianluhuiCard => {
                for t in self.targets.clone() {
                    if ctx.core.player(t).alive {
                        ctx.change_hujia(t, 1);
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            // 隐月：伤害前置窗——num+1 写回（上游 damageBegin4 num++），
            // 并记录伤害类型（雷/火/锦囊牌；上游 damage 面记录，原生同窗
            // 记录，登记差异：防止伤害的窗口内已记录）
            HostBehavior::HlssYueyin => {
                let (nature, card) = {
                    let d = ctx.current_damage_mut();
                    if let Some(amount) = d.amount.checked_add(1) {
                        d.amount = amount;
                    } else {
                        ctx.fail_damage_amount();
                        return Flow::Done;
                    }
                    (d.nature, d.card)
                };
                let mut bits = 0u8;
                match nature {
                    crate::event::DamageNature::Thunder => bits |= HLSS_DAMAGE_THUNDER,
                    crate::event::DamageNature::Fire => bits |= HLSS_DAMAGE_FIRE,
                    crate::event::DamageNature::Normal => {}
                }
                if let Some(card) = card {
                    // 上游按 `get.type2(card)` 记 'trick'/'delay' 两类键；
                    // 原生合并记「锦囊牌伤害」一位（登记差异）。
                    let kind = ctx.catalog.card_kind(card);
                    if !kind.is_basic_kind() && !kind.is_equip() {
                        bits |= HLSS_DAMAGE_TRICK;
                    }
                }
                ctx.core.player_mut(self.owner).hlss_damage |= bits;
                Flow::Done
            }
            // 起风·闭 / 改命：锁定旗标（无机器步；无懈门/判定机内联）
            HostBehavior::HlssFengqi2 => Flow::Done,
            HostBehavior::HlssGaiming => Flow::Done,
            // 潜龙：按触发面累加道心值（上限 99），随后按阈值刷新解锁
            HostBehavior::MbQianlong => {
                let is_phase = matches!(
                    ctx.res.frame(ctx.current_event_frame_id()).map(|f| &f.data),
                    Some(crate::event::EventData::Phase { begin: true, .. })
                );
                let inc: u32 = if is_phase {
                    // 开局面：另有魏势力持「卫统」者 → 60，否则 20
                    let boost = ctx.core.alive_players().any(|p| {
                        p != self.owner
                            && ctx.core.effective_faction(ctx.catalog, p)
                                == Some(crate::catalog::Faction::Wei)
                            && ctx
                                .catalog
                                .skill_by_name(HLSS_MBWEITONG_KEY)
                                .map(|d| ctx.core.has_skill(p, d))
                                .unwrap_or(false)
                    });
                    if boost {
                        60
                    } else {
                        20
                    }
                } else {
                    match ctx.current_damage() {
                        Ok(d) if d.target == self.owner => {
                            (d.amount.max(0) as u32).saturating_mul(10)
                        }
                        Ok(d) => (d.amount.max(0) as u32).saturating_mul(15),
                        Err(_) => 5,
                    }
                };
                let cur = u32::from(ctx.core.player(self.owner).mbqianlong_marks);
                let next = (cur + inc).min(99);
                ctx.core.player_mut(self.owner).mbqianlong_marks = next as u8;
                mbqianlong_refresh(ctx, self.owner);
                Flow::Done
            }
            // 涅槃：弃全部牌 + 解除横置 + 翻回正面 + 摸 3 + 回复至 3；
            // 随后触发浴火（上游独立 niepanAfter 事件面 → 原生链式，登记差异）
            HostBehavior::Niepan => {
                if ctx.core.player(self.owner).niepan_used {
                    return Flow::Done;
                }
                niepan_effect(ctx, self.owner);
                Flow::Done
            }
            HostBehavior::Yuhuo => Flow::Done,
            HostBehavior::Shenwei => Flow::Done,
            // 止息：标记 +1；锦囊 → 置锦囊记录；显示标记 = 体力 - 计数
            HostBehavior::NewZhixi => {
                let used_trick = match ctx.res.frame(ctx.current_event_frame_id()).map(|f| &f.data)
                {
                    Some(EventData::CardUsed(u)) => {
                        let kind = u.use_as.unwrap_or_else(|| u.effective_kind(ctx.catalog));
                        !kind.is_basic_kind() && kind.equip_slot().is_none()
                    }
                    _ => false,
                };
                let def = ctx
                    .catalog
                    .skill_by_name(L27_NEW_ZHIXI_KEY)
                    .map(u32::from)
                    .unwrap_or_default();
                let entry = ctx.core.vm_aux.marks.entry((self.owner, def)).or_insert(0);
                *entry += 1;
                if used_trick {
                    let mark2 = ctx
                        .catalog
                        .skill_by_name(L27_NEW_ZHIXI_MARK2_KEY)
                        .map(u32::from)
                        .unwrap_or_default();
                    ctx.core.vm_aux.marks.insert((self.owner, mark2), 1);
                }
                // 上游显示标记 = 体力 - 计数（展示面）；原生保留**计数**语义
                let _ = ctx.core.vm_aux.marks.get(&(self.owner, def)).copied();
                ctx.log(LogEvent::SkillTriggered {
                    skill: "止息",
                    owner: self.owner,
                });
                Flow::Done
            }
            // 撷芳：刷新「存活女性数」标记（0 时清除）
            HostBehavior::Xiefang => {
                let count = (0..ctx.core.players.len() as PlayerId)
                    .filter(|p| {
                        ctx.core.player(*p).alive
                            && ctx.core.effective_gender(*p)
                                == crate::content::standard::Gender::Female
                    })
                    .count() as i32;
                let def = ctx
                    .catalog
                    .skill_by_name(L27_XIEFANG_KEY)
                    .map(u32::from)
                    .unwrap_or_default();
                if count > 0 {
                    ctx.core.vm_aux.marks.insert((self.owner, def), count);
                } else {
                    ctx.core.vm_aux.marks.remove(&(self.owner, def));
                }
                Flow::Done
            }
            // 疾风：多目标使用 → 按概率整牌取消（上游 `trigger.cancel()`）
            HostBehavior::Jifeng => {
                let n = match ctx.res.frame(ctx.current_event_frame_id()).map(|f| &f.data) {
                    Some(EventData::CardUsed(u)) => u.targets.len() as u32,
                    _ => return Flow::Done,
                };
                // 上游：Math.random() > (1.3 + n/5)/n 时取消（即取消概率
                // = 1 - (1.3 + n/5)/n）。原生以 gen_range(1000) 千分位判定。
                let cancel_permille =
                    ((1000.0 * (1.0 - (1.3 + n as f64 / 5.0) / n as f64)).round()).max(0.0) as u32;
                let roll = ctx.core.rng.gen_range(1000);
                if roll < cancel_permille {
                    ctx.current_card_use_mut().cancelled = true;
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "疾风·失误",
                        owner: self.owner,
                    });
                }
                Flow::Done
            }
            // 昆仑镜：阶段收束记录快照 / 阶段开始（机器内确认门）回滚
            HostBehavior::Kunlunjing => {
                // A2 H11：快照面＝整回合结束 phaseAfter 段；比对面＝TurnBegin 窗。
                let begin = matches!(
                    ctx.res.frame(ctx.current_event_frame_id()).map(|f| &f.data),
                    Some(EventData::TurnBegin { .. })
                );
                if !begin {
                    let mut snap = Vec::new();
                    for p in 0..ctx.core.players.len() as PlayerId {
                        let pl = ctx.core.player(p);
                        snap.push((p, pl.hand.clone(), pl.equips.clone(), pl.judge.clone()));
                    }
                    let hp = ctx.core.player(self.owner).hp;
                    ctx.core.turn.kunlunjing_snapshot = snap;
                    ctx.core.turn.kunlunjing_hp = hp;
                    return Flow::Done;
                }
                if self.stage == HostStage::Act {
                    self.stage = HostStage::SelectNature;
                    return Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec::confirm("trigger_confirm"),
                    };
                }
                if !ctx.answer().contains(&ChoiceOption::Confirm) {
                    return Flow::Done;
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "昆仑镜",
                    owner: self.owner,
                });
                // 回滚：把快照中的牌拉回各自原位（当前所在区任意）
                let snap = std::mem::take(&mut ctx.core.turn.kunlunjing_snapshot);
                for (p, hand, equips, judge) in snap {
                    for c in hand.iter().copied() {
                        kunlunjing_move_to(ctx, c, p, 0);
                    }
                    for c in equips.iter().copied() {
                        kunlunjing_move_to(ctx, c, p, 1);
                    }
                    for c in judge.iter().copied() {
                        kunlunjing_move_to(ctx, c, p, 2);
                    }
                }
                Flow::Done
            }
            HostBehavior::Shengong => Flow::Done,
            // 轩辕剑·反噬：失去 1 点体力（无「轩辕」时）
            HostBehavior::XuanyuanjianBacklash => {
                let has_xuanyuan = ctx
                    .catalog
                    .skill_by_name(SWD_XUANYUAN_KEY)
                    .map(|d| ctx.core.has_skill(self.owner, d))
                    .unwrap_or(false);
                ctx.core.turn.xuanyuanjian_marked = false;
                if !has_xuanyuan {
                    ctx.emit(EventData::LoseHp {
                        player: self.owner,
                        amount: 1,
                    });
                }
                Flow::Done
            }
            // 改造：选一个空栏位 → 写入 declared_slots（提交按新栏位入槽）
            HostBehavior::Gaizao => match self.stage {
                HostStage::Act => {
                    let card = match ctx.res.frame(ctx.current_event_frame_id()).map(|f| &f.data) {
                        Some(EventData::Equipped { card, .. }) => *card,
                        _ => return Flow::Done,
                    };
                    let occupied = |slot: crate::catalog::EquipSlot| {
                        ctx.core
                            .player(self.owner)
                            .equips
                            .iter()
                            .any(|c| ctx.catalog.card_slot_of(ctx.core, *c) == Some(slot))
                    };
                    let slots = [
                        (0u8, crate::catalog::EquipSlot::Weapon),
                        (1, crate::catalog::EquipSlot::Armor),
                        (2, crate::catalog::EquipSlot::HorsePlus),
                        (3, crate::catalog::EquipSlot::HorseMinus),
                        (4, crate::catalog::EquipSlot::Treasure),
                    ];
                    let options: Vec<ChoiceOption> = slots
                        .iter()
                        .filter(|(_, s)| !occupied(*s))
                        .map(|(i, _)| ChoiceOption::Tag(*i))
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.card = Some(card);
                    self.stage = HostStage::SelectNature;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: "gaizao_slot",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectNature => {
                    let Some(ChoiceOption::Tag(slot)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let Some(card) = self.card else {
                        return Flow::Done;
                    };
                    let slot = match slot {
                        0 => crate::catalog::EquipSlot::Weapon,
                        1 => crate::catalog::EquipSlot::Armor,
                        2 => crate::catalog::EquipSlot::HorsePlus,
                        3 => crate::catalog::EquipSlot::HorseMinus,
                        _ => crate::catalog::EquipSlot::Treasure,
                    };
                    ctx.core.declared_slots.insert(card, slot);
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "改造",
                        owner: self.owner,
                    });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::Tianlun => Flow::Done,
            // 革新印：选花色 → 静默弃该花色全部手牌 → 选另一花色 → 获得等量
            HostBehavior::Gxianyin => match self.stage {
                HostStage::Act => {
                    let mut suits: Vec<u8> = Vec::new();
                    for c in ctx.core.player(self.owner).hand.iter().copied() {
                        if let Some(suit) = mbcm_suit_of(ctx, c) {
                            if !suits.contains(&suit) {
                                suits.push(suit);
                            }
                        }
                    }
                    if suits.is_empty() {
                        return Flow::Done;
                    }
                    suits.sort_unstable();
                    self.stage = HostStage::SelectNature;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: "gxianyin_suit",
                            options: suits.into_iter().map(ChoiceOption::Tag).collect(),
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectNature => {
                    let Some(ChoiceOption::Tag(suit)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    // 静默弃置该花色全部手牌（上游 `_triggered=null`：不触发
                    // 失去牌类事件）
                    let cards: Vec<CardId> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| mbcm_suit_of(ctx, *c) == Some(suit))
                        .collect();
                    if cards.is_empty() {
                        return Flow::Done;
                    }
                    let count = cards.len();
                    self.cards = cards.iter().map(|c| u32::from(*c)).collect();
                    for c in cards.iter().copied() {
                        ctx.take_hand_card(self.owner, c);
                        ctx.to_discard(c);
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "革新印",
                        owner: self.owner,
                    });
                    // 选另一花色（其余三色）
                    let rest: Vec<u8> = (0..=3u8).filter(|s| *s != suit).collect();
                    self.card = Some(count as CardId);
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: "gxianyin_gain_suit",
                            options: rest.into_iter().map(ChoiceOption::Tag).collect(),
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Tag(suit)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let count = self.card.unwrap_or(0) as usize;
                    // 从牌堆（不足则弃牌堆）取该花色至多 count 张入手
                    let mut gained: Vec<CardId> = Vec::new();
                    for src_discard in [false, true] {
                        if gained.len() >= count {
                            break;
                        }
                        let pool: Vec<CardId> = if src_discard {
                            ctx.core.discard.clone()
                        } else {
                            ctx.core.deck.clone()
                        };
                        for c in pool {
                            if gained.len() >= count {
                                break;
                            }
                            if mbcm_suit_of(ctx, c) != Some(suit) {
                                continue;
                            }
                            if src_discard {
                                ctx.core.discard.retain(|x| *x != c);
                            } else {
                                ctx.core.deck.retain(|x| *x != c);
                            }
                            ctx.core.player_mut(self.owner).hand.push(c);
                            gained.push(c);
                        }
                    }
                    if !gained.is_empty() {
                        ctx.log(LogEvent::CardsGiven {
                            from: self.owner,
                            to: self.owner,
                            cards: gained,
                        });
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 冰封·封：翻回正面 → 移除自身
            HostBehavior::Bingfeng2 => {
                if let Some(def) = ctx.catalog.skill_by_name(SWD_BINGFENG2_KEY) {
                    ctx.core.skills.remove(&(self.owner, def));
                    ctx.log(LogEvent::SkillToggled {
                        player: self.owner,
                        skill: ctx.catalog.skill_name(def),
                        enabled: false,
                    });
                }
                Flow::Done
            }
            // 魂殉志：万箭齐发 + 授予武圣/咆哮/殉 + 觉醒自身
            HostBehavior::Hxunzhi => {
                if !ctx.core.player(self.owner).alive {
                    return Flow::Done;
                }
                // 觉醒（一次性）：禁用自身实例
                if let Some(def) = ctx.catalog.skill_by_name(SWD_HXUNZHI_KEY) {
                    if let Some(inst) = ctx.core.skills.get_mut(&(self.owner, def)) {
                        inst.enabled = false;
                    }
                    ctx.log(LogEvent::SkillToggled {
                        player: self.owner,
                        skill: ctx.catalog.skill_name(def),
                        enabled: false,
                    });
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "魂殉志",
                    owner: self.owner,
                });
                // 视为对全部其他存活角色使用万箭齐发（座位序）
                let targets: Vec<PlayerId> = ctx
                    .core
                    .seat_order_from(self.owner)
                    .into_iter()
                    .filter(|p| *p != self.owner && ctx.core.player(*p).alive)
                    .collect();
                if !targets.is_empty() {
                    ctx.use_virtual_card(
                        self.owner,
                        crate::catalog::CardKind::ArcheryAttack,
                        targets,
                    );
                }
                // 授予武圣/咆哮（标准目录技）+ 殉（旗标）
                for name in ["武圣", "咆哮"] {
                    let Some(def) = ctx.catalog.skill_by_name(name) else {
                        continue;
                    };
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
                        ctx.log(LogEvent::SkillToggled {
                            player: self.owner,
                            skill: ctx.catalog.skill_name(def),
                            enabled: true,
                        });
                    }
                }
                if let Some(def) = ctx.catalog.skill_by_name(SWD_HXUNZHI2_KEY) {
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
                Flow::Done
            }
            HostBehavior::Hxunzhi2 => Flow::Done,
            // 天香·受：按本次伤害量摸牌后移除自身
            HostBehavior::Tianxiang2 => {
                let amount = ctx.current_damage().map(|d| d.amount).unwrap_or(0);
                if let Some(def) = ctx.catalog.skill_by_name(SWD_TIANXIANG2_KEY) {
                    ctx.core.skills.remove(&(self.owner, def));
                }
                if amount > 0 {
                    ctx.start_draw(self.owner, amount as u32);
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "天香·受",
                    owner: self.owner,
                });
                Flow::Done
            }
            // 卫统：登记面（实际加成由潜龙内容按同条件判定）
            HostBehavior::MbWeitong => Flow::Done,
            // 隐月·判：取消翻面（写回前置帧取消位）
            HostBehavior::HlssYueyin2 => {
                let (_, cancelled) = ctx.current_turn_over_before_mut();
                *cancelled = true;
                Flow::Done
            }
            HostBehavior::ShaDamageForceDiscard => {
                if let Some(t) = self.target {
                    if ctx.core.player(t).alive {
                        if let Some(c) = ctx.core.player(t).hand.first().copied() {
                            ctx.discard_from_hand(t, c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: t,
                                cards: vec![c],
                            });
                        }
                    }
                }
                Flow::Done
            }
            HostBehavior::ShengkeCheck => {
                // 被克 → 弃一张手牌（确定性首张，登记差异：上游为目标
                // 自选「he」）；被生 → 摸一张；无关系 → 无事
                let card = match self.card {
                    Some(c) => c,
                    None => return Flow::Done,
                };
                let cnat = match ctx.core.card_natures.get(&card).copied() {
                    Some(n) => n,
                    None => return Flow::Done,
                };
                let onat = match ctx.core.player(self.owner).wunature {
                    Some(n) => n as usize,
                    None => return Flow::Done,
                };
                if KE[cnat as usize] == onat as u8 {
                    if let Some(c) = ctx.core.player(self.owner).hand.first().copied() {
                        ctx.discard_from_hand(self.owner, c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: self.owner,
                            cards: vec![c],
                        });
                    }
                } else if SHENG[cnat as usize] == onat as u8 {
                    ctx.start_draw(self.owner, 1);
                }
                Flow::Done
            }
            HostBehavior::SijiDiscardDraw => {
                let n = self
                    .cards
                    .iter()
                    .filter(|c| ctx.catalog.card_kind(**c).is_sha_like())
                    .count() as u32;
                if n > 0 {
                    ctx.start_draw(self.owner, 2 * n);
                }
                Flow::Done
            }
            HostBehavior::MoshouImmunity => {
                // 墨守（玩点论杀/费诗，规则 v36）：锁定——延迟锦囊（乐不思蜀、
                // 兵粮寸断）不能指定持有者为目标。上游仅 mod.targetEnabled；
                // 效果在 `blocks_delayed_trick`（同文件 2755 行）与延迟锦囊目标
                // 过滤，宿主无机器步骤。
                Flow::Done
            }
            HostBehavior::ShuigongCard => {
                // 多目标强制弃装备（登记差异：上游为目标自选，原生取首张）
                let mut any = false;
                for t in &self.targets {
                    if ctx.core.player(*t).alive {
                        if let Some(c) = ctx.core.player(*t).equips.first().copied() {
                            ctx.core.player_mut(*t).equips.retain(|x| *x != c);
                            crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, *t, c);
                            ctx.to_discard(c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: *t,
                                cards: vec![c],
                            });
                            any = true;
                        }
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                let _ = any;
                Flow::Done
            }
            HostBehavior::ChenhuodajieCard => {
                // 获得目标一张手牌（登记差异：确定性首张）
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if let Some(c) = ctx.core.player(t).hand.first().copied() {
                    ctx.core.player_mut(t).hand.retain(|x| *x != c);
                    ctx.core.player_mut(self.owner).hand.push(c);
                    ctx.log(LogEvent::CardsGiven {
                        from: t,
                        to: self.owner,
                        cards: vec![c],
                    });
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::ZhaomingdanCard => {
                // 弃目标一张手牌（确定性首张），目标与使用者各摸 1
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if let Some(c) = ctx.core.player(t).hand.first().copied() {
                    ctx.discard_from_hand(t, c);
                    ctx.log(LogEvent::CardDiscarded {
                        player: t,
                        cards: vec![c],
                    });
                }
                ctx.start_draw(t, 1);
                ctx.start_draw(self.owner, 1);
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::JihuocardDraw => {
                ctx.start_draw(self.owner, 1);
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::ShezhanqunruCompare => {
                // 与所有有手牌的响应者拼点（确定性首张）：
                // 响应者首张点数 > 使用者首张点数 → 响应者摸 1；否则使用者摸 1
                let own_rank = ctx
                    .core
                    .player(self.owner)
                    .hand
                    .first()
                    .map(|c| ctx.catalog.cards[*c as usize].rank);
                let Some(own_r) = own_rank else {
                    return Flow::Done;
                };
                for t in &self.targets {
                    let Some(t_card) = ctx.core.player(*t).hand.first().copied() else {
                        continue;
                    };
                    let t_r = ctx.catalog.cards[t_card as usize].rank;
                    if t_r > own_r {
                        ctx.start_draw(*t, 1);
                    } else {
                        ctx.start_draw(self.owner, 1);
                    }
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::JinlianzhuCancel => match self.stage {
                HostStage::Act => {
                    // 伤害前置：持有者确认是否打出金莲烛
                    if !ctx.core.player(self.owner).alive {
                        return Flow::Done;
                    }
                    let has = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .any(|c| ctx.catalog.card_kind(*c) == CardKind::Jinlianzhu);
                    if !has {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::Confirm,
                            prompt: GJ_JINLIANZHU_CONFIRM,
                            options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    // 确认：弃一张金莲烛（确定性首张）+ 防止此伤害 +
                    // 来源摸 1；拒绝：本持有者放弃（其余持有者由各自
                    // 监听技实例继续询问）
                    if matches!(ctx.answer_first(), ChoiceOption::Confirm) {
                        let candle = ctx
                            .core
                            .player(self.owner)
                            .hand
                            .iter()
                            .copied()
                            .find(|c| ctx.catalog.card_kind(*c) == CardKind::Jinlianzhu);
                        if let Some(c) = candle {
                            ctx.core.player_mut(self.owner).hand.retain(|x| *x != c);
                            ctx.log(LogEvent::CardDiscarded {
                                player: self.owner,
                                cards: vec![c],
                            });
                        }
                        ctx.current_damage_mut().prevented = true;
                        let source = self.targets.first().copied();
                        if let Some(src) = source {
                            if ctx.core.player(src).alive {
                                ctx.start_draw(src, 1);
                            }
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::HuanleMonkeyCard => {
                // 确认后（OptionalGate 通过）：取消桃的结算——
                // ① 使用者手牌中的桃移入自己手牌；② 弃掉自己的猴子
                // （离场撤销装备技）；③ 写回取消位，提交阶段跳过结算。
                let (used_card, user) = {
                    let u = ctx.current_card_use_mut();
                    (u.card.expect("physical Peach source"), u.player)
                };
                // ① 夺桃：使用者手牌 → 自己手牌。
                let user_hand = &mut ctx.core.players[user as usize].hand;
                let pos = user_hand
                    .iter()
                    .position(|c| *c == used_card)
                    .expect("被取消使用的桃应在使用者手牌");
                let peach = user_hand.remove(pos);
                ctx.core.players[self.owner as usize].hand.push(peach);
                ctx.log(LogEvent::CardsGiven {
                    from: user,
                    to: self.owner,
                    cards: vec![peach],
                });
                // ② 弃猴子（装备区 → 弃牌堆 + 离场撤销装备技）。
                if let Some(pos) = ctx.core.players[self.owner as usize]
                    .equips
                    .iter()
                    .position(|c| ctx.catalog.card_kind(*c) == crate::catalog::CardKind::Monkey)
                {
                    let monkey = ctx.core.players[self.owner as usize].equips.remove(pos);
                    crate::kernel::revoke_equip_skills(
                        &mut ctx.core,
                        ctx.catalog,
                        self.owner,
                        monkey,
                    );
                    ctx.core.discard.push(monkey);
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: vec![monkey],
                    });
                }
                // ③ 写回取消位：提交阶段跳过桃的结算。
                ctx.current_card_use_mut().cancelled = true;
                Flow::Done
            }
            HostBehavior::HuanleMianjuCard => {
                // 强制锁定：写回翻面取消位，提交阶段跳过翻转。
                let (_p, c) = ctx.current_turn_over_before_mut();
                *c = true;
                Flow::Done
            }
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}
