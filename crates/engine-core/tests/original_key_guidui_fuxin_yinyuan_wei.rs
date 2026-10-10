//! Golden coverage for KEY 归队 / 复心 / 姻缘 / 魏势 rewritten from honest noops.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const KENGO_GUIDUI: u32 = 3_185_675_503;
const NAGISA_FUXIN: u32 = 2_871_163_155;
const SEIRA_YINYUAN: u32 = 4_047_635_802;
const KOTORI_SKILL_WEI: u32 = 3_333_120_229;

const GUIDUI_SRC: &str = include_str!("../../../content/original/ported/key/kengo_guidui.skill");
const FUXIN_SRC: &str = include_str!("../../../content/original/ported/key/nagisa_fuxin.skill");
const YINYUAN_SRC: &str = include_str!("../../../content/original/ported/key/seira_yinyuan.skill");
const WEI_SRC: &str = include_str!("../../../content/original/ported/key/kotori_skill_wei.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn bootstrap(source: &str, name: &str, skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(skill_def), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
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
fn guidui_source_and_npack_publish_prepare_restore() {
    let compiled = content_compiler::compile_source_named("kengo_guidui", GUIDUI_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == KENGO_GUIDUI)
        .expect("guidui trigger");
    assert_eq!(trigger.event, "phase_prepare");
    assert_eq!(trigger.filter, "owner_is_target");
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == KENGO_GUIDUI && trigger.event == "phase_prepare"
    }));
}

#[test]
fn guidui_prepare_restores_disabled_slots() {
    let (mut tx, owner, _) = bootstrap(GUIDUI_SRC, "guidui-test", KENGO_GUIDUI);
    tx.state_mut()
        .players
        .get_mut(owner)
        .unwrap()
        .disabled_equip_slots = [1, 1, 1, 1, 1];
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("prepare dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    assert!(tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .disabled_equip_slots
        .iter()
        .all(|slot| *slot == 0));
}

#[test]
fn fuxin_source_and_npack_publish_other_damage_draw() {
    let compiled = content_compiler::compile_source_named("nagisa_fuxin", FUXIN_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NAGISA_FUXIN)
        .expect("fuxin trigger");
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_alive_not_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == NAGISA_FUXIN && trigger.filter == "owner_alive_not_target"
    }));
}

#[test]
fn fuxin_lets_damaged_other_draw_one() {
    let (tx, owner, other) = bootstrap(FUXIN_SRC, "fuxin-test", NAGISA_FUXIN);
    let other_hand = tx.state().hand_count(other);
    let mut tx = tx;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::DamageDealt {
                source: owner,
                target: other,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.hand_count(other), other_hand + 1);
}

#[test]
fn yinyuan_source_and_npack_publish_active_gain() {
    let compiled = content_compiler::compile_source_named("seira_yinyuan", YINYUAN_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == SEIRA_YINYUAN));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == SEIRA_YINYUAN));
}

#[test]
fn wei_source_and_npack_publish_turn_begin_recast() {
    let compiled = content_compiler::compile_source_named("kotori_skill_wei", WEI_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == KOTORI_SKILL_WEI)
        .expect("wei trigger");
    assert_eq!(trigger.event, "phase_begin");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == KOTORI_SKILL_WEI && trigger.event == "phase_begin"
    }));
}

#[test]
fn wei_turn_begin_discards_one_and_draws_one() {
    let (mut tx, owner, _) = bootstrap(WEI_SRC, "wei-test", KOTORI_SKILL_WEI);
    inject_hand(&mut tx, owner, 333_101);
    let before = tx.state().hand_count(owner);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnBegan { player: owner }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 3);
    assert_eq!(state.hand_count(owner), before);
}
