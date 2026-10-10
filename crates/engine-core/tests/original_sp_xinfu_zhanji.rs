//! Golden for SP `xinfu_zhanji`, rewritten from legacy JavaScript.

use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XINFU_ZHANJI: u32 = 4_089_086_847;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XINFU_ZHANJI),
        }],
    )
    .unwrap();
    (tx, owner)
}

#[test]
fn xinfu_zhanji_rewards_each_external_play_phase_draw_without_self_recursion() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == XINFU_ZHANJI)
        .unwrap();
    assert_eq!(trigger.event, "cards_drawn");
    assert_eq!(trigger.filter, "owner_draws_during_own_play_not_self");

    let (mut tx, owner) = started(2_026_071_521);
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::DrawCards {
                player: owner,
                count: 1,
            },
            EffectRequest::DrawCards {
                player: owner,
                count: 1,
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 4);

    tx.state_mut().turn.phase = Phase::End;
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);
}
