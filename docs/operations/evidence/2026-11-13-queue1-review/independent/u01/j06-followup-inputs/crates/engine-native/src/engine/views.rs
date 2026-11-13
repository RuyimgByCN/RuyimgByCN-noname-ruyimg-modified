use super::*;

impl Engine {
    /// N10：座位 `seat` 的可见视图——自己的手牌可见，对手手牌与牌库内容隐藏。
    pub fn view_for(&self, seat: PlayerId) -> crate::view::PlayerView {
        self.build_view(Some(seat), false)
    }

    /// N10：观战者视图——所有手牌隐藏，仅公开信息。
    pub fn view_spectator(&self) -> crate::view::PlayerView {
        self.build_view(None, false)
    }

    /// Trusted host projection: all physical cards, identities and both general slots.
    /// This grants visibility only; it does not create a seat or authorize commands.
    pub fn view_referee(&self) -> crate::view::PlayerView {
        self.build_view(None, true)
    }

    fn build_view(&self, seat: Option<PlayerId>, full_info: bool) -> crate::view::PlayerView {
        let core = &self.state.core;
        let roles = core.identity_roles();
        let players = core
            .players
            .iter()
            .enumerate()
            .map(|(i, p)| {
                let i = i as PlayerId;
                let own = seat == Some(i);
                let kingdoms = matches!(core.mode, crate::state::GameMode::Kingdoms);
                // 身份可见性（RULE-IDENTITY-VIEW-RESTORE）：本人、主公（公开）
                // 与已死亡（遗言公开）可见；其余他人身份不可见。
                let role = roles.and_then(|rs| {
                    let r = *rs.get(i as usize)?;
                    (full_info || own || r == crate::state::Role::Lord || !p.alive).then_some(r)
                });
                // 国战门控（ADR 0757）：本座位双将可见；他人只见明置槽。
                // 非国战模式（原型/身份/对决/斗地主/Boss）无暗置概念，
                // 武将公开可见（`effective_faction` 同口径）。名称/势力经
                // `catalog.general_meta` 取——覆盖标准、扩展与 Boss 段。
                let mut generals: [Option<crate::view::GeneralView>; 2] = [None, None];
                for (s, g_slot) in p.general_slots.iter().enumerate() {
                    let Some(g) = g_slot else { continue };
                    if !full_info && !own && kingdoms && !p.revealed_generals[s] {
                        continue; // 国战暗置槽：他人不可见
                    }
                    if let Some((name, faction, _, _, _)) = self.catalog.general_meta(*g) {
                        generals[s] = Some(crate::view::GeneralView {
                            def: *g,
                            name,
                            faction,
                        });
                    }
                }
                crate::view::PlayerCellView {
                    seat: i,
                    alive: p.alive,
                    hp: p.hp,
                    max_hp: p.max_hp,
                    hand: if full_info || own {
                        crate::view::HandView::Own(p.hand.clone())
                    } else {
                        crate::view::HandView::Hidden
                    },
                    hand_count: p.hand.len(),
                    equips: p.equips.clone(),
                    judge: p.judge.clone(),
                    turned_over: p.face_down,
                    linked: p.linked,
                    skills: {
                        let mut names: Vec<&'static str> = core
                            .skills
                            .range((i, u16::MIN)..)
                            .take_while(|((owner, _), _)| *owner == i)
                            .filter_map(|((_, def), inst)| {
                                if !inst.enabled {
                                    return None;
                                }
                                // R05-H（§8）：派生 helper 是内部实现名，
                                // 不进公开技能列表（分项走 r05_effects）。
                                if self.catalog.skills[*def as usize].name.contains("#r05:") {
                                    return None;
                                }
                                Some(self.catalog.skills[*def as usize].name)
                            })
                            .collect();
                        names.sort_unstable();
                        names
                    },
                    r05_effects: self.r05_effects_for(i),
                    generals,
                    revealed_generals: p.revealed_generals,
                    role,
                }
            })
            .collect();
        crate::view::PlayerView {
            seat: seat.unwrap_or(PlayerId::MAX),
            players,
            cards: {
                let mut held = Vec::new();
                for park in &self.state.resolution.parks {
                    let machine = park.machine.snapshot();
                    if full_info {
                        crate::snap::machine_held_cards(&machine, &mut held);
                    } else if let crate::snap::MachineSnapshot::NativeInvocation {
                        owner,
                        progress:
                            crate::event::NativeInvocationProgress::QinglongLoss {
                                target,
                                card,
                                public,
                            },
                        ..
                    } = machine
                    {
                        if public || seat == Some(owner) || seat == Some(target) {
                            held.push(card);
                        }
                    }
                }
                crate::snap::physical_cards(core, &held, None)
                    .into_iter()
                    .filter(|card| {
                        use crate::snap::PhysicalCardZone::*;
                        full_info
                            || match card.zone {
                                Hand => seat.is_some() && card.owner == seat,
                                Equip | Judge | DiscardPile | Processing => true,
                                // R05：R 帧 held 与 detached 实体区对席位/
                                // 观战隐藏牌面（Referee 经全信息路径另行可见）。
                                DrawPile | Removed | Expansion | DrawHeld | Detached => false,
                            }
                    })
                    .collect()
            },
            deck_count: core.deck.len(),
            discard: core.discard.clone(),
            table: core.table.clone(),
            turn: core.turn.current,
            phase: core.turn.phase,
            round: core.turn.round,
            outcome: core.outcome.as_ref().map(|o| o.winner),
            ruleset_version: core.ruleset_version,
        }
    }

    /// N10 第二批：开放选择的座位可见性——观察者见完整选项，
    /// 他人仅见询问存在（选项会暴露应答者手牌）。无开放选择返回 None。
    pub fn choice_view_for(&self, seat: Option<PlayerId>) -> Option<crate::view::ChoiceView> {
        self.build_choice_view(seat, false)
    }

    /// Trusted referee sees the current choice without becoming its owner.
    pub fn choice_view_referee(&self) -> Option<crate::view::ChoiceView> {
        self.build_choice_view(None, true)
    }

    /// R05-H（§8）：座位在四角色语义槽上的公开分项聚合——helper 定义号
    /// 经 `Catalog::r05_identities` 归属到公开逻辑身份 I，`BTreeMap` 按 I
    /// 稳定排序；marks 条目在即算槽存在（`Some(0)`/负 Combs 均非空项），
    /// 仅至少一槽存在的身份输出。只读汇总，不改状态；对全部观察者同值。
    fn r05_effects_for(&self, seat: u8) -> Vec<crate::view::R05EffectView> {
        if self.catalog.r05_identities.is_empty() {
            return Vec::new();
        }
        let mut by_identity: std::collections::BTreeMap<
            &crate::catalog::QualifiedSkillRef,
            [Option<i32>; 4],
        > = std::collections::BTreeMap::new();
        for (def, identity) in &self.catalog.r05_identities {
            let Some(value) = self.state.core.vm_aux.marks.get(&(seat, *def as u32)) else {
                continue;
            };
            // 角色由派生 helper 名后缀判定（与 hooks `r05_role_defs` 同口径）。
            let slot = match self.catalog.skills[*def as usize].name.rsplit_once("#r05:") {
                Some((_, "limit")) => 0,
                Some((_, "combs")) => 1,
                Some((_, "next")) => 2,
                Some((_, "current")) => 3,
                _ => continue,
            };
            by_identity.entry(identity).or_insert([None; 4])[slot] = Some(*value);
        }
        by_identity
            .into_iter()
            .map(|(skill, slots)| crate::view::R05EffectView {
                skill: skill.clone(),
                tiqi_limit: slots[0],
                baoshu_combs: slots[1],
                dcbihuo_next: slots[2],
                dcbihuo_current: slots[3],
            })
            .collect()
    }

    /// R05-H（§8）：涕泣符号询问（`rvm_tiqi_sign`）的 Own context——沿
    /// `choice_source_skill` 同款纪律定位唯一产生该询问的内容机器，从其
    /// **已验证本次内容局部** `[stage=3, n, target+1]` 派生
    /// `TiqiAdjustment{skill, target(0 基 PlayerId), delta=|n−2|}`；不从
    /// history 重算 d。找不到唯一产生者/形状不符即 None（可见性不授予应答权限）。
    fn tiqi_context(
        &self,
        open: &crate::choice::OpenChoice,
    ) -> Option<crate::view::ChoiceContext> {
        if open.spec.prompt != "rvm_tiqi_sign" {
            return None;
        }
        let resolution = &self.state.resolution;
        let consumer = crate::resolution::ready_park_index(resolution)?;
        let mut producer = None;
        for (index, park) in resolution.parks.iter().enumerate() {
            let snapshot = park.machine.snapshot();
            if let Some(a) = crate::snap::active_ask(&snapshot, &self.state.core, &self.catalog) {
                if producer.is_some()
                    || a.observer != open.observer
                    || a.prompt != open.spec.prompt
                    || a.kind != open.spec.kind
                    || a.options != open.spec.options
                    || a.min != open.spec.min
                    || a.max != open.spec.max
                    || a.allow_cancel != open.spec.allow_cancel
                {
                    return None;
                }
                producer = Some((index, snapshot));
            }
        }
        let (index, snapshot) = producer?;
        if index != consumer {
            return None;
        }
        let crate::snap::MachineSnapshot::Rvm {
            commit_state,
            commit_pending,
            skill_name,
            ..
        } = snapshot
        else {
            return None;
        };
        // 已验证内容局部形状：[stage=3（符号）, n, target+1]。
        if commit_state.len() != 3 || commit_state.first() != Some(&3) {
            return None;
        }
        let effects = crate::rvm_ops::dec_effects(&commit_pending)?;
        if !matches!(
            effects.first(),
            Some(rule_api::EffectRequest::ResolveTiqi { .. })
        ) {
            return None;
        }
        let delta = (commit_state[1] as i32 - 2).unsigned_abs();
        let target = u8::try_from(commit_state[2].checked_sub(1)?).ok()?;
        if usize::from(target) >= self.state.core.players.len() {
            return None;
        }
        let helper = format!("{skill_name}#r05:limit");
        let def = self.catalog.skill_by_name(&helper)?;
        let skill = self.catalog.r05_identities.get(&def)?.clone();
        Some(crate::view::ChoiceContext::TiqiAdjustment {
            skill,
            target,
            delta,
        })
    }

    fn build_choice_view(
        &self,
        seat: Option<PlayerId>,
        full_info: bool,
    ) -> Option<crate::view::ChoiceView> {
        let open = self.state.resolution.open_choice.as_ref()?;
        Some(if full_info || seat == Some(open.observer) {
            crate::view::ChoiceView::Own {
                id: open.id,
                kind: open.spec.kind,
                prompt: open.spec.prompt,
                options: open.spec.options.clone(),
                min: open.spec.min,
                max: open.spec.max,
                allow_cancel: open.spec.allow_cancel,
                source_skill: self.choice_source_skill(open),
                context: self.tiqi_context(open),
            }
        } else {
            crate::view::ChoiceView::Other {
                id: open.id,
                observer: open.observer,
                prompt: open.spec.prompt,
            }
        })
    }

    /// N10 第二批：日志事件的座位可见性——他人摸牌仅数量（牌 id 隐藏），
    /// 其余事件公开；自己的摸牌完整可见。
    pub fn event_view_for(&self, seat: PlayerId) -> Vec<crate::view::EventView> {
        // N10-a（ADR 0768）：私有选择应答仅观察者可见——沿日志维护
        // 询问 id → 观察者映射，非观察者收到内容不透明的脱敏事件。
        let mut choice_observers: std::collections::HashMap<u64, PlayerId> =
            std::collections::HashMap::new();
        self.log
            .iter()
            .filter_map(|l| match l {
                crate::machine::LogEvent::CardsDrawn { player, cards } if *player != seat => {
                    Some(crate::view::EventView::CardsDrawnHidden {
                        player: *player,
                        count: cards.len(),
                    })
                }
                crate::machine::LogEvent::ChoiceOpened { id, observer, .. } => {
                    choice_observers.insert(*id, *observer);
                    Some(crate::view::EventView::Public(l.clone()))
                }
                crate::machine::LogEvent::ChoiceAnswered { id, .. } => {
                    match choice_observers.get(id) {
                        Some(&observer) if observer == seat => {
                            Some(crate::view::EventView::Public(l.clone()))
                        }
                        _ => Some(crate::view::EventView::ChoiceAnsweredHidden { id: *id }),
                    }
                }
                crate::machine::LogEvent::CardsTaken {
                    from,
                    to,
                    cards,
                    public: false,
                }
                | crate::machine::LogEvent::CardsGiven { from, to, cards }
                    if *from != seat && *to != seat =>
                {
                    Some(crate::view::EventView::CardsGivenHidden {
                        from: *from,
                        to: *to,
                        count: cards.len(),
                    })
                }
                other => Some(crate::view::EventView::Public(other.clone())),
            })
            .collect()
    }

    /// 从日志下标 `start` 起投影公开事件码（不泄露牌 id/选项内容）。
    pub fn public_event_codes_from(&self, start: usize) -> Vec<u16> {
        self.log
            .iter()
            .skip(start)
            .filter_map(crate::view::public_event_code)
            .collect()
    }

    /// N10：AI 只读视角会话——座位固定，仅暴露可见性投影。
    pub fn ai_session(&self, seat: PlayerId) -> crate::view::AiSession<'_> {
        crate::view::AiSession { engine: self, seat }
    }
}
