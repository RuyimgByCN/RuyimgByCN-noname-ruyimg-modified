use crate::engine::Engine;
use crate::engine::q4_ext::{decode_biased_i32, encode_biased_i32};
use crate::state::GameMode;

const RANK_BIAS: i32 = 10_000;

impl Engine {
    fn general_name_of(&self, def: u16) -> Option<&'static str> {
        self.catalog.general_meta(def).map(|m| m.0)
    }

    fn set_named_flag(&mut self, prefix: &str, name: &str, enabled: bool) {
        let key = (0u8, format!("{prefix}{name}"));
        if enabled {
            self.state.core.vm_aux.flags.insert(key, 1);
        } else {
            self.state.core.vm_aux.flags.remove(&key);
        }
    }

    fn get_named_flag(&self, prefix: &str, name: &str) -> bool {
        self.state
            .core
            .vm_aux
            .flags
            .get(&(0u8, format!("{prefix}{name}")))
            .copied()
            .unwrap_or(0)
            != 0
    }

    /// Persist `L15` / `U02` `forbidai` general flag into `vm_aux.flags`.
    pub fn set_general_forbidai(&mut self, def: u16, forbid: bool) {
        if let Some(name) = self.general_name_of(def) {
            self.set_named_flag("ext_forbidai:", name, forbid);
        }
    }

    /// Persist `L15` `forbidai` by general name (works for both standard and extension generals).
    pub fn set_general_forbidai_named(&mut self, name: &str, forbid: bool) {
        self.set_named_flag("ext_forbidai:", name, forbid);
    }

    pub fn is_general_forbidai(&self, def: u16) -> bool {
        self.general_name_of(def)
            .is_some_and(|name| self.get_named_flag("ext_forbidai:", name))
    }

    /// Persist `L15` `unseen` general flag into `vm_aux.flags`.
    pub fn set_general_unseen(&mut self, def: u16, unseen: bool) {
        if let Some(name) = self.general_name_of(def) {
            self.set_named_flag("ext_unseen:", name, unseen);
        }
    }

    pub fn set_general_unseen_named(&mut self, name: &str, unseen: bool) {
        self.set_named_flag("ext_unseen:", name, unseen);
    }

    pub fn is_general_unseen(&self, def: u16) -> bool {
        self.general_name_of(def)
            .is_some_and(|name| self.get_named_flag("ext_unseen:", name))
    }

    /// Persist `L15` human/pool ban categories (`sexforbid`, `txhj`, `packs`) into `vm_aux.flags`.
    pub fn set_general_ban_category(&mut self, name: &str, category: &str, banned: bool) {
        let prefix = match category {
            "sex" | "sexforbid" => "ext_ban_sex:",
            "txhj" => "ext_ban_txhj:",
            "packs" => "ext_ban_packs:",
            _ => "ext_ban_other:",
        };
        self.set_named_flag(prefix, name, banned);
    }

    pub fn is_general_human_banned(&self, def: u16) -> bool {
        let Some(name) = self.general_name_of(def) else {
            return true;
        };
        self.get_named_flag("ext_ban_sex:", name)
            || self.get_named_flag("ext_ban_txhj:", name)
            || self.get_named_flag("ext_ban_packs:", name)
            || self.get_named_flag("ext_ban_other:", name)
    }

    /// Persist `L15` `changePlayer_pool` membership bitmask (bit 0=huanyuan, 1=yinjian, 2=doudizhu).
    pub fn set_l15_pool_bits(&mut self, name: &str, bits: u32) {
        let key = (0u8, format!("l15_pool:{name}"));
        if bits == 0 {
            self.state.core.vm_aux.flags.remove(&key);
        } else {
            self.state.core.vm_aux.flags.insert(key, bits);
        }
    }

    pub fn l15_pool_bits(&self, name: &str) -> u32 {
        self.state
            .core
            .vm_aux
            .flags
            .get(&(0u8, format!("l15_pool:{name}")))
            .copied()
            .unwrap_or(0)
    }

    /// Filter catalog general candidates by `L15` unseen, ban categories, and `forbidai`.
    pub fn filter_selectable_generals(&self, for_ai: bool) -> Vec<u16> {
        self.general_candidates()
            .into_iter()
            .filter(|&def| {
                !self.is_general_unseen(def)
                    && !self.is_general_human_banned(def)
                    && (!for_ai || !self.is_general_forbidai(def))
            })
            .collect()
    }

    /// Configure `U02`/`L03` boss mode `isBossAllowed` pool gating.
    pub fn set_boss_pool_filtering(&mut self, active: bool) {
        let key = (0u8, "ext_boss_pool_filtered".to_string());
        if active {
            self.state.core.vm_aux.flags.insert(key, 1);
        } else {
            self.state.core.vm_aux.flags.remove(&key);
        }
    }

    pub fn set_boss_general_allowed(&mut self, name: &str, allowed: bool) {
        self.set_named_flag("ext_boss_allowed:", name, allowed);
    }

    /// Selectable boss-mode candidates (`isBossAllowed` when boss filtering is active).
    pub fn filter_boss_generals(&self) -> Vec<u16> {
        let active = self
            .state
            .core
            .vm_aux
            .flags
            .get(&(0u8, "ext_boss_pool_filtered".to_string()))
            .copied()
            .unwrap_or(0)
            != 0;
        self.general_candidates()
            .into_iter()
            .filter(|&def| {
                let Some(name) = self.general_name_of(def) else {
                    return false;
                };
                !self.is_general_unseen(def)
                    && (!active || self.get_named_flag("ext_boss_allowed:", name))
            })
            .collect()
    }

    /// Configure `L17` `AIqianghuaXuanjiang` flags and per-general Doudizhu star rankings.
    pub fn set_doudizhu_ai_config(&mut self, enabled: bool, online: bool) {
        let e_key = (0u8, "l17_ai_xuanjiang".to_string());
        let o_key = (0u8, "l17_doudizhu_online".to_string());
        if enabled {
            self.state.core.vm_aux.flags.insert(e_key, 1);
        } else {
            self.state.core.vm_aux.flags.remove(&e_key);
        }
        if online {
            self.state.core.vm_aux.flags.insert(o_key, 1);
        } else {
            self.state.core.vm_aux.flags.remove(&o_key);
        }
    }

    pub fn set_doudizhu_ai_ranks(&mut self, general_name: &str, lord_score: i32, farmer_score: i32) {
        self.state.core.vm_aux.flags.insert(
            (0u8, format!("l17_rank_lord:{general_name}")),
            encode_biased_i32(lord_score, RANK_BIAS),
        );
        self.state.core.vm_aux.flags.insert(
            (0u8, format!("l17_rank_farmer:{general_name}")),
            encode_biased_i32(farmer_score, RANK_BIAS),
        );
    }

    pub fn doudizhu_ai_score(&self, general_name: &str, is_lord: bool) -> Option<i32> {
        let prefix = if is_lord {
            "l17_rank_lord:"
        } else {
            "l17_rank_farmer:"
        };
        self.state
            .core
            .vm_aux
            .flags
            .get(&(0u8, format!("{prefix}{general_name}")))
            .copied()
            .map(|raw| decode_biased_i32(raw, RANK_BIAS))
    }

    /// Select Doudizhu AI general(s) from `candidates` using `L17 AIqianghuaXuanjiang` scoring
    /// when `online && Doudizhu`, falling back to candidate-order selection otherwise.
    pub fn pick_doudizhu_ai_general(
        &mut self,
        candidates: &[u16],
        is_lord: bool,
        double_mode: bool,
    ) -> (Option<u16>, Option<u16>) {
        if !matches!(self.state.core.mode, GameMode::Doudizhu { .. }) {
            return (None, None);
        }
        let filtered: Vec<u16> = candidates
            .iter()
            .copied()
            .filter(|&def| {
                self.general_name_of(def).is_some()
                    && !self.is_general_unseen(def)
                    && !self.is_general_human_banned(def)
                    && !self.is_general_forbidai(def)
            })
            .collect();
        if filtered.is_empty() {
            return (None, None);
        }
        let enabled = self
            .state
            .core
            .vm_aux
            .flags
            .get(&(0u8, "l17_ai_xuanjiang".to_string()))
            .copied()
            .unwrap_or(0)
            != 0;
        let online = self
            .state
            .core
            .vm_aux
            .flags
            .get(&(0u8, "l17_doudizhu_online".to_string()))
            .copied()
            .unwrap_or(0)
            != 0;
        if !enabled || !online {
            let first = filtered[0];
            let second = if double_mode && filtered.len() > 1 {
                Some(filtered[1])
            } else {
                None
            };
            return (Some(first), second);
        }
        let scores: Vec<Option<i32>> = filtered
            .iter()
            .map(|&def| {
                self.general_name_of(def)
                    .and_then(|name| self.doudizhu_ai_score(name, is_lord))
            })
            .collect();
        let mut order: Vec<usize> = (0..filtered.len()).collect();
        order.sort_by(|&l, &r| match (scores[l], scores[r]) {
            (Some(ls), Some(rs)) => rs.cmp(&ls).then(l.cmp(&r)),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => l.cmp(&r),
        });
        if double_mode {
            let first = filtered[order[0]];
            let second = if order.len() > 1 {
                let pick = self.state.core.rng.gen_range((order.len() - 1) as u32) as usize;
                Some(filtered[order[pick + 1]])
            } else {
                None
            };
            return (Some(first), second);
        }
        let gap = if is_lord { 1 } else { 2 };
        let threshold = if is_lord { 800 } else { 700 };
        let roll = self.state.core.rng.gen_range(1000);
        let first_score = scores[order[0]];
        let second_score = order.get(1).and_then(|&idx| scores[idx]);
        let take_second = roll >= threshold
            && order.len() > 1
            && matches!(
                (first_score, second_score),
                (Some(high), Some(low)) if low != 0 && i64::from(high) - i64::from(low) <= gap
            );
        if take_second {
            (Some(filtered[order[1]]), None)
        } else {
            (Some(filtered[order[0]]), None)
        }
    }

    /// Map the live `GameMode` to its `L17 斗转星移` mode string key.
    pub fn default_douzhuan_mode_key(&self) -> &'static str {
        match self.state.core.mode {
            GameMode::Identity { .. } => "identity",
            GameMode::Doudizhu { .. } => "doudizhu",
            GameMode::Boss { .. } => "boss",
            GameMode::Versus { .. } => "versus",
            GameMode::Kingdoms => "guozhan",
            GameMode::Kof { .. } => "single",
            GameMode::Prototype => "prototype",
        }
    }

    pub fn set_douzhuan_mode(&mut self, mode: &str) {
        self.state
            .core
            .vm_aux
            .flags
            .retain(|(p, k), _| !(*p == 0 && k.starts_with("l17_mode:")));
        if !mode.is_empty() {
            self.state
                .core
                .vm_aux
                .flags
                .insert((0u8, format!("l17_mode:{mode}")), 1);
        }
    }

    pub fn douzhuan_mode(&self) -> String {
        self.state
            .core
            .vm_aux
            .flags
            .iter()
            .find_map(|((p, k), &v)| {
                (*p == 0 && v != 0)
                    .then(|| k.strip_prefix("l17_mode:"))
                    .flatten()
                    .map(str::to_string)
            })
            .unwrap_or_else(|| self.default_douzhuan_mode_key().to_string())
    }

    pub fn set_douzhuan_miaoshu_enabled(&mut self, enabled: bool) {
        let key = (0u8, "l17_miaoshu_xiugai".to_string());
        if enabled {
            self.state.core.vm_aux.flags.insert(key, 1);
        } else {
            self.state.core.vm_aux.flags.remove(&key);
        }
    }

    pub fn is_douzhuan_miaoshu_enabled(&self) -> bool {
        self.state
            .core
            .vm_aux
            .flags
            .get(&(0u8, "l17_miaoshu_xiugai".to_string()))
            .copied()
            .unwrap_or(0)
            != 0
    }

    pub fn format_skill_description(&self, info: &str) -> String {
        if self.is_douzhuan_miaoshu_enabled() {
            rewrite_douzhuan_skill_text(info)
        } else {
            info.to_string()
        }
    }

    pub fn set_douzhuan_reload_guard(&mut self, forbid_reload: bool) {
        let key = (0u8, "l17_forbid_reload".to_string());
        if forbid_reload {
            self.state.core.vm_aux.flags.insert(key, 1);
        } else {
            self.state.core.vm_aux.flags.remove(&key);
        }
    }

    pub fn is_douzhuan_forbid_reload(&self) -> bool {
        self.state
            .core
            .vm_aux
            .flags
            .get(&(0u8, "l17_forbid_reload".to_string()))
            .copied()
            .unwrap_or(0)
            != 0
    }

    pub fn douzhuan_would_reload(&self, previous_tempmode: Option<&str>) -> bool {
        !self.is_douzhuan_forbid_reload()
            && previous_tempmode
                .map(str::trim)
                .is_some_and(|prev| !prev.is_empty() && prev != self.douzhuan_mode())
    }

    pub fn clear_douzhuan_l17_bans(&mut self) {
        let tracked: Vec<(String, bool, u32)> = self
            .state
            .core
            .vm_aux
            .flags
            .iter()
            .filter_map(|((p, k), &v)| {
                if *p != 0 {
                    return None;
                }
                if let Some(name) = k.strip_prefix("l17_jj_ban_unseen:") {
                    Some((name.to_string(), true, v))
                } else {
                    k.strip_prefix("l17_jj_ban_forbidai:")
                        .map(|name| (name.to_string(), false, v))
                }
            })
            .collect();
        for (name, unseen, marker) in tracked {
            if unseen {
                self.state
                    .core
                    .vm_aux
                    .flags
                    .remove(&(0u8, format!("l17_jj_ban_unseen:{name}")));
                if marker == 1 {
                    self.set_general_unseen_named(&name, false);
                }
            } else {
                self.state
                    .core
                    .vm_aux
                    .flags
                    .remove(&(0u8, format!("l17_jj_ban_forbidai:{name}")));
                if marker == 1 {
                    self.set_general_forbidai_named(&name, false);
                }
            }
        }
    }

    pub fn set_douzhuan_ban_named(&mut self, name: &str, forbid_unseen: bool) {
        let (tracker_prefix, ext_prefix) = if forbid_unseen {
            ("l17_jj_ban_unseen:", "ext_unseen:")
        } else {
            ("l17_jj_ban_forbidai:", "ext_forbidai:")
        };
        let existed_externally = self.get_named_flag(ext_prefix, name)
            && !self.get_named_flag(tracker_prefix, name);
        let marker = if existed_externally { 2 } else { 1 };
        self.state
            .core
            .vm_aux
            .flags
            .insert((0u8, format!("{tracker_prefix}{name}")), marker);
        if forbid_unseen {
            self.set_general_unseen_named(name, true);
        } else {
            self.set_general_forbidai_named(name, true);
        }
    }

    pub fn set_douzhuan_jinjiang_state(
        &mut self,
        gate: bool,
        forbid_unseen: bool,
        plan_index: Option<u8>,
    ) {
        let gate_key = (0u8, "l17_jj_gate".to_string());
        let type_key = (0u8, "l17_jj_forbid_unseen".to_string());
        let plan_key = (0u8, "l17_jj_plan_index".to_string());
        if gate {
            self.state.core.vm_aux.flags.insert(gate_key, 1);
        } else {
            self.state.core.vm_aux.flags.remove(&gate_key);
        }
        if forbid_unseen {
            self.state.core.vm_aux.flags.insert(type_key, 1);
        } else {
            self.state.core.vm_aux.flags.remove(&type_key);
        }
        match plan_index {
            Some(idx) if gate => {
                self.state
                    .core
                    .vm_aux
                    .flags
                    .insert(plan_key, u32::from(idx) + 1);
            }
            _ => {
                self.state.core.vm_aux.flags.remove(&plan_key);
            }
        }
    }

    pub fn douzhuan_jinjiang_gate(&self) -> bool {
        self.state
            .core
            .vm_aux
            .flags
            .get(&(0u8, "l17_jj_gate".to_string()))
            .copied()
            .unwrap_or(0)
            != 0
    }

    pub fn douzhuan_jinjiang_forbid_type(&self) -> &'static str {
        if self
            .state
            .core
            .vm_aux
            .flags
            .get(&(0u8, "l17_jj_forbid_unseen".to_string()))
            .copied()
            .unwrap_or(0)
            != 0
        {
            "unseen"
        } else {
            "forbidai"
        }
    }

    pub fn douzhuan_plan_index(&self) -> Option<u8> {
        if !self.douzhuan_jinjiang_gate() {
            return None;
        }
        self.state
            .core
            .vm_aux
            .flags
            .get(&(0u8, "l17_jj_plan_index".to_string()))
            .copied()
            .filter(|&v| v > 0)
            .map(|v| (v - 1) as u8)
    }
}

/// Rewrite `不大于`/`不小于` and swap `阴`/`阳` per `L17 miaoshuXiuGai` (`extension.js:1425-1437`).
pub fn rewrite_douzhuan_skill_text(text: &str) -> String {
    let swapped = text
        .replace("不大于", "temp")
        .replace("不小于", "≥")
        .replace("temp", "≤");
    swapped
        .replace('阴', "temp")
        .replace('阳', "阴")
        .replace("temp", "阳")
}
