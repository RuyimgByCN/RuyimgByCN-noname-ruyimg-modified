//! Golden coverage for 直谏（shenhua 张昭张纮，ABI 512）.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZHIJIAN: u32 = 2_679_144_247;
const SOURCE: &str = include_str!("../../../../content/original/ported/shenhua/zhijian.skill");

fn started() -> (GameState, PlayerId, PlayerId, SkillId) {
    let package = content_compiler::compile_source_named("zhijian-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 2, 20_260_802, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, other) = (seats[0], seats[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZHIJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_110),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 2,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == ZHIJIAN)
        .unwrap();
    (tx.commit().state, owner, other, skill)
}

/// Like [`started`], but also places a non-equipment hand card on the owner.
fn started_with_ordinary() -> (GameState, PlayerId, PlayerId, SkillId, CardId) {
    let (state, owner, other, skill) = started();
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let ordinary = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_111),
        kind: CardKind::Tao,
        suit: Suit::Heart,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(hand)
        .unwrap()
        .cards
        .push(ordinary);
    (tx.commit().state, owner, other, skill, ordinary)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    cards: Vec<CardId>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: cards,
            },
        },
    )
}

fn resolve_choice(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted: {:?}",
        result.result
    );
    next
}

#[test]
fn package_exposes_give_equipment_contract() {
    let compiled = content_compiler::compile_source_named("zhijian", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 512);
    let active = compiled
        .active_index
        .iter()
        .find(|active| active.skill_def_id == ZHIJIAN)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, Some(1));
    let selection = active.selection.as_ref().unwrap();
    assert_eq!(selection.zone, "hand");
    assert_eq!((selection.min, selection.max), (1, 1));
    assert_eq!(selection.filter.as_deref(), Some("card_equip"));
}

#[test]
fn gives_hand_weapon_to_other_player_and_draws_one() {
    let (mut state, owner, other, skill) = started();
    let weapon = state.hand_cards(owner)[0];
    let equip_before = state
        .zones
        .find(ZoneKind::Equip, other)
        .and_then(|zone| state.zones.get(zone))
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();

    let (state, activated) = activate(state, owner, skill, vec![weapon], 1);
    assert!(
        matches!(activated.result, CommandResult::Accepted { .. }),
        "activation must be accepted: {:?}",
        activated.result
    );
    let mut state = resolve_choice(state, owner, vec![other.raw()], 2);

    assert_eq!(
        state.cards.get(weapon).unwrap().zone,
        state.zones.find(ZoneKind::Equip, other).unwrap(),
        "the weapon must be equipped by the other player"
    );
    let equip_after = state
        .zones
        .find(ZoneKind::Equip, other)
        .and_then(|zone| state.zones.get(zone))
        .map(|zone| zone.cards.clone())
        .unwrap_or_default();
    assert_eq!(
        equip_after.len(),
        equip_before.len() + 1,
        "the other player gained exactly the given equipment"
    );
    assert_eq!(state.hand_count(owner), 1, "the owner drew one card");
    assert_eq!(state.skills.get(skill).unwrap().use_count, 1);
}

#[test]
fn rejects_non_equipment_and_owner_as_target() {
    let (mut state, owner, other, skill, ordinary) = started_with_ordinary();
    let weapon = state.hand_cards(owner)[0];

    let before = canonical_state_hash(&state);
    let (same, rejected) = activate(state.clone(), owner, skill, vec![ordinary], 1);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let (state, activated) = activate(state, owner, skill, vec![weapon], 2);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let options: Vec<_> = activated
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert!(
        !options.contains(&owner.raw()),
        "the owner cannot be the 直谏 target"
    );
    assert!(options.contains(&other.raw()));
}
