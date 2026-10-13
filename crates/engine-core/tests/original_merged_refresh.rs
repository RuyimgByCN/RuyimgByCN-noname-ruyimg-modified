//! O2 全量合并批次 1（2026-10-14）：家族 refresh 的 50 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_refresh_botu {
//! refresh/botu 博图（第八十五批·跨包复用第二消费者）：本回合四花色齐 →
//! 队列一个纯额外回合（queue_extra_turn check=0）。suit 历史由引擎 CardUsed
//! 路径自动记录（包 ABI≥232 opt-in）。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const BOTU: u32 = 3_144_154_445;

const SOURCES_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../content/original/ported/refresh"
);

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

fn started() -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("refresh-botu", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_831, "identity-standard").unwrap();
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
            skill_def: SkillDefId::new(BOTU),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}

fn inject_suit_card(tx: &mut Transaction, owner: PlayerId, suit: Suit, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: engine_core::state::CardKind::Tao,
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

#[test]
fn botu_publishes_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../content/original/ported/refresh/refresh.npack.json"
    ))
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 723);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == BOTU
            && t.event == "turn_ended"
            && t.filter == "owner_alive_turn_used_all_suits"
            && t.optional
    }));
}

#[test]
fn botu_queues_extra_turn_after_fourth_suit_turn_end() {
    let (state, holder) = started();
    let mut tx = Transaction::begin(state);

    // 四张不同花色的桃，逐一合成 CardUsed——引擎路径回填位掩码。
    let suits = [Suit::Spade, Suit::Heart, Suit::Club, Suit::Diamond];
    for (index, suit) in suits.iter().enumerate() {
        let card = inject_suit_card(&mut tx, holder, *suit, 5 + index as u8);
        drive(
            &mut tx,
            &[engine_api::DomainEvent::CardUsed {
                player: holder,
                card,
                targets: vec![],
            }],
        )
        .unwrap();

        if index < 3 {
            // 未齐四花色：回合结束不触发。
            let mut probe_tx = Transaction::begin(tx.commit().state);
            let cur = probe_tx.state().turn.current;
            let quiet = drive(
                &mut probe_tx,
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
            quiet.unwrap();
            assert!(!probe_tx.state().has_pending_choice());
            tx = probe_tx;
        }
    }

    // 第四花色后回合结束：confirm → 入队纯额外回合。
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
    let final_state = resolve(tx.state, holder, vec![1], 801);
    assert_eq!(final_state.turn.extra_turn_queue.first(), Some(&holder));
    assert_eq!(
        final_state
            .turn
            .flag(engine_core::state::turn_flag::extra_turn_check_pending_by_player(
                holder
            )),
        0,
        "check=0：博图无尾随代价"
    );
}
}

mod original_refresh_dcqiaomeng {
//! Golden coverage for REFRESH 公孙瓒「趫猛」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DC_GONGSUNZAN: u32 = 2_326_980_049;
const DCYICONG: u32 = 3_742_694_356;
const DCQIAOMENG: u32 = 3_871_427_916;
const DCQIAOMENG_EFFECT: u32 = 2_662_675_115;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/dcqiaomeng.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("dcqiaomeng-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_730_313, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend([
        EffectRequest::AddSkill {
            player: seats[0],
            skill_def: SkillDefId::new(DCQIAOMENG),
        },
        EffectRequest::SetTurnState {
            current: seats[0],
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, seats[0], seats[1], seats[2])
}

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
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn play(
    state: GameState,
    owner: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult) {
    command(
        state,
        owner,
        CommandKind::PlayCard {
            card,
            targets,
            card_targets: vec![],
        },
        key,
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

fn discard_contains(state: &GameState, card: CardId) -> bool {
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    state.zones.get(discard).unwrap().cards.contains(&card)
}

fn choose_target_and_card(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    selected: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    let (state, result) = resolve(state, owner, vec![1], key);
    assert_eq!(
        result.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets
    );
    let (state, result) = resolve(state, owner, vec![target.raw()], key + 1);
    assert_eq!(
        result.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    resolve(state, owner, vec![selected.raw()], key + 2)
}

#[test]
fn package_publishes_dc_gongsunzan_and_exact_abi313_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 313);
    assert!(package.character_index.iter().any(|character| {
        character.id == DC_GONGSUNZAN
            && character.name_key == "dc_gongsunzan"
            && character.max_hp == 4
            && character.skills == [DCYICONG, DCQIAOMENG]
    }));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCQIAOMENG)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional
        ),
        (
            "card_targeted",
            "owner_source_first_target_black_card_any_target_other_discardable_hand_equip",
            true
        )
    );
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == DCQIAOMENG_EFFECT
            && modifier.target == "card_use_unrespondable_for_skill_card_policy"
    }));
}

#[test]
fn equipment_is_gained_and_discarded_trick_makes_the_sha_unrespondable() {
    let (state, owner, target, _) = setup();
    let mut tx = Transaction::begin(state);
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        313_001,
    );
    let weapon = inject(
        &mut tx,
        target,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Diamond,
        313_002,
    );
    let hp = tx.state().players.get(target).unwrap().hp;
    let (offered, result) = play(tx.commit().state, owner, sha, vec![target], 1);
    assert_eq!(
        result.choice.as_ref().map(|choice| choice.kind),
        Some(ChoiceKind::Confirm),
        "{result:?}"
    );
    let (resolved, result) = choose_target_and_card(offered, owner, target, weapon, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(resolved.hand_cards(owner).contains(&weapon));
    assert_eq!(resolved.players.get(target).unwrap().hp, hp - 1);

    let (state, owner, target, _) = setup();
    let mut tx = Transaction::begin(state);
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Club,
        313_011,
    );
    let _shan = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Heart,
        313_012,
    );
    let trick = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::TrickGuohe,
        Suit::Diamond,
        313_013,
    );
    let hp = tx.state().players.get(target).unwrap().hp;
    let (offered, _) = play(tx.commit().state, owner, sha, vec![target], 11);
    let (resolved, result) = choose_target_and_card(offered, owner, target, trick, 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert!(discard_contains(&resolved, trick));
    assert_eq!(resolved.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn basic_keeps_normal_response_red_is_rejected_and_all_targets_are_available_once() {
    let (state, owner, target, _) = setup();
    let mut tx = Transaction::begin(state);
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        313_021,
    );
    let shan = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Heart,
        313_022,
    );
    let tao = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Tao,
        Suit::Heart,
        313_023,
    );
    let hp = tx.state().players.get(target).unwrap().hp;
    let (offered, _) = play(tx.commit().state, owner, sha, vec![target], 21);
    let (responding, result) = choose_target_and_card(offered, owner, target, tao, 22);
    assert_eq!(result.choice.as_ref().unwrap().observer, target);
    let (resolved, result) = resolve(responding, target, vec![shan.raw()], 25);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(discard_contains(&resolved, tao));
    assert_eq!(resolved.players.get(target).unwrap().hp, hp);

    let (state, owner, target, _) = setup();
    let mut tx = Transaction::begin(state);
    let red_sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Heart,
        313_031,
    );
    inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Tao,
        Suit::Club,
        313_032,
    );
    let (_, result) = play(tx.commit().state, owner, red_sha, vec![target], 31);
    assert!(result.choice.is_none());

    let (state, owner, first, second) = setup();
    let mut tx = Transaction::begin(state);
    let trick = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::TrickTiaojiyanmei,
        Suit::Club,
        313_041,
    );
    let first_card = inject(
        &mut tx,
        first,
        ZoneKind::Hand,
        CardKind::Tao,
        Suit::Heart,
        313_042,
    );
    let second_card = inject(
        &mut tx,
        second,
        ZoneKind::Hand,
        CardKind::Tao,
        Suit::Diamond,
        313_043,
    );
    inject(
        &mut tx,
        second,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Heart,
        313_044,
    );
    let (offered, result) = play(tx.commit().state, owner, trick, vec![first, second], 41);
    assert_eq!(
        result.choice.as_ref().map(|choice| choice.kind),
        Some(ChoiceKind::Confirm),
        "{result:?}"
    );
    let (selecting_target, result) = resolve(offered, owner, vec![1], 42);
    let choice = result.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == first.raw()));
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == second.raw()));
    let (selecting_card, result) = resolve(selecting_target, owner, vec![second.raw()], 43);
    assert_eq!(
        result.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let (resolved, result) = resolve(selecting_card, owner, vec![second_card.raw()], 44);
    assert!(discard_contains(&resolved, second_card));
    assert!(resolved.hand_cards(first).contains(&first_card));
    assert!(!result
        .choice
        .as_ref()
        .is_some_and(|choice| choice.observer == owner && choice.kind == ChoiceKind::Confirm));
}
}

mod original_refresh_dcshibei {
//! Golden for 界限突破「矢北」rewritten from the legacy JavaScript semantics.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DCSHIBEI: u32 = 2_817_420_663;
const RE_JUSHOU: u32 = 3_303_423_936;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

#[test]
fn package_contains_locked_first_and_second_damage_branches_and_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == DCSHIBEI && skill.locked));
    assert_eq!(
        package
            .trigger_index
            .iter()
            .filter(|trigger| trigger.skill_def_id == DCSHIBEI)
            .map(|trigger| trigger.filter.as_str())
            .collect::<Vec<_>>(),
        [
            "owner_is_target_first_damage",
            "owner_is_target_second_damage"
        ]
    );
    assert!(package.character_index.iter().any(|character| {
        character.id == RE_JUSHOU && character.skills == [2_862_591_465, DCSHIBEI]
    }));
}

#[test]
fn only_first_and_second_damage_events_have_follow_up_effects() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_716_21),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_716_21,
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCSHIBEI),
    });
    run_effects(&mut tx, effects).unwrap();
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.max_hp = 6;
        player.hp = 5;
    }

    let damage = || EffectRequest::DealDamage {
        source,
        target: owner,
        amount: 1,
        is_lose_hp: false,
        nature: 0,
    };
    run_effects(&mut tx, vec![damage()]).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 5);

    run_effects(&mut tx, vec![damage()]).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 3);

    run_effects(&mut tx, vec![damage()]).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCSHIBEI)
        .unwrap();
    assert_eq!(skill.turn_fire_count, 2);
}
}

mod original_refresh_decadejingce {
//! Golden coverage for 界郭淮「精策」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DECADEJINGCE: u32 = 2_705_593_131;
const RE_GUOHUAI: u32 = 3_763_685_284;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/decadejingce.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn inject(state: &mut GameState, owner: PlayerId, def: u32, suit: Suit) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Jiu,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn setup(seed: u64, suits: &[Suit]) -> (GameState, PlayerId, PlayerId, Vec<CardId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().revision = 1;
    let owner = tx.state().turn.current;
    let next = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.character_def = RE_GUOHUAI;
        player.max_hp = 2;
        player.hp = 2;
    }
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DECADEJINGCE),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let cards = suits
        .iter()
        .enumerate()
        .map(|(index, suit)| inject(tx.state_mut(), owner, 232_000 + index as u32, *suit))
        .collect();
    (tx.commit().state, owner, next, cards)
}

fn command(
    state: GameState,
    actor: PlayerId,
    key: u64,
    kind: CommandKind,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn play(state: GameState, actor: PlayerId, card: CardId, key: u64) -> GameState {
    let (state, result) = command(
        state,
        actor,
        key,
        CommandKind::PlayCard {
            card,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn end(state: GameState, actor: PlayerId, key: u64) -> (GameState, PipelineResult) {
    command(state, actor, key, CommandKind::EndPhase)
}

fn choose(state: GameState, actor: PlayerId, token: u32, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = command(
        state,
        actor,
        key,
        CommandKind::ResolveChoice {
            choice_id,
            selections: vec![token],
            cancelled: false,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn reach_end(state: GameState, owner: PlayerId, key: u64) -> (GameState, PipelineResult) {
    let (state, play_end) = end(state, owner, key);
    assert!(matches!(play_end.result, CommandResult::Accepted { .. }));
    let (state, discard_end) = end(state, owner, key + 1);
    assert!(matches!(discard_end.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::End);
    (state, discard_end)
}

#[test]
fn package_publishes_rewritten_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 251);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_GUOHUAI)
        .unwrap();
    assert_eq!(character.skills, [DECADEJINGCE]);
    let compiled = content_compiler::compile_source_named("decadejingce", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 232);
}

#[test]
fn insufficient_suits_choose_one_complete_extra_phase() {
    let (mut state, owner, next, cards) = setup(2_026_072_801, &[Suit::Spade, Suit::Spade]);
    for (index, card) in cards.into_iter().enumerate() {
        state = play(state, owner, card, 10 + index as u64);
    }
    let (state_after_end, optional) = reach_end(state, owner, 20);
    assert!(optional.choice.is_some());
    let state = choose(state_after_end, owner, 1, 22);
    assert!(state.has_pending_choice());
    let hand_before = state.hand_count(owner);
    let state = choose(state, owner, 1, 23);
    assert_eq!(state.turn.phase, Phase::End);
    let (state, _) = end(state, owner, 24);
    assert_eq!(state.turn.current, owner);
    assert_eq!(state.turn.phase, Phase::Draw);
    assert_eq!(state.hand_count(owner), hand_before + 2);

    let (state, result) = end(state, owner, 25);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.current, next);
    assert_ne!(state.turn.phase, Phase::Play);
}

#[test]
fn enough_suits_automatically_runs_draw_then_play() {
    let (mut state, owner, next, cards) = setup(2_026_072_802, &[Suit::Spade, Suit::Heart]);
    for (index, card) in cards.into_iter().enumerate() {
        state = play(state, owner, card, 30 + index as u64);
    }
    let (state, optional) = reach_end(state, owner, 40);
    assert!(optional.choice.is_some());
    let hand_before = state.hand_count(owner);
    let state = choose(state, owner, 1, 42);
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.phase, Phase::End);
    let (state, _) = end(state, owner, 43);
    assert_eq!(state.turn.phase, Phase::Draw);
    assert_eq!(state.hand_count(owner), hand_before + 2);

    let (state, _) = end(state, owner, 44);
    assert_eq!(state.turn.current, owner);
    assert_eq!(state.turn.phase, Phase::Play);
    let (state, _) = end(state, owner, 45);
    assert_eq!(state.turn.current, next);
}

#[test]
fn card_count_below_hp_does_not_offer_jingce() {
    let (state, owner, next, cards) = setup(2_026_072_803, &[Suit::Diamond]);
    let state = play(state, owner, cards[0], 50);
    let (state, result) = reach_end(state, owner, 51);
    assert!(result.choice.is_none());
    let (state, result) = end(state, owner, 53);
    assert!(result.choice.is_none());
    assert_eq!(state.turn.current, next);
}
}

mod original_refresh_decadezhenjun_yujin_yujin {
//! Golden coverage for 界于禁「镇军」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const YUJIN_YUJIN: u32 = 3_873_317_599;
const DECADEZHENJUN: u32 = 2_634_318_954;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/decadezhenjun.skill");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const LEGACY_REZHENJUN_SOURCE: &str =
    include_str!("../../../content/original/ported/yijiang/rezhenjun.skill");
const LEGACY_REZHENJUN_LOGIC_HASH: [u8; 32] = [
    111, 13, 76, 138, 108, 127, 148, 182, 111, 104, 225, 58, 129, 205, 156, 80, 118, 110, 160, 1,
    127, 135, 68, 171, 236, 217, 252, 46, 248, 205, 119, 131,
];
const PREPARE_LOGIC_HASH: [u8; 32] = [
    67, 78, 214, 35, 141, 175, 103, 224, 23, 144, 140, 58, 237, 221, 126, 20, 185, 154, 185, 71,
    96, 39, 131, 225, 17, 76, 168, 77, 167, 234, 8, 131,
];
const END_LOGIC_HASH: [u8; 32] = [
    179, 87, 145, 107, 121, 213, 84, 251, 60, 113, 203, 232, 177, 247, 173, 173, 187, 152, 252,
    151, 223, 98, 215, 32, 184, 29, 214, 229, 62, 249, 232, 90,
];

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
}

fn setup(seed: u64, phase: Phase) -> Setup {
    let package = content_compiler::compile_source_named("decadezhenjun-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
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
        skill_def: SkillDefId::new(DECADEZHENJUN),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in players {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.hp = 4;
        participant.max_hp = 4;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = phase;
    tx.state_mut().revision = 1;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|instance| instance.owner == owner && instance.def.raw() == DECADEZHENJUN)
        .unwrap()
        .id;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
    }
}

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let command = Command {
        actor,
        expected_revision: ExpectedRevision(state.revision),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(state.resolution.pending_choice_id),
            selections,
            cancelled,
        },
    };
    apply_command(state, command)
}

fn open_effect(setup: &Setup) -> GameState {
    let mut tx = Transaction::begin(setup.state.clone());
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDecadezhenjun {
                player: setup.owner,
                target: PlayerId::NONE,
                stage: 0,
                count: 0,
                skill: setup.skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw
    );
    assert_eq!(state.resolution.continuation_payload[0], 3);
    state
}

fn select_target(state: GameState, setup: &Setup, key: u64) -> GameState {
    let (state, result) = resolve(state, setup.owner, vec![setup.target.raw()], false, key);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        result.choice.as_ref().map(|choice| choice.kind),
        Some(ChoiceKind::SelectCards)
    );
    assert_eq!(state.resolution.continuation_payload[0], 4);
    state
}

fn fire_phase(mut setup: Setup, phase: TurnPhaseTag, key: u64) -> GameState {
    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: setup.owner,
                phase,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    setup.state = tx.commit().state;
    let (state, result) = resolve(setup.state, setup.owner, vec![1], false, key);
    assert!(result
        .choice
        .as_ref()
        .is_some_and(|choice| choice.kind == ChoiceKind::SelectTargets));
    state
}

#[test]
fn package_publishes_complete_character_and_freezes_legacy_rezhenjun_contract() {
    let compiled = content_compiler::compile_source_named("decadezhenjun", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 298);
    assert_eq!(compiled.trigger_index.len(), 2);
    assert!(compiled.trigger_index.iter().all(|trigger| {
        trigger.optional
            && trigger.filter == "owner_is_target_any_alive_discardable_by_owner_hand_equip"
    }));
    assert!(compiled
        .trigger_index
        .iter()
        .any(|trigger| trigger.event == "phase_prepare"));
    assert!(compiled
        .trigger_index
        .iter()
        .any(|trigger| trigger.event == "phase_end"));
    let prepare = compiled
        .modules
        .iter()
        .find(|module| module.name == "decadezhenjun_phase_prepare")
        .unwrap();
    assert_eq!(prepare.code, [83, 90, 29, 1, 1]);
    assert_eq!(prepare.logic_hash, PREPARE_LOGIC_HASH);
    let end = compiled
        .modules
        .iter()
        .find(|module| module.name == "decadezhenjun_phase_end")
        .unwrap();
    assert_eq!(end.code, [83, 90, 29, 1, 1]);
    assert_eq!(end.logic_hash, END_LOGIC_HASH);

    let package = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 298);
    assert!(package
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "yijiang"));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == YUJIN_YUJIN)
        .unwrap();
    assert_eq!(character.name_key, "yujin_yujin");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [DECADEZHENJUN]);
    assert!(compiled.modules.iter().all(|expected| {
        package
            .modules
            .iter()
            .any(|actual| actual.name == expected.name && actual.logic_hash == expected.logic_hash)
    }));

    let legacy =
        content_compiler::compile_source_named("legacy-rezhenjun", LEGACY_REZHENJUN_SOURCE)
            .unwrap();
    assert_eq!(legacy.manifest.rule_vm_abi_version, 62);
    assert_eq!(legacy.modules.len(), 1);
    assert_eq!(legacy.modules[0].code, [83, 80, 91, 1]);
    assert_eq!(legacy.modules[0].logic_hash, LEGACY_REZHENJUN_LOGIC_HASH);
}

#[test]
fn prepare_and_end_filters_require_a_legal_target_and_include_self() {
    let empty = setup(2_026_073_001, Phase::Prepare);
    let mut tx = Transaction::begin(empty.state.clone());
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: empty.owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());

    let mut prepare = empty;
    inject_hand(&mut prepare.state, prepare.owner, 99_801, CardKind::Other);
    let owner = prepare.owner;
    let state = fire_phase(prepare, TurnPhaseTag::Prepare, 1);
    assert_eq!(state.resolution.continuation_payload[0], 3);
    assert!(state
        .resolution
        .continuation_payload
        .get(4..)
        .is_some_and(|targets| targets.contains(&owner.raw())));

    let mut end = setup(2_026_073_002, Phase::End);
    inject_hand(&mut end.state, end.owner, 99_802, CardKind::Other);
    let state = fire_phase(end, TurnPhaseTag::End, 2);
    assert_eq!(state.resolution.continuation_payload[0], 3);
}

#[test]
fn dynamic_target_count_and_any_equipment_finish_without_owner_branch() {
    let mut setup = setup(2_026_073_003, Phase::Prepare);
    setup.state.players.get_mut(setup.target).unwrap().hp = 1;
    let ordinary = inject_hand(&mut setup.state, setup.target, 99_811, CardKind::Other);
    let equipment = inject_hand(&mut setup.state, setup.target, 99_812, CardKind::Weapon);
    let remaining = inject_hand(&mut setup.state, setup.target, 99_813, CardKind::Other);
    let state = select_target(open_effect(&setup), &setup, 10);
    assert_eq!(state.resolution.continuation_payload[4], 2);
    let (state, result) = resolve(
        state,
        setup.owner,
        vec![ordinary.raw(), equipment.raw()],
        false,
        11,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_cards(setup.target), [remaining]);
}

#[test]
fn all_non_equipment_requires_one_owner_card_while_cancel_or_inability_draws_n() {
    let mut paid = setup(2_026_073_004, Phase::End);
    paid.state.players.get_mut(paid.target).unwrap().hp = 1;
    let first = inject_hand(&mut paid.state, paid.target, 99_821, CardKind::Other);
    let second = inject_hand(&mut paid.state, paid.target, 99_822, CardKind::Other);
    inject_hand(&mut paid.state, paid.target, 99_823, CardKind::Other);
    let cost = inject_hand(&mut paid.state, paid.owner, 99_824, CardKind::Other);
    let state = select_target(open_effect(&paid), &paid, 20);
    let (state, result) = resolve(
        state,
        paid.owner,
        vec![first.raw(), second.raw()],
        false,
        21,
    );
    assert_eq!(
        result.choice.as_ref().map(|choice| (
            choice.kind,
            choice.min_count,
            choice.max_count,
            choice.allow_cancel
        )),
        Some((ChoiceKind::SelectCards, 1, 1, true))
    );
    assert_eq!(state.resolution.continuation_payload[0], 5);
    assert_eq!(state.resolution.continuation_payload[4], 2);
    let (state, result) = resolve(state, paid.owner, vec![cost.raw()], false, 22);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(paid.owner), 0);
    assert_eq!(state.hand_count(paid.target), 1);

    let mut cancelled = setup(2_026_073_005, Phase::Prepare);
    cancelled
        .state
        .players
        .get_mut(cancelled.target)
        .unwrap()
        .hp = 1;
    let first = inject_hand(
        &mut cancelled.state,
        cancelled.target,
        99_831,
        CardKind::Other,
    );
    let second = inject_hand(
        &mut cancelled.state,
        cancelled.target,
        99_832,
        CardKind::Other,
    );
    inject_hand(
        &mut cancelled.state,
        cancelled.target,
        99_833,
        CardKind::Other,
    );
    inject_hand(
        &mut cancelled.state,
        cancelled.owner,
        99_834,
        CardKind::Other,
    );
    let state = select_target(open_effect(&cancelled), &cancelled, 30);
    let (state, _) = resolve(
        state,
        cancelled.owner,
        vec![first.raw(), second.raw()],
        false,
        31,
    );
    let before = canonical_state_hash(&state);
    let (state, forged) = resolve(state, cancelled.owner, vec![9_999_999], false, 32);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
    let (state, result) = resolve(state, cancelled.owner, Vec::new(), true, 33);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(cancelled.target), 3);

    let mut unable = setup(2_026_073_006, Phase::Prepare);
    unable.state.players.get_mut(unable.target).unwrap().hp = 1;
    let first = inject_hand(&mut unable.state, unable.target, 99_841, CardKind::Other);
    let second = inject_hand(&mut unable.state, unable.target, 99_842, CardKind::Other);
    inject_hand(&mut unable.state, unable.target, 99_843, CardKind::Other);
    let state = select_target(open_effect(&unable), &unable, 40);
    let (state, result) = resolve(
        state,
        unable.owner,
        vec![first.raw(), second.raw()],
        false,
        41,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(unable.target), 3);
}
}

mod original_refresh_fenwei {
//! Golden coverage for 界甘宁「奋威」rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_GANNING: u32 = 2_643_035_493;
const QIXI: u32 = 27;
const FENWEI: u32 = 3_203_648_912;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/fenwei.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    actor: PlayerId,
    lower: PlayerId,
    higher: PlayerId,
    owner: PlayerId,
    trick: CardId,
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64) -> Setup {
    let package = compile_source_named("refresh-fenwei-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (actor, lower, higher, owner) = (seats[0], seats[1], seats[2], seats[3]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(FENWEI),
        },
        EffectRequest::SetTurnState {
            current: actor,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    inject(tx.state_mut(), lower, 99_301, CardKind::Other);
    for def in 99_302..99_305 {
        inject(tx.state_mut(), higher, def, CardKind::Other);
    }
    let trick = inject(tx.state_mut(), actor, 99_305, CardKind::TrickTiaojiyanmei);
    Setup {
        state: tx.commit().state,
        actor,
        lower,
        higher,
        owner,
        trick,
    }
}

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn play(setup: Setup, key: u64) -> (Setup, PipelineResult) {
    let (state, output) = command(
        setup.state,
        setup.actor,
        CommandKind::PlayCard {
            card: setup.trick,
            targets: vec![setup.lower, setup.higher],
            card_targets: Vec::new(),
        },
        key,
    );
    (Setup { state, ..setup }, output)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn has_fenwei(state: &GameState, owner: PlayerId) -> bool {
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
                .is_some_and(|skill| skill.def.raw() == FENWEI)
        })
}

#[test]
fn package_closes_re_ganning_without_legacy_javascript() {
    let compiled = compile_source_named("refresh-fenwei", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 308);
    assert_eq!(
        rule_vm::bytecode::effect_tag::CHOOSE_CURRENT_TRICK_TARGETS_TO_NULLIFY,
        292
    );

    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_GANNING)
        .unwrap();
    assert_eq!(character.skills, [QIXI, FENWEI]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == FENWEI));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn selected_targets_are_nullified_and_the_limited_skill_is_consumed() {
    let (setup, played) = play(setup(20_260_730_308), 1);
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    let choice = played.choice.unwrap();
    assert_eq!(choice.observer, setup.owner);
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!((choice.min_count, choice.max_count), (1, 2));
    assert!(choice.allow_cancel);

    let (state, resolved) = resolve(setup.state, setup.owner, vec![setup.higher.raw()], false, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(resolved.choice.is_none());
    assert_eq!(state.hand_count(setup.lower), 2);
    assert_eq!(state.hand_count(setup.higher), 3);
    assert!(!has_fenwei(&state, setup.owner));
}

#[test]
fn cancellation_keeps_fenwei_and_allows_the_original_trick_to_resolve() {
    let (setup, played) = play(setup(20_260_730_309), 3);
    assert!(played.choice.is_some());
    let (state, resolved) = resolve(setup.state, setup.owner, Vec::new(), true, 4);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.lower), 2);
    assert!(has_fenwei(&state, setup.owner));
    assert_eq!(resolved.choice.unwrap().observer, setup.higher);
}
}

mod original_refresh_jiezhong {
//! Golden for REFRESH 郭淮「精忠」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const JIEZHONG: u32 = 2_811_249_808;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn isolated(max_hp: u32) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(JIEZHONG as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: JIEZHONG as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
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
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(JIEZHONG),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let player = tx.state_mut().players.get_mut(owner).unwrap();
    player.max_hp = max_hp;
    player.hp = player.hp.min(max_hp as i32);
    (tx, owner)
}

fn offer(tx: &mut Transaction, owner: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn resolve(
    tx: &Transaction,
    owner: PlayerId,
    accept: bool,
    key: u64,
) -> engine_core::state::GameState {
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![u32::from(accept)],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn has_jiezhong(state: &engine_core::state::GameState, owner: PlayerId) -> bool {
    state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == JIEZHONG)
}

#[test]
fn committed_package_exposes_optional_limited_play_phase_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == JIEZHONG)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional,
        ),
        ("phase_play", "owner_is_target_hand_below_max_hp", true)
    );
}

#[test]
fn accepting_draws_to_max_hp_and_consumes_the_skill() {
    let (mut tx, owner) = isolated(3);
    assert_eq!(offer(&mut tx, owner), Err(EffectError::ChoiceRequested));
    let state = resolve(&tx, owner, true, 1);
    assert_eq!(state.hand_count(owner), 3);
    assert!(!has_jiezhong(&state, owner));
}

#[test]
fn draw_is_capped_at_five_and_declining_does_not_consume() {
    let (mut capped, owner) = isolated(8);
    assert_eq!(offer(&mut capped, owner), Err(EffectError::ChoiceRequested));
    let state = resolve(&capped, owner, true, 2);
    assert_eq!(state.hand_count(owner), 5);
    assert!(!has_jiezhong(&state, owner));

    let (mut declined, owner) = isolated(4);
    assert_eq!(
        offer(&mut declined, owner),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(&declined, owner, false, 3);
    assert_eq!(state.hand_count(owner), 0);
    assert!(has_jiezhong(&state, owner));
}
}

mod original_refresh_new_reluoyi_re_xuzhu {
//! Golden coverage for 界许褚「裸衣」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_XUZHU: u32 = 4_251_485_779;
const NEW_RELUOYI: u32 = 3_935_960_913;
const RELUOYI2: u32 = 3_768_135_792;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/new_reluoyi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    top: Vec<CardId>,
}

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
        def: CardDefId::new(99_000 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64, top_kinds: [CardKind; 3]) -> Setup {
    let package = content_compiler::compile_source_named("new-reluoyi-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
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
        skill_def: SkillDefId::new(NEW_RELUOYI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Judge;
    let top = top_kinds
        .into_iter()
        .enumerate()
        .map(|(index, kind)| {
            inject(
                &mut tx,
                PlayerId::NONE,
                ZoneKind::DrawPile,
                kind,
                [Suit::Spade, Suit::Heart, Suit::Club][index],
                10 + index as u8,
            )
        })
        .collect();
    Setup {
        state: tx.commit().state,
        owner,
        top,
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

fn open_choice(setup: &Setup) -> (GameState, PipelineResult) {
    let (state, result) = submit(setup.state.clone(), setup.owner, CommandKind::EndPhase, 1);
    let choice = result.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::Confirm);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    (state, result)
}

fn resolve_bool(
    state: GameState,
    owner: PlayerId,
    accepted: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id,
            selections: vec![u32::from(accepted)],
            cancelled: false,
        },
        key,
    )
}

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
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

#[test]
fn package_publishes_complete_re_xuzhu_and_abi317_module() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 317);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_XUZHU)
        .unwrap();
    assert_eq!(character.skills, [NEW_RELUOYI]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NEW_RELUOYI)
        .unwrap();
    assert_eq!(trigger.event, "phase_draw");
    assert_eq!(trigger.filter, "owner_is_target_draw_not_replaced");
    assert!(!trigger.optional);
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == RELUOYI2 && modifier.target == "sha_damage" && modifier.value == 1
    }));

    let compiled = content_compiler::compile_source_named("new_reluoyi", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 317);
    for expected in &compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn accepting_gains_basic_duel_and_weapon_and_expires_buff_next_owner_turn() {
    let setup = setup(
        2_026_073_301,
        [CardKind::Sha, CardKind::TrickJuedou, CardKind::Weapon],
    );
    let (state, _) = open_choice(&setup);
    let (mut state, result) = resolve_bool(state, setup.owner, true, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.draw_phase_fixed_count(), Some(0));
    assert!(setup
        .top
        .iter()
        .all(|card| state.hand_cards(setup.owner).contains(card)));
    assert!(has_skill(&state, setup.owner, RELUOYI2));
    assert_eq!(
        engine_core::query::sha_damage_for(&state, setup.owner, 1),
        2
    );

    for key in 3..20 {
        if state.turn.current == setup.owner && state.turn.phase == Phase::Prepare {
            break;
        }
        let current = state.turn.current;
        let (next, advanced) = submit(state, current, CommandKind::EndPhase, key);
        assert!(matches!(advanced.result, CommandResult::Accepted { .. }));
        assert!(advanced.choice.is_none());
        state = next;
    }
    assert_eq!(state.turn.current, setup.owner);
    assert_eq!(state.turn.phase, Phase::Prepare);
    assert!(!has_skill(&state, setup.owner, RELUOYI2));
    assert_eq!(
        engine_core::query::sha_damage_for(&state, setup.owner, 1),
        1
    );
}

#[test]
fn accepting_discards_cards_outside_the_eligible_categories() {
    let setup = setup(
        2_026_073_302,
        [CardKind::Tao, CardKind::Armor, CardKind::TrickWuzhong],
    );
    let eligible = setup.top[0];
    let rejected = [setup.top[1], setup.top[2]];
    let (state, _) = open_choice(&setup);
    let (state, _) = resolve_bool(state, setup.owner, true, 2);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state.hand_cards(setup.owner).contains(&eligible));
    assert!(rejected
        .iter()
        .all(|card| state.zones.get(discard).unwrap().cards.contains(card)));
}

#[test]
fn declining_discards_all_revealed_cards_and_keeps_the_normal_draw() {
    let setup = setup(
        2_026_073_303,
        [CardKind::Sha, CardKind::TrickJuedou, CardKind::Weapon],
    );
    let (state, _) = open_choice(&setup);
    let (state, result) = resolve_bool(state, setup.owner, false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), 2);
    assert!(!has_skill(&state, setup.owner, RELUOYI2));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(setup
        .top
        .iter()
        .all(|card| state.zones.get(discard).unwrap().cards.contains(card)));
}
}

mod original_refresh_new_retuxi {
//! Golden coverage for 界张辽「突袭」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_ZHANGLIAO: u32 = 3_345_021_499;
const NEW_RETUXI: u32 = 3_295_460_539;
const DRAW_THREE: u32 = 4_000_000_311;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/new_retuxi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    first: PlayerId,
    second: PlayerId,
    third: PlayerId,
}

fn setup(seed: u64) -> Setup {
    setup_inner(seed, false)
}

fn setup_with_draw_bonus(seed: u64) -> Setup {
    setup_inner(seed, true)
}

fn setup_inner(seed: u64, draw_bonus: bool) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    if draw_bonus {
        let bonus = content_compiler::compile_source_named(
            "new-retuxi-draw-bonus",
            "skill draw_three(4000000311) { modifier draw_count add 1; }",
        )
        .unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&bonus).unwrap();
    }
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(NEW_RETUXI),
    });
    if draw_bonus {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DRAW_THREE),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Judge;
    Setup {
        state: tx.commit().state,
        owner,
        first: players[1],
        second: players[2],
        third: players[3],
    }
}

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn open_optional(setup: &Setup) -> GameState {
    let (state, result) = command(setup.state.clone(), setup.owner, CommandKind::EndPhase, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Draw);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    state
}

#[test]
fn package_publishes_complete_re_zhangliao_and_exact_abi311_module() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 311);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_ZHANGLIAO)
        .unwrap();
    assert_eq!(character.skills, [NEW_RETUXI]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NEW_RETUXI)
        .unwrap();
    assert_eq!(trigger.event, "phase_draw");
    assert_eq!(
        trigger.filter,
        "owner_is_target_positive_draw_other_hand_nonempty"
    );
    assert!(trigger.optional);
    let compiled = content_compiler::compile_source_named("new_retuxi", SOURCE).unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn choosing_one_target_gains_one_hand_card_and_keeps_one_normal_draw() {
    let setup = setup(2_026_073_404);
    let owner_before = setup.state.hand_count(setup.owner);
    let target_before = setup.state.hand_count(setup.first);
    let draw = setup
        .state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let draw_before = setup.state.zones.get(draw).unwrap().cards.len();
    let gained = setup.state.hand_cards(setup.first)[0];

    let state = open_optional(&setup);
    let (state, confirmed) = resolve(state, setup.owner, vec![1], false, 2);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::TuxiGainOtherHandsReplaceDraw
    );
    let (state, targeted) = resolve(state, setup.owner, vec![setup.first.raw()], false, 3);
    assert!(matches!(targeted.result, CommandResult::Accepted { .. }));
    let (state, selected) = resolve(state, setup.owner, vec![gained.raw()], false, 4);
    assert!(matches!(selected.result, CommandResult::Accepted { .. }));

    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), owner_before + 2);
    assert_eq!(state.hand_count(setup.first), target_before - 1);
    assert!(state.hand_cards(setup.owner).contains(&gained));
    assert_eq!(state.zones.get(draw).unwrap().cards.len(), draw_before - 1);
    assert_eq!(state.turn.draw_phase_fixed_count(), Some(1));
}

#[test]
fn three_card_draw_allows_three_targets_and_reduces_the_draw_to_zero() {
    let setup = setup_with_draw_bonus(2_026_073_405);
    let owner_before = setup.state.hand_count(setup.owner);
    let first_before = setup.state.hand_count(setup.first);
    let second_before = setup.state.hand_count(setup.second);
    let third_before = setup.state.hand_count(setup.third);
    let first_card = setup.state.hand_cards(setup.first)[0];
    let second_card = setup.state.hand_cards(setup.second)[0];
    let third_card = setup.state.hand_cards(setup.third)[0];
    let draw = setup
        .state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let draw_before = setup.state.zones.get(draw).unwrap().cards.len();

    let state = open_optional(&setup);
    let (state, _) = resolve(state, setup.owner, vec![1], false, 2);
    let (state, targeted) = resolve(
        state,
        setup.owner,
        vec![setup.third.raw(), setup.first.raw(), setup.second.raw()],
        false,
        3,
    );
    assert!(matches!(targeted.result, CommandResult::Accepted { .. }));
    let (state, _) = resolve(state, setup.owner, vec![first_card.raw()], false, 4);
    let (state, _) = resolve(state, setup.owner, vec![second_card.raw()], false, 5);
    let (state, completed) = resolve(state, setup.owner, vec![third_card.raw()], false, 6);
    assert!(matches!(completed.result, CommandResult::Accepted { .. }));

    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), owner_before + 3);
    assert_eq!(state.hand_count(setup.first), first_before - 1);
    assert_eq!(state.hand_count(setup.second), second_before - 1);
    assert_eq!(state.hand_count(setup.third), third_before - 1);
    assert_eq!(state.zones.get(draw).unwrap().cards.len(), draw_before);
    assert_eq!(state.turn.draw_phase_fixed_count(), Some(0));
}
}

mod original_refresh_new_rewusheng {
//! Golden coverage for REFRESH「界武圣」的红牌转化与方片【杀】距离规则。

use engine_api::{CardDefId, CardId, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::card_distance_unlimited_for_card;
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NEW_REWUSHENG: u32 = 3_038_912_803;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/new_rewusheng.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, def: u32) -> CardId {
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
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn new_rewusheng_converts_red_personal_cards_and_only_diamond_sha_ignores_distance() {
    let compiled = content_compiler::compile_source_named("new-rewusheng-test", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 335);
    for window in ["play_phase", "response"] {
        assert!(compiled.view_as_index.iter().any(|view| {
            view.skill_def_id == NEW_REWUSHENG
                && view.as_kind == "sha"
                && view.window == window
                && view.filter == "red"
                && view.source_zone.as_deref() == Some("hand_equip_expansion")
        }));
    }

    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&compiled).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, NEW_REWUSHENG as u64, "identity-standard").unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NEW_REWUSHENG),
        }],
    )
    .unwrap();
    let diamond_sha = inject(&mut tx, owner, CardKind::Sha, Suit::Diamond, 335_001);
    let heart_sha = inject(&mut tx, owner, CardKind::Sha, Suit::Heart, 335_002);
    let diamond_trick = inject(
        &mut tx,
        owner,
        CardKind::TrickJuedou,
        Suit::Diamond,
        335_003,
    );
    assert!(card_distance_unlimited_for_card(
        tx.state(),
        owner,
        diamond_sha
    ));
    assert!(!card_distance_unlimited_for_card(
        tx.state(),
        owner,
        heart_sha
    ));
    assert!(!card_distance_unlimited_for_card(
        tx.state(),
        owner,
        diamond_trick
    ));

    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 723);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NEW_REWUSHENG));
}
}

mod original_refresh_new_reyiji {
//! Golden coverage for REFRESH 界郭嘉's rewritten 遗计.

use engine_api::{
    CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NEW_REYIJI: u32 = 3_740_567_047;
const RE_GUOJIA: u32 = 3_968_003_419;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/new_reyiji.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    other: PlayerId,
}

fn setup() -> Setup {
    let package =
        content_compiler::compile_source_named("refresh-new-reyiji-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_726_203, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let other = players[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(NEW_REYIJI),
    });
    run_effects(&mut tx, effects).unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        source,
        other,
    }
}

fn damage(state: GameState, source: PlayerId, target: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
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
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    apply_command(
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
    )
}

fn tokens(pipe: &PipelineResult) -> Vec<u32> {
    pipe.choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect()
}

#[test]
fn package_publishes_abi232_complete_re_guojia() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 251);
    assert!(package.skill_index.len() >= 76);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_GUOJIA)
        .unwrap();
    assert_eq!(character.skills, vec![6, NEW_REYIJI]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NEW_REYIJI)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_is_target_per_damage_point");
    assert!(trigger.optional);
}

#[test]
fn plans_current_hand_for_multiple_recipients_then_moves_all_at_finish() {
    let setup = setup();
    let owner_hand = setup.state.hand_cards(setup.owner);
    let original = owner_hand[0];
    let owner_before = setup.state.hand_count(setup.owner);
    let source_before = setup.state.hand_count(setup.source);
    let other_before = setup.state.hand_count(setup.other);

    let state = damage(setup.state, setup.source, setup.owner);
    let (state, confirmed) = resolve(state, setup.owner, vec![1], false, 1);
    assert_eq!(
        confirmed.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    assert!(tokens(&confirmed).contains(&original.raw()));
    assert_eq!(state.hand_count(setup.owner), owner_before + 2);

    let (state, first_cards) = resolve(state, setup.owner, vec![original.raw()], false, 2);
    assert_eq!(
        first_cards.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets
    );
    let (state, first_target) = resolve(state, setup.owner, vec![setup.source.raw()], false, 3);
    assert_eq!(
        first_target.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let owner_hand_zone = state.zones.find(ZoneKind::Hand, setup.owner).unwrap();
    assert_eq!(state.cards.get(original).unwrap().zone, owner_hand_zone);

    let second = CardId::new(tokens(&first_target)[0]);
    let (state, second_cards) = resolve(state, setup.owner, vec![second.raw()], false, 4);
    assert_eq!(
        second_cards.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets
    );
    let (state, completed) = resolve(state, setup.owner, vec![setup.other.raw()], false, 5);
    assert!(matches!(completed.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), owner_before);
    assert_eq!(state.hand_count(setup.source), source_before + 1);
    assert_eq!(state.hand_count(setup.other), other_before + 1);
}

#[test]
fn distribution_may_be_declined_after_drawing() {
    let setup = setup();
    let owner_before = setup.state.hand_count(setup.owner);
    let source_before = setup.state.hand_count(setup.source);
    let state = damage(setup.state, setup.source, setup.owner);
    let (state, confirmed) = resolve(state, setup.owner, vec![1], false, 11);
    assert_eq!(
        confirmed.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let (state, declined) = resolve(state, setup.owner, Vec::new(), true, 12);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), owner_before + 2);
    assert_eq!(state.hand_count(setup.source), source_before);
}
}

mod original_refresh_new_yajiao_old_zhaoyun {
//! Golden coverage for 新涯角 and old 赵云 rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NEW_YAJIAO: u32 = 3_751_106_523;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/new_yajiao.skill");

fn inject(
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
        suit: Suit::Diamond,
        rank: 7,
        weapon_range: kind.is_equip().then_some(2).unwrap_or(0),
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(top_kind: CardKind, seed: u64) -> (GameState, PlayerId, PlayerId, CardId, CardId) {
    let package = content_compiler::compile_source_named("new-yajiao-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 284);
    assert_eq!(package.trigger_index.len(), 2);
    assert!(package
        .trigger_index
        .iter()
        .all(|trigger| { trigger.filter == "owner_source_outside_turn_card_used_or_responded" }));
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let current = players[0];
    let owner = players[1];
    let recipient = players[2];
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NEW_YAJIAO),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = current;
    tx.state_mut().turn.phase = Phase::Play;
    let source_card = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 98_301);
    let discard_card = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Shan, 98_302);
    let revealed = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        top_kind,
        98_303,
    );
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def == SkillDefId::new(NEW_YAJIAO))
        .unwrap()
        .id;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveNewYajiao {
                player: owner,
                source_card,
                revealed: CardId::NONE,
                target: PlayerId::NONE,
                source_category: 0,
                stage: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let processing = tx
        .state()
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .unwrap();
    assert_eq!(tx.state().cards.get(revealed).unwrap().zone, processing);
    (tx.commit().state, owner, recipient, revealed, discard_card)
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
fn revealed_card_is_given_and_only_category_mismatch_forces_owner_discard() {
    let (state, owner, recipient, revealed, discard_card) = setup(CardKind::Weapon, 2_026_072_904);
    let state = choose(state, owner, recipient.raw(), 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveNewYajiao
    );
    assert_eq!(state.resolution.continuation_payload, vec![1, owner.raw()]);
    let state = choose(state, owner, discard_card.raw(), 2);
    let recipient_hand = state.zones.find(ZoneKind::Hand, recipient).unwrap();
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(revealed).unwrap().zone, recipient_hand);
    assert_eq!(state.cards.get(discard_card).unwrap().zone, discard);

    let (state, owner, recipient, revealed, discard_card) = setup(CardKind::Tao, 2_026_072_905);
    let state = choose(state, owner, recipient.raw(), 3);
    assert_eq!(state.resolution.pending_choice_id, 0);
    let recipient_hand = state.zones.find(ZoneKind::Hand, recipient).unwrap();
    let owner_hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    assert_eq!(state.cards.get(revealed).unwrap().zone, recipient_hand);
    assert_eq!(state.cards.get(discard_card).unwrap().zone, owner_hand);
}
}

mod original_refresh_olduanliang {
//! 第一百八十五批：refresh「断粮」（OL徐晃 olduanliang）零新原语重写。
//! 上游：将一张黑色非锦囊牌当做【兵粮寸断】使用；若当前回合未造成过伤害则无
//! 距离限制。实现：view_as bingliang play_phase black_basic_equip from hand_equip
//! （shenhua duanliang 同款源过滤器——非锦囊=基本+装备精确匹配）。
//! 近似（登记债）：条件距离 mod（本回合未造成伤害时）未建模。

use engine_api::PlayerId;
use engine_core::state::GameState;

const OLDUANLIANG: u32 = 3_305_084_926;
const OL_XUHUANG: u32 = 3_406_665_963;
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

#[test]
fn package_publishes_olduanliang_view_as_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 681);
    let view_as = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == OLDUANLIANG)
        .expect("olduanliang view_as contract");
    assert_eq!(view_as.as_kind, "bingliang");
    assert_eq!(view_as.window, "play_phase");
    assert_eq!(view_as.filter, "black_basic_equip");
    assert_eq!(view_as.source_zone.as_deref(), Some("hand_equip"));
    assert!(
        !package.trigger_index.iter().any(|t| t.skill_def_id == OLDUANLIANG),
        "olduanliang is a pure view_as skill"
    );
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == OL_XUHUANG)
        .expect("ol_xuhuang in roster");
    assert!(character.skills.contains(&OLDUANLIANG));
}

#[test]
fn contract_only_placeholder_compiles_without_triggers() {
    // 断粮无触发面：包内该技能不应产生 modules 条目（view_as 不进 trigger/modules）。
    let package = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 681);
    assert!(
        package
            .modules
            .iter()
            .all(|module| !module.name.contains("olduanliang")),
        "view_as skill must not emit trigger modules"
    );
    let _ = PlayerId::NONE;
    let _: GameState = GameState::empty();
}
}

mod original_refresh_oljieming {
//! Golden coverage for REFRESH「节命」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLJIEMING: u32 = 2_506_669_514;
const QUHU: u32 = 4_267_019_884;
const OL_XUNYU: u32 = 3_981_874_840;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/oljieming.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    first: PlayerId,
    second: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("oljieming-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, OLJIEMING as u64, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let source = seats[0];
    let owner = seats[1];
    let first = seats[2];
    let second = seats[3];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLJIEMING),
    });
    run_effects(&mut tx, effects).unwrap();
    {
        let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
        owner_state.hp = 8;
        owner_state.max_hp = 8;
    }
    {
        let first_state = tx.state_mut().players.get_mut(first).unwrap();
        first_state.hp = 3;
        first_state.max_hp = 3;
    }
    {
        let second_state = tx.state_mut().players.get_mut(second).unwrap();
        second_state.hp = 6;
        second_state.max_hp = 6;
    }
    Setup {
        state: tx.commit().state,
        owner,
        source,
        first,
        second,
    }
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

fn damage(state: GameState, source: PlayerId, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    tx.commit().state
}

fn kill(state: GameState, source: PlayerId, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::KillPlayer {
                player: owner,
                killer: source,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    tx.commit().state
}

fn draw(state: GameState, player: PlayerId, count: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(&mut tx, vec![EffectRequest::DrawCards { player, count }]).unwrap();
    tx.commit().state
}

#[test]
fn package_publishes_abi122_skill_and_complete_ol_xunyu() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 251);
    assert!(package.skill_index.len() >= 76);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == OLJIEMING)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert!(triggers.iter().any(|trigger| {
        trigger.event == "damage"
            && trigger.filter == "owner_is_target_per_damage_point"
            && trigger.optional
    }));
    assert!(triggers.iter().any(|trigger| {
        trigger.event == "death" && trigger.filter == "owner_is_target" && trigger.optional
    }));
    assert!(package.character_index.iter().any(|character| {
        character.id == OL_XUNYU
            && character.name_key == "ol_xunyu"
            && character.max_hp == 3
            && character.skills == [QUHU, OLJIEMING]
    }));

    let compiled = content_compiler::compile_source_named("oljieming", SOURCE).unwrap();
    assert!(compiled.modules.iter().all(|expected| {
        package
            .modules
            .iter()
            .any(|actual| actual.name == expected.name && actual.logic_hash == expected.logic_hash)
    }));
}

#[test]
fn damage_draws_by_capped_max_hp_then_target_discards_exact_excess() {
    let setup = setup();
    let state = draw(setup.state, setup.first, 2);
    let state = damage(state, setup.source, setup.owner);
    let (state, confirmed) = resolve(state, setup.owner, vec![1], 1);
    assert!(
        matches!(confirmed.result, CommandResult::Accepted { .. }),
        "{:?}",
        confirmed.result
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDrawMaxHpCapThenDiscardExcess
    );

    let (state, targeted) = resolve(state, setup.owner, vec![setup.first.raw()], 2);
    assert!(matches!(targeted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.first), 5);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::AdjustHandTo
    );
    assert_eq!(state.resolution.pending_observer, setup.first.raw());

    let discarded: Vec<_> = state
        .hand_cards(setup.first)
        .into_iter()
        .take(2)
        .map(|card| card.raw())
        .collect();
    let (state, adjusted) = resolve(state, setup.first, discarded, 3);
    assert!(matches!(adjusted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.first), 3);
    assert!(!state.has_pending_choice());
}

#[test]
fn maximum_hp_above_five_draws_only_five_without_extra_discard() {
    let setup = setup();
    let state = damage(setup.state, setup.source, setup.owner);
    let (state, confirmed) = resolve(state, setup.owner, vec![1], 4);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    let (state, targeted) = resolve(state, setup.owner, vec![setup.second.raw()], 5);
    assert!(matches!(targeted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.second), 5);
    assert!(!state.has_pending_choice());
}

#[test]
fn dead_owner_can_resolve_death_trigger_and_choose_a_living_target() {
    let setup = setup();
    let state = kill(setup.state, setup.source, setup.owner);
    assert!(!state.players.get(setup.owner).unwrap().alive);
    let (state, confirmed) = resolve(state, setup.owner, vec![1], 6);
    assert!(
        matches!(confirmed.result, CommandResult::Accepted { .. }),
        "{:?}",
        confirmed.result
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDrawMaxHpCapThenDiscardExcess
    );
    let (state, targeted) = resolve(state, setup.owner, vec![setup.first.raw()], 7);
    assert!(matches!(targeted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.first), 3);
    assert!(!state.has_pending_choice());
}

#[test]
fn zero_max_hp_target_is_rejected_at_choice_barrier() {
    let setup = setup();
    let mut state = damage(setup.state, setup.source, setup.owner);
    let (next, confirmed) = resolve(state, setup.owner, vec![1], 8);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    state = next;
    state.players.get_mut(setup.first).unwrap().max_hp = 0;
    let (_, rejected) = resolve(state, setup.owner, vec![setup.first.raw()], 9);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
}
}

mod original_refresh_ollongdan_sp_zhaoyun {
//! Golden coverage for REFRESH「龙胆」与 SP赵云的旧 JavaScript 语义重写。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit, ZoneId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::dying_save_card_for;
use engine_core::state::{Card, CardKind, GameState, Phase, Zone, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLLONGDAN: u32 = 2_506_574_121;
const CHONGZHEN: u32 = 4_144_500_448;
const SP_ZHAOYUN: u32 = 2_240_373_669;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/ollongdan.skill");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const SP: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, SkillId) {
    let package = content_compiler::compile_source_named("ollongdan-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, OLLONGDAN as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
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
            skill_def: SkillDefId::new(OLLONGDAN),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == OLLONGDAN)
        .unwrap();
    (tx.commit().state, owner, target, skill)
}

fn create_expansion(state: &mut GameState, owner: PlayerId) -> ZoneId {
    state.zones.alloc(Zone {
        id: ZoneId::NONE,
        kind: ZoneKind::Expansion,
        owner,
        cards: Vec::new(),
    })
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone: ZoneId,
    kind: CardKind,
    def: u32,
) -> CardId {
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    target: PlayerId,
    card: CardId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![card],
            },
        },
    )
}

#[test]
fn packages_publish_four_typed_rules_and_close_sp_zhaoyun() {
    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    let sp = content_schema::ContentPackage::from_json_bytes(SP).unwrap();
    assert!(refresh.manifest.rule_vm_abi_version >= 298);
    let rules: Vec<_> = refresh
        .view_as_index
        .iter()
        .filter(|rule| rule.skill_def_id == OLLONGDAN)
        .collect();
    assert_eq!(rules.len(), 4);
    assert_eq!(
        rules
            .iter()
            .map(|rule| {
                (
                    rule.as_kind.as_str(),
                    rule.window.as_str(),
                    rule.filter.as_str(),
                )
            })
            .collect::<Vec<_>>(),
        [
            ("shan", "response", "card_sha"),
            ("sha", "any", "card_shan"),
            ("jiu", "any", "card_tao"),
            ("tao", "any", "card_jiu"),
        ]
    );
    assert!(rules.iter().all(|rule| {
        rule.source_zone.as_deref() == Some("hand_expansion")
            && rule.min_cards == 1
            && rule.max_cards == 1
    }));
    assert!(sp
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "refresh"));
    assert_eq!(
        sp.character_index
            .iter()
            .find(|character| character.id == SP_ZHAOYUN)
            .unwrap()
            .skills,
        [OLLONGDAN, CHONGZHEN]
    );
    assert!(!serde_json::to_string(&(refresh, sp))
        .unwrap()
        .contains(".js"));
}

#[test]
fn hs_accepts_expansion_rejects_equipment_and_supports_both_save_conversions() {
    let (mut state, owner, target, skill) = setup();
    let equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let expansion = create_expansion(&mut state, owner);
    let equip_shan = inject(&mut state, owner, equip, CardKind::Shan, 290_001);
    let expansion_shan = inject(&mut state, owner, expansion, CardKind::Shan, 290_002);
    let unchanged = state.clone();
    let (rejected_state, rejected) = activate(state, owner, skill, target, equip_shan, 1);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(rejected_state, unchanged);

    let hp = unchanged.players.get(target).unwrap().hp;
    let (state, accepted) = activate(unchanged, owner, skill, target, expansion_shan, 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert!(state.zones.get(equip).unwrap().cards.contains(&equip_shan));

    let (mut state, owner, _, _) = setup();
    state.players.get_mut(owner).unwrap().hp = 0;
    let equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let expansion = create_expansion(&mut state, owner);
    inject(&mut state, owner, equip, CardKind::Jiu, 290_003);
    let source = inject(&mut state, owner, expansion, CardKind::Jiu, 290_004);
    assert_eq!(dying_save_card_for(&state, owner, owner), Some(source));
    state.cards.get_mut(source).unwrap().kind = CardKind::Tao;
    assert_eq!(dying_save_card_for(&state, owner, owner), Some(source));
}
}

mod original_refresh_oltishen {
//! Golden for REFRESH 张飞「替身」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const OLTISHEN: u32 = 3_850_927_603;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn isolated(hp: i32) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(OLTISHEN as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: OLTISHEN as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
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
        skill_def: SkillDefId::new(OLTISHEN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp;
    (tx, owner)
}

fn offer(tx: &mut Transaction, owner: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn resolve(
    tx: &Transaction,
    owner: PlayerId,
    accept: bool,
    key: u64,
) -> engine_core::state::GameState {
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![u32::from(accept)],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn has_oltishen(state: &engine_core::state::GameState, owner: PlayerId) -> bool {
    state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == OLTISHEN)
}

#[test]
fn committed_package_exposes_optional_limited_prepare_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLTISHEN)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional,
        ),
        ("phase_prepare", "owner_is_target_wounded", true)
    );
}

#[test]
fn accepting_recovers_to_max_draws_lost_hp_and_consumes_the_skill() {
    let (mut tx, owner) = isolated(2);
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    let hand = tx.state().hand_count(owner);
    assert_eq!(offer(&mut tx, owner), Err(EffectError::ChoiceRequested));
    let state = resolve(&tx, owner, true, 1);
    assert_eq!(state.players.get(owner).unwrap().hp, max_hp as i32);
    assert_eq!(state.hand_count(owner), hand + max_hp - 2);
    assert!(!has_oltishen(&state, owner));
}

#[test]
fn declining_preserves_state_and_healthy_owner_is_not_offered() {
    let (mut wounded, owner) = isolated(2);
    let hp = wounded.state().players.get(owner).unwrap().hp;
    let hand = wounded.state().hand_count(owner);
    assert_eq!(
        offer(&mut wounded, owner),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(&wounded, owner, false, 2);
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    assert_eq!(state.hand_count(owner), hand);
    assert!(has_oltishen(&state, owner));

    let (mut healthy, owner) = isolated(4);
    let max_hp = healthy.state().players.get(owner).unwrap().max_hp as i32;
    healthy.state_mut().players.get_mut(owner).unwrap().hp = max_hp;
    assert_eq!(offer(&mut healthy, owner), Ok(Vec::new()));
    assert!(!healthy.state().has_pending_choice());
    assert!(has_oltishen(healthy.state(), owner));
}
}

mod original_refresh_olzhiji {
//! 第一百八十四批：refresh「志继」（OL姜维 olzhiji）觉醒技零新原语重写。
//! 上游：准备阶段/结束阶段（forced juexingji），若没有手牌：减 1 点体力上限，
//! 摸两张牌或回复 2 点体力（chooseDrawRecover），获得〖观星〗。
//! 实现：双臂 phase_prepare/phase_end + owner_hand_empty → once-per-game 以
//! 自身 def 标记门控；效果顺序按批 182 惯例（选择型 effect 置尾）——change_max_hp
//! /add_skill/add_mark 先行，choose_draw_or_recover 收尾。

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLZHIJI: u32 = 4_244_253_651;
const REGUANXING: u32 = 4_220_147_649;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/olzhiji.skill");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("olzhiji-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
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
        skill_def: SkillDefId::new(OLZHIJI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn empty_hand(state: &mut GameState, player: PlayerId) {
    let mut tx = Transaction::begin(state.clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        }],
    )
    .unwrap();
    *state = tx.commit().state;
}

fn fire(state: GameState, owner: PlayerId, phase: TurnPhaseTag) -> (GameState, Result<(), EffectError>) {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let outcome = run_effects(&mut tx, effects);
    (tx.commit().state, outcome.map(|_| ()))
}

fn resolve(state: GameState, owner: PlayerId, selections: Vec<u32>, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: owner,
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

fn has_skill(state: &GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == skill_def)
}

#[test]
fn package_publishes_awakening_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 681);
    let arms: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == OLZHIJI)
        .collect();
    assert_eq!(arms.len(), 2, "prepare + end arms");
    for arm in &arms {
        assert_eq!(arm.filter, "owner_hand_empty");
        assert!(!arm.optional);
    }
    assert!(arms.iter().any(|arm| arm.event == "phase_prepare"));
    assert!(arms.iter().any(|arm| arm.event == "phase_end"));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == 3_456_200_434)
        .expect("ol_jiangwei in roster");
    assert!(character.skills.contains(&OLZHIJI));
}

#[test]
fn awakening_fires_on_empty_hand_draw_branch_and_grants_reguanxing() {
    let (mut state, owner, _other) = setup(20_260_909_03);
    empty_hand(&mut state, owner);
    let max_hp_before = state.players.get(owner).unwrap().max_hp;

    let (state, outcome) = fire(state, owner, TurnPhaseTag::Prepare);
    // 满血时恢复无意义：原语契约自动改为摸牌（不询问），效果链一次跑完。
    assert!(
        outcome.is_ok(),
        "healthy owner auto-draws, got {outcome:?}"
    );
    assert!(!state.has_pending_choice());
    // 上限-1、获得观星、自动摸 2。
    assert_eq!(state.players.get(owner).unwrap().max_hp, max_hp_before - 1);
    assert!(has_skill(&state, owner, REGUANXING));
    assert_eq!(state.hand_count(owner), 2, "auto draw branch: draw two");
}

#[test]
fn recover_branch_and_mark_gate_prevent_re_awakening() {
    let (mut state, owner, _other) = setup(20_260_909_04);
    empty_hand(&mut state, owner);
    let mut state = {
        let mut tx = Transaction::begin(state);
        tx.state_mut().players.get_mut(owner).unwrap().hp = 2; // 可观察回复分支
        tx.commit().state
    };
    let (state, outcome) = fire(state, owner, TurnPhaseTag::End);
    assert!(matches!(outcome, Err(EffectError::ChoiceRequested)));
    let max_hp_after_awaken = state.players.get(owner).unwrap().max_hp;
    let (state, _) = resolve(state, owner, vec![2], 20_260_909_14);
    assert_eq!(state.players.get(owner).unwrap().hp, 3, "recover branch");
    assert!(!state.has_pending_choice());

    // 再次空手 + 阶段触发：过滤器成立但标记门控拦截，无任何效果。
    let mut state2 = state;
    empty_hand(&mut state2, owner);
    let hp_before = state2.players.get(owner).unwrap().hp;
    let hand_before = state2.hand_count(owner);
    let (state2, outcome) = fire(state2, owner, TurnPhaseTag::Prepare);
    assert!(
        outcome.is_ok(),
        "mark gate must suppress the effect chain, got {outcome:?}"
    );
    assert!(!state2.has_pending_choice());
    assert_eq!(state2.players.get(owner).unwrap().hp, hp_before);
    assert_eq!(state2.hand_count(owner), hand_before);
    assert_eq!(
        state2.players.get(owner).unwrap().max_hp,
        max_hp_after_awaken,
        "max hp must not drop twice"
    );
}
}

mod original_refresh_qiaomeng {
//! Golden coverage for REFRESH「趫猛」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{apply_effect, bootstrap_match, move_one_card, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const QIAOMENG: u32 = 2_724_752_546;
const REQIAOMENG: u32 = 2_397_519_504;
const REYICONG: u32 = 3_426_755_583;
const RE_GONGSUNZAN: u32 = 3_363_210_554;
const GONGSUNZAN: u32 = 3_561_346_810;
const XINYICONG: u32 = 3_328_413_812;
const XIN_GONGSUNZAN: u32 = 574_905_297;
const PROTECT: u32 = 91_847;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/refresh/qiaomeng.skill"),
    include_str!("../../../content/original/ported/refresh/reqiaomeng.skill"),
    "\nskill protect(91847) locked { modifier card_discard_protected(equip) override 1; }\n"
);
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn setup(
    damage_kind: CardKind,
    damage_suit: Suit,
    equipment_kind: CardKind,
    protected: bool,
) -> (GameState, PlayerId, PlayerId, CardId, CardId) {
    let package = content_compiler::compile_source_named("qiaomeng-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_718, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(QIAOMENG),
    });
    if protected {
        effects.push(EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(PROTECT),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    let damage_card = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        damage_kind,
        damage_suit,
        92_000,
    );
    let equipment = inject(
        &mut tx,
        target,
        ZoneKind::Equip,
        equipment_kind,
        Suit::Club,
        92_001,
    );
    (tx.commit().state, owner, target, damage_card, equipment)
}

fn setup_reqiaomeng(
    damage_kind: CardKind,
    target_zone: ZoneKind,
    target_card_kind: CardKind,
) -> (GameState, PlayerId, PlayerId, CardId, CardId) {
    let package = content_compiler::compile_source_named("reqiaomeng-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_719, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REQIAOMENG),
    });
    run_effects(&mut tx, effects).unwrap();
    let damage_card = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        damage_kind,
        Suit::Heart,
        92_100,
    );
    let target_card = inject(
        &mut tx,
        target,
        target_zone,
        target_card_kind,
        Suit::Diamond,
        92_101,
    );
    (tx.commit().state, owner, target, damage_card, target_card)
}

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
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    card: CardId,
    kind: CardKind,
) -> (GameState, Result<Vec<DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            card,
            card_kind: kind.tag(),
            card_flags: 0,
        }],
    );
    (tx.commit().state, result)
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

fn open_equipment_choice(equipment_kind: CardKind) -> (GameState, PlayerId, PlayerId, CardId) {
    let (state, owner, target, sha, equipment) =
        setup(CardKind::Sha, Suit::Spade, equipment_kind, false);
    let (state, result) = damage(state, owner, target, sha, CardKind::Sha);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, accepted) = resolve(state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        accepted.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::DiscardTargetCardGainMount
    );
    assert_eq!(
        accepted
            .choice
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![equipment.raw()]
    );
    (state, owner, target, equipment)
}

#[test]
fn packages_publish_exact_skill_character_and_dependency_contract() {
    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert!(refresh.manifest.rule_vm_abi_version >= 251);
    assert!(refresh.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == QIAOMENG
            && trigger.event == "damage"
            && trigger.filter == "owner_source_black_sha_target_alive_has_discardable_equip"
            && trigger.optional
    }));
    assert!(refresh.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == REQIAOMENG
            && trigger.event == "damage"
            && trigger.filter == "owner_source_sha_target_alive_has_discardable_region_card"
            && trigger.optional
    }));
    let re_gongsunzan = refresh
        .character_index
        .iter()
        .find(|character| character.id == RE_GONGSUNZAN)
        .unwrap();
    assert_eq!(re_gongsunzan.skills, vec![REQIAOMENG, REYICONG]);
    let mobile = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    for package_id in ["refresh", "yijiang"] {
        assert!(mobile.manifest.dependencies.iter().any(|dependency| {
            dependency.namespace == "official"
                && dependency.package_id == package_id
                && dependency.version == "0.1.0"
        }));
    }
    let character = mobile
        .character_index
        .iter()
        .find(|character| character.id == XIN_GONGSUNZAN)
        .unwrap();
    assert_eq!(character.skills, vec![XINYICONG, QIAOMENG]);
}

#[test]
fn filter_requires_a_black_logical_sha_and_discardable_equipment() {
    for (kind, suit, protected) in [
        (CardKind::Sha, Suit::Heart, false),
        (CardKind::TrickHuogong, Suit::Spade, false),
        (CardKind::Sha, Suit::Club, true),
    ] {
        let (state, owner, target, card, _) = setup(kind, suit, CardKind::Weapon, protected);
        let (state, result) = damage(state, owner, target, card, kind);
        assert!(result.is_ok());
        assert!(!state.has_pending_choice());
    }
}

#[test]
fn declining_optional_trigger_leaves_the_equipment_untouched() {
    let (state, owner, target, sha, equipment) =
        setup(CardKind::Sha, Suit::Club, CardKind::Weapon, false);
    let (state, result) = damage(state, owner, target, sha, CardKind::Sha);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let original_zone = state.cards.get(equipment).unwrap().zone;
    let (state, declined) = resolve(state, owner, vec![0], false, 10);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.cards.get(equipment).unwrap().zone, original_zone);
}

#[test]
fn forced_choice_rejects_cancel_and_revalidates_the_equipment() {
    let (state, owner, target, equipment) = open_equipment_choice(CardKind::Weapon);
    let (_, cancelled) = resolve(state.clone(), owner, Vec::new(), true, 20);
    assert!(matches!(cancelled.result, CommandResult::Rejected { .. }));

    let mut reprotected = Transaction::begin(state.clone());
    apply_effect(
        &mut reprotected,
        EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(PROTECT),
        },
    )
    .unwrap();
    let (_, protected) = resolve(
        reprotected.commit().state,
        owner,
        vec![equipment.raw()],
        false,
        21,
    );
    assert!(matches!(protected.result, CommandResult::Rejected { .. }));

    let mut moved = Transaction::begin(state);
    let equip = moved.state().cards.get(equipment).unwrap().zone;
    let discard = moved
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    move_one_card(&mut moved, equipment, equip, discard, PlayerId::NONE, true).unwrap();
    let (state, forged) = resolve(
        moved.commit().state,
        owner,
        vec![equipment.raw()],
        false,
        22,
    );
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(state.cards.get(equipment).unwrap().zone, discard);
}

#[test]
fn non_mount_is_discarded_once_but_mount_is_discarded_then_gained() {
    let (state, owner, _, weapon) = open_equipment_choice(CardKind::Weapon);
    let (state, discarded) = resolve(state, owner, vec![weapon.raw()], false, 30);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(weapon).unwrap().zone, discard);
    assert_eq!(
        discarded
            .events
            .iter()
            .filter(|event| matches!(event, DomainEvent::CardsMoved { .. }))
            .count(),
        1
    );

    let (state, owner, _, mount) = open_equipment_choice(CardKind::HorseOff);
    let (state, gained) = resolve(state, owner, vec![mount.raw()], false, 31);
    assert!(matches!(gained.result, CommandResult::Accepted { .. }));
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    assert_eq!(state.cards.get(mount).unwrap().zone, hand);
    let moves: Vec<_> = gained
        .events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::CardsMoved { to, .. } => Some(*to),
            _ => None,
        })
        .collect();
    assert_eq!(moves, vec![discard, hand]);
}

#[test]
fn reqiaomeng_accepts_red_sha_and_discards_each_region_zone() {
    for (index, (zone, card_kind)) in [
        (ZoneKind::Hand, CardKind::Shan),
        (ZoneKind::Equip, CardKind::Weapon),
        (ZoneKind::Judge, CardKind::TrickLe),
    ]
    .into_iter()
    .enumerate()
    {
        let (state, owner, target, sha, target_card) =
            setup_reqiaomeng(CardKind::Sha, zone, card_kind);
        let (state, result) = damage(state, owner, target, sha, CardKind::Sha);
        assert_eq!(result, Err(EffectError::ChoiceRequested));
        let (state, accepted) = resolve(state, owner, vec![1], false, 100 + index as u64 * 2);
        assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
        assert_eq!(
            state.resolution.continuation_kind,
            ContinuationKind::DiscardTargetCardGainMount
        );
        assert!(accepted
            .choice
            .as_ref()
            .unwrap()
            .options
            .iter()
            .any(|option| option.token == target_card.raw()));
        let (state, discarded) = resolve(
            state,
            owner,
            vec![target_card.raw()],
            false,
            101 + index as u64 * 2,
        );
        assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
        let discard = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        assert_eq!(state.cards.get(target_card).unwrap().zone, discard);
    }
}

#[test]
fn reqiaomeng_rejects_non_sha_but_gains_a_discarded_mount() {
    let (state, owner, target, trick, _) =
        setup_reqiaomeng(CardKind::TrickHuogong, ZoneKind::Hand, CardKind::Shan);
    let (state, result) = damage(state, owner, target, trick, CardKind::TrickHuogong);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());

    let (state, owner, target, sha, mount) =
        setup_reqiaomeng(CardKind::Sha, ZoneKind::Equip, CardKind::HorseDef);
    let (state, result) = damage(state, owner, target, sha, CardKind::Sha);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, accepted) = resolve(state, owner, vec![1], false, 120);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    let (state, gained) = resolve(state, owner, vec![mount.raw()], false, 121);
    assert!(matches!(gained.result, CommandResult::Accepted { .. }));
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    assert_eq!(state.cards.get(mount).unwrap().zone, hand);
}
}

mod original_refresh_qinxue_re_lvmeng {
//! Golden coverage for 界吕蒙「勤学」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const RE_LVMENG: u32 = 2_276_140_348;
const KEJI: u32 = 31;
const QINXUE: u32 = 2_693_472_089;
const REBOTU: u32 = 2_231_312_450;
const GONGXIN: u32 = 3_100_453_257;
const DECADEZHENJUN: u32 = 2_634_318_954;
const YUJIN_YUJIN: u32 = 3_873_317_599;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/qinxue.skill");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const EXTRA: &[u8] = include_bytes!("../../../content/original/ported/extra/extra.npack.json");
const MODULE_CODE: [u8; 57] = [
    83, 81, 4, 16, 1, 0, 0, 0, 48, 83, 81, 5, 52, 65, 40, 0, 87, 80, 15, 83, 16, 255, 255, 255,
    255, 80, 5, 83, 16, 2, 0, 0, 0, 16, 1, 0, 0, 0, 16, 0, 0, 0, 0, 80, 39, 83, 16, 137, 41, 205,
    184, 80, 11, 64, 0, 0, 1,
];
const PREPARE_LOGIC_HASH: [u8; 32] = [
    164, 225, 152, 211, 188, 230, 193, 167, 115, 85, 35, 65, 9, 110, 106, 149, 51, 171, 123, 246,
    101, 34, 118, 94, 56, 61, 145, 191, 26, 211, 225, 31,
];
const END_LOGIC_HASH: [u8; 32] = [
    46, 91, 242, 236, 249, 215, 166, 53, 71, 1, 71, 194, 208, 230, 45, 183, 115, 182, 199, 12, 228,
    241, 136, 237, 123, 7, 85, 7, 144, 209, 148, 138,
];

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
}

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32, suit: Suit) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn has_skill(state: &GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == skill_def)
}

fn setup(seed: u64, hp: i32, max_hp: u32, hand_count: u32) -> Setup {
    let qinxue = content_compiler::compile_source_named("qinxue-test", SOURCE).unwrap();
    let extra = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&qinxue).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&extra).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
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
        skill_def: SkillDefId::new(QINXUE),
    });
    run_effects(&mut tx, effects).unwrap();
    {
        let participant = tx.state_mut().players.get_mut(owner).unwrap();
        participant.hp = hp;
        participant.max_hp = max_hp;
    }
    for index in 0..hand_count {
        inject_hand(tx.state_mut(), owner, 99_900 + index, Suit::Spade);
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target,
    }
}

fn fire_phase(state: GameState, owner: PlayerId, phase: TurnPhaseTag) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
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
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_re_lvmeng_and_keeps_refresh_on_abi_298() {
    let compiled = content_compiler::compile_source_named("qinxue", SOURCE).unwrap();
    assert_eq!(compiled.trigger_index.len(), 2);
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == QINXUE && skill.locked));
    assert!(compiled
        .trigger_index
        .iter()
        .all(|trigger| { !trigger.optional && trigger.filter == "owner_is_target" }));
    assert!(compiled
        .trigger_index
        .iter()
        .any(|trigger| trigger.event == "phase_prepare"));
    assert!(compiled
        .trigger_index
        .iter()
        .any(|trigger| trigger.event == "phase_end"));
    let prepare = compiled
        .modules
        .iter()
        .find(|module| module.name == "qinxue_phase_prepare")
        .unwrap();
    assert_eq!(prepare.code, MODULE_CODE);
    assert_eq!(prepare.logic_hash, PREPARE_LOGIC_HASH);
    let end = compiled
        .modules
        .iter()
        .find(|module| module.name == "qinxue_phase_end")
        .unwrap();
    assert_eq!(end.code, MODULE_CODE);
    assert_eq!(end.logic_hash, END_LOGIC_HASH);

    let package = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 298);
    assert!(package
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "extra"));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_LVMENG)
        .unwrap();
    assert_eq!(character.name_key, "re_lvmeng");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [KEJI, QINXUE, REBOTU]);
    assert!(compiled.modules.iter().all(|expected| {
        package
            .modules
            .iter()
            .any(|actual| actual.name == expected.name && actual.logic_hash == expected.logic_hash)
    }));

    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == DECADEZHENJUN));
    assert!(package
        .character_index
        .iter()
        .any(|character| character.id == YUJIN_YUJIN && character.skills == [DECADEZHENJUN]));
}

#[test]
fn exact_threshold_fires_at_hp_plus_two_on_both_phase_boundaries_and_only_once() {
    let mut setup = setup(20_260_730_01, 3, 4, 4);
    setup.state = fire_phase(setup.state, setup.owner, TurnPhaseTag::Prepare);
    assert!(has_skill(&setup.state, setup.owner, QINXUE));
    assert_eq!(setup.state.players.get(setup.owner).unwrap().max_hp, 4);

    inject_hand(&mut setup.state, setup.owner, 99_910, Suit::Spade);
    setup.state = fire_phase(setup.state, setup.owner, TurnPhaseTag::End);
    assert!(!has_skill(&setup.state, setup.owner, QINXUE));
    assert!(has_skill(&setup.state, setup.owner, GONGXIN));
    assert_eq!(setup.state.players.get(setup.owner).unwrap().max_hp, 3);
    assert_eq!(setup.state.hand_count(setup.owner), 7);

    let before_hp = setup.state.players.get(setup.owner).unwrap().hp;
    let before_max_hp = setup.state.players.get(setup.owner).unwrap().max_hp;
    let before_hand = setup.state.hand_count(setup.owner);
    let after = fire_phase(setup.state, setup.owner, TurnPhaseTag::Prepare);
    assert_eq!(after.players.get(setup.owner).unwrap().hp, before_hp);
    assert_eq!(
        after.players.get(setup.owner).unwrap().max_hp,
        before_max_hp
    );
    assert_eq!(after.hand_count(setup.owner), before_hand);
    assert!(!has_skill(&after, setup.owner, QINXUE));
}

#[test]
fn full_health_loses_max_hp_then_automatically_draws_two() {
    let setup = setup(20_260_730_02, 4, 4, 6);
    let state = fire_phase(setup.state, setup.owner, TurnPhaseTag::Prepare);
    let owner = state.players.get(setup.owner).unwrap();
    assert_eq!((owner.hp, owner.max_hp), (3, 3));
    assert_eq!(state.hand_count(setup.owner), 8);
    assert!(!state.has_pending_choice());
    assert!(!has_skill(&state, setup.owner, QINXUE));
    assert!(has_skill(&state, setup.owner, GONGXIN));
}

#[test]
fn wounded_owner_gets_a_forced_choice_and_gongxin_is_live_after_resolution() {
    let setup = setup(20_260_730_03, 2, 4, 4);
    let mut tx = Transaction::begin(setup.state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    let state = committed.state;
    assert!(!has_skill(&state, setup.owner, QINXUE));
    assert!(!has_skill(&state, setup.owner, GONGXIN));
    assert_eq!(state.players.get(setup.owner).unwrap().max_hp, 3);
    let choice = committed
        .output
        .choice
        .expect("draw-or-recover choice must remain pending");
    assert_eq!(choice.kind, ChoiceKind::Custom);
    assert_eq!(choice.min_count, 1);
    assert_eq!(choice.max_count, 1);
    assert!(!choice.allow_cancel);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );

    let (unchanged, cancelled) = resolve(state.clone(), setup.owner, Vec::new(), true, 1);
    assert!(matches!(cancelled.result, CommandResult::Rejected { .. }));
    assert_eq!(unchanged, state);

    let (mut state, recovered) = resolve(state, setup.owner, vec![2], false, 2);
    assert!(matches!(recovered.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 3);
    assert!(has_skill(&state, setup.owner, GONGXIN));

    let heart = inject_hand(&mut state, setup.target, 99_920, Suit::Heart);
    state.turn.current = setup.owner;
    state.turn.phase = Phase::Play;
    let gongxin = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == setup.owner && skill.def.raw() == GONGXIN)
        .unwrap()
        .id;
    let revision = state.revision;
    let (state, activated) = apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ActivateSkill {
                skill: gongxin,
                targets: vec![setup.target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let choice = activated.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [heart.raw()]
    );
    assert!(activated.events.iter().any(|event| {
        matches!(event, DomainEvent::CardsRevealed { cards, to }
            if cards == &[heart] && to == &[setup.owner])
    }));
    assert!(state.has_pending_choice());
}
}

mod original_refresh_re_caocao {
//! Golden coverage for REFRESH 界曹操's rewritten 奸雄/护驾 contract.

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NEW_REJIANXIONG: u32 = 3_449_956_505;
const REHUJIA: u32 = 3_691_667_990;
const RE_CAOCAO: u32 = 3_230_095_684;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/re_caocao.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn character(id: u32, group: &str, is_lord: bool) -> PackageCharacterEntry {
    PackageCharacterEntry {
        id,
        name_key: format!("re_caocao_test_{id}"),
        max_hp: 4,
        initial_hp: 0,
        skills: vec![],
        sex: "male".into(),
        group: group.into(),
        is_lord,
        clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: vec![],
    }
}

struct Setup {
    state: GameState,
    owner: PlayerId,
    wei: PlayerId,
    other: PlayerId,
}

fn setup() -> Setup {
    let mut package = content_compiler::compile_source_named("refresh-re-caocao-test", SOURCE)
        .expect("compile re_caocao");
    package.character_index = vec![
        character(RE_CAOCAO, "wei", true),
        character(9_199_001, "wei", false),
        character(9_199_002, "shu", false),
    ];
    package.manifest.logic_hash = package.compute_logic_hash();

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_726_199, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats
        .iter()
        .copied()
        .find(|player| tx.state().players.get(*player).unwrap().identity == Identity::Lord)
        .unwrap();
    let mut rest = seats.iter().copied().filter(|player| *player != owner);
    let wei = rest.next().unwrap();
    let other = rest.next().unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().character_def = RE_CAOCAO;
    tx.state_mut().players.get_mut(wei).unwrap().character_def = 9_199_001;
    tx.state_mut().players.get_mut(other).unwrap().character_def = 9_199_002;

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
            skill_def: SkillDefId::new(NEW_REJIANXIONG),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REHUJIA),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 20;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 20;
    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::Play;
    Setup {
        state: tx.commit().state,
        owner,
        wei,
        other,
    }
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

fn inject_processing_card(state: &mut GameState) -> CardId {
    let zone = state
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_199_100),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn package_publishes_abi232_complete_re_caocao() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 251);
    assert!(package.skill_index.len() >= 76);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_CAOCAO)
        .unwrap();
    assert!(character.is_lord);
    assert_eq!(character.skills, vec![NEW_REJIANXIONG, REHUJIA]);
    let hujia = package
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == REHUJIA)
        .unwrap();
    assert!(hujia.lord_only);
    assert!(package.view_as_index.iter().any(|view| {
        view.skill_def_id == REHUJIA
            && view.as_kind == "shan"
            && view.source_zone.as_deref() == Some("group_wei")
    }));
}

#[test]
fn new_rejianxiong_gains_processing_damage_card_then_draws() {
    let mut setup = setup();
    let card = inject_processing_card(&mut setup.state);
    let hand_before = setup.state.hand_count(setup.owner);
    let mut tx = Transaction::begin(setup.state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source: setup.other,
            target: setup.owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(tx.state().resolution.pending_observer, setup.owner.raw());
    let (state, resolved) = resolve(tx.commit().state, setup.owner, vec![1], 1);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), hand_before + 2);
    assert!(state.hand_cards(setup.owner).contains(&card));
}

#[test]
fn rehujia_draw_is_chosen_by_outside_turn_wei_player_once_per_turn() {
    let initial = setup();
    let hand_before = initial.state.hand_count(initial.owner);
    let mut tx = Transaction::begin(initial.state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::CardResponded {
            player: initial.wei,
            respond_to: initial.other,
            card: CardId::NONE,
            response_kind: CardKind::Shan.tag(),
        }],
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(tx.state().resolution.pending_observer, initial.wei.raw());
    let (state, resolved) = resolve(tx.commit().state, initial.wei, vec![1], 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(initial.owner), hand_before + 1);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::CardResponded {
            player: initial.wei,
            respond_to: initial.other,
            card: CardId::NONE,
            response_kind: CardKind::Shan.tag(),
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(initial.owner), hand_before + 1);

    let mut own_turn = setup();
    own_turn.state.turn.current = own_turn.wei;
    let mut tx = Transaction::begin(own_turn.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::CardResponded {
            player: own_turn.wei,
            respond_to: own_turn.other,
            card: CardId::NONE,
            response_kind: CardKind::Shan.tag(),
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
}
}

mod original_refresh_re_taishici_hanzhan {
//! Golden coverage for 界太史慈「酣战」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_TAISHICI: u32 = 4_231_532_233;
const TIANYI: u32 = 3_117_995_535;
const HANZHAN: u32 = 4_071_802_825;
const HANZHAN_GAIN: u32 = 3_019_387_228;
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

struct Setup {
    state: GameState,
    initiator: PlayerId,
    target: PlayerId,
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind, rank: u8) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64, hanzhan_owner_is_initiator: bool) -> Setup {
    let shenhua = content_schema::ContentPackage::from_json_bytes(SHENHUA).unwrap();
    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&shenhua).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&refresh).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (initiator, target) = (seats[0], seats[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend([
        EffectRequest::AddSkill {
            player: initiator,
            skill_def: SkillDefId::new(TIANYI),
        },
        EffectRequest::AddSkill {
            player: if hanzhan_owner_is_initiator {
                initiator
            } else {
                target
            },
            skill_def: SkillDefId::new(HANZHAN),
        },
        EffectRequest::SetTurnState {
            current: initiator,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    Setup {
        state: tx.commit().state,
        initiator,
        target,
    }
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .find_map(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
                .then_some(*skill)
        })
        .unwrap()
}

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

fn activate(state: GameState, initiator: PlayerId, target: PlayerId, card: CardId) -> GameState {
    let tianyi = skill_id(&state, initiator, TIANYI);
    let (state, result) = command(
        state,
        initiator,
        CommandKind::ActivateSkill {
            skill: tianyi,
            targets: vec![target],
            card_targets: vec![card],
        },
        1,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_complete_re_taishici_dependency_and_hooks() {
    let package = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 251);
    assert!(package.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "shenhua"
            && dependency.version == "0.1.0"
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_TAISHICI)
        .unwrap();
    assert_eq!(character.skills, [TIANYI, HANZHAN]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| { skill.skill_def_id == HANZHAN_GAIN && skill.name == "hanzhan_gain" }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == HANZHAN
            && trigger.event == "hand_compare_started"
            && trigger.filter == "owner_hand_compare_participant"
            && trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == HANZHAN
            && trigger.event == "hand_compare_finished"
            && trigger.filter == "owner_hand_compare_participant_has_sha"
            && trigger.optional
    }));
}

#[test]
fn initiator_can_randomize_target_without_opening_target_card_choice() {
    let mut setup = setup(2_026_072_201, true);
    let initiator_card = inject(
        &mut setup.state,
        setup.initiator,
        98_001,
        CardKind::Other,
        9,
    );
    let target_a = inject(&mut setup.state, setup.target, 98_002, CardKind::Other, 2);
    let target_b = inject(&mut setup.state, setup.target, 98_003, CardKind::Other, 3);
    let state = activate(setup.state, setup.initiator, setup.target, initiator_card);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, accepted) = resolve(state, setup.initiator, vec![1], 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(
        !state.has_pending_choice(),
        "unexpected pending choice: {:?} {:?}",
        state.resolution.continuation_kind,
        accepted.choice
    );
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(initiator_card).unwrap().zone, discard);
    assert_eq!(
        [target_a, target_b]
            .into_iter()
            .filter(|card| state.cards.get(*card).unwrap().zone == discard)
            .count(),
        1
    );
}

#[test]
fn target_can_randomize_initiator_then_choose_own_card() {
    let mut setup = setup(2_026_072_202, false);
    let initiator_a = inject(
        &mut setup.state,
        setup.initiator,
        98_010,
        CardKind::Other,
        4,
    );
    let initiator_b = inject(
        &mut setup.state,
        setup.initiator,
        98_011,
        CardKind::Other,
        5,
    );
    let target_card = inject(&mut setup.state, setup.target, 98_012, CardKind::Other, 6);
    let state = activate(setup.state, setup.initiator, setup.target, initiator_a);
    let (state, accepted) = resolve(state, setup.target, vec![1], 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseCompareTargetHandCard
    );
    let randomized = CardId::new(state.resolution.continuation_payload[2]);
    assert!([initiator_a, initiator_b].contains(&randomized));
    let (state, compared) = resolve(state, setup.target, vec![target_card.raw()], 3);
    assert!(
        matches!(compared.result, CommandResult::Accepted { .. }),
        "comparison failed: {:?}",
        compared.result
    );
    assert!(!state.has_pending_choice());
}

#[test]
fn declining_randomization_then_accepting_gain_collects_all_tied_highest_sha() {
    let mut setup = setup(2_026_072_203, true);
    let initiator_sha = inject(&mut setup.state, setup.initiator, 98_020, CardKind::Sha, 11);
    let target_sha = inject(&mut setup.state, setup.target, 98_021, CardKind::Sha, 11);
    let state = activate(setup.state, setup.initiator, setup.target, initiator_sha);
    let (state, declined) = resolve(state, setup.initiator, vec![0], 2);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseCompareTargetHandCard
    );
    let (state, compared) = resolve(state, setup.target, vec![target_sha.raw()], 3);
    assert!(matches!(compared.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, gained) = resolve(state, setup.initiator, vec![1], 4);
    assert!(matches!(gained.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    let hand = state.zones.find(ZoneKind::Hand, setup.initiator).unwrap();
    for card in [initiator_sha, target_sha] {
        let card = state.cards.get(card).unwrap();
        assert_eq!(card.owner, setup.initiator);
        assert_eq!(card.zone, hand);
    }
}
}

mod original_refresh_re_zhangfei_olpaoxiao {
//! Golden coverage for REFRESH「界张飞·咆哮」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLPAOXIAO: u32 = 2_829_854_661;
const OLPAOXIAO2: u32 = 3_946_229_744;
const OLTISHEN: u32 = 3_850_927_603;
const RE_ZHANGFEI: u32 = 2_213_648_390;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/refresh/olpaoxiao.skill"),
    include_str!("../../../content/original/ported/refresh/olpaoxiao2.skill"),
);

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("olpaoxiao-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_721, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let target = seats[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLPAOXIAO),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in seats {
        let state = tx.state_mut().players.get_mut(player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, target)
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
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
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn play(
    state: GameState,
    actor: PlayerId,
    card: CardId,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    )
}

fn resolve_shan(
    state: GameState,
    target: PlayerId,
    shan: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: target,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![shan.raw()],
                cancelled: false,
            },
        },
    )
}

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
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

fn mark_count(state: &GameState, player: PlayerId, mark: u32) -> i32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(id, _)| *id == mark)
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[test]
fn package_publishes_complete_re_zhangfei_closure() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 251);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_ZHANGFEI)
        .unwrap();
    assert_eq!(character.name_key, "re_zhangfei");
    assert_eq!(character.skills, [OLPAOXIAO, OLTISHEN]);
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == OLPAOXIAO2
            && modifier.target == "damage_amount"
            && modifier.selector.as_deref() == Some("sha")
            && modifier.value_source.as_deref() == Some("owner_skill_mark")
    }));
}

#[test]
fn each_full_dodge_adds_one_mark_and_next_sha_damage_consumes_all_marks() {
    let (state, owner, target) = setup();
    let mut tx = Transaction::begin(state);
    let sha1 = inject(&mut tx, owner, CardKind::Sha, 97_000);
    let sha2 = inject(&mut tx, owner, CardKind::Sha, 97_001);
    let sha3 = inject(&mut tx, owner, CardKind::Sha, 97_002);
    let sha4 = inject(&mut tx, owner, CardKind::Sha, 97_003);
    let shan1 = inject(&mut tx, target, CardKind::Shan, 97_004);
    let shan2 = inject(&mut tx, target, CardKind::Shan, 97_005);
    let state = tx.commit().state;

    assert_eq!(
        engine_core::query::sha_use_limit_for(&state, owner),
        i32::MAX as u32
    );
    let (state, result) = play(state, owner, sha1, target, 1);
    assert!(result.choice.is_some());
    let (state, result) = resolve_shan(state, target, shan1, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(mark_count(&state, owner, OLPAOXIAO2), 1);
    assert!(has_skill(&state, owner, OLPAOXIAO2));

    let (state, result) = play(state, owner, sha2, target, 3);
    assert!(result.choice.is_some());
    let (state, result) = resolve_shan(state, target, shan2, 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(mark_count(&state, owner, OLPAOXIAO2), 2);

    let hp = state.players.get(target).unwrap().hp;
    let (state, result) = play(state, owner, sha3, target, 5);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 3);
    assert_eq!(mark_count(&state, owner, OLPAOXIAO2), 0);
    assert!(!has_skill(&state, owner, OLPAOXIAO2));

    let hp = state.players.get(target).unwrap().hp;
    let (state, result) = play(state, owner, sha4, target, 6);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn unused_marks_are_cleared_when_the_owners_turn_ends() {
    let (state, owner, target) = setup();
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, owner, CardKind::Sha, 97_010);
    let shan = inject(&mut tx, target, CardKind::Shan, 97_011);
    let (state, _) = play(tx.commit().state, owner, sha, target, 10);
    let (state, _) = resolve_shan(state, target, shan, 11);
    assert_eq!(mark_count(&state, owner, OLPAOXIAO2), 1);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: target,
                phase: Phase::Prepare.to_u8(),
            },
        ],
    )
    .unwrap();
    assert_eq!(mark_count(tx.state(), owner, OLPAOXIAO2), 0);
    assert!(!has_skill(tx.state(), owner, OLPAOXIAO2));
}
}

mod original_refresh_reanguo_re_zhuzhi {
//! Golden coverage for 界朱治「安国」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_ZHUZHI: u32 = 3_807_098_010;
const REANGUO: u32 = 3_231_751_411;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/reanguo.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    other: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("reanguo-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
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
            skill_def: SkillDefId::new(REANGUO),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target: players[1],
        other: players[2],
    }
}

fn inject(
    state: &mut GameState,
    zone_kind: ZoneKind,
    owner: PlayerId,
    kind: CardKind,
    def: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == REANGUO)
        .unwrap()
        .id
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    let skill = skill(&state, owner);
    submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        key,
    )
}

fn resolve_recast(
    state: GameState,
    owner: PlayerId,
    cards: Vec<CardId>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id,
            selections: cards.into_iter().map(CardId::raw).collect(),
            cancelled: false,
        },
        key,
    )
}

fn equip_count(state: &GameState, player: PlayerId) -> usize {
    state
        .zones
        .find(ZoneKind::Equip, player)
        .and_then(|zone| state.zones.get(zone))
        .map_or(0, |zone| zone.cards.len())
}

#[test]
fn package_publishes_complete_re_zhuzhi_and_abi321_module() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 321);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_ZHUZHI)
        .unwrap();
    assert_eq!(character.name_key, "re_zhuzhi");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [REANGUO]);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == REANGUO)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    let targets = active.targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max), (1, 1));
    assert!(!targets.allow_self);
    let compiled = content_compiler::compile_source_named("reanguo", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 321);
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
fn target_completes_all_three_branches_then_owner_may_recast() {
    let mut setup = setup(32_117_001);
    let recast = inject(
        &mut setup.state,
        ZoneKind::Hand,
        setup.owner,
        CardKind::Sha,
        99_601,
    );
    setup.state.players.get_mut(setup.target).unwrap().hp = 2;
    setup.state.players.get_mut(setup.owner).unwrap().hp = 3;
    setup.state.players.get_mut(setup.other).unwrap().hp = 4;

    let (state, activated) = activate(setup.state, setup.owner, setup.target, 1);
    assert_eq!(
        activated.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveReanguo
    );
    assert_eq!(state.hand_count(setup.target), 1);
    assert_eq!(state.players.get(setup.target).unwrap().hp, 3);
    assert_eq!(equip_count(&state, setup.target), 1);
    assert_eq!(equip_count(&state, setup.owner), 0);
    assert!(activated
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .any(|option| option.token == recast.raw()));

    let (state, resolved) = resolve_recast(state, setup.owner, vec![recast], 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), 1);
    assert_ne!(state.hand_cards(setup.owner), [recast]);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(recast).unwrap().zone, discard);
}

#[test]
fn owner_executes_only_the_branches_the_target_did_not_complete() {
    let mut setup = setup(32_117_002);
    let recast = inject(
        &mut setup.state,
        ZoneKind::Hand,
        setup.owner,
        CardKind::Sha,
        99_610,
    );
    inject(
        &mut setup.state,
        ZoneKind::Equip,
        setup.target,
        CardKind::Armor,
        99_611,
    );
    inject(
        &mut setup.state,
        ZoneKind::Hand,
        setup.other,
        CardKind::Shan,
        99_612,
    );
    setup.state.players.get_mut(setup.owner).unwrap().hp = 2;
    setup.state.players.get_mut(setup.target).unwrap().hp = 4;
    setup.state.players.get_mut(setup.other).unwrap().hp = 3;

    let (state, activated) = activate(setup.state, setup.owner, setup.target, 10);
    assert_eq!(
        activated.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    assert_eq!(state.hand_count(setup.target), 1);
    assert_eq!(state.players.get(setup.target).unwrap().hp, 4);
    assert_eq!(equip_count(&state, setup.target), 1);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 3);
    assert_eq!(equip_count(&state, setup.owner), 1);
    assert!(activated
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .any(|option| option.token == recast.raw()));
}

#[test]
fn missing_draw_pile_equipment_does_not_count_as_completed_or_open_recast() {
    let mut setup = setup(32_117_003);
    let retained = inject(
        &mut setup.state,
        ZoneKind::Hand,
        setup.owner,
        CardKind::Sha,
        99_620,
    );
    setup.state.players.get_mut(setup.target).unwrap().hp = 2;
    setup.state.players.get_mut(setup.owner).unwrap().hp = 3;
    setup.state.players.get_mut(setup.other).unwrap().hp = 4;
    let draw = setup
        .state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let discard = setup
        .state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let equipment: Vec<_> = setup
        .state
        .zones
        .get(draw)
        .unwrap()
        .cards
        .iter()
        .copied()
        .filter(|card| setup.state.cards.get(*card).unwrap().kind.is_equip())
        .collect();
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: equipment,
            to: discard,
            from: Some(draw),
        }],
    )
    .unwrap();
    let state = tx.commit().state;

    let (state, activated) = activate(state, setup.owner, setup.target, 20);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert!(activated.choice.is_none());
    assert!(!state.has_pending_choice());
    assert_eq!(equip_count(&state, setup.target), 0);
    assert_eq!(equip_count(&state, setup.owner), 0);
    assert_eq!(state.cards.get(retained).unwrap().owner, setup.owner);
}
}

mod original_refresh_rebeige_mobile_re_caiwenji {
//! Golden coverage for REFRESH Rebeige and MOBILE Re Caiwenji.

use content_schema::PackageCharacterEntry;
use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const REBEIGE: u32 = 3_378_605_169;
const DUANCHANG: u32 = 2_554_836_650;
const RE_CAIWENJI: u32 = 2_656_544_451;
const TEST_REWRITE: u32 = 9_296_001;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/rebeige.skill");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");
const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

struct Setup {
    tx: Transaction,
    owner: PlayerId,
    target: PlayerId,
    source: PlayerId,
    skill: engine_api::SkillId,
}

fn character(id: u32) -> PackageCharacterEntry {
    PackageCharacterEntry {
        id,
        name_key: format!("rebeige_test_{id}"),
        max_hp: 4,
        initial_hp: 0,
        skills: vec![],
        sex: "female".into(),
        group: "qun".into(),
        is_lord: false,
        clans: vec![],
        tags: vec![],
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: vec![],
    }
}

fn setup(rewriter: bool) -> Setup {
    let source = if rewriter {
        format!(
            "{SOURCE}\nskill test_rewrite({TEST_REWRITE}) locked {{ modifier judgment_rewrite add 1; }}"
        )
    } else {
        SOURCE.to_owned()
    };
    let mut package = content_compiler::compile_source_named("rebeige-test", &source).unwrap();
    package.character_index = (0..3).map(|index| character(9_296_010 + index)).collect();
    package.manifest.logic_hash = package.compute_logic_hash();

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_730_01, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target, source) = (seats[0], seats[1], seats[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REBEIGE),
    });
    if rewriter {
        effects.push(EffectRequest::AddSkill {
            player: source,
            skill_def: SkillDefId::new(TEST_REWRITE),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == REBEIGE)
        .unwrap()
        .id;
    Setup {
        tx,
        owner,
        target,
        source,
        skill,
    }
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    suit: Suit,
    rank: u8,
) -> CardId {
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_296_100 + rank as u32),
        kind: CardKind::Other,
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

fn inject_hand(tx: &mut Transaction, player: PlayerId, suit: Suit, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    inject(tx, player, hand, suit, rank)
}

fn inject_judgment(tx: &mut Transaction, suit: Suit, rank: u8) -> CardId {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    inject(tx, PlayerId::NONE, draw, suit, rank)
}

fn begin(
    mut setup: Setup,
    suit: Suit,
) -> (GameState, CardId, Result<Vec<DomainEvent>, EffectError>) {
    let cost = inject_hand(&mut setup.tx, setup.owner, Suit::Club, 1);
    inject_judgment(&mut setup.tx, suit, 7);
    let result = run_effects(
        &mut setup.tx,
        vec![EffectRequest::ResolveBeigeConfigured {
            player: setup.owner,
            target: setup.target,
            source: setup.source,
            stage: 0,
            result_card: CardId::NONE,
            suit: Suit::None,
            rank: 0,
            rewriters: Vec::new(),
            heart_recover_amount: 2,
            diamond_draw_count: 3,
            skill: setup.skill,
        }],
    );
    (setup.tx.commit().state, cost, result)
}

fn resolve_rewrite(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
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
                cancelled,
            },
        },
    )
}

#[test]
fn compiled_contract_and_mobile_cross_package_character_are_closed() {
    let package = content_compiler::compile_source_named("rebeige-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 296);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == REBEIGE)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(
        trigger.filter,
        "owner_alive_sha_damage_source_target_alive_owner_hand_equip_nonempty"
    );
    assert!(trigger.optional);

    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    let mobile = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    let shenhua = content_schema::ContentPackage::from_json_bytes(SHENHUA).unwrap();
    assert!(refresh.manifest.rule_vm_abi_version >= 298);
    assert!(refresh
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == REBEIGE));
    let caiwenji = mobile
        .character_index
        .iter()
        .find(|character| character.id == RE_CAIWENJI)
        .unwrap();
    assert_eq!(caiwenji.name_key, "re_caiwenji");
    assert_eq!(caiwenji.sex, "female");
    assert_eq!(caiwenji.group, "qun");
    assert_eq!(caiwenji.max_hp, 3);
    assert_eq!(caiwenji.skills, [REBEIGE, DUANCHANG]);
    assert!(mobile
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "refresh"));
    assert!(mobile
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "shenhua"));
    assert!(!mobile
        .skill_index
        .iter()
        .any(|skill| matches!(skill.skill_def_id, REBEIGE | DUANCHANG)));
    assert!(caiwenji.skills.iter().all(|wanted| refresh
        .skill_index
        .iter()
        .chain(shenhua.skill_index.iter())
        .any(|skill| skill.skill_def_id == *wanted)));
}

#[test]
fn configured_heart_uses_event_amount_and_diamond_draws_three_after_judgment() {
    let mut heart = setup(false);
    heart
        .tx
        .state_mut()
        .players
        .get_mut(heart.target)
        .unwrap()
        .hp = 1;
    let target = heart.target;
    let (state, _, events) = begin(heart, Suit::Heart);
    let events = events.unwrap();
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    let judgment = events
        .iter()
        .position(|event| matches!(event, DomainEvent::JudgmentResolved { player, .. } if *player == target))
        .unwrap();
    let recovered = events
        .iter()
        .position(|event| matches!(event, DomainEvent::HpRecovered { player, amount: 2, .. } if *player == target))
        .unwrap();
    assert!(judgment < recovered);

    let diamond = setup(false);
    let target = diamond.target;
    let hand = diamond.tx.state().hand_count(target);
    let (state, _, events) = begin(diamond, Suit::Diamond);
    let events = events.unwrap();
    assert_eq!(state.hand_count(target), hand + 3);
    let judgment = events
        .iter()
        .position(|event| matches!(event, DomainEvent::JudgmentResolved { player, .. } if *player == target))
        .unwrap();
    let drawn = events
        .iter()
        .position(|event| matches!(event, DomainEvent::CardsDrawn { player, cards } if *player == target && cards.len() == 3))
        .unwrap();
    assert!(judgment < drawn);
}

#[test]
fn configured_rewrite_payload_suffix_survives_replace_and_cancel_paths() {
    let mut replaced = setup(true);
    let source = replaced.source;
    let target = replaced.target;
    replaced.tx.state_mut().players.get_mut(target).unwrap().hp = 1;
    let rewrite = inject_hand(&mut replaced.tx, source, Suit::Diamond, 9);
    let (state, _, result) = begin(replaced, Suit::Heart);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::RewriteBeigeJudgment
    );
    let payload = &state.resolution.continuation_payload;
    let rewriter_end = 8 + payload[7] as usize;
    assert_eq!(&payload[rewriter_end..], [1, 2, 3]);
    let (state, pipeline) = resolve_rewrite(state, source, vec![rewrite.raw()], false, 1);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 1);
    assert_eq!(state.hand_count(target), 3);
    let judgment = pipeline
        .events
        .iter()
        .position(|event| matches!(event, DomainEvent::JudgmentResolved { player, result_card, .. } if *player == target && *result_card == rewrite))
        .unwrap();
    let drawn = pipeline
        .events
        .iter()
        .position(|event| matches!(event, DomainEvent::CardsDrawn { player, cards } if *player == target && cards.len() == 3))
        .unwrap();
    assert!(judgment < drawn);

    let mut cancelled = setup(true);
    let source = cancelled.source;
    let target = cancelled.target;
    cancelled.tx.state_mut().players.get_mut(target).unwrap().hp = 1;
    inject_hand(&mut cancelled.tx, source, Suit::Spade, 10);
    let (state, _, result) = begin(cancelled, Suit::Heart);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let payload = &state.resolution.continuation_payload;
    let rewriter_end = 8 + payload[7] as usize;
    assert_eq!(&payload[rewriter_end..], [1, 2, 3]);
    let (state, pipeline) = resolve_rewrite(state, source, Vec::new(), true, 2);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    let judgment = pipeline
        .events
        .iter()
        .position(|event| matches!(event, DomainEvent::JudgmentResolved { player, .. } if *player == target))
        .unwrap();
    let recovered = pipeline
        .events
        .iter()
        .position(|event| matches!(event, DomainEvent::HpRecovered { player, amount: 2, .. } if *player == target))
        .unwrap();
    assert!(judgment < recovered);
}
}

mod original_refresh_refanjian {
//! Golden coverage for REFRESH「反间」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const REFANJIAN: u32 = 2_975_386_328;
const REYINGZI: u32 = 2_412_932_912;
const RE_ZHOUYU: u32 = 4_255_995_705;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/refanjian.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REFANJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == REFANJIAN)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
    }
}

fn activate(setup: Setup, card: CardId, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        setup.state.clone(),
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(setup.state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: setup.skill,
                targets: vec![setup.target],
                card_targets: vec![card],
            },
        },
    )
}

fn resolve(
    state: GameState,
    target: PlayerId,
    branch: u32,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: target,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![branch],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_publishes_abi123_active_and_complete_re_zhouyu() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 251);
    assert!(package.skill_index.len() >= 76);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == REFANJIAN)
        .expect("refanjian active entry");
    assert_eq!(active.uses_per_turn, Some(1));
    assert_eq!(
        active
            .targets
            .as_ref()
            .map(|targets| (targets.min, targets.max, targets.allow_self)),
        Some((1, 1, false))
    );
    assert_eq!(
        active.selection.as_ref().map(|selection| (
            selection.zone.as_str(),
            selection.min,
            selection.max
        )),
        Some(("hand", 1, 1))
    );
    assert!(package.character_index.iter().any(|character| {
        character.id == RE_ZHOUYU
            && character.name_key == "re_zhouyu"
            && character.max_hp == 3
            && character.skills == [REYINGZI, REFANJIAN]
    }));

    let compiled = content_compiler::compile_source_named("refanjian", SOURCE).unwrap();
    assert!(compiled.modules.iter().all(|expected| {
        package
            .modules
            .iter()
            .any(|actual| actual.name == expected.name && actual.logic_hash == expected.logic_hash)
    }));
}

#[test]
fn discard_branch_reveals_hand_and_discards_matching_hand_equip_cards() {
    let mut setup = setup(2_026_072_401);
    let gift = setup.state.hand_cards(setup.owner)[0];
    let target_cards = setup.state.hand_cards(setup.target);
    let same_hand = target_cards[0];
    let same_equip = target_cards[1];
    let different = target_cards[2];
    for card in setup.state.hand_cards(setup.target) {
        setup.state.cards.get_mut(card).unwrap().suit = Suit::Spade;
    }
    setup.state.cards.get_mut(gift).unwrap().suit = Suit::Heart;
    setup.state.cards.get_mut(same_hand).unwrap().suit = Suit::Heart;
    {
        let equipment = setup.state.cards.get_mut(same_equip).unwrap();
        equipment.suit = Suit::Heart;
        equipment.kind = CardKind::Weapon;
        equipment.weapon_range = 2;
    }
    let target_hand = setup
        .state
        .zones
        .find(ZoneKind::Hand, setup.target)
        .unwrap();
    let target_equip = setup
        .state
        .zones
        .find(ZoneKind::Equip, setup.target)
        .unwrap();
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![same_equip],
            to: target_equip,
            from: Some(target_hand),
        }],
    )
    .unwrap();
    setup.state = tx.commit().state;
    let hp = setup.state.players.get(setup.target).unwrap().hp;

    let target = setup.target;
    let (state, activated) = activate(setup, gift, 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp
    );
    assert_eq!(state.resolution.pending_observer, target.raw());
    assert_eq!(
        activated.choice.as_ref().map(|choice| {
            choice
                .options
                .iter()
                .map(|option| option.token)
                .collect::<Vec<_>>()
        }),
        Some(vec![0, 1])
    );
    assert!(activated.events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::CardsRevealed { cards, to }
            if cards == &vec![gift] && to.is_empty()
    )));
    assert!(state.hand_cards(target).contains(&gift));

    let (state, discarded) = resolve(state, target, 0, 2);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(target).unwrap().hp, hp);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for card in [gift, same_hand, same_equip] {
        assert_eq!(state.cards.get(card).unwrap().zone, discard);
    }
    assert!(state.hand_cards(target).contains(&different));
    assert!(discarded.events.iter().any(|event| matches!(
        event,
        engine_api::DomainEvent::CardsRevealed { cards, to }
            if cards.contains(&gift) && cards.contains(&same_hand) && to.is_empty()
    )));
}

#[test]
fn hp_branch_keeps_matching_cards_and_loses_one_hp() {
    let mut setup = setup(2_026_072_402);
    let gift = setup.state.hand_cards(setup.owner)[0];
    setup.state.cards.get_mut(gift).unwrap().suit = Suit::Club;
    let hp = setup.state.players.get(setup.target).unwrap().hp;
    let target = setup.target;

    let (state, activated) = activate(setup, gift, 3);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let (state, lost_hp) = resolve(state, target, 1, 4);
    assert!(matches!(lost_hp.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert!(state.hand_cards(target).contains(&gift));
    assert!(!lost_hp
        .events
        .iter()
        .any(|event| matches!(event, engine_api::DomainEvent::CardsRevealed { .. })));
}

#[test]
fn empty_hand_at_follow_up_barrier_automatically_loses_hp() {
    let setup = setup(2_026_072_403);
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: setup.target,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let hp = tx.state().players.get(setup.target).unwrap().hp;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::GiveSelectedThenTargetChooseDiscardSameSuitOrLoseHp {
                player: setup.owner,
                target: setup.target,
                card: CardId::NONE,
                suit: Suit::Diamond,
                skill: setup.skill,
                ready_to_choose: true,
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(setup.target).unwrap().hp, hp - 1);
    assert!(!tx.state().has_pending_choice());
}
}

mod original_refresh_refankui {
//! Golden coverage for 界反馈's per-damage-point card-taking contract.

use content_compiler::compile_source_named;
use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const REFANKUI: u32 = 3_719_484_801;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/refankui.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = compile_source_named("refankui-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_718, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let old_skills: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| skill.id)
        .collect();
    let mut effects: Vec<_> = old_skills
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REFANKUI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, source)
}

fn damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    (tx.commit().state, result)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections,
                cancelled: false,
            },
        },
    )
}

fn clear_personal_cards(tx: &mut Transaction, player: PlayerId) {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for kind in [ZoneKind::Hand, ZoneKind::Equip] {
        let zone = tx.state().zones.find(kind, player).unwrap();
        let cards = tx.state().zones.get(zone).unwrap().cards.clone();
        if !cards.is_empty() {
            run_effects(
                tx,
                vec![EffectRequest::MoveCards {
                    cards,
                    to: discard,
                    from: Some(zone),
                }],
            )
            .unwrap();
        }
    }
}

#[test]
fn committed_package_declares_refankui_with_required_abi() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 22);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == REFANKUI
            && trigger.event == "damage"
            && trigger.filter == "owner_is_target_source_alive_gainable_card_per_damage_point"
            && trigger.optional
    }));
}

#[test]
fn other_source_damage_offers_one_take_for_each_damage_point() {
    let (state, owner, source) = setup();
    let owner_hand = state.hand_count(owner);
    let source_hand = state.hand_count(source);
    let (state, result) = damage(state, source, owner, 2);
    assert_eq!(result, Err(EffectError::ChoiceRequested));

    let (state, accepted_first) = resolve(state, owner, vec![1], 1);
    assert!(matches!(
        accepted_first.result,
        CommandResult::Accepted { .. }
    ));
    // 第一次触发的选牌先于第二次触发的确认结算。
    assert_eq!(
        accepted_first.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let first = accepted_first.choice.as_ref().unwrap().options[0].token;
    let (state, first_taken) = resolve(state, owner, vec![first], 2);
    assert_eq!(
        first_taken.choice.as_ref().unwrap().kind,
        ChoiceKind::Confirm
    );
    let (state, accepted_second) = resolve(state, owner, vec![1], 3);
    assert_eq!(
        accepted_second.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let second = accepted_second.choice.as_ref().unwrap().options[0].token;
    let (state, taken) = resolve(state, owner, vec![second], 4);
    assert!(matches!(taken.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_hand + 2);
    assert_eq!(state.hand_count(source), source_hand - 2);
}

#[test]
fn self_damage_only_offers_equipment_and_returns_it_to_hand() {
    let (state, owner, _) = setup();
    let card = state.hand_cards(owner)[0];
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: equip,
            from: Some(hand),
        }],
    )
    .unwrap();
    let hand_before = tx.state().hand_count(owner);
    let (state, result) = damage(tx.commit().state, owner, owner, 1);
    assert_eq!(result, Err(EffectError::ChoiceRequested));

    let (state, accepted) = resolve(state, owner, vec![1], 5);
    assert_eq!(
        accepted.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let options: Vec<_> = accepted
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(options, vec![card.raw()]);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::PickTargetCard
    );

    let (state, taken) = resolve(state, owner, vec![card.raw()], 6);
    assert!(matches!(taken.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), hand_before + 1);
    assert_eq!(
        state.cards.get(card).unwrap().zone,
        state.zones.find(ZoneKind::Hand, owner).unwrap()
    );
}

#[test]
fn sourced_damage_with_no_gainable_card_opens_no_choice() {
    let (state, owner, source) = setup();
    let mut tx = Transaction::begin(state);
    clear_personal_cards(&mut tx, source);
    let (state, result) = damage(tx.commit().state, source, owner, 1);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());
}
}

mod original_refresh_regongao_rejuyi {
//! Golden coverage for REFRESH 界诸葛诞「界功獒」「界举义」.

use engine_api::{DomainEvent, PlayerId, SkillDefId, TurnPhaseTag, RULE_VM_ABI_VERSION};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const RE_ZHUGEDAN: u32 = 3_722_727_883;
const REGONGAO: u32 = 2_805_903_172;
const REJUYI: u32 = 2_754_242_355;
const BENGHUAI: u32 = 3_365_741_831;
const REWEIZHONG: u32 = 4_262_017_394;
const REGONGAO_SOURCE: &str =
    include_str!("../../../content/original/ported/refresh/regongao.skill");
const REJUYI_SOURCE: &str = include_str!("../../../content/original/ported/refresh/rejuyi.skill");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
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
                .is_some_and(|skill| skill.def.raw() == def)
        })
}

fn fire_dying(tx: &mut Transaction, player: PlayerId) {
    {
        let target = tx.state_mut().players.get_mut(player).unwrap();
        target.hp = 0;
        target.dying = true;
    }
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::EnteredDying {
            player,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(tx, effects).unwrap();
}

fn enter_prepare(tx: &mut Transaction, owner: PlayerId) {
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(tx, effects).unwrap();
}

#[test]
fn package_and_runtime_preserve_first_dying_per_target_and_exact_awakening() {
    let handle = std::thread::Builder::new()
        .stack_size(16 << 20)
        .spawn(|| {
    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    let shenhua = content_schema::ContentPackage::from_json_bytes(SHENHUA).unwrap();
    assert_eq!(refresh.manifest.rule_vm_abi_version, 723);
    assert!(refresh
        .manifest
        .dependencies
        .iter()
        .any(|dependency| { dependency.package_id == "shenhua" && dependency.version == "0.1.0" }));
    assert!(refresh.character_index.iter().any(|character| {
        character.id == RE_ZHUGEDAN
            && character.name_key == "re_zhugedan"
            && character.skills == [REGONGAO, REJUYI]
    }));
    let mut dependency_probe = GameState::empty();
    std::sync::Arc::make_mut(&mut dependency_probe.content).load_package(&shenhua).unwrap();
    std::sync::Arc::make_mut(&mut dependency_probe.content).load_package(&refresh).unwrap();

    let source = format!(
        "{REGONGAO_SOURCE}\n{REJUYI_SOURCE}\nskill benghuai({BENGHUAI}) {{}}\nskill reweizhong({REWEIZHONG}) {{}}"
    );
    let package = content_compiler::compile_source_named("regongao-rejuyi-test", &source).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 2_026_073_111, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
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
            skill_def: SkillDefId::new(REGONGAO),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REJUYI),
        },
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    {
        let owner = tx.state_mut().players.get_mut(owner).unwrap();
        owner.max_hp = 4;
        owner.hp = 3;
    }

    fire_dying(&mut tx, seats[1]);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 5);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 4);
    fire_dying(&mut tx, seats[1]);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 5);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 4);
    fire_dying(&mut tx, seats[2]);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, 6);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 5);

    enter_prepare(&mut tx, owner);
    assert!(!has_skill(tx.state(), owner, REJUYI));
    assert!(has_skill(tx.state(), owner, BENGHUAI));
    assert!(has_skill(tx.state(), owner, REWEIZHONG));
    assert_eq!(tx.state().hand_count(owner), 6);
    enter_prepare(&mut tx, owner);
    assert_eq!(tx.state().hand_count(owner), 6);
    })
    .unwrap();
    // 分离线程会吞掉断言 panic（历史 `== 356` 失真未被发现的原因），join 使失败可见。
    handle.join().expect("test thread panicked");
}
}

mod original_refresh_reguanxing {
//! 第一百八十四批：refresh「观星」（界诸葛亮 reguanxing）零新原语重写。
//! 上游：准备阶段（frequent）观看牌堆顶 X 张（存活<4 → 3，否则 5），任意重排
//! （拖拽顶/底）；取消后有结束阶段补偿臂（reguanxing_on 门控，未移植——登记债）。
//! 实现：phase_prepare + owner_is_target（forced）→ guanxing_top(self, 3|5)
//! （逐张二选顶/底近似拖拽——pxlangya 同债；循环拒绝 cancel 属原语契约）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const REGUANXING: u32 = 4_220_147_649;
const RE_ZHUGELIANG: u32 = 2_446_214_915;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/reguanxing.skill");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("reguanxing-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
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
        skill_def: SkillDefId::new(REGUANXING),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn fire_prepare(state: GameState, owner: PlayerId) -> (GameState, Result<(), EffectError>) {
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
    let outcome = run_effects(&mut tx, effects);
    (tx.commit().state, outcome.map(|_| ()))
}

fn resolve(state: GameState, owner: PlayerId, pick: u32, key: u64) -> GameState {
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
        "guanxing round rejected: {:?}",
        result.result
    );
    next
}

#[test]
fn package_publishes_reguanxing_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 681);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == REGUANXING)
        .unwrap();
    assert_eq!(trigger.event, "phase_prepare");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_ZHUGELIANG)
        .expect("re_zhugeliang in roster");
    assert!(character.skills.contains(&REGUANXING));
}

#[test]
fn prepare_phase_opens_three_card_guanxing_loop_in_small_game() {
    // 2 人局存活数 < 4 → 观星 3 张（countPlayer()<4 ? 3 : 5 的存活数口径）。
    let (state, owner, _other) = setup(20_260_909_01);
    let (state, outcome) = fire_prepare(state, owner);
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "guanxing loop must open, got {outcome:?}"
    );
    assert!(state.has_pending_choice());

    // 三轮逐张二选：前两张置底（token 2），最后一张置顶（token 1）。
    let mut next = state;
    let mut key = 20_260_909_11;
    for round in 0..3 {
        assert!(next.has_pending_choice(), "round {round} must stay open");
        next = resolve(next, owner, if round == 2 { 1 } else { 2 }, key);
        key += 1;
    }
    assert!(!next.has_pending_choice(), "loop must close after 3 rounds");
    let processing = next
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .unwrap();
    assert!(
        next.zones.get(processing).unwrap().cards.is_empty(),
        "processing must drain after guanxing settles"
    );
    let draw = next.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    assert!(
        next.zones.get(draw).unwrap().cards.len() >= 3,
        "draw pile must hold the reordered cards"
    );
}
}

mod original_refresh_reguicai {
//! Golden coverage for 界鬼才's hand/equipment rewrite and ordered rewrite windows.

use engine_api::{
    CardDefId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const REGUICAI: u32 = 2_369_807_074;
const REFANKUI: u32 = 3_719_484_801;
const RE_SIMAYI: u32 = 4_084_361_848;
const CLASSIC_GUICAI: u32 = 2_000_000_114;
const RESPONSE_LOCK: u32 = 2_000_000_115;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/reguicai.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: engine_api::ZoneId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> engine_api::CardId {
    let card = Card {
        id: engine_api::CardId::NONE,
        def: CardDefId::new(12_000 + kind.tag() as u32 * 30 + rank as u32),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: kind.is_delayed_trick(),
    };
    let card_id = tx.state_mut().cards.alloc(card);
    tx.state_mut()
        .zones
        .get_mut(zone)
        .unwrap()
        .cards
        .push(card_id);
    card_id
}

fn setup(source: &str, players: u8) -> Transaction {
    let package = content_compiler::compile_source_named("reguicai-test", source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, players, 20_260_718, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    tx.state_mut().revision = 1;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for player in tx.state().players.alive_by_seat() {
        let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
        let cards = tx.state().hand_cards(player);
        if !cards.is_empty() {
            effects.push(EffectRequest::MoveCards {
                cards,
                to: discard,
                from: Some(hand),
            });
        }
    }
    run_effects(&mut tx, effects).unwrap();
    tx
}

fn add_skill(tx: &mut Transaction, player: PlayerId, skill_def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(skill_def),
        }],
    )
    .unwrap();
}

fn prepare_le_judgment(tx: &mut Transaction, judged: PlayerId) {
    let judge = tx.state().zones.find(ZoneKind::Judge, judged).unwrap();
    inject(tx, judged, judge, CardKind::TrickLe, Suit::Club, 6);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    inject(tx, PlayerId::NONE, draw, CardKind::Other, Suit::Spade, 3);
    tx.state_mut().turn.current = judged;
    tx.state_mut().turn.phase = Phase::Judge;
}

fn end_judge(state: GameState, judged: PlayerId, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: judged,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::EndPhase,
        },
    )
}

fn choose(
    state: GameState,
    result: &PipelineResult,
    actor: PlayerId,
    card: engine_api::CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice = result.choice.as_ref().unwrap();
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![card.raw()],
                cancelled: false,
            },
        },
    )
}

fn cancel(
    state: GameState,
    result: &PipelineResult,
    actor: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice = result.choice.as_ref().unwrap();
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![],
                cancelled: true,
            },
        },
    )
}

#[test]
fn committed_package_declares_abi24_reguicai_and_re_simayi() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 24);
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == REGUICAI && modifier.target == "judgment_rewrite_hand_equip"
    }));
    assert!(package.character_index.iter().any(|character| {
        character.id == RE_SIMAYI
            && character.max_hp == 3
            && character.skills == vec![REFANKUI, REGUICAI]
    }));
}

#[test]
fn reguicai_can_rewrite_with_an_equipment_card() {
    let mut tx = setup(SOURCE, 2);
    let judged = tx.state().turn.current;
    let rewriter = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != judged)
        .unwrap();
    add_skill(&mut tx, rewriter, REGUICAI);
    let equip = tx.state().zones.find(ZoneKind::Equip, rewriter).unwrap();
    let heart = inject(&mut tx, rewriter, equip, CardKind::Armor, Suit::Heart, 5);
    prepare_le_judgment(&mut tx, judged);

    let (state, first) = end_judge(tx.state().clone(), judged, 1);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    let choice = first.choice.as_ref().unwrap();
    assert_eq!(choice.observer, rewriter);
    assert_eq!(choice.options.len(), 1);
    assert_eq!(choice.options[0].token, heart.raw());

    let (state, resolved) = choose(state, &first, rewriter, heart, 2);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.flag(turn_flag::SKIP_PLAY), 0);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(heart).unwrap().zone, discard);
}

#[test]
fn classic_hand_only_rewrite_does_not_offer_equipment() {
    let source = format!(
        "{SOURCE}\nskill classic_guicai({CLASSIC_GUICAI}) {{ modifier judgment_rewrite add 1; }}"
    );
    let mut tx = setup(&source, 2);
    let judged = tx.state().turn.current;
    let rewriter = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != judged)
        .unwrap();
    add_skill(&mut tx, rewriter, CLASSIC_GUICAI);
    let equip = tx.state().zones.find(ZoneKind::Equip, rewriter).unwrap();
    inject(&mut tx, rewriter, equip, CardKind::Armor, Suit::Heart, 5);
    prepare_le_judgment(&mut tx, judged);

    let (state, result) = end_judge(tx.state().clone(), judged, 10);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.flag(turn_flag::SKIP_PLAY), 1);
}

#[test]
fn response_lock_removes_all_reguicai_options() {
    let source = format!(
        "{SOURCE}\nskill response_lock({RESPONSE_LOCK}) locked {{ modifier card_response_cancel override 1; }}"
    );
    let mut tx = setup(&source, 2);
    let judged = tx.state().turn.current;
    let rewriter = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != judged)
        .unwrap();
    add_skill(&mut tx, rewriter, REGUICAI);
    add_skill(&mut tx, rewriter, RESPONSE_LOCK);
    let equip = tx.state().zones.find(ZoneKind::Equip, rewriter).unwrap();
    inject(&mut tx, rewriter, equip, CardKind::Armor, Suit::Heart, 5);
    prepare_le_judgment(&mut tx, judged);

    let (state, result) = end_judge(tx.state().clone(), judged, 20);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.flag(turn_flag::SKIP_PLAY), 1);
}

#[test]
fn multiple_rewriters_are_offered_in_seat_order_and_the_last_result_wins() {
    let mut tx = setup(SOURCE, 3);
    let judged = tx.state().turn.current;
    let order = tx.state().players.alive_by_seat();
    let start = order.iter().position(|player| *player == judged).unwrap();
    let second = order[(start + 1) % order.len()];
    add_skill(&mut tx, judged, REGUICAI);
    add_skill(&mut tx, second, REGUICAI);
    let judged_hand = tx.state().zones.find(ZoneKind::Hand, judged).unwrap();
    let second_hand = tx.state().zones.find(ZoneKind::Hand, second).unwrap();
    let first_card = inject(
        &mut tx,
        judged,
        judged_hand,
        CardKind::Other,
        Suit::Diamond,
        7,
    );
    let second_card = inject(
        &mut tx,
        second,
        second_hand,
        CardKind::Other,
        Suit::Heart,
        8,
    );
    prepare_le_judgment(&mut tx, judged);

    let (state, first) = end_judge(tx.state().clone(), judged, 30);
    assert_eq!(first.choice.as_ref().unwrap().observer, judged);
    let (state, second_choice) = choose(state, &first, judged, first_card, 31);
    assert!(matches!(
        second_choice.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(second_choice.choice.as_ref().unwrap().observer, second);
    let (state, resolved) = choose(state, &second_choice, second, second_card, 32);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.flag(turn_flag::SKIP_PLAY), 0);
}

#[test]
fn declining_one_rewrite_still_offers_the_next_player() {
    let mut tx = setup(SOURCE, 3);
    let judged = tx.state().turn.current;
    let order = tx.state().players.alive_by_seat();
    let start = order.iter().position(|player| *player == judged).unwrap();
    let second = order[(start + 1) % order.len()];
    add_skill(&mut tx, judged, REGUICAI);
    add_skill(&mut tx, second, REGUICAI);
    let judged_hand = tx.state().zones.find(ZoneKind::Hand, judged).unwrap();
    let second_hand = tx.state().zones.find(ZoneKind::Hand, second).unwrap();
    inject(
        &mut tx,
        judged,
        judged_hand,
        CardKind::Other,
        Suit::Diamond,
        7,
    );
    let heart = inject(
        &mut tx,
        second,
        second_hand,
        CardKind::Other,
        Suit::Heart,
        8,
    );
    prepare_le_judgment(&mut tx, judged);

    let (state, first) = end_judge(tx.state().clone(), judged, 40);
    assert_eq!(first.choice.as_ref().unwrap().observer, judged);
    let (state, second_choice) = cancel(state, &first, judged, 41);
    assert_eq!(second_choice.choice.as_ref().unwrap().observer, second);
    let (state, resolved) = choose(state, &second_choice, second, heart, 42);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.flag(turn_flag::SKIP_PLAY), 0);
}
}

mod original_refresh_rehuaiyi_re_gongsunyuan {
//! Golden coverage for 界公孙渊「怀异」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_GONGSUNYUAN: u32 = 3_935_634_515;
const REHUAIYI: u32 = 4_054_411_167;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/rehuaiyi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    first: PlayerId,
    second: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("rehuaiyi-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, u64::from(REHUAIYI), "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
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
            skill_def: SkillDefId::new(REHUAIYI),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        first: players[1],
        second: players[2],
    }
}

fn inject(
    state: &mut GameState,
    zone_kind: ZoneKind,
    owner: PlayerId,
    kind: CardKind,
    suit: Suit,
    def: u32,
) -> CardId {
    let zone_owner = if matches!(
        zone_kind,
        ZoneKind::DrawPile | ZoneKind::DiscardPile | ZoneKind::Processing
    ) {
        PlayerId::NONE
    } else {
        owner
    };
    let zone = state.zones.find(zone_kind, zone_owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner: zone_owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == REHUAIYI)
        .unwrap()
        .id
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn activate(state: GameState, owner: PlayerId, key: u64) -> (GameState, PipelineResult) {
    let skill = skill(&state, owner);
    submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        key,
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

#[test]
fn package_publishes_complete_re_gongsunyuan_and_abi320_module() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 320);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_GONGSUNYUAN)
        .unwrap();
    assert_eq!(character.name_key, "re_gongsunyuan");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [REHUAIYI]);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == REHUAIYI)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_play_phase, Some(2));
    assert!(active.hand_nonempty);
    let compiled = content_compiler::compile_source_named("rehuaiyi", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 320);
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
fn uniform_hand_draws_and_permits_exactly_one_more_use_this_play_phase() {
    let mut setup = setup();
    inject(
        &mut setup.state,
        ZoneKind::Hand,
        setup.owner,
        CardKind::Sha,
        Suit::Heart,
        99_501,
    );
    inject(
        &mut setup.state,
        ZoneKind::DrawPile,
        PlayerId::NONE,
        CardKind::Shan,
        Suit::Diamond,
        99_502,
    );
    let (mut state, first) = activate(setup.state, setup.owner, 1);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), 2);
    assert_eq!(
        state
            .skills
            .get(skill(&state, setup.owner))
            .unwrap()
            .play_phase_use_count,
        1
    );

    inject(
        &mut state,
        ZoneKind::DrawPile,
        PlayerId::NONE,
        CardKind::Tao,
        Suit::Heart,
        99_503,
    );
    let (state, second) = activate(state, setup.owner, 2);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), 3);
    assert_eq!(
        state
            .skills
            .get(skill(&state, setup.owner))
            .unwrap()
            .play_phase_use_count,
        2
    );

    let before = canonical_state_hash(&state);
    let (state, third) = activate(state, setup.owner, 3);
    assert!(matches!(third.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
}

#[test]
fn mixed_hand_discards_one_color_gains_by_seat_and_loses_hp_after_two_cards() {
    let mut setup = setup();
    inject(
        &mut setup.state,
        ZoneKind::Hand,
        setup.owner,
        CardKind::Sha,
        Suit::Heart,
        99_510,
    );
    inject(
        &mut setup.state,
        ZoneKind::Hand,
        setup.owner,
        CardKind::Tao,
        Suit::Diamond,
        99_514,
    );
    inject(
        &mut setup.state,
        ZoneKind::Hand,
        setup.owner,
        CardKind::Shan,
        Suit::Spade,
        99_511,
    );
    let first_card = inject(
        &mut setup.state,
        ZoneKind::Hand,
        setup.first,
        CardKind::Tao,
        Suit::Club,
        99_512,
    );
    let second_card = inject(
        &mut setup.state,
        ZoneKind::Equip,
        setup.second,
        CardKind::Weapon,
        Suit::Diamond,
        99_513,
    );
    let initial_hp = setup.state.players.get(setup.owner).unwrap().hp;

    let (state, color_choice) = activate(setup.state, setup.owner, 10);
    assert_eq!(
        color_choice.choice.as_ref().unwrap().kind,
        ChoiceKind::Custom
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveRehuaiyi
    );
    let (state, target_choice) = resolve(state, setup.owner, vec![0b01], false, 11);
    assert_eq!(
        target_choice.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets
    );
    assert_eq!(state.hand_count(setup.owner), 1);

    let (state, first_gain) = resolve(
        state,
        setup.owner,
        vec![setup.second.raw(), setup.first.raw()],
        false,
        12,
    );
    assert_eq!(
        first_gain.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    assert_eq!(
        first_gain.choice.as_ref().unwrap().options[0].token,
        first_card.raw()
    );

    let before_forgery = canonical_state_hash(&state);
    let (state, forged) = resolve(state, setup.owner, vec![second_card.raw()], false, 13);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before_forgery);

    let (state, second_gain) = resolve(state, setup.owner, vec![first_card.raw()], false, 14);
    assert_eq!(
        second_gain.choice.as_ref().unwrap().options[0].token,
        second_card.raw()
    );
    let (state, completed) = resolve(state, setup.owner, vec![second_card.raw()], false, 15);
    assert!(matches!(completed.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), 3);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, initial_hp - 1);
    assert_eq!(state.cards.get(first_card).unwrap().owner, setup.owner);
    assert_eq!(state.cards.get(second_card).unwrap().owner, setup.owner);

    let before = canonical_state_hash(&state);
    let (state, second_use) = activate(state, setup.owner, 16);
    assert!(matches!(second_use.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
}

#[test]
fn cancelling_target_choice_gains_nothing_and_does_not_lose_hp() {
    let mut setup = setup();
    inject(
        &mut setup.state,
        ZoneKind::Hand,
        setup.owner,
        CardKind::Sha,
        Suit::Heart,
        99_520,
    );
    inject(
        &mut setup.state,
        ZoneKind::Hand,
        setup.owner,
        CardKind::Shan,
        Suit::Spade,
        99_521,
    );
    let target_card = inject(
        &mut setup.state,
        ZoneKind::Hand,
        setup.first,
        CardKind::Tao,
        Suit::Club,
        99_522,
    );
    let initial_hp = setup.state.players.get(setup.owner).unwrap().hp;

    let (state, _) = activate(setup.state, setup.owner, 20);
    let (state, _) = resolve(state, setup.owner, vec![0b01], false, 21);
    let (state, cancelled) = resolve(state, setup.owner, Vec::new(), true, 22);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, initial_hp);
    assert_eq!(state.cards.get(target_card).unwrap().owner, setup.first);
}
}

mod original_refresh_rejianchu {
//! Golden coverage for REFRESH 界庞德「鞬出」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::sha_use_limit_for;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::TriggerFireRecord;
use rule_api::EffectRequest;

const OL_PANGDE: u32 = 3_770_183_463;
const MASHU: u32 = 2;
const REJIANCHU: u32 = 3_270_398_373;
const REJIANCHU2: u32 = 3_848_225_310;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/refresh/rejianchu.skill"),
    include_str!("../../../content/original/ported/refresh/rejianchu2.skill"),
);
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("rejianchu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_730, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let target = seats[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REJIANCHU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, target)
}

fn inject(
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
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn play_sha(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    sha: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    command(
        state,
        owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![target],
            card_targets: vec![],
        },
        key,
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> Option<engine_api::SkillId> {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .find_map(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
                .then_some(*skill)
        })
}

fn mark_count(state: &GameState, owner: PlayerId) -> i32 {
    state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find_map(|(mark, count)| (*mark == REJIANCHU2).then_some(*count))
        .unwrap_or(0)
}

#[test]
fn package_publishes_complete_ol_pangde_with_abi313_support() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 313);
    assert!(package.character_index.iter().any(|character| {
        character.id == OL_PANGDE
            && character.name_key == "ol_pangde"
            && character.max_hp == 4
            && character.skills == [MASHU, REJIANCHU]
    }));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == REJIANCHU)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional
        ),
        (
            "card_targeted",
            "owner_source_sha_target_alive_has_discardable_hand_equip",
            true
        )
    );
}

#[test]
fn basic_card_gives_the_target_the_physical_sha_and_keeps_normal_response() {
    let (state, owner, target) = setup();
    let mut tx = Transaction::begin(state);
    let sha = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 312_001);
    let shan = inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan, 312_002);
    let tao = inject(&mut tx, target, ZoneKind::Hand, CardKind::Tao, 312_003);
    let hp = tx.state().players.get(target).unwrap().hp;

    let (offered, result) = play_sha(tx.commit().state, owner, target, sha, 1);
    assert_eq!(result.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    let (selecting, result) = resolve(offered, owner, vec![1], 2);
    assert_eq!(
        result.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    let (responding, result) = resolve(selecting, owner, vec![tao.raw()], 3);
    assert_eq!(result.choice.as_ref().unwrap().observer, target);
    assert!(responding.hand_cards(target).contains(&sha));
    assert_eq!(mark_count(&responding, owner), 0);
    assert!(skill_id(&responding, owner, REJIANCHU2).is_none());
    let (resolved, result) = resolve(responding, target, vec![shan.raw()], 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(resolved.players.get(target).unwrap().hp, hp);
}

#[test]
fn nonbasic_cards_stack_sha_uses_then_the_turn_end_clears_the_bonus() {
    let (state, owner, target) = setup();
    let mut tx = Transaction::begin(state);
    let first_sha = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 312_011);
    let second_sha = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 312_012);
    let first_trick = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        312_013,
    );
    let second_trick = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::TrickGuohe,
        312_014,
    );
    let hp = tx.state().players.get(target).unwrap().hp;

    let (offered, _) = play_sha(tx.commit().state, owner, target, first_sha, 11);
    let (selecting, _) = resolve(offered, owner, vec![1], 12);
    let (resolved, result) = resolve(selecting, owner, vec![first_trick.raw()], 13);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(resolved.players.get(target).unwrap().hp, hp - 1);
    assert_eq!(mark_count(&resolved, owner), 1);
    assert_eq!(sha_use_limit_for(&resolved, owner), 2);

    let (offered, _) = play_sha(resolved, owner, target, second_sha, 14);
    let (selecting, _) = resolve(offered, owner, vec![1], 15);
    let (resolved, result) = resolve(selecting, owner, vec![second_trick.raw()], 16);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(resolved.players.get(target).unwrap().hp, hp - 2);
    assert_eq!(mark_count(&resolved, owner), 2);
    assert_eq!(sha_use_limit_for(&resolved, owner), 3);

    let mut tx = Transaction::begin(resolved);
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: owner,
            phase: TurnPhaseTag::End,
            damage_sources: 1,
            damaged_players: 1,
            card_targeted_other: true,
            current_dealt_damage_other: true,
            phase_targeted_players: 1,
            phase_hp_changed_players: 1,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert!(skill_id(tx.state(), owner, REJIANCHU2).is_none());
    assert_eq!(mark_count(tx.state(), owner), 0);
    assert_eq!(sha_use_limit_for(tx.state(), owner), 1);
}
}

mod original_refresh_relongyin {
//! Golden coverage for 界关平「界龙吟」and compatibility with the original 龙吟 ABI.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RELONGYIN: u32 = 2_474_771_514;
const JIEZHONG: u32 = 2_811_249_808;
const RE_GUANPING: u32 = 2_498_702_026;
const LONGYIN: u32 = 3_578_226_944;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/relongyin.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const OLD_SOURCE: &str = include_str!("../../../content/original/ported/yijiang/longyin.skill");
const OLD_PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    owner: PlayerId,
    skill: SkillId,
    sha: CardId,
    cost: CardId,
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    def: u32,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
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
    package_bytes: &[u8],
    skill_def: u32,
    seed: u64,
    sha_suit: Suit,
    sha_rank: u8,
    cost_rank: u8,
) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(package_bytes).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let source = tx.state().turn.current;
    let target = players
        .iter()
        .copied()
        .find(|player| *player != source)
        .unwrap();
    let owner = players
        .iter()
        .copied()
        .find(|player| *player != source && *player != target)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in players {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        },
        EffectRequest::SetTurnState {
            current: source,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == skill_def)
        .unwrap();
    let sha = inject(
        tx.state_mut(),
        source,
        295_001,
        CardKind::Sha,
        sha_suit,
        sha_rank,
    );
    let cost = inject(
        tx.state_mut(),
        owner,
        295_002,
        CardKind::Jiu,
        Suit::Club,
        cost_rank,
    );
    Setup {
        state: tx.commit().state,
        source,
        target,
        owner,
        skill,
        sha,
        cost,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    apply_command(
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
    )
}

fn offer_and_accept(setup: &Setup, key: u64) -> GameState {
    let revision = setup.state.revision;
    let (state, offered) = apply_command(
        setup.state.clone(),
        Command {
            actor: setup.source,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card: setup.sha,
                targets: vec![setup.target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);
    assert_eq!(state.resolution.pending_observer, setup.owner.raw());
    let (state, accepted) = resolve(state, setup.owner, vec![1], key + 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardOneForShaRefundAndDrawIfRed
    );
    state
}

fn pay(mut state: GameState, setup: &Setup, already_refunded: bool, key: u64) -> GameState {
    if already_refunded {
        state.turn.set_flag(turn_flag::SHA_PLAYED, 0);
        let flag_index = match state.resolution.resume_kind {
            ContinuationKind::ResumePlayedSha => 4,
            ContinuationKind::ResumePlayedShaQueue => 3,
            other => panic!("unexpected saved Sha continuation: {other:?}"),
        };
        state.resolution.resume_payload[flag_index] |= 4;
        state
            .resolution
            .sha_resolution_records
            .iter_mut()
            .filter(|record| record.source == setup.source && record.card == setup.sha)
            .for_each(|record| record.use_refunded = true);
    }
    let (state, paid) = resolve(state, setup.owner, vec![setup.cost.raw()], key);
    assert!(matches!(paid.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    state
}

fn has_skill(state: &GameState, player: PlayerId, skill_def: u32) -> bool {
    state
        .players
        .get(player)
        .into_iter()
        .flat_map(|player| player.skills.iter())
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == skill_def)
        })
}

#[test]
fn package_publishes_current_abi_and_complete_re_guanping() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 298);
    assert_eq!(package.manifest.logic_hash, package.compute_logic_hash());
    assert!(package.skill_index.len() >= 85);
    assert!(package.character_index.len() >= 26);
    assert!(package.modules.len() >= 63);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_GUANPING)
        .unwrap();
    assert_eq!(character.name_key, "re_guanping");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [RELONGYIN, JIEZHONG]);
    let compiled = content_compiler::compile_source_named("relongyin", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 295);
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.code, compiled.modules[0].code);
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn effective_rank_and_refund_branches_match_legacy_relongyin() {
    struct Case {
        suit: Suit,
        sha_rank: u8,
        cost_rank: u8,
        already_refunded: bool,
        expected_hand: u32,
        expected_restore: bool,
    }
    let cases = [
        Case {
            suit: Suit::Heart,
            sha_rank: 7,
            cost_rank: 7,
            already_refunded: false,
            expected_hand: 1,
            expected_restore: true,
        },
        Case {
            suit: Suit::Spade,
            sha_rank: 8,
            cost_rank: 8,
            already_refunded: false,
            expected_hand: 0,
            expected_restore: true,
        },
        Case {
            suit: Suit::Diamond,
            sha_rank: 9,
            cost_rank: 10,
            already_refunded: false,
            expected_hand: 1,
            expected_restore: false,
        },
        Case {
            suit: Suit::Heart,
            sha_rank: 11,
            cost_rank: 11,
            already_refunded: true,
            expected_hand: 1,
            expected_restore: true,
        },
    ];
    for (index, case) in cases.into_iter().enumerate() {
        let setup = setup(
            PACKAGE,
            RELONGYIN,
            2_026_073_100 + index as u64,
            case.suit,
            case.sha_rank,
            case.cost_rank,
        );
        let state = offer_and_accept(&setup, 100 + index as u64 * 10);
        assert_eq!(
            state.resolution.continuation_payload,
            [
                setup.owner.raw(),
                setup.source.raw(),
                setup.sha.raw(),
                setup.skill.raw(),
                JIEZHONG,
            ],
            "case {index}"
        );
        let state = pay(
            state,
            &setup,
            case.already_refunded,
            102 + index as u64 * 10,
        );
        assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 0, "case {index}");
        assert_eq!(
            state.hand_count(setup.owner),
            case.expected_hand,
            "case {index}"
        );
        assert_eq!(
            has_skill(&state, setup.owner, JIEZHONG),
            case.expected_restore,
            "case {index}"
        );
    }
}

#[test]
fn original_longyin_module_and_four_word_continuation_remain_stable() {
    let compiled = content_compiler::compile_source_named("longyin", OLD_SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 43);
    assert_eq!(compiled.modules[0].code, [83, 85, 86, 80, 61, 1]);
    assert_eq!(
        compiled.modules[0].logic_hash,
        [
            23, 183, 110, 115, 100, 48, 169, 6, 243, 112, 120, 102, 154, 3, 105, 63, 222, 225, 72,
            185, 146, 62, 190, 239, 208, 78, 135, 229, 116, 236, 163, 70,
        ]
    );
    let package = content_schema::ContentPackage::from_json_bytes(OLD_PACKAGE).unwrap();
    let committed = package
        .modules
        .iter()
        .find(|module| module.name == "longyin_card_used")
        .unwrap();
    assert_eq!(committed.code, compiled.modules[0].code);
    assert_eq!(committed.logic_hash, compiled.modules[0].logic_hash);

    let setup = setup(OLD_PACKAGE, LONGYIN, 2_026_073_200, Suit::Heart, 7, 7);
    let state = offer_and_accept(&setup, 200);
    assert_eq!(
        state.resolution.continuation_payload,
        [
            setup.owner.raw(),
            setup.source.raw(),
            setup.sha.raw(),
            setup.skill.raw(),
        ]
    );
    assert_eq!(
        canonical_state_hash(&state),
        [
            93, 189, 86, 88, 231, 121, 116, 47, 231, 144, 116, 198, 180, 63, 94, 81, 158, 142, 151, 39, 225, 117, 114, 113, 228, 120, 149, 124, 26, 228, 190, 42,
        ]
    );
}
}

mod original_refresh_reluanwu_reweimu {
//! Golden for REFRESH 界贾诩「界乱武」「界帷幕」及 OFFLINE 复用闭包。

use engine_api::{DomainEvent, SkillDefId, RULE_VM_ABI_VERSION};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_JIAXU: u32 = 3_147_472_843;
const SCL_JIAXU: u32 = 4_227_787_579;
const REWANSHA: u32 = 3_382_466_194;
const SCLWANSHA: u32 = 3_972_375_066;
const RELUANWU: u32 = 2_626_196_598;
const REWEIMU: u32 = 2_589_015_751;
const RELUANWU_SOURCE: &str =
    include_str!("../../../content/original/ported/refresh/reluanwu.skill");
const REWEIMU_SOURCE: &str = include_str!("../../../content/original/ported/refresh/reweimu.skill");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

#[test]
fn packages_close_both_jiaxu_and_reweimu_only_prevents_damage_in_owners_turn() {
    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    let offline = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    assert_eq!(refresh.manifest.rule_vm_abi_version, 723);
    assert!(refresh.character_index.iter().any(|character| {
        character.id == RE_JIAXU && character.skills == [REWANSHA, RELUANWU, REWEIMU]
    }));
    assert!(offline.character_index.iter().any(|character| {
        character.id == SCL_JIAXU && character.skills == [SCLWANSHA, RELUANWU, REWEIMU]
    }));
    assert!(offline
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "refresh"));

    let package = content_compiler::compile_source_named(
        "reluanwu-reweimu-test",
        &format!("{RELUANWU_SOURCE}\n{REWEIMU_SOURCE}"),
    )
    .unwrap();
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == REWEIMU
            && modifier.target == "card_target_cancel"
            && modifier.selector.as_deref() == Some("black_trick")
    }));

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 2_026_080_101, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let source = players[0];
    let owner = players[1];
    let old_skills = tx
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
            skill_def: SkillDefId::new(REWEIMU),
        }],
    )
    .unwrap();

    let hp = tx.state().players.get(owner).unwrap().hp;
    let hand = tx.state().hand_count(owner);
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
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
    assert_eq!(tx.state().hand_count(owner), hand);

    tx.state_mut().turn.current = owner;
    let hand = tx.state().hand_count(owner);
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
    assert_eq!(tx.state().hand_count(owner), hand + 4);
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::DamagePrevented {
            source: event_source,
            target,
            current,
        } if *event_source == source && *target == owner && *current == owner
    )));
}
}

mod original_refresh_reluoshen {
//! Golden coverage for refresh 界洛神 rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::query::discard_to_limit_tokens;
use engine_core::state::{GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const RE_ZHENJI: u32 = 3_716_126_919;
const RELUOSHEN: u32 = 3_265_545_896;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/reluoshen.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    black: engine_api::CardId,
}

fn setup() -> Setup {
    let package = compile_source_named("refresh-reluoshen-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_730_309, "identity-refresh").unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(RELUOSHEN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let black = *tx.state().zones.get(draw).unwrap().cards.last().unwrap();
    tx.state_mut().cards.get_mut(black).unwrap().suit = Suit::Spade;
    Setup {
        state: tx.commit().state,
        owner,
        black,
    }
}

fn offer(setup: Setup) -> (GameState, PlayerId, engine_api::CardId) {
    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: setup.owner,
                phase: TurnPhaseTag::Prepare,
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let committed = tx.commit();
    assert_eq!(
        committed.output.choice.as_ref().unwrap().kind,
        ChoiceKind::Confirm
    );
    (committed.state, setup.owner, setup.black)
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

#[test]
fn package_closes_re_zhenji_without_legacy_javascript() {
    let compiled = compile_source_named("refresh-reluoshen", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 309);
    assert_eq!(
        rule_vm::bytecode::effect_tag::JUDGE_BLACK_GAIN_REPEAT_PROTECT_HAND_LIMIT,
        293
    );
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_ZHENJI)
        .unwrap();
    assert_eq!(character.skills, [RELUOSHEN, 24]);
    assert!(!PACKAGE.windows(3).any(|bytes| bytes == b".js"));
}

#[test]
fn gained_black_card_is_protected_only_for_the_current_turn() {
    let (state, owner, black) = offer(setup());
    let ordinary_hand_card = state.hand_cards(owner)[0];
    let (state, repeated) = resolve(state, owner, vec![1], false, 1);
    assert_eq!(repeated.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    assert!(state.hand_cards(owner).contains(&black));
    assert!(state.turn.hand_limit_exempt_cards.contains(&(owner, black)));
    let discardable = discard_to_limit_tokens(&state, owner);
    assert!(!discardable.contains(&black));
    assert!(discardable.contains(&ordinary_hand_card));

    let (mut state, stopped) = resolve(state, owner, Vec::new(), true, 2);
    assert!(matches!(stopped.result, CommandResult::Accepted { .. }));
    state.turn.phase = Phase::End;
    let (state, advanced) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(advanced.result, CommandResult::Accepted { .. }));
    assert!(state.turn.hand_limit_exempt_cards.is_empty());
    assert!(discard_to_limit_tokens(&state, owner).contains(&black));
}
}

mod original_refresh_remoshi_ol_huangzhong {
//! Golden coverage for OL界黄忠「没矢」rewritten from legacy JavaScript semantics.

use engine_api::{CardDefId, CardId, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::{damage_card_flag, EffectRequest};

const OL_HUANGZHONG: u32 = 3_398_640_656;
const XINLIEGONG: u32 = 3_272_111_610;
const REMOSHI: u32 = 3_096_399_491;
const REMOSHI_STUCK: u32 = 4_090_725_192;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/remoshi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    sha: CardId,
    equipment: CardId,
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    def: CardDefId,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def,
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(equipment_kind: CardKind, physical_sha: bool) -> Setup {
    let package = content_compiler::compile_source_named("remoshi-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_730, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REMOSHI),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    let sha = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::Processing,
        CardKind::Sha,
        if physical_sha {
            CardDefId::new(99_318)
        } else {
            CardDefId::NONE
        },
    );
    let equipment = inject(
        &mut tx,
        target,
        ZoneKind::Equip,
        equipment_kind,
        CardDefId::new(99_319),
    );
    Setup {
        state: tx.commit().state,
        owner,
        target,
        sha,
        equipment,
    }
}

fn damage(mut setup: Setup, card_flags: u8) -> Setup {
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source: setup.owner,
            target: setup.target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            card: setup.sha,
            card_kind: CardKind::Sha.tag(),
            card_flags,
        }],
    )
    .unwrap();
    setup.state = tx.commit().state;
    setup
}

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
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

#[test]
fn package_publishes_complete_character_and_abi318_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 318);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == OL_HUANGZHONG)
            .unwrap()
            .skills,
        [XINLIEGONG, REMOSHI]
    );
    assert!(package
        .skill_index
        .iter()
        .any(|skill| { skill.skill_def_id == REMOSHI_STUCK && skill.name == "remoshi_stuck" }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == REMOSHI
            && trigger.event == "damage"
            && trigger.filter
                == "owner_source_direct_sha_damage_target_armor_or_mount_card_gainable"
            && !trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == REMOSHI
            && trigger.event == "cards_moved"
            && trigger.filter
                == "owner_alive_event_player_lost_armor_or_mount_has_originating_skill_pile"
            && !trigger.optional
    }));
}

#[test]
fn physical_sha_is_stored_then_returned_when_target_loses_armor() {
    let setup = damage(setup(CardKind::Armor, true), 0);
    assert_eq!(
        setup.state.skill_pile_cards(setup.target, REMOSHI),
        [setup.sha]
    );
    assert!(has_skill(&setup.state, setup.target, REMOSHI_STUCK));

    let equip = setup
        .state
        .zones
        .find(ZoneKind::Equip, setup.target)
        .unwrap();
    let discard = setup
        .state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![setup.equipment],
            to: discard,
            from: Some(equip),
        }],
    )
    .unwrap();
    assert!(tx
        .state()
        .skill_pile_cards(setup.target, REMOSHI)
        .is_empty());
    assert!(tx.state().hand_cards(setup.owner).contains(&setup.sha));
}

#[test]
fn multi_card_view_as_sha_stores_every_physical_source() {
    let mut setup = setup(CardKind::Armor, true);
    let mut tx = Transaction::begin(setup.state);
    let extra = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        CardKind::TrickWuzhong,
        CardDefId::new(99_321),
    );
    tx.state_mut().resolution.register_sha_resolution(
        setup.owner,
        setup.sha,
        &[setup.target],
        false,
    );
    assert!(tx.state_mut().resolution.register_sha_source_cards(
        setup.owner,
        setup.sha,
        &[setup.sha, extra]
    ));
    setup.state = tx.commit().state;

    let setup = damage(setup, 0);
    assert_eq!(
        setup.state.skill_pile_cards(setup.target, REMOSHI),
        [setup.sha, extra]
    );
}

#[test]
fn losing_a_weapon_does_not_return_the_stuck_sha() {
    let mut setup = damage(setup(CardKind::Armor, true), 0);
    let mut tx = Transaction::begin(setup.state);
    let weapon = inject(
        &mut tx,
        setup.target,
        ZoneKind::Equip,
        CardKind::Weapon,
        CardDefId::new(99_320),
    );
    setup.state = tx.commit().state;
    let equip = setup
        .state
        .zones
        .find(ZoneKind::Equip, setup.target)
        .unwrap();
    let discard = setup
        .state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![weapon],
            to: discard,
            from: Some(equip),
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().skill_pile_cards(setup.target, REMOSHI),
        [setup.sha]
    );
}

#[test]
fn chain_damage_virtual_sha_and_weapon_only_target_do_not_place_cards() {
    for setup in [
        damage(
            setup(CardKind::Armor, true),
            damage_card_flag::CHAIN_PROPAGATED,
        ),
        damage(setup(CardKind::Armor, false), 0),
        damage(setup(CardKind::Weapon, true), 0),
    ] {
        assert!(setup
            .state
            .skill_pile_cards(setup.target, REMOSHI)
            .is_empty());
        assert_eq!(
            setup.state.cards.get(setup.sha).unwrap().zone,
            setup
                .state
                .zones
                .find(ZoneKind::Processing, PlayerId::NONE)
                .unwrap()
        );
    }
}
}

mod original_refresh_repindi_dc_chenqun {
//! Golden coverage for 界陈群「界品第」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DC_CHENQUN: u32 = 2_755_954_297;
const REPINDI: u32 = 3_416_188_911;
const DCFAEN: u32 = 2_290_375_740;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/repindi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    first: PlayerId,
}

fn setup() -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, u64::from(REPINDI), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
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
            skill_def: SkillDefId::new(REPINDI),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        first: players[1],
    }
}

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn repindi_skill(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == REPINDI)
        .unwrap()
        .id
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    card: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    let skill = repindi_skill(&state, owner);
    submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![card],
        },
        key,
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

fn enter_new_play_phase(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Play.to_u8(),
            },
        ],
    )
    .unwrap();
    tx.commit().state
}

fn is_linked(state: &GameState, player: PlayerId) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0)
}

#[test]
fn package_publishes_complete_dc_chenqun_and_exact_abi314_module() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 314);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == DC_CHENQUN)
        .unwrap();
    assert_eq!(character.skills, [REPINDI, DCFAEN]);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == REPINDI)
        .unwrap();
    let targets = active.targets.as_ref().unwrap();
    assert!(targets.allow_self);
    assert!(targets.unused_by_skill_this_play_phase);
    assert!(!targets.unused_by_skill_this_turn);
    assert!(
        active
            .selection
            .as_ref()
            .unwrap()
            .unused_category_by_skill_this_play_phase
    );
    let compiled = content_compiler::compile_source_named("repindi", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 314);
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
fn phase_uniqueness_turn_count_and_link_toggle_match_legacy_js() {
    let mut setup = setup();
    {
        let owner = setup.state.players.get_mut(setup.owner).unwrap();
        owner.max_hp = 4;
        owner.hp = 3;
    }

    let first_cost = inject(&mut setup.state, setup.owner, CardKind::Sha, 99_301);
    inject(&mut setup.state, setup.owner, CardKind::Tao, 99_300);
    let (state, opened) = activate(setup.state, setup.owner, setup.owner, first_cost, 1);
    assert!(matches!(opened.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDrawOrDiscardBySkillUseCountLinkIfWounded
    );
    let (mut state, drawn) = resolve(state, setup.owner, vec![1], 2);
    assert!(matches!(drawn.result, CommandResult::Accepted { .. }));
    assert!(is_linked(&state, setup.owner));

    let repeated_target_cost = inject(&mut state, setup.owner, CardKind::TrickWuzhong, 99_302);
    let before = canonical_state_hash(&state);
    let (state, repeated_target) =
        activate(state, setup.owner, setup.owner, repeated_target_cost, 3);
    assert!(matches!(
        repeated_target.result,
        CommandResult::Rejected { .. }
    ));
    assert_eq!(canonical_state_hash(&state), before);

    let mut state = state;
    let repeated_category_cost = inject(&mut state, setup.owner, CardKind::Shan, 99_303);
    let before = canonical_state_hash(&state);
    let (state, repeated_category) =
        activate(state, setup.owner, setup.first, repeated_category_cost, 4);
    assert!(matches!(
        repeated_category.result,
        CommandResult::Rejected { .. }
    ));
    assert_eq!(canonical_state_hash(&state), before);

    let mut state = state;
    let second_cost = inject(&mut state, setup.owner, CardKind::TrickWuzhong, 99_304);
    let only_target_card = inject(&mut state, setup.first, CardKind::Tao, 99_305);
    let (state, second) = activate(state, setup.owner, setup.first, second_cost, 5);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    let (state, discard_branch) = resolve(state, setup.owner, vec![2], 6);
    let choice = discard_branch.choice.as_ref().unwrap();
    assert_eq!(choice.observer, setup.first);
    assert_eq!((choice.min_count, choice.max_count), (1, 1));
    let (state, discarded) = resolve(state, setup.first, vec![only_target_card.raw()], 7);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert!(is_linked(&state, setup.owner));
    assert_eq!(
        state
            .skills
            .get(repindi_skill(&state, setup.owner))
            .unwrap()
            .use_count,
        2
    );

    let mut state = enter_new_play_phase(state, setup.owner);
    let third_cost = inject(&mut state, setup.owner, CardKind::Sha, 99_306);
    let (state, third) = activate(state, setup.owner, setup.owner, third_cost, 8);
    assert!(matches!(third.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state
            .skills
            .get(repindi_skill(&state, setup.owner))
            .unwrap()
            .use_count,
        3
    );
    let (state, resolved) = resolve(state, setup.owner, vec![1], 9);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(!is_linked(&state, setup.owner));
}
}

mod original_refresh_repojun3 {
//! Golden for 界限突破「破军」伤害子技能的逐目标手牌、装备比较语义。

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::intent::{resolve_damage_intent_with_kind, DamageIntent};
use engine_core::state::{CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const REPOJUN3: u32 = 2_423_810_510;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn isolated() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == REPOJUN3 && skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(REPOJUN3 as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: REPOJUN3 as u64,
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
        skill_def: SkillDefId::new(REPOJUN3),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, players[0], players[1])
}

fn damage_amount(tx: &Transaction, source: PlayerId, target: PlayerId, logical_sha: bool) -> u32 {
    resolve_damage_intent_with_kind(
        tx.state(),
        DamageIntent {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            cancelled: false,
        },
        logical_sha,
    )
    .amount
}

fn equip_one_hand_card(tx: &mut Transaction, player: PlayerId) {
    let card = tx.state().hand_cards(player)[0];
    tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Armor;
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let equip = tx.state().zones.find(ZoneKind::Equip, player).unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: equip,
            from: Some(hand),
        }],
    )
    .unwrap();
}

#[test]
fn bonus_requires_both_hand_and_equipment_counts_to_be_no_smaller() {
    let (mut tx, source, target) = isolated();

    assert_eq!(damage_amount(&tx, source, target, true), 2);

    equip_one_hand_card(&mut tx, target);
    assert_eq!(damage_amount(&tx, source, target, true), 1);

    equip_one_hand_card(&mut tx, source);
    assert_eq!(damage_amount(&tx, source, target, true), 2);

    let source_hand = tx.state().hand_count(source);
    let target_hand = tx.state().hand_count(target);
    let discard_count = source_hand.saturating_sub(target_hand).saturating_add(1) as usize;
    let cards = tx.state().hand_cards(source)[..discard_count].to_vec();
    let hand = tx.state().zones.find(ZoneKind::Hand, source).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards,
            to: discard,
            from: Some(hand),
        }],
    )
    .unwrap();
    assert_eq!(damage_amount(&tx, source, target, true), 1);
}

#[test]
fn bonus_is_limited_to_logical_sha_damage() {
    let (tx, source, target) = isolated();
    assert_eq!(damage_amount(&tx, source, target, true), 2);
    assert_eq!(damage_amount(&tx, source, target, false), 1);
}
}

mod original_refresh_repolu_mobile_re_sunjian {
//! Golden coverage for REFRESH「破虏」and MOBILE 界孙坚 rewritten from legacy JavaScript.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const GZYINGHUN: u32 = 2_790_748_306;
const REPOLU: u32 = 2_820_205_777;
const RE_SUNJIAN: u32 = 976_445_267;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/repolu.skill");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    others: Vec<PlayerId>,
}

fn setup(player_count: u8, owner_index: usize) -> Setup {
    let package = content_compiler::compile_source_named("repolu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 291);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, player_count, REPOLU as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[owner_index];
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
        skill_def: SkillDefId::new(REPOLU),
    });
    run_effects(&mut tx, effects).unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        others: players
            .into_iter()
            .filter(|player| *player != owner)
            .collect(),
    }
}

fn kill(state: GameState, player: PlayerId, killer: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(&mut tx, vec![EffectRequest::KillPlayer { player, killer }],),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseRepoluTargetsDraw
    );
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
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
                cancelled,
            },
        },
    )
}

fn mark_count(state: &GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == REPOLU)
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[test]
fn packages_publish_forced_death_branches_and_close_re_sunjian() {
    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    let mobile = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    assert!(refresh.manifest.rule_vm_abi_version >= 298);
    let triggers: Vec<_> = refresh
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == REPOLU)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert!(triggers.iter().all(|trigger| {
        trigger.event == "death"
            && !trigger.optional
            && matches!(
                trigger.filter.as_str(),
                "owner_alive_is_source" | "owner_is_target"
            )
    }));
    assert!(mobile
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "refresh"));
    assert!(mobile
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "shenhua"));
    assert_eq!(
        mobile
            .character_index
            .iter()
            .find(|character| character.id == RE_SUNJIAN)
            .unwrap()
            .skills,
        [GZYINGHUN, REPOLU]
    );
    assert!(!serde_json::to_string(&(refresh, mobile))
        .unwrap()
        .contains(".js"));
}

#[test]
fn cancellation_does_not_count_and_successive_kills_draw_one_then_two() {
    let setup = setup(5, 0);
    let [first, second, third, recipient] = setup.others.as_slice() else {
        panic!("expected four other players");
    };
    let state = kill(setup.state, *first, setup.owner);
    let (state, cancelled) = resolve(state, setup.owner, Vec::new(), true, 1);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert_eq!(mark_count(&state, setup.owner), 0);

    let state = kill(state, *second, setup.owner);
    let (state, first_use) = resolve(
        state,
        setup.owner,
        vec![recipient.raw(), setup.owner.raw()],
        false,
        2,
    );
    assert!(matches!(first_use.result, CommandResult::Accepted { .. }));
    assert_eq!(mark_count(&state, setup.owner), 1);
    assert_eq!(state.hand_count(setup.owner), 1);
    assert_eq!(state.hand_count(*recipient), 1);

    let state = kill(state, *third, setup.owner);
    let (state, second_use) = resolve(state, setup.owner, vec![recipient.raw()], false, 3);
    assert!(matches!(second_use.result, CommandResult::Accepted { .. }));
    assert_eq!(mark_count(&state, setup.owner), 2);
    assert_eq!(state.hand_count(*recipient), 3);
    assert!(!state.has_pending_choice());
}

#[test]
fn self_death_opens_exactly_one_choice_for_the_dead_owner() {
    let setup = setup(3, 1);
    let recipient = setup.others[1];
    let state = kill(setup.state, setup.owner, setup.owner);
    assert!(!state.players.get(setup.owner).unwrap().alive);
    assert!(!state.outcome.is_finished());
    let (state, resolved) = resolve(state, setup.owner, vec![recipient.raw()], false, 4);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(mark_count(&state, setup.owner), 1);
    assert_eq!(state.hand_count(recipient), 1);
    assert!(!state.has_pending_choice());
}
}

mod original_refresh_reqiangxi_mobile_re_dianwei {
//! Golden coverage for REFRESH「强袭」and MOBILE 界典韦 rewritten from legacy JavaScript.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const REQIANGXI: u32 = 2_185_312_161;
const QIANGXIX: u32 = 3_151_452_629;
const RE_DIANWEI: u32 = 598_273_341;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/reqiangxi.skill");
const QIANGXIX_SOURCE: &str =
    include_str!("../../../content/original/ported/shenhua/qiangxix.skill");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");
const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    reqiangxi: SkillId,
    qiangxix: SkillId,
}

fn setup() -> Setup {
    let source = format!("{SOURCE}\n{QIANGXIX_SOURCE}");
    let package = content_compiler::compile_source_named("reqiangxi-test", &source).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 292);
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, u64::from(REQIANGXI), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
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
            skill_def: SkillDefId::new(REQIANGXI),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(QIANGXIX),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.max_hp = 12;
        participant.hp = 12;
    }
    let skill = |definition| {
        tx.state()
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .copied()
            .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == definition)
            .unwrap()
    };
    let reqiangxi = skill(REQIANGXI);
    let qiangxix = skill(QIANGXIX);
    Setup {
        state: tx.commit().state,
        owner,
        target: players[1],
        reqiangxi,
        qiangxix,
    }
}

fn activate(
    state: GameState,
    owner: PlayerId,
    skill: SkillId,
    target: PlayerId,
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
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    )
}

fn enter_new_play_phase(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Play.to_u8(),
            },
        ],
    )
    .unwrap();
    tx.commit().state
}

#[test]
fn packages_publish_play_phase_target_scope_and_close_re_dianwei() {
    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    let mobile = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    assert!(refresh.manifest.rule_vm_abi_version >= 298);
    let targets = refresh
        .active_index
        .iter()
        .find(|active| active.skill_def_id == REQIANGXI)
        .and_then(|active| active.targets.as_ref())
        .unwrap();
    assert!(!targets.allow_self);
    assert!(targets.in_attack_range);
    assert!(targets.unused_by_skill_this_play_phase);
    assert!(!targets.unused_by_skill_this_turn);
    assert!(mobile
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "refresh"));
    assert_eq!(
        mobile
            .character_index
            .iter()
            .find(|character| character.id == RE_DIANWEI)
            .unwrap()
            .skills,
        [REQIANGXI]
    );
    assert!(!serde_json::to_string(&(refresh, mobile))
        .unwrap()
        .contains(".js"));
}

#[test]
fn phase_scope_resets_without_clearing_existing_turn_scope() {
    let setup = setup();
    let (state, first) = activate(setup.state, setup.owner, setup.reqiangxi, setup.target, 1);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    let (same, repeated) = activate(state.clone(), setup.owner, setup.reqiangxi, setup.target, 2);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
    assert_eq!(same, state);

    let state = enter_new_play_phase(state, setup.owner);
    let (state, next_phase) = activate(state, setup.owner, setup.reqiangxi, setup.target, 3);
    assert!(matches!(next_phase.result, CommandResult::Accepted { .. }));

    let (state, turn_scoped) = activate(state, setup.owner, setup.qiangxix, setup.target, 4);
    assert!(matches!(turn_scoped.result, CommandResult::Accepted { .. }));
    let state = enter_new_play_phase(state, setup.owner);
    let (_, still_used) = activate(state, setup.owner, setup.qiangxix, setup.target, 5);
    assert!(matches!(still_used.result, CommandResult::Rejected { .. }));
}
}

mod original_refresh_reqimou {
//! Golden coverage for REFRESH 界魏延「奇谋」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, RejectReason, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{query, sha_use_limit_for, QueryContext};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::TriggerFireRecord;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const OL_WEIYAN: u32 = 2_765_423_130;
const XINKUANGGU: u32 = 4_089_972_415;
const REQIMOU: u32 = 3_862_622_181;
const REQIMOU2: u32 = 2_749_452_350;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/refresh/reqimou.skill"),
    include_str!("../../../content/original/ported/refresh/reqimou2.skill"),
);
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> Option<engine_api::SkillId> {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .find_map(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
                .then_some(*skill)
        })
}

fn mark_count(state: &GameState, owner: PlayerId) -> i32 {
    state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find_map(|(mark, count)| (*mark == REQIMOU2).then_some(*count))
        .unwrap_or(0)
}

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

#[test]
fn package_publishes_complete_ol_weiyan_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 251);
    assert!(package.skill_index.len() >= 76);
    assert!(package.character_index.iter().any(|character| {
        character.id == OL_WEIYAN
            && character.name_key == "ol_weiyan"
            && character.max_hp == 4
            && character.skills == [XINKUANGGU, REQIMOU]
    }));
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == REQIMOU)
        .unwrap();
    assert_eq!(active.window, "play_phase");
    assert!(active.once_per_game);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == REQIMOU2
            && trigger.event == "turn_ended"
            && trigger.filter == "owner_is_target"
    }));
}

#[test]
fn loses_and_draws_chosen_amount_grants_bonuses_once_then_expires() {
    let package = content_compiler::compile_source_named("reqimou-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, 20_260_724, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let far = players[2];
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
            skill_def: SkillDefId::new(REQIMOU),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let player = tx.state_mut().players.get_mut(owner).unwrap();
    player.hp = 4;
    player.max_hp = 4;
    let hand = tx.state().hand_count(owner);
    let active = skill_id(tx.state(), owner, REQIMOU).unwrap();

    let (state, opened) = command(
        tx.commit().state,
        owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        1,
    );
    let choice = opened.choice.unwrap();
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2, 3, 4]
    );
    let (state, resolved) = command(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(choice.choice_id.raw()),
            selections: vec![3],
            cancelled: false,
        },
        2,
    );
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 1);
    assert_eq!(state.hand_count(owner), hand + 3);
    assert_eq!(mark_count(&state, owner), 3);
    assert_eq!(sha_use_limit_for(&state, owner), 4);
    assert!(matches!(
        query(
            &state,
            &QueryRequest::Distance {
                from: owner,
                to: far,
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(1)
    ));
    let (_, repeated) = command(
        state.clone(),
        owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        3,
    );
    assert!(matches!(
        repeated.result,
        CommandResult::Rejected {
            reason: RejectReason::IllegalAction { .. }
        }
    ));

    let mut tx = Transaction::begin(state);
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: owner,
            phase: TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert!(skill_id(tx.state(), owner, REQIMOU2).is_none());
    assert_eq!(mark_count(tx.state(), owner), 0);
    assert_eq!(sha_use_limit_for(tx.state(), owner), 1);
}
}

mod original_refresh_retieji_re_machao {
//! Golden coverage for 界马超「铁骑」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_MACHAO: u32 = 4_268_725_118;
const RETIEJI: u32 = 3_731_623_167;
const TARGET_OPEN: u32 = 90_100;
const TARGET_LOCKED: u32 = 90_101;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/retieji.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    sha: CardId,
    shan: CardId,
    matching: Option<CardId>,
}

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
        def: CardDefId::new(98_000 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64, with_matching_card: bool) -> Setup {
    let source = format!(
        "{SOURCE}\n\
         skill target_open({TARGET_OPEN}) {{ modifier hand_limit add 1; }}\n\
         skill target_locked({TARGET_LOCKED}) locked {{ modifier hand_limit add 1; }}"
    );
    let package = content_compiler::compile_source_named("retieji-test", &source).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
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
            skill_def: SkillDefId::new(RETIEJI),
        },
        EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(TARGET_OPEN),
        },
        EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(TARGET_LOCKED),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let shan = inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Club,
        2,
    );
    let matching = with_matching_card.then(|| {
        inject(
            &mut tx,
            target,
            ZoneKind::Equip,
            CardKind::Weapon,
            Suit::Heart,
            6,
        )
    });
    inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Other,
        Suit::Heart,
        7,
    );
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        8,
    );
    Setup {
        state: tx.commit().state,
        owner,
        target,
        sha,
        shan,
        matching,
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

fn play_and_accept(setup: Setup) -> (GameState, PipelineResult, Setup) {
    let (state, offered) = submit(
        setup.state.clone(),
        setup.owner,
        CommandKind::PlayCard {
            card: setup.sha,
            targets: vec![setup.target],
            card_targets: Vec::new(),
        },
        1,
    );
    let choice = offered.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::Confirm);
    let (state, result) = submit(
        state,
        setup.owner,
        CommandKind::ResolveChoice {
            choice_id: choice.choice_id,
            selections: vec![1],
            cancelled: false,
        },
        2,
    );
    (state, result, setup)
}

fn skill_enabled(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state.skills.get(*skill).is_some_and(|instance| {
                instance.def.raw() == def && state.skill_effectively_enabled(instance)
            })
        })
}

#[test]
fn package_publishes_complete_re_machao_and_abi316_module() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 316);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_MACHAO)
        .unwrap();
    assert_eq!(character.skills, [2, RETIEJI]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == RETIEJI)
        .unwrap();
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(trigger.filter, "owner_source_sha");
    assert!(trigger.optional);

    let compiled = content_compiler::compile_source_named("retieji", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 316);
    let expected = compiled.modules.first().unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn matching_suit_card_is_given_and_avoids_the_sha_penalty() {
    let setup = setup(2_026_073_201, true);
    let owner = setup.owner;
    let target = setup.target;
    let matching = setup.matching.unwrap();
    let (state, result, setup) = play_and_accept(setup);
    let choice = result.choice.unwrap();
    assert_eq!(choice.observer, target);
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![matching.raw()]
    );
    assert!(!skill_enabled(&state, target, TARGET_OPEN));
    assert!(skill_enabled(&state, target, TARGET_LOCKED));

    let (state, result) = submit(
        state,
        target,
        CommandKind::ResolveChoice {
            choice_id: choice.choice_id,
            selections: vec![matching.raw()],
            cancelled: false,
        },
        3,
    );
    assert!(state.hand_cards(owner).contains(&matching));
    let shan_choice = result.choice.unwrap();
    assert!(shan_choice
        .options
        .iter()
        .any(|option| option.token == setup.shan.raw()));
    let hp = state.players.get(target).unwrap().hp;
    let (state, result) = submit(
        state,
        target,
        CommandKind::ResolveChoice {
            choice_id: shan_choice.choice_id,
            selections: vec![setup.shan.raw()],
            cancelled: false,
        },
        4,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp);

    let mut state = state;
    for key in 5..8 {
        (state, _) = submit(state, owner, CommandKind::EndPhase, key);
    }
    assert!(skill_enabled(&state, target, TARGET_OPEN));
}

#[test]
fn declining_the_gift_makes_only_this_sha_unrespondable_and_adds_one_damage() {
    let setup = setup(2_026_073_202, true);
    let target = setup.target;
    let hp = setup.state.players.get(target).unwrap().hp;
    let (state, result, _) = play_and_accept(setup);
    let choice = result.choice.unwrap();
    let (state, result) = submit(
        state,
        target,
        CommandKind::ResolveChoice {
            choice_id: choice.choice_id,
            selections: Vec::new(),
            cancelled: true,
        },
        3,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(state.players.get(target).unwrap().hp, hp - 2);
}

#[test]
fn missing_same_suit_card_applies_the_penalty_without_opening_a_gift_choice() {
    let setup = setup(2_026_073_203, false);
    let target = setup.target;
    let hp = setup.state.players.get(target).unwrap().hp;
    let (state, result, setup) = play_and_accept(setup);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(state.players.get(target).unwrap().hp, hp - 2);
    assert!(state.hand_cards(target).contains(&setup.shan));
}
}

mod original_refresh_retongxin {
//! Golden for REFRESH「同心」的攻击范围修正。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const RETONGXIN: u32 = 2_855_175_601;
const REWEIZHONG: u32 = 4_262_017_394;
const REBIYUE: u32 = 4_285_757_447;
const REZONGSHI: u32 = 3_490_528_973;
const REZONGSHI_PAOXIAO: u32 = 2_831_909_821;
const NEW_REPAOXIAO2: u32 = 2_901_743_430;
const NEW_REPAOXIAO: u32 = 2_733_157_850;
const REFANGQUAN2: u32 = 2_792_995_788;
const REYINGZI: u32 = 2_412_932_912;
const REDANXIN: u32 = 2_528_063_581;
const REDANXIN_MARK: u32 = 380_579_933;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn has_skill(tx: &Transaction, player: PlayerId, skill_def: u32) -> bool {
    tx.state()
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == skill_def)
}

#[test]
fn attack_range_adds_two() {
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
    let mut tx = Transaction::begin(engine.state().clone());
    let base = match query(
        tx.state(),
        &QueryRequest::AttackRange { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    };
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(RETONGXIN),
        }],
    )
    .unwrap();
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::AttackRange { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(value) if value == base + 2
    ));
}

#[test]
fn reweizhong_draws_two_after_max_hp_changes() {
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
            skill_def: SkillDefId::new(REWEIZHONG),
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
    assert_eq!(tx.state().hand_count(owner), hand + 2);
}

#[test]
fn rebiyue_draws_two_when_empty_and_one_otherwise() {
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

    for empty in [false, true] {
        let mut tx = Transaction::begin(engine.state().clone());
        let mut setup = vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REBIYUE),
        }];
        if empty {
            setup.push(EffectRequest::DiscardFromHand {
                player: owner,
                count: u32::MAX,
            });
        }
        run_effects(&mut tx, setup).unwrap();
        let hand = tx.state().hand_count(owner);
        assert_eq!(
            run_effects(
                &mut tx,
                vec![EffectRequest::SetTurnState {
                    current: owner,
                    phase: engine_core::state::Phase::End.to_u8(),
                }],
            )
            .unwrap_err(),
            EffectError::ChoiceRequested
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
        assert_eq!(state.hand_count(owner), hand + if empty { 2 } else { 1 });
    }
}

#[test]
fn paoxiao_temp_skills_remove_sha_count_and_distance_limits() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_717,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(REZONGSHI_PAOXIAO),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(NEW_REPAOXIAO2),
            },
        ],
    )
    .unwrap();
    assert_eq!(
        engine_core::query::sha_use_limit_for(tx.state(), owner),
        i32::MAX as u32
    );
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
fn rezongshi_adds_group_hand_limit_and_grants_paoxiao_only_for_the_qualifying_turn() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: 20_260_717_2,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    for skill in tx.state().players.get(owner).unwrap().skills.clone() {
        run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    }

    let base_limit = match query(
        tx.state(),
        &QueryRequest::HandLimit { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    };
    let group_count = tx
        .state()
        .players
        .iter_live()
        .filter(|player| player.alive)
        .filter_map(|player| tx.state().content.character(player.character_def))
        .map(|character| character.group.as_str())
        .filter(|group| !group.is_empty())
        .collect::<std::collections::BTreeSet<_>>()
        .len() as i32;
    assert!(group_count > 0);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REZONGSHI),
        }],
    )
    .unwrap();
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(value) if value == base_limit + group_count
    ));

    let hand = tx.state().hand_count(owner);
    assert!(hand > 0);
    tx.state_mut().players.get_mut(owner).unwrap().hp = hand as i32 - 1;
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: engine_core::state::Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
    assert!(has_skill(&tx, owner, REZONGSHI_PAOXIAO));
    assert_eq!(
        engine_core::query::sha_use_limit_for(tx.state(), owner),
        i32::MAX as u32
    );

    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: engine_core::state::Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: other,
                phase: engine_core::state::Phase::Prepare.to_u8(),
            },
        ],
    )
    .unwrap();
    assert!(!has_skill(&tx, owner, REZONGSHI_PAOXIAO));
    assert_ne!(
        engine_core::query::sha_use_limit_for(tx.state(), owner),
        i32::MAX as u32
    );

    tx.state_mut().players.get_mut(owner).unwrap().hp = tx.state().hand_count(owner) as i32;
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: engine_core::state::Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
    assert!(!has_skill(&tx, owner, REZONGSHI_PAOXIAO));
}

#[test]
fn new_repaoxiao_removes_the_sha_limit_then_range_after_the_first_sha() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_717_1,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    for skill in tx.state().players.get(owner).unwrap().skills.clone() {
        run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NEW_REPAOXIAO),
        }],
    )
    .unwrap();

    assert_eq!(
        engine_core::query::sha_use_limit_for(tx.state(), owner),
        i32::MAX as u32
    );
    assert!(!engine_core::query::sha_distance_unlimited_for(
        tx.state(),
        owner
    ));
    tx.state_mut()
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 1);
    assert!(engine_core::query::sha_distance_unlimited_for(
        tx.state(),
        owner
    ));
}

#[test]
fn max_hp_hand_limit_skills_share_the_same_public_query_surface() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
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
    let owner = engine.state().turn.current;

    for skill_def in [REFANGQUAN2, REYINGZI] {
        let mut tx = Transaction::begin(engine.state().clone());
        tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    // Roster-independent: the booted standard/refresh roster includes
    // hand-limit and choice skills (e.g. 袁术 庸肆, 旧界李典 询询); drop every
    // booted skill so the queries below measure only the granted skills.
    {
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
    }

        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(skill_def),
            }],
        )
        .unwrap();
        let max_hp = tx.state().players.get(owner).unwrap().max_hp as i32;
        assert!(matches!(
            query(
                tx.state(),
                &QueryRequest::HandLimit { player: owner },
                &mut QueryContext::new(),
            ),
            QueryResponse::Int(value) if value == max_hp
        ));
        if skill_def == REYINGZI {
            assert_eq!(engine_core::query::draw_count_for(tx.state(), owner), 3);
        }
    }
}

#[test]
fn redanxin_draws_each_time_and_caps_its_upgrade_mark_at_two() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_719),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_719,
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
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.hp = 10;
        player.max_hp = 10;
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REDANXIN),
        }],
    )
    .unwrap();
    let initial_hand = tx.state().hand_count(owner);
    let mut state = tx.state().clone();

    for activation in 1..=3 {
        let mut damage = Transaction::begin(state);
        assert_eq!(
            run_effects(
                &mut damage,
                vec![EffectRequest::DealDamage {
                    source,
                    target: owner,
                    amount: 1,
                    is_lose_hp: false,
                    nature: rule_api::damage_nature::NORMAL,
                }],
            )
            .unwrap_err(),
            EffectError::ChoiceRequested
        );
        let (next, result) = apply_command(
            damage.state().clone(),
            Command {
                actor: owner,
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(activation),
                kind: CommandKind::ResolveChoice {
                    choice_id: ChoiceId::new(damage.state().resolution.pending_choice_id),
                    selections: vec![1],
                    cancelled: false,
                },
            },
        );
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        assert_eq!(next.hand_count(owner), initial_hand + activation as u32);
        assert_eq!(
            next.players
                .get(owner)
                .unwrap()
                .marks
                .iter()
                .find(|(mark, _)| *mark == REDANXIN_MARK)
                .map_or(0, |(_, count)| *count),
            activation.min(2) as i32
        );
        state = next;
    }
}
}

mod original_refresh_rewansha {
//! Golden coverage for REFRESH「完杀」.

use engine_api::{DomainEvent, MarkDefId, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const REWANSHA: u32 = 3_382_466_194;
const JILUE_WANSHA: u32 = 3_522_766_701;
const PROBE: u32 = 1_001;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/refresh/rewansha.skill"),
    "skill probe(1001) {}\n",
);
const JILUE_SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/extra/shen_simayi.skill"),
    "skill probe(1001) {}\n",
);

fn dispatch(tx: &mut Transaction, event: DomainEvent) {
    let effects = dispatch_for_events(tx, &[event], &mut TriggerFireRecord::new()).unwrap();
    run_effects(tx, effects).unwrap();
}

#[test]
fn suppresses_only_non_dying_others_during_the_dying_window() {
    let package = content_compiler::compile_source_named("rewansha", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_731, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let owner = tx.state().turn.current;
    let others: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
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
            skill_def: SkillDefId::new(REWANSHA),
        },
        EffectRequest::AddSkill {
            player: others[0],
            skill_def: SkillDefId::new(PROBE),
        },
        EffectRequest::AddSkill {
            player: others[1],
            skill_def: SkillDefId::new(PROBE),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(others[1]).unwrap().dying = true;

    dispatch(
        &mut tx,
        DomainEvent::EnteredDying {
            player: others[1],
            source: owner,
        },
    );
    let probe = |tx: &Transaction, player: PlayerId| {
        let skill = tx
            .state()
            .players
            .get(player)
            .unwrap()
            .skills
            .iter()
            .find_map(|skill| {
                tx.state()
                    .skills
                    .get(*skill)
                    .filter(|skill| skill.def.raw() == PROBE)
            })
            .unwrap();
        tx.state().skill_effectively_enabled(skill)
    };
    assert!(!probe(&tx, others[0]));
    assert!(probe(&tx, others[1]));

    tx.state_mut().players.get_mut(others[1]).unwrap().dying = false;
    dispatch(&mut tx, DomainEvent::LeftDying { player: others[1] });
    assert!(probe(&tx, others[0]));
}

#[test]
fn jilue_wansha_suppresses_only_while_its_activation_mark_exists() {
    let package = content_compiler::compile_source_named("jilue-wansha", JILUE_SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_731, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let owner = tx.state().turn.current;
    let others: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let old: Vec<_> = tx
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
                skill_def: SkillDefId::new(JILUE_WANSHA),
            },
            EffectRequest::AddSkill {
                player: others[0],
                skill_def: SkillDefId::new(PROBE),
            },
        ],
    )
    .unwrap();
    let probe_enabled = |tx: &Transaction| {
        let skill = tx
            .state()
            .players
            .get(others[0])
            .unwrap()
            .skills
            .iter()
            .find_map(|skill| {
                tx.state()
                    .skills
                    .get(*skill)
                    .filter(|skill| skill.def.raw() == PROBE)
            })
            .unwrap();
        tx.state().skill_effectively_enabled(skill)
    };
    tx.state_mut().players.get_mut(others[1]).unwrap().dying = true;
    dispatch(
        &mut tx,
        DomainEvent::EnteredDying {
            player: others[1],
            source: owner,
        },
    );
    assert!(probe_enabled(&tx));

    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: owner,
            mark: MarkDefId::new(JILUE_WANSHA),
            delta: 1,
        }],
    )
    .unwrap();
    dispatch(
        &mut tx,
        DomainEvent::EnteredDying {
            player: others[1],
            source: owner,
        },
    );
    assert!(!probe_enabled(&tx));

    tx.state_mut().players.get_mut(others[1]).unwrap().dying = false;
    dispatch(&mut tx, DomainEvent::LeftDying { player: others[1] });
    assert!(probe_enabled(&tx));
}
}

mod original_refresh_rezhongyong {
//! Golden coverage for 界周仓「忠勇」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_ZHOUCANG: u32 = 2_752_214_624;
const REZHONGYONG: u32 = 3_348_933_734;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/rezhongyong.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    recipient: PlayerId,
    other: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
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
        skill_def: SkillDefId::new(REZHONGYONG),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target: players[1],
        recipient: players[2],
        other: players[3],
    }
}

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, suit: Suit, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn confirm_trigger(state: GameState, owner: PlayerId, key: u64) -> (GameState, PipelineResult) {
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    resolve(state, owner, vec![1], false, key)
}

fn choice_tokens(result: &PipelineResult) -> Vec<u32> {
    result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect()
}

fn hand_count(state: &GameState, player: PlayerId) -> usize {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    state.zones.get(hand).unwrap().cards.len()
}

#[test]
fn package_publishes_complete_re_zhoucang_and_exact_abi310_module() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 310);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_ZHOUCANG)
        .unwrap();
    assert_eq!(character.skills, [REZHONGYONG]);
    let compiled = content_compiler::compile_source_named("rezhongyong", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 310);
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn all_sha_and_shan_sources_are_gifted_and_red_black_rewards_both_resolve() {
    let mut setup = setup(2_026_073_403);
    let sha = inject(
        &mut setup.state,
        setup.owner,
        CardKind::Sha,
        Suit::Club,
        99_421,
    );
    let shan = inject(
        &mut setup.state,
        setup.target,
        CardKind::Shan,
        Suit::Heart,
        99_422,
    );
    let (state, played) = submit(
        setup.state,
        setup.owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![setup.target],
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    let (state, responded) = resolve(state, setup.target, vec![shan.raw()], false, 2);
    assert!(matches!(responded.result, CommandResult::Accepted { .. }));
    let (state, recipient_choice) = confirm_trigger(state, setup.owner, 3);
    let recipients = choice_tokens(&recipient_choice);
    assert!(!recipients.contains(&setup.owner.raw()));
    assert!(!recipients.contains(&setup.target.raw()));
    assert!(recipients.contains(&setup.recipient.raw()));
    assert!(recipients.contains(&setup.other.raw()));

    let before = canonical_state_hash(&state);
    let (state, forged) = resolve(state, setup.owner, vec![setup.target.raw()], false, 4);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);

    let (state, gifted) = resolve(state, setup.owner, vec![setup.recipient.raw()], false, 5);
    assert!(matches!(gifted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(sha).unwrap().owner, setup.recipient);
    assert_eq!(state.cards.get(shan).unwrap().owner, setup.recipient);
    assert_eq!(hand_count(&state, setup.recipient), 2);
    assert_eq!(state.resolution.continuation_payload.first(), Some(&2));

    let (state, cancelled_sha) = resolve(state, setup.recipient, Vec::new(), true, 6);
    assert!(matches!(
        cancelled_sha.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(hand_count(&state, setup.recipient), 3);
    assert_eq!(state.resolution.continuation_kind, ContinuationKind::None);
}
}

mod original_refresh_shebian {
//! 第一百八十四批：refresh「设变」（OL夏侯渊 shebian）零新原语重写 + 新过滤器。
//! 上游：当你的武将牌翻面后（player turnOverEnd），可移动场上的一张装备牌。
//! 实现：character_state_changed + owner_turned_over（本批新注册，双向翻面，
//! 5 处注册零 ABI）→ optional 确认 → choose_move_field_card_to(self, self)。
//! 近似（登记债）：原语可选集含判定区、目标固定为自己。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SHEBIAN: u32 = 3_980_781_998;
const OL_XIAHOUYUAN: u32 = 3_694_177_949;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/shebian.skill");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("shebian-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
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
        skill_def: SkillDefId::new(SHEBIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

/// 给 `target` 装一匹防御马（可被移动的场牌——移动者自己没有防马时合法）。
fn equip_mount(state: &mut GameState, target: PlayerId, kind: CardKind) -> CardId {
    let mut tx = Transaction::begin(state.clone());
    let zone = tx.state().zones.find(ZoneKind::Equip, target).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_999),
        kind,
        suit: Suit::Spade,
        rank: 1,
        weapon_range: 0,
        zone,
        owner: target,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    *state = tx.commit().state;
    card
}

/// 派发翻面事件。optional 触发的 confirm 在 dispatch 期打开（Err(ChoiceRequested)
/// 由命令管线负责提交），此处手动提交并返回「confirm 是否打开」。
fn turn_over(state: GameState, who: PlayerId) -> (GameState, bool) {
    let mut tx = Transaction::begin(state);
    let confirmed = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::PlayerTurnedOver {
            player: who,
            turned_over: true,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Err(EffectError::ChoiceRequested) => true,
        Ok(effects) => {
            run_effects(&mut tx, effects).unwrap();
            false
        }
        Err(other) => panic!("unexpected dispatch error: {other:?}"),
    };
    (tx.commit().state, confirmed)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let (next, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    );
    (next, result)
}

#[test]
fn package_publishes_shebian_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 681);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SHEBIAN)
        .unwrap();
    assert_eq!(trigger.event, "character_state_changed");
    assert_eq!(trigger.filter, "owner_turned_over");
    assert!(trigger.optional, "upstream shebian is a may-trigger");
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == OL_XIAHOUYUAN)
        .expect("ol_xiahouyuan in roster");
    assert!(character.skills.contains(&SHEBIAN));
}

#[test]
fn own_turn_over_opens_confirm_then_field_card_moves_to_owner() {
    let (mut state, owner, other) = setup(20_260_909_05);
    let horse = equip_mount(&mut state, other, CardKind::HorseDef);

    let (state, confirmed) = turn_over(state, owner);
    assert!(confirmed, "optional confirm must open");
    assert!(state.has_pending_choice());
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::state::ContinuationKind::OptionalTrigger,
    );

    // 段一：确认发动。
    let (state, result) = resolve(state, owner, vec![1], false, 20_260_909_51);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice(), "SelectCards must follow confirm");

    // 段二：选那张防马移到自己装备区。
    let (state, result) = resolve(state, owner, vec![horse.raw()], false, 20_260_909_52);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "move rejected: {:?}",
        result.result
    );
    assert!(!state.has_pending_choice());
    let owner_equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    assert!(
        state
            .zones
            .get(owner_equip)
            .unwrap()
            .cards
            .contains(&horse),
        "horse must land in owner's equip zone"
    );
    let other_equip = state.zones.find(ZoneKind::Equip, other).unwrap();
    assert!(
        !state.zones.get(other_equip).unwrap().cards.contains(&horse),
        "horse must leave other's equip zone"
    );
}

#[test]
fn others_turn_over_does_not_trigger() {
    let (mut state, owner, other) = setup(20_260_909_06);
    let _ = equip_mount(&mut state, other, CardKind::HorseDef);

    let (state, confirmed) = turn_over(state, other);
    assert!(
        !confirmed && !state.has_pending_choice(),
        "owner filter must suppress other-player turn-overs"
    );
}
}

mod original_refresh_shizhan {
//! Golden coverage for 势斩's reversed virtual-Duel active skill.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{CardKind, GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SHIZHAN: u32 = 3_879_999_347;
const REYAOWU: u32 = 2_805_206_003;
const RE_HUAXIONG: u32 = 3_874_151_406;
const JUE_DOU_LOCK: u32 = 2_000_000_113;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/shizhan.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn setup(lock_target: bool) -> (GameState, PlayerId, PlayerId, SkillId) {
    let source = if lock_target {
        format!(
            "{SOURCE}\nskill juedou_lock({JUE_DOU_LOCK}) locked {{ modifier card_use_cancel(juedou) override 1; }}"
        )
    } else {
        SOURCE.to_owned()
    };
    let package = content_compiler::compile_source_named("shizhan-test", &source).unwrap();
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
        skill_def: SkillDefId::new(SHIZHAN),
    });
    if lock_target {
        effects.push(EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(JUE_DOU_LOCK),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    for player in [owner, target] {
        for card in tx.state().hand_cards(player) {
            tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
        }
    }
    tx.state_mut().players.get_mut(owner).unwrap().hp = 6;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 6;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == SHIZHAN)
        .unwrap();
    (tx.state().clone(), owner, target, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
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
                targets: vec![target],
                card_targets: vec![],
            },
        },
    )
}

#[test]
fn committed_package_declares_target_legality_and_huaxiong() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 23);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == SHIZHAN)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(2));
    let targets = active.targets.as_ref().unwrap();
    assert!(!targets.allow_self);
    assert_eq!(targets.can_use_on_owner.as_deref(), Some("juedou"));
    assert!(package.character_index.iter().any(|character| {
        character.id == RE_HUAXIONG
            && character.max_hp == 6
            && character.skills == vec![REYAOWU, SHIZHAN]
    }));
}

#[test]
fn selected_target_uses_duel_on_owner_and_limit_is_two() {
    let (state, owner, target, skill) = setup(false);
    let owner_hp = state.players.get(owner).unwrap().hp;
    let target_hp = state.players.get(target).unwrap().hp;

    let (state, first) = activate(state, owner, target, skill, 1);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp);

    let (state, second) = activate(state, owner, target, skill, 2);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 2);
    assert_eq!(state.skills.get(skill).unwrap().use_count, 2);

    let (same, third) = activate(state.clone(), owner, target, skill, 3);
    assert!(matches!(third.result, CommandResult::Rejected { .. }));
    assert_eq!(same, state);
}

#[test]
fn target_that_cannot_use_duel_is_rejected_without_consuming_the_skill() {
    let (state, owner, target, skill) = setup(true);
    let (same, result) = activate(state.clone(), owner, target, skill, 10);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(same, state);
    assert_eq!(same.skills.get(skill).unwrap().use_count, 0);
}
}

mod original_refresh_view_as {
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
}

mod original_refresh_xinjiangchi_re_caozhang {
//! Golden coverage for 界曹彰「将驰」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{
    card_distance_unlimited_for_card, sha_response_cancelled_for, sha_use_limit_for,
};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_CAOZHANG: u32 = 2_557_216_502;
const XINJIANGCHI: u32 = 3_041_318_040;
const XINJIANGCHI_LESS: u32 = 2_327_944_393;
const XINJIANGCHI_MORE: u32 = 3_884_080_518;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/xinjiangchi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
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
        skill_def: SkillDefId::new(XINJIANGCHI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Draw;
    Setup {
        state: tx.commit().state,
        owner,
        other: players[1],
    }
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn end_phase(state: GameState, owner: PlayerId, key: u64) -> (GameState, PipelineResult) {
    submit(state, owner, CommandKind::EndPhase, key)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn open_branch_choice(setup: Setup, key: u64) -> (GameState, PipelineResult) {
    let (state, opened) = end_phase(setup.state, setup.owner, key);
    assert!(matches!(opened.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDrawOrDiscardAddSkill
    );
    (state, opened)
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

fn finish_turn(mut state: GameState, owner: PlayerId, key: u64) -> GameState {
    for offset in 0..3 {
        let (next, result) = end_phase(state, owner, key + offset);
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        state = next;
    }
    state
}

#[test]
fn package_publishes_complete_re_caozhang_and_exact_abi315_modules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 315);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_CAOZHANG)
        .unwrap();
    assert_eq!(character.skills, [XINJIANGCHI]);
    for skill in [XINJIANGCHI, XINJIANGCHI_LESS, XINJIANGCHI_MORE] {
        assert!(package
            .skill_index
            .iter()
            .any(|entry| entry.skill_def_id == skill));
    }
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == XINJIANGCHI)
        .unwrap();
    assert_eq!(trigger.event, "phase_play");
    assert!(!trigger.optional);

    let compiled = content_compiler::compile_source_named("xinjiangchi", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 315);
    for expected in compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn cancel_and_plain_draw_are_available_without_a_discard_cost() {
    let cancellable = setup(2_026_073_101);
    let owner = cancellable.owner;
    let hand = cancellable.state.hand_count(owner);
    let (state, branch) = open_branch_choice(cancellable, 1);
    let choice = branch.choice.as_ref().unwrap();
    assert!(choice.allow_cancel);
    assert_eq!((choice.min_count, choice.max_count), (0, 1));
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![1, 2]
    );
    let (state, cancelled) = resolve(state, owner, Vec::new(), true, 2);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand);
    assert!(!has_skill(&state, owner, XINJIANGCHI_LESS));
    assert!(!has_skill(&state, owner, XINJIANGCHI_MORE));

    let plain = setup(2_026_073_102);
    let owner = plain.owner;
    let hand = plain.state.hand_count(owner);
    let (state, _) = open_branch_choice(plain, 3);
    let (state, drawn) = resolve(state, owner, vec![1], false, 4);
    assert!(matches!(drawn.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 1);
    assert!(!has_skill(&state, owner, XINJIANGCHI_LESS));
    assert!(!has_skill(&state, owner, XINJIANGCHI_MORE));
}

#[test]
fn draw_two_branch_blocks_sha_use_and_response_until_turn_end() {
    let setup = setup(2_026_073_103);
    let owner = setup.owner;
    let hand = setup.state.hand_count(owner);
    let (state, _) = open_branch_choice(setup, 10);
    let (state, drawn) = resolve(state, owner, vec![2], false, 11);
    assert!(matches!(drawn.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand + 2);
    assert!(has_skill(&state, owner, XINJIANGCHI_LESS));
    assert_eq!(sha_use_limit_for(&state, owner), 0);
    assert!(sha_response_cancelled_for(&state, owner));

    let state = finish_turn(state, owner, 12);
    assert!(!has_skill(&state, owner, XINJIANGCHI_LESS));
}

#[test]
fn discard_branch_revalidates_cost_and_boosts_sha_until_turn_end() {
    let mut setup = setup(2_026_073_104);
    let owner = setup.owner;
    let cost = inject(
        &mut setup.state,
        owner,
        ZoneKind::Equip,
        99_321,
        CardKind::Weapon,
    );
    let sha = inject(
        &mut setup.state,
        owner,
        ZoneKind::Hand,
        99_322,
        CardKind::Sha,
    );
    let forged = inject(
        &mut setup.state,
        setup.other,
        ZoneKind::Hand,
        99_323,
        CardKind::Tao,
    );
    let (state, branch) = open_branch_choice(setup, 20);
    assert_eq!(
        branch
            .choice
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    let (state, cost_choice) = resolve(state, owner, vec![3], false, 21);
    assert!(matches!(cost_choice.result, CommandResult::Accepted { .. }));

    let before = canonical_state_hash(&state);
    let (same, rejected) = resolve(state, owner, vec![forged.raw()], false, 22);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&same), before);

    let (state, paid) = resolve(same, owner, vec![cost.raw()], false, 23);
    assert!(matches!(paid.result, CommandResult::Accepted { .. }));
    assert!(has_skill(&state, owner, XINJIANGCHI_MORE));
    assert_eq!(sha_use_limit_for(&state, owner), 2);
    assert!(card_distance_unlimited_for_card(&state, owner, sha));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(cost).unwrap().zone, discard);

    let state = finish_turn(state, owner, 24);
    assert!(!has_skill(&state, owner, XINJIANGCHI_MORE));
}
}

mod original_refresh_yijue {
//! Golden coverage for 关羽「义绝」.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const YIJUE: u32 = 3_159_500_535;
const YIJUE_WIN: u32 = 3_572_578_464;
const YIJUE_LOSE: u32 = 3_300_215_091;
const YIJUE2: u32 = 2_542_175_146;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/refresh/yijue.skill"),
    include_str!("../../../content/original/ported/refresh/yijue2.skill"),
);

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("yijue-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_801, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, players[0], players[1])
}

fn add_skill(state: GameState, player: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    tx.commit().state
}

fn compare_effects(
    tx: &mut Transaction,
    owner: PlayerId,
    target: PlayerId,
    winner: PlayerId,
) -> Vec<EffectRequest> {
    dispatch_for_events(
        tx,
        &[DomainEvent::HandCompareFinished {
            player: owner,
            target,
            player_card: CardId::NONE,
            target_card: CardId::NONE,
            winner,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
}

#[test]
fn package_win_lock_and_non_win_recovery_are_exact() {
    let package = content_compiler::compile_source_named("yijue-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 356);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == YIJUE)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.targets.as_ref().unwrap().can_compare);

    let (state, owner, target) = setup();
    let state = add_skill(add_skill(state, owner, YIJUE_WIN), target, YIJUE);
    let mut tx = Transaction::begin(state);
    let effects = compare_effects(&mut tx, owner, target, owner);
    run_effects(&mut tx, effects).unwrap();
    assert!(tx
        .state()
        .turn
        .non_locked_skill_suppressed_players
        .contains(&target));
    let target_skills: Vec<_> = tx
        .state()
        .players
        .get(target)
        .unwrap()
        .skills
        .iter()
        .filter_map(|skill| tx.state().skills.get(*skill))
        .collect();
    assert!(target_skills.iter().any(|skill| skill.def.raw() == YIJUE2));
    assert!(target_skills
        .iter()
        .find(|skill| skill.def.raw() == YIJUE)
        .is_some_and(|skill| !tx.state().skill_effectively_enabled(skill)));

    let (mut state, owner, target) = setup();
    state.players.get_mut(target).unwrap().hp -= 1;
    let state = add_skill(state, owner, YIJUE_LOSE);
    let mut tx = Transaction::begin(state);
    let effects = compare_effects(&mut tx, owner, target, target);
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let hp = tx.state().players.get(target).unwrap().hp;
    let state = tx.commit().state;
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![2],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp + 1);
}
}

mod original_refresh_zhaxiang_re_huanggai {
//! Golden coverage for REFRESH「诈降」and the complete re_huanggai closure.

use engine_api::{CardDefId, CardId, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{
    card_distance_unlimited_for_card, sha_unrespondable_for_card, sha_use_limit_for_card,
};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZHAXIANG: u32 = 3_298_876_867;
const ZHAXIANG_EFFECT: u32 = 4_056_078_041;
const RE_HUANGGAI: u32 = 3_020_853_843;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/zhaxiang.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn started(phase: Phase) -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("zhaxiang-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, ZHAXIANG as u64, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(ZHAXIANG),
    });
    effects.push(EffectRequest::SetTurnState {
        current: owner,
        phase: phase.to_u8(),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn inject_sha(tx: &mut Transaction, owner: PlayerId, suit: Suit, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn lose_hp(tx: &mut Transaction, owner: PlayerId, amount: u32) {
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: owner,
            amount,
            is_lose_hp: true,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
}

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
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

fn mark_count(state: &GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(id, _)| *id == ZHAXIANG_EFFECT)
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[test]
fn package_publishes_abi106_skill_helper_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 251);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == ZHAXIANG && skill.name == "zhaxiang"));
    assert!(package
        .skill_index
        .iter()
        .any(|skill| { skill.skill_def_id == ZHAXIANG_EFFECT && skill.name == "zhaxiang_effect" }));
    assert!(package.character_index.iter().any(|character| {
        character.id == RE_HUANGGAI
            && character.skills == vec![2_651_237_190, ZHAXIANG]
            && character.max_hp == 4
    }));
}

#[test]
fn hp_loss_draws_three_per_point_but_only_play_phase_grants_the_helper() {
    let (mut tx, owner, _) = started(Phase::End);
    let before = tx.state().hand_count(owner);
    lose_hp(&mut tx, owner, 2);
    assert_eq!(tx.state().hand_count(owner), before + 6);
    assert!(!has_skill(tx.state(), owner, ZHAXIANG_EFFECT));
    assert_eq!(mark_count(tx.state(), owner), 0);
}

#[test]
fn play_phase_hp_loss_stacks_sha_uses_and_red_sha_rules_until_turn_end() {
    let (mut tx, owner, other) = started(Phase::Play);
    let red_sha = inject_sha(&mut tx, owner, Suit::Heart, 106_001);
    let black_sha = inject_sha(&mut tx, owner, Suit::Spade, 106_002);
    let base_limit = sha_use_limit_for_card(tx.state(), owner, red_sha);
    let before = tx.state().hand_count(owner);

    lose_hp(&mut tx, owner, 2);

    assert_eq!(tx.state().hand_count(owner), before + 6);
    assert!(has_skill(tx.state(), owner, ZHAXIANG_EFFECT));
    assert_eq!(mark_count(tx.state(), owner), 2);
    assert_eq!(
        sha_use_limit_for_card(tx.state(), owner, red_sha),
        base_limit + 2
    );
    assert!(card_distance_unlimited_for_card(tx.state(), owner, red_sha));
    assert!(sha_unrespondable_for_card(tx.state(), owner, red_sha));
    assert!(!card_distance_unlimited_for_card(
        tx.state(),
        owner,
        black_sha
    ));
    assert!(!sha_unrespondable_for_card(tx.state(), owner, black_sha));

    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::SetTurnState {
                current: other,
                phase: Phase::Prepare.to_u8(),
            },
        ],
    )
    .unwrap();
    assert!(!has_skill(tx.state(), owner, ZHAXIANG_EFFECT));
    assert_eq!(mark_count(tx.state(), owner), 0);
    assert_eq!(
        sha_use_limit_for_card(tx.state(), owner, red_sha),
        base_limit
    );
    assert!(!card_distance_unlimited_for_card(
        tx.state(),
        owner,
        red_sha
    ));
    assert!(!sha_unrespondable_for_card(tx.state(), owner, red_sha));
}
}
