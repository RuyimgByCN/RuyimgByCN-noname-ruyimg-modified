//! 判定阶段改判技能与可选改判牌查询（自 `judge.rs` 拆出，保持判定执行机精简）。

use super::*;

/// Source GuiCai is hs outside guozhan, hes inside; GuiDao is black he, Tiandao is any he.
pub fn judge_rewrite_cards(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    player: PlayerId,
) -> Vec<crate::ids::CardId> {
    let mut cards = Vec::new();
    for def in judge_rewrite_providers(core, catalog, player, &[]) {
        for card in judge_provider_cards(core, catalog, player, def) {
            if !cards.contains(&card) {
                cards.push(card);
            }
        }
    }
    cards
}

/// 查询 `player` 当前尚未在本轮判定介入的改判技能定义列表。
pub fn judge_rewrite_providers(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    player: PlayerId,
    used: &[(PlayerId, crate::ids::SkillDefId)],
) -> Vec<crate::ids::SkillDefId> {
    [
        crate::vocabulary::names::GUI_CAI,
        crate::vocabulary::names::GUI_DAO,
        crate::vocabulary::names::BOSS_TIANDAO,
        "gtiandao",
        crate::vocabulary::host::SWD_TIANLUN_KEY,
    ]
    .into_iter()
    .filter_map(|name| catalog.skill_by_name(name))
    .filter(|d| {
        core.has_skill(player, *d)
            && !used.contains(&(player, *d))
            && (catalog.skill_name(*d) == crate::vocabulary::host::SWD_TIANLUN_KEY
                || !judge_provider_cards(core, catalog, player, *d).is_empty())
    })
    .collect()
}

/// 查询单个改判技能 `provider` 在 `player` 当前区域的可用改判牌。
pub fn judge_provider_cards(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    player: PlayerId,
    provider: crate::ids::SkillDefId,
) -> Vec<crate::ids::CardId> {
    let name = catalog.skill_name(provider);
    let cai = name == crate::vocabulary::names::GUI_CAI;
    let dao = name == crate::vocabulary::names::GUI_DAO;
    let any = name == crate::vocabulary::names::BOSS_TIANDAO || name == "gtiandao";
    let black = |c: &crate::ids::CardId| {
        matches!(
            catalog.cards[*c as usize].suit,
            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
        )
    };
    let p = core.player(player);
    p.hand
        .iter()
        .filter(|c| any || cai || dao && black(c))
        .chain(p.equips.iter().filter(|c| {
            any || cai && matches!(core.mode, crate::state::GameMode::Kingdoms) || dao && black(c)
        }))
        .copied()
        .collect()
}
