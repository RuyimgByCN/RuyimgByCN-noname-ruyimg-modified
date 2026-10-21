//! Golden coverage for BINGSHI 势黄祖「鸱张」「断鞅」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const MBCHIZHANG: u32 = 3_712_476_166;
const MBDUANYANG: u32 = 2_631_325_253;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(u64::from(skill_def)),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: u64::from(skill_def),
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    for player in [owner, target] {
        let cards = tx.state().hand_cards(player);
        if !cards.is_empty() {
            run_effects(&mut tx, vec![EffectRequest::DiscardCards { player, cards }]).unwrap();
        }
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
    (tx, owner, target)
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, n: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(505_000 + n),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(state: GameState, selections: Vec<u32>, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn chizhang_locks_matching_response_color_and_duanyang_stores_ownerless_sha() {
    let (mut tx, owner, target) = setup(MBCHIZHANG);
    let sha = inject(&mut tx, owner, CardKind::Sha, Suit::Spade, 1);
    let red_cost = inject(&mut tx, owner, CardKind::Tao, Suit::Heart, 2);
    let red_shan = inject(&mut tx, target, CardKind::Shan, Suit::Diamond, 3);
    let black_shan = inject(&mut tx, target, CardKind::Shan, Suit::Club, 4);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    let state = resolve(state, vec![1], 2);
    let state = resolve(state, vec![red_cost.raw()], 3);
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let response = |selection| Command {
        actor: target,
        expected_revision: ExpectedRevision(revision),
        idempotency_key: IdempotencyKey(u64::from(selection)),
        kind: CommandKind::ResolveChoice {
            choice_id,
            selections: vec![selection],
            cancelled: false,
        },
    };
    let (_, red_output) = apply_command(state.clone(), response(red_shan.raw()));
    assert!(matches!(red_output.result, CommandResult::Rejected { .. }));
    let (_, black_output) = apply_command(state, response(black_shan.raw()));
    assert!(matches!(
        black_output.result,
        CommandResult::Accepted { .. }
    ));

    let (mut tx, owner, _) = setup(MBDUANYANG);
    let stored = inject(&mut tx, owner, CardKind::Sha, Suit::Heart, 5);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    tx.state_mut()
        .zones
        .get_mut(hand)
        .unwrap()
        .cards
        .retain(|card| *card != stored);
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(stored);
    let card = tx.state_mut().cards.get_mut(stored).unwrap();
    card.zone = discard;
    card.owner = PlayerId::NONE;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == MBDUANYANG)
        .unwrap()
        .id;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbduanyang {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            mode: 0,
            stage: 0,
            cards: vec![stored],
            skill,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().skill_pile_cards(owner, MBDUANYANG), [stored]);
    assert!(tx.state().playable_hand_cards(owner).contains(&stored));
}
