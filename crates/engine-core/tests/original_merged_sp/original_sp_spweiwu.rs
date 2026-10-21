//! Golden for SP 韩遂「违忤」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SPWEIWU: u32 = 4_254_049_099;
const ZHANYI_BASIC_SHA: u32 = 3_535_942_143;
const ZHANYI_BASIC_TAO: u32 = 3_584_326_530;
const ZHANYI_BASIC_JIU: u32 = 3_922_392_848;
const YANZHENG: u32 = 2_899_287_996;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

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

fn strip_skills(tx: &mut Transaction, player: PlayerId) {
    let skills = tx.state().players.get(player).unwrap().skills.clone();
    for skill in skills {
        run_effects(tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    }
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
        def: CardDefId::new(99_500 + kind.tag() as u32 * 20 + rank as u32),
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

fn pass_wuxie(
    mut state: GameState,
    mut pipe: PipelineResult,
    mut key: u64,
) -> (GameState, PipelineResult, u64) {
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        let choice = pipe.choice.clone().unwrap();
        key += 1;
        (state, pipe) = apply_command(
            state,
            Command {
                actor: choice.observer,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    }
    (state, pipe, key)
}

#[test]
fn package_declares_red_hand_equip_once_and_distance_bypass() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let view = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == SPWEIWU)
        .unwrap();
    assert_eq!(view.as_kind, "shunshou");
    assert_eq!(view.window, "play_phase");
    assert_eq!(view.filter, "red");
    assert_eq!(view.source_zone.as_deref(), Some("hand_equip"));
    assert_eq!(view.uses_per_turn, Some(1));
    assert!(view.ignore_distance);
    assert!(package.manifest.rule_vm_abi_version >= 70);
    for (skill, kind) in [
        (ZHANYI_BASIC_SHA, "sha"),
        (ZHANYI_BASIC_TAO, "tao"),
        (ZHANYI_BASIC_JIU, "jiu"),
    ] {
        for window in ["play_phase", "response"] {
            assert!(package.view_as_index.iter().any(|view| {
                view.skill_def_id == skill
                    && view.as_kind == kind
                    && view.window == window
                    && view.filter == "card_basic"
                    && view.source_zone.is_none()
            }));
        }
    }
}

#[test]
fn red_equipment_takes_at_distance_two_but_black_and_second_use_fail() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 20_260_724,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let actor = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let actor_index = seats.iter().position(|player| *player == actor).unwrap();
    let far = seats[(actor_index + 2) % seats.len()];
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    for player in seats {
        strip_skills(&mut tx, player);
    }
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, far);
    let skill = add_skill(&mut tx, actor, SPWEIWU);
    let ordinary = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::TrickShunshou,
        Suit::Club,
        2,
    );
    let black = inject(
        &mut tx,
        actor,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Spade,
        3,
    );
    let red = inject(
        &mut tx,
        actor,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Heart,
        4,
    );
    let second_red = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Diamond,
        5,
    );
    let prize = inject(&mut tx, far, ZoneKind::Hand, CardKind::Tao, Suit::Heart, 6);

    let (_, ordinary_pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: ordinary,
                targets: vec![far],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(
        ordinary_pipe.result,
        CommandResult::Rejected { .. }
    ));

    let (_, black_pipe) = apply_command(
        tx.state().clone(),
        activate(actor, skill, vec![far], black, 2),
    );
    assert!(matches!(black_pipe.result, CommandResult::Rejected { .. }));

    let (state, pipe) = apply_command(
        tx.state().clone(),
        activate(actor, skill, vec![far], red, 3),
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let (state, pipe, key) = pass_wuxie(state, pipe, 3);
    let take = pipe.choice.unwrap();
    assert_eq!(take.observer, actor);
    assert!(take
        .options
        .iter()
        .any(|option| option.token == prize.raw()));
    let (state, pipe) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key + 1),
            kind: CommandKind::ResolveChoice {
                choice_id: take.choice_id,
                selections: vec![prize.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(actor).contains(&prize));

    let (_, second_pipe) = apply_command(
        state,
        activate(actor, skill, vec![far], second_red, key + 2),
    );
    assert!(matches!(second_pipe.result, CommandResult::Rejected { .. }));
}

#[test]
fn zhanyi_basic_tao_accepts_only_basic_hand_cards() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(30),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_730,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let actor = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    strip_skills(&mut tx, actor);
    clear_hand(&mut tx, actor);
    tx.state_mut().turn.phase = Phase::Play;
    let skill = add_skill(&mut tx, actor, ZHANYI_BASIC_TAO);
    let equipment = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::Armor,
        Suit::Club,
        7,
    );
    let basic = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        8,
    );
    tx.state_mut().players.get_mut(actor).unwrap().hp -= 1;

    let (_, rejected) = apply_command(
        tx.state().clone(),
        activate(actor, skill, vec![], equipment, 31),
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let hp = tx.state().players.get(actor).unwrap().hp;
    let (state, accepted) = apply_command(
        tx.state().clone(),
        activate(actor, skill, vec![], basic, 32),
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(actor).unwrap().hp, hp + 1);
}

#[test]
fn zhanyi_basic_sha_is_offered_for_duel_response() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(33),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_731,
            ruleset: "identity-standard".into(),
        },
    });
    let source = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != source)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_skills(&mut tx, source);
    strip_skills(&mut tx, target);
    clear_hand(&mut tx, source);
    clear_hand(&mut tx, target);
    tx.state_mut().turn.phase = Phase::Play;
    add_skill(&mut tx, target, ZHANYI_BASIC_SHA);
    let duel = inject(
        &mut tx,
        source,
        ZoneKind::Hand,
        CardKind::TrickJuedou,
        Suit::Spade,
        9,
    );
    let basic = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Heart,
        10,
    );
    let equipment = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Armor,
        Suit::Club,
        11,
    );

    let (_, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(34),
            kind: CommandKind::PlayCard {
                card: duel,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let choice = pipe.choice.expect("战杀应进入决斗出杀响应");
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == basic.raw()));
    assert!(!choice
        .options
        .iter()
        .any(|option| option.token == equipment.raw()));
}

#[test]
fn zhanyi_basic_jiu_can_self_save_from_dying() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(35),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_732,
            ruleset: "identity-standard".into(),
        },
    });
    let actor = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != actor)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    strip_skills(&mut tx, actor);
    clear_hand(&mut tx, actor);
    add_skill(&mut tx, actor, ZHANYI_BASIC_JIU);
    let basic = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Diamond,
        12,
    );
    tx.state_mut().players.get_mut(actor).unwrap().hp = 1;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: actor,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(
        engine_core::query::dying_save_card(tx.state(), actor),
        Some(basic)
    );

    let committed = tx.commit();
    let choice_id = engine_api::ChoiceId::new(committed.state.resolution.pending_choice_id);
    let (state, pipe) = apply_command(
        committed.state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(36),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let player = state.players.get(actor).unwrap();
    assert!(player.alive && !player.dying);
    assert_eq!(player.hp, 1);
    assert!(!state.hand_cards(actor).contains(&basic));
}

#[test]
fn yanzheng_offers_equipment_as_wuxie_only_when_hand_exceeds_hp() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let view = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == YANZHENG)
        .unwrap();
    assert_eq!(view.as_kind, "wuxie");
    assert_eq!(view.window, "response");
    assert_eq!(view.filter, "owner_hand_gt_hp");
    assert_eq!(view.source_zone.as_deref(), Some("equip"));
    assert!(package.manifest.rule_vm_abi_version >= 70);

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_736),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_736,
            ruleset: "identity-standard".into(),
        },
    });
    let actor = engine.state().turn.current;
    let responder = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != actor)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    strip_skills(&mut tx, actor);
    strip_skills(&mut tx, responder);
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, responder);
    add_skill(&mut tx, responder, YANZHENG);
    tx.state_mut().players.get_mut(responder).unwrap().hp = 1;
    let first_trick = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        Suit::Heart,
        13,
    );
    let second_trick = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        Suit::Diamond,
        14,
    );
    inject(
        &mut tx,
        responder,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Heart,
        15,
    );
    let equipment = inject(
        &mut tx,
        responder,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Club,
        16,
    );
    let equal_hand_hp = tx.state().clone();

    let (_, result) = apply_command(
        equal_hand_hp.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: first_trick,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());

    let mut high_hand = Transaction::begin(equal_hand_hp);
    let hand_card = inject(
        &mut high_hand,
        responder,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        17,
    );
    let (_, result) = apply_command(
        high_hand.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::PlayCard {
                card: second_trick,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    let choice = result.choice.expect("严整应提供装备区牌作为无懈");
    assert_eq!(choice.observer, responder);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == equipment.raw()));
    assert!(!choice
        .options
        .iter()
        .any(|option| option.token == hand_card.raw()));
}
