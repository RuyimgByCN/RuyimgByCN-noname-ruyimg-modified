//! State-dependent card/skill/target queries on [`Catalog`].

use super::{CardId, Catalog, EquipSlot, EventKind, SkillDefId, Suit, TriggerMeta};

impl Catalog {
    /// L20 标记补充 (`xinhongyan`, `extension.js:1105-1135`): `mod.suit` (`Spade -> Heart` when `owner` holds `xinhongyan`).
    pub fn effective_card_suit(
        &self,
        core: &crate::state::GameStateCore,
        owner: crate::ids::PlayerId,
        card: CardId,
    ) -> Suit {
        let raw = self.cards.get(card as usize).map_or(Suit::Club, |c| c.suit);
        if raw == Suit::Spade
            && self
                .skill_by_name("xinhongyan")
                .is_some_and(|def| core.has_skill(owner, def))
        {
            Suit::Heart
        } else {
            raw
        }
    }

    /// L20 标记补充 (`fz_new_tieji` / `sbtieji`): whether `p`'s non-locked skills are sealed this turn.
    pub fn is_tieji_sealed(
        &self,
        core: &crate::state::GameStateCore,
        p: crate::ids::PlayerId,
    ) -> bool {
        core.vm_aux
            .flags
            .get(&(p, "fz_new_tieji_target_sealed".to_string()))
            .copied()
            .unwrap_or(0)
            > 0
            || core
                .vm_aux
                .flags
                .get(&(p, "sbtieji_target_sealed".to_string()))
                .copied()
                .unwrap_or(0)
                > 0
    }

    /// Whether a skill is a locked skill (`锁定技` / `forced: true` / `charlotte: true`) exempt from `tieji` seal.
    pub fn is_locked_skill_name(name: &str, eff_meta: Option<&TriggerMeta>) -> bool {
        if matches!(
            name,
            "zhenyi_spade"
                | "xinfu_langxi"
                | "fz_new_longdan"
                | "fz_new_tieji"
                | "fz_liegong"
                | "fz_xinkuanggu"
                | "rekuanshi"
                | "kuanshi"
                | "xinfu_lingren"
                | "sbtieji"
                | "sbduanliang"
                | "shouye"
                | "mbmowang"
                | "xianqu_mark"
                | "zhulianbihe_mark"
                | "yinyangyu_mark"
                | "_guozhan_marks"
                | "_chongzhu"
                | "taoluan"
                | "shouxi"
                | "dinghan"
                | "quanji"
                | "zili"
                | "paiyi"
                | "qimou"
                | "reqimou"
                | "zhaxiang"
                | "potzhuangshi"
                | "twxiafeng"
                | "twzuhuo"
        ) {
            return false;
        }
        if matches!(
            name,
            "qingyu"
                | "mibei"
                | "tspowei"
                | "rechuhai"
                | "xinhongyan"
                | "fz_gzpaoxiao"
                | "kuanshi2"
                | "zhengsu"
                | "aozhan"
                | "aozhan_shan"
                | "diaohulishan"
                | "txmanjia"
                | "txmanjia1"
                | "txmanjia2"
                | "txfusha"
                | "txxiebi"
                | "txshixin"
                | "twwushen"
                | "shizhi"
                | "paoxiao"
                | "mashu"
                | "wushuang"
                | "roulin"
                | "weimu"
                | "qianxun"
                | "hongyan"
                | "wushen"
                | "wansha"
                | "benghuai"
                | "kuangcai"
        ) {
            return true;
        }
        eff_meta.is_some_and(|m| !m.optional)
    }

    /// 牌的**实际**装备栏（swd 批 143 改造）：`declared_slots` 覆盖优先，
    /// 否则取牌种固有栏位。装备技能仍按牌种授予（牌种未变）。
    pub fn card_slot_of(
        &self,
        core: &crate::state::GameStateCore,
        card: CardId,
    ) -> Option<EquipSlot> {
        core.declared_slots
            .get(&card)
            .copied()
            .or_else(|| self.card_kind(card).equip_slot())
    }

    /// 某事件类别的候选触发定义（静态顺序，确定）。
    pub fn candidates(&self, kind: EventKind) -> &[SkillDefId] {
        self.dispatch.get(&kind).map(Vec::as_slice).unwrap_or(&[])
    }

    /// L20 标记补充 (`diaohulishan`, `extension.js:1432`): whether `p` is currently under `diaohulishan`.
    pub fn has_diaohulishan(
        &self,
        core: &crate::state::GameStateCore,
        p: crate::ids::PlayerId,
    ) -> bool {
        if core
            .vm_aux
            .flags
            .get(&(p, "diaohulishan_active".to_string()))
            .copied()
            .unwrap_or(0)
            > 0
            || core
                .vm_aux
                .flags
                .get(&(p, "mbmowang_resting".to_string()))
                .copied()
                .unwrap_or(0)
                > 0
        {
            return true;
        }
        self.skill_by_name("diaohulishan")
            .is_some_and(|def| core.has_skill(p, def))
    }

    /// 出牌封锁查询（swd 批 134 冰封）：玩家任一启用实例的行为声明封锁。
    pub fn blocks_all_cards(
        &self,
        core: &crate::state::GameStateCore,
        p: crate::ids::PlayerId,
    ) -> bool {
        if self.has_diaohulishan(core, p) {
            return true;
        }
        core.skills.iter().any(|((o, def), inst)| {
            *o == p
                && inst.enabled
                && self.skills[*def as usize]
                    .behavior
                    .blocks_all_cards_when(self, core, p)
        })
    }

    /// G1 目标过滤原语：`p` 是否持有「延时锦囊目标免疫」类技能（墨守：乐/兵不能指定其为目标）。
    pub fn blocks_delayed_trick(
        &self,
        core: &crate::state::GameStateCore,
        p: crate::ids::PlayerId,
    ) -> bool {
        core.skills
            .keys()
            .any(|(o, _)| *o == p)
            .then(|| {
                core.skills.iter().any(|((o, def), inst)| {
                    *o == p
                        && inst.enabled
                        && self.skills[*def as usize]
                            .behavior
                            .blocks_delayed_trick_target()
                })
            })
            .unwrap_or(false)
    }
}
