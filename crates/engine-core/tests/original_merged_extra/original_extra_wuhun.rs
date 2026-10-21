//! Golden for EXTRA「武魂」的死亡反噬语义。

use engine_api::{
    CardDefId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, MarkDefId, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::effect::{run_effects, EffectError};
use engine_core::projection::project_player;
use engine_core::query::{query, QueryContext};
use engine_core::state::{CardKind, ContinuationKind, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const WUHUN: u32 = 4_260_309_017;
const XINJUEJING: u32 = 2_679_563_700;
const CHANGANDAJIAN_EQUIP5: u32 = 3_727_522_753;
const JUEJING: u32 = 3_977_263_126;
const WUSHEN: u32 = 2_400_906_129;

const NEW_WUHUN: u32 = 3_278_077_669;
const NEW_GUIXIN: u32 = 3_519_522_126;
const GUIXIN: u32 = 3_519_219_618;
const GUIXIN_SOURCE: &str = include_str!("../../../../content/original/ported/extra/guixin.skill");
const GUIXIN_LOGIC_HASH: [u8; 32] = [
    131, 229, 90, 4, 186, 198, 104, 70, 86, 232, 54, 203, 123, 216, 108, 106, 162, 185, 21, 128,
    76, 230, 51, 135, 225, 1, 120, 29, 62, 138, 32, 77,
];
const JILIN: u32 = 3_993_974_385;
const YINGYOU: u32 = 2_877_848_391;
const YINGTIAN: u32 = 4_275_778_563;
const YINGTIAN_EFFECT: u32 = 3_066_580_555;
const YINGTIAN_GUICAI: u32 = 2_431_815_904;
const YINGTIAN_WANSHA: u32 = 2_204_719_707;
const YINGTIAN_LIANPO: u32 = 2_579_407_392;
const DILI: u32 = 3_732_080_996;
const YUHENG: u32 = 3_006_389_674;
const DILI_LINES: [u32; 6] = [
    3_511_235_243,
    3_520_220_242,
    4_057_699_019,
    2_682_708_493,
    3_616_039_425,
    2_521_447_965,
];
const YUHENG_POOL: [u32; 18] = [
    2_790_748_306,
    3_138_095_197,
    2_238_473_770,
    3_285_976_176,
    3_859_813_014,
    3_075_552_814,
    4_211_343_786,
    4_068_014_439,
    2_231_312_450,
    2_455_708_599,
    2_662_349_862,
    2_398_589_569,
    2_873_874_515,
    3_295_441_821,
    2_509_094_453,
    3_040_931_439,
    3_834_102_895,
    3_473_961_755,
];
const CHANGANDAJIAN_EQUIP5_CARD: u32 = 4_174_045_670;
const NZRY_JUNLVE: u32 = 3_829_050_927;
const NZRY_JUNLVE_MARK: u32 = 1_681_567_279;
const NZRY_CUIKE: u32 = 2_516_554_182;
const NZRY_DINGHUO: u32 = 3_907_113_145;
const SHELIE: u32 = 4_211_343_786;
const GONGXIN: u32 = 3_100_453_257;
const YEYAN: u32 = 3_293_607_680;
const QINYIN: u32 = 3_841_403_199;
const RELONGHUN: u32 = 2_722_411_221;
const BAONU: u32 = 3_399_710_515;
const BAONU_MARK: u32 = 1_252_226_867;
const WUMOU: u32 = 4_039_474_555;
const OL_WUQIAN: u32 = 4_021_373_467;
const OL_SHENFEN: u32 = 2_674_950_952;
const WUQIAN: u32 = 2_841_820_068;
const SHENFEN: u32 = 2_409_921_791;

const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/extra/extra.npack.json");
const STANDARD_PACKAGE: &[u8] = include_bytes!("../../../../content/standard/standard.npack.json");

#[test]
fn guixin_external_abi_host_continuation_and_module_are_frozen() {
    let compiled =
        content_compiler::compile_source_named("guixin-contract", GUIXIN_SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 237);
    let module = compiled
        .modules
        .iter()
        .find(|module| module.name == "guixin_damage")
        .unwrap();
    assert_eq!(module.code, [83, 80, 231, 1]);
    assert_eq!(module.logic_hash, GUIXIN_LOGIC_HASH);
    assert_eq!(
        rule_vm::bytecode::effect_tag::CHOOSE_GAIN_CARD_FROM_EACH_OTHER_THEN_TURN_OVER,
        231
    );
    assert_eq!(
        ContinuationKind::ChooseGainCardFromEachOtherThenTurnOver.tag(),
        164
    );
    assert_eq!(
        ContinuationKind::from_tag(164),
        ContinuationKind::ChooseGainCardFromEachOtherThenTurnOver
    );

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let published = committed
        .modules
        .iter()
        .find(|module| module.name == "guixin_damage")
        .unwrap();
    assert_eq!(published.code, module.code);
    assert_eq!(published.logic_hash, module.logic_hash);
}

#[test]
fn committed_package_makes_the_living_killer_enter_dying() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut compiled = content_compiler::compile_source_named("extra", &extra_sources()).unwrap();
    compiled.character_index = committed.character_index.clone();
    compiled.card_index = committed.card_index.clone();
    compiled.manifest.dependencies = committed.manifest.dependencies.clone();
    compiled.manifest.logic_hash = compiled.compute_logic_hash();
    assert_eq!(committed.manifest.logic_hash, compiled.manifest.logic_hash);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == WUHUN));
    assert!(committed.character_index.iter().any(|character| {
        character.id == 2_268_373_161
            && character.max_hp == 5
            && character.skills == [WUSHEN, NEW_WUHUN]
    }));
    assert!(committed.character_index.iter().any(|character| {
        character.id == 3_839_224_428
            && character.max_hp == 3
            && character.skills == [NEW_GUIXIN, 2_669_566_023]
    }));
    assert!(committed.character_index.iter().any(|character| {
        character.id == 2_412_883_799
            && character.max_hp == 5
            && character.skills == [BAONU, WUMOU, OL_WUQIAN, OL_SHENFEN]
    }));
    assert!(committed.character_index.iter().any(|character| {
        character.id == 3_606_714_722
            && character.max_hp == 4
            && character.skills == [NZRY_JUNLVE, NZRY_CUIKE, NZRY_DINGHUO]
    }));
    assert!(committed.character_index.iter().any(|character| {
        character.id == 2_338_672_614
            && character.max_hp == 3
            && character.skills == [SHELIE, GONGXIN]
    }));
    assert!(committed.character_index.iter().any(|character| {
        character.id == 2_166_903_732
            && character.max_hp == 4
            && character.skills == [YEYAN, QINYIN]
    }));
    assert!(committed.character_index.iter().any(|character| {
        character.id == 3_401_672_399
            && character.max_hp == 2
            && character.skills == [XINJUEJING, RELONGHUN]
    }));
    assert!(committed.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == NEW_GUIXIN
            && trigger.filter == "owner_is_target_per_damage_point_other_hand_equip_judge_nonempty"
            && trigger.optional
    }));

    let mut engine = Engine::new();
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
    engine.load_package(&committed).unwrap();
    let killer = engine.state().turn.current;
    let victim = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != killer)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let players = tx.state().players.alive_by_seat();
    for (player, identity) in players.into_iter().zip([
        Identity::Lord,
        Identity::Loyalist,
        Identity::Rebel,
        Identity::Renegade,
    ]) {
        tx.state_mut().players.get_mut(player).unwrap().identity = identity;
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: victim,
            skill_def: SkillDefId::new(WUHUN),
        }],
    )
    .unwrap();
    let saver = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != killer && *player != victim)
        .unwrap();
    let tao = tx.state().hand_cards(saver)[0];
    tx.state_mut().cards.get_mut(tao).unwrap().kind = CardKind::Tao;

    let result = run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer,
        }],
    );
    assert!(
        matches!(result, Err(EffectError::ChoiceRequested)),
        "{result:?}"
    );
    let killer = tx.state().players.get(killer).unwrap();
    assert_eq!(killer.hp, 0);
    assert!(killer.alive && killer.dying);
}

#[test]
fn dili_and_yuheng_use_rule_rng_and_persist_the_fate_pool() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(DILI as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: DILI as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let mut tx = Transaction::begin(engine.state().clone());
    let owner = tx.state().turn.current;
    let old = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(DILI),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(YUHENG),
            },
        ],
    )
    .unwrap();
    let skill_by_def = |tx: &Transaction, def: u32| {
        tx.state()
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .find(|skill| {
                tx.state()
                    .skills
                    .get(**skill)
                    .is_some_and(|instance| instance.def.raw() == def)
            })
            .copied()
    };
    let dili = skill_by_def(&tx, DILI).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDiliYuheng {
            player: owner,
            action: 0,
            card: engine_api::CardId::NONE,
            event_cards: Vec::new(),
            skill: dili,
        }],
    )
    .unwrap();
    let lines: Vec<_> = DILI_LINES
        .iter()
        .filter_map(|def| skill_by_def(&tx, *def))
        .collect();
    assert_eq!(lines.len(), 1);
    assert!(!tx.state().skills.get(lines[0]).unwrap().enabled);

    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let yuheng = skill_by_def(&tx, YUHENG).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDiliYuheng {
            player: owner,
            action: 1,
            card: engine_api::CardId::NONE,
            event_cards: Vec::new(),
            skill: yuheng,
        }],
    )
    .unwrap();
    assert_eq!(
        YUHENG_POOL
            .iter()
            .filter(|def| skill_by_def(&tx, **def).is_some())
            .count(),
        2
    );
    let hp = tx.state().players.get(owner).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDiliYuheng {
            player: owner,
            action: 2,
            card: engine_api::CardId::NONE,
            event_cards: Vec::new(),
            skill: yuheng,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
    tx.state_mut()
        .skills
        .get_mut(yuheng)
        .unwrap()
        .play_phase_use_count = 1;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDiliYuheng {
            player: owner,
            action: 2,
            card: engine_api::CardId::NONE,
            event_cards: Vec::new(),
            skill: yuheng,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
}

#[test]
fn package_card_metadata_installs_and_destroys_changandajian() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.card_index.len(), 6);
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(CHANGANDAJIAN_EQUIP5_CARD as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: CHANGANDAJIAN_EQUIP5_CARD as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let mut tx = Transaction::begin(engine.state().clone());
    let owner = tx.state().turn.current;
    let old = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    let card = tx.state().hand_cards(owner)[0];
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    {
        let card = tx.state_mut().cards.get_mut(card).unwrap();
        card.def = CardDefId::new(CHANGANDAJIAN_EQUIP5_CARD);
        card.kind = CardKind::Treasure;
    }
    let base_limit = match query(
        tx.state(),
        &QueryRequest::HandLimit { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected hand-limit query: {other:?}"),
    };
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: equip,
            from: Some(hand),
        }],
    )
    .unwrap();
    assert!(tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == CHANGANDAJIAN_EQUIP5));
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(value) if value == base_limit + 2
    ));
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: discard,
            from: Some(equip),
        }],
    )
    .unwrap();
    let removed = tx
        .state()
        .zones
        .find(ZoneKind::Removed, PlayerId::NONE)
        .unwrap();
    assert_eq!(tx.state().cards.get(card).unwrap().zone, removed);
    assert!(!tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == CHANGANDAJIAN_EQUIP5));
}

#[test]
fn tiaojiyanmei_resumes_after_discard_and_can_give_that_card() {
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(38),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: 38,
            ruleset: "identity-standard".into(),
        },
    });
    let mut tx = Transaction::begin(engine.state().clone());
    let players = tx.state().players.alive_by_seat();
    let (actor, lower, higher) = (players[0], players[1], players[2]);
    let old = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    let lower_hand = tx.state().zones.find(ZoneKind::Hand, lower).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let reduce_lower = tx.state().hand_cards(lower)[..2].to_vec();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: reduce_lower,
            to: discard,
            from: Some(lower_hand),
        }],
    )
    .unwrap();
    assert_eq!(
        (tx.state().hand_count(lower), tx.state().hand_count(higher)),
        (2, 4)
    );
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![
                EffectRequest::DrawCards {
                    player: lower,
                    count: 1,
                },
                EffectRequest::BeginTiaojiyanmeiDiscard {
                    actor,
                    left: lower,
                    right: higher,
                    higher,
                },
            ],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let committed = tx.commit();
    let discard_choice = committed.output.choice.unwrap();
    let discarded = committed.state.hand_cards(higher)[0];
    let (state, output) = engine_core::command::apply_command(
        committed.state,
        Command {
            actor: higher,
            expected_revision: ExpectedRevision(discard_choice.revision),
            idempotency_key: IdempotencyKey(39),
            kind: CommandKind::ResolveChoice {
                choice_id: discard_choice.choice_id,
                selections: vec![discarded.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    let gain_choice = output
        .choice
        .expect("equal hands should offer discarded card");
    let (state, output) = engine_core::command::apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(gain_choice.revision),
            idempotency_key: IdempotencyKey(40),
            kind: CommandKind::ResolveChoice {
                choice_id: gain_choice.choice_id,
                selections: vec![lower.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    let gained = state.cards.get(discarded).unwrap();
    assert_eq!(gained.owner, lower);
    assert_eq!(
        state.zones.get(gained.zone).map(|zone| zone.kind),
        Some(ZoneKind::Hand)
    );
}

#[test]
fn xinjuejing_draws_on_both_dying_edges_and_adds_two_to_hand_limit() {
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
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XINJUEJING),
        }],
    )
    .unwrap();

    let base_limit = tx.state().players.get(owner).unwrap().hp;
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(value) if value == base_limit + 2
    ));
    let hand = tx.state().hand_count(owner);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;

    run_effects(&mut tx, vec![EffectRequest::EnterDying { player: owner }]).unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);

    run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source: owner,
            player: owner,
            amount: 1,
        }],
    )
    .unwrap();
    assert!(!tx.state().players.get(owner).unwrap().dying);
    assert_eq!(tx.state().hand_count(owner), hand + 2);
}

#[test]
fn juejing_adds_lost_hp_to_draw_count_and_two_to_hand_limit() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_817,
            ruleset: "identity-standard".into(),
        },
    });
    cancel_startup_choices(&mut engine);
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JUEJING),
        }],
    )
    .unwrap();

    let hp = tx.state().players.get(owner).unwrap().hp;
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(value) if value == hp + 2
    ));
    assert_eq!(engine_core::query::draw_count_for(tx.state(), owner), 2);

    tx.state_mut().players.get_mut(owner).unwrap().hp -= 2;
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(value) if value == hp
    ));
    assert_eq!(engine_core::query::draw_count_for(tx.state(), owner), 4);
}

#[test]
fn changandajian_equip5_adds_two_to_hand_limit() {
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
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(CHANGANDAJIAN_EQUIP5),
        }],
    )
    .unwrap();

    let base_limit = tx.state().players.get(owner).unwrap().hp;
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(value) if value == base_limit + 2
    ));
}

#[test]
fn wushen_turns_hearts_into_unlimited_unanswerable_normal_sha() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_715_1,
            ruleset: "identity-standard".into(),
        },
    });
    cancel_startup_choices(&mut engine);
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    for skill in tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| skill.id)
        .collect::<Vec<_>>()
    {
        run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(WUSHEN),
        }],
    )
    .unwrap();

    let cards = tx.state().hand_cards(owner);
    let heart = cards[0];
    let ordinary = cards[1];
    {
        let card = tx.state_mut().cards.get_mut(heart).unwrap();
        card.kind = CardKind::Tao;
        card.suit = Suit::Heart;
    }
    {
        let card = tx.state_mut().cards.get_mut(ordinary).unwrap();
        card.kind = CardKind::Sha;
        card.suit = Suit::Spade;
    }
    let shan = tx.state().hand_cards(target)[0];
    tx.state_mut().cards.get_mut(shan).unwrap().kind = CardKind::Shan;
    tx.state_mut().turn.phase = Phase::Play;

    let physical = tx.state().cards.get(heart).unwrap();
    assert_eq!(physical.kind, CardKind::Tao);
    assert_eq!(
        engine_core::query::effective_card_kind(tx.state(), owner, physical),
        CardKind::Sha
    );
    assert!(engine_core::query::card_use_unlimited_for_card(
        tx.state(),
        owner,
        heart
    ));
    assert!(engine_core::query::card_distance_unlimited_for_card(
        tx.state(),
        owner,
        heart
    ));
    assert!(engine_core::query::sha_unrespondable_for_card(
        tx.state(),
        owner,
        heart
    ));
    assert!(!engine_core::query::sha_unrespondable_for_card(
        tx.state(),
        owner,
        ordinary
    ));

    let hp = tx.state().players.get(target).unwrap().hp;
    let before = tx.state().clone();
    let (after, result) = engine_core::command::apply_command(
        before.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(before.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: heart,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert!(!after.has_pending_choice());
    assert_eq!(after.players.get(target).unwrap().hp, hp - 1);
    assert!(after.hand_cards(target).contains(&shan));
    assert_eq!(after.cards.get(heart).unwrap().kind, CardKind::Tao);
}

#[test]
fn new_wuhun_marks_damage_source_and_only_tao_or_taoyuan_prevents_death() {
    assert!(new_wuhun_death_result(CardKind::Tao));
    assert!(new_wuhun_death_result(CardKind::TrickTaoyuan));
    assert!(!new_wuhun_death_result(CardKind::Other));
}

#[test]
fn new_guixin_obeys_zone_priority_then_turns_over() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(NEW_GUIXIN as u64),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: NEW_GUIXIN as u64,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&package).unwrap();
    let players = engine.state().players.alive_by_seat();
    let (owner, equip_target, judge_target) = (players[1], players[2], players[0]);
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
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NEW_GUIXIN),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == NEW_GUIXIN)
        .unwrap()
        .id;

    let equip_card = tx.state().hand_cards(equip_target)[0];
    let equip_zone = tx
        .state()
        .zones
        .find(ZoneKind::Equip, equip_target)
        .unwrap();
    let equip_hand = tx.state().zones.find(ZoneKind::Hand, equip_target).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![equip_card],
            to: equip_zone,
            from: Some(equip_hand),
        }],
    )
    .unwrap();

    let judge_cards = tx.state().hand_cards(judge_target);
    let judge_card = judge_cards[0];
    let judge_zone = tx
        .state()
        .zones
        .find(ZoneKind::Judge, judge_target)
        .unwrap();
    let judge_hand = tx.state().zones.find(ZoneKind::Hand, judge_target).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![judge_card],
            to: judge_zone,
            from: Some(judge_hand),
        }],
    )
    .unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: judge_cards[1..].to_vec(),
            to: discard,
            from: Some(judge_hand),
        }],
    )
    .unwrap();

    let hand_before = tx.state().hand_count(owner);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![
                EffectRequest::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver {
                    player: owner,
                    priority: u8::MAX,
                    remaining_targets: Vec::new(),
                    skill,
                },
            ],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseZonePriorityGainRandomCardFromEachOtherThenTurnOver
    );
    let committed = tx.commit();
    let choice = committed.output.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::Custom);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [0, 1, 2]
    );
    let (state, result) = engine_core::command::apply_command(
        committed.state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(committed.state.revision),
            idempotency_key: IdempotencyKey(NEW_GUIXIN as u64 + 1),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert!(!state.has_pending_choice());
    assert!(state.players.get(owner).unwrap().turned_over);
    assert_eq!(state.hand_count(owner), hand_before + 3);
    let owner_hand = state.hand_cards(owner);
    assert!(owner_hand.contains(&equip_card));
    assert!(owner_hand.contains(&judge_card));
}

#[test]
fn guixin_chooses_one_hej_card_from_each_other_then_turns_over() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(GUIXIN as u64),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: GUIXIN as u64,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&package).unwrap();
    let players = engine.state().players.alive_by_seat();
    let (owner, equip_target, judge_target) = (players[0], players[1], players[2]);
    let mut tx = Transaction::begin(engine.state().clone());
    let old = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(GUIXIN),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == GUIXIN)
        .unwrap()
        .id;
    let equip_card = tx.state().hand_cards(equip_target)[0];
    let equip_hand = tx.state().zones.find(ZoneKind::Hand, equip_target).unwrap();
    let equip_zone = tx
        .state()
        .zones
        .find(ZoneKind::Equip, equip_target)
        .unwrap();
    let judge_card = tx.state().hand_cards(judge_target)[0];
    let judge_hand = tx.state().zones.find(ZoneKind::Hand, judge_target).unwrap();
    let judge_zone = tx
        .state()
        .zones
        .find(ZoneKind::Judge, judge_target)
        .unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::MoveCards {
                cards: vec![equip_card],
                to: equip_zone,
                from: Some(equip_hand),
            },
            EffectRequest::MoveCards {
                cards: vec![judge_card],
                to: judge_zone,
                from: Some(judge_hand),
            },
        ],
    )
    .unwrap();
    let hand_before = tx.state().hand_count(owner);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ChooseGainCardFromEachOtherThenTurnOver {
                player: owner,
                initialized: false,
                remaining_targets: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseGainCardFromEachOtherThenTurnOver
    );
    let payload = &tx.state().resolution.continuation_payload;
    assert_eq!(
        &payload[..3],
        [owner.raw(), equip_target.raw(), skill.raw()]
    );
    assert_eq!(payload.len(), 4 + payload[3] as usize);
    assert!(payload[4..].contains(&equip_card.raw()));
    let committed = tx.commit();
    let first = committed.output.choice.unwrap();
    assert!(first
        .options
        .iter()
        .any(|option| option.token == equip_card.raw()));
    let (state, second_result) = engine_core::command::apply_command(
        committed.state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(committed.state.revision),
            idempotency_key: IdempotencyKey(GUIXIN as u64 + 1),
            kind: CommandKind::ResolveChoice {
                choice_id: first.choice_id,
                selections: vec![equip_card.raw()],
                cancelled: false,
            },
        },
    );
    let second = second_result.choice.unwrap();
    assert!(second
        .options
        .iter()
        .any(|option| option.token == judge_card.raw()));
    let (state, result) = engine_core::command::apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(GUIXIN as u64 + 2),
            kind: CommandKind::ResolveChoice {
                choice_id: second.choice_id,
                selections: vec![judge_card.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert!(state.players.get(owner).unwrap().turned_over);
    assert_eq!(state.hand_count(owner), hand_before + 2);
    let hand = state.hand_cards(owner);
    assert!(hand.contains(&equip_card));
    assert!(hand.contains(&judge_card));
}

#[test]
fn jilin_hidden_pile_reveals_exchanges_and_drives_yingyou_draw() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(JILIN as u64),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: JILIN as u64,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&package).unwrap();
    let players = engine.state().players.alive_by_seat();
    let (owner, observer) = (players[0], players[1]);
    let mut tx = Transaction::begin(engine.state().clone());
    let old = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JILIN),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(YINGYOU),
            },
        ],
    )
    .unwrap();
    let skill = |tx: &Transaction, def| {
        tx.state()
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .copied()
            .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == def)
            .unwrap()
    };
    let jilin = skill(&tx, JILIN);
    let yingyou = skill(&tx, YINGYOU);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveHiddenSkillPile {
            player: owner,
            pile_skill_def: SkillDefId::new(JILIN),
            action: 0,
            card: engine_api::CardId::NONE,
            stage: 0,
            hand_cards: Vec::new(),
            pile_cards: Vec::new(),
            event_cards: Vec::new(),
            skill: jilin,
        }],
    )
    .unwrap();
    let hidden = tx.state().hidden_skill_pile_cards(owner, JILIN);
    assert_eq!(hidden.len(), 2);
    let owner_view = project_player(tx.state(), owner);
    let other_view = project_player(tx.state(), observer);
    for card in &hidden {
        assert!(owner_view
            .known_cards
            .iter()
            .any(|view| view.id == *card && view.def.is_some()));
        assert!(other_view
            .known_cards
            .iter()
            .any(|view| view.id == *card && view.def.is_none()));
    }

    let mut cancel_tx = Transaction::begin(tx.state().clone());
    let attacker = observer;
    let sha = cancel_tx.state().hand_cards(attacker)[0];
    cancel_tx.state_mut().resolution.resume_kind = ContinuationKind::ResumePlayedSha;
    cancel_tx.state_mut().resolution.resume_payload =
        vec![attacker.raw(), owner.raw(), 1, 0, 1, sha.raw()];
    let hp_before_cancel = cancel_tx.state().players.get(owner).unwrap().hp;
    assert!(matches!(
        run_effects(
            &mut cancel_tx,
            vec![EffectRequest::ResolveHiddenSkillPile {
                player: owner,
                pile_skill_def: SkillDefId::new(JILIN),
                action: 1,
                card: sha,
                stage: 0,
                hand_cards: Vec::new(),
                pile_cards: Vec::new(),
                event_cards: Vec::new(),
                skill: jilin,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let committed = cancel_tx.commit();
    let choice = committed.output.choice.unwrap();
    let (cancelled_state, cancel_result) = engine_core::command::apply_command(
        committed.state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(choice.revision),
            idempotency_key: IdempotencyKey(JILIN as u64 + 10),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![hidden[0].raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(
        cancel_result.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        cancelled_state.players.get(owner).unwrap().hp,
        hp_before_cancel
    );
    assert!(cancelled_state.cards.get(hidden[0]).unwrap().face_up);

    let hand_before_reveal = tx.state().hand_count(owner);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveHiddenSkillPile {
                player: owner,
                pile_skill_def: SkillDefId::new(JILIN),
                action: 3,
                card: engine_api::CardId::NONE,
                stage: 0,
                hand_cards: Vec::new(),
                pile_cards: Vec::new(),
                event_cards: Vec::new(),
                skill: yingyou,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let committed = tx.commit();
    let choice = committed.output.choice.unwrap();
    let (state, reveal_result) = engine_core::command::apply_command(
        committed.state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(choice.revision),
            idempotency_key: IdempotencyKey(JILIN as u64 + 1),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![hidden[0].raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(
        reveal_result.result,
        CommandResult::Accepted { .. }
    ));
    assert!(state.cards.get(hidden[0]).unwrap().face_up);
    assert_eq!(state.hand_count(owner), hand_before_reveal + 1);

    let mut tx = Transaction::begin(state);
    let outgoing = tx.state().hand_cards(owner)[0];
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveHiddenSkillPile {
                player: owner,
                pile_skill_def: SkillDefId::new(JILIN),
                action: 2,
                card: engine_api::CardId::NONE,
                stage: 0,
                hand_cards: Vec::new(),
                pile_cards: Vec::new(),
                event_cards: Vec::new(),
                skill: jilin,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let committed = tx.commit();
    let hand_choice = committed.output.choice.unwrap();
    let (state, pile_result) = engine_core::command::apply_command(
        committed.state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(hand_choice.revision),
            idempotency_key: IdempotencyKey(JILIN as u64 + 2),
            kind: CommandKind::ResolveChoice {
                choice_id: hand_choice.choice_id,
                selections: vec![outgoing.raw()],
                cancelled: false,
            },
        },
    );
    let pile_choice = pile_result.choice.unwrap();
    let (state, exchange_result) = engine_core::command::apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(pile_choice.revision),
            idempotency_key: IdempotencyKey(JILIN as u64 + 3),
            kind: CommandKind::ResolveChoice {
                choice_id: pile_choice.choice_id,
                selections: vec![hidden[1].raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(
        exchange_result.result,
        CommandResult::Accepted { .. }
    ));
    assert!(state.hand_cards(owner).contains(&hidden[1]));
    assert!(state
        .hidden_skill_pile_cards(owner, JILIN)
        .contains(&outgoing));

    let mut tx = Transaction::begin(state);
    let matching = tx.state().hand_cards(owner)[0];
    let revealed_suit = tx.state().cards.get(hidden[0]).unwrap().suit;
    tx.state_mut().cards.get_mut(matching).unwrap().suit = revealed_suit;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand_before_loss = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![matching],
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand_before_loss);
}

#[test]
fn yingtian_helpers_apply_distance_and_only_suppress_eligible_skills() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    for skill_def in [
        YINGTIAN,
        YINGTIAN_EFFECT,
        YINGTIAN_GUICAI,
        YINGTIAN_WANSHA,
        YINGTIAN_LIANPO,
    ] {
        assert!(
            package
                .skill_index
                .iter()
                .any(|skill| skill.skill_def_id == skill_def),
            "missing skill {skill_def}"
        );
    }

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(YINGTIAN as u64),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: YINGTIAN as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let mut tx = Transaction::begin(engine.state().clone());
    let owner = tx.state().turn.current;
    let others: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let old = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(YINGTIAN_WANSHA),
            },
            EffectRequest::AddSkill {
                player: others[0],
                skill_def: SkillDefId::new(YINGTIAN_EFFECT),
            },
            EffectRequest::AddSkill {
                player: others[0],
                skill_def: SkillDefId::new(YINGTIAN_GUICAI),
            },
            EffectRequest::AddSkill {
                player: others[1],
                skill_def: SkillDefId::new(YINGTIAN_GUICAI),
            },
        ],
    )
    .unwrap();
    let skill = |tx: &Transaction, player, def| {
        tx.state()
            .players
            .get(player)
            .unwrap()
            .skills
            .iter()
            .copied()
            .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == def)
            .unwrap()
    };
    let source = skill(&tx, owner, YINGTIAN_WANSHA);
    let locked = skill(&tx, others[0], YINGTIAN_EFFECT);
    let non_locked = skill(&tx, others[0], YINGTIAN_GUICAI);
    let dying_non_locked = skill(&tx, others[1], YINGTIAN_GUICAI);
    assert!(engine_core::query::card_distance_unlimited_for(
        tx.state(),
        others[0]
    ));

    tx.state_mut().players.get_mut(others[1]).unwrap().dying = true;
    run_effects(
        &mut tx,
        vec![EffectRequest::SetNonLockedSkillSuppression {
            player: owner,
            enabled: true,
            skill: source,
        }],
    )
    .unwrap();
    assert!(!tx
        .state()
        .skill_effectively_enabled(tx.state().skills.get(non_locked).unwrap()));
    assert!(tx
        .state()
        .skill_effectively_enabled(tx.state().skills.get(locked).unwrap()));
    assert!(tx
        .state()
        .skill_effectively_enabled(tx.state().skills.get(dying_non_locked).unwrap()));

    tx.state_mut().players.get_mut(others[1]).unwrap().dying = false;
    run_effects(
        &mut tx,
        vec![EffectRequest::SetNonLockedSkillSuppression {
            player: owner,
            enabled: false,
            skill: source,
        }],
    )
    .unwrap();
    assert!(tx
        .state()
        .skill_effectively_enabled(tx.state().skills.get(non_locked).unwrap()));
}

#[test]
fn nzry_junlve_adds_one_mark_for_each_damage_point_dealt_or_taken() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(NZRY_JUNLVE as u64),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: NZRY_JUNLVE as u64,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&package).unwrap();
    let owner = engine.state().turn.current;
    let other = engine
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
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NZRY_JUNLVE),
        }],
    )
    .unwrap();

    let marks = |tx: &Transaction| {
        tx.state()
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .find(|(mark, _)| *mark == NZRY_JUNLVE_MARK)
            .map(|(_, count)| *count)
            .unwrap_or(0)
    };
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
    .unwrap();
    assert_eq!(marks(&tx), 2);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(marks(&tx), 3);
}

#[test]
fn baonu_starts_at_two_before_the_first_turn_and_tracks_damage_amount() {
    let mut skill_package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut roster = content_schema::ContentPackage::from_json_bytes(STANDARD_PACKAGE).unwrap();
    for character in &mut skill_package.character_index {
        character.skills = vec![BAONU];
    }
    skill_package.manifest.logic_hash = skill_package.compute_logic_hash();
    for character in &mut roster.character_index {
        character.skills = vec![BAONU];
    }
    roster.manifest.logic_hash = roster.compute_logic_hash();

    let mut engine = Engine::new();
    engine.load_package(&skill_package).unwrap();
    engine.load_package(&roster).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(BAONU as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: BAONU as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let mark_count = |state: &engine_core::state::GameState, player: PlayerId| {
        state
            .players
            .get(player)
            .unwrap()
            .marks
            .iter()
            .find(|(mark, _)| *mark == BAONU_MARK)
            .map(|(_, count)| *count)
            .unwrap_or(0)
    };
    let players = engine.state().players.alive_by_seat();
    for player in &players {
        assert_eq!(mark_count(engine.state(), *player), 2);
    }
    let mark_event = engine
        .last_events()
        .iter()
        .position(|event| {
            matches!(
                event,
                DomainEvent::MarkChanged { mark_def, count: 2, .. } if *mark_def == BAONU_MARK
            )
        })
        .expect("game-start mark event");
    let first_turn = engine
        .last_events()
        .iter()
        .position(|event| matches!(event, DomainEvent::TurnStateChanged { .. }))
        .expect("first turn event");
    assert!(mark_event < first_turn);

    let source = players[0];
    let target = players[1];
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(mark_count(tx.state(), source), 4);
    assert_eq!(mark_count(tx.state(), target), 4);
}

#[test]
fn ol_wuqian_requires_two_marks_and_wushuang_expires_without_removing_permanent_skill() {
    let (state, owner, skill) = extra_active_state(OL_WUQIAN, 1, 2);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let (_, rejected) = activate_extra(state, owner, skill, vec![target], 1);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let (state, owner, skill) = extra_active_state(OL_WUQIAN, 4, 2);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let (mut state, accepted) = activate_extra(state, owner, skill, vec![target], 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    let (same_state, repeated) = activate_extra(state.clone(), owner, skill, vec![target], 20);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
    assert_eq!(same_state, state);
    let temporary = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == 18)
        .unwrap();
    assert_eq!(temporary.temporary_until_turn_end_of, owner);
    state.turn.phase = Phase::End;
    let revision = state.revision;
    let (state, ended) = engine_core::command::apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(ended.result, CommandResult::Accepted { .. }));
    assert!(!state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == 18));

    let (state, owner, skill) = extra_active_state(OL_WUQIAN, 2, 2);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(18),
        }],
    )
    .unwrap();
    let target = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let (mut state, accepted) = activate_extra(tx.state().clone(), owner, skill, vec![target], 4);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == owner && skill.def.raw() == 18)
            .unwrap()
            .temporary_until_turn_end_of,
        PlayerId::NONE
    );
    state.turn.phase = Phase::End;
    let revision = state.revision;
    let (state, _) = engine_core::command::apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(5),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == 18));
}

#[test]
fn wuqian_requires_missing_wushuang_then_spends_two_marks_for_temporary_wushuang() {
    let (state, owner, wuqian) = extra_active_state(WUQIAN, 2, 2);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(18),
        }],
    )
    .unwrap();
    let state = tx.state().clone();
    let (unchanged, rejected) = activate_extra(state.clone(), owner, wuqian, Vec::new(), 21);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(unchanged, state);

    let wushuang = state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == 18)
        .unwrap();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::RemoveSkill { skill: wushuang }],
    )
    .unwrap();
    let (state, accepted) = activate_extra(tx.state().clone(), owner, wuqian, Vec::new(), 22);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .find(|(mark, _)| *mark == BAONU_MARK)
            .map_or(0, |(_, count)| *count),
        0
    );
    let temporary = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == 18)
        .unwrap();
    assert_eq!(temporary.temporary_until_turn_end_of, owner);
}

#[test]
fn wumou_loses_hp_without_baonu_and_offers_the_two_forced_branches_with_baonu() {
    let (state, owner, _) = extra_active_state(WUMOU, 0, 2);
    let mut tx = Transaction::begin(state);
    let card = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::TrickWuzhong;
    let hp = tx.state().players.get(owner).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: owner,
            card,
            targets: Vec::new(),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);

    let (state, owner, _) = extra_active_state(WUMOU, 1, 2);
    let mut tx = Transaction::begin(state);
    let card = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::TrickWuzhong;
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: owner,
                card,
                targets: Vec::new(),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::VmResume
    );
}

#[test]
fn ol_shenfen_resolves_damage_equipment_chosen_hand_discard_and_turn_over_in_order() {
    let (state, owner, skill) = extra_active_state(OL_SHENFEN, 6, 2);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: target,
            count: 2,
        }],
    )
    .unwrap();
    let equip = tx.state().hand_cards(target)[0];
    let hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, target).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![equip],
            to: equip_zone,
            from: Some(hand),
        }],
    )
    .unwrap();
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let (state, activated) = activate_extra(tx.state().clone(), owner, skill, Vec::new(), 10);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert!(state.zones.get(equip_zone).unwrap().cards.is_empty());
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::DiscardHandForMassDamage
    );
    let offered = state
        .resolution
        .continuation_payload
        .get(3..)
        .unwrap()
        .to_vec();
    let selected = offered.iter().rev().take(4).copied().collect::<Vec<_>>();
    let choice_id = state.resolution.pending_choice_id;
    let revision = state.revision;
    let (state, resolved) = engine_core::command::apply_command(
        state,
        Command {
            actor: target,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(11),
            kind: CommandKind::ResolveChoice {
                choice_id: engine_api::ChoiceId::new(choice_id),
                selections: selected,
                cancelled: false,
            },
        },
    );
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(target), 1);
    assert!(state.players.get(owner).unwrap().turned_over);
}

#[test]
fn shenfen_once_discards_four_combined_hand_equip_without_turning_over() {
    let (state, owner, skill) = extra_active_state(SHENFEN, 6, 2);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: target,
            count: 2,
        }],
    )
    .unwrap();
    let equip = tx.state().hand_cards(target)[0];
    let hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, target).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![equip],
            to: equip_zone,
            from: Some(hand),
        }],
    )
    .unwrap();
    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, activated) = activate_extra(tx.state().clone(), owner, skill, Vec::new(), 30);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert!(!state.players.get(owner).unwrap().turned_over);
    let offered = state.resolution.continuation_payload[4..].to_vec();
    assert!(offered.contains(&equip.raw()));
    let mut selected = vec![equip.raw()];
    selected.extend(
        offered
            .into_iter()
            .filter(|card| *card != equip.raw())
            .take(3),
    );
    let choice_id = state.resolution.pending_choice_id;
    let revision = state.revision;
    let (state, resolved) = engine_core::command::apply_command(
        state,
        Command {
            actor: target,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(31),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice_id),
                selections: selected,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(resolved.result, CommandResult::Accepted { .. }),
        "{resolved:?}"
    );
    assert!(!state.has_pending_choice());
    assert!(state.zones.get(equip_zone).unwrap().cards.is_empty());
    assert_eq!(state.hand_count(target), 2);
    assert!(!state.players.get(owner).unwrap().turned_over);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: owner,
            mark: MarkDefId::new(BAONU_MARK),
            delta: 6,
        }],
    )
    .unwrap();
    let (_, repeated) = activate_extra(tx.state().clone(), owner, skill, Vec::new(), 32);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
}

#[path = "extra_skill_support.rs"]
mod extra_skill_support;
use extra_skill_support::*;
