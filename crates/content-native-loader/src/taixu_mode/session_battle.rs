//! L11 太虚幻境实局战斗发射、胜负/独特/超时结算、升级9选4与全效果执行（`session_battle.rs`）。

use engine_native::content::extension::ExtensionPack;
use engine_native::{Engine, TaixuBattleNpc, TaixuBattleSetup, Winner};

use super::catalog::{
    taixu_max_exp_for_level, TaixuEffectKind, L11_TAIXU_BUFF_STORE, L11_TAIXU_CARD_RANK,
    L11_TAIXU_EFFECT_PACK,
};
use super::seasons::{get_dlc_event_by_idx, TaixuBattleTarget, TaixuNpcSpec};
use super::session::{taixu_majun_upgrade_name, TaixuModeSession};
use super::session_types::{
    TaixuLevelUpChoice, TaixuOwnedEffect, TaixuPendingInteraction, TaixuRunScore,
};
use super::skill_rank::L11_TAIXU_SKILL_RANK;
use crate::taixu::TaixuError;
use crate::taixu_campaign_data::resolve_taixu_servant_skills;

pub(super) fn is_taixu_equip_resolvable(card_id: &str) -> bool {
    matches!(
        card_id,
        "txhj_xingtianpojunfu"
            | "txhj_zhuge"
            | "txhj_qilingong"
            | "txhj_fangtianhuaji"
            | "txhj_xuwangzhimian"
            | "txhj_piliche"
            | "txhj_liulongcanjia"
            | "txhj_leijimu"
            | "txhj_juechenjinge"
            | "txhj_guofengyupao"
            | "txhj_feilong"
            | "txhj_feilongduofeng"
            | "txhj_dinglanyemingzhu"
            | "txhj_chiyanzhenhunqin"
            | "txhj_lingbaoxianhu"
            | "txhj_chongyingshenfu"
            | "txminguangkai"
            | "txhj_taijifuchen"
            | "txhj_baihu"
            | "txzhaoshu"
            | "txzhaoshu_skill"
            | "txzhaoshu_global"
            | "qinglong"
            | "qinggang"
            | "qilin"
            | "hanbing"
            | "zhuge"
            | "guanshi"
            | "zhangba"
            | "cixiong"
            | "fangtian"
            | "renwang"
            | "bagua"
            | "zixin"
            | "dawan"
            | "chitu"
            | "jingfanma"
            | "dilu"
            | "jueying"
            | "zhuahuang"
            | "hualiu"
            | "rewrite_bagua"
            | "rewrite_renwang"
            | "rewrite_zhuge"
    )
}

fn non_txhj_general_base_hp(name: &str) -> Option<(i32, i32)> {
    match name {
        "lijue" => Some((4, 6)),
        "guosi" | "zhangji" => Some((4, 4)),
        _ => None,
    }
}

impl TaixuModeSession {
    /// 构造当前待决战斗对应的 `TaixuBattleSetup` 并启动真实原生 `Engine`（零假桩）。
    pub fn launch_battle(&self, packs: &[&ExtensionPack]) -> Result<Engine, TaixuError> {
        let Some(TaixuPendingInteraction::Battle(ref bspec)) = self.pending else {
            return Err(TaixuError::UnknownCommand("no pending battle".to_string()));
        };
        let def = bspec
            .dlc_def_idx
            .and_then(get_dlc_event_by_idx)
            .ok_or_else(|| TaixuError::UnknownCommand(format!("missing dlc def {}", bspec.event_id)))?;
        let (npc_enemies, npc_friends): (&[TaixuNpcSpec], &[TaixuNpcSpec]) =
            if let Some(bi) = bspec.choose_btn_idx {
                let b = &def.buttons[bi];
                (b.enemies, b.friends)
            } else {
                (def.enemies, def.friends)
            };

        let total_seats = 1 + npc_enemies.len() + npc_friends.len();
        let mut used = vec![false; total_seats];
        let mut placed: Vec<(u8, bool, &TaixuNpcSpec)> = Vec::new();
        for &(is_friend, spec) in &npc_enemies
            .iter()
            .map(|x| (false, x))
            .chain(npc_friends.iter().map(|x| (true, x)))
            .collect::<Vec<_>>()
        {
            let target_s = if spec.seat >= 0 && (spec.seat as usize) < total_seats {
                spec.seat as usize
            } else {
                used.iter().position(|&u| !u).unwrap_or(0)
            };
            let s = if !used[target_s] {
                target_s
            } else {
                used.iter().position(|&u| !u).unwrap_or(target_s)
            };
            used[s] = true;
            placed.push((s as u8, is_friend, spec));
        }
        let player_seat = used.iter().position(|&u| !u).unwrap_or(0) as u8;

        let eff_pack = L11_TAIXU_EFFECT_PACK
            .iter()
            .find(|e| e.id == self.effect_id);
        let global_skills: Vec<String> = eff_pack
            .map(|e| e.global_skills.iter().map(|s| (*s).to_string()).collect())
            .unwrap_or_default();
        let hp_delta: i32 = match self.effect_id.as_str() {
            "lingqiyiman" => 2,
            "lingqikuijie" => -1,
            _ => 0,
        };
        let enemy_min_hs_delta: i32 = {
            let mut d = match self.rank {
                5..=u8::MAX => 3,
                4 => 2,
                3 => 1,
                _ => 0,
            };
            if self.effect_id == "chongfenbeizhan" {
                d += 2;
            } else if self.effect_id == "cangcuchuji" {
                d -= 3;
            }
            d
        };

        let mut high_equips: Vec<&'static str> = L11_TAIXU_CARD_RANK
            .iter()
            .filter(|c| c.rank >= 3 && c.category.is_equip() && !c.unique && is_taixu_equip_resolvable(c.card_id))
            .map(|c| c.card_id)
            .collect();
        if high_equips.is_empty() {
            high_equips.push("qinglong");
        }
        let mut npcs = Vec::with_capacity(placed.len());
        let mut r = self.rng.clone();
        for (seat, is_friend, spec) in placed {
            let prof =
                crate::taixu_general_profiles::lookup_l11_general_profile(spec.general_name);
            let (base_hp, base_max_hp) = prof
                .map(|p| (p.starting_hp as i32, p.max_hp as i32))
                .or_else(|| non_txhj_general_base_hp(spec.general_name))
                .unwrap_or((4, 4));
            let (mut n_hp, mut n_max_hp) = if spec.hp > 0 {
                (i32::from(spec.hp), i32::from(spec.hp))
            } else {
                (base_hp, base_max_hp)
            };
            n_max_hp += spec.max_hp_bonus;
            n_hp += spec.max_hp_bonus;
            if spec.correct_scaling && self.level > spec.level {
                let diff = i32::from(self.level - spec.level);
                let delta = diff * if self.rank > 1 { 2 } else { 1 };
                n_max_hp += delta;
                n_hp += delta;
            }
            if !is_friend && self.rank > 1 {
                n_max_hp += 1;
                n_hp += 1;
            }
            n_max_hp = (n_max_hp + hp_delta).max(1);
            n_hp = (n_hp + hp_delta).clamp(1, n_max_hp);
            let min_hs = (spec.min_hs as i32 + enemy_min_hs_delta).max(0) as u32;
            let mut nskills: Vec<String> = global_skills.clone();
            if !is_friend && self.rank > 2 {
                nskills.push("reyingzi".to_string());
                nskills.push("mashu".to_string());
            }
            let mut nequips: Vec<String> = spec.equips.iter().map(|s| (*s).to_string()).collect();
            if !is_friend && self.rank > 3 {
                for _ in 0..2 {
                    let pick = r.gen_range(high_equips.len());
                    nequips.push(high_equips[pick].to_string());
                }
            }
            if self.effect_id == "huanjingcaoge" {
                let pick = r.gen_range(high_equips.len());
                nequips.push(high_equips[pick].to_string());
            }
            npcs.push(TaixuBattleNpc {
                seat,
                is_friend,
                is_boss: spec.is_boss,
                general_name: spec.general_name.to_string(),
                max_hp: n_max_hp as u32,
                hp: n_hp as u32,
                min_hs,
                equip_names: nequips,
                card_names: Vec::new(),
                skills: nskills,
            });
        }

        let has_majun = self.buffs.iter().any(|b| b == "buff_txhj_majundegaizao");
        let player_equip_names: Vec<String> = [&self.equip1, &self.equip2, &self.equip3, &self.equip4]
            .into_iter()
            .flatten()
            .map(|n| {
                if has_majun {
                    taixu_majun_upgrade_name(n).to_string()
                } else {
                    n.clone()
                }
            })
            .collect();
        let mut player_skills: Vec<String> = self.use_skills.clone();
        player_skills.extend(self.buffs.iter().cloned());
        for eq in [&self.equip1, &self.equip2, &self.equip3, &self.equip4]
            .into_iter()
            .flatten()
        {
            if let Some(b) = Self::artifact_buff_for_equip(eq) {
                if !player_skills.iter().any(|s| s == b) {
                    player_skills.push(b.to_string());
                }
            }
        }
        if let Some(ref sid) = self.servant_id {
            if let Some(sks) = resolve_taixu_servant_skills(sid) {
                for sk in sks {
                    player_skills.push((*sk).to_string());
                }
            }
        }
        let player_min_hs = if self.buffs.iter().any(|b| b == "buff_txhj_pofuchenzhou") {
            1
        } else if self.buffs.iter().any(|b| b == "buff_txhj_fuzu") {
            self.max_hp.max(1) as u32
        } else if self.min_hs >= 0 {
            self.min_hs as u32
        } else {
            4
        };

        let setup = TaixuBattleSetup {
            seed: self.rng.state ^ (u64::from(self.score_fight) << 8),
            player_seat,
            player_general: self.player_general.clone(),
            player_max_hp: self.max_hp.max(1) as u32,
            player_hp: self.hp.clamp(1, self.max_hp.max(1)) as u32,
            player_min_hs,
            player_max_hs_bonus: self.max_hs.max(0) as u32,
            player_adjust: self.adjust.max(0) as u32,
            rank: self.rank,
            servant_grade: self.servant_grade,
            target_kill: bspec.target() == TaixuBattleTarget::Kill,
            target_repair: bspec.target() == TaixuBattleTarget::Repair,
            target_subsist: bspec.target() == TaixuBattleTarget::Subsist,
            max_rounds: bspec.max_rounds,
            player_paiku_cards: self.cards.clone(),
            player_equip_names,
            player_skills,
            npcs,
        };
        Engine::new_taixu_versus(&setup, packs).map_err(|e| TaixuError::InvalidField {
            field: "battle_setup",
            reason: format!("engine battle setup failed: {e:?}"),
        })
    }

    /// 战后结算（严格还原 `TaiXuHuanJingState`：`win | unique/timeout false | fatal null`、
    /// `aozhan` / `xianhujiqu`、`spoils`、`aHaoSkill1`、升级 9 选 4 与切章/`Ultimate`）。
    pub fn settle_battle(&mut self, engine: &Engine) -> Result<Option<TaixuPendingInteraction>, TaixuError> {
        let Some(TaixuPendingInteraction::Battle(bspec)) = self.pending.take() else {
            return Err(TaixuError::UnknownCommand("no pending battle to settle".to_string()));
        };
        let core = engine.state_core();
        let me_seat = (0..core.players.len() as u8)
            .find(|&p| core.vm_aux.flags.get(&(p, "l11_txhj_active_battle".to_string())).copied().unwrap_or(0) > 0)
            .unwrap_or(0);
        let outcome = engine.outcome().ok_or_else(|| {
            TaixuError::UnknownCommand("cannot settle battle before outcome".to_string())
        })?;
        let my_team = match &core.mode {
            engine_native::GameMode::Versus { teams, .. } => {
                teams.iter().position(|t| t.contains(&me_seat)).unwrap_or(0)
            }
            _ => 0,
        };
        let won = outcome.winner == Winner::Versus(my_team);
        let timed_out_loss = core
            .vm_aux
            .flags
            .get(&(me_seat, "l11_timelimit_outcome".to_string()))
            .copied()
            .unwrap_or(0)
            == 2;
        let battle_result_not_fatal = won || bspec.unique || timed_out_loss;

        self.score_fight += 1;
        self.score_round += core.turn.round;

        let end_pl = core.player(me_seat);
        if self.buffs.iter().any(|b| b == "buff_txhj_aozhan") {
            self.max_hp = (end_pl.max_hp as i32).max(1);
            self.hp = if end_pl.hp > 0 { end_pl.hp } else { 1 };
        } else {
            let has_xianhu = self.buffs.iter().any(|b| b == "buff_txhj_xianhujiqu")
                || self.active_equip_has_buff("buff_txhj_xianhujiqu");
            if battle_result_not_fatal && has_xianhu {
                let end_max = end_pl.max_hp as i32;
                if end_max > self.max_hp {
                    self.max_hp = end_max;
                }
            }
            self.hp = self.max_hp;
        }

        if !battle_result_not_fatal {
            self.failed = true;
            self.final_score = Some(self.compute_score(false));
            return Ok(None);
        }

        // `Ultimate` 胜负差异（`extension_framework.js:8921,9447-9484`）：
        // - `boss` 即使 non-fatal defeat（`unique` / 超时）仍消费槽位并推进 `chapter++`（防卡章）；
        // - `Ultimate` 仅在胜利时标记通关与消费槽位，non-fatal loss 保持 `Ultimate` 可重挑且不判通关。
        if won || bspec.event_kind != "Ultimate" {
            if !self.seen_events.contains(&bspec.event_id) {
                self.seen_events.push(bspec.event_id.clone());
            }
            self.consume_slot(bspec.slot_idx);
        }

        if won {
            if let Some(def) = bspec.dlc_def_idx.and_then(get_dlc_event_by_idx) {
                if let Some(bi) = bspec.choose_btn_idx {
                    if let Some(btn) = def.buttons.get(bi) {
                        self.apply_spoils_list(btn.spoils);
                        for npc in btn.enemies {
                            self.apply_spoils_list(npc.spoils);
                        }
                    }
                } else {
                    for npc in def.enemies {
                        self.apply_spoils_list(npc.spoils);
                    }
                }
            }
            if self.buffs.iter().any(|b| b == "txhj_aHaoSkill1") {
                self.apply_effect_op(TaixuEffectKind::RandomEquip, 1, "");
            }
        }

        let is_boss_advance = bspec.event_kind == "boss";
        let is_ultimate_win = won && bspec.event_kind == "Ultimate";

        let new_exp = self.exp + bspec.exp;
        if new_exp >= self.max_exp && self.level < 10 {
            self.exp = 0;
            self.level += 1;
            self.max_exp = taixu_max_exp_for_level(self.level);
            let mut opts = vec![
                TaixuLevelUpChoice { kind_key: "randomSkill".into(), name: "获得技能".into(), number: 1 },
                TaixuLevelUpChoice { kind_key: "randomCard".into(), name: "获得卡牌".into(), number: 1 },
                TaixuLevelUpChoice { kind_key: "randomEquip".into(), name: "获得装备".into(), number: 1 },
                TaixuLevelUpChoice { kind_key: "adjust".into(), name: "调度次数+1".into(), number: 1 },
                TaixuLevelUpChoice { kind_key: "maxHp".into(), name: "体力上限+1".into(), number: 1 },
                TaixuLevelUpChoice { kind_key: "minHs".into(), name: "初始手牌+1".into(), number: 1 },
                TaixuLevelUpChoice { kind_key: "maxHs".into(), name: "手牌上限+2".into(), number: 2 },
                TaixuLevelUpChoice { kind_key: "coin".into(), name: "金币+300".into(), number: 300 },
                TaixuLevelUpChoice { kind_key: "randomBuff".into(), name: "获得祝福".into(), number: 1 },
            ];
            let mut four = Vec::new();
            while four.len() < 4 && !opts.is_empty() {
                let idx = self.rng.gen_range(opts.len());
                four.push(opts.swap_remove(idx));
            }
            let lvl = TaixuPendingInteraction::LevelUp {
                choices: four,
                pending_chapter_clear: is_boss_advance,
                pending_run_victory: is_ultimate_win,
            };
            self.pending = Some(lvl.clone());
            return Ok(Some(lvl));
        }
        self.exp = new_exp;

        if is_ultimate_win {
            self.completed = true;
            self.final_score = Some(self.compute_score(true));
        } else if is_boss_advance {
            let next_chap = self.chapter_idx + 1;
            self.enter_chapter(next_chap)?;
        }
        Ok(None)
    }

    pub fn choose_level_up(&mut self, choice_idx: usize) -> Result<(), TaixuError> {
        let Some(TaixuPendingInteraction::LevelUp {
            choices,
            pending_chapter_clear,
            pending_run_victory,
        }) = self.pending.take()
        else {
            return Err(TaixuError::UnknownCommand("no pending level_up".to_string()));
        };
        let pick = choices
            .get(choice_idx)
            .ok_or_else(|| TaixuError::UnknownCommand(format!("levelup idx {choice_idx} oob")))?;
        let eff = TaixuOwnedEffect {
            kind_key: pick.kind_key.clone(),
            number: pick.number,
            result: String::new(),
        };
        self.apply_effect_op(eff.kind(), eff.number, "");
        if pending_run_victory {
            self.completed = true;
            self.final_score = Some(self.compute_score(true));
        } else if pending_chapter_clear {
            let next_chap = self.chapter_idx + 1;
            self.enter_chapter(next_chap)?;
        }
        Ok(())
    }

    /// 执行单条 `moveEffect`（完整对齐 `extension_framework.js:4751-5424`）。
    pub fn apply_effect_op(&mut self, kind: TaixuEffectKind, number: i32, result: &str) {
        match kind {
            TaixuEffectKind::Coin => {
                self.coin = (self.coin + number).max(0);
                self.max_coin += number;
            }
            TaixuEffectKind::Exp => {
                self.exp = (self.exp as i32 + number).max(0) as u32;
                if self.exp >= self.max_exp && self.level < 10 {
                    self.exp = 0;
                    self.level += 1;
                    self.max_exp = taixu_max_exp_for_level(self.level);
                }
            }
            TaixuEffectKind::Hp => {
                self.hp += number;
                if self.hp < 0 {
                    self.hp = 1;
                }
                if self.hp > self.max_hp {
                    self.hp = self.max_hp;
                }
            }
            TaixuEffectKind::MaxHp => {
                self.max_hp += number;
                self.hp += number;
                if self.max_hp < 0 {
                    self.max_hp = 1;
                }
                if self.hp > self.max_hp {
                    self.hp = self.max_hp;
                }
            }
            TaixuEffectKind::MinHs => self.min_hs = (self.min_hs + number).max(0),
            TaixuEffectKind::MaxHs => self.max_hs = (self.max_hs + number).max(0),
            TaixuEffectKind::Adjust => self.adjust = (self.adjust + number).max(0),
            TaixuEffectKind::MaxSkills => {
                self.max_skills = ((self.max_skills as i32) + number).max(0) as usize;
            }
            TaixuEffectKind::Card => {
                for _ in 0..number.max(0) {
                    if !result.is_empty() {
                        self.cards.push(result.to_string());
                        *self.collect_cards.entry(result.to_string()).or_insert(0) += 1;
                    }
                }
            }
            TaixuEffectKind::RandomCard | TaixuEffectKind::Basic | TaixuEffectKind::Trick => {
                let pool: Vec<&'static str> = L11_TAIXU_CARD_RANK
                    .iter()
                    .filter(|c| match kind {
                        TaixuEffectKind::Basic => c.category == super::catalog::TaixuCardCategory::Basic,
                        TaixuEffectKind::Trick => matches!(
                            c.category,
                            super::catalog::TaixuCardCategory::Trick | super::catalog::TaixuCardCategory::Delay
                        ),
                        _ => !c.category.is_equip(),
                    })
                    .map(|c| c.card_id)
                    .collect();
                for _ in 0..number.max(0) {
                    if !pool.is_empty() {
                        let c = pool[self.rng.gen_range(pool.len())];
                        self.cards.push(c.to_string());
                        *self.collect_cards.entry(c.to_string()).or_insert(0) += 1;
                    }
                }
            }
            TaixuEffectKind::RemoveCard => {
                for _ in 0..number.max(0) {
                    if self.cards.is_empty() {
                        self.apply_effect_op(TaixuEffectKind::MaxHp, -1, "");
                    } else {
                        let idx = self.rng.gen_range(self.cards.len());
                        self.cards.remove(idx);
                    }
                }
            }
            TaixuEffectKind::Equip => {
                for _ in 0..number.max(0) {
                    if !result.is_empty() {
                        self.acquire_equip(result);
                    }
                }
            }
            TaixuEffectKind::RandomEquip
            | TaixuEffectKind::Equip1
            | TaixuEffectKind::Equip2
            | TaixuEffectKind::Equip5
            | TaixuEffectKind::RandomUnique => {
                let pool: Vec<&'static str> = L11_TAIXU_CARD_RANK
                    .iter()
                    .filter(|c| match kind {
                        TaixuEffectKind::Equip1 => !c.unique && c.category == super::catalog::TaixuCardCategory::Equip1,
                        TaixuEffectKind::Equip2 => !c.unique && c.category == super::catalog::TaixuCardCategory::Equip2,
                        TaixuEffectKind::Equip5 => !c.unique && c.category == super::catalog::TaixuCardCategory::Equip5,
                        TaixuEffectKind::RandomUnique => c.unique && c.category.is_equip(),
                        _ => !c.unique && c.category.is_equip(),
                    })
                    .map(|c| c.card_id)
                    .collect();
                for _ in 0..number.max(0) {
                    if !pool.is_empty() {
                        let eq = pool[self.rng.gen_range(pool.len())];
                        self.acquire_equip(eq);
                    }
                }
            }
            TaixuEffectKind::RemoveEquip => {
                for _ in 0..number.max(0) {
                    let mut slots = Vec::new();
                    if self.equip1.is_some() { slots.push(1usize); }
                    if self.equip2.is_some() { slots.push(2usize); }
                    if self.equip3.is_some() { slots.push(3usize); }
                    if self.equip4.is_some() { slots.push(4usize); }
                    if !self.equips.is_empty() { slots.push(5usize); }
                    if slots.is_empty() {
                        self.apply_effect_op(TaixuEffectKind::Hp, -1, "");
                    } else {
                        let removed = match slots[self.rng.gen_range(slots.len())] {
                            1 => self.equip1.take(),
                            2 => self.equip2.take(),
                            3 => self.equip3.take(),
                            4 => self.equip4.take(),
                            _ => {
                                let idx = self.rng.gen_range(self.equips.len());
                                let _ = self.equips.remove(idx);
                                None
                            }
                        };
                        self.sync_equip_buffs(removed.as_deref());
                    }
                }
            }
            TaixuEffectKind::Buff => {
                if !result.is_empty() && !self.buffs.iter().any(|b| b == result) {
                    self.buffs.push(result.to_string());
                    *self.collect_cards.entry(result.to_string()).or_insert(0) += 1;
                }
            }
            TaixuEffectKind::RandomBuff => {
                let mut pool: Vec<&'static str> = L11_TAIXU_BUFF_STORE
                    .iter()
                    .filter(|b| b.store && !self.buffs.iter().any(|x| x == b.id))
                    .map(|b| b.id)
                    .collect();
                for _ in 0..number.max(0) {
                    if !pool.is_empty() {
                        let b = pool.swap_remove(self.rng.gen_range(pool.len()));
                        self.buffs.push(b.to_string());
                        *self.collect_cards.entry(b.to_string()).or_insert(0) += 1;
                    }
                }
            }
            TaixuEffectKind::RemoveBuff => {
                for _ in 0..number.max(0) {
                    if self.buffs.is_empty() {
                        self.apply_effect_op(TaixuEffectKind::Hp, -1, "");
                    } else {
                        let idx = self.rng.gen_range(self.buffs.len());
                        self.buffs.remove(idx);
                    }
                }
            }
            TaixuEffectKind::Skill => {
                if !result.is_empty() {
                    self.acquire_skill(result);
                }
            }
            TaixuEffectKind::RandomSkill
            | TaixuEffectKind::Attack
            | TaixuEffectKind::Defense
            | TaixuEffectKind::Assist
            | TaixuEffectKind::Burst => {
                let mask = match kind {
                    TaixuEffectKind::Attack => 1,
                    TaixuEffectKind::Defense => 2,
                    TaixuEffectKind::Assist => 4,
                    TaixuEffectKind::Burst => 8,
                    _ => 0,
                };
                let mut pool: Vec<&'static str> = L11_TAIXU_SKILL_RANK
                    .iter()
                    .filter(|s| {
                        (mask == 0 || (s.type_mask & mask) != 0)
                            && !self.use_skills.iter().any(|u| u == s.skill_id)
                            && self
                                .available_skills_filter
                                .as_ref()
                                .is_none_or(|f| f.iter().any(|x| x == s.skill_id))
                    })
                    .map(|s| s.skill_id)
                    .collect();
                for _ in 0..number.max(0) {
                    if !pool.is_empty() {
                        let sk = pool.swap_remove(self.rng.gen_range(pool.len()));
                        self.acquire_skill(sk);
                    }
                }
            }
            TaixuEffectKind::RemoveSkill => {
                for _ in 0..number.max(0) {
                    let tot = self.use_skills.len() + self.skills.len();
                    if tot == 0 {
                        self.apply_effect_op(TaixuEffectKind::Hp, -1, "");
                    } else {
                        let pick = self.rng.gen_range(tot);
                        if pick < self.use_skills.len() {
                            self.use_skills.remove(pick);
                        } else {
                            self.skills.remove(pick - self.use_skills.len());
                        }
                    }
                }
            }
        }
    }

    fn apply_spoils_list(&mut self, spoils: &[super::seasons::TaixuSpoilSpec]) {
        for sp in spoils {
            if self.rng.chance_thousandths(sp.random_thousandths) {
                for ef in sp.effects {
                    self.apply_effect_op(ef.kind, ef.number, ef.result);
                }
            }
        }
    }

    /// 9 件太虚神器对应的被动祝福映射（`extension_framework.js:4560-4595` `equipBuffUpdate`）。
    pub fn artifact_buff_for_equip(card_id: &str) -> Option<&'static str> {
        match card_id {
            "txhj_feilong" | "txhj_feilongduofeng" => Some("buff_txhj_pozhenzhifeng"),
            "txhj_chiyanzhenhunqin" => Some("buff_txhj_yanhuozhiren"),
            "txhj_lingbaoxianhu" => Some("buff_txhj_xianhujiqu"),
            "txhj_taijifuchen" => Some("buff_txhj_fuchendangmo"),
            "txhj_chongyingshenfu" | "txhj_xuwangzhimian" => Some("buff_txhj_shenfuhuaxie"),
            "txhj_guofengyupao" => Some("buff_txhj_juejingzhice"),
            "txhj_juechenjinge" => Some("buff_txhj_wuzhongshengshan"),
            "txhj_liulongcanjia" => Some("buff_txhj_wuzhongshengsha"),
            _ => None,
        }
    }

    fn active_equip_has_buff(&self, buff_id: &str) -> bool {
        [&self.equip1, &self.equip2, &self.equip3, &self.equip4]
            .into_iter()
            .flatten()
            .any(|eq| Self::artifact_buff_for_equip(eq) == Some(buff_id))
    }

    fn equip_slot_for_card_id(card_id: &str) -> Option<&'static str> {
        match card_id {
            "txhj_feilong" | "txhj_feilongduofeng" => Some("equip1"),
            "txzhaoshu" | "txzhaoshu_skill" | "txzhaoshu_global" => Some("equip5"),
            _ => L11_TAIXU_CARD_RANK
                .iter()
                .find(|c| c.card_id == card_id)
                .and_then(|c| c.category.equip_slot_name()),
        }
    }

    /// 同步 `equip1..4` 装备栏与 `self.buffs` 神器祝福状态（`equipBuffUpdate` + `home.update`）。
    pub fn sync_equip_buffs(&mut self, removed_equip: Option<&str>) {
        if let Some(old) = removed_equip {
            if let Some(b) = Self::artifact_buff_for_equip(old) {
                self.buffs.retain(|x| x != b);
            }
        }
        for eq in [&self.equip1, &self.equip2, &self.equip3, &self.equip4]
            .into_iter()
            .flatten()
        {
            if let Some(b) = Self::artifact_buff_for_equip(eq) {
                if !self.buffs.iter().any(|x| x == b) {
                    self.buffs.push(b.to_string());
                }
            }
        }
    }

    /// 从备选装备栏 (`self.equips`) 装备到 `equip1..4`（映射 `lookEquip` 宝物栏1/宝物栏2与 `equipBuffUpdate`）。
    pub fn equip_from_bag(&mut self, bag_idx: usize) -> Result<(), TaixuError> {
        if bag_idx >= self.equips.len() {
            return Err(TaixuError::UnknownCommand(format!("equip bag idx {bag_idx} oob")));
        }
        let item = self.equips.remove(bag_idx);
        let slot = Self::equip_slot_for_card_id(&item);
        let replaced = match slot {
            Some("equip1") => self.equip1.replace(item),
            Some("equip2") => self.equip2.replace(item),
            _ if self.equip3.is_none() => self.equip3.replace(item),
            _ => self.equip4.replace(item),
        };
        if let Some(ref old) = replaced {
            self.equips.push(old.clone());
        }
        self.sync_equip_buffs(replaced.as_deref());
        Ok(())
    }

    /// 将当前穿戴槽位 (`1..=4`) 卸入 `self.equips` 并同步移除独占神器祝福（`lookEquip` 卸下）。
    pub fn unequip_slot(&mut self, slot: usize) -> Result<(), TaixuError> {
        let old = match slot {
            1 => self.equip1.take(),
            2 => self.equip2.take(),
            3 => self.equip3.take(),
            4 => self.equip4.take(),
            _ => return Err(TaixuError::UnknownCommand(format!("invalid equip slot {slot}"))),
        };
        if let Some(ref eq) = old {
            self.equips.push(eq.clone());
        }
        self.sync_equip_buffs(old.as_deref());
        Ok(())
    }

    pub(super) fn acquire_equip(&mut self, card_id: &str) {
        let slot = Self::equip_slot_for_card_id(card_id);
        match slot {
            Some("equip1") if self.equip1.is_none() => self.equip1 = Some(card_id.to_string()),
            Some("equip2") if self.equip2.is_none() => self.equip2 = Some(card_id.to_string()),
            Some("equip3") if self.equip3.is_none() => self.equip3 = Some(card_id.to_string()),
            Some("equip4") if self.equip4.is_none() => self.equip4 = Some(card_id.to_string()),
            _ => self.equips.push(card_id.to_string()),
        }
        self.sync_equip_buffs(None);
        *self.collect_cards.entry(card_id.to_string()).or_insert(0) += 1;
    }

    pub(super) fn acquire_skill(&mut self, skill_id: &str) {
        if self.use_skills.len() < self.max_skills && !self.use_skills.iter().any(|s| s == skill_id) {
            self.use_skills.push(skill_id.to_string());
        } else if !self.skills.iter().any(|s| s == skill_id) && !self.use_skills.iter().any(|s| s == skill_id) {
            self.skills.push(skill_id.to_string());
        }
        *self.collect_cards.entry(skill_id.to_string()).or_insert(0) += 1;
    }

    pub fn compute_score(&self, won: bool) -> TaixuRunScore {
        let mut value_score = 0u32;
        for sk in &self.use_skills {
            let r = L11_TAIXU_SKILL_RANK.iter().find(|s| s.skill_id == sk).map_or(1, |s| s.rank);
            value_score += u32::from(r) * 100;
        }
        for b in &self.buffs {
            let r = L11_TAIXU_BUFF_STORE.iter().find(|x| x.id == b).map_or(1, |x| x.rank);
            value_score += u32::from(r) * 100;
        }
        for cname in &self.cards {
            let r = L11_TAIXU_CARD_RANK.iter().find(|c| c.card_id == cname).map_or(1, |c| c.rank);
            value_score += u32::from(r) * 50;
        }
        for eq in [&self.equip1, &self.equip2, &self.equip3, &self.equip4].into_iter().flatten() {
            let r = L11_TAIXU_CARD_RANK.iter().find(|c| c.card_id == eq).map_or(1, |c| c.rank);
            value_score += u32::from(r) * 50;
        }
        let fight_score = self.score_fight * 100;
        let spent_coin = (self.max_coin - self.coin).max(0) as u32;
        let eff_m = L11_TAIXU_EFFECT_PACK
            .iter()
            .find(|e| e.id == self.effect_id)
            .map_or(0, |e| e.score_num_thousandths);
        let base_total = value_score + fight_score + spent_coin;
        let effect_bonus = if eff_m > 1000 {
            (eff_m / 1000) as u32
        } else {
            ((u64::from(base_total) * (eff_m.max(0) as u64)) / 1000) as u32
        };
        let mut total = base_total + effect_bonus;
        if won {
            total += total / 10;
        }
        TaixuRunScore {
            won,
            fights: self.score_fight,
            total_rounds: self.score_round,
            max_coin: self.max_coin,
            value_score,
            effect_bonus,
            total_score: total,
        }
    }
}
