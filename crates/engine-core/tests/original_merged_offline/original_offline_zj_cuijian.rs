//! Golden coverage for 智将张虎「摧坚」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, CardMoveReason, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZJ_CUIJIAN: u32 = 3_620_039_376;
const ZJ_ZHANGHU: u32 = 2_696_580_984;
const SOURCE: &str = include_str!("../../../../content/original/ported/offline/zj_cuijian.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    targets: Vec<PlayerId>,
    skill: SkillId,
}

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> Setup {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package())
        .unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let targets: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        tx.state()
            .players
            .alive_by_seat()
            .into_iter()
            .map(|player| EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }),
    );
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZJ_CUIJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == ZJ_CUIJIAN)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        targets,
        skill,
    }
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    )
}

#[test]
fn package_publishes_abi150_exact_contract_and_character() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(package.skill_index.len(), 851);
    assert_eq!(package.character_index.len(), 385);
    assert_eq!(package.trigger_index.len(), 587);
    assert_eq!(package.active_index.len(), 137);
    assert_eq!(package.modifier_index.len(), 141);
    assert_eq!(package.modules.len(), 724);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == ZJ_CUIJIAN && skill.name == "zj_cuijian"));
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == ZJ_CUIJIAN)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (
            targets.min,
            targets.max,
            targets.allow_self,
            targets.hand_nonempty,
        ),
        (1, 1, false, true)
    );
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == ZJ_ZHANGHU)
        .unwrap();
    assert_eq!(character.name_key, "zj_zhanghu");
    assert_eq!(character.skills, [ZJ_CUIJIAN]);

    let compiled = content_compiler::compile_source_named("zj_cuijian", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 150);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn gains_every_shan_and_armor_after_revealing_the_complete_hand() {
    let mut setup = setup(2_026_072_501);
    let target = setup.targets[0];
    let shan = inject_hand(&mut setup.state, target, 150_001, CardKind::Shan);
    let armor = inject_hand(&mut setup.state, target, 150_002, CardKind::Armor);
    let sha = inject_hand(&mut setup.state, target, 150_003, CardKind::Sha);
    let weapon = inject_hand(&mut setup.state, target, 150_004, CardKind::Weapon);
    let (state, output) = activate(setup.state, setup.owner, target, setup.skill, 1);

    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), 2);
    assert_eq!(state.hand_count(target), 2);
    for card in [shan, armor] {
        assert_eq!(state.cards.get(card).unwrap().owner, setup.owner);
    }
    for card in [sha, weapon] {
        assert_eq!(state.cards.get(card).unwrap().owner, target);
    }
    assert!(output.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to }
            if cards == &[shan, armor, sha, weapon] && to.is_empty()
    )));
    assert!(output.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsMoved {
            cards,
            reason: CardMoveReason::Give,
            ..
        } if cards == &[shan, armor]
    )));

    let before = canonical_state_hash(&state);
    let (same, repeated) = activate(state, setup.owner, target, setup.skill, 2);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}

#[test]
fn no_matching_card_draws_two_and_an_empty_target_is_rejected_atomically() {
    let setup = setup(2_026_072_502);
    let empty_target = setup.targets[0];
    let before = canonical_state_hash(&setup.state);
    let (same, rejected) = activate(setup.state, setup.owner, empty_target, setup.skill, 10);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let mut state = same;
    let sha = inject_hand(&mut state, empty_target, 150_011, CardKind::Sha);
    let weapon = inject_hand(&mut state, empty_target, 150_012, CardKind::Weapon);
    let (state, output) = activate(state, setup.owner, empty_target, setup.skill, 11);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), 2);
    assert_eq!(state.hand_count(empty_target), 2);
    assert_eq!(state.cards.get(sha).unwrap().owner, empty_target);
    assert_eq!(state.cards.get(weapon).unwrap().owner, empty_target);
    assert!(output.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to }
            if cards == &[sha, weapon] && to.is_empty()
    )));
    assert!(!output.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsMoved {
            cards,
            reason: CardMoveReason::Give,
            ..
        } if cards.contains(&sha) || cards.contains(&weapon)
    )));
}
