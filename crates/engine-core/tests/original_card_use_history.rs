//! Golden tests for legacy skills driven by per-player card-use history.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const NSLONGYUE: u32 = 2_643_354_275;
const NSLONGYUE_AI: u32 = 3_436_136_112;
const JDJIZHI: u32 = 3_847_328_245;
const DCJIANYING: u32 = 2_862_591_465;
const REJIZHI: u32 = 3_401_109_480;
const REQICAI: u32 = 2_792_486_309;
const RE_HUANGYUEYING: u32 = 4_247_764_745;
const DIY: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");
const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn setup(package: &[u8], skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(package).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == skill_def));
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(u64::from(skill_def)),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: u64::from(skill_def),
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    // roster 变动会平移 bootstrap 身份分配（offline 新将批 +3）——排空开局可选
    // 选择，避免后续 run_effects 被挂起 ChoiceRequested 打断（qimen 模式）。
    let mut drain_key = u64::from(skill_def).wrapping_mul(2654435761);
    while engine.state().has_pending_choice() {
        drain_key += 1;
        let choice_id = engine.state().resolution.pending_choice_id;
        let revision = engine.state().revision;
        let observer = engine.state().resolution.pending_observer;
        let _ = engine.submit(Command {
            actor: PlayerId::new(observer),
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(drain_key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice_id),
                selections: vec![],
                cancelled: true,
            },
        });
    }
    let mut tx = Transaction::begin(engine.state().clone());
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
            skill_def: SkillDefId::new(skill_def),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn use_card(
    tx: &mut Transaction,
    player: PlayerId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> Result<(), EffectError> {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(90_000 + kind.tag() as u32 * 16 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    run_effects(
        tx,
        vec![EffectRequest::UseCard {
            player,
            card,
            targets: vec![],
        }],
    )
    .map(|_| ())
}

fn resolve(tx: Transaction, owner: PlayerId, accept: bool, key: u64) -> Transaction {
    let state = tx.state();
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: accept.then_some(1).into_iter().collect(),
                cancelled: !accept,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    Transaction::begin(state)
}

fn hand_limit(tx: &Transaction, player: PlayerId) -> i32 {
    match query(
        tx.state(),
        &QueryRequest::HandLimit { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("expected hand-limit integer, got {other:?}"),
    }
}

#[test]
fn refresh_package_closes_re_huangyueying_with_physical_rejizhi() {
    let package = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 723);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == RE_HUANGYUEYING)
            .unwrap()
            .skills,
        [REJIZHI, REQICAI]
    );
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == REQICAI));
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == REJIZHI)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert!(triggers.iter().any(|trigger| {
        trigger.event == "card_used"
            && trigger.filter == "owner_source_physical_ordinary_trick"
            && trigger.optional
    }));
    assert!(triggers.iter().any(|trigger| {
        trigger.event == "turn_ended" && trigger.filter == "owner_alive" && !trigger.optional
    }));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == REJIZHI
            && modifier.target == "hand_limit"
            && modifier.mode == "add"
            && modifier.value_source.as_deref() == Some("owner_skill_mark")
    }));
}

#[test]
fn rejizhi_discards_the_basic_draw_for_hand_limit_until_turn_end() {
    let (mut tx, owner, other) = setup(REFRESH, REJIZHI);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let drawn = *tx.state().zones.get(draw).unwrap().cards.last().unwrap();
    tx.state_mut().cards.get_mut(drawn).unwrap().kind = CardKind::Sha;
    let base_limit = hand_limit(&tx, owner);

    assert_eq!(
        use_card(&mut tx, owner, CardKind::TrickWuzhong, Suit::Heart, 7),
        Err(EffectError::ChoiceRequested)
    );
    let tx = resolve(tx, owner, true, 10);
    assert!(tx.state().has_pending_choice());
    let mut tx = resolve(tx, owner, true, 11);
    assert!(!tx.state().hand_cards(owner).contains(&drawn));
    assert_eq!(hand_limit(&tx, owner), base_limit + 1);
    assert_eq!(
        tx.state()
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .find(|(mark, _)| *mark == REJIZHI)
            .map(|(_, count)| *count),
        Some(1)
    );

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
    assert_eq!(hand_limit(&tx, owner), base_limit);
}

#[test]
fn nslongyue_only_offers_for_each_players_first_ordinary_trick() {
    let package = content_schema::ContentPackage::from_json_bytes(DIY).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NSLONGYUE_AI));
    assert!(package
        .trigger_index
        .iter()
        .all(|trigger| trigger.skill_def_id != NSLONGYUE_AI));

    let (mut tx, owner, source) = setup(DIY, NSLONGYUE);
    let initial = tx.state().hand_count(source);
    let paused = use_card(&mut tx, source, CardKind::TrickWuzhong, Suit::Spade, 7).unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    let mut tx = resolve(tx, owner, true, 1);
    assert_eq!(tx.state().hand_count(source), initial + 1);
    use_card(&mut tx, source, CardKind::TrickGuohe, Suit::Heart, 8).unwrap();
    assert!(!tx.state().has_pending_choice());

    let (mut tx, _, source) = setup(DIY, NSLONGYUE);
    use_card(&mut tx, source, CardKind::TrickLe, Suit::Club, 6).unwrap();
    use_card(&mut tx, source, CardKind::TrickWuzhong, Suit::Diamond, 9).unwrap();
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn jdjizhi_draws_and_uses_its_full_u32_skill_id_as_a_turn_mark() {
    let (mut tx, owner, current) = setup(OFFLINE, JDJIZHI);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current,
            phase: Phase::Play.to_u8(),
        }],
    )
    .unwrap();
    let initial_hand = tx.state().hand_count(owner);
    let base_limit = hand_limit(&tx, owner);
    use_card(&mut tx, owner, CardKind::TrickWuzhong, Suit::Heart, 7).unwrap();
    assert_eq!(tx.state().hand_count(owner), initial_hand + 1);
    assert_eq!(hand_limit(&tx, owner), base_limit + 1);
    assert_eq!(
        tx.state()
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .find(|(mark, _)| *mark == JDJIZHI)
            .map(|(_, count)| *count),
        Some(1)
    );
    use_card(&mut tx, owner, CardKind::TrickLe, Suit::Spade, 8).unwrap();
    assert_eq!(tx.state().hand_count(owner), initial_hand + 1);
    assert_eq!(hand_limit(&tx, owner), base_limit + 1);

    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Prepare.to_u8(),
            },
        ],
    )
    .unwrap();
    assert_eq!(hand_limit(&tx, owner), base_limit);
}

#[test]
fn dcjianying_compares_with_the_previous_used_card_across_turns() {
    let (mut tx, owner, other) = setup(REFRESH, DCJIANYING);
    let initial = tx.state().hand_count(owner);
    use_card(&mut tx, owner, CardKind::Other, Suit::Spade, 7).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: other,
                phase: Phase::Prepare.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Play.to_u8(),
            },
        ],
    )
    .unwrap();
    let paused = use_card(&mut tx, owner, CardKind::Other, Suit::Heart, 7).unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    let mut tx = resolve(tx, owner, true, 2);
    assert_eq!(tx.state().hand_count(owner), initial + 1);

    let paused = use_card(&mut tx, owner, CardKind::Other, Suit::Heart, 8).unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    let tx = resolve(tx, owner, false, 3);
    assert_eq!(tx.state().hand_count(owner), initial + 1);
}
