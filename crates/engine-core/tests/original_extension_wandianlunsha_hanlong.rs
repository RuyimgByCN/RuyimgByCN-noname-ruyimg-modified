//! Golden for archived extension 玩点论杀 韩龙【伺机/刺酋】（ABI 609）。

use engine_api::{CardDefId, CardId, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SIJI: u32 = 2_867_665_051;
const CIQIU: u32 = 3_777_188_862;
const CIQIU_DYING: u32 = 3_613_978_472;
const SOURCE: &str = include_str!("../../../content/original/ported/wandianlunsha/hanlong.skill");

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: ZoneKind,
    kind: CardKind,
    serial: u32,
) -> CardId {
    let zone_id = tx.state().zones.find(zone, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_090 + serial),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: zone_id,
        owner,
        face_up: zone != ZoneKind::Hand,
    });
    tx.state_mut()
        .zones
        .get_mut(zone_id)
        .unwrap()
        .cards
        .push(card);
    card
}

fn skill(tx: &Transaction, owner: PlayerId, def: u32) -> engine_api::SkillId {
    tx.state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

#[test]
fn siji_draws_twice_discarded_sha_count_and_ciqiu_kills_lethal_healthy_target() {
    let package = content_compiler::compile_source_named("wandianlunsha-hanlong", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(SIJI),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(CIQIU),
            },
        ],
    )
    .unwrap();

    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_091),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: discard,
        owner: PlayerId::NONE,
        face_up: true,
    });
    let shan = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_092),
        kind: CardKind::Shan,
        suit: Suit::Heart,
        rank: 8,
        weapon_range: 0,
        zone: discard,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .extend([sha, shan]);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::DISCARD_PHASE_LOST_BASE, 1);
    tx.state_mut().turn.record_card_loss(owner, sha);
    tx.state_mut().turn.record_card_loss(owner, shan);
    tx.state_mut().turn.record_discarded_cards([sha, shan]);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    let hand_before = tx.state().hand_count(owner);
    let siji = skill(&tx, owner, SIJI);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveWdbHanlong {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            helper_skill_def: SkillDefId::new(CIQIU_DYING),
            action: 0,
            skill: siji,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand_before + 2);

    tx.state_mut().turn.phase = Phase::Play;
    let target_state = tx.state_mut().players.get_mut(target).unwrap();
    target_state.max_hp = 2;
    target_state.hp = 2;
    let card = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 3);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source: owner,
            target,
            card,
            amount: 1,
            nature: rule_api::damage_nature::NORMAL,
            card_flags: 0,
        }],
    )
    .unwrap();
    assert!(!tx.state().players.get(target).unwrap().alive);
    assert!(tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .all(|skill| tx.state().skills.get(*skill).unwrap().def.raw() != CIQIU));
}
