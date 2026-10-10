//! Golden coverage for「彻骨」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const CHEGU: u32 = 4_294_592_545;
const SOURCE: &str = include_str!("../../../content/original/ported/old/chegu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_271 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
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
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    (state, result)
}

#[test]
fn cancels_the_original_sha_and_uniform_discards_raise_this_turns_limit() {
    let package = content_compiler::compile_source_named("chegu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_072_914, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let sha_target = players[1];
    let discard_target = players[2];
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
        skill_def: SkillDefId::new(CHEGU),
    });
    run_effects(&mut tx, effects).unwrap();
    let sha = inject(&mut tx, owner, CardKind::Sha, Suit::Spade, 7);
    let red_basic = inject(&mut tx, discard_target, CardKind::Tao, Suit::Heart, 3);
    let red_equip = inject(&mut tx, discard_target, CardKind::Weapon, Suit::Diamond, 4);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let target_hp = tx.state().players.get(sha_target).unwrap().hp;
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    let state = tx.commit().state;

    let revision = state.revision;
    let (state, played) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![sha_target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    assert!(played.choice.is_some());
    let (state, _) = resolve(state, owner, vec![1], 2);
    let (state, _) = resolve(state, owner, vec![1], 3);
    let (state, _) = resolve(state, owner, vec![discard_target.raw()], 4);
    let (state, _) = resolve(state, owner, vec![red_basic.raw(), red_equip.raw()], 5);

    assert_eq!(state.players.get(sha_target).unwrap().hp, target_hp);
    assert_eq!(state.cards.get(red_basic).unwrap().zone, discard);
    assert_eq!(state.cards.get(red_equip).unwrap().zone, discard);

    let mut tx = Transaction::begin(state);
    let extra1 = inject(&mut tx, discard_target, CardKind::Shan, Suit::Club, 5);
    let extra2 = inject(&mut tx, discard_target, CardKind::Tao, Suit::Heart, 6);
    let extra3 = inject(&mut tx, discard_target, CardKind::Weapon, Suit::Spade, 8);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveChegu {
                player: owner,
                card: sha,
                targets: vec![discard_target],
                cards: Vec::new(),
                branch: 1,
                index: 0,
                stage: 2,
                skill: SkillId::new(skill.raw()),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let choice = tx.commit().output.choice.unwrap();
    assert_eq!((choice.min_count, choice.max_count), (1, 3));
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == extra1.raw()));
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == extra2.raw()));
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == extra3.raw()));
}

#[test]
fn package_publishes_chegu_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == CHEGU && skill.name == "chegu"));
    assert!(package.character_index.iter().any(|character| {
        character.id == 2_763_282_459 && character.name_key == "two_yj_hanbing"
    }));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}
