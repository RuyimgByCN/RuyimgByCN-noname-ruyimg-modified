//! Golden for MOBILE「招祸」的全局濒死语义。

use engine_api::{
    CardDefId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{draw_count_for, optional_draw_bonuses_for, query, QueryContext};
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::{content, Engine};
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const ZHAOHUO: u32 = 2_173_746_528;
const REJIANGCHI: u32 = 3_537_053_421;
const REJIANGCHI3: u32 = 2_242_073_380;
const XIN_CAOZHANG: u32 = 3_739_830_718;
const HUAIZI: u32 = 4_199_126_566;
const MBRANSHANG: u32 = 3_354_404_150;
const MBRANSHANG_MARK: u32 = 1_206_920_502;
const TIANSUAN2_DAMAGE: u32 = 2_231_135_297;
const TIANSUAN2_0: u32 = 3_462_769_895;
const TIANSUAN2_1: u32 = 3_136_054_404;
const TIANSUAN2_2: u32 = 2_298_819_832;
const TIANSUAN2_3: u32 = 3_501_902_691;
const TIANSUAN2_4: u32 = 3_604_116_303;
const DUJIN: u32 = 3_000_807_303;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn grant(tx: &mut Transaction, owner: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZHAOHUO),
        }],
    )
    .unwrap();
}

#[test]
fn rejiangchi3_cancels_the_proactive_sha_limit() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    grant_skill(&mut tx, owner, REJIANGCHI3);

    assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 0);
}

#[test]
fn rejiangchi_publishes_the_play_phase_choice_and_complete_character() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = committed
        .character_index
        .iter()
        .find(|character| character.id == XIN_CAOZHANG)
        .unwrap();
    assert_eq!(character.skills, [REJIANGCHI]);

    let trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == REJIANGCHI)
        .unwrap();
    assert_eq!(trigger.event, "phase_play");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);

    let cleanup = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == REJIANGCHI3)
        .unwrap();
    assert_eq!(cleanup.event, "phase_play_end");
    assert!(!cleanup.optional);
}

fn grant_skill(tx: &mut Transaction, owner: PlayerId, skill_def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
}

#[test]
fn dujin_draw_bonus_uses_half_the_equip_count_rounded_up_plus_one() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(DUJIN as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: DUJIN as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, existing).unwrap();
    let base = draw_count_for(tx.state(), owner);
    grant_skill(&mut tx, owner, DUJIN);
    assert_eq!(draw_count_for(tx.state(), owner), base);
    assert_eq!(optional_draw_bonuses_for(tx.state(), owner), vec![1]);

    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    for (index, expected) in [2, 2, 3].into_iter().enumerate() {
        let card = tx.state_mut().cards.alloc(Card {
            id: engine_api::CardId::NONE,
            def: CardDefId::new(30_000 + index as u32),
            kind: CardKind::Treasure,
            suit: Suit::Diamond,
            rank: 1,
            weapon_range: 0,
            zone: equip,
            owner,
            face_up: true,
        });
        tx.state_mut()
            .zones
            .get_mut(equip)
            .unwrap()
            .cards
            .push(card);
        assert_eq!(draw_count_for(tx.state(), owner), base);
        assert_eq!(optional_draw_bonuses_for(tx.state(), owner), vec![expected]);
    }
}

fn equip_tengjia(tx: &mut Transaction, owner: PlayerId) {
    let zone = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let card = Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(content::DEF_TENGJIA),
        kind: CardKind::Armor,
        suit: Suit::Club,
        rank: 2,
        weapon_range: 0,
        zone,
        owner,
        face_up: true,
    };
    let card = tx.state_mut().cards.alloc(card);
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
}

#[test]
fn tiansuan2_damage_is_optional_and_draws_the_resolved_damage_amount() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(TIANSUAN2_DAMAGE as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: TIANSUAN2_DAMAGE as u64,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    grant_skill(&mut tx, owner, TIANSUAN2_DAMAGE);
    let hand = tx.state().hand_count(owner);

    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    assert_eq!(result.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(tx.state().hand_count(owner), hand);

    let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 2);
}

#[test]
fn tiansuan2_0_prevents_damage_but_not_lose_hp() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(TIANSUAN2_0 as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: TIANSUAN2_0 as u64,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    grant_skill(&mut tx, owner, TIANSUAN2_0);
    let hp = tx.state().players.get(owner).unwrap().hp;

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 3,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
}

#[test]
fn tiansuan2_1_caps_final_fire_damage_then_offers_one_card() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(TIANSUAN2_1 as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: TIANSUAN2_1 as u64,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    grant_skill(&mut tx, owner, TIANSUAN2_1);
    equip_tengjia(&mut tx, owner);
    let hp = tx.state().players.get(owner).unwrap().hp;
    let hand = tx.state().hand_count(owner);

    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 3,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
        }],
    );
    assert_eq!(result.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
    assert_eq!(tx.state().hand_count(owner), hand);

    let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 1);

    let mut lose_hp = Transaction::begin(state);
    run_effects(
        &mut lose_hp,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 2,
            is_lose_hp: true,
            nature: rule_api::damage_nature::FIRE,
        }],
    )
    .unwrap();
    assert_eq!(lose_hp.state().players.get(owner).unwrap().hp, hp - 3);
    assert_eq!(lose_hp.state().hand_count(owner), hand + 1);
}

#[test]
fn tiansuan2_2_turns_damage_to_fire_then_caps_after_tengjia() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(TIANSUAN2_2 as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: TIANSUAN2_2 as u64,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    grant_skill(&mut tx, owner, TIANSUAN2_2);
    equip_tengjia(&mut tx, owner);
    let hp = tx.state().players.get(owner).unwrap().hp;

    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 3,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
    assert!(events.contains(&DomainEvent::DamageDealt {
        source,
        target: owner,
        amount: 1,
        is_lose_hp: false,
        nature: rule_api::damage_nature::FIRE,
    }));
}

#[test]
fn tiansuan2_3_adds_one_to_damage_but_not_lose_hp() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(TIANSUAN2_3 as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: TIANSUAN2_3 as u64,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    grant_skill(&mut tx, owner, TIANSUAN2_3);
    let hp = tx.state().players.get(owner).unwrap().hp;

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 2);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 3);
}

#[test]
fn jiu_can_self_save_while_tiansuan2_4_blocks_tao_jiu_and_adds_damage() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(TIANSUAN2_4 as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: TIANSUAN2_4 as u64,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    let hand = tx.state().hand_cards(owner);
    assert!(hand.len() >= 2);
    for card in &hand {
        tx.state_mut().cards.get_mut(*card).unwrap().kind = CardKind::Sha;
    }
    let jiu = hand[0];
    let tao = hand[1];
    tx.state_mut().cards.get_mut(jiu).unwrap().kind = CardKind::Jiu;
    assert_eq!(
        engine_core::query::dying_save_card(tx.state(), owner),
        Some(jiu)
    );

    let mut self_save = Transaction::begin(tx.state().clone());
    self_save.state_mut().players.get_mut(owner).unwrap().hp = 1;
    let result = run_effects(
        &mut self_save,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    assert_eq!(result.unwrap_err(), EffectError::ChoiceRequested);
    let choice_id = ChoiceId::new(self_save.state().resolution.pending_choice_id);
    let (saved, result) = apply_command(
        self_save.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(saved.players.get(owner).unwrap().hp, 1);
    assert!(!saved.hand_cards(owner).contains(&jiu));

    tx.state_mut().cards.get_mut(tao).unwrap().kind = CardKind::Tao;
    grant_skill(&mut tx, owner, TIANSUAN2_4);
    assert!(engine_core::query::card_use_cancelled_for_card(
        tx.state(),
        owner,
        tao
    ));
    assert!(engine_core::query::card_use_cancelled_for_card(
        tx.state(),
        owner,
        jiu
    ));
    assert!(engine_core::query::card_save_cancelled_for_kind(
        tx.state(),
        owner,
        CardKind::Tao
    ));
    assert!(engine_core::query::card_save_cancelled_for_kind(
        tx.state(),
        owner,
        CardKind::Jiu
    ));
    assert_eq!(engine_core::query::dying_save_card(tx.state(), owner), None);

    let hp = tx.state().players.get(owner).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 2);
}

#[test]
fn huaizi_uses_max_hp_as_the_hand_limit_base() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_715,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let max_hp = tx.state().players.get(owner).unwrap().max_hp as i32;
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    grant_skill(&mut tx, owner, HUAIZI);

    assert_eq!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(max_hp)
    );
}

#[test]
fn committed_package_only_reacts_to_another_players_dying() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == ZHAOHUO));

    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_713,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;

    let mut other_dying = Transaction::begin(engine.state().clone());
    {
        // Drop every booted skill: a booted 华佗 would answer the dying
        // window with 急救 and open an optional-trigger choice.
        let booted: Vec<_> = other_dying
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        if !booted.is_empty() {
            run_effects(&mut other_dying, booted).unwrap();
        }
        let player = other_dying.state_mut().players.get_mut(owner).unwrap();
        player.hp = 4;
        player.max_hp = 4;
    }
    // Clear every hand so the dying-save window cannot open, independent of
    // which 桃 the booted roster was dealt.
    for p in [owner, other] {
        let hand = other_dying.state().zones.find(ZoneKind::Hand, p).unwrap();
        let cards: Vec<_> = other_dying.state().hand_cards(p);
        if !cards.is_empty() {
            let discard = other_dying
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap();
            run_effects(
                &mut other_dying,
                vec![EffectRequest::MoveCards {
                    cards,
                    to: discard,
                    from: Some(hand),
                }],
            )
            .unwrap();
        }
    }
    grant(&mut other_dying, owner);
    let hand = other_dying.state().hand_count(owner);
    run_effects(
        &mut other_dying,
        vec![EffectRequest::EnterDying { player: other }],
    )
    .unwrap();
    assert_eq!(other_dying.state().players.get(owner).unwrap().max_hp, 1);
    assert_eq!(other_dying.state().hand_count(owner), hand + 3);

    let mut self_dying = Transaction::begin(engine.state().clone());
    {
        let booted: Vec<_> = self_dying
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        if !booted.is_empty() {
            run_effects(&mut self_dying, booted).unwrap();
        }
        let player = self_dying.state_mut().players.get_mut(owner).unwrap();
        player.hp = 4;
        player.max_hp = 4;
    }
    for p in [owner, other] {
        let hand = self_dying.state().zones.find(ZoneKind::Hand, p).unwrap();
        let cards: Vec<_> = self_dying.state().hand_cards(p);
        if !cards.is_empty() {
            let discard = self_dying
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap();
            run_effects(
                &mut self_dying,
                vec![EffectRequest::MoveCards {
                    cards,
                    to: discard,
                    from: Some(hand),
                }],
            )
            .unwrap();
        }
    }
    grant(&mut self_dying, owner);
    run_effects(
        &mut self_dying,
        vec![EffectRequest::EnterDying { player: owner }],
    )
    .unwrap();
    assert_eq!(self_dying.state().players.get(owner).unwrap().max_hp, 4);
}

#[test]
fn mbranshang_counts_only_fire_damage_then_resolves_the_end_phase_penalty() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(MBRANSHANG as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: MBRANSHANG as u64,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    grant_skill(&mut tx, owner, MBRANSHANG);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
        }],
    )
    .unwrap();
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::MarkCount {
                player: owner,
                mark: MBRANSHANG_MARK,
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(2)
    ));
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::MarkCount {
                player: owner,
                mark: MBRANSHANG_MARK,
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(2)
    ));

    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.hp = 8;
        player.max_hp = 8;
    }
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: engine_core::state::Phase::End.to_u8(),
        }],
    )
    .unwrap();
    let player = tx.state().players.get(owner).unwrap();
    assert_eq!((player.hp, player.max_hp), (6, 6));
    assert_eq!(tx.state().hand_count(owner), hand + 2);
}
