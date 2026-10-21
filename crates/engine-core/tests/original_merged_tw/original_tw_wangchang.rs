//! Golden coverage for TW 王昶「开济/慑叛」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWKAIJI: u32 = 3_117_219_235;
const TWSHEPAN: u32 = 3_217_560_878;
const SOURCE: &str = include_str!("../../../../content/original/ported/tw/twwangchang.skill");

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
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

fn setup(
    skill_def: u32,
    seed: u64,
) -> (
    Transaction,
    PlayerId,
    PlayerId,
    PlayerId,
    engine_api::SkillId,
) {
    let package = content_compiler::compile_source_named("tw-wangchang-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 386);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
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
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    (tx, owner, first, second, skill)
}

#[test]
fn kaiji_scales_and_shepan_refunds_or_places_on_top() {
    let (mut tx, owner, first, second, kaiji) = setup(TWKAIJI, 2_026_080_201);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::EnterDying { player: first },
            EffectRequest::EnterDying { player: second },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().lifetime_dying_players.len(), 2);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top = *tx.state().zones.get(draw).unwrap().cards.last().unwrap();
    let top_state = tx.state_mut().cards.get_mut(top).unwrap();
    top_state.def = engine_api::CardDefId::NONE;
    top_state.kind = CardKind::Weapon;
    let first_hand = tx.state().hand_count(first);
    let second_hand = tx.state().hand_count(second);
    let owner_hand = tx.state().hand_count(owner);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwwangchang {
                player: owner,
                target: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                selected_card: engine_api::CardId::NONE,
                action: 0,
                stage: 0,
                targets: Vec::new(),
                skill: kaiji,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![first.raw(), second.raw()], 1);
    assert_eq!(state.hand_count(first), first_hand + 1);
    assert_eq!(state.hand_count(second), second_hand + 1);
    assert_eq!(state.hand_count(owner), owner_hand + 1);

    let (mut tx, owner, target, _, shepan) = setup(TWSHEPAN, 2_026_080_202);
    let owner_hand_zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let target_hand = tx.state().hand_count(target);
    let owner_hand = tx.state().hand_count(owner);
    if owner_hand >= target_hand {
        let discard_count = owner_hand.saturating_sub(target_hand).saturating_add(1) as usize;
        let cards = tx.state().hand_cards(owner)[..discard_count].to_vec();
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards,
                to: discard,
                from: Some(owner_hand_zone),
            }],
        )
        .unwrap();
    } else {
        run_effects(
            &mut tx,
            vec![EffectRequest::DrawCards {
                player: owner,
                count: (target_hand - 1 - owner_hand) as u32,
            }],
        )
        .unwrap();
    }
    assert_eq!(tx.state().hand_count(owner) + 1, target_hand);
    tx.state_mut()
        .skills
        .get_mut(shepan)
        .unwrap()
        .turn_fire_count = 1;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwwangchang {
            player: owner,
            target,
            card: engine_api::CardId::NONE,
            selected_card: engine_api::CardId::NONE,
            action: 1,
            stage: 0,
            targets: Vec::new(),
            skill: shepan,
        }],
    )
    .unwrap_err();
    let state = choose(tx.commit().state, owner, vec![0], 2);
    assert_eq!(state.skills.get(shepan).unwrap().turn_fire_count, 0);
    assert!(state.has_pending_choice());

    let (mut tx, owner, target, _, shepan) = setup(TWSHEPAN, 2_026_080_203);
    let selected = tx.state().hand_cards(target)[0];
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwwangchang {
                player: owner,
                target,
                card: engine_api::CardId::NONE,
                selected_card: engine_api::CardId::NONE,
                action: 1,
                stage: 0,
                targets: Vec::new(),
                skill: shepan,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![1], 3);
    let state = choose(state, owner, vec![selected.raw()], 4);
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(
        *state.zones.get(draw).unwrap().cards.last().unwrap(),
        selected
    );
}
