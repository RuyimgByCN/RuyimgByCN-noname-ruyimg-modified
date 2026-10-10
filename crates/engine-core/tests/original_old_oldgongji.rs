//! Golden for 旧韩当「弓骑」的装备牌转无距离【杀】与响应语义。

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

const OLDGONGJI: u32 = 4_171_347_410;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/old/old.npack.json");

fn package_engine(seed: u64, player_count: u8) -> Engine {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    engine
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

fn remove_all_skills(tx: &mut Transaction) {
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(tx, effects).unwrap();
}

fn add_skill(tx: &mut Transaction, owner: PlayerId) -> SkillId {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(OLDGONGJI),
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == OLDGONGJI)
        .unwrap()
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
        def: CardDefId::new(97_500 + kind.tag() as u32 * 20 + rank as u32),
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
fn package_exposes_equipment_sha_use_response_and_distance_contracts() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 5);
    for window in ["play_phase", "response"] {
        let view = package
            .view_as_index
            .iter()
            .find(|view| view.skill_def_id == OLDGONGJI && view.window == window)
            .unwrap();
        assert_eq!(view.as_kind, "sha");
        assert_eq!(view.filter, "card_equip");
        assert_eq!(view.source_zone.as_deref(), Some("hand_equip"));
        assert_eq!(view.ignore_distance, window == "play_phase");
    }
}

#[test]
fn hand_equipment_reaches_distance_two_but_non_equipment_is_rejected() {
    let engine = package_engine(20_260_716, 4);
    let actor = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let actor_index = seats.iter().position(|player| *player == actor).unwrap();
    let far = seats[(actor_index + 2) % seats.len()];
    let mut tx = Transaction::begin(engine.state().clone());
    remove_all_skills(&mut tx);
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, far);
    let skill = add_skill(&mut tx, actor);
    let non_equip = inject(
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
        ZoneKind::Hand,
        CardKind::Armor,
        Suit::Club,
        3,
    );

    let (_, pipe) = apply_command(
        tx.state().clone(),
        activate(actor, skill, far, non_equip, 1),
    );
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));

    let hp = tx.state().players.get(far).unwrap().hp;
    let (state, pipe) = apply_command(tx.state().clone(), activate(actor, skill, far, source, 2));
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(far).unwrap().hp, hp - 1);
}

#[test]
fn duel_offers_equipment_zone_card_as_response_sha() {
    let engine = package_engine(20_260_717, 2);
    let source = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != source)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    remove_all_skills(&mut tx);
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
    let equipment = inject(
        &mut tx,
        target,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Spade,
        8,
    );
    let non_equip = inject(
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
    let choice = pipe.choice.expect("弓骑应提供装备牌作为响应杀");
    assert_eq!(choice.observer, target);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == equipment.raw()));
    assert!(!choice
        .options
        .iter()
        .any(|option| option.token == non_equip.raw()));
}
