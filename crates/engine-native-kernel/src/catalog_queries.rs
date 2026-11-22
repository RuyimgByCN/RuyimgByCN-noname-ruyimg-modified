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

    /// L10 `rechanyuan` 缠怨状态查询（`refresh.js:10760-10783`）。
    pub fn has_rechanyuan(
        &self,
        core: &crate::state::GameStateCore,
        p: crate::ids::PlayerId,
    ) -> bool {
        core.vm_aux
            .flags
            .get(&(p, "l10_rechanyuan".to_string()))
            .copied()
            .unwrap_or(0)
            > 0
            || self
                .skill_by_name("rechanyuan")
                .is_some_and(|def| core.has_skill(p, def))
    }

    /// L10 `rechanyuan` 封技门控（`refresh.js:10770`）：`hp <= 1` 时除
    /// `chanyuan`/`rechanyuan` 与 `charlotte` 外所有技能（含锁定技）均失效。
    pub fn is_rechanyuan_sealed_def(
        &self,
        core: &crate::state::GameStateCore,
        p: crate::ids::PlayerId,
        def: SkillDefId,
    ) -> bool {
        if core.player(p).hp > 1 || !self.has_rechanyuan(core, p) {
            return false;
        }
        let Some(skill) = self.skills.get(def as usize) else {
            return false;
        };
        !matches!(skill.name, "chanyuan" | "rechanyuan") && !self.is_charlotte_skill(def)
    }

    /// 按技能名查询是否被 `rechanyuan`（或 `tieji` 非锁定技）封印。
    pub fn is_rechanyuan_sealed_name(
        &self,
        core: &crate::state::GameStateCore,
        p: crate::ids::PlayerId,
        name: &str,
    ) -> bool {
        if core.player(p).hp > 1 || !self.has_rechanyuan(core, p) {
            return false;
        }
        if matches!(name, "chanyuan" | "rechanyuan") {
            return false;
        }
        if let Some(def) = self.skill_by_name(name) {
            !self.is_charlotte_skill(def)
        } else {
            !name.starts_with('_')
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
            || self
                .skill_by_name("txhj_feilong_skill_blank")
                .is_some_and(|def| core.has_skill(p, def))
            || self
                .skill_by_name("txtanbei_effect1")
                .is_some_and(|def| core.has_skill(p, def))
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
                | "kuanshi2"
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
                | "buff_txhj_gusoejueyan"
                | "_buff_txhj_guosejueyan"
                | "_buff_txhj_wuweizhile"
                | "_buff_txhj_jiezizhidao"
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
                | "txmazhan"
                | "txhuangfu"
                | "txleizhen"
                | "txlianbi"
                | "txbaobian"
                | "diychunxiao"
                | "diyzhongdu"
                | "cxyYangWu"
                | "cxyJingQi"
                | "cxyRuiQi"
                | "cxyJiaoXia"
                | "cxyTunJun"
                | "cxyFengYing"
                | "cxyPoLu"
                | "txhj_ruiSkill1"
                | "txhj_dundunSkill2"
                | "txhj_jiuweiSkill1"
                | "txhj_tengsheSkill1"
                | "txhj_tengsheSkill3"
                | "txhj_yayaSkill2"
                | "txhj_qilinSkill3"
                | "txhj_chiyanzhenhunqin"
                | "txhj_lingbaoxianhu"
                | "txhj_chongyingshenfu"
                | "txminguangkai"
                | "txminguangkai_cancel"
                | "txminguangkai_link"
                | "txhj_feilong_skill_blank"
                | "txhj_guofengyupao"
                | "txhj_xuwangzhimian"
                | "txhj_aleSkill1"
                | "txhj_aleSkill2"
                | "txhj_dundunSkill1"
                | "txhj_datongSkill1"
                | "txhj_luluSkill1"
                | "txhj_luluSkill2"
                | "txhj_ruiSkill2"
                | "txhj_ruiSkill3"
                | "txhj_yueerSkill1"
                | "txhj_yueerSkill2"
                | "txhj_yanSkill1"
                | "txhj_yanSkill2"
                | "txhj_yanSkill3"
                | "txhj_liuliSkill1"
                | "txhj_liuliSkill2"
                | "txhj_manmanSkill1"
                | "txhj_manmanSkill2"
                | "txhj_xiaoxiaoSkill1"
                | "txhj_xiaoxiaoSkill2"
                | "txhj_xuerenSkill1"
                | "txhj_xuanwuSkill1"
                | "txhj_xuanwuSkill2"
                | "txhj_xuanwuSkill3"
                | "txhj_aHaoSkill2"
                | "txhj_aHaoSkill3"
                | "oldfuman2"
                | "txhuao"
                | "hgkurou_effect"
                | "txtanbei_effect1"
                | "txtanbei_effect2"
                | "txtanbei_effect3"
                | "ljxuechi"
                | "txzhuxin"
                | "txnianrui"
                | "txmengtai"
                | "txshouyi"
                | "txrenxing"
                | "txyinli"
                | "txzhulian"
                | "txkeji"
                | "txkeji_add"
                | "mdqianxi72"
                | "zglkongcheng"
                | "txjuxiang"
                | "txhj_huangmen"
                | "txhj_suzhi"
                | "txmanzhen"
                | "txleizhou"
                | "txyongguan"
                | "txyongguan_fate"
                | "txbenji"
                | "txhj_oldhuxiao"
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
                | "txhj_kapaikeshi"
                | "txhj_wenhejibei"
                | "txhj_yueyingdezhihui"
                | "txhj_changbingguangren"
                | "txhj_xiongziyingfa"
                | "txhj_wuxiangdexinxue"
                | "txhj_wuweizhile"
                | "txhj_yuanduandilue"
        ) {
            return true;
        }
        eff_meta.is_some_and(|m| !m.optional)
    }

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
                self.card_kind(card).equip_slot()
            }
        })
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
        if self.has_diaohulishan(core, p)
            || core
                .vm_aux
                .flags
                .get(&(p, "chixueqingfeng2_count".to_string()))
                .copied()
                .unwrap_or(0)
                > 0
            || core
                .vm_aux
                .flags
                .get(&(p, "l11_noda_axe2_turn".to_string()))
                .copied()
                .unwrap_or(0)
                > 0
            || self
                .skill_by_name("txhj_noda_axe2")
                .is_some_and(|def| core.has_skill(p, def))
        {
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

    /// Whether a skill has `charlotte: true` (global `_` / runtime helper / trait):
    /// excluded from `xuelunyang` theft and `mbcmfangzhu` skill-disable.
    pub fn is_charlotte_skill(&self, def: crate::ids::SkillDefId) -> bool {
        let Some(skill) = self.skills.get(def as usize) else {
            return false;
        };
        skill.behavior.is_charlotte()
            || skill.name.starts_with('_')
            || skill.name.contains("#r05:")
            || matches!(
                skill.name,
                crate::vocabulary::names::WANG_ZUN_MARK | "wuxing_shengke" | "diaohulishan"
            )
    }

    /// Whether a skill has `persevereSkill / forever: true` (`持恒技`):
    /// exempt from skill-disable effects such as `mbcmfangzhu`.
    pub fn is_forever_skill(&self, def: crate::ids::SkillDefId) -> bool {
        self.skills
            .get(def as usize)
            .is_some_and(|skill| skill.behavior.is_forever())
    }
}
