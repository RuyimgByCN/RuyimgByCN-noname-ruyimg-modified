//! Golden coverage for BINGSHI 势辛宪英「诫节/清识」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{card_use_unlimited_for_card, query, QueryContext};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse, RelationTag};

const JIEJIE: u32 = 3_788_885_537;
const QINGSHI: u32 = 2_678_891_040;
const USED: u32 = 4_200_000_554;
const SUIT_DEFS: [u32; 4] = [4_200_000_555, 4_200_000_556, 4_200_000_557, 4_200_000_558];
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/pot_xinxianying.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 554);
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let order = engine.state().players.alive_by_seat();
    let owner = order[0];
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([JIEJIE, QINGSHI].map(|def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    }));
    run_effects(&mut tx, effects).unwrap();
    clear_hands(&mut tx);
    (tx.commit().state, owner, order)
}

fn clear_hands(tx: &mut Transaction) {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let effects: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter_map(|player| {
            let cards = tx.state().hand_cards(player);
            (!cards.is_empty()).then(|| EffectRequest::MoveCards {
                cards,
                to: discard,
                from: tx.state().zones.find(ZoneKind::Hand, player),
            })
        })
        .collect();
    run_effects(tx, effects).unwrap();
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, serial: u32, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(554_000 + serial),
        kind: CardKind::Sha,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: PlayerId::new(state.resolution.pending_observer),
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_complete_pot_xinxianying_at_abi558() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(package.character_index.len(), 36);
    assert_eq!(package.skill_index.len(), 103);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_xinxianying")
            .unwrap()
            .skills,
        [JIEJIE, QINGSHI]
    );
}

#[test]
fn jiejie_discards_other_suits_and_grants_turn_suit_limit() {
    let (state, owner, order) = setup(5541);
    let actor = order[1];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    let spade = inject_hand(&mut tx, actor, 1, Suit::Spade);
    inject_hand(&mut tx, actor, 2, Suit::Heart);
    let jiejie = skill(tx.state(), owner, JIEJIE);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotjiejie {
                player: owner,
                actor,
                used_def: SkillDefId::new(USED),
                suit_defs: SUIT_DEFS.map(SkillDefId::new),
                qingshi_def: SkillDefId::new(QINGSHI),
                skill: jiejie,
            }]
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, output) = resolve(tx.commit().state, vec![0], false, 5542);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_cards(actor), [spade]);
    assert!(card_use_unlimited_for_card(&state, actor, spade));
    assert!(state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == actor && skill.def.raw() == USED));
    assert_ne!(
        state.resolution.pending_choice_id, 0,
        "unique maximum invokes 清识"
    );
    let (_, output) = resolve(state, Vec::new(), true, 5543);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
}

#[test]
fn qingshi_resolves_ally_draw_and_enemy_discards() {
    let (state, owner, order) = setup(5544);
    let qingshi = skill(&state, owner, QINGSHI);
    let mut tx = Transaction::begin(state);
    let before = tx.state().hand_count(owner);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotqingshi {
                player: owner,
                skill: qingshi,
            }]
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, output) = resolve(tx.commit().state, vec![owner.raw()], false, 5545);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 2);

    let enemy = order
        .into_iter()
        .find(|target| {
            matches!(
                query(
                    &state,
                    &QueryRequest::Relation {
                        a: owner,
                        b: *target,
                    },
                    &mut QueryContext::new(),
                ),
                QueryResponse::Relation(RelationTag::Enemy)
            )
        })
        .unwrap();
    let mut tx = Transaction::begin(state);
    clear_hands(&mut tx);
    let owner_card = inject_hand(&mut tx, owner, 3, Suit::Club);
    let enemy_card = inject_hand(&mut tx, enemy, 4, Suit::Diamond);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePotqingshi {
            player: owner,
            skill: qingshi,
        }],
    )
    .unwrap_err();
    let (state, _) = resolve(tx.commit().state, vec![enemy.raw()], false, 5546);
    let (state, _) = resolve(state, vec![owner_card.raw()], false, 5547);
    let (state, output) = resolve(state, vec![enemy_card.raw()], false, 5548);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(owner).is_empty());
    assert!(state.hand_cards(enemy).is_empty());
}
