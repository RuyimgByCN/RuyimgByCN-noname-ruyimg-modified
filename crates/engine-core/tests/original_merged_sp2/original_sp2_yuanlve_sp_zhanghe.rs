//! Golden coverage for SP2 张郃「远略」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SP_ZHANGHE: u32 = 4_161_518_802;
const YUANLVE: u32 = 3_073_596_171;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp2/sp2.npack.json");

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn add_hand_card(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_497),
        kind,
        suit: Suit::Heart,
        rank,
        weapon_range: u8::from(matches!(kind, CardKind::Weapon)),
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_and_runtime_preserve_exact_yuanlve_flow() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 753);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == SP_ZHANGHE)
            .unwrap()
            .skills,
        [YUANLVE]
    );
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == YUANLVE)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    assert_eq!(
        active.selection.as_ref().unwrap().filter.as_deref(),
        Some("card_non_equip")
    );

    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_080_202, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, recipient, victim) = (players[0], players[1], players[2]);
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(YUANLVE),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let sha = add_hand_card(&mut tx, owner, CardKind::Sha, 7);
    let equipment = add_hand_card(&mut tx, owner, CardKind::Weapon, 8);
    let skill: SkillId = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == YUANLVE)
        .unwrap();
    let state = tx.commit().state;
    let hand_before = state.hand_count(owner);

    let revision = state.revision;
    let (state, rejected) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![recipient],
                card_targets: vec![equipment],
            },
        },
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let revision = state.revision;
    let (state, accepted) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![recipient],
                card_targets: vec![sha],
            },
        },
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before - 1);
    let state = resolve(state, recipient, vec![sha.raw()], false, 3);
    let mut state = resolve(state, recipient, vec![victim.raw()], false, 4);
    let mut key = 5;
    while state.has_pending_choice() {
        let actor = PlayerId::new(state.resolution.pending_observer);
        state = resolve(state, actor, Vec::new(), true, key);
        key += 1;
    }
    assert_eq!(
        state.hand_count(owner),
        hand_before,
        "successful use draws one"
    );
    assert_ne!(state.cards.get(sha).unwrap().owner, recipient);
}
