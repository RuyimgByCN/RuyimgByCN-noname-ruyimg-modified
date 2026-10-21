//! Golden coverage for SP OL 牛辅「纵掠」(olzonglve).
//! Play-phase view-as Sha once per turn; dealing damage takes one he from the victim.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLZONGLVE: u32 = 3_052_953_425;
const SOURCE: &str = include_str!("../../../../content/original/ported/sp/olzonglve.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn inject_hand(state: &mut GameState, player: PlayerId, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup() -> (GameState, PlayerId, PlayerId, CardId) {
    let compiled = content_compiler::compile_source_named("olzonglve-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_32, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLZONGLVE),
    });
    run_effects(&mut tx, effects).unwrap();
    let card = inject_hand(tx.state_mut(), other, 170_601);
    (tx.commit().state, owner, other, card)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
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
    )
}

#[test]
fn compiled_source_and_npack_publish_sha_and_take() {
    let compiled = content_compiler::compile_source_named("olzonglve", SOURCE).unwrap();
    let view = compiled
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == OLZONGLVE)
        .expect("olzonglve view_as");
    assert_eq!(view.as_kind, "sha");
    assert_eq!(view.uses_per_turn, Some(1));
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLZONGLVE)
        .expect("olzonglve trigger");
    assert_eq!(trigger.event, "damage");
    assert_eq!(
        trigger.filter,
        "owner_source_target_alive_has_hand_or_equip"
    );

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .view_as_index
        .iter()
        .any(|view| view.skill_def_id == OLZONGLVE && view.as_kind == "sha"));
    assert!(committed.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == OLZONGLVE
            && trigger.filter == "owner_source_target_alive_has_hand_or_equip"
    }));
}

#[test]
fn damage_takes_one_card_from_the_victim() {
    let (state, owner, other, card) = setup();
    let owner_hand = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target: other,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, taken) = resolve(tx.commit().state, owner, vec![card.raw()], 1);
    assert!(
        matches!(taken.result, CommandResult::Accepted { .. }),
        "{:?}",
        taken.result
    );
    assert_eq!(state.hand_count(owner), owner_hand + 1);
    assert_eq!(state.cards.get(card).unwrap().owner, owner);
}
