//! Golden coverage for TW 阎象「睿敛」.

use engine_api::{
    CardDefId, CardId, CardMoveReason, ChoiceId, Command, CommandKind, CommandResult,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWRUILIAN: u32 = 3_419_838_857;
const SOURCE: &str = include_str!("../../../../content/original/ported/tw/twruilian.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/tw/tw.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_365_000 + u32::from(rank)),
        kind,
        suit: Suit::Heart,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn submit(state: GameState, actor: PlayerId, kind: CommandKind, key: u64) -> GameState {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn tracks_discard_categories_then_gains_for_target_and_owner() {
    let package = content_compiler::compile_source_named("twruilian-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 365);
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .character_index
        .iter()
        .any(|character| character.id == 2_630_677_433
            && character.skills == [2_903_587_563, TWRUILIAN]));

    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_080_105, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWRUILIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.round = 1;
    tx.state_mut().turn.current = target;
    let basic = inject(&mut tx, target, CardKind::Sha, 1);
    let trick = inject(&mut tx, target, CardKind::TrickWuzhong, 2);
    let spare_basic = inject(&mut tx, owner, CardKind::Tao, 3);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == TWRUILIAN)
        .unwrap()
        .id;

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwruilian {
                player: owner,
                target: PlayerId::NONE,
                event_cards: Vec::new(),
                action: 0,
                category: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    state = submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections: vec![target.raw()],
            cancelled: false,
        },
        1,
    );

    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let target_hand = state.zones.find(ZoneKind::Hand, target).unwrap();
    let owner_hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::MoveCardsWithReason {
                cards: vec![basic, trick],
                to: discard,
                from: Some(target_hand),
                reason: CardMoveReason::Discard,
            },
            EffectRequest::MoveCards {
                cards: vec![spare_basic],
                to: discard,
                from: Some(owner_hand),
            },
        ],
    )
    .unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwruilian {
                player: owner,
                target,
                event_cards: Vec::new(),
                action: 2,
                category: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    state = submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections: vec![1],
            cancelled: false,
        },
        2,
    );
    assert!(state.hand_cards(target).contains(&spare_basic));
    assert!(state.hand_cards(owner).contains(&basic));
    assert!(state.zones.get(discard).unwrap().cards.contains(&trick));
}
