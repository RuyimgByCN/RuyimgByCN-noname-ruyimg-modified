//! 宿主技能：声明或转化后使用牌（起风、集控、蛊惑、极略、此桥邪）。

use super::super::*;

impl HostMachine {
    pub(super) fn step_skills_virtual_use(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 起风：选一张未用过的可用锦囊牌名并使用（上游 chooseButton
            // + chooseUseTarget；原生以目录牌种码承载牌名选择）
            HostBehavior::HlssFengqi => match self.stage {
                HostStage::Act => {
                    let options: Vec<ChoiceOption> = hlss_fengqi_candidates(ctx, self.owner)
                        .into_iter()
                        .map(|k| ChoiceOption::Kind(crate::archive::kind_code(k)))
                        .collect();
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "fengqi_card",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Kind(code)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let Ok(kind) = crate::archive::kind_from_code(code) else {
                        return Flow::Done;
                    };
                    let (min, max) = hlss_fengqi_targets(kind);
                    if max == 0 {
                        // 无目标锦囊：直接视为使用（无中/五谷/桃园/南蛮/万箭）
                        let targets = hlss_fengqi_all_targets(ctx, self.owner, kind);
                        ctx.use_virtual_card(self.owner, kind, targets);
                        ctx.core.player_mut(self.owner).hlss_fengqi_used.push(code);
                        return Flow::Done;
                    }
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .alive_players()
                        .filter(|p| hlss_fengqi_target_ok(ctx, self.owner, *p, kind))
                        .map(ChoiceOption::Player)
                        .collect();
                    if (options.len() as u8) < min {
                        return Flow::Done;
                    }
                    self.cards.push(u32::from(code));
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "fengqi_targets",
                            max: options.len().min(max as usize),
                            options,
                            min: min as usize,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(code) = self.cards.first().copied() else {
                        return Flow::Done;
                    };
                    let Ok(kind) = crate::archive::kind_from_code(code as u16) else {
                        return Flow::Done;
                    };
                    let targets: Vec<PlayerId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    if targets.is_empty() {
                        return Flow::Done;
                    }
                    ctx.use_virtual_card(self.owner, kind, targets);
                    ctx.core
                        .player_mut(self.owner)
                        .hlss_fengqi_used
                        .push(code as u16);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 集控：选攻击距离内一名其他角色 → 虚拟雷杀（取消=不消耗次数）
            // 每步：弃一张牌 → 令目标本回合止息（+ 条件性距离视为 1）
            HostBehavior::NewMeibu => match self.stage {
                HostStage::Act => {
                    let Some(target) = self.target else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(target).alive {
                        return Flow::Done;
                    }
                    let mut options: Vec<ChoiceOption> = ctx
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
                    options.truncate(64);
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "new_meibu_discard",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(target) = self.target else {
                        return Flow::Done;
                    };
                    let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(target).alive {
                        return Flow::Done;
                    }
                    let is_sha = ctx.catalog.card_kind(card).is_sha_like();
                    let is_trick = {
                        let k = ctx.catalog.card_kind(card);
                        !k.is_basic_kind() && k.equip_slot().is_none()
                    };
                    let is_black = matches!(
                        ctx.catalog.cards[card as usize].suit,
                        crate::catalog::Suit::Spade | crate::catalog::Suit::Club
                    );
                    if ctx.core.player(self.owner).hand.contains(&card) {
                        ctx.discard_from_hand(self.owner, card);
                    } else if ctx.core.player(self.owner).equips.contains(&card) {
                        ctx.core
                            .player_mut(self.owner)
                            .equips
                            .retain(|c| *c != card);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, self.owner, card);
                        ctx.to_discard(card);
                    }
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "每步",
                        owner: self.owner, presentation: None });
                    if let Some(def) = ctx.catalog.skill_by_name(L27_NEW_ZHIXI_KEY) {
                        ctx.insert_skill(
                            (target, def),
                            crate::state::SkillInstance {
                                owner: target,
                                enabled: true,
                                once_used: false,
                                expires_end_of_turn: true,
                                rounds: 0,
                            },
                        );
                        // 止息初始使用计数为 0，清理挂于出牌阶段结束（extension.js:1735-1748）
                        ctx.core.vm_aux.marks.remove(&(target, u32::from(def)));
                        if let Some(m2) = ctx.catalog.skill_by_name(L27_NEW_ZHIXI_MARK2_KEY) {
                            ctx.core.vm_aux.marks.remove(&(target, u32::from(m2)));
                        }
                        ctx.core
                            .vm_aux
                            .flags
                            .remove(&(target, "new_zhixi".to_string()));
                        ctx.core
                            .vm_aux
                            .flags
                            .remove(&(target, "new_zhixi_mark2".to_string()));
                        ctx.core.vm_aux.flags.insert(
                            (target, "l27_temp_phase_use:new_zhixi".to_string()),
                            1,
                        );
                    }
                    if let Some(rec_def) = ctx.catalog.skill_by_name("new_zhixi_recover") {
                        ctx.insert_skill(
                            (target, rec_def),
                            crate::state::SkillInstance {
                                owner: target,
                                enabled: true,
                                once_used: false,
                                expires_end_of_turn: true,
                                rounds: 0,
                            },
                        );
                        ctx.core.vm_aux.flags.insert(
                            (target, "l27_temp_phase_use:new_zhixi_recover".to_string()),
                            1,
                        );
                    }
                    if !is_sha && !is_trick && !is_black {
                        if let Some(def) = ctx.catalog.skill_by_name(L27_NEW_MEIBU_RANGE_KEY) {
                            ctx.insert_skill(
                                (target, def),
                                crate::state::SkillInstance {
                                    owner: target,
                                    enabled: true,
                                    once_used: false,
                                    expires_end_of_turn: true,
                                    rounds: 0,
                                },
                            );
                            ctx.core
                                .vm_aux
                                .marks
                                .insert((target, u32::from(def)), i32::from(self.owner));
                            ctx.core.vm_aux.flags.insert(
                                (target, "l27_temp_phase_use:new_meibu_range".to_string()),
                                1,
                            );
                        }
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 蛊惑：选牌 → 声明基本/锦囊牌名 → (可选)选目标 → 质疑 → 结算/后效
            HostBehavior::Reguhuo => match self.stage {
                // `reguhuo_phase`: 仅进入质疑时写入 `vm_aux.once`（取消不消耗）
                HostStage::Act => super::l10_reguhuo::step_act(self, ctx),
                HostStage::SelectCard => super::l10_reguhuo::step_select_card(self, ctx),
                HostStage::SelectNature => super::l10_reguhuo::step_select_nature(self, ctx),
                HostStage::TargetChoice => super::l10_reguhuo::step_target_choice(self, ctx),
                HostStage::ReguhuoGuessPick => super::l10_reguhuo::step_guess_pick(self, ctx),
                HostStage::DrawSettle => super::l10_reguhuo::step_draw_settle(self, ctx),
                HostStage::ReguhuoFinalPick => super::l10_reguhuo::step_final_pick(self, ctx),
                _ => Flow::Done,
            },
            // 极略（swd/skill.js:3308）：看牌堆顶 3 张（不足 3 张且弃牌堆非空时先洗入弃牌堆并保留既有堆顶）→ 选一张可使用的基本/锦囊牌 → 选目标（含借刀两段）→ 置桌面使用
            HostBehavior::Jilve => match self.stage {
                HostStage::Act => {
                    if ctx.core.deck.len() < 3 && !ctx.core.discard.is_empty() {
                        let existing_top = std::mem::take(&mut ctx.core.deck);
                        let mut washed = std::mem::take(&mut ctx.core.discard);
                        ctx.core.rng.shuffle(&mut washed);
                        crate::kernel::elo_on_wash_deck(ctx.core, &washed);
                        washed.extend(existing_top);
                        ctx.core.deck = washed;
                        ctx.log(LogEvent::DeckReshuffled);
                    }
                    let options = jilve_card_options(ctx.core, ctx.catalog, self.owner);
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "jilve_card",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !ctx.core.deck.contains(&card)
                        || !jilve_card_playable(ctx.core, ctx.catalog, self.owner, card)
                    {
                        return Flow::Done;
                    }
                    let kind = ctx.catalog.card_kind(card);
                    let (min, max, targets) =
                        jilve_legal_targets(ctx.core, ctx.catalog, kind, self.owner);
                    self.card = Some(card);
                    if min == 0 && max == 0 {
                        // 无须手选目标（无中生有 targets=[], AOE targets=合法目标列表）：牌堆 → 处理区 table → CardUsed
                        ctx.core.deck.retain(|c| *c != card);
                        ctx.core.table.push(card);
                        ctx.core.player_mut(self.owner).jilve_used += 1;
                        ctx.log(LogEvent::SkillTriggered {
                            skill: "极略",
                            owner: self.owner,
                            presentation: None,
                        });
                        ctx.emit(EventData::CardUsed(crate::machine::card_use(
                            card,
                            self.owner,
                            targets,
                        )));
                        self.stage = HostStage::ShuchuiTargetLoop;
                        return Flow::Yield;
                    }
                    if targets.is_empty() {
                        return Flow::Done;
                    }
                    self.stage = HostStage::TargetChoice;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectPlayers,
                            prompt: "jilve_targets",
                            options: targets.into_iter().map(ChoiceOption::Player).collect(),
                            min: min as usize,
                            max: max as usize,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::TargetChoice => {
                    let Some(card) = self.card else {
                        return Flow::Done;
                    };
                    let targets: Vec<PlayerId> = ctx
                        .answer()
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Player(p) => Some(*p),
                            _ => None,
                        })
                        .collect();
                    if targets.is_empty() || !ctx.core.deck.contains(&card) {
                        return Flow::Done;
                    }
                    if ctx.catalog.card_kind(card) == CardKind::Jiedao {
                        let holder = targets[0];
                        let victims = ctx.core.legal_jiedao_victims(ctx.catalog, holder);
                        if victims.is_empty() {
                            return Flow::Done;
                        }
                        self.target = Some(holder);
                        self.stage = HostStage::JiefenTarget;
                        return Flow::WaitChoice {
                            observer: self.owner,
                            spec: ChoiceSpec {
                                kind: ChoiceKind::SelectPlayers,
                                prompt: "jilve_jiedao_victim",
                                options: victims.into_iter().map(ChoiceOption::Player).collect(),
                                min: 1,
                                max: 1,
                                allow_cancel: false,
                            },
                        };
                    }
                    ctx.core.deck.retain(|c| *c != card);
                    ctx.core.table.push(card);
                    ctx.core.player_mut(self.owner).jilve_used += 1;
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "极略",
                        owner: self.owner,
                        presentation: None,
                    });
                    ctx.emit(EventData::CardUsed(crate::machine::card_use(
                        card, self.owner, targets,
                    )));
                    self.stage = HostStage::ShuchuiTargetLoop;
                    Flow::Yield
                }
                HostStage::JiefenTarget => {
                    let Some(card) = self.card else {
                        return Flow::Done;
                    };
                    let Some(holder) = self.target else {
                        return Flow::Done;
                    };
                    let Some(ChoiceOption::Player(victim)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    if !ctx.core.deck.contains(&card) {
                        return Flow::Done;
                    }
                    ctx.core.deck.retain(|c| *c != card);
                    ctx.core.table.push(card);
                    ctx.core.player_mut(self.owner).jilve_used += 1;
                    ctx.log(LogEvent::SkillTriggered {
                        skill: "极略",
                        owner: self.owner,
                        presentation: None,
                    });
                    ctx.emit(EventData::CardUsed(crate::machine::card_use(
                        card,
                        self.owner,
                        vec![holder, victim],
                    )));
                    self.stage = HostStage::ShuchuiTargetLoop;
                    Flow::Yield
                }
                HostStage::ShuchuiTargetLoop => Flow::Done,
                _ => Flow::Done,
            },
            // 此桥邪：失去装备后逐次使用机关牌（自目标/单目标/AOE）
            HostBehavior::Cyqiaoxie => match self.stage {
                HostStage::Act => {
                    let lost_equips =
                        match ctx.res.frame(ctx.current_event_frame_id()).map(|f| &f.data) {
                            Some(EventData::CardsLost { cards, origins, .. }) => cards
                                .iter()
                                .enumerate()
                                .filter(|(i, c)| {
                                    origins.get(*i) == Some(&crate::event::LostOrigin::Equip)
                                        || ctx.catalog.card_kind(**c).is_equip()
                                })
                                .count(),
                            _ => 0,
                        };
                    if lost_equips == 0 {
                        return Flow::Done;
                    }
                    self.cards = (0..lost_equips).map(|_| 0u32).collect();
                    self.stage = HostStage::ShuchuiTargetLoop;
                    Flow::Yield
                }
                HostStage::ShuchuiTargetLoop => {
                    if self.cards.is_empty() {
                        return Flow::Done;
                    }
                    let options = cyqiaoxie_candidates(ctx, self.owner);
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    self.cards.pop();
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "cyqiaoxie_card",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectCard => {
                    let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    self.card = Some(card);
                    let name = ctx.catalog.cards[card as usize].name;
                    match cyqiaoxie_rule(name) {
                        Some(CyqiaoxieRule::AllOthers) => {
                            let targets: Vec<PlayerId> = ctx
                                .core
                                .seat_order_from(self.owner)
                                .into_iter()
                                .filter(|p| *p != self.owner && ctx.core.player(*p).alive)
                                .collect();
                            cyqiaoxie_use(ctx, self.owner, card, targets);
                            self.stage = HostStage::ShuchuiTargetLoop;
                            Flow::Yield
                        }
                        Some(CyqiaoxieRule::OneOther) => {
                            let options: Vec<ChoiceOption> = ctx
                                .core
                                .alive_players()
                                .filter(|p| *p != self.owner && cyqiaoxie_target_ok(ctx, *p, name))
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
                                    prompt: "cyqiaoxie_target",
                                    options,
                                    min: 1,
                                    max: 1,
                                    allow_cancel: false,
                                },
                            }
                        }
                        Some(CyqiaoxieRule::SelfTarget) => {
                            cyqiaoxie_use(ctx, self.owner, card, vec![self.owner]);
                            self.stage = HostStage::ShuchuiTargetLoop;
                            Flow::Yield
                        }
                        None => Flow::Done,
                    }
                }
                HostStage::TargetChoice => {
                    let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                        return Flow::Done;
                    };
                    let Some(card) = self.card else {
                        return Flow::Done;
                    };
                    cyqiaoxie_use(ctx, self.owner, card, vec![target]);
                    self.stage = HostStage::ShuchuiTargetLoop;
                    Flow::Yield
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to the wrong step module"),
        }
    }
}

pub(in crate::content::host) fn can_activate_sha_viewas(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    owner: PlayerId,
) -> bool {
    matches!(core.turn.phase, crate::event::Phase::Play)
        && core.turn.current == owner
        && !catalog.blocks_all_cards(core, owner)
        && !sha_blocked_for(core, catalog, owner)
        && !core.legal_targets(catalog, CardKind::Sha, owner).2.is_empty()
}

pub(in crate::content::host) fn sha_blocked_for(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    owner: PlayerId,
) -> bool {
    let birin = catalog
        .skill_by_name(GW_BIRIN_STATE_KEY)
        .is_some_and(|def| core.skills.get(&(owner, def)).is_some_and(|i| i.enabled));
    let shouhu = catalog
        .skill_by_name(WUDIAN_SHOUHU_KEY)
        .is_some_and(|def| core.skills.get(&(owner, def)).is_some_and(|i| i.enabled));
    let qz = catalog
        .skill_by_name(crate::vocabulary::names::BOSS_QINGZHU)
        .is_some_and(|def| core.skills.get(&(owner, def)).is_some_and(|i| i.enabled))
        && !catalog
            .skill_by_name(crate::vocabulary::names::BOSS_JIDING)
            .is_some_and(|def| core.has_skill(owner, def));
    if birin || shouhu || qz {
        return true;
    }
    if !core.turn.sha_used_play_phase {
        return false;
    }
    if core.current_land == Some(CardKind::Longlushanfeng) && !core.turn.land_sha_used {
        return false;
    }
    !super::l10_reguhuo::has_unlimited_sha(core, catalog, owner)
}

pub(in crate::content::host) fn jilve_legal_targets(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    kind: CardKind,
    owner: PlayerId,
) -> (u8, u8, Vec<PlayerId>) {
    let (min, max, mut targets) = core.legal_targets(catalog, kind, owner);
    if matches!(kind, CardKind::Le | CardKind::Bing | CardKind::Guiyoujie) {
        targets.retain(|&p| !catalog.blocks_delayed_trick(core, p));
    }
    (min, max, targets)
}

pub(in crate::content::host) fn jilve_card_playable(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    owner: PlayerId,
    card: CardId,
) -> bool {
    let kind = catalog.card_kind(card);
    if kind.is_equip() || !kind.is_basic_trick_or_equip() {
        return false;
    }
    if matches!(
        kind,
        CardKind::Shan
            | CardKind::Nullify
            | CardKind::Hufu
            | CardKind::Youdishenru
            | CardKind::Jinlianzhu
            | CardKind::Shenmiguo
            | CardKind::Jiguan
            | CardKind::Hslingjian
    ) {
        return false;
    }
    if core.player(owner).wangshi_ban && kind.is_basic_kind() {
        return false;
    }
    if kind.is_sha_like() && sha_blocked_for(core, catalog, owner) {
        return false;
    }
    if matches!(kind, CardKind::Jiu | CardKind::Tianxianjiu) && core.turn.jiu_used {
        return false;
    }
    if kind == CardKind::Tao && crate::kernel::tao_futai2_banned(core, catalog, owner) {
        return false;
    }
    if kind == CardKind::Changshezhen
        && !core.inline(catalog, owner)
        && !core
            .alive_players()
            .any(|p| p != owner && core.is_friend_of(catalog, owner, p))
    {
        return false;
    }
    if kind == CardKind::Dizaizhen
        && !core
            .alive_players()
            .any(|p| !core.is_major_faction(catalog, p))
    {
        return false;
    }
    if matches!(
        kind,
        CardKind::Tianfuzhen | CardKind::Qixingzhen | CardKind::Niaoxiangzhen
    ) && core.players.len() < 4
    {
        return false;
    }
    let (min, max, targets) = jilve_legal_targets(core, catalog, kind, owner);
    if min == 0 && max == 0 {
        if matches!(
            kind,
            CardKind::Wugu
                | CardKind::Taoyuan
                | CardKind::SavageAttack
                | CardKind::ArcheryAttack
                | CardKind::Jingleishan
                | CardKind::Chiyuxi
                | CardKind::Xianluhui
        ) {
            return !targets.is_empty();
        }
        return true;
    }
    targets.len() >= min as usize
}

pub(in crate::content::host) fn jilve_card_options(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    owner: PlayerId,
) -> Vec<ChoiceOption> {
    core.deck
        .iter()
        .rev()
        .take(3)
        .copied()
        .filter(|c| jilve_card_playable(core, catalog, owner, *c))
        .map(ChoiceOption::Card)
        .collect()
}
