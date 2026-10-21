//! Golden coverage for BINGSHI 势娄圭「灌沙」「急御」.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const POTGUANSHA: u32 = 2_542_568_623;
const POTGUANSHA_HAND: u32 = 2_542_568_624;
const POTJIYU: u32 = 3_103_425_312;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(skill_def: u32) -> (engine_core::state::GameState, PlayerId, engine_api::SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(520),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 520,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == skill_def)
        .unwrap()
        .id;
    (tx.commit().state, owner, skill)
}

fn hand_limit(state: &engine_core::state::GameState, player: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::HandLimit { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected hand-limit response: {other:?}"),
    }
}

fn category(
    state: &engine_core::state::GameState,
    player: PlayerId,
    card: engine_api::CardId,
) -> u8 {
    let kind =
        engine_core::query::effective_card_kind(state, player, state.cards.get(card).unwrap());
    if kind.is_basic() {
        1
    } else if kind.is_trick() {
        2
    } else if kind.is_equip() {
        3
    } else {
        0
    }
}

#[test]
fn package_publishes_complete_pot_lougui_at_abi520() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    let character = package
        .character_index
        .iter()
        .find(|character| character.name_key == "pot_lougui")
        .unwrap();
    assert_eq!(character.skills, vec![POTGUANSHA, POTJIYU]);
}

#[test]
fn guansha_replaces_every_hand_card_with_basics_and_adds_equal_hand_limit() {
    let (state, owner, skill) = setup(POTGUANSHA);
    let before_limit = hand_limit(&state, owner);
    let replaced = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePotguansha {
            player: owner,
            helper_skill_def: SkillDefId::new(POTGUANSHA_HAND),
            skill,
        }],
    )
    .unwrap();
    let gained = tx.state().hand_cards(owner);
    assert_eq!(gained.len() as u32, replaced);
    assert!(gained
        .iter()
        .all(|card| category(tx.state(), owner, *card) == 1));
    assert_eq!(
        hand_limit(tx.state(), owner),
        before_limit + replaced as i32
    );
}

#[test]
fn jiyu_gains_other_categories_and_only_resets_twice() {
    let (state, owner, skill) = setup(POTJIYU);
    let mut state = state;
    for expected_reset in [true, true, false] {
        let selected = state.hand_cards(owner)[0];
        let selected_category = category(&state, owner, selected);
        let before = state.hand_cards(owner);
        let mut tx = Transaction::begin(state);
        tx.state_mut().skills.get_mut(skill).unwrap().use_count = 1;
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotjiyu {
                player: owner,
                card: selected,
                stage: 0,
                skill,
            }],
        )
        .unwrap();
        let gained: Vec<_> = tx
            .state()
            .hand_cards(owner)
            .into_iter()
            .filter(|card| !before.contains(card))
            .collect();
        assert!(!gained.is_empty());
        assert!(gained
            .iter()
            .all(|card| category(tx.state(), owner, *card) != selected_category));
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: owner,
                cards: gained,
            }],
        )
        .unwrap();
        assert_eq!(
            tx.state().skills.get(skill).unwrap().use_count == 0,
            expected_reset
        );
        state = tx.commit().state;
    }
}
