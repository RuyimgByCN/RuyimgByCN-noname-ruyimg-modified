//! O2 全量合并批次 1（2026-10-14）：家族 tw 的 56 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_tw_batch_al_dsl {
//! Golden for batch-AL rewrites:
//! 浮沉 (twfucheng, forced): at prepare phase, an all-red hand grants 璧月
//!   (sb/sbbiyue 4152380059) and an all-black hand grants 武神
//!   (sb/sbwushuang 4242421006); mixed or empty hands grant nothing.
//! Also verifies the 13 tw *_sxrm_caocao audio-only noops are finalized
//! (honest noops, no stale TODO headers — upstream `{ audio: 1 }`).

use engine_api::{
    CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const TWFUCHENG: u32 = 3_898_925_066;
const SBBIYUE: u32 = 4_152_380_059;
const SBWUSHUANG: u32 = 4_242_421_006;
const TW_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");
const SB_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sb/sb.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
    let tw = content_schema::ContentPackage::from_json_bytes(TW_PACKAGE).unwrap();
    let sb = content_schema::ContentPackage::from_json_bytes(SB_PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&tw).unwrap();
    engine.load_package(&sb).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn inject(state: &mut GameState, player: PlayerId, suit: engine_api::Suit, tag: u32) {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(91_900 + tag),
        kind: CardKind::Tao,
        suit,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
}

fn with_clean_owner(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn has_skill(state: &GameState, owner: PlayerId, def: u32) -> bool {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

fn run_prepare(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

#[test]
fn package_publishes_batch_al_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(TW_PACKAGE).unwrap();
    let entry = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == TWFUCHENG)
        .expect("twfucheng trigger");
    assert_eq!(entry.event.as_str(), "phase_prepare");
    assert_eq!(entry.filter.as_str(), "owner_is_target");
    assert!(!entry.optional);
}

#[test]
fn twfucheng_all_red_hand_grants_sbbiyue() {
    let (state, owner) = started(2_026_081_540);
    let mut state = with_clean_owner(state, owner);
    // Empty the hand, then refill with red suits only.
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let mut state = tx.commit().state;
    inject(&mut state, owner, engine_api::Suit::Heart, 1);
    inject(&mut state, owner, engine_api::Suit::Diamond, 2);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWFUCHENG),
        }],
    )
    .unwrap();
    let state = tx.commit().state;

    let state = run_prepare(state, owner);
    assert!(has_skill(&state, owner, SBBIYUE));
    assert!(!has_skill(&state, owner, SBWUSHUANG));
}

#[test]
fn twfucheng_all_black_hand_grants_sbwushuang() {
    let (state, owner) = started(2_026_081_541);
    let mut state = with_clean_owner(state, owner);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let mut state = tx.commit().state;
    inject(&mut state, owner, engine_api::Suit::Spade, 3);
    inject(&mut state, owner, engine_api::Suit::Club, 4);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWFUCHENG),
        }],
    )
    .unwrap();
    let state = tx.commit().state;

    let state = run_prepare(state, owner);
    assert!(has_skill(&state, owner, SBWUSHUANG));
    assert!(!has_skill(&state, owner, SBBIYUE));
}

#[test]
fn twfucheng_mixed_hand_grants_nothing() {
    let (state, owner) = started(2_026_081_542);
    let state = with_clean_owner(state, owner);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let mut state = tx.commit().state;
    inject(&mut state, owner, engine_api::Suit::Heart, 5);
    inject(&mut state, owner, engine_api::Suit::Spade, 6);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWFUCHENG),
        }],
    )
    .unwrap();
    let state = tx.commit().state;

    let state = run_prepare(state, owner);
    assert!(!has_skill(&state, owner, SBBIYUE));
    assert!(!has_skill(&state, owner, SBWUSHUANG));
}

#[test]
fn sxrm_caocao_audio_only_noops_are_finalized() {
    // The 13 *_sxrm_caocao skills are upstream `{ audio: 1 }` aliases with no rule
    // surface; their .skill files must be honest noops without stale TODO headers.
    let package = content_schema::ContentPackage::from_json_bytes(TW_PACKAGE).unwrap();
    let names = [
        "zhichi_sxrm_caocao",
        "reganglie_sxrm_caocao",
        "refankui_sxrm_caocao",
        "reyiji_sxrm_caocao",
        "oljieming_sxrm_caocao",
        "fangzhu_sxrm_caocao",
        "shibei_sxrm_caocao",
        "rechengxiang_sxrm_caocao",
        "zhiyu_sxrm_caocao",
        "jilei_sxrm_caocao",
        "benyu_sxrm_caocao",
        "chouce_sxrm_caocao",
        "wuhun_sxrm_caocao",
    ];
    for name in names {
        let path = format!("{}/../../content/original/ported/tw/{name}.skill", env!("CARGO_MANIFEST_DIR"));
        let source = std::fs::read_to_string(&path).unwrap();
        assert!(!source.contains("// TODO"), "{name} still has a TODO header");
        assert!(source.contains("INTENTIONAL_EMPTY_NOOP"), "{name} lost its noop marker");
        assert!(
            source.contains("audio-only"),
            "{name} lost its upstream audio-only citation"
        );
    }
    // Their defs still resolve in the committed package.
    for def in [
        4_274_763_640u32, 3_210_720_173, 4_249_767_960, 3_253_555_636, 4_093_162_705,
        2_307_818_525, 3_631_705_171, 2_285_536_647, 4_195_097_884, 3_398_468_226,
        3_695_755_888, 3_218_932_758, 2_867_527_212,
    ] {
        assert!(
            package.skill_index.iter().any(|skill| skill.skill_def_id == def),
            "def {def} missing from tw package"
        );
    }
}
}

mod original_tw_bianfuren {
//! Golden coverage for TW 卞夫人「挽危/约俭」.

use engine_api::{CardDefId, CardId, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWWANWEI: u32 = 3_733_856_593;
const TWWANWEI_EFFECT: u32 = 3_469_371_143;
const TWYUEJIAN: u32 = 3_308_823_251;
const TWYUEJIAN_EFFECT: u32 = 3_469_371_145;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twbianfuren.skill");

fn inject(
    tx: &mut Transaction,
    zone: ZoneKind,
    owner: PlayerId,
    kind: CardKind,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_376_000 + u32::from(rank)),
        kind,
        suit: Suit::Heart,
        rank,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn wanwei_prevents_and_uses_bottom_while_yuejian_grants_tiered_rewards() {
    let package = content_compiler::compile_source_named("tw-bianfuren-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 376);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 2_026_080_117, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        [TWWANWEI, TWYUEJIAN].map(|skill_def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    let skill = |tx: &Transaction, def| {
        tx.state()
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == owner && skill.def.raw() == def)
            .unwrap()
            .id
    };

    let wanwei = skill(&tx, TWWANWEI);
    tx.state_mut().resolution.current_damage_id = 77;
    let hp = tx.state().players.get(owner).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwbianfuren {
            player: owner,
            target,
            helper_skill_def: TWWANWEI_EFFECT,
            action: 0,
            stage: 0,
            cards: Vec::new(),
            skill: wanwei,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
    assert!(tx.state().resolution.prevented_damage_ids.contains(&77));

    let yuejian = skill(&tx, TWYUEJIAN);
    let cards = [
        inject(&mut tx, ZoneKind::Hand, owner, CardKind::Shan, 3),
        inject(&mut tx, ZoneKind::Hand, owner, CardKind::Shan, 4),
        inject(&mut tx, ZoneKind::Hand, owner, CardKind::Shan, 5),
    ];
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwbianfuren {
            player: owner,
            target: owner,
            helper_skill_def: TWYUEJIAN_EFFECT,
            action: 2,
            stage: 1,
            cards: cards.to_vec(),
            skill: yuejian,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, max_hp + 1);
    assert!(tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .contains(&(TWYUEJIAN_EFFECT, 1)));

    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWWANWEI_EFFECT),
        }],
    )
    .unwrap();
    let helper = skill(&tx, TWWANWEI_EFFECT);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    tx.state_mut().zones.get_mut(draw).unwrap().cards.clear();
    let bottom = inject(
        &mut tx,
        ZoneKind::DrawPile,
        PlayerId::NONE,
        CardKind::Tao,
        1,
    );
    let top = inject(
        &mut tx,
        ZoneKind::DrawPile,
        PlayerId::NONE,
        CardKind::Sha,
        2,
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwbianfuren {
            player: owner,
            target: PlayerId::NONE,
            helper_skill_def: 0,
            action: 1,
            stage: 0,
            cards: Vec::new(),
            skill: helper,
        }],
    )
    .unwrap();
    assert!(tx.state().hand_cards(owner).contains(&top));
    assert_eq!(
        tx.state().cards.get(bottom).unwrap().zone,
        tx.state()
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}
}

mod original_tw_caoang {
//! Golden coverage for TW 曹昂「孝廉」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const TWXIAOLIAN: u32 = 2_735_068_140;
const HORSE: u32 = 2_420_900_957;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twcaoang.skill");

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    state
}

fn distance(state: &GameState, from: PlayerId, to: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::Distance { from, to },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected distance response {other:?}"),
    }
}

#[test]
fn redirects_single_sha_and_stores_horse_on_original_target() {
    let package = content_compiler::compile_source_named("tw-caoang-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 390);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_209, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, source, original) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWXIAOLIAN),
        },
        EffectRequest::SetTurnState {
            current: source,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let horse = inject(tx.state_mut(), owner, CardKind::Other, 99_010);
    let sha = inject(tx.state_mut(), source, CardKind::Sha, 99_011);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let before = distance(tx.state(), source, original);
    let state = tx.commit().state;
    let (state, result) = apply_command(
        state,
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![original],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let state = choose(state, owner, vec![1], 2);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
    let state = choose(state, owner, vec![horse.raw()], 3);
    assert!(!state.has_pending_choice());
    assert_eq!(state.skill_pile_cards(original, HORSE), vec![horse]);
    assert_eq!(distance(&state, source, original), before + 1);
}
}

mod original_tw_caoxiu {
//! Golden coverage for TW 曹休「前驱乐/倾袭」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWQIANJU: u32 = 2_546_046_607;
const TWQINGXI: u32 = 2_467_573_737;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twcaoxiu.skill");

fn inject(
    tx: &mut Transaction,
    zone_kind: ZoneKind,
    owner: PlayerId,
    kind: CardKind,
    def: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 2,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill(tx: &Transaction, owner: PlayerId, def: u32) -> engine_api::SkillId {
    tx.state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn qianju_equips_an_empty_slot_and_qingxi_discards_equipment_for_damage() {
    let package = content_compiler::compile_source_named("tw-caoxiu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 379);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 2_026_080_120, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        players
            .iter()
            .copied()
            .map(|player| EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }),
    );
    effects.extend(
        [TWQIANJU, TWQINGXI].map(|skill_def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();

    let gained = inject(
        &mut tx,
        ZoneKind::DrawPile,
        PlayerId::NONE,
        CardKind::Armor,
        93_790,
    );
    let qianju = skill(&tx, owner, TWQIANJU);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwqianju {
            player: owner,
            target,
            skill: qianju,
        }],
    )
    .unwrap();
    assert!(tx
        .state()
        .zones
        .find(ZoneKind::Equip, owner)
        .and_then(|zone| tx.state().zones.get(zone))
        .unwrap()
        .cards
        .contains(&gained));

    let source_weapon = inject(&mut tx, ZoneKind::Equip, owner, CardKind::Weapon, 93_791);
    inject(&mut tx, ZoneKind::Equip, target, CardKind::HorseDef, 93_792);
    let sha = inject(&mut tx, ZoneKind::Hand, owner, CardKind::Sha, 93_793);
    tx.state_mut().resolution.resume_kind = ContinuationKind::ResumePlayedSha;
    tx.state_mut().resolution.resume_payload = vec![owner.raw(), target.raw(), 1, 0, 1, sha.raw()];
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let qingxi = skill(&tx, owner, TWQINGXI);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwqingxi {
                player: owner,
                target,
                card: sha,
                stage: 0,
                count: 0,
                cards: Vec::new(),
                skill: qingxi,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, target, vec![1], 1);
    let state = choose(state, target, vec![source_weapon.raw()], 2);
    let target_equip = state.zones.find(ZoneKind::Equip, target).unwrap();
    let owner_equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    assert!(state.zones.get(target_equip).unwrap().cards.is_empty());
    assert!(!state
        .zones
        .get(owner_equip)
        .unwrap()
        .cards
        .contains(&source_weapon));
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 2);
}
}

mod original_tw_caozhao {
//! Golden coverage for TW 曹肇「复纂/宠齐/非服」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWFUZUAN: u32 = 3_482_762_704;
const TWCHONGQI: u32 = 4_284_388_882;
const TWFEIFU: u32 = 3_948_553_827;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twcaozhao.skill");

fn choose(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    state
}

fn setup(seed: u64, skills: &[(usize, u32)]) -> (Transaction, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("tw-caozhao-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 387);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == TWFEIFU && skill.conversion));
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(skills.iter().map(|(index, skill)| EffectRequest::AddSkill {
        player: players[*index],
        skill_def: SkillDefId::new(*skill),
    }));
    run_effects(&mut tx, effects).unwrap();
    (tx, players)
}

fn skill(tx: &Transaction, owner: PlayerId, def: u32) -> engine_api::SkillId {
    tx.state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

#[test]
fn fuzuan_toggles_chongqi_grants_and_feifu_gives_equipment() {
    let (mut tx, players) = setup(2_026_080_204, &[(0, TWFUZUAN), (1, TWFEIFU)]);
    let (owner, target) = (players[0], players[1]);
    let fuzuan = skill(&tx, owner, TWFUZUAN);
    let feifu = skill(&tx, target, TWFEIFU);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwcaozhao {
                player: owner,
                target: PlayerId::NONE,
                source: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                selected_skill_def: 0,
                twfuzuan_skill_def: TWFUZUAN,
                twfeifu_skill_def: TWFEIFU,
                action: 0,
                stage: 0,
                skill: fuzuan,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![target.raw()], false, 1);
    assert_eq!(state.skills.get(feifu).unwrap().view_as_branch, 1);

    let (mut tx, players) = setup(2_026_080_205, &[(0, TWCHONGQI)]);
    let (owner, target) = (players[0], players[1]);
    let chongqi = skill(&tx, owner, TWCHONGQI);
    let old_max_hp = tx.state().players.get(owner).unwrap().max_hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwcaozhao {
                player: owner,
                target: PlayerId::NONE,
                source: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                selected_skill_def: 0,
                twfuzuan_skill_def: TWFUZUAN,
                twfeifu_skill_def: TWFEIFU,
                action: 1,
                stage: 0,
                skill: chongqi,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![target.raw()], false, 2);
    assert_eq!(state.players.get(owner).unwrap().max_hp, old_max_hp - 1);
    for player in &players {
        assert!(state
            .players
            .get(*player)
            .unwrap()
            .skills
            .iter()
            .any(|skill| {
                state
                    .skills
                    .get(*skill)
                    .is_some_and(|skill| skill.def.raw() == TWFEIFU)
            }));
    }
    assert!(state
        .players
        .get(target)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == TWFUZUAN)
        }));

    let (mut tx, players) = setup(2_026_080_206, &[(0, TWFEIFU)]);
    let (owner, source) = (players[0], players[1]);
    let feifu = skill(&tx, owner, TWFEIFU);
    let card = tx.state().hand_cards(owner)[0];
    let card_state = tx.state_mut().cards.get_mut(card).unwrap();
    card_state.def = engine_api::CardDefId::NONE;
    card_state.kind = CardKind::Weapon;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwcaozhao {
                player: owner,
                target: owner,
                source,
                card: engine_api::CardId::NONE,
                selected_skill_def: 0,
                twfuzuan_skill_def: TWFUZUAN,
                twfeifu_skill_def: TWFEIFU,
                action: 2,
                stage: 0,
                skill: feifu,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![card.raw()], false, 3);
    assert_eq!(state.skills.get(feifu).unwrap().view_as_branch, 1);
    assert!(state.has_pending_choice());
    let state = choose(state, source, vec![1], false, 4);
    let equip = state.zones.find(ZoneKind::Equip, source).unwrap();
    assert_eq!(state.cards.get(card).unwrap().zone, equip);
}
}

mod original_tw_chengpu {
//! Golden coverage for TW 程普「疠火/醇醪」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWLIHUO: u32 = 2_910_350_044;
const TWCHUNLAO: u32 = 2_257_064_269;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twchengpu.skill");

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_001),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );

    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    state
}

fn setup(skill_def: u32, phase: Phase) -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("tw-chengpu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 389);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_208, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(skill_def),
    });
    effects.push(EffectRequest::SetTurnState {
        current: players[0],
        phase: phase.to_u8(),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, players)
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

#[test]
fn converted_sha_costs_hp_only_when_it_causes_dying() {
    for (target_hp, expected_owner_hp) in [(4, 4), (1, 3)] {
        let (mut state, players) = setup(TWLIHUO, Phase::Play);
        let (owner, target) = (players[0], players[1]);
        for player in &players {
            state.players.get_mut(*player).unwrap().hp = 4;
        }
        state.players.get_mut(target).unwrap().hp = target_hp;
        for player in &players {
            let hand = state.hand_cards(*player);
            let mut tx = Transaction::begin(state);
            if !hand.is_empty() {
                run_effects(
                    &mut tx,
                    vec![EffectRequest::DiscardCards {
                        player: *player,
                        cards: hand,
                    }],
                )
                .unwrap();
            }
            state = tx.commit().state;
        }
        let sha = inject(&mut state, owner, CardKind::Sha);
        let (state, result) = apply_command(
            state,
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(10 + target_hp as u64),
                kind: CommandKind::PlayCard {
                    card: sha,
                    targets: vec![target],
                    card_targets: Vec::new(),
                },
            },
        );
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        // 疠火在当前实现中无条件将普通杀转为火属性；加目标为唯一 choice，取消之。
        let state = choose(state, owner, Vec::new(), true, 20 + target_hp as u64);
        assert_eq!(state.players.get(owner).unwrap().hp, expected_owner_hp);
    }
}

#[test]
fn chunlao_stores_one_field_card_and_spends_it_to_rescue() {
    let (state, players) = setup(TWCHUNLAO, Phase::Idle);
    let (owner, target) = (players[0], players[1]);
    let skill = skill(&state, owner, TWCHUNLAO);
    let selected = state.hand_cards(target)[0];
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwchengpu {
                player: owner,
                target: owner,
                source: PlayerId::NONE,
                card: CardId::NONE,
                chunlao_skill_def: TWCHUNLAO,
                action: 1,
                stage: 0,
                selected_card: CardId::NONE,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![target.raw()], false, 41);
    let mut state = choose(state, owner, vec![selected.raw()], false, 42);
    assert_eq!(state.skill_pile_cards(target, TWCHUNLAO), vec![selected]);
    state.players.get_mut(target).unwrap().hp = 0;
    state.players.get_mut(target).unwrap().dying = true;
    let hand_before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwchengpu {
            player: owner,
            target,
            source: PlayerId::NONE,
            card: CardId::NONE,
            chunlao_skill_def: TWCHUNLAO,
            action: 3,
            stage: 0,
            selected_card: CardId::NONE,
            skill,
        }],
    )
    .unwrap();
    assert!(tx.state().skill_pile_cards(target, TWCHUNLAO).is_empty());
    assert_eq!(tx.state().hand_count(owner), hand_before + 1);
    assert_eq!(tx.state().players.get(target).unwrap().hp, 1);
}
}

mod original_tw_chenzhen {
//! Golden coverage for TW 陈震「睦约/察异」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWMUYUE: u32 = 2_544_676_263;
const TWMUYUE_EFFECT: u32 = 3_469_371_151;
const TWCHAYI: u32 = 3_796_108_342;
const TWCHAYI_EFFECT: u32 = 3_469_371_153;
const TWCHAYI_RE: u32 = 3_469_371_154;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twchenzhen.skill");

fn inject(
    tx: &mut Transaction,
    zone: ZoneKind,
    owner: PlayerId,
    kind: CardKind,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_378_000 + u32::from(rank)),
        kind,
        suit: Suit::Diamond,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn skill(tx: &Transaction, owner: PlayerId, def: u32) -> engine_api::SkillId {
    tx.state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

#[test]
fn muyue_matching_discard_grants_free_use_and_chayi_switches_branch() {
    let package = content_compiler::compile_source_named("tw-chenzhen-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 378);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 2_026_080_119, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([TWMUYUE, TWCHAYI].map(|skill_def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    }));
    run_effects(&mut tx, effects).unwrap();

    let cost = inject(&mut tx, ZoneKind::Hand, owner, CardKind::Shan, 1);
    let reward = inject(
        &mut tx,
        ZoneKind::DrawPile,
        PlayerId::NONE,
        CardKind::Shan,
        2,
    );
    let muyue = skill(&tx, owner, TWMUYUE);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwchenzhen {
                player: owner,
                target: PlayerId::NONE,
                helper_skill_def: TWMUYUE_EFFECT,
                record_skill_def: 0,
                action: 0,
                stage: 2,
                card_kind: CardKind::Shan.tag(),
                card: cost,
                branch: 0,
                skill: muyue,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![target.raw()], 1);
    assert!(state.hand_cards(target).contains(&reward));
    let mut tx = Transaction::begin(state);
    assert!(tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|id| {
            tx.state()
                .skills
                .get(*id)
                .is_some_and(|skill| skill.def.raw() == TWMUYUE_EFFECT)
        }));

    let chayi = skill(&tx, owner, TWCHAYI);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwchenzhen {
                player: owner,
                target: owner,
                helper_skill_def: TWCHAYI_EFFECT,
                record_skill_def: TWCHAYI_RE,
                action: 1,
                stage: 0,
                card_kind: 0,
                card: CardId::NONE,
                branch: 0,
                skill: chayi,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![target.raw()], 2);
    let state = choose(state, target, vec![0], 3);
    let mut tx = Transaction::begin(state);
    inject(&mut tx, ZoneKind::Hand, target, CardKind::Sha, 3);
    let record = skill(&tx, target, TWCHAYI_RE);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwchenzhen {
            player: target,
            target,
            helper_skill_def: TWCHAYI_EFFECT,
            record_skill_def: 0,
            action: 3,
            stage: 0,
            card_kind: 0,
            card: CardId::NONE,
            branch: 0,
            skill: record,
        }],
    )
    .unwrap();
    assert!(tx
        .state()
        .players
        .get(target)
        .unwrap()
        .marks
        .contains(&(TWCHAYI_EFFECT, 1)));
    assert!(tx.state().skills.get(record).is_none());
}
}

mod original_tw_cuijin {
//! Golden coverage for TW 乐就「催进」.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const CUIJIN: u32 = 2_582_559_793;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/cuijin.skill");

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_395_000 + u32::from(rank)),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(
    with_shan: bool,
) -> (
    GameState,
    PlayerId,
    PlayerId,
    CardId,
    CardId,
    Option<CardId>,
) {
    let package = content_compiler::compile_source_named("cuijin-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 395);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 2_026_080_215, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(CUIJIN),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let sha = inject(tx.state_mut(), owner, CardKind::Sha, 7);
    let cost = inject(tx.state_mut(), owner, CardKind::Other, 8);
    let shan = with_shan.then(|| inject(tx.state_mut(), target, CardKind::Shan, 9));
    (tx.commit().state, owner, target, sha, cost, shan)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    )
}

fn play_and_accept(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    sha: CardId,
    cost: CardId,
) -> (GameState, PipelineResult) {
    let (state, played) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    // 上游 cuijin 的 cost 是单个 chooseToDiscard（选牌即可取消），无独立确认段。
    assert_eq!(played.choice.unwrap().kind, ChoiceKind::SelectCards);
    resolve(state, owner, vec![cost.raw()], 2)
}

#[test]
fn accepted_cuijin_adds_one_damage_and_clears_exact_sha_tracking() {
    let (state, owner, target, sha, cost, _) = setup(false);
    let target_hp = state.players.get(target).unwrap().hp;
    let owner_hp = state.players.get(owner).unwrap().hp;
    let (state, result) = play_and_accept(state, owner, target, sha, cost);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 2);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp);
    let skill = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == CUIJIN)
        .unwrap();
    assert!(skill.state_blob.is_empty());
}

#[test]
fn dodged_cuijin_draws_then_damages_the_sha_user() {
    let (state, owner, target, sha, cost, shan) = setup(true);
    let owner_hp = state.players.get(owner).unwrap().hp;
    let owner_hand = state.hand_count(owner);
    let (state, response) = play_and_accept(state, owner, target, sha, cost);
    let shan = shan.unwrap();
    assert_eq!(response.choice.unwrap().kind, ChoiceKind::SelectCards);
    let (state, result) = resolve(state, target, vec![shan.raw()], 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
    assert_eq!(state.hand_count(owner), owner_hand - 1);
}
}

mod original_tw_feiyi {
//! Golden coverage for TW 费祎「生息/宽济」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWSHENGXI: u32 = 3_391_342_890;
const TWKUANJI: u32 = 3_010_237_936;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twfeiyi.skill");

fn inject(
    tx: &mut Transaction,
    zone: ZoneKind,
    owner: PlayerId,
    kind: CardKind,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_377_000 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn shengxi_creates_salt_and_gains_zhinang_while_kuanji_transfers_discard() {
    let package = content_compiler::compile_source_named("tw-feiyi-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 377);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 2_026_080_118, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        [TWSHENGXI, TWKUANJI].map(|skill_def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    let skill = |tx: &Transaction, def| {
        tx.state()
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == owner && skill.def.raw() == def)
            .unwrap()
            .id
    };
    let shengxi = skill(&tx, TWSHENGXI);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwfeiyi {
            player: owner,
            action: 0,
            stage: 0,
            card: CardId::NONE,
            event_cards: Vec::new(),
            skill: shengxi,
        }],
    )
    .unwrap();
    assert!(tx.state().hand_cards(owner).iter().any(|card| {
        tx.state()
            .cards
            .get(*card)
            .is_some_and(|card| card.kind == CardKind::TrickTiaojiyanmei && card.rank == 6)
    }));

    let zhinang = inject(
        &mut tx,
        ZoneKind::DrawPile,
        PlayerId::NONE,
        CardKind::TrickWuzhong,
        8,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwfeiyi {
                player: owner,
                action: 1,
                stage: 0,
                card: CardId::NONE,
                event_cards: Vec::new(),
                skill: shengxi,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(
        tx.commit().state,
        owner,
        vec![CardKind::TrickWuzhong.tag().into()],
        1,
    );
    assert!(state.hand_cards(owner).contains(&zhinang));

    let mut tx = Transaction::begin(state);
    let kuanji = skill(&tx, TWKUANJI);
    let discard_card = inject(
        &mut tx,
        ZoneKind::DiscardPile,
        PlayerId::NONE,
        CardKind::Shan,
        9,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwfeiyi {
                player: owner,
                action: 2,
                stage: 0,
                card: CardId::NONE,
                event_cards: vec![discard_card],
                skill: kuanji,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![discard_card.raw()], 2);
    assert!(state.has_pending_choice());
    let state = choose(state, owner, vec![target.raw()], 3);
    assert!(state.hand_cards(target).contains(&discard_card));
}
}

mod original_tw_fidelity_color_batch {
//! Golden coverage for TW 渐营 / 武神 / 振良 rewritten from honest noops.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWJIANYING: u32 = 3_378_394_717;
const TWWUSHEN: u32 = 2_977_352_429;
const TWZHENLIANG: u32 = 3_711_733_177;
const JIANYING_SRC: &str =
    include_str!("../../../content/original/ported/tw/twjianying.skill");
const WUSHEN_SRC: &str = include_str!("../../../content/original/ported/tw/twwushen.skill");
const ZHENLIANG_SRC: &str =
    include_str!("../../../content/original/ported/tw/twzhenliang.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn bootstrap(source: &str, name: &str, skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(skill_def), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn inject_hand(tx: &mut Transaction, owner: PlayerId, suit: Suit, rank: u8, kind: CardKind) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(90_000 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}

#[test]
fn twjianying_source_and_npack_publish_last_used_suit_or_rank() {
    let compiled = content_compiler::compile_source_named("twjianying", JIANYING_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == TWJIANYING)
        .unwrap();
    assert_eq!(trigger.event, "card_used");
    assert_eq!(trigger.filter, "owner_source_matches_last_used_suit_or_rank");
    assert!(trigger.optional);

    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == TWJIANYING
            && trigger.filter == "owner_source_matches_last_used_suit_or_rank"));
}

#[test]
fn twjianying_offers_draw_when_rank_matches_previous_used_card() {
    let (mut tx, owner, _) = bootstrap(JIANYING_SRC, "twjianying-test", TWJIANYING);
    let first = inject_hand(&mut tx, owner, Suit::Spade, 7, CardKind::Other);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: owner,
            card: first,
            targets: vec![],
        }],
    )
    .unwrap();
    let initial = tx.state().hand_count(owner);
    let second = inject_hand(&mut tx, owner, Suit::Heart, 7, CardKind::Other);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: owner,
                card: second,
                targets: vec![],
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.hand_count(owner), initial + 1);
}

#[test]
fn twwushen_source_and_npack_rewrite_hearts_to_unlimited_sha() {
    let compiled = content_compiler::compile_source_named("twwushen", WUSHEN_SRC).unwrap();
    let mods: Vec<_> = compiled
        .modifier_index
        .iter()
        .filter(|item| item.skill_def_id == TWWUSHEN)
        .map(|item| item.target.as_str())
        .collect();
    assert!(mods.contains(&"card_name"));
    assert!(mods.contains(&"card_distance_unlimited"));
    assert!(mods.contains(&"card_use_unlimited"));
    assert!(mods.contains(&"sha_unrespondable"));

    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .modifier_index
        .iter()
        .any(|item| item.skill_def_id == TWWUSHEN && item.target == "card_name"));
}

#[test]
fn twwushen_heart_tao_is_effective_sha() {
    let (mut tx, owner, _) = bootstrap(WUSHEN_SRC, "twwushen-test", TWWUSHEN);
    let heart = inject_hand(&mut tx, owner, Suit::Heart, 3, CardKind::Tao);
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
}

#[test]
fn twzhenliang_source_and_npack_publish_active_damage_and_reduce() {
    let compiled = content_compiler::compile_source_named("twzhenliang", ZHENLIANG_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == TWZHENLIANG));
    assert!(compiled.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == TWZHENLIANG
            && trigger.event == "damage_before"
            && trigger.filter == "owner_is_target_other_source_alive"
            && trigger.optional
    }));

    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == TWZHENLIANG));
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == TWZHENLIANG && trigger.event == "damage_before"
    }));
}

#[test]
fn twzhenliang_can_cancel_incoming_damage_of_one() {
    let (tx, owner, other) = bootstrap(ZHENLIANG_SRC, "twzhenliang-test", TWZHENLIANG);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let mut tx = tx;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 3);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp);
}
}

mod original_tw_fuwan {
//! Golden coverage for TW 伏完「谋溃」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWMOUKUI: u32 = 3_462_273_565;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twfuwan.skill");

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, actor: PlayerId, selection: u32, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    state
}

#[test]
fn backwater_draws_discards_then_target_retaliates_after_nonlethal_sha() {
    let package = content_compiler::compile_source_named("tw-fuwan-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 393);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_212, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWMOUKUI),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let sha = inject(tx.state_mut(), owner, CardKind::Sha, 99_031);
    let owner_spare = inject(tx.state_mut(), owner, CardKind::Other, 99_032);
    let target_card = inject(tx.state_mut(), target, CardKind::Other, 99_033);
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let state = tx.commit().state;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));

    let state = choose(state, owner, 3, 2);
    assert_eq!(state.resolution.pending_observer, owner.raw());
    let state = choose(state, owner, target_card.raw(), 3);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert_eq!(state.resolution.pending_observer, target.raw());
    let state = choose(state, target, owner_spare.raw(), 4);
    assert!(!state.has_pending_choice());
    assert_eq!(
        state
            .zones
            .get(state.cards.get(owner_spare).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::DiscardPile
    );
}
}

mod original_tw_guohuai {
//! Golden coverage for TW 郭淮「精策/御章」.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWJINGCE: u32 = 3_238_855_457;
const YUZHANG: u32 = 2_473_230_123;
const DONT_USE: u32 = 3_964_351_777;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twguohuai.skill");

fn skill(tx: &Transaction, owner: PlayerId, def: u32) -> engine_api::SkillId {
    tx.state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

#[test]
fn jingce_earns_mark_and_yuzhang_bans_source() {
    let package = content_compiler::compile_source_named("tw-guohuai-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 388);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_207, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, source) = (players[0], players[1]);
    let remove: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(TWJINGCE),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(YUZHANG),
            },
        ],
    )
    .unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let jingce = skill(&tx, owner, TWJINGCE);
    let card = tx.state().hand_cards(owner)[0];
    let hand_before = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwguohuai {
            player: owner,
            target: owner,
            source: owner,
            card,
            mark_def: YUZHANG,
            helper_skill_def: DONT_USE,
            action: 0,
            stage: 0,
            skill: jingce,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand_before + 2);
    assert!(!tx.state().skills.get(jingce).unwrap().state_blob.is_empty());
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwguohuai {
            player: owner,
            target: source,
            source: owner,
            card,
            mark_def: YUZHANG,
            helper_skill_def: DONT_USE,
            action: 1,
            stage: 0,
            skill: jingce,
        }],
    )
    .unwrap();
    assert!(tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .contains(&(YUZHANG, 1)));

    let yuzhang = skill(&tx, owner, YUZHANG);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwguohuai {
                player: owner,
                target: owner,
                source,
                card: CardId::NONE,
                mark_def: YUZHANG,
                helper_skill_def: DONT_USE,
                action: 3,
                stage: 0,
                skill: yuzhang,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
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
    assert!(!state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, _)| *mark == YUZHANG));
    assert!(state
        .players
        .get(source)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == DONT_USE)
        }));

    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let yuzhang = skill(&tx, owner, YUZHANG);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddMark {
                player: owner,
                mark: engine_api::MarkDefId::new(YUZHANG),
                delta: 1,
            },
            EffectRequest::ResolveTwguohuai {
                player: owner,
                target: owner,
                source: PlayerId::NONE,
                card: CardId::NONE,
                mark_def: YUZHANG,
                helper_skill_def: DONT_USE,
                action: 4,
                stage: 0,
                skill: yuzhang,
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().turn.phase, Phase::Judge);
}
}

mod original_tw_jieqiu_impl {
//! tw/twjieqiu 劫囚（第一百零二批·§7 装备栏编排首迁）：
//! resolve_twjieqiu 多路器 sub0（选目标禁全五槽+摸 k+BUFF 标）/ sub2（余禁用栏
//! → owner 入队纯额外回合）。BUFF 标 = blake2b("twjieqiu_buff") = 385592780。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWJIEQIU: u32 = 4_201_797_709;
const TWJIEQIU_BUFF: u32 = 385_592_780;

const SOURCES_DIR: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../content/original/ported/tw");

fn sources() -> String {
    let dir = std::path::Path::new(SOURCES_DIR);
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "skill"))
        .map(|e| e.path())
        .collect();
    files.sort();
    files
        .iter()
        .map(|p| {
            let mut s = std::fs::read_to_string(p).unwrap();
            s.push('\n');
            s
        })
        .collect()
}


fn drive(
    tx: &mut Transaction,
    events: &[engine_api::DomainEvent],
) -> Result<(), EffectError> {
    match engine_core::trigger::dispatch_for_events(
        tx,
        events,
        &mut engine_core::trigger::TriggerFireRecord::new(),
    ) {
        Err(e) => Err(e),
        Ok(effects) => run_effects(tx, effects).map(|_| ()),
    }
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}

fn started() -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("tw-jieqiu-impl", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_837, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWJIEQIU),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}

fn others_of(state: &GameState, owner: PlayerId) -> Vec<PlayerId> {
    state
        .players
        .iter_live()
        .map(|p| p.id)
        .filter(|id| *id != owner)
        .collect()
}

fn buff_count(state: &GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(player)
        .and_then(|p| {
            p.marks
                .iter()
                .find(|(id, _)| *id == TWJIEQIU_BUFF)
                .map(|(_, c)| *c)
        })
        .unwrap_or(0)
}

fn equip_card(tx: &mut Transaction, owner: PlayerId, slot_tag: u8) -> engine_api::CardId {
    let zone = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let (kind, def) = match slot_tag {
        1 => (engine_core::state::CardKind::Weapon, 11),
        2 => (engine_core::state::CardKind::Armor, 12),
        3 => (engine_core::state::CardKind::HorseDef, 13),
        4 => (engine_core::state::CardKind::HorseOff, 14),
        _ => (engine_core::state::CardKind::Treasure, 15),
    };
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(def),
        kind,
        suit: engine_api::Suit::Spade,
        rank: 1,
        weapon_range: 0,
        zone,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn twjieqiu_publishes_contract() {
    let compiled = content_compiler::compile_source_named("tw-jieqiu-contract", &sources()).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 733);
    assert!(compiled.active_index.iter().any(|a| a.skill_def_id == TWJIEQIU));
}

#[test]
fn twjieqiu_disables_all_slots_draws_and_marks_target() {
    let (state, holder) = started();
    let victim = others_of(&state, holder)[0];
    let mut tx = Transaction::begin(state);

    // 给 victim 两件装备（武器+防具马），记录禁用前手牌数。
    equip_card(&mut tx, victim, 1);
    equip_card(&mut tx, victim, 3);
    let victim_hand_before = tx.state().hand_count(victim);

    // 发动 sub0：选目标段。
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    tx.state_mut().turn.current = holder;
    let sid = {
        let s = tx.state();
        s.skills
            .sorted_live()
            .iter()
            .find(|s| s.def.raw() == TWJIEQIU)
            .unwrap()
            .id
    };
    let opened = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwjieqiu {
            player: holder,
            target: engine_api::PlayerId::NONE,
            subaction: 0,
            skill: sid,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::TwjieqiuSelect
    );
    let committed = tx.commit();
    let observer = committed.state.resolution.pending_observer;
    let final_state = resolve(committed.state, PlayerId::new(observer), vec![victim.raw()], 1601);

    // 全五槽禁用、BUFF 标记在场、摸 k=2 张。
    let victim_state = final_state.players.get(victim).unwrap();
    assert!(victim_state
        .disabled_equip_slots
        .iter()
        .all(|&count| count >= 1));
    assert_eq!(buff_count(&final_state, victim), 1);
    assert_eq!(final_state.hand_count(victim), victim_hand_before + 2);
}

#[test]
fn twjieqiu_sub2_enqueues_owner_only_when_slots_remain() {
    let (state, holder) = started();
    let victim = others_of(&state, holder)[0];
    let mut tx = Transaction::begin(state);
    let sid = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == TWJIEQIU)
        .unwrap()
        .id;

    // sub0 废除（无装备：k=0，仅打 BUFF 标）。
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    tx.state_mut().turn.current = holder;
    let opened = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwjieqiu {
            player: holder,
            target: engine_api::PlayerId::NONE,
            subaction: 0,
            skill: sid,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let committed = tx.commit();
    let observer = committed.state.resolution.pending_observer;
    let bound = resolve(committed.state, PlayerId::new(observer), vec![victim.raw()], 1611);
    assert_eq!(buff_count(&bound, victim), 1);

    // sub2：目标仍有禁用栏 → owner 入队纯额外回合。
    let mut tx = Transaction::begin(bound);
    tx.state_mut().turn.current = victim;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwjieqiu {
            player: holder,
            target: victim,
            subaction: 2,
            skill: sid,
        }],
    )
    .unwrap();
    let judged = tx.commit().state;
    assert_eq!(judged.turn.extra_turn_queue.first(), Some(&holder));
    assert_eq!(
        judged
            .turn
            .flag(engine_core::state::turn_flag::extra_turn_check_pending_by_player(
                holder
            )),
        0
    );
}


#[test]
fn twjieqiu_discard_restores_disabled_slots_incrementally() {
    let (state, holder) = started();
    let victim = others_of(&state, holder)[0];
    let mut tx = Transaction::begin(state);
    let sid = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == TWJIEQIU)
        .unwrap()
        .id;

    // sub0：禁全五槽+BUFF 标。
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    tx.state_mut().turn.current = holder;
    let opened = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwjieqiu {
            player: holder,
            target: engine_api::PlayerId::NONE,
            subaction: 0,
            skill: sid,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let committed = tx.commit();
    let observer = committed.state.resolution.pending_observer;
    let bound = resolve(committed.state, PlayerId::new(observer), vec![victim.raw()], 1701);
    let disabled_total = |state: &GameState, player: PlayerId| -> i32 {
        state
            .players
            .get(player)
            .map(|p| p.disabled_equip_slots.iter().filter(|&&c| c > 0).count() as i32)
            .unwrap_or(0)
    };
    assert_eq!(disabled_total(&bound, victim), 5);

    // 弃 3 张 → 即时恢复 3 槽（剩 2）。
    let mut tx = Transaction::begin(bound);
    // 恢复钩子按「当前回合角色的弃牌」计数：victim 弃牌发生在 victim 回合。
    tx.state_mut().turn.current = victim;
    let cards = inject_cards(&mut tx, victim, 3);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, engine_api::PlayerId::NONE)
        .unwrap();
    let moved = drive(
        &mut tx,
        &[engine_api::DomainEvent::CardsMoved {
            cards,
            from: engine_api::ZoneId::NONE,
            to: discard,
            reason: engine_api::CardMoveReason::Discard,
        }],
    );
    moved.unwrap();
    let after = tx.commit().state;
    assert_eq!(disabled_total(&after, victim), 2, "3 discard restores 3 of 5 slots");
}

fn inject_cards(tx: &mut Transaction, owner: PlayerId, count: u8) -> Vec<engine_api::CardId> {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    (0..count)
        .map(|index| {
            let card = tx.state_mut().cards.alloc(Card {
                id: CardId::NONE,
                def: engine_api::CardDefId::new(1 + index as u32),
                kind: engine_core::state::CardKind::Tao,
                suit: engine_api::Suit::Heart,
                rank: 2 + index,
                weapon_range: 0,
                zone: hand,
                owner,
                face_up: true,
            });
            tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
            card
        })
        .collect()
}
}

mod original_tw_juyan {
//! Golden for TW「炬湮」的火焰伤害语义。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const JUYAN: u32 = 2_964_355_463;
const MANJI: u32 = 3_959_991_082;
const LINGBAO_XIANHU: u32 = 2_267_146_098;
const JINTAO: u32 = 3_887_207_405;
const TWSAOTAO: u32 = 4_248_892_988;
/// Concatenate every committed tw source in pack order (alphabetical),
/// mirroring `content-cli pack` input concatenation so the logic hash matches
/// the committed npack regardless of future roster growth.
fn tw_sources() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/tw");
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

const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

#[test]
fn committed_package_only_reacts_to_fire_damage_from_owner() {
    std::thread::Builder::new().stack_size(16 << 20).spawn(|| {
        let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
        let mut compiled = content_compiler::compile_source_named("tw", &tw_sources()).unwrap();
        compiled.character_index = committed.character_index.clone();
        compiled.manifest.dependencies = committed.manifest.dependencies.clone();
        compiled.manifest.logic_hash = compiled.compute_logic_hash();
        assert_eq!(committed.manifest.logic_hash, compiled.manifest.logic_hash);
        assert!(committed
            .skill_index
            .iter()
            .any(|skill| skill.skill_def_id == JUYAN));

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
                skill_def: SkillDefId::new(JUYAN),
            }],
        )
        .unwrap();
        let hand = tx.state().hand_count(owner);

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
        assert_eq!(tx.state().hand_count(owner), hand);
        assert_eq!(tx.state().players.get(other).unwrap().max_hp, 10);

        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target: other,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::FIRE,
            }],
        )
        .unwrap();
        assert_eq!(tx.state().hand_count(owner), hand + 1);
        assert_eq!(tx.state().players.get(other).unwrap().max_hp, 9);
    });
}

#[test]
fn manji_executes_both_effects_when_hp_becomes_equal() {
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
    let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
    owner_state.hp = 2;
    owner_state.max_hp = 4;
    tx.state_mut().players.get_mut(other).unwrap().hp = 3;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(MANJI),
        }],
    )
    .unwrap();
    let hand = tx.state().hand_count(owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: other,
            amount: 1,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(other).unwrap().hp, 2);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 3);
    assert_eq!(tx.state().hand_count(owner), hand + 1);
}

#[test]
fn lingbao_xianhu_requires_large_source_damage_but_accepts_any_death() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: 20_260_715,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&committed).unwrap();
    let owner = engine.state().turn.current;
    let mut others = engine
        .state()
        .players
        .iter_live()
        .filter(|player| player.id != owner)
        .map(|player| player.id);
    let target = others.next().unwrap();
    let victim = others.next().unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
    owner_state.hp = 2;
    owner_state.max_hp = 4;
    let target_state = tx.state_mut().players.get_mut(target).unwrap();
    target_state.hp = 10;
    target_state.max_hp = 10;
    for player in [owner, target, victim] {
        for skill in tx.state().players.get(player).unwrap().skills.clone() {
            run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
        }
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(LINGBAO_XIANHU),
        }],
    )
    .unwrap();
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;

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
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, max_hp);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, max_hp + 1);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 3);

    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer: target,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, max_hp + 2);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 4);
}

#[test]
fn jintao_rewrites_the_first_two_sha_uses_only() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(JINTAO as u64),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: JINTAO as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let owner_index = seats.iter().position(|player| *player == owner).unwrap();
    let target = seats[(owner_index + 2) % seats.len()];
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    for player in [owner, target] {
        for skill in tx.state().players.get(player).unwrap().skills.clone() {
            run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
        }
    }
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(JINTAO),
            },
            EffectRequest::DiscardFromHand {
                player: target,
                count: u32::MAX,
            },
        ],
    )
    .unwrap();
    let target_state = tx.state_mut().players.get_mut(target).unwrap();
    target_state.hp = 10;
    target_state.max_hp = 10;
    assert_eq!(engine_core::query::sha_use_limit_for(tx.state(), owner), 2);

    let owner_hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let first_sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(JINTAO + 1),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: owner_hand,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(owner_hand)
        .unwrap()
        .cards
        .push(first_sha);
    assert!(engine_core::query::card_distance_unlimited_for_target(
        tx.state(),
        owner,
        first_sha,
        target
    ));

    tx.state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 1);
    assert_eq!(
        engine_core::query::sha_damage_for_card(tx.state(), owner, first_sha, 1),
        2
    );
    assert!(!engine_core::query::sha_unrespondable_for_card_target(
        tx.state(),
        owner,
        target,
        first_sha
    ));
    tx.state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 2);
    assert_eq!(
        engine_core::query::sha_damage_for_card(tx.state(), owner, first_sha, 1),
        1
    );
    assert!(engine_core::query::sha_unrespondable_for_card_target(
        tx.state(),
        owner,
        target,
        first_sha
    ));
    tx.state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 3);
    assert_eq!(
        engine_core::query::sha_damage_for_card(tx.state(), owner, first_sha, 1),
        1
    );
    assert!(!engine_core::query::sha_unrespondable_for_card_target(
        tx.state(),
        owner,
        target,
        first_sha
    ));
    tx.state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 0);

    let (state, first_result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: first_sha,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(
        first_result.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(state.players.get(target).unwrap().hp, 8);
    assert_eq!(
        state.turn.flag(engine_core::state::turn_flag::SHA_PLAYED),
        1
    );

    let mut tx = Transaction::begin(state);
    let target_hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let shan = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(JINTAO + 2),
        kind: CardKind::Shan,
        suit: Suit::Heart,
        rank: 2,
        weapon_range: 0,
        zone: target_hand,
        owner: target,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(target_hand)
        .unwrap()
        .cards
        .push(shan);
    let second_sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(JINTAO + 3),
        kind: CardKind::Sha,
        suit: Suit::Club,
        rank: 8,
        weapon_range: 0,
        zone: owner_hand,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(owner_hand)
        .unwrap()
        .cards
        .push(second_sha);
    let (state, second_result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::PlayCard {
                card: second_sha,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(
        second_result.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(state.players.get(target).unwrap().hp, 7);
    assert!(state.hand_cards(target).contains(&shan));
    assert!(!state.has_pending_choice());
}

#[test]
fn twsaotao_makes_sha_and_ordinary_tricks_unrespondable() {
    let compiled = content_compiler::compile_source_named(
        "twsaotao",
        include_str!("../../../content/original/ported/tw/twsaotao.skill"),
    )
    .unwrap();
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == TWSAOTAO && skill.locked));
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == TWSAOTAO
            && modifier.target == "sha_unrespondable"
            && modifier.selector.as_deref() == Some("any")
    }));
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == TWSAOTAO
            && modifier.target == "card_use_unrespondable"
            && modifier.selector.as_deref() == Some("ordinary_trick")
    }));
}
}

mod original_tw_niujin {
//! Golden coverage for TW 牛金「挫锐」「裂围」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWCUORUI: u32 = 3_369_253_433;
const TWLIEWEI: u32 = 2_387_969_096;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twniujin.skill");

fn choose(state: GameState, actor: PlayerId, selection: u32) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn cuorui_disables_judge_and_liewei_can_reset_it() {
    let package = content_compiler::compile_source_named("tw-niujin-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 391);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_210, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, other) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWCUORUI),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWLIEWEI),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let cuorui = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWCUORUI)
        .unwrap()
        .id;
    let liewei = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWLIEWEI)
        .unwrap()
        .id;
    let target_hand = tx.state().hand_count(other);
    // 挫锐当前实现与上游一致：自动补牌（至最高手牌、最多 5 张）并废除判定区，无确认 choice。
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(state.hand_count(owner), target_hand.min(5));
    assert!(state.disabled_judge_zones.contains(&owner));
    assert!(state.zones.find(ZoneKind::Judge, owner).is_some());
    assert_eq!(state.skills.get(cuorui).unwrap().lifetime_fire_count, 1);

    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwniujin {
                player: owner,
                limited_skill_def: SkillDefId::new(TWCUORUI),
                action: 1,
                stage: 0,
                skill: liewei,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = choose(tx.commit().state, owner, 1);
    assert_eq!(state.skills.get(cuorui).unwrap().lifetime_fire_count, 0);
}
}

mod original_tw_old_quancong {
//! Golden coverage for TW 旧全琮「振赡」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const ZHENSHAN: u32 = 2_898_394_319;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/zhenshan.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn command(actor: PlayerId, key: u64, kind: CommandKind) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind,
    }
}

#[test]
fn zhenshan_swaps_with_lower_hand_target_then_obeys_turn_limit() {
    let package = content_compiler::compile_source_named("tw-zhenshan-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 383);
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .character_index
        .iter()
        .any(|character| { character.id == 3_552_851_534 && character.skills == [ZHENSHAN] }));
    let mut engine = Engine::new();
    engine.submit(command(
        PlayerId::NONE,
        1,
        CommandKind::StartGame {
            player_count: 2,
            rng_seed: 2_026_080_124,
            ruleset: "identity-standard".into(),
        },
    ));
    engine.load_package(&package).unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let owner = tx.state().turn.current;
    let target = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZHENSHAN),
        }],
    )
    .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let extra = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_840),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank: 6,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(hand)
        .unwrap()
        .cards
        .push(extra);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == ZHENSHAN)
        .unwrap();
    let owner_cards = tx.state().hand_cards(owner);
    let target_cards = tx.state().hand_cards(target);
    let state = tx.commit().state;

    let (state, result) = apply_command(
        state,
        command(
            owner,
            2,
            CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::Jiu.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        ),
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    assert!(state.has_pending_choice());
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        command(
            owner,
            3,
            CommandKind::ResolveChoice {
                choice_id,
                selections: vec![target.raw()],
                cancelled: false,
            },
        ),
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_cards(owner), target_cards);
    assert_eq!(state.hand_cards(target), owner_cards);
    assert_eq!(state.skills.get(skill).unwrap().use_count, 1);

    let (_, result) = apply_command(
        state,
        command(
            owner,
            4,
            CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::Jiu.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        ),
    );
    assert!(!matches!(result.result, CommandResult::Accepted { .. }));
}
}

mod original_tw_old_twrouke {
//! Golden for TW「柔克」的多牌获得与摸牌阶段排除语义。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::state::Phase;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const OLD_TWROUKE: u32 = 3_805_481_666;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn isolated_state() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(
        package
            .trigger_index
            .iter()
            .filter(|trigger| trigger.skill_def_id == OLD_TWROUKE)
            .count(),
        2
    );

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(OLD_TWROUKE as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: OLD_TWROUKE as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLD_TWROUKE),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, other)
}

#[test]
fn draws_once_for_each_multi_card_gain_outside_the_owners_draw_phase() {
    let (mut tx, owner, other) = isolated_state();

    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 2,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 3);

    tx.state_mut().turn.phase = Phase::Draw;
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 2,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 2);

    tx.state_mut().turn.phase = Phase::Play;
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: other,
            count: 2,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand);

    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);

    let from = tx
        .state()
        .zones
        .find(engine_core::state::ZoneKind::Hand, other)
        .unwrap();
    let to = tx
        .state()
        .zones
        .find(engine_core::state::ZoneKind::Hand, owner)
        .unwrap();
    let cards = tx.state().hand_cards(other)[..2].to_vec();
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards,
            to,
            from: Some(from),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 3);
}
}

mod original_tw_puyangxing {
//! Golden coverage for TW 濮阳兴「征建/众斥」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWZHENGJIAN: u32 = 3_256_371_970;
const TWZHONGCHI_EFFECT: u32 = 3_469_371_163;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twpuyangxing.skill");

fn inject(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit: Suit::Diamond,
        rank: 8,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn distinct_tribute_sources_awaken_zhongchi_then_enable_damage() {
    let package = content_compiler::compile_source_named("tw-puyangxing-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 381);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 4, 2_026_080_122, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second, third) = (players[0], players[1], players[2], players[3]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWZHENGJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWZHENGJIAN)
        .unwrap()
        .id;
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwzhengjian {
                player: owner,
                target: owner,
                helper_skill_def: TWZHONGCHI_EFFECT,
                action: 0,
                stage: 0,
                card: CardId::NONE,
                branch: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = choose(tx.commit().state, owner, vec![0], 1);

    for (index, target) in [first, second].into_iter().enumerate() {
        let mut tx = Transaction::begin(state);
        let card = inject(&mut tx, target, 93_810 + index as u32);
        assert_eq!(
            run_effects(
                &mut tx,
                vec![EffectRequest::ResolveTwzhengjian {
                    player: owner,
                    target,
                    helper_skill_def: TWZHONGCHI_EFFECT,
                    action: 1,
                    stage: 0,
                    card: CardId::NONE,
                    branch: 0,
                    skill,
                }],
            ),
            Err(EffectError::ChoiceRequested)
        );
        state = choose(
            tx.commit().state,
            target,
            vec![card.raw()],
            10 + index as u64 * 2,
        );
        state = choose(state, owner, vec![0], 11 + index as u64 * 2);
    }

    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert!(state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == TWZHONGCHI_EFFECT)
        }));
    let target_hp = state.players.get(third).unwrap().hp;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwzhengjian {
                player: owner,
                target: third,
                helper_skill_def: TWZHONGCHI_EFFECT,
                action: 1,
                stage: 0,
                card: CardId::NONE,
                branch: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![1], 20);
    assert_eq!(state.players.get(third).unwrap().hp, target_hp - 1);
}
}

mod original_tw_sunyi {
//! Golden coverage for TW 孙翊「躁厉」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWZAOLI: u32 = 2_591_520_200;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twsunyi.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn inject(
    tx: &mut Transaction,
    zone_kind: ZoneKind,
    owner: PlayerId,
    kind: CardKind,
    def: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 6,
        weapon_range: 2,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn zaoli_allows_only_new_hand_cards_and_refills_three_equipment_slots() {
    let package = content_compiler::compile_source_named("tw-sunyi-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 380);
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .character_index
        .iter()
        .any(|character| { character.id == 3_013_394_897 && character.skills == [TWZAOLI] }));
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 2_026_080_121, "identity-standard").unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let initial_hand = tx.state().hand_cards(owner);
    if !initial_hand.is_empty() {
        effects.push(EffectRequest::MoveCards {
            cards: initial_hand,
            to: tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap(),
            from: Some(tx.state().zones.find(ZoneKind::Hand, owner).unwrap()),
        });
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWZAOLI),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let old = inject(&mut tx, ZoneKind::Hand, owner, CardKind::Sha, 93_800);
    let gained = inject(
        &mut tx,
        ZoneKind::DrawPile,
        PlayerId::NONE,
        CardKind::Shan,
        93_801,
    );
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![gained],
            to: hand,
            from: Some(draw),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().playable_hand_cards(owner), vec![gained]);

    for (kind, def) in [
        (CardKind::Weapon, 93_802),
        (CardKind::Armor, 93_803),
        (CardKind::HorseDef, 93_804),
    ] {
        inject(&mut tx, ZoneKind::Equip, owner, kind, def);
        inject(&mut tx, ZoneKind::DrawPile, PlayerId::NONE, kind, def + 10);
    }
    let zaoli = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWZAOLI)
        .unwrap()
        .id;
    let hp = tx.state().players.get(owner).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwzaoli {
                player: owner,
                stage: 0,
                cards: Vec::new(),
                slots: Vec::new(),
                count: 0,
                skill: zaoli,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![old.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
    assert_eq!(
        state
            .zones
            .find(ZoneKind::Equip, owner)
            .and_then(|zone| state.zones.get(zone))
            .unwrap()
            .cards
            .len(),
        3
    );
}
}

mod original_tw_tianyu {
//! Golden coverage for TW 田豫「震袭/扬师」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWZHENXI: u32 = 3_713_822_964;
const TWYANGSHI: u32 = 4_235_144_837;
const TWYANGSHI_DISTANCE: u32 = 3_469_371_167;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twtianyu.skill");

fn inject(tx: &mut Transaction, zone: ZoneKind, owner: PlayerId, kind: CardKind) -> CardId {
    let zone = tx.state().zones.find(zone, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_820 + kind.tag() as u32),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 2,
        zone,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn zhenxi_backwater_discards_then_moves_and_yangshi_grows_then_gains_sha() {
    let package = content_compiler::compile_source_named("tw-tianyu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 382);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 4, 2_026_080_123, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target, destination) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWZHENXI),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWYANGSHI),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let discarded = inject(&mut tx, ZoneKind::Hand, target, CardKind::Shan);
    let weapon = inject(&mut tx, ZoneKind::Equip, target, CardKind::Weapon);
    let zhenxi = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWZHENXI)
        .unwrap()
        .id;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwtianyu {
                player: owner,
                target,
                helper_skill_def: TWYANGSHI_DISTANCE,
                action: 0,
                stage: 0,
                card: CardId::NONE,
                branch: 0,
                skill: zhenxi,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![3], 1);
    let state = choose(state, owner, vec![discarded.raw()], 2);
    let state = choose(state, owner, vec![weapon.raw()], 3);
    let state = choose(state, owner, vec![destination.raw()], 4);
    assert_eq!(
        state.cards.get(discarded).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert_eq!(
        state.cards.get(weapon).unwrap().zone,
        state.zones.find(ZoneKind::Equip, destination).unwrap()
    );

    let yangshi = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWYANGSHI)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwtianyu {
            player: owner,
            target: owner,
            helper_skill_def: TWYANGSHI_DISTANCE,
            action: 1,
            stage: 0,
            card: CardId::NONE,
            branch: 0,
            skill: yangshi,
        }],
    )
    .unwrap();
    assert!(tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == TWYANGSHI_DISTANCE && *count == 1));
    let hand_before = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwtianyu {
            player: owner,
            target: owner,
            helper_skill_def: TWYANGSHI_DISTANCE,
            action: 1,
            stage: 0,
            card: CardId::NONE,
            branch: 0,
            skill: yangshi,
        }],
    )
    .unwrap();
    // 四座时并非全员在攻击范围内：追加距离标记而非直接获得【杀】。
    assert_eq!(tx.state().hand_count(owner), hand_before);
    assert!(tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == TWYANGSHI_DISTANCE && *count == 2));
}
}

mod original_tw_tw_sunluban {
//! Golden coverage for TW 孙鲁班「骄矜」and package closure.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TW_SUNLUBAN: u32 = 2_766_500_142;
const TW_FENGXI: u32 = 3_675_106_908;
const TWZENHUI: u32 = 3_053_684_104;
const XINJIAOJIN: u32 = 2_479_658_413;
const TW: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");
const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn inherited_jiaojin_discards_equipment_and_prevents_the_whole_damage() {
    let tw = content_schema::ContentPackage::from_json_bytes(TW).unwrap();
    let mobile = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    assert_eq!(tw.manifest.rule_vm_abi_version, 733);
    assert_eq!(mobile.manifest.rule_vm_abi_version, 734);
    assert_eq!(
        tw.character_index
            .iter()
            .find(|character| character.id == TW_SUNLUBAN)
            .unwrap()
            .skills,
        [TWZENHUI, XINJIAOJIN]
    );

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 2_026_080_204, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&mobile).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&tw).unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, source) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINJIAOJIN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().character_def = TW_SUNLUBAN;
    tx.state_mut()
        .players
        .get_mut(source)
        .unwrap()
        .character_def = TW_FENGXI;
    let hp = tx.state().players.get(owner).unwrap().hp;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let equipment = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_801),
        kind: CardKind::Armor,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(hand)
        .unwrap()
        .cards
        .push(equipment);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target: owner,
                amount: 2,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }]
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 1);
    let state = resolve(state, owner, vec![equipment.raw()], 2);
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(equipment).unwrap().zone, discard);
}
}

mod original_tw_twbaimei {
//! Golden coverage for TW 马良「白眉」.

use engine_api::{CardId, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{CardKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWBAIMEI: u32 = 3_304_922_519;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twbaimei.skill");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("twbaimei-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 358);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, source) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWBAIMEI),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, source)
}

fn damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    kind: CardKind,
    nature: u8,
) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature,
            card: CardId::NONE,
            card_kind: kind.tag(),
            card_flags: 0,
        }],
    )
    .unwrap();
    tx.commit().state
}

#[test]
fn empty_hand_prevents_elemental_or_ordinary_trick_only() {
    let (state, owner, source) = setup(20_260_801);
    let hp = state.players.get(owner).unwrap().hp;
    let state = damage(
        state,
        source,
        owner,
        CardKind::ShaFire,
        rule_api::damage_nature::FIRE,
    );
    assert_eq!(state.players.get(owner).unwrap().hp, hp);

    let (state, owner, source) = setup(20_260_802);
    let hp = state.players.get(owner).unwrap().hp;
    let state = damage(
        state,
        source,
        owner,
        CardKind::TrickJuedou,
        rule_api::damage_nature::NORMAL,
    );
    assert_eq!(state.players.get(owner).unwrap().hp, hp);

    let (state, owner, source) = setup(20_260_803);
    let hp = state.players.get(owner).unwrap().hp;
    let state = damage(
        state,
        source,
        owner,
        CardKind::Sha,
        rule_api::damage_nature::NORMAL,
    );
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
}
}

mod original_tw_twchenxun {
//! Golden coverage for TW 幻刘封「陈勋」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{CardKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TWCHENXUN: u32 = 4_060_298_774;
const HUAN_LIUFENG: u32 = 3_034_906_798;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twchenxun.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn setup(seed: u64, responder_has_sha: bool) -> (Transaction, [PlayerId; 3], Option<u32>) {
    let package = content_compiler::compile_source_named("twchenxun-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 397);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players: [PlayerId; 3] = tx.state().players.alive_by_seat().try_into().unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(TWCHENXUN),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in players {
        for card in tx.state().hand_cards(player) {
            tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
        }
    }
    let response = responder_has_sha.then(|| {
        let card = tx.state().hand_cards(players[1])[0];
        tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Sha;
        card.raw()
    });
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::Prepare;
    tx.state_mut().turn.round = 1;
    (tx, players, response)
}

fn fire(tx: &mut Transaction, owner: PlayerId) -> Result<(), EffectError> {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )?;
    run_effects(tx, effects).map(|_| ())
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn chains_only_after_target_damage_and_can_be_cancelled() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == HUAN_LIUFENG)
            .unwrap()
            .skills,
        vec![TWCHENXUN]
    );

    let (mut tx, [owner, first, second], _) = setup(2_026_080_201, false);
    let owner_hand = tx.state().hand_count(owner);
    assert_eq!(fire(&mut tx, owner), Err(EffectError::ChoiceRequested));
    let (state, result) = resolve(tx.commit().state, owner, vec![first.raw()], false, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_hand + 1);
    assert_eq!(state.resolution.continuation_payload[4..], [second.raw()]);
    let (state, result) = resolve(state, owner, vec![second.raw()], false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_hand + 2);
    assert!(!state.has_pending_choice());

    let (mut tx, [owner, target, _], response) = setup(2_026_080_202, true);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    assert_eq!(fire(&mut tx, owner), Err(EffectError::ChoiceRequested));
    let (state, _) = resolve(tx.commit().state, owner, vec![target.raw()], false, 3);
    let (state, result) = resolve(state, target, vec![response.unwrap()], false, 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 2);
    assert!(!state.has_pending_choice());

    let (mut tx, [owner, _, _], _) = setup(2_026_080_203, false);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    assert_eq!(fire(&mut tx, owner), Err(EffectError::ChoiceRequested));
    let (state, result) = resolve(tx.commit().state, owner, Vec::new(), true, 5);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp);
    assert!(!state.has_pending_choice());
}
}

mod original_tw_twchuhai {
//! Golden coverage for TW 周处「除害」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWCHUHAI: u32 = 3_782_702_310;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twchuhai.skill");

fn inject(tx: &mut Transaction, owner: PlayerId, zone_kind: ZoneKind, rank: u8) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_372_000 + u32::from(rank)),
        kind: CardKind::Tao,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(state: GameState, actor: PlayerId, card: CardId, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![card.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn success_disables_judge_zone_collects_clockwise_and_discards_each_new_card() {
    let package = content_compiler::compile_source_named("twchuhai-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 372);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_113, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWCHUHAI),
    });
    run_effects(&mut tx, effects).unwrap();
    let mission = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWCHUHAI)
        .unwrap()
        .id;
    let judge = inject(&mut tx, owner, ZoneKind::Judge, 1);
    let first_card = inject(&mut tx, first, ZoneKind::Hand, 2);
    let second_card = inject(&mut tx, second, ZoneKind::Equip, 3);
    tx.state_mut().turn.record_dying_caused(owner, first);
    tx.state_mut().turn.record_dying_caused(owner, second);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwchuhai {
            player: owner,
            action: 0,
            stage: 0,
            index: 0,
            event_cards: Vec::new(),
            skill: mission,
        }],
    );
    assert!(matches!(
        result,
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    let mut state = tx.commit().state;

    state = resolve(state, first, first_card, 1);
    state = resolve(state, owner, first_card, 2);
    state = resolve(state, second, second_card, 3);
    state = resolve(state, owner, second_card, 4);

    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state.disabled_judge_zones.contains(&owner));
    assert_eq!(state.cards.get(judge).unwrap().zone, discard);
    assert!(state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.id == mission));
    assert_eq!(state.cards.get(first_card).unwrap().zone, discard);
    assert_eq!(state.cards.get(second_card).unwrap().zone, discard);
    assert!(!state.has_pending_choice());
}
}

mod original_tw_twdangjiang {
//! Golden coverage for TW「荡疆」(twdangjiang).
//! Play-phase view-as: a hand/equip card as 无中生有 or a distance-ignoring Sha.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWDANGJIANG: u32 = 2_498_780_514;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twdangjiang.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, SkillId, CardId) {
    let compiled = content_compiler::compile_source_named("twdangjiang-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_44, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWDANGJIANG),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_804),
        kind: CardKind::Shan,
        suit: Suit::Club,
        rank: 3,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == TWDANGJIANG)
        .unwrap();
    (tx.commit().state, owner, other, skill, card)
}

#[test]
fn compiled_source_and_npack_publish_wuzhong_and_sha() {
    let compiled = content_compiler::compile_source_named("twdangjiang", SOURCE).unwrap();
    let mut views: Vec<_> = compiled
        .view_as_index
        .iter()
        .filter(|view| view.skill_def_id == TWDANGJIANG)
        .map(|view| (view.as_kind.as_str(), view.ignore_distance))
        .collect();
    views.sort();
    assert_eq!(views, vec![("sha", true), ("wuzhong", false)]);

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut pack: Vec<_> = committed
        .view_as_index
        .iter()
        .filter(|view| view.skill_def_id == TWDANGJIANG)
        .map(|view| view.as_kind.as_str())
        .collect();
    pack.sort();
    assert_eq!(pack, vec!["sha", "wuzhong"]);
}

#[test]
fn hand_card_converts_to_sha() {
    let (state, owner, other, skill, card) = setup();
    let hp = state.players.get(other).unwrap().hp;
    let (state, pipe) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![other],
                card_targets: vec![card],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert_eq!(state.players.get(other).unwrap().hp, hp - 1);
}
}

mod original_tw_twdanji {
//! Golden coverage for TW JSP 关羽「单骑」.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWDANJI: u32 = 2_211_821_528;
const TWDANJI_EFFECT: u32 = 2_360_461_242;
const TW_JSP_GUANYU: u32 = 4_246_243_945;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/tw/twdanji.skill"),
    "\nskill test_wusheng(16) { view_as sha play_phase red; }\n",
);
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_357_000 + u32::from(rank)),
        kind: CardKind::Tao,
        suit: Suit::Heart,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_and_first_transformed_sha_draw_are_exact() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed.manifest.rule_vm_abi_version >= 357);
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == TW_JSP_GUANYU)
            .unwrap()
            .skills,
        [16, TWDANJI]
    );

    let package = content_compiler::compile_source_named("twdanji-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend(
        [16, TWDANJI_EFFECT].map(|skill_def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == 16)
        .unwrap()
        .id;
    let first = inject(&mut tx, owner, 3);
    let second = inject(&mut tx, owner, 4);
    let before = tx.state().hand_count(owner);

    let use_sha = |state: GameState, card, key| {
        let (state, result) = apply_command(
            state,
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ActivateSkill {
                    skill,
                    targets: vec![target],
                    card_targets: vec![card],
                },
            },
        );
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        state
    };

    let mut state = use_sha(tx.commit().state, first, 1);
    assert_eq!(state.hand_count(owner), before);
    state.turn.set_flag(turn_flag::SHA_PLAYED, 0);
    state.turn.set_flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
    let state = use_sha(state, second, 2);
    assert_eq!(state.hand_count(owner), before - 1);
}
}

mod original_tw_twdaoji {
//! Golden coverage for TW 胡车儿「盗戟」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWDAOJI: u32 = 3_856_876_988;
const TW_HUCHEER: u32 = 4_271_674_660;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twdaoji.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("twdaoji-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWDAOJI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWDAOJI)
        .unwrap()
        .id;
    Setup {
        tx,
        owner,
        target,
        skill,
    }
}

fn inject(tx: &mut Transaction, owner: PlayerId, zone: ZoneKind, kind: CardKind) -> CardId {
    let zone_id = tx.state().zones.find(zone, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_355_000 + kind.tag() as u32),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone: zone_id,
        owner,
        face_up: zone == ZoneKind::Equip,
    });
    tx.state_mut()
        .zones
        .get_mut(zone_id)
        .unwrap()
        .cards
        .push(card);
    card
}

fn resolve_choice(state: GameState, owner: PlayerId, card: CardId, key: u64) -> GameState {
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![card.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn start(mut setup: Setup, kind: CardKind, zone: ZoneKind, key: u64) -> (GameState, CardId) {
    let card = inject(&mut setup.tx, setup.target, zone, kind);
    let result = run_effects(
        &mut setup.tx,
        vec![EffectRequest::ResolveDaoji {
            player: setup.owner,
            target: setup.target,
            card: CardId::NONE,
            stage: 0,
            tw: true,
            skill: setup.skill,
        }],
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let state = setup.tx.commit().state;
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveDaoji
    );
    (resolve_choice(state, setup.owner, card, key), card)
}

#[test]
fn package_and_tw_category_rewards_are_exact() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 355);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == TW_HUCHEER)
            .unwrap()
            .skills,
        [4_258_102_225, TWDAOJI]
    );
    let targets = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == TWDAOJI)
        .unwrap()
        .targets
        .as_ref()
        .unwrap();
    assert!(targets.has_gainable_hand_equip && targets.in_attack_range);

    let basic = setup(2_026_080_101);
    let owner = basic.owner;
    let before = basic.tx.state().hand_count(owner);
    let (state, card) = start(basic, CardKind::Sha, ZoneKind::Hand, 1);
    assert_eq!(state.cards.get(card).unwrap().owner, owner);
    assert_eq!(state.hand_count(owner), before + 2);

    let equipment = setup(2_026_080_102);
    let owner = equipment.owner;
    let target = equipment.target;
    let hp = equipment.tx.state().players.get(target).unwrap().hp;
    let (state, card) = start(equipment, CardKind::Armor, ZoneKind::Equip, 2);
    let equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    assert_eq!(state.cards.get(card).unwrap().zone, equip);
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}
}

mod original_tw_twdianyi {
//! Golden for TW 王粲「典仪」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::ContinuationKind;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const DIANYI: u32 = 3_416_973_271;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn isolated(hand: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(DIANYI as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: DIANYI as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::DrawCards {
            player: owner,
            count: hand,
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DIANYI),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn phase_end_effects(tx: &mut Transaction, player: PlayerId) -> Vec<EffectRequest> {
    dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: player,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
}

#[test]
fn committed_package_exposes_both_mutually_exclusive_turn_damage_contracts() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 733);
    let filters: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == DIANYI)
        .map(|trigger| trigger.filter.as_str())
        .collect();
    assert_eq!(
        filters,
        vec![
            "owner_is_target_turn_damage",
            "owner_is_target_no_turn_damage"
        ]
    );
}

#[test]
fn damage_branch_discards_every_hand_card_and_other_turn_end_does_nothing() {
    let (mut tx, owner, other) = isolated(5);
    assert!(phase_end_effects(&mut tx, other).is_empty());
    tx.state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::DAMAGE_DEALT, 1);
    let effects = phase_end_effects(&mut tx, owner);
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), 0);
}

#[test]
fn no_damage_branch_draws_up_to_four() {
    let (mut tx, owner, _) = isolated(2);
    let effects = phase_end_effects(&mut tx, owner);
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), 4);
    assert!(!tx.state().has_pending_choice());
}

#[test]
fn no_damage_branch_forces_exact_excess_selection() {
    let (mut tx, owner, _) = isolated(6);
    let original = tx.state().hand_cards(owner);
    let effects = phase_end_effects(&mut tx, owner);
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::AdjustHandTo
    );
    let discarded = vec![original[1].raw(), original[4].raw()];
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(DIANYI as u64 + 1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: discarded.clone(),
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let hand = state.hand_cards(owner);
    assert_eq!(hand.len(), 4);
    assert!(discarded
        .into_iter()
        .all(|card| !hand.iter().any(|remaining| remaining.raw() == card)));
}
}

mod original_tw_twfengpo {
//! Golden coverage for TW 马云騄「凤魄」.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWFENGPO: u32 = 4_055_936_832;
const TW_MAYUNLU: u32 = 2_212_792_733;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twfengpo.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    def: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup() -> (GameState, [PlayerId; 3]) {
    let package = content_compiler::compile_source_named("twfengpo-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_207, "identity-standard").unwrap();
    let players: [PlayerId; 3] = tx.state().players.alive_by_seat().try_into().unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend([
        EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(TWFENGPO),
        },
        EffectRequest::SetTurnState {
            current: players[0],
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    for player in players {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.max_hp = 20;
        participant.hp = 15;
    }
    (tx.commit().state, players)
}

fn play(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    )
}

fn choose(
    state: GameState,
    pipeline: PipelineResult,
    actor: PlayerId,
    token: u32,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice = pipeline.choice.unwrap();
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice.choice_id.raw()),
                selections: vec![token],
                cancelled: false,
            },
        },
    )
}

fn activate_and_increase(
    state: GameState,
    pipeline: PipelineResult,
    owner: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    assert_eq!(pipeline.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    let (state, branch) = choose(state, pipeline, owner, 1, key);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::VmResume,
        "after accept: {:?} {:?}",
        branch.result,
        branch.choice
    );
    assert_eq!(branch.choice.as_ref().unwrap().kind, ChoiceKind::Custom);
    choose(state, branch, owner, 2, key + 1)
}

fn cancel_responses(
    mut state: GameState,
    mut pipeline: PipelineResult,
    mut key: u64,
) -> (GameState, PipelineResult) {
    while let Some(choice) = pipeline.choice.take() {
        let revision = state.revision;
        (state, pipeline) = apply_command(
            state,
            Command {
                actor: choice.observer,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: Vec::new(),
                    cancelled: true,
                },
            },
        );
        key += 1;
    }
    (state, pipeline)
}

#[test]
fn counts_hand_only_on_every_single_target_use_and_switches_to_red_after_kill() {
    std::thread::Builder::new().stack_size(16 << 20).spawn(|| {
        let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
        assert!(committed.manifest.rule_vm_abi_version >= 490);
        assert_eq!(
            committed
                .character_index
                .iter()
                .find(|character| character.id == TW_MAYUNLU)
                .unwrap()
                .skills,
            [2, TWFENGPO]
        );

        let (mut state, [owner, target, victim]) = setup();
        inject(
            &mut state,
            target,
            ZoneKind::Hand,
            CardKind::Other,
            Suit::Diamond,
            490_001,
        );
        inject(
            &mut state,
            target,
            ZoneKind::Hand,
            CardKind::Other,
            Suit::Diamond,
            490_002,
        );
        inject(
            &mut state,
            target,
            ZoneKind::Hand,
            CardKind::Other,
            Suit::Heart,
            490_003,
        );
        inject(
            &mut state,
            target,
            ZoneKind::Equip,
            CardKind::Armor,
            Suit::Diamond,
            490_004,
        );
        let sha = inject(
            &mut state,
            owner,
            ZoneKind::Hand,
            CardKind::Sha,
            Suit::Spade,
            490_005,
        );
        let hp = state.players.get(target).unwrap().hp;
        let (state, optional) = play(state, owner, sha, target, 1);
        let (state, result) = activate_and_increase(state, optional, owner, 2);
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        assert_eq!(state.players.get(target).unwrap().hp, hp - 3);

        let mut tx = Transaction::begin(state);
        let effects = engine_core::trigger::dispatch_for_events(
            &mut tx,
            &[engine_api::DomainEvent::PlayerKilled {
                player: victim,
                killer: owner,
                hand_equip_cards: Vec::new(),
            }],
            &mut engine_core::trigger::TriggerFireRecord::new(),
        )
        .unwrap();
        run_effects(&mut tx, effects).unwrap();
        let mut state = tx.commit().state;
        let juedou = inject(
            &mut state,
            owner,
            ZoneKind::Hand,
            CardKind::TrickJuedou,
            Suit::Club,
            490_006,
        );
        let hp = state.players.get(target).unwrap().hp;
        let (state, optional) = play(state, owner, juedou, target, 4);
        let (state, result) = activate_and_increase(state, optional, owner, 5);
        let (state, result) = cancel_responses(state, result, 7);
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        assert_eq!(state.players.get(target).unwrap().hp, hp - 4);

        let mut tx = Transaction::begin(state);
        let skill = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == owner && skill.def.raw() == TWFENGPO)
            .unwrap()
            .id;
        run_effects(
            &mut tx,
            vec![
                EffectRequest::RemoveSkill { skill },
                EffectRequest::AddSkill {
                    player: owner,
                    skill_def: SkillDefId::new(TWFENGPO),
                },
            ],
        )
        .unwrap();
        let replacement = tx
            .state()
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == owner && skill.def.raw() == TWFENGPO)
            .unwrap();
        assert_eq!(replacement.content_flags, 0);
    });
}
}

mod original_tw_twfenwu {
//! Golden coverage for TW 张南「奋武」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{CardKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TWFENWU: u32 = 2_254_911_276;
const TW_ZHANGNAN: u32 = 2_701_918_325;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twfenwu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("twfenwu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 361);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::DrawCards {
        player: owner,
        count: 2,
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWFENWU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    (tx, owner, target)
}

fn fire(tx: &mut Transaction, owner: PlayerId) -> Result<(), EffectError> {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )?;
    run_effects(tx, effects).map(|_| ())
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn twfenwu_keeps_target_first_cost_timing_and_basic_variant_damage() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let zhangnan = committed
        .character_index
        .iter()
        .find(|character| character.id == TW_ZHANGNAN)
        .unwrap();
    assert_eq!(zhangnan.skills, vec![TWFENWU]);

    let (mut cancel, owner, _) = setup(2_026_080_101);
    let hp = cancel.state().players.get(owner).unwrap().hp;
    assert_eq!(fire(&mut cancel, owner), Err(EffectError::ChoiceRequested));
    let optional = cancel.commit();
    // 当前实现：结束阶段触发直接打开“选目标”（可取消）；选中后失 1 体力并使用虚拟杀。
    let (state, accepted) = resolve(optional.state, owner, vec![2], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);

    let (mut damage, owner, target) = setup(2_026_080_102);
    let cards = damage.state().hand_cards(owner);
    assert!(cards.len() >= 2);
    damage.state_mut().cards.get_mut(cards[0]).unwrap().kind = CardKind::Shan;
    damage.state_mut().cards.get_mut(cards[1]).unwrap().kind = CardKind::Tao;
    dispatch_for_events(
        &mut damage,
        &[
            DomainEvent::CardUsed {
                player: owner,
                card: cards[0],
                targets: vec![],
            },
            DomainEvent::CardUsed {
                player: owner,
                card: cards[1],
                targets: vec![],
            },
        ],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(damage.state().turn.distinct_basic_card_variants(owner), 2);
    let owner_hp = damage.state().players.get(owner).unwrap().hp;
    let target_hp = damage.state().players.get(target).unwrap().hp;
    assert_eq!(fire(&mut damage, owner), Err(EffectError::ChoiceRequested));
    let optional = damage.commit();
    let (state, used) = resolve(optional.state, owner, vec![target.raw()], false, 3);
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 2);
}
}

mod original_tw_twfenxian {
//! Golden coverage for 幻黄盖「焚险」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWFENXIAN: u32 = 3_149_854_807;
const TWJUYAN: u32 = 2_964_355_463;
const HUAN_HUANGGAI: u32 = 3_838_174_457;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twfenxian.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    def: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> (GameState, [PlayerId; 3], engine_api::SkillId) {
    let package = content_compiler::compile_source_named("twfenxian-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 399);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players: [PlayerId; 3] = tx.state().players.alive_by_seat().try_into().unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(TWFENXIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in players {
        for card in tx.state().hand_cards(player) {
            tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
        }
    }
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == players[0] && skill.def.raw() == TWFENXIAN)
        .unwrap()
        .id;
    (tx.commit().state, players, skill)
}

fn submit(state: GameState, actor: PlayerId, kind: CommandKind, key: u64) -> GameState {
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "key {key}: {:?}",
        result.result
    );
    state
}

fn choose(state: GameState, actor: PlayerId, token: u32, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections: vec![token],
            cancelled: false,
        },
        key,
    )
}

#[test]
fn target_uses_field_card_as_duel_or_owner_uses_virtual_huogong() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed.manifest.rule_vm_abi_version >= 399);
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == HUAN_HUANGGAI)
            .unwrap()
            .skills,
        [TWFENXIAN, TWJUYAN]
    );

    let (state, [owner, actor, victim], skill) = setup(2_026_080_205);
    let mut tx = Transaction::begin(state);
    let field = inject(
        &mut tx,
        actor,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Club,
        99_901,
    );
    let victim_hp = tx.state().players.get(victim).unwrap().hp;
    let state = submit(
        tx.commit().state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![actor],
            card_targets: Vec::new(),
        },
        1,
    );
    let state = choose(state, actor, 0, 2);
    let state = choose(state, actor, field.raw(), 3);
    assert_eq!(state.resolution.pending_observer, actor.raw());
    let state = choose(state, actor, victim.raw(), 4);
    assert_eq!(state.players.get(victim).unwrap().hp, victim_hp - 1);

    let (state, [owner, target, _], skill) = setup(2_026_080_206);
    let mut tx = Transaction::begin(state);
    let shown = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Spade,
        99_902,
    );
    let matching = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        99_903,
    );
    let hp = tx.state().players.get(target).unwrap().hp;
    let state = submit(
        tx.commit().state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        5,
    );
    let state = choose(state, target, 1, 6);
    let state = choose(state, target, shown.raw(), 7);
    let state = choose(state, owner, matching.raw(), 8);
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}
}

mod original_tw_twfujian {
//! Golden coverage for TW 夏侯恩「负剑」.

use engine_api::{CardDefId, CardId, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, EquipSlot, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWFUJIAN: u32 = 2_940_701_334;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twfujian.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

#[test]
fn equips_one_weapon_and_loses_hp_only_when_it_leaves_outside_the_turn() {
    let package = content_compiler::compile_source_named("twfujian-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 366);
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == TWFUJIAN));

    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_080_106, "identity-standard").unwrap();
    // Roster-independent: the standard roster gained choice-bearing characters
    // (gongsunzan 义从 / old_re_lidian 询询+忘隙) whose booted presence changes
    // range/hand-limit/trigger answers; this test grants its own skills.
    let booted: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    if !booted.is_empty() {
        run_effects(&mut tx, booted).unwrap();
    }

    let players = tx.state().players.alive_by_seat();
    let (owner, other) = (players[0], players[1]);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let injected = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_366_001),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 1,
        weapon_range: 2,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(draw)
        .unwrap()
        .cards
        .push(injected);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWFUJIAN),
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::EquipWeaponFromDrawPile { player: owner }],
    )
    .unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let weapon = tx
        .state()
        .zones
        .get(equip)
        .unwrap()
        .cards
        .iter()
        .copied()
        .find(|card| {
            tx.state().cards.get(*card).unwrap().kind.equip_slot() == Some(EquipSlot::Weapon)
        })
        .unwrap();
    let hp = tx.state().players.get(owner).unwrap().hp;
    tx.state_mut().turn.current = other;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![weapon],
            to: discard,
            from: Some(equip),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
}
}

mod original_tw_twfuman {
//! Golden coverage for TW 马忠「抚蛮」.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::effective_card_kind;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWFUMAN: u32 = 2_838_266_052;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twfuman.skill");

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_394_000 + u32::from(rank)),
        kind,
        suit: Suit::Heart,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn submit(state: GameState, actor: PlayerId, kind: CommandKind, key: u64) -> GameState {
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    state
}

#[test]
fn gifted_physical_card_becomes_sha_and_damage_draws_two_after_resolution() {
    let package = content_compiler::compile_source_named("twfuman-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 394);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_214, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, user, target) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWFUMAN),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let gift = inject(tx.state_mut(), owner, CardKind::Tao, 7);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWFUMAN)
        .unwrap()
        .id;
    let state = submit(
        tx.commit().state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![user],
            card_targets: vec![gift],
        },
        1,
    );
    assert_eq!(
        effective_card_kind(&state, user, state.cards.get(gift).unwrap()),
        CardKind::Sha
    );
    let owner_hand = state.hand_count(owner);
    let target_hp = state.players.get(target).unwrap().hp;
    let mut state = state;
    state.turn.current = user;
    state.turn.phase = Phase::Play;
    let state = submit(
        state,
        user,
        CommandKind::PlayCard {
            card: gift,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        2,
    );
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert_eq!(state.hand_count(owner), owner_hand + 2);
    assert_eq!(
        effective_card_kind(&state, user, state.cards.get(gift).unwrap()),
        CardKind::Tao
    );
}
}

mod original_tw_twfupan {
//! Golden coverage for TW 呼厨泉「复叛」.

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TWFUPAN: u32 = 2_667_455_987;
const TW_HUCHUQUAN: u32 = 3_374_283_771;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twfupan.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("twfupan-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 362);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_080_103, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target, other) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWFUPAN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, target, other)
}

fn fire(tx: &mut Transaction, owner: PlayerId, target: PlayerId, amount: u32) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::DamageDealt {
            source: owner,
            target,
            amount,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(tx, effects),
        Err(EffectError::ChoiceRequested)
    );
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

fn accept_and_give(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    // 当前实现：触发后直接是交牌 SelectCards（无独立确认段）。
    let card = state.hand_cards(owner)[0];
    let (state, selected) = resolve(state, owner, vec![card.raw()], false, key);
    let target_choice = selected.choice.unwrap();
    assert_eq!(target_choice.kind, ChoiceKind::SelectTargets);
    assert!(target_choice
        .options
        .iter()
        .any(|option| option.token == target.raw()));
    resolve(state, owner, vec![target.raw()], false, key + 1)
}

#[test]
fn twfupan_draws_by_damage_and_tracks_each_recipient_through_the_damage_lock() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed.manifest.rule_vm_abi_version >= 362);
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == TW_HUCHUQUAN)
            .unwrap()
            .skills,
        [TWFUPAN]
    );

    let (mut tx, owner, target, other) = setup();
    let owner_hand = tx.state().hand_count(owner);
    let target_hand = tx.state().hand_count(target);
    fire(&mut tx, owner, target, 2);
    let optional = tx.commit();
    let (state, first) = accept_and_give(optional.state, owner, target, 1);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_hand + 3);
    assert_eq!(state.hand_count(target), target_hand + 1);

    let mut tx = Transaction::begin(state);
    fire(&mut tx, owner, other, 1);
    let optional = tx.commit();
    let hp = optional.state.players.get(target).unwrap().hp;
    let (state, second) = accept_and_give(optional.state, owner, target, 10);
    let confirm = second.choice.unwrap();
    assert_eq!(confirm.kind, ChoiceKind::Confirm);
    assert_eq!(state.players.get(target).unwrap().hp, hp);
    let (state, damaged) = resolve(state, owner, vec![1], false, 12);
    assert!(matches!(damaged.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);

    // 复叛造成的伤害会再次触发复叛；交牌为 forced（不可取消），完成它。
    let (state, nested) = accept_and_give(state, owner, other, 14);
    assert!(matches!(nested.result, CommandResult::Accepted { .. }));
    let mut tx = Transaction::begin(state);
    fire(&mut tx, owner, other, 1);
    let optional = tx.commit();
    let card = optional.state.hand_cards(owner)[0];
    let (_, selected) = resolve(optional.state, owner, vec![card.raw()], false, 20);
    let targets = selected.choice.unwrap();
    assert!(!targets
        .options
        .iter()
        .any(|option| option.token == target.raw()));
}
}

mod original_tw_twguoyi {
//! Golden coverage for TW 周处「果毅」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWGUOYI: u32 = 3_002_953_620;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twguoyi.skill");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_371_000 + u32::from(rank)),
        kind,
        suit: Suit::Diamond,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn play(state: GameState, source: PlayerId, card: CardId, target: PlayerId, key: u64) -> GameState {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: source,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn choosing_both_branches_replays_the_second_card_after_original_resolution() {
    let package = content_compiler::compile_source_named("twguoyi-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 371);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 2_026_080_112, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (source, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: source,
        skill_def: SkillDefId::new(TWGUOYI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = source;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().players.get_mut(source).unwrap().max_hp = 5;
    tx.state_mut().players.get_mut(source).unwrap().hp = 5;
    let tiesuo = inject(&mut tx, source, CardKind::TrickTiesuo, 1);
    let sha = inject(&mut tx, source, CardKind::Sha, 2);
    inject(&mut tx, source, CardKind::Tao, 3);
    inject(&mut tx, source, CardKind::Tao, 4);
    let discard = inject(&mut tx, target, CardKind::Tao, 5);
    inject(&mut tx, target, CardKind::Tao, 6);
    inject(&mut tx, target, CardKind::Tao, 7);
    inject(&mut tx, target, CardKind::Tao, 8);
    inject(&mut tx, target, CardKind::Tao, 9);
    let mut state = tx.commit().state;

    state = play(state, source, tiesuo, target, 1);
    // 当前实现：果毅触发后由目标直接选择分支（无独立 optional 确认段）。
    state = resolve(state, target, vec![0], 2); // choose hand-card lock
    assert!(state
        .players
        .get(target)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0));

    let hp = state.players.get(target).unwrap().hp;
    state = play(state, source, sha, target, 4);
    state = resolve(state, target, vec![1], 5); // choose discard branch
    assert_eq!(state.resolution.pending_card_replays.len(), 1);
    assert_eq!(
        state.resolution.pending_card_replays[0].kind,
        CardKind::Sha.tag()
    );
    assert_eq!(state.resolution.pending_card_replays[0].targets, [target]);
    assert_eq!(state.players.get(target).unwrap().hp, hp);
    state = resolve(state, target, vec![discard.raw()], 7);

    assert!(state.resolution.pending_card_replays.is_empty());
    assert_eq!(state.players.get(target).unwrap().hp, hp - 2);
    assert!(!state.hand_cards(target).contains(&discard));
    assert!(!state.has_pending_choice());
}
}

mod original_tw_twjianwei {
//! Golden coverage for TW 夏侯恩「剑威」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWJIANWEI: u32 = 3_685_532_230;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twjianwei.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_367_000 + u32::from(rank)),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: (kind == CardKind::Weapon).then_some(3).unwrap_or(0),
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn submit(state: GameState, actor: PlayerId, kind: CommandKind, key: u64) -> GameState {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections,
            cancelled: false,
        },
        key,
    )
}

#[test]
fn attack_range_changes_winner_and_both_prepare_branches_settle_exactly() {
    let package = content_compiler::compile_source_named("twjianwei-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 367);
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed.character_index.iter().any(|character| {
        character.id == 3_331_986_699 && character.skills == [2_940_701_334, TWJIANWEI]
    }));

    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_080_107, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, other) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWJIANWEI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let weapon = inject(&mut tx, owner, ZoneKind::Equip, CardKind::Weapon, 1);
    let owner_card = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 4);
    let other_card = inject(&mut tx, other, ZoneKind::Hand, CardKind::Shan, 6);
    let other_hand = inject(&mut tx, other, ZoneKind::Hand, CardKind::Tao, 2);
    let other_equip = inject(&mut tx, other, ZoneKind::Equip, CardKind::Armor, 3);
    let other_judge = inject(&mut tx, other, ZoneKind::Judge, CardKind::TrickLe, 5);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == TWJIANWEI)
        .unwrap()
        .id;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwjianwei {
                player: owner,
                actor: owner,
                target: PlayerId::NONE,
                card: CardId::NONE,
                winner: PlayerId::NONE,
                stage: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    state = choose(state, owner, vec![other.raw()], 1);
    state = choose(state, owner, vec![owner_card.raw()], 2);
    state = choose(state, other, vec![other_card.raw()], 3);
    state = choose(
        state,
        owner,
        vec![other_hand.raw(), other_equip.raw(), other_judge.raw()],
        4,
    );
    assert!([other_hand, other_equip, other_judge]
        .iter()
        .all(|card| state.hand_cards(owner).contains(card)));

    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::Prepare;
    let challenger = inject(&mut tx, other, ZoneKind::Hand, CardKind::Sha, 13);
    let owner_low = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Shan, 1);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwjianwei {
                player: owner,
                actor: other,
                target: PlayerId::NONE,
                card: CardId::NONE,
                winner: PlayerId::NONE,
                stage: 1,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    state = choose(state, other, vec![1], 5);
    state = choose(state, other, vec![challenger.raw()], 6);
    state = choose(state, owner, vec![owner_low.raw()], 7);
    assert!(state.hand_cards(other).contains(&weapon));
}
}

mod original_tw_twjiaohua {
//! Golden coverage for TW 李遗「教化」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TWJIAOHUA: u32 = 3_924_813_285;
const TW_LIWEI: u32 = 2_988_712_070;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twjiaohua.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn inject(tx: &mut Transaction, zone_kind: ZoneKind, owner: PlayerId, kind: CardKind) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(90_000),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::DiscardPile,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("twjiaohua-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 363);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_080_104, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let other = players[2];
    tx.state_mut().players.get_mut(owner).unwrap().hp = 4;
    tx.state_mut().players.get_mut(target).unwrap().hp = 1;
    tx.state_mut().players.get_mut(other).unwrap().hp = 3;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let removed = tx
        .state()
        .zones
        .find(ZoneKind::Removed, PlayerId::NONE)
        .unwrap();
    for kind in [ZoneKind::DrawPile, ZoneKind::DiscardPile] {
        let zone = tx.state().zones.find(kind, PlayerId::NONE).unwrap();
        let cards = tx.state().zones.get(zone).unwrap().cards.clone();
        if !cards.is_empty() {
            effects.push(EffectRequest::MoveCards {
                cards,
                to: removed,
                from: Some(zone),
            });
        }
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWJIAOHUA),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, target)
}

fn fire(tx: &mut Transaction, target: PlayerId, cards: Vec<CardId>) -> bool {
    // 教化当前为锁定自动执行（无确认 choice）：有触发计划即视为触发。
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::CardsDrawn {
            player: target,
            cards,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    if effects.is_empty() {
        return false;
    }
    run_effects(tx, effects).is_ok()
}


#[test]
fn twjiaohua_excludes_drawn_and_turn_used_categories_then_stops() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed.manifest.rule_vm_abi_version >= 363);
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == TW_LIWEI)
            .unwrap()
            .skills,
        [TWJIAOHUA]
    );

    let (mut tx, owner, target) = setup();
    let drawn_basic = inject(&mut tx, ZoneKind::Hand, target, CardKind::Shan);
    let discard_trick = inject(
        &mut tx,
        ZoneKind::DiscardPile,
        PlayerId::NONE,
        CardKind::TrickWuzhong,
    );
    assert!(fire(&mut tx, target, vec![drawn_basic]));
    let state = tx.commit().state;
    assert_eq!(
        state.cards.get(discard_trick).unwrap().zone,
        state.zones.find(ZoneKind::Hand, target).unwrap()
    );
    assert_eq!(
        state
            .turn
            .flag(turn_flag::twjiaohua_category_by_player(target)),
        0b010
    );

    let mut tx = Transaction::begin(state);
    let draw_basic = inject(&mut tx, ZoneKind::DrawPile, PlayerId::NONE, CardKind::Sha);
    let drawn_equip = inject(&mut tx, ZoneKind::Hand, target, CardKind::Armor);
    assert!(fire(&mut tx, target, vec![drawn_equip]));
    let state = tx.commit().state;
    assert_eq!(
        state.cards.get(draw_basic).unwrap().zone,
        state.zones.find(ZoneKind::Hand, target).unwrap()
    );
    assert_eq!(
        state
            .turn
            .flag(turn_flag::twjiaohua_category_by_player(target)),
        0b011
    );
    assert!(!state.has_pending_choice());
    assert!(state.resolution.optional_trigger.is_none());
    assert!(state.resolution.deferred_effects.is_empty());
    assert!(state.resolution.deferred_triggers.is_empty());

    let mut tx = Transaction::begin(state);
    let drawn_equip = inject(&mut tx, ZoneKind::Hand, target, CardKind::Armor);
    assert!(tx.state().cards.get(drawn_equip).unwrap().kind.is_equip());
    assert_eq!(
        tx.state()
            .turn
            .flag(turn_flag::twjiaohua_category_by_player(target)),
        0b011
    );
    assert!(!fire(&mut tx, target, vec![drawn_equip]));
}
}

mod original_tw_twjieyuan {
//! Golden coverage for TW「竭缘」(twjieyuan).
//! Outgoing damage_before: optional confirm, then damage +1 and a forced hand discard.
//! Incoming damage_before from another source: optional confirm, then damage -1 and discard.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWJIEYUAN: u32 = 4_287_464_553;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twjieyuan.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
    hand_card: CardId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("twjieyuan-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_93, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWJIEYUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = inject_hand(tx.state_mut(), owner, 170_301);
    Setup {
        state: tx.commit().state,
        owner,
        other,
        hand_card: hand,
    }
}

fn inject_hand(state: &mut GameState, player: PlayerId, def: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn compiled_source_and_npack_publish_both_damage_before_windows() {
    let compiled = content_compiler::compile_source_named("twjieyuan", SOURCE).unwrap();
    let mut filters: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == TWJIEYUAN)
        .map(|trigger| (trigger.event.as_str(), trigger.filter.as_str(), trigger.optional))
        .collect();
    filters.sort();
    assert_eq!(
        filters,
        vec![
            ("damage_before", "owner_is_target_other_source_alive", true),
            ("damage_before", "owner_source_other_damage_before", true),
        ]
    );

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == TWJIEYUAN && skill.name == "twjieyuan"));
    let pack_filters: Vec<_> = committed
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == TWJIEYUAN)
        .map(|trigger| trigger.filter.as_str())
        .collect();
    assert!(pack_filters.contains(&"owner_source_other_damage_before"));
    assert!(pack_filters.contains(&"owner_is_target_other_source_alive"));
}

#[test]
fn outgoing_damage_increases_then_discards_one_hand_card() {
    let setup = setup();
    let other_hp = setup.state.players.get(setup.other).unwrap().hp;
    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: setup.owner,
                target: setup.other,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, confirmed) = resolve(tx.commit().state, setup.owner, vec![1], 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    let before = state.hand_count(setup.owner);
    let (state, discarded) = resolve(state, setup.owner, vec![setup.hand_card.raw()], 2);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }), "{:?}", discarded.result);
    assert_eq!(state.hand_count(setup.owner), before - 1);
    assert_eq!(
        state.players.get(setup.other).unwrap().hp,
        other_hp - 2,
        "outgoing branch should deal 2; events={:?}",
        discarded.events
    );
}

#[test]
fn incoming_damage_decreases_to_zero_when_base_is_one() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: setup.other,
                target: setup.owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, confirmed) = resolve(tx.commit().state, setup.owner, vec![1], 3);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    let owner_hp = state.players.get(setup.owner).unwrap().hp;
    let (state, discarded) = resolve(state, setup.owner, vec![setup.hand_card.raw()], 4);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }), "{:?}", discarded.result);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, owner_hp);
}
}

mod original_tw_twkujian {
//! Golden coverage for TW 阎象「苦谏」.

use engine_api::{
    CardDefId, CardId, CardMoveReason, ChoiceId, Command, CommandKind, CommandResult,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TWKUJIAN: u32 = 2_903_587_563;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twkujian.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_364_000 + u32::from(rank)),
        kind: CardKind::Sha,
        suit: Suit::Heart,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn submit(state: GameState, actor: PlayerId, kind: CommandKind, key: u64) -> GameState {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn tracked_cards_reward_use_response_and_force_discards_for_other_losses() {
    let package = content_compiler::compile_source_named("twkujian-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 364);
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == TWKUJIAN));

    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_080_104, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWKUJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let owner_cost = inject(&mut tx, owner, 1);
    let target_cost = inject(&mut tx, target, 2);
    let tracked = [
        inject(&mut tx, owner, 3),
        inject(&mut tx, owner, 4),
        inject(&mut tx, owner, 5),
    ];
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == TWKUJIAN)
        .unwrap()
        .id;
    let mut state = tx.commit().state;

    state = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: tracked.to_vec(),
        },
        1,
    );
    assert!(tracked
        .iter()
        .all(|card| state.hand_cards(target).contains(card)));

    let owner_before = state.hand_count(owner);
    let target_before = state.hand_count(target);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let target_hand = state.zones.find(ZoneKind::Hand, target).unwrap();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCardsWithReason {
            cards: vec![tracked[0]],
            to: discard,
            from: Some(target_hand),
            reason: CardMoveReason::UseOrRespond,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), owner_before + 1);
    assert_eq!(tx.state().hand_count(target), target_before);

    let owner_after_reward = tx.state().hand_count(owner);
    let target_after_reward = tx.state().hand_count(target);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[engine_api::DomainEvent::CardResponded {
                player: target,
                respond_to: owner,
                card: tracked[0],
                response_kind: CardKind::Sha.tag(),
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap(),
        Vec::<EffectRequest>::new(),
        "the movement and CardResponded events must reward the same card only once",
    );
    assert_eq!(tx.state().hand_count(owner), owner_after_reward);
    assert_eq!(tx.state().hand_count(target), target_after_reward);

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: vec![tracked[1]],
                to: discard,
                from: Some(target_hand),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    state = submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections: vec![owner_cost.raw()],
            cancelled: false,
        },
        2,
    );
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    state = submit(
        state,
        target,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections: vec![target_cost.raw()],
            cancelled: false,
        },
        3,
    );
    assert!(!state.hand_cards(owner).contains(&owner_cost));
    assert!(!state.hand_cards(target).contains(&target_cost));
    assert!(state.hand_cards(target).contains(&tracked[2]));
}
}

mod original_tw_twluanchou {
//! Golden coverage for TW 桥公「乱筹/共患」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWLUANCHOU: u32 = 2_306_392_424;
const TWGONGHUAN: u32 = 3_503_235_693;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twluanchou.skill");

#[test]
fn resets_two_holders_and_redirects_only_once_with_original_damage_fields() {
    let package = content_compiler::compile_source_named("twluanchou-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 374);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_115, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, victim, source) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWLUANCHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    let main = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWLUANCHOU)
        .unwrap()
        .id;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwluanchou {
            player: owner,
            helper_skill_def: TWGONGHUAN,
            action: 0,
            targets: vec![owner, victim],
            source: PlayerId::NONE,
            target: PlayerId::NONE,
            amount: 0,
            nature: 0,
            card: engine_api::CardId::NONE,
            card_kind: 0,
            card_flags: 0,
            skill: main,
        }],
    )
    .unwrap();
    let holders: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .filter(|skill| skill.def.raw() == TWGONGHUAN)
        .map(|skill| skill.owner)
        .collect();
    assert_eq!(holders, [owner, victim]);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let victim_hp = tx.state().players.get(victim).unwrap().hp;
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target: victim,
            amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
            card: engine_api::CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    );
    // 共患当前为 filter 门控的自动重定向（无确认 choice）。
    assert!(result.is_ok());
    let state = tx.commit().state;
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 2);
    assert_eq!(state.players.get(victim).unwrap().hp, victim_hp);
    assert!(!state.has_pending_choice());

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target: victim,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::THUNDER,
            card: engine_api::CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(state.players.get(victim).unwrap().hp, victim_hp - 1);
    assert!(!state.has_pending_choice());
}
}

mod original_tw_twqingkou {
//! Golden coverage for TW 冯习「清口」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, CardKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TWQINGKOU: u32 = 2_492_727_906;
const TW_FENGXI: u32 = 3_711_625_118;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twqingkou.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn setup(seed: u64, target_has_sha: bool) -> (Transaction, PlayerId, PlayerId, Option<u32>) {
    let package = content_compiler::compile_source_named("twqingkou-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 396);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWQINGKOU),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in players {
        for card in tx.state().hand_cards(player) {
            tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
        }
    }
    let response = target_has_sha.then(|| {
        let card = tx.state().hand_cards(target)[0];
        tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Sha;
        card.raw()
    });
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    (tx, owner, target, response)
}

fn fire(tx: &mut Transaction, owner: PlayerId) -> Result<(), EffectError> {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )?;
    run_effects(tx, effects).map(|_| ())
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn duel_winner_draws_and_only_owner_win_skips_judge_and_discard() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == TW_FENGXI)
            .unwrap()
            .skills,
        vec![TWQINGKOU]
    );

    let (mut owner_wins, owner, target, _) = setup(2_026_080_201, false);
    let owner_hand = owner_wins.state().hand_count(owner);
    let target_hp = owner_wins.state().players.get(target).unwrap().hp;
    assert_eq!(
        fire(&mut owner_wins, owner),
        Err(EffectError::ChoiceRequested)
    );
    // 当前实现：清口触发后直接打开虚拟决斗目标选择（无独立 optional 确认段）。
    let (state, used) = resolve(owner_wins.commit().state, owner, vec![target.raw()], 1);
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert_eq!(state.hand_count(owner), owner_hand + 1);
    assert!(state.pending_judge_skips.contains(&owner));
    assert_ne!(state.turn.flag(turn_flag::SKIP_DISCARD), 0);

    let (mut target_wins, owner, target, response) = setup(2_026_080_202, true);
    let owner_hp = target_wins.state().players.get(owner).unwrap().hp;
    let target_hand = target_wins.state().hand_count(target);
    assert_eq!(
        fire(&mut target_wins, owner),
        Err(EffectError::ChoiceRequested)
    );
    let (state, _) = resolve(target_wins.commit().state, owner, vec![1], 3);
    let (state, _) = resolve(state, owner, vec![target.raw()], 4);
    let (state, answered) = resolve(state, target, vec![response.unwrap()], 5);
    assert!(matches!(answered.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
    assert_eq!(state.hand_count(target), target_hand);
    assert!(!state.pending_judge_skips.contains(&owner));
    assert_eq!(state.turn.flag(turn_flag::SKIP_DISCARD), 0);
}
}

mod original_tw_twrangyi {
//! Golden coverage for TW 马良「让驿」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWRANGYI: u32 = 2_884_981_024;
const TW_MALIANG: u32 = 2_410_358_039;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twrangyi.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_359_000 + u32::from(rank)),
        kind,
        suit: Suit::Heart,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(
    seed: u64,
) -> (
    GameState,
    PlayerId,
    PlayerId,
    engine_api::SkillId,
    [CardId; 2],
) {
    let package = content_compiler::compile_source_named("twrangyi-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 359);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWRANGYI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWRANGYI)
        .unwrap()
        .id;
    let cards = [
        inject(&mut tx, owner, CardKind::Weapon, 3),
        inject(&mut tx, owner, CardKind::Tao, 4),
    ];
    (tx.commit().state, owner, target, skill, cards)
}

fn submit(state: GameState, actor: PlayerId, kind: CommandKind, key: u64) -> GameState {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "key {key}: {:?}",
        result.result,
    );
    state
}

#[test]
fn chosen_card_is_used_and_the_rest_returns_or_decline_deals_damage() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed.manifest.rule_vm_abi_version >= 359);
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == TW_MALIANG)
            .unwrap()
            .skills,
        [TWRANGYI, 3_304_922_519]
    );

    let (mut state, owner, target, skill, cards) = setup(20_260_801);
    state = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        1,
    );
    assert_eq!(state.hand_count(owner), 0, "cards were not gifted");
    assert_eq!(state.hand_count(target), 2, "target did not receive cards");
    assert_ne!(state.resolution.pending_choice_id, 0);
    assert_eq!(state.resolution.pending_observer, target.raw());
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    state = submit(
        state,
        target,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections: vec![cards[0].raw()],
            cancelled: false,
        },
        2,
    );
    assert!(state.hand_cards(owner).contains(&cards[1]));
    assert!(!state.hand_cards(target).contains(&cards[0]));
    let equip = state.zones.find(ZoneKind::Equip, target).unwrap();
    assert!(state.zones.get(equip).unwrap().cards.contains(&cards[0]));

    let (mut state, owner, target, skill, cards) = setup(20_260_802);
    let hp = state.players.get(target).unwrap().hp;
    state = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        3,
    );
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    state = submit(
        state,
        target,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections: Vec::new(),
            cancelled: true,
        },
        4,
    );
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert!(cards
        .iter()
        .all(|card| state.hand_cards(target).contains(card)));
}
}

mod original_tw_twruilian {
//! Golden coverage for TW 阎象「睿敛」.

use engine_api::{
    CardDefId, CardId, CardMoveReason, ChoiceId, Command, CommandKind, CommandResult,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWRUILIAN: u32 = 3_419_838_857;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twruilian.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_365_000 + u32::from(rank)),
        kind,
        suit: Suit::Heart,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn submit(state: GameState, actor: PlayerId, kind: CommandKind, key: u64) -> GameState {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn tracks_discard_categories_then_gains_for_target_and_owner() {
    let package = content_compiler::compile_source_named("twruilian-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 365);
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .character_index
        .iter()
        .any(|character| character.id == 2_630_677_433
            && character.skills == [2_903_587_563, TWRUILIAN]));

    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_080_105, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWRUILIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.round = 1;
    tx.state_mut().turn.current = target;
    let basic = inject(&mut tx, target, CardKind::Sha, 1);
    let trick = inject(&mut tx, target, CardKind::TrickWuzhong, 2);
    let spare_basic = inject(&mut tx, owner, CardKind::Tao, 3);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == TWRUILIAN)
        .unwrap()
        .id;

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwruilian {
                player: owner,
                target: PlayerId::NONE,
                event_cards: Vec::new(),
                action: 0,
                category: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    state = submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections: vec![target.raw()],
            cancelled: false,
        },
        1,
    );

    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let target_hand = state.zones.find(ZoneKind::Hand, target).unwrap();
    let owner_hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::MoveCardsWithReason {
                cards: vec![basic, trick],
                to: discard,
                from: Some(target_hand),
                reason: CardMoveReason::Discard,
            },
            EffectRequest::MoveCards {
                cards: vec![spare_basic],
                to: discard,
                from: Some(owner_hand),
            },
        ],
    )
    .unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwruilian {
                player: owner,
                target,
                event_cards: Vec::new(),
                action: 2,
                category: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    state = submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections: vec![1],
            cancelled: false,
        },
        2,
    );
    assert!(state.hand_cards(target).contains(&spare_basic));
    assert!(state.hand_cards(owner).contains(&basic));
    assert!(state.zones.get(discard).unwrap().cards.contains(&trick));
}
}

mod original_tw_twshiji {
//! Golden coverage for TW「势击」(twshiji): prepare-phase draw 2.

use engine_api::{DomainEvent, PlayerId, SkillDefId, TurnPhaseTag};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TWSHIJI: u32 = 2_278_923_284;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twshiji.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn setup() -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("twshiji-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_82, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let owner = tx.state().turn.current;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWSHIJI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner)
}

#[test]
fn compiled_source_and_npack_publish_prepare_draw() {
    let compiled = content_compiler::compile_source_named("twshiji", SOURCE).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == TWSHIJI)
        .unwrap();
    assert_eq!(trigger.event, "phase_prepare");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == TWSHIJI && trigger.event == "phase_prepare"));
}

#[test]
fn prepare_draws_two() {
    let (state, owner) = setup();
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 2);
}
}

mod original_tw_twtanfeng {
//! Golden coverage for TW 夏侯尚「探锋」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWTANFENG: u32 = 4_187_893_298;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twtanfeng.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, zone_kind: ZoneKind, kind: CardKind) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_368_001),
        kind,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn arbitrary_card_sha_and_cancelled_fire_branch_both_resolve() {
    let package = content_compiler::compile_source_named("twtanfeng-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 368);
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .character_index
        .iter()
        .any(|character| { character.id == 3_609_594_146 && character.skills == [TWTANFENG] }));

    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_080_108, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWTANFENG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let discarded = inject(&mut tx, target, ZoneKind::Judge, CardKind::TrickLe);
    let converted = inject(&mut tx, target, ZoneKind::Hand, CardKind::Tao);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == TWTANFENG)
        .unwrap()
        .id;
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwtanfeng {
                player: owner,
                target: PlayerId::NONE,
                card: CardId::NONE,
                stage: 0,
                phase: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    state = resolve(state, owner, vec![target.raw()], false, 1);
    state = resolve(state, owner, vec![discarded.raw()], false, 2);
    state = resolve(state, target, vec![converted.raw()], false, 3);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);

    let mut tx = Transaction::begin(state);
    let discarded = inject(&mut tx, target, ZoneKind::Judge, CardKind::TrickLe);
    let _available_sha_source = inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan);
    let target_hp = tx.state().players.get(target).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwtanfeng {
                player: owner,
                target: PlayerId::NONE,
                card: CardId::NONE,
                stage: 0,
                phase: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    state = resolve(state, owner, vec![target.raw()], false, 4);
    state = resolve(state, owner, vec![discarded.raw()], false, 5);
    state = resolve(state, target, Vec::new(), true, 6);
    state = resolve(state, target, vec![Phase::Draw.to_u8() as u32], false, 7);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert!(state.turn.skip_draw());
}
}

mod original_tw_twtongce {
//! 第一百五十一批：tw 幻法正「通策」（twtongce）零新原语重写。
//! 上游：游戏开始时，你获得〖遗计〗，你可令一名其他角色交给你一张牌并获得〖遗计〗。
//! 实现：game_started 触发 + add_skill(self) + choose_other_add_skill（ABI 675）。
//! 近似：交牌臂省略；被授〖遗计〗本体当前为诚实 noop（授予关系真实）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TWTONGCE: u32 = 2_501_855_754;
const TWYISHI: u32 = 4_055_710_510;
const TWTONGCE_SRC: &str =
    include_str!("../../../content/original/ported/tw/twtongce.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn bootstrap() -> (GameState, PlayerId, PlayerId) {
    let compiled =
        content_compiler::compile_source_named("twtongce", TWTONGCE_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(TWTONGCE), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWTONGCE),
    });
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    (state, owner, other)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "resolve rejected: {:?}",
        result.result
    );
    state
}

fn has_skill_def(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

/// 派发开局事件并完成通策授予（choose_other_add_skill 为强制选择，无取消路径）。
/// 返回授予完成后的状态。
fn start_game_and_grant(state: GameState, owner: PlayerId, other: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    );
    // forced 触发：dispatch 直接返回效果请求（Ok）；choose_other_add_skill 在
    // run_effects 执行中打开目标选择。
    let effects = opened.unwrap_or_else(|err| panic!("unexpected error: {err:?}"));
    match run_effects(&mut tx, effects) {
        Ok(_) => panic!("expected the target-selection choice to open"),
        Err(EffectError::ChoiceRequested) => {}
        Err(err) => panic!("unexpected error: {err:?}"),
    }
    let state = tx.commit().state;
    assert!(has_skill_def(&state, owner, TWYISHI), "owner must gain 遗计");
    resolve(state, owner, vec![other.raw()], false, 10)
}

#[test]
fn tongce_grants_yishi_to_owner_and_chosen_other() {
    let (state, owner, other) = bootstrap();
    let state = start_game_and_grant(state, owner, other);
    assert!(
        has_skill_def(&state, other, TWYISHI),
        "chosen other must gain 遗计"
    );
    assert!(!state.has_pending_choice(), "resolution expected to finish");
}


#[test]
fn package_publishes_twtongce_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == TWTONGCE
            && trigger.event == "game_started"
            && trigger.filter == "owner_alive"
            && !trigger.optional
    }));
    // 通策不再保留诚实 noop 的 hand_limit +0 modifier。
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == TWTONGCE && modifier.target == "hand_limit"));
}
}

mod original_tw_twxiawei {
//! Golden coverage for TW 桥蕤「狭威/穷技」.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWXIAWEI: u32 = 3_007_668_730;
const TWQIONGJI: u32 = 2_723_606_029;
const WANGXING: u32 = 3_476_660_685;
const TWQIONGJI_DRAW: u32 = 3_469_371_140;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twxiawei.skill");

fn inject_draw(tx: &mut Transaction, kind: CardKind, rank: u8) -> CardId {
    let zone = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_375_000 + u32::from(rank)),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn wei_cards_are_playable_draw_once_and_empty_pile_increases_damage() {
    let package = content_compiler::compile_source_named("twxiawei-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 375);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 2_026_080_116, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    for skill_def in [TWXIAWEI, TWQIONGJI, TWQIONGJI_DRAW] {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    let xiawei = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWXIAWEI)
        .unwrap()
        .id;
    let sha = inject_draw(&mut tx, CardKind::Sha, 1);
    let spare = inject_draw(&mut tx, CardKind::ShaFire, 2);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwxiawei {
            player: owner,
            wangxing_skill_def: WANGXING,
            action: 0,
            stage: 0,
            skill: xiawei,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().skill_pile_cards(owner, TWXIAWEI), [spare, sha]);
    assert!(tx.state().playable_hand_cards(owner).contains(&sha));
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let owner_hand = tx.state().hand_count(owner);
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let mut state = tx.commit().state;

    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state = next;
    assert_eq!(state.hand_count(owner), owner_hand + 1);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwxiawei {
            player: owner,
            wangxing_skill_def: WANGXING,
            action: 1,
            stage: 0,
            skill: xiawei,
        }],
    )
    .unwrap();
    assert!(tx.state().skill_pile_cards(owner, TWXIAWEI).is_empty());
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: target,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, owner_hp - 2);

    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwxiawei {
            player: owner,
            wangxing_skill_def: WANGXING,
            action: 2,
            stage: 2,
            skill: xiawei,
        }],
    )
    .unwrap();
    let wangxing = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == WANGXING)
        .unwrap()
        .id;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwxiawei {
            player: owner,
            wangxing_skill_def: WANGXING,
            action: 4,
            stage: 0,
            skill: wangxing,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, max_hp - 1);
    assert!(tx.state().skills.get(wangxing).is_none());
}
}

mod original_tw_twxingluan {
//! Golden coverage for TW 樊稠「兴乱」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWXINGLUAN: u32 = 3_047_451_649;
const TW_FANCHOU: u32 = 3_054_071_833;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twxingluan.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn inject(tx: &mut Transaction, kind: CardKind, def: u32) -> CardId {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(draw).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, output) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn distributes_one_category_with_cap_then_applies_owner_share_threshold() {
    let compiled = content_compiler::compile_source_named("twxingluan-test", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 491);
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 733);
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == TW_FANCHOU)
            .unwrap()
            .skills,
        [TWXINGLUAN]
    );

    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_208, "identity-standard").unwrap();
    let players: [PlayerId; 3] = tx.state().players.alive_by_seat().try_into().unwrap();
    let removed = tx
        .state()
        .zones
        .find(ZoneKind::Removed, PlayerId::NONE)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for kind in [ZoneKind::DrawPile, ZoneKind::DiscardPile] {
        let zone = tx.state().zones.find(kind, PlayerId::NONE).unwrap();
        let cards = tx.state().zones.get(zone).unwrap().cards.clone();
        if !cards.is_empty() {
            effects.push(EffectRequest::MoveCards {
                cards,
                to: removed,
                from: Some(zone),
            });
        }
    }
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(TWXINGLUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let basic = [
        inject(&mut tx, CardKind::Sha, 99_101),
        inject(&mut tx, CardKind::Shan, 99_102),
        inject(&mut tx, CardKind::Tao, 99_103),
        inject(&mut tx, CardKind::Jiu, 99_104),
    ];
    let trick = inject(&mut tx, CardKind::TrickJuedou, 99_105);
    let equip = inject(&mut tx, CardKind::Weapon, 99_106);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == players[0] && skill.def.raw() == TWXINGLUAN)
        .unwrap()
        .id;
    let hp: Vec<_> = players
        .iter()
        .map(|player| tx.state().players.get(*player).unwrap().hp)
        .collect();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![
                EffectRequest::RevealChooseCategoryDistributeSelfThresholdHpLoss {
                    player: players[0],
                    reveal_count: 6,
                    recipient_cap: 3,
                    skill,
                }
            ],
        ),
        Err(EffectError::ChoiceRequested)
    );

    let state = choose(tx.commit().state, players[0], vec![1], 1);
    let state = choose(state, players[0], vec![basic[0].raw()], 2);
    let state = choose(state, players[0], vec![players[0].raw()], 3);
    let state = choose(state, players[0], vec![basic[1].raw(), basic[2].raw()], 4);
    let state = choose(state, players[0], vec![players[1].raw()], 5);
    let state = choose(state, players[0], vec![basic[3].raw()], 6);
    let state = choose(state, players[0], vec![players[2].raw()], 7);

    assert!(!state.has_pending_choice());
    for (index, player) in players.iter().enumerate() {
        assert_eq!(state.players.get(*player).unwrap().hp, hp[index] - 1);
    }
    assert!(state.hand_cards(players[0]).contains(&basic[0]));
    assert!(state.hand_cards(players[1]).contains(&basic[1]));
    assert!(state.hand_cards(players[1]).contains(&basic[2]));
    assert!(state.hand_cards(players[2]).contains(&basic[3]));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(trick).unwrap().zone, discard);
    assert_eq!(state.cards.get(equip).unwrap().zone, discard);
}
}

mod original_tw_twyizhu {
//! Golden coverage for TW 桥公「遗珠」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWYIZHU: u32 = 2_650_180_014;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twyizhu.skill");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_373_000 + u32::from(rank)),
        kind,
        suit: Suit::Heart,
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
fn planted_card_used_by_other_can_add_target_then_is_untracked_and_draws() {
    let package = content_compiler::compile_source_named("twyizhu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 373);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_114, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, user, target) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWYIZHU),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWYIZHU)
        .unwrap()
        .id;
    let sha = inject(&mut tx, owner, CardKind::Sha, 1);
    let pearl = inject(&mut tx, owner, CardKind::Tao, 2);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwyizhu {
            player: owner,
            source: PlayerId::NONE,
            card: CardId::NONE,
            action: 0,
            stage: 1,
            event_cards: Vec::new(),
            skill,
        }],
    )
    .unwrap();
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    assert!(tx.state().zones.get(draw).unwrap().cards.contains(&sha));
    let user_hand = tx.state().zones.find(ZoneKind::Hand, user).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![sha],
            to: user_hand,
            from: Some(draw),
        }],
    )
    .unwrap();
    tx.state_mut().turn.current = user;
    tx.state_mut().turn.phase = Phase::Play;
    let mut state = tx.commit().state;
    let owner_hand_before = state.hand_count(owner);
    let owner_hp = state.players.get(owner).unwrap().hp;
    let target_hp = state.players.get(target).unwrap().hp;

    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor: user,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state = next;
    assert_eq!(result.choice.as_ref().unwrap().observer, owner);
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![owner.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));

    // Drive the 杀: both targets decline 闪, then the damage commits.
    let mut state = state;
    let mut key = 3u64;
    while state.has_pending_choice() {
        let (next, r) = apply_command(
            state.clone(),
            Command {
                actor: PlayerId::new(state.resolution.pending_observer),
                expected_revision: ExpectedRevision(state.revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(matches!(r.result, CommandResult::Accepted { .. }), "{:?}", r.result);
        state = next;
        key += 1;
    }
    assert_eq!(state.hand_count(owner), owner_hand_before + 1);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    let instance = state.skills.get(skill).unwrap();
    assert_eq!(&instance.state_blob[..4], b"KJN1");
    assert_eq!(&instance.state_blob[4..], &pearl.raw().to_le_bytes());
    assert!(!state.has_pending_choice());

    let mut tx = Transaction::begin(state);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![pearl],
            to: discard,
            from: Some(draw),
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(state.skills.get(skill).unwrap().state_blob, b"KJN1");
}
}

mod original_tw_twyuyan {
//! Golden coverage for TW 宗预「御严」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWYUYAN: u32 = 3_341_156_482;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twyuyan.skill");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_370_000 + u32::from(rank)),
        kind,
        suit: Suit::Diamond,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(
    gift_rank: Option<u8>,
    seed: u64,
) -> (GameState, PlayerId, PlayerId, CardId, Option<CardId>) {
    let package = content_compiler::compile_source_named("twyuyan-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 370);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (source, owner) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWYUYAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = source;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().players.get_mut(source).unwrap().hp = 4;
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    let sha = inject(&mut tx, source, CardKind::Sha, 5);
    let gift = gift_rank.map(|rank| inject(&mut tx, source, CardKind::Tao, rank));
    (tx.commit().state, source, owner, sha, gift)
}

fn play(state: GameState, source: PlayerId, owner: PlayerId, sha: CardId, key: u64) -> GameState {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: source,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![owner],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn higher_rank_gift_keeps_target_and_no_eligible_card_cancels_it() {
    let (state, source, owner, sha, gift) = setup(Some(8), 2_026_080_110);
    let hp = state.players.get(owner).unwrap().hp;
    let state = play(state, source, owner, sha, 1);
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let gift = gift.unwrap();
    let (state, result) = apply_command(
        state,
        Command {
            actor: source,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![gift.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(owner).contains(&gift));
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);

    let (state, source, owner, sha, _) = setup(Some(2), 2_026_080_111);
    let hp = state.players.get(owner).unwrap().hp;
    let state = play(state, source, owner, sha, 3);
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    assert!(!state.has_pending_choice());
}
}

mod original_tw_twzhibian {
//! Golden coverage for TW 宗预「直辩」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWZHIBIAN: u32 = 4_091_292_152;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twzhibian.skill");

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    rank: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_369_000 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn win_backwater_moves_corresponding_zone_and_tie_or_loss_loses_hp() {
    let package = content_compiler::compile_source_named("twzhibian-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 369);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_080_109, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWZHIBIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    let owner_high = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 13);
    let cost = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Armor, 2);
    let target_low = inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan, 1);
    let judge = inject(&mut tx, target, ZoneKind::Judge, CardKind::TrickLe, 6);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == TWZHIBIAN)
        .unwrap()
        .id;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwzhibian {
                player: owner,
                target: PlayerId::NONE,
                winner: PlayerId::NONE,
                stage: 0,
                skill
            }]
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    state = choose(state, owner, vec![target.raw()], 1);
    state = choose(state, owner, vec![owner_high.raw()], 2);
    state = choose(state, target, vec![target_low.raw()], 3);
    state = choose(state, owner, vec![3], 4);
    state = choose(state, owner, vec![cost.raw()], 5);
    state = choose(state, owner, vec![judge.raw()], 6);
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert!(state
        .zones
        .get(state.zones.find(ZoneKind::Judge, owner).unwrap())
        .unwrap()
        .cards
        .contains(&judge));

    let mut tx = Transaction::begin(state);
    let owner_low = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 1);
    let target_high = inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan, 13);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwzhibian {
                player: owner,
                target: PlayerId::NONE,
                winner: PlayerId::NONE,
                stage: 0,
                skill
            }]
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    state = choose(state, owner, vec![target.raw()], 7);
    state = choose(state, owner, vec![owner_low.raw()], 8);
    state = choose(state, target, vec![target_high.raw()], 9);
    assert_eq!(state.players.get(owner).unwrap().hp, 2);
}
}

mod original_tw_wangcan {
//! Golden coverage for TW 王粲「应机/商和」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWSHANGHE: u32 = 4_227_896_164;
const TWYINGJI: u32 = 2_583_490_638;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twwangcan.skill");

fn shanghe_case(kind: CardKind, expected_hp: i32, key: u64) {
    let package = content_compiler::compile_source_named("tw-wangcan-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 385);
    assert_eq!(package.view_as_index.len(), 2);
    assert!(package
        .view_as_index
        .iter()
        .all(|rule| { rule.cost.as_deref() == Some("draw_one_outside_turn_empty_hand") }));

    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 2_026_080_126 + key, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, giver) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWSHANGHE),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    let hand = tx.state().zones.find(ZoneKind::Hand, giver).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_860 + key as u32),
        kind,
        suit: Suit::Diamond,
        rank: 9,
        weapon_range: 0,
        zone: hand,
        owner: giver,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
    owner_state.hp = 0;
    owner_state.dying = true;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwshanghe {
                player: owner,
                stage: 0,
                remaining: Vec::new(),
                got_jiu: false,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    assert_eq!(state.resolution.pending_observer, giver.raw());
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: giver,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![card.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, expected_hp);
    assert_eq!(
        state.cards.get(card).unwrap().zone,
        state.zones.find(ZoneKind::Hand, owner).unwrap()
    );
}

#[test]
fn shanghe_recovers_without_jiu_and_does_not_recover_with_jiu() {
    shanghe_case(CardKind::Sha, 1, 1);
    shanghe_case(CardKind::Jiu, 0, 2);
}

#[test]
fn yingji_draws_then_supplies_a_source_free_shan_outside_turn() {
    let package = content_compiler::compile_source_named("tw-wangcan-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 2_026_080_129, "identity-standard").unwrap();
    let actor = tx.state().turn.current;
    let responder = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != actor)
        .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let responder_hand = tx.state().zones.find(ZoneKind::Hand, responder).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, engine_api::PlayerId::NONE)
        .unwrap();
    let old_hand = tx.state().hand_cards(responder);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::MoveCards {
                cards: old_hand,
                to: discard,
                from: Some(responder_hand),
            },
            EffectRequest::AddSkill {
                player: responder,
                skill_def: SkillDefId::new(TWYINGJI),
            },
        ],
    )
    .unwrap();
    let actor_hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_870),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: actor_hand,
        owner: actor,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(actor_hand)
        .unwrap()
        .cards
        .push(sha);
    let hp = tx.state().players.get(responder).unwrap().hp;
    let (state, offered) = apply_command(
        tx.commit().state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![responder],
                card_targets: vec![],
            },
        },
    );
    let choice = offered.choice.unwrap();
    assert_eq!(choice.observer, responder);
    let token = choice.options[0].token;
    let (state, resolved) = apply_command(
        state,
        Command {
            actor: responder,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(21),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![token],
                cancelled: false,
            },
        },
    );
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(responder).unwrap().hp, hp);
    assert_eq!(state.hand_count(responder), 1);
}
}

mod original_tw_wangchang {
//! Golden coverage for TW 王昶「开济/慑叛」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWKAIJI: u32 = 3_117_219_235;
const TWSHEPAN: u32 = 3_217_560_878;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twwangchang.skill");

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    state
}

fn setup(
    skill_def: u32,
    seed: u64,
) -> (
    Transaction,
    PlayerId,
    PlayerId,
    PlayerId,
    engine_api::SkillId,
) {
    let package = content_compiler::compile_source_named("tw-wangchang-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 386);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    (tx, owner, first, second, skill)
}

#[test]
fn kaiji_scales_and_shepan_refunds_or_places_on_top() {
    let (mut tx, owner, first, second, kaiji) = setup(TWKAIJI, 2_026_080_201);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::EnterDying { player: first },
            EffectRequest::EnterDying { player: second },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().lifetime_dying_players.len(), 2);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top = *tx.state().zones.get(draw).unwrap().cards.last().unwrap();
    let top_state = tx.state_mut().cards.get_mut(top).unwrap();
    top_state.def = engine_api::CardDefId::NONE;
    top_state.kind = CardKind::Weapon;
    let first_hand = tx.state().hand_count(first);
    let second_hand = tx.state().hand_count(second);
    let owner_hand = tx.state().hand_count(owner);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwwangchang {
                player: owner,
                target: PlayerId::NONE,
                card: engine_api::CardId::NONE,
                selected_card: engine_api::CardId::NONE,
                action: 0,
                stage: 0,
                targets: Vec::new(),
                skill: kaiji,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![first.raw(), second.raw()], 1);
    assert_eq!(state.hand_count(first), first_hand + 1);
    assert_eq!(state.hand_count(second), second_hand + 1);
    assert_eq!(state.hand_count(owner), owner_hand + 1);

    let (mut tx, owner, target, _, shepan) = setup(TWSHEPAN, 2_026_080_202);
    let owner_hand_zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let target_hand = tx.state().hand_count(target);
    let owner_hand = tx.state().hand_count(owner);
    if owner_hand >= target_hand {
        let discard_count = owner_hand.saturating_sub(target_hand).saturating_add(1) as usize;
        let cards = tx.state().hand_cards(owner)[..discard_count].to_vec();
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards,
                to: discard,
                from: Some(owner_hand_zone),
            }],
        )
        .unwrap();
    } else {
        run_effects(
            &mut tx,
            vec![EffectRequest::DrawCards {
                player: owner,
                count: (target_hand - 1 - owner_hand) as u32,
            }],
        )
        .unwrap();
    }
    assert_eq!(tx.state().hand_count(owner) + 1, target_hand);
    tx.state_mut()
        .skills
        .get_mut(shepan)
        .unwrap()
        .turn_fire_count = 1;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwwangchang {
            player: owner,
            target,
            card: engine_api::CardId::NONE,
            selected_card: engine_api::CardId::NONE,
            action: 1,
            stage: 0,
            targets: Vec::new(),
            skill: shepan,
        }],
    )
    .unwrap_err();
    let state = choose(tx.commit().state, owner, vec![0], 2);
    assert_eq!(state.skills.get(shepan).unwrap().turn_fire_count, 0);
    assert!(state.has_pending_choice());

    let (mut tx, owner, target, _, shepan) = setup(TWSHEPAN, 2_026_080_203);
    let selected = tx.state().hand_cards(target)[0];
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwwangchang {
                player: owner,
                target,
                card: engine_api::CardId::NONE,
                selected_card: engine_api::CardId::NONE,
                action: 1,
                stage: 0,
                targets: Vec::new(),
                skill: shepan,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![1], 3);
    let state = choose(state, owner, vec![selected.raw()], 4);
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(
        *state.zones.get(draw).unwrap().cards.last().unwrap(),
        selected
    );
}
}

mod original_tw_wujing {
//! Golden coverage for TW 吴景「锋捍/从击」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWFENGHAN: u32 = 3_542_733_092;
const TWCONGJI: u32 = 3_604_610_940;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twwujing.skill");

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    state
}

fn inject_discard(tx: &mut Transaction, suit: Suit, def: u32) -> CardId {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit,
        rank: 8,
        weapon_range: 0,
        zone: discard,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(card);
    card
}

#[test]
fn fenghan_draws_selected_targets_and_congji_gives_selected_red_discards() {
    let package = content_compiler::compile_source_named("tw-wujing-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 384);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_125, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWFENGHAN),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWCONGJI),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let skill = |tx: &Transaction, def| {
        tx.state()
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == owner && skill.def.raw() == def)
            .unwrap()
            .id
    };
    let fenghan = skill(&tx, TWFENGHAN);
    let first_hand = tx.state().hand_count(first);
    let second_hand = tx.state().hand_count(second);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwwujing {
                player: owner,
                target_count: 2,
                action: 0,
                event_cards: vec![],
                skill: fenghan,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![first.raw(), second.raw()], 1);
    assert_eq!(state.hand_count(first), first_hand + 1);
    assert_eq!(state.hand_count(second), second_hand + 1);

    let mut tx = Transaction::begin(state);
    let red = inject_discard(&mut tx, Suit::Heart, 93_850);
    let red_two = inject_discard(&mut tx, Suit::Diamond, 93_851);
    let black = inject_discard(&mut tx, Suit::Spade, 93_852);
    let congji = skill(&tx, TWCONGJI);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwwujing {
                player: owner,
                target_count: 0,
                action: 1,
                event_cards: vec![red, red_two, black],
                skill: congji,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![red.raw(), red_two.raw()], 2);
    let state = choose(state, owner, vec![first.raw()], 3);
    let hand = state.zones.find(ZoneKind::Hand, first).unwrap();
    assert_eq!(state.cards.get(red).unwrap().zone, hand);
    assert_eq!(state.cards.get(red_two).unwrap().zone, hand);
    assert_eq!(
        state.cards.get(black).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}
}

mod original_tw_zhengfu {
//! Golden coverage for TW 凯撒「征服」.

use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZHENGFU: u32 = 4_010_507_840;
const KAISA: u32 = 3_063_102_378;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/zhengfu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    sha: CardId,
    shan: CardId,
    tao: CardId,
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_360_000 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("zhengfu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 360);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZHENGFU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let sha = inject(&mut tx, owner, CardKind::Sha, Suit::Spade, 7);
    let shan = inject(&mut tx, target, CardKind::Shan, Suit::Club, 8);
    let tao = inject(&mut tx, target, CardKind::Tao, Suit::Heart, 9);
    Setup {
        state: tx.commit().state,
        owner,
        target,
        sha,
        shan,
        tao,
    }
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn play_and_choose_basic(setup: &Setup, key: u64) -> (GameState, PipelineResult) {
    let (state, result) = submit(
        setup.state.clone(),
        setup.owner,
        CommandKind::PlayCard {
            card: setup.sha,
            targets: vec![setup.target],
            card_targets: Vec::new(),
        },
        key,
    );
    // 当前实现：征赋在打杀后直接打开基本牌类别选择（无独立确认段）。
    let category = result.choice.unwrap();
    assert_eq!(category.kind, ChoiceKind::Custom);
    assert_eq!(
        category
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2, 3]
    );
    submit(
        state,
        setup.owner,
        CommandKind::ResolveChoice {
            choice_id: category.choice_id,
            selections: vec![1],
            cancelled: false,
        },
        key + 1,
    )
}

#[test]
fn matching_gift_keeps_shan_live_while_decline_makes_sha_unrespondable() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed.manifest.rule_vm_abi_version >= 360);
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == KAISA)
            .unwrap()
            .skills,
        [ZHENGFU]
    );

    let first = setup(20_260_801);
    let hp = first.state.players.get(first.target).unwrap().hp;
    let (state, result) = play_and_choose_basic(&first, 1);
    let gift = result.choice.unwrap();
    assert_eq!(gift.observer, first.target);
    assert_eq!(gift.kind, ChoiceKind::SelectCards);
    assert!(gift
        .options
        .iter()
        .any(|option| option.token == first.tao.raw()));
    let (state, result) = submit(
        state,
        first.target,
        CommandKind::ResolveChoice {
            choice_id: gift.choice_id,
            selections: vec![first.tao.raw()],
            cancelled: false,
        },
        4,
    );
    assert!(state.hand_cards(first.owner).contains(&first.tao));
    let shan = result.choice.unwrap();
    assert!(shan
        .options
        .iter()
        .any(|option| option.token == first.shan.raw()));
    let (state, result) = submit(
        state,
        first.target,
        CommandKind::ResolveChoice {
            choice_id: shan.choice_id,
            selections: vec![first.shan.raw()],
            cancelled: false,
        },
        5,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(first.target).unwrap().hp, hp);

    let setup = setup(20_260_802);
    let hp = setup.state.players.get(setup.target).unwrap().hp;
    let (state, result) = play_and_choose_basic(&setup, 10);
    let gift = result.choice.unwrap();
    let (state, result) = submit(
        state,
        setup.target,
        CommandKind::ResolveChoice {
            choice_id: gift.choice_id,
            selections: Vec::new(),
            cancelled: true,
        },
        13,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(state.players.get(setup.target).unwrap().hp, hp - 1);
}
}

mod original_tw_zumao {
//! Golden coverage for TW 祖茂「替巾」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWTIJIN: u32 = 3_420_559_729;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twzumao.skill");

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, actor: PlayerId, selection: u32, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn redirects_in_range_sha_then_discards_from_its_user_after_resolution() {
    let package = content_compiler::compile_source_named("tw-zumao-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 392);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_211, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, source, original) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TWTIJIN),
        },
        EffectRequest::SetTurnState {
            current: source,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let sha = inject(tx.state_mut(), source, CardKind::Sha, 99_021);
    let discarded = inject(tx.state_mut(), source, CardKind::Other, 99_022);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let state = tx.commit().state;
    let (state, result) = apply_command(
        state,
        Command {
            actor: source,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![original],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    // 当前实现：替巾直接重定向（无确认段）；结算后唯一 choice 是弃使用者一张牌。
    let state = choose(state, owner, discarded.raw(), 2);
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
    assert!(!state.has_pending_choice());
    assert_eq!(
        state
            .zones
            .get(state.cards.get(discarded).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::DiscardPile
    );
    let skill = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWTIJIN)
        .unwrap();
    assert!(skill.state_blob.is_empty());
}
}
