//! Golden for the first original-content DSL rewrite: SP 诸葛诞「功獒」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const GONGAO: u32 = 2_416_186_304;
const LIEWEI: u32 = 3_656_060_804;
const ZHUISHE: u32 = 2_536_634_542;
const WEIZHONG: u32 = 2_998_881_378;
const GUILONGZHANYUEDAO: u32 = 4_091_349_421;
const CHOUHAI: u32 = 2_265_775_915;
const RECHOUHAI: u32 = 4_121_233_014;
const ZHUANGSHU_EQUIP: u32 = 2_527_325_512;
const RANSHANG: u32 = 3_675_767_094;
const RANSHANG2: u32 = 3_932_018_676;
const RANSHANG_MARK: u32 = 638_947_751;
const JINGONG2: u32 = 3_807_235_921;
const SHENGXI: u32 = 2_200_786_615;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

#[test]
fn gongao_package_hash_and_death_semantics() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(
        committed.manifest.logic_hash,
        committed.compute_logic_hash()
    );
    let gongao = committed
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == GONGAO)
        .unwrap();
    assert!(gongao.locked);
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == 3_007_668_710)
            .unwrap()
            .skills,
        vec![3_565_936_884]
    );
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == 2_408_538_527)
            .unwrap()
            .skills,
        vec![3_157_963_287, 2_415_897_133]
    );
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == 4_052_796_884)
            .unwrap()
            .skills,
        vec![3_743_761_196]
    );
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == 2_327_484_329)
            .unwrap()
            .skills,
        vec![2_416_186_304, 2_383_670_146]
    );
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == 4_039_499_432)
            .unwrap()
            .skills,
        vec![3_675_767_094, 3_661_917_608]
    );
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == 3_981_632_700)
            .unwrap()
            .skills,
        vec![3_504_783_479, 2_944_037_719]
    );
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == 2_211_264_628)
            .unwrap()
            .skills,
        vec![2_533_816_770, 3_184_006_504]
    );

    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: 20_260_713,
            ruleset: "identity-standard".into(),
        },
    });

    let owner = engine.state().turn.current;
    let victim = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    // 清除开局角色的默认技能，避免并发新增角色的死亡触发干扰。
    let mut setup_effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup_effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(GONGAO),
    });
    run_effects(&mut tx, setup_effects).unwrap();
    let before = tx.state().players.get(owner).unwrap().clone();

    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer: owner,
        }],
    )
    .unwrap();

    let after_max_hp = tx.state().players.get(owner).unwrap().max_hp;
    let after_hp = tx.state().players.get(owner).unwrap().hp;
    assert_eq!(after_max_hp, before.max_hp + 1);
    assert_eq!(after_hp, before.hp + 1);
    let killer = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: owner,
            killer,
        }],
    )
    .unwrap();
    let dead_owner = tx.state().players.get(owner).unwrap();
    assert!(!dead_owner.alive);
    assert_eq!(dead_owner.hp, 0);
    assert_eq!(
        dead_owner.max_hp, after_max_hp,
        "功獒不能在拥有者死亡后发动"
    );
}

#[test]
fn weizhong_draws_two_at_minimum_hand_and_one_otherwise() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_716,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(WEIZHONG),
            },
            EffectRequest::DiscardFromHand {
                player: owner,
                count: u32::MAX,
            },
        ],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ChangeMaxHp {
            player: owner,
            delta: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 2);

    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: other,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ChangeMaxHp {
            player: owner,
            delta: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);
}

#[test]
fn zhuishe_adds_one_to_the_default_sha_use_limit() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
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
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .filter(|skill| skill.owner == owner)
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZHUISHE),
    });
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 2);
}

#[test]
fn liewei_offers_three_cards_only_after_the_owner_kills() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: 20_260_714,
            ruleset: "identity-standard".into(),
        },
    });

    let owner = engine.state().turn.current;
    let victim = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(LIEWEI),
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);

    let paused = run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer: owner,
        }],
    )
    .unwrap_err();
    assert_eq!(paused, EffectError::ChoiceRequested);
    assert_eq!(tx.state().hand_count(owner), hand);

    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 3);
}

#[test]
fn guilongzhanyuedao_makes_only_red_sha_unrespondable() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_716),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_716,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(GUILONGZHANYUEDAO),
        }],
    )
    .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let mut make_sha = |suit| {
        tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(92_120_000 + suit as u32),
            kind: CardKind::Sha,
            suit,
            rank: 5,
            weapon_range: 0,
            zone: hand,
            owner,
            face_up: false,
        })
    };
    let red = make_sha(Suit::Heart);
    let black = make_sha(Suit::Spade);
    assert!(engine_core::query::sha_unrespondable_for_card(
        tx.state(),
        owner,
        red
    ));
    assert!(!engine_core::query::sha_unrespondable_for_card(
        tx.state(),
        owner,
        black
    ));
}

#[test]
fn chouhai_adds_one_damage_only_while_the_owner_has_no_hand_cards() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_717),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_717,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(CHOUHAI),
        }],
    )
    .unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 10;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 10;

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 9);

    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    tx.state_mut().zones.get_mut(hand).unwrap().cards.clear();
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 7);
}

#[test]
fn rechouhai_only_adds_damage_for_sha_while_the_owner_has_no_hand_cards() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(RECHOUHAI as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: RECHOUHAI as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let source = engine.state().turn.current;
    let owner = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != source)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let natural_skills = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, natural_skills).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(RECHOUHAI),
        }],
    )
    .unwrap();
    assert_eq!(
        engine_core::query::sha_damage_taken_for(tx.state(), owner, 1),
        1
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    assert_eq!(
        engine_core::query::sha_damage_taken_for(tx.state(), owner, 1),
        2
    );
    tx.state_mut().players.get_mut(owner).unwrap().hp = 10;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 10;
    let hand = tx.state().zones.find(ZoneKind::Hand, source).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(RECHOUHAI),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: source,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);

    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![owner],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 8);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 7);
}

#[test]
fn zhuangshu_equip_draws_to_the_lower_of_hand_limit_and_five() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(ZHUANGSHU_EQUIP as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: ZHUANGSHU_EQUIP as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;

    for (hand_limit, expected) in [(3, 3), (8, 5)] {
        let mut tx = Transaction::begin(engine.state().clone());
        let remove_existing: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        run_effects(&mut tx, remove_existing).unwrap();
        run_effects(
            &mut tx,
            vec![
                EffectRequest::AddSkill {
                    player: owner,
                    skill_def: SkillDefId::new(ZHUANGSHU_EQUIP),
                },
                EffectRequest::DiscardFromHand {
                    player: owner,
                    count: u32::MAX,
                },
                EffectRequest::DrawCards {
                    player: owner,
                    count: 1,
                },
            ],
        )
        .unwrap();
        {
            let player = tx.state_mut().players.get_mut(owner).unwrap();
            player.hp = hand_limit;
            player.max_hp = hand_limit as u32;
        }
        run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: owner,
                phase: engine_core::state::Phase::Play.to_u8(),
            }],
        )
        .unwrap();
        run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: owner,
                phase: engine_core::state::Phase::Discard.to_u8(),
            }],
        )
        .unwrap();
        assert_eq!(tx.state().hand_count(owner), expected);
    }
}

#[test]
fn ranshang_adds_burn_only_for_fire_damage() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(RANSHANG as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: RANSHANG as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let source = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 10;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 10;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(RANSHANG),
        }],
    )
    .unwrap();

    for (nature, amount, expected) in [
        (rule_api::damage_nature::NORMAL, 1, 0),
        (rule_api::damage_nature::FIRE, 2, 2),
    ] {
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target: owner,
                amount,
                is_lose_hp: false,
                nature,
            }],
        )
        .unwrap();
        let burns = tx
            .state()
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .find(|(mark, _)| *mark == RANSHANG_MARK)
            .map_or(0, |(_, count)| *count);
        assert_eq!(burns, expected);
    }
}

#[test]
fn ranshang2_resolves_burn_marks_at_the_end_phase() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(RANSHANG2 as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: RANSHANG2 as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;

    for (burns, expected_hp, expected_max_hp, expected_draws) in [(2, 8, 10, 0), (3, 7, 8, 2)] {
        let mut tx = Transaction::begin(engine.state().clone());
        let remove_existing: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        run_effects(&mut tx, remove_existing).unwrap();
        {
            let player = tx.state_mut().players.get_mut(owner).unwrap();
            player.hp = 10;
            player.max_hp = 10;
        }
        run_effects(
            &mut tx,
            vec![
                EffectRequest::AddSkill {
                    player: owner,
                    skill_def: SkillDefId::new(RANSHANG2),
                },
                EffectRequest::AddMark {
                    player: owner,
                    mark: engine_api::MarkDefId::new(RANSHANG_MARK),
                    delta: burns,
                },
            ],
        )
        .unwrap();
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
        assert_eq!(player.hp, expected_hp);
        assert_eq!(player.max_hp, expected_max_hp);
        assert_eq!(tx.state().hand_count(owner), hand + expected_draws);
    }
}

#[test]
fn jingong2_only_penalizes_a_turn_without_damage() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(JINGONG2 as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: JINGONG2 as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;

    for dealt_damage in [false, true] {
        let mut tx = Transaction::begin(engine.state().clone());
        let target = tx
            .state()
            .players
            .iter_live()
            .find(|player| player.id != owner)
            .unwrap()
            .id;
        let remove_existing: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        run_effects(&mut tx, remove_existing).unwrap();
        add_test_skill(&mut tx, owner, JINGONG2);
        tx.state_mut().players.get_mut(owner).unwrap().hp = 4;
        if dealt_damage {
            run_effects(
                &mut tx,
                vec![EffectRequest::DealDamage {
                    source: owner,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
            .unwrap();
        }
        run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: owner,
                phase: engine_core::state::Phase::End.to_u8(),
            }],
        )
        .unwrap();
        assert_eq!(
            tx.state().players.get(owner).unwrap().hp,
            if dealt_damage { 4 } else { 3 }
        );
    }
}

#[test]
fn shengxi_only_offers_two_cards_in_a_damage_free_discard_phase() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_827),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_827,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;

    for dealt_damage in [false, true] {
        let mut tx = Transaction::begin(engine.state().clone());
        let remove_existing: Vec<_> = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
            .collect();
        run_effects(&mut tx, remove_existing).unwrap();
        tx.state_mut().players.get_mut(other).unwrap().hp = 10;
        tx.state_mut().players.get_mut(other).unwrap().max_hp = 10;
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(SHENGXI),
            }],
        )
        .unwrap();
        if dealt_damage {
            run_effects(
                &mut tx,
                vec![EffectRequest::DealDamage {
                    source: owner,
                    target: other,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
            .unwrap();
        }
        let hand = tx.state().hand_count(owner);
        let result = run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: owner,
                phase: engine_core::state::Phase::Discard.to_u8(),
            }],
        );
        if dealt_damage {
            result.unwrap();
            assert_eq!(tx.state().hand_count(owner), hand);
        } else {
            assert_eq!(result.unwrap_err(), EffectError::ChoiceRequested);
            let (state, result) = apply_command(
                tx.state().clone(),
                Command {
                    actor: owner,
                    expected_revision: ExpectedRevision(0),
                    idempotency_key: IdempotencyKey(1),
                    kind: CommandKind::ResolveChoice {
                        choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                        selections: vec![1],
                        cancelled: false,
                    },
                },
            );
            assert!(matches!(result.result, CommandResult::Accepted { .. }));
            assert_eq!(state.hand_count(owner), hand + 2);
        }
    }
}

fn add_test_skill(tx: &mut Transaction, owner: PlayerId, skill_def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
}
