//! ActivateSkill — active VM body or view-as conversion.

use super::play::{play_as_kind, play_as_kind_with_extra_damage_flags};
use crate::content::ViewAsDef;
use crate::effect::{run_effects, EffectError};
use crate::query::{query, QueryContext};
use crate::state::{Card, CardKind, EquipSlot, Phase, ZoneKind};
use crate::transaction::Transaction;
use engine_api::{CardDefId, CardId, MarkDefId, PlayerId, SkillId};
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

pub(crate) const DDD_FUSI_DEF: u32 = 1_632_230_116;
pub(crate) const DDD_FUSI_GLOBAL_DEF: u32 = 2_198_174_483;
pub(crate) const DDD_FENGZHENG_DEF: u32 = 986_583_704;
pub(crate) const DDD_FENGZHENG_GLOBAL_DEF: u32 = 1_253_882_292;

mod ddd_fengzheng_provider_exists;
pub(crate) use ddd_fengzheng_provider_exists::*;
mod validate_source_free_cost;
pub(crate) use validate_source_free_cost::*;
mod handle_view_as;
pub(crate) use handle_view_as::*;
