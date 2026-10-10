//! Golden coverage for old「安恤」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLDANXU: u32 = 2_342_781_658;
const SOURCE: &str = include_str!("../../../content/original/ported/old/oldanxu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    giver: PlayerId,
    receiver: PlayerId,
    skill: SkillId,
    card: CardId,
}

fn setup(suit: Suit) -> Setup {
    let package = content_compiler::compile_source_named("oldanxu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_804, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let &[owner, giver, receiver] = players.as_slice() else {
        unreachable!()
    };
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in players {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let cards = tx.state().zones.get(hand).unwrap().cards.clone();
        if !cards.is_empty() {
            effects.push(EffectRequest::MoveCards {
                cards,
                to: discard,
                from: Some(hand),
            });
        }
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLDANXU),
    });
    run_effects(&mut tx, effects).unwrap();
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == OLDANXU)
        .unwrap();
    let card = inject(&mut tx, giver, suit, 7);
    inject(&mut tx, giver, Suit::Heart, 8);
    Setup {
        state: tx.commit().state,
        owner,
        giver,
        receiver,
        skill,
        card,
    }
}

fn inject(tx: &mut Transaction, owner: PlayerId, suit: Suit, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(97_000 + u32::from(rank)),
        kind: CardKind::Shan,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, actor: PlayerId, token: u32, key: u64) -> GameState {
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
                selections: vec![token],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_uses_extended_effect_tag_and_publishes_oldanxu() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLDANXU && skill.name == "oldanxu"));
    let module = package
        .modules
        .iter()
        .find(|module| module.name.starts_with("oldanxu_active"))
        .unwrap();
    assert!(module.code.windows(3).any(|bytes| bytes == [0x5A, 0, 1]));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn lower_hand_target_gains_card_and_non_spade_draws_for_owner() {
    let setup = setup(Suit::Club);
    let owner_before = setup.state.hand_count(setup.owner);
    let revision = setup.state.revision;
    let (state, activated) = apply_command(
        setup.state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: setup.skill,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveOldAnxu
    );
    let state = choose(state, setup.owner, setup.giver.raw(), 2);
    let state = choose(state, setup.owner, setup.receiver.raw(), 3);
    let state = choose(state, setup.receiver, setup.card.raw(), 4);
    assert_eq!(state.cards.get(setup.card).unwrap().owner, setup.receiver);
    assert_eq!(state.hand_count(setup.giver), 1);
    assert_eq!(state.hand_count(setup.receiver), 1);
    assert_eq!(state.hand_count(setup.owner), owner_before + 1);
}
