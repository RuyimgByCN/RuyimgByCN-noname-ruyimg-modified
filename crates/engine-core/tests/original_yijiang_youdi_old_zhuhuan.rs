//! Golden coverage for 游敌 and old 朱桓 rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const YOUDI: u32 = 3_686_673_279;
const SOURCE: &str = include_str!("../../../content/original/ported/yijiang/youdi.skill");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(owner_kind: CardKind, seed: u64) -> (GameState, PlayerId, PlayerId, CardId, CardId) {
    let package = content_compiler::compile_source_named("youdi-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 280);
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_is_target_phase_end_hand_equip_nonempty_other_alive"
    );
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
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
        skill_def: SkillDefId::new(YOUDI),
    });
    run_effects(&mut tx, effects).unwrap();
    let owner_card = inject(&mut tx, owner, owner_kind, 98_001);
    let target_card = inject(&mut tx, target, CardKind::Shan, 98_002);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def == SkillDefId::new(YOUDI))
        .unwrap()
        .id;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveYoudi {
                player: owner,
                target: PlayerId::NONE,
                card: CardId::NONE,
                stage: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    (tx.commit().state, owner, target, owner_card, target_card)
}

fn resolve(state: GameState, actor: PlayerId, selection: u32, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    state
}

#[test]
fn non_sha_grants_counter_gain_but_sha_does_not() {
    let (state, owner, target, owner_card, target_card) = setup(CardKind::Shan, 2_026_072_901);
    assert_eq!(state.resolution.pending_observer, owner.raw());
    let state = resolve(state, owner, target.raw(), 1);
    assert_eq!(state.resolution.pending_observer, target.raw());
    let state = resolve(state, target, owner_card.raw(), 2);
    assert_eq!(state.resolution.pending_observer, owner.raw());
    let state = resolve(state, owner, target_card.raw(), 3);
    assert_eq!(state.resolution.pending_choice_id, 0);
    assert!(state.hand_cards(owner).contains(&target_card));
    assert!(!state.hand_cards(target).contains(&target_card));

    let (state, owner, target, owner_sha, target_card) = setup(CardKind::Sha, 2_026_072_902);
    let state = resolve(state, owner, target.raw(), 4);
    let state = resolve(state, target, owner_sha.raw(), 5);
    assert_eq!(state.resolution.pending_choice_id, 0);
    assert!(state.hand_cards(target).contains(&target_card));
}
