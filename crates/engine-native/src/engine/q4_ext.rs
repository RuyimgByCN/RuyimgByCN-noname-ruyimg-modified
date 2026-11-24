//! Batch Q4 runtime integration on `Engine`:
//! - `L26` (`Fool_liegong`, `Fool_skill`) live queries and candidate filtering
//! - `L15` (`手杀全扩`) + `U02`/`L03` general pool filters and `L17` (`AIqianghuaXuanjiang`)
//! - `U04` (`coin`) + `L21` (`martial`) match runtime flows persisted in `vm_aux.flags`

mod coin_martial;
mod fool;
mod pool_ai;

pub use coin_martial::{MartialConfigSpec, MartialHpRule, U04CoinMode, U04SettleInput};
pub use fool::FoolLiegongStyle;
pub use pool_ai::rewrite_douzhuan_skill_text;

fn decode_biased_i32(raw: u32, bias: i32) -> i32 {
    (raw as i64 - i64::from(bias)) as i32
}

fn encode_biased_i32(val: i32, bias: i32) -> u32 {
    (i64::from(val) + i64::from(bias)).max(0) as u32
}
