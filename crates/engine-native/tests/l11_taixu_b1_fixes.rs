//! Batch L11-1 (`L11 太虚幻境`) authentic upstream regression suite:
//! - 32 cards (`cards.rs`: 14 non-weapon tricks/zhaoshu, `equips.rs`: 18+ equip/mount/treasure skills & sub-states)
//! - S01–S28 (`servants.rs`: `txhj_daiyuan*` + 24 servant combat skills)
//! - S29–S71 (`skills_a.rs`, `skills_b.rs`, `skills_c.rs`: 43 general skills)
//!
//! All tests drive real `GameCommand`/`GameAction` paths (`PlayCard`, `ActivateSkill`,
//! `NextPhase`, `Answer`), never `debug_execute_effect`, with upstream `file:line` refs.

#[path = "l11_taixu_b1/common.rs"]
mod common;

#[path = "l11_taixu_b1/cards.rs"]
mod cards;

#[path = "l11_taixu_b1/equips.rs"]
mod equips;

#[path = "l11_taixu_b1/servants.rs"]
mod servants;

#[path = "l11_taixu_b1/skills_a.rs"]
mod skills_a;

#[path = "l11_taixu_b1/skills_b.rs"]
mod skills_b;

#[path = "l11_taixu_b1/skills_c.rs"]
mod skills_c;

#[path = "l11_taixu_b1/round2_fixes.rs"]
mod round2_fixes;
