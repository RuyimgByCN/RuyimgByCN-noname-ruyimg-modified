//! Golden coverage for 用间曹昂「血拼」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, CardMoveReason, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult,
    DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const YJXUEPIN: u32 = 2_926_558_107;
const YJ_CAOANG: u32 = 3_284_926_958;
const SOURCE: &str = include_str!("../../../../content/original/ported/offline/yjxuepin.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    in_range: PlayerId,
    out_of_range: PlayerId,
    skill: SkillId,
}

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> Setup {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package())
        .unwrap();
    let owner = tx.state().turn.current;
    tx.state_mut().players.get_mut(owner).unwrap().hp = 4;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 4;
    let others: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let in_range = others
        .iter()
        .copied()
        .find(|target| engine_core::query::player_in_attack_range(tx.state(), owner, *target))
        .unwrap();
    let out_of_range = others
        .iter()
        .copied()
        .find(|target| !engine_core::query::player_in_attack_range(tx.state(), owner, *target))
        .unwrap();
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
        skill_def: SkillDefId::new(YJXUEPIN),
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == YJXUEPIN)
        .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        in_range,
        out_of_range,
        skill,
    }
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

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_publishes_exact_abi155_contract_and_character() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(package.skill_index.len(), 851);
    assert_eq!(package.character_index.len(), 385);
    assert_eq!(package.trigger_index.len(), 587);
    assert_eq!(package.active_index.len(), 137);
    assert_eq!(package.modifier_index.len(), 141);
    assert_eq!(package.modules.len(), 724);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == YJXUEPIN)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!(
        (targets.min, targets.max, targets.allow_self),
        (1, 1, false)
    );
    assert!(targets.in_attack_range);
    assert!(targets.has_discardable_hand_equip);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == YJ_CAOANG)
        .unwrap();
    assert_eq!(character.name_key, "yj_caoang");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [YJXUEPIN]);

    let compiled = content_compiler::compile_source_named("yjxuepin", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 155);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn two_same_category_cards_recover_after_the_forced_discard() {
    let mut setup = setup(2_026_072_508);
    let sha = inject_hand(&mut setup.state, setup.in_range, 155_101, CardKind::Sha);
    let shan = inject_hand(&mut setup.state, setup.in_range, 155_102, CardKind::Shan);
    let trick = inject_hand(
        &mut setup.state,
        setup.in_range,
        155_103,
        CardKind::TrickWuzhong,
    );
    let (state, offered) = activate(setup.state, setup.owner, setup.in_range, setup.skill, 1);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 3);
    let choice = offered.choice.unwrap();
    assert_eq!(choice.observer, setup.owner);
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!((choice.min_count, choice.max_count), (2, 2));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardTargetCardsRecoverSameCategory
    );

    let (state, resolved) = resolve(state, setup.owner, vec![sha.raw(), shan.raw()], 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 4);
    assert_eq!(state.hand_count(setup.in_range), 1);
    assert_eq!(state.cards.get(trick).unwrap().owner, setup.in_range);
    assert!(resolved.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsMoved {
            cards,
            reason: CardMoveReason::Discard,
            ..
        } if cards == &[sha, shan]
    )));
    assert!(resolved.events.iter().any(|event| matches!(
        event,
        DomainEvent::HpRecovered {
            player,
            amount: 1,
            ..
        } if *player == setup.owner
    )));
}

#[test]
fn one_card_or_mixed_categories_do_not_recover() {
    let mut one_card = setup(2_026_072_509);
    let only = inject_hand(
        &mut one_card.state,
        one_card.in_range,
        155_201,
        CardKind::Sha,
    );
    let (state, output) = activate(
        one_card.state,
        one_card.owner,
        one_card.in_range,
        one_card.skill,
        10,
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(one_card.owner).unwrap().hp, 3);
    assert_eq!(state.hand_count(one_card.in_range), 0);
    assert_eq!(
        state.cards.get(only).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );

    let mut setup = setup(2_026_072_510);
    let basic = inject_hand(&mut setup.state, setup.in_range, 155_211, CardKind::Sha);
    let trick = inject_hand(
        &mut setup.state,
        setup.in_range,
        155_212,
        CardKind::TrickWuzhong,
    );
    inject_hand(&mut setup.state, setup.in_range, 155_213, CardKind::Shan);
    let (state, _) = activate(setup.state, setup.owner, setup.in_range, setup.skill, 11);
    let (state, output) = resolve(state, setup.owner, vec![basic.raw(), trick.raw()], 12);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 3);
    assert!(!output
        .events
        .iter()
        .any(|event| matches!(event, DomainEvent::HpRecovered { .. })));
}

#[test]
fn range_empty_target_and_duplicate_selection_are_rejected_atomically() {
    let mut setup = setup(2_026_072_511);
    inject_hand(&mut setup.state, setup.out_of_range, 155_301, CardKind::Sha);
    let before = canonical_state_hash(&setup.state);
    let (same, rejected) = activate(
        setup.state,
        setup.owner,
        setup.out_of_range,
        setup.skill,
        20,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let before = canonical_state_hash(&same);
    let (same, rejected) = activate(same, setup.owner, setup.in_range, setup.skill, 21);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let mut state = same;
    let first = inject_hand(&mut state, setup.in_range, 155_311, CardKind::Sha);
    inject_hand(&mut state, setup.in_range, 155_312, CardKind::Shan);
    inject_hand(&mut state, setup.in_range, 155_313, CardKind::TrickWuzhong);
    let (state, _) = activate(state, setup.owner, setup.in_range, setup.skill, 22);
    let before = canonical_state_hash(&state);
    let (same, rejected) = resolve(state, setup.owner, vec![first.raw(), first.raw()], 23);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);
}
