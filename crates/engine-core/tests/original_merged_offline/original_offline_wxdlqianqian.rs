//! Golden coverage for OFFLINE 文心雕龙「慊慊」(wxdlqianqian).
//! First card the owner uses each turn cannot be a heart. After any successful
//! use, the lock is withdrawn until the current turn ends.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{card_use_cancelled_for_card, query, QueryContext};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const WXDLQIANQIAN: u32 = 3_312_200_979;
const SOURCE: &str = include_str!("../../../../content/original/ported/offline/wxdlqianqian.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

fn setup() -> (GameState, PlayerId, CardId, CardId) {
    let package = content_compiler::compile_source_named("wxdlqianqian-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_815_86, "identity-standard").unwrap();
    let owner = tx.state().turn.current;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::DiscardFromHand {
        player: owner,
        count: u32::MAX,
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(WXDLQIANQIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let heart = inject_hand(tx.state_mut(), owner, Suit::Heart, CardKind::Sha, 170_201);
    let spade = inject_hand(tx.state_mut(), owner, Suit::Spade, CardKind::Sha, 170_202);
    (tx.commit().state, owner, heart, spade)
}

fn inject_hand(
    state: &mut GameState,
    player: PlayerId,
    suit: Suit,
    kind: CardKind,
    def: u32,
) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn compiled_source_and_npack_publish_the_qianqian_contract() {
    let compiled = content_compiler::compile_source_named("wxdlqianqian", SOURCE).unwrap();
    assert!(compiled.skill_index.iter().any(|skill| {
        skill.skill_def_id == WXDLQIANQIAN && skill.name == "wxdlqianqian" && skill.locked
    }));
    assert!(compiled.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == WXDLQIANQIAN
            && trigger.event == "card_used"
            && trigger.filter == "owner_is_source"
            && !trigger.optional
    }));
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == WXDLQIANQIAN && modifier.target.contains("card_use_cancel")
    }));

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == WXDLQIANQIAN && skill.name == "wxdlqianqian"));
    let pack_trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == WXDLQIANQIAN)
        .expect("npack wxdlqianqian trigger");
    assert_eq!(pack_trigger.event, "card_used");
    assert_eq!(pack_trigger.filter, "owner_is_source");
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .expect("npack module for wxdlqianqian");
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn first_card_this_turn_cannot_be_heart_until_another_card_is_used() {
    let (state, owner, heart, spade) = setup();
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    assert!(card_use_cancelled_for_card(tx.state(), owner, heart));
    assert!(!card_use_cancelled_for_card(tx.state(), owner, spade));
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::CardUsable {
                player: owner,
                card: heart
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Bool(false)
    ));
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::CardUsable {
                player: owner,
                card: spade
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Bool(true)
    ));

    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: spade,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(fired.iter().any(|effect| matches!(
        effect,
        EffectRequest::DisableSkillByDefUntilTurnEnd { player, skill_def }
            if *player == owner && skill_def.raw() == WXDLQIANQIAN
    )));
    run_effects(&mut tx, fired).unwrap();
    assert!(!card_use_cancelled_for_card(tx.state(), owner, heart));
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::CardUsable {
                player: owner,
                card: heart
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Bool(true)
    ));
}

#[test]
fn npack_loads_into_a_started_match() {
    let mut engine = engine_core::Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_815_87,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(engine
        .state()
        .content
        .skills()
        .any(|def| def.skill_def.raw() == WXDLQIANQIAN));
}
