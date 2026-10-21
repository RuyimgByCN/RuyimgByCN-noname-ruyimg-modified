//! Golden coverage for old 吴懿「奔袭」rewritten from legacy JavaScript semantics.

use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{query, sha_target_limit_for, QueryContext};
use engine_core::state::{turn_flag, CardKind, GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const BENXI: u32 = 2_283_450_478;
const RENWANG: u32 = 4_083_737_417;
const SOURCE: &str = include_str!("../../../../content/original/ported/yijiang/benxi.skill");
const SP: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");
const MOBILE: &[u8] =
    include_bytes!("../../../../content/original/ported/mobile/mobile.npack.json");

fn distance(state: &GameState, from: PlayerId, to: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::Distance { from, to },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("distance query failed: {other:?}"),
    }
}

fn target_legal(
    state: &GameState,
    user: PlayerId,
    card: engine_api::CardId,
    target: PlayerId,
) -> bool {
    match query(
        state,
        &QueryRequest::TargetLegal { user, card, target },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Bool(value) => value,
        other => panic!("target query failed: {other:?}"),
    }
}

#[test]
fn current_turn_distance_one_enables_extra_sha_target_and_armor_bypass() {
    let package = content_compiler::compile_source_named("benxi-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 281);
    let mut state = GameState::empty();
    for bytes in [SP, MOBILE] {
        std::sync::Arc::make_mut(&mut state.content)
            .load_package(&content_schema::ContentPackage::from_json_bytes(bytes).unwrap())
            .unwrap();
    }
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 2_026_072_903, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let far = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(BENXI),
        },
        EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(RENWANG),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let sha = tx.state().hand_cards(owner)[0];
    {
        let card = tx.state_mut().cards.get_mut(sha).unwrap();
        card.kind = CardKind::Sha;
        card.suit = engine_api::Suit::Spade;
    }

    assert_eq!(distance(tx.state(), owner, far), 2);
    assert_eq!(sha_target_limit_for(tx.state(), owner), 1);
    assert!(!target_legal(tx.state(), owner, sha, target));

    tx.state_mut()
        .turn
        .set_flag(turn_flag::card_used_by_player(owner), 1);
    assert_eq!(distance(tx.state(), owner, far), 1);
    assert_eq!(sha_target_limit_for(tx.state(), owner), 2);
    assert!(target_legal(tx.state(), owner, sha, target));

    tx.state_mut().turn.current = target;
    assert_eq!(distance(tx.state(), owner, far), 2);
    assert_eq!(sha_target_limit_for(tx.state(), owner), 1);
    assert!(!target_legal(tx.state(), owner, sha, target));
}
