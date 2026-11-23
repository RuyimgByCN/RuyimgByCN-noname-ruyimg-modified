//! U05-5b Batch A 询问 `ChoiceSpec` 构建与快照恢复共享模块。

use super::*;

pub(super) const ASK_SQLONGNU_PICK: u32 = 1;
pub(super) const ASK_SQLONGNU_DAMAGE: u32 = 2;
pub(super) const ASK_SQLONGWU_USE_CARD: u32 = 3;
pub(super) const ASK_SQLONGWU_USE_TARGET: u32 = 4;
pub(super) const ASK_JIELUE_TARGET: u32 = 5;
pub(super) const ASK_GWHUANSHUANG_PICK: u32 = 6;
pub(super) const ASK_GWHUANSHUANG_TARGET: u32 = 7;
pub(super) const ASK_GWJINLI_CARD: u32 = 8;
pub(super) const ASK_GWJINLI_TARGET: u32 = 9;
pub(super) const ASK_GWHUANBI_COST: u32 = 10;
pub(super) const ASK_GWHUANBI_PICK: u32 = 11;
pub(super) const ASK_GWHUANBI_GIVE: u32 = 12;
pub(super) const ASK_GWMINXIANG_CARD: u32 = 13;
pub(super) const ASK_GWMINXIANG_TARGETS: u32 = 14;
pub(super) const ASK_GWLANGSHI_TARGET: u32 = 15;
pub(super) const ASK_GWJINGSHI_GUESS: u32 = 17;
pub(super) const ASK_GWJINGSHI_STEAL: u32 = 18;
pub(super) const ASK_GWGOUHUN_CARD: u32 = 19;
pub(super) const ASK_GWGOUHUN_TARGET: u32 = 20;
pub(super) const ASK_GWGOUHUN_REPLY: u32 = 21;
pub(super) const ASK_GWFUTIAN_TARGET: u32 = 22;
pub(super) const ASK_GWFUTIAN_DISCARD: u32 = 23;
pub(super) const ASK_GWYEWU_COST: u32 = 24;

pub(super) const GWMAOXIAN_NAMES: &[&str] = &[
    "gwmaoxian_yioufeisi",
    "gwmaoxian_luoqi",
    "gwmaoxian_jieluote",
    "gwmaoxian_yenaifa",
    "gwmaoxian_telisi",
    "gwmaoxian_hengsaite",
    "gwmaoxian_fuertaisite",
    "gwmaoxian_laduoweide",
    "gwmaoxian_enxier",
    "gwmaoxian_fulisi",
    "gwmaoxian_kaerweite",
    "gwmaoxian_bulanwang",
    "gwmaoxian_kuite",
    "gwmaoxian_haluo",
    "gwmaoxian_dagong",
    "gwmaoxian_gaier",
    "gwmaoxian_airuiting",
    "gwmaoxian_aisinie",
    "gwmaoxian_falanxisika",
    "gwmaoxian_huoge",
];

pub(super) fn minxiang_valid_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    c: CardId,
) -> Vec<ChoiceOption> {
    let kind = catalog.card_kind(c);
    let name = clean_gwent_name(catalog.cards[c as usize].name);
    if !(kind.is_basic_kind() || kind.is_trick_kind())
        || matches!(
            kind,
            CardKind::Le
                | CardKind::Bing
                | CardKind::ShanDian
                | CardKind::HongShui
                | CardKind::Guiyoujie
        )
    {
        return Vec::new();
    }
    if kind.aoe_need().is_some()
        || matches!(
            kind,
            CardKind::Shan
                | CardKind::Nullify
                | CardKind::Taoyuan
                | CardKind::Wugu
                | CardKind::Chain
                | CardKind::Jiedao
        )
        || (kind == CardKind::Wuzhong && (name.starts_with("gw") || name.starts_with("tuteng")))
    {
        return Vec::new();
    }
    if crate::kernel::is_gw_dudayuanshuai2_locked(core, catalog, owner, c) {
        return Vec::new();
    }
    core.seat_order_from(owner)
        .into_iter()
        .filter(|&p| is_single_target_legal_no_dist(core, catalog, owner, c, p))
        .map(ChoiceOption::Player)
        .collect()
}

pub(super) fn minxiang_card_eligible(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    c: CardId,
) -> bool {
    minxiang_valid_targets(core, catalog, owner, c).len() >= 2
}

pub(super) fn sqlongwu_playable_hand(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
) -> Vec<ChoiceOption> {
    core.player(owner)
        .hand
        .iter()
        .copied()
        .filter(|&c| b5b_card_can_use(core, catalog, owner, c, true))
        .map(ChoiceOption::Card)
        .collect()
}

pub(super) fn jielue_card_targets(
    core: &GameStateCore,
    catalog: &Catalog,
    owner: PlayerId,
    c: CardId,
) -> Option<(Vec<ChoiceOption>, usize, usize)> {
    b5b_card_targets(core, catalog, owner, c)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_ask_group_a(
    ask_id: u32,
    owner: PlayerId,
    target: Option<PlayerId>,
    _targets: &[PlayerId],
    card: Option<u32>,
    cards: &[CardId],
    core: &GameStateCore,
    catalog: &Catalog,
) -> Option<ExpectedAsk> {
    match ask_id {
        ASK_SQLONGNU_PICK => {
            let opts: Vec<ChoiceOption> = cards
                .iter()
                .copied()
                .filter(|c| core.deck.contains(c))
                .map(ChoiceOption::Card)
                .collect();
            Some(ask_spec(
                owner,
                "sqlongnu_pick",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_SQLONGNU_DAMAGE => {
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| core.player(p).alive)
                .map(ChoiceOption::Player)
                .collect();
            Some(ask_spec(
                owner,
                "sqlongnu_damage",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_SQLONGWU_USE_CARD => Some(ask_spec(
            owner,
            "sqlongwu_use_card",
            ChoiceKind::SelectCards,
            sqlongwu_playable_hand(core, catalog, owner),
            1,
            1,
            true,
        )),
        ASK_SQLONGWU_USE_TARGET => {
            let c = card?;
            if let Some(holder) = target {
                if catalog.card_kind(c) == CardKind::Jiedao {
                    let victims = b5b_jiedao_victim_options(core, catalog, owner, holder);
                    return Some(ask_spec(
                        owner,
                        "sqlongwu_use_target",
                        ChoiceKind::SelectPlayers,
                        victims,
                        1,
                        1,
                        false,
                    ));
                }
            }
            let (opts, min_t, max_t) = b5b_card_targets(core, catalog, owner, c)?;
            Some(ask_spec(
                owner,
                "sqlongwu_use_target",
                ChoiceKind::SelectPlayers,
                opts,
                min_t,
                max_t,
                false,
            ))
        }
        ASK_JIELUE_TARGET => {
            let c = *cards.first()?;
            if let Some(holder) = target {
                if catalog.card_kind(c) == CardKind::Jiedao {
                    let victims = b5b_jiedao_victim_options(core, catalog, owner, holder);
                    return Some(ask_spec(
                        owner,
                        "jielue_target",
                        ChoiceKind::SelectPlayers,
                        victims,
                        1,
                        1,
                        false,
                    ));
                }
            }
            let (opts, min_t, max_t) = jielue_card_targets(core, catalog, owner, c)?;
            Some(ask_spec(
                owner,
                "jielue_target",
                ChoiceKind::SelectPlayers,
                opts,
                min_t,
                max_t,
                false,
            ))
        }
        ASK_GWHUANSHUANG_PICK => {
            let n = flag(core, owner, "gw_huanshuang_n");
            let mut opts = Vec::new();
            for i in 0..n {
                let tag = flag(core, owner, &format!("gw_huanshuang_opt:{i}"));
                if tag > 0 {
                    opts.push(ChoiceOption::Tag((tag - 1) as u8));
                }
            }
            if opts.is_empty() {
                opts = eligible_bronze_tags(core, catalog, owner);
            }
            Some(ask_spec(
                owner,
                "gwhuanshuang_pick",
                ChoiceKind::PickTag,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_GWHUANSHUANG_TARGET => {
            let idx = card? as usize;
            let &(_, kind) = BRONZE_SPELL_NAMES.get(idx)?;
            let opts = eligible_bronze_targets(core, catalog, owner, kind);
            Some(ask_spec(
                owner,
                "gwhuanshuang_target",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                1,
                false,
            ))
        }
        ASK_GWJINLI_CARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    !crate::kernel::is_gw_dudayuanshuai2_locked(core, catalog, owner, c)
                })
                .map(ChoiceOption::Card)
                .collect();
            Some(ask_spec(
                owner,
                "gwjinli_card",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_GWJINLI_TARGET => {
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| core.player(p).alive && !has_named_skill(core, catalog, p, "gwjinli_jiu"))
                .map(ChoiceOption::Player)
                .collect();
            Some(ask_spec(
                owner,
                "gwjinli_target",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                1,
                false,
            ))
        }
        ASK_GWHUANBI_COST => {
            let opts: Vec<ChoiceOption> = discardable_hand_and_equips(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            Some(ask_spec(
                owner,
                "gwhuanbi_cost",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_GWHUANBI_PICK => {
            let n = flag(core, owner, "gw_huanbi_n");
            let mut opts = Vec::new();
            for i in 0..n {
                let t = flag(core, owner, &format!("gw_huanbi_opt:{i}"));
                if t > 0 {
                    opts.push(ChoiceOption::Tag((t - 1) as u8));
                }
            }
            Some(ask_spec(
                owner,
                "gwhuanbi_pick",
                ChoiceKind::PickTag,
                opts,
                1,
                1,
                false,
            ))
        }
        ASK_GWHUANBI_GIVE => {
            let t = target?;
            let opts: Vec<ChoiceOption> = core
                .player(t)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            Some(ask_spec(
                t,
                "gwhuanbi_give",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_GWMINXIANG_CARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| minxiang_card_eligible(core, catalog, owner, c))
                .map(ChoiceOption::Card)
                .collect();
            Some(ask_spec(
                owner,
                "gwminxiang_card",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_GWMINXIANG_TARGETS => {
            let c = card?;
            let opts = minxiang_valid_targets(core, catalog, owner, c);
            Some(ask_spec(
                owner,
                "gwminxiang_targets",
                ChoiceKind::SelectPlayers,
                opts,
                2,
                2,
                true,
            ))
        }
        ASK_GWLANGSHI_TARGET => {
            let victim = target?;
            let min_hp = core.player(victim).hp;
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| {
                    p != owner
                        && p != victim
                        && core.player(p).alive
                        && core.player(p).hp >= min_hp
                })
                .map(ChoiceOption::Player)
                .collect();
            Some(ask_spec(
                owner,
                "gwlangshi_target",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_GWJINGSHI_GUESS => {
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| core.player(p).alive && !core.player(p).hand.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            Some(ask_spec(
                owner,
                "gwjingshi_guess",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                1,
                false,
            ))
        }
        ASK_GWJINGSHI_STEAL => {
            let mut opts = Vec::new();
            for p in core.seat_order_from(owner) {
                if p != owner && core.player(p).alive {
                    opts.extend(core.player(p).hand.iter().copied().map(ChoiceOption::Card));
                }
            }
            Some(ask_spec(
                owner,
                "gwjingshi_steal",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                false,
            ))
        }
        ASK_GWGOUHUN_CARD => {
            let opts: Vec<ChoiceOption> = core
                .player(owner)
                .hand
                .iter()
                .copied()
                .filter(|&c| {
                    !crate::kernel::is_gw_dudayuanshuai2_locked(core, catalog, owner, c)
                })
                .map(ChoiceOption::Card)
                .collect();
            Some(ask_spec(
                owner,
                "gwgouhun_card",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            ))
        }
        ASK_GWGOUHUN_TARGET => {
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| p != owner && core.player(p).alive && !core.player(p).hand.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            Some(ask_spec(
                owner,
                "gwgouhun_target",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                1,
                false,
            ))
        }
        ASK_GWGOUHUN_REPLY => {
            let t = target?;
            let suit_tag = card? as u8;
            let num1 = core
                .player(t)
                .hand
                .iter()
                .filter(|&&c| suit_to_tag(catalog.cards[c as usize].suit) == suit_tag)
                .count();
            let num2 = core.player(t).hand.len().saturating_sub(num1);
            let mut opts = Vec::new();
            if num1 > 0 {
                opts.push(ChoiceOption::Tag(0));
            }
            if num2 > 0 {
                opts.push(ChoiceOption::Tag(1));
            }
            opts.push(ChoiceOption::Tag(2));
            Some(ask_spec(
                t,
                "gwgouhun_reply",
                ChoiceKind::PickTag,
                opts,
                1,
                1,
                false,
            ))
        }
        ASK_GWFUTIAN_TARGET => {
            let opts: Vec<ChoiceOption> = core
                .seat_order_from(owner)
                .into_iter()
                .filter(|&p| p != owner && core.player(p).alive && !core.player(p).hand.is_empty())
                .map(ChoiceOption::Player)
                .collect();
            Some(ask_spec(
                owner,
                "gwfutian_target",
                ChoiceKind::SelectPlayers,
                opts,
                1,
                1,
                false,
            ))
        }
        ASK_GWFUTIAN_DISCARD => {
            let t = target?;
            let opts: Vec<ChoiceOption> = core
                .player(t)
                .hand
                .iter()
                .copied()
                .map(ChoiceOption::Card)
                .collect();
            Some(ask_spec(
                owner,
                "gwfutian_discard",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                false,
            ))
        }
        ASK_GWYEWU_COST => {
            let opts: Vec<ChoiceOption> = discardable_hand(core, catalog, owner)
                .into_iter()
                .map(ChoiceOption::Card)
                .collect();
            Some(ask_spec(
                owner,
                "gwyewu_cost",
                ChoiceKind::SelectCards,
                opts,
                1,
                1,
                true,
            ))
        }
        _ => None,
    }
}

pub(super) fn suit_to_tag(s: Suit) -> u8 {
    match s {
        Suit::Spade => 0,
        Suit::Heart => 1,
        Suit::Club => 2,
        Suit::Diamond => 3,
    }
}
