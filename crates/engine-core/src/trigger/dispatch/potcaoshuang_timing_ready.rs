//! Trigger operations for `filter_match` and related helpers.

use super::*;

pub(crate) fn potcaoshuang_timing_ready(
    state: &crate::state::GameState,
    active: &ActiveTrigger,
    timing: u8,
) -> bool {
    state.skills.get(active.skill_id).is_some_and(|skill| {
        matches!(
            skill.def.raw(),
            crate::state::POTDIANYI_DEF | crate::state::POTSHEQUAN_DEF
        ) && {
            let (assigned, triggered, _) = skill.potcaoshuang_state(state.turn.round);
            assigned & timing != 0 && triggered & timing == 0
        }
    })
}

pub(crate) fn potjianzhuan_timing_available(
    state: &crate::state::GameState,
    active: &ActiveTrigger,
    timing: u8,
) -> bool {
    let Some(skill) = state.skills.get(active.skill_id) else {
        return false;
    };
    if skill.def.raw() != crate::state::POTJIANZHUAN_DEF || skill.lifetime_fire_count != 0 {
        return false;
    }
    let assigned = state
        .players
        .get(active.owner)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .filter_map(|id| state.skills.get(*id))
        .filter(|instance| {
            matches!(
                instance.def.raw(),
                crate::state::POTDIANYI_DEF | crate::state::POTSHEQUAN_DEF
            ) && state.skill_effectively_enabled(instance)
        })
        .fold(0, |mask, instance| {
            mask | instance.potcaoshuang_state(state.turn.round).0
        });
    assigned != 0b111 && assigned & timing == 0 || assigned == 0b111
}

pub(crate) fn mbfeijing_record_matches(
    state: &crate::state::GameState,
    owner: PlayerId,
    card: CardId,
    target: Option<PlayerId>,
) -> bool {
    state
        .players
        .get(owner)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .filter_map(|skill| state.skills.get(*skill))
        .any(|skill| {
            let blob = &skill.state_blob;
            blob.get(..4) == Some(b"FJG1")
                && blob.len() >= 12
                && (blob.len() - 12) % 8 == 0
                && u32::from_le_bytes(blob[4..8].try_into().unwrap()) == card.raw()
                && target.is_none_or(|target| {
                    blob[12..].chunks_exact(8).any(|pair| {
                        u32::from_le_bytes(pair[..4].try_into().unwrap()) == target.raw()
                    })
                })
        })
}

pub(crate) fn xicha_state_has_source(blob: &[u8], source: PlayerId) -> bool {
    if blob.get(..4) != Some(b"XCH1") {
        return false;
    }
    let mut cursor = 4;
    while cursor + 5 <= blob.len() {
        let saved = PlayerId::new(u32::from_le_bytes(
            blob[cursor..cursor + 4].try_into().unwrap(),
        ));
        let count = blob[cursor + 4] as usize;
        cursor += 5;
        let bytes = count.saturating_mul(4);
        if cursor + bytes > blob.len() {
            return false;
        }
        if saved == source {
            return true;
        }
        cursor += bytes;
    }
    false
}

pub(crate) fn same_sex(left: &str, right: &str) -> bool {
    !matches!(left, "" | "unknown")
        && !matches!(right, "" | "unknown")
        && (left == "double" || right == "double" || left == right)
}

pub(crate) fn same_logical_card_name(
    previous_kind: CardKind,
    previous_def: u32,
    current_kind: CardKind,
    current_def: u32,
) -> bool {
    let canonical = |kind: CardKind| {
        if kind.is_sha() {
            CardKind::Sha
        } else {
            kind
        }
    };
    let previous_kind = canonical(previous_kind);
    let current_kind = canonical(current_kind);
    previous_kind == current_kind
        && (!previous_kind.is_equip() && previous_kind != CardKind::Other
            || previous_def == current_def)
}

pub(crate) fn turn_limit(filter: &TriggerFilter) -> Option<u32> {
    match filter {
        TriggerFilter::OncePerTurn
        | TriggerFilter::OwnerAliveEventPlayerPlayPhaseSameSuitAtLeastTwoOncePerTurn
        | TriggerFilter::OwnerSourceOncePerTurn
        | TriggerFilter::OwnerIsEventPlayerOncePerTurn
        | TriggerFilter::OwnerIsEventPlayerSourceAliveOncePerTurn
        | TriggerFilter::OwnerTargetedByOtherShaOncePerTurn
        | TriggerFilter::OwnerTargetedByOtherCardOncePerTurn
        | TriggerFilter::OwnerAliveNotEventPlayerOncePerTurn
        | TriggerFilter::OwnerTargetedByShaOrJuedouDamagedHpLeSourceOnce
        | TriggerFilter::OwnerAliveFirstCardsMovedToHand
        | TriggerFilter::OwnerAliveOtherDiscardedBasicOutsideTurnOncePerTurn
        | TriggerFilter::OwnerAliveOtherShanOutsideTurnGroupOncePerTurn(_)
        | TriggerFilter::OwnerAliveOtherSourceShaResolvedNotTargetGainableOncePerTurn
        | TriggerFilter::OwnerSourceDamageCardOncePerTurn
        | TriggerFilter::OwnerSourceOtherDamageBeforeOncePerTurn
        | TriggerFilter::OwnerSourceTargetEarlierSeatHandBelowTargetMaxHpOncePerTurn
        | TriggerFilter::OwnerSourceOrTargetedClubLinkedExistsOncePerTurn
        | TriggerFilter::OwnerGainsOutsideTurnMarkBelowAliveOncePerTurn(_)
        | TriggerFilter::OwnerAliveEventPlayerOutsideTurnFromHandEmptyOncePerTurn
        | TriggerFilter::OwnerSourceDamageTargetDistanceAtMostOneEmptyEquipSlotUnused
        | TriggerFilter::OwnerFirstTransformedShaUsedThisTurn
        | TriggerFilter::OwnerSourceShaTargetTwzhenxiAvailableOncePerTurn
        | TriggerFilter::OwnerSourceFirstTargetDamageCardOncePerTurn
        | TriggerFilter::OwnerSourceDamageTargetMarkPositiveOnce(_) => Some(1),
        TriggerFilter::OwnerAliveSourceBasicDddFushiAvailable => Some(1),
        TriggerFilter::OwnerSourceCardResolvedHasOtherTargetDcsbyingmouOncePerTurn => Some(1),
        TriggerFilter::OwnerSourceWoundedExistsOncePerTurn => Some(1),
        TriggerFilter::OwnerLostHandOwnerlessShaNotUseOncePerTurn => Some(1),
        TriggerFilter::OwnerAliveDamageBeforeTargetDistanceLeOneOncePerTurn => Some(1),
        TriggerFilter::OwnerAliveNeighborGainedCardsOutsideOwnDrawOncePerTurn => Some(1),
        TriggerFilter::OwnerGainedCardsPotduzuoOncePerTurn => Some(1),
        TriggerFilter::OwnerSourceOrTargetHandCountDiffersOncePerTurn => Some(1),
        TriggerFilter::OwnerTargetedSourceHpHandDifferOncePerTurn => Some(1),
        TriggerFilter::OwnerSourceTrickResolvedOwnPlayPhaseOncePerTurn => Some(1),
        TriggerFilter::OwnerAliveCurrentOrTurnFiresBelowHp
        | TriggerFilter::OwnerIsEventPlayerFirstDamagePerTurn
        | TriggerFilter::OwnerIsEventPlayerSecondDamagePerTurn
        | TriggerFilter::OwnerIsEventPlayerAfterFirstDamagePerTurn => Some(u32::MAX),
        TriggerFilter::OwnerAliveOtherWeiRecoveryOwnPhaseOncePerTurn => Some(1),
        TriggerFilter::OwnerSourceOtherTwicePerTurn
        | TriggerFilter::OwnerAliveOtherSourceShaTwicePerTurn
        | TriggerFilter::OwnerFromHandEmptyOrDyingMbsijianTwicePerTurn
        | TriggerFilter::OwnerIsTargetCanMoveFieldCardMbyuanmoTwicePerTurn
        | TriggerFilter::OwnerSourceFirstSingleTargetShaMbfeijingTwicePerTurn => Some(2),
        _ => None,
    }
}

/// 整肃三技能中该角色未有效拥有的数量（ABI 432）。
pub(crate) fn zhengsu_missing_count(
    state: &crate::state::GameState,
    player: engine_api::PlayerId,
    defs: &[u32; 3],
) -> u32 {
    defs.iter()
        .filter(|def| {
            !state.skills.sorted_live().iter().any(|instance| {
                instance.owner == player
                    && instance.def.raw() == **def
                    && state.skill_effectively_enabled(instance)
            })
        })
        .count() as u32
}

pub(crate) fn play_phase_limit(filter: &TriggerFilter) -> Option<u32> {
    match filter {
        TriggerFilter::OwnerSourcePlayShaOtherAliveOncePerPlayPhase
        | TriggerFilter::OwnerDiscardedHasHandOncePerPhase
        | TriggerFilter::OwnerAliveLostHandThisTurn
        | TriggerFilter::OwnerSourceFirstShaThisPlayPhase => Some(1),
        TriggerFilter::OwnerSourceCardHasOtherTargetCurrentPlayPhaseFirstTwo => Some(2),
        TriggerFilter::OwnerSourceBoundCategoryFirstThreePlayPhase => Some(3),
        _ => None,
    }
}

pub(crate) fn once_per_round(filter: &TriggerFilter) -> bool {
    matches!(
        filter,
        TriggerFilter::OwnerAliveOtherDyingOncePerRound
            | TriggerFilter::OwnerAliveSourceAliveOncePerRound
            | TriggerFilter::OwnerAliveSourceAliveEventPlayerDyingOncePerRound
            | TriggerFilter::OwnerAliveOncePerRound
            | TriggerFilter::OwnerAliveEventPlayerHandNonemptyOncePerRound
            | TriggerFilter::OwnerIsEventPlayerDiscardableHandEquipGteTwoOncePerRound
            | TriggerFilter::OwnerIsEventPlayerDiscardableHandEquipOncePerRound
            | TriggerFilter::OwnerAlivePotdingpinAvailableOncePerRound
            | TriggerFilter::OwnerAliveOtherQunPlayPhaseCanGiveOwner
    )
}

pub(crate) fn lifetime_limit(filter: &TriggerFilter) -> Option<u32> {
    matches!(
        filter,
        TriggerFilter::OwnerIsEventPlayerNoEnabledEquipSlotOncePerGame
            | TriggerFilter::OwnerAliveOtherDyingHandNonemptyOncePerGame
            | TriggerFilter::OwnerDyingOtherHandEquipNonemptyOncePerGame
            | TriggerFilter::OwnerIsEventPlayerOtherHigherHandOncePerGame
            | TriggerFilter::OwnerPhasePlayEndPotguanshaAvailableOncePerGame
            | TriggerFilter::OwnerDyingZhupeilanOncePerGame
            | TriggerFilter::OwnerAliveDddJiexingRecoverOncePerGame
            | TriggerFilter::OwnerAliveDddJiexingEquipOncePerGame
    )
    .then_some(1)
}

pub(crate) fn native_handler_effects(
    active: &ActiveTrigger,
    handler: &TriggerHandler,
) -> Vec<EffectRequest> {
    match handler {
        TriggerHandler::DrawSelf { count } => vec![EffectRequest::DrawCardsFromSkill {
            player: active.owner,
            count: *count,
            skill: active.skill_id,
        }],
        TriggerHandler::EmitNote { code } => vec![EffectRequest::EmitRuleEvent {
            code: *code,
            payload: vec![active.skill_id.raw()],
        }],
        TriggerHandler::RecoverSelf { amount } => vec![EffectRequest::RecoverHp {
            source: active.owner,
            player: active.owner,
            amount: *amount,
        }],
        TriggerHandler::RunVm { .. } => Vec::new(),
    }
}

pub(crate) fn timing_from_u16(v: u16) -> TimingBand {
    match v {
        0 => TimingBand::FirstDo,
        200 => TimingBand::LastDo,
        _ => TimingBand::Normal,
    }
}

/// Decrement trigger budget once per planned trigger batch item that is a "fire".
pub fn spend_trigger_budget(tx: &mut Transaction, n: u32) -> Result<(), EffectError> {
    let b = tx.state().resolution.trigger_budget;
    if b < n {
        return Err(EffectError::BudgetExceeded);
    }
    tx.state_mut().resolution.trigger_budget = b - n;
    Ok(())
}
