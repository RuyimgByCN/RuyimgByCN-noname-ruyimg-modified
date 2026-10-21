//! Golden coverage for Standard 激将 rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JIJIANG: u32 = 2_377_283_906;
const SOURCE: &str = include_str!("../../../../content/standard/jijiang.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/standard/standard.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    first_shu: PlayerId,
    second_shu: PlayerId,
    target: PlayerId,
    first_sha: CardId,
    second_sha: CardId,
    unlock_card: CardId,
    skill: SkillId,
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(97_000 + u32::from(rank)),
        kind,
        suit: Suit::Club,
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
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 4, 20_260_725_187, "identity-standard").unwrap();
    let [owner, first_shu, second_shu, target]: [PlayerId; 4] =
        tx.state().players.alive_by_seat().try_into().unwrap();
    for (player, character) in [(owner, 8), (first_shu, 9), (second_shu, 10), (target, 1)] {
        tx.state_mut()
            .players
            .get_mut(player)
            .unwrap()
            .character_def = character;
    }
    tx.state_mut().players.get_mut(owner).unwrap().identity = Identity::Lord;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([owner, first_shu, second_shu, target].map(|player| {
        EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        }
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JIJIANG),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in [owner, first_shu, second_shu, target] {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }
    let first_sha = inject(&mut tx, first_shu, CardKind::Sha, 2);
    let second_sha = inject(&mut tx, second_shu, CardKind::Sha, 3);
    let unlock_card = inject(&mut tx, owner, CardKind::Jiu, 4);
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == JIJIANG)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        first_shu,
        second_shu,
        target,
        first_sha,
        second_sha,
        unlock_card,
        skill,
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

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn use_jijiang(setup: &Setup) -> CommandKind {
    CommandKind::UseSkillCard {
        skill: setup.skill,
        card_kind: CardKind::Sha.tag(),
        targets: vec![setup.target],
        equip_slot: 0,
    }
}

#[test]
fn package_closes_liubei_with_lord_only_group_sha() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let liubei = package
        .character_index
        .iter()
        .find(|character| character.id == 8)
        .unwrap();
    assert_eq!(liubei.skills, [11, JIJIANG]);
    let skill = package
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == JIJIANG)
        .unwrap();
    assert!(skill.lord_only);
    let view = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == JIJIANG)
        .unwrap();
    assert_eq!(view.as_kind, "sha");
    assert_eq!(view.window, "any");
    assert_eq!(view.source_zone.as_deref(), Some("group_shu"));
    let compiled = content_compiler::compile_source_named("standard-jijiang", SOURCE).unwrap();
    assert_eq!(compiled.view_as_index, vec![view.clone()]);
}

#[test]
fn shu_players_are_asked_in_seat_order_and_provider_card_becomes_lords_sha() {
    let setup = setup();
    let hp = setup.state.players.get(setup.target).unwrap().hp;
    let (state, first) = submit(setup.state.clone(), setup.owner, use_jijiang(&setup), 1);
    assert_eq!(first.choice.as_ref().unwrap().observer, setup.first_shu);
    let (state, second) = resolve(state, setup.first_shu, vec![], true, 2);
    assert_eq!(second.choice.as_ref().unwrap().observer, setup.second_shu);
    let (state, used) = resolve(
        state,
        setup.second_shu,
        vec![setup.second_sha.raw()],
        false,
        3,
    );
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    assert!(used.choice.is_none());
    assert_eq!(state.players.get(setup.target).unwrap().hp, hp - 1);
    assert!(state.hand_cards(setup.first_shu).contains(&setup.first_sha));
    assert_eq!(
        state.cards.get(setup.second_sha).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert!(used.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardResponded {
            player,
            respond_to,
            card,
            ..
        } if *player == setup.second_shu
            && *respond_to == setup.owner
            && *card == setup.second_sha
    )));
    assert!(used.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardUsed {
            player,
            targets,
            ..
        } if *player == setup.owner && targets == &[setup.target]
    )));
}

#[test]
fn delegated_sha_can_answer_duel_for_the_lord() {
    let mut setup = setup();
    setup.state.turn.current = setup.target;
    let mut tx = Transaction::begin(setup.state);
    let duel = inject(&mut tx, setup.target, CardKind::TrickJuedou, 5);
    setup.state = tx.commit().state;
    let owner_hp = setup.state.players.get(setup.owner).unwrap().hp;
    let target_hp = setup.state.players.get(setup.target).unwrap().hp;
    let (state, duel_choice) = submit(
        setup.state,
        setup.target,
        CommandKind::PlayCard {
            card: duel,
            targets: vec![setup.owner],
            card_targets: vec![],
        },
        4,
    );
    let token = duel_choice
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .find(|option| option.token > u32::MAX - 1_000)
        .unwrap()
        .token;
    let (state, provider) = resolve(state, setup.owner, vec![token], false, 5);
    assert_eq!(provider.choice.as_ref().unwrap().observer, setup.first_shu);
    let (state, resolved) = resolve(
        state,
        setup.first_shu,
        vec![setup.first_sha.raw()],
        false,
        6,
    );
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(resolved.choice.is_none());
    assert_eq!(state.players.get(setup.owner).unwrap().hp, owner_hp);
    assert_eq!(state.players.get(setup.target).unwrap().hp, target_hp - 1);
}

#[test]
fn all_decline_blocks_only_an_immediate_retry() {
    let setup = setup();
    let use_skill = use_jijiang(&setup);
    let (state, first) = submit(setup.state, setup.owner, use_skill.clone(), 7);
    assert_eq!(first.choice.as_ref().unwrap().observer, setup.first_shu);
    let (state, second) = resolve(state, setup.first_shu, vec![], true, 8);
    assert_eq!(second.choice.as_ref().unwrap().observer, setup.second_shu);
    let (state, declined) = resolve(state, setup.second_shu, vec![], true, 9);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    assert!(declined.choice.is_none());

    let (state, blocked) = submit(state, setup.owner, use_skill.clone(), 10);
    assert!(matches!(blocked.result, CommandResult::Rejected { .. }));
    assert!(blocked.choice.is_none());

    let (state, unlocked) = submit(
        state,
        setup.owner,
        CommandKind::PlayCard {
            card: setup.unlock_card,
            targets: vec![],
            card_targets: vec![],
        },
        11,
    );
    assert!(matches!(unlocked.result, CommandResult::Accepted { .. }));
    let (_, retry) = submit(state, setup.owner, use_skill, 12);
    assert_eq!(retry.choice.as_ref().unwrap().observer, setup.first_shu);
}
