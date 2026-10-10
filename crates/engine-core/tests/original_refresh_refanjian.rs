//! Golden coverage for REFRESH「反间」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const REFANJIAN: u32 = 2_975_386_328;
const REYINGZI: u32 = 2_412_932_912;
const RE_ZHOUYU: u32 = 4_255_995_705;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/refanjian.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REFANJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == REFANJIAN)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
    }
}

fn activate(setup: Setup, card: CardId, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        setup.state.clone(),
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(setup.state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: setup.skill,
                targets: vec![setup.target],
                card_targets: vec![card],
            },
        },
    )
}

fn resolve(
    state: GameState,
    target: PlayerId,
    branch: u32,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: target,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![branch],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_publishes_abi123_active_and_complete_re_zhouyu() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 251);
    assert!(package.skill_index.len() >= 76);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == REFANJIAN)
        .expect("refanjian active entry");
    assert_eq!(active.uses_per_turn, Some(1));
    assert_eq!(
        active
            .targets
            .as_ref()
            .map(|targets| (targets.min, targets.max, targets.allow_self)),
        Some((1, 1, false))
    );
    assert_eq!(
        active.selection.as_ref().map(|selection| (
            selection.zone.as_str(),
            selection.min,
            selection.max
        )),
        Some(("hand", 1, 1))
    );
    assert!(package.character_index.iter().any(|character| {
        character.id == RE_ZHOUYU
            && character.name_key == "re_zhouyu"
            && character.max_hp == 3
            && character.skills == [REYINGZI, REFANJIAN]
    }));

    let compiled = content_compiler::compile_source_named("refanjian", SOURCE).unwrap();
    assert!(compiled.modules.iter().all(|expected| {
        package
            .modules
            .iter()
            .any(|actual| actual.name == expected.name && actual.logic_hash == expected.logic_hash)
    }));
}

#[test]
fn discard_branch_reveals_hand_and_discards_matching_hand_equip_cards() {
    let mut setup = setup(2_026_072_401);
    let gift = setup.state.hand_cards(setup.owner)[0];
    let target_cards = setup.state.hand_cards(setup.target);
    let same_hand = target_cards[0];
    let same_equip = target_cards[1];
    let different = target_cards[2];
    for card in setup.state.hand_cards(setup.target) {
        setup.state.cards.get_mut(card).unwrap().suit = Suit::Spade;
    }
    setup.state.cards.get_mut(gift).unwrap().suit = Suit::Heart;
    setup.state.cards.get_mut(same_hand).unwrap().suit = Suit::Heart;
    {
        let equipment = setup.state.cards.get_mut(same_equip).unwrap();
        equipment.suit = Suit::Heart;
        equipment.kind = CardKind::Weapon;
        equipment.weapon_range = 2;
    }
    let target_hand = setup
        .state
        .zones
        .find(ZoneKind::Hand, setup.target)
        .unwrap();
    let target_equip = setup
        .state
        .zones
        .find(ZoneKind::Equip, setup.target)
        .unwrap();
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![same_equip],
            to: target_equip,
            from: Some(target_hand),
        }],
    )
    .unwrap();
    setup.state = tx.commit().state;
    let hp = setup.state.players.get(setup.target).unwrap().hp;

    let target = setup.target;
    let (state, activated) = activate(setup, gift, 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp
    );
    assert_eq!(state.resolution.pending_observer, target.raw());
    assert_eq!(
        activated.choice.as_ref().map(|choice| {
            choice
                .options
                .iter()
                .map(|option| option.token)
                .collect::<Vec<_>>()
        }),
        Some(vec![0, 1])
    );
    assert!(activated.events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::CardsRevealed { cards, to }
            if cards == &vec![gift] && to.is_empty()
    )));
    assert!(state.hand_cards(target).contains(&gift));

    let (state, discarded) = resolve(state, target, 0, 2);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(target).unwrap().hp, hp);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for card in [gift, same_hand, same_equip] {
        assert_eq!(state.cards.get(card).unwrap().zone, discard);
    }
    assert!(state.hand_cards(target).contains(&different));
    assert!(discarded.events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::CardsRevealed { cards, to }
            if cards.contains(&gift) && cards.contains(&same_hand) && to.is_empty()
    )));
}

#[test]
fn hp_branch_keeps_matching_cards_and_loses_one_hp() {
    let mut setup = setup(2_026_072_402);
    let gift = setup.state.hand_cards(setup.owner)[0];
    setup.state.cards.get_mut(gift).unwrap().suit = Suit::Club;
    let hp = setup.state.players.get(setup.target).unwrap().hp;
    let target = setup.target;

    let (state, activated) = activate(setup, gift, 3);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let (state, lost_hp) = resolve(state, target, 1, 4);
    assert!(matches!(lost_hp.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert!(state.hand_cards(target).contains(&gift));
    assert!(!lost_hp
        .events
        .iter()
        .any(|event| matches!(event, engine_api::DomainEvent::CardsRevealed { .. })));
}

#[test]
fn empty_hand_at_follow_up_barrier_automatically_loses_hp() {
    let setup = setup(2_026_072_403);
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: setup.target,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let hp = tx.state().players.get(setup.target).unwrap().hp;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
                player: setup.owner,
                target: setup.target,
                card: CardId::NONE,
                suit: Suit::Diamond,
                skill: setup.skill,
                ready_to_choose: true,
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(setup.target).unwrap().hp, hp - 1);
    assert!(!tx.state().has_pending_choice());
}
