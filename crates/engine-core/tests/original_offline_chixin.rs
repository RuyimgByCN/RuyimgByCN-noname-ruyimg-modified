//! Goldens for OFFLINE「赤心」的两个方片牌转化子技能。

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

const CHIXIN1: u32 = 4_208_743_202;
const CHIXIN2: u32 = 2_383_513_879;
const YYYAZHEN: u32 = 3_406_865_651;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

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

fn add_skill(tx: &mut Transaction, owner: PlayerId, def: u32) -> SkillId {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(def),
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == def)
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

fn clear_skills(tx: &mut Transaction) {
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(tx, effects).unwrap();
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
        def: CardDefId::new(99_800 + kind.tag() as u32 * 20 + rank as u32),
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
fn package_exposes_diamond_hand_equip_sha_and_shan_rules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    for window in ["play_phase", "response"] {
        assert!(package.view_as_index.iter().any(|view| {
            view.skill_def_id == CHIXIN1
                && view.as_kind == "sha"
                && view.window == window
                && view.filter == "diamond"
                && view.source_zone.as_deref() == Some("hand_equip")
        }));
    }
    assert!(package.view_as_index.iter().any(|view| {
        view.skill_def_id == CHIXIN2
            && view.as_kind == "shan"
            && view.window == "response"
            && view.filter == "diamond"
            && view.source_zone.as_deref() == Some("hand_equip")
    }));
    for window in ["play_phase", "response"] {
        assert!(package.view_as_index.iter().any(|view| {
            view.skill_def_id == YYYAZHEN
                && view.as_kind == "sha"
                && view.window == window
                && view.filter == "any"
                && view.source_zone.as_deref() == Some("equip")
        }));
    }
}

#[test]
fn chixin1_uses_diamond_equipment_as_sha_but_rejects_other_suits() {
    let engine = package_engine(20_260_725);
    let actor = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != actor)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    clear_skills(&mut tx);
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, target);
    let skill = add_skill(&mut tx, actor, CHIXIN1);
    let heart = inject(
        &mut tx,
        actor,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Heart,
        2,
    );
    let diamond = inject(
        &mut tx,
        actor,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Diamond,
        3,
    );

    let (_, pipe) = apply_command(tx.state().clone(), activate(actor, skill, target, heart, 1));
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));

    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, pipe) = apply_command(
        tx.state().clone(),
        activate(actor, skill, target, diamond, 2),
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn diamond_equipment_is_offered_for_sha_and_shan_responses() {
    for (case, response_skill, attack_kind, expected_kind) in [
        (0, CHIXIN1, CardKind::TrickJuedou, "sha"),
        (1, CHIXIN2, CardKind::Sha, "shan"),
    ] {
        let engine = package_engine(20_260_726 + case);
        let source = engine.state().turn.current;
        let target = engine
            .state()
            .players
            .iter_live()
            .find(|player| player.id != source)
            .unwrap()
            .id;
        let mut tx = Transaction::begin(engine.state().clone());
        clear_skills(&mut tx);
        tx.state_mut().turn.phase = Phase::Play;
        clear_hand(&mut tx, source);
        clear_hand(&mut tx, target);
        add_skill(&mut tx, target, response_skill);
        let attack = inject(&mut tx, source, ZoneKind::Hand, attack_kind, Suit::Club, 6);
        let diamond = inject(
            &mut tx,
            target,
            ZoneKind::Equip,
            CardKind::Armor,
            Suit::Diamond,
            7,
        );
        let heart = inject(
            &mut tx,
            target,
            ZoneKind::Hand,
            CardKind::Other,
            Suit::Heart,
            8,
        );

        let (_, pipe) = apply_command(
            tx.state().clone(),
            Command {
                actor: source,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(10 + case),
                kind: CommandKind::PlayCard {
                    card: attack,
                    targets: vec![target],
                    card_targets: vec![],
                },
            },
        );
        assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
        let choice = pipe
            .choice
            .unwrap_or_else(|| panic!("expected {expected_kind} response"));
        assert_eq!(choice.observer, target);
        assert!(choice
            .options
            .iter()
            .any(|option| option.token == diamond.raw()));
        assert!(!choice
            .options
            .iter()
            .any(|option| option.token == heart.raw()));
    }
}

#[test]
fn yyyazhen_accepts_any_equipment_but_never_a_hand_card() {
    let engine = package_engine(20_260_728);
    let actor = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != actor)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    clear_skills(&mut tx);
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, target);
    let skill = add_skill(&mut tx, actor, YYYAZHEN);
    let hand = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Diamond,
        9,
    );
    let equip = inject(
        &mut tx,
        actor,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Club,
        10,
    );

    let (_, pipe) = apply_command(tx.state().clone(), activate(actor, skill, target, hand, 20));
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));

    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, pipe) = apply_command(
        tx.state().clone(),
        activate(actor, skill, target, equip, 21),
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);

    let engine = package_engine(20_260_729);
    let source = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != source)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    clear_skills(&mut tx);
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, source);
    clear_hand(&mut tx, target);
    add_skill(&mut tx, target, YYYAZHEN);
    let duel = inject(
        &mut tx,
        source,
        ZoneKind::Hand,
        CardKind::TrickJuedou,
        Suit::Spade,
        11,
    );
    let hand = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Diamond,
        12,
    );
    let equip = inject(
        &mut tx,
        target,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Heart,
        13,
    );
    let (_, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(22),
            kind: CommandKind::PlayCard {
                card: duel,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    let choice = pipe.choice.unwrap();
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == equip.raw()));
    assert!(!choice
        .options
        .iter()
        .any(|option| option.token == hand.raw()));
}
