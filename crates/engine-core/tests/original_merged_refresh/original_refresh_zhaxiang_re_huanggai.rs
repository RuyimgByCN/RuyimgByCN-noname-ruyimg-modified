//! Golden coverage for REFRESH「诈降」and the complete re_huanggai closure.

use engine_api::{CardDefId, CardId, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{
    card_distance_unlimited_for_card, sha_unrespondable_for_card, sha_use_limit_for_card,
};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZHAXIANG: u32 = 3_298_876_867;
const ZHAXIANG_EFFECT: u32 = 4_056_078_041;
const RE_HUANGGAI: u32 = 3_020_853_843;
const SOURCE: &str = include_str!("../../../../content/original/ported/refresh/zhaxiang.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");

fn started(phase: Phase) -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("zhaxiang-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, ZHAXIANG as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZHAXIANG),
    });
    effects.push(EffectRequest::SetTurnState {
        current: owner,
        phase: phase.to_u8(),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn inject_sha(tx: &mut Transaction, owner: PlayerId, suit: Suit, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn lose_hp(tx: &mut Transaction, owner: PlayerId, amount: u32) {
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: owner,
            amount,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
}

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

fn mark_count(state: &GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(id, _)| *id == ZHAXIANG_EFFECT)
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[test]
fn package_publishes_abi106_skill_helper_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 251);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == ZHAXIANG && skill.name == "zhaxiang"));
    assert!(package
        .skill_index
        .iter()
        .any(|skill| { skill.skill_def_id == ZHAXIANG_EFFECT && skill.name == "zhaxiang_effect" }));
    assert!(package.character_index.iter().any(|character| {
        character.id == RE_HUANGGAI
            && character.skills == vec![2_651_237_190, ZHAXIANG]
            && character.max_hp == 4
    }));
}

#[test]
fn hp_loss_draws_three_per_point_but_only_play_phase_grants_the_helper() {
    let (mut tx, owner, _) = started(Phase::End);
    let before = tx.state().hand_count(owner);
    lose_hp(&mut tx, owner, 2);
    assert_eq!(tx.state().hand_count(owner), before + 6);
    assert!(!has_skill(tx.state(), owner, ZHAXIANG_EFFECT));
    assert_eq!(mark_count(tx.state(), owner), 0);
}

#[test]
fn play_phase_hp_loss_stacks_sha_uses_and_red_sha_rules_until_turn_end() {
    let (mut tx, owner, other) = started(Phase::Play);
    let red_sha = inject_sha(&mut tx, owner, Suit::Heart, 106_001);
    let black_sha = inject_sha(&mut tx, owner, Suit::Spade, 106_002);
    let base_limit = sha_use_limit_for_card(tx.state(), owner, red_sha);
    let before = tx.state().hand_count(owner);

    lose_hp(&mut tx, owner, 2);

    assert_eq!(tx.state().hand_count(owner), before + 6);
    assert!(has_skill(tx.state(), owner, ZHAXIANG_EFFECT));
    assert_eq!(mark_count(tx.state(), owner), 2);
    assert_eq!(
        sha_use_limit_for_card(tx.state(), owner, red_sha),
        base_limit + 2
    );
    assert!(card_distance_unlimited_for_card(tx.state(), owner, red_sha));
    assert!(sha_unrespondable_for_card(tx.state(), owner, red_sha));
    assert!(!card_distance_unlimited_for_card(
        tx.state(),
        owner,
        black_sha
    ));
    assert!(!sha_unrespondable_for_card(tx.state(), owner, black_sha));

    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: other,
                phase: Phase::Prepare.to_u8(),
            },
        ],
    )
    .unwrap();
    assert!(!has_skill(tx.state(), owner, ZHAXIANG_EFFECT));
    assert_eq!(mark_count(tx.state(), owner), 0);
    assert_eq!(
        sha_use_limit_for_card(tx.state(), owner, red_sha),
        base_limit
    );
    assert!(!card_distance_unlimited_for_card(
        tx.state(),
        owner,
        red_sha
    ));
    assert!(!sha_unrespondable_for_card(tx.state(), owner, red_sha));
}
