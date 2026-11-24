//! Batch Q4-Remaining (99 items across 9 sources) comprehensive real-event TDD suite.
//! Covers `U02` (15 generals + 2 modes), `L03` (8 generals), `U03` / `L04` (29 pile buckets
//! + `cardpile_arenaReady`), `L15` (3 pools + 8 configs), `L21` (martial + 6 configs),
//! `U04` (2 coin modes), `L17` (`AIqianghuaXuanjiang`), and `L26` (`Fool_liegong` + `Fool_skill`),
//! including full archive `save`/`encode`/`decode`/`restore_with_extensions` round-trips.

#[path = "q4_remaining/helpers.rs"]
mod helpers;

#[path = "q4_remaining/boss_play.rs"]
mod boss_play;

#[path = "q4_remaining/boss_play_extra.rs"]
mod boss_play_extra;

#[path = "q4_remaining/boss_play_round2.rs"]
mod boss_play_round2;

#[path = "q4_remaining/boss_play_round3.rs"]
mod boss_play_round3;

#[path = "q4_remaining/boss_play_round4.rs"]
mod boss_play_round4;

#[path = "q4_remaining/cardpile.rs"]
mod cardpile;

#[path = "q4_remaining/shousha_douzhuan.rs"]
mod shousha_douzhuan;

#[path = "q4_remaining/martial_coin_fool.rs"]
mod martial_coin_fool;
