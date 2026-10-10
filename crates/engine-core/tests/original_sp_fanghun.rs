//! Golden for sp 梅影 (fanghun): damage grants 「梅影」 marks; each mark may be
//! spent to convert a hand/equip card into 【桃】/【酒】 with a 1-card draw.
//!
//! Exercises the `cost hand_equip_mark mark_cost` view_as deep variant
//! (mark + source-card cost, `post_draw`), and the damage → `add_mark` trigger.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, MarkDefId, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const FANGHUN: u32 = 3_479_603_365;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_261_401,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(result, CommandResult::Accepted { .. }));
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

fn add_fanghun(tx: &mut Transaction, owner: PlayerId) -> engine_api::SkillId {
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
        skill_def: SkillDefId::new(FANGHUN),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == FANGHUN)
        .unwrap()
        .id
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
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
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn marks_of(tx: &Transaction, player: PlayerId) -> i32 {
    tx.state()
        .players
        .get(player)
        .and_then(|p| p.marks.iter().find(|(id, _)| *id == FANGHUN))
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[test]
fn package_declares_fanghun_damage_trigger_and_mark_cost_view_as() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == FANGHUN && t.event == "damage"
    }));
    for kind in ["tao", "jiu"] {
        let view = pkg
            .view_as_index
            .iter()
            .find(|v| v.skill_def_id == FANGHUN && v.as_kind == kind)
            .unwrap();
        assert_eq!(view.source_zone.as_deref(), Some("none"));
        assert_eq!(view.cost.as_deref(), Some("hand_equip_mark"));
        assert_eq!(view.mark_cost, Some(FANGHUN));
        assert_eq!(view.post_draw, 1);
    }
}

#[test]
fn damage_grants_marks_equal_to_amount() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_fanghun(&mut tx, owner);
    assert_eq!(marks_of(&tx, owner), 0);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap_or_else(|e| panic!("damage error: {e:?}"));

    assert_eq!(marks_of(&tx, owner), 2, "damage grants 2 梅影 marks");
}

#[test]
fn spending_a_mark_converts_a_hand_card_to_tao_and_draws_one() {
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = add_fanghun(&mut tx, owner);
    let source = inject_hand(&mut tx, owner, CardKind::Shan);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3; // wounded → 桃 usable
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: owner,
            mark: MarkDefId::new(FANGHUN),
            delta: 2,
        }],
    )
    .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let hand_before = tx.state().hand_count(owner);

    // Use 桃 via 梅影: mark is consumed and a source-card choice opens.
    let (state, first) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::Tao.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        },
    );
    assert!(
        matches!(first.result, CommandResult::Accepted { .. }),
        "梅影 桃 conversion accepted: {:?}",
        first.result
    );
    let source_choice = first
        .choice
        .expect("hand_equip_mark opens the source-card choice");
    assert_eq!(source_choice.observer, owner);
    assert!(
        source_choice
            .options
            .iter()
            .any(|option| option.token == source.raw()),
        "the hand card is offered as the conversion source"
    );

    // Pick the source card → 桃 resolves (heal 1) + post_draw 1.
    let (state, healed) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: source_choice.choice_id,
                selections: vec![source.raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(healed.result, CommandResult::Accepted { .. }),
        "source pick accepted: {:?}",
        healed.result
    );
    let mut tx = Transaction::begin(state);
    assert_eq!(
        marks_of(&tx, owner),
        1,
        "one 梅影 mark spent for the conversion"
    );
    assert_eq!(
        tx.state().players.get(owner).unwrap().hp,
        4,
        "the converted 桃 healed the owner"
    );
    assert_eq!(
        tx.state().hand_count(owner),
        hand_before,
        "source card consumed (-1) + post_draw 1 = unchanged hand size"
    );
    let _ = DomainEvent::MarkChanged {
        player: owner,
        mark_def: FANGHUN,
        count: 1,
    };
}
