//! Goldens for 界「看破」「倾国」的手牌/装备牌响应来源。

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

const REKANPO: u32 = 2_767_584_414;
const OLKANPO: u32 = 4_205_693_026;
const REQINGGUO: u32 = 2_359_932_684;
const REHUOJI: u32 = 4_176_322_956;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn package_engine_with_players(seed: u64, player_count: u8) -> Engine {
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

fn package_engine(seed: u64) -> Engine {
    package_engine_with_players(seed, 2)
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
        def: CardDefId::new(98_000 + kind.tag() as u32 * 20 + rank as u32),
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

fn play(actor: PlayerId, card: engine_api::CardId, targets: Vec<PlayerId>, key: u64) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::PlayCard {
            card,
            targets,
            card_targets: vec![],
        },
    }
}

fn activate(
    actor: PlayerId,
    skill: SkillId,
    targets: Vec<PlayerId>,
    card: engine_api::CardId,
    key: u64,
) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::ActivateSkill {
            skill,
            targets,
            card_targets: vec![card],
        },
    }
}

#[test]
fn package_exposes_hand_equip_response_sources() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    for (def, kind) in [(REKANPO, "wuxie"), (REQINGGUO, "shan")] {
        assert!(package.view_as_index.iter().any(|view| {
            view.skill_def_id == def
                && view.as_kind == kind
                && view.window == "response"
                && view.filter == "black"
                && view.source_zone.as_deref() == Some("hand_equip")
        }));
    }
    assert!(package.view_as_index.iter().any(|view| {
        view.skill_def_id == REHUOJI
            && view.as_kind == "huogong"
            && view.window == "play_phase"
            && view.filter == "red"
            && view.source_zone.as_deref() == Some("hand_equip")
    }));
    assert!(package.view_as_index.iter().any(|view| {
        view.skill_def_id == OLKANPO
            && view.as_kind == "wuxie"
            && view.window == "response"
            && view.filter == "black"
            && view.source_zone.as_deref() == Some("hand_equip")
    }));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == OLKANPO
            && modifier.target == "card_use_unrespondable"
            && modifier.selector.as_deref() == Some("wuxie")
            && modifier.value == 1
    }));
}

#[test]
fn olkanpo_wuxie_cannot_be_countered() {
    let engine = package_engine_with_players(20_260_716, 3);
    let seats = engine.state().players.alive_by_seat();
    let actor = engine.state().turn.current;
    let responder = seats
        .iter()
        .copied()
        .find(|player| *player != actor)
        .unwrap();
    let counter = seats
        .iter()
        .copied()
        .find(|player| *player != actor && *player != responder)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    clear_skills(&mut tx);
    tx.state_mut().turn.phase = Phase::Play;
    for player in [actor, responder, counter] {
        clear_hand(&mut tx, player);
    }
    add_skill(&mut tx, responder, OLKANPO);
    let trick = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        Suit::Heart,
        7,
    );
    let source = inject(
        &mut tx,
        responder,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Club,
        8,
    );
    let counter_wuxie = inject(
        &mut tx,
        counter,
        ZoneKind::Hand,
        CardKind::TrickWuxie,
        Suit::Spade,
        12,
    );

    let (state, pipe) = apply_command(tx.state().clone(), play(actor, trick, vec![], 11));
    let choice = pipe.choice.expect("OL 看破应提供装备区黑牌");
    assert_eq!(choice.observer, responder);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == source.raw()));

    let (state, pipe) = apply_command(
        state,
        Command {
            actor: responder,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(12),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![source.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert!(pipe.choice.is_none());
    assert_eq!(state.hand_count(actor), 0);
    assert!(state.hand_cards(counter).contains(&counter_wuxie));
}

#[test]
fn rehuoji_consumes_red_equipment_and_runs_huogong() {
    let engine = package_engine(20_260_721);
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
    let skill = add_skill(&mut tx, actor, REHUOJI);
    let black = inject(
        &mut tx,
        actor,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Club,
        2,
    );
    let source = inject(
        &mut tx,
        actor,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Heart,
        3,
    );
    let matching = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        7,
    );
    let shown = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Tao,
        Suit::Spade,
        9,
    );

    let (_, pipe) = apply_command(
        tx.state().clone(),
        activate(actor, skill, vec![target], black, 5),
    );
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));

    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, pipe) = apply_command(
        tx.state().clone(),
        activate(actor, skill, vec![target], source, 6),
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let reveal = pipe.choice.unwrap();
    assert_eq!(reveal.observer, target);

    let (state, pipe) = apply_command(
        state,
        Command {
            actor: target,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(7),
            kind: CommandKind::ResolveChoice {
                choice_id: reveal.choice_id,
                selections: vec![shown.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let discard = pipe.choice.unwrap();
    assert_eq!(discard.observer, actor);

    let (state, pipe) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(8),
            kind: CommandKind::ResolveChoice {
                choice_id: discard.choice_id,
                selections: vec![matching.raw()],
                cancelled: false,
            },
        },
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
fn rekanpo_consumes_black_equipment_as_wuxie() {
    let engine = package_engine(20_260_719);
    let actor = engine.state().turn.current;
    let responder = engine
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
    clear_hand(&mut tx, responder);
    add_skill(&mut tx, responder, REKANPO);
    let trick = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        Suit::Heart,
        7,
    );
    let source = inject(
        &mut tx,
        responder,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Club,
        8,
    );

    let (state, pipe) = apply_command(tx.state().clone(), play(actor, trick, vec![], 1));
    let choice = pipe.choice.expect("界看破应提供装备区黑牌");
    assert_eq!(choice.observer, responder);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == source.raw()));
    let (resolved, pipe) = apply_command(
        state,
        Command {
            actor: responder,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![source.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(resolved.hand_count(actor), 0);
    assert_eq!(
        resolved.cards.get(source).unwrap().zone,
        resolved
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}

#[test]
fn reqingguo_consumes_black_equipment_as_shan() {
    let engine = package_engine(20_260_720);
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
    add_skill(&mut tx, target, REQINGGUO);
    let sha = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Heart,
        9,
    );
    let source = inject(
        &mut tx,
        target,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Spade,
        10,
    );
    let hp = tx.state().players.get(target).unwrap().hp;

    let (state, pipe) = apply_command(tx.state().clone(), play(actor, sha, vec![target], 3));
    let choice = pipe.choice.expect("界倾国应提供装备区黑牌");
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == source.raw()));
    let (resolved, pipe) = apply_command(
        state,
        Command {
            actor: target,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(4),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![source.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(resolved.players.get(target).unwrap().hp, hp);
    assert_eq!(
        resolved.cards.get(source).unwrap().zone,
        resolved
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}
