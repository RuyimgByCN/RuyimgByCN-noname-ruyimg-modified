//! Golden coverage for KEY 幻界 / 神视 / 断罚 / 涉猎 rewritten from honest noops.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const KEY_HUANJIE: u32 = 2_802_218_609;
const HINA_SHENSHI: u32 = 3_019_937_721;
const KYOU_DUANFA: u32 = 2_857_414_296;
const KYOKO_SHELIE: u32 = 3_597_507_893;

const HUANJIE_SRC: &str = include_str!("../../../../content/original/ported/key/key_huanjie.skill");
const SHENSHI_SRC: &str =
    include_str!("../../../../content/original/ported/key/hina_shenshi.skill");
const DUANFA_SRC: &str = include_str!("../../../../content/original/ported/key/kyou_duanfa.skill");
const SHELIE_SRC: &str = include_str!("../../../../content/original/ported/key/kyoko_shelie.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/key/key.npack.json");

fn bootstrap(source: &str, name: &str, skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(skill_def), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn inject_hand(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Shan,
        suit: Suit::Club,
        rank: 6,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}

#[test]
fn huanjie_source_and_npack_publish_draw_from_bottom() {
    let compiled = content_compiler::compile_source_named("key_huanjie", HUANJIE_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == KEY_HUANJIE)
        .expect("huanjie trigger");
    assert_eq!(trigger.event, "phase_draw");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| { trigger.skill_def_id == KEY_HUANJIE && trigger.event == "phase_draw" }));
}

#[test]
fn huanjie_draw_phase_sets_bottom_flag() {
    let (mut tx, owner, _) = bootstrap(HUANJIE_SRC, "huanjie-test", KEY_HUANJIE);
    tx.state_mut().turn.phase = Phase::Draw;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("draw dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().turn.flag(turn_flag::DRAW_PHASE_FROM_BOTTOM), 1);
}

#[test]
fn shenshi_source_and_npack_publish_play_phase_draws() {
    let compiled = content_compiler::compile_source_named("hina_shenshi", SHENSHI_SRC).unwrap();
    let events: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == HINA_SHENSHI)
        .map(|trigger| trigger.event.as_str())
        .collect();
    assert!(events.contains(&"phase_play"));
    assert!(events.contains(&"phase_play_end"));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| { trigger.skill_def_id == HINA_SHENSHI && trigger.event == "phase_play" }));
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == HINA_SHENSHI && trigger.event == "phase_play_end"
    }));
}

#[test]
fn shenshi_play_begin_and_end_draw_two() {
    let (tx, owner, _) = bootstrap(SHENSHI_SRC, "shenshi-test", HINA_SHENSHI);
    let before = tx.state().hand_count(owner);
    let mut tx = tx;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::Play,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.hand_count(owner), before + 2);

    let mut tx = Transaction::begin(state);
    let mid = tx.state().hand_count(owner);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::PhaseEnded {
                current: owner,
                phase: TurnPhaseTag::Play,
                damage_sources: 0,
                damaged_players: 0,
                card_targeted_other: false,
                current_dealt_damage_other: false,
                phase_targeted_players: 0,
                phase_hp_changed_players: 0,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 3);
    assert_eq!(state.hand_count(owner), mid + 2);
}

#[test]
fn duanfa_source_and_npack_publish_damage_before() {
    let compiled = content_compiler::compile_source_named("kyou_duanfa", DUANFA_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == KYOU_DUANFA)
        .expect("duanfa trigger");
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == KYOU_DUANFA && trigger.event == "damage_before"
    }));
}

#[test]
fn duanfa_prevents_lethal_damage_and_recovers() {
    let (mut tx, owner, other) = bootstrap(DUANFA_SRC, "duanfa-test", KYOU_DUANFA);
    inject_hand(&mut tx, owner, 285_101);
    inject_hand(&mut tx, owner, 285_102);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: owner,
                amount: 2,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 4);
    assert_eq!(state.hand_count(owner), 0);
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        hp_before + 1,
        "damage prevented then recover 1"
    );
    assert!(!state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.def.raw() == KYOU_DUANFA));
}

#[test]
fn shelie_source_and_npack_publish_distinct_suit_draw() {
    let compiled = content_compiler::compile_source_named("kyoko_shelie", SHELIE_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == KYOKO_SHELIE)
        .expect("shelie trigger");
    assert_eq!(trigger.event, "phase_draw");
    assert_eq!(trigger.filter, "owner_is_target");
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| { trigger.skill_def_id == KYOKO_SHELIE && trigger.event == "phase_draw" }));
}

#[test]
fn shelie_draw_phase_zeroes_and_requests_suit_selection() {
    let (mut tx, owner, _) = bootstrap(SHELIE_SRC, "shelie-test", KYOKO_SHELIE);
    tx.state_mut().turn.phase = Phase::Draw;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("shelie dispatch error: {e:?}"));
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(tx.state().turn.flag(turn_flag::DRAW_PHASE_DRAW_ZEROED), 1);
}
