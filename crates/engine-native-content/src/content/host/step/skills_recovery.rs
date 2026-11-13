use super::super::*;

impl HostMachine {
    pub(super) fn step_skills_recovery(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 狂骨：选「回复 1」或「摸 1」
            HostBehavior::FzXinKuanggu => match self.stage {
                HostStage::Act => {
                    self.stage = HostStage::SelectNature;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: "fz_xinkuanggu_choice",
                            // 0=回复 1 点体力；1=摸 1 张牌
                            options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectNature => {
                    let Some(ChoiceOption::Tag(choice)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if choice == 0 {
                        ctx.recover(self.owner, 1, None);
                    } else {
                        ctx.start_draw(self.owner, 1);
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "狂骨",
                        owner: self.owner, presentation: None });
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 血量翻倍：体力上限 + 当前体力并回复等量（一次性，`extension.js:2261-2269`）
            HostBehavior::XiguaXueliangFanbei => {
                let hp = ctx.core.player(self.owner).hp.max(0);
                if hp > 0 {
                    ctx.core.player_mut(self.owner).max_hp += hp as u32;
                    ctx.recover(self.owner, hp as u32, None);
                }
                ctx.core
                    .vm_aux
                    .flags
                    .insert((self.owner, "xigua_xueliangfanbei_done".to_string()), 1);
                if let Some(def) = ctx
                    .catalog
                    .skill_by_name(&self.def_key)
                    .or_else(|| ctx.catalog.skill_by_name(L20_XUELIANG_KEY))
                {
                    ctx.core
                        .vm_aux
                        .marks
                        .insert((self.owner, u32::from(def)), 1);
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "血量翻倍",
                    owner: self.owner, presentation: None });
                Flow::Done
            }
            // 誓师：体力上限 +1 并回复 1
            HostBehavior::Shishi => {
                ctx.core.player_mut(self.owner).max_hp += 1;
                ctx.recover(self.owner, 1, None);
                ctx.log(LogEvent::SkillTriggered {
                    skill: "誓师",
                    owner: self.owner, presentation: None });
                Flow::Done
            }
            // 魔化：移除旧三技 → 获得撒旦四技 → 上限 4 → 回复 2 → 摸 2
            HostBehavior::Mohua2 => {
                for name in ["miles_xueyi", "aojian", "mohua2"] {
                    if let Some(def) = ctx.catalog.skill_by_name(name) {
                        if ctx.core.skills.remove(&(self.owner, def)).is_some() {
                            ctx.log(LogEvent::SkillToggled {
                                player: self.owner,
                                skill: ctx.catalog.skill_name(def),
                                enabled: false,
                            });
                        }
                    }
                }
                for name in ["moyan", "miedao", "jifeng", "swd_xiuluo"] {
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
                    }
                    ctx.log(LogEvent::SkillToggled {
                        player: self.owner,
                        skill: ctx.catalog.skill_name(def),
                        enabled: true,
                    });
                }
                // 撒旦形态体力上限 4（上游 lib.character.swd_satan 同口径）
                {
                    let p = ctx.core.player_mut(self.owner);
                    p.max_hp = 4;
                    if p.hp > 4 {
                        p.hp = 4;
                    }
                }
                ctx.log(LogEvent::SkillTriggered {
                    skill: "魔化",
                    owner: self.owner, presentation: None });
                ctx.recover(self.owner, 2, None);
                ctx.start_draw(self.owner, 2);
                Flow::Done
            }
            // 脉轮：选 1..8 效果 → 立即结算（1/2）或置模式位（3..8）
            HostBehavior::Mailun => match self.stage {
                HostStage::Act => {
                    let max_hp = ctx.core.player(self.owner).max_hp;
                    // 上游：maxHp==6 时不提供效果 1
                    let options: Vec<ChoiceOption> = (1..=8u8)
                        .filter(|i| !(*i == 1 && max_hp == 6))
                        .map(ChoiceOption::Tag)
                        .collect();
                    self.stage = HostStage::SelectNature;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::PickTag,
                            prompt: "mailun_mode",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectNature => {
                    let Some(ChoiceOption::Tag(mode)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "脉轮",
                        owner: self.owner, presentation: None });
                    // 清除旧效果（上游 step0 移除 mailun31..8）
                    {
                        let p = ctx.core.player_mut(self.owner);
                        p.mailun_mode = 0;
                        p.mailun_out_used = false;
                        p.mailun_in_used = false;
                    }
                    match mode {
                        1 => {
                            // 减少一点体力并增加一点体力上限
                            let max = ctx.core.player(self.owner).max_hp;
                            ctx.emit(EventData::LoseHp {
                                player: self.owner,
                                amount: 1,
                            });
                            ctx.core.player_mut(self.owner).max_hp = max + 1;
                        }
                        2 => {
                            // 增加一点体力并减少一点体力上限
                            ctx.recover(self.owner, 1, None);
                            ctx.lose_max_hp(self.owner, 1);
                        }
                        m => {
                            ctx.core.player_mut(self.owner).mailun_mode = m;
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 护天：选任意张牌 → 选体力上限足够的目标 → 寄存 + 补至张数
            HostBehavior::Hutian => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(self.owner)
                        .hand
                        .iter()
                        .chain(ctx.core.player(self.owner).equips.iter())
                        .copied()
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    let max = options.len();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "hutian_cards",
                            options,
                            min: 1,
                            max,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let picked: Vec<CardId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    if picked.is_empty() {
                        return Flow::Done;
                    }
                    self.cards = picked.iter().map(|c| u32::from(*c)).collect();
                    let need = picked.len() as u16;
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| *p != self.owner && ctx.core.player(*p).max_hp as u16 >= need)
                        .map(ChoiceOption::Player)
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "hutian_target",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(target).alive {
                        return Flow::Done;
                    }
                    let cards: Vec<CardId> = self.cards.iter().map(|c| *c as CardId).collect();
                    let count = cards.len() as i32;
                    // 牌离场入寄存区（手牌移除 / 装备撤技后移除；不入弃堆）
                    for c in cards.iter().copied() {
                        if ctx.core.player(self.owner).hand.contains(&c) {
                            ctx.take_hand_card(self.owner, c);
                        } else if ctx.core.player(self.owner).equips.contains(&c) {
                            ctx.core.player_mut(self.owner).equips.retain(|x| *x != c);
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                self.owner,
                                c,
                            );
                        } else {
                            return Flow::Done;
                        }
                    }
                    {
                        let p = ctx.core.player_mut(self.owner);
                        p.hutian_deposit = Some(target);
                        p.hutian_just_given = true;
                    }
                    {
                        let t = ctx.core.player_mut(target);
                        t.hutian_owner = Some(self.owner);
                        t.hutian_hold.extend(cards.iter().copied());
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "护天",
                        owner: self.owner, presentation: None });
                    // 目标体力低于寄存数 → 回复至该数（上游 step2）
                    let hp = ctx.core.player(target).hp;
                    if hp < count {
                        ctx.recover(target, (count - hp) as u32, None);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            HostBehavior::PhaseEndRecoverOrDraw => {
                let p = ctx.core.player(self.owner);
                if p.hp < p.max_hp as i32 {
                    ctx.recover(self.owner, 1, None);
                } else {
                    ctx.start_draw(self.owner, 1);
                }
                Flow::Done
            }
            HostBehavior::WangmeiZhikeCard => {
                // 条件增益（上游 content）：最小体力且已受伤 → 回复 1；
                // 最小手牌数 → 摸（已回复 ? 1 : 2）张
                let Some(t) = self.target else {
                    return Flow::Done;
                };
                if !ctx.core.player(t).alive {
                    return Flow::Done;
                }
                let mut min_hp = i32::MAX;
                let mut min_hand = usize::MAX;
                for p in ctx.core.alive_players() {
                    let pl = ctx.core.player(p);
                    min_hp = min_hp.min(pl.hp);
                    min_hand = min_hand.min(pl.hand.len());
                }
                let tp = ctx.core.player(t);
                let damaged = tp.hp < tp.max_hp as i32;
                let is_min_hp = tp.hp == min_hp;
                let is_min_hand = tp.hand.len() == min_hand;
                let mut rec = false;
                if is_min_hp && damaged {
                    ctx.recover(t, 1, self.card);
                    rec = true;
                }
                if is_min_hand {
                    ctx.start_draw(t, if rec { 1 } else { 2 });
                }
                if let Some(c) = self.card {
                    ctx.to_discard(c);
                    ctx.core.table.retain(|x| *x != c);
                }
                Flow::Done
            }
            HostBehavior::KDanggu => match self.stage {
                HostStage::Act => {
                    let returning = scs_flag(ctx.core, self.owner, "k_mowang_rest_returning") > 0;
                    if returning {
                        set_scs_flag(ctx.core, self.owner, "k_mowang_rest_returning", 0);
                        if scs_flag(ctx.core, self.owner, "k_danggu_inited") == 0 {
                            set_scs_flag(ctx.core, self.owner, "k_danggu_inited", 1);
                            if scs_flag(ctx.core, self.owner, "k_danggu_roster_mask") == 0
                                && scs_flag(ctx.core, self.owner, "k_mowang_exhausted") == 0
                            {
                                set_scs_flag(ctx.core, self.owner, "k_danggu_roster_mask", 0x3FF);
                            }
                        }
                        draw_two_for_danggu_back(ctx, self.owner);
                    } else if scs_flag(ctx.core, self.owner, "k_danggu_inited") == 0 {
                        set_scs_flag(ctx.core, self.owner, "k_danggu_inited", 1);
                        if scs_flag(ctx.core, self.owner, "k_danggu_roster_mask") == 0
                            && scs_flag(ctx.core, self.owner, "k_mowang_exhausted") == 0
                        {
                            set_scs_flag(ctx.core, self.owner, "k_danggu_roster_mask", 0x3FF);
                        }
                    }
                    self.run_k_danggu_clique(ctx)
                }
                HostStage::SelectNature => self.finish_k_danggu_partner(ctx),
                _ => Flow::Done,
            },
            HostBehavior::KMowang => match self.stage {
                HostStage::Act => {
                    let branch = self.card.unwrap_or_else(|| {
                        if ctx.core.player(self.owner).dying || ctx.core.player(self.owner).hp <= 0 {
                            0
                        } else if scs_flag(ctx.core, self.owner, "k_mowang_resting") > 0 {
                            1
                        } else {
                            2
                        }
                    });
                    match branch {
                        0 => {
                            if scs_flag(ctx.core, self.owner, "k_danggu_roster_mask") != 0
                                && ctx.core.player(self.owner).max_hp > 0
                            {
                                enter_k_mowang_rest(ctx, self.owner, true);
                            }
                            Flow::Done
                        }
                        1 => {
                            set_scs_flag(ctx.core, self.owner, "k_mowang_resting", 0);
                            set_scs_flag(ctx.core, self.owner, "k_mowang_return_resolved", 1);
                            {
                                let max_hp = ctx.core.player(self.owner).max_hp as i32;
                                let p = ctx.core.player_mut(self.owner);
                                p.hp = max_hp;
                                p.dying = false;
                            }
                            let kdanggu_queued = ctx.res.stack.last().is_some_and(|f| {
                                f.queue.iter().any(|q| {
                                    q.owner == self.owner
                                        && ctx.catalog.skills[q.def as usize]
                                            .behavior
                                            .as_host_behavior()
                                            == Some(HostBehavior::KDanggu)
                                })
                            });
                            if kdanggu_queued {
                                set_scs_flag(ctx.core, self.owner, "k_mowang_rest_returning", 1);
                                Flow::Done
                            } else {
                                set_scs_flag(ctx.core, self.owner, "k_mowang_rest_returning", 0);
                                if scs_flag(ctx.core, self.owner, "k_danggu_inited") == 0 {
                                    set_scs_flag(ctx.core, self.owner, "k_danggu_inited", 1);
                                    if scs_flag(ctx.core, self.owner, "k_danggu_roster_mask") == 0
                                        && scs_flag(ctx.core, self.owner, "k_mowang_exhausted") == 0
                                    {
                                        set_scs_flag(
                                            ctx.core,
                                            self.owner,
                                            "k_danggu_roster_mask",
                                            0x3FF,
                                        );
                                    }
                                }
                                draw_two_for_danggu_back(ctx, self.owner);
                                self.run_k_danggu_clique(ctx)
                            }
                        }
                        _ => {
                            let roster = scs_flag(ctx.core, self.owner, "k_danggu_roster_mask");
                            if roster != 0 && ctx.core.player(self.owner).max_hp > 0 {
                                enter_k_mowang_rest(ctx, self.owner, false);
                            } else {
                                set_scs_flag(ctx.core, self.owner, "k_mowang_exhausted", 1);
                                {
                                    let p = ctx.core.player_mut(self.owner);
                                    p.hp = 0;
                                    p.dying = true;
                                }
                                ctx.emit(EventData::Dying {
                                    player: self.owner,
                                    source: None,
                                });
                            }
                            Flow::Done
                        }
                    }
                }
                HostStage::SelectNature => self.finish_k_danggu_partner(ctx),
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }

    fn run_k_danggu_clique(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        revoke_k_danggu_companions(ctx, self.owner);
        let mut roster = scs_flag(ctx.core, self.owner, "k_danggu_roster_mask");
        if roster == 0 {
            return Flow::Done;
        }
        let available: Vec<u32> = (0..10u32)
            .filter(|&i| (roster & (1u32 << i)) != 0)
            .collect();
        if available.is_empty() {
            return Flow::Done;
        }
        let pick_pos = ctx.core.rng.gen_range(available.len() as u32) as usize;
        let first = available[pick_pos];
        roster &= !(1u32 << first);
        set_scs_flag(ctx.core, self.owner, "k_danggu_roster_mask", roster);
        set_scs_flag(ctx.core, self.owner, "k_danggu_first", first);
        set_scs_flag(ctx.core, self.owner, "k_danggu_active_eunuch", first + 1);
        set_scs_flag(ctx.core, self.owner, "k_danggu_triggered", 1);

        let mut rem: Vec<u32> = (0..10u32)
            .filter(|&i| (roster & (1u32 << i)) != 0)
            .collect();
        let mut others = Vec::with_capacity(rem.len().min(4));
        while others.len() < 4 && !rem.is_empty() {
            let j = ctx.core.rng.gen_range(rem.len() as u32) as usize;
            others.push(rem.remove(j));
        }
        if (roster & (1u32 << 9)) != 0 && others.len() == 4 && !others.contains(&9) {
            others[3] = 9;
        }
        let mut selectable: Vec<u32> = others
            .iter()
            .copied()
            .filter(|&c| !k_danggu_conflicts(first, c))
            .collect();
        if selectable.is_empty() {
            selectable = others;
        }
        if selectable.is_empty() {
            set_scs_flag(ctx.core, self.owner, "k_danggu_active_mask", 1u32 << first);
            set_scs_flag(ctx.core, self.owner, "k_danggu_back_resolved", 1);
            grant_k_danggu_companion(ctx, self.owner, first);
            return Flow::Done;
        }
        self.card = Some(first);
        self.cards = selectable.clone();
        self.stage = HostStage::SelectNature;
        Flow::WaitChoice {
            observer: self.owner,
            spec: ChoiceSpec {
                prompt: K_DANGGU_PARTNER,
                kind: ChoiceKind::PickTag,
                options: selectable
                    .into_iter()
                    .map(|t| ChoiceOption::Tag(t as u8))
                    .collect(),
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        }
    }

    fn finish_k_danggu_partner(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let Some(ChoiceOption::Tag(chosen_tag)) = ctx.answer().first().copied() else {
            return Flow::Done;
        };
        let chosen = u32::from(chosen_tag);
        let first = self
            .card
            .unwrap_or_else(|| scs_flag(ctx.core, self.owner, "k_danggu_first"));
        let roster = scs_flag(ctx.core, self.owner, "k_danggu_roster_mask") & !(1u32 << chosen);
        set_scs_flag(ctx.core, self.owner, "k_danggu_roster_mask", roster);
        let active_mask = (1u32 << first) | (1u32 << chosen);
        set_scs_flag(ctx.core, self.owner, "k_danggu_active_mask", active_mask);
        set_scs_flag(ctx.core, self.owner, "k_danggu_partner_eunuch", chosen + 1);
        set_scs_flag(ctx.core, self.owner, "k_danggu_back_resolved", 1);
        grant_k_danggu_companion(ctx, self.owner, first);
        grant_k_danggu_companion(ctx, self.owner, chosen);
        Flow::Done
    }
}

const L10_SCS_COMPANION_SKILLS: [&str; 10] = [
    "scstaoluan",
    "scschiyan",
    "scszimou",
    "scspicai",
    "scsyaozhuo",
    "scsxiaolu",
    "scskuiji",
    "scschihe",
    "scsniqu",
    "scsanruo",
];

fn scs_flag(core: &crate::state::GameStateCore, owner: PlayerId, key: &str) -> u32 {
    core.vm_aux
        .flags
        .get(&(owner, key.to_string()))
        .copied()
        .unwrap_or(0)
}

fn set_scs_flag(core: &mut crate::state::GameStateCore, owner: PlayerId, key: &str, val: u32) {
    core.vm_aux.flags.insert((owner, key.to_string()), val);
}

fn k_danggu_conflicts(a: u32, b: u32) -> bool {
    matches!(
        (a.min(b), a.max(b)),
        (3, 5) | (7, 8) | (3, 9) | (5, 9) | (7, 9) | (8, 9)
    )
}

fn revoke_k_danggu_companions(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let active_mask = scs_flag(ctx.core, owner, "k_danggu_active_mask");
    for (idx, &sk) in L10_SCS_COMPANION_SKILLS.iter().enumerate() {
        if (active_mask & (1u32 << idx)) != 0
            || scs_flag(ctx.core, owner, &format!("l10_scs_skill:{sk}")) != 0
        {
            ctx.core
                .vm_aux
                .flags
                .remove(&(owner, format!("l10_scs_skill:{sk}")));
            if sk == "scschihe" {
                ctx.core
                    .vm_aux
                    .flags
                    .remove(&(owner, "l10_scschihe_sha_boost".to_string()));
            }
            if let Some(def) = ctx.catalog.skill_by_name(sk) {
                if ctx.core.skills.remove(&(owner, def)).is_some() {
                    ctx.log(LogEvent::SkillToggled {
                        player: owner,
                        skill: ctx.catalog.skill_name(def),
                        enabled: false,
                    });
                }
            }
        }
    }
    set_scs_flag(ctx.core, owner, "k_danggu_active_mask", 0);
}

fn grant_k_danggu_companion(ctx: &mut Ctx<'_>, owner: PlayerId, idx: u32) {
    let Some(&sk) = L10_SCS_COMPANION_SKILLS.get(idx as usize) else {
        return;
    };
    set_scs_flag(ctx.core, owner, &format!("l10_scs_skill:{sk}"), 1);
    if sk == "scschihe" {
        set_scs_flag(ctx.core, owner, "l10_scschihe_sha_boost", 1);
    }
    if let Some(def) = ctx.catalog.skill_by_name(sk) {
        if !ctx.core.skills.contains_key(&(owner, def)) {
            ctx.insert_skill(
                (owner, def),
                crate::state::SkillInstance {
                    owner,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: false,
                    rounds: 0,
                },
            );
        } else if let Some(inst) = ctx.core.skills.get_mut(&(owner, def)) {
            inst.enabled = true;
        }
        ctx.log(LogEvent::SkillToggled {
            player: owner,
            skill: ctx.catalog.skill_name(def),
            enabled: true,
        });
    }
}

fn draw_two_for_danggu_back(ctx: &mut Ctx<'_>, owner: PlayerId) {
    let mut drawn = Vec::with_capacity(2);
    for _ in 0..2 {
        if ctx.core.deck.is_empty() && !ctx.core.discard.is_empty() {
            let mut cards = std::mem::take(&mut ctx.core.discard);
            ctx.core.rng.shuffle(&mut cards);
            crate::kernel::elo_on_wash_deck(ctx.core, &cards);
            ctx.core.deck = cards;
            ctx.out.push(LogEvent::DeckReshuffled);
        }
        if let Some(c) = ctx.core.deck.pop() {
            ctx.core.player_mut(owner).hand.push(c);
            drawn.push(c);
        }
    }
    if !drawn.is_empty() {
        ctx.out.push(LogEvent::CardsDrawn {
            player: owner,
            cards: drawn,
        });
    }
}

fn enter_k_mowang_rest(ctx: &mut Ctx<'_>, owner: PlayerId, from_dying: bool) {
    set_scs_flag(ctx.core, owner, "k_mowang_resting", 1);
    set_scs_flag(ctx.core, owner, "k_mowang_revived", 1);
    let before = ctx.core.player(owner).hp;
    if from_dying && before < 1 {
        ctx.recover(owner, (1 - before) as u32, None);
    }
    {
        let p = ctx.core.player_mut(owner);
        p.hp = 1;
        p.dying = false;
        p.linked = false;
        p.face_down = false;
    }
    revoke_k_danggu_companions(ctx, owner);
    let all_cards: Vec<CardId> = ctx
        .core
        .player(owner)
        .hand
        .iter()
        .chain(ctx.core.player(owner).equips.iter())
        .chain(ctx.core.player(owner).judge.iter())
        .copied()
        .collect();
    for c in all_cards.iter().copied() {
        ctx.take_owned_card(owner, c);
        ctx.to_discard(c);
    }
    if !all_cards.is_empty() {
        ctx.log(LogEvent::CardDiscarded {
            player: owner,
            cards: all_cards,
        });
    }
}
