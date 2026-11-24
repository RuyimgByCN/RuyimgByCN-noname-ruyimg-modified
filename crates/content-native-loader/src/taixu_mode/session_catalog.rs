//! L11 太虚幻境目录过滤与查表辅助（`session_catalog.rs`）。
//!
//! 对齐上游 `extension_framework.js:5339, 6800, 6818, 6870`：
//! 按当前已加载扩展包构建的 `Catalog` 动态筛选 `skillRank` 与 `cardRank`，
//! 并在商店库存、`apply_effect_op` 随机卡/装备池、以及「幻境操戈」初始装备抽选中生效。

use engine_native::catalog::Catalog;
use engine_native::content::extension::ExtensionPack;
use engine_native::{taixu_card_fallback_candidates, taixu_skill_alias};

use super::catalog::L11_TAIXU_CARD_RANK;
use super::exams_a::{TaixuExamQuestionSpec, L11_TAIXU_EXAMS_1_2};
use super::exams_b::L11_TAIXU_EXAMS_3_4;
use super::seasons::{
    TaixuBattleTarget, TaixuChapterEventRef, TaixuDlcEventSpec, TaixuSeasonSpec,
    L11_TAIXU_CHAPTER_EVENT_REFS_A, L11_TAIXU_CHAPTER_EVENT_REFS_B, L11_TAIXU_SEASONS,
};
use super::session::TaixuModeSession;
use super::session_battle::is_taixu_equip_resolvable;
use super::session_types::TaixuSlotNode;
use super::skill_rank::L11_TAIXU_SKILL_RANK;
use super::universal::{
    TaixuUniversalEventSpec, L11_TAIXU_CHANGE_PACK, L11_TAIXU_GAIN_PACK,
};
use crate::taixu::TaixuError;

fn is_skill_available_in_catalog(catalog: &Catalog, skill_id: &str) -> bool {
    catalog.skill_by_name(skill_id).is_some()
        || taixu_skill_alias(skill_id).is_some_and(|al| catalog.skill_by_name(al).is_some())
}

fn is_card_available_in_catalog(catalog: &Catalog, card_id: &str) -> bool {
    let lookup = card_id.strip_prefix("rewrite_").unwrap_or(card_id);
    catalog.cards.iter().any(|cd| cd.name == lookup)
        || taixu_card_fallback_candidates(lookup)
            .iter()
            .any(|&alt| catalog.cards.iter().any(|cd| cd.name == alt))
}

impl TaixuModeSession {
    /// 按当前激活扩展包构建原生 `Catalog`，动态筛选 `L11_TAIXU_SKILL_RANK` 与 `L11_TAIXU_CARD_RANK`
    /// （对齐 `extension_framework.js:5339, 6800, 6818, 6870` 对 `lib.skill` / `lib.card` 的门控）。
    pub fn bind_available_catalog(&mut self, packs: &[&ExtensionPack]) -> Result<(), TaixuError> {
        let catalog = engine_native::content::standard::standard_catalog_with_extensions(packs)
            .map_err(|e| TaixuError::UnknownCommand(format!("build taixu catalog: {e:?}")))?;

        let mut avail_skills = Vec::new();
        for s in &L11_TAIXU_SKILL_RANK {
            if is_skill_available_in_catalog(&catalog, s.skill_id)
                && !avail_skills.iter().any(|x| x == s.skill_id)
            {
                avail_skills.push(s.skill_id.to_string());
            }
        }
        self.available_skills_filter = Some(avail_skills);

        let mut avail_cards = Vec::new();
        for c in &L11_TAIXU_CARD_RANK {
            if is_card_available_in_catalog(&catalog, c.card_id)
                && !avail_cards.iter().any(|x| x == c.card_id)
            {
                avail_cards.push(c.card_id.to_string());
            }
        }
        self.available_cards_filter = Some(avail_cards);

        if self.effect_id == "huanjingcaoge" {
            let needs_reapply = [&self.equip1, &self.equip2, &self.equip3, &self.equip4]
                .into_iter()
                .flatten()
                .any(|eq| !self.is_card_available_in_pool(eq));
            if needs_reapply {
                let mut removed = Vec::new();
                if self.equip1.as_deref().is_some_and(|e| !self.is_card_available_in_pool(e)) {
                    removed.push(self.equip1.take());
                }
                if self.equip2.as_deref().is_some_and(|e| !self.is_card_available_in_pool(e)) {
                    removed.push(self.equip2.take());
                }
                if self.equip3.as_deref().is_some_and(|e| !self.is_card_available_in_pool(e)) {
                    removed.push(self.equip3.take());
                }
                if self.equip4.as_deref().is_some_and(|e| !self.is_card_available_in_pool(e)) {
                    removed.push(self.equip4.take());
                }
                for eq in removed.into_iter().flatten() {
                    self.sync_equip_buffs(Some(&eq));
                }
                self.apply_huanjingcaoge_equip();
            }
        }
        Ok(())
    }

    /// 以指定扩展包目录过滤直接启动赛季战役（确保「幻境操戈」等开局效果亦受 `available_cards_filter` 约束）。
    pub fn start_season_with_catalog(
        seed: u64,
        season_id: &str,
        rank: u8,
        player_general: &str,
        effect_id: &str,
        servant_id: Option<&str>,
        servant_grade: u8,
        packs: &[&ExtensionPack],
    ) -> Result<Self, TaixuError> {
        let mut sess = Self::start_season(
            seed,
            season_id,
            rank,
            player_general,
            effect_id,
            servant_id,
            servant_grade,
        )?;
        sess.bind_available_catalog(packs)?;
        Ok(sess)
    }

    pub(super) fn is_card_available_in_pool(&self, card_id: &str) -> bool {
        self.available_cards_filter
            .as_ref()
            .is_none_or(|flt| flt.iter().any(|x| x == card_id))
    }

    pub(super) fn is_skill_available_in_pool(&self, skill_id: &str) -> bool {
        self.available_skills_filter
            .as_ref()
            .is_none_or(|flt| flt.iter().any(|x| x == skill_id))
    }

    /// 效果「幻境操戈」随机初始装备筛选与穿戴（受 `rank`、`available_cards_filter` 与可解析性约束）。
    pub(super) fn apply_huanjingcaoge_equip(&mut self) {
        let equips: Vec<&'static str> = L11_TAIXU_CARD_RANK
            .iter()
            .filter(|c| {
                c.category.is_equip()
                    && c.rank <= self.rank
                    && is_taixu_equip_resolvable(c.card_id)
                    && self.is_card_available_in_pool(c.card_id)
            })
            .map(|c| c.card_id)
            .collect();
        if equips.is_empty() {
            return;
        }
        let pick = self.rng.gen_range(equips.len());
        let card_id = equips[pick];
        let slot = L11_TAIXU_CARD_RANK
            .iter()
            .find(|c| c.card_id == card_id)
            .and_then(|c| c.category.equip_slot_name());
        match slot {
            Some("equip1") if self.equip1.is_none() => self.equip1 = Some(card_id.to_string()),
            Some("equip2") if self.equip2.is_none() => self.equip2 = Some(card_id.to_string()),
            Some("equip3") if self.equip3.is_none() => self.equip3 = Some(card_id.to_string()),
            Some("equip4") if self.equip4.is_none() => self.equip4 = Some(card_id.to_string()),
            _ => self.equips.push(card_id.to_string()),
        }
        self.sync_equip_buffs(None);
    }
}

pub(super) fn target_to_str(t: TaixuBattleTarget) -> &'static str {
    match t {
        TaixuBattleTarget::All => "all",
        TaixuBattleTarget::Kill => "kill",
        TaixuBattleTarget::Repair => "repair",
        TaixuBattleTarget::Subsist => "subsist",
    }
}

pub(super) fn find_season(id: &str) -> Option<&'static TaixuSeasonSpec> {
    L11_TAIXU_SEASONS
        .iter()
        .find(|s| s.id.eq_ignore_ascii_case(id))
}

pub fn find_exam_by_id(id: &str) -> Option<&'static TaixuExamQuestionSpec> {
    L11_TAIXU_EXAMS_1_2
        .iter()
        .chain(L11_TAIXU_EXAMS_3_4.iter())
        .find(|q| q.id == id)
}

pub fn find_universal_event(id: &str) -> Option<&'static TaixuUniversalEventSpec> {
    L11_TAIXU_GAIN_PACK
        .iter()
        .chain(L11_TAIXU_CHANGE_PACK.iter())
        .find(|e| e.id == id)
}

pub(super) fn iter_chapter_refs<'a>(
    season: &'a str,
    chapter: &'a str,
) -> impl Iterator<Item = &'static TaixuChapterEventRef> + 'a {
    L11_TAIXU_CHAPTER_EVENT_REFS_A
        .iter()
        .chain(L11_TAIXU_CHAPTER_EVENT_REFS_B.iter())
        .filter(move |r| r.season == season && r.chapter == chapter)
}

pub(super) fn slot_from_dlc(
    seat: u16,
    season: &str,
    chapter: &str,
    def_idx: usize,
    def: &'static TaixuDlcEventSpec,
) -> TaixuSlotNode {
    TaixuSlotNode {
        slot_seat: seat,
        season: season.to_string(),
        chapter: chapter.to_string(),
        id: def.id.to_string(),
        kind: def.kind.to_string(),
        level: def.level,
        change: def.change,
        dlc_def_idx: Some(def_idx),
        exam_tier: None,
        override_buttons: None,
    }
}

pub(super) fn slot_from_exam(
    seat: u16,
    season: &str,
    chapter: &str,
    q: &'static TaixuExamQuestionSpec,
) -> TaixuSlotNode {
    TaixuSlotNode {
        slot_seat: seat,
        season: season.to_string(),
        chapter: chapter.to_string(),
        id: q.id.to_string(),
        kind: q.tier.to_string(),
        level: 1,
        change: true,
        dlc_def_idx: None,
        exam_tier: Some(q.tier.to_string()),
        override_buttons: None,
    }
}

pub(super) fn slot_from_universal(
    seat: u16,
    season: &str,
    chapter: &str,
    u: &'static TaixuUniversalEventSpec,
) -> TaixuSlotNode {
    TaixuSlotNode {
        slot_seat: seat,
        season: season.to_string(),
        chapter: chapter.to_string(),
        id: u.id.to_string(),
        kind: u.kind.to_string(),
        level: u.level,
        change: u.change,
        dlc_def_idx: None,
        exam_tier: None,
        override_buttons: None,
    }
}
