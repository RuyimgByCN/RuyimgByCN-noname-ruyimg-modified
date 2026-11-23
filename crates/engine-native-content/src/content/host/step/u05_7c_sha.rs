//! U05 Batch 7c 杀结算 `shaBegin` 宿主钩子：`linyun2`（先出闪响应否则直击且跳过防具）与 `lingyue`（可选令目标自选弃 1 张 he 牌）。

use super::super::*;
use super::u08_batch_d::{flag, has_skill_named, set_flag};

/// `!priority_ge_6` 阶段 `shaBegin` 钩子：先于 `cike` 检查 `linyun2` 与 `lingyue`。
pub(super) fn sha_begin_step_u05_7c(
    ctx: &mut Ctx<'_>,
    sha_frame_id: crate::ids::EventId,
    source: PlayerId,
    target: PlayerId,
    sha_card: Option<CardId>,
) -> bool {
    let linyun_active = (has_skill_named(ctx.core, ctx.catalog, source, "linyun")
        || has_skill_named(ctx.core, ctx.catalog, source, "linyun2"))
        && !ctx.catalog.is_tieji_sealed(ctx.core, source)
        && sha_card
            .is_some_and(|cid| flag(ctx.core, source, &format!("swd_linyun_sha:{cid}")) != 0)
        && flag(ctx.core, source, &format!("u05_linyun2_seen:{sha_frame_id}:{target}")) == 0;
    if linyun_active {
        set_flag(
            ctx.core,
            source,
            &format!("u05_linyun2_seen:{sha_frame_id}:{target}"),
            1,
        );
        let shans: Vec<ChoiceOption> = ctx
            .core
            .player(target)
            .hand
            .iter()
            .copied()
            .filter(|c| ctx.catalog.card_kind(*c) == CardKind::Shan)
            .map(ChoiceOption::Card)
            .collect();
        if shans.is_empty() {
            super::u08_batch_d::mark_sha_unrespondable(ctx, source, target, sha_card);
        } else {
            ctx.res.parks.push(crate::resolution::Park {
                resume_depth: ctx.res.stack.len(),
                slot: crate::resolution::Slot::Trigger { frame: sha_frame_id },
                machine: Box::new(HostMachine {
                    def_key: "linyun2".to_string(),
                    behavior: HostBehavior::SwdLinyun,
                    owner: source,
                    target: Some(target),
                    card: sha_card,
                    cards: Vec::new(),
                    targets: Vec::new(),
                    stage: HostStage::HuanrenPick,
                }),
                awaiting_answer: false,
            });
            return true;
        }
    }
    let lingyue_active = ctx
        .catalog
        .skill_by_name("lingyue")
        .is_some_and(|ldef| {
            ctx.core.has_skill(source, ldef)
                && !ctx.catalog.is_tieji_sealed(ctx.core, source)
                && !ctx.catalog.is_rechanyuan_sealed_def(ctx.core, source, ldef)
        })
        && (!ctx.core.player(target).hand.is_empty() || !ctx.core.player(target).equips.is_empty())
        && flag(ctx.core, source, &format!("u05_lingyue_seen:{sha_frame_id}:{target}")) == 0;
    if lingyue_active {
        set_flag(
            ctx.core,
            source,
            &format!("u05_lingyue_seen:{sha_frame_id}:{target}"),
            1,
        );
        ctx.res.parks.push(crate::resolution::Park {
            resume_depth: ctx.res.stack.len(),
            slot: crate::resolution::Slot::Trigger { frame: sha_frame_id },
            machine: Box::new(HostMachine {
                def_key: "lingyue".to_string(),
                behavior: HostBehavior::SwdLingyue,
                owner: source,
                target: Some(target),
                card: None,
                cards: Vec::new(),
                targets: Vec::new(),
                stage: HostStage::Act,
            }),
            awaiting_answer: false,
        });
        return true;
    }
    false
}

impl HostMachine {
    pub(super) fn step_u05_7c_sha(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.behavior {
            // 林韵·次面（linyun2）：目标可选打出一张【闪】；否则该杀不可闪避
            HostBehavior::SwdLinyun => match self.stage {
                HostStage::HuanrenPick => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    let options: Vec<ChoiceOption> = ctx
                        .core
                        .player(t)
                        .hand
                        .iter()
                        .copied()
                        .filter(|c| ctx.catalog.card_kind(*c) == CardKind::Shan)
                        .map(ChoiceOption::Card)
                        .collect();
                    if options.is_empty() {
                        super::u08_batch_d::mark_sha_unrespondable(ctx, self.owner, t, self.card);
                        return Flow::Done;
                    }
                    self.stage = HostStage::SelectNature;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "linyun2_shan",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                HostStage::SelectNature => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                        if ctx.core.player(t).hand.contains(&c) {
                            ctx.discard_from_hand(t, c);
                            ctx.emit(EventData::CardResponded { card: c, player: t });
                            ctx.log(LogEvent::CardResponded {
                                card: c,
                                player: t,
                                presentation: None,
                            });
                            return Flow::Done;
                        }
                    }
                    super::u08_batch_d::mark_sha_unrespondable(ctx, self.owner, t, self.card);
                    Flow::Done
                }
                _ => Flow::Done,
            },
            // 玲月（swd/skill.js:1560）：shaBegin 确认是否发动，确定后目标自选弃 1 张 he 牌
            HostBehavior::SwdLingyue => match self.stage {
                HostStage::Act => {
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive
                        || (ctx.core.player(t).hand.is_empty() && ctx.core.player(t).equips.is_empty())
                    {
                        return Flow::Done;
                    }
                    self.stage = HostStage::CompareTargetPick;
                    Flow::WaitChoice {
                        observer: self.owner,
                        spec: ChoiceSpec::confirm("lingyue_confirm"),
                    }
                }
                HostStage::CompareTargetPick => {
                    if !matches!(ctx.answer().first(), Some(ChoiceOption::Confirm)) {
                        return Flow::Done;
                    }
                    let Some(t) = self.target else {
                        return Flow::Done;
                    };
                    if !ctx.core.player(t).alive {
                        return Flow::Done;
                    }
                    let options = super::u05_7c_skills::he_card_options(ctx.core, t);
                    if options.is_empty() {
                        return Flow::Done;
                    }
                    if let Some(ldef) = ctx.catalog.skill_by_name("lingyue") {
                        ctx.log(LogEvent::SkillTriggered {
                            skill: ctx.catalog.skills[ldef as usize].name,
                            owner: self.owner,
                            presentation: None,
                        });
                    }
                    self.stage = HostStage::SelectCard;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "lingyue_discard",
                            options,
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    }
                }
                HostStage::SelectCard => {
                    if let (Some(t), Some(ChoiceOption::Card(c))) =
                        (self.target, ctx.answer().first().copied())
                    {
                        let _ = super::u05_7c_skills::discard_he_card(ctx, t, c);
                    }
                    Flow::Done
                }
                _ => Flow::Done,
            },
            _ => unreachable!("behavior routed to u05_7c_sha"),
        }
    }
}
