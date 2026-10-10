//! Golden coverage for old「蛊惑」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLD_GUHUO: u32 = 3_484_005_747;
const SOURCE: &str = include_str!("../../../content/original/ported/old/old_guhuo.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    challenger: PlayerId,
    target: PlayerId,
    source: CardId,
}

fn setup(kind: CardKind, suit: Suit) -> Setup {
    let package = content_compiler::compile_source_named("old-guhuo-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 273);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_931, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let challenger = players[1];
    let target = players[2];
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
    for player in &players {
        effects.push(EffectRequest::MoveCards {
            cards: tx.state().hand_cards(*player),
            to: discard,
            from: tx.state().zones.find(ZoneKind::Hand, *player),
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLD_GUHUO),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let source = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(90_273),
        kind,
        suit,
        rank: 7,
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
        .push(source);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        state: tx.commit().state,
        owner,
        challenger,
        target,
        source,
    }
}

fn choose(state: GameState, token: u32, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
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

fn declare_sha(setup: Setup, decisions: [u32; 2]) -> GameState {
    let skill = setup.state.players.get(setup.owner).unwrap().skills[0];
    let revision = setup.state.revision;
    let (state, result) = apply_command(
        setup.state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::Sha.tag(),
                targets: vec![setup.target],
                equip_slot: 0,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let state = choose(state, setup.source.raw(), 2);
    let state = choose(state, decisions[0], 3);
    choose(state, decisions[1], 4)
}

#[test]
fn challenge_truth_fake_and_no_challenge_follow_legacy_resolution() {
    let truthful_heart = setup(CardKind::Sha, Suit::Heart);
    let target = truthful_heart.target;
    let challenger = truthful_heart.challenger;
    let target_hp = truthful_heart.state.players.get(target).unwrap().hp;
    let challenger_hp = truthful_heart.state.players.get(challenger).unwrap().hp;
    let state = declare_sha(truthful_heart, [1, 0]);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert_eq!(state.players.get(challenger).unwrap().hp, challenger_hp - 1);

    let truthful_spade = setup(CardKind::Sha, Suit::Spade);
    let target = truthful_spade.target;
    let source = truthful_spade.source;
    let target_hp = truthful_spade.state.players.get(target).unwrap().hp;
    let state = declare_sha(truthful_spade, [1, 0]);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp);
    assert_eq!(
        state
            .zones
            .get(state.cards.get(source).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::DiscardPile
    );

    let fake = setup(CardKind::Tao, Suit::Heart);
    let target = fake.target;
    let challenger = fake.challenger;
    let target_hp = fake.state.players.get(target).unwrap().hp;
    let hand_before = fake.state.hand_count(challenger);
    let state = declare_sha(fake, [1, 0]);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp);
    assert_eq!(state.hand_count(challenger), hand_before + 1);

    let unchallenged_fake = setup(CardKind::Tao, Suit::Spade);
    let target = unchallenged_fake.target;
    let target_hp = unchallenged_fake.state.players.get(target).unwrap().hp;
    let state = declare_sha(unchallenged_fake, [0, 0]);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
}

#[test]
fn package_publishes_old_guhuo_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLD_GUHUO && skill.name == "old_guhuo"));
    assert!(package
        .character_index
        .iter()
        .any(|character| character.id == 3_730_153_489 && character.name_key == "yuji"));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}
