//! Golden coverage for JSRG 居下 / 绝禋 / 逆乱.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const JUXIA: u32 = 3_771_277_794;
const JUEYIN: u32 = 2_928_968_300;
const NILUAN: u32 = 3_518_365_032;
const JUXIA_SRC: &str =
    include_str!("../../../../content/original/ported/jsrg/jsrg_new_juxia.skill");
const JUEYIN_SRC: &str = include_str!("../../../../content/original/ported/jsrg/jsrgjueyin.skill");
const NILUAN_SRC: &str = include_str!("../../../../content/original/ported/jsrg/jsrgniluan.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/jsrg/jsrg.npack.json");

fn bootstrap(source: &str, name: &str, skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(skill_def), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
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
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 8,
        weapon_range: 1,
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
fn juxia_source_and_npack_publish_targeted_draw() {
    let compiled = content_compiler::compile_source_named("jsrg_new_juxia", JUXIA_SRC).unwrap();
    assert!(compiled.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JUXIA && trigger.event == "card_targeted" && trigger.optional
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == JUXIA && trigger.event == "card_targeted"));
}

#[test]
fn juxia_draws_two_when_targeted_by_other() {
    let (mut tx, owner, other) = bootstrap(JUXIA_SRC, "juxia-test", JUXIA);
    let card = inject_hand(&mut tx, other, 191_101);
    let before = tx.state().hand_count(owner);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: other,
                card,
                targets: vec![owner],
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.hand_count(owner), before + 2);
}

#[test]
fn jueyin_source_and_npack_publish_first_damage_draw() {
    let compiled = content_compiler::compile_source_named("jsrgjueyin", JUEYIN_SRC).unwrap();
    assert!(compiled.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JUEYIN
            && trigger.event == "damage"
            && trigger.filter == "owner_is_target_first_damage"
            && trigger.optional
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == JUEYIN && trigger.filter == "owner_is_target_first_damage"
    }));
}

#[test]
fn jueyin_draws_three_after_first_damage() {
    let (tx, owner, other) = bootstrap(JUEYIN_SRC, "jueyin-test", JUEYIN);
    let before = tx.state().hand_count(owner);
    let mut tx = tx;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 3);
    assert_eq!(state.hand_count(owner), before + 3);
}

#[test]
fn niluan_source_and_npack_publish_prepare_damage() {
    let compiled = content_compiler::compile_source_named("jsrgniluan", NILUAN_SRC).unwrap();
    assert!(compiled.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == NILUAN && trigger.event == "phase_prepare" && trigger.optional
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| { trigger.skill_def_id == NILUAN && trigger.event == "phase_prepare" }));
}

#[test]
fn niluan_prepare_can_damage_other() {
    let (tx, owner, other) = bootstrap(NILUAN_SRC, "niluan-test", NILUAN);
    let other_hp = tx.state().players.get(other).unwrap().hp;
    let mut tx = tx;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::Prepare,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 4);
    assert!(state.has_pending_choice());
    let state = resolve(state, owner, vec![other.raw()], 5);
    assert_eq!(state.players.get(other).unwrap().hp, other_hp - 1);
}
