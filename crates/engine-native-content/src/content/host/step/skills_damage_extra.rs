//! 自 `skills_damage.rs` 尾部分离的伤害步骤实现（`CaochuanCard` 与 `D3Moyan`）。

use super::super::*;

pub(super) fn step_caochuan_card(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    match machine.stage {
        // 分支选择（PickTag：0=交杀+反噬 1=展示+被弃一张）
        HostStage::SelectNature => {
            let Some(t) = machine.target else {
                return Flow::Done;
            };
            let has_sha = ctx
                .core
                .player(t)
                .hand
                .iter()
                .any(|c| ctx.catalog.card_kind(*c) == CardKind::Sha);
            if !has_sha {
                // 无杀：直接走展示+被弃分支
                machine.stage = HostStage::Act;
                machine.cards = vec![1];
                let options = vec![ChoiceOption::Tag(1)];
                return Flow::WaitChoice {
                    observer: machine.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::PickTag,
                        prompt: WD_CAOCHUAN_BRANCH,
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                };
            }
            machine.stage = HostStage::Act;
            let options = vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)];
            Flow::WaitChoice {
                observer: t,
                spec: ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: WD_CAOCHUAN_BRANCH,
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            }
        }
        HostStage::Act => {
            let Some(t) = machine.target else {
                return Flow::Done;
            };
            match ctx.answer().first() {
                Some(ChoiceOption::Tag(0)) => {
                    // 交出所有手牌杀给使用者；使用者受 1 点反噬杀
                    //（登记差异：上游走完整杀结算）
                    let shas: Vec<CardId> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c) == CardKind::Sha)
                        .collect();
                    for c in &shas {
                        ctx.core.player_mut(t).hand.retain(|x| x != c);
                        ctx.core.player_mut(machine.owner).hand.push(*c);
                    }
                    if !shas.is_empty() {
                        ctx.log(LogEvent::CardsGiven {
                            from: t,
                            to: machine.owner,
                            cards: shas,
                        });
                    }
                    if ctx.core.player(machine.owner).alive {
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(t),
                            machine.owner,
                            1,
                            crate::event::DamageNature::Normal,
                            None,
                        )));
                    }
                }
                _ => {
                    // 展示手牌 + 被弃一张（确定性首张）
                    if let Some(c) = ctx.core.player(t).hand.first().copied() {
                        ctx.discard_from_hand(t, c);
                        ctx.log(LogEvent::CardDiscarded {
                            player: t,
                            cards: vec![c],
                        });
                    }
                }
            }
            if let Some(c) = machine.card {
                ctx.to_discard(c);
                ctx.core.table.retain(|x| *x != c);
            }
            Flow::Done
        }
        _ => Flow::Done,
    }
}

pub(super) fn step_d3_moyan(machine: &mut HostMachine, ctx: &mut Ctx<'_>) -> Flow {
    let frame_id = ctx.current_event_frame_id();
    let is_damage = ctx
        .res
        .frame(frame_id)
        .is_some_and(|frame| matches!(frame.data, EventData::Damage(_)));
    if is_damage {
        if !ctx.modify_damage_amount(1) {
            return Flow::Done;
        }
        Flow::Done
    } else {
        moyan_round_step(machine, ctx)
    }
}
