//! Golden coverage for KEY 三礼 / 风阀 / 绝义 / 烈音 rewritten from honest noops.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::effect::run_effects;
use engine_core::state::{turn_flag, Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const LIYINGXIA_SANLI: u32 = 3_091_825_769;
const MIA_FENGFA: u32 = 2_660_197_008;
const YOSHINO_JUEYI: u32 = 3_841_113_685;
const YUI_LIEYIN: u32 = 2_976_604_975;

const SANLI_SRC: &str =
    include_str!("../../../../content/original/ported/key/liyingxia_sanli.skill");
const FENGFA_SRC: &str = include_str!("../../../../content/original/ported/key/mia_fengfa.skill");
const JUEYI_SRC: &str = include_str!("../../../../content/original/ported/key/yoshino_jueyi.skill");
const LIEYIN_SRC: &str = include_str!("../../../../content/original/ported/key/yui_lieyin.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/key/key.npack.json");

fn setup(source: &str, name: &str, skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_815_70),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_815_70,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&compiled).unwrap();
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn inject_hand(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    suit: Suit,
    def: u32,
) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn sanli_source_and_npack_publish_targeted_draw() {
    let compiled = content_compiler::compile_source_named("liyingxia_sanli", SANLI_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == LIYINGXIA_SANLI)
        .expect("sanli trigger");
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(trigger.filter, "owner_targeted_by_other_card_once_per_turn");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == LIYINGXIA_SANLI && trigger.event == "card_targeted"
    }));
}

#[test]
fn sanli_draws_one_when_targeted_by_other() {
    let (mut tx, owner, other) = setup(SANLI_SRC, "sanli-test", LIYINGXIA_SANLI);
    let card = inject_hand(&mut tx, other, CardKind::Sha, Suit::Spade, 309_101);
    let before = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: other,
            card,
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

#[test]
fn fengfa_source_and_npack_publish_draw_bonus() {
    let compiled = content_compiler::compile_source_named("mia_fengfa", FENGFA_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MIA_FENGFA)
        .expect("fengfa trigger");
    assert_eq!(trigger.event, "phase_draw");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == MIA_FENGFA && trigger.event == "phase_draw"));
}

#[test]
fn fengfa_increases_current_draw_count_by_one() {
    let (mut tx, owner, _) = setup(FENGFA_SRC, "fengfa-test", MIA_FENGFA);
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
    assert_eq!(tx.state().turn.flag(turn_flag::CURRENT_DRAW_COUNT_BONUS), 1);
}

#[test]
fn jueyi_source_and_npack_publish_outgoing_plus_one() {
    let compiled = content_compiler::compile_source_named("yoshino_jueyi", JUEYI_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == YOSHINO_JUEYI)
        .expect("jueyi trigger");
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(trigger.filter, "owner_source_other_damage_before");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == YOSHINO_JUEYI && trigger.event == "damage_before"
    }));
}

#[test]
fn jueyi_increases_outgoing_damage_by_one() {
    let (tx, owner, other) = setup(JUEYI_SRC, "jueyi-test", YOSHINO_JUEYI);
    let other_hp = tx.state().players.get(other).unwrap().hp;
    let mut tx = tx;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(other).unwrap().hp, other_hp - 2);
}

#[test]
fn lieyin_source_and_npack_publish_red_as_sha() {
    let compiled = content_compiler::compile_source_named("yui_lieyin", LIEYIN_SRC).unwrap();
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == YUI_LIEYIN
            && modifier.target == "card_name"
            && modifier.selector.as_deref() == Some("red")
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .modifier_index
        .iter()
        .any(|modifier| { modifier.skill_def_id == YUI_LIEYIN && modifier.target == "card_name" }));
}

#[test]
fn lieyin_red_tao_is_effective_sha() {
    let (mut tx, owner, _) = setup(LIEYIN_SRC, "lieyin-test", YUI_LIEYIN);
    let heart = inject_hand(&mut tx, owner, CardKind::Tao, Suit::Heart, 297_601);
    let physical = tx.state().cards.get(heart).unwrap();
    assert_eq!(physical.kind, CardKind::Tao);
    assert_eq!(
        engine_core::query::effective_card_kind(tx.state(), owner, physical),
        CardKind::Sha
    );
    let club = inject_hand(&mut tx, owner, CardKind::Shan, Suit::Club, 297_602);
    let physical = tx.state().cards.get(club).unwrap();
    assert_eq!(
        engine_core::query::effective_card_kind(tx.state(), owner, physical),
        CardKind::Shan
    );
}
