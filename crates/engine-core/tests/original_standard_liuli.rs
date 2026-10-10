//! Golden coverage for Standard 流离 rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LIULI: u32 = 3_733_125_857;
const SOURCE: &str = include_str!("../../../content/standard/liuli.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

struct Setup {
    state: GameState,
    source: PlayerId,
    owner: PlayerId,
    candidate: PlayerId,
    far: PlayerId,
    sha: CardId,
    cost: CardId,
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup() -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 4, 20_260_725_186, "identity-standard").unwrap();
    let [source, owner, candidate, far]: [PlayerId; 4] =
        tx.state().players.alive_by_seat().try_into().unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([source, owner, candidate, far].into_iter().map(|player| {
        EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        }
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LIULI),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in [source, owner, candidate, far] {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }
    tx.state_mut().turn.current = source;
    tx.state_mut().turn.phase = Phase::Play;
    let cost = inject(&mut tx, owner, CardKind::Other, Suit::Club, 2);
    let sha = inject(&mut tx, source, CardKind::Sha, Suit::Spade, 3);
    Setup {
        state: tx.commit().state,
        source,
        owner,
        candidate,
        far,
        sha,
        cost,
    }
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

#[test]
fn package_closes_daqiao_with_guose_and_exact_liuli() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let daqiao = package
        .character_index
        .iter()
        .find(|character| character.id == 20)
        .unwrap();
    assert_eq!(daqiao.skills, [28, LIULI]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == LIULI)
        .unwrap();
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(
        trigger.filter,
        "owner_sha_target_can_discard_redirect_to_other_in_attack_range"
    );
    let compiled = content_compiler::compile_source_named("standard-liuli", SOURCE).unwrap();
    assert_eq!(compiled.trigger_index[0].skill_def_id, trigger.skill_def_id);
    assert_eq!(compiled.trigger_index[0].filter, trigger.filter);
    assert_eq!(compiled.trigger_index[0].module, trigger.module);
}

#[test]
fn discard_then_target_redirects_saved_sha_and_preserves_original_target() {
    let setup = setup();
    assert!(engine_core::query::player_in_attack_range(
        &setup.state,
        setup.owner,
        setup.candidate
    ));
    assert!(!engine_core::query::player_in_attack_range(
        &setup.state,
        setup.source,
        setup.candidate
    ));
    let owner_hp = setup.state.players.get(setup.owner).unwrap().hp;
    let candidate_hp = setup.state.players.get(setup.candidate).unwrap().hp;
    let (state, cost_choice) = submit(
        setup.state,
        setup.source,
        CommandKind::PlayCard {
            card: setup.sha,
            targets: vec![setup.owner],
            card_targets: vec![],
        },
        1,
    );
    let cost_choice = cost_choice.choice.unwrap();
    assert_eq!(cost_choice.observer, setup.owner);
    assert_eq!(cost_choice.kind, ChoiceKind::SelectCards);
    assert_eq!(
        cost_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [setup.cost.raw()]
    );
    let (state, target_choice) = submit(
        state,
        setup.owner,
        CommandKind::ResolveChoice {
            choice_id: cost_choice.choice_id,
            selections: vec![setup.cost.raw()],
            cancelled: false,
        },
        2,
    );
    let target_choice = target_choice.choice.unwrap();
    assert_eq!(target_choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(
        target_choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [setup.candidate.raw()]
    );
    assert!(!target_choice
        .options
        .iter()
        .any(|option| option.token == setup.source.raw()));
    assert!(!target_choice
        .options
        .iter()
        .any(|option| option.token == setup.far.raw()));
    let (state, redirected) = submit(
        state,
        setup.owner,
        CommandKind::ResolveChoice {
            choice_id: target_choice.choice_id,
            selections: vec![setup.candidate.raw()],
            cancelled: false,
        },
        3,
    );
    assert!(matches!(redirected.result, CommandResult::Accepted { .. }));
    assert!(redirected.choice.is_none());
    assert_eq!(state.players.get(setup.owner).unwrap().hp, owner_hp);
    assert_eq!(
        state.players.get(setup.candidate).unwrap().hp,
        candidate_hp - 1
    );
    assert_eq!(
        state.cards.get(setup.cost).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert!(redirected.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardTargetAdded {
            player,
            card,
            target,
            targets,
        } if *player == setup.source
            && *card == setup.sha
            && *target == setup.candidate
            && targets == &[setup.candidate]
    )));
}

#[test]
fn declining_the_discard_keeps_the_original_sha_target() {
    let setup = setup();
    let owner_hp = setup.state.players.get(setup.owner).unwrap().hp;
    let candidate_hp = setup.state.players.get(setup.candidate).unwrap().hp;
    let (state, offered) = submit(
        setup.state,
        setup.source,
        CommandKind::PlayCard {
            card: setup.sha,
            targets: vec![setup.owner],
            card_targets: vec![],
        },
        4,
    );
    let choice = offered.choice.unwrap();
    let (state, result) = submit(
        state,
        setup.owner,
        CommandKind::ResolveChoice {
            choice_id: choice.choice_id,
            selections: vec![],
            cancelled: true,
        },
        5,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, owner_hp - 1);
    assert_eq!(state.players.get(setup.candidate).unwrap().hp, candidate_hp);
    assert!(state.hand_cards(setup.owner).contains(&setup.cost));
}
