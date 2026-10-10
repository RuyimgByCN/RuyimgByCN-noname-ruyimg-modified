//! Golden for 3D精选 薛灵芸【涟洏/暗织】（ABI 636）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const LIANER: u32 = 1_083_711_762;
const ANZHI: u32 = 1_654_153_098;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/xuelingyun.skill");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-xuelingyun", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 636);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.trigger_index.len(), 7);
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&standard).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 4, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend([LIANER, ANZHI].map(|skill_def| EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(skill_def),
    }));
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, players)
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    serial: u32,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone_owner = if matches!(zone_kind, ZoneKind::Hand) {
        owner
    } else {
        PlayerId::NONE
    };
    let zone = state.zones.find(zone_kind, zone_owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_636_000 + serial),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner: zone_owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(
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
fn lianer_reclaims_red_material_and_enforces_rank_ceiling_until_turn_end() {
    let (mut state, players) = setup();
    let owner = players[0];
    let skill = skill_id(&state, owner, LIANER);
    let resolved = inject(
        &mut state,
        owner,
        ZoneKind::DiscardPile,
        1,
        CardKind::Sha,
        Suit::Heart,
        7,
    );
    let lower = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        2,
        CardKind::Shan,
        Suit::Club,
        6,
    );
    let equal = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        3,
        CardKind::Tao,
        Suit::Diamond,
        7,
    );
    let higher = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        4,
        CardKind::Sha,
        Suit::Spade,
        8,
    );
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddXuelingyun {
            player: owner,
            actor: owner,
            card: resolved,
            cards: vec![resolved],
            action: 0,
            skill,
        }],
    )
    .unwrap();
    assert!(tx.state().hand_cards(owner).contains(&resolved));
    assert_eq!(tx.state().skills.get(skill).unwrap().mark, 7);
    assert!(!engine_core::query::card_use_cancelled_for_card(
        tx.state(),
        owner,
        lower
    ));
    assert!(engine_core::query::card_use_cancelled_for_card(
        tx.state(),
        owner,
        equal
    ));
    assert!(engine_core::query::card_use_cancelled_for_card(
        tx.state(),
        owner,
        higher
    ));

    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddXuelingyun {
            player: owner,
            actor: owner,
            card: CardId::NONE,
            cards: Vec::new(),
            action: 1,
            skill,
        }],
    )
    .unwrap();
    assert!(!engine_core::query::card_use_cancelled_for_card(
        tx.state(),
        owner,
        higher
    ));
}

#[test]
fn anzhi_excludes_only_the_first_black_card_target_in_the_armed_turn() {
    let (mut state, players) = setup();
    let owner = players[0];
    let actor = players[1];
    let target = players[2];
    let skill = skill_id(&state, owner, ANZHI);
    let black = inject(
        &mut state,
        actor,
        ZoneKind::Hand,
        5,
        CardKind::Sha,
        Suit::Spade,
        9,
    );
    state.turn.current = actor;
    state.turn.phase = Phase::Play;

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddXuelingyun {
            player: owner,
            actor,
            card: black,
            cards: Vec::new(),
            action: 2,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddXuelingyun {
                player: owner,
                actor,
                card: CardId::NONE,
                cards: Vec::new(),
                action: 3,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (mut state, result) = choose(tx.commit().state, owner, vec![target.raw()], 1);
    assert!(result.choice.is_none());
    state.resolution.resume_kind = ContinuationKind::ResumePlayedSha;
    state.resolution.resume_payload = vec![actor.raw(), target.raw(), 1, 0, 1, black.raw()];

    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: actor,
            card: black,
            targets: vec![target],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(
        tx.state().resolution.resume_payload.get(6),
        Some(&target.raw())
    );
    assert_eq!(tx.state().skills.get(skill).unwrap().state_blob.len(), 13);
    assert_eq!(
        &tx.state().skills.get(skill).unwrap().state_blob[5..],
        &[0; 8]
    );

    tx.state_mut().resolution.resume_payload.truncate(6);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: actor,
            card: black,
            targets: vec![target],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().resolution.resume_payload.len(), 6);
}
