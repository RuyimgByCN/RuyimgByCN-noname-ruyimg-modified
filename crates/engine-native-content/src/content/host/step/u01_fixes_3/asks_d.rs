use super::*;

#[allow(clippy::too_many_arguments)]
pub(super) fn active_ask_u01_fix3_d(
    ask_id: u32,
    _stage: HostStage,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    _card: Option<CardId>,
    _cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    match ask_id {
        3040 => {
            let cur_holder = global_youxue_holder(core).unwrap_or(owner);
            let opts: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != cur_holder)
                .map(ChoiceOption::Player)
                .collect();
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddyouxue_target",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        3041 => {
            let opts: Vec<ChoiceOption> = he_cards(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddchengjing_card",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        3042 => {
            let &pile_c = core
                .vm_aux
                .piles
                .get(&(owner, "dddchengjing".to_string()))?
                .first()?;
            let as_kind = catalog.card_kind(pile_c);
            let opts: Vec<ChoiceOption> = virtual_legal_targets(core, catalog, owner, as_kind)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddchengjing_target",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        3043 => Some((owner, u01_confirm_spec("dddchengjing_give_confirm", true))),
        3044 => {
            let opts: Vec<ChoiceOption> = he_cards(core, owner)
                .into_iter()
                .filter(|&c| {
                    let k = eff_card_kind_3(core, catalog, c);
                    is_black_3(core, catalog, owner, c) && (k.is_basic_kind() || k.is_equip())
                })
                .map(ChoiceOption::Card)
                .collect();
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddduanbing_card",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        3045 => {
            let opts: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner && !is_sha_target_blocked(core, catalog, owner, p, CardKind::Sha, DamageNature::Normal))
                .map(ChoiceOption::Player)
                .collect();
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddduanbing_target",
                    options: opts,
                    min: 0,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        3046 => Some((owner, u01_confirm_spec("dddduanbing_transfer", true))),
        3048 => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddzhuanshe_give",
                    options: opts,
                    min: 0,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        3049 => {
            let t = target?;
            let opts: Vec<ChoiceOption> = targets
                .iter()
                .copied()
                .filter(|&p| p != t && core.player(p).alive)
                .map(ChoiceOption::Player)
                .collect();
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddzhuanshe_add_target",
                    options: opts,
                    min: 0,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        3050 => Some((owner, u01_confirm_spec("dddzhuanshe_damage", true))),
        3051 => Some((owner, u01_confirm_spec("dddweiqiu_confirm", false))),
        3052 => Some((owner, u01_confirm_spec("dddlianer_confirm", true))),
        3053 => {
            let sha_user = target?;
            let opts: Vec<ChoiceOption> = core
                .player(sha_user)
                .hand
                .iter()
                .copied()
                .filter(|&c| eff_card_kind_3(core, catalog, c).is_basic_kind())
                .map(ChoiceOption::Card)
                .collect();
            Some((
                sha_user,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "xiangle_discard",
                    options: opts,
                    min: 0,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        3054 => Some((owner, u01_confirm_spec("jiushi_turnover", true))),
        3055 => {
            let opts: Vec<ChoiceOption> = he_cards(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            let max = opts.len().max(1);
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "rezhiheng_cards",
                    options: opts,
                    min: 1,
                    max,
                    allow_cancel: true,
                },
            ))
        }
        3056 => {
            let qun_cnt = core
                .alive_players()
                .filter(|&p| core.effective_faction(catalog, p) == Some(crate::catalog::Faction::Qun))
                .count();
            let linked_cnt = core.alive_players().filter(|&p| core.player(p).linked).count();
            let min = qun_cnt.saturating_sub(linked_cnt).max(1);
            let opts: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| !core.player(p).linked)
                .map(ChoiceOption::Player)
                .collect();
            let max = opts.len().max(min);
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "ddddanggu_targets",
                    options: opts,
                    min,
                    max,
                    allow_cancel: false,
                },
            ))
        }
        3057 => {
            let forced = flag(core, owner, "dddxuanlun_del") != 0;
            Some((owner, u01_confirm_spec("dddlanghuai_confirm", !forced)))
        }
        3058 => Some((owner, u01_confirm_spec("dddxuanlun_confirm", true))),
        3059 => {
            let he_len = he_cards(core, owner).len();
            let del = flag(core, owner, "dddxuanlun_del") != 0;
            let mut opts = Vec::new();
            if he_len >= 4 || del {
                opts.push(ChoiceOption::Tag(0));
            }
            if !del {
                opts.push(ChoiceOption::Tag(1));
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "dddxuanlun_end_mode",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        3060 => {
            let opts: Vec<ChoiceOption> = he_cards(core, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            let need = opts.len().min(4);
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddxuanlun_cards",
                    options: opts,
                    min: need,
                    max: need,
                    allow_cancel: false,
                },
            ))
        }
        3061 => {
            let hold = core
                .vm_aux
                .piles
                .get(&(owner, "u01_xuanlun_hold".to_string()))
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let opts: Vec<ChoiceOption> = hold.iter().copied().map(ChoiceOption::Card).collect();
            let max = opts.len();
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddxuanlun_top",
                    options: opts,
                    min: 0,
                    max,
                    allow_cancel: true,
                },
            ))
        }
        3062 => {
            let hold = core
                .vm_aux
                .piles
                .get(&(owner, "u01_xuanlun_hold".to_string()))
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let opts: Vec<ChoiceOption> = hold.iter().copied().map(ChoiceOption::Card).collect();
            let n = opts.len();
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddxuanlun_bottom",
                    options: opts,
                    min: n,
                    max: n,
                    allow_cancel: false,
                },
            ))
        }
        3063 => Some((owner, u01_confirm_spec("dddjiexing_confirm", true))),
        3064 => {
            let opts: Vec<ChoiceOption> = dddbailei_eligible_targets(core)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddbailei_target",
                    options: opts,
                    min: 0,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        3065 => {
            let t = target?;
            let pl = core.player(t);
            let both = pl.hp == 1 && is_strict_max_equip(core, t) && !he_cards(core, t).is_empty();
            let opts: Vec<ChoiceOption> = he_cards(core, t)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddbailei_pick_card",
                    options: opts,
                    min: usize::from(!both),
                    max: 1,
                    allow_cancel: both,
                },
            ))
        }
        3066 => {
            let holder = *targets.first()?;
            let opts: Vec<ChoiceOption> =
                super::super::u01_fixes_1::jiedao_victims_for(core, catalog, holder)
                    .into_iter()
                    .map(ChoiceOption::Player)
                    .collect();
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddchengjing_jiedao_victim",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        _ => None,
    }
}
