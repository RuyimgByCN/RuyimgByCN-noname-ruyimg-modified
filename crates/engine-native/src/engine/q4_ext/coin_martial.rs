use std::collections::BTreeMap;

use crate::engine::{Engine, RejectReason};
use crate::ids::PlayerId;
use crate::state::{apply_general_assembly, combine_double_hp, GameMode};

/// Game-mode branch for `U04` coin settlement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum U04CoinMode {
    Identity = 0,
    Guozhan = 1,
    Versus = 2,
    Chess = 3,
    Other = 4,
}

impl U04CoinMode {
    fn from_u32(v: u32) -> Self {
        match v {
            0 => Self::Identity,
            1 => Self::Guozhan,
            2 => Self::Versus,
            3 => Self::Chess,
            _ => Self::Other,
        }
    }
}

/// Settlement parameters for `U04` match completion.
#[derive(Debug, Clone, PartialEq)]
pub struct U04SettleInput {
    pub victory: bool,
    pub identity_zhu_or_zhong: bool,
    pub identity_fan: bool,
    pub identity_nei: bool,
    pub enhance_zhu: bool,
    pub guozhan_ye: bool,
    pub guozhan_population: u32,
    pub versus_friends: Option<u32>,
    pub chess_combat_extra: bool,
    pub additional_reward: i64,
    pub coin_coeff: Option<f64>,
    pub unit_random_override: Option<f64>,
}

/// `L21 武皇争霸` double-general HP rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MartialHpRule {
    Hejiansan = 0,
    Pingjun = 1,
    Zuidazhi = 2,
    Zuixiaozhi = 3,
    Zonghe = 4,
}

impl MartialHpRule {
    pub fn from_config_str(value: &str) -> Result<Self, &'static str> {
        match value {
            "hejiansan" => Ok(Self::Hejiansan),
            "pingjun" => Ok(Self::Pingjun),
            "zuidazhi" => Ok(Self::Zuidazhi),
            "zuixiaozhi" => Ok(Self::Zuixiaozhi),
            "zonghe" => Ok(Self::Zonghe),
            _ => Err("INVALID_DOUBLE_HP_RULE"),
        }
    }
}

/// Runtime configuration for `L21 武皇争霸` match setup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MartialConfigSpec {
    pub show_number: u8,
    pub battle_number: u8,
    pub hide_number: u8,
    pub double_character: bool,
    pub double_hp: MartialHpRule,
    pub free_choose: bool,
    pub change_choice: bool,
    pub auto_join: bool,
}

fn store_i64_flag(flags: &mut BTreeMap<(u8, String), u32>, key: &str, value: i64) {
    let raw = value as u64;
    flags.insert((0u8, format!("{key}_lo")), (raw & 0xFFFF_FFFF) as u32);
    flags.insert((0u8, format!("{key}_hi")), ((raw >> 32) & 0xFFFF_FFFF) as u32);
}

fn load_i64_flag(flags: &BTreeMap<(u8, String), u32>, key: &str) -> Option<i64> {
    let lo = flags.get(&(0u8, format!("{key}_lo"))).copied()?;
    let hi = flags.get(&(0u8, format!("{key}_hi"))).copied()?;
    Some((u64::from(lo) | (u64::from(hi) << 32)) as i64)
}

impl Engine {
    /// Initialize `U04` coin state on `Engine` (`_status.coin = 0`, persist & online gates).
    pub fn u04_coin_init(
        &mut self,
        initial_balance: i64,
        mode: U04CoinMode,
        chess_leader: bool,
        video: bool,
        online: bool,
    ) {
        let is_chess_leader = mode == U04CoinMode::Chess && chess_leader;
        let persist = !video && !online && !is_chess_leader;
        let match_active = !online && !is_chess_leader;
        let flags = &mut self.state.core.vm_aux.flags;
        flags.insert((0u8, "u04_coin_inited".to_string()), 1);
        flags.insert((0u8, "u04_coin_mode".to_string()), mode as u32);
        flags.insert((0u8, "u04_coin_persist".to_string()), u32::from(persist));
        flags.insert((0u8, "u04_coin_online".to_string()), u32::from(online));
        flags.insert((0u8, "u04_coin_match".to_string()), u32::from(match_active));
        flags.insert((0u8, "u04_coin_bet".to_string()), 0);
        flags.insert((0u8, "u04_coin_phase_started".to_string()), 0);
        store_i64_flag(flags, "u04_coin_balance", initial_balance);
        store_i64_flag(flags, "u04_coin_match_score", 0);
    }

    pub fn u04_coin_balance(&self) -> Option<i64> {
        let flags = &self.state.core.vm_aux.flags;
        (flags.get(&(0u8, "u04_coin_inited".to_string())).copied() == Some(1))
            .then(|| load_i64_flag(flags, "u04_coin_balance"))
            .flatten()
    }

    pub fn u04_match_coin(&self) -> Option<i64> {
        let flags = &self.state.core.vm_aux.flags;
        (flags.get(&(0u8, "u04_coin_match".to_string())).copied() == Some(1))
            .then(|| load_i64_flag(flags, "u04_coin_match_score"))
            .flatten()
    }

    pub fn u04_mark_phase_started(&mut self) {
        if self
            .state
            .core
            .vm_aux
            .flags
            .contains_key(&(0u8, "u04_coin_inited".to_string()))
        {
            self.state
                .core
                .vm_aux
                .flags
                .insert((0u8, "u04_coin_phase_started".to_string()), 1);
        }
    }

    /// Mutate persistent coin balance (`game.changeCoin`) when persistence gate is active.
    pub fn u04_change_coin(&mut self, delta: i64) -> Result<i64, &'static str> {
        let flags = &mut self.state.core.vm_aux.flags;
        if flags.get(&(0u8, "u04_coin_persist".to_string())).copied() != Some(1) {
            return Err("COIN_PERSIST_INACTIVE");
        }
        let next = load_i64_flag(flags, "u04_coin_balance")
            .unwrap_or(0)
            .saturating_add(delta);
        store_i64_flag(flags, "u04_coin_balance", next);
        Ok(next)
    }

    /// Place pre-turn -20 coin bet if online/phase gates allow.
    pub fn u04_place_bet(&mut self) -> Result<i64, &'static str> {
        let flags = &self.state.core.vm_aux.flags;
        if flags.get(&(0u8, "u04_coin_persist".to_string())).copied() != Some(1) {
            return Err("COIN_PERSIST_INACTIVE");
        }
        if flags.get(&(0u8, "u04_coin_online".to_string())).copied() == Some(1)
            || flags.get(&(0u8, "u04_coin_phase_started".to_string())).copied() == Some(1)
        {
            return Err("BET_CLOSED");
        }
        if flags.get(&(0u8, "u04_coin_bet".to_string())).copied() == Some(1) {
            return Err("ALREADY_BET");
        }
        self.state
            .core
            .vm_aux
            .flags
            .insert((0u8, "u04_coin_bet".to_string()), 1);
        self.u04_change_coin(-20)
    }

    /// Settle end-of-match coin payout using upstream `game.over` formula and deterministic RNG.
    pub fn u04_settle_game(&mut self, input: &U04SettleInput) -> Result<i64, &'static str> {
        if self
            .state
            .core
            .vm_aux
            .flags
            .get(&(0u8, "u04_coin_match".to_string()))
            .copied()
            != Some(1)
        {
            return Err("COIN_MATCH_INACTIVE");
        }
        let rng_unit = f64::from(self.state.core.rng.gen_range(1000)) / 1000.0;
        let unit = input.unit_random_override.unwrap_or(rng_unit).clamp(0.0, 1.0);
        let coeff = unit * 0.4 + 0.8;
        let mode = U04CoinMode::from_u32(
            self.state
                .core
                .vm_aux
                .flags
                .get(&(0u8, "u04_coin_mode".to_string()))
                .copied()
                .unwrap_or(4),
        );
        let seats = self.state.core.players.len() as f64;
        let alive = self.state.core.alive_count() as f64;
        let mut added = if input.victory {
            match mode {
                U04CoinMode::Identity => {
                    let base = if input.identity_zhu_or_zhong {
                        if input.enhance_zhu { 10.0 } else { 20.0 }
                    } else if input.identity_fan {
                        if input.enhance_zhu { 16.0 } else { 8.0 }
                    } else if input.identity_nei {
                        40.0
                    } else {
                        0.0
                    };
                    base * seats / 8.0
                }
                U04CoinMode::Guozhan => {
                    let base = if input.guozhan_ye {
                        8.0
                    } else if input.guozhan_population == 0 {
                        return Err("EMPTY_POPULATION");
                    } else {
                        5.0 / f64::from(input.guozhan_population)
                    };
                    base * seats
                }
                U04CoinMode::Versus => match input.versus_friends {
                    Some(friends) => 5.0 * (alive + f64::from(friends)),
                    None => 0.0,
                },
                U04CoinMode::Chess | U04CoinMode::Other => 10.0,
            }
        } else {
            10.0
        };
        if mode == U04CoinMode::Chess && input.chess_combat_extra {
            added = 2.0;
        }
        let bet = self
            .state
            .core
            .vm_aux
            .flags
            .get(&(0u8, "u04_coin_bet".to_string()))
            .copied()
            == Some(1);
        let mut pool = added * coeff;
        let mut bet_return = 0i64;
        if input.victory {
            pool += 20.0 + input.additional_reward as f64;
            if bet {
                pool += 10.0;
                bet_return = 20;
            }
        }
        if let Some(extra) = input.coin_coeff {
            if !extra.is_finite() {
                return Err("INVALID_COEFF");
            }
            pool *= extra;
        }
        let payout = pool.ceil() as i64;
        let flags = &mut self.state.core.vm_aux.flags;
        let match_total = load_i64_flag(flags, "u04_coin_match_score")
            .unwrap_or(0)
            .saturating_add(payout);
        store_i64_flag(flags, "u04_coin_match_score", match_total);
        if flags.get(&(0u8, "u04_coin_persist".to_string())).copied() == Some(1) {
            let bal = load_i64_flag(flags, "u04_coin_balance")
                .unwrap_or(0)
                .saturating_add(payout + bet_return);
            store_i64_flag(flags, "u04_coin_balance", bal);
        }
        Ok(payout)
    }

    /// Construct a real `L21 武皇争霸` 2-player KOF match with drafting pools,
    /// `double_character`/`double_hp`, and opening draw (`zhu=4, fan=5`).
    pub fn new_martial(
        seed: u64,
        pools: [Vec<u16>; 2],
        config: &MartialConfigSpec,
    ) -> Result<Self, &'static str> {
        let mut engine = Self::new_kof(seed, pools, [0, 0], 0);
        engine.configure_martial_match(config)?;
        Ok(engine)
    }

    /// Configure an active `GameMode::Kof` engine into `L21 武皇争霸` mode.
    pub fn configure_martial_match(&mut self, config: &MartialConfigSpec) -> Result<(), &'static str> {
        let mut pools = match &self.state.core.mode {
            GameMode::Kof { pools, .. } => pools.clone(),
            _ => return Err("MARTIAL_REQUIRES_KOF"),
        };
        if pools[0].is_empty() || pools[1].is_empty() {
            return Err("MARTIAL_EMPTY_POOL");
        }
        {
            let flags = &mut self.state.core.vm_aux.flags;
            flags.insert((0u8, "l21_martial_active".to_string()), 1);
            flags.insert((0u8, "l21_show_number".to_string()), u32::from(config.show_number));
            flags.insert((0u8, "l21_battle_number".to_string()), u32::from(config.battle_number));
            flags.insert((0u8, "l21_hide_number".to_string()), u32::from(config.hide_number));
            flags.insert((0u8, "l21_double_char".to_string()), u32::from(config.double_character));
            flags.insert((0u8, "l21_double_hp".to_string()), config.double_hp as u32);
            flags.insert((0u8, "l21_free_choose".to_string()), u32::from(config.free_choose));
            flags.insert((0u8, "l21_change_choice".to_string()), u32::from(config.change_choice));
            flags.insert((0u8, "l21_auto_join".to_string()), u32::from(config.auto_join));
        }

        for seat in [0u8, 1u8] {
            let si = seat as usize;
            let g1 = pools[si].remove(0);
            let g2 = if config.double_character && config.auto_join && !pools[si].is_empty() {
                Some(pools[si].remove(0))
            } else {
                None
            };
            self.mount_martial_generals(seat, g1, g2, config.double_hp)?;
        }
        if let GameMode::Kof { pools: ref mut p, .. } = self.state.core.mode {
            *p = pools;
        }
        // Seat 0 = zhu (4 cards), Seat 1 = fan (5 cards).
        while self.state.core.player(1).hand.len() < 5 {
            if let Some(c) = self.state.core.deck.pop() {
                self.state.core.player_mut(1).hand.push(c);
            } else {
                break;
            }
        }
        Ok(())
    }

    fn mount_martial_generals(
        &mut self,
        seat: PlayerId,
        g1: u16,
        g2: Option<u16>,
        hp_rule: MartialHpRule,
    ) -> Result<(), &'static str> {
        let Some(meta1) = self.catalog.general_meta(g1) else {
            return Err("UNKNOWN_GENERAL");
        };
        if let Some(sub) = g2 {
            if self.catalog.general_meta(sub).is_none() {
                return Err("UNKNOWN_DEPUTY_GENERAL");
            }
        }
        let catalog = self.catalog.clone();
        let removed: Vec<_> = self
            .state
            .core
            .skills
            .keys()
            .filter(|(o, _)| *o == seat)
            .map(|(_, d)| *d)
            .collect();
        for d in removed {
            self.state
                .core
                .remove_skill_and_r05_mark(&catalog, seat, d);
        }
        {
            let p = self.state.core.player_mut(seat);
            p.general_slots = [Some(g1), g2];
            p.revealed_generals = [true, g2.is_some()];
            p.alive = true;
            p.dying = false;
        }
        apply_general_assembly(&mut self.state.core, &catalog, seat, 0);
        if let Some(sub) = g2 {
            apply_general_assembly(&mut self.state.core, &catalog, seat, 1);
            let hp1 = meta1.3;
            let hp2 = catalog.general_meta(sub).map(|m| m.3).unwrap_or(4);
            let combined = combine_double_hp(hp_rule as u32, hp1, hp2);
            let p = self.state.core.player_mut(seat);
            p.max_hp = combined;
            p.hp = combined as i32;
        } else {
            let p = self.state.core.player_mut(seat);
            p.hp = p.max_hp as i32;
        }
        crate::resolution::sync_trigger_owners(&self.state.core, &mut self.state.resolution);
        Ok(())
    }

    /// `L21` `free_choose` action on `seat` when enabled in match config.
    pub fn martial_free_choose(&mut self, seat: PlayerId, general: u16) -> Result<(), RejectReason> {
        let allowed = self
            .state
            .core
            .vm_aux
            .flags
            .get(&(0u8, "l21_free_choose".to_string()))
            .copied()
            .unwrap_or(0)
            != 0;
        if !allowed || self.is_general_unseen(general) {
            return Err(RejectReason::WrongPhase);
        }
        self.assign_general(seat, general)
    }

    /// `L21` `change_choice` swap: swap active lead general with `pool_index` backup when enabled.
    pub fn martial_change_choice(
        &mut self,
        seat: PlayerId,
        pool_index: usize,
    ) -> Result<u16, &'static str> {
        let flags = &self.state.core.vm_aux.flags;
        if flags.get(&(0u8, "l21_change_choice".to_string())).copied().unwrap_or(0) == 0 {
            return Err("CHANGE_CHOICE_DISABLED");
        }
        let hp_rule = match flags.get(&(0u8, "l21_double_hp".to_string())).copied().unwrap_or(0) {
            1 => MartialHpRule::Pingjun,
            2 => MartialHpRule::Zuidazhi,
            3 => MartialHpRule::Zuixiaozhi,
            4 => MartialHpRule::Zonghe,
            _ => MartialHpRule::Hejiansan,
        };
        let old_lead = self.state.core.player(seat).general_slots[0].ok_or("NO_ACTIVE_GENERAL")?;
        let old_sub = self.state.core.player(seat).general_slots[1];
        let next_lead = match &mut self.state.core.mode {
            GameMode::Kof { pools, .. } => {
                let pool = pools.get_mut(seat as usize).ok_or("BAD_SEAT")?;
                if pool_index >= pool.len() {
                    return Err("BAD_POOL_INDEX");
                }
                let picked = pool[pool_index];
                pool[pool_index] = old_lead;
                picked
            }
            _ => return Err("MARTIAL_REQUIRES_KOF"),
        };
        self.mount_martial_generals(seat, next_lead, old_sub, hp_rule)?;
        Ok(next_lead)
    }
}
