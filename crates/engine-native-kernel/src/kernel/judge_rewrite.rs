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

/// 查询单个改判技能在成功改判后是否弃置原判定牌（鬼才 / 新鬼才系）。
pub fn judge_provider_discards_old(
    catalog: &crate::catalog::Catalog,
    provider: crate::ids::SkillDefId,
) -> bool {
    catalog.skill_name(provider) == crate::vocabulary::names::GUI_CAI
        || crate::rvm_bridge::modifiers::skill_has_judgment_rewrite_hand_equip(catalog, provider)
}

/// 查询 `player` 当前尚未在本轮判定介入的改判技能定义列表。
pub fn judge_rewrite_providers(
    core: &crate::state::GameStateCore,
    catalog: &crate::catalog::Catalog,
    player: PlayerId,
    used: &[(PlayerId, crate::ids::SkillDefId)],
) -> Vec<crate::ids::SkillDefId> {
    let mut providers: Vec<crate::ids::SkillDefId> = [
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
    .collect();
    for d in crate::rvm_bridge::modifiers::player_judgment_rewrite_hand_equip_defs(
        core, catalog, player,
    ) {
        if !providers.contains(&d)
            && !used.contains(&(player, d))
            && !judge_provider_cards(core, catalog, player, d).is_empty()
        {
            providers.push(d);
        }
    }
    providers
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
    let reguicai =
        crate::rvm_bridge::modifiers::skill_has_judgment_rewrite_hand_equip(catalog, provider);
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
        .filter(|c| any || cai || reguicai || dao && black(c))
        .chain(p.equips.iter().filter(|c| {
            any || reguicai
                || cai && matches!(core.mode, crate::state::GameMode::Kingdoms)
                || dao && black(c)
        }))
        .copied()
        .collect()
}
