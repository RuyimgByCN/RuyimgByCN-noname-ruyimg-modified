//! Golden semantics for EXTRA 神刘备「龙怒」「结营」。

use engine_api::{
    CardDefId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const LONGNU: u32 = 2_666_801_006;
const LONGNU_FIRE: u32 = 3_653_578_858;
const LONGNU_THUNDER: u32 = 2_167_979_660;
const JIEYING: u32 = 3_363_786_286;
const SHEN_LIUBEI: u32 = 3_081_060_085;
const EXTRA: &[u8] = include_bytes!("../../../content/original/ported/extra/extra.npack.json");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup(player_count: u8) -> (GameState, Vec<PlayerId>) {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let extra = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&standard).unwrap();
    engine.load_package(&extra).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(LONGNU as u64),
            kind: CommandKind::StartGame {
                player_count,
                rng_seed: LONGNU as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let mut key = 1;
    while engine.state().has_pending_choice() {
        let choice = engine.last_choice().unwrap().clone();
        assert!(matches!(
            engine.submit(Command {
                actor: choice.observer,
                expected_revision: ExpectedRevision(engine.state().revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: Vec::new(),
                    cancelled: true,
                },
            }),
            CommandResult::Accepted { .. }
        ));
        key += 1;
    }
    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let old = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    (tx.state().clone(), players)
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> SkillId {
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

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return;
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
}

fn inject_trick(tx: &mut Transaction, owner: PlayerId, rank: u8) -> engine_api::CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(98_000 + u32::from(rank)),
        kind: CardKind::TrickWuzhong,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn activate(
    state: GameState,
    actor: PlayerId,
    skill: SkillId,
    target: PlayerId,
    card: engine_api::CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![card],
            },
        },
    )
}

#[test]
fn package_closes_shen_liubei_with_source_free_contracts() {
    let package = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SHEN_LIUBEI)
        .unwrap();
    assert_eq!(character.skills, [LONGNU, JIEYING]);
    content_runtime::check_engine_compat(
        &package.manifest,
        &content_runtime::tooling_host_versions(),
    )
    .unwrap();
    assert!(package.view_as_index.iter().any(|view| {
        view.skill_def_id == LONGNU_FIRE
            && view.filter == "red"
            && view.nature.as_deref() == Some("fire")
            && view.ignore_distance
    }));
    assert!(package.view_as_index.iter().any(|view| {
        view.skill_def_id == LONGNU_THUNDER
            && view.filter == "card_trick"
            && view.nature.as_deref() == Some("thunder")
    }));
}

#[test]
fn longnu_starts_on_fire_then_alternates_to_thunder() {
    let (state, players) = setup(2);
    let owner = players[0];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.hp = 6;
    participant.max_hp = 6;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(LONGNU),
        }],
    )
    .unwrap();
    let longnu = skill(tx.state(), owner, LONGNU);
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveNzryLongnu {
            player: owner,
            fire_skill_def: SkillDefId::new(LONGNU_FIRE),
            thunder_skill_def: SkillDefId::new(LONGNU_THUNDER),
            stage: 0,
            skill: longnu,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 5);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 6);
    assert_eq!(tx.state().hand_count(owner), hand + 1);
    let fire = skill(tx.state(), owner, LONGNU_FIRE);

    run_effects(
        &mut tx,
        vec![
            EffectRequest::RemoveSkill { skill: fire },
            EffectRequest::ResolveNzryLongnu {
                player: owner,
                fire_skill_def: SkillDefId::new(LONGNU_FIRE),
                thunder_skill_def: SkillDefId::new(LONGNU_THUNDER),
                stage: 0,
                skill: longnu,
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 5);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 5);
    assert_eq!(tx.state().hand_count(owner), hand + 2);
    assert_eq!(
        tx.state()
            .skills
            .get(skill(tx.state(), owner, LONGNU_THUNDER))
            .unwrap()
            .def
            .raw(),
        LONGNU_THUNDER
    );
}

#[test]
fn thunder_longnu_can_use_multiple_converted_sha() {
    let (state, players) = setup(2);
    let owner = players[0];
    let target = players[1];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, target);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(LONGNU_THUNDER),
        }],
    )
    .unwrap();
    let helper = skill(tx.state(), owner, LONGNU_THUNDER);
    let first = inject_trick(&mut tx, owner, 7);
    let second = inject_trick(&mut tx, owner, 8);
    let hp = tx.state().players.get(target).unwrap().hp;

    let (state, first_result) = activate(tx.state().clone(), owner, helper, target, first, 10);
    assert!(matches!(
        first_result.result,
        CommandResult::Accepted { .. }
    ));
    let (state, second_result) = activate(state, owner, helper, target, second, 11);
    assert!(matches!(
        second_result.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 2);
}

#[test]
fn jieying_prevents_unlink_adds_two_once_and_forces_an_unlinked_target() {
    let (state, players) = setup(3);
    let owner = players[0];
    let linked = players[1];
    let target = players[2];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    for player in &players {
        tx.state_mut()
            .players
            .get_mut(*player)
            .unwrap()
            .marks
            .retain(|(mark, _)| *mark != engine_core::content::MARK_CHAIN);
    }
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JIEYING),
            },
            EffectRequest::AddSkill {
                player: target,
                skill_def: SkillDefId::new(JIEYING),
            },
            EffectRequest::AddMark {
                player: owner,
                mark: engine_api::MarkDefId::new(engine_core::content::MARK_CHAIN),
                delta: 1,
            },
            EffectRequest::AddMark {
                player: linked,
                mark: engine_api::MarkDefId::new(engine_core::content::MARK_CHAIN),
                delta: 1,
            },
        ],
    )
    .unwrap();
    let base = tx.state().players.get(linked).unwrap().hp.max(0);
    assert_eq!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: linked },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(base + 2)
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::RemoveMark {
            player: owner,
            mark: engine_api::MarkDefId::new(engine_core::content::MARK_CHAIN),
            amount: 0,
        }],
    )
    .unwrap();
    assert!(tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0));

    let jieying = skill(tx.state(), owner, JIEYING);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ChooseLinkOther {
                player: owner,
                skill: jieying,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseLinkOther
    );
    let state = tx.state().clone();
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(12),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![target.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state
        .players
        .get(target)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0));
}
