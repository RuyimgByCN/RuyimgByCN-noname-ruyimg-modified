//! Standard content packs (characters, deck composition) + live package registry.

mod characters;
mod deck;
mod registry;
mod standard_pack;

pub use characters::{
    character_by_id, guozhan_card_entries, guozhan_character_entries, skill_def, CharacterDef,
    GUOZHAN_CARDS, GUOZHAN_CHARACTERS, STANDARD_CHARACTERS,
};
pub use deck::{
    build_deck_from_config, build_deck_specs, build_standard_deck_specs, card_def_id_for_name,
    guozhan_deck_specs, is_recastable_card_def, DeckCardSpec, EquipSlot,
};
pub use registry::{
    jianxiong_skill_def, ActiveSkillDef, ContentLoadError, ContentRegistry, LoadedPackageMeta,
    SkillMetadata, ViewAsDef, PACKAGE_MODIFIER_ID_BASE, PACKAGE_TRIGGER_ID_BASE,
};
pub use standard_pack::ensure_guozhan_package;
pub(crate) use standard_pack::ensure_standard_content;
pub use standard_pack::ensure_standard_package;

/// Mark def: 酒（下一张杀伤害 +1）。
pub const MARK_JIU: u32 = 1;
/// Mark def: 铁索连环（横置）。
pub const MARK_CHAIN: u32 = 2;
/// Mark def: 护甲（优先吸收普通伤害），上限由获得效果约束。
pub const MARK_HUJIA: u32 = 3;
/// jsrg 蓄谋牌卡牌标记（place_xumou 置牌时打；判定结算时视为使用原牌）。
pub const XUMOU_TAG: u32 = 0x584D_4F55;
/// 劫囚 BUFF 标（tw twjieqiu）：合成 blake2b("twjieqiu_buff",4B)。
pub const TWJIEQIU_BUFF_MARK: u32 = 385_592_780;

/// 八卦阵 card def id (deck).
pub const DEF_BAGUA: u32 = 110;
/// 藤甲 card def id (deck).
pub const DEF_TENGJIA: u32 = 111;

/// Choice token: activate 八卦阵 to judge as 闪 (not a CardId).
pub const BAGUA_RESPONSE_TOKEN: u32 = 0xFFFF_FF01;

/// Physical 八卦阵 card, or `CardId::NONE` when a skill provides a virtual one.
pub fn player_bagua_source(
    state: &crate::state::GameState,
    player: engine_api::PlayerId,
) -> Option<engine_api::CardId> {
    player_equip_card_by_def(state, player, DEF_BAGUA).or_else(|| {
        crate::query::virtual_bagua_available_for(state, player).then_some(engine_api::CardId::NONE)
    })
}

/// Whether `player` can invoke an equipped or virtual 八卦阵 response.
pub fn player_has_bagua(state: &crate::state::GameState, player: engine_api::PlayerId) -> bool {
    player_bagua_source(state, player).is_some()
}

/// Whether `player` has 藤甲 equipped.
pub fn player_has_tengjia(state: &crate::state::GameState, player: engine_api::PlayerId) -> bool {
    player_has_equip_def(state, player, DEF_TENGJIA)
}

/// Whether `player` has an equipment with the given definition id.
pub fn player_has_equip_def(
    state: &crate::state::GameState,
    player: engine_api::PlayerId,
    def: u32,
) -> bool {
    player_equip_card_by_def(state, player, def).is_some()
}

/// Equipped physical card with the requested definition id.
pub fn player_equip_card_by_def(
    state: &crate::state::GameState,
    player: engine_api::PlayerId,
    def: u32,
) -> Option<engine_api::CardId> {
    use crate::state::ZoneKind;
    let Some(eq) = state.zones.find(ZoneKind::Equip, player) else {
        return None;
    };
    let Some(z) = state.zones.get(eq) else {
        return None;
    };
    z.cards.iter().copied().find(|cid| {
        state
            .cards
            .get(*cid)
            .map(|c| c.def.raw() == def)
            .unwrap_or(false)
    })
}
