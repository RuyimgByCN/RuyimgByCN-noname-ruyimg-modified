use super::*;

impl Engine {
    /// N10：座位 `seat` 的可见视图——自己的手牌可见，对手手牌与牌库内容隐藏。
    pub fn view_for(&self, seat: PlayerId) -> crate::view::PlayerView {
        self.build_view(Some(seat))
    }

    /// N10：观战者视图——所有手牌隐藏，仅公开信息。
    pub fn view_spectator(&self) -> crate::view::PlayerView {
        self.build_view(None)
    }

    pub(super) fn build_view(&self, seat: Option<PlayerId>) -> crate::view::PlayerView {
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
                    (own || r == crate::state::Role::Lord || !p.alive).then_some(r)
                });
                // 国战门控（ADR 0757）：本座位双将可见；他人只见明置槽。
                // 非国战模式（原型/身份/对决/斗地主/Boss）无暗置概念，
                // 武将公开可见（`effective_faction` 同口径）。名称/势力经
                // `catalog.general_meta` 取——覆盖标准、扩展与 Boss 段。
                let mut generals: [Option<crate::view::GeneralView>; 2] = [None, None];
                for (s, g_slot) in p.general_slots.iter().enumerate() {
                    let Some(g) = g_slot else { continue };
                    if !own && kingdoms && !p.revealed_generals[s] {
                        continue; // 国战暗置槽：他人不可见
                    }
                    if let Some((name, faction, _, _, _)) = self.catalog.general_meta(*g) {
                        generals[s] = Some(crate::view::GeneralView { name, faction });
                    }
                }
                crate::view::PlayerCellView {
                    seat: i,
                    alive: p.alive,
                    hp: p.hp,
                    max_hp: p.max_hp,
                    hand: if own {
                        crate::view::HandView::Own(p.hand.clone())
                    } else {
                        crate::view::HandView::Hidden
                    },
                    hand_count: p.hand.len(),
                    equips: p.equips.clone(),
                    judge: p.judge.clone(),
                    turned_over: p.face_down,
                    skills: {
                        let mut names: Vec<&'static str> = core
                            .skills
                            .range((i, u16::MIN)..)
                            .take_while(|((owner, _), _)| *owner == i)
                            .filter_map(|((_, def), inst)| {
                                if !inst.enabled {
                                    return None;
                                }
                                Some(self.catalog.skills[*def as usize].name)
                            })
                            .collect();
                        names.sort_unstable();
                        names
                    },
                    generals,
                    revealed_generals: p.revealed_generals,
                    role,
                }
            })
            .collect();
        crate::view::PlayerView {
            seat: seat.unwrap_or(PlayerId::MAX),
            players,
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
        let open = self.state.resolution.open_choice.as_ref()?;
        Some(if seat == Some(open.observer) {
            crate::view::ChoiceView::Own {
                id: open.id,
                prompt: open.spec.prompt,
                options: open.spec.options.clone(),
                min: open.spec.min,
                max: open.spec.max,
                allow_cancel: open.spec.allow_cancel,
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
                other => Some(crate::view::EventView::Public(other.clone())),
            })
            .collect()
    }

    /// N10：AI 只读视角会话——座位固定，仅暴露可见性投影。
    pub fn ai_session(&self, seat: PlayerId) -> crate::view::AiSession<'_> {
        crate::view::AiSession { engine: self, seat }
    }
}
