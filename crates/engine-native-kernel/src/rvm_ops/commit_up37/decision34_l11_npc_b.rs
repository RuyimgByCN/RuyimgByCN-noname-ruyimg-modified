//! `L11` (`太虚幻境`) cross-package NPC fallback resolver (`decision34_l11_npc_b.rs`).
//!
//! Remaining entry: `"txhj_tunshi"` (part of `L11_TAIXU_OWN_NPC_SKILLS`, not Part B `[80..164]`).
//! All Part B `[80..164]` skills now execute natively via `HostBehavior::L11CrossNpcB`.

use super::commit_up37::filtered_target;
use super::*;

pub(super) fn try_resolve_l11_npc_skill_b(
    ctx: &mut Ctx<'_>,
    skill_name: &str,
    owner: PlayerId,
) -> bool {
    match skill_name {
        "txhj_tunshi" => {
            // `extension.js:811-838`: `phaseBegin`（准备阶段）且敌方手牌数 > 自己手牌数时造成 1 点伤害
            if ctx.core.turn.phase == crate::event::Phase::Prepare {
                if let Some(t) = filtered_target(ctx, owner, true, |core, p| {
                    core.player(p).hand.len() > core.player(owner).hand.len()
                }) {
                    ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                        Some(owner),
                        t,
                        1,
                        DamageNature::Normal,
                        None,
                    )));
                }
            }
            true
        }
        _ => false,
    }
}
