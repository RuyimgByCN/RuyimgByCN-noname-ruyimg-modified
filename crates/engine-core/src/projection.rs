//! Server-side observer projection (architecture §5.3 / §14.5).
//!
//! Full GameState → ObserverSnapshot. Clients never receive full authority state.

use crate::hash::canonical_state_hash;
use crate::query::{observer_foreign_hand_faces_visible, observer_hidden_identities_visible};
use crate::state::{CardKind, ContinuationKind, GameState, Identity, ZoneKind};
use engine_api::{
    CardKindTag, CardView, CardZoneTag, IdentityTag, ObserverKind, ObserverSnapshot, PlayerId,
    SeatView, SkillView,
};

/// Project authority state for one observer.
pub fn project_observer(state: &GameState, observer: ObserverKind) -> ObserverSnapshot {
    let self_id = match observer {
        ObserverKind::Player(id) => Some(id),
        ObserverKind::ReplayExporter {
            privacy: engine_api::ReplayPrivacy::Player(id),
        } => Some(id),
        _ => None,
    };
    let full = matches!(
        observer,
        ObserverKind::Referee
            | ObserverKind::ReplayExporter {
                privacy: engine_api::ReplayPrivacy::Full
            }
    );
    let foreign_hand_faces_visible = self_id
        .map(|observer| observer_foreign_hand_faces_visible(state, observer))
        .unwrap_or(false);
    let hidden_identities_visible = self_id
        .map(|observer| observer_hidden_identities_visible(state, observer))
        .unwrap_or(false);
    let private_choice_cards = self_id
        .map(|observer| private_choice_card_ids(state, observer))
        .unwrap_or_default();

    let mut seats: Vec<SeatView> = state
        .players
        .sorted_live()
        .into_iter()
        .map(|p| {
            let identity = if full
                || hidden_identities_visible
                || self_id == Some(p.id)
                || p.identity == Identity::Lord
                || !p.alive
            {
                // Lord always public; dead identities revealed; self sees self
                Some(identity_tag(p.identity))
            } else {
                None
            };
            SeatView {
                player: p.id,
                character_def: p.character_def,
                generals: [None, None],
                seat: p.seat,
                hp: p.hp,
                max_hp: p.max_hp,
                alive: p.alive,
                turned_over: p.turned_over,
                linked: false,
                hand_count: state.hand_count(p.id),
                identity,
                skills: p
                    .skills
                    .iter()
                    .filter_map(|skill_id| state.skills.get(*skill_id))
                    .map(|skill| SkillView {
                        id: skill.id,
                        def: skill.def.raw(),
                        enabled: state.skill_effectively_enabled(skill),
                        use_count: skill.use_count,
                        active: state.content.first_active_for_skill(skill.def).is_some(),
                        view_as: !state.content.view_as_defs_for_skill(skill.def).is_empty(),
                        locked: state
                            .content
                            .skill_metadata(skill.def)
                            .map(|metadata| metadata.locked)
                            .unwrap_or(false),
                    })
                    .collect(),
                r05_effects: Vec::new(),
            }
        })
        .collect();
    seats.sort_by_key(|s| s.seat.raw());

    let mut known_cards = Vec::new();
    for c in state.cards.sorted_live() {
        let zone = state
            .zones
            .get(c.zone)
            .expect("live card must reference a live zone");
        let deliberately_hidden = state.face_down_hand_cards.contains(&c.id);
        let visible = full
            || c.face_up
            || private_choice_cards.contains(&c.id.raw())
            || c.owner == self_id.unwrap_or(PlayerId::NONE) && !deliberately_hidden
            || self_id.is_some()
                && zone.kind == ZoneKind::Hand
                && crate::query::pothaoshi_grant(
                    state,
                    zone.owner,
                    self_id.unwrap_or(PlayerId::NONE),
                )
                .is_some()
                && !deliberately_hidden
            || foreign_hand_faces_visible
                && self_id.is_some()
                && zone.kind == ZoneKind::Hand
                && zone.owner != self_id.unwrap_or(PlayerId::NONE)
                && !deliberately_hidden
            || matches!(
                c.kind,
                CardKind::Weapon
                    | CardKind::Armor
                    | CardKind::HorseOff
                    | CardKind::HorseDef
                    | CardKind::Treasure
            ) && zone.kind == ZoneKind::Equip
            || zone.kind == ZoneKind::DiscardPile;

        if visible {
            known_cards.push(CardView {
                id: c.id,
                def: Some(c.def.raw()),
                suit: Some(c.suit),
                rank: Some(c.rank),
                owner: if c.owner.is_none() {
                    None
                } else {
                    Some(c.owner)
                },
                zone: zone_tag(zone.kind),
                kind: kind_tag(c.kind),
            });
        } else if zone.kind == ZoneKind::Expansion {
            known_cards.push(CardView {
                id: c.id,
                def: None,
                suit: None,
                rank: None,
                owner: Some(c.owner),
                zone: CardZoneTag::Expansion,
                kind: CardKindTag::Other,
            });
        } else if c.owner == self_id.unwrap_or(PlayerId::NONE) {
            // own hand already covered by owner check
        } else if self_id.is_some()
            && zone.kind == ZoneKind::Hand
            && zone.owner != self_id.unwrap_or(PlayerId::NONE)
        {
            // Other hands: only id + zone owner count, hide face
            // Represent as hidden card with id only for tracking moves (optional).
            // Architecture: hand count is on seat; omit face-down foreign hands.
        }
    }

    ObserverSnapshot {
        revision: state.revision,
        state_hash: canonical_state_hash(state),
        observer,
        round: state.turn.round,
        current_player: state.turn.current,
        phase: state.turn.phase.to_tag(),
        seats,
        known_cards,
        outcome: state.outcome.clone(),
    }
}

fn private_choice_card_ids(state: &GameState, observer: PlayerId) -> Vec<u32> {
    let resolution = &state.resolution;
    if resolution.pending_choice_id == 0 || resolution.pending_observer != observer.raw() {
        return Vec::new();
    }
    let payload = &resolution.continuation_payload;
    if resolution.continuation_kind == ContinuationKind::ResolveXinfuGuolun
        && payload.first().copied() == Some(1)
    {
        return payload.get(4).copied().into_iter().collect();
    }
    if resolution.continuation_kind == ContinuationKind::ChooseMoveTargetEquipmentOrDraw
        && payload.first().copied() == Some(5)
    {
        let count = payload.get(4).copied().unwrap_or_default() as usize;
        return payload.get(5..5 + count).unwrap_or_default().to_vec();
    }
    if resolution.continuation_kind == ContinuationKind::ResolveBeizhu
        && payload.first().copied() == Some(0)
    {
        let count = payload.get(4).copied().unwrap_or_default() as usize;
        return payload.get(5..5 + count).unwrap_or_default().to_vec();
    }
    let (count_index, cards_start) = match resolution.continuation_kind {
        ContinuationKind::ReorderDrawPile
        | ContinuationKind::ReorderDrawPileFourSplitTwoThenDraw => (1, 2),
        ContinuationKind::ChooseGainTopCardsBySuit => (2, 3),
        ContinuationKind::ChooseDrltPoxiCards => (4, 5),
        ContinuationKind::ChooseJilueJizhiDiscard => (0, 0),
        _ => return Vec::new(),
    };
    let Some(count) = payload
        .get(count_index)
        .copied()
        .map(|count| count as usize)
    else {
        return Vec::new();
    };
    payload
        .get(cards_start..cards_start + count)
        .unwrap_or_default()
        .to_vec()
}

fn zone_tag(zone: ZoneKind) -> CardZoneTag {
    match zone {
        ZoneKind::DrawPile => CardZoneTag::DrawPile,
        ZoneKind::DiscardPile => CardZoneTag::DiscardPile,
        ZoneKind::Hand => CardZoneTag::Hand,
        ZoneKind::Equip => CardZoneTag::Equip,
        ZoneKind::Judge => CardZoneTag::Judge,
        ZoneKind::Processing => CardZoneTag::Processing,
        ZoneKind::Removed => CardZoneTag::Removed,
        ZoneKind::Expansion => CardZoneTag::Expansion,
    }
}

fn kind_tag(kind: CardKind) -> CardKindTag {
    match kind {
        CardKind::Ying => CardKindTag::Other,
        CardKind::Sha => CardKindTag::Sha,
        CardKind::Tao => CardKindTag::Tao,
        CardKind::Shan => CardKindTag::Shan,
        CardKind::Jiu => CardKindTag::Jiu,
        CardKind::ShaFire => CardKindTag::ShaFire,
        CardKind::ShaThunder => CardKindTag::ShaThunder,
        CardKind::Weapon => CardKindTag::Weapon,
        CardKind::Armor => CardKindTag::Armor,
        CardKind::HorseOff => CardKindTag::HorseOff,
        CardKind::HorseDef => CardKindTag::HorseDef,
        CardKind::Treasure => CardKindTag::Treasure,
        CardKind::TrickWuzhong => CardKindTag::TrickWuzhong,
        CardKind::TrickGuohe => CardKindTag::TrickGuohe,
        CardKind::TrickShunshou => CardKindTag::TrickShunshou,
        CardKind::TrickJuedou => CardKindTag::TrickJuedou,
        CardKind::TrickNanman => CardKindTag::TrickNanman,
        CardKind::TrickWanjian => CardKindTag::TrickWanjian,
        CardKind::TrickTaoyuan => CardKindTag::TrickTaoyuan,
        CardKind::TrickWugu => CardKindTag::TrickWugu,
        CardKind::TrickLe => CardKindTag::TrickLe,
        CardKind::TrickBingliang => CardKindTag::TrickBingliang,
        CardKind::TrickShandian => CardKindTag::TrickShandian,
        CardKind::TrickJiedao => CardKindTag::TrickJiedao,
        CardKind::TrickTiesuo => CardKindTag::TrickTiesuo,
        CardKind::TrickHuogong => CardKindTag::TrickHuogong,
        CardKind::TrickWuxie => CardKindTag::TrickWuxie,
        CardKind::TrickYuanjiao => CardKindTag::TrickYuanjiao,
        CardKind::TrickTiaojiyanmei => CardKindTag::TrickTiaojiyanmei,
        CardKind::TrickMantianguohai => CardKindTag::TrickMantianguohai,
        CardKind::TrickYoudishenru => CardKindTag::TrickYoudishenru,
        CardKind::TrickDiaobingqianjiang => CardKindTag::TrickDiaobingqianjiang,
        CardKind::TrickWangmeizhike => CardKindTag::TrickWangmeizhike,
        CardKind::TrickYiyi => CardKindTag::TrickYiyi,
        CardKind::TrickShuiyanqijunx => CardKindTag::TrickShuiyanqijunx,
        CardKind::TrickLulitongxin => CardKindTag::TrickLulitongxin,
        CardKind::TrickKefuzhongyuan => CardKindTag::TrickKefuzhongyuan,
        CardKind::TrickGuguoanbang => CardKindTag::TrickGuguoanbang,
        CardKind::TrickHuoshaolianying => CardKindTag::TrickHuoshaolianying,
        CardKind::TrickLianjunshengyan => CardKindTag::TrickLianjunshengyan,
        CardKind::TrickWenheluanwu => CardKindTag::TrickWenheluanwu,
        CardKind::TrickHaolingtianxia => CardKindTag::TrickHaolingtianxia,
        CardKind::TrickChiling => CardKindTag::TrickChiling,
        CardKind::TrickChuqibuyi => CardKindTag::TrickChuqibuyi,
        CardKind::Other => CardKindTag::Other,
    }
}

fn identity_tag(i: Identity) -> IdentityTag {
    match i {
        Identity::Lord => IdentityTag::Lord,
        Identity::Loyalist => IdentityTag::Loyalist,
        Identity::Rebel => IdentityTag::Rebel,
        Identity::Renegade => IdentityTag::Renegade,
    }
}

/// Convenience: project for a seated player.
pub fn project_player(state: &GameState, player: PlayerId) -> ObserverSnapshot {
    project_observer(state, ObserverKind::Player(player))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::bootstrap_match;
    use crate::transaction::Transaction;

    #[test]
    fn other_player_does_not_see_hand_faces() {
        let mut tx = Transaction::begin(GameState::empty());
        bootstrap_match(&mut tx, 2, 1, "identity-standard").unwrap();
        let state = tx.state();
        let p1 = PlayerId::new(1);
        let p2 = PlayerId::new(2);
        let view = project_player(state, p1);
        assert!(view.seats.iter().all(|seat| {
            state
                .players
                .get(seat.player)
                .is_some_and(|player| seat.character_def == player.character_def)
        }));
        // p1 hand cards visible
        let p1_hand = state.hand_cards(p1);
        for cid in &p1_hand {
            assert!(
                view.known_cards
                    .iter()
                    .any(|c| c.id == *cid && c.def.is_some() && c.zone == CardZoneTag::Hand),
                "own hand should be known"
            );
        }
        // p2 hand faces hidden
        let p2_hand = state.hand_cards(p2);
        for cid in &p2_hand {
            assert!(
                !view
                    .known_cards
                    .iter()
                    .any(|c| c.id == *cid && c.def.is_some()),
                "foreign hand faces must be hidden"
            );
        }
        // Lord identity visible
        assert!(view
            .seats
            .iter()
            .any(|s| s.identity == Some(IdentityTag::Lord)));
    }
}
