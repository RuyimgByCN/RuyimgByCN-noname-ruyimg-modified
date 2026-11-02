//! Golden for OFFLINE「孤落」的空手濒死语义。

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const GULUO: u32 = 3_452_606_144;
const CHENGGONG: u32 = 3_971_525_290;
const ZHANGU: u32 = 3_458_676_792;
const JUN_JIANG: u32 = 4_092_230_721;
const SPSHUDE: u32 = 2_834_305_041;
const JIE: u32 = 3_115_545_698;
const YJQINGJIAO: u32 = 4_207_753_903;
const STDQINGJIAO: u32 = 4_273_235_044;
const EU_XIRU: u32 = 3_520_897_772;
const SCLS_YINSHI: u32 = 3_553_948_732;
const GZSUISHI2: u32 = 3_663_559_953;
const GZSUISHI: u32 = 4_271_491_554;
const JYLIJUN: u32 = 2_928_426_897;
const EU_JIQUAN: u32 = 2_434_098_210;
const PSDIWAN: u32 = 3_273_351_282;
const HM_ZHONG_DIAMOND_SKILL: u32 = 2_520_096_911;
const ZJ_HANCI: u32 = 3_201_112_152;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");
const STANDARD_PACKAGE: &[u8] = include_bytes!("../../../../content/standard/standard.npack.json");

fn isolated(engine: &Engine) -> Transaction {
    let mut tx = Transaction::begin(engine.state().clone());
    let effects: Vec<EffectRequest> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    // Removal may surface (or leave) optional-trigger choices for
    // deal-dependent skills; keep removing and cancelling until the state is
    // choice-free and skill-free.
    let mut key = 9_000u64;
    loop {
        if !tx.state().skills.sorted_live().is_empty() {
            match run_effects(&mut tx, effects.clone()) {
                Ok(_) => {}
                Err(EffectError::ChoiceRequested) => {}
                Err(e) => panic!("effect error: {e:?}"),
            }
        }
        if tx.state().resolution.pending_choice_id == 0 {
            break;
        }
        let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
        let (next, _) = apply_command(
            tx.state().clone(),
            Command {
                actor: PlayerId::NONE,
                expected_revision: ExpectedRevision(tx.state().revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        tx = Transaction::begin(next);
        key += 1;
    }
    tx
}

fn empty_dying(engine: &Engine, max_hp: u32) -> Transaction {
    let owner = engine.state().turn.current;
    let mut tx = isolated(engine);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(GULUO),
            },
            EffectRequest::DiscardFromHand {
                player: owner,
                count: u32::MAX,
            },
        ],
    )
    .unwrap();
    let player = tx.state_mut().players.get_mut(owner).unwrap();
    player.hp = 0;
    player.max_hp = max_hp;
    match run_effects(&mut tx, vec![EffectRequest::EnterDying { player: owner }]) {
        Ok(_) => {}
        Err(e) => eprintln!("DBG EnterDying err: {:?}", e),
    }
    tx
}

#[test]
fn hanci_draws_for_each_living_player_when_one_skill_is_added() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_714_4),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714_4,
            ruleset: "identity-standard".into(),
        },
    });
    let players = engine.state().players.alive_by_seat();
    let (owner, recipient) = (players[0], players[1]);
    let mut tx = isolated(&engine);
    let initial_owner_hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZJ_HANCI),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), initial_owner_hand + 2);

    let (owner_hand, recipient_hand) = (
        tx.state().hand_count(owner),
        tx.state().hand_count(recipient),
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: recipient,
            skill_def: SkillDefId::new(SCLS_YINSHI),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), owner_hand + 1);
    assert_eq!(tx.state().hand_count(recipient), recipient_hand + 1);

    tx.state_mut().players.get_mut(recipient).unwrap().alive = false;
    let hands = (
        tx.state().hand_count(owner),
        tx.state().hand_count(recipient),
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: recipient,
            skill_def: SkillDefId::new(JIE),
        }],
    )
    .unwrap();
    assert_eq!(
        (
            tx.state().hand_count(owner),
            tx.state().hand_count(recipient)
        ),
        hands
    );

    tx.state_mut().players.get_mut(recipient).unwrap().alive = true;
    tx.state_mut().players.get_mut(owner).unwrap().alive = false;
    let hands = (
        tx.state().hand_count(owner),
        tx.state().hand_count(recipient),
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: recipient,
            skill_def: SkillDefId::new(HM_ZHONG_DIAMOND_SKILL),
        }],
    )
    .unwrap();
    assert_eq!(
        (
            tx.state().hand_count(owner),
            tx.state().hand_count(recipient)
        ),
        hands
    );
}

#[test]
fn xiru_counts_all_living_mount_cards_and_rounds_up() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_714_2),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: 20_260_714_2,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = isolated(&engine);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(EU_XIRU),
        }],
    )
    .unwrap();
    assert_eq!(engine_core::query::draw_count_for(tx.state(), owner), 2);

    let players = tx.state().players.alive_by_seat();
    for (index, expected) in [3, 3, 4].into_iter().enumerate() {
        let holder = players[index];
        let zone = tx.state().zones.find(ZoneKind::Equip, holder).unwrap();
        let kind = if index % 2 == 0 {
            CardKind::HorseOff
        } else {
            CardKind::HorseDef
        };
        let card = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(99_000 + index as u32),
            kind,
            suit: Suit::Spade,
            rank: 5,
            weapon_range: 0,
            zone,
            owner: holder,
            face_up: true,
        });
        tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
        assert_eq!(
            engine_core::query::draw_count_for(tx.state(), owner),
            expected
        );
    }
}

#[test]
fn yinshi_skips_judgment_and_leaves_delayed_cards_in_place() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_714_3),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714_3,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = isolated(&engine);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SCLS_YINSHI),
        }],
    )
    .unwrap();
    let judge = tx.state().zones.find(ZoneKind::Judge, owner).unwrap();
    let delayed = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_100),
        kind: CardKind::TrickLe,
        suit: Suit::Spade,
        rank: 6,
        weapon_range: 0,
        zone: judge,
        owner,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(judge)
        .unwrap()
        .cards
        .push(delayed);
    tx.state_mut().turn.phase = engine_core::state::Phase::Judge;

    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, engine_core::state::Phase::Draw);
    assert_eq!(state.cards.get(delayed).unwrap().zone, judge);
}

#[test]
fn qingjiao_draws_after_damaging_another_qun_character() {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD_PACKAGE).unwrap();
    let offline = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();

    for skill_def in [STDQINGJIAO, YJQINGJIAO] {
        for (target_character, expected_draw) in [(24, 1), (1, 0)] {
            let mut engine = Engine::new();
            engine.load_package(&standard).unwrap();
            engine.load_package(&offline).unwrap();
            engine.submit(Command {
                actor: PlayerId::NONE,
                expected_revision: ExpectedRevision(0),
                // offline 新将批 roster 平移：+salt 避开 identity 分配坏种子窗。
                idempotency_key: IdempotencyKey(skill_def as u64 + target_character as u64 + 7),
                kind: CommandKind::StartGame {
                    player_count: 4,
                    rng_seed: skill_def as u64 + target_character as u64 + 7,
                    ruleset: "identity-standard".into(),
                },
            });
            let owner = engine
                .state()
                .players
                .iter_live()
                .find(|player| player.identity == engine_core::state::Identity::Lord)
                .unwrap()
                .id;
            let target = engine
                .state()
                .players
                .iter_live()
                .find(|player| player.id != owner)
                .unwrap()
                .id;
            let mut tx = isolated(&engine);
            tx.state_mut().turn.current = owner;
            tx.state_mut().turn.phase = engine_core::state::Phase::Play;
            tx.state_mut()
                .players
                .get_mut(target)
                .unwrap()
                .character_def = target_character;
            run_effects(
                &mut tx,
                vec![
                    EffectRequest::AddSkill {
                        player: owner,
                        skill_def: SkillDefId::new(skill_def),
                    },
                    EffectRequest::DealDamage {
                        source: owner,
                        target,
                        amount: 1,
                        is_lose_hp: false,
                        nature: 0,
                    },
                ],
            )
            .unwrap();
            let before = tx.state().hand_count(owner);
            run_effects(
                &mut tx,
                vec![EffectRequest::SetTurnState {
                    current: owner,
                    phase: engine_core::state::Phase::End.to_u8(),
                }],
            )
            .unwrap();
            assert_eq!(tx.state().hand_count(owner), before + expected_draw);
        }
    }
}

#[test]
fn jie_only_adds_damage_to_red_sha() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_822),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_822,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = isolated(&engine);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JIE),
        }],
    )
    .unwrap();
    let card = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(card).unwrap().kind = engine_core::state::CardKind::Sha;

    tx.state_mut().cards.get_mut(card).unwrap().suit = Suit::Heart;
    assert_eq!(
        engine_core::query::sha_damage_for_card(tx.state(), owner, card, 1),
        2
    );
    tx.state_mut().cards.get_mut(card).unwrap().suit = Suit::Spade;
    assert_eq!(
        engine_core::query::sha_damage_for_card(tx.state(), owner, card, 1),
        1
    );
    assert_eq!(engine_core::query::sha_damage_for(tx.state(), owner, 1), 1);
}

#[test]
fn jun_jiang_offers_draw_only_after_the_owner_uses_a_red_card() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_713_1,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;

    let prepare = |suit| {
        let mut tx = isolated(&engine);
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JUN_JIANG),
            }],
        )
        .unwrap();
        let card = tx.state().hand_cards(owner)[0];
        tx.state_mut().cards.get_mut(card).unwrap().suit = suit;
        (tx, card)
    };

    let (mut red, red_card) = prepare(Suit::Heart);
    let hand = red.state().hand_count(owner);
    assert_eq!(
        dispatch_for_events(
            &mut red,
            &[DomainEvent::CardUsed {
                player: owner,
                card: red_card,
                targets: vec![],
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        engine_core::effect::EffectError::ChoiceRequested
    );
    let (state, result) = apply_command(
        red.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(red.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(red.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 1);

    let (mut black, black_card) = prepare(Suit::Spade);
    let effects = dispatch_for_events(
        &mut black,
        &[DomainEvent::CardUsed {
            player: owner,
            card: black_card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!black.state().has_pending_choice());
}

#[test]
fn spshude_offers_to_draw_to_max_hp_only_when_hand_is_below_it() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714_1,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;

    let prepare = || {
        let mut tx = isolated(&engine);
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(SPSHUDE),
            }],
        )
        .unwrap();
        tx
    };

    let mut below = prepare();
    run_effects(
        &mut below,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let max_hp = below.state().players.get(owner).unwrap().max_hp;
    assert_eq!(below.state().hand_count(owner), 0);
    assert_eq!(
        dispatch_for_events(
            &mut below,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::End,
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        engine_core::effect::EffectError::ChoiceRequested
    );
    let (state, result) = apply_command(
        below.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(below.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(below.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), max_hp);

    let mut full = prepare();
    let max_hp = full.state().players.get(owner).unwrap().max_hp;
    let hand = full.state().hand_count(owner);
    if hand < max_hp {
        run_effects(
            &mut full,
            vec![EffectRequest::DrawCards {
                player: owner,
                count: max_hp - hand,
            }],
        )
        .unwrap();
    }
    let effects = dispatch_for_events(
        &mut full,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!full.state().has_pending_choice());

    let mut dead = prepare();
    run_effects(
        &mut dead,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    dead.state_mut().players.get_mut(owner).unwrap().alive = false;
    let effects = dispatch_for_events(
        &mut dead,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert!(!dead.state().has_pending_choice());
}

#[test]
fn committed_package_recovers_or_dies_when_max_hp_reaches_zero() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(
        committed.manifest.logic_hash,
        committed.compute_logic_hash()
    );
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == GULUO));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == CHENGGONG));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == ZHANGU));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == EU_XIRU));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == SCLS_YINSHI));

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

    let recovered = empty_dying(&engine, 4);
    let player = recovered.state().players.get(owner).unwrap();
    assert_eq!(
        (player.max_hp, player.hp, player.alive, player.dying),
        (3, 1, true, false)
    );

    let dead = empty_dying(&engine, 1);
    let player = dead.state().players.get(owner).unwrap();
    assert_eq!((player.max_hp, player.hp, player.alive), (0, 0, false));
}

#[test]
fn zhangu_requires_max_hp_above_one_and_an_empty_zone() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
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
    let owner = engine.state().turn.current;

    let mut active = isolated(&engine);
    run_effects(
        &mut active,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZHANGU),
        }],
    )
    .unwrap();
    let max_hp = active.state().players.get(owner).unwrap().max_hp;
    let hand = active.state().hand_count(owner);
    run_effects(
        &mut active,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: engine_core::state::Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(
        active.state().players.get(owner).unwrap().max_hp,
        max_hp - 1
    );
    assert_eq!(active.state().hand_count(owner), hand + 2);

    let mut blocked = isolated(&engine);
    run_effects(
        &mut blocked,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZHANGU),
        }],
    )
    .unwrap();
    blocked.state_mut().players.get_mut(owner).unwrap().max_hp = 1;
    let hand = blocked.state().hand_count(owner);
    run_effects(
        &mut blocked,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: engine_core::state::Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(blocked.state().players.get(owner).unwrap().max_hp, 1);
    assert_eq!(blocked.state().hand_count(owner), hand);
}

#[test]
fn gzsuishi2_loses_hp_only_when_the_dead_player_has_the_same_group() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();

    for (victim_character, expected_hp) in [(9_215_002, 4), (9_215_003, 5)] {
        let mut roster = content_schema::ContentPackage::from_json_bytes(STANDARD_PACKAGE).unwrap();
        roster.manifest.package_id = "gzsuishi2-test-roster".into();
        roster.character_index = [(9_215_001, "wei"), (9_215_002, "wei"), (9_215_003, "shu")]
            .into_iter()
            .map(|(id, group)| PackageCharacterEntry {
                voice_assets: Default::default(),
                id,
                name_key: format!("group_{id}"),
                max_hp: 5,
                initial_hp: 5,
                skills: vec![],
                sex: "male".into(),
                group: group.into(),
                is_lord: false,
                clans: vec![],
                tags: vec![],
                sort_key: String::new(),
                portrait_asset: None,
                skin_assets: vec![],
            })
            .collect();
        roster.manifest.logic_hash = roster.compute_logic_hash();

        let mut engine = Engine::new();
        engine.load_package(&committed).unwrap();
        engine.load_package(&roster).unwrap();
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(victim_character as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: victim_character as u64,
                ruleset: "identity-standard".into(),
            },
        });
        let owner = engine.state().turn.current;
        let mut tx = isolated(&engine);
        let victim = tx
            .state()
            .players
            .iter_live()
            .find(|player| player.id != owner)
            .unwrap()
            .id;
        {
            let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
            owner_state.character_def = 9_215_001;
            owner_state.hp = 5;
            owner_state.max_hp = 5;
        }
        tx.state_mut()
            .players
            .get_mut(victim)
            .unwrap()
            .character_def = victim_character;
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(GZSUISHI2),
            }],
        )
        .unwrap();
        run_effects(
            &mut tx,
            vec![EffectRequest::KillPlayer {
                player: victim,
                killer: PlayerId::NONE,
            }],
        )
        .unwrap();
        assert_eq!(tx.state().players.get(owner).unwrap().hp, expected_hp);
    }
}

#[test]
fn jylijun_only_offers_to_draw_for_a_living_wei_damage_target() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();

    for (target_group, should_trigger) in [("wei", true), ("shu", false)] {
        let mut roster = content_schema::ContentPackage::from_json_bytes(STANDARD_PACKAGE).unwrap();
        roster.manifest.package_id = "jylijun-test-roster".into();
        roster.character_index = [(9_217_001, "wei"), (9_217_002, target_group)]
            .into_iter()
            .map(|(id, group)| PackageCharacterEntry {
                voice_assets: Default::default(),
                id,
                name_key: format!("lijun_group_{id}"),
                max_hp: 5,
                initial_hp: 5,
                skills: vec![],
                sex: "male".into(),
                group: group.into(),
                is_lord: false,
                clans: vec![],
                tags: vec![],
                sort_key: String::new(),
                portrait_asset: None,
                skin_assets: vec![],
            })
            .collect();
        roster.manifest.logic_hash = roster.compute_logic_hash();

        let mut engine = Engine::new();
        engine.load_package(&committed).unwrap();
        engine.load_package(&roster).unwrap();
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_825),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_825,
                ruleset: "identity-standard".into(),
            },
        });
        let owner = engine.state().turn.current;
        let target = engine
            .state()
            .players
            .iter_live()
            .find(|player| player.id != owner)
            .unwrap()
            .id;
        let mut tx = isolated(&engine);
        tx.state_mut().players.get_mut(owner).unwrap().character_def = 9_217_001;
        {
            let target_state = tx.state_mut().players.get_mut(target).unwrap();
            target_state.character_def = 9_217_002;
            target_state.hp = 5;
            target_state.max_hp = 5;
        }
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JYLIJUN),
            }],
        )
        .unwrap();
        let hand = tx.state().hand_count(target);
        let result = run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        );
        if should_trigger {
            assert_eq!(
                result.unwrap_err(),
                engine_core::effect::EffectError::ChoiceRequested
            );
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
            assert_eq!(state.hand_count(target), hand + 1);
        } else {
            result.unwrap();
            assert_eq!(tx.state().hand_count(target), hand);
        }
    }
}

#[test]
fn gzsuishi_uses_the_damage_source_group_when_another_player_enters_dying() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();

    for (source_group, expected_draws) in [("wei", 1), ("shu", 0)] {
        let mut roster = content_schema::ContentPackage::from_json_bytes(STANDARD_PACKAGE).unwrap();
        roster.manifest.package_id = "gzsuishi-test-roster".into();
        roster.character_index = [
            (9_218_001, "wei"),
            (9_218_002, source_group),
            (9_218_003, "qun"),
        ]
        .into_iter()
        .map(|(id, group)| PackageCharacterEntry {
            voice_assets: Default::default(),
            id,
            name_key: format!("suishi_group_{id}"),
            max_hp: 5,
            initial_hp: 5,
            skills: vec![],
            sex: "male".into(),
            group: group.into(),
            is_lord: false,
            clans: vec![],
            tags: vec![],
            sort_key: String::new(),
            portrait_asset: None,
            skin_assets: vec![],
        })
        .collect();
        roster.manifest.logic_hash = roster.compute_logic_hash();

        let mut engine = Engine::new();
        engine.load_package(&committed).unwrap();
        engine.load_package(&roster).unwrap();
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_826),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_826,
                ruleset: "identity-standard".into(),
            },
        });
        let owner = engine.state().turn.current;
        let others: Vec<_> = engine
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|player| *player != owner)
            .collect();
        let source = others[0];
        let target = others[1];
        let mut tx = isolated(&engine);
        tx.state_mut().players.get_mut(owner).unwrap().character_def = 9_218_001;
        tx.state_mut()
            .players
            .get_mut(source)
            .unwrap()
            .character_def = 9_218_002;
        {
            let target_state = tx.state_mut().players.get_mut(target).unwrap();
            target_state.character_def = 9_218_003;
            target_state.hp = 1;
            target_state.max_hp = 5;
        }
        let mut setup_effects: Vec<_> = tx
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .map(|player| EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            })
            .collect();
        setup_effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(GZSUISHI),
        });
        run_effects(&mut tx, setup_effects).unwrap();
        let draw = tx
            .state()
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .unwrap();
        let safe_draw = tx.state_mut().cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(9_218_100),
            kind: CardKind::Other,
            suit: Suit::Club,
            rank: 1,
            weapon_range: 0,
            zone: draw,
            owner: PlayerId::NONE,
            face_up: false,
        });
        tx.state_mut()
            .zones
            .get_mut(draw)
            .unwrap()
            .cards
            .push(safe_draw);
        let hand = tx.state().hand_count(owner);
        let events = run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .unwrap();
        assert_eq!(tx.state().hand_count(owner), hand + expected_draws);
        assert!(!tx.state().has_pending_choice());
        assert!(!tx.state().players.get(target).unwrap().alive);
        assert!(events.iter().any(|event| matches!(
            event,
            engine_api::DomainEvent::PlayerKilled { player, .. } if *player == target
        )));
    }
}

#[test]
fn eu_jiquan_only_fires_for_a_living_western_turn_owner() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();

    for (group, expected_draws) in [("western", 1), ("wei", 0)] {
        let mut roster = content_schema::ContentPackage::from_json_bytes(STANDARD_PACKAGE).unwrap();
        roster.manifest.package_id = "eu-jiquan-test-roster".into();
        roster.character_index = [(9_219_001, "wei"), (9_219_002, group)]
            .into_iter()
            .map(|(id, group)| PackageCharacterEntry {
                voice_assets: Default::default(),
                id,
                name_key: format!("jiquan_group_{id}"),
                max_hp: 5,
                initial_hp: 5,
                skills: vec![],
                sex: "male".into(),
                group: group.into(),
                is_lord: false,
                clans: vec![],
                tags: vec![],
                sort_key: String::new(),
                portrait_asset: None,
                skin_assets: vec![],
            })
            .collect();
        roster.manifest.logic_hash = roster.compute_logic_hash();

        let mut engine = Engine::new();
        engine.load_package(&committed).unwrap();
        engine.load_package(&roster).unwrap();
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_826),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_826,
                ruleset: "identity-standard".into(),
            },
        });
        let owner = engine
            .state()
            .players
            .iter_live()
            .find(|player| player.identity == engine_core::state::Identity::Lord)
            .unwrap()
            .id;
        let current = engine
            .state()
            .players
            .iter_live()
            .find(|player| player.id != owner)
            .unwrap()
            .id;
        let mut tx = isolated(&engine);
        tx.state_mut().players.get_mut(owner).unwrap().character_def = 9_219_001;
        tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
        tx.state_mut()
            .players
            .get_mut(current)
            .unwrap()
            .character_def = 9_219_002;
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(EU_JIQUAN),
            }],
        )
        .unwrap();
        let hand = tx.state().hand_count(owner);
        match run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current,
                phase: engine_core::state::Phase::Prepare.to_u8(),
            }],
        ) {
            Ok(_) => {}
            Err(EffectError::ChoiceRequested) => {}
            Err(e) => panic!("SetTurnState error: {e:?}"),
        }
        assert_eq!(tx.state().hand_count(owner), hand + expected_draws);
        assert_eq!(
            tx.state().players.get(owner).unwrap().hp,
            3 + expected_draws as i32
        );
    }
}

#[test]
fn psdiwan_draws_the_declared_sha_target_count_for_its_user() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_827),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: 20_260_827,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let targets: Vec<_> = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let mut tx = isolated(&engine);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PSDIWAN),
        }],
    )
    .unwrap();
    let card = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Sha;
    let hand = tx.state().hand_count(owner);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardUsed {
                player: owner,
                card,
                targets: targets.clone(),
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        engine_core::effect::EffectError::ChoiceRequested
    );
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + targets.len() as u32);

    for (source, kind) in [(owner, CardKind::Tao), (targets[0], CardKind::Sha)] {
        let mut tx = isolated(&engine);
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(PSDIWAN),
            }],
        )
        .unwrap();
        let card = tx.state().hand_cards(source)[0];
        tx.state_mut().cards.get_mut(card).unwrap().kind = kind;
        assert!(dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardUsed {
                player: source,
                card,
                targets: targets.clone(),
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap()
        .is_empty());
        assert!(!tx.state().has_pending_choice());
    }
}

#[test]
fn hm_zhong_diamond_draws_two_after_damage_only() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_714_3),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714_3,
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
    let mut tx = isolated(&engine);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(HM_ZHONG_DIAMOND_SKILL),
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);
    let hp = tx.state().players.get(owner).unwrap().hp;

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 2);
    assert_eq!(tx.state().hand_count(owner), hand + 2);

    let hand = tx.state().hand_count(owner);
    for (target, is_lose_hp) in [(other, false), (owner, true)] {
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target,
                amount: 1,
                is_lose_hp,
                nature: 0,
            }],
        )
        .unwrap();
        assert_eq!(tx.state().hand_count(owner), hand);
    }
}
