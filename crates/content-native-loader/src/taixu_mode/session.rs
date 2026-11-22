//! L11 太虚幻境完整战役模式运行时（`extension_framework.js` 状态机、商店、题库、奇遇、实局桥接与档案）。

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::catalog::{taixu_max_exp_for_level, L11_TAIXU_CARD_RANK, L11_TAIXU_EFFECT_PACK};
use super::exams_a::{TaixuExamQuestionSpec, L11_TAIXU_EXAMS_1_2};
use super::exams_b::L11_TAIXU_EXAMS_3_4;
use super::seasons::{
    get_dlc_event_by_idx, TaixuBattleTarget, TaixuDlcEventSpec, TaixuSeasonSpec,
    L11_TAIXU_CHAPTER_EVENT_REFS_A, L11_TAIXU_CHAPTER_EVENT_REFS_B, L11_TAIXU_SEASONS,
};
use super::session_types::*;
use super::universal::{
    TaixuBtnResult, TaixuUniversalEventSpec, L11_TAIXU_CHANGE_PACK, L11_TAIXU_GAIN_PACK,
};
use crate::taixu::TaixuError;

/// 马钧的改造装备升级映射（`extension_buff.js` + `extension_skill.js`）。
pub fn taixu_majun_upgrade_name(equip_name: &str) -> &str {
    match equip_name {
        "zhuge" => "rewrite_zhuge",
        "lanyinjia" => "rewrite_lanyinjia",
        "tengjia" => "rewrite_tengjia",
        "baiyin" => "rewrite_baiyin",
        "bagua" => "rewrite_bagua",
        "renwang" => "rewrite_renwang",
        other => other,
    }
}

/// 完整太虚幻境战役模式会话（映射 `lib.config.taixuhuanjing`、`txhj_collect` 与待决状态）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TaixuModeSession {
    pub rng: TaixuSplitMix64,
    pub season: String,
    pub chapter_idx: usize,
    pub rank: u8,
    pub player_general: String,
    pub effect_id: String,
    pub servant_id: Option<String>,
    pub servant_grade: u8,
    pub level: u8,
    pub exp: u32,
    pub max_exp: u32,
    pub coin: i32,
    pub max_coin: i32,
    pub hp: i32,
    pub max_hp: i32,
    pub min_hs: i32,
    pub max_hs: i32,
    pub adjust: i32,
    pub max_skills: usize,
    pub buffs: Vec<String>,
    pub skills: Vec<String>,
    pub use_skills: Vec<String>,
    pub cards: Vec<String>,
    pub equips: Vec<String>,
    pub equip1: Option<String>,
    pub equip2: Option<String>,
    pub equip3: Option<String>,
    pub equip4: Option<String>,
    pub procedure: Vec<TaixuSlotNode>,
    pub optional: [Option<TaixuSlotNode>; 3],
    pub seen_events: Vec<String>,
    pub seen_exams: Vec<String>,
    pub forbidden_exams: Vec<String>,
    pub bowen_marked_opt: Option<String>,
    pub stores: BTreeMap<String, TaixuShopStock>,
    pub collect_cards: BTreeMap<String, u32>,
    pub collect_chars: BTreeMap<String, u32>,
    pub score_fight: u32,
    pub score_round: u32,
    pub next_slot_seat: u16,
    pub pending: Option<TaixuPendingInteraction>,
    pub completed: bool,
    pub failed: bool,
    pub final_score: Option<TaixuRunScore>,
    pub available_skills_filter: Option<Vec<String>>,
}

impl TaixuModeSession {
    /// 启动一个完整的太虚幻境赛季战役（`updateModeData` + 章节 `procedure` 初始化）。
    pub fn start_season(
        seed: u64,
        season_id: &str,
        rank: u8,
        player_general: &str,
        effect_id: &str,
        servant_id: Option<&str>,
        servant_grade: u8,
    ) -> Result<Self, TaixuError> {
        let season_spec = find_season(season_id).ok_or_else(|| TaixuError::InvalidField {
            field: "season",
            reason: format!("未知太虚赛季: {season_id}"),
        })?;
        let base_hp = crate::taixu_generals_data::L11_TAIXU_GENERALS
            .iter()
            .find(|g| g.0 == player_general)
            .map(|g| g.3 as i32)
            .unwrap_or(4);
        let mut s = Self {
            rng: TaixuSplitMix64::from_seed(seed),
            season: season_spec.id.to_string(),
            chapter_idx: 0,
            rank: rank.clamp(1, 5),
            player_general: player_general.to_string(),
            effect_id: if effect_id.is_empty() {
                "pinganwushi".to_string()
            } else {
                effect_id.to_string()
            },
            servant_id: servant_id.map(str::to_string),
            servant_grade: servant_grade.min(5),
            level: 1,
            exp: 0,
            max_exp: taixu_max_exp_for_level(1),
            coin: 0,
            max_coin: 0,
            hp: base_hp,
            max_hp: base_hp,
            min_hs: 4,
            max_hs: 0,
            adjust: 0,
            max_skills: 5,
            buffs: Vec::new(),
            skills: Vec::new(),
            use_skills: Vec::new(),
            cards: Vec::new(),
            equips: Vec::new(),
            equip1: None,
            equip2: None,
            equip3: None,
            equip4: None,
            procedure: Vec::new(),
            optional: [None, None, None],
            seen_events: Vec::new(),
            seen_exams: Vec::new(),
            forbidden_exams: Vec::new(),
            bowen_marked_opt: None,
            stores: BTreeMap::new(),
            collect_cards: BTreeMap::new(),
            collect_chars: BTreeMap::new(),
            score_fight: 0,
            score_round: 0,
            next_slot_seat: 1,
            pending: None,
            completed: false,
            failed: false,
            final_score: None,
            available_skills_filter: None,
        };
        if let Some(eff_spec) = L11_TAIXU_EFFECT_PACK.iter().find(|e| e.id == s.effect_id) {
            for &sk in eff_spec.global_skills {
                if s.use_skills.len() < s.max_skills {
                    if !s.use_skills.iter().any(|x| x == sk) {
                        s.use_skills.push(sk.to_string());
                    }
                } else if !s.skills.iter().any(|x| x == sk) && !s.use_skills.iter().any(|x| x == sk)
                {
                    s.skills.push(sk.to_string());
                }
            }
        }
        match s.effect_id.as_str() {
            "huanjingcaoge" => {
                let equips: Vec<&'static str> = L11_TAIXU_CARD_RANK
                    .iter()
                    .filter(|c| {
                        c.category.is_equip()
                            && c.rank <= s.rank
                            && super::session_battle::is_taixu_equip_resolvable(c.card_id)
                    })
                    .map(|c| c.card_id)
                    .collect();
                if !equips.is_empty() {
                    let pick = s.rng.gen_range(equips.len());
                    let card_id = equips[pick];
                    let slot = L11_TAIXU_CARD_RANK
                        .iter()
                        .find(|c| c.card_id == card_id)
                        .and_then(|c| c.category.equip_slot_name());
                    match slot {
                        Some("equip1") if s.equip1.is_none() => s.equip1 = Some(card_id.to_string()),
                        Some("equip2") if s.equip2.is_none() => s.equip2 = Some(card_id.to_string()),
                        Some("equip3") if s.equip3.is_none() => s.equip3 = Some(card_id.to_string()),
                        Some("equip4") if s.equip4.is_none() => s.equip4 = Some(card_id.to_string()),
                        _ => s.equips.push(card_id.to_string()),
                    }
                    s.sync_equip_buffs(None);
                }
            }
            "lingqiyiman" => {
                s.max_hp += 2;
                s.hp += 2;
            }
            "lingqikuijie" => {
                s.max_hp -= 1;
                s.hp -= 1;
            }
            "chongfenbeizhan" => {
                s.min_hs += 2;
            }
            "cangcuchuji" => {
                s.min_hs -= 3;
            }
            _ => {}
        }
        if s.rank > 4 && !s.buffs.iter().any(|b| b == "buff_txhj_aozhan") {
            s.buffs.push("buff_txhj_aozhan".to_string());
        }
        if s.rank > 2 {
            s.hp -= 1;
            s.max_hp -= 1;
        }
        s.enter_chapter(0)?;
        Ok(s)
    }

    pub fn serialize_snapshot(&self) -> String {
        serde_json::to_string(self).expect("TaixuModeSession serializes cleanly")
    }

    pub fn deserialize_snapshot(raw: &str) -> Result<Self, TaixuError> {
        serde_json::from_str(raw)
            .map_err(|e| TaixuError::SnapshotMalformed(format!("session decode failed: {e}")))
    }

    pub fn current_chapter(&self) -> Option<&'static super::seasons::TaixuChapterSpec> {
        find_season(&self.season).and_then(|s| s.chapters.get(self.chapter_idx))
    }

    fn alloc_seat(&mut self) -> u16 {
        let s = self.next_slot_seat;
        self.next_slot_seat = self.next_slot_seat.wrapping_add(1).max(1);
        s
    }

    pub fn enter_chapter(&mut self, chap_idx: usize) -> Result<(), TaixuError> {
        let season = find_season(&self.season).ok_or_else(|| TaixuError::InvalidField {
            field: "season",
            reason: format!("未知太虚赛季: {}", self.season),
        })?;
        let Some(chap) = season.chapters.get(chap_idx) else {
            self.completed = true;
            self.final_score = Some(self.compute_score(true));
            return Ok(());
        };
        self.chapter_idx = chap_idx;
        self.stores.clear();
        self.procedure.clear();
        self.optional = [None, None, None];
        let chap_id = chap.id.to_string();
        for &proc_kind in chap.procedure {
            let slot = match proc_kind {
                "side" | "main" | "micheng" | "boss" | "Ultimate" => {
                    self.pick_random_battle(&chap_id, proc_kind)
                }
                _ => self.pick_random_event(&chap_id, proc_kind),
            };
            if let Some(node) = slot {
                self.procedure.push(node);
            }
        }
        self.refill_optional_slots();
        Ok(())
    }

    fn refill_optional_slots(&mut self) {
        for slot in &mut self.optional {
            if slot.is_none() && !self.procedure.is_empty() {
                let next = self.procedure.remove(0);
                if next.kind.starts_with("exam") && !self.forbidden_exams.contains(&next.id) {
                    self.forbidden_exams.push(next.id.clone());
                    if self.forbidden_exams.len() >= 50 {
                        self.forbidden_exams.remove(0);
                    }
                }
                *slot = Some(next);
            }
        }
    }

    fn pick_random_battle(&mut self, chap_id: &str, proc_kind: &str) -> Option<TaixuSlotNode> {
        let refs: Vec<_> = iter_chapter_refs(&self.season, chap_id).collect();
        let mut pool = Vec::new();
        for r in refs {
            let Some(def) = (r.def_idx >= 0).then(|| get_dlc_event_by_idx(r.def_idx as usize)).flatten() else {
                continue;
            };
            if def.rank > self.rank || def.kind != proc_kind {
                continue;
            }
            if self.procedure.iter().any(|p| p.id == def.id && p.chapter == chap_id) {
                continue;
            }
            if def.forced {
                let seat = self.alloc_seat();
                return Some(slot_from_dlc(seat, &self.season, chap_id, r.def_idx as usize, def));
            }
            pool.push((r.def_idx as usize, def));
        }
        if pool.is_empty() {
            return None;
        }
        for _ in 0..32 {
            let pick = self.rng.gen_range(pool.len());
            let (def_idx, def) = pool[pick];
            if self.rng.chance_thousandths(def.random_thousandths) {
                let seat = self.alloc_seat();
                return Some(slot_from_dlc(seat, &self.season, chap_id, def_idx, def));
            }
        }
        let (def_idx, def) = pool[0];
        let seat = self.alloc_seat();
        Some(slot_from_dlc(seat, &self.season, chap_id, def_idx, def))
    }

    fn pick_random_event(&mut self, chap_id: &str, raw_kind: &str) -> Option<TaixuSlotNode> {
        let eff_kind = if raw_kind == "random" {
            if self.rng.gen_range(10) == 0 {
                "exam"
            } else {
                "gain"
            }
        } else {
            raw_kind
        };
        let refs: Vec<_> = iter_chapter_refs(&self.season, chap_id).collect();
        let mut pool1: Vec<TaixuSlotNode> = Vec::new();
        for r in refs {
            if r.kind != eff_kind {
                continue;
            }
            if r.def_idx >= 0 {
                if let Some(def) = get_dlc_event_by_idx(r.def_idx as usize) {
                    if def.rank > self.rank {
                        continue;
                    }
                    if self.procedure.iter().any(|p| p.id == def.id && p.chapter == chap_id) {
                        continue;
                    }
                    let seat = self.alloc_seat();
                    let node = slot_from_dlc(seat, &self.season, chap_id, r.def_idx as usize, def);
                    if def.forced {
                        return Some(node);
                    }
                    pool1.push(node);
                }
            } else if !r.exam_id.is_empty() {
                if let Some(eq) = find_exam_by_id(r.exam_id) {
                    if eq.rank > self.rank
                        || self.seen_exams.contains(&eq.id.to_string())
                        || self.forbidden_exams.contains(&eq.id.to_string())
                    {
                        continue;
                    }
                    let seat = self.alloc_seat();
                    pool1.push(slot_from_exam(seat, &self.season, chap_id, eq));
                }
            }
        }

        if pool1.len() < 5 {
            let mut pool2: Vec<TaixuSlotNode> = Vec::new();
            match eff_kind {
                "gain" | "trade" => {
                    const PRIO: [u8; 10] = [4, 4, 4, 4, 3, 3, 3, 2, 2, 1];
                    let prio = PRIO[self.rng.gen_range(PRIO.len())];
                    for g in &L11_TAIXU_GAIN_PACK {
                        if g.rank <= self.rank && g.priority == prio {
                            let seat = self.alloc_seat();
                            pool2.push(slot_from_universal(seat, &self.season, chap_id, g));
                        }
                    }
                    if pool2.is_empty() {
                        for g in &L11_TAIXU_GAIN_PACK {
                            if g.rank <= self.rank {
                                let seat = self.alloc_seat();
                                pool2.push(slot_from_universal(seat, &self.season, chap_id, g));
                            }
                        }
                    }
                }
                "exam" | "exam1" | "exam2" | "exam3" | "exam4" => {
                    for eq in L11_TAIXU_EXAMS_1_2.iter().chain(L11_TAIXU_EXAMS_3_4.iter()) {
                        if (eff_kind == "exam" || eq.tier == eff_kind)
                            && eq.rank <= self.rank
                            && !self.seen_exams.contains(&eq.id.to_string())
                            && !self.forbidden_exams.contains(&eq.id.to_string())
                        {
                            let seat = self.alloc_seat();
                            pool2.push(slot_from_exam(seat, &self.season, chap_id, eq));
                        }
                    }
                }
                "store" => {
                    for c in &L11_TAIXU_CHANGE_PACK {
                        if c.kind == "store" && c.rank <= self.rank {
                            let seat = self.alloc_seat();
                            pool2.push(slot_from_universal(seat, &self.season, chap_id, c));
                        }
                    }
                }
                _ => {}
            }
            while pool1.len() < 5 && !pool2.is_empty() {
                let idx = self.rng.gen_range(pool2.len());
                pool1.push(pool2.swap_remove(idx));
            }
        }
        if pool1.is_empty() {
            return None;
        }
        let mut chosen = pool1.swap_remove(self.rng.gen_range(pool1.len()));
        if chosen.kind == "gain"
            && !chosen.change
            && L11_TAIXU_CHANGE_PACK.iter().any(|c| c.id == chosen.id)
            && self.rng.chance_thousandths(30)
        {
            chosen.change = true;
        }
        Some(chosen)
    }

    /// 打开地图槽位 `0..3`，返回对应的真实交互或战斗前置挂起。
    pub fn open_slot(&mut self, slot_idx: usize) -> Result<TaixuPendingInteraction, TaixuError> {
        if self.completed || self.failed {
            return Err(TaixuError::AlreadyCompleted);
        }
        if let Some(ref pend) = self.pending {
            return Ok(pend.clone());
        }
        let node = self
            .optional
            .get(slot_idx)
            .and_then(|o| o.clone())
            .ok_or_else(|| TaixuError::UnknownCommand(format!("empty optional slot {slot_idx}")))?;

        let interaction = match node.kind.as_str() {
            "exam" | "exam1" | "exam2" | "exam3" | "exam4" => {
                let q = find_exam_by_id(&node.id)
                    .ok_or_else(|| TaixuError::UnknownCommand(format!("missing exam {}", node.id)))?;
                let mut idxs: Vec<usize> = (0..q.buttons.len()).collect();
                // Fisher-Yates 洗牌，映射真实 `lookAnswer` 随机按钮排布
                for i in (1..idxs.len()).rev() {
                    let j = self.rng.gen_range(i + 1);
                    idxs.swap(i, j);
                }
                let has_bowen = self.buffs.iter().any(|b| b == "buff_txhj_bowenqiangzhi");
                let mut bowen_target = self.bowen_marked_opt.clone();
                let mut buttons = Vec::new();
                for &orig in &idxs {
                    let btn = &q.buttons[orig];
                    let res = btn.result == TaixuBtnResult::True;
                    let mut marked = false;
                    if !res && has_bowen {
                        if bowen_target.as_deref() == Some(btn.name) {
                            marked = true;
                        } else if bowen_target.is_none() {
                            bowen_target = Some(btn.name.to_string());
                            self.bowen_marked_opt = bowen_target.clone();
                            marked = true;
                        }
                    }
                    buttons.push(TaixuButtonOptionView {
                        orig_idx: Some(orig),
                        name: btn.name.to_string(),
                        result: Some(res),
                        bowen_marked_wrong: marked,
                    });
                }
                if buttons.len() == 1 || buttons.len() == 3 {
                    buttons.push(TaixuButtonOptionView {
                        orig_idx: None,
                        name: "思考一下".to_string(),
                        result: None,
                        bowen_marked_wrong: false,
                    });
                }
                TaixuPendingInteraction::Quiz {
                    slot_idx,
                    tier: q.tier.to_string(),
                    exam_id: q.id.to_string(),
                    buttons,
                }
            }
            "gain" | "trade" => {
                let is_trade = node.kind == "trade";
                let mut buttons = Vec::new();
                if let Some(ref ovs) = node.override_buttons {
                    for (i, b) in ovs.iter().enumerate() {
                        buttons.push(TaixuButtonOptionView {
                            orig_idx: Some(i),
                            name: b.name.clone(),
                            result: b.result_true,
                            bowen_marked_wrong: false,
                        });
                    }
                } else {
                    let ev = find_universal_event(&node.id).ok_or_else(|| {
                        TaixuError::UnknownCommand(format!("missing universal event {}", node.id))
                    })?;
                    for (i, b) in ev.buttons.iter().enumerate() {
                        buttons.push(TaixuButtonOptionView {
                            orig_idx: Some(i),
                            name: b.name.to_string(),
                            result: match b.result {
                                TaixuBtnResult::True => Some(true),
                                TaixuBtnResult::False => Some(false),
                                TaixuBtnResult::ThinkLater => None,
                            },
                            bowen_marked_wrong: false,
                        });
                    }
                }
                if !is_trade && (buttons.len() == 1 || buttons.len() == 3) {
                    buttons.push(TaixuButtonOptionView {
                        orig_idx: None,
                        name: "思考一下".to_string(),
                        result: None,
                        bowen_marked_wrong: false,
                    });
                }
                TaixuPendingInteraction::GainOrTrade {
                    slot_idx,
                    event_id: node.id.clone(),
                    is_trade,
                    is_sub_trade: node.override_buttons.is_some(),
                    buttons,
                }
            }
            "choose" => {
                let def = node
                    .dlc_def_idx
                    .and_then(get_dlc_event_by_idx)
                    .ok_or_else(|| TaixuError::UnknownCommand(format!("missing choose {}", node.id)))?;
                let mut buttons: Vec<TaixuChooseOptionView> = def
                    .buttons
                    .iter()
                    .enumerate()
                    .map(|(i, b)| TaixuChooseOptionView {
                        idx: Some(i),
                        name: b.name.to_string(),
                        launches_battle: !b.enemies.is_empty(),
                        opens_sub_trade: b.enemies.is_empty() && !b.sub_buttons.is_empty(),
                    })
                    .collect();
                if buttons.len() == 1 || buttons.len() == 3 {
                    buttons.push(TaixuChooseOptionView {
                        idx: None,
                        name: "思考一下".to_string(),
                        launches_battle: false,
                        opens_sub_trade: false,
                    });
                }
                TaixuPendingInteraction::Choose {
                    slot_idx,
                    event_id: node.id.clone(),
                    buttons,
                }
            }
            "store" => {
                self.ensure_shop_stock(&node.id, node.level);
                TaixuPendingInteraction::Shop {
                    slot_idx,
                    shop_id: node.id.clone(),
                }
            }
            _ => {
                let def = node
                    .dlc_def_idx
                    .and_then(get_dlc_event_by_idx)
                    .ok_or_else(|| TaixuError::UnknownCommand(format!("missing battle {}", node.id)))?;
                TaixuPendingInteraction::Battle(TaixuPendingBattleSpec {
                    slot_idx,
                    event_id: def.id.to_string(),
                    event_kind: def.kind.to_string(),
                    season: node.season.clone(),
                    chapter: node.chapter.clone(),
                    dlc_def_idx: node.dlc_def_idx,
                    choose_btn_idx: None,
                    target_str: target_to_str(def.target).to_string(),
                    unique: def.unique,
                    max_rounds: def.max_rounds,
                    exp: def.exp,
                })
            }
        };
        self.pending = Some(interaction.clone());
        Ok(interaction)
    }

    /// 回答题库（返回 `(已作答消耗槽位, 是否答对)`；选 `'思考一下'` 返回 `(false, false)`）。
    pub fn answer_quiz(&mut self, displayed_idx: usize) -> Result<(bool, bool), TaixuError> {
        let Some(TaixuPendingInteraction::Quiz {
            slot_idx,
            exam_id,
            buttons,
            ..
        }) = self.pending.clone()
        else {
            return Err(TaixuError::UnknownCommand("no pending quiz".to_string()));
        };
        let opt = buttons
            .get(displayed_idx)
            .ok_or_else(|| TaixuError::UnknownCommand(format!("quiz idx {displayed_idx} oob")))?;
        let Some(orig) = opt.orig_idx else {
            self.pending = None;
            return Ok((false, false));
        };
        let q = find_exam_by_id(&exam_id)
            .ok_or_else(|| TaixuError::UnknownCommand(format!("missing exam {exam_id}")))?;
        let btn = &q.buttons[orig];
        let passed = btn.result == TaixuBtnResult::True;
        self.bowen_marked_opt = None;
        if !self.seen_exams.contains(&exam_id) {
            self.seen_exams.push(exam_id);
        }
        for eff in btn.effects {
            self.apply_effect_op(eff.kind, eff.number, eff.result);
        }
        self.pending = None;
        self.consume_slot(slot_idx);
        Ok((true, passed))
    }

    /// 响应 `gain` / `trade` / `sub_trade` 事件（选 `'思考一下'` 不消耗槽位）。
    pub fn answer_gain_or_trade(&mut self, displayed_idx: usize) -> Result<bool, TaixuError> {
        let Some(TaixuPendingInteraction::GainOrTrade {
            slot_idx,
            event_id,
            is_trade,
            is_sub_trade,
            buttons,
        }) = self.pending.clone()
        else {
            return Err(TaixuError::UnknownCommand("no pending gain/trade".to_string()));
        };
        let opt = buttons
            .get(displayed_idx)
            .ok_or_else(|| TaixuError::UnknownCommand(format!("event idx {displayed_idx} oob")))?;
        let Some(orig) = opt.orig_idx else {
            self.pending = None;
            return Ok(false);
        };
        let slot_node = self.optional[slot_idx].clone();
        if is_sub_trade {
            if let Some(ref ovs) = slot_node.as_ref().and_then(|n| n.override_buttons.as_ref()) {
                let b = &ovs[orig];
                for ef in &b.effects {
                    self.apply_effect_op(ef.kind(), ef.number, &ef.result);
                }
            }
        } else {
            let ev = find_universal_event(&event_id)
                .ok_or_else(|| TaixuError::UnknownCommand(format!("missing universal {event_id}")))?;
            let b = &ev.buttons[orig];
            for ef in b.effects {
                self.apply_effect_op(ef.kind, ef.number, ef.result);
            }
        }
        if !self.seen_events.contains(&event_id) {
            self.seen_events.push(event_id);
        }
        self.pending = None;
        if !is_trade && !is_sub_trade {
            if let Some(node) = slot_node {
                self.run_change_event(&node);
            }
        }
        self.consume_slot(slot_idx);
        Ok(true)
    }

    /// 响应 `choose` 节点抉择（可转为战斗、子交易或直接获得战利品）。
    pub fn answer_choose(&mut self, idx: usize) -> Result<Option<TaixuPendingInteraction>, TaixuError> {
        let Some(TaixuPendingInteraction::Choose {
            slot_idx,
            event_id,
            buttons,
        }) = self.pending.clone()
        else {
            return Err(TaixuError::UnknownCommand("no pending choose".to_string()));
        };
        let opt = buttons
            .get(idx)
            .ok_or_else(|| TaixuError::UnknownCommand(format!("choose idx {idx} oob")))?;
        let Some(orig) = opt.idx else {
            self.pending = None;
            return Ok(None);
        };
        let node = self.optional[slot_idx]
            .clone()
            .ok_or_else(|| TaixuError::UnknownCommand("missing choose slot".to_string()))?;
        let def = node
            .dlc_def_idx
            .and_then(get_dlc_event_by_idx)
            .ok_or_else(|| TaixuError::UnknownCommand(format!("missing choose {event_id}")))?;
        let btn = &def.buttons[orig];

        if !btn.enemies.is_empty() {
            let battle = TaixuPendingInteraction::Battle(TaixuPendingBattleSpec {
                slot_idx,
                event_id: def.id.to_string(),
                event_kind: def.kind.to_string(),
                season: node.season,
                chapter: node.chapter,
                dlc_def_idx: node.dlc_def_idx,
                choose_btn_idx: Some(orig),
                target_str: "all".to_string(),
                unique: def.unique,
                max_rounds: def.max_rounds,
                exp: def.exp,
            });
            self.pending = Some(battle.clone());
            return Ok(Some(battle));
        }

        for sp in btn.spoils {
            if self.rng.chance_thousandths(sp.random_thousandths) {
                for ef in sp.effects {
                    self.apply_effect_op(ef.kind, ef.number, ef.result);
                }
            }
        }
        for ef in btn.effects {
            self.apply_effect_op(ef.kind, ef.number, ef.result);
        }

        if !btn.sub_buttons.is_empty() {
            let owned_subs: Vec<TaixuOwnedSubButton> = btn
                .sub_buttons
                .iter()
                .map(|sb| TaixuOwnedSubButton {
                    name: sb.name.to_string(),
                    result_true: Some(sb.result == TaixuBtnResult::True),
                    effects: sb
                        .effects
                        .iter()
                        .map(|e| TaixuOwnedEffect {
                            kind_key: TaixuOwnedEffect::kind_name(e.kind).to_string(),
                            number: e.number,
                            result: e.result.to_string(),
                        })
                        .collect(),
                })
                .collect();
            let views: Vec<TaixuButtonOptionView> = owned_subs
                .iter()
                .enumerate()
                .map(|(i, sb)| TaixuButtonOptionView {
                    orig_idx: Some(i),
                    name: sb.name.clone(),
                    result: sb.result_true,
                    bowen_marked_wrong: false,
                })
                .collect();
            if let Some(ref mut cur) = self.optional[slot_idx] {
                cur.override_buttons = Some(owned_subs);
                cur.kind = "trade".to_string();
            }
            let next_int = TaixuPendingInteraction::GainOrTrade {
                slot_idx,
                event_id,
                is_trade: true,
                is_sub_trade: true,
                buttons: views,
            };
            self.pending = Some(next_int.clone());
            return Ok(Some(next_int));
        }

        if !self.seen_events.contains(&event_id) {
            self.seen_events.push(event_id);
        }
        self.pending = None;
        self.consume_slot(slot_idx);
        Ok(None)
    }

    fn run_change_event(&mut self, chosen_node: &TaixuSlotNode) {
        let mut pool: Vec<&'static TaixuUniversalEventSpec> = L11_TAIXU_CHANGE_PACK
            .iter()
            .filter(|c| chosen_node.level >= c.level)
            .collect();
        if pool.is_empty() {
            return;
        }
        for slot in &mut self.optional {
            if let Some(ref existing) = slot {
                if existing.seat_mismatch(chosen_node.slot_seat) && existing.change && !pool.is_empty() {
                    let pick = self.rng.gen_range(pool.len());
                    let new_ev = pool.swap_remove(pick);
                    let seat = self.next_slot_seat;
                    self.next_slot_seat = self.next_slot_seat.wrapping_add(1).max(1);
                    *slot = Some(slot_from_universal(
                        seat,
                        &self.season,
                        &chosen_node.chapter,
                        new_ev,
                    ));
                }
            }
        }
    }

    pub(super) fn consume_slot(&mut self, slot_idx: usize) {
        if slot_idx < self.optional.len() {
            self.optional[slot_idx] = None;
            self.refill_optional_slots();
        }
    }
}

impl TaixuSlotNode {
    fn seat_mismatch(&self, other_seat: u16) -> bool {
        self.slot_seat != other_seat
    }
}

fn target_to_str(t: TaixuBattleTarget) -> &'static str {
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
) -> impl Iterator<Item = &'static super::seasons::TaixuChapterEventRef> + 'a {
    L11_TAIXU_CHAPTER_EVENT_REFS_A
        .iter()
        .chain(L11_TAIXU_CHAPTER_EVENT_REFS_B.iter())
        .filter(move |r| r.season == season && r.chapter == chapter)
}

fn slot_from_dlc(
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

fn slot_from_exam(
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

fn slot_from_universal(
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
