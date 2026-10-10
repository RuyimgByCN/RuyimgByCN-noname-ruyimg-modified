//! Golden coverage for 神话「颂威」rewritten from legacy JavaScript semantics.

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::content;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SONGWEI: u32 = 2_169_124_968;
const TEST_REWRITE: u32 = 9_216_001;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/songwei.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn character(id: u32, group: &str) -> PackageCharacterEntry {
    PackageCharacterEntry {
        id,
        name_key: format!("songwei_test_{id}"),
        max_hp: 4,
        initial_hp: 0,
        skills: vec![],
        sex: "male".into(),
        group: group.into(),
        is_lord: false,
        clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: vec![],
    }
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_216_100 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: kind.is_delayed_trick(),
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

struct Setup {
    tx: Transaction,
    lord: PlayerId,
    judged: PlayerId,
    other: PlayerId,
}

fn setup(include_rewriter: bool) -> Setup {
    let source = if include_rewriter {
        format!(
            "{SOURCE}\nskill test_rewrite({TEST_REWRITE}) locked {{ modifier judgment_rewrite add 1; }}"
        )
    } else {
        SOURCE.to_owned()
    };
    let mut package = content_compiler::compile_source_named("songwei-test", &source).unwrap();
    package.character_index = vec![
        character(9_216_010, "wei"),
        character(9_216_011, "wei"),
        character(9_216_012, "shu"),
        character(9_216_013, "wu"),
    ];
    package.manifest.logic_hash = package.compute_logic_hash();

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, 20_260_722_86, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let lord = seats
        .iter()
        .copied()
        .find(|player| tx.state().players.get(*player).unwrap().identity == Identity::Lord)
        .unwrap();
    let mut rest = seats.iter().copied().filter(|player| *player != lord);
    let judged = rest.next().unwrap();
    let other = rest.next().unwrap();
    let fourth = rest.next().unwrap();
    tx.state_mut().players.get_mut(lord).unwrap().character_def = 9_216_010;
    tx.state_mut()
        .players
        .get_mut(judged)
        .unwrap()
        .character_def = 9_216_011;
    tx.state_mut().players.get_mut(other).unwrap().character_def = 9_216_012;
    tx.state_mut()
        .players
        .get_mut(fourth)
        .unwrap()
        .character_def = 9_216_013;

    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: lord,
        skill_def: SkillDefId::new(SONGWEI),
    });
    if include_rewriter {
        effects.push(EffectRequest::AddSkill {
            player: other,
            skill_def: SkillDefId::new(TEST_REWRITE),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    Setup {
        tx,
        lord,
        judged,
        other,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
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
fn package_exposes_lord_event_player_choice_contract() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 671);
    assert!(committed.skill_index.iter().any(|skill| {
        skill.skill_def_id == SONGWEI && skill.name == "songwei" && skill.lord_only
    }));
    let trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SONGWEI)
        .unwrap();
    assert_eq!(trigger.event, "judgment_resolved");
    assert_eq!(
        trigger.filter,
        "owner_alive_other_judgment_black_event_player_group_wei"
    );
    assert!(trigger.optional);
    assert_eq!(
        trigger.choice_actor,
        content_schema::TriggerChoiceActor::EventPlayer
    );
}

#[test]
fn only_another_wei_players_black_judgment_prompts_that_player() {
    let setup = setup(false);
    let discard = setup
        .tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut cases = [
        (setup.judged, Suit::Spade, true),
        (setup.judged, Suit::Heart, false),
        (setup.other, Suit::Club, false),
        (setup.lord, Suit::Spade, false),
    ];
    for (index, (judged, suit, expected)) in cases.iter_mut().enumerate() {
        let mut tx = Transaction::begin(setup.tx.state().clone());
        let result_card = inject(&mut tx, PlayerId::NONE, discard, CardKind::Other, *suit, 7);
        let result = run_effects(
            &mut tx,
            vec![EffectRequest::EmitJudgmentResolved {
                player: *judged,
                result_card,
                suit: *suit,
                rank: 7,
                cause_card: CardId::NONE,
            }],
        );
        if *expected {
            assert_eq!(result, Err(EffectError::ChoiceRequested));
            assert_eq!(tx.state().resolution.pending_observer, setup.judged.raw());
        } else {
            result.unwrap();
            assert!(!tx.state().has_pending_choice(), "case {index}");
        }
    }
}

#[test]
fn rewritten_physical_card_drives_songwei_then_delayed_trick_resumes() {
    let mut setup = setup(true);
    let judge = setup
        .tx
        .state()
        .zones
        .find(ZoneKind::Judge, setup.judged)
        .unwrap();
    let delayed = inject(
        &mut setup.tx,
        setup.judged,
        judge,
        CardKind::TrickLe,
        Suit::Club,
        6,
    );
    let draw = setup
        .tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let original = inject(
        &mut setup.tx,
        PlayerId::NONE,
        draw,
        CardKind::Other,
        Suit::Heart,
        3,
    );
    let hand = setup
        .tx
        .state()
        .zones
        .find(ZoneKind::Hand, setup.other)
        .unwrap();
    let rewrite = inject(
        &mut setup.tx,
        setup.other,
        hand,
        CardKind::Other,
        Suit::Spade,
        9,
    );
    setup.tx.state_mut().turn.current = setup.judged;
    setup.tx.state_mut().turn.phase = Phase::Judge;
    let state = setup.tx.commit().state;

    let (state, rewrite_choice) = apply_command(
        state.clone(),
        Command {
            actor: setup.judged,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(
        rewrite_choice.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        rewrite_choice.choice.as_ref().unwrap().observer,
        setup.other
    );

    let lord_hand = state.hand_count(setup.lord);
    let (state, songwei_choice) = resolve(state, setup.other, vec![rewrite.raw()], 2);
    assert!(matches!(
        songwei_choice.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        songwei_choice.choice.as_ref().unwrap().observer,
        setup.judged
    );
    assert!(songwei_choice.events.iter().any(|event| {
        matches!(
            event,
            DomainEvent::JudgmentResolved {
                player,
                result_card,
                cause_card,
                ..
            } if *player == setup.judged && *result_card == rewrite && *cause_card == delayed
        )
    }));
    assert!(!songwei_choice.events.iter().any(|event| {
        matches!(event, DomainEvent::JudgmentResolved { result_card, .. } if *result_card == original)
    }));

    let (state, accepted) = resolve(state, setup.judged, vec![1], 3);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.lord), lord_hand + 1);
    assert_eq!(state.turn.flag(turn_flag::SKIP_PLAY), 1);
    assert!(!state.has_pending_choice());
}

#[test]
fn black_bagua_judgment_prompts_songwei_then_reopens_sha_response() {
    let mut setup = setup(false);
    let equip = setup
        .tx
        .state()
        .zones
        .find(ZoneKind::Equip, setup.judged)
        .unwrap();
    let bagua = inject(
        &mut setup.tx,
        setup.judged,
        equip,
        CardKind::Armor,
        Suit::Spade,
        2,
    );
    setup.tx.state_mut().cards.get_mut(bagua).unwrap().def = CardDefId::new(content::DEF_BAGUA);
    let draw = setup
        .tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let result_card = inject(
        &mut setup.tx,
        PlayerId::NONE,
        draw,
        CardKind::Other,
        Suit::Spade,
        8,
    );
    let hand = setup
        .tx
        .state()
        .zones
        .find(ZoneKind::Hand, setup.other)
        .unwrap();
    let sha = inject(
        &mut setup.tx,
        setup.other,
        hand,
        CardKind::Sha,
        Suit::Club,
        6,
    );
    setup.tx.state_mut().turn.current = setup.other;
    setup.tx.state_mut().turn.phase = Phase::Play;
    let state = setup.tx.commit().state;

    let (state, response) = apply_command(
        state.clone(),
        Command {
            actor: setup.other,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(10),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![setup.judged],
                card_targets: vec![],
            },
        },
    );
    assert_eq!(response.choice.as_ref().unwrap().observer, setup.judged);
    let lord_hand = state.hand_count(setup.lord);
    let (state, songwei) = resolve(state, setup.judged, vec![content::BAGUA_RESPONSE_TOKEN], 11);
    assert_eq!(songwei.choice.as_ref().unwrap().observer, setup.judged);
    assert!(songwei.events.iter().any(|event| {
        matches!(
            event,
            DomainEvent::JudgmentResolved {
                player,
                result_card: card,
                cause_card,
                ..
            } if *player == setup.judged && *card == result_card && *cause_card == bagua
        )
    }));

    let (state, accepted) = resolve(state, setup.judged, vec![1], 12);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.lord), lord_hand + 1);
    assert!(state.has_pending_choice());
    assert_eq!(state.resolution.pending_observer, setup.judged.raw());
}
