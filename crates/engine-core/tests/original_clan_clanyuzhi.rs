//! Golden coverage for clan 迂志 (clanyuzhi): locked clan skill driven by the new
//! round_started / round_ended engine events (emitted at the seat-0 turn rollover).
//!
//! Round start: the owner chooses one hand card → reveal + gaintag + draw its name
//! length (tagging the draws) + record the round draw counts in per-skill keyed
//! storage. Round end: discard every gaintagged hand card; if this round's 迂志 draws
//! exceeded the previous round's (both positive) or the round's used-card count, the
//! owner loses 保族 or takes 1 thunder damage.
//!
//! Storage keys (see `clanyuzhi` primitives in pack_refresh.rs): 1 = drawn this
//! round, 2 = drawn last round, 3 = name-length mark.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const CLANYUZHI: u32 = 2_443_866_463;
/// 保族 (clanbaozu) — the clan skill 迂志 can remove at round end.
const CLANBAOZU: u32 = 3_600_751_244;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/clan/clan.npack.json");

const YUZHI_KEY_DRAWN_ROUND: u32 = 1;
const YUZHI_KEY_DRAWN_LAST: u32 = 2;

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_811,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn strip_and_add_clanyuzhi(tx: &mut Transaction, owner: PlayerId) -> engine_api::SkillId {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(CLANYUZHI),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == CLANYUZHI)
        .unwrap()
        .id
}

fn add_clanbaozu(tx: &mut Transaction, owner: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(CLANBAOZU),
        }],
    )
    .unwrap();
}

fn inject_hand(
    state: &mut engine_core::state::GameState,
    player: PlayerId,
    kind: CardKind,
) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_100 + kind.tag() as u32),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn dispatch_round_event(
    tx: &mut Transaction,
    event: DomainEvent,
) -> Result<(), engine_core::effect::EffectError> {
    let effects = dispatch_for_events(tx, &[event], &mut TriggerFireRecord::new())?;
    match run_effects(tx, effects) {
        Ok(_) | Err(engine_core::effect::EffectError::ChoiceRequested) => Ok(()),
        Err(e) => Err(e),
    }
}

fn resolve_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    idempotency_key: IdempotencyKey,
    selections: Vec<u32>,
) -> (engine_core::state::GameState, CommandResult) {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, pipeline) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key,
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    (state, pipeline.result)
}

#[test]
fn committed_package_publishes_clanyuzhi_round_triggers() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let triggers: Vec<_> = pkg
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == CLANYUZHI)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert!(triggers
        .iter()
        .any(|trigger| trigger.event == "round_started" && trigger.filter == "owner_alive"));
    assert!(triggers
        .iter()
        .any(|trigger| trigger.event == "round_ended" && trigger.filter == "owner_alive"));
}

#[test]
fn round_start_choice_reveals_and_draws_name_length_tagged() {
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = strip_and_add_clanyuzhi(&mut tx, owner);
    // A 决斗 (name length 2) in hand; the round-start choice must offer it.
    let juedou = inject_hand(tx.state_mut(), owner, CardKind::TrickJuedou);

    // Round rolls to round 2 with the owner first: the round_started event fires the
    // trigger, which opens the hand-card choice.
    dispatch_round_event(
        &mut tx,
        DomainEvent::RoundStarted {
            round: 2,
            current: owner,
        },
    )
    .unwrap();
    assert!(
        tx.state().has_pending_choice(),
        "迂志 reveal choice must open at round start"
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ClanyuzhiRevealChoice
    );

    // Stage 0: pick the 决斗 → reveal + gaintag + draw 2 (tagged) + storage.
    let (state, result) = resolve_choice(
        tx.clone().commit().state,
        owner,
        IdempotencyKey(11),
        vec![juedou.raw()],
    );
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert!(
        state.card_has_content_tag(juedou, CLANYUZHI),
        "chosen card must be gaintagged"
    );
    let drawn: Vec<_> = state
        .hand_cards(owner)
        .into_iter()
        .filter(|card| *card != juedou && state.card_has_content_tag(*card, CLANYUZHI))
        .collect();
    assert_eq!(drawn.len(), 2, "name length 2 → exactly 2 tagged draws");
    let instance = state.skills.get(skill).unwrap();
    assert_eq!(
        instance.content_storage.get(&YUZHI_KEY_DRAWN_ROUND),
        Some(&2),
        "this round's 迂志 draw count recorded"
    );
    assert_eq!(
        instance.content_storage.get(&YUZHI_KEY_DRAWN_LAST),
        Some(&0),
        "previous round's count rotates in as 0"
    );
}

#[test]
fn round_end_discards_tagged_and_takes_thunder_damage_when_ambitious() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = strip_and_add_clanyuzhi(&mut tx, owner);
    // 保族 present → the ambition check offers the choice (lose 保族 or damage).
    add_clanbaozu(&mut tx, owner);
    // A tagged card in hand (as if revealed at round start).
    let tagged = inject_hand(tx.state_mut(), owner, CardKind::Sha);
    tx.state_mut().add_content_card_tag(tagged, CLANYUZHI);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    // This round drew 2, previous round drew 1 → ambitious (2 > 1 > 0).
    tx.state_mut()
        .skills
        .storage_set(skill, YUZHI_KEY_DRAWN_ROUND, 2);
    tx.state_mut()
        .skills
        .storage_set(skill, YUZHI_KEY_DRAWN_LAST, 1);

    dispatch_round_event(
        &mut tx,
        DomainEvent::RoundEnded {
            round: 1,
            current: other,
        },
    )
    .unwrap();
    // The tagged card was discarded before the ambition choice.
    assert!(
        !tx.state().hand_cards(owner).contains(&tagged),
        "tagged card discarded at round end"
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ClanyuzhiEndChoice,
        "ambition choice opens when draws exceed previous round"
    );

    // Choice: 1 = take 1 thunder damage.
    let (state, result) = resolve_choice(
        tx.clone().commit().state,
        owner,
        IdempotencyKey(21),
        vec![1],
    );
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        owner_hp - 1,
        "owner takes 1 thunder damage"
    );
}

#[test]
fn round_end_without_ambition_is_noop() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_and_add_clanyuzhi(&mut tx, owner);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    // No tagged cards, no draws recorded → nothing happens.
    dispatch_round_event(
        &mut tx,
        DomainEvent::RoundEnded {
            round: 1,
            current: other,
        },
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().players.get(owner).unwrap().hp, owner_hp);
}

#[test]
fn round_end_without_baozu_deals_thunder_damage_directly() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = strip_and_add_clanyuzhi(&mut tx, owner);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    tx.state_mut()
        .skills
        .storage_set(skill, YUZHI_KEY_DRAWN_ROUND, 2);
    tx.state_mut()
        .skills
        .storage_set(skill, YUZHI_KEY_DRAWN_LAST, 1);

    dispatch_round_event(
        &mut tx,
        DomainEvent::RoundEnded {
            round: 1,
            current: other,
        },
    )
    .unwrap();
    // No 保族 → upstream takes the damage directly without a choice.
    assert!(!tx.state().has_pending_choice());
    assert_eq!(
        tx.state().players.get(owner).unwrap().hp,
        owner_hp - 1,
        "owner takes 1 thunder damage without a choice"
    );
}

#[test]
fn round_end_can_remove_clanbaozu_instead_of_damage() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = strip_and_add_clanyuzhi(&mut tx, owner);
    add_clanbaozu(&mut tx, owner);
    assert!(tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .any(|instance| instance.def.raw() == CLANBAOZU));
    tx.state_mut()
        .skills
        .storage_set(skill, YUZHI_KEY_DRAWN_ROUND, 2);
    tx.state_mut()
        .skills
        .storage_set(skill, YUZHI_KEY_DRAWN_LAST, 1);

    dispatch_round_event(
        &mut tx,
        DomainEvent::RoundEnded {
            round: 1,
            current: other,
        },
    )
    .unwrap();
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ClanyuzhiEndChoice
    );

    // Choice: 0 = lose 保族; HP untouched.
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let (state, result) = resolve_choice(
        tx.clone().commit().state,
        owner,
        IdempotencyKey(31),
        vec![0],
    );
    assert!(matches!(result, CommandResult::Accepted { .. }));
    assert!(
        !state
            .skills
            .sorted_live()
            .iter()
            .any(|instance| instance.def.raw() == CLANBAOZU),
        "保族 removed"
    );
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp);
}
