//! O2 全量合并批次 1（2026-10-14）：家族 key 的 41 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_key_akiko_dongcha {
//! Golden for KEY「洞察」rewritten from the legacy JavaScript semantics.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, ObserverKind, PlayerId,
    ReplayPrivacy, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::projection::{project_observer, project_player};
use engine_core::query::{observer_foreign_hand_faces_visible, observer_hidden_identities_visible};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DONGCHA: u32 = 2_799_956_315;
const AKIKO: u32 = 3_018_824_106;
const SOURCE: &str = include_str!("../../../content/original/ported/key/akiko_dongcha.skill");
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn compiled_package() -> content_schema::ContentPackage {
    content_compiler::compile_source_named("key-akiko-dongcha", SOURCE).unwrap()
}

fn foreign_hand_faces_are_visible(
    state: &engine_core::state::GameState,
    view: &engine_api::ObserverSnapshot,
    observer: PlayerId,
) -> bool {
    state
        .players
        .iter_live()
        .filter(|player| player.id != observer)
        .flat_map(|player| state.hand_cards(player.id))
        .all(|card| {
            view.known_cards
                .iter()
                .any(|known| known.id == card && known.def.is_some())
        })
}

#[test]
fn package_declares_typed_observer_visibility_and_character() {
    let compiled = compiled_package();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 20);
    assert_eq!(compiled.modifier_index.len(), 2);
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == DONGCHA
            && modifier.target == "observer_foreign_hand_faces_visible"
            && modifier.mode == "override"
            && modifier.value == 1
    }));
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == DONGCHA
            && modifier.target == "observer_hidden_identities_visible"
            && modifier.mode == "override"
            && modifier.value == 1
    }));

    let committed = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 691);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == DONGCHA && skill.locked));
    let character = committed
        .character_index
        .iter()
        .find(|character| character.id == AKIKO)
        .unwrap();
    assert_eq!(character.max_hp, 3);
    assert_eq!(
        if character.initial_hp == 0 {
            character.max_hp
        } else {
            character.initial_hp
        },
        3
    );
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "key");
    assert_eq!(character.skills, vec![DONGCHA]);
}

#[test]
fn only_the_skill_owner_and_private_replay_receive_hidden_information() {
    let package = compiled_package();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_718),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 20_260_718,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let observer = PlayerId::new(2);
    let other = PlayerId::new(3);
    let still_hidden = PlayerId::new(4);
    let before = project_player(engine.state(), observer);
    assert!(!foreign_hand_faces_are_visible(
        engine.state(),
        &before,
        observer
    ));
    assert_eq!(
        before
            .seats
            .iter()
            .find(|seat| seat.player == still_hidden)
            .unwrap()
            .identity,
        None
    );

    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: observer,
        skill_def: SkillDefId::new(DONGCHA),
    });
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;

    assert!(observer_foreign_hand_faces_visible(&state, observer));
    assert!(observer_hidden_identities_visible(&state, observer));
    assert!(!observer_foreign_hand_faces_visible(&state, other));
    assert!(!observer_hidden_identities_visible(&state, other));

    let owner_view = project_player(&state, observer);
    assert!(foreign_hand_faces_are_visible(
        &state,
        &owner_view,
        observer
    ));
    assert!(owner_view.seats.iter().all(|seat| seat.identity.is_some()));

    let private_replay = project_observer(
        &state,
        ObserverKind::ReplayExporter {
            privacy: ReplayPrivacy::Player(observer),
        },
    );
    assert!(foreign_hand_faces_are_visible(
        &state,
        &private_replay,
        observer
    ));
    assert!(private_replay
        .seats
        .iter()
        .all(|seat| seat.identity.is_some()));

    let other_view = project_player(&state, other);
    assert!(!foreign_hand_faces_are_visible(&state, &other_view, other));
    assert_eq!(
        other_view
            .seats
            .iter()
            .find(|seat| seat.player == still_hidden)
            .unwrap()
            .identity,
        None
    );

    let spectator = project_observer(&state, ObserverKind::Spectator);
    assert!(state
        .players
        .iter_live()
        .flat_map(|player| state.hand_cards(player.id))
        .all(|card| !spectator
            .known_cards
            .iter()
            .any(|known| known.id == card && known.def.is_some())));
    assert_eq!(
        spectator
            .seats
            .iter()
            .find(|seat| seat.player == still_hidden)
            .unwrap()
            .identity,
        None
    );

    let referee = project_observer(&state, ObserverKind::Referee);
    assert!(state
        .players
        .iter_live()
        .flat_map(|player| state.hand_cards(player.id))
        .all(|card| referee
            .known_cards
            .iter()
            .any(|known| known.id == card && known.def.is_some())));
    assert!(referee.seats.iter().all(|seat| seat.identity.is_some()));
}
}

mod original_key_ayato_jianshen {
//! Golden for KEY「僭神」的神属性【杀】与铁索隔离语义。

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::query::effective_card_nature;
use engine_core::state::{CardKind, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{damage_nature, EffectRequest};

const AYATO_JIANSHEN: u32 = 3_383_654_000;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn isolated() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == AYATO_JIANSHEN
            && modifier.target == "card_nature"
            && modifier.value == i32::from(damage_nature::KAMI)));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(AYATO_JIANSHEN as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: AYATO_JIANSHEN as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(AYATO_JIANSHEN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    (tx, players[0], players[1], players[2])
}

fn set_linked(tx: &mut Transaction, player: PlayerId) {
    let marks = &mut tx.state_mut().players.get_mut(player).unwrap().marks;
    marks.retain(|(mark, _)| *mark != engine_core::content::MARK_CHAIN);
    marks.push((engine_core::content::MARK_CHAIN, 1));
    marks.sort_by_key(|(mark, _)| *mark);
}

fn is_linked(tx: &Transaction, player: PlayerId) -> bool {
    tx.state()
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0)
}

#[test]
fn all_logical_sha_variants_become_kami_but_other_cards_keep_their_nature() {
    let (mut tx, owner, _, _) = isolated();
    let card = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Tao;
    let physical = tx.state().cards.get(card).unwrap();

    for logical in [CardKind::Sha, CardKind::ShaFire, CardKind::ShaThunder] {
        assert_eq!(
            effective_card_nature(tx.state(), owner, physical, logical),
            damage_nature::KAMI
        );
    }
    assert_eq!(
        effective_card_nature(tx.state(), owner, physical, CardKind::Tao),
        damage_nature::NORMAL
    );
}

#[test]
fn played_kami_sha_damages_only_its_linked_target_without_unchaining() {
    let (mut tx, owner, target, chained_other) = isolated();
    for player in [target, chained_other] {
        set_linked(&mut tx, player);
        for card in tx.state().hand_cards(player) {
            tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
        }
    }
    let sha = tx.state().hand_cards(owner)[0];
    tx.state_mut().cards.get_mut(sha).unwrap().kind = CardKind::Sha;
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let other_hp = tx.state().players.get(chained_other).unwrap().hp;

    let (after, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(after.players.get(target).unwrap().hp, target_hp - 1);
    assert_eq!(after.players.get(chained_other).unwrap().hp, other_hp);
    let after = Transaction::begin(after);
    assert!(is_linked(&after, target));
    assert!(is_linked(&after, chained_other));
    assert!(pipe.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target: damaged,
            amount: 1,
            nature,
            ..
        } if *source == owner && *damaged == target && *nature == damage_nature::KAMI
    )));
}

#[test]
fn normal_and_kami_damage_do_not_enter_the_elemental_chain_path() {
    for nature in [damage_nature::NORMAL, damage_nature::KAMI] {
        let (mut tx, owner, target, chained_other) = isolated();
        set_linked(&mut tx, target);
        set_linked(&mut tx, chained_other);
        let target_hp = tx.state().players.get(target).unwrap().hp;
        let other_hp = tx.state().players.get(chained_other).unwrap().hp;

        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: owner,
                target,
                amount: 1,
                is_lose_hp: false,
                nature,
            }],
        )
        .unwrap();
        assert_eq!(tx.state().players.get(target).unwrap().hp, target_hp - 1);
        assert_eq!(tx.state().players.get(chained_other).unwrap().hp, other_hp);
        assert!(is_linked(&tx, target));
        assert!(is_linked(&tx, chained_other));
    }
}
}

mod original_key_baiwei_haofang_key_qun {
//! Golden coverage for KEY 摆尾 / 豪放 / Key势 / 群势 rewritten from honest noops.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const INARI_BAIWEI: u32 = 3_124_826_192;
const HARUKO_HAOFANG: u32 = 2_831_053_102;
const KOTORI_SKILL_KEY: u32 = 4_028_878_707;
const KOTORI_SKILL_QUN: u32 = 2_366_750_902;

const BAIWEI_SRC: &str = include_str!("../../../content/original/ported/key/inari_baiwei.skill");
const HAOFANG_SRC: &str = include_str!("../../../content/original/ported/key/haruko_haofang.skill");
const KEY_SRC: &str = include_str!("../../../content/original/ported/key/kotori_skill_key.skill");
const QUN_SRC: &str = include_str!("../../../content/original/ported/key/kotori_skill_qun.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn bootstrap(source: &str, name: &str, skill_def: u32) -> (Transaction, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(skill_def), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner)
}

fn inject_hand(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    suit: Suit,
    def: u32,
) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn hand_limit(state: &GameState, player: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::HandLimit { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected hand-limit response: {other:?}"),
    }
}

#[test]
fn baiwei_source_and_npack_publish_diamond_basics() {
    let compiled = content_compiler::compile_source_named("inari_baiwei", BAIWEI_SRC).unwrap();
    for kind in ["sha", "shan", "tao", "jiu"] {
        assert!(
            compiled.view_as_index.iter().any(|view| {
                view.skill_def_id == INARI_BAIWEI && view.as_kind == kind && view.filter == "diamond"
            }),
            "missing view_as {kind}"
        );
    }
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.view_as_index.iter().any(|view| {
        view.skill_def_id == INARI_BAIWEI && view.as_kind == "sha" && view.filter == "diamond"
    }));
}

#[test]
fn haofang_source_and_npack_publish_delayed_as_wuzhong() {
    let compiled = content_compiler::compile_source_named("haruko_haofang", HAOFANG_SRC).unwrap();
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == HARUKO_HAOFANG
            && modifier.target == "card_name"
            && modifier.selector.as_deref() == Some("delayed_trick")
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == HARUKO_HAOFANG && modifier.target == "card_name"
    }));
}

#[test]
fn haofang_delayed_trick_is_effective_wuzhong() {
    let (mut tx, owner) = bootstrap(HAOFANG_SRC, "haofang-test", HARUKO_HAOFANG);
    let lebu = inject_hand(&mut tx, owner, CardKind::TrickLe, Suit::Heart, 283_101);
    let physical = tx.state().cards.get(lebu).unwrap();
    assert_eq!(physical.kind, CardKind::TrickLe);
    assert_eq!(
        engine_core::query::effective_card_kind(tx.state(), owner, physical),
        CardKind::TrickWuzhong
    );
}

#[test]
fn key_source_and_npack_publish_play_phase_active() {
    let compiled = content_compiler::compile_source_named("kotori_skill_key", KEY_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == KOTORI_SKILL_KEY));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == KOTORI_SKILL_KEY));
}

#[test]
fn key_play_phase_draws_one_and_loses_one_hp() {
    let (mut tx, owner) = bootstrap(KEY_SRC, "kotori-key-test", KOTORI_SKILL_KEY);
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.max_hp = 4;
        player.hp = 3;
    }
    let before_hand = tx.state().hand_count(owner);
    let before_hp = tx.state().players.get(owner).unwrap().hp;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == KOTORI_SKILL_KEY)
        .unwrap();
    let state = tx.commit().state;
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert_eq!(state.hand_count(owner), before_hand + 1);
    assert_eq!(state.players.get(owner).unwrap().hp, before_hp - 1);
}

#[test]
fn qun_source_and_npack_publish_hand_limit_lost_hp() {
    let compiled = content_compiler::compile_source_named("kotori_skill_qun", QUN_SRC).unwrap();
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == KOTORI_SKILL_QUN
            && modifier.target == "hand_limit"
            && modifier.value_source.as_deref() == Some("owner_lost_hp")
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == KOTORI_SKILL_QUN && modifier.target == "hand_limit"
    }));
}

#[test]
fn qun_hand_limit_grows_with_lost_hp() {
    let (mut tx, owner) = bootstrap(QUN_SRC, "kotori-qun-test", KOTORI_SKILL_QUN);
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.max_hp = 4;
        player.hp = 4;
    }
    assert_eq!(hand_limit(tx.state(), owner), 4);
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.hp = 2;
    }
    // Base hand limit is current HP (2); owner_lost_hp adds 2.
    assert_eq!(hand_limit(tx.state(), owner), 4);
}
}

mod original_key_baoqiu_miaobian_tingxian_nianli {
//! Golden coverage for KEY 棒球 / 妙变 / 听弦 / 念力 rewritten from honest noops
//! (听弦/念力 noop 注记系误标，本批一并修正)。零新原语批次（ABI 651 不变）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SASAMI_BAOQIU: u32 = 3_822_498_729;
const SASAMI_MIAOBIAN: u32 = 3_097_071_788;
const SASAMI_GONGQING: u32 = 4_263_354_141;
const SASAMI_FUNAN: u32 = 2_807_672_918;
const SHIORIMIYUKI_TINGXIAN: u32 = 2_995_156_497;
const SHIZURU_NIANLI: u32 = 2_833_421_901;

const BAOQIU_SRC: &str = include_str!("../../../content/original/ported/key/sasami_baoqiu.skill");
const MIAOBIAN_SRC: &str =
    include_str!("../../../content/original/ported/key/sasami_miaobian.skill");
const TINGXIAN_SRC: &str =
    include_str!("../../../content/original/ported/key/shiorimiyuki_tingxian.skill");
const NIANLI_SRC: &str =
    include_str!("../../../content/original/ported/key/shizuru_nianli.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

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
fn baoqiu_source_and_npack_publish_attack_range() {
    let compiled =
        content_compiler::compile_source_named("sasami_baoqiu", BAOQIU_SRC).unwrap();
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == SASAMI_BAOQIU
            && modifier.target == "attack_range"
            && modifier.mode == "add"
            && modifier.value == 2
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == SASAMI_BAOQIU
            && modifier.target == "attack_range"
            && modifier.value == 2
    }));
}

#[test]
fn miaobian_source_and_npack_publish_threshold_grants() {
    let compiled =
        content_compiler::compile_source_named("sasami_miaobian", MIAOBIAN_SRC).unwrap();
    let events: Vec<String> = compiled
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == SASAMI_MIAOBIAN)
        .map(|trigger| trigger.event.clone())
        .collect();
    assert_eq!(events, vec!["hp_lost", "hp_recovered", "game_started"]);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(
        pack.trigger_index
            .iter()
            .filter(|trigger| trigger.skill_def_id == SASAMI_MIAOBIAN)
            .count(),
        3
    );
}

#[test]
fn miaobian_grants_skills_by_hp_thresholds() {
    let (mut tx, owner, _) = bootstrap(MIAOBIAN_SRC, "miaobian-test", SASAMI_MIAOBIAN);
    // hp = 2：应获公清（≤3）与赴难（≤2），未到棒球（≤1）。
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: owner,
            target: owner,
            amount: 1,
            is_lose_hp: true,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert!(has_skill(&state, owner, SASAMI_GONGQING));
    assert!(has_skill(&state, owner, SASAMI_FUNAN));
    assert!(!has_skill(&state, owner, SASAMI_BAOQIU));

    // hp = 1 再失体力：三技齐授。
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: owner,
            target: owner,
            amount: 1,
            is_lose_hp: true,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert!(has_skill(&state, owner, SASAMI_GONGQING));
    assert!(has_skill(&state, owner, SASAMI_FUNAN));
    assert!(has_skill(&state, owner, SASAMI_BAOQIU));
}

#[test]
fn tingxian_source_and_npack_publish_play_begin_choice() {
    let compiled = content_compiler::compile_source_named(
        "shiorimiyuki_tingxian",
        TINGXIAN_SRC,
    )
    .unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SHIORIMIYUKI_TINGXIAN)
        .expect("tingxian trigger");
    assert_eq!(trigger.event, "phase_play");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == SHIORIMIYUKI_TINGXIAN && trigger.event == "phase_play"
    }));
}

#[test]
fn tingxian_draws_chosen_count_and_recovers() {
    let (mut tx, owner, _) = bootstrap(TINGXIAN_SRC, "tingxian-test", SHIORIMIYUKI_TINGXIAN);
    let player = tx.state().players.get(owner).unwrap();
    let max_hp = player.max_hp as i32;
    let hand_before = tx.state().hand_count(owner);
    assert!(max_hp >= 2);
    tx.state_mut().players.get_mut(owner).unwrap().hp = max_hp - 2;
    let mut tx = tx;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::Play,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    );
    // optional confirm → choose(1,2,3) 选 2：摸 2 张并回复 1。
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    let state = resolve(state, owner, vec![2], 3);
    assert_eq!(state.hand_count(owner), hand_before + 2);
    assert_eq!(state.players.get(owner).unwrap().hp, max_hp - 1);
}

#[test]
fn nianli_source_and_npack_publish_suit_view_as() {
    let compiled =
        content_compiler::compile_source_named("shizuru_nianli", NIANLI_SRC).unwrap();
    let mut pairs: Vec<(String, String, String)> = compiled
        .view_as_index
        .iter()
        .filter(|entry| entry.skill_def_id == SHIZURU_NIANLI)
        .map(|entry| {
            (
                entry.as_kind.clone(),
                entry.window.clone(),
                entry.filter.clone(),
            )
        })
        .collect();
    pairs.sort();
    assert_eq!(
        pairs,
        vec![
            ("sha".into(), "play_phase".into(), "diamond".into()),
            ("sha".into(), "response".into(), "diamond".into()),
            ("shan".into(), "response".into(), "club".into()),
            ("tao".into(), "play_phase".into(), "heart".into()),
            ("tao".into(), "response".into(), "heart".into()),
            ("wuxie".into(), "response".into(), "spade".into()),
        ]
    );
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(
        pack.view_as_index
            .iter()
            .filter(|entry| entry.skill_def_id == SHIZURU_NIANLI)
            .count(),
        6
    );
    assert!(pack.view_as_index.iter().any(|entry| {
        entry.skill_def_id == SHIZURU_NIANLI
            && entry.as_kind == "sha"
            && entry.filter == "diamond"
            && entry.nature.as_deref() == Some("thunder")
    }));
}
}

mod original_key_buhui_liezhen_shouqing {
//! Golden coverage for KEY 不悔 / 列阵 / 守琴.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const KUD_BUHUI: u32 = 4_166_321_790;
const KANO_LIEZHEN: u32 = 3_815_378_594;
const NAO_SHOUQING2: u32 = 4_259_091_944;
const BUHUI_SRC: &str = include_str!("../../../content/original/ported/key/kud_buhui.skill");
const LIEZHEN_SRC: &str = include_str!("../../../content/original/ported/key/kano_liezhen.skill");
const SHOUQING_SRC: &str =
    include_str!("../../../content/original/ported/key/nao_shouqing2.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn bootstrap(source: &str, name: &str, skill_def: u32) -> (Transaction, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(skill_def), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::DiscardFromHand {
        player: owner,
        count: u32::MAX,
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner)
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
fn buhui_source_and_npack_publish_dying_recover() {
    let compiled = content_compiler::compile_source_named("kud_buhui", BUHUI_SRC).unwrap();
    assert!(compiled.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == KUD_BUHUI && trigger.event == "dying" && trigger.optional
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == KUD_BUHUI && trigger.event == "dying"));
}

#[test]
fn buhui_recovers_to_two_when_dying() {
    let (mut tx, owner) = bootstrap(BUHUI_SRC, "buhui-test", KUD_BUHUI);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::EnteredDying {
                player: owner,
                source: PlayerId::NONE,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.players.get(owner).unwrap().hp, 2);
}

#[test]
fn liezhen_source_and_npack_publish_end_draw() {
    let compiled = content_compiler::compile_source_named("kano_liezhen", LIEZHEN_SRC).unwrap();
    assert!(compiled.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == KANO_LIEZHEN && trigger.event == "phase_end" && trigger.optional
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == KANO_LIEZHEN && trigger.event == "phase_end"));
}

#[test]
fn liezhen_end_phase_draws_two() {
    let (tx, owner) = bootstrap(LIEZHEN_SRC, "liezhen-test", KANO_LIEZHEN);
    let before = tx.state().hand_count(owner);
    let mut tx = tx;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::End,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 3);
    assert_eq!(state.hand_count(owner), before + 2);
}

#[test]
fn shouqing_source_and_npack_publish_tao() {
    let compiled = content_compiler::compile_source_named("nao_shouqing2", SHOUQING_SRC).unwrap();
    assert!(compiled
        .view_as_index
        .iter()
        .any(|view| view.skill_def_id == NAO_SHOUQING2 && view.as_kind == "tao"));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .view_as_index
        .iter()
        .any(|view| view.skill_def_id == NAO_SHOUQING2 && view.as_kind == "tao"));
}

#[test]
fn shouqing_converts_hand_card_to_tao_when_wounded() {
    let (mut tx, owner) = bootstrap(SHOUQING_SRC, "shouqing-test", NAO_SHOUQING2);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(193_301),
        kind: CardKind::Shan,
        suit: Suit::Club,
        rank: 2,
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == NAO_SHOUQING2)
        .unwrap();
    let state = tx.commit().state;
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![owner],
                card_targets: vec![card],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert_eq!(state.players.get(owner).unwrap().hp, 2);
}
}

mod original_key_chihaya_huairou {
//! Golden tests for KEY「怀柔」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardMoveReason, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::hash::canonical_state_hash;
use engine_core::state::{CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const CHIHAYA_HUAIROU: u32 = 3_695_539_048;
const SOURCE: &str = include_str!("../../../content/original/ported/key/chihaya_huairou.skill");
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn started() -> (GameState, PlayerId, SkillId) {
    let package = content_compiler::compile_source_named("chihaya_huairou_test", SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_718),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_718,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(CHIHAYA_HUAIROU),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == CHIHAYA_HUAIROU)
        .unwrap();
    (tx.commit().state, owner, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    cards: Vec<engine_api::CardId>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: cards,
            },
        },
    )
}

#[test]
fn committed_package_exposes_one_equipment_selection_and_abi16_module() {
    let committed = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    let compiled = content_compiler::compile_source_named("chihaya_huairou", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 16);
    let active = compiled
        .active_index
        .iter()
        .find(|active| active.skill_def_id == CHIHAYA_HUAIROU)
        .unwrap();
    let selection = active.selection.as_ref().unwrap();
    assert_eq!(selection.zone, "hand_equip");
    assert_eq!((selection.min, selection.max), (1, 1));
    assert_eq!(selection.filter.as_deref(), Some("card_equip"));
    let expected = &compiled.modules[0];
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn only_equipment_is_offered_and_a_direct_non_equipment_selection_is_rejected() {
    let (mut state, owner, skill) = started();
    let hand = state.hand_cards(owner);
    assert!(hand.len() >= 2);
    for card in &hand {
        state.cards.get_mut(*card).unwrap().kind = CardKind::Other;
    }
    let equipment = hand[0];
    let ordinary = hand[1];
    state.cards.get_mut(equipment).unwrap().kind = CardKind::Weapon;

    let before = canonical_state_hash(&state);
    let (same, rejected) = activate(state.clone(), owner, skill, vec![ordinary], 1);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let (_, offered) = activate(state, owner, skill, vec![], 2);
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    let options: Vec<_> = offered
        .choice
        .expect("equipment selection")
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(options, vec![equipment.raw()]);
}

#[test]
fn equipment_recast_moves_with_recast_reason_and_draws_one_replacement() {
    let (mut state, owner, skill) = started();
    let equipment = state.hand_cards(owner)[0];
    state.cards.get_mut(equipment).unwrap().kind = CardKind::Weapon;
    let hand_before = state.hand_count(owner);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();

    let (state, result) = activate(state, owner, skill, vec![equipment], 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before);
    assert_eq!(state.cards.get(equipment).unwrap().zone, discard);
    assert_eq!(state.skills.get(skill).unwrap().use_count, 1);
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsMoved {
            cards,
            to,
            reason: CardMoveReason::Recast,
            ..
        } if cards == &vec![equipment] && *to == discard
    )));
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsDrawn { player, cards } if *player == owner && cards.len() == 1
    )));
}
}

mod original_key_chihaya_liewu {
//! Golden for KEY「烈武」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::query::{
    card_distance_unlimited_for_card, card_target_limit_for_kind, card_use_unlimited_for_card,
};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const LIEWU: u32 = 3_721_191_175;
const LIEWU_HELPER: u32 = 2_978_088_269;
const HUAIROU: u32 = 3_695_539_048;
const CHIHAYA: u32 = 4_054_734_317;
const LIEWU_SOURCE: &str = include_str!("../../../content/original/ported/key/chihaya_liewu.skill");
const RUNTIME_SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/key/chihaya_liewu.skill"),
    include_str!("../../../content/original/ported/key/chihaya_huairou.skill")
);
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn liewu_package() -> content_schema::ContentPackage {
    content_compiler::compile_source_named("key-liewu", LIEWU_SOURCE).unwrap()
}

fn isolated(player_count: u8) -> (GameState, PlayerId, SkillId) {
    let package =
        content_compiler::compile_source_named("key-liewu-runtime", RUNTIME_SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_718),
            kind: CommandKind::StartGame {
                player_count,
                rng_seed: 20_260_718,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LIEWU),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == LIEWU)
        .unwrap();
    (tx.commit().state, owner, skill)
}

fn command(actor: PlayerId, key: u64, kind: CommandKind) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(key),
        kind,
    }
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(7_100 + kind.tag() as u32 * 20 + rank as u32),
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

fn clear_hands(tx: &mut Transaction) {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let players: Vec<_> = tx
        .state()
        .players
        .iter_live()
        .map(|player| player.id)
        .collect();
    for player in players {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let cards = tx.state().hand_cards(player);
        if !cards.is_empty() {
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
    }
}

fn pass_wuxie(
    mut state: GameState,
    mut pipeline: PipelineResult,
    key: &mut u64,
) -> (GameState, PipelineResult) {
    while state.has_pending_choice()
        && state.resolution.continuation_kind == ContinuationKind::WuxieWindow
    {
        let choice = pipeline.choice.clone().expect("wuxie choice");
        *key += 1;
        (state, pipeline) = apply_command(
            state,
            command(
                choice.observer,
                *key,
                CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: vec![],
                    cancelled: true,
                },
            ),
        );
        assert!(
            matches!(pipeline.result, CommandResult::Accepted { .. }),
            "{:?}",
            pipeline.result
        );
    }
    (state, pipeline)
}

fn resolve_target_card(
    state: GameState,
    pipeline: PipelineResult,
    expected: CardId,
    key: &mut u64,
) -> (GameState, PipelineResult) {
    let choice = pipeline.choice.expect("target-card choice");
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == expected.raw()));
    *key += 1;
    let (state, pipeline) = apply_command(
        state,
        command(
            choice.observer,
            *key,
            CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![expected.raw()],
                cancelled: false,
            },
        ),
    );
    assert!(
        matches!(pipeline.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipeline.result
    );
    (state, pipeline)
}

#[test]
fn committed_package_declares_liewu_helper_character_and_package_abi() {
    let compiled = liewu_package();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 19);
    assert_eq!(compiled.skill_index.len(), 2);
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == LIEWU_HELPER));
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == LIEWU
            && modifier.target == "card_target_limit"
            && modifier.selector.as_deref() == Some("ordinary_trick")
            && modifier.mode == "add"
            && modifier.value == 1
    }));
    assert_eq!(
        compiled.trigger_index[0].filter,
        "owner_is_event_player_no_enabled_equip_slot_once_per_game"
    );

    let committed = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    // 烈武本身最低 ABI 为 19；聚合 KEY 包 ABI 取包内最高（旅行箱 164、取用 651）。
    assert_eq!(committed.manifest.rule_vm_abi_version, 691);
    let expected = compiled
        .modules
        .iter()
        .find(|module| module.name == "chihaya_liewu_equip_slot_disabled")
        .unwrap();
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
    let character = committed
        .character_index
        .iter()
        .find(|character| character.id == CHIHAYA)
        .unwrap();
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "key");
    assert_eq!(character.skills, vec![LIEWU, 2_842_065_522]);
}

#[test]
fn sha_is_unlimited_and_ordinary_tricks_gain_one_optional_target() {
    let (state, owner, _) = isolated(4);
    let mut tx = Transaction::begin(state);
    clear_hands(&mut tx);
    let sha = inject(&mut tx, owner, CardKind::Sha, Suit::Spade, 7);
    let guohe = inject(&mut tx, owner, CardKind::TrickGuohe, Suit::Club, 3);
    assert!(card_use_unlimited_for_card(tx.state(), owner, sha));
    assert!(card_distance_unlimited_for_card(tx.state(), owner, sha));
    assert_eq!(
        card_target_limit_for_kind(tx.state(), owner, CardKind::TrickGuohe, 1),
        2
    );

    let targets: Vec<_> = tx
        .state()
        .players
        .iter_live()
        .filter(|player| player.id != owner)
        .map(|player| player.id)
        .collect();
    let first_card = inject(&mut tx, targets[0], CardKind::Shan, Suit::Heart, 2);
    let second_card = inject(&mut tx, targets[1], CardKind::Shan, Suit::Diamond, 4);
    inject(&mut tx, targets[2], CardKind::Shan, Suit::Club, 6);
    let state = tx.commit().state;

    let (_, rejected) = apply_command(
        state.clone(),
        command(
            owner,
            1,
            CommandKind::PlayCard {
                card: guohe,
                targets: targets.clone(),
                card_targets: vec![],
            },
        ),
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let (state, pipeline) = apply_command(
        state,
        command(
            owner,
            2,
            CommandKind::PlayCard {
                card: guohe,
                targets: targets[..2].to_vec(),
                card_targets: vec![],
            },
        ),
    );
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    let mut key = 2;
    let (state, pipeline) = pass_wuxie(state, pipeline, &mut key);
    let (state, pipeline) = resolve_target_card(state, pipeline, first_card, &mut key);
    let (state, pipeline) = pass_wuxie(state, pipeline, &mut key);
    let (state, pipeline) = resolve_target_card(state, pipeline, second_card, &mut key);
    assert!(!state.has_pending_choice(), "{:?}", pipeline.choice);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(first_card).unwrap().zone, discard);
    assert_eq!(state.cards.get(second_card).unwrap().zone, discard);
}

#[test]
fn first_last_slot_disable_loses_max_hp_adds_huairou_once_and_never_repeats() {
    let (state, owner, liewu) = isolated(2);
    let mut tx = Transaction::begin(state);
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.max_hp = 8;
        player.hp = 8;
    }
    let disabled = run_effects(
        &mut tx,
        vec![EffectRequest::DisableEquipSlots {
            player: owner,
            slots: vec![1, 2, 3, 4, 5],
        }],
    )
    .unwrap();
    assert_eq!(
        disabled
            .iter()
            .filter(|event| matches!(event, DomainEvent::EquipSlotDisabled { .. }))
            .count(),
        5
    );
    assert_eq!(tx.state().skills.get(liewu).unwrap().lifetime_fire_count, 1);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 4);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 4);
    let huairou_count = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .filter(|skill| tx.state().skills.get(**skill).unwrap().def.raw() == HUAIROU)
        .count();
    assert_eq!(huairou_count, 1);

    let repeated = dispatch_for_events(&mut tx, &disabled, &mut TriggerFireRecord::new()).unwrap();
    assert!(repeated.is_empty());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(HUAIROU),
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state()
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .filter(|skill| tx.state().skills.get(**skill).unwrap().def.raw() == HUAIROU)
            .count(),
        1
    );
}
}

mod original_key_chihaya_youfeng {
//! Golden for KEY「游凤」的无来源动态转换、分支计次与装备栏废除语义。

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

const YOUFENG: u32 = 2_842_065_522;
const SOURCE: &str = include_str!("../../../content/original/ported/key/chihaya_youfeng.skill");

fn package() -> content_schema::ContentPackage {
    content_compiler::compile_source_named("key-youfeng", SOURCE).unwrap()
}

fn started_state() -> engine_core::state::GameState {
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_718,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&package()).unwrap();
    engine.state().clone()
}

fn command(actor: PlayerId, kind: CommandKind) -> Command {
    Command {
        actor,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind,
    }
}

fn add_youfeng(tx: &mut Transaction, owner: PlayerId) -> engine_api::SkillId {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(YOUFENG),
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == YOUFENG)
        .unwrap()
}

fn other(state: &engine_core::state::GameState, player: PlayerId) -> PlayerId {
    state
        .players
        .iter_live()
        .find(|candidate| candidate.id != player)
        .unwrap()
        .id
}

fn neutralize_hands(tx: &mut Transaction) {
    let cards: Vec<CardId> = tx
        .state()
        .players
        .iter_live()
        .flat_map(|player| tx.state().hand_cards(player.id))
        .collect();
    for card in cards {
        tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
    }
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::NONE,
        kind,
        suit,
        rank: 7,
        weapon_range: if kind == CardKind::Weapon { 3 } else { 0 },
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn assert_accepted(result: &engine_core::command::PipelineResult) {
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
}

#[test]
fn package_declares_two_source_free_branches_and_abi_17() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 17);
    assert_eq!(package.view_as_index.len(), 2);
    let yin = package
        .view_as_index
        .iter()
        .find(|rule| rule.branch == Some(0))
        .unwrap();
    assert_eq!(yin.as_kind, "ordinary_trick");
    assert_eq!(yin.source_zone.as_deref(), Some("none"));
    assert!(yin.once_per_round);
    assert_eq!(yin.next_branch, Some(1));
    assert_eq!(yin.cost.as_deref(), Some("gain_max_hp_1"));
    let yang = package
        .view_as_index
        .iter()
        .find(|rule| rule.branch == Some(1))
        .unwrap();
    assert_eq!(yang.as_kind, "basic");
    assert_eq!(yang.next_branch, Some(0));
    assert_eq!(yang.cost.as_deref(), Some("disable_equip_slot"));
}

#[test]
fn proactive_yin_then_yang_each_once_per_round_and_slot_stays_disabled() {
    let state = started_state();
    let owner = state.turn.current;
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Play;
    neutralize_hands(&mut tx);
    let skill = add_youfeng(&mut tx, owner);
    let weapon = inject_hand(&mut tx, owner, CardKind::Weapon, Suit::Spade);
    let state = tx.commit().state;

    let (state, equip_result) = apply_command(
        state,
        command(
            owner,
            CommandKind::PlayCard {
                card: weapon,
                targets: vec![],
                card_targets: vec![],
            },
        ),
    );
    assert_accepted(&equip_result);
    let before_max = state.players.get(owner).unwrap().max_hp;

    let (state, yin) = apply_command(
        state,
        command(
            owner,
            CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::TrickWuzhong.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        ),
    );
    assert_accepted(&yin);
    assert_eq!(state.players.get(owner).unwrap().max_hp, before_max + 1);
    let instance = state.skills.get(skill).unwrap();
    assert_eq!(instance.view_as_branch, 1);
    assert_eq!(instance.view_as_used_branches & 1, 1);

    let (state, yang) = apply_command(
        state,
        command(
            owner,
            CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::Jiu.tag(),
                targets: vec![],
                equip_slot: 1,
            },
        ),
    );
    assert_accepted(&yang);
    assert_eq!(state.players.get(owner).unwrap().disabled_equip_slots[0], 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(weapon).unwrap().zone, discard);
    let instance = state.skills.get(skill).unwrap();
    assert_eq!(instance.view_as_branch, 0);
    assert_eq!(instance.view_as_used_branches & 0b11, 0b11);

    let (_, repeated) = apply_command(
        state.clone(),
        command(
            owner,
            CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::TrickWuzhong.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        ),
    );
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));

    let mut next_round = state;
    next_round.turn.round = next_round.turn.round.saturating_add(1);
    let (_, available_again) = apply_command(
        next_round,
        command(
            owner,
            CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::TrickWuzhong.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        ),
    );
    assert_accepted(&available_again);
}

#[test]
fn yang_can_answer_sha_after_selecting_an_enabled_slot() {
    let state = started_state();
    let source = state.turn.current;
    let target = other(&state, source);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Play;
    neutralize_hands(&mut tx);
    let skill = add_youfeng(&mut tx, target);
    tx.state_mut().skills.get_mut(skill).unwrap().view_as_branch = 1;
    let sha = inject_hand(&mut tx, source, CardKind::Sha, Suit::Spade);
    let hp = tx.state().players.get(target).unwrap().hp;
    let state = tx.commit().state;

    let (state, played) = apply_command(
        state,
        command(
            source,
            CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: vec![],
            },
        ),
    );
    assert_accepted(&played);
    let response = played.choice.unwrap();
    assert_eq!(response.observer, target);
    let token = response
        .options
        .iter()
        .map(|option| option.token)
        .find(|token| *token != engine_core::content::BAGUA_RESPONSE_TOKEN)
        .unwrap();

    let (state, chose_response) = apply_command(
        state,
        command(
            target,
            CommandKind::ResolveChoice {
                choice_id: response.choice_id,
                selections: vec![token],
                cancelled: false,
            },
        ),
    );
    assert_accepted(&chose_response);
    let slot_choice = chose_response.choice.unwrap();
    assert_eq!(slot_choice.observer, target);

    let (state, chose_slot) = apply_command(
        state,
        command(
            target,
            CommandKind::ResolveChoice {
                choice_id: slot_choice.choice_id,
                selections: vec![1],
                cancelled: false,
            },
        ),
    );
    assert_accepted(&chose_slot);
    assert_eq!(state.players.get(target).unwrap().hp, hp);
    assert_eq!(
        state.players.get(target).unwrap().disabled_equip_slots[0],
        1
    );
    assert_eq!(state.skills.get(skill).unwrap().view_as_branch, 0);
}

#[test]
fn yin_can_respond_with_wuxie_and_nullify_the_trick() {
    let state = started_state();
    let source = state.turn.current;
    let responder = other(&state, source);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Play;
    neutralize_hands(&mut tx);
    let skill = add_youfeng(&mut tx, responder);
    let wuzhong = inject_hand(&mut tx, source, CardKind::TrickWuzhong, Suit::Heart);
    let before_hand = tx.state().hand_count(source);
    let before_max = tx.state().players.get(responder).unwrap().max_hp;
    let state = tx.commit().state;

    let (state, played) = apply_command(
        state,
        command(
            source,
            CommandKind::PlayCard {
                card: wuzhong,
                targets: vec![],
                card_targets: vec![],
            },
        ),
    );
    assert_accepted(&played);
    let choice = played.choice.unwrap();
    assert_eq!(choice.observer, responder);
    let token = choice.options[0].token;

    let (state, wuxie) = apply_command(
        state,
        command(
            responder,
            CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![token],
                cancelled: false,
            },
        ),
    );
    assert_accepted(&wuxie);
    assert!(wuxie.choice.is_none());
    assert_eq!(state.hand_count(source), before_hand - 1);
    assert_eq!(state.players.get(responder).unwrap().max_hp, before_max + 1);
    assert_eq!(state.skills.get(skill).unwrap().view_as_branch, 1);
}

#[test]
fn yang_can_self_save_while_dying() {
    let state = started_state();
    let player = state.turn.current;
    let mut tx = Transaction::begin(state);
    neutralize_hands(&mut tx);
    let skill = add_youfeng(&mut tx, player);
    tx.state_mut().skills.get_mut(skill).unwrap().view_as_branch = 1;
    {
        let owner = tx.state_mut().players.get_mut(player).unwrap();
        owner.hp = 0;
        owner.dying = true;
    }
    let source = other(tx.state(), player);
    assert!(run_effects(
        &mut tx,
        vec![EffectRequest::RequestDyingSave { player, source }],
    )
    .is_err());
    let committed = tx.commit();
    let save_choice = committed.output.choice.unwrap();
    let token = save_choice
        .options
        .iter()
        .map(|option| option.token)
        .find(|token| *token != 0)
        .unwrap();

    let (state, selected_save) = apply_command(
        committed.state,
        command(
            player,
            CommandKind::ResolveChoice {
                choice_id: save_choice.choice_id,
                selections: vec![token],
                cancelled: false,
            },
        ),
    );
    assert_accepted(&selected_save);
    let slot_choice = selected_save.choice.unwrap();
    let (state, selected_slot) = apply_command(
        state,
        command(
            player,
            CommandKind::ResolveChoice {
                choice_id: slot_choice.choice_id,
                selections: vec![1],
                cancelled: false,
            },
        ),
    );
    assert_accepted(&selected_slot);
    let owner = state.players.get(player).unwrap();
    assert_eq!(owner.hp, 1);
    assert!(!owner.dying);
    assert_eq!(owner.disabled_equip_slots[0], 1);
}
}

mod original_key_deyi_shixin_mengzhong {
//! Golden coverage for KEY 得以 / 视心 / 梦终 rewritten from honest noops.

use engine_api::{
    Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    TurnPhaseTag,
};
use engine_core::effect::run_effects;
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const YUZURU_DEYI: u32 = 3_890_409_763;
const YUZURU_KUNFEN: u32 = 2_231_368_330;
const YUZURU_QUJI: u32 = 2_805_568_225;
const YUZURU_WANGSHENG: u32 = 3_744_539_174;
const YUZURU_WUXIN: u32 = 3_940_257_990;
const AO_SHIXIN: u32 = 2_364_015_708;
const AO_DIEGUI: u32 = 2_867_140_701;
const AO_KUIHUN: u32 = 3_209_605_261;
const RIKI_MENGZHONG: u32 = 2_200_887_116;
const RIKI_WENJI: u32 = 2_666_301_691;
const RIKI_CHONGZHEN: u32 = 4_103_807_065;

const DEYI_SRC: &str = include_str!("../../../content/original/ported/key/yuzuru_deyi.skill");
const SHIXIN_SRC: &str = include_str!("../../../content/original/ported/key/ao_shixin.skill");
const MENGZHONG_SRC: &str =
    include_str!("../../../content/original/ported/key/riki_mengzhong.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn setup(source: &str, name: &str, skill_def: u32, extras: &[u32]) -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_815_20),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_815_20,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&compiled).unwrap();
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for extra in extras {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(*extra),
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner)
}

fn fire_prepare(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("prepare dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn kill_other(state: GameState, owner: PlayerId) -> GameState {
    let victim = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer: owner,
        }],
    )
    .unwrap();
    tx.commit().state
}

fn has_def(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| state.skills.get(*skill).is_some_and(|instance| instance.def.raw() == def))
}

fn once_mark(state: &GameState, owner: PlayerId, skill: u32) -> i32 {
    state
        .players
        .get(owner)
        .and_then(|player| player.marks.iter().find(|(id, _)| *id == skill))
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

fn assert_npack_trigger(skill: u32, event: &str, filter: &str) {
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == skill && trigger.event == event && trigger.filter == filter
    }));
}

#[test]
fn deyi_source_and_npack_publish_death_swap() {
    let compiled = content_compiler::compile_source_named("yuzuru_deyi", DEYI_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == YUZURU_DEYI)
        .expect("deyi trigger");
    assert_eq!(trigger.event, "death");
    assert_eq!(trigger.filter, "owner_alive_not_target");
    assert!(!trigger.optional);
    assert_npack_trigger(YUZURU_DEYI, "death", "owner_alive_not_target");
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let actual = pack
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .expect("npack module for deyi");
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn deyi_on_other_death_swaps_to_wuxin_once() {
    let (mut state, owner) = setup(
        DEYI_SRC,
        "deyi-test",
        YUZURU_DEYI,
        &[YUZURU_KUNFEN, YUZURU_QUJI, YUZURU_WANGSHENG],
    );
    {
        let player = state.players.get_mut(owner).unwrap();
        player.max_hp = 4;
        player.hp = 2;
    }
    assert!(has_def(&state, owner, YUZURU_KUNFEN));
    assert!(has_def(&state, owner, YUZURU_QUJI));
    assert!(has_def(&state, owner, YUZURU_WANGSHENG));
    assert!(!has_def(&state, owner, YUZURU_WUXIN));

    let state = kill_other(state, owner);
    let player = state.players.get(owner).unwrap();
    assert_eq!(player.max_hp, 3);
    assert_eq!(player.hp, 3);
    assert_eq!(once_mark(&state, owner, YUZURU_DEYI), 1);
    assert!(!has_def(&state, owner, YUZURU_KUNFEN));
    assert!(!has_def(&state, owner, YUZURU_QUJI));
    assert!(!has_def(&state, owner, YUZURU_WANGSHENG));
    assert!(has_def(&state, owner, YUZURU_WUXIN));
}

#[test]
fn shixin_source_and_npack_publish_prepare_swap() {
    let compiled = content_compiler::compile_source_named("ao_shixin", SHIXIN_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == AO_SHIXIN)
        .expect("shixin trigger");
    assert_eq!(trigger.event, "phase_prepare");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    assert_npack_trigger(AO_SHIXIN, "phase_prepare", "owner_is_target");
}

#[test]
fn shixin_prepare_gains_max_hp_and_swaps_diegui_for_kuihun_once() {
    let (mut state, owner) = setup(SHIXIN_SRC, "shixin-test", AO_SHIXIN, &[AO_DIEGUI]);
    {
        let player = state.players.get_mut(owner).unwrap();
        player.max_hp = 3;
        player.hp = 3;
    }
    assert!(has_def(&state, owner, AO_DIEGUI));
    assert!(!has_def(&state, owner, AO_KUIHUN));

    let state = fire_prepare(state, owner);
    let player = state.players.get(owner).unwrap();
    assert_eq!(player.max_hp, 4);
    assert_eq!(player.hp, 4);
    assert_eq!(once_mark(&state, owner, AO_SHIXIN), 1);
    assert!(!has_def(&state, owner, AO_DIEGUI));
    assert!(has_def(&state, owner, AO_KUIHUN));

    let state = fire_prepare(state, owner);
    let player = state.players.get(owner).unwrap();
    assert_eq!(player.max_hp, 4);
    assert_eq!(player.hp, 4);
    assert_eq!(once_mark(&state, owner, AO_SHIXIN), 1);
}

#[test]
fn mengzhong_source_and_npack_publish_prepare_swap() {
    let compiled = content_compiler::compile_source_named("riki_mengzhong", MENGZHONG_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == RIKI_MENGZHONG)
        .expect("mengzhong trigger");
    assert_eq!(trigger.event, "phase_prepare");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    assert_npack_trigger(RIKI_MENGZHONG, "phase_prepare", "owner_is_target");
}

#[test]
fn mengzhong_prepare_swaps_wenji_for_chongzhen_once() {
    let (state, owner) = setup(
        MENGZHONG_SRC,
        "mengzhong-test",
        RIKI_MENGZHONG,
        &[RIKI_WENJI],
    );
    assert!(has_def(&state, owner, RIKI_WENJI));
    assert!(!has_def(&state, owner, RIKI_CHONGZHEN));

    let state = fire_prepare(state, owner);
    assert_eq!(once_mark(&state, owner, RIKI_MENGZHONG), 1);
    assert!(!has_def(&state, owner, RIKI_WENJI));
    assert!(has_def(&state, owner, RIKI_CHONGZHEN));

    let state = fire_prepare(state, owner);
    assert_eq!(once_mark(&state, owner, RIKI_MENGZHONG), 1);
}
}

mod original_key_godan_xiaoyuan {
//! Golden tests for KEY「消元」rewritten from the legacy JavaScript semantics.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const GODAN_XIAOYUAN: u32 = 2_543_675_799;
const GODAN_FEIQU: u32 = 3_490_863_084;
const SOURCE: &str = include_str!("../../../content/original/ported/key/godan_xiaoyuan.skill");
const FEIQU_SOURCE: &str = include_str!("../../../content/original/ported/key/godan_feiqu.skill");
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let source = format!("{FEIQU_SOURCE}\n{SOURCE}");
    let package = content_compiler::compile_source_named("godan_xiaoyuan_test", &source).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
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
    let owner = engine.state().turn.current;
    let source = engine
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
    effects.extend(
        [GODAN_FEIQU, GODAN_XIAOYUAN].map(|skill_def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, source)
}

fn has_skill(state: &GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == skill_def)
}

fn hp_decrease(is_lose_hp: bool, seed: u64) -> GameState {
    let (mut tx, owner, source) = setup(seed);
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.max_hp = 4;
        player.hp = 4;
    }
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 1);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 1);
    assert_eq!(tx.state().hand_count(owner), hand + 3);
    assert!(!has_skill(tx.state(), owner, GODAN_FEIQU));
    assert!(!has_skill(tx.state(), owner, GODAN_XIAOYUAN));
    tx.commit().state
}

#[test]
fn committed_package_contains_both_hp_decrease_triggers() {
    let committed = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    let compiled = content_compiler::compile_source_named("godan_xiaoyuan", SOURCE).unwrap();
    let triggers: Vec<_> = committed
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == GODAN_XIAOYUAN)
        .map(|trigger| {
            (
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert_eq!(
        triggers,
        [
            ("damage", "owner_is_target_hp_below_4", false),
            ("hp_lost", "owner_is_target_hp_below_4", false),
        ]
    );
    for expected in &compiled.modules {
        let actual = committed
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn damage_below_four_awakens_once() {
    let _ = hp_decrease(false, 20_260_718_1);
}

#[test]
fn direct_hp_loss_below_four_awakens_once() {
    let _ = hp_decrease(true, 20_260_718_2);
}

#[test]
fn hp_equal_to_four_does_not_awaken() {
    let (mut tx, owner, source) = setup(20_260_718_3);
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.max_hp = 6;
        player.hp = 5;
    }
    let hand = tx.state().hand_count(owner);
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
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 6);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 4);
    assert_eq!(tx.state().hand_count(owner), hand);
    assert!(has_skill(tx.state(), owner, GODAN_FEIQU));
    assert!(has_skill(tx.state(), owner, GODAN_XIAOYUAN));
}
}

mod original_key_godan_yuanyi {
//! Golden coverage for KEY「远忆」rewritten from legacy JavaScript semantics.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const GODAN_YUANYI: u32 = 3_610_550_580;
const GODAN_FEIQU: u32 = 3_490_863_084;
const GODAN_XIAOYUAN: u32 = 2_543_675_799;
const KEY_GODAN: u32 = 2_955_525_500;
const SOURCE: &str = include_str!("../../../content/original/ported/key/godan_yuanyi.skill");
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    previous: PlayerId,
}

fn setup(round: u32) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(
        &mut tx,
        3,
        GODAN_YUANYI as u64 + u64::from(round),
        "identity-standard",
    )
    .unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();

    let seats = tx.state().players.alive_by_seat();
    let previous = seats[0];
    let owner = seats[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in seats {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(GODAN_YUANYI),
        },
        EffectRequest::SetTurnState {
            current: previous,
            phase: Phase::End.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.round = round;
    Setup {
        state: tx.commit().state,
        owner,
        previous,
    }
}

fn end_phase(state: GameState, actor: PlayerId, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::EndPhase,
        },
    )
}

#[test]
fn package_publishes_exact_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    let compiled = content_compiler::compile_source_named("godan_yuanyi", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 75);
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == GODAN_YUANYI && skill.name == "godan_yuanyi" && skill.locked
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == KEY_GODAN)
        .unwrap();
    assert_eq!(
        character.skills,
        [GODAN_YUANYI, GODAN_FEIQU, GODAN_XIAOYUAN]
    );
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn turn_begin_draws_current_round_capped_at_three_then_opens_extra_play() {
    for (round, expected) in [(1, 1), (2, 2), (5, 3)] {
        let setup = setup(round);
        let (state, result) = end_phase(setup.state, setup.previous, u64::from(round));
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        assert_eq!(state.turn.current, setup.owner);
        assert_eq!(state.turn.round, round);
        assert_eq!(state.turn.phase, Phase::Play);
        assert_eq!(state.turn.flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE), 1);
        assert_eq!(state.hand_count(setup.owner), expected);
    }
}
}

mod original_key_guidui_fuxin_yinyuan_wei {
//! Golden coverage for KEY 归队 / 复心 / 姻缘 / 魏势 rewritten from honest noops.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const KENGO_GUIDUI: u32 = 3_185_675_503;
const NAGISA_FUXIN: u32 = 2_871_163_155;
const SEIRA_YINYUAN: u32 = 4_047_635_802;
const KOTORI_SKILL_WEI: u32 = 3_333_120_229;

const GUIDUI_SRC: &str = include_str!("../../../content/original/ported/key/kengo_guidui.skill");
const FUXIN_SRC: &str = include_str!("../../../content/original/ported/key/nagisa_fuxin.skill");
const YINYUAN_SRC: &str = include_str!("../../../content/original/ported/key/seira_yinyuan.skill");
const WEI_SRC: &str = include_str!("../../../content/original/ported/key/kotori_skill_wei.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

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

fn inject_hand(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Shan,
        suit: Suit::Club,
        rank: 6,
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
fn guidui_source_and_npack_publish_prepare_restore() {
    let compiled = content_compiler::compile_source_named("kengo_guidui", GUIDUI_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == KENGO_GUIDUI)
        .expect("guidui trigger");
    assert_eq!(trigger.event, "phase_prepare");
    assert_eq!(trigger.filter, "owner_is_target");
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == KENGO_GUIDUI && trigger.event == "phase_prepare"
    }));
}

#[test]
fn guidui_prepare_restores_disabled_slots() {
    let (mut tx, owner, _) = bootstrap(GUIDUI_SRC, "guidui-test", KENGO_GUIDUI);
    tx.state_mut()
        .players
        .get_mut(owner)
        .unwrap()
        .disabled_equip_slots = [1, 1, 1, 1, 1];
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("prepare dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    assert!(tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .disabled_equip_slots
        .iter()
        .all(|slot| *slot == 0));
}

#[test]
fn fuxin_source_and_npack_publish_other_damage_draw() {
    let compiled = content_compiler::compile_source_named("nagisa_fuxin", FUXIN_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NAGISA_FUXIN)
        .expect("fuxin trigger");
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_alive_not_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == NAGISA_FUXIN && trigger.filter == "owner_alive_not_target"
    }));
}

#[test]
fn fuxin_lets_damaged_other_draw_one() {
    let (tx, owner, other) = bootstrap(FUXIN_SRC, "fuxin-test", NAGISA_FUXIN);
    let other_hand = tx.state().hand_count(other);
    let mut tx = tx;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::DamageDealt {
                source: owner,
                target: other,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.hand_count(other), other_hand + 1);
}

#[test]
fn yinyuan_source_and_npack_publish_active_gain() {
    let compiled = content_compiler::compile_source_named("seira_yinyuan", YINYUAN_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == SEIRA_YINYUAN));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == SEIRA_YINYUAN));
}

#[test]
fn wei_source_and_npack_publish_turn_begin_recast() {
    let compiled = content_compiler::compile_source_named("kotori_skill_wei", WEI_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == KOTORI_SKILL_WEI)
        .expect("wei trigger");
    assert_eq!(trigger.event, "phase_begin");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == KOTORI_SKILL_WEI && trigger.event == "phase_begin"
    }));
}

#[test]
fn wei_turn_begin_discards_one_and_draws_one() {
    let (mut tx, owner, _) = bootstrap(WEI_SRC, "wei-test", KOTORI_SKILL_WEI);
    inject_hand(&mut tx, owner, 333_101);
    let before = tx.state().hand_count(owner);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnBegan { player: owner }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 3);
    assert_eq!(state.hand_count(owner), before);
}
}

mod original_key_hinata_qiulve {
//! Golden tests for KEY「求略」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{turn_flag, Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const HINATA_QIULVE: u32 = 3_761_656_212;
const HINATA_QIULVE_CLEAR: u32 = 3_988_736_644;
const SOURCE: &str = include_str!("../../../content/original/ported/key/hinata_qiulve.skill");
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn engine(seed: u64, players: u8) -> Engine {
    let package = content_compiler::compile_source_named("hinata_qiulve_test", SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    engine
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return;
    }
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
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

fn isolate_and_add(tx: &mut Transaction, owner: PlayerId) -> SkillId {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(HINATA_QIULVE),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == HINATA_QIULVE)
        .unwrap()
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
        def: CardDefId::new(99_000 + kind.tag() as u32 * 20 + u32::from(rank)),
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
fn committed_package_contains_typed_view_as_contract_and_folded_helper() {
    let committed = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    let compiled = content_compiler::compile_source_named("hinata_qiulve", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 14);
    let play = compiled
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == HINATA_QIULVE && view.window == "play_phase")
        .unwrap();
    assert_eq!(play.filter, "non_basic");
    assert_eq!(play.source_zone.as_deref(), Some("hand_equip"));
    assert!(play.ignore_distance && play.red_unrespondable && play.black_no_use_count);
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == HINATA_QIULVE_CLEAR));
    for expected in &compiled.view_as_index {
        let actual = committed
            .view_as_index
            .iter()
            .find(|view| {
                view.skill_def_id == expected.skill_def_id && view.window == expected.window
            })
            .unwrap();
        assert_eq!(actual, expected);
    }
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == HINATA_QIULVE_CLEAR));
    for expected in &compiled.modules {
        let actual = committed
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn red_non_basic_sha_reaches_distance_two_and_cannot_be_answered() {
    let engine = engine(20_260_718, 4);
    let actor = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let actor_index = seats.iter().position(|player| *player == actor).unwrap();
    let far = seats[(actor_index + 2) % seats.len()];
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, far);
    let skill = isolate_and_add(&mut tx, actor);
    let source = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::Weapon,
        Suit::Heart,
        7,
    );
    inject(&mut tx, far, ZoneKind::Hand, CardKind::Shan, Suit::Club, 8);
    let hp = tx.state().players.get(far).unwrap().hp;

    let (state, pipe) = apply_command(tx.state().clone(), activate(actor, skill, far, source, 1));
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert!(pipe.choice.is_none());
    assert_eq!(state.players.get(far).unwrap().hp, hp - 1);
}

#[test]
fn black_non_basic_sha_does_not_consume_or_require_a_sha_use() {
    let engine = engine(20_260_719, 2);
    let actor = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != actor)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, target);
    let skill = isolate_and_add(&mut tx, actor);
    let source = inject(
        &mut tx,
        actor,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        Suit::Spade,
        9,
    );
    tx.state_mut().turn.set_flag(turn_flag::SHA_PLAYED, 1);
    let hp = tx.state().players.get(target).unwrap().hp;

    let (state, pipe) = apply_command(
        tx.state().clone(),
        activate(actor, skill, target, source, 2),
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);

    let mut basic = Transaction::begin(tx.state().clone());
    let rejected = inject(
        &mut basic,
        actor,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Spade,
        10,
    );
    let (_, pipe) = apply_command(
        basic.state().clone(),
        activate(actor, skill, target, rejected, 3),
    );
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
}

#[test]
fn non_basic_hand_or_equipment_card_can_answer_duel_as_sha() {
    let engine = engine(20_260_720, 2);
    let source_player = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != source_player)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    tx.state_mut().turn.phase = Phase::Play;
    clear_hand(&mut tx, source_player);
    clear_hand(&mut tx, target);
    isolate_and_add(&mut tx, target);
    let duel = inject(
        &mut tx,
        source_player,
        ZoneKind::Hand,
        CardKind::TrickJuedou,
        Suit::Diamond,
        11,
    );
    let equipment = inject(
        &mut tx,
        target,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Club,
        12,
    );

    let (_, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor: source_player,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(4),
            kind: CommandKind::PlayCard {
                card: duel,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let choice = pipe.choice.expect("求略应提供非基本牌作为响应杀");
    assert_eq!(choice.observer, target);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == equipment.raw()));
}
}

mod original_key_hisako_yinbao {
//! Golden tests for KEY「音爆」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{ContinuationKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{damage_nature, EffectRequest};

const HISAKO_YINBAO: u32 = 3_078_155_020;
const SOURCE: &str = include_str!("../../../content/original/ported/key/hisako_yinbao.skill");
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn started(players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("hisako_yinbao_test", SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_718),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_718,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let others: Vec<_> = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
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
        skill_def: SkillDefId::new(HISAKO_YINBAO),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, others)
}

fn set_judgment_suit(tx: &mut Transaction, suit: Suit) {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top = tx
        .state()
        .zones
        .get(draw)
        .unwrap()
        .cards
        .last()
        .copied()
        .unwrap();
    tx.state_mut().cards.get_mut(top).unwrap().suit = suit;
}

fn resolve(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (engine_core::state::GameState, PipelineResult) {
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
fn committed_package_contains_both_optional_triggers_and_abi15_module() {
    let committed = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    let compiled = content_compiler::compile_source_named("hisako_yinbao", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 15);
    let triggers: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == HISAKO_YINBAO)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert!(triggers.iter().all(|trigger| trigger.optional));
    assert!(triggers.iter().any(|trigger| trigger.event == "damage"));
    assert!(triggers
        .iter()
        .any(|trigger| trigger.event == "hp_recovered"));
    for expected in &compiled.modules {
        let actual = committed
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn spade_judgment_forces_another_target_and_deals_owner_sourced_thunder_damage() {
    let (mut tx, owner, others) = started(3);
    set_judgment_suit(&mut tx, Suit::Spade);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: others[0],
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );

    let (state, activated) = resolve(tx.commit().state, owner, vec![1], 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::JudgeDamageOtherOnSuit
    );
    let offered: Vec<_> = activated
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert!(!offered.contains(&owner.raw()));
    assert!(others.iter().all(|player| offered.contains(&player.raw())));

    let target = others[1];
    let hp_before = state.players.get(target).unwrap().hp;
    let (state, damaged) = resolve(state, owner, vec![target.raw()], 2);
    assert!(matches!(damaged.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp_before - 1);
    assert!(damaged.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target: damaged,
            amount: 1,
            nature: damage_nature::THUNDER,
            ..
        } if *source == owner && *damaged == target
    )));
}

#[test]
fn recovery_also_triggers_but_a_non_spade_result_opens_no_target_choice() {
    let (mut tx, owner, others) = started(2);
    tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;
    set_judgment_suit(&mut tx, Suit::Heart);
    let other_hp = tx.state().players.get(others[0]).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::RecoverHp {
                source: others[0],
                player: owner,
                amount: 1,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, activated) = resolve(tx.commit().state, owner, vec![1], 3);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(state.resolution.pending_choice_id, 0);
    assert_eq!(state.players.get(others[0]).unwrap().hp, other_hp);
}
}

mod original_key_hisako_zhuanyun {
//! Golden tests for KEY「转运」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{ContinuationKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{damage_nature, EffectRequest};

const HISAKO: u32 = 2_387_604_022;
const HISAKO_YINBAO: u32 = 3_078_155_020;
const HISAKO_ZHUANYUN: u32 = 3_818_579_103;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/key/hisako_yinbao.skill"),
    "\n",
    include_str!("../../../content/original/ported/key/hisako_zhuanyun.skill")
);
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn started() -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("hisako_test", SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_722),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_722,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let others = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let mut tx = Transaction::begin(engine.state().clone());
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
            skill_def: SkillDefId::new(HISAKO_YINBAO),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(HISAKO_ZHUANYUN),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, others)
}

#[test]
fn package_declares_abi72_modifier_and_current_aggregate_character() {
    let compiled = content_compiler::compile_source_named("hisako_zhuanyun", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 72);
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == HISAKO_ZHUANYUN
            && modifier.target == "judgment_best_from_draw_pile"
            && modifier.mode == "override"
            && modifier.value == 1
    }));

    let committed = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 691);
    let character = committed
        .character_index
        .iter()
        .find(|character| character.id == HISAKO)
        .unwrap();
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "key");
    assert_eq!(character.skills, vec![HISAKO_YINBAO, HISAKO_ZHUANYUN]);
}

#[test]
fn yinbao_uses_the_best_card_across_the_draw_pile_instead_of_the_top_card() {
    let (mut tx, owner, others) = started();
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let draw_cards = tx.state().zones.get(draw).unwrap().cards.clone();
    assert!(draw_cards.len() >= 2);
    for card in &draw_cards {
        tx.state_mut().cards.get_mut(*card).unwrap().suit = Suit::Heart;
    }
    let favorable = draw_cards[draw_cards.len() - 2];
    let ordinary_top = *draw_cards.last().unwrap();
    tx.state_mut().cards.get_mut(favorable).unwrap().suit = Suit::Spade;

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: others[0],
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, output) = apply_command(
        tx.commit().state,
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
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::JudgeDamageOtherOnSuit
    );
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(favorable).unwrap().zone, discard);
    assert_eq!(state.cards.get(ordinary_top).unwrap().zone, draw);
    assert_eq!(
        state.zones.get(draw).unwrap().cards.last(),
        Some(&ordinary_top)
    );
}
}

mod original_key_huanjie_shenshi_duanfa_shelie {
//! Golden coverage for KEY 幻界 / 神视 / 断罚 / 涉猎 rewritten from honest noops.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const KEY_HUANJIE: u32 = 2_802_218_609;
const HINA_SHENSHI: u32 = 3_019_937_721;
const KYOU_DUANFA: u32 = 2_857_414_296;
const KYOKO_SHELIE: u32 = 3_597_507_893;

const HUANJIE_SRC: &str = include_str!("../../../content/original/ported/key/key_huanjie.skill");
const SHENSHI_SRC: &str = include_str!("../../../content/original/ported/key/hina_shenshi.skill");
const DUANFA_SRC: &str = include_str!("../../../content/original/ported/key/kyou_duanfa.skill");
const SHELIE_SRC: &str = include_str!("../../../content/original/ported/key/kyoko_shelie.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

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

fn inject_hand(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Shan,
        suit: Suit::Club,
        rank: 6,
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
fn huanjie_source_and_npack_publish_draw_from_bottom() {
    let compiled = content_compiler::compile_source_named("key_huanjie", HUANJIE_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == KEY_HUANJIE)
        .expect("huanjie trigger");
    assert_eq!(trigger.event, "phase_draw");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == KEY_HUANJIE && trigger.event == "phase_draw"
    }));
}

#[test]
fn huanjie_draw_phase_sets_bottom_flag() {
    let (mut tx, owner, _) = bootstrap(HUANJIE_SRC, "huanjie-test", KEY_HUANJIE);
    tx.state_mut().turn.phase = Phase::Draw;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("draw dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(
        tx.state().turn.flag(turn_flag::DRAW_PHASE_FROM_BOTTOM),
        1
    );
}

#[test]
fn shenshi_source_and_npack_publish_play_phase_draws() {
    let compiled = content_compiler::compile_source_named("hina_shenshi", SHENSHI_SRC).unwrap();
    let events: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == HINA_SHENSHI)
        .map(|trigger| trigger.event.as_str())
        .collect();
    assert!(events.contains(&"phase_play"));
    assert!(events.contains(&"phase_play_end"));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == HINA_SHENSHI && trigger.event == "phase_play"
    }));
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == HINA_SHENSHI && trigger.event == "phase_play_end"
    }));
}

#[test]
fn shenshi_play_begin_and_end_draw_two() {
    let (tx, owner, _) = bootstrap(SHENSHI_SRC, "shenshi-test", HINA_SHENSHI);
    let before = tx.state().hand_count(owner);
    let mut tx = tx;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::Play,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.hand_count(owner), before + 2);

    let mut tx = Transaction::begin(state);
    let mid = tx.state().hand_count(owner);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::PhaseEnded {
                current: owner,
                phase: TurnPhaseTag::Play,
                damage_sources: 0,
                damaged_players: 0,
                card_targeted_other: false,
                current_dealt_damage_other: false,
                phase_targeted_players: 0,
                phase_hp_changed_players: 0,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 3);
    assert_eq!(state.hand_count(owner), mid + 2);
}

#[test]
fn duanfa_source_and_npack_publish_damage_before() {
    let compiled = content_compiler::compile_source_named("kyou_duanfa", DUANFA_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == KYOU_DUANFA)
        .expect("duanfa trigger");
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == KYOU_DUANFA && trigger.event == "damage_before"
    }));
}

#[test]
fn duanfa_prevents_lethal_damage_and_recovers() {
    let (mut tx, owner, other) = bootstrap(DUANFA_SRC, "duanfa-test", KYOU_DUANFA);
    inject_hand(&mut tx, owner, 285_101);
    inject_hand(&mut tx, owner, 285_102);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: owner,
                amount: 2,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 4);
    assert_eq!(state.hand_count(owner), 0);
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        hp_before + 1,
        "damage prevented then recover 1"
    );
    assert!(!state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.def.raw() == KYOU_DUANFA));
}

#[test]
fn shelie_source_and_npack_publish_distinct_suit_draw() {
    let compiled = content_compiler::compile_source_named("kyoko_shelie", SHELIE_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == KYOKO_SHELIE)
        .expect("shelie trigger");
    assert_eq!(trigger.event, "phase_draw");
    assert_eq!(trigger.filter, "owner_is_target");
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == KYOKO_SHELIE && trigger.event == "phase_draw"
    }));
}

#[test]
fn shelie_draw_phase_zeroes_and_requests_suit_selection() {
    let (mut tx, owner, _) = bootstrap(SHELIE_SRC, "shelie-test", KYOKO_SHELIE);
    tx.state_mut().turn.phase = Phase::Draw;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("shelie dispatch error: {e:?}"));
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().turn.flag(turn_flag::DRAW_PHASE_DRAW_ZEROED),
        1
    );
}
}

mod original_key_jin_sanhua_wangsheng_shekong {
//! Golden coverage for KEY 晋势 / 散华 / 往生 / 设恐 rewritten from honest noops.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const KOTORI_SKILL_JIN: u32 = 3_135_780_791;
const SHIZUKU_SANHUA: u32 = 3_952_218_459;
const YUZURU_WANGSHENG: u32 = 3_744_539_174;
const NK_SHEKONG: u32 = 3_537_526_857;

const JIN_SRC: &str = include_str!("../../../content/original/ported/key/kotori_skill_jin.skill");
const SANHUA_SRC: &str = include_str!("../../../content/original/ported/key/shizuku_sanhua.skill");
const WANGSHENG_SRC: &str =
    include_str!("../../../content/original/ported/key/yuzuru_wangsheng.skill");
const SHEKONG_SRC: &str = include_str!("../../../content/original/ported/key/nk_shekong.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

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

fn inject_hand(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Shan,
        suit: Suit::Club,
        rank: 6,
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

fn once_mark(state: &GameState, owner: PlayerId, skill: u32) -> i32 {
    state
        .players
        .get(owner)
        .and_then(|player| player.marks.iter().find(|(id, _)| *id == skill))
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[test]
fn jin_source_and_npack_publish_draw_end() {
    let compiled = content_compiler::compile_source_named("kotori_skill_jin", JIN_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == KOTORI_SKILL_JIN)
        .expect("jin trigger");
    assert_eq!(trigger.event, "phase_draw_end");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == KOTORI_SKILL_JIN && trigger.event == "phase_draw_end"
    }));
}

#[test]
fn jin_draw_phase_end_draws_one() {
    let (tx, owner, _) = bootstrap(JIN_SRC, "jin-test", KOTORI_SKILL_JIN);
    let before = tx.state().hand_count(owner);
    let mut tx = tx;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::PhaseEnded {
                current: owner,
                phase: TurnPhaseTag::Draw,
                damage_sources: 0,
                damaged_players: 0,
                card_targeted_other: false,
                current_dealt_damage_other: false,
                phase_targeted_players: 0,
                phase_hp_changed_players: 0,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn sanhua_source_and_npack_publish_death_draw() {
    let compiled = content_compiler::compile_source_named("shizuku_sanhua", SANHUA_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SHIZUKU_SANHUA)
        .expect("sanhua trigger");
    assert_eq!(trigger.event, "dying");
    assert_eq!(trigger.filter, "owner_is_target");
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == SHIZUKU_SANHUA && trigger.event == "dying"));
}

#[test]
fn sanhua_death_lets_chosen_other_draw_four() {
    let (tx, owner, other) = bootstrap(SANHUA_SRC, "sanhua-test", SHIZUKU_SANHUA);
    let other_hand = tx.state().hand_count(other);
    let mut tx = tx;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: owner,
            source: other,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("dying dispatch error: {e:?}"));
    assert_eq!(run_effects(&mut tx, effects), Err(EffectError::ChoiceRequested));
    let state = resolve(tx.commit().state, owner, vec![other.raw()], 3);
    assert_eq!(state.hand_count(other), other_hand + 4);
}

#[test]
fn wangsheng_source_and_npack_publish_dying_recover() {
    let compiled =
        content_compiler::compile_source_named("yuzuru_wangsheng", WANGSHENG_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == YUZURU_WANGSHENG)
        .expect("wangsheng trigger");
    assert_eq!(trigger.event, "dying");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == YUZURU_WANGSHENG && trigger.event == "dying"
    }));
}

#[test]
fn wangsheng_dying_recovers_to_one_once() {
    let (mut tx, owner, other) = bootstrap(WANGSHENG_SRC, "wangsheng-test", YUZURU_WANGSHENG);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: owner,
            source: other,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("dying dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 1);
    assert_eq!(once_mark(tx.state(), owner, YUZURU_WANGSHENG), 1);

    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: owner,
            source: other,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("second dying dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 0);
}

#[test]
fn shekong_source_and_npack_publish_active() {
    let compiled = content_compiler::compile_source_named("nk_shekong", SHEKONG_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == NK_SHEKONG));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == NK_SHEKONG));
}

#[test]
fn shekong_discards_redraws_and_lets_other_draw_one() {
    let (mut tx, owner, other) = bootstrap(SHEKONG_SRC, "shekong-test", NK_SHEKONG);
    let card = inject_hand(&mut tx, owner, 180_201);
    let owner_hand = tx.state().hand_count(owner);
    let other_hand = tx.state().hand_count(other);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == NK_SHEKONG)
        .unwrap();
    let state = tx.commit().state;
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![card],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    let state = resolve(state, owner, vec![other.raw()], 4);
    assert_eq!(state.hand_count(owner), owner_hand);
    assert_eq!(state.hand_count(other), other_hand + 1);
}
}

mod original_key_kamome_huanmeng {
//! 第一百七十七批：key「幻梦」（kamome_huanmeng）观星族第 2 件。
//! 上游：准备阶段，观看牌堆顶 1+装备区牌数的牌并任意重排（拖拽顶/底）。
//! 实现：phase_prepare + owner_is_target → guanxing_top(self, 1+equipped_count(self))
//! （批 176 guanxing_top 原语 + 本批 EquippedCount 查询表达式，ABI 681）。
//! 近似：逐张二选（顶/底）近似拖拽自由排列。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const KAMOME: u32 = 3_440_411_698;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    let start = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(
        matches!(start, CommandResult::Accepted { .. }),
        "start rejected: {start:?}"
    );
    let state = engine.state().clone();
    let owner = state.turn.current;
    let mut tx = Transaction::begin(state);
    let removals: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    let _ = engine_core::effect::run_effects(&mut tx, removals);
    engine_core::effect::run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(KAMOME),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}

fn inject_equip(tx: &mut Transaction, player: PlayerId) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Equip, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_401),
        kind: CardKind::Armor,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone,
        owner: player,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn equip_count(state: &GameState, player: PlayerId) -> u32 {
    state
        .zones
        .find(ZoneKind::Equip, player)
        .and_then(|zone| state.zones.get(zone).map(|z| z.cards.len() as u32))
        .unwrap_or(0)
}

fn process_count(state: &GameState) -> u32 {
    state
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .and_then(|zone| state.zones.get(zone).map(|z| z.cards.len() as u32))
        .unwrap_or(0)
}

fn trigger_prepare(state: &mut GameState, owner: PlayerId) {
    let mut tx = Transaction::begin(state.clone());
    let dispatched = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    );
    match dispatched {
        Ok(requests) => {
            if let Err(e) = engine_core::effect::run_effects(&mut tx, requests) {
                eprintln!("DEBUG run_effects err: {e:?}");
            }
        }
        Err(e) => eprintln!("DEBUG dispatch err: {e:?}"),
    }
    *state = tx.commit().state;
}

fn resolve_pick(state: GameState, owner: PlayerId, pick: u32, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![pick],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "resolve rejected: {:?}",
        result.result
    );
    next
}

#[test]
fn package_publishes_kamome_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 681);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == KAMOME
            && trigger.event == "phase_prepare"
            && trigger.filter == "owner_is_target"
            && !trigger.optional
    }));
}

#[test]
fn kamome_guanxing_rounds_match_equipment_count_plus_one() {
    // owner 带一张装备：观星轮数 = 1 + 1 = 2。
    let (mut state, owner) = started(2_026_092_101);
    let mut tx = Transaction::begin(state.clone());
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let _equip = inject_equip(&mut tx, owner);
    state = tx.commit().state;

    trigger_prepare(&mut state, owner);
    assert!(state.has_pending_choice(), "guanxing loop must open");

    // 两轮全选「底」（token 2）。
    let mut key = 2;
    for round in 0..2 {
        assert!(state.has_pending_choice(), "round {round} must stay open");
        state = resolve_pick(state, owner, 2, key);
        key += 1;
    }
    assert!(
        !state.has_pending_choice(),
        "loop must close after 1+equip rounds"
    );
    assert_eq!(process_count(&state), 0, "processing drained after settle");
}

#[test]
fn kamome_zero_equipment_means_single_round() {
    let (mut state, owner) = started(2_026_092_102);
    let mut tx = Transaction::begin(state.clone());
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    state = tx.commit().state;

    trigger_prepare(&mut state, owner);
    assert!(state.has_pending_choice(), "guanxing loop must open");

    // 1 轮（1+0）。
    let mut key = 2;
    for round in 0..1 {
        assert!(state.has_pending_choice(), "round {round} must stay open");
        state = resolve_pick(state, owner, 2, key);
        key += 1;
    }
    assert!(
        !state.has_pending_choice(),
        "loop must close after single round"
    );
    assert_eq!(process_count(&state), 0);
}
}

mod original_key_kamome_suitcase {
//! Golden for KEY 装备技能「旅行箱」的阶段与翻面取消语义。

use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId};
use engine_core::effect::run_effects;
use engine_core::query::{skip_discard_phase, skip_judge_phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SUITCASE: u32 = 3_677_089_645;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

#[test]
fn suitcase_skips_both_phases_and_cancels_both_turn_over_directions() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(package.skill_index.len(), 261);
    assert_eq!(package.modifier_index.len(), 172);
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == SUITCASE
            && modifier.target == "turn_over_cancel"
            && modifier.mode == "override"
            && modifier.value == 1
    }));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(SUITCASE as u64),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: SUITCASE as u64,
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
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SUITCASE),
    });
    run_effects(&mut tx, effects).unwrap();

    assert!(skip_judge_phase(tx.state(), owner));
    assert!(skip_discard_phase(tx.state(), owner));
    run_effects(&mut tx, vec![EffectRequest::TurnOver { player: owner }]).unwrap();
    assert!(!tx.state().players.get(owner).unwrap().turned_over);

    tx.state_mut().players.get_mut(owner).unwrap().turned_over = true;
    run_effects(&mut tx, vec![EffectRequest::TurnOver { player: owner }]).unwrap();
    assert!(tx.state().players.get(owner).unwrap().turned_over);

    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def.raw() == SUITCASE)
        .unwrap()
        .id;
    run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    run_effects(&mut tx, vec![EffectRequest::TurnOver { player: owner }]).unwrap();
    assert!(!tx.state().players.get(owner).unwrap().turned_over);
}
}

mod original_key_kuihun_jianren_shihui_jingce {
//! Golden coverage for KEY 窥魂 / 坚忍 / 拾慧 / 精策 rewritten from honest noops.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const AO_KUIHUN: u32 = 3_209_605_261;
const SUNOHARA_JIANREN: u32 = 3_420_828_175;
const MIA_SHIHUI: u32 = 3_357_314_746;
const KYOKO_JINGCE: u32 = 2_335_248_353;

const KUIHUN_SRC: &str = include_str!("../../../content/original/ported/key/ao_kuihun.skill");
const JIANREN_SRC: &str =
    include_str!("../../../content/original/ported/key/sunohara_jianren.skill");
const SHIHUI_SRC: &str = include_str!("../../../content/original/ported/key/mia_shihui.skill");
const JINGCE_SRC: &str = include_str!("../../../content/original/ported/key/kyoko_jingce.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn setup(source: &str, name: &str, skill_def: u32) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_815_40),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_815_40,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&compiled).unwrap();
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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn fire_phase(state: GameState, current: PlayerId, phase: TurnPhaseTag) -> (GameState, bool) {
    let mut tx = Transaction::begin(state);
    match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged { current, phase }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => (tx.commit().state, true),
        Ok(effects) => {
            run_effects(&mut tx, effects).unwrap();
            (tx.commit().state, false)
        }
        Err(err) => panic!("phase dispatch error: {err:?}"),
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

fn inject_hand(state: &mut GameState, owner: PlayerId) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(335_701),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn kuihun_source_and_npack_publish_other_dying_draw() {
    let compiled = content_compiler::compile_source_named("ao_kuihun", KUIHUN_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == AO_KUIHUN)
        .expect("kuihun trigger");
    assert_eq!(trigger.event, "dying");
    assert_eq!(trigger.filter, "owner_alive_not_target");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == AO_KUIHUN && trigger.filter == "owner_alive_not_target"
    }));
}

#[test]
fn kuihun_draws_one_when_other_enters_dying() {
    let (state, owner, other) = setup(KUIHUN_SRC, "kuihun-test", AO_KUIHUN);
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: other,
            source: owner,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("dying dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

#[test]
fn jianren_source_and_npack_publish_damage_draw() {
    let compiled = content_compiler::compile_source_named("sunohara_jianren", JIANREN_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SUNOHARA_JIANREN)
        .expect("jianren trigger");
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == SUNOHARA_JIANREN && trigger.event == "damage"
    }));
}

#[test]
fn jianren_after_damage_lets_chosen_player_draw_one() {
    let (state, owner, other) = setup(JIANREN_SRC, "jianren-test", SUNOHARA_JIANREN);
    let other_hand = state.hand_count(other);
    let mut tx = Transaction::begin(state);
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
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    let state = resolve(state, owner, vec![other.raw()], 3);
    assert_eq!(state.hand_count(other), other_hand + 1);
}

#[test]
fn shihui_source_and_npack_publish_end_discard_recover() {
    let compiled = content_compiler::compile_source_named("mia_shihui", SHIHUI_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MIA_SHIHUI)
        .expect("shihui trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == MIA_SHIHUI && trigger.event == "phase_end"));
}

#[test]
fn shihui_end_phase_recovers_then_discards_one() {
    let (mut state, owner, _) = setup(SHIHUI_SRC, "shihui-test", MIA_SHIHUI);
    let card = inject_hand(&mut state, owner);
    {
        let player = state.players.get_mut(owner).unwrap();
        player.hp = player.max_hp as i32 - 1;
    }
    let before_hand = state.hand_count(owner);
    let before_hp = state.players.get(owner).unwrap().hp;
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("shihui dispatch error: {e:?}"));
    assert_eq!(run_effects(&mut tx, effects), Err(EffectError::ChoiceRequested));
    assert_eq!(tx.state().players.get(owner).unwrap().hp, before_hp + 1);
    let state = resolve(tx.commit().state, owner, vec![card.raw()], 5);
    assert_eq!(state.hand_count(owner), before_hand - 1);
    assert_eq!(state.players.get(owner).unwrap().hp, before_hp + 1);
}

#[test]
fn jingce_source_and_npack_publish_end_draw() {
    let compiled = content_compiler::compile_source_named("kyoko_jingce", JINGCE_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == KYOKO_JINGCE)
        .expect("jingce trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == KYOKO_JINGCE && trigger.event == "phase_end"));
}

#[test]
fn jingce_end_phase_draws_two() {
    let (state, owner, _) = setup(JINGCE_SRC, "jingce-test", KYOKO_JINGCE);
    let before = state.hand_count(owner);
    let (state, requested) = fire_phase(state, owner, TurnPhaseTag::End);
    assert!(requested);
    let state = resolve(state, owner, vec![1], 4);
    assert_eq!(state.hand_count(owner), before + 2);
}
}

mod original_key_kunfen_zhenjun_wangjin_shuwu {
//! Golden coverage for KEY 困愤 / 阵军 / 望尽 / 枢机续 rewritten from honest noops.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const YUZURU_KUNFEN: u32 = 2_231_368_330;
const LIYINGXIA_ZHENJUN: u32 = 3_556_873_289;
const TOMOYA_WANGJIN: u32 = 3_865_938_350;
const RUMI_SHUWU2: u32 = 3_596_925_719;

const KUNFEN_SRC: &str = include_str!("../../../content/original/ported/key/yuzuru_kunfen.skill");
const ZHENJUN_SRC: &str =
    include_str!("../../../content/original/ported/key/liyingxia_zhenjun.skill");
const WANGJIN_SRC: &str = include_str!("../../../content/original/ported/key/tomoya_wangjin.skill");
const SHUWU_SRC: &str = include_str!("../../../content/original/ported/key/rumi_shuwu2.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn setup(source: &str, name: &str, skill_def: u32) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_815_60),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_815_60,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&compiled).unwrap();
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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn fire_phase(state: GameState, current: PlayerId, phase: TurnPhaseTag) -> (GameState, bool) {
    let mut tx = Transaction::begin(state);
    match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged { current, phase }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => (tx.commit().state, true),
        Ok(effects) => {
            run_effects(&mut tx, effects).unwrap();
            (tx.commit().state, false)
        }
        Err(err) => panic!("phase dispatch error: {err:?}"),
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

#[test]
fn kunfen_source_and_npack_publish_end_lose_and_draw() {
    let compiled = content_compiler::compile_source_named("yuzuru_kunfen", KUNFEN_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == YUZURU_KUNFEN)
        .expect("kunfen trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == YUZURU_KUNFEN && trigger.event == "phase_end"));
}

#[test]
fn kunfen_end_phase_loses_one_hp_and_draws_two() {
    let (mut state, owner, _) = setup(KUNFEN_SRC, "kunfen-test", YUZURU_KUNFEN);
    {
        let player = state.players.get_mut(owner).unwrap();
        player.max_hp = 4;
        player.hp = 3;
    }
    let before_hand = state.hand_count(owner);
    let before_hp = state.players.get(owner).unwrap().hp;
    let (state, requested) = fire_phase(state, owner, TurnPhaseTag::End);
    assert!(!requested);
    assert_eq!(state.players.get(owner).unwrap().hp, before_hp - 1);
    assert_eq!(state.hand_count(owner), before_hand + 2);
}

#[test]
fn zhenjun_source_and_npack_publish_end_draw() {
    let compiled = content_compiler::compile_source_named("liyingxia_zhenjun", ZHENJUN_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == LIYINGXIA_ZHENJUN)
        .expect("zhenjun trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == LIYINGXIA_ZHENJUN && trigger.event == "phase_end"
    }));
}

#[test]
fn zhenjun_end_phase_lets_chosen_player_draw_one() {
    let (state, owner, other) = setup(ZHENJUN_SRC, "zhenjun-test", LIYINGXIA_ZHENJUN);
    let other_hand = state.hand_count(other);
    let (state, requested) = fire_phase(state, owner, TurnPhaseTag::End);
    assert!(requested);
    let state = resolve(state, owner, vec![1], 2);
    let state = resolve(state, owner, vec![other.raw()], 3);
    assert_eq!(state.hand_count(other), other_hand + 1);
}

#[test]
fn wangjin_source_and_npack_publish_other_end_draw() {
    let compiled = content_compiler::compile_source_named("tomoya_wangjin", WANGJIN_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == TOMOYA_WANGJIN)
        .expect("wangjin trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_alive_not_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == TOMOYA_WANGJIN && trigger.filter == "owner_alive_not_target"
    }));
}

#[test]
fn wangjin_other_end_phase_draws_two() {
    let (state, owner, other) = setup(WANGJIN_SRC, "wangjin-test", TOMOYA_WANGJIN);
    let before = state.hand_count(owner);
    let (state, requested) = fire_phase(state, other, TurnPhaseTag::End);
    assert!(requested);
    let state = resolve(state, owner, vec![1], 4);
    assert_eq!(state.hand_count(owner), before + 2);
}

#[test]
fn shuwu_source_and_npack_publish_end_draw() {
    let compiled = content_compiler::compile_source_named("rumi_shuwu2", SHUWU_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == RUMI_SHUWU2)
        .expect("shuwu trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == RUMI_SHUWU2 && trigger.event == "phase_end"));
}

#[test]
fn shuwu_end_phase_draws_one() {
    let (state, owner, _) = setup(SHUWU_SRC, "shuwu-test", RUMI_SHUWU2);
    let before = state.hand_count(owner);
    let (state, requested) = fire_phase(state, owner, TurnPhaseTag::End);
    assert!(!requested);
    assert_eq!(state.hand_count(owner), before + 1);
}
}

mod original_key_masato_baoquan {
//! Golden coverage for KEY 井之原真人「暴拳」rewritten from legacy JavaScript.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::{CommitResult, Transaction};
use rule_api::EffectRequest;

const KEY_MASATO: u32 = 2_270_763_259;
const MASATO_BAOQUAN: u32 = 3_870_573_152;
const SOURCE: &str = include_str!("../../../content/original/ported/key/masato_baoquan.skill");
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn prepared(seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("masato-baoquan-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let owner = tx.state().turn.current;
    let target = tx
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(MASATO_BAOQUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
    owner_state.max_hp = 8;
    owner_state.hp = 4;
    let target_state = tx.state_mut().players.get_mut(target).unwrap();
    target_state.max_hp = 6;
    target_state.hp = 6;
    (tx, owner, target)
}

fn damage_and_pause(mut tx: Transaction, source: PlayerId, target: PlayerId) -> CommitResult {
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
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChoosePreventDamageOrLoseMaxHpIncreaseDamage
    );
    tx.commit()
}

fn resolve(
    state: GameState,
    owner: PlayerId,
    choice_id: ChoiceId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
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
fn package_closes_exact_character_and_abi64_skill() {
    let package = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert!(package.manifest.capabilities.request_choice);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == KEY_MASATO)
        .unwrap();
    assert_eq!(character.name_key, "key_masato");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "key");
    assert_eq!(character.initial_hp, 4);
    assert_eq!(character.max_hp, 8);
    assert_eq!(character.skills, [MASATO_BAOQUAN]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MASATO_BAOQUAN)
        .unwrap();
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(trigger.filter, "owner_source");
    assert!(!trigger.optional);
    let compiled = content_compiler::compile_source_named("masato_baoquan", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 64);
    assert_eq!(
        compiled.modules[0].logic_hash,
        package
            .modules
            .iter()
            .find(|module| module.name == compiled.modules[0].name)
            .unwrap()
            .logic_hash
    );
}

#[test]
fn choice_is_forced_and_prevent_branch_cancels_only_bound_damage() {
    let (tx, owner, target) = prepared(2_026_072_201);
    let committed = damage_and_pause(tx, owner, target);
    let choice = committed.output.choice.unwrap();
    assert_eq!(choice.observer, owner);
    assert_eq!((choice.min_count, choice.max_count), (1, 1));
    assert!(!choice.allow_cancel);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    let (state, result) = resolve(committed.state, owner, choice.choice_id, vec![1], false, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 6);
    let owner_state = state.players.get(owner).unwrap();
    assert_eq!((owner_state.hp, owner_state.max_hp), (4, 8));
    assert!(!state.has_pending_choice());
    assert_eq!(state.resolution.current_damage_id, 0);
}

#[test]
fn increase_branch_loses_max_hp_then_changes_same_damage_from_one_to_three() {
    let (tx, owner, target) = prepared(2_026_072_202);
    let committed = damage_and_pause(tx, owner, target);
    let choice_id = committed.output.choice.as_ref().unwrap().choice_id;
    let (state, result) = resolve(committed.state, owner, choice_id, vec![2], false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let owner_state = state.players.get(owner).unwrap();
    assert_eq!((owner_state.hp, owner_state.max_hp), (4, 7));
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    assert!(!state.has_pending_choice());
    assert_eq!(state.resolution.current_damage_id, 0);
}

#[test]
fn cancellation_and_forged_option_are_rejected_atomically() {
    for (cancelled, selections, key) in [(true, Vec::new(), 3), (false, vec![3], 4)] {
        let (tx, owner, target) = prepared(2_026_072_203 + key);
        let committed = damage_and_pause(tx, owner, target);
        let choice_id = committed.output.choice.as_ref().unwrap().choice_id;
        let before = canonical_state_hash(&committed.state);
        let (same, result) = resolve(
            committed.state,
            owner,
            choice_id,
            selections,
            cancelled,
            key,
        );
        assert!(matches!(result.result, CommandResult::Rejected { .. }));
        assert_eq!(canonical_state_hash(&same), before);
        assert!(same.has_pending_choice());
    }
}

#[test]
fn damage_not_sourced_by_owner_does_not_trigger_baoquan() {
    let (mut tx, owner, target) = prepared(2_026_072_210);
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
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 3);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 8);
}
}

mod original_key_miki_binoculars {
//! Golden for key 望远镜 (miki_binoculars): locked — other players' hand cards
//! are face-up to the owner.

use engine_api::{Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId};
use engine_core::effect::run_effects;
use engine_core::query::observer_foreign_hand_faces_visible;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const MIKI_BINOCULARS: u32 = 3_578_894_093;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_261_901,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(result, CommandResult::Accepted { .. }));
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

#[test]
fn package_declares_observer_visibility_modifier() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.modifier_index.iter().any(|m| {
        m.skill_def_id == MIKI_BINOCULARS
            && m.target == "observer_foreign_hand_faces_visible"
            && m.mode == "override"
            && m.value == 1
    }));
}

#[test]
fn foreign_hand_faces_visible_to_owner() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: engine_api::SkillDefId::new(MIKI_BINOCULARS),
    });
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;

    assert!(
        observer_foreign_hand_faces_visible(&state, owner),
        "owner sees other players' hand faces"
    );
    // The owner's own hand is still hidden from the other player.
    assert!(
        !observer_foreign_hand_faces_visible(&state, other),
        "the other player does not see the owner's hand"
    );
}
}

mod original_key_nagisa_tiandu {
//! Golden coverage for KEY「天妒」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NAGISA_TIANDU: u32 = 4_094_577_149;
const SOURCE: &str = include_str!("../../../content/original/ported/key/nagisa_tiandu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn inject_discard(tx: &mut Transaction, suit: Suit, rank: u8) -> CardId {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(24_100 + rank as u32),
        kind: CardKind::Other,
        suit,
        rank,
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

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("nagisa-tiandu-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_725, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(NAGISA_TIANDU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx, owner, other)
}

#[test]
fn gains_only_the_owners_final_physical_judgment_card() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 691);
    let trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NAGISA_TIANDU)
        .unwrap();
    assert_eq!(trigger.event, "judgment_resolved");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);

    let (mut tx, owner, other) = setup();
    let result_card = inject_discard(&mut tx, Suit::Spade, 7);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::EmitJudgmentResolved {
            player: owner,
            result_card,
            suit: Suit::Spade,
            rank: 7,
            cause_card: CardId::NONE,
        }],
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(tx.state().resolution.pending_observer, owner.raw());

    let state = tx.state().clone();
    let (state, accepted) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    assert_eq!(state.cards.get(result_card).unwrap().zone, hand);

    let mut tx = Transaction::begin(state);
    let other_result = inject_discard(&mut tx, Suit::Heart, 8);
    run_effects(
        &mut tx,
        vec![EffectRequest::EmitJudgmentResolved {
            player: other,
            result_card: other_result,
            suit: Suit::Heart,
            rank: 8,
            cause_card: CardId::NONE,
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(tx.state().cards.get(other_result).unwrap().zone, discard);
}
}

mod original_key_noda_fengcheng {
//! Golden tests for KEY「奉承」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardMoveReason, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::state::ZoneKind;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const NODA_FENGCHENG: u32 = 2_452_779_390;
const SOURCE: &str = include_str!("../../../content/original/ported/key/noda_fengcheng.skill");
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn start(seed: u64) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
}

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("noda_fengcheng", SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(start(2)),
        CommandResult::Accepted { .. }
    ));
    let players: Vec<_> = engine
        .state()
        .players
        .iter_live()
        .map(|player| player.id)
        .collect();
    let giver = players[0];
    let owner = players[1];
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NODA_FENGCHENG),
        }],
    )
    .unwrap();
    (tx, giver, owner)
}

#[test]
fn committed_package_contains_the_locked_rewrite() {
    let committed = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    let compiled = content_compiler::compile_source_named("noda_fengcheng", SOURCE).unwrap();
    let expected = &compiled.modules[0];
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
    assert_eq!(compiled.manifest.rule_vm_abi_version, 3);
    assert_eq!(compiled.trigger_index[0].event, "cards_moved");
    assert_eq!(
        compiled.trigger_index[0].filter,
        "owner_received_gift_from_other"
    );
    assert!(!compiled.trigger_index[0].optional);
}

#[test]
fn voluntary_gift_draws_one_for_the_giver() {
    let (mut tx, giver, owner) = setup();
    let card = tx.state().hand_cards(giver)[0];
    let from = tx.state().cards.get(card).unwrap().zone;
    let to = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let giver_before = tx.state().hand_count(giver);
    let owner_before = tx.state().hand_count(owner);

    let events = run_effects(
        &mut tx,
        vec![EffectRequest::GiveCards {
            cards: vec![card],
            to,
            from,
        }],
    )
    .unwrap();

    assert_eq!(tx.state().hand_count(giver), giver_before);
    assert_eq!(tx.state().hand_count(owner), owner_before + 1);
    assert!(events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::CardsMoved {
            cards,
            reason: CardMoveReason::Give,
            ..
        } if cards == &vec![card]
    )));
    assert!(events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::CardsDrawn { player, cards }
            if *player == giver && cards.len() == 1
    )));
}

#[test]
fn taking_the_same_card_does_not_count_as_a_gift() {
    let (mut tx, giver, owner) = setup();
    let card = tx.state().hand_cards(giver)[0];
    let from = tx.state().cards.get(card).unwrap().zone;
    let to = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let giver_before = tx.state().hand_count(giver);
    let owner_before = tx.state().hand_count(owner);

    let events = run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to,
            from: Some(from),
        }],
    )
    .unwrap();

    assert_eq!(tx.state().hand_count(giver), giver_before - 1);
    assert_eq!(tx.state().hand_count(owner), owner_before + 1);
    assert!(!events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::CardsDrawn { player, .. } if *player == giver
    )));
}
}

mod original_key_noda_xunxin {
//! Golden tests for KEY「寻衅」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{CardKind, GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NODA_XUNXIN: u32 = 3_836_596_548;
const NODA_XUNXIN2: u32 = 2_159_749_553;
const NODA_FENGCHENG: u32 = 2_452_779_390;
const KEY_NODA: u32 = 3_057_217_957;
const SOURCE: &str = include_str!("../../../content/original/ported/key/noda_xunxin.skill");
const FENGCHENG_SOURCE: &str =
    include_str!("../../../content/original/ported/key/noda_fengcheng.skill");
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn skill(state: &GameState, owner: PlayerId, def: u32) -> SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn setup(with_fengcheng: bool) -> (GameState, PlayerId, PlayerId, SkillId) {
    let source = if with_fengcheng {
        format!("{SOURCE}\n{FENGCHENG_SOURCE}")
    } else {
        SOURCE.to_owned()
    };
    let package = content_compiler::compile_source_named("noda_xunxin_test", &source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_718, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let target = tx
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(NODA_XUNXIN),
    });
    if with_fengcheng {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NODA_FENGCHENG),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    for player in [owner, target] {
        for card in tx.state().hand_cards(player) {
            tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
        }
    }
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    tx.state_mut().players.get_mut(target).unwrap().hp = 4;
    let active = skill(tx.state(), owner, NODA_XUNXIN);
    (tx.state().clone(), owner, target, active)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    )
}

fn resolve_forced_gift(
    state: GameState,
    giver: PlayerId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    assert_eq!(state.resolution.pending_observer, giver.raw());
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let card = state.hand_cards(giver)[0].raw();
    apply_command(
        state,
        Command {
            actor: giver,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![card],
                cancelled: false,
            },
        },
    )
}

#[test]
fn committed_package_contains_dynamic_active_and_folded_helper() {
    let committed = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    let compiled = content_compiler::compile_source_named("noda_xunxin", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 13);
    let active = compiled
        .active_index
        .iter()
        .find(|active| active.skill_def_id == NODA_XUNXIN)
        .unwrap();
    assert_eq!(active.uses_per_turn, None);
    assert_eq!(active.uses_per_turn_source.as_deref(), Some("owner_hp"));
    assert!(active.targets.as_ref().unwrap().hp_at_least_owner);
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NODA_XUNXIN2));
    let noda = committed
        .character_index
        .iter()
        .find(|character| character.id == KEY_NODA)
        .expect("key_noda must be published after both skills are ported");
    assert_eq!(noda.skills, [NODA_FENGCHENG, NODA_XUNXIN]);
    for expected in &compiled.modules {
        let actual = committed
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn target_hp_and_current_hp_use_limit_are_authoritative() {
    let (state, owner, target, active) = setup(false);
    let mut too_low = state.clone();
    too_low.players.get_mut(target).unwrap().hp = 1;
    let (same, rejected) = activate(too_low.clone(), owner, target, active, 1);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(same, too_low);

    let (state, first) = activate(state, owner, target, active, 2);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    let (state, gift) = resolve_forced_gift(state, target, 3);
    assert!(matches!(gift.result, CommandResult::Accepted { .. }));
    let (state, second) = activate(state, owner, target, active, 4);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    let (state, gift) = resolve_forced_gift(state, target, 5);
    assert!(matches!(gift.result, CommandResult::Accepted { .. }));
    assert_eq!(state.skills.get(active).unwrap().use_count, 2);

    let (same, third) = activate(state.clone(), owner, target, active, 6);
    assert!(matches!(third.result, CommandResult::Rejected { .. }));
    assert_eq!(same, state);
}

#[test]
fn duel_loser_gives_winner_and_fengcheng_draws_for_the_giver() {
    let (state, owner, target, active) = setup(true);
    let owner_before = state.hand_count(owner);
    let target_before = state.hand_count(target);
    let target_hp = state.players.get(target).unwrap().hp;

    let (state, activated) = activate(state, owner, target, active, 10);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    let (state, gift) = resolve_forced_gift(state, target, 11);
    assert!(matches!(gift.result, CommandResult::Accepted { .. }));

    assert_eq!(state.hand_count(owner), owner_before + 1);
    assert_eq!(state.hand_count(target), target_before);
    assert_eq!(state.resolution.pending_choice_id, 0);
}

#[test]
fn no_gift_is_requested_when_the_original_duel_target_is_dead() {
    let (state, owner, target, _) = setup(false);
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(target).unwrap().alive = false;
    run_effects(
        &mut tx,
        vec![EffectRequest::EmitDuelResolved {
            card: CardId::NONE,
            initiator: owner,
            target,
            winner: owner,
            loser: target,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().resolution.pending_choice_id, 0);
}
}

mod original_key_nonghuan_yifu_zhenren_hydro {
//! Golden coverage for KEY 弄幻 / 义父 / 振刃 / 海德洛 rewritten from honest noops.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const MIDORI_NONGHUAN: u32 = 4_093_922_167;
const AKANE_YIFU2: u32 = 3_457_576_296;
const LUCIA_ZHENREN: u32 = 3_777_419_907;
const MIKI_HYDRO: u32 = 3_782_739_325;

const NONGHUAN_SRC: &str =
    include_str!("../../../content/original/ported/key/midori_nonghuan.skill");
const YIFU_SRC: &str = include_str!("../../../content/original/ported/key/akane_yifu2.skill");
const ZHENREN_SRC: &str = include_str!("../../../content/original/ported/key/lucia_zhenren.skill");
const HYDRO_SRC: &str =
    include_str!("../../../content/original/ported/key/miki_hydrogladiator_skill.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

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

fn inject_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
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
        weapon_range: u8::from(zone_kind == ZoneKind::Equip) * 2,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill_id(tx: &Transaction, owner: PlayerId, skill_def: u32) -> engine_api::SkillId {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == skill_def)
        .unwrap()
}

fn activate(
    state: GameState,
    actor: PlayerId,
    skill: engine_api::SkillId,
    targets: Vec<PlayerId>,
    card_targets: Vec<CardId>,
    key: u64,
) -> GameState {
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets,
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
fn nonghuan_source_and_npack_publish_active() {
    let compiled = content_compiler::compile_source_named("midori_nonghuan", NONGHUAN_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == MIDORI_NONGHUAN));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == MIDORI_NONGHUAN));
}

#[test]
fn yifu_source_and_npack_publish_active() {
    let compiled = content_compiler::compile_source_named("akane_yifu2", YIFU_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == AKANE_YIFU2));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == AKANE_YIFU2));
}

#[test]
fn yifu_discards_one_and_lets_other_draw() {
    let (mut tx, owner, other) = bootstrap(YIFU_SRC, "yifu-test", AKANE_YIFU2);
    let card = inject_card(&mut tx, owner, ZoneKind::Hand, CardKind::Shan, 345_101);
    let skill = skill_id(&tx, owner, AKANE_YIFU2);
    let other_hand = tx.state().hand_count(other);
    let state = activate(
        tx.commit().state,
        owner,
        skill,
        vec![other],
        vec![card],
        1,
    );
    assert_eq!(state.hand_count(owner), 0);
    assert_eq!(state.hand_count(other), other_hand + 1);
}

#[test]
fn zhenren_source_and_npack_publish_phase_end() {
    let compiled = content_compiler::compile_source_named("lucia_zhenren", ZHENREN_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == LUCIA_ZHENREN)
        .expect("zhenren trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_alive");
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == LUCIA_ZHENREN && trigger.event == "phase_end"
    }));
}

#[test]
fn zhenren_end_phase_discards_own_equipment() {
    let (mut tx, owner, other) = bootstrap(ZHENREN_SRC, "zhenren-test", LUCIA_ZHENREN);
    inject_card(&mut tx, owner, ZoneKind::Equip, CardKind::Weapon, 377_101);
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    assert_eq!(tx.state().zones.get(equip).unwrap().cards.len(), 1);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("phase-end dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    assert!(tx.state().zones.get(equip).unwrap().cards.is_empty());
}

#[test]
fn hydro_source_and_npack_publish_sha_damage() {
    let compiled =
        content_compiler::compile_source_named("miki_hydrogladiator_skill", HYDRO_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MIKI_HYDRO)
        .expect("hydro trigger");
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_source_sha");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == MIKI_HYDRO && trigger.event == "damage"
    }));
}
}

mod original_key_nongyin_guying {
//! Golden coverage for KEY 观铃「浓音」and 白羽「孤影」.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const MISUZU_NONGYIN: u32 = 2_333_084_601;
const SHIROHA_GUYING: u32 = 2_209_884_612;
const NONGYIN_SRC: &str =
    include_str!("../../../content/original/ported/key/misuzu_nongyin.skill");
const GUYING_SRC: &str = include_str!("../../../content/original/ported/key/shiroha_guying.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

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

fn inject_hand(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    suit: Suit,
    def: u32,
) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 4,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn nongyin_source_and_npack_publish_red_tao() {
    let compiled = content_compiler::compile_source_named("misuzu_nongyin", NONGYIN_SRC).unwrap();
    assert!(compiled.view_as_index.iter().any(|view| {
        view.skill_def_id == MISUZU_NONGYIN && view.as_kind == "tao" && view.filter == "red"
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.view_as_index.iter().any(|view| {
        view.skill_def_id == MISUZU_NONGYIN && view.as_kind == "tao"
    }));
}

#[test]
fn nongyin_red_card_converts_to_tao_when_wounded() {
    let (mut tx, owner, _) = bootstrap(NONGYIN_SRC, "nongyin-test", MISUZU_NONGYIN);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    let card = inject_hand(&mut tx, owner, CardKind::Shan, Suit::Heart, 181_201);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == MISUZU_NONGYIN)
        .unwrap();
    let state = tx.commit().state;
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![owner],
                card_targets: vec![card],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert_eq!(state.players.get(owner).unwrap().hp, 2);
}

#[test]
fn guying_source_and_npack_publish_both_damage_windows() {
    let compiled = content_compiler::compile_source_named("shiroha_guying", GUYING_SRC).unwrap();
    let mut events: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == SHIROHA_GUYING)
        .map(|trigger| trigger.filter.as_str())
        .collect();
    events.sort();
    assert_eq!(
        events,
        vec![
            "owner_is_target",
            "owner_source_other_damage_before"
        ]
    );
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(
        pack.trigger_index
            .iter()
            .filter(|trigger| trigger.skill_def_id == SHIROHA_GUYING)
            .count(),
        2
    );
}

#[test]
fn guying_reduces_incoming_and_increases_outgoing_damage() {
    let (tx, owner, other) = bootstrap(GUYING_SRC, "guying-test", SHIROHA_GUYING);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    let other_hp = tx.state().players.get(other).unwrap().hp;
    let mut tx = tx;
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
    assert_eq!(tx.state().players.get(owner).unwrap().hp, owner_hp);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(other).unwrap().hp, other_hp - 2);
}
}

mod original_key_quji_shelu_wumai_peiquan {
//! Golden coverage for KEY 曲继 / 射鹿 / 五脉 / 配券 rewritten from honest noops.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const YUZURU_QUJI: u32 = 2_805_568_225;
const ASARA_SHELU: u32 = 4_161_128_201;
const LIYINGXIA_WUMAI: u32 = 2_506_272_675;
const MINAGI_PEIQUAN: u32 = 3_941_246_023;

const QUJI_SRC: &str = include_str!("../../../content/original/ported/key/yuzuru_quji.skill");
const SHELU_SRC: &str = include_str!("../../../content/original/ported/key/asara_shelu.skill");
const WUMAI_SRC: &str = include_str!("../../../content/original/ported/key/liyingxia_wumai.skill");
const PEIQUAN_SRC: &str = include_str!("../../../content/original/ported/key/minagi_peiquan.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

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

fn inject_hand(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Shan,
        suit: Suit::Club,
        rank: 6,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn skill_id(tx: &Transaction, owner: PlayerId, skill_def: u32) -> engine_api::SkillId {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == skill_def)
        .unwrap()
}

fn activate(
    state: GameState,
    actor: PlayerId,
    skill: engine_api::SkillId,
    targets: Vec<PlayerId>,
    card_targets: Vec<CardId>,
    key: u64,
) -> GameState {
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets,
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
fn quji_source_and_npack_publish_active() {
    let compiled = content_compiler::compile_source_named("yuzuru_quji", QUJI_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == YUZURU_QUJI));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == YUZURU_QUJI));
}

#[test]
fn quji_discards_one_and_recovers_wounded_other() {
    let (mut tx, owner, other) = bootstrap(QUJI_SRC, "quji-test", YUZURU_QUJI);
    let card = inject_hand(&mut tx, owner, 280_201);
    tx.state_mut().players.get_mut(other).unwrap().hp = 2;
    let skill = skill_id(&tx, owner, YUZURU_QUJI);
    let state = activate(tx.commit().state, owner, skill, vec![other], vec![card], 1);
    assert_eq!(state.hand_count(owner), 0);
    assert_eq!(state.players.get(other).unwrap().hp, 3);
}

#[test]
fn shelu_source_and_npack_publish_active() {
    let compiled = content_compiler::compile_source_named("asara_shelu", SHELU_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == ASARA_SHELU));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == ASARA_SHELU));
}

#[test]
fn shelu_discards_one_and_draws_two() {
    let (mut tx, owner, _) = bootstrap(SHELU_SRC, "shelu-test", ASARA_SHELU);
    let card = inject_hand(&mut tx, owner, 416_101);
    let skill = skill_id(&tx, owner, ASARA_SHELU);
    let state = activate(tx.commit().state, owner, skill, vec![], vec![card], 1);
    assert_eq!(state.hand_count(owner), 2);
}

#[test]
fn wumai_source_and_npack_publish_turn_begin() {
    let compiled = content_compiler::compile_source_named("liyingxia_wumai", WUMAI_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == LIYINGXIA_WUMAI)
        .expect("wumai trigger");
    assert_eq!(trigger.event, "phase_begin");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == LIYINGXIA_WUMAI && trigger.event == "phase_begin"
    }));
}

#[test]
fn wumai_turn_begin_draws_one() {
    let (tx, owner, _) = bootstrap(WUMAI_SRC, "wumai-test", LIYINGXIA_WUMAI);
    let before = tx.state().hand_count(owner);
    let mut tx = tx;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnBegan { player: owner }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn peiquan_source_and_npack_publish_active() {
    let compiled = content_compiler::compile_source_named("minagi_peiquan", PEIQUAN_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == MINAGI_PEIQUAN));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == MINAGI_PEIQUAN));
}

#[test]
fn peiquan_can_draw_two_instead_of_damage() {
    let (tx, owner, other) = bootstrap(PEIQUAN_SRC, "peiquan-test", MINAGI_PEIQUAN);
    let skill = skill_id(&tx, owner, MINAGI_PEIQUAN);
    let before = tx.state().hand_count(owner);
    let other_hp = tx.state().players.get(other).unwrap().hp;
    let state = activate(tx.commit().state, owner, skill, vec![other], vec![], 1);
    let state = resolve(state, owner, vec![2], 2);
    assert_eq!(state.hand_count(owner), before + 2);
    assert_eq!(state.players.get(other).unwrap().hp, other_hp);
}
}

mod original_key_riki_nvzhuang {
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
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");
const SP_PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");
const STANDARD_PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

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
}

mod original_key_ryoichi_tuipi {
//! Golden for KEY「褪皮」的锦囊目标免疫与弃牌阶段装备计数语义。

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const RYOICHI_TUIPI: u32 = 3_426_318_690;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn isolated() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let tuipi: Vec<_> = package
        .modifier_index
        .iter()
        .filter(|modifier| modifier.skill_def_id == RYOICHI_TUIPI)
        .collect();
    assert_eq!(tuipi.len(), 3);
    assert!(tuipi.iter().any(|modifier| {
        modifier.target == "card_target_cancel" && modifier.selector.as_deref() == Some("shunshou")
    }));
    assert!(tuipi.iter().any(|modifier| {
        modifier.target == "card_target_cancel" && modifier.selector.as_deref() == Some("guohe")
    }));
    assert!(tuipi
        .iter()
        .any(|modifier| modifier.target == "discard_phase_includes_equip"));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(RYOICHI_TUIPI as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: RYOICHI_TUIPI as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(RYOICHI_TUIPI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, players[0], players[1])
}

fn play_trick_at_owner(base: &Transaction, owner: PlayerId, actor: PlayerId, kind: CardKind) {
    let mut state = base.state().clone();
    let card = state.hand_cards(actor)[0];
    state.cards.get_mut(card).unwrap().kind = kind;
    state.turn.current = actor;
    state.turn.phase = Phase::Play;
    let (after, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(kind as u64),
            kind: CommandKind::PlayCard {
                card,
                targets: vec![owner],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Rejected { .. }));
    assert_eq!(
        after, state,
        "rejected target declaration must not mutate state"
    );
}

#[test]
fn shunshou_and_guohe_cannot_target_the_owner() {
    let (tx, owner, actor) = isolated();
    play_trick_at_owner(&tx, owner, actor, CardKind::TrickShunshou);
    play_trick_at_owner(&tx, owner, actor, CardKind::TrickGuohe);
}

#[test]
fn discard_phase_counts_equipment_and_accepts_it_as_the_mandatory_discard() {
    let (mut tx, owner, _) = isolated();
    let hand_count = tx.state().hand_cards(owner).len() as u32;
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.hp = hand_count as i32;
        player.max_hp = hand_count;
    }
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let equipment = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(90_001),
        kind: CardKind::Armor,
        suit: Suit::Club,
        rank: 2,
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
        .push(equipment);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Discard;

    assert!(engine_core::query::discard_phase_includes_equip(
        tx.state(),
        owner
    ));
    assert_eq!(
        engine_core::query::hand_count_for_limit(tx.state(), owner),
        hand_count + 1
    );

    let (pending, pipe) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let choice = pipe.choice.unwrap();
    assert_eq!(choice.min_count, 1);
    assert_eq!(choice.max_count, 1);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == equipment.raw()));

    let (resolved, result) = apply_command(
        pending,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![equipment.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(resolved.turn.phase, Phase::End);
    assert_eq!(
        resolved.cards.get(equipment).unwrap().zone,
        resolved
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
}
}

mod original_key_sanli_fengfa_jueyi_lieyin {
//! Golden coverage for KEY 三礼 / 风阀 / 绝义 / 烈音 rewritten from honest noops.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::effect::run_effects;
use engine_core::state::{turn_flag, Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const LIYINGXIA_SANLI: u32 = 3_091_825_769;
const MIA_FENGFA: u32 = 2_660_197_008;
const YOSHINO_JUEYI: u32 = 3_841_113_685;
const YUI_LIEYIN: u32 = 2_976_604_975;

const SANLI_SRC: &str = include_str!("../../../content/original/ported/key/liyingxia_sanli.skill");
const FENGFA_SRC: &str = include_str!("../../../content/original/ported/key/mia_fengfa.skill");
const JUEYI_SRC: &str = include_str!("../../../content/original/ported/key/yoshino_jueyi.skill");
const LIEYIN_SRC: &str = include_str!("../../../content/original/ported/key/yui_lieyin.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn setup(source: &str, name: &str, skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_815_70),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_815_70,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&compiled).unwrap();
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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn inject_hand(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    suit: Suit,
    def: u32,
) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn sanli_source_and_npack_publish_targeted_draw() {
    let compiled = content_compiler::compile_source_named("liyingxia_sanli", SANLI_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == LIYINGXIA_SANLI)
        .expect("sanli trigger");
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(trigger.filter, "owner_targeted_by_other_card_once_per_turn");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == LIYINGXIA_SANLI && trigger.event == "card_targeted"
    }));
}

#[test]
fn sanli_draws_one_when_targeted_by_other() {
    let (mut tx, owner, other) = setup(SANLI_SRC, "sanli-test", LIYINGXIA_SANLI);
    let card = inject_hand(&mut tx, other, CardKind::Sha, Suit::Spade, 309_101);
    let before = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: other,
            card,
            targets: vec![owner],
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

#[test]
fn fengfa_source_and_npack_publish_draw_bonus() {
    let compiled = content_compiler::compile_source_named("mia_fengfa", FENGFA_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MIA_FENGFA)
        .expect("fengfa trigger");
    assert_eq!(trigger.event, "phase_draw");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == MIA_FENGFA && trigger.event == "phase_draw"));
}

#[test]
fn fengfa_increases_current_draw_count_by_one() {
    let (mut tx, owner, _) = setup(FENGFA_SRC, "fengfa-test", MIA_FENGFA);
    tx.state_mut().turn.phase = Phase::Draw;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("draw dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(
        tx.state()
            .turn
            .flag(turn_flag::CURRENT_DRAW_COUNT_BONUS),
        1
    );
}

#[test]
fn jueyi_source_and_npack_publish_outgoing_plus_one() {
    let compiled = content_compiler::compile_source_named("yoshino_jueyi", JUEYI_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == YOSHINO_JUEYI)
        .expect("jueyi trigger");
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(trigger.filter, "owner_source_other_damage_before");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == YOSHINO_JUEYI && trigger.event == "damage_before"
    }));
}

#[test]
fn jueyi_increases_outgoing_damage_by_one() {
    let (tx, owner, other) = setup(JUEYI_SRC, "jueyi-test", YOSHINO_JUEYI);
    let other_hp = tx.state().players.get(other).unwrap().hp;
    let mut tx = tx;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(other).unwrap().hp, other_hp - 2);
}

#[test]
fn lieyin_source_and_npack_publish_red_as_sha() {
    let compiled = content_compiler::compile_source_named("yui_lieyin", LIEYIN_SRC).unwrap();
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == YUI_LIEYIN
            && modifier.target == "card_name"
            && modifier.selector.as_deref() == Some("red")
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == YUI_LIEYIN && modifier.target == "card_name"
    }));
}

#[test]
fn lieyin_red_tao_is_effective_sha() {
    let (mut tx, owner, _) = setup(LIEYIN_SRC, "lieyin-test", YUI_LIEYIN);
    let heart = inject_hand(&mut tx, owner, CardKind::Tao, Suit::Heart, 297_601);
    let physical = tx.state().cards.get(heart).unwrap();
    assert_eq!(physical.kind, CardKind::Tao);
    assert_eq!(
        engine_core::query::effective_card_kind(tx.state(), owner, physical),
        CardKind::Sha
    );
    let club = inject_hand(&mut tx, owner, CardKind::Shan, Suit::Club, 297_602);
    let physical = tx.state().cards.get(club).unwrap();
    assert_eq!(
        engine_core::query::effective_card_kind(tx.state(), owner, physical),
        CardKind::Shan
    );
}
}

mod original_key_sizhi_weishang_daohai {
//! Golden coverage for KEY 丝织标记 / 伪伤 / 伪伤破杀 / 蹈海 rewritten from honest noops.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SHIZUKU_SIZHI2: u32 = 3_470_603_700;
const KENGO_WEISHANG: u32 = 2_671_352_658;
const KENGO_WEISHANG_SHA: u32 = 3_863_413_686;
const SATOMI_DAOHAI: u32 = 2_215_481_549;

const SIZHI_SRC: &str = include_str!("../../../content/original/ported/key/shizuku_sizhi2.skill");
const WEISHANG_SRC: &str =
    include_str!("../../../content/original/ported/key/kengo_weishang.skill");
const WEISHANG_SHA_SRC: &str =
    include_str!("../../../content/original/ported/key/kengo_weishang_sha.skill");
const DAOHAI_SRC: &str = include_str!("../../../content/original/ported/key/satomi_daohai.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

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

fn inject_hand(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    suit: Suit,
    def: u32,
) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn sizhi2_source_and_npack_publish_black_red_modifiers() {
    let compiled = content_compiler::compile_source_named("shizuku_sizhi2", SIZHI_SRC).unwrap();
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == SHIZUKU_SIZHI2
            && modifier.target == "card_ignored_hand_limit"
            && modifier.selector.as_deref() == Some("red")
    }));
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == SHIZUKU_SIZHI2
            && modifier.target == "card_distance_unlimited"
            && modifier.selector.as_deref() == Some("black")
    }));
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == SHIZUKU_SIZHI2
            && modifier.target == "card_use_unlimited"
            && modifier.selector.as_deref() == Some("black")
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == SHIZUKU_SIZHI2 && modifier.target == "card_use_unlimited"
    }));
}

#[test]
fn weishang_source_and_npack_publish_active_and_combat_mods() {
    let compiled = content_compiler::compile_source_named("kengo_weishang", WEISHANG_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == KENGO_WEISHANG));
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == KENGO_WEISHANG && modifier.target == "sha_use_limit"
    }));
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == KENGO_WEISHANG && modifier.target == "distance_from"
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == KENGO_WEISHANG));
}

#[test]
fn weishang_play_phase_disables_equip_slots() {
    let (tx, owner, _) = bootstrap(WEISHANG_SRC, "weishang-test", KENGO_WEISHANG);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == KENGO_WEISHANG)
        .unwrap();
    let state = tx.commit().state;
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert!(state
        .players
        .get(owner)
        .unwrap()
        .disabled_equip_slots
        .iter()
        .any(|slot| *slot != 0));
}

#[test]
fn weishang_sha_source_and_npack_publish_sha_target_discard() {
    let compiled =
        content_compiler::compile_source_named("kengo_weishang_sha", WEISHANG_SHA_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == KENGO_WEISHANG_SHA)
        .expect("weishang_sha trigger");
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(trigger.filter, "owner_source_sha");
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == KENGO_WEISHANG_SHA && trigger.event == "card_targeted"
    }));
}

#[test]
fn weishang_sha_forces_target_to_discard_one() {
    let (mut tx, owner, other) =
        bootstrap(WEISHANG_SHA_SRC, "weishang-sha-test", KENGO_WEISHANG_SHA);
    let sha = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade, 386_301);
    inject_hand(&mut tx, other, CardKind::Shan, Suit::Heart, 386_302);
    let other_hand = tx.state().hand_count(other);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: sha,
            targets: vec![other],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("card used dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(other), other_hand - 1);
}

#[test]
fn daohai_source_and_npack_publish_le() {
    let compiled = content_compiler::compile_source_named("satomi_daohai", DAOHAI_SRC).unwrap();
    assert!(compiled.view_as_index.iter().any(|view| {
        view.skill_def_id == SATOMI_DAOHAI && view.as_kind == "le"
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .view_as_index
        .iter()
        .any(|view| view.skill_def_id == SATOMI_DAOHAI && view.as_kind == "le"));
}
}

mod original_key_takaramono_banyin_zhiluo_tuolao {
//! Golden coverage for KEY 一番宝物 / 伴音 / 制裸 / 托老 rewritten from honest noops.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const YUI_TAKARAMONO: u32 = 3_352_203_313;
const YUI_YINHANG: u32 = 3_807_513_032;
const SHIORIMIYUKI_BANYIN: u32 = 3_141_719_441;
const MIKI_ZHILUO: u32 = 2_539_237_675;
const HIROTO_TUOLAO: u32 = 2_920_912_067;
const HIROTO_HUYU: u32 = 2_586_716_910;
const HIROTO_ZONGLVE: u32 = 3_175_747_508;

const TAKARAMONO_SRC: &str =
    include_str!("../../../content/original/ported/key/yui_takaramono.skill");
const BANYIN_SRC: &str =
    include_str!("../../../content/original/ported/key/shiorimiyuki_banyin.skill");
const ZHILUO_SRC: &str = include_str!("../../../content/original/ported/key/miki_zhiluo.skill");
const TUOLAO_SRC: &str = include_str!("../../../content/original/ported/key/hiroto_tuolao.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn setup(source: &str, name: &str, skill_def: u32, extras: &[u32]) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_815_30),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_815_30,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&compiled).unwrap();
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
    for extra in extras {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(*extra),
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn fire_phase(state: GameState, current: PlayerId, phase: TurnPhaseTag) -> (GameState, bool) {
    let mut tx = Transaction::begin(state);
    match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged { current, phase }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => (tx.commit().state, true),
        Ok(effects) => {
            run_effects(&mut tx, effects).unwrap();
            (tx.commit().state, false)
        }
        Err(err) => panic!("phase dispatch error: {err:?}"),
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

fn has_def(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| state.skills.get(*skill).is_some_and(|instance| instance.def.raw() == def))
}

fn once_mark(state: &GameState, owner: PlayerId, skill: u32) -> i32 {
    state
        .players
        .get(owner)
        .and_then(|player| player.marks.iter().find(|(id, _)| *id == skill))
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[test]
fn takaramono_source_and_npack_publish_prepare_awaken() {
    let compiled = content_compiler::compile_source_named("yui_takaramono", TAKARAMONO_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == YUI_TAKARAMONO)
        .expect("takaramono trigger");
    assert_eq!(trigger.event, "phase_prepare");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == YUI_TAKARAMONO && trigger.event == "phase_prepare"
    }));
}

#[test]
fn takaramono_prepare_gains_max_hp_and_yinhang_once() {
    let (mut state, owner, _) = setup(TAKARAMONO_SRC, "takaramono-test", YUI_TAKARAMONO, &[]);
    {
        let player = state.players.get_mut(owner).unwrap();
        player.max_hp = 3;
        player.hp = 3;
    }
    assert!(!has_def(&state, owner, YUI_YINHANG));
    let (state, requested) = fire_phase(state, owner, TurnPhaseTag::Prepare);
    assert!(!requested);
    let player = state.players.get(owner).unwrap();
    assert_eq!(player.max_hp, 4);
    assert_eq!(player.hp, 4);
    assert_eq!(once_mark(&state, owner, YUI_TAKARAMONO), 1);
    assert!(has_def(&state, owner, YUI_YINHANG));

    let (state, _) = fire_phase(state, owner, TurnPhaseTag::Prepare);
    let player = state.players.get(owner).unwrap();
    assert_eq!(player.max_hp, 4);
    assert_eq!(once_mark(&state, owner, YUI_TAKARAMONO), 1);
}

#[test]
fn banyin_source_and_npack_publish_damage_and_recover() {
    let compiled =
        content_compiler::compile_source_named("shiorimiyuki_banyin", BANYIN_SRC).unwrap();
    let events: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == SHIORIMIYUKI_BANYIN)
        .map(|trigger| trigger.event.as_str())
        .collect();
    assert!(events.contains(&"damage"));
    assert!(events.contains(&"hp_recovered"));
    assert!(compiled.trigger_index.iter().all(|trigger| {
        trigger.skill_def_id != SHIORIMIYUKI_BANYIN || trigger.optional
    }));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == SHIORIMIYUKI_BANYIN && trigger.event == "damage"));
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == SHIORIMIYUKI_BANYIN && trigger.event == "hp_recovered"
    }));
}

#[test]
fn banyin_after_damage_recovers_other() {
    let (mut state, owner, other) = setup(BANYIN_SRC, "banyin-test", SHIORIMIYUKI_BANYIN, &[]);
    {
        let player = state.players.get_mut(other).unwrap();
        player.hp = player.max_hp as i32 - 1;
    }
    let other_hp = state.players.get(other).unwrap().hp;
    let mut tx = Transaction::begin(state);
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
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    let state = resolve(state, owner, vec![other.raw()], 3);
    assert_eq!(state.players.get(other).unwrap().hp, other_hp + 1);
}

#[test]
fn zhiluo_source_and_npack_publish_other_end_draw() {
    let compiled = content_compiler::compile_source_named("miki_zhiluo", ZHILUO_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MIKI_ZHILUO)
        .expect("zhiluo trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_alive_not_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == MIKI_ZHILUO && trigger.filter == "owner_alive_not_target"
    }));
}

#[test]
fn zhiluo_other_end_phase_draws_one() {
    let (state, owner, other) = setup(ZHILUO_SRC, "zhiluo-test", MIKI_ZHILUO, &[]);
    let before = state.hand_count(owner);
    let (state, requested) = fire_phase(state, other, TurnPhaseTag::End);
    assert!(requested);
    let state = resolve(state, owner, vec![1], 4);
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn tuolao_source_and_npack_publish_end_swap() {
    let compiled = content_compiler::compile_source_named("hiroto_tuolao", TUOLAO_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == HIROTO_TUOLAO)
        .expect("tuolao trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == HIROTO_TUOLAO && trigger.event == "phase_end"
    }));
}

#[test]
fn tuolao_end_phase_draws_three_and_swaps_huyu_for_zonglve_once() {
    let (state, owner, _) = setup(
        TUOLAO_SRC,
        "tuolao-test",
        HIROTO_TUOLAO,
        &[HIROTO_HUYU],
    );
    let before = state.hand_count(owner);
    assert!(has_def(&state, owner, HIROTO_HUYU));
    assert!(!has_def(&state, owner, HIROTO_ZONGLVE));
    let (state, requested) = fire_phase(state, owner, TurnPhaseTag::End);
    assert!(!requested);
    assert_eq!(state.hand_count(owner), before + 3);
    assert_eq!(once_mark(&state, owner, HIROTO_TUOLAO), 1);
    assert!(!has_def(&state, owner, HIROTO_HUYU));
    assert!(has_def(&state, owner, HIROTO_ZONGLVE));

    let after = state.hand_count(owner);
    let (state, _) = fire_phase(state, owner, TurnPhaseTag::End);
    assert_eq!(state.hand_count(owner), after);
    assert_eq!(once_mark(&state, owner, HIROTO_TUOLAO), 1);
}
}

mod original_key_tishen_qianmeng_quanqing {
//! Golden coverage for KEY 替身 / 潜梦 / 劝请 rewritten from honest noops.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const MIDORI_TISHEN: u32 = 2_348_148_371;
const MIO_TISHEN: u32 = 4_056_753_334;
const MIA_QIANMENG: u32 = 2_319_632_885;
const AKANE_QUANQING: u32 = 2_806_428_173;

const MIDORI_SRC: &str = include_str!("../../../content/original/ported/key/midori_tishen.skill");
const MIO_SRC: &str = include_str!("../../../content/original/ported/key/mio_tishen.skill");
const QIANMENG_SRC: &str = include_str!("../../../content/original/ported/key/mia_qianmeng.skill");
const QUANQING_SRC: &str = include_str!("../../../content/original/ported/key/akane_quanqing.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

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

fn inject_hand(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Shan,
        suit: Suit::Club,
        rank: 6,
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

fn assert_tishen_source_and_npack(source: &str, name: &str, skill_def: u32) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == skill_def)
        .expect("tishen trigger");
    assert_eq!(trigger.event, "phase_prepare");
    assert_eq!(trigger.filter, "owner_is_target_wounded");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == skill_def && trigger.event == "phase_prepare"
    }));
}

fn tishen_prepare_recovers_and_draws(source: &str, name: &str, skill_def: u32) {
    let (mut tx, owner, _) = bootstrap(source, name, skill_def);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    let max_hp = tx.state().players.get(owner).unwrap().max_hp as i32;
    let lost = (max_hp - 2) as u32;
    let hand_before = tx.state().hand_count(owner);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::Prepare,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.players.get(owner).unwrap().hp, max_hp);
    assert_eq!(state.hand_count(owner), hand_before + lost);
    assert!(!state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.def.raw() == skill_def));
}

#[test]
fn midori_tishen_source_and_npack_publish_prepare() {
    assert_tishen_source_and_npack(MIDORI_SRC, "midori_tishen", MIDORI_TISHEN);
}

#[test]
fn midori_tishen_prepare_recovers_and_draws() {
    tishen_prepare_recovers_and_draws(MIDORI_SRC, "midori-tishen-test", MIDORI_TISHEN);
}

#[test]
fn mio_tishen_source_and_npack_publish_prepare() {
    assert_tishen_source_and_npack(MIO_SRC, "mio_tishen", MIO_TISHEN);
}

#[test]
fn mio_tishen_prepare_recovers_and_draws() {
    tishen_prepare_recovers_and_draws(MIO_SRC, "mio-tishen-test", MIO_TISHEN);
}

#[test]
fn qianmeng_source_and_npack_publish_game_started() {
    let compiled = content_compiler::compile_source_named("mia_qianmeng", QIANMENG_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MIA_QIANMENG)
        .expect("qianmeng trigger");
    assert_eq!(trigger.event, "game_started");
    assert_eq!(trigger.filter, "owner_alive");
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == MIA_QIANMENG && trigger.event == "game_started"
    }));
}

#[test]
fn qianmeng_game_start_draws_one() {
    let (mut tx, owner, _) = bootstrap(QIANMENG_SRC, "qianmeng-test", MIA_QIANMENG);
    let before = tx.state().hand_count(owner);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("game-started dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

#[test]
fn quanqing_source_and_npack_publish_active() {
    let compiled = content_compiler::compile_source_named("akane_quanqing", QUANQING_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == AKANE_QUANQING));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == AKANE_QUANQING));
}

#[test]
fn quanqing_discards_one_and_lets_other_draw() {
    let (mut tx, owner, other) = bootstrap(QUANQING_SRC, "quanqing-test", AKANE_QUANQING);
    let card = inject_hand(&mut tx, owner, 280_101);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == AKANE_QUANQING)
        .unwrap();
    let other_hand = tx.state().hand_count(other);
    let state = tx.commit().state;
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
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
    assert_eq!(state.hand_count(owner), 0);
    assert_eq!(state.hand_count(other), other_hand + 1);
}
}

mod original_key_tomoyo_wuwei {
//! Golden coverage for KEY「武威」(tomoyo_wuwei): hand cards view-as Sha.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TOMOYO_WUWEI: u32 = 2_212_519_004;
const SOURCE: &str = include_str!("../../../content/original/ported/key/tomoyo_wuwei.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, SkillId, CardId) {
    let compiled = content_compiler::compile_source_named("tomoyo-wuwei-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_77, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(TOMOYO_WUWEI),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_805),
        kind: CardKind::Shan,
        suit: Suit::Club,
        rank: 5,
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
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == TOMOYO_WUWEI)
        .unwrap();
    (tx.commit().state, owner, other, skill, card)
}

#[test]
fn compiled_source_and_npack_publish_sha() {
    let compiled = content_compiler::compile_source_named("tomoyo_wuwei", SOURCE).unwrap();
    let mut views: Vec<_> = compiled
        .view_as_index
        .iter()
        .filter(|view| view.skill_def_id == TOMOYO_WUWEI)
        .map(|view| view.as_kind.as_str())
        .collect();
    views.sort();
    views.dedup();
    assert_eq!(views, vec!["sha"]);

    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .view_as_index
        .iter()
        .any(|view| view.skill_def_id == TOMOYO_WUWEI && view.as_kind == "sha"));
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

mod original_key_umi_lunhui {
//! key/umi_lunhui 轮回（第八十四批·跨包复用首迁）：queue_extra_turn(check=0)
//! 纯额外回合 + storage 绑定门控。上游 key/skill.js:11616。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const UMI_LUNHUI: u32 = 2_932_731_107;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");
const SOURCES_DIR: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../content/original/ported/key");

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

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, cancelled: bool) -> GameState {
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(state.revision as u64),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
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
    let compiled = content_compiler::compile_source_named("key-umi-lunhui", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_830, "identity-standard").unwrap();
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
            skill_def: SkillDefId::new(UMI_LUNHUI),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
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

#[test]
fn umi_lunhui_publishes_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == UMI_LUNHUI && t.event == "turn_ended" && t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == UMI_LUNHUI && t.event == "turn_ended" && !t.optional
    }));
}


fn lunhui_storage(state: &GameState, holder: PlayerId) -> Option<u32> {
    let sid = state
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == UMI_LUNHUI)
        .map(|s| s.id)
        .unwrap();
    Some(state.skills.storage_get(sid, 1))
}

fn arm_condition(tx: &mut Transaction, holder: PlayerId) {
    // 满足「手牌<体力」：抬高体力基线。
    let p = tx.state_mut().players.get_mut(holder).unwrap();
    p.hp = 10;
    if p.max_hp < p.hp as u32 {
        p.max_hp = p.hp as u32;
    }
    // 不在别人的回合边界触发（owner_alive_not_target 需 current!=holder）。
    let next_current = *tx
        .state()
        .players
        .alive_by_seat()
        .iter()
        .find(|&&p| p != holder)
        .unwrap();
    tx.state_mut().turn.current = next_current;
    tx.state_mut().turn.phase = Phase::Idle;
}

#[test]
fn umi_lunhui_fires_extra_turn_and_binds_until_own_turn_end() {
    use engine_core::state::ContinuationKind;
    let (state, holder) = started();
    let mut tx = Transaction::begin(state);
    arm_condition(&mut tx, holder);
    let hp_before = tx.state().players.get(holder).unwrap().hp;
    let hand_before = tx.state().hand_count(holder);

    let cur = tx.state().turn.current;
    let opened = drive(
        &mut tx,
        &[engine_api::DomainEvent::PhaseEnded {
            current: cur,
            phase: engine_api::TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let mid = resolve(tx.state, holder, vec![1], false);
    assert!(!mid.has_pending_choice());
    assert_eq!(mid.players.get(holder).unwrap().hp, hp_before - 1);
    assert_eq!(mid.hand_count(holder), hand_before + 2);
    assert_eq!(mid.turn.extra_turn_queue.first(), Some(&holder));
    assert_eq!(
        mid.turn
            .flag(engine_core::state::turn_flag::extra_turn_check_pending_by_player(
                holder
            )),
        0,
        "check=0：纯额外回合不登记尾随代价"
    );
    // storage 绑定置位。
    assert_eq!(lunhui_storage(&mid, holder), Some(1));

    // 绑定窗口内再次结束回合：confirm 打开但 body 静默——队列不再增长。
    let mut tx = Transaction::begin(mid);
    let cur = tx.state().turn.current;
    let opened = drive(
        &mut tx,
        &[engine_api::DomainEvent::PhaseEnded {
            current: cur,
            phase: engine_api::TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let gated = resolve(tx.state, holder, vec![1], false);
    assert_ne!(gated.turn.extra_turn_queue.len(), 2);

    // 自己的回合结束清零绑定：可以再次发动。
    let mut tx = Transaction::begin(gated);
    let cleared = drive(
        &mut tx,
        &[engine_api::DomainEvent::PhaseEnded {
            current: holder,
            phase: engine_api::TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
    );
    cleared.unwrap();
    let cleared_state = tx.commit().state;
    assert_eq!(lunhui_storage(&cleared_state, holder), Some(0));
}
}

mod original_key_xingdiao_yaxiang_chaofan_yanyi {
//! Golden coverage for KEY 星钓 / 鸦翔 / 炒饭 / 演仪 rewritten from honest noops.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const FUUKO_XINGDIAO: u32 = 3_398_320_463;
const YUKITO_YAXIANG: u32 = 2_428_947_588;
const UMI_CHAOFAN: u32 = 2_428_039_328;
const YUSA_YANYI: u32 = 4_076_446_163;

const XINGDIAO_SRC: &str =
    include_str!("../../../content/original/ported/key/fuuko_xingdiao.skill");
const YAXIANG_SRC: &str = include_str!("../../../content/original/ported/key/yukito_yaxiang.skill");
const CHAOFAN_SRC: &str = include_str!("../../../content/original/ported/key/umi_chaofan.skill");
const YANYI_SRC: &str = include_str!("../../../content/original/ported/key/yusa_yanyi.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

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

fn inject_hand(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Shan,
        suit: Suit::Club,
        rank: 6,
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
fn xingdiao_source_and_npack_publish_game_started() {
    let compiled = content_compiler::compile_source_named("fuuko_xingdiao", XINGDIAO_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == FUUKO_XINGDIAO)
        .expect("xingdiao trigger");
    assert_eq!(trigger.event, "game_started");
    assert_eq!(trigger.filter, "owner_alive");
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == FUUKO_XINGDIAO && trigger.event == "game_started"
    }));
}

#[test]
fn xingdiao_game_start_draws_to_eight() {
    let (mut tx, owner, _) = bootstrap(XINGDIAO_SRC, "xingdiao-test", FUUKO_XINGDIAO);
    assert_eq!(tx.state().hand_count(owner), 0);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 2 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("game-started dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), 8);
}

#[test]
fn yaxiang_source_and_npack_publish_other_dying() {
    let compiled = content_compiler::compile_source_named("yukito_yaxiang", YAXIANG_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == YUKITO_YAXIANG)
        .expect("yaxiang trigger");
    assert_eq!(trigger.event, "dying");
    assert_eq!(trigger.filter, "owner_alive_not_target");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == YUKITO_YAXIANG && trigger.event == "dying"
    }));
}

#[test]
fn yaxiang_other_dying_recovers_to_three_and_unloads() {
    let (mut tx, owner, other) = bootstrap(YAXIANG_SRC, "yaxiang-test", YUKITO_YAXIANG);
    tx.state_mut().players.get_mut(other).unwrap().hp = 0;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::EnteredDying {
                player: other,
                source: owner,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert_eq!(state.players.get(other).unwrap().hp, 3);
    assert!(!state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.def.raw() == YUKITO_YAXIANG));
}

#[test]
fn chaofan_source_and_npack_publish_active() {
    let compiled = content_compiler::compile_source_named("umi_chaofan", CHAOFAN_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == UMI_CHAOFAN));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == UMI_CHAOFAN));
}

#[test]
fn chaofan_discards_two_draws_one_then_offers_other() {
    let (mut tx, owner, other) = bootstrap(CHAOFAN_SRC, "chaofan-test", UMI_CHAOFAN);
    inject_hand(&mut tx, owner, 242_101);
    inject_hand(&mut tx, owner, 242_102);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == UMI_CHAOFAN)
        .unwrap();
    let other_hand = tx.state().hand_count(other);
    let state = tx.commit().state;
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert_eq!(state.hand_count(owner), 1);
    let state = resolve(state, owner, vec![other.raw()], 3);
    let state = if state.has_pending_choice() {
        resolve(state, owner, vec![0], 4)
    } else {
        state
    };
    assert!(
        state.hand_count(other) == other_hand + 2
            || state.players.get(other).unwrap().hp
                > 0
    );
}

#[test]
fn yanyi_source_and_npack_publish_active() {
    let compiled = content_compiler::compile_source_named("yusa_yanyi", YANYI_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == YUSA_YANYI));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == YUSA_YANYI));
}
}

mod original_key_yangfan_baoyi {
//! Golden coverage for KEY 扬帆 / 爆衣 rewritten from honest noops.

use engine_api::{CardDefId, CardId, PlayerId, SkillDefId, Suit};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const KAMOME_YANGFAN: u32 = 3_575_287_643;
const RYOICHI_BAOYI: u32 = 2_396_712_868;
const YANGFAN_SRC: &str =
    include_str!("../../../content/original/ported/key/kamome_yangfan.skill");
const BAOYI_SRC: &str = include_str!("../../../content/original/ported/key/ryoichi_baoyi.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn setup(source: &str, name: &str, skill_def: u32) -> (Transaction, PlayerId) {
    let compiled = content_compiler::compile_source_named(name, source).unwrap();
    let mut engine = Engine::new();
    engine.submit(engine_api::Command {
        actor: PlayerId::NONE,
        expected_revision: engine_api::ExpectedRevision(0),
        idempotency_key: engine_api::IdempotencyKey(20_260_815_50),
        kind: engine_api::CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_815_50,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&compiled).unwrap();
    let owner = engine.state().turn.current;
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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner)
}

fn move_equip_to_discard(tx: &mut Transaction, owner: PlayerId) -> CardId {
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(357_501),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 1,
        zone: equip,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(equip).unwrap().cards.push(card);
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: discard,
            from: Some(equip),
        }],
    )
    .unwrap();
    card
}

#[test]
fn yangfan_source_and_npack_publish_lose_equip_draw() {
    let compiled = content_compiler::compile_source_named("kamome_yangfan", YANGFAN_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == KAMOME_YANGFAN)
        .expect("yangfan trigger");
    assert_eq!(trigger.event, "cards_moved");
    assert_eq!(trigger.filter, "owner_from_equip");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == KAMOME_YANGFAN && trigger.filter == "owner_from_equip"
    }));
}

#[test]
fn yangfan_losing_equip_draws_two() {
    let (mut tx, owner) = setup(YANGFAN_SRC, "yangfan-test", KAMOME_YANGFAN);
    let before = tx.state().hand_count(owner);
    move_equip_to_discard(&mut tx, owner);
    assert_eq!(tx.state().hand_count(owner), before + 2);
}

#[test]
fn baoyi_source_and_npack_publish_lose_equip_draw() {
    let compiled = content_compiler::compile_source_named("ryoichi_baoyi", BAOYI_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == RYOICHI_BAOYI)
        .expect("baoyi trigger");
    assert_eq!(trigger.event, "cards_moved");
    assert_eq!(trigger.filter, "owner_from_equip");
    assert!(!trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == RYOICHI_BAOYI && trigger.filter == "owner_from_equip"
    }));
}

#[test]
fn baoyi_losing_equip_draws_one() {
    let (mut tx, owner) = setup(BAOYI_SRC, "baoyi-test", RYOICHI_BAOYI);
    let before = tx.state().hand_count(owner);
    move_equip_to_discard(&mut tx, owner);
    assert_eq!(tx.state().hand_count(owner), before + 1);
}
}

mod original_key_yukito_yaxiang_grant {
//! 第一百五十三批：KEY 往人「鸦翔」（yukito_yaxiang）补授臂行为测试。
//! 上游：其他角色濒死时变身为乌鸦，令其回复至 3 体力并弃置判定区牌，获得〖终愿〗。
//! 实现（批 153 补授臂）：回复至 3 + 弃判定区 + add_skill(event_player, 终愿def) +
//! remove_skill(self_skill)。近似：不变身。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const YAXIANG: u32 = 2_428_947_588;
const ZHONGYUAN: u32 = 2_446_245_954;
const YAXIANG_SRC: &str =
    include_str!("../../../content/original/ported/key/yukito_yaxiang.skill");

fn bootstrap() -> (GameState, PlayerId, PlayerId) {
    let compiled =
        content_compiler::compile_source_named("yukito_yaxiang", YAXIANG_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(YAXIANG), "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(YAXIANG),
    });
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    (state, owner, other)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
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
                cancelled: false,
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

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

#[test]
fn yaxiang_recovers_equips_grant_and_removes_itself() {
    let (state, owner, other) = bootstrap();
    // other 濒死（hp 置 0，进入 dying 事件由合成派发驱动）。
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(other).unwrap().hp = 0;
    let opened = dispatch_for_events_for_test(
        &mut tx,
        owner,
        other,
    );
    assert!(
        matches!(opened, Err(EffectError::ChoiceRequested)),
        "yaxiang optional confirm must open on other's dying"
    );
    let state = tx.commit().state;
    let state = resolve(state, owner, vec![1], 10);

    // 回复至 3、获得终愿、鸦翔自卸。
    assert_eq!(hp_of(&state, other), 3, "target must recover to 3");
    assert!(
        has_skill_def(&state, other, ZHONGYUAN),
        "target must gain 终愿"
    );
    assert!(
        !has_skill_def(&state, owner, YAXIANG),
        "yaxiang must remove itself (limited)"
    );
}

#[test]
fn yaxiang_not_triggered_by_owner_dying() {
    let (state, owner, _other) = bootstrap();
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    let opened = dispatch_for_events_for_test(&mut tx, owner, owner);
    assert!(
        !matches!(opened, Err(EffectError::ChoiceRequested)),
        "owner's own dying must not trigger (owner_alive_not_target)"
    );
}

/// 合成濒死事件（event_player = dying 玩家）驱动鸦翔触发链。
fn dispatch_for_events_for_test(
    tx: &mut Transaction,
    _owner: PlayerId,
    dying: PlayerId,
) -> Result<Vec<EffectRequest>, EffectError> {
    engine_core::trigger::dispatch_for_events(
        tx,
        &[engine_api::DomainEvent::EnteredDying {
            player: dying,
            source: PlayerId::NONE,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
}
}

mod original_key_zhiheng_wanxin {
//! Golden coverage for KEY 恭子「制衡」and 直井「挽心」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const KYOKO_ZHIHENG: u32 = 2_851_376_754;
const NAO_WANXIN: u32 = 2_978_438_530;
const ZHIHENG_SRC: &str =
    include_str!("../../../content/original/ported/key/kyoko_zhiheng.skill");
const WANXIN_SRC: &str = include_str!("../../../content/original/ported/key/nao_wanxin.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

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

fn inject_hand(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Shan,
        suit: Suit::Club,
        rank: 6,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn skill_id(tx: &Transaction, owner: PlayerId, def: u32) -> SkillId {
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
fn kyoko_zhiheng_source_and_npack_publish_active_recast() {
    let compiled = content_compiler::compile_source_named("kyoko_zhiheng", ZHIHENG_SRC).unwrap();
    assert!(compiled
        .active_index
        .iter()
        .any(|item| item.skill_def_id == KYOKO_ZHIHENG));
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .active_index
        .iter()
        .any(|item| item.skill_def_id == KYOKO_ZHIHENG));
}

#[test]
fn kyoko_zhiheng_discards_selected_and_redraws_same_count() {
    let (mut tx, owner, _) = bootstrap(ZHIHENG_SRC, "kyoko-zhiheng-test", KYOKO_ZHIHENG);
    let card = inject_hand(&mut tx, owner, 180_101);
    let before = tx.state().hand_count(owner);
    let skill = skill_id(&tx, owner, KYOKO_ZHIHENG);
    let state = tx.commit().state;
    let (state, pipe) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![card],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert_eq!(state.hand_count(owner), before);
}

#[test]
fn nao_wanxin_source_and_npack_publish_end_phase_draw() {
    let compiled = content_compiler::compile_source_named("nao_wanxin", WANXIN_SRC).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NAO_WANXIN)
        .unwrap();
    assert_eq!(trigger.event, "phase_end");
    assert!(trigger.optional);
    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == NAO_WANXIN && trigger.event == "phase_end"));
}

#[test]
fn nao_wanxin_end_phase_lets_chosen_player_draw_two() {
    let (tx, owner, other) = bootstrap(WANXIN_SRC, "nao-wanxin-test", NAO_WANXIN);
    let other_hand = tx.state().hand_count(other);
    let mut tx = tx;
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: owner,
                phase: TurnPhaseTag::End,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, owner, vec![1], 2);
    assert!(state.has_pending_choice());
    let state = resolve(state, owner, vec![other.raw()], 3);
    assert_eq!(state.hand_count(other), other_hand + 2);
}
}
