//! Golden tests for small active skills rewritten from legacy JavaScript.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const PSZHIYI: u32 = 2_980_176_526;
const NHXIANSHOU: u32 = 2_305_482_906;
const NHHUOQI: u32 = 2_275_957_194;
const REKUROU: u32 = 2_651_237_190;
const DCDUANXIE: u32 = 4_278_513_633;
const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");
const SP2: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const XIANDING: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn started(package: &[u8], skill_def: u32, seed: u64) -> (GameState, PlayerId, PlayerId, SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(package).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let target = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == skill_def)
        .unwrap();
    (tx.state().clone(), owner, target, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    cards: Vec<engine_api::CardId>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: cards,
            },
        },
    )
}

fn is_linked(state: &GameState, player: PlayerId) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0)
}

#[test]
fn packages_publish_one_target_once_per_turn_contracts() {
    for (bytes, skill) in [
        (OFFLINE, PSZHIYI),
        (SP2, NHXIANSHOU),
        (SP2, NHHUOQI),
        (XIANDING, DCDUANXIE),
    ] {
        let package = content_schema::ContentPackage::from_json_bytes(bytes).unwrap();
        let active = package
            .active_index
            .iter()
            .find(|active| active.skill_def_id == skill)
            .unwrap();
        assert_eq!(active.uses_per_turn, Some(1));
        assert_eq!(
            active
                .targets
                .as_ref()
                .map(|target| (target.min, target.max)),
            Some((1, 1))
        );
    }
}

#[test]
fn dcduanxie_requires_an_unlinked_other_target_and_links_both_once() {
    let package = content_schema::ContentPackage::from_json_bytes(XIANDING).unwrap();
    let targets = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DCDUANXIE)
        .unwrap()
        .targets
        .as_ref()
        .unwrap();
    assert_eq!((targets.min, targets.max), (1, 1));
    assert!(!targets.allow_self);
    assert!(targets.unlinked);

    let (mut linked, owner, target, skill) = started(XIANDING, DCDUANXIE, 2_026_071_602);
    linked
        .players
        .get_mut(target)
        .unwrap()
        .marks
        .push((engine_core::content::MARK_CHAIN, 1));
    let before = canonical_state_hash(&linked);
    let (same, pipe) = activate(linked, owner, target, skill, vec![], 1);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
    assert!(!is_linked(&same, owner));

    let (state, owner, target, skill) = started(XIANDING, DCDUANXIE, 2_026_071_603);
    let (mut state, pipe) = activate(state, owner, target, skill, vec![], 2);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert!(is_linked(&state, owner));
    assert!(is_linked(&state, target));

    state
        .players
        .get_mut(target)
        .unwrap()
        .marks
        .retain(|(mark, _)| *mark != engine_core::content::MARK_CHAIN);
    let before = canonical_state_hash(&state);
    let (same, pipe) = activate(state, owner, target, skill, vec![], 3);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}

#[test]
fn pszhiyi_draws_then_deals_normal_damage_and_cannot_repeat() {
    let (state, owner, target, skill) = started(OFFLINE, PSZHIYI, 2_026_071_514);
    let hand = state.hand_count(target);
    let hp = state.players.get(target).unwrap().hp;
    let (state, pipe) = activate(state, owner, target, skill, vec![], 1);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(target), hand + 1);
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert!(pipe.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target: damaged,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        } if *source == owner && *damaged == target
    )));

    let before = canonical_state_hash(&state);
    let (same, pipe) = activate(state, owner, target, skill, vec![], 2);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}

#[test]
fn nhxianshou_draws_two_when_healthy_and_one_when_wounded() {
    for (seed, wounded, expected) in [(2_026_071_515, false, 2), (2_026_071_516, true, 1)] {
        let (mut state, owner, target, skill) = started(SP2, NHXIANSHOU, seed);
        if wounded {
            state.players.get_mut(target).unwrap().hp -= 1;
        }
        let hand = state.hand_count(target);
        let (state, pipe) = activate(state, owner, target, skill, vec![], 1);
        assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
        assert_eq!(state.hand_count(target), hand + expected);
    }
}

#[test]
fn nhhuoqi_accepts_equipment_cost_and_only_a_lowest_hp_target() {
    let (mut state, owner, target, skill) = started(SP2, NHHUOQI, 2_026_071_517);
    state.players.get_mut(target).unwrap().hp = 2;
    let equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(90_001),
        kind: CardKind::Treasure,
        suit: Suit::Club,
        rank: 6,
        weapon_range: 0,
        zone: equip,
        owner,
        face_up: true,
    });
    state.zones.get_mut(equip).unwrap().cards.push(card);
    let hand = state.hand_count(target);
    let (state, pipe) = activate(state, owner, target, skill, vec![card], 1);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    assert_eq!(state.hand_count(target), hand + 1);
    assert_eq!(
        state.cards.get(card).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );

    let (mut state, owner, target, skill) = started(SP2, NHHUOQI, 2_026_071_518);
    state.players.get_mut(owner).unwrap().hp = 2;
    state.players.get_mut(target).unwrap().hp = 3;
    let cost = state.hand_cards(owner)[0];
    let before = canonical_state_hash(&state);
    let (same, pipe) = activate(state, owner, target, skill, vec![cost], 1);
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}

#[test]
fn rekurou_discards_one_card_loses_hp_and_cannot_repeat() {
    let package = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == REKUROU)
        .expect("苦肉 active entry");
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.targets.is_none());
    let selection = active.selection.as_ref().unwrap();
    assert_eq!(
        (selection.zone.as_str(), selection.min, selection.max),
        ("hand_equip", 1, 1)
    );

    let (state, owner, _, skill) = started(REFRESH, REKUROU, 2_026_071_601);
    let hp = state.players.get(owner).unwrap().hp;
    let cost = state.hand_cards(owner)[0];
    let (state, pipe) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![cost],
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
    assert_eq!(
        state.cards.get(cost).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );

    let second_cost = state.hand_cards(owner)[0];
    let before = canonical_state_hash(&state);
    let (same, pipe) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![second_cost],
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
