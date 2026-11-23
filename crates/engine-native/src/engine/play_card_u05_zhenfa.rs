//! Batch U05-1 阵法牌 (`recastable: true`)、万智牌扩展地 (`mtg_duzhao`/`mtg_cangbaohaiwan`)、
//! 毒牌 (`mtg_du_*`)、火山 (`huoshan`/`yc_huoshan_*`) 与隔岸观火 (`<=2` 重铸) 校验与分派。

use super::*;

fn raw_base_key(raw: &str) -> &str {
    match raw.rsplit_once('_') {
        Some((stem, tail)) if !tail.is_empty() && tail.bytes().all(|b| b.is_ascii_digit()) => stem,
        _ => raw,
    }
}

pub(super) fn is_zhenfa_card(catalog: &SharedCatalog, card: CardId, kind: CardKind) -> bool {
    if matches!(
        kind,
        CardKind::Dizaizhen
            | CardKind::Changshezhen
            | CardKind::Pozhenjue
            | CardKind::Tianfuzhen
            | CardKind::Qixingzhen
            | CardKind::Niaoxiangzhen
    ) {
        return true;
    }
    catalog.cards.get(card as usize).is_some_and(|cd| {
        matches!(
            raw_base_key(cd.name),
            "fengyangzhen"
                | "zf_fengyangzhen"
                | "yunchuizhen"
                | "zf_yunchuizhen"
                | "shepanzhen"
                | "zf_shepanzhen"
                | "longfeizhen"
                | "zf_longfeizhen"
                | "huyizhen"
                | "zf_huyizhen"
        )
    })
}

fn execute_recast_hand_card(state: &mut GameState, actor: PlayerId, card: CardId) {
    state.core.player_mut(actor).hand.retain(|&c| c != card);
    if !state.core.discard.contains(&card) {
        state.core.discard.push(card);
    }
    crate::resolution::push_frame(
        &mut state.resolution,
        EventData::Draw {
            player: actor,
            requested: 1,
            held_cards: Vec::new(),
            terminal: crate::event::DrawTerminal::Running,
        },
    );
    crate::resolution::push_frame(
        &mut state.resolution,
        EventData::CardsDiscarded {
            player: actor,
            cards: vec![card],
        },
    );
}

fn zhenfa_neighbors(core: &GameStateCore, actor: PlayerId) -> Vec<PlayerId> {
    let (next, prev) = core.alive_neighbors(actor);
    let mut out = Vec::new();
    if let Some(n) = next {
        if n != actor {
            out.push(n);
        }
    }
    if let Some(p) = prev {
        if p != actor && !out.contains(&p) {
            out.push(p);
        }
    }
    out
}

fn is_zhenfa_enabled(
    core: &GameStateCore,
    catalog: &SharedCatalog,
    actor: PlayerId,
    card: CardId,
    kind: CardKind,
) -> bool {
    let key = catalog
        .cards
        .get(card as usize)
        .map(|cd| raw_base_key(cd.name))
        .unwrap_or("");
    match (kind, key) {
        (CardKind::Pozhenjue, _) => true,
        (CardKind::Changshezhen, _) => {
            core.effective_faction(catalog, actor).is_some()
                && (core.inline(catalog, actor)
                    || core
                        .alive_players()
                        .any(|p| p != actor && core.is_friend_of(catalog, actor, p)))
        }
        (CardKind::Dizaizhen, _) => core.alive_players().any(|p| !core.is_major_faction(catalog, p)),
        (CardKind::Tianfuzhen, _) => {
            core.players.len() >= 4
                && core.alive_players().any(|p| {
                    core.is_major_faction(catalog, p)
                        && (!core.player(p).hand.is_empty() || !core.player(p).equips.is_empty())
                })
        }
        (CardKind::Qixingzhen, _) => {
            let under_siege = core.alive_players().any(|q| core.sieges(catalog, q, actor));
            let (n, pv) = core.alive_neighbors(actor);
            let besieging = n.is_some_and(|x| core.sieged(catalog, x))
                || pv.is_some_and(|x| core.sieged(catalog, x));
            core.players.len() >= 4 && (under_siege || besieging)
        }
        (CardKind::Niaoxiangzhen, _) => {
            core.players.len() >= 4
                && core.alive_players().any(|t| {
                    t != actor && !core.is_friend_of(catalog, actor, t) && {
                        let (n, pv) = core.alive_neighbors(t);
                        n.is_some_and(|q| core.effective_faction(catalog, t) == core.effective_faction(catalog, q))
                            || pv.is_some_and(|q| core.effective_faction(catalog, t) == core.effective_faction(catalog, q))
                    }
                })
        }
        (_, "fengyangzhen" | "zf_fengyangzhen") => {
            core.players.len() >= 4 && core.alive_players().any(|p| core.sieged(catalog, p))
        }
        (_, "yunchuizhen" | "zf_yunchuizhen") => {
            core.players.len() >= 4
                && core
                    .alive_players()
                    .any(|p| core.alive_players().any(|v| core.sieges(catalog, p, v)))
        }
        (_, "shepanzhen" | "zf_shepanzhen") => {
            core.players.len() >= 4
                && core.effective_faction(catalog, actor).is_some()
                && core.alive_players().any(|p| {
                    p != actor
                        && core.is_friend_of(catalog, actor, p)
                        && !core.inline_with(catalog, actor, p)
                })
        }
        (_, "longfeizhen" | "zf_longfeizhen") => {
            core.players.len() >= 4
                && core.sieged(catalog, actor)
                && zhenfa_neighbors(core, actor).iter().any(|&p| {
                    !core.player(p).hand.is_empty() || !core.player(p).equips.is_empty()
                })
        }
        (_, "huyizhen" | "zf_huyizhen") => {
            core.players.len() >= 4
                && core
                    .alive_players()
                    .any(|v| core.sieges(catalog, actor, v))
        }
        _ => true,
    }
}

/// 优先处理 Batch U05-1 出牌分支；返回 `Some(result)` 表示已接管，`None` 继续走原 `play_card`。
pub(super) fn try_play_u05_b1_special(
    state: &mut GameState,
    catalog: &SharedCatalog,
    actor: PlayerId,
    card: CardId,
    kind: CardKind,
    targets: Vec<PlayerId>,
) -> Option<Result<(), RejectReason>> {
    let raw_name = catalog.cards.get(card as usize).map(|c| c.name).unwrap_or("");
    let key = raw_base_key(raw_name);
    let core = &state.core;

    // 1. `yc_huoshan_0` / `huoshan` (`CardKind::Le`, 自放置延时锦囊)
    if key == "yc_huoshan" || key == "huoshan" {
        if !(targets.is_empty() || targets == [actor]) {
            return Some(Err(RejectReason::IllegalTargets));
        }
        if core
            .player(actor)
            .judge
            .iter()
            .any(|&c| catalog.cards.get(c as usize).is_some_and(|cd| matches!(raw_base_key(cd.name), "yc_huoshan" | "huoshan")))
        {
            return Some(Err(RejectReason::IllegalTargets));
        }
        crate::resolution::push_frame(
            &mut state.resolution,
            EventData::CardUsed(crate::machine::card_use_as(card, actor, vec![actor], Some(CardKind::Le))),
        );
        return Some(Ok(()));
    }

    // 2. `mtg_du_*` 毒牌不可主动使用（`card/yongjian.js`: 无 enable） / `mtg_duzhao` / `mtg_cangbaohaiwan`
    if key == "mtg_du" {
        return Some(Err(RejectReason::WrongPhase));
    }
    if matches!(
        key,
        "mtg_duzhao" | "g_mtg_duzhao" | "mtg_cangbaohaiwan" | "g_mtg_cangbaohaiwan"
    ) {
        if !targets.is_empty() || core.turn.land_used {
            return Some(Err(RejectReason::IllegalTargets));
        }
        crate::resolution::push_frame(
            &mut state.resolution,
            EventData::CardUsed(crate::machine::card_use(card, actor, targets)),
        );
        return Some(Ok(()));
    }

    // 3. `geanguanhuo` (`alive <= 2` 时不可正常使用但可无目标重铸)
    if kind == CardKind::Geanguanhuo {
        let alive_cnt = core.alive_players().count();
        if alive_cnt <= 2 {
            if !targets.is_empty() {
                return Some(Err(RejectReason::IllegalTargets));
            }
            execute_recast_hand_card(state, actor, card);
            return Some(Ok(()));
        }
        return None;
    }

    // 4. 所有 11 张阵法牌 (`card/zhenfa/card.js`): `!enable && targets.is_empty()` 直接重铸；否则校验各自目标
    if is_zhenfa_card(catalog, card, kind) {
        if !is_zhenfa_enabled(core, catalog, actor, card, kind) {
            if targets.is_empty() {
                execute_recast_hand_card(state, actor, card);
                return Some(Ok(()));
            }
            return Some(Err(RejectReason::IllegalTargets));
        }
        match key {
            "fengyangzhen" | "zf_fengyangzhen" | "yunchuizhen" | "zf_yunchuizhen"
            | "shepanzhen" | "zf_shepanzhen" => {
                if !targets.is_empty() {
                    return Some(Err(RejectReason::IllegalTargets));
                }
                crate::resolution::push_frame(
                    &mut state.resolution,
                    EventData::CardUsed(crate::machine::card_use(card, actor, targets)),
                );
                return Some(Ok(()));
            }
            "longfeizhen" | "zf_longfeizhen" => {
                let expected: Vec<PlayerId> = zhenfa_neighbors(core, actor)
                    .into_iter()
                    .filter(|&p| !core.player(p).hand.is_empty() || !core.player(p).equips.is_empty())
                    .collect();
                if expected.is_empty()
                    || targets.len() != expected.len()
                    || !expected.iter().all(|t| targets.contains(t))
                {
                    return Some(Err(RejectReason::IllegalTargets));
                }
                crate::resolution::push_frame(
                    &mut state.resolution,
                    EventData::CardUsed(crate::machine::card_use(card, actor, targets)),
                );
                return Some(Ok(()));
            }
            "huyizhen" | "zf_huyizhen" => {
                let expected: Vec<PlayerId> = core
                    .seat_order_from(actor)
                    .into_iter()
                    .filter(|&p| core.player(p).alive && core.sieges(catalog, actor, p))
                    .collect();
                if expected.is_empty()
                    || targets.len() != expected.len()
                    || !expected.iter().all(|t| targets.contains(t))
                {
                    return Some(Err(RejectReason::IllegalTargets));
                }
                crate::resolution::push_frame(
                    &mut state.resolution,
                    EventData::CardUsed(crate::machine::card_use(card, actor, targets)),
                );
                return Some(Ok(()));
            }
            _ => return None,
        }
    }
    None
}
