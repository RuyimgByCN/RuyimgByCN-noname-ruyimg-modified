//! Golden for KEY「女装」的结束阶段摸牌语义。

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const RIKI_NVZHUANG: u32 = 3_571_736_403;
const KOTORI_SKILL_WU: u32 = 2_195_433_348;
const KOTORI_SKILL_SHU: u32 = 4_206_024_959;
const YUIKO_FENGLUN2: u32 = 2_522_572_214;
const YOSHINO_FAIL: u32 = 3_956_929_622;
const YUI_LIEYIN0: u32 = 3_080_747_547;
const YUI_LIEYIN1: u32 = 3_727_853_068;
const DORUJI_FEIQU: u32 = 3_475_207_521;
const GODAN_FEIQU: u32 = 3_490_863_084;
const ABYUSA_DUNYING: u32 = 3_195_600_790;
const SASAMI_GONGQING: u32 = 4_263_354_141;
const XINFU_GONGQING: u32 = 3_566_698_689;
const AKANE_JUGU: u32 = 2_452_932_850;
const JUGU: u32 = 2_684_521_577;
const MIO_TUIFU: u32 = 3_496_176_961;
const KEY_NODA: u32 = 3_057_217_957;
const KEY_YOUTA: u32 = 3_271_138_322;
const NODA_FENGCHENG: u32 = 2_452_779_390;
const NODA_XUNXIN: u32 = 3_836_596_548;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/key/key.npack.json");
const SP_PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");
const STANDARD_PACKAGE: &[u8] = include_bytes!("../../../../content/standard/standard.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn key_source() -> String {
    let dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../content/original/ported/key");
    let mut paths: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "skill"))
        .collect();
    paths.sort();
    paths
        .into_iter()
        .map(|path| std::fs::read_to_string(path).unwrap())
        .collect()
}

fn started_engine() -> Engine {
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
    engine.load_package(&package()).unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let natural_skills: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, natural_skills).unwrap();
    let clean = engine_core::Snapshot::capture(&tx.commit().state);
    engine.restore_snapshot(&clean).unwrap();
    engine
}

fn add_skill(tx: &mut Transaction, owner: PlayerId) {
    add_skill_def(tx, owner, RIKI_NVZHUANG);
}

fn add_skill_def(tx: &mut Transaction, owner: PlayerId, skill_def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
}

fn other_player(tx: &Transaction, owner: PlayerId) -> PlayerId {
    tx.state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id
}

fn remove_natural_responses(tx: &mut Transaction, player: PlayerId) {
    for card in tx.state().hand_cards(player) {
        tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
    }
}

fn enter_end_phase(tx: &mut Transaction, owner: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: engine_core::state::Phase::End.to_u8(),
        }],
    )
    .unwrap();
}

#[test]
fn committed_package_matches_source_and_draws_by_hand_state() {
    let committed = package();
    let mut compiled = content_compiler::compile_source_named("key", &key_source()).unwrap();
    compiled.character_index = committed.character_index.clone();
    compiled.manifest.dependencies = committed.manifest.dependencies.clone();
    compiled.manifest.logic_hash = compiled.compute_logic_hash();
    assert_eq!(committed.manifest.logic_hash, compiled.manifest.logic_hash);
    assert_eq!(committed.skill_index.len(), 261);
    assert_eq!(committed.character_index.len(), 88);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == RIKI_NVZHUANG));
    let noda = committed
        .character_index
        .iter()
        .find(|character| character.id == KEY_NODA)
        .expect("key_noda must be published with its complete skill closure");
    assert_eq!(noda.name_key, "key_noda");
    assert_eq!(noda.sex, "male");
    assert_eq!(noda.group, "key");
    assert_eq!(noda.max_hp, 4);
    let effective_initial_hp = if noda.initial_hp == 0 {
        noda.max_hp
    } else {
        noda.initial_hp
    };
    assert_eq!(effective_initial_hp, 4);
    assert_eq!(noda.skills, [NODA_FENGCHENG, NODA_XUNXIN]);

    let youta = committed
        .character_index
        .iter()
        .find(|character| character.id == KEY_YOUTA)
        .expect("key_youta must be published as the legacy zero-skill character");
    assert_eq!(youta.name_key, "key_youta");
    assert_eq!(youta.sex, "male");
    assert_eq!(youta.group, "key");
    assert_eq!(youta.max_hp, 4);
    assert_eq!(youta.initial_hp, 0);
    assert!(youta.skills.is_empty());

    let engine = started_engine();
    let owner = engine.state().turn.current;

    let mut with_hand = Transaction::begin(engine.state().clone());
    add_skill(&mut with_hand, owner);
    let before = with_hand.state().hand_count(owner);
    assert!(before > 0);
    enter_end_phase(&mut with_hand, owner);
    assert_eq!(with_hand.state().hand_count(owner), before + 1);

    let mut empty = Transaction::begin(engine.state().clone());
    add_skill(&mut empty, owner);
    run_effects(
        &mut empty,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    assert_eq!(empty.state().hand_count(owner), 0);
    enter_end_phase(&mut empty, owner);
    assert_eq!(empty.state().hand_count(owner), 2);
}

#[test]
fn mio_tuifu_draws_for_same_or_double_sex_damage_only() {
    let mut package = package();
    package.character_index = [
        (9_214_001, "male"),
        (9_214_002, "female"),
        (9_214_003, "double"),
    ]
    .into_iter()
    .map(|(id, sex)| PackageCharacterEntry {
        voice_assets: Default::default(),
        id,
        name_key: format!("sex_{id}"),
        max_hp: 10,
        initial_hp: 10,
        skills: vec![],
        sex: sex.into(),
        group: "key".into(),
        is_lord: false,
        clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: vec![],
    })
    .collect();
    package.manifest.logic_hash = package.compute_logic_hash();

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(MIO_TUIFU as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: MIO_TUIFU as u64,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let target = other_player(&tx, owner);
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    add_skill_def(&mut tx, owner, MIO_TUIFU);
    tx.state_mut().players.get_mut(target).unwrap().hp = 10;
    tx.state_mut().players.get_mut(target).unwrap().max_hp = 10;
    tx.state_mut().players.get_mut(owner).unwrap().character_def = 9_214_001;
    tx.state_mut()
        .players
        .get_mut(target)
        .unwrap()
        .character_def = 9_214_001;
    let hand = tx.state().hand_count(owner);

    for (target_character, expected_draws) in [(9_214_001, 1), (9_214_002, 1), (9_214_003, 2)] {
        tx.state_mut()
            .players
            .get_mut(target)
            .unwrap()
            .character_def = target_character;
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
        assert_eq!(tx.state().hand_count(owner), hand + expected_draws);
    }
}

#[test]
fn jugu_uses_max_hp_for_initial_cards_and_hand_limit() {
    for (package_bytes, skill_def) in [(PACKAGE, AKANE_JUGU), (SP_PACKAGE, JUGU)] {
        let mut skill_package =
            content_schema::ContentPackage::from_json_bytes(package_bytes).unwrap();
        skill_package.character_index.clear();
        skill_package.manifest.logic_hash = skill_package.compute_logic_hash();
        let mut roster = content_schema::ContentPackage::from_json_bytes(STANDARD_PACKAGE).unwrap();
        for character in &mut roster.character_index {
            character.max_hp = 4;
            character.initial_hp = 0;
            character.skills = vec![skill_def];
        }
        roster.manifest.logic_hash = roster.compute_logic_hash();

        let mut engine = Engine::new();
        engine.load_package(&skill_package).unwrap();
        engine.load_package(&roster).unwrap();
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: skill_def as u64,
                ruleset: "identity-standard".into(),
            },
        });
        let players = engine.state().players.alive_by_seat();
        for (index, player) in players.into_iter().enumerate() {
            let max_hp = engine.state().players.get(player).unwrap().max_hp;
            let initial_plus_first_draw = if index == 0 {
                4 + max_hp + 2
            } else {
                4 + max_hp
            };
            assert_eq!(engine.state().hand_count(player), initial_plus_first_draw);
            assert_eq!(
                query(
                    engine.state(),
                    &QueryRequest::HandLimit { player },
                    &mut QueryContext::new(),
                ),
                QueryResponse::Int((max_hp * 2) as i32)
            );
        }
    }
}

#[test]
fn feiqu_locks_sha_responses_for_its_source_or_target_owner() {
    for skill_def in [DORUJI_FEIQU, GODAN_FEIQU] {
        let engine = started_engine();
        let source = engine.state().turn.current;
        let mut source_owned = Transaction::begin(engine.state().clone());
        let target = other_player(&source_owned, source);
        let sha = source_owned.state().hand_cards(source)[0];
        source_owned.state_mut().cards.get_mut(sha).unwrap().kind = CardKind::Sha;
        add_skill_def(&mut source_owned, source, skill_def);
        assert!(engine_core::query::sha_unrespondable_for_card(
            source_owned.state(),
            source,
            sha
        ));
        assert!(engine_core::query::sha_unrespondable_for_card_target(
            source_owned.state(),
            source,
            target,
            sha
        ));

        let engine = started_engine();
        let source = engine.state().turn.current;
        let mut target_owned = Transaction::begin(engine.state().clone());
        let target = other_player(&target_owned, source);
        let sha = target_owned.state().hand_cards(source)[0];
        target_owned.state_mut().cards.get_mut(sha).unwrap().kind = CardKind::Sha;
        add_skill_def(&mut target_owned, target, skill_def);
        assert!(!engine_core::query::sha_unrespondable_for_card(
            target_owned.state(),
            source,
            sha
        ));
        assert!(engine_core::query::sha_unrespondable_for_card_target(
            target_owned.state(),
            source,
            target,
            sha
        ));

        target_owned.state_mut().cards.get_mut(sha).unwrap().kind = CardKind::Tao;
        assert!(!engine_core::query::sha_unrespondable_for_card_target(
            target_owned.state(),
            source,
            target,
            sha
        ));
    }
}

#[test]
fn dunying_uses_current_lost_hp_for_distance_and_both_draw_windows() {
    let engine = started_engine();
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let other = other_player(&tx, owner);
    let base_distance = match query(
        tx.state(),
        &QueryRequest::Distance {
            from: other,
            to: owner,
        },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        response => panic!("unexpected distance response: {response:?}"),
    };
    let max_hp = tx.state().players.get(owner).unwrap().max_hp as i32;
    tx.state_mut().players.get_mut(owner).unwrap().hp = max_hp - 2;
    add_skill_def(&mut tx, owner, ABYUSA_DUNYING);
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::Distance {
                from: other,
                to: owner,
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(value) if value == base_distance + 2
    ));

    let before_prepare = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before_prepare + 2);

    let before_end = tx.state().hand_count(owner);
    enter_end_phase(&mut tx, owner);
    assert_eq!(tx.state().hand_count(owner), before_end + 2);
}

#[test]
fn gongqing_uses_the_damage_sources_final_attack_range() {
    let sp = content_schema::ContentPackage::from_json_bytes(SP_PACKAGE).unwrap();
    for skill_def in [SASAMI_GONGQING, XINFU_GONGQING] {
        for (range, expected_loss) in [(1, 1), (3, 2), (4, 3)] {
            let mut engine = started_engine();
            engine.load_package(&sp).unwrap();
            let source = engine.state().turn.current;
            let mut tx = Transaction::begin(engine.state().clone());
            let target = other_player(&tx, source);
            if range > 1 {
                let equip = tx.state().zones.find(ZoneKind::Equip, source).unwrap();
                let weapon = tx.state_mut().cards.alloc(Card {
                    id: engine_api::CardId::NONE,
                    def: CardDefId::new(88_000 + range),
                    kind: CardKind::Weapon,
                    suit: Suit::Spade,
                    rank: 5,
                    weapon_range: range as u8,
                    zone: equip,
                    owner: source,
                    face_up: true,
                });
                tx.state_mut()
                    .zones
                    .get_mut(equip)
                    .unwrap()
                    .cards
                    .push(weapon);
            }
            add_skill_def(&mut tx, target, skill_def);
            tx.state_mut().players.get_mut(target).unwrap().max_hp = 8;
            tx.state_mut().players.get_mut(target).unwrap().hp = 8;
            let hp = tx.state().players.get(target).unwrap().hp;
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
            assert_eq!(
                tx.state().players.get(target).unwrap().hp,
                hp - expected_loss
            );
        }
    }
}

#[test]
fn lieyin_uses_effective_card_names_without_mutating_physical_cards() {
    let engine = started_engine();
    let owner = engine.state().turn.current;
    let mut red_as_sha = Transaction::begin(engine.state().clone());
    red_as_sha.state_mut().turn.phase = Phase::Play;
    let target = other_player(&red_as_sha, owner);
    remove_natural_responses(&mut red_as_sha, target);
    let card = red_as_sha.state().hand_cards(owner)[0];
    {
        let physical = red_as_sha.state_mut().cards.get_mut(card).unwrap();
        physical.kind = CardKind::Tao;
        physical.suit = Suit::Heart;
    }
    red_as_sha.state_mut().players.get_mut(owner).unwrap().hp =
        red_as_sha.state().players.get(owner).unwrap().max_hp as i32;
    add_skill_def(&mut red_as_sha, owner, YUI_LIEYIN0);
    let physical = red_as_sha.state().cards.get(card).unwrap();
    assert_eq!(physical.kind, CardKind::Tao);
    assert_eq!(
        engine_core::query::effective_card_kind(red_as_sha.state(), owner, physical),
        CardKind::Sha
    );
    assert!(matches!(
        query(
            red_as_sha.state(),
            &QueryRequest::CardUsable {
                player: owner,
                card,
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Bool(true)
    ));
    let hp = red_as_sha.state().players.get(target).unwrap().hp;
    let (after_sha, result) = apply_command(
        red_as_sha.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_820),
            kind: CommandKind::PlayCard {
                card,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(after_sha.players.get(target).unwrap().hp, hp - 1);
    assert_eq!(after_sha.cards.get(card).unwrap().kind, CardKind::Tao);

    let engine = started_engine();
    let owner = engine.state().turn.current;
    let mut sha_as_duel = Transaction::begin(engine.state().clone());
    sha_as_duel.state_mut().turn.phase = Phase::Play;
    let target = other_player(&sha_as_duel, owner);
    remove_natural_responses(&mut sha_as_duel, target);
    remove_natural_responses(&mut sha_as_duel, owner);
    let card = sha_as_duel.state().hand_cards(owner)[0];
    sha_as_duel.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Sha;
    add_skill_def(&mut sha_as_duel, owner, YUI_LIEYIN1);
    let physical = sha_as_duel.state().cards.get(card).unwrap();
    assert_eq!(
        engine_core::query::effective_card_kind(sha_as_duel.state(), owner, physical),
        CardKind::TrickJuedou
    );
    let hp = sha_as_duel.state().players.get(target).unwrap().hp;
    let (after_duel, result) = apply_command(
        sha_as_duel.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_821),
            kind: CommandKind::PlayCard {
                card,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(after_duel.players.get(target).unwrap().hp, hp - 1);
    assert_eq!(after_duel.cards.get(card).unwrap().kind, CardKind::Sha);
    assert_eq!(
        after_duel
            .turn
            .flag(engine_core::state::turn_flag::SHA_PLAYED),
        0
    );
}

#[test]
fn yoshino_fail_blocks_only_the_current_turn_owner() {
    let engine = started_engine();
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    add_skill_def(&mut tx, target, YOSHINO_FAIL);

    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(9_921),
        kind: CardKind::Sha,
        suit: Suit::Heart,
        rank: 8,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);

    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::TargetLegal {
                user: owner,
                card: sha,
                target,
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Bool(false)
    ));
    let before = tx.state().clone();
    let (after, pipe) = apply_command(
        before.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(before.revision),
            idempotency_key: IdempotencyKey(10),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(after, before);

    tx.state_mut().turn.current = target;
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::TargetLegal {
                user: owner,
                card: sha,
                target,
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Bool(true)
    ));
}

#[test]
fn yuiko_fenglun2_removes_real_sha_use_and_card_distance_limits() {
    let engine = started_engine();
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 1);
    tx.state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 1);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(9_920),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);
    let command = Command {
        actor: owner,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(9),
        kind: CommandKind::PlayCard {
            card: sha,
            targets: vec![target],
            card_targets: vec![],
        },
    };

    let (_, rejected) = apply_command(tx.state().clone(), command.clone());
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    add_skill_def(&mut tx, owner, YUIKO_FENGLUN2);
    assert!(engine_core::query::card_use_unlimited_for(
        tx.state(),
        owner
    ));
    assert!(engine_core::query::card_distance_unlimited_for(
        tx.state(),
        owner
    ));
    assert_eq!(
        engine_core::query::sha_use_limit_for(tx.state(), owner),
        u32::MAX
    );
    let (_, accepted) = apply_command(tx.state().clone(), command);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
}

#[test]
fn kotori_wu_draws_only_when_hand_and_hp_differ() {
    let engine = started_engine();
    let owner = engine.state().turn.current;

    let mut equal = Transaction::begin(engine.state().clone());
    add_skill_def(&mut equal, owner, KOTORI_SKILL_WU);
    let hp = equal.state().players.get(owner).unwrap().hp as u32;
    let hand = equal.state().hand_count(owner);
    assert!(hand >= hp);
    run_effects(
        &mut equal,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: hand - hp,
        }],
    )
    .unwrap();
    enter_end_phase(&mut equal, owner);
    assert_eq!(equal.state().hand_count(owner), hp);

    let mut different = Transaction::begin(engine.state().clone());
    add_skill_def(&mut different, owner, KOTORI_SKILL_WU);
    run_effects(
        &mut different,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    enter_end_phase(&mut different, owner);
    assert_eq!(different.state().hand_count(owner), 1);
}

#[test]
fn kotori_shu_adds_one_sha_and_draws_after_the_second_use() {
    let engine = started_engine();
    let owner = engine.state().turn.current;

    let mut one_sha = Transaction::begin(engine.state().clone());
    add_skill_def(&mut one_sha, owner, KOTORI_SKILL_SHU);
    assert_eq!(
        engine_core::query::sha_use_limit_for(one_sha.state(), owner),
        2
    );
    run_effects(
        &mut one_sha,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: engine_core::state::Phase::Play.to_u8(),
        }],
    )
    .unwrap();
    one_sha
        .state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 1);
    let before = one_sha.state().hand_count(owner);
    run_effects(
        &mut one_sha,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: engine_core::state::Phase::Discard.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(one_sha.state().hand_count(owner), before);

    let mut two_sha = Transaction::begin(engine.state().clone());
    add_skill_def(&mut two_sha, owner, KOTORI_SKILL_SHU);
    run_effects(
        &mut two_sha,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: engine_core::state::Phase::Play.to_u8(),
        }],
    )
    .unwrap();
    two_sha
        .state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 2);
    let before = two_sha.state().hand_count(owner);
    run_effects(
        &mut two_sha,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: engine_core::state::Phase::Discard.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(two_sha.state().hand_count(owner), before + 1);
}

#[test]
fn ending_play_emits_phase_ended_before_discard_begins() {
    let mut engine = started_engine();
    let owner = engine.state().turn.current;
    for _ in 0..2 {
        let result = engine.submit(Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::EndPhase,
        });
        assert!(matches!(result, engine_api::CommandResult::Accepted { .. }));
    }
    assert!(matches!(
        engine.last_events().first(),
        Some(engine_api::DomainEvent::PhaseEnded {
            current,
            phase: engine_api::TurnPhaseTag::Play,
            ..
        }) if *current == owner
    ));
    assert!(matches!(
        engine.last_events().get(1),
        Some(engine_api::DomainEvent::TurnStateChanged {
            current,
            phase: engine_api::TurnPhaseTag::Discard,
        }) if *current == owner
    ));
}
