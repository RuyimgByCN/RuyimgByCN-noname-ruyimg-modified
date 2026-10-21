use super::*;

pub(super) fn apply_command(
    state: &mut GameState,
    cmd: GameCommand,
    catalog: &SharedCatalog,
    log: &mut Vec<LogEvent>,
) -> Result<(), RejectReason> {
    let actor = cmd.actor;
    match cmd.action {
        GameAction::RevealGeneral { slot } => {
            // 国战限定（ADR 0757 明置门控）：其余模式无武将牌语义。
            if !matches!(state.core.mode, crate::state::GameMode::Kingdoms) {
                return Err(RejectReason::WrongMode);
            }
            if slot > 1 {
                return Err(RejectReason::IllegalTargets);
            }
            // 时机门控：自己回合的出牌阶段空闲点，或成为牌目标的
            // 补偿窗口（结算栈中存在以 actor 为目标的牌）。
            let compensatory = state.resolution.stack.iter().any(|f| match &f.data {
                EventData::CardUsed(u) => u.targets.contains(&actor),
                _ => false,
            });
            if !compensatory {
                if actor != state.core.turn.current {
                    return Err(RejectReason::NotYourTurn);
                }
                if state.core.turn.phase != crate::event::Phase::Play {
                    return Err(RejectReason::WrongPhase);
                }
            }
            let p = state
                .core
                .players
                .get_mut(actor as usize)
                .ok_or(RejectReason::UnknownActor)?;
            if p.general_slots[slot as usize].is_none() {
                return Err(RejectReason::NoSuchGeneral);
            }
            if p.revealed_generals[slot as usize] {
                return Err(RejectReason::AlreadyRevealed);
            }
            p.revealed_generals[slot as usize] = true;
            // N11 将领装配（ADR 0769）：明置实际授予该武将技能集；
            // 首次明置主将时配置体力上限（副将明置不改体力）。
            apply_general_assembly(&mut state.core, catalog, actor, slot as usize);
            // 明置是公开行为：状态与视图即时可见（无隐藏事件载荷）。
            Ok(())
        }
        GameAction::ActivateSkill { skill } => {
            // 授权：主动技仅当前回合玩家可发动
            if actor != state.core.turn.current {
                return Err(RejectReason::NotYourTurn);
            }
            let Some(def) = catalog.skills.get(skill as usize) else {
                return Err(RejectReason::NoSuchSkill);
            };
            // 阶段门控（N08 上游对齐）：主动技默认限出牌阶段（上游
            // `enable: "phaseUse"`——仁德曾在判定阶段被放行）；响应入口
            // 类技能（激将 `chooseToUse`/`chooseToRespond`）在目录声明
            // 例外后不绑定阶段。
            if state.core.turn.phase != crate::event::Phase::Play && !def.limits.active_outside_play
            {
                return Err(RejectReason::WrongPhase);
            }
            let Some(inst) = state.core.skills.get(&(actor, skill)) else {
                return Err(RejectReason::NoSuchSkill);
            };
            if !inst.enabled {
                return Err(RejectReason::SkillDisabled);
            }
            if inst.once_used {
                return Err(RejectReason::SkillAlreadyUsed);
            }
            let qctx = crate::catalog::QueryCtx {
                state: &state.core,
                catalog,
            };
            let Some(machine) = def.behavior.activate(&qctx, actor) else {
                return Err(RejectReason::InvalidChoice);
            };
            if def.limits.once_per_turn {
                if let Some(inst) = state.core.skills.get_mut(&(actor, skill)) {
                    inst.once_used = true;
                }
            }
            crate::resolution::push_frame(
                &mut state.resolution,
                EventData::SkillToggled {
                    player: actor,
                    skill,
                    enabled: true,
                    temporary: false,
                },
            );
            let frame_id = state.resolution.next_event_id;
            let _ = frame_id;
            // 主动技机器入栈顶挂起槽（复用 Intrinsic 槽语义）
            state.resolution.parks.push(crate::resolution::Park {
                resume_depth: state.resolution.stack.len(),
                slot: crate::resolution::Slot::Intrinsic {
                    frame: crate::resolution::NO_FRAME,
                },
                machine,
            });
            Ok(())
        }
        GameAction::Answer {
            choice_id,
            selection,
        } => {
            let Some(open) = state.resolution.open_choice.as_ref() else {
                return Err(RejectReason::NoSuchChoice);
            };
            if open.id != choice_id {
                return Err(RejectReason::NoSuchChoice);
            }
            // 授权：只有选择观察者本人可以应答（不能替别人决策）。
            if actor != open.observer {
                return Err(RejectReason::WrongPlayer);
            }
            if !open.spec.accepts(&selection) {
                return Err(RejectReason::InvalidChoice);
            }
            let open = state.resolution.open_choice.take().expect("checked");
            log.push(LogEvent::ChoiceAnswered {
                id: open.id,
                options: selection.iter().map(|o| o.describe()).collect(),
            });
            state.resolution.answer = Some(selection);
            Ok(())
        }
        GameAction::PlayCard { card, targets } => play_card(state, catalog, actor, card, targets),
        GameAction::NextPhase => {
            let core = &state.core;
            // 授权：只有当前回合玩家可以推进自己的阶段。
            if actor != core.turn.current {
                return Err(RejectReason::NotYourTurn);
            }
            let player = core.turn.current;
            let phase = core.turn.phase;
            crate::resolution::push_frame(
                &mut state.resolution,
                EventData::Phase {
                    player,
                    phase,
                    begin: false,
                },
            );
            Ok(())
        }
    }
}
