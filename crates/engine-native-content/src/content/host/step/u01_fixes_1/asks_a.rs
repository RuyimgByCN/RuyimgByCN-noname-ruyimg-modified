use super::*;

pub(super) fn dddshilie_unshown_hand(core: &GameStateCore, owner: PlayerId) -> Vec<CardId> {
    core.player(owner)
        .hand
        .iter()
        .copied()
        .filter(|&c| flag(core, owner, &format!("dddshilie_shown_card:{c}")) == 0)
        .collect()
}

pub(super) fn dddshilie_threshold(core: &GameStateCore, owner: PlayerId) -> i32 {
    core.player(owner).hp + core.player(core.turn.current).hp
}

pub(super) fn dddshilie_max_unshown_sum(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> i32 {
    dddshilie_unshown_hand(core, owner)
        .into_iter()
        .map(|c| i32::from(catalog.cards[c as usize].rank))
        .sum()
}

pub(super) fn card_slot_of_or_kind(
    core: &GameStateCore,
    catalog: &Catalog,
    c: CardId,
) -> Option<EquipSlot> {
    catalog
        .card_slot_of(core, c)
        .or_else(|| catalog.cards[c as usize].kind.equip_slot())
}

pub(super) fn zhishu_valid_dests(
    core: &GameStateCore,
    catalog: &Catalog,
    card: CardId,
) -> Vec<PlayerId> {
    let mut owner_of = None;
    let mut is_judge = false;
    for p in core.alive_players() {
        if core.player(p).equips.contains(&card) {
            owner_of = Some(p);
            break;
        }
        if core.player(p).judge.contains(&card) {
            owner_of = Some(p);
            is_judge = true;
            break;
        }
    }
    let Some(src) = owner_of else {
        return Vec::new();
    };
    if is_judge {
        let kind = core
            .declared_kinds
            .get(&card)
            .copied()
            .unwrap_or_else(|| catalog.card_kind(card));
        core.alive_players()
            .filter(|&p| {
                p != src
                    && !core.player(p).judge.iter().any(|&jc| {
                        core.declared_kinds
                            .get(&jc)
                            .copied()
                            .unwrap_or_else(|| catalog.card_kind(jc))
                            == kind
                    })
            })
            .collect()
    } else if let Some(slot) = card_slot_of_or_kind(core, catalog, card) {
        core.alive_players()
            .filter(|&p| p != src && !slot_occupied(core, catalog, p, slot))
            .collect()
    } else {
        Vec::new()
    }
}

pub(super) fn zhishu_movable_cards(core: &GameStateCore, catalog: &Catalog) -> Vec<CardId> {
    let mut out = Vec::new();
    for p in core.alive_players() {
        for &c in core.player(p).equips.iter().chain(core.player(p).judge.iter()) {
            if !zhishu_valid_dests(core, catalog, c).is_empty() {
                out.push(c);
            }
        }
    }
    out
}

pub(super) fn dddguiying_wu_others(core: &GameStateCore, catalog: &Catalog, owner: PlayerId) -> Vec<PlayerId> {
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&p| {
            p != owner
                && core.player(p).alive
                && core.effective_faction(catalog, p) == Some(crate::catalog::Faction::Wu)
                && !core.player(p).hand.is_empty()
        })
        .collect()
}

pub(super) fn dddguiying_max_he_cards(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
) -> Vec<CardId> {
    let cards = he_cards(core, p);
    let max_rank = cards
        .iter()
        .map(|&c| catalog.cards[c as usize].rank)
        .max()
        .unwrap_or(0);
    cards
        .into_iter()
        .filter(|&c| catalog.cards[c as usize].rank == max_rank)
        .collect()
}

pub(super) fn dddguiying_min_ej_rank(core: &GameStateCore, catalog: &Catalog) -> Option<u8> {
    core.alive_players()
        .flat_map(|p| {
            core.player(p)
                .equips
                .iter()
                .chain(core.player(p).judge.iter())
                .copied()
        })
        .map(|c| catalog.cards[c as usize].rank)
        .min()
}

pub(super) fn dddguiying_min_ej_of(
    core: &GameStateCore,
    catalog: &Catalog,
    p: PlayerId,
) -> Vec<CardId> {
    let Some(min_rank) = dddguiying_min_ej_rank(core, catalog) else {
        return Vec::new();
    };
    core.player(p)
        .equips
        .iter()
        .chain(core.player(p).judge.iter())
        .copied()
        .filter(|&c| catalog.cards[c as usize].rank == min_rank)
        .collect()
}

pub(super) fn build_ask_a_fix(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    match ask_id {
        // ---- dddjuxian (1001..=1004) ----
        1001 => {
            let mut opts = Vec::new();
            let top_used = is_scope4_used(core, owner, "dddjuxian_top");
            let hand_used = is_scope4_used(core, owner, "dddjuxian_hand");
            if !top_used && !he_cards(core, owner).is_empty() {
                opts.push(ChoiceOption::Tag(0));
            }
            if !hand_used && !he_cards(core, owner).is_empty() && core.alive_players().any(|p| p != owner) {
                opts.push(ChoiceOption::Tag(1));
            }
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "dddjuxian_mode",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        1002 => {
            let t = target?;
            let player = core.player(t);
            let options = crate::event::qinglong_options(owner, t, &player.hand, &player.equips);
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddjuxian_gain",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        1003 => {
            let options = he_options(core, owner);
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddjuxian_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        1004 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner && !he_cards(core, p).is_empty())
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddjuxian_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        1038 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner)
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddjuxian_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        // ---- dddzhishu (1005..=1006) ----
        1005 => {
            let options: Vec<ChoiceOption> = zhishu_movable_cards(core, catalog)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddzhishu_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        1006 => {
            let cid = card?;
            let options: Vec<ChoiceOption> = zhishu_valid_dests(core, catalog, cid)
                .into_iter()
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddzhishu_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        // ---- dddyeshen (1007..=1009) ----
        1007 => Some((owner, ChoiceSpec::confirm("dddyeshen_confirm"))),
        1008 => {
            let t = target?;
            let options: Vec<ChoiceOption> = cards
                .iter()
                .copied()
                .filter(|&c| is_black(catalog, c))
                .map(ChoiceOption::Card)
                .collect();
            (!options.is_empty()).then_some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddyeshen_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        1009 => {
            let t = target?;
            let picked = card?;
            let max_num = card_zh_name_len(catalog, picked).min(core.alive_players().count()).max(1);
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| {
                    !is_target_protected(core, p)
                        && !crate::rvm_bridge::modifiers::eval_card_target_canceled(
                            core,
                            catalog,
                            t,
                            p,
                            picked,
                            CardKind::Chain,
                            DamageNature::Normal,
                        )
                })
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddyeshen_target",
                    options,
                    min: 1,
                    max: max_num,
                    allow_cancel: true,
                },
            ))
        }
        // ---- dddguiying (1010..=1013) ----
        1010 => Some((target?, ChoiceSpec::confirm("dddguiying_confirm"))),
        1011 => {
            let t = target?;
            let options: Vec<ChoiceOption> = dddguiying_max_he_cards(core, catalog, t)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            (!options.is_empty()).then_some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddguiying_give_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        1012 => {
            let t = target?;
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| !dddguiying_min_ej_of(core, catalog, p).is_empty())
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddguiying_gain_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        1013 => {
            let t = target?;
            let p2 = card? as PlayerId;
            let options: Vec<ChoiceOption> = dddguiying_min_ej_of(core, catalog, p2)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            (!options.is_empty()).then_some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddguiying_gain_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        // ---- dddjungui (1014..=1016) ----
        1014 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddjungui_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        1015 => {
            let used =
                turn_flag(core, owner, "dddjungui_suits_mask", "dddjungui_suits_stamp")
                    .count_ones() as usize;
            if used == 0 {
                return None;
            }
            let options: Vec<ChoiceOption> = (1..=used).map(|i| ChoiceOption::Tag(i as u8)).collect();
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "dddjungui_reduce",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        1016 => {
            let t = target?;
            let need = card? as usize;
            let options = hand_options(core, t);
            (need > 0 && options.len() >= need).then_some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddjungui_discard",
                    options,
                    min: need,
                    max: need,
                    allow_cancel: false,
                },
            ))
        }
        // ---- dddshilie (1017, 1029) ----
        1017 => {
            let unshown = dddshilie_unshown_hand(core, owner);
            let options: Vec<ChoiceOption> = unshown.into_iter().map(ChoiceOption::Card).collect();
            let max = options.len();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddshilie_cards",
                    options,
                    min: 1,
                    max,
                    allow_cancel: true,
                },
            ))
        }
        1029 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| is_sha_legal_play_target(core, catalog, owner, p))
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddshilie_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        // ---- dddqiaoduan (1019..=1021) ----
        1019 => {
            let n = card? as usize;
            let options = hand_options(core, owner);
            (options.len() >= n && n > 0).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddqiaoduan_cards",
                    options,
                    min: n,
                    max: n,
                    allow_cancel: true,
                },
            ))
        }
        1020 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| core.player(p).hp < core.player(p).max_hp as i32)
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddqiaoduan_recover_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        1021 => {
            let n = card? as usize;
            let options: Vec<ChoiceOption> = core.alive_players().map(ChoiceOption::Player).collect();
            (n > 0 && !options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddqiaoduan_draw_targets",
                    options,
                    min: 1,
                    max: n,
                    allow_cancel: true,
                },
            ))
        }
        // ---- ddddiedang (1023..=1024) ----
        1023 => {
            let options = he_options(core, owner);
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "ddddiedang_discard",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        1024 => {
            let options = he_options(core, owner);
            (options.len() >= 3).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "ddddiedang_discard",
                    options,
                    min: 3,
                    max: 3,
                    allow_cancel: true,
                },
            ))
        }
        // ---- dddanliu (1025..=1028) ----
        1025 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner && !core.player(p).hand.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddanliu_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        1026 => {
            let options = hand_options(core, target?);
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddanliu_pick_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        1027 => {
            let options = hand_options(core, owner);
            (!options.is_empty()).then_some((
                target?,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddanliu_pick_owner",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        1028 => Some((owner, ChoiceSpec::confirm("dddanliu_swap_confirm"))),
        // ---- dddbingjian (1030..=1033) ----
        1030 => {
            let diff = core.player(owner).hand.len().saturating_sub(2);
            let options = hand_options(core, owner);
            (diff > 0 && options.len() >= diff).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddbingjian_discard",
                    options,
                    min: diff,
                    max: diff,
                    allow_cancel: true,
                },
            ))
        }
        1031 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner)
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddbingjian_partner",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        1032 => {
            let diff = card? as usize;
            let t = target?;
            let player = core.player(t);
            let options = crate::event::qinglong_options(owner, t, &player.hand, &player.equips);
            let n = diff.min(options.len());
            (n > 0).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddbingjian_discard_target",
                    options,
                    min: n,
                    max: n,
                    allow_cancel: false,
                },
            ))
        }
        1033 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| is_sha_legal_play_target(core, catalog, owner, p))
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddbingjian_sha_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        // ---- dddtongyu (1034..=1037) ----
        1034 => {
            let used_mask = cards
                .iter()
                .fold(0u32, |acc, &c| acc | effective_suit_bit(core, catalog, owner, c));
            let options: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| (effective_suit_bit(core, catalog, owner, c) & used_mask) == 0)
                .map(ChoiceOption::Card)
                .collect();
            let max = options.len().min(4);
            (max > 0).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddtongyu_cards",
                    options,
                    min: 1,
                    max,
                    allow_cancel: true,
                },
            ))
        }
        1035 => Some((
            owner,
            ChoiceSpec {
                kind: ChoiceKind::PickTag,
                prompt: "dddtongyu_direction",
                options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                min: 1,
                max: 1,
                allow_cancel: false,
            },
        )),
        1036 => {
            let t = target?;
            Some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "dddtongyu_target_choice",
                    options: vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)],
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        1037 => {
            let options: Vec<ChoiceOption> = _targets
                .iter()
                .copied()
                .filter(|&p| core.player(p).alive)
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddtongyu_remain_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        1018 | 1022 => None,
        _ => None,
    }
}
