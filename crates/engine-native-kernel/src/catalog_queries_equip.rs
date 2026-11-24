//! Equipment-slot and Taixu `equip5` resolution queries on [`Catalog`].

use super::{CardId, Catalog, EquipSlot};

impl Catalog {
    /// Returns whether `card` is in the 11-card upstream `_txhj_equip5` pool
    /// (`extension_skill.js:299`: `taipingyaoshu`, 8 standard horses, `txhj_liulongcanjia`, `txhj_juechenjinge`).
    pub fn is_txhj_equip5_card(&self, card: CardId) -> bool {
        let Some(cd) = self.cards.get(card as usize) else {
            return false;
        };
        matches!(
            cd.name,
            "taipingyaoshu"
                | "th_taipingyaoshu"
                | "chitu"
                | "zixin"
                | "dawan"
                | "jingfanma"
                | "dilu"
                | "zhuahuang"
                | "hualiu"
                | "jueying"
                | "horse_plus"
                | "horse_minus"
                | "txhj_liulongcanjia"
                | "txhj_juechenjinge"
        ) || cd.original_source.is_some_and(|os| {
            matches!(
                os.raw_symbol,
                "taipingyaoshu"
                    | "chitu"
                    | "zixin"
                    | "dawan"
                    | "jingfanma"
                    | "dilu"
                    | "zhuahuang"
                    | "hualiu"
                    | "jueying"
            )
        })
    }

    /// Returns whether `_txhj_equip5` is currently active on any seat in `core`.
    pub fn txhj_equip5_active(&self, core: &crate::state::GameStateCore) -> bool {
        self.skill_by_name("_txhj_equip5").is_some_and(|def| {
            (0..core.players.len() as crate::ids::PlayerId).any(|p| core.has_skill(p, def))
        })
    }

    /// 牌的**实际**装备栏（swd 批 143 改造）：`declared_slots` 覆盖优先，
    /// 否则在 `_txhj_equip5` 启用时将 11 张太虚宝物转换牌映射为宝物栏，再否则取牌种固有栏位。
    pub fn card_slot_of(
        &self,
        core: &crate::state::GameStateCore,
        card: CardId,
    ) -> Option<EquipSlot> {
        core.declared_slots.get(&card).copied().or_else(|| {
            if self.txhj_equip5_active(core) && self.is_txhj_equip5_card(card) {
                Some(EquipSlot::Treasure)
            } else {
                self.resolved_card_kind(core, card).equip_slot()
            }
        })
    }
}
