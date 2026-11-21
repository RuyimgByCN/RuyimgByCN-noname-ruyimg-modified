use super::*;

pub(crate) fn turn_used_or_responded_players(core: &GameStateCore) -> Vec<PlayerId> {
    let stamp = l28_stamp(core);
    if flag(core, 0, "turn_use_resp_stamp") != stamp {
        return Vec::new();
    }
    let mask = flag(core, 0, "turn_use_resp_mask");
    (0..core.players.len() as PlayerId)
        .filter(|&p| (mask & (1u32 << p)) != 0)
        .collect()
}

pub(crate) fn dddzhilian_targets_eligible(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Option<Vec<PlayerId>> {
    let players = turn_used_or_responded_players(core);
    if players.len() != 2
        || !players.contains(&owner)
        || players.iter().any(|&p| !core.player(p).alive)
    {
        return None;
    }
    if players.iter().any(|&p| p != owner && is_male(core, catalog, p)) {
        Some(
            core.seat_order_from(core.turn.current)
                .into_iter()
                .filter(|p| players.contains(p))
                .collect(),
        )
    } else {
        None
    }
}

pub(super) fn dddjijian_color_valid_cards(core: &GameStateCore, catalog: &Catalog, p: PlayerId) -> Vec<CardId> {
    core.player(p)
        .hand
        .iter()
        .copied()
        .filter(|&c| {
            let k = catalog.card_kind(c);
            k.is_sha_like() || (!k.is_basic_kind() && !k.is_equip() && k.is_basic_trick_or_equip() && !is_delayed(k))
        })
        .collect()
}

pub(super) fn dddjijian_old_snapshot_names(
    core: &GameStateCore,
    catalog: &Catalog,
    t: PlayerId,
    shown: &[CardId],
) -> Vec<u32> {
    let mut names = Vec::new();
    for &c in &core.player(t).hand {
        if shown.contains(&c) {
            continue;
        }
        let k = canonical_virt_kind(catalog, c);
        if !(k.is_basic_kind() || (!k.is_equip() && k.is_basic_trick_or_equip() && !is_delayed(k))) {
            continue;
        }
        names.push(u32::from(k as u16));
    }
    names
}

pub(super) fn dddjijian_old_usable_codes(
    core: &GameStateCore,
    catalog: &Catalog,
    t: PlayerId,
    rem_names: &[u32],
) -> Vec<u16> {
    let mut opts = Vec::new();
    for &raw in rem_names {
        let Ok(code) = u16::try_from(raw) else {
            continue;
        };
        let Some(k) = kind_from_code(code) else {
            continue;
        };
        if !opts.contains(&code) && virtual_kind_can_use_play(core, catalog, t, k) {
            opts.push(code);
        }
    }
    opts
}

pub(super) fn kind_from_code(code: u16) -> Option<CardKind> {
    match code {
        x if x == CardKind::Sha as u16 => Some(CardKind::Sha),
        x if x == CardKind::FireSha as u16 => Some(CardKind::FireSha),
        x if x == CardKind::LeiSha as u16 => Some(CardKind::LeiSha),
        x if x == CardKind::Shan as u16 => Some(CardKind::Shan),
        x if x == CardKind::Tao as u16 => Some(CardKind::Tao),
        x if x == CardKind::Jiu as u16 => Some(CardKind::Jiu),
        x if x == CardKind::Duel as u16 => Some(CardKind::Duel),
        x if x == CardKind::Dismantle as u16 => Some(CardKind::Dismantle),
        x if x == CardKind::Snatch as u16 => Some(CardKind::Snatch),
        x if x == CardKind::Wuzhong as u16 => Some(CardKind::Wuzhong),
        x if x == CardKind::Taoyuan as u16 => Some(CardKind::Taoyuan),
        x if x == CardKind::Wugu as u16 => Some(CardKind::Wugu),
        x if x == CardKind::SavageAttack as u16 => Some(CardKind::SavageAttack),
        x if x == CardKind::ArcheryAttack as u16 => Some(CardKind::ArcheryAttack),
        x if x == CardKind::Chain as u16 => Some(CardKind::Chain),
        x if x == CardKind::Jiedao as u16 => Some(CardKind::Jiedao),
        x if x == CardKind::Linghunzhihuo as u16 => Some(CardKind::Linghunzhihuo),
        x if x == CardKind::ShouLiJian as u16 => Some(CardKind::ShouLiJian),
        x if x == CardKind::WangmeiZhike as u16 => Some(CardKind::WangmeiZhike),
        x if x == CardKind::Shuigong as u16 => Some(CardKind::Shuigong),
        _ => None,
    }
}

pub(super) fn build_ask_bcd_fix(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<(PlayerId, ChoiceSpec)> {
    match ask_id {
        // ---- dddzhilian (1051..=1052) ----
        1051 => {
            let tgts = dddzhilian_targets_eligible(core, catalog, owner)?;
            let mut opts = vec![ChoiceOption::Tag(0)];
            if tgts.iter().any(|&p| !he_cards(core, p).is_empty()) {
                opts.push(ChoiceOption::Tag(1));
            }
            Some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "dddzhilian_mode",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        1052 => {
            let t = target?;
            let options = he_options(core, t);
            (!options.is_empty()).then_some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddzhilian_discard",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        // ---- dddjijian (1053..=1057) ----
        1053 => {
            let options: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.card_kind(c).is_sha_like())
                .map(ChoiceOption::Card)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddjijian_sha",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        1054 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner && is_male(core, catalog, p))
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddjijian_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        1055 => {
            let t = target?;
            let options: Vec<ChoiceOption> = dddjijian_color_valid_cards(core, catalog, t)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            let max = options.len().min(2);
            (max > 0).then_some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddjijian_show",
                    options,
                    min: 1,
                    max,
                    allow_cancel: false,
                },
            ))
        }
        1056 => {
            let actor = target?;
            let shown_cnt = card? as usize;
            let used = &cards[shown_cnt..];
            let mut opts = Vec::new();
            for &c in &cards[..shown_cnt] {
                let k = canonical_virt_kind(catalog, c);
                let ckey = canonical_jijian_key(k);
                let code = k as u16;
                if !used.contains(&ckey)
                    && virtual_kind_can_use_play(core, catalog, actor, k)
                    && !opts.contains(&ChoiceOption::Kind(code))
                {
                    opts.push(ChoiceOption::Kind(code));
                }
            }
            (!opts.is_empty()).then_some((
                actor,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "dddjijian_use_card",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        1057 => {
            let actor = target?;
            let vcode = flag(core, owner, "u01_active_vkind");
            let raw_code = if vcode != 0 {
                vcode
            } else {
                cards.last().copied().unwrap_or(0)
            };
            let k = kind_from_code(u16::try_from(raw_code).ok()?)?;
            let jh = flag(core, actor, "u01_jiedao_holder");
            let tgts = if k == CardKind::Jiedao && jh != 0 {
                jiedao_victims_for(core, catalog, (jh - 1) as PlayerId)
            } else {
                virtual_legal_targets(core, catalog, actor, k)
            };
            let options: Vec<ChoiceOption> = tgts.into_iter().map(ChoiceOption::Player).collect();
            let max = if k == CardKind::Chain { 2 } else { 1 };
            (!options.is_empty()).then_some((
                actor,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddjijian_use_target",
                    options,
                    min: 1,
                    max,
                    allow_cancel: false,
                },
            ))
        }
        // ---- dddjijian_old (1058..=1062) ----
        1058 => {
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| p != owner && is_male(core, catalog, p) && !core.player(p).hand.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddjijian_old_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        1059 => {
            let t = target?;
            let options = hand_options(core, t);
            let max = options.len().min(2);
            (max > 0).then_some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddjijian_old_show",
                    options,
                    min: 1,
                    max,
                    allow_cancel: false,
                },
            ))
        }
        1060 => {
            let options: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.card_kind(c).is_sha_like())
                .map(ChoiceOption::Card)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddjijian_old_give_sha",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        1061 => {
            let t = target?;
            let shown_cnt = cards.first().copied().unwrap_or(0) as usize;
            let rem_names = &cards[1 + shown_cnt..];
            let codes = dddjijian_old_usable_codes(core, catalog, t, rem_names);
            let options: Vec<ChoiceOption> = codes.into_iter().map(ChoiceOption::Kind).collect();
            (!options.is_empty()).then_some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "dddjijian_old_use_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        1062 => {
            let t = target?;
            let k = kind_from_code(u16::try_from(card?).ok()?)?;
            let jh = flag(core, t, "u01_jiedao_holder");
            let tgts = if k == CardKind::Jiedao && jh != 0 {
                jiedao_victims_for(core, catalog, (jh - 1) as PlayerId)
            } else {
                virtual_legal_targets(core, catalog, t, k)
            };
            let options: Vec<ChoiceOption> = tgts.into_iter().map(ChoiceOption::Player).collect();
            let max = if k == CardKind::Chain { 2 } else { 1 };
            (!options.is_empty()).then_some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddjijian_old_use_target",
                    options,
                    min: 1,
                    max,
                    allow_cancel: false,
                },
            ))
        }
        1072 => {
            let hg_tgt = flag(core, owner, "u01_hg_target").checked_sub(1)? as PlayerId;
            let options = hand_options(core, hg_tgt);
            (!options.is_empty()).then_some((
                hg_tgt,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddjijian_huogong_show",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        1073 => {
            let actor = target?;
            let hg_tgt = flag(core, owner, "u01_hg_target").checked_sub(1)? as PlayerId;
            let shown_c = flag(core, owner, "u01_hg_shown");
            let suit = catalog.effective_card_suit(core, hg_tgt, shown_c);
            let options: Vec<ChoiceOption> = core
                .player(actor)
                .hand
                .iter()
                .copied()
                .filter(|&c| catalog.effective_card_suit(core, actor, c) == suit)
                .map(ChoiceOption::Card)
                .collect();
            (!options.is_empty()).then_some((
                actor,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddjijian_huogong_discard",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        // ---- dddfushi (1063..=1065) ----
        1063 => {
            let p1 = core.vm_aux.piles.get(&(owner, "dddmiaoxing_1".to_string())).is_some_and(|v| !v.is_empty());
            let p2 = core.vm_aux.piles.get(&(owner, "dddmiaoxing_2".to_string())).is_some_and(|v| !v.is_empty());
            let has_hand = !core.player(owner).hand.is_empty();
            let mut opts = Vec::new();
            if p1 || has_hand {
                opts.push(ChoiceOption::Tag(0));
            }
            if p2 || has_hand {
                opts.push(ChoiceOption::Tag(1));
            }
            (!opts.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "dddfushi_pile",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        1064 => {
            let p1 = core.vm_aux.piles.get(&(owner, "dddmiaoxing_1".to_string())).is_some_and(|v| !v.is_empty());
            let p2 = core.vm_aux.piles.get(&(owner, "dddmiaoxing_2".to_string())).is_some_and(|v| !v.is_empty());
            let options: Vec<ChoiceOption> = core
                .alive_players()
                .filter(|&p| {
                    is_male(core, catalog, p) && (!core.player(p).hand.is_empty() || p1 || p2)
                })
                .map(ChoiceOption::Player)
                .collect();
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectPlayers,
                    prompt: "dddfushi_male_target",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        1065 => {
            let t = target?;
            let p1 = core.vm_aux.piles.get(&(owner, "dddmiaoxing_1".to_string())).is_some_and(|v| !v.is_empty());
            let p2 = core.vm_aux.piles.get(&(owner, "dddmiaoxing_2".to_string())).is_some_and(|v| !v.is_empty());
            let has_hand = !core.player(t).hand.is_empty();
            let mut opts = Vec::new();
            if p1 || has_hand {
                opts.push(ChoiceOption::Tag(0));
            }
            if p2 || has_hand {
                opts.push(ChoiceOption::Tag(1));
            }
            (!opts.is_empty()).then_some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::PickTag,
                    prompt: "dddfushi_target_pile",
                    options: opts,
                    min: 1,
                    max: 1,
                    allow_cancel: true,
                },
            ))
        }
        // ---- dddlangzhi (1066..=1069) ----
        1066 => Some((owner, ChoiceSpec::confirm("dddlangzhi_confirm"))),
        1067 => {
            let t = target?;
            let player = core.player(t);
            let options = crate::event::qinglong_options(owner, t, &player.hand, &player.equips);
            (!options.is_empty()).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddlangzhi_reveal_target_card",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        1068 => {
            let options = he_options(core, owner);
            let max = options.len().min(cards.len());
            (max > 0).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddlangzhi_own_card",
                    options,
                    min: 1,
                    max,
                    allow_cancel: true,
                },
            ))
        }
        1069 => {
            let k = card? as usize;
            let pool_end = cards.len().saturating_sub(k);
            let options: Vec<ChoiceOption> = cards[..pool_end].iter().copied().map(ChoiceOption::Card).collect();
            (options.len() >= k && k > 0).then_some((
                owner,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddlangzhi_swap_cards",
                    options,
                    min: k,
                    max: k,
                    allow_cancel: false,
                },
            ))
        }
        // ---- dddfuzong (1070) ----
        1070 => {
            let t = target?;
            let _ = targets;
            let player = core.player(owner);
            let options = crate::event::qinglong_options(t, owner, &player.hand, &player.equips);
            (!options.is_empty()).then_some((
                t,
                ChoiceSpec {
                    kind: ChoiceKind::SelectCards,
                    prompt: "dddfuzong_discard",
                    options,
                    min: 1,
                    max: 1,
                    allow_cancel: false,
                },
            ))
        }
        _ => None,
    }
}
