//! Golden for DIY「天弓」雷伤子技能。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const TIANGONG: u32 = 3_918_974_780;
const TIANGONG2: u32 = 3_632_366_807;
const NSXINSHENG: u32 = 2_604_261_106;
const NSLINGYING: u32 = 4_189_702_714;
const NSJIANSHU: u32 = 2_657_107_569;
const NSCANGJIAN: u32 = 3_998_621_536;
const JUNKTAOLUAN3: u32 = 3_391_710_812;
const NSGUANYONG: u32 = 2_587_050_995;
const NSXUEZHU: u32 = 2_233_994_945;
const NSGEFA: u32 = 2_990_982_842;
/// Concatenate every committed diy source in pack order (alphabetical),
/// mirroring `content-cli pack` input concatenation so the logic hash matches
/// the committed npack regardless of future roster growth.
fn diy_sources() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/diy");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension().map_or(false, |ext| ext == "skill")
        })
        .map(|entry| entry.path())
        .collect();
    files.sort();
    files
        .iter()
        .map(|path| {
            let mut source = std::fs::read_to_string(path).unwrap();
            source.push('\n');
            source
        })
        .collect()
}

const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");


fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_000_000 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn committed_package_draws_only_after_thunder_damage_from_owner() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut compiled = content_compiler::compile_source_named("diy", &diy_sources()).unwrap();
    compiled.character_index = committed.character_index.clone();
    compiled.manifest.dependencies = committed.manifest.dependencies.clone();
    compiled.manifest.logic_hash = compiled.compute_logic_hash();
    assert_eq!(committed.manifest.logic_hash, compiled.manifest.logic_hash);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == TIANGONG));
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == TIANGONG2));

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
    let mut tx = Transaction::begin(engine.state().clone());
    let target = tx.state_mut().players.get_mut(other).unwrap();
    target.hp = 10;
    target.max_hp = 10;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TIANGONG2),
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);

    for (nature, expected) in [
        (rule_api::damage_nature::NORMAL, hand),
        (rule_api::damage_nature::THUNDER, hand + 1),
    ] {
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target: other,
                amount: 1,
                is_lose_hp: false,
                nature,
            }],
        )
        .unwrap();
        assert_eq!(tx.state().hand_count(owner), expected);
    }
}

#[test]
fn tiangong_parent_prevents_incoming_thunder_and_keeps_group_draw() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_715),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_715,
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
    // 清除开局角色的默认技能，避免并发新增角色的可选触发干扰结算。
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TIANGONG),
    });
    run_effects(&mut tx, effects).unwrap();
    let hp = tx.state().players.get(owner).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::THUNDER,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp);

    let target = tx.state_mut().players.get_mut(other).unwrap();
    target.hp = 10;
    target.max_hp = 10;
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::THUNDER,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);
}

#[test]
fn nsxinsheng_is_optional_and_scales_with_damage() {
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
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let target = tx.state_mut().players.get_mut(other).unwrap();
    target.hp = 10;
    target.max_hp = 10;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NSXINSHENG),
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    assert_eq!(result.unwrap_err(), EffectError::ChoiceRequested);

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
    assert_eq!(state.players.get(owner).unwrap().max_hp, max_hp + 2);
    assert_eq!(state.hand_count(owner), hand + 2);

    let mut wounded = Transaction::begin(state);
    run_effects(
        &mut wounded,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert!(!wounded.state().has_pending_choice());
    assert_eq!(
        wounded.state().players.get(owner).unwrap().max_hp,
        max_hp + 2
    );
    assert_eq!(wounded.state().hand_count(owner), hand + 2);
}

#[test]
fn nslingying_removes_sha_distance_and_adds_one_use() {
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
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NSLINGYING),
        }],
    )
    .unwrap();
    assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 2);
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::AttackRange { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(i32::MAX)
    ));
}

#[test]
fn nsjianshu_requires_an_extra_shan_only_while_a_weapon_is_equipped() {
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
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NSJIANSHU),
        }],
    )
    .unwrap();
    assert_eq!(
        engine_core::query::response_count_for(tx.state(), owner, 1),
        1
    );

    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let weapon = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(92_080_001),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 3,
        zone: equip,
        owner,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(equip)
        .unwrap()
        .cards
        .push(weapon);
    assert_eq!(
        engine_core::query::response_count_for(tx.state(), owner, 1),
        2
    );

    tx.state_mut().zones.get_mut(equip).unwrap().cards.clear();
    assert_eq!(
        engine_core::query::response_count_for(tx.state(), owner, 1),
        1
    );
}

#[test]
fn junktaoluan3_loses_one_hp_only_at_its_owners_turn_end() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_714_380,
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
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JUNKTAOLUAN3),
        }],
    )
    .unwrap();
    let hp = tx.state().players.get(owner).unwrap().hp;

    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: other,
            phase: Phase::End.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp);

    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::End.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
}

#[test]
fn nscangjian_only_offers_the_damaged_players_equipment() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_715_530,
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
    let mut tx = Transaction::begin(engine.state().clone());
    let old_skills: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old_skills).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NSCANGJIAN),
        }],
    )
    .unwrap();
    let equip_zone = tx.state().zones.find(ZoneKind::Equip, target).unwrap();
    let equip = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_999),
        kind: CardKind::Weapon,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 2,
        zone: equip_zone,
        owner: target,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(equip_zone)
        .unwrap()
        .cards
        .push(equip);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let optional = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, accepted) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(11),
            kind: CommandKind::ResolveChoice {
                choice_id: optional,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    let choice = accepted.choice.unwrap();
    assert_eq!(choice.options.len(), 1);
    assert_eq!(choice.options[0].token, equip.raw());

    let take = ChoiceId::new(state.resolution.pending_choice_id);
    let forged_hand = state.hand_cards(target)[0];
    let (_, rejected) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(12),
            kind: CommandKind::ResolveChoice {
                choice_id: take,
                selections: vec![forged_hand.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(13),
            kind: CommandKind::ResolveChoice {
                choice_id: take,
                selections: vec![equip.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(equip).unwrap().owner, owner);
    assert_eq!(
        state.cards.get(equip).unwrap().zone,
        state.zones.find(ZoneKind::Hand, owner).unwrap()
    );
}

#[test]
fn nsguanyong_uses_any_hand_card_as_response_sha_only() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_827,
            ruleset: "identity-standard".into(),
        },
    });
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
    let setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(
            [source, target]
                .into_iter()
                .map(|player| EffectRequest::DiscardFromHand {
                    player,
                    count: u32::MAX,
                }),
        )
        .chain([EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(NSGUANYONG),
        }])
        .collect();
    run_effects(&mut tx, setup).unwrap();
    let juedou = inject(
        &mut tx,
        source,
        ZoneKind::Hand,
        CardKind::TrickJuedou,
        Suit::Spade,
        10,
    );
    let hand_card = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Heart,
        11,
    );
    let equip_card = inject(
        &mut tx,
        target,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Club,
        12,
    );

    let revision = tx.state().revision;
    let (_, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: source,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: juedou,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let choice = result.choice.expect("冠勇应提供手牌作为响应杀");
    assert_eq!(choice.observer, target);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == hand_card.raw()));
    assert!(!choice
        .options
        .iter()
        .any(|option| option.token == equip_card.raw()));
}

#[test]
fn nsgefa_only_uses_a_club_card_while_its_owner_is_dying() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let view = committed
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == NSGEFA)
        .unwrap();
    assert_eq!(view.as_kind, "tao");
    assert_eq!(view.window, "response");
    assert_eq!(view.filter, "owner_dying_club");
    assert_eq!(view.source_zone.as_deref(), Some("hand_equip"));
    assert_eq!(committed.manifest.rule_vm_abi_version, 684);

    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_733),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_733,
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
    let setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }])
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NSGEFA),
        }])
        .collect();
    run_effects(&mut tx, setup).unwrap();
    let non_club = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Heart,
        13,
    );
    let club = inject(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Club,
        14,
    );

    assert_eq!(engine_core::query::dying_save_card(tx.state(), owner), None);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        engine_core::query::dying_save_card(tx.state(), owner),
        Some(club)
    );

    let committed = tx.commit();
    let choice_id = ChoiceId::new(committed.state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        committed.state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_734),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let player = state.players.get(owner).unwrap();
    assert!(player.alive && !player.dying);
    assert_eq!(player.hp, 1);
    assert!(state.hand_cards(owner).contains(&non_club));
    assert_eq!(
        state.cards.get(club).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}

#[test]
fn nsxuezhu_draws_for_and_turns_over_the_damaged_player_from_both_roles() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NSXUEZHU && !skill.locked));
    let trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NSXUEZHU)
        .unwrap();
    assert_eq!(
        (trigger.event.as_str(), trigger.filter.as_str()),
        ("damage", "owner_is_source_or_target")
    );
    assert!(trigger.optional);

    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_716_06),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_716_06,
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
    let mut tx = Transaction::begin(engine.state().clone());
    let mut setup: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    setup.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(NSXUEZHU),
    });
    run_effects(&mut tx, setup).unwrap();
    for player in [owner, other] {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.max_hp = 10;
        state.hp = 10;
    }

    for (key, source, target) in [(1, owner, other), (2, other, owner)] {
        let hand = tx.state().hand_count(target);
        assert_eq!(
            run_effects(
                &mut tx,
                vec![EffectRequest::DealDamage {
                    source,
                    target,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
            .unwrap_err(),
            EffectError::ChoiceRequested
        );
        let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
        let (state, result) = apply_command(
            tx.state().clone(),
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(tx.state().revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: vec![1],
                    cancelled: false,
                },
            },
        );
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        assert_eq!(state.hand_count(target), hand + 2);
        assert!(state.players.get(target).unwrap().turned_over);
        tx = Transaction::begin(state);
    }
}
