//! Golden for MOBILE 韩遂「骁袭」的主动与响应【杀】转换。

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const XIAOXI_HANSUI: u32 = 2_531_367_472;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn package_engine(seed: u64) -> Engine {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    engine
}

fn add_skill(tx: &mut Transaction, owner: PlayerId) -> SkillId {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XIAOXI_HANSUI),
        }],
    )
    .unwrap();
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| {
            tx.state()
                .skills
                .get(*skill)
                .is_some_and(|item| item.def.raw() == XIAOXI_HANSUI)
        })
        .unwrap()
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return;
    }
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

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> engine_api::CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(99_000 + kind.tag() as u32 * 20 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn activate(
    actor: PlayerId,
    skill: SkillId,
    target: PlayerId,
    card: engine_api::CardId,
    key: u64,
) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![card],
        },
    }
}

#[test]
fn package_exposes_both_black_hand_equip_view_as_rules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    for window in ["play_phase", "response"] {
        assert!(package.view_as_index.iter().any(|view| {
            view.skill_def_id == XIAOXI_HANSUI
                && view.as_kind == "sha"
                && view.window == window
                && view.filter == "black"
                && view.source_zone.as_deref() == Some("hand_equip")
        }));
    }
}

#[test]
fn black_equipment_acts_as_proactive_sha_but_red_card_does_not() {
    let engine = package_engine(20_260_722);
    let actor = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != actor)
        .unwrap()
        .id;

    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, target);
    let removals: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, removals).unwrap();
    let skill = add_skill(&mut tx, actor);
    let red = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Heart,
        2,
    );
    let source = inject(
        &mut tx,
        actor,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Club,
        3,
    );

    let (_, pipe) = apply_command(tx.state().clone(), activate(actor, skill, target, red, 1));
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));

    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, pipe) = apply_command(
        tx.state().clone(),
        activate(actor, skill, target, source, 2),
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert_eq!(
        state.cards.get(source).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}

#[test]
fn duel_accepts_black_equipment_as_response_sha() {
    let engine = package_engine(20_260_723);
    let source = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != source)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, source);
    clear_hand(&mut tx, target);
    add_skill(&mut tx, target);
    let duel = inject(
        &mut tx,
        source,
        ZoneKind::Hand,
        CardKind::TrickJuedou,
        Suit::Diamond,
        7,
    );
    let black = inject(
        &mut tx,
        target,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Spade,
        8,
    );
    let red = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Diamond,
        9,
    );

    let (_, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::PlayCard {
                card: duel,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let choice = pipe.choice.expect("骁袭应提供黑色装备牌作为响应杀");
    assert_eq!(choice.observer, target);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == black.raw()));
    assert!(!choice
        .options
        .iter()
        .any(|option| option.token == red.raw()));
}
