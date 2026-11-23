//! 标准主动技与顺手牵羊/过河拆桥衍生询问派生（从 `snap/asks.rs` 按职责拆出）。

use super::*;

pub(super) fn active_standard_skill_or_trick_ask(
    machine: &MachineSnapshot,
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    match machine {
        MachineSnapshot::QingNang { owner, stage, .. } => match stage {
            QingNangStageSnap::TargetRead => {
                let wounded: Vec<ChoiceOption> = core
                    .alive_players()
                    .filter(|p| {
                        *p != *owner && {
                            let pl = core.player(*p);
                            pl.alive && (pl.hp as i64) < (pl.max_hp as i64)
                        }
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                Some(ask(
                    *owner,
                    "qingnang_target",
                    ChoiceKind::SelectPlayers,
                    wounded,
                    1,
                    1,
                    true,
                ))
            }
            QingNangStageSnap::CardRead => Some(ask(
                *owner,
                "qingnang_card",
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(*owner).hand),
                1,
                1,
                true,
            )),
            _ => None,
        },
        MachineSnapshot::JiJi { owner, stage, .. } => match stage {
            JiJiStageSnap::GiveCard => Some(ask(
                *owner,
                "jiji_target",
                ChoiceKind::SelectPlayers,
                other_player_options(core, *owner),
                0,
                1,
                true,
            )),
            JiJiStageSnap::GiveChooseCard => Some(ask(
                *owner,
                "jiji_card",
                ChoiceKind::SelectCards,
                hand_card_options(&core.player(*owner).hand),
                1,
                1,
                true,
            )),
            _ => None,
        },
        MachineSnapshot::ZhiHeng { owner, stage } => match stage {
            ZhiHengStageSnap::Draw => {
                let hand = &core.player(*owner).hand;
                let max = hand.len();
                Some(ask(
                    *owner,
                    "zhiheng_select",
                    ChoiceKind::SelectCards,
                    hand_card_options(hand),
                    0,
                    max,
                    true,
                ))
            }
            _ => None,
        },
        MachineSnapshot::Jiedao {
            holder,
            stage: JiedaoStageSnap::Read,
            ..
        } => Some(ask(
            *holder,
            "jiedao_sha_or_give",
            ChoiceKind::PickTag,
            vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
            1,
            1,
            false,
        )),
        MachineSnapshot::Dismantle {
            source,
            target,
            asked,
            ..
        } => {
            if !*asked
                || !catalog
                    .skill_by_name("txhj_kapaikeshi")
                    .is_some_and(|d| core.has_skill(*source, d))
            {
                return None;
            }
            let t = core.player(*target);
            let options: Vec<ChoiceOption> = t
                .hand
                .iter()
                .chain(t.equips.iter())
                .chain(t.judge.iter())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            (!options.is_empty()).then(|| {
                ask(
                    *source,
                    "txhj_kapaikeshi:choose_card",
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    false,
                )
            })
        }
        MachineSnapshot::Snatch {
            source,
            target,
            asked,
            ..
        } => {
            if !*asked
                || !catalog
                    .skill_by_name("txhj_kapaikeshi")
                    .is_some_and(|d| core.has_skill(*source, d))
            {
                return None;
            }
            let t = core.player(*target);
            let options: Vec<ChoiceOption> = t
                .hand
                .iter()
                .chain(t.equips.iter())
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            (!options.is_empty()).then(|| {
                ask(
                    *source,
                    "txhj_kapaikeshi:choose_card",
                    ChoiceKind::SelectCards,
                    options,
                    1,
                    1,
                    false,
                )
            })
        }
        _ => None,
    }
}

// Only OptionalGate::step has the direct delegation contract used by this source.
pub(super) fn optional_source(machine: &MachineSnapshot, catalog: &Catalog) -> Option<&'static str> {
    let MachineSnapshot::OptionalGate {
        def, asked, inner, ..
    } = machine
    else {
        return None;
    };
    if let Some(inner) = inner {
        return optional_source(inner, catalog);
    }
    if !asked {
        return None;
    }
    let declared = catalog.skills.get(*def as usize)?;
    if declared.name.is_empty()
        || declared.behavior.is_vm_skill()
        || !declared.trigger.as_ref().is_some_and(|meta| meta.optional)
        || catalog
            .skills
            .iter()
            .filter(|entry| entry.name == declared.name)
            .count()
            != 1
    {
        return None;
    }
    Some(declared.name)
}
