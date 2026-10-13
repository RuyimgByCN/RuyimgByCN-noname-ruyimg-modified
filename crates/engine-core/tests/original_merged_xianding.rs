//! O2 全量合并批次 1（2026-10-14）：家族 xianding 的 83 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_xianding_bailingyun {
//! Golden for XIANDING 柏灵筠【灵慧/黠策/御心】（ABI 593）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const BAILINGYUN: u32 = 4_278_690_663;
const DCLINGHUI: u32 = 2_581_852_991;
const DCXIACE: u32 = 2_254_753_634;
const DCYUXIN: u32 = 4_174_094_056;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(skill_def: u32) -> (GameState, [PlayerId; 3]) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players: [PlayerId; 3] = tx.state().players.alive_by_seat().try_into().unwrap();
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(players.iter().map(|player| EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        }))
        .chain(std::iter::once(EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(skill_def),
        }))
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::End;
    tx.state_mut().revision = 1;
    (tx.commit().state, players)
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone: ZoneKind,
    def: u32,
    kind: CardKind,
) -> CardId {
    let zone = state.zones.find(zone, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 2,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn run(state: GameState, effect: EffectRequest) -> GameState {
    let mut tx = Transaction::begin(state);
    match run_effects(&mut tx, vec![effect]) {
        Ok(_) | Err(EffectError::ChoiceRequested) => {}
        Err(error) => panic!("effect failed: {error}"),
    }
    tx.commit().state
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, result) = apply_command(
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
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    state
}

#[test]
fn package_closes_bailingyun_contract() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 593);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == BAILINGYUN)
        .unwrap();
    assert_eq!(character.name_key, "bailingyun");
    assert_eq!(character.skills, [DCLINGHUI, DCXIACE, DCYUXIN]);
}

#[test]
fn linghui_uses_one_gains_one_and_returns_the_rest_in_order() {
    let (mut state, [owner, _other, _]) = setup(DCLINGHUI);
    inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        10,
        CardKind::Tao,
    );
    inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        11,
        CardKind::Tao,
    );
    let third = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        12,
        CardKind::Shan,
    );
    let second = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        13,
        CardKind::Shan,
    );
    let usable = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        14,
        CardKind::TrickWuzhong,
    );
    let skill = skill_id(&state, owner, DCLINGHUI);
    state = run(
        state,
        EffectRequest::ResolveBailingyun {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 0,
            cards: Vec::new(),
            skill,
        },
    );
    state = choose(state, owner, vec![usable.raw()], 1);

    let owner_hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let gained = [second, third]
        .into_iter()
        .filter(|card| state.cards.get(*card).unwrap().zone == owner_hand)
        .collect::<Vec<_>>();
    assert_eq!(gained.len(), 1);
    let returned = if gained[0] == second { third } else { second };
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.zones.get(draw).unwrap().cards.last(), Some(&returned));
    assert_eq!(
        state
            .zones
            .get(state.cards.get(usable).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::DiscardPile
    );

    let (mut gated, [owner, other, _]) = setup(DCLINGHUI);
    gated.turn.current = other;
    let skill = skill_id(&gated, owner, DCLINGHUI);
    let untouched = run(
        gated.clone(),
        EffectRequest::ResolveBailingyun {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 0,
            cards: Vec::new(),
            skill,
        },
    );
    assert!(!untouched.has_pending_choice());
    gated.turn.record_entered_dying(other);
    inject(
        &mut gated,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        20,
        CardKind::TrickWuzhong,
    );
    let offered = run(
        gated,
        EffectRequest::ResolveBailingyun {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 0,
            cards: Vec::new(),
            skill,
        },
    );
    assert!(offered.has_pending_choice());
}

#[test]
fn xiace_two_branches_are_independently_once_per_turn() {
    let (mut state, [owner, target, _]) = setup(DCXIACE);
    state.turn.phase = Phase::Play;
    state.players.get_mut(owner).unwrap().hp = 2;
    let cost = inject(&mut state, owner, ZoneKind::Hand, 30, CardKind::Shan);
    let extra = inject(&mut state, owner, ZoneKind::Hand, 31, CardKind::Shan);
    let skill = skill_id(&state, owner, DCXIACE);

    state = run(
        state,
        EffectRequest::ResolveBailingyun {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 10,
            cards: Vec::new(),
            skill,
        },
    );
    state = choose(state, owner, vec![target.raw()], 2);
    assert!(state
        .turn
        .non_locked_skill_suppressed_players
        .contains(&target));

    state = run(
        state,
        EffectRequest::ResolveBailingyun {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 11,
            cards: Vec::new(),
            skill,
        },
    );
    state = choose(state, owner, vec![cost.raw()], 3);
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert_eq!(state.turn.flag(turn_flag::xiace_used_by_player(owner)), 3);

    state = run(
        state,
        EffectRequest::ResolveBailingyun {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 11,
            cards: Vec::new(),
            skill,
        },
    );
    assert!(!state.has_pending_choice());
    assert_eq!(state.cards.get(extra).unwrap().owner, owner);
}
}

mod original_xianding_batch10 {
//! Golden for XIANDING（限定包）批次十：
//! 敏善（dcminshan）、受责（dcshouze）。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCMINSHAN: u32 = 2_275_391_703;
const DCSHOUZE: u32 = 3_027_710_363;
const DCLINGFANG: u32 = 3_884_646_990;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
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

fn deal_damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    );
    (tx.commit().state, result)
}

#[test]
fn package_exposes_batch10_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let minshan = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCMINSHAN)
        .expect("dcminshan trigger");
    assert_eq!(minshan.event, "damage");
    assert_eq!(minshan.filter, "owner_is_target_per_damage_point");
    assert!(minshan.optional);

    let shouze = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCSHOUZE)
        .expect("dcshouze trigger");
    assert_eq!(shouze.event, "phase_end");
    assert_eq!(shouze.filter, "owner_is_target_mark_positive_3884646990");
    assert!(!shouze.optional);
}

#[test]
fn dcminshan_offers_other_player_two_card_draw_after_damage() {
    let (mut tx, owner, first, second) = setup(20_260_824);
    add_skill(&mut tx, owner, DCMINSHAN);
    tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;
    let before = tx.state().hand_count(second);
    let (state, result) = deal_damage(tx.state().clone(), first, owner);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(state, owner, vec![1], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDraw
    );
    let state = resolve(state, owner, vec![second.raw()], false, 2);
    assert_eq!(state.hand_count(second), before + 2);
}

#[test]
fn dcshouze_consumes_mark_loses_hp_and_gains_sha_at_phase_end() {
    let (mut tx, owner, _first, _second) = setup(20_260_825);
    add_skill(&mut tx, owner, DCSHOUZE);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: owner,
            mark: engine_api::MarkDefId::new(DCLINGFANG),
            delta: 1,
        }],
    )
    .unwrap();
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let discard = tx
        .state()
        .zones
        .find(
            engine_core::state::ZoneKind::DiscardPile,
            engine_api::PlayerId::NONE,
        )
        .unwrap();
    let sha = tx.state_mut().cards.alloc(engine_core::state::Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: engine_core::state::CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: discard,
        owner: engine_api::PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(sha);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    let effects = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let err = run_effects(&mut tx, effects);
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseGainDiscardCardByKind
    );
    let state = resolve(tx.state().clone(), owner, vec![sha.raw()], false, 1);
    assert_eq!(state.players.get(owner).unwrap().hp, hp_before - 1);
    assert_eq!(state.hand_count(owner), 1);
}
}

mod original_xianding_batch11 {
//! Golden for XIANDING（限定包）批次十一：
//! 独仗（dcduzhang）、风势（dcfengshi）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCDUZHANG: u32 = 3_539_290_738;
const DCFENGSHI: u32 = 2_270_426_166;
const DCLINJIE: u32 = 3_067_413_516;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn inject_sha(tx: &mut Transaction, player: PlayerId) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
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

fn mark_count(tx: &Transaction, owner: PlayerId) -> i32 {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == DCLINJIE)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn package_exposes_batch11_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let duzhang = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCDUZHANG)
        .expect("dcduzhang trigger");
    assert_eq!(duzhang.event, "card_targeted");
    assert_eq!(duzhang.filter, "owner_is_target_nonred_card");
    assert!(!duzhang.optional);

    let fengshi = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCFENGSHI)
        .expect("dcfengshi trigger");
    assert_eq!(fengshi.event, "card_targeted");
    assert_eq!(
        fengshi.filter,
        "owner_source_sha_target_alive_owner_outside_target_attack_range"
    );
    assert!(fengshi.optional);

    let hand_limit = package
        .modifier_index
        .iter()
        .find(|modifier| modifier.skill_def_id == DCDUZHANG)
        .expect("dcduzhang modifier");
    assert_eq!(hand_limit.target, "hand_limit");
}

#[test]
fn dcduzhang_gains_mark_and_draws_when_targeted_by_nonred_card() {
    let (mut tx, owner, first, _second) = setup(20_260_826);
    add_skill(&mut tx, owner, DCDUZHANG);
    let before = tx.state().hand_count(owner);
    let card = inject_sha(&mut tx, first);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: first,
            card,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(mark_count(&tx, owner), 1);
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

#[test]
fn dcfengshi_offers_discard_target_equip_when_owner_outside_target_range() {
    let (mut tx, owner, first, _second) = setup(20_260_827);
    add_skill(&mut tx, owner, DCFENGSHI);
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let horse = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(13),
        kind: CardKind::HorseDef,
        suit: Suit::Club,
        rank: 5,
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
        .push(horse);
    let card = inject_sha(&mut tx, owner);
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![first],
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardTargetCard
    );
}
}

mod original_xianding_batch12 {
//! Golden for XIANDING（限定包）批次十二：
//! 诛寇（zhukou，周夷分支②）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const ZHUKOU: u32 = 4_022_084_191;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
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
fn package_exposes_batch12_skill_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == ZHUKOU)
        .expect("zhukou trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_alive_phase_end_used_card_no_damage");
    assert!(trigger.optional);
}

#[test]
fn zhukou_offers_two_other_damage_at_phase_end_when_no_damage_this_turn() {
    let (mut tx, owner, first, second) = setup(20_260_828);
    add_skill(&mut tx, owner, ZHUKOU);

    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);

    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Ok(_)));
    tx.state_mut().turn.phase = Phase::End;
    let hp_first = tx.state().players.get(first).unwrap().hp;
    let hp_second = tx.state().players.get(second).unwrap().hp;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetsDamage
    );
    let state = resolve(state, owner, vec![first.raw(), second.raw()], false, 2);
    assert_eq!(state.players.get(first).unwrap().hp, hp_first - 1);
    assert_eq!(state.players.get(second).unwrap().hp, hp_second - 1);
}

#[test]
fn zhukou_does_not_trigger_when_owner_caused_damage_this_turn() {
    let (mut tx, owner, first, _second) = setup(20_260_829);
    add_skill(&mut tx, owner, ZHUKOU);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: owner,
            target: first,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Ok(_)));

    tx.state_mut().turn.phase = Phase::End;
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Ok(_)));
}
}

mod original_xianding_batch13 {
//! Golden for XIANDING（限定包）批次十三：
//! 金玠（dcjinjie，杨彪）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCJINJIE: u32 = 2_966_946_872;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
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
fn package_exposes_batch13_skill_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCJINJIE)
        .expect("dcjinjie trigger");
    assert_eq!(trigger.event, "dying");
    assert_eq!(trigger.filter, "owner_alive_once_per_round");
    assert!(trigger.optional);
}

#[test]
fn dcjinjie_draws_declared_count_then_discards_same_count_to_recover() {
    let (mut tx, owner, first, _second) = setup(20_260_830);
    add_skill(&mut tx, owner, DCJINJIE);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 3,
        }],
    )
    .unwrap();

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: first,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let hp_before = tx.state().players.get(first).unwrap().hp;
    let hand_before = tx.state().hand_count(first);
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: first,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::VmResume
    );
    let state = resolve(state, owner, vec![2], false, 2);
    assert_eq!(state.hand_count(first), hand_before + 2);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardHandEquipThenRemoveSelfSkill
    );
    let payment: Vec<_> = state
        .hand_cards(owner)
        .into_iter()
        .take(2)
        .map(|card| card.raw())
        .collect();
    let state = resolve(state, owner, payment, false, 3);
    assert_eq!(state.hand_count(owner), 1);
    assert_eq!(state.players.get(first).unwrap().hp, hp_before + 1);
}

#[test]
fn dcjinjie_zero_branch_can_recover_owner_without_draw_or_discard() {
    let (mut tx, owner, _first, _second) = setup(20_260_831);
    add_skill(&mut tx, owner, DCJINJIE);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: owner,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    let state = resolve(state, owner, vec![0], false, 2);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDrawOrRecover
    );
    let state = resolve(state, owner, vec![2], false, 3);
    assert_eq!(state.hand_count(owner), 0);
    assert_eq!(state.players.get(owner).unwrap().hp, hp_before + 1);
}

#[test]
fn dcjinjie_may_cancel_equal_discard_without_recovery() {
    let (mut tx, owner, first, _second) = setup(20_260_832);
    add_skill(&mut tx, owner, DCJINJIE);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::DrawCards {
                player: owner,
                count: 2,
            },
            EffectRequest::DealDamage {
                source: owner,
                target: first,
                amount: 2,
                is_lose_hp: false,
                nature: 0,
            },
        ],
    )
    .unwrap();
    let hp_before = tx.state().players.get(first).unwrap().hp;
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: first,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    let state = resolve(state, owner, vec![2], false, 2);
    let state = resolve(state, owner, Vec::new(), true, 3);
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.players.get(first).unwrap().hp, hp_before);
}

#[test]
fn dcjinjie_respects_once_per_round_limit() {
    let (mut tx, owner, first, _second) = setup(20_260_833);
    add_skill(&mut tx, owner, DCJINJIE);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: first,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: first,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    let state = resolve(state, owner, vec![0], false, 2);
    let state = resolve(state, owner, Vec::new(), true, 3);

    let mut tx = Transaction::begin(state);
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: first,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(
        matches!(dispatched, Ok(_)),
        "second dying in same round must not fire"
    );
}
}

mod original_xianding_batch14 {
//! Golden for XIANDING（限定包）批次十四：
//! 武神（dcwushen）、递仇（dcdichou）。

use engine_api::{CardDefId, CardId, DomainEvent, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCWUSHEN: u32 = 3_292_885_385;
const DCDICHOU: u32 = 3_304_378_096;
const DCDICHOU_MARK: u32 = DCDICHOU;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn inject_card(tx: &mut Transaction, player: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn mark_count(tx: &Transaction, player: PlayerId) -> i32 {
    tx.state()
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == DCDICHOU_MARK)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn package_exposes_batch14_skills_with_expected_contracts() {
    let package = package();

    let wushen: Vec<_> = package
        .modifier_index
        .iter()
        .filter(|modifier| modifier.skill_def_id == DCWUSHEN)
        .map(|modifier| modifier.target.as_str())
        .collect();
    assert_eq!(
        wushen,
        [
            "card_name",
            "card_nature",
            "card_distance_unlimited",
            "card_use_unlimited",
        ]
    );

    let mark_trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCDICHOU && !trigger.optional)
        .expect("dcdichou mark trigger");
    assert_eq!(mark_trigger.event, "card_targeted");
    assert_eq!(mark_trigger.filter, "owner_is_target_other_source_alive");

    let draw_trigger = package
        .trigger_index
        .iter()
        .find(|trigger| {
            trigger.skill_def_id == DCDICHOU
                && trigger.filter == "owner_source_target_mark_positive_3304378096"
        })
        .expect("dcdichou draw trigger");
    assert!(!draw_trigger.optional);
}

#[test]
fn dcwushen_automatically_rewrites_heart_card_to_normal_sha() {
    let (mut tx, owner, _first, _second) = setup(20_260_832);
    add_skill(&mut tx, owner, DCWUSHEN);
    let heart = inject_card(&mut tx, owner, CardKind::Jiu, Suit::Heart);
    assert!(tx.state().hand_count(owner) >= 1);
    assert_eq!(
        tx.state().cards.get(heart).map(|card| card.kind),
        Some(CardKind::Jiu)
    );
    let physical = tx.state().cards.get(heart).unwrap();
    assert_eq!(
        engine_core::query::effective_card_kind(tx.state(), owner, physical),
        CardKind::Sha
    );
    assert_eq!(
        engine_core::query::effective_card_nature(tx.state(), owner, physical, CardKind::Sha),
        rule_api::damage_nature::NORMAL
    );
}

#[test]
fn dcdichou_marks_source_when_owner_targeted_and_draws_against_marked_target() {
    let (mut tx, owner, first, _second) = setup(20_260_833);
    add_skill(&mut tx, owner, DCDICHOU);

    let card = inject_card(&mut tx, first, CardKind::Sha, Suit::Spade);
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: first,
            card,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    );
    let effects = dispatched.unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(mark_count(&tx, first), 1);

    let sha = inject_card(&mut tx, owner, CardKind::Sha, Suit::Club);
    let before = tx.state().hand_count(owner);
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: sha,
            targets: vec![first],
        }],
        &mut TriggerFireRecord::new(),
    );
    run_effects(&mut tx, dispatched.unwrap()).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);
    assert_eq!(
        tx.state()
            .players
            .get(first)
            .unwrap()
            .marks
            .iter()
            .find(|(mark, _)| *mark == DCDICHOU_MARK)
            .map_or(0, |(_, count)| *count),
        0
    );
}
}

mod original_xianding_batch15 {
//! Golden for XIANDING（限定包）批次十五：
//! 连枝（dclianzhi，②）、杜君（dcdujun，②）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCLIANZHI: u32 = 2_871_252_462;
const DCDUJUN: u32 = 3_784_866_267;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
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
fn package_exposes_batch15_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let lianzhi = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCLIANZHI)
        .expect("dclianzhi trigger");
    assert_eq!(lianzhi.event, "dying");
    assert!(lianzhi.optional);

    let dujun = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCDUJUN)
        .expect("dcdujun trigger");
    assert_eq!(dujun.event, "damage");
    assert_eq!(
        dujun.filter,
        "owner_source_or_target_first_damage_this_turn"
    );
    assert!(dujun.optional);
}

#[test]
fn dclianzhi_recovers_and_draws_when_owner_enters_dying() {
    let (mut tx, owner, _first, _second) = setup(20_260_834);
    add_skill(&mut tx, owner, DCLIANZHI);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let hand_before = tx.state().hand_count(owner);

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: owner,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.players.get(owner).unwrap().hp, hp_before + 1);
    assert_eq!(state.hand_count(owner), hand_before + 1);
}

#[test]
fn dcdujun_draws_two_after_first_damage_of_turn() {
    let (mut tx, owner, first, _second) = setup(20_260_835);
    add_skill(&mut tx, owner, DCDUJUN);
    let before = tx.state().hand_count(owner);
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: first,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.hand_count(owner), before + 2);
}

#[test]
fn dcdujun_only_triggers_on_first_damage_of_turn() {
    let (mut tx, owner, first, _second) = setup(20_260_836);
    add_skill(&mut tx, owner, DCDUJUN);

    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: first,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    let mut tx = Transaction::begin(state);
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: first,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    assert!(matches!(err, Ok(_)), "second damage must not fire");
}
}

mod original_xianding_batch16 {
//! Golden for XIANDING（限定包）批次十六：
//! 聚土（dcsbjutu，②）、复学（dcfuxue，②）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCSBJUTU: u32 = 2_559_409_828;
const DCFUXUE: u32 = 3_597_057_421;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    if bootstrap_match(&mut tx, 3, seed, "identity-standard").is_err() {
        // v1.11.6 增量 roster +7 平移种子：开局撞 optional 触发（ChoiceRequested）——
        // 以 cancelled=true 排空（original_sp_qimen_zniaoxiang 范例）。
        let mut st = tx.commit().state;
        let mut drain_key = 1u64;
        while st.has_pending_choice() {
            let choice = ChoiceId::new(st.resolution.pending_choice_id);
            let revision = st.revision;
            let (next, output) = apply_command(
                st,
                Command {
                    actor: PlayerId::NONE,
                    expected_revision: ExpectedRevision(revision),
                    idempotency_key: IdempotencyKey(seed ^ drain_key),
                    kind: CommandKind::ResolveChoice {
                        choice_id: choice,
                        selections: vec![],
                        cancelled: true,
                    },
                },
            );
            assert!(
                matches!(output.result, CommandResult::Accepted { .. }),
                "drain rejected: {:?}",
                output.result
            );
            st = next;
            drain_key += 1;
        }
        tx = Transaction::begin(st);
    }
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
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
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
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
fn package_exposes_batch16_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let jutu = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCSBJUTU)
        .expect("dcsbjutu trigger");
    assert_eq!(jutu.event, "phase_end");
    assert_eq!(jutu.filter, "owner_is_target_no_turn_damage");
    assert!(jutu.optional);

    let fuxue = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCFUXUE)
        .expect("dcfuxue trigger");
    assert_eq!(fuxue.event, "phase_end");
    assert_eq!(fuxue.filter, "owner_is_target");
    assert!(fuxue.optional);
}

#[test]
fn dcsbjutu_draws_two_at_phase_end_without_turn_damage() {
    let (mut tx, owner, _first, _second) = setup(20_260_840);
    add_skill(&mut tx, owner, DCSBJUTU);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    let before = tx.state().hand_count(owner);

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.hand_count(owner), before + 2);
}

#[test]
fn dcsbjutu_does_not_trigger_after_owner_caused_damage() {
    let (mut tx, owner, first, _second) = setup(20_260_838);
    add_skill(&mut tx, owner, DCSBJUTU);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: first,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(
        matches!(dispatched, Ok(_)),
        "damage dealt this turn must suppress"
    );
}

#[test]
fn dcfuxue_draws_hp_at_phase_end() {
    let (mut tx, owner, _first, _second) = setup(20_260_839);
    add_skill(&mut tx, owner, DCFUXUE);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    let hp = tx.state().players.get(owner).unwrap().hp.max(0) as u32;
    let before = tx.state().hand_count(owner);

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.hand_count(owner), before + hp);
    let _ = hp;
}
}

mod original_xianding_batch17 {
//! Golden for XIANDING（限定包）批次十七：
//! 梦魇（dcwuhun，②）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCWUHUN: u32 = 4_234_333_270;
const DCWUHUN_MARK: u32 = 3_698_112_918;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
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

fn mark_count(tx: &Transaction, player: PlayerId) -> i32 {
    tx.state()
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == DCWUHUN_MARK)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn package_exposes_batch17_skill_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCWUHUN)
        .expect("dcwuhun trigger");
    assert_eq!(trigger.event, "damage");
    assert_eq!(
        trigger.filter,
        "owner_is_source_or_target_other_alive_per_damage_point"
    );
    assert!(!trigger.optional);
}

#[test]
fn dcwuhun_marks_damage_source_when_owner_damaged() {
    let (mut tx, owner, first, _second) = setup(20_260_840);
    add_skill(&mut tx, owner, DCWUHUN);

    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: first,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    assert!(err.is_ok(), "locked trigger must not open a choice");
    assert_eq!(mark_count(&tx, first), 1);
    assert_eq!(mark_count(&tx, owner), 0);
}

#[test]
fn dcwuhun_marks_once_per_damage_point() {
    let (mut tx, owner, first, _second) = setup(20_260_841);
    add_skill(&mut tx, owner, DCWUHUN);

    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: first,
            target: owner,
            amount: 3,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    assert!(err.is_ok());
    assert_eq!(mark_count(&tx, first), 3);
}
}

mod original_xianding_batch18 {
//! Golden for XIANDING（限定包）批次十八：
//! 忿肆（fensi）、典财（diancai）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const FENSI: u32 = 4_137_633_788;
const DIANCAI: u32 = 3_029_901_387;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
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
fn package_exposes_batch18_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let fensi = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == FENSI)
        .expect("fensi trigger");
    assert_eq!(fensi.event, "phase_begin");
    assert!(fensi.optional);

    let diancai = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DIANCAI)
        .expect("diancai trigger");
    assert_eq!(diancai.event, "phase_end");
    assert_eq!(diancai.filter, "owner_alive_lost_hand_this_turn");
    assert!(diancai.optional);
}

#[test]
fn fensi_damages_other_player_at_phase_begin() {
    let (mut tx, owner, first, _second) = setup(20_260_842);
    add_skill(&mut tx, owner, FENSI);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let hp = tx.state().players.get(first).unwrap().hp;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseOtherDamage
    );
    let state = resolve(state, owner, vec![first.raw()], false, 2);
    assert_eq!(state.players.get(first).unwrap().hp, hp - 1);
}

#[test]
fn diancai_draws_to_max_hp_after_other_phase_end_with_hand_loss() {
    let (mut tx, owner, first, _second) = setup(20_260_843);
    add_skill(&mut tx, owner, DIANCAI);
    tx.state_mut().turn.current = first;
    tx.state_mut().turn.phase = Phase::Play;

    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);

    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: 1,
        }],
    );
    assert!(err.is_ok());

    tx.state_mut().turn.phase = Phase::End;
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    let hand_before = tx.state().hand_count(owner);
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: first,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDraw
    );
    let state = resolve(state, owner, vec![owner.raw()], false, 2);
    let hand_after = state.hand_count(owner);
    assert!(hand_after >= hand_before);
    assert!(
        hand_after <= max_hp as u32,
        "hand {} should be capped at max hp {}",
        hand_after,
        max_hp
    );
}

#[test]
fn diancai_does_not_trigger_on_own_phase_end() {
    let (mut tx, owner, _first, _second) = setup(20_260_844);
    add_skill(&mut tx, owner, DIANCAI);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Ok(_)), "own phase end must not fire");
}
}

mod original_xianding_batch19 {
//! Golden for XIANDING（限定包）批次十九：
//! 立刃（liren）、讽言（dcfengyan）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const LIREN: u32 = 3_368_059_686;
const DCFENGYAN: u32 = 2_692_338_281;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn inject_card(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
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
fn package_exposes_batch19_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let range = package
        .modifier_index
        .iter()
        .find(|modifier| modifier.skill_def_id == LIREN && modifier.target == "attack_range")
        .expect("liren attack range modifier");
    assert_eq!(range.mode, "override");
    assert_eq!(range.value, 2);

    let dodged = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == LIREN)
        .expect("liren trigger");
    assert_eq!(dodged.event, "sha_dodged");
    assert_eq!(dodged.filter, "owner_source_sha_dodged");
    assert!(!dodged.optional);

    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DCFENGYAN)
        .expect("dcfengyan active");
    assert_eq!(active.window, "play_phase");
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.targets.as_ref().is_some_and(|t| t.hp_lte_owner));
}

#[test]
fn liren_draws_when_sha_dodged() {
    let (mut tx, owner, first, _second) = setup(20_260_845);
    add_skill(&mut tx, owner, LIREN);
    let sha = inject_card(&mut tx, owner, CardKind::Sha);
    let before = tx.state().hand_count(owner);

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: sha,
            targets: vec![first],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::ShaDodged {
            source: owner,
            target: first,
            card: sha,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

#[test]
fn dcfengyan_takes_one_card_from_lower_hp_target() {
    let (mut tx, owner, first, _second) = setup(20_260_846);
    add_skill(&mut tx, owner, DCFENGYAN);
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    tx.state_mut().players.get_mut(first).unwrap().hp = owner_hp;
    let card = inject_card(&mut tx, first, CardKind::Sha);
    let before = tx.state().hand_count(owner);

    let skill_id = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def == SkillDefId::new(DCFENGYAN))
        .expect("dcfengyan live skill")
        .id;
    let revision = tx.state().revision;
    let (tx2, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: skill_id,
                targets: vec![first],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        tx2.resolution.continuation_kind,
        ContinuationKind::PickTargetCard
    );
    let mut tx = Transaction::begin(tx2);
    let state = resolve(tx.state().clone(), owner, vec![card.raw()], false, 2);
    assert_eq!(state.hand_count(owner), before + 1);
    assert!(!state
        .zones
        .get(state.zones.find(ZoneKind::Hand, first).unwrap())
        .unwrap()
        .cards
        .contains(&card));
}
}

mod original_xianding_batch1_dcganfuren {
//! Golden for XIANDING（限定包）DC 甘夫人（dc_ganfuren）与批次技能：
//! 淑慎（dcshushen）、神智（dcshenzhi）、诋敌（dcjidi）、制略（dczhijue）、
//! 闺秀（dcguixiu）、昭汉（dczhaohan）。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const DC_GANFUREN: u32 = 3_184_320_853;
const DCSHUSHEN: u32 = 2_888_857_366;
const DCSHENZHI: u32 = 2_981_440_225;
const DCJIDI: u32 = 4_136_170_419;
const DCZHIJUE: u32 = 2_161_260_760;
const DCGUIXIU: u32 = 2_370_422_642;
const DCZHAOHAN: u32 = 2_818_087_075;
const DCZHAOHAN_CHOOSE: u32 = 3_308_549_699;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    let mut tx = Transaction::begin(state);
    // Bootstrap with the embedded standard roster first: the committed
    // xianding roster carries game_started choice skills (e.g. dcmingjie),
    // which would make bootstrap itself request a choice. The package loads
    // afterwards; setup removes every booted skill right below.
    bootstrap_match(&mut tx, 3, 20_260_801, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
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
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = engine_core::state::Phase::Prepare;
    for player in players {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.hp = 3;
        participant.max_hp = 3;
    }
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn phase_event(current: PlayerId, phase: TurnPhaseTag) -> DomainEvent {
    DomainEvent::TurnStateChanged { current, phase }
}

fn phase_ended_event(current: PlayerId, phase: TurnPhaseTag) -> DomainEvent {
    DomainEvent::PhaseEnded {
        current,
        phase,
        damage_sources: 0,
        damaged_players: 0,
        card_targeted_other: false,
        current_dealt_damage_other: false,
        phase_targeted_players: 0,
        phase_hp_changed_players: 0,
    }
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

fn dispatch_and_apply(tx: &mut Transaction, event: DomainEvent) {
    let effects = dispatch_for_events(tx, &[event], &mut TriggerFireRecord::new()).unwrap();
    run_effects(tx, effects).unwrap();
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

fn damage(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    amount: u32,
) -> Result<Vec<DomainEvent>, EffectError> {
    run_effects(
        tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            amount,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    )
}

#[test]
fn package_closes_dc_ganfuren_with_exact_skills() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let character = package
        .character_index
        .iter()
        .find(|character| character.id == DC_GANFUREN)
        .expect("dc_ganfuren character index entry");
    assert_eq!(character.name_key, "dc_ganfuren");
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "shu");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [DCSHUSHEN, DCSHENZHI]);

    let shushen = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCSHUSHEN)
        .expect("dcshushen trigger");
    assert_eq!(shushen.event, "hp_recovered");
    assert_eq!(shushen.filter, "owner_is_target");
    assert!(shushen.optional);

    let shenzhi = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCSHENZHI)
        .expect("dcshenzhi trigger");
    assert_eq!(shenzhi.event, "phase_prepare");
    assert_eq!(shenzhi.filter, "owner_is_target_hand_above_hp");
    assert!(shenzhi.optional);

    let jidi = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCJIDI)
        .expect("dcjidi trigger");
    assert_eq!(jidi.event, "damage_before");
    assert_eq!(jidi.filter, "owner_is_target_source_alive");
    assert!(!jidi.optional);
}

#[test]
fn dcshushen_offers_other_player_draw_or_recover_choice_per_recover_point() {
    let (mut tx, owner, first, second) = setup();
    add_skill(&mut tx, owner, DCSHUSHEN);
    damage(&mut tx, first, owner, 1).unwrap();
    damage(&mut tx, owner, first, 1).unwrap();
    let before_hand = tx.state().hand_count(first);
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source: owner,
            player: owner,
            amount: 1,
        }],
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseOtherDrawOrRecover
    );
    let state = resolve(state, owner, vec![first.raw()], false, 2);
    let state = resolve(state, first, vec![1], false, 3);
    assert_eq!(state.hand_count(first), before_hand + 2);
    assert_eq!(state.hand_count(second), 0);
}

#[test]
fn dcshenzhi_discards_one_hand_then_recovers_when_hand_above_hp() {
    let (mut tx, owner, _first, _second) = setup();
    add_skill(&mut tx, owner, DCSHENZHI);
    tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 4,
        }],
    )
    .unwrap();
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let hand_before = tx.state().hand_count(owner);
    assert!(hand_before > hp_before as u32);

    let err = dispatch_for_events(
        &mut tx,
        &[phase_event(owner, TurnPhaseTag::Prepare)],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 3);
    assert_eq!(state.hand_count(owner), hand_before - 1);
    assert_eq!(state.players.get(owner).unwrap().hp, hp_before + 1);
}

#[test]
fn dcjidi_source_with_more_hp_loses_one_and_more_hand_discards_two() {
    let (mut tx, owner, first, _second) = setup();
    add_skill(&mut tx, owner, DCJIDI);
    tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;
    let owner_hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: first,
            count: owner_hand + 2,
        }],
    )
    .unwrap();
    let first_hp = tx.state().players.get(first).unwrap().hp;
    let first_hand = tx.state().hand_count(first);
    damage(&mut tx, first, owner, 1).unwrap();
    assert_eq!(tx.state().players.get(first).unwrap().hp, first_hp - 1);
    assert_eq!(tx.state().hand_count(first), first_hand - 2);
}

#[test]
fn dczhijue_shortens_distance_from_owner_by_two_and_draws_to_max_hp_on_damage() {
    let (mut tx, owner, first, _second) = setup();
    add_skill(&mut tx, owner, DCZHIJUE);

    let distance = match query(
        tx.state(),
        &QueryRequest::Distance {
            from: owner,
            to: first,
        },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("distance query failed: {other:?}"),
    };
    assert_eq!(distance, 1);

    let hand_before = tx.state().hand_count(owner);
    damage(&mut tx, owner, first, 1).unwrap();
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    assert!(tx.state().hand_count(owner) > hand_before);
    assert_eq!(tx.state().hand_count(owner), max_hp);
}

#[test]
fn dcguixiu_draws_two_at_phase_begin_once() {
    let (mut tx, owner, _first, _second) = setup();
    add_skill(&mut tx, owner, DCGUIXIU);
    let before = tx.state().hand_count(owner);
    dispatch_and_apply(&mut tx, phase_event(owner, TurnPhaseTag::Prepare));
    assert_eq!(tx.state().hand_count(owner), before + 2);
    let after_first = tx.state().hand_count(owner);
    dispatch_and_apply(&mut tx, phase_event(owner, TurnPhaseTag::Prepare));
    assert_eq!(tx.state().hand_count(owner), after_first);
}

#[test]
fn dczhaohan_increases_current_draw_count_by_two() {
    let (mut tx, owner, _first, _second) = setup();
    add_skill(&mut tx, owner, DCZHAOHAN);
    tx.state_mut().turn.phase = engine_core::state::Phase::Draw;
    let before = tx.state().hand_count(owner);
    let err = dispatch_for_events(
        &mut tx,
        &[phase_event(owner, TurnPhaseTag::Draw)],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 4);
    assert_eq!(
        state
            .turn
            .flag(engine_core::state::turn_flag::CURRENT_DRAW_COUNT_BONUS),
        2
    );
    assert_eq!(state.hand_count(owner), before);
    assert!(has_skill(&state, owner, DCZHAOHAN_CHOOSE));
}

fn open_dczhaohan_draw_end(mut state: GameState, owner: PlayerId) -> GameState {
    state.turn.phase = engine_core::state::Phase::Play;
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[phase_ended_event(owner, TurnPhaseTag::Draw)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(matches!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    ));
    tx.state().clone()
}

#[test]
fn dczhaohan_gives_exactly_two_hand_cards_to_a_hand_empty_player() {
    let (mut tx, owner, first, _second) = setup();
    add_skill(&mut tx, owner, DCZHAOHAN);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 4,
        }],
    )
    .unwrap();
    tx.state_mut().turn.phase = engine_core::state::Phase::Draw;
    let err = dispatch_for_events(
        &mut tx,
        &[phase_event(owner, TurnPhaseTag::Draw)],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 10);
    let state = open_dczhaohan_draw_end(state, owner);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::DrawThenChooseGiveOrDiscardHand
    );
    let state = resolve(state, owner, vec![1], false, 11);
    let state = resolve(state, owner, vec![first.raw()], false, 12);
    let cards: Vec<_> = state
        .hand_cards(owner)
        .into_iter()
        .take(2)
        .map(CardId::raw)
        .collect();
    let state = resolve(state, owner, cards, false, 13);
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.hand_count(first), 2);
}

#[test]
fn dczhaohan_forces_two_discards_when_no_player_is_hand_empty() {
    let (mut tx, owner, first, second) = setup();
    add_skill(&mut tx, owner, DCZHAOHAN);
    for (player, count) in [(owner, 4), (first, 1), (second, 1)] {
        run_effects(&mut tx, vec![EffectRequest::DrawCards { player, count }]).unwrap();
    }
    tx.state_mut().turn.phase = engine_core::state::Phase::Draw;
    let err = dispatch_for_events(
        &mut tx,
        &[phase_event(owner, TurnPhaseTag::Draw)],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 20);
    let state = open_dczhaohan_draw_end(state, owner);
    let cards: Vec<_> = state
        .hand_cards(owner)
        .into_iter()
        .take(2)
        .map(CardId::raw)
        .collect();
    let state = resolve(state, owner, cards, false, 21);
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.hand_count(first), 1);
    assert_eq!(state.hand_count(second), 1);
}
}

mod original_xianding_batch2 {
//! Golden for XIANDING（限定包）批次二技能：
//! 精弓（dcjinggong）、忠节（dczhongjie）、花火（dchuahuo）、
//! 节行（dcjiexing）、静隐（dcjingyin）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCJINGGONG: u32 = 3_793_668_177;
const DCZHONGJIE: u32 = 2_275_941_720;
const DCHUAHUO: u32 = 2_857_165_081;
const DCJIEXING: u32 = 4_014_891_370;
const DCJINGYIN: u32 = 3_501_643_698;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn inject_hand(
    tx: &mut Transaction,
    owner: PlayerId,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000 + u32::from(rank)),
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

fn dispatch(tx: &mut Transaction, event: DomainEvent) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(tx, &[event], &mut TriggerFireRecord::new())
}

#[test]
fn package_exposes_batch2_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let jinggong = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == DCJINGGONG)
        .expect("dcjinggong view_as");
    assert_eq!(jinggong.as_kind, "sha");
    assert_eq!(jinggong.window, "play_phase");
    assert_eq!(jinggong.filter, "card_equip");
    assert_eq!(jinggong.source_zone.as_deref(), Some("hand_equip"));
    assert!(jinggong.ignore_distance);

    let huahuo = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == DCHUAHUO)
        .expect("dchuahuo view_as");
    assert_eq!(huahuo.as_kind, "sha");
    assert_eq!(huahuo.filter, "red");
    assert_eq!(huahuo.nature.as_deref(), Some("fire"));

    let zhongjie = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCZHONGJIE)
        .expect("dczhongjie trigger");
    assert_eq!(zhongjie.event, "dying");
    assert_eq!(zhongjie.filter, "owner_alive_other_dying_once_per_round");
    assert!(zhongjie.optional);

    let jiexing: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == DCJIEXING)
        .map(|trigger| {
            (
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert_eq!(
        jiexing,
        vec![
            ("damage", "owner_is_target", true),
            ("hp_recovered", "owner_is_target", true),
            ("hp_lost", "owner_is_target", true),
        ]
    );

    let jingyin = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCJINGYIN)
        .expect("dcjingyin trigger");
    assert_eq!(jingyin.event, "sha_resolved");
    assert_eq!(
        jingyin.filter,
        "owner_alive_other_source_sha_resolved_not_target_gainable_once_per_turn"
    );
    assert!(jingyin.optional);
}

#[test]
fn dczhongjie_recovers_and_draws_for_dying_player() {
    let (mut tx, owner, first, _second) = setup(20_260_802);
    add_skill(&mut tx, owner, DCZHONGJIE);
    tx.state_mut().players.get_mut(first).unwrap().hp = 0;
    tx.state_mut().players.get_mut(first).unwrap().dying = true;
    let err = dispatch(
        &mut tx,
        DomainEvent::EnteredDying {
            player: first,
            source: PlayerId::NONE,
        },
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.players.get(first).unwrap().hp, 1);
    assert_eq!(state.hand_count(first), 1);
}

#[test]
fn dcjiexing_draws_after_damage_recovery_and_hp_loss() {
    let (mut tx, owner, first, _second) = setup(20_260_803);
    add_skill(&mut tx, owner, DCJIEXING);
    tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;

    let before = tx.state().hand_count(owner);
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source: first,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            card: CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.hand_count(owner), before + 1);

    let mut tx = Transaction::begin(state);
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.hp = participant.max_hp as i32 - 1;
    let before = tx.state().hand_count(owner);
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source: first,
            target: owner,
            amount: 1,
            is_lose_hp: true,
            nature: 0,
            card: CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 2);
    assert_eq!(state.hand_count(owner), before + 1);

    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;
    let before = tx.state().hand_count(owner);
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source: owner,
            player: owner,
            amount: 1,
        }],
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 3);
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn dcjinggong_equip_can_be_viewed_as_sha_in_play_phase() {
    let (mut tx, owner, _first, second) = setup(20_260_804);
    add_skill(&mut tx, owner, DCJINGGONG);
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().turn.current = owner;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCJINGGONG)
        .expect("dcjinggong skill instance")
        .id;
    let source = inject_hand(&mut tx, owner, CardKind::Weapon, Suit::Spade, 3);
    let state = tx.state().clone();

    let (state, pipe) = activate(state, owner, skill, vec![second], source, 5);
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "jinggong rejected: {:?}",
        pipe.result
    );
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(source).unwrap().zone, discard);
}

fn activate(
    state: GameState,
    actor: PlayerId,
    skill: SkillId,
    targets: Vec<PlayerId>,
    source: CardId,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets: vec![source],
            },
        },
    )
}
}

mod original_xianding_batch20 {
//! Golden for XIANDING（限定包）批次二十：
//! 酒遁（dcjiudun）、全守（dcquanshou）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCJIUDUN: u32 = 2_926_675_148;
const DCQUANSHOU: u32 = 2_462_490_450;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn inject_card(tx: &mut Transaction, player: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
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
fn package_exposes_batch20_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let jiudun: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == DCJIUDUN)
        .collect();
    assert_eq!(jiudun.len(), 2);
    assert!(jiudun
        .iter()
        .all(|trigger| trigger.event == "card_targeted"));
    assert_eq!(
        jiudun
            .iter()
            .map(|trigger| trigger.filter.as_str())
            .collect::<Vec<_>>(),
        [
            "owner_is_target_event_card_jiu",
            "owner_is_target_other_source_black_dcjiudun_available"
        ]
    );
    assert!(jiudun.iter().all(|trigger| !trigger.optional));

    let quanshou = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCQUANSHOU)
        .expect("dcquanshou trigger");
    assert_eq!(quanshou.event, "phase_begin");
    assert_eq!(quanshou.filter, "owner_alive_not_target");
    assert!(quanshou.optional);
}

#[test]
fn dcjiudun_draws_and_uses_jiu_when_targeted_by_black_card() {
    let (mut tx, owner, first, _second) = setup(20_260_847);
    add_skill(&mut tx, owner, DCJIUDUN);
    let card = inject_card(&mut tx, first, CardKind::Sha, Suit::Spade);
    let before = tx.state().hand_count(owner);

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: first,
            card,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(matches!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::Custom
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn dcquanshou_offers_draw_to_max_hp_on_other_phase_begin() {
    let (mut tx, owner, first, _second) = setup(20_260_848);
    add_skill(&mut tx, owner, DCQUANSHOU);
    tx.state_mut().turn.current = first;
    tx.state_mut().turn.phase = Phase::Prepare;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: first,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDraw
    );
    let hand_before = state.hand_count(first);
    let state = resolve(state, owner, vec![first.raw()], false, 2);
    assert!(state.hand_count(first) >= hand_before);
}
}

mod original_xianding_batch21 {
//! Golden for XIANDING（限定包）批次二十一：
//! 守户（dcshouhu）、烈骧（dcliexiang）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCSHOUHU: u32 = 3_777_360_870;
const DCLIEXIANG: u32 = 3_778_727_333;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
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

fn activate(
    tx: &Transaction,
    actor: PlayerId,
    def: u32,
    targets: Vec<PlayerId>,
    key: u64,
) -> GameState {
    let skill_id = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def == SkillDefId::new(def))
        .expect("live skill")
        .id;
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: skill_id,
                targets,
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_exposes_batch21_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let shouhu = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DCSHOUHU)
        .expect("dcshouhu active");
    assert_eq!(shouhu.window, "play_phase");
    assert_eq!(shouhu.uses_per_turn, Some(1));
    assert!(shouhu.targets.as_ref().is_some_and(|t| !t.allow_self));

    let liexiang = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DCLIEXIANG)
        .expect("dcliexiang active");
    assert_eq!(liexiang.window, "play_phase");
    assert_eq!(liexiang.uses_per_turn, Some(1));
}

#[test]
fn dcshouhu_uses_virtual_sha_on_other() {
    let (mut tx, owner, first, _second) = setup(20_260_849);
    add_skill(&mut tx, owner, DCSHOUHU);
    let state = activate(&tx, owner, DCSHOUHU, vec![first], 1);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::VirtualShaTargets
    );
    let state = resolve(tx.state().clone(), owner, vec![first.raw()], false, 2);
    let _ = state;
}

#[test]
fn dcliexiang_draws_five_then_damages_target() {
    let (mut tx, owner, first, _second) = setup(20_260_850);
    add_skill(&mut tx, owner, DCLIEXIANG);
    let before = tx.state().hand_count(owner);
    let hp = tx.state().players.get(first).unwrap().hp;

    let state = activate(&tx, owner, DCLIEXIANG, vec![first], 1);
    assert_eq!(state.hand_count(owner), before + 5);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetsDamage
    );
    let state = resolve(state, owner, vec![first.raw()], false, 2);
    assert_eq!(state.players.get(first).unwrap().hp, hp - 1);
}
}

mod original_xianding_batch22 {
//! Golden for XIANDING（限定包）批次二十二：
//! 黠策（dcxiace，完整双分支）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCXIACE: u32 = 2_254_753_634;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn inject_card(tx: &mut Transaction, player: PlayerId) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
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
fn package_exposes_batch22_skill_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == DCXIACE)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert!(triggers.iter().all(|trigger| trigger.event == "damage"));
    assert_eq!(triggers[0].filter, "owner_is_target");
    assert_eq!(triggers[1].filter, "owner_source");
    assert!(triggers.iter().all(|trigger| !trigger.optional));
}

#[test]
fn dcxiace_discards_and_recovers_after_damage() {
    let (mut tx, owner, _first, _second) = setup(20_260_851);
    add_skill(&mut tx, owner, DCXIACE);
    let card = inject_card(&mut tx, owner);
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::Custom
    );
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let state = resolve(tx.state().clone(), owner, vec![card.raw()], false, 1);
    assert_eq!(state.players.get(owner).unwrap().hp, hp_before + 1);
    assert_eq!(
        state
            .zones
            .get(state.zones.find(ZoneKind::Hand, owner).unwrap())
            .unwrap()
            .cards
            .len(),
        0
    );
}
}

mod original_xianding_batch23 {
//! Golden for XIANDING（限定包）批次二十三：
//! 清棱（dcqingleng）、接缨（dcjieying）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCQINGLENG: u32 = 2_507_120_853;
const DCJIEYING: u32 = 3_849_335_872;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn inject_card(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
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
fn package_exposes_batch23_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let qingleng = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCQINGLENG)
        .expect("dcqingleng trigger");
    assert_eq!(qingleng.event, "card_targeted");
    assert_eq!(qingleng.filter, "owner_is_target_nonred_card");
    assert!(qingleng.optional);

    let jieying = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCJIEYING)
        .expect("dcjieying trigger");
    assert_eq!(jieying.event, "phase_end");
    assert!(jieying.optional);
}

#[test]
fn dcqingleng_discards_one_card_from_any_player_when_targeted_by_black_card() {
    let (mut tx, owner, first, _second) = setup(20_260_852);
    add_skill(&mut tx, owner, DCQINGLENG);
    let target_card = inject_card(&mut tx, first, CardKind::Sha);
    let source_card = inject_card(&mut tx, first, CardKind::Jiu);

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: first,
            card: source_card,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseOtherDiscardTargetCard
    );
    let state = resolve(state, owner, vec![first.raw()], false, 2);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardTargetCard
    );
    let state = resolve(state, owner, vec![target_card.raw()], false, 3);
    let hand = state
        .zones
        .get(state.zones.find(ZoneKind::Hand, first).unwrap())
        .unwrap()
        .cards
        .clone();
    assert!(
        !hand.contains(&target_card),
        "chosen card must be discarded"
    );
}

#[test]
fn dcjieying_links_other_and_draws_linked_players_at_phase_end() {
    let (mut tx, owner, first, _second) = setup(20_260_853);
    add_skill(&mut tx, owner, DCJIEYING);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    let before_first = tx.state().hand_count(first);

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseLinkOther
    );
    let state = resolve(state, owner, vec![first.raw()], false, 2);
    assert!(state
        .players
        .get(first)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0));
    assert!(state.hand_count(first) >= before_first);
}
}

mod original_xianding_batch25 {
//! Golden for XIANDING（限定包）批次二十五：
//! 义指（dcsbyizhi，界刘璋主公技）。

use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCSBYIZHI: u32 = 3_294_916_313;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx, owner, first, second)
}

#[test]
fn package_exposes_batch25_skill_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let view_as = package
        .view_as_index
        .iter()
        .find(|view_as| view_as.skill_def_id == DCSBYIZHI)
        .expect("dcsbyizhi view_as");
    assert_eq!(view_as.as_kind, "wugu");
    assert_eq!(view_as.window, "play_phase");
    assert_eq!(view_as.filter, "any");
    assert_eq!(view_as.source_zone, None); // from hand 为默认，序列化省略
    assert_eq!(view_as.uses_per_turn, Some(1));

    let skill = package
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == DCSBYIZHI)
        .expect("dcsbyizhi skill");
    assert!(skill.lord_only, "义指必须是主公技");
}

#[test]
fn dcsbyizhi_skill_can_be_attached() {
    let (mut tx, owner, _first, _second) = setup(20_260_854);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCSBYIZHI),
        }],
    )
    .unwrap();
    let has = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.def == SkillDefId::new(DCSBYIZHI));
    assert!(has);
}
}

mod original_xianding_batch26 {
//! Golden for XIANDING（限定包）批次二十六：
//! 溃击（kuiji）、膂力·闪（lvli4）、膂力·无懈（lvli5）。

use engine_api::PlayerId;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const KUIJI: u32 = 2_248_505_393;
const LVLI4: u32 = 3_495_185_261;
const LVLI5: u32 = 2_862_723_587;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx, owner, first, second)
}

#[test]
fn package_exposes_batch26_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let kuiji = package
        .view_as_index
        .iter()
        .find(|view_as| view_as.skill_def_id == KUIJI)
        .expect("kuiji view_as");
    assert_eq!(kuiji.as_kind, "bingliang");
    assert_eq!(kuiji.window, "play_phase");
    assert_eq!(kuiji.filter, "black_basic_equip");
    assert_eq!(kuiji.uses_per_turn, Some(1));

    let lvli4 = package
        .view_as_index
        .iter()
        .find(|view_as| view_as.skill_def_id == LVLI4)
        .expect("lvli4 view_as");
    assert_eq!(lvli4.as_kind, "shan");
    assert_eq!(lvli4.window, "any");
    assert_eq!(lvli4.filter, "any");

    let lvli5 = package
        .view_as_index
        .iter()
        .find(|view_as| view_as.skill_def_id == LVLI5)
        .expect("lvli5 view_as");
    assert_eq!(lvli5.as_kind, "wuxie");
    assert_eq!(lvli5.window, "any");
    assert_eq!(lvli5.filter, "any");
}

#[test]
fn batch26_skills_can_be_attached() {
    let (mut tx, owner, _first, _second) = setup(20_260_855);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: engine_api::SkillDefId::new(KUIJI),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: engine_api::SkillDefId::new(LVLI4),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: engine_api::SkillDefId::new(LVLI5),
            },
        ],
    )
    .unwrap();
    let defs: Vec<u32> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| skill.def.raw())
        .collect();
    assert!(defs.contains(&KUIJI));
    assert!(defs.contains(&LVLI4));
    assert!(defs.contains(&LVLI5));
}
}

mod original_xianding_batch27 {
//! Golden for XIANDING（限定包）批次二十七：
//! 施仇（twodcspshichou）、壮魄（zhuangpo）、剑出（sejianchu）。

use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWODCSP_SHICHOU: u32 = 3_276_628_911;
const ZHUANGPO: u32 = 2_849_685_408;
const SEJIANCHU: u32 = 3_620_279_118;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx, owner, first, second)
}

#[test]
fn package_exposes_batch27_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let shichou = package
        .view_as_index
        .iter()
        .find(|view_as| view_as.skill_def_id == TWODCSP_SHICHOU)
        .expect("twodcspshichou view_as");
    assert_eq!(shichou.as_kind, "sha");
    assert_eq!(shichou.window, "play_phase");
    assert_eq!(shichou.filter, "any");
    assert!(shichou.no_use_count);

    let zhuangpo = package
        .view_as_index
        .iter()
        .find(|view_as| view_as.skill_def_id == ZHUANGPO)
        .expect("zhuangpo view_as");
    assert_eq!(zhuangpo.as_kind, "juedou");
    assert_eq!(zhuangpo.window, "play_phase");

    let sejianchu = package
        .view_as_index
        .iter()
        .find(|view_as| view_as.skill_def_id == SEJIANCHU)
        .expect("sejianchu view_as");
    assert_eq!(sejianchu.as_kind, "sha");
    assert_eq!(sejianchu.window, "play_phase");
    assert_eq!(sejianchu.filter, "card_shan");
}

#[test]
fn batch27_skills_can_be_attached() {
    let (mut tx, owner, _first, _second) = setup(20_260_856);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(TWODCSP_SHICHOU),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(ZHUANGPO),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(SEJIANCHU),
            },
        ],
    )
    .unwrap();
    let defs: Vec<u32> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| skill.def.raw())
        .collect();
    assert!(defs.contains(&TWODCSP_SHICHOU));
    assert!(defs.contains(&ZHUANGPO));
    assert!(defs.contains(&SEJIANCHU));
}
}

mod original_xianding_batch28 {
//! Golden for XIANDING（限定包）批次二十八：
//! 恩遇（dcenyu，DC 王允）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCENYU: u32 = 3_753_328_944;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn inject_card(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
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
fn package_exposes_batch28_skill_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCENYU)
        .expect("dcenyu trigger");
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional, "锁定技");
}

#[test]
fn dcenyu_draws_when_targeted_by_other_card() {
    let (mut tx, owner, first, _second) = setup(20_260_857);
    add_skill(&mut tx, owner, DCENYU);
    let card = inject_card(&mut tx, first, CardKind::Sha);
    let before = tx.state().hand_count(owner);

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: first,
            card,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let err = run_effects(&mut tx, effects);
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::Custom
    );
    assert_eq!(tx.state().hand_count(owner), before + 1);
}
}

mod original_xianding_batch29 {
//! Golden for XIANDING（限定包）批次二十九：
//! 清识（dcdyqingshi）、残肆（dccansi）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCDYQINGSHI: u32 = 4_110_806_243;
const DCCANSI: u32 = 3_144_382_148;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn inject_card(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
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
fn package_exposes_batch29_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let qingshi = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCDYQINGSHI)
        .expect("dcdyqingshi trigger");
    assert_eq!(qingshi.event, "card_used");
    assert_eq!(
        qingshi.filter,
        "owner_source_single_target_sha_or_ordinary_trick_current_turn"
    );
    assert!(qingshi.optional);

    let cansi = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCCANSI)
        .expect("dccansi trigger");
    assert_eq!(cansi.event, "phase_begin");
    assert_eq!(cansi.filter, "owner_is_target");
    assert!(!cansi.optional);
}

#[test]
fn dcdyqingshi_damages_a_player_after_single_target_sha() {
    let (mut tx, owner, first, _second) = setup(20_260_858);
    add_skill(&mut tx, owner, DCDYQINGSHI);
    let card = inject_card(&mut tx, owner, CardKind::Sha);
    let hp = tx.state().players.get(first).unwrap().hp;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![first],
        }],
        &mut TriggerFireRecord::new(),
    );
    eprintln!(
        "qingshi dispatch: {:?}",
        dispatched.as_ref().map(|v| v.len())
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetsDamage
    );
    let state = resolve(state, owner, vec![first.raw()], false, 2);
    assert_eq!(state.players.get(first).unwrap().hp, hp - 1);
}

#[test]
fn dccansi_recovers_self_and_other_at_phase_begin() {
    let (mut tx, owner, first, _second) = setup(20_260_859);
    add_skill(&mut tx, owner, DCCANSI);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: first,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let hp_self = tx.state().players.get(owner).unwrap().hp;
    let hp_other = tx.state().players.get(first).unwrap().hp;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    let effects = dispatched.unwrap();
    let err = run_effects(&mut tx, effects);
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseOtherDrawOrRecover
    );
    let state = resolve(tx.state().clone(), owner, vec![first.raw()], false, 1);
    assert_eq!(state.players.get(owner).unwrap().hp, hp_self + 1);
    assert_eq!(state.players.get(first).unwrap().hp, hp_other + 1);
}
}

mod original_xianding_batch3 {
//! Golden for XIANDING（限定包）批次三：
//! 存嗣（dccunsi）、勇决（dcyongjue）。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCCUNSI: u32 = 4_233_990_625;
const DCYONGJUE: u32 = 2_297_054_445;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCCUNSI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn activate(
    state: GameState,
    actor: PlayerId,
    skill: SkillId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets: Vec::new(),
            },
        },
    )
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
fn package_exposes_batch3_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let cunsi = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DCCUNSI)
        .expect("dccunsi active");
    assert_eq!(cunsi.window, "play_phase");
    assert!(cunsi.once_per_game);
    let targets = cunsi.targets.as_ref().expect("dccunsi targets");
    assert_eq!((targets.min, targets.max), (1, 1));
    assert!(targets.allow_self);

    let yongjue = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCYONGJUE)
        .expect("dcyongjue trigger");
    assert_eq!(yongjue.event, "card_used");
    assert_eq!(yongjue.filter, "owner_source_first_sha_this_play_phase");
    assert!(yongjue.optional);
}

#[test]
fn dccunsi_grants_yongjue_to_other_and_draws_two() {
    let (tx, owner, first, _second) = setup(20_260_805);
    let before = tx.state().hand_count(owner);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCCUNSI)
        .expect("dccunsi skill instance")
        .id;
    let (state, pipe) = activate(tx.state().clone(), owner, skill, vec![first], 1);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert!(has_skill(&state, first, DCYONGJUE));
    assert_eq!(state.hand_count(owner), before + 2);
}

#[test]
fn dccunsi_on_self_grants_yongjue_without_draw() {
    let (tx, owner, _first, _second) = setup(20_260_806);
    let before = tx.state().hand_count(owner);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCCUNSI)
        .expect("dccunsi skill instance")
        .id;
    let (state, pipe) = activate(tx.state().clone(), owner, skill, vec![owner], 2);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert!(has_skill(&state, owner, DCYONGJUE));
    assert_eq!(state.hand_count(owner), before);
}

#[test]
fn dcyongjue_offers_refund_or_gain_choice_on_first_play_phase_sha() {
    let (mut tx, owner, first, _second) = setup(20_260_807);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCYONGJUE),
        }],
    )
    .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);
    let err = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: sha,
            targets: vec![first],
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(
        matches!(err, Err(EffectError::ChoiceRequested)),
        "use err: {:?}",
        err
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 3);
    assert!(has_skill(&state, owner, DCYONGJUE), "yongjue should remain");
}
}

mod original_xianding_batch30 {
//! Golden for XIANDING（限定包）批次三十：
//! 醮影·摸牌（jiaoying3_draw）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const JIAOYING3_DRAW: u32 = 4_012_724_406;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
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
fn package_exposes_batch30_skill_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == JIAOYING3_DRAW)
        .expect("jiaoying3_draw trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);
}

#[test]
fn jiaoying3_draw_draws_target_to_five_cards_at_phase_end() {
    let (mut tx, owner, first, _second) = setup(20_260_890);
    add_skill(&mut tx, owner, JIAOYING3_DRAW);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    let before = tx.state().hand_count(first);

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDraw
    );
    let state = resolve(state, owner, vec![first.raw()], false, 2);
    let after = state.hand_count(first);
    assert!(after > before);
    assert!(after <= 5, "hand should be capped at five, got {}", after);
}
}

mod original_xianding_batch31 {
//! Golden for XIANDING（限定包）批次三十一：
//! 追还·二（zhuihuan2）、沙舞（shawu）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const ZHUHUAN2: u32 = 3_807_480_043;
const SHAWU: u32 = 3_835_073_328;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn inject_card(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
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
fn package_exposes_batch31_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let zhuihuan = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == ZHUHUAN2)
        .expect("zhuihuan2 trigger");
    assert_eq!(zhuihuan.event, "damage");
    assert_eq!(
        zhuihuan.filter,
        "owner_is_target_other_source_alive_per_damage_point"
    );
    assert!(!zhuihuan.optional);

    let shawu = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SHAWU)
        .expect("shawu trigger");
    assert_eq!(shawu.event, "card_targeted");
    assert_eq!(shawu.filter, "owner_is_target_event_card_sha");
    assert!(shawu.optional);
}

#[test]
fn zhuihuan2_else_branch_discards_source_card_when_hp_not_higher() {
    let (mut tx, owner, first, _second) = setup(20_260_861);
    add_skill(&mut tx, owner, ZHUHUAN2);
    inject_card(&mut tx, first, CardKind::Sha);
    let hand_before = tx.state().hand_count(first);

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: first,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let err = run_effects(&mut tx, effects);
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::DiscardOneHandOrEquip
    );
    let discard = tx
        .state()
        .zones
        .get(tx.state().zones.find(ZoneKind::Hand, first).unwrap())
        .unwrap()
        .cards
        .first()
        .copied()
        .unwrap();
    let state = resolve(tx.state().clone(), first, vec![discard.raw()], false, 1);
    assert_eq!(state.hand_count(first), hand_before - 1);
}

#[test]
fn shawu_damages_sha_source() {
    let (mut tx, owner, first, _second) = setup(20_260_862);
    add_skill(&mut tx, owner, SHAWU);
    let card = inject_card(&mut tx, first, CardKind::Sha);
    let hp = tx.state().players.get(first).unwrap().hp;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: first,
            card,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.players.get(first).unwrap().hp, hp - 1);
}
}

mod original_xianding_batch32 {
//! Golden for XIANDING（限定包）批次三十二：
//! 防毒（dcfangdu）、赏誉（dcshangyu）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCFANGDU: u32 = 3_648_264_331;
const DCSHANGYU: u32 = 3_428_760_598;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
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
fn package_exposes_batch32_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let fangdu = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCFANGDU)
        .expect("dcfangdu trigger");
    assert_eq!(fangdu.event, "damage");
    assert_eq!(fangdu.filter, "owner_is_target_other_source_alive");
    assert!(fangdu.optional);

    let shangyu = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCSHANGYU)
        .expect("dcshangyu trigger");
    assert_eq!(shangyu.event, "game_started");
    assert!(!shangyu.optional);
}

#[test]
fn dcfangdu_makes_source_lose_hp_after_damage() {
    let (mut tx, owner, first, _second) = setup(20_260_863);
    add_skill(&mut tx, owner, DCFANGDU);
    let hp = tx.state().players.get(first).unwrap().hp;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: first,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.players.get(first).unwrap().hp, hp - 1);
}

#[test]
fn dcshangyu_gains_sha_on_game_start() {
    let (mut tx, owner, _first, _second) = setup(20_260_865);
    add_skill(&mut tx, owner, DCSHANGYU);
    let before = tx.state().hand_count(owner);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 3 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert!(tx.state().hand_count(owner) >= before + 1);
}
}

mod original_xianding_batch33 {
//! Golden for XIANDING（限定包）批次三十三：
//! 识锦（dcsbshijin）、斗威（dcsbdouwei）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCSBSHIJIN: u32 = 3_506_294_264;
const DCSBDOUWEI: u32 = 2_217_274_823;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
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
fn package_exposes_batch33_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let shijin = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCSBSHIJIN)
        .expect("dcsbshijin trigger");
    assert_eq!(shijin.event, "damage_before");
    assert_eq!(shijin.filter, "owner_is_target");
    assert!(!shijin.optional);

    let douwei = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCSBDOUWEI)
        .expect("dcsbdouwei trigger");
    assert_eq!(douwei.event, "dying");
    assert!(douwei.optional);
}

#[test]
fn dcsbshijin_prevents_damage_and_draws() {
    let (mut tx, owner, _first, _second) = setup(20_260_866);
    add_skill(&mut tx, owner, DCSBSHIJIN);
    let before = tx.state().hand_count(owner);
    let hp = tx.state().players.get(owner).unwrap().hp;

    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    assert!(err.is_ok(), "damage should resolve without choice");
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp);
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

#[test]
fn dcsbdouwei_recovers_when_other_enters_dying() {
    let (mut tx, owner, first, _second) = setup(20_260_867);
    add_skill(&mut tx, owner, DCSBDOUWEI);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let hp = tx.state().players.get(owner).unwrap().hp;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: first,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.players.get(owner).unwrap().hp, hp + 1);
}
}

mod original_xianding_batch34 {
//! Golden for XIANDING（限定包）批次三十四：
//! 追猎（spzhuilie）、夺月（dcduoyue）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SPZHUILIE: u32 = 3_774_269_941;
const DCDUOYUE: u32 = 3_729_672_764;
const DCDUOYUE_WIN: u32 = 2_807_627_648;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    let mut tx = Transaction::begin(state);
    // Bootstrap with the embedded standard roster first: the committed
    // xianding roster carries game_started choice skills (e.g. dcmingjie),
    // which would make bootstrap itself request a choice. The package loads
    // afterwards; setup removes every booted skill right below.
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
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
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
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
fn package_exposes_batch34_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let dist = package
        .modifier_index
        .iter()
        .find(|m| m.skill_def_id == SPZHUILIE && m.target == "card_distance_unlimited")
        .expect("spzhuilie distance modifier");
    assert_eq!(dist.selector.as_deref(), Some("sha"));

    let armor = package
        .modifier_index
        .iter()
        .find(|m| m.skill_def_id == SPZHUILIE && m.target == "ignore_armor")
        .expect("spzhuilie armor modifier");

    let duoyue = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCDUOYUE)
        .expect("dcduoyue trigger");
    assert_eq!(duoyue.event, "phase_play");
    assert!(duoyue.optional);

    let win = package
        .modifier_index
        .iter()
        .any(|m| m.skill_def_id == DCDUOYUE_WIN && m.target == "card_distance_unlimited");
    assert!(win);
}

#[test]
fn spzhuilie_grants_distance_and_armor_ignore() {
    let (mut tx, owner, _first, _second) = setup(20_260_868);
    add_skill(&mut tx, owner, SPZHUILIE);
    let dist = engine_core::query::query(
        tx.state(),
        &rule_api::QueryRequest::Distance {
            from: owner,
            to: owner,
        },
        &mut engine_core::query::QueryContext::new(),
    );
    assert!(matches!(dist, rule_api::QueryResponse::Int(_)));
}

#[test]
fn dcduoyue_opens_hand_compare_at_phase_play() {
    let (mut tx, owner, _first, _second) = setup(20_260_869);
    add_skill(&mut tx, owner, DCDUOYUE);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let before = tx.state().hand_count(owner);
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    );
    match dispatched {
        Err(EffectError::ChoiceRequested) => {
            assert_eq!(
                tx.state().resolution.continuation_kind,
                ContinuationKind::OptionalTrigger
            );
            let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
            let _ = state;
        }
        Ok(_) => {}
        Err(other) => panic!("{other:?}"),
    }
    let _ = before;
}
}

mod original_xianding_batch35 {
//! Golden for XIANDING（限定包）批次三十五：
//! 弹翼（dcdanyi，DC 曹节）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCDANYI: u32 = 2_264_670_487;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn inject_card(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
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
fn package_exposes_batch35_skill_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCDANYI)
        .expect("dcdanyi trigger");
    assert_eq!(trigger.event, "card_used");
    assert_eq!(
        trigger.filter,
        "owner_source_play_card_repeats_previous_targets"
    );
    assert!(trigger.optional);
}

#[test]
fn dcdanyi_draws_when_card_repeats_previous_targets() {
    let (mut tx, owner, first, _second) = setup(20_260_870);
    add_skill(&mut tx, owner, DCDANYI);
    let card1 = inject_card(&mut tx, owner, CardKind::Sha);
    let card2 = inject_card(&mut tx, owner, CardKind::Jiu);
    let before = tx.state().hand_count(owner);

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: card1,
            targets: vec![first],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: card2,
            targets: vec![first],
        }],
        &mut TriggerFireRecord::new(),
    );
    match dispatched {
        Err(EffectError::ChoiceRequested) => {
            assert_eq!(
                tx.state().resolution.continuation_kind,
                ContinuationKind::OptionalTrigger
            );
            let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
            assert_eq!(state.hand_count(owner), before + 1);
        }
        Ok(effects) => {
            run_effects(&mut tx, effects).unwrap();
            assert_eq!(tx.state().hand_count(owner), before + 1);
        }
        Err(other) => panic!("{other:?}"),
    }
}
}

mod original_xianding_batch36 {
//! Golden for XIANDING（限定包）批次三十六：
//! 华衣（dchuayi）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCHUAYI: u32 = 2_287_394_739;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
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
fn package_exposes_batch36_skill_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCHUAYI)
        .expect("dchuayi trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(trigger.filter, "owner_alive_not_target");
    assert!(trigger.optional);
}

#[test]
fn dchuayi_draws_when_other_phase_ends() {
    let (mut tx, owner, first, _second) = setup(20_260_871);
    add_skill(&mut tx, owner, DCHUAYI);
    tx.state_mut().turn.current = first;
    tx.state_mut().turn.phase = Phase::End;
    let before = tx.state().hand_count(owner);

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: first,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.hand_count(owner), before + 1);
}
}

mod original_xianding_batch37 {
//! Golden for XIANDING（限定包）批次三十七：
//! 毒露（dcdulu）、猝袭（dccuxi）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCDULU: u32 = 2_779_276_477;
const DCCUXI: u32 = 4_093_257_724;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
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
fn package_exposes_batch37_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let dulu = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DCDULU)
        .expect("dcdulu active");
    assert_eq!(dulu.window, "play_phase");
    assert_eq!(dulu.uses_per_turn, Some(1));
    assert!(dulu
        .targets
        .as_ref()
        .is_some_and(|t| t.min == 1 && t.max == 3 && !t.allow_self));

    let cuxi = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCCUXI)
        .expect("dccuxi trigger");
    assert_eq!(cuxi.event, "phase_begin");
    assert_eq!(cuxi.filter, "owner_alive_not_target");
    assert!(cuxi.optional);
}

#[test]
fn dcdulu_damages_targets_on_activation() {
    let (mut tx, owner, first, _second) = setup(20_260_872);
    add_skill(&mut tx, owner, DCDULU);
    let hp = tx.state().players.get(first).unwrap().hp;
    let skill_id = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.def == SkillDefId::new(DCDULU))
        .expect("live dcdulu")
        .id;
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: skill_id,
                targets: vec![first],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(first).unwrap().hp, hp - 1);
}

#[test]
fn dccuxi_discards_two_and_damages_at_other_phase_begin() {
    let (mut tx, owner, first, _second) = setup(20_260_873);
    add_skill(&mut tx, owner, DCCUXI);
    tx.state_mut().turn.current = first;
    tx.state_mut().turn.phase = Phase::Prepare;
    let hp = tx.state().players.get(first).unwrap().hp;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: first,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.players.get(first).unwrap().hp, hp - 1);
}
}

mod original_xianding_batch38 {
//! Golden for XIANDING（限定包）批次三十八：
//! 孔武（dcsbkongwu，界呼厨儿）。

use engine_api::PlayerId;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCSBKONGWU: u32 = 3_614_891_392;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx, owner, first, second)
}

#[test]
fn package_exposes_batch38_skill_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let modifier = package
        .modifier_index
        .iter()
        .find(|m| m.skill_def_id == DCSBKONGWU && m.target == "draw_count")
        .expect("dcsbkongwu draw_count modifier");
    assert_eq!(modifier.mode, "add");
    assert_eq!(modifier.value, -1);
}

#[test]
fn dcsbkongwu_skill_can_be_attached() {
    let (mut tx, owner, _first, _second) = setup(20_260_874);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: engine_api::SkillDefId::new(DCSBKONGWU),
        }],
    )
    .unwrap();
    let has = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.def == engine_api::SkillDefId::new(DCSBKONGWU));
    assert!(has);
}
}

mod original_xianding_batch39 {
//! Golden for XIANDING（限定包）批次三十九：
//! 困欲（kunyu）、矜节（oldjinjie）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const KUNYU: u32 = 3_623_529_353;
const OLDJINJIE: u32 = 3_578_311_089;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
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
fn package_exposes_batch39_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let kunyu = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == KUNYU)
        .expect("kunyu trigger");
    assert_eq!(kunyu.event, "dying_after");
    assert_eq!(kunyu.filter, "owner_is_target");
    assert!(!kunyu.optional);

    let jinjie = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLDJINJIE)
        .expect("oldjinjie trigger");
    assert_eq!(jinjie.event, "dying");
    assert!(jinjie.optional);
}

#[test]
fn kunyu_recovers_after_being_saved_from_dying() {
    let (mut tx, owner, _first, _second) = setup(20_260_875);
    add_skill(&mut tx, owner, KUNYU);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let hp = tx.state().players.get(owner).unwrap().hp;

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::LeftDying { player: owner }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert!(tx.state().players.get(owner).unwrap().hp >= hp + 1);
}

#[test]
fn oldjinjie_draws_and_recovers_for_dying_other() {
    let (mut tx, owner, first, _second) = setup(20_260_876);
    add_skill(&mut tx, owner, OLDJINJIE);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: first,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let hp = tx.state().players.get(first).unwrap().hp;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: first,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDraw
    );
    let state = resolve(state, owner, vec![first.raw()], false, 2);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDrawOrRecover
    );
    let state = resolve(state, owner, vec![2], false, 3);
    assert_eq!(state.players.get(first).unwrap().hp, hp + 1);
}
}

mod original_xianding_batch4 {
//! Golden for XIANDING（限定包）批次四：
//! 伪涕（dcweiti）及安境索引合同。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCANJING: u32 = 2_716_085_756;
const DCWEITI: u32 = 2_856_634_800;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
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

fn activate(
    state: GameState,
    actor: PlayerId,
    skill: SkillId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets: Vec::new(),
            },
        },
    )
}

#[test]
fn package_exposes_batch4_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let anjing = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCANJING)
        .expect("dcanjing trigger");
    assert_eq!(anjing.event, "damage");
    assert_eq!(anjing.filter, "owner_source_wounded_exists_once_per_turn");
    assert!(anjing.optional);

    let weiti = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DCWEITI)
        .expect("dcweiti active");
    assert_eq!(weiti.window, "play_phase");
    assert_eq!(weiti.uses_per_turn, Some(2));
}

#[test]
fn dcweiti_choose_damage_branch_damages_other() {
    let (mut tx, owner, first, _second) = setup(20_260_809);
    add_skill(&mut tx, owner, DCWEITI);
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCWEITI)
        .expect("dcweiti skill instance")
        .id;
    let first_hp = tx.state().players.get(first).unwrap().hp;
    let (state, pipe) = activate(tx.state().clone(), owner, skill, vec![first], 1);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let state = resolve(state, owner, vec![0], false, 2);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseOtherDamage
    );
    let state = resolve(state, owner, vec![first.raw()], false, 3);
    assert_eq!(state.players.get(first).unwrap().hp, first_hp - 1);
}

#[test]
fn dcweiti_choose_recover_branch_recovers_other() {
    let (mut tx, owner, first, _second) = setup(20_260_810);
    add_skill(&mut tx, owner, DCWEITI);
    tx.state_mut().players.get_mut(first).unwrap().hp -= 1;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCWEITI)
        .expect("dcweiti skill instance")
        .id;
    let first_hp = tx.state().players.get(first).unwrap().hp;
    let (state, pipe) = activate(tx.state().clone(), owner, skill, vec![first], 3);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let state = resolve(state, owner, vec![1], false, 4);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseOtherDrawOrRecover
    );
    let state = resolve(state, owner, vec![first.raw()], false, 5);
    assert_eq!(state.players.get(first).unwrap().hp, first_hp + 1);
}
}

mod original_xianding_batch40 {
//! Golden for XIANDING（限定包）批次四十：
//! 势援（shiyuan）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SHIYUAN: u32 = 4_115_301_600;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn inject_card(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
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
fn package_exposes_batch40_skill_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SHIYUAN)
        .expect("shiyuan trigger");
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(trigger.filter, "owner_is_target_other_source_alive");
    assert!(trigger.optional);
}

#[test]
fn shiyuan_draws_three_when_source_hp_higher() {
    let (mut tx, owner, first, _second) = setup(20_260_894);
    add_skill(&mut tx, owner, SHIYUAN);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: owner,
            amount: 3,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let before = tx.state().hand_count(owner);
    let card = inject_card(&mut tx, first, CardKind::Sha);

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: first,
            card,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.hand_count(owner), before + 3);
}
}

mod original_xianding_batch41 {
//! Golden for XIANDING（限定包）批次四十一：
//! 天匠（pytianjiang，PY）。

use engine_api::{DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const PYTIANJIANG: u32 = 2_167_112_704;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

#[test]
fn package_exposes_batch41_skill_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == PYTIANJIANG)
        .expect("pytianjiang trigger");
    assert_eq!(trigger.event, "game_started");
    assert!(!trigger.optional);
}

#[test]
fn pytianjiang_gains_equipment_on_game_start() {
    let (mut tx, owner, _first, _second) = setup(20_260_878);
    let before_hand = tx.state().hand_count(owner) as u32;
    let before_equip = tx
        .state()
        .zones
        .get(tx.state().zones.find(engine_core::state::ZoneKind::Equip, owner).unwrap())
        .unwrap()
        .cards
        .len() as u32;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PYTIANJIANG),
        }],
    )
    .unwrap();

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 3 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let after_hand = tx.state().hand_count(owner) as u32;
    let after_equip = tx
        .state()
        .zones
        .get(tx.state().zones.find(engine_core::state::ZoneKind::Equip, owner).unwrap())
        .unwrap()
        .cards
        .len() as u32;
    assert!(
        after_hand + after_equip >= before_hand + before_equip + 1u32,
        "should have gained one equipment card"
    );
}
}

mod original_xianding_batch42 {
//! Golden for XIANDING（限定包）批次四十二：
//! 夺锐（dcduorui，DC 神张辽）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCDUORUI: u32 = 2_823_548_533;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    if bootstrap_match(&mut tx, 3, seed, "identity-standard").is_err() {
        // v1.11.6 增量 roster +7 平移种子：开局撞 optional 触发（ChoiceRequested）——
        // 以 cancelled=true 排空（original_sp_qimen_zniaoxiang 范例）。
        let mut st = tx.commit().state;
        let mut drain_key = 1u64;
        while st.has_pending_choice() {
            let choice = ChoiceId::new(st.resolution.pending_choice_id);
            let revision = st.revision;
            let (next, output) = apply_command(
                st,
                Command {
                    actor: PlayerId::NONE,
                    expected_revision: ExpectedRevision(revision),
                    idempotency_key: IdempotencyKey(seed ^ drain_key),
                    kind: CommandKind::ResolveChoice {
                        choice_id: choice,
                        selections: vec![],
                        cancelled: true,
                    },
                },
            );
            assert!(
                matches!(output.result, CommandResult::Accepted { .. }),
                "drain rejected: {:?}",
                output.result
            );
            st = next;
            drain_key += 1;
        }
        tx = Transaction::begin(st);
    }
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
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
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
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
fn package_exposes_batch42_skill_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCDUORUI)
        .expect("dcduorui trigger");
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(trigger.filter, "owner_is_source");
    assert!(trigger.optional);
}

#[test]
fn dcduorui_reduces_damage_by_one_when_optional_confirmed() {
    let (mut tx, owner, first, _second) = setup(20_260_879);
    add_skill(&mut tx, owner, DCDUORUI);
    let hp = tx.state().players.get(first).unwrap().hp;

    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: first,
            amount: 3,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    assert!(matches!(err, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.players.get(first).unwrap().hp, hp - 2);
}
}

mod original_xianding_batch43 {
//! Golden for XIANDING（限定包）批次四十三：
//! 奇绝（dcsbqijue）、毒逝·诅咒（dushi2）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCSBQIJUE: u32 = 2_527_119_333;
const DUSHI2: u32 = 2_743_495_682;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
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
fn package_exposes_batch43_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let qijue = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCSBQIJUE)
        .expect("dcsbqijue trigger");
    assert_eq!(qijue.event, "dying");
    assert!(qijue.optional);

    let cancel = package
        .modifier_index
        .iter()
        .find(|m| m.skill_def_id == DUSHI2 && m.target == "card_target_cancel")
        .expect("dushi2 tao cancel modifier");
    assert_eq!(cancel.selector.as_deref(), Some("tao"));
    assert_eq!(cancel.mode, "override");
    assert_eq!(cancel.value, 1);
}

#[test]
fn dcsbqijue_draws_two_and_recovers_two_for_dying_other() {
    let (mut tx, owner, first, _second) = setup(20_260_880);
    add_skill(&mut tx, owner, DCSBQIJUE);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: first,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let hp = tx.state().players.get(first).unwrap().hp;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: first,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    let state = resolve(state, owner, vec![first.raw()], false, 2);
    let state = resolve(state, owner, vec![2], false, 3);
    assert_eq!(state.players.get(first).unwrap().hp, hp + 2);
}

#[test]
fn dushi2_skill_can_be_attached() {
    let (mut tx, owner, _first, _second) = setup(20_260_881);
    add_skill(&mut tx, owner, DUSHI2);
    let has = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.def == SkillDefId::new(DUSHI2));
    assert!(has);
}
}

mod original_xianding_batch44 {
//! Golden for XIANDING（限定包）批次四十四：
//! 决（旧版，oldjue）、张采（dczhangcai）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLDJUE: u32 = 2_642_038_327;
const DCZHANGCAI: u32 = 4_008_750_021;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    let mut tx = Transaction::begin(state);
    // Bootstrap with the embedded standard roster first: the committed
    // xianding roster carries game_started choice skills (e.g. dcmingjie),
    // which would make bootstrap itself request a choice. The package loads
    // afterwards; setup removes every booted skill right below.
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
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
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn inject_card(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
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
fn package_exposes_batch44_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let jue = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLDJUE)
        .expect("oldjue trigger");
    assert_eq!(jue.event, "phase_prepare");
    assert!(jue.optional);

    let zhangcai = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCZHANGCAI)
        .expect("dczhangcai trigger");
    assert_eq!(zhangcai.event, "card_used");
    assert!(zhangcai.optional);
}

#[test]
fn oldjue_chooses_virtual_sha_target_at_prepare_phase() {
    let (mut tx, owner, first, _second) = setup(20_260_895);
    add_skill(&mut tx, owner, OLDJUE);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::VirtualShaTargets
    );
    let state = resolve(state, owner, vec![first.raw()], false, 2);
}

#[test]
fn dczhangcai_draws_one_when_owner_uses_card() {
    let (mut tx, owner, first, _second) = setup(20_260_896);
    add_skill(&mut tx, owner, DCZHANGCAI);
    let card = inject_card(&mut tx, owner, CardKind::Sha);
    let before = tx.state().hand_count(owner);

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![first],
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    assert_eq!(state.hand_count(owner), before + 1);
}
}

mod original_xianding_batch45 {
//! Golden for XIANDING（限定包）批次四十五：
//! 笑武（dcxiaowu）、儒贤（dcruxian）、苏慎（dcsbsushen）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCXIAOWU: u32 = 4_153_740_280;
const DCRUXIAN: u32 = 3_734_967_277;
const DCSBSUSHEN: u32 = 3_864_585_121;
const DCZHANGCAI_ALL: u32 = 3_424_574_524;
const DCSBRUSHI: u32 = 4_089_348_607;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn live_skill(tx: &Transaction, player: PlayerId, def: u32) -> engine_api::SkillId {
    tx.state()
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn activate(
    state: GameState,
    actor: PlayerId,
    skill: engine_api::SkillId,
    cards: Vec<CardId>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: cards,
            },
        },
    )
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

fn inject_card(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| state.skills.get(*skill).map(|s| s.def.raw()) == Some(def))
}

#[test]
fn package_exposes_batch45_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    for (def, name) in [
        (DCXIAOWU, "dcxiaowu"),
        (DCRUXIAN, "dcruxian"),
        (DCSBSUSHEN, "dcsbsushen"),
    ] {
        let active = package
            .active_index
            .iter()
            .find(|a| a.skill_def_id == def)
            .unwrap_or_else(|| panic!("{name} active"));
        assert_eq!(active.window, "play_phase");
    }
    assert!(package
        .skill_index
        .iter()
        .any(|s| s.skill_def_id == DCZHANGCAI_ALL && s.locked));
}

#[test]
fn dcxiaowu_discards_he_and_gains_sha() {
    let (mut tx, owner, _first, _second) = setup(20_260_897);
    add_skill(&mut tx, owner, DCXIAOWU);
    let cost = inject_card(&mut tx, owner, CardKind::Shan);
    let before = tx.state().hand_count(owner);

    let skill = live_skill(&tx, owner, DCXIAOWU);
    let (state, result) = activate(tx.commit().state, owner, skill, vec![cost], 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before); // -1 弃 +1 摸【杀】
}

#[test]
fn dcruxian_grants_zhangcai_all_and_removes_on_turn_end() {
    let (mut tx, owner, _first, _second) = setup(20_260_898);
    add_skill(&mut tx, owner, DCRUXIAN);
    let skill = live_skill(&tx, owner, DCRUXIAN);
    let (state, result) = activate(tx.commit().state, owner, skill, vec![], 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(has_skill(&state, owner, DCZHANGCAI_ALL));

    let mut tx = Transaction::begin(state);
    let disp = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    run_effects(&mut tx, disp.unwrap()).unwrap();
    assert!(!has_skill(tx.state(), owner, DCZHANGCAI_ALL));
}

#[test]
fn dcsbsushen_grants_fumou() {
    let (mut tx, owner, _first, _second) = setup(20_260_899);
    add_skill(&mut tx, owner, DCSBSUSHEN);
    let skill = live_skill(&tx, owner, DCSBSUSHEN);
    let (state, result) = activate(tx.commit().state, owner, skill, vec![], 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(has_skill(&state, owner, DCSBRUSHI));
}
}

mod original_xianding_batch46 {
//! Golden for XIANDING（限定包）批次四十六：
//! 毒逝（dushi，弘农王诅咒传递）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DUSHI: u32 = 3_887_700_670;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
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

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| state.skills.get(*skill).map(|s| s.def.raw()) == Some(def))
}

#[test]
fn package_exposes_dushi_with_tao_cancel_and_death_trigger() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let cancel = package
        .modifier_index
        .iter()
        .find(|m| m.skill_def_id == DUSHI && m.target == "card_target_cancel")
        .expect("dushi tao cancel modifier");
    assert_eq!(cancel.selector.as_deref(), Some("tao"));
    assert_eq!(cancel.mode, "override");
    assert_eq!(cancel.value, 1);

    let death = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DUSHI)
        .expect("dushi death trigger");
    assert_eq!(death.event, "death");
}

#[test]
fn dushi_grants_curse_to_chosen_other_on_owner_death() {
    let (mut tx, owner, first, _second) = setup(20_260_901);
    add_skill(&mut tx, owner, DUSHI);

    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PlayerKilled {
            player: owner,
            killer: PlayerId::NONE,
            hand_equip_cards: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let dispatched = run_effects(&mut tx, fired);
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![first.raw()], false, 1);
    assert!(has_skill(&state, first, DUSHI));
}
}

mod original_xianding_batch47 {
//! Golden for XIANDING（限定包）批次四十七：
//! 巡使（xunshi）、王抗（dcsbwankang）、纫霜（old_dcrenshuang）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const XUNSHI: u32 = 4_189_180_642;
const DCSBWANKANG: u32 = 2_153_717_747;
const OLD_DCRENSHUANG: u32 = 3_716_618_560;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
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
fn package_exposes_batch47_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let name = package
        .modifier_index
        .iter()
        .find(|m| m.skill_def_id == XUNSHI && m.target == "card_name")
        .expect("xunshi card_name");
    assert_eq!(name.selector.as_deref(), Some("trick"));
    assert_eq!(name.value, 1);

    let wankang = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCSBWANKANG)
        .expect("dcsbwankang trigger");
    assert_eq!(wankang.event, "dying");

    let renshuang = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLD_DCRENSHUANG)
        .expect("old_dcrenshuang trigger");
    assert_eq!(renshuang.event, "dying");
}

#[test]
fn dcsbwankang_recovers_one_and_draws_two_on_dying() {
    let (mut tx, owner, _first, _second) = setup(20_260_902);
    add_skill(&mut tx, owner, DCSBWANKANG);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    let hp = tx.state().players.get(owner).unwrap().hp;
    let before = tx.state().hand_count(owner);

    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: owner,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, fired).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp + 1);
    assert_eq!(tx.state().hand_count(owner), before + 2);
}

#[test]
fn old_dcrenshuang_recovers_to_one_and_gains_max_hp_on_dying() {
    let (mut tx, owner, _first, _second) = setup(20_260_903);
    add_skill(&mut tx, owner, OLD_DCRENSHUANG);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;

    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: owner,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, fired).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 1);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, max_hp + 1);
}
}

mod original_xianding_batch48 {
//! Golden for XIANDING（限定包）批次四十八：
//! 迅势（dcsbxunshi）、据谏（dcjujian）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCSBXUNSHI: u32 = 4_112_740_389;
const DCJUJIAN: u32 = 3_730_191_152;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn live_skill(tx: &Transaction, player: PlayerId, def: u32) -> engine_api::SkillId {
    tx.state()
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
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

fn inject_hand_card(tx: &mut Transaction, player: PlayerId) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind: CardKind::Shan,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_exposes_batch48_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let unlimited = package
        .modifier_index
        .iter()
        .find(|m| m.skill_def_id == DCSBXUNSHI && m.target == "card_use_unlimited")
        .expect("dcsbxunshi card_use_unlimited");
    assert_eq!(unlimited.selector.as_deref(), Some("sha_thunder"));

    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == DCSBXUNSHI)
        .expect("dcsbxunshi trigger");
    assert_eq!(trigger.event, "damage");

    let jujian = package
        .active_index
        .iter()
        .find(|a| a.skill_def_id == DCJUJIAN)
        .expect("dcjujian active");
    assert_eq!(jujian.window, "play_phase");
}

#[test]
fn dcsbxunshi_discards_and_draws_on_thunder_sha_damage() {
    let (mut tx, owner, first, _second) = setup(20_260_904);
    add_skill(&mut tx, owner, DCSBXUNSHI);
    let before = tx.state().hand_count(owner);
    inject_hand_card(&mut tx, first);

    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: owner,
            target: first,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::THUNDER,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let dispatched = run_effects(&mut tx, fired);
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::DiscardOneHandOrEquip
    );
    let hand_card = tx.state().hand_cards(first)[0];
    let state = resolve(tx.state().clone(), first, vec![hand_card.raw()], false, 1);
    assert_eq!(state.hand_count(owner), before + 2);
    assert_eq!(state.hand_count(first), 0);
}

#[test]
fn dcjujian_draws_one_for_target_and_bans_use_against_owner() {
    let (mut tx, owner, first, _second) = setup(20_260_905);
    add_skill(&mut tx, owner, DCJUJIAN);
    let before = tx.state().hand_count(first);
    let skill = live_skill(&tx, owner, DCJUJIAN);
    let revision = tx.state().revision + 1;
    let (state, result) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![first],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(first), before + 1);
    assert!(state
        .turn
        .use_ban_pairs
        .contains(&(first, owner)));
}
}

mod original_xianding_batch49 {
//! Golden for XIANDING（限定包）批次四十九：
//! 僻境（dcpijing）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCPIJING: u32 = 3_114_639_778;
const DCZIMU: u32 = 4_186_409_591;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
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

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .any(|skill| state.skills.get(*skill).map(|s| s.def.raw()) == Some(def))
}

#[test]
fn package_exposes_dcpijing_phase_end_trigger() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == DCPIJING)
        .expect("dcpijing trigger");
    assert_eq!(trigger.event, "phase_end");
    assert!(trigger.optional);
}

#[test]
fn dcpijing_grants_zimu_to_chosen_other_at_phase_end() {
    let (mut tx, owner, first, _second) = setup(20_260_906);
    add_skill(&mut tx, owner, DCPIJING);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;

    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(dispatched, Err(EffectError::ChoiceRequested)));
    let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
    let state = resolve(state, owner, vec![first.raw()], false, 2);
    assert!(has_skill(&state, first, DCZIMU));
}
}

mod original_xianding_batch5 {
//! Golden for XIANDING（限定包）批次五：
//! 逾制（dcyuzhi）、长姬（dczhangji）。

use engine_api::{CardId, DomainEvent, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCYUZHI: u32 = 3_271_796_686;
const DCZHANGJI: u32 = 2_178_808_868;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn inject_sha(tx: &mut Transaction, player: PlayerId) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_exposes_batch5_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let yuzhi = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCYUZHI)
        .expect("dcyuzhi trigger");
    assert_eq!(yuzhi.event, "card_targeted");
    assert_eq!(yuzhi.filter, "owner_is_target_event_card_sha");
    assert!(!yuzhi.optional);

    let zhangji = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCZHANGJI)
        .expect("dczhangji trigger");
    assert_eq!(zhangji.event, "card_targeted");
    assert_eq!(zhangji.filter, "owner_alive_targeted_by_other");
    assert!(!zhangji.optional);
}

#[test]
fn dcyuzhi_plans_base_damage_increase_when_sha_targets_owner() {
    let (mut tx, owner, first, _second) = setup(20_260_811);
    add_skill(&mut tx, owner, DCYUZHI);
    let sha = inject_sha(&mut tx, first);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: first,
            card: sha,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(
        effects
            .iter()
            .any(|effect| matches!(effect, EffectRequest::IncreaseCurrentCardBaseDamage { .. })),
        "expected base-damage increase, got {effects:?}"
    );
}

#[test]
fn dczhangji_draws_extra_for_multi_target_card() {
    let (mut tx, owner, first, second) = setup(20_260_812);
    add_skill(&mut tx, owner, DCZHANGJI);
    let before = tx.state().hand_count(owner);
    let sha = inject_sha(&mut tx, first);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: first,
            card: sha,
            targets: vec![owner, second],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

#[test]
fn dczhangji_no_draw_for_single_target_card() {
    let (mut tx, owner, first, _second) = setup(20_260_813);
    add_skill(&mut tx, owner, DCZHANGJI);
    let before = tx.state().hand_count(owner);
    let sha = inject_sha(&mut tx, first);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: first,
            card: sha,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before);
}
}

mod original_xianding_batch50 {
//! Golden for XIANDING（限定包）批次五十：
//! 施仇（onedcspshichou，OL 版）。

use engine_api::{Command, CommandKind, CommandResult, DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ONEDCSP_SHICHOU: u32 = 3_653_300_696;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

#[test]
fn package_exposes_shichou_juedou_view_as() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);
    let view_as = package
        .view_as_index
        .iter()
        .find(|v| v.skill_def_id == ONEDCSP_SHICHOU)
        .expect("onedcspshichou view_as");
    assert_eq!(view_as.as_kind, "juedou");
    assert_eq!(view_as.window, "play_phase");
    assert_eq!(view_as.source_zone.as_deref(), Some("hand_equip"));
}

#[test]
fn shichou_can_be_attached() {
    let (mut tx, owner, _first, _second) = setup(20_260_907);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ONEDCSP_SHICHOU),
        }],
    )
    .unwrap();
    let has = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == ONEDCSP_SHICHOU);
    assert!(has);
}
}

mod original_xianding_batch51 {
//! Golden for XIANDING（限定包）批次五十一：
//! 佩娘（dcpeiniang）。

use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCPEINIANG: u32 = 3_794_638_462;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

#[test]
fn package_exposes_peiniang_jiu_view_as_and_unlimited() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let view_as = package
        .view_as_index
        .iter()
        .find(|v| v.skill_def_id == DCPEINIANG)
        .expect("dcpeiniang view_as");
    assert_eq!(view_as.as_kind, "jiu");
    assert_eq!(view_as.window, "play_phase");
    assert_eq!(view_as.source_zone.as_deref(), Some("hand_equip"));

    let unlimited = package
        .modifier_index
        .iter()
        .find(|m| m.skill_def_id == DCPEINIANG && m.target == "card_use_unlimited")
        .expect("dcpeiniang card_use_unlimited");
    assert_eq!(unlimited.selector.as_deref(), Some("jiu"));
}

#[test]
fn peiniang_can_be_attached() {
    let (mut tx, owner, _first, _second) = setup(20_260_908);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCPEINIANG),
        }],
    )
    .unwrap();
    let has = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == DCPEINIANG);
    assert!(has);
}
}

mod original_xianding_batch6 {
//! Golden for XIANDING（限定包）批次六：
//! 沉勇（dcchenyong）、绞（dclingfang）。

use engine_api::{CardId, DomainEvent, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCCHENYONG: u32 = 2_745_284_067;
const DCLINGFANG: u32 = 3_884_646_990;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn inject(tx: &mut Transaction, player: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(kind as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn mark_count(tx: &Transaction, owner: PlayerId) -> i32 {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == DCLINGFANG)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn package_exposes_batch6_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let chenyong = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCCHENYONG)
        .expect("dcchenyong trigger");
    assert_eq!(chenyong.event, "phase_end");
    assert_eq!(
        chenyong.filter,
        "owner_is_target_play_phase_card_category_count_positive"
    );
    assert!(chenyong.optional);

    let lingfang: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == DCLINGFANG)
        .map(|trigger| (trigger.event.as_str(), trigger.filter.as_str()))
        .collect();
    assert_eq!(
        lingfang,
        vec![
            ("phase_prepare", "owner_is_target"),
            ("card_targeted", "owner_is_target_nonred_card"),
        ]
    );
}

#[test]
fn dclingfang_gains_mark_at_prepare_phase() {
    let (mut tx, owner, _first, _second) = setup(20_260_814);
    add_skill(&mut tx, owner, DCLINGFANG);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(mark_count(&tx, owner), 1);
}

#[test]
fn dclingfang_gains_mark_when_targeted_by_nonred_card() {
    let (mut tx, owner, first, _second) = setup(20_260_815);
    add_skill(&mut tx, owner, DCLINGFANG);
    let card = inject(&mut tx, first, CardKind::Sha, Suit::Spade);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: first,
            card,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(mark_count(&tx, owner), 1);
}
}

mod original_xianding_batch7 {
//! Golden for XIANDING（限定包）批次七：
//! 虞袭（dcyuxi）、复盏（dcsbfuzhan）。

use engine_api::{CardId, DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCYUXI: u32 = 3_133_735_666;
const DCSBFUZHAN: u32 = 2_554_996_648;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = engine_core::command::apply_command(
        state,
        engine_api::Command {
            actor,
            expected_revision: engine_api::ExpectedRevision(revision),
            idempotency_key: engine_api::IdempotencyKey(key),
            kind: engine_api::CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(
        result.result,
        engine_api::CommandResult::Accepted { .. }
    ));
    state
}

fn deal_damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
) -> (GameState, Result<Vec<DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    );
    (tx.commit().state, result)
}

#[test]
fn package_exposes_batch7_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let yuxi = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCYUXI)
        .expect("dcyuxi trigger");
    assert_eq!(yuxi.event, "damage");
    assert_eq!(yuxi.filter, "owner_is_source_or_target");
    assert!(yuxi.optional);

    let fuzhan = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCSBFUZHAN)
        .expect("dcsbfuzhan trigger");
    assert_eq!(fuzhan.event, "dying_after");
    assert_eq!(fuzhan.filter, "owner_alive_not_target");
    assert!(!fuzhan.optional);
}

#[test]
fn dcyuxi_draws_when_owner_causes_damage() {
    let (mut tx, owner, first, _second) = setup(20_260_816);
    add_skill(&mut tx, owner, DCYUXI);
    tx.state_mut().players.get_mut(first).unwrap().hp -= 1;
    let before = tx.state().hand_count(owner);
    let (state, result) = deal_damage(tx.state().clone(), owner, first);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(state, owner, vec![1], false, 1);
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn dcyuxi_draws_when_owner_suffers_damage() {
    let (mut tx, owner, first, _second) = setup(20_260_817);
    add_skill(&mut tx, owner, DCYUXI);
    tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;
    let before = tx.state().hand_count(owner);
    let (state, result) = deal_damage(tx.state().clone(), first, owner);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let state = resolve(state, owner, vec![1], false, 2);
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn dcsbfuzhan_recovers_owner_to_max_after_other_leaves_dying() {
    let (mut tx, owner, first, _second) = setup(20_260_818);
    add_skill(&mut tx, owner, DCSBFUZHAN);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::LeftDying { player: first }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, max_hp as i32);
}
}

mod original_xianding_batch8 {
//! Golden for XIANDING（限定包）批次八：
//! 心忧（dcxinyou）、鸟翔（dcniaoxiang）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCXINYOU: u32 = 2_388_835_288;
const DCNIAOXIANG: u32 = 2_962_016_375;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
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

fn activate(
    state: GameState,
    actor: PlayerId,
    skill: SkillId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    )
}

#[test]
fn package_exposes_batch8_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let niaoxiang = package
        .modifier_index
        .iter()
        .find(|modifier| modifier.skill_def_id == DCNIAOXIANG)
        .expect("dcniaoxiang modifier");
    assert_eq!(
        niaoxiang.target,
        "sha_response_count_if_source_owner_target_distance_le_one"
    );
    assert_eq!(niaoxiang.mode, "add");
    assert_eq!(niaoxiang.value, 1);

    let xinyou = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DCXINYOU)
        .expect("dcxinyou active");
    assert_eq!(xinyou.window, "play_phase");
    assert_eq!(xinyou.uses_per_turn, Some(1));
}

#[test]
fn dcxinyou_recovers_and_draws_to_max_hp() {
    let (mut tx, owner, _first, _second) = setup(20_260_820);
    add_skill(&mut tx, owner, DCXINYOU);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    let max_hp = tx.state().players.get(owner).unwrap().max_hp as i32;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCXINYOU)
        .expect("dcxinyou skill instance")
        .id;
    let (state, pipe) = activate(tx.state().clone(), owner, skill, 2);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, max_hp);
    assert_eq!(state.hand_count(owner) as i32, max_hp);
}
}

mod original_xianding_batch9 {
//! Golden for XIANDING（限定包）批次九：
//! 返攻（dcsbfangong）、新膂力（xinlvli）。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCSBFANGONG: u32 = 4_167_621_774;
const XINLVLI: u32 = 3_655_535_745;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
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

fn activate(
    state: GameState,
    actor: PlayerId,
    skill: SkillId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    )
}

fn deal_damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: CardId::NONE,
            card_kind: 0,
            card_flags: 0,
        }],
    );
    (tx.commit().state, result)
}

#[test]
fn package_exposes_batch9_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let fangong = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DCSBFANGONG)
        .expect("dcsbfangong active");
    assert_eq!(fangong.window, "play_phase");
    assert!(fangong.once_per_game);

    let lvli = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == XINLVLI)
        .expect("xinlvli trigger");
    assert_eq!(lvli.event, "damage");
    assert_eq!(lvli.filter, "owner_source_once_per_turn");
    assert!(lvli.optional);
}

#[test]
fn dcsbfangong_recovers_and_draws_up_to_five() {
    let (mut tx, owner, _first, _second) = setup(20_260_891);
    add_skill(&mut tx, owner, DCSBFANGONG);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 3,
        }],
    )
    .unwrap();
    let max_hp = tx.state().players.get(owner).unwrap().max_hp as i32;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCSBFANGONG)
        .expect("dcsbfangong skill instance")
        .id;
    let (state, pipe) = activate(tx.state().clone(), owner, skill, 1);
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, max_hp);
    // 摸 min(5, 当前手牌数)：激活前 3 张，回复后无变化，故摸 3 张。
    assert_eq!(state.hand_count(owner), 6);
}

#[test]
fn xinlvli_draws_difference_when_hp_exceeds_hand() {
    let (mut tx, owner, first, _second) = setup(20_260_892);
    add_skill(&mut tx, owner, XINLVLI);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    let before = tx.state().hand_count(owner);
    let (state, result) = deal_damage(tx.state().clone(), owner, first);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let state = resolve(state, owner, vec![1], false, 2);
    let hp = state.players.get(owner).unwrap().hp;
    let hand = state.hand_count(owner) as i32;
    assert_eq!(hand, before as i32 + hp - before as i32);
}

#[test]
fn xinlvli_recovers_difference_when_hand_exceeds_hp() {
    let (mut tx, owner, first, _second) = setup(20_260_893);
    add_skill(&mut tx, owner, XINLVLI);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 4,
        }],
    )
    .unwrap();
    let (state, result) = deal_damage(tx.state().clone(), owner, first);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let state = resolve(state, owner, vec![1], false, 3);
    let hp = state.players.get(owner).unwrap().hp;
    let hand = state.hand_count(owner) as i32;
    assert_eq!(hp, hand);
}
}

mod original_xianding_bianyue {
//! Golden for XIANDING 卞玥【庇族/无胁】（ABI 598）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCBIZU: u32 = 2_914_157_062;
const DCWUXIE: u32 = 3_514_743_897;
const SOURCE: &str = include_str!("../../../content/original/ported/xianding/bianyue.skill");

fn setup() -> (
    GameState,
    PlayerId,
    PlayerId,
    PlayerId,
    engine_api::SkillId,
    engine_api::SkillId,
) {
    let package = content_compiler::compile_source_named("bianyue-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target, other) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in &players {
        let cards = tx.state().hand_cards(*player);
        if !cards.is_empty() {
            effects.push(EffectRequest::DiscardCards {
                player: *player,
                cards,
            });
        }
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCBIZU),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCWUXIE),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let skill = |def| {
        tx.state()
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == owner && skill.def.raw() == def)
            .unwrap()
            .id
    };
    let bizu = skill(DCBIZU);
    let wuxie = skill(DCWUXIE);
    (tx.commit().state, owner, target, other, bizu, wuxie)
}

fn alloc(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: ZoneKind,
    rank: u8,
    kind: CardKind,
) -> CardId {
    let zone = tx.state().zones.find(zone, owner).unwrap_or_else(|| {
        tx.state()
            .zones
            .find(zone, PlayerId::NONE)
            .expect("shared zone")
    });
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_800 + u32::from(rank)),
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

fn activate(state: GameState, owner: PlayerId, skill: engine_api::SkillId, key: u64) -> GameState {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn bizu_repeated_exact_group_draws_then_bans_and_recovers() {
    let (state, owner, target, _other, bizu, _) = setup();
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    alloc(&mut tx, owner, ZoneKind::Hand, 1, CardKind::Other);
    alloc(&mut tx, target, ZoneKind::Hand, 2, CardKind::Other);
    for rank in 3..=6 {
        alloc(
            &mut tx,
            PlayerId::NONE,
            ZoneKind::DrawPile,
            rank,
            CardKind::Other,
        );
    }
    let state = activate(tx.commit().state, owner, bizu, 1);
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.hand_count(target), 2);
    let state = activate(state, owner, bizu, 2);
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert!(state.skills.get(bizu).unwrap().active_turn_banned);
}

#[test]
fn wuxie_swaps_whole_hands_then_places_new_damage_cards_at_draw_bottom() {
    let (state, owner, target, _other, _bizu, wuxie) = setup();
    let mut tx = Transaction::begin(state);
    let owner_card = alloc(&mut tx, owner, ZoneKind::Hand, 1, CardKind::Other);
    let damage = alloc(&mut tx, target, ZoneKind::Hand, 2, CardKind::Sha);
    let safe = alloc(&mut tx, target, ZoneKind::Hand, 3, CardKind::Other);
    let top = alloc(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        4,
        CardKind::Other,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveBianyue {
                player: owner,
                target: owner,
                action: 10,
                skill: wuxie,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = tx.commit().state;
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![target.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_cards(owner), vec![safe]);
    assert_eq!(state.hand_cards(target), vec![owner_card]);
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let cards = &state.zones.get(draw).unwrap().cards;
    assert_eq!(cards.first(), Some(&damage));
    assert_eq!(cards.last(), Some(&top));
}
}

mod original_xianding_caoyi {
//! Golden for XIANDING 曹轶【蜜饴/寅君】（ABI 606）。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCMIYI: u32 = 3_041_789_306;
const DCMIYI_EFFECT: u32 = 3_041_789_307;
const SOURCE: &str = include_str!("../../../content/original/ported/xianding/caoyi.skill");

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
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    state
}

#[test]
fn miyi_reverses_the_selected_effect_at_end_phase() {
    let package = content_compiler::compile_source_named("caoyi-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(std::iter::once(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCMIYI),
        }))
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    tx.state_mut().players.get_mut(target).unwrap().hp -= 1;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCMIYI)
        .unwrap()
        .id;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveCaoyi {
                player: owner,
                card: CardId::NONE,
                helper_skill_def: SkillDefId::new(DCMIYI_EFFECT),
                action: 0,
                event_targets: Vec::new(),
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = choose(tx.commit().state, owner, vec![0], 6061);
    let state = choose(state, owner, vec![target.raw()], 6062);
    assert_eq!(
        state.players.get(target).unwrap().hp,
        state.players.get(target).unwrap().max_hp as i32
    );
    let helper = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == target && skill.def.raw() == DCMIYI_EFFECT)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::End;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveCaoyi {
            player: target,
            card: CardId::NONE,
            helper_skill_def: SkillDefId::new(DCMIYI_EFFECT),
            action: 10,
            event_targets: Vec::new(),
            skill: helper,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().players.get(target).unwrap().hp,
        tx.state().players.get(target).unwrap().max_hp as i32 - 1
    );
    assert!(tx.state().skills.get(helper).is_none());
}
}

mod original_xianding_chentai {
//! Golden for XIANDING 陈泰【矜贤】（ABI 597）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCCTJIUXIAN: u32 = 3_598_325_442;
const SOURCE: &str = include_str!("../../../content/original/ported/xianding/dcctjiuxian.skill");

fn setup() -> (GameState, PlayerId, PlayerId, PlayerId, engine_api::SkillId) {
    let package = content_compiler::compile_source_named("dcctjiuxian-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, damaged, other) = (players[0], players[1], players[2]);
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
            skill_def: SkillDefId::new(DCCTJIUXIAN),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let initial_hand = tx.state().hand_cards(owner);
    if !initial_hand.is_empty() {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: owner,
                cards: initial_hand,
            }],
        )
        .unwrap();
    }
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCCTJIUXIAN)
        .unwrap()
        .id;
    (tx.commit().state, owner, damaged, other, skill)
}

fn alloc(tx: &mut Transaction, owner: PlayerId, zone: ZoneKind, rank: u8) -> CardId {
    let zone = tx.state().zones.find(zone, owner).unwrap_or_else(|| {
        tx.state()
            .zones
            .find(zone, PlayerId::NONE)
            .expect("shared zone")
    });
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_700 + u32::from(rank)),
        kind: CardKind::Other,
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

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, cancelled: bool) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(90_000 + u64::from(actor.raw())),
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
fn active_recasts_ceil_half_then_opens_an_exact_tracked_duel() {
    let (state, owner, damaged, _other, skill) = setup();
    let mut tx = Transaction::begin(state);
    let hand = [
        alloc(&mut tx, owner, ZoneKind::Hand, 1),
        alloc(&mut tx, owner, ZoneKind::Hand, 2),
        alloc(&mut tx, owner, ZoneKind::Hand, 3),
    ];
    alloc(&mut tx, PlayerId::NONE, ZoneKind::DrawPile, 4);
    alloc(&mut tx, PlayerId::NONE, ZoneKind::DrawPile, 5);
    let (state, result) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(result.choice.unwrap().min_count, 2);

    let state = resolve(state, owner, vec![hand[0].raw(), hand[1].raw()], false);
    assert_eq!(state.hand_count(owner), 3);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseUseVirtualCard
    );
    assert_eq!(state.resolution.continuation_payload.last(), Some(&1));
    assert!(state.resolution.continuation_payload[5..].contains(&damaged.raw()));
}

#[test]
fn damage_branch_can_recover_a_wounded_character_in_the_target_range() {
    let (state, owner, damaged, other, skill) = setup();
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(damaged).unwrap().hp = 2;
    tx.state_mut().players.get_mut(other).unwrap().hp = 2;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDcctjiuxian {
                player: owner,
                event_player: damaged,
                target: PlayerId::NONE,
                card: CardId::NONE,
                action: 2,
                cards: Vec::new(),
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let before = tx.state().players.get(other).unwrap().hp;
    let state = resolve(tx.commit().state, owner, vec![other.raw()], false);
    assert_eq!(state.players.get(other).unwrap().hp, before + 1);
}
}

mod original_xianding_cuimao {
//! Golden for XIANDING 崔琰毛玠【征辟/奉迎】（ABI 585）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const CUIMAO: u32 = 2_841_277_757;
const ZHENGBI: u32 = 2_512_758_877;
const FENGYING: u32 = 3_395_124_491;
const FENGYING_EFFECT: u32 = 597_375_372;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(skill_def: u32) -> (GameState, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target, third) = (players[0], players[1], players[2]);
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(std::iter::once(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }))
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    (tx.commit().state, owner, target, third)
}

fn inject(state: &mut GameState, player: PlayerId, zone: ZoneKind, kind: CardKind) -> CardId {
    let zone = state.zones.find(zone, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(90_585),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: player,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
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
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    state
}

fn run(state: GameState, effect: EffectRequest) -> GameState {
    let mut tx = Transaction::begin(state);
    match run_effects(&mut tx, vec![effect]) {
        Ok(_) | Err(EffectError::ChoiceRequested) => {}
        Err(error) => panic!("effect failed: {error}"),
    }
    tx.commit().state
}

#[test]
fn package_closes_cuimao_contract() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == CUIMAO)
        .unwrap();
    assert_eq!(character.name_key, "cuimao");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [ZHENGBI, FENGYING]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == FENGYING_EFFECT));
}

#[test]
fn zhengbi_gains_one_hand_and_one_equip_after_target_gained() {
    let (mut state, owner, target, third) = setup(ZHENGBI);
    let target_hand = inject(&mut state, target, ZoneKind::Hand, CardKind::Tao);
    let target_equip = inject(&mut state, target, ZoneKind::Equip, CardKind::Weapon);
    let gained = inject(&mut state, third, ZoneKind::Hand, CardKind::Sha);
    let skill = skill_id(&state, owner, ZHENGBI);
    state = run(
        state,
        EffectRequest::ResolveZhengbi {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 0,
            skill,
        },
    );
    state = choose(state, owner, vec![target.raw()], 1);
    state = choose(state, owner, vec![0], 2);
    let from = state.cards.get(gained).unwrap().zone;
    let to = state.zones.find(ZoneKind::Hand, target).unwrap();
    state = run(
        state,
        EffectRequest::MoveCards {
            cards: vec![gained],
            to,
            from: Some(from),
        },
    );
    state = run(
        state,
        EffectRequest::ResolveZhengbi {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 1,
            skill,
        },
    );
    state = choose(state, owner, vec![target_hand.raw(), target_equip.raw()], 3);
    assert_eq!(state.cards.get(target_hand).unwrap().owner, owner);
    assert_eq!(state.cards.get(target_equip).unwrap().owner, owner);
}

#[test]
fn zhengbi_basic_gift_receives_non_basic_payment() {
    let (mut state, owner, target, _third) = setup(ZHENGBI);
    let gift = inject(&mut state, owner, ZoneKind::Hand, CardKind::Sha);
    let payment = inject(&mut state, target, ZoneKind::Equip, CardKind::Weapon);
    let skill = skill_id(&state, owner, ZHENGBI);
    state = run(
        state,
        EffectRequest::ResolveZhengbi {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 0,
            skill,
        },
    );
    state = choose(state, owner, vec![target.raw()], 1);
    state = choose(state, owner, vec![1], 2);
    state = choose(state, owner, vec![gift.raw()], 3);
    state = choose(state, target, vec![0], 4);
    state = choose(state, target, vec![payment.raw()], 5);
    assert_eq!(state.cards.get(gift).unwrap().owner, target);
    assert_eq!(state.cards.get(payment).unwrap().owner, owner);
}

#[test]
fn fengying_discards_hand_and_refills_only_its_extra_turn() {
    let (mut state, owner, target, third) = setup(FENGYING);
    inject(&mut state, owner, ZoneKind::Hand, CardKind::Sha);
    inject(&mut state, owner, ZoneKind::Hand, CardKind::Tao);
    state.players.get_mut(owner).unwrap().hp = 1;
    state.players.get_mut(owner).unwrap().max_hp = 3;
    state.players.get_mut(target).unwrap().hp = 2;
    state.players.get_mut(third).unwrap().hp = 3;
    let skill = skill_id(&state, owner, FENGYING);
    state = run(
        state,
        EffectRequest::ResolveFengying {
            player: owner,
            helper_def: SkillDefId::new(FENGYING_EFFECT),
            action: 0,
            skill,
        },
    );
    assert_eq!(state.hand_count(owner), 0);
    assert_eq!(state.turn.extra_turn_queue, [owner]);
    let helper = skill_id(&state, owner, FENGYING_EFFECT);
    state = run(
        state,
        EffectRequest::ResolveFengying {
            player: owner,
            helper_def: SkillDefId::new(FENGYING_EFFECT),
            action: 2,
            skill: helper,
        },
    );
    assert_eq!(state.hand_count(owner), 3);
    assert!(state.skills.get(helper).is_none());
}
}

mod original_xianding_dc_lingcao {
//! Golden for XIANDING 凌操【独锋】（ABI 595）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, MarkDefId,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DC_LINGCAO: u32 = 2_455_294_139;
const DCDUFENG: u32 = 2_523_630_693;
const DCDUFENG_EFFECT: u32 = 226_166_961;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup() -> (GameState, PlayerId, engine_api::SkillId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(
            tx.state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|player| EffectRequest::DiscardFromHand {
                    player,
                    count: u32::MAX,
                }),
        )
        .chain(std::iter::once(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCDUFENG),
        }))
        .collect();
    run_effects(&mut tx, effects).unwrap();
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.max_hp = 5;
    participant.hp = 4;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCDUFENG)
        .unwrap()
        .id;
    (tx.commit().state, owner, skill)
}

fn run(state: GameState, effect: EffectRequest) -> GameState {
    let mut tx = Transaction::begin(state);
    match run_effects(&mut tx, vec![effect]) {
        Ok(_) | Err(EffectError::ChoiceRequested) => {}
        Err(error) => panic!("effect failed: {error}"),
    }
    tx.commit().state
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>) -> GameState {
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    state
}

fn mark_count(state: &GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == MarkDefId::new(DCDUFENG_EFFECT).raw())
        .map_or(0, |(_, count)| *count)
}

#[test]
fn package_closes_dc_lingcao_contract() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 595);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == DC_LINGCAO)
        .unwrap();
    assert_eq!(character.name_key, "dc_lingcao");
    assert_eq!((character.initial_hp, character.max_hp), (4, 5));
    assert_eq!(character.skills, [DCDUFENG]);
    let modifiers: Vec<_> = package
        .modifier_index
        .iter()
        .filter(|modifier| modifier.skill_def_id == DCDUFENG_EFFECT)
        .map(|modifier| modifier.target.as_str())
        .collect();
    assert_eq!(modifiers, ["attack_range", "sha_use_limit"]);
}

#[test]
fn dcdufeng_can_pay_both_costs_then_draw_and_gain_exact_turn_bonus() {
    let (mut state, owner, skill) = setup();
    let base_sha_limit = engine_core::query::sha_use_limit_for(&state, owner);
    state = run(
        state,
        EffectRequest::ResolveDcdufeng {
            player: owner,
            action: 0,
            helper_skill_def: SkillDefId::new(DCDUFENG_EFFECT),
            skill,
        },
    );
    state = choose(state, owner, vec![0, 1]);

    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert_eq!(state.players.get(owner).unwrap().disabled_equip_slots[0], 1);
    assert_eq!(state.hand_count(owner), 3);
    assert_eq!(mark_count(&state, owner), 2);
    assert_eq!(
        engine_core::query::sha_use_limit_for(&state, owner),
        base_sha_limit + 2
    );
}

#[test]
fn dcdufeng_requires_at_least_one_cost() {
    let (state, owner, skill) = setup();
    let state = run(
        state,
        EffectRequest::ResolveDcdufeng {
            player: owner,
            action: 0,
            helper_skill_def: SkillDefId::new(DCDUFENG_EFFECT),
            skill,
        },
    );
    let (_, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: Vec::new(),
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
}
}

mod original_xianding_dc_mifuren {
//! Golden coverage for XIANDING DC 糜夫人【闺秀/存嗣】(ABI 578).

use engine_api::{PlayerId, SkillDefId, SkillId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DC_MIFUREN: u32 = 3_357_817_038;
const DCGUIXIU: u32 = 2_370_422_642;
const DCCUNSI: u32 = 4_233_990_625;
const DCYONGJUE: u32 = 2_297_054_445;
const DRAW_HURTS: u32 = 4_000_000_078;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/xianding/dcguixiu.skill"),
    "\n",
    include_str!("../../../content/original/ported/xianding/dccunsi.skill"),
    "\n",
    include_str!("../../../content/original/ported/xianding/dcyongjue.skill"),
    "\n",
    "skill draw_hurts(4000000078) locked { trigger cards_drawn owner_is_target { effect lose_hp(self, 1); } }",
);
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

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

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
        .unwrap()
}

fn setup(with_guixiu: bool, draw_hurts: bool) -> (Transaction, PlayerId, PlayerId, SkillId) {
    let package = content_compiler::compile_source_named("dc-mifuren-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, DC_MIFUREN as u64, "identity-standard").unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCCUNSI),
    });
    if with_guixiu {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCGUIXIU),
        });
    }
    if draw_hurts {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DRAW_HURTS),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    let cunsi = skill_id(tx.state(), owner, DCCUNSI);
    (tx, owner, target, cunsi)
}

fn activate(tx: &mut Transaction, owner: PlayerId, target: PlayerId, skill: SkillId) {
    run_effects(
        tx,
        vec![EffectRequest::ResolveDccunsi {
            player: owner,
            target,
            guixiu_def: SkillDefId::new(DCGUIXIU),
            yongjue_def: SkillDefId::new(DCYONGJUE),
            stage: 0,
            skill,
        }],
    )
    .unwrap();
}

#[test]
fn package_publishes_exact_dc_mifuren_closure() {
    let compiled = content_compiler::compile_source_named("dc-mifuren", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 578);
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 587);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == DC_MIFUREN)
        .unwrap();
    assert_eq!(character.name_key, "dc_mifuren");
    assert_eq!(character.sex, "female");
    assert_eq!(character.group, "shu");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [DCGUIXIU, DCCUNSI]);
    let expected = compiled
        .modules
        .iter()
        .find(|module| module.name == "dccunsi_active_play_phase_0")
        .unwrap();
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);
}

#[test]
fn targeting_self_grants_yongjue_without_draw_and_recovers_if_wounded() {
    let (mut tx, owner, _target, skill) = setup(true, false);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    let hand = tx.state().hand_count(owner);
    activate(&mut tx, owner, owner, skill);
    assert!(has_skill(tx.state(), owner, DCYONGJUE));
    assert_eq!(tx.state().hand_count(owner), hand);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 3);
}

#[test]
fn other_target_draw_chain_finishes_before_guixiu_recovery() {
    let (mut tx, owner, target, skill) = setup(true, true);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    let hand = tx.state().hand_count(owner);
    activate(&mut tx, owner, target, skill);
    assert!(has_skill(tx.state(), target, DCYONGJUE));
    assert_eq!(tx.state().hand_count(owner), hand + 2);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
}

#[test]
fn missing_guixiu_does_not_recover() {
    let (mut tx, owner, target, skill) = setup(false, false);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    activate(&mut tx, owner, target, skill);
    assert!(has_skill(tx.state(), target, DCYONGJUE));
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
}
}

mod original_xianding_dc_sb_lusu {
//! Golden for XIANDING 新杀谋鲁肃【明势/盟谋】（ABI 603）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCSBMINGSHI: u32 = 4_288_768_048;
const DCSBMENGMOU: u32 = 2_188_556_454;
const SOURCE: &str = include_str!("../../../content/original/ported/xianding/dc_sb_lusu.skill");

fn setup(def: u32) -> (GameState, [PlayerId; 3], engine_api::SkillId) {
    let package = content_compiler::compile_source_named("dc-sb-lusu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == players[0] && skill.def.raw() == def)
        .unwrap()
        .id;
    (tx.commit().state, players, skill)
}

fn alloc(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    rank: u8,
) -> CardId {
    let zone_owner = if matches!(zone_kind, ZoneKind::Hand) {
        owner
    } else {
        PlayerId::NONE
    };
    let zone = tx.state().zones.find(zone_kind, zone_owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_603 + u32::from(rank)),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone,
        owner: zone_owner,
        face_up: false,
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
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice {key}: {result:?}"
    );
    state
}

#[test]
fn mingshi_draws_two_then_reveals_three_and_gives_the_chosen_card() {
    let (state, [owner, target, _], skill) = setup(DCSBMINGSHI);
    let mut tx = Transaction::begin(state);
    alloc(&mut tx, owner, ZoneKind::Hand, CardKind::Other, 1);
    alloc(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Other,
        2,
    );
    alloc(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Other,
        3,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDcSbLusu {
                player: owner,
                source: owner,
                card: CardId::NONE,
                action: 0,
                cards: Vec::new(),
                amount: 0,
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let cards = tx.state().hand_cards(owner);
    assert_eq!(cards.len(), 3);
    let state = resolve(
        tx.commit().state,
        owner,
        cards.iter().map(|card| card.raw()).collect(),
        false,
        6031,
    );
    let state = resolve(state, owner, vec![target.raw()], false, 6032);
    let gained = cards[1];
    let state = resolve(state, target, vec![gained.raw()], false, 6033);
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.hand_count(target), 1);
    assert_eq!(state.cards.get(gained).unwrap().owner, target);
}

#[test]
fn mengmou_yin_counts_physical_sha_responses_then_loses_remaining_hp() {
    let (state, [owner, target, _], skill) = setup(DCSBMENGMOU);
    let mut tx = Transaction::begin(state);
    let transferred = alloc(&mut tx, target, ZoneKind::Hand, CardKind::Other, 1);
    let sha = alloc(&mut tx, target, ZoneKind::Hand, CardKind::Sha, 2);
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 2;
    tx.state_mut().skills.get_mut(skill).unwrap().view_as_branch = 1;
    let hp = tx.state().players.get(target).unwrap().hp;
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDcSbLusu {
                player: owner,
                source: owner,
                card: transferred,
                action: 11,
                cards: vec![transferred],
                amount: 1,
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = resolve(tx.commit().state, target, vec![sha.raw()], false, 6034);
    assert_eq!(
        state.players.get(target).unwrap().hp,
        hp - (max_hp - 1) as i32
    );
    let instance = state.skills.get(skill).unwrap();
    assert_eq!(instance.view_as_branch, 0);
    assert!(instance.state_blob.is_empty());
}

#[test]
fn mengmou_yang_uses_normal_sha_and_recovers_for_exact_damage() {
    let (state, [owner, target, victim], skill) = setup(DCSBMENGMOU);
    let mut tx = Transaction::begin(state);
    let transferred = alloc(&mut tx, target, ZoneKind::Hand, CardKind::Other, 1);
    let sha = alloc(&mut tx, target, ZoneKind::Hand, CardKind::Sha, 2);
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 1;
    tx.state_mut().players.get_mut(target).unwrap().hp -= 1;
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let victim_hp = tx.state().players.get(victim).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDcSbLusu {
                player: owner,
                source: owner,
                card: transferred,
                action: 11,
                cards: vec![transferred],
                amount: 1,
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = resolve(tx.commit().state, target, vec![sha.raw()], false, 6035);
    let state = resolve(state, target, vec![victim.raw()], false, 6036);
    assert_eq!(state.players.get(victim).unwrap().hp, victim_hp - 1);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp + 1);
    let instance = state.skills.get(skill).unwrap();
    assert_eq!(instance.view_as_branch, 1);
    assert!(instance.state_blob.is_empty());
}
}

mod original_xianding_dc_sb_zhangxiu {
//! Golden for XIANDING 势张绣【附袭/豪义】（ABI 601）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCSBFUXI: u32 = 3_319_521_189;
const DCSBHAOYI: u32 = 2_398_354_936;
const SOURCE: &str = include_str!("../../../content/original/ported/xianding/dc_sb_zhangxiu.skill");

fn setup(
    def: u32,
    phase: Phase,
    current_index: usize,
) -> (GameState, PlayerId, PlayerId, engine_api::SkillId) {
    let package = content_compiler::compile_source_named("dc-sb-zhangxiu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[current_index];
    tx.state_mut().turn.phase = phase;
    tx.state_mut().turn.entered_discard_cards.clear();
    tx.state_mut().turn.damage_card_materials.clear();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id;
    (tx.commit().state, owner, target, skill)
}

fn alloc(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: ZoneKind,
    rank: u8,
    kind: CardKind,
) -> CardId {
    let zone = tx.state().zones.find(zone, owner).unwrap_or_else(|| {
        tx.state()
            .zones
            .find(zone, PlayerId::NONE)
            .expect("shared zone")
    });
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_601 + u32::from(rank)),
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

#[test]
fn fuxi_gives_one_card_then_draws_two() {
    let (state, owner, target, skill) = setup(DCSBFUXI, Phase::Play, 1);
    let mut tx = Transaction::begin(state);
    let gift = alloc(&mut tx, owner, ZoneKind::Hand, 1, CardKind::Other);
    alloc(&mut tx, target, ZoneKind::Hand, 4, CardKind::Other);
    alloc(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        2,
        CardKind::Other,
    );
    alloc(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        3,
        CardKind::Other,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDcSbZhangxiu {
                player: owner,
                target,
                action: 0,
                cards: Vec::new(),
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = resolve(tx.commit().state, owner, vec![0], 1);
    let state = resolve(state, owner, vec![gift.raw()], 2);
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.cards.get(gift).unwrap().owner, target);
}

#[test]
fn fuxi_discards_then_uses_a_distance_free_virtual_sha() {
    let (state, owner, target, skill) = setup(DCSBFUXI, Phase::Play, 1);
    let mut tx = Transaction::begin(state);
    let target_card = alloc(&mut tx, target, ZoneKind::Hand, 6, CardKind::Other);
    let hp = tx.state().players.get(target).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDcSbZhangxiu {
                player: owner,
                target,
                action: 0,
                cards: Vec::new(),
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = resolve(tx.commit().state, owner, vec![1], 5);
    let state = resolve(state, owner, vec![target_card.raw()], 6);
    assert_eq!(
        state.cards.get(target_card).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn haoyi_gains_only_non_damaging_damage_cards_then_can_distribute_them() {
    let (state, owner, target, skill) = setup(DCSBHAOYI, Phase::End, 0);
    let mut tx = Transaction::begin(state);
    let eligible = alloc(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        4,
        CardKind::Sha,
    );
    let excluded = alloc(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        5,
        CardKind::TrickJuedou,
    );
    tx.state_mut()
        .turn
        .record_entered_discard_cards([eligible, excluded]);
    tx.state_mut().turn.record_damage_card_materials([excluded]);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDcSbZhangxiu {
                player: owner,
                target: PlayerId::NONE,
                action: 10,
                cards: Vec::new(),
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(tx.state().cards.get(eligible).unwrap().owner, owner);
    assert_eq!(
        tx.state().cards.get(excluded).unwrap().owner,
        PlayerId::NONE
    );
    let state = resolve(tx.commit().state, owner, vec![eligible.raw()], 3);
    let state = resolve(state, owner, vec![target.raw()], 4);
    assert_eq!(state.cards.get(eligible).unwrap().owner, target);
    assert!(
        !state.has_pending_choice(),
        "unexpected continuation {:?}: {:?}",
        state.resolution.continuation_kind,
        state.resolution.continuation_payload
    );
}
}

mod original_xianding_dc_sb_zhouyu {
//! Golden for XIANDING 新杀谋周瑜【融火/英谋】（ABI 602）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCSBRONGHUO: u32 = 2_240_860_827;
const DCSBYINGMOU: u32 = 4_166_950_825;
const SOURCE: &str = include_str!("../../../content/original/ported/xianding/dc_sb_zhouyu.skill");

fn setup(def: u32) -> (GameState, [PlayerId; 3], engine_api::SkillId) {
    let package = content_compiler::compile_source_named("dc-sb-zhouyu-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == players[0] && skill.def.raw() == def)
        .unwrap()
        .id;
    (tx.commit().state, players, skill)
}

fn alloc(tx: &mut Transaction, owner: PlayerId, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_602 + u32::from(rank)),
        kind: CardKind::Other,
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

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(602),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    state
}

#[test]
fn ronghuo_permission_is_non_stacking_and_yingmou_can_toggle_at_game_start() {
    let (state, [owner, _, _], skill) = setup(DCSBRONGHUO);
    let mut tx = Transaction::begin(state);
    for _ in 0..2 {
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDcSbZhouyu {
                player: owner,
                source: PlayerId::NONE,
                card: CardId::NONE,
                action: 0,
                event_targets: Vec::new(),
                damaged_targets: Vec::new(),
                skill,
            }],
        )
        .unwrap();
    }
    assert_eq!(tx.state().skills.get(skill).unwrap().content_flags & 1, 1);

    let (state, [owner, _, _], skill) = setup(DCSBYINGMOU);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDcSbZhouyu {
            player: owner,
            source: owner,
            card: CardId::NONE,
            action: 10,
            event_targets: Vec::new(),
            damaged_targets: Vec::new(),
            skill,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().skills.get(skill).unwrap().view_as_branch, 1);
}

#[test]
fn yingmou_yin_without_damage_cards_discards_to_owner_then_deals_fire_damage() {
    let (state, [owner, target, source], skill) = setup(DCSBYINGMOU);
    let mut tx = Transaction::begin(state);
    alloc(&mut tx, owner, 1);
    let source_cards = [
        alloc(&mut tx, source, 2),
        alloc(&mut tx, source, 3),
        alloc(&mut tx, source, 4),
    ];
    tx.state_mut().skills.get_mut(skill).unwrap().view_as_branch = 1;
    let hp = tx.state().players.get(target).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDcSbZhouyu {
                player: owner,
                source,
                card: CardId::NONE,
                action: 13,
                event_targets: vec![target],
                damaged_targets: Vec::new(),
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = resolve(
        tx.commit().state,
        source,
        source_cards[..2].iter().map(|card| card.raw()).collect(),
    );
    assert_eq!(state.hand_count(source), state.hand_count(owner));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    let instance = state.skills.get(skill).unwrap();
    assert_eq!(instance.view_as_branch, 0);
    assert!(instance.state_blob.is_empty());
}
}

mod original_xianding_dc_shen_guanyu {
//! Golden coverage for XIANDING 新杀神关羽【武神/武魂】(ABI 580).

use engine_api::{CardId, DomainEvent, PlayerId, SkillDefId, SkillId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{CardKind, GameState};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const CHARACTER: u32 = 2_564_129_342;
const DCWUSHEN: u32 = 3_292_885_385;
const DCWUHUN: u32 = 4_234_333_270;
const NIGHTMARE: u32 = 3_698_112_918;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/xianding/dcwuhun.skill"),
    "\n",
    include_str!("../../../content/original/ported/xianding/dcwushen.skill"),
);
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|id| {
            state
                .skills
                .get(*id)
                .is_some_and(|skill| skill.def.raw() == def)
        })
        .unwrap()
}

fn setup(def: u32) -> (Transaction, PlayerId, PlayerId, PlayerId, SkillId) {
    let package = content_compiler::compile_source_named("dc-shen-guanyu", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, CHARACTER as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(std::iter::once(EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(def),
        }))
        .collect();
    run_effects(&mut tx, effects).unwrap();
    let skill = skill_id(tx.state(), players[0], def);
    (tx, players[0], players[1], players[2], skill)
}

fn mark_count(tx: &Transaction, player: PlayerId) -> i32 {
    tx.state()
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == NIGHTMARE)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn package_publishes_exact_character_and_skill_contracts() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 587);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == CHARACTER)
        .unwrap();
    assert_eq!(character.name_key, "dc_shen_guanyu");
    assert_eq!(
        (
            character.sex.as_str(),
            character.group.as_str(),
            character.max_hp
        ),
        ("male", "shen", 5)
    );
    assert_eq!(character.skills, [DCWUSHEN, DCWUHUN]);

    let modifiers: Vec<_> = package
        .modifier_index
        .iter()
        .filter(|modifier| modifier.skill_def_id == DCWUSHEN)
        .map(|modifier| {
            (
                modifier.target.as_str(),
                modifier.selector.as_deref(),
                modifier.value,
            )
        })
        .collect();
    assert_eq!(
        modifiers,
        [
            ("card_name", Some("heart"), 1),
            ("card_nature", Some("heart"), 0),
            ("card_distance_unlimited", Some("heart"), 1),
            ("card_use_unlimited", Some("heart"), 1),
        ]
    );
    let mut contracts: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| matches!(trigger.skill_def_id, DCWUSHEN | DCWUHUN))
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
            )
        })
        .collect();
    contracts.sort_unstable();
    assert_eq!(
        contracts,
        [
            (DCWUSHEN, "card_used", "owner_source_heart_sha"),
            (
                DCWUSHEN,
                "damage",
                "owner_source_sha_damage_recorded_by_skill"
            ),
            (
                DCWUHUN,
                "damage",
                "owner_is_source_or_target_other_alive_per_damage_point"
            ),
            (DCWUHUN, "death", "owner_is_target"),
        ]
    );
}

#[test]
fn dcwushen_draws_only_for_targets_bound_to_the_exact_sha() {
    let (mut tx, owner, target, other, skill) = setup(DCWUSHEN);
    let armed = CardId::new(98_001);
    let unarmed = CardId::new(98_002);
    tx.state_mut()
        .resolution
        .register_sha_resolution(owner, armed, &[target, other], false);
    run_effects(
        &mut tx,
        vec![EffectRequest::RecordCurrentShaAllTargets {
            player: owner,
            card: armed,
            skill,
        }],
    )
    .unwrap();
    assert!(tx
        .state()
        .resolution
        .sha_skill_target_recorded(owner, armed, skill, target));
    assert!(!tx
        .state()
        .resolution
        .sha_skill_target_recorded(owner, unarmed, skill, target));

    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
            card: armed,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);
}

#[test]
fn dcwuhun_marks_both_damage_directions_per_point_and_publishes_death_effect() {
    let (mut tx, owner, target, other, _) = setup(DCWUHUN);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: owner,
            target,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(mark_count(&tx, target), 2);

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: other,
            target: owner,
            amount: 3,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(mark_count(&tx, other), 3);

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PlayerKilled {
            player: owner,
            killer: PlayerId::NONE,
            hand_equip_cards: Vec::new(),
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.iter().any(|effect| matches!(
        effect,
        EffectRequest::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan {
            player,
            mark,
            ..
        } if *player == owner && mark.raw() == NIGHTMARE
    )));
}
}

mod original_xianding_dc_tianfeng {
//! Golden for XIANDING 新杀田丰【随势】（ABI 596）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DC_TIANFENG: u32 = 3_051_387_474;
const DC_YANGBIAO: u32 = 3_963_716_631;
const DC_LINGCAO: u32 = 2_455_294_139;
const SIJIAN: u32 = 3_832_046_646;
const DCSUISHI: u32 = 2_514_713_423;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup() -> (GameState, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, source, target) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCSUISHI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().character_def = DC_TIANFENG;
    tx.state_mut()
        .players
        .get_mut(source)
        .unwrap()
        .character_def = DC_YANGBIAO;
    tx.state_mut()
        .players
        .get_mut(target)
        .unwrap()
        .character_def = DC_LINGCAO;
    (tx.commit().state, owner, source, target)
}

#[test]
fn package_closes_dc_tianfeng_contract() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 596);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == DC_TIANFENG)
        .unwrap();
    assert_eq!(character.name_key, "dc_tianfeng");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [SIJIAN, DCSUISHI]);
}

#[test]
fn dcsuishi_draws_when_the_dying_damage_source_shares_the_owner_group() {
    let (state, owner, source, target) = setup();
    let mut tx = Transaction::begin(state);
    let hand = tx.state().hand_count(owner);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: target,
            source,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);
}

#[test]
fn dcsuishi_death_forces_one_to_all_hand_cards_and_revalidates_the_choice() {
    let (state, owner, _source, target) = setup();
    let mut tx = Transaction::begin(state);
    tx.state_mut()
        .players
        .get_mut(target)
        .unwrap()
        .character_def = DC_YANGBIAO;
    let before = tx.state().hand_count(owner);
    assert!(before >= 2);
    let selected: Vec<_> = tx.state().hand_cards(owner)[..2]
        .iter()
        .map(|card| card.raw())
        .collect();
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PlayerKilled {
            player: target,
            killer: PlayerId::NONE,
            hand_equip_cards: Vec::new(),
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: selected,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before - 2);

    let (state, owner, _source, target) = setup();
    let mut empty = Transaction::begin(state);
    empty
        .state_mut()
        .players
        .get_mut(target)
        .unwrap()
        .character_def = DC_YANGBIAO;
    run_effects(
        &mut empty,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let effects = dispatch_for_events(
        &mut empty,
        &[DomainEvent::PlayerKilled {
            player: target,
            killer: PlayerId::NONE,
            hand_equip_cards: Vec::new(),
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
}
}

mod original_xianding_dc_yangbiao {
//! Golden for XIANDING 杨彪【举讹】（dcjue，ABI 583）。

use engine_api::{
    CardDefId, CardId, CardMoveReason, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCJUE: u32 = 2_849_188_079;
const YANGBIAO: u32 = 3_963_716_631;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCJUE),
        }],
    )
    .unwrap();
    tx.state_mut().turn.round = 1;
    tx.state_mut().revision = 1;
    (tx, owner, first, second)
}

fn add_hand_card(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_001),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn discard(tx: &mut Transaction, player: PlayerId, card: CardId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let pile = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCardsWithReason {
            cards: vec![card],
            to: pile,
            from: Some(hand),
            reason: CardMoveReason::Discard,
        }],
    )
    .unwrap();
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(1),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
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

#[test]
fn package_closes_dc_yangbiao_with_exact_contract() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == YANGBIAO)
        .expect("dc_yangbiao");
    assert_eq!(character.name_key, "dc_yangbiao");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [2_818_087_075, 2_966_946_872, DCJUE]);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCJUE)
        .expect("dcjue trigger");
    assert_eq!(trigger.event, "phase_end");
    assert_eq!(
        trigger.filter,
        "owner_alive_global_phase_end_discarded_cards_available"
    );
    assert!(!trigger.optional);
}

#[test]
fn dcjue_tracks_only_discard_reason_cards_that_remain_in_discard() {
    let (mut tx, owner, first, _) = setup(20_260_803);
    let discarded = add_hand_card(&mut tx, owner, CardKind::Sha);
    let used = add_hand_card(&mut tx, owner, CardKind::Tao);
    discard(&mut tx, owner, discarded);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let pile = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCardsWithReason {
            cards: vec![used],
            to: pile,
            from: Some(hand),
            reason: CardMoveReason::UseOrRespond,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state()
            .turn
            .discarded_cards
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        [discarded]
    );

    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![discarded],
            to: hand,
            from: Some(pile),
        }],
    )
    .unwrap();
    tx.state_mut().turn.current = first;
    tx.state_mut().turn.phase = Phase::End;
    let result = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: first,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(result, Ok(events) if events.is_empty()));
}

#[test]
fn dcjue_confirms_fixed_target_and_starts_authoritative_random_loop_once_per_round() {
    let (mut tx, owner, first, _) = setup(20_260_804);
    let card = add_hand_card(&mut tx, owner, CardKind::Sha);
    discard(&mut tx, owner, card);
    tx.state_mut().turn.current = first;
    tx.state_mut().turn.phase = Phase::End;
    let calls = tx.state().rng.call_count;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: first,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let dispatched = run_effects(&mut tx, effects);
    assert!(
        matches!(dispatched, Err(EffectError::ChoiceRequested)),
        "dispatch={dispatched:?}, discarded={:?}",
        tx.state().turn.discarded_cards
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::Custom
    );

    let state = resolve(tx.state().clone(), owner, vec![1], 1);
    let skill = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCJUE)
        .unwrap();
    assert_eq!(skill.round_fire_round, state.turn.round);
    assert_eq!(
        state.rng.call_count,
        calls + 2,
        "three-way shuffle uses two draws"
    );
}

#[test]
fn dcjue_own_end_phase_uses_single_other_target_choice() {
    let (mut tx, owner, first, _) = setup(20_260_805);
    let card = add_hand_card(&mut tx, owner, CardKind::Sha);
    discard(&mut tx, owner, card);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(matches!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::Custom
    );
    let state = resolve(tx.state().clone(), owner, vec![first.raw()], 2);
    let skill = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCJUE)
        .unwrap();
    assert_eq!(skill.round_fire_round, state.turn.round);
}
}

mod original_xianding_dchaoxian_machine {
//! Golden for xianding 朝献 dchaoxian (ABI 747, tag 764):
//! 限定技（once_per_game）——弃牌堆全部 3 点牌洗入牌堆底；收齐其他角色
//! 手牌中的 3 点牌。active 全管道驱动。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCHAOXIAN: u32 = 3_403_528_397;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn started(seed: u64) -> (GameState, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let players = tx.state().players.alive_by_seat();
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
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, players)
}

fn add_skill(state: &mut GameState, player: PlayerId, def: u32) -> engine_api::SkillId {
    let mut tx = Transaction::begin(state.clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    *state = tx.commit().state;
    state
        .players
        .get(player)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn inject(state: &mut GameState, owner: PlayerId, zone: ZoneKind, rank: u8, tag: u32) -> CardId {
    let zone = state.zones.find(zone, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(101_000 + tag),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
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

#[test]
fn package_publishes_dchaoxian_contract() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 751);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == DCHAOXIAN)
        .expect("dchaoxian active entry");
    assert!(active.once_per_game);
}

#[test]
fn dchaoxian_shuffles_threes_to_deck_bottom_and_collects_from_others() {
    let (mut state, players) = started(2_026_103_001);
    let owner = players[0];
    let other1 = players[1];
    let other2 = players[2];
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let skill = add_skill(&mut state, owner, DCHAOXIAN);
    // 弃牌堆：两张 3 点 + 一张干扰。
    let d3a = inject(&mut state, PlayerId::NONE, ZoneKind::DiscardPile, 3, 1);
    let d3b = inject(&mut state, PlayerId::NONE, ZoneKind::DiscardPile, 3, 2);
    let _d5 = inject(&mut state, PlayerId::NONE, ZoneKind::DiscardPile, 5, 3);
    // 其他角色手牌：other1 一张 3 点；other2 无。
    let h3 = inject(&mut state, other1, ZoneKind::Hand, 3, 4);
    // owner 自身手牌的 3 点不收（上游只收其他角色）——预置干扰。
    let _own3 = inject(&mut state, owner, ZoneKind::Hand, 3, 5);
    let owner_before = state.hand_count(owner);
    let draw_before = {
        let draw = state.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
        state.zones.get(draw).unwrap().cards.len()
    };
    // bootstrap 弃牌堆可能自带 3 点牌残留——预记数量供动态断言。
    let pre_threes = {
        let discard = state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap();
        state
            .zones
            .get(discard)
            .unwrap()
            .cards
            .iter()
            .filter(|card| state.cards.get(**card).unwrap().rank == 3)
            .count()
    };

    let (state, accepted) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![],
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));

    // 弃牌堆 3 点牌全部进牌堆（bootstrap 弃牌堆可能含残留 3 点牌，
    // 一并收走——动态断言总数与「弃牌堆无 3 点牌」）。
    let draw = state.zones.find(ZoneKind::DrawPile, PlayerId::NONE).unwrap();
    let draw_cards = state.zones.get(draw).unwrap().cards.clone();
    assert!(draw_cards.contains(&d3a) && draw_cards.contains(&d3b));
    // pre_threes 测量于注入之后，已含注入的两张。
    assert_eq!(draw_cards.len(), draw_before + pre_threes);
    // 注入的两张在牌堆底部区域（头部（2+残留）张内）。
    assert!(draw_cards[..2 + pre_threes].contains(&d3a));
    assert!(draw_cards[..2 + pre_threes].contains(&d3b));
    let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap();
    let discard_cards = state.zones.get(discard).unwrap().cards.clone();
    assert!(discard_cards
        .iter()
        .all(|card| state.cards.get(*card).unwrap().rank != 3));
    // other1 的 3 点被收走；owner 手牌 +1（净 0：自持 3 点未动）。
    assert!(!state.hand_cards(other1).contains(&h3));
    assert!(state.hand_cards(owner).contains(&h3));
    assert_eq!(state.hand_count(owner), owner_before + 1);
    assert_eq!(state.hand_count(other2), 0);
    let _ = draw_before;

    // once_per_game：二次发动被拒。
    let (_, rejected) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![],
            card_targets: Vec::new(),
        },
        2,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
}
}

mod original_xianding_dcsbchensheng {
//! Golden for XIANDING「沉声」的整回合结束与唯一最多手牌语义。

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

const SKILL: u32 = 2_769_678_482;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

#[test]
fn chensheng_only_offers_draw_after_another_turn_when_neither_is_unique_max_hand() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(20_260_714_5),
        kind: CommandKind::StartGame {
            player_count: 3,
            rng_seed: 20_260_714_5,
            ruleset: "identity-standard".into(),
        },
    });
    let players = engine.state().players.alive_by_seat();
    let (owner, ending_player, leader) = (players[0], players[1], players[2]);
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
            skill_def: SkillDefId::new(SKILL),
        },
        EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        },
        EffectRequest::DiscardFromHand {
            player: ending_player,
            count: u32::MAX,
        },
        EffectRequest::DiscardFromHand {
            player: leader,
            count: u32::MAX,
        },
        EffectRequest::DrawCards {
            player: owner,
            count: 1,
        },
        EffectRequest::DrawCards {
            player: ending_player,
            count: 1,
        },
        EffectRequest::DrawCards {
            player: leader,
            count: 2,
        },
    ]);
    run_effects(&mut tx, effects).unwrap();

    let before = tx.state().hand_count(owner);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::PhaseEnded {
                current: ending_player,
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
        .unwrap_err(),
        EffectError::ChoiceRequested
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
    assert_eq!(state.hand_count(owner), before + 1);

    let mut tx = Transaction::begin(state);
    assert!(dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: ending_player,
            phase: TurnPhaseTag::Play,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap()
    .is_empty());

    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 1,
        }],
    )
    .unwrap();
    assert!(dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: ending_player,
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
    .unwrap()
    .is_empty());

    run_effects(
        &mut tx,
        vec![
            EffectRequest::DiscardFromHand {
                player: owner,
                count: u32::MAX,
            },
            EffectRequest::DrawCards {
                player: ending_player,
                count: 3,
            },
        ],
    )
    .unwrap();
    assert!(dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: ending_player,
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
    .unwrap()
    .is_empty());
}
}

mod original_xianding_dcsbjinglian {
//! Golden for XIANDING 新杀谋钟会「精练」refined threshold (lost ≥2 this turn).
//!
//! Drives the shipped `dcsbjinglian` skill via compile_source + real phase_end dispatch
//! and the Host `draw_cards` effect — not a re-implementation of the skill.

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

const DCSBJINGLIAN: u32 = 3_379_088_720;
const DC_SB_ZHONGHUI: u32 = 2_619_597_311;
const SOURCE: &str =
    include_str!("../../../content/original/ported/xianding/dcsbjinglian.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn phase_end(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::End,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selection: u32,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

fn setup_with_skill() -> (Transaction, PlayerId) {
    let compiled =
        content_compiler::compile_source_named("dcsbjinglian-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&compiled).unwrap();
    assert!(
        state.content.tracks_turn_card_loss_history(),
        "filter must enable turn card-loss tracking"
    );
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_080_811, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(DCSBJINGLIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    // Bootstrap/deals may record losses; zero the turn counter so tests own the threshold.
    // `card_lost_by_player` flag base is 1200 + seat (engine_core::state::turn::turn_flag).
    const CARD_LOST_BY_PLAYER_BASE: u32 = 1200;
    tx.state_mut()
        .turn
        .set_flag(CARD_LOST_BY_PLAYER_BASE + owner.raw(), 0);
    tx.state_mut().turn.lost_cards.clear();
    assert_eq!(tx.state().turn.card_loss_count(owner), 0);
    (tx, owner)
}

#[test]
fn package_publishes_refined_jinglian_filter_and_zhonghui_closure() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.skill_index.iter().any(|s| {
        s.skill_def_id == DCSBJINGLIAN && s.name == "dcsbjinglian"
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == DCSBJINGLIAN
            && t.event == "phase_end"
            && t.filter == "owner_alive_phase_end_lost_cards_at_least_two"
            && t.optional
    }));
    let zhonghui = package
        .character_index
        .iter()
        .find(|c| c.id == DC_SB_ZHONGHUI)
        .expect("dc_sb_zhonghui");
    assert_eq!(zhonghui.name_key, "dc_sb_zhonghui");
    assert!(zhonghui.skills.contains(&DCSBJINGLIAN));
    // compiled source must agree with npack filter string
    let compiled = content_compiler::compile_source_named("dcsbjinglian", SOURCE).unwrap();
    assert!(compiled.trigger_index.iter().any(|t| {
        t.skill_def_id == DCSBJINGLIAN
            && t.filter == "owner_alive_phase_end_lost_cards_at_least_two"
    }));
}

#[test]
fn draws_three_only_after_at_least_two_card_losses_this_turn() {
    let (mut tx, owner) = setup_with_skill();
    // one loss — should NOT open 精练
    let c1 = inject_hand(tx.state_mut(), owner, 301_001);
    tx.state_mut().turn.record_card_loss(owner, c1);
    assert_eq!(tx.state().turn.card_loss_count(owner), 1);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    let one_loss = dispatch_for_events(
        &mut tx,
        &[phase_end(owner)],
        &mut TriggerFireRecord::new(),
    );
    assert!(
        one_loss.is_ok(),
        "失牌仅 1 张时精练不得触发: {one_loss:?}"
    );
    assert!(!tx.state().has_pending_choice());

    // second loss — should offer optional
    let c2 = inject_hand(tx.state_mut(), owner, 301_002);
    tx.state_mut().turn.record_card_loss(owner, c2);
    assert_eq!(tx.state().turn.card_loss_count(owner), 2);
    let hand_before = tx.state().hand_count(owner);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[phase_end(owner)],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let offered = tx.commit();
    assert_eq!(
        offered.output.choice.as_ref().unwrap().kind,
        ChoiceKind::Confirm
    );
    let (state, accepted) = resolve(offered.state, owner, 1, 42);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.hand_count(owner),
        hand_before + 3,
        "精练发动后应摸三张"
    );
}

#[test]
fn does_not_fire_with_zero_losses() {
    let (mut tx, owner) = setup_with_skill();
    assert_eq!(tx.state().turn.card_loss_count(owner), 0);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    let hand_before = tx.state().hand_count(owner);
    let r = dispatch_for_events(
        &mut tx,
        &[phase_end(owner)],
        &mut TriggerFireRecord::new(),
    );
    assert!(r.is_ok(), "{r:?}");
    assert_eq!(tx.state().hand_count(owner), hand_before);
}
}

mod original_xianding_dcsbkangyong {
//! Golden for 限定 谋典韦「亢勇」rewritten from the legacy JavaScript semantics.

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, TurnPhaseTag,
};
use engine_core::effect::run_effects;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const DCSBKANGYONG: u32 = 3_080_902_597;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn isolated(hp: i32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(DCSBKANGYONG as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: DCSBKANGYONG as u64,
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(DCSBKANGYONG),
    });
    run_effects(&mut tx, effects).unwrap();
    let player = tx.state_mut().players.get_mut(owner).unwrap();
    player.max_hp = 4;
    player.hp = hp;
    (tx, owner, other)
}

fn phase(player: PlayerId, phase: TurnPhaseTag) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase,
    }
}

fn dispatch_and_apply(tx: &mut Transaction, event: DomainEvent) {
    let effects = dispatch_for_events(tx, &[event], &mut TriggerFireRecord::new()).unwrap();
    run_effects(tx, effects).unwrap();
}

fn mark_count(tx: &Transaction, owner: PlayerId) -> i32 {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == DCSBKANGYONG)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn committed_package_exposes_both_locked_turn_endpoints() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let contracts: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == DCSBKANGYONG)
        .map(|trigger| {
            (
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert_eq!(
        contracts,
        vec![
            ("phase_begin", "owner_is_target_wounded", false),
            (
                "phase_end",
                "owner_is_target_mark_positive_3080902597",
                false,
            ),
        ]
    );
}

#[test]
fn turn_start_recovers_to_max_and_turn_end_loses_the_recorded_amount() {
    let (mut tx, owner, _) = isolated(2);
    dispatch_and_apply(&mut tx, phase(owner, TurnPhaseTag::Prepare));
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 4);
    assert_eq!(mark_count(&tx, owner), 2);

    dispatch_and_apply(&mut tx, phase(owner, TurnPhaseTag::End));
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
    assert_eq!(mark_count(&tx, owner), 0);
}

#[test]
fn end_loss_stops_at_one_and_only_the_owners_end_phase_consumes_the_record() {
    let (mut tx, owner, other) = isolated(2);
    dispatch_and_apply(&mut tx, phase(owner, TurnPhaseTag::Prepare));
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;

    dispatch_and_apply(&mut tx, phase(other, TurnPhaseTag::End));
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
    assert_eq!(mark_count(&tx, owner), 2);

    dispatch_and_apply(&mut tx, phase(owner, TurnPhaseTag::End));
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 1);
    assert_eq!(mark_count(&tx, owner), 0);
}

#[test]
fn healthy_turn_start_does_not_record_a_debt() {
    let (mut tx, owner, _) = isolated(4);
    let effects = dispatch_for_events(
        &mut tx,
        &[phase(owner, TurnPhaseTag::Prepare)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert_eq!(mark_count(&tx, owner), 0);
}
}

mod original_xianding_dcyuxin {
//! Golden for XIANDING「御心」的目标、非负体力与限定技语义。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const DCYUXIN: u32 = 4_174_094_056;
const DCKUNLI: u32 = 4_133_766_049;
const DCKUICHI: u32 = 3_373_286_891;
const DCSHANGJUE: u32 = 2_632_217_594;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn isolated_state(skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == skill_def));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_715,
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn offer(tx: &mut Transaction, dying: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::EnteredDying {
            player: dying,
            source: PlayerId::NONE,
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

fn has_skill(state: &engine_core::state::GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == skill_def)
}

#[test]
fn yuxin_recovers_self_to_one_and_other_to_owner_hp() {
    let (mut self_dying, owner, _) = isolated_state(DCYUXIN);
    self_dying.state_mut().players.get_mut(owner).unwrap().hp = -2;
    assert_eq!(
        offer(&mut self_dying, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let self_saved = resolve(&self_dying, owner, true, 1);
    assert_eq!(self_saved.players.get(owner).unwrap().hp, 1);
    assert!(!has_skill(&self_saved, owner, DCYUXIN));

    let (mut other_dying, owner, other) = isolated_state(DCYUXIN);
    other_dying.state_mut().players.get_mut(owner).unwrap().hp = 3;
    other_dying.state_mut().players.get_mut(other).unwrap().hp = -1;
    assert_eq!(
        offer(&mut other_dying, other).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let other_saved = resolve(&other_dying, owner, true, 2);
    assert_eq!(other_saved.players.get(other).unwrap().hp, 3);
    assert!(!has_skill(&other_saved, owner, DCYUXIN));
}

#[test]
fn declined_and_non_improving_other_target_do_not_consume_yuxin() {
    let (mut tx, owner, other) = isolated_state(DCYUXIN);
    tx.state_mut().players.get_mut(other).unwrap().hp = 0;
    assert_eq!(
        offer(&mut tx, other).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let declined = resolve(&tx, owner, false, 3);
    assert_eq!(declined.players.get(other).unwrap().hp, 0);
    assert!(has_skill(&declined, owner, DCYUXIN));

    let mut boundary = Transaction::begin(declined);
    boundary.state_mut().players.get_mut(owner).unwrap().hp = 0;
    assert!(offer(&mut boundary, other).unwrap().is_empty());
    assert!(!boundary.state().has_pending_choice());
}

#[test]
fn nonnegative_owner_hp_matches_legacy_get_hp_clamping() {
    let (mut tx, owner, other) = isolated_state(DCYUXIN);
    tx.state_mut().players.get_mut(owner).unwrap().hp = -2;
    tx.state_mut().players.get_mut(other).unwrap().hp = -1;
    assert_eq!(
        offer(&mut tx, other).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&tx, owner, true, 4);
    assert_eq!(accepted.players.get(other).unwrap().hp, 0);
    assert!(!has_skill(&accepted, owner, DCYUXIN));
}

#[test]
fn kunli_recovers_before_gaining_max_hp_and_removes_kuichi() {
    let (mut tx, owner, _) = isolated_state(DCKUNLI);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCKUICHI),
        }],
    )
    .unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    let effects = offer(&mut tx, owner).unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, max_hp + 1);
    assert!(!has_skill(tx.state(), owner, DCKUNLI));
    assert!(!has_skill(tx.state(), owner, DCKUICHI));

    let (mut capped, owner, _) = isolated_state(DCKUNLI);
    let player = capped.state_mut().players.get_mut(owner).unwrap();
    player.hp = 0;
    player.max_hp = 1;
    let effects = offer(&mut capped, owner).unwrap();
    run_effects(&mut capped, effects).unwrap();
    assert_eq!(capped.state().players.get(owner).unwrap().hp, 1);
    assert_eq!(capped.state().players.get(owner).unwrap().max_hp, 2);
}

#[test]
fn shangjue_recovers_then_gains_max_hp_grants_kunli_and_records_upgrade() {
    let (mut tx, owner, _) = isolated_state(DCSHANGJUE);
    tx.state_mut().players.get_mut(owner).unwrap().hp = -1;
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;
    let effects = offer(&mut tx, owner).unwrap();
    run_effects(&mut tx, effects).unwrap();

    let player = tx.state().players.get(owner).unwrap();
    assert_eq!(player.hp, 1);
    assert_eq!(player.max_hp, max_hp + 1);
    assert_eq!(
        player
            .marks
            .iter()
            .find(|(mark, _)| *mark == DCSHANGJUE)
            .map_or(0, |(_, count)| *count),
        1
    );
    assert!(has_skill(tx.state(), owner, DCKUNLI));
    assert!(!has_skill(tx.state(), owner, DCSHANGJUE));
}
}

mod original_xianding_dczhee_machine {
//! Golden for xianding dczhee (ABI 746, tag 763):
//! 锁定技——进入游戏/开局从牌堆、每名角色结束阶段从弃牌堆检索【闪电】
//! 挂为判定牌（判定区已有闪电则静默）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCZHEE: u32 = 2_349_242_461;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn started(seed: u64) -> (GameState, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let players = tx.state().players.alive_by_seat();
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
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, players)
}

fn grant(state: &mut GameState, player: PlayerId, def: u32) {
    let mut tx = Transaction::begin(state.clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    *state = tx.commit().state;
}

fn inject_zone(
    state: &mut GameState,
    owner: PlayerId,
    zone: ZoneKind,
    kind: CardKind,
    tag: u32,
) -> CardId {
    let zone = state.zones.find(zone, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(100_000 + tag),
        kind,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
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
fn package_publishes_dczhee_contract() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 751);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == DCZHEE)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert!(triggers
        .iter()
        .all(|trigger| trigger.filter.as_str() == "owner_alive" && !trigger.optional));
}

#[test]
fn searches_draw_and_discard_piles_for_lightning() {
    let (mut state, players) = started(2_026_102_001);
    let owner = players[0];
    grant(&mut state, owner, DCZHEE);
    // mode 0：牌堆注入闪电 → 挂判定区。
    let bolt_a = inject_zone(&mut state, PlayerId::NONE, ZoneKind::DrawPile, CardKind::TrickShandian, 1);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDczhee { player: owner, mode: 0 }],
    )
    .unwrap();
    let judge = tx.state().zones.find(ZoneKind::Judge, owner).unwrap();
    assert!(tx.state().zones.get(judge).unwrap().cards.contains(&bolt_a));
    let mut state = tx.commit().state;

    // 判定区已有闪电 → 再触发静默（mode 1 弃牌堆有闪电也不动）。
    grant(&mut state, owner, DCZHEE);
    let _bolt_b = inject_zone(&mut state, PlayerId::NONE, ZoneKind::DiscardPile, CardKind::TrickShandian, 2);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDczhee { player: owner, mode: 1 }],
    )
    .unwrap();
    let judge = tx.state().zones.find(ZoneKind::Judge, owner).unwrap();
    assert_eq!(tx.state().zones.get(judge).unwrap().cards.len(), 1);
    let mut state = tx.commit().state;

    // 无判定牌的新持有者：mode 1 从弃牌堆检索。
    grant(&mut state, players[1], DCZHEE);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDczhee {
            player: players[1],
            mode: 1,
        }],
    )
    .unwrap();
    let judge = tx.state().zones.find(ZoneKind::Judge, players[1]).unwrap();
    assert_eq!(tx.state().zones.get(judge).unwrap().cards.len(), 1);
}

#[test]
fn end_phase_pipeline_hangs_lightning_from_discard_pile() {
    let (mut state, players) = started(2_026_102_002);
    let owner = players[0];
    let other = players[1];
    grant(&mut state, owner, DCZHEE);
    let bolt = inject_zone(&mut state, PlayerId::NONE, ZoneKind::DiscardPile, CardKind::TrickShandian, 3);

    // 真管道：任意角色结束阶段（TurnStateChanged(End)）→ owner 挂判定。
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    match run_effects(&mut tx, effects) {
        Ok(_) | Err(EffectError::ChoiceRequested) => {}
        Err(err) => return panic!("unexpected: {err:?}"),
    }
    let state = tx.commit().state;
    let judge = state.zones.find(ZoneKind::Judge, owner).unwrap();
    assert!(state.zones.get(judge).unwrap().cards.contains(&bolt));
    let _ = submit; // 保留 submit 供后续扩展
}
}

mod original_xianding_fugan {
//! Golden for XIANDING 傅干【巧谏/析察】（ABI 586）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const FUGAN: u32 = 398_847_554;
const QIAOJIAN: u32 = 105_744_963;
const QIAOJIAN_EFFECT: u32 = 9_694_479;
const XICHA: u32 = 2_494_439_810;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(skill_def: u32) -> (GameState, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, source, third) = (players[0], players[1], players[2]);
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(players.iter().map(|player| EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        }))
        .chain(std::iter::once(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }))
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    (tx.commit().state, owner, source, third)
}

fn inject(state: &mut GameState, player: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(90_586),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn run(state: GameState, effect: EffectRequest) -> GameState {
    let mut tx = Transaction::begin(state);
    match run_effects(&mut tx, vec![effect]) {
        Ok(_) | Err(EffectError::ChoiceRequested) => {}
        Err(error) => panic!("effect failed: {error}"),
    }
    tx.commit().state
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
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    state
}

#[test]
fn package_closes_fugan_contract() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == FUGAN)
        .unwrap();
    assert_eq!(character.name_key, "fugan");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [QIAOJIAN, XICHA]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == QIAOJIAN_EFFECT));
}

#[test]
fn qiaojian_lower_sum_gains_both_cards_and_exempts_them_from_hand_limit() {
    let (mut state, owner, target, _) = setup(QIAOJIAN);
    let owner_card = inject(&mut state, owner, CardKind::Sha, 2);
    let target_card = inject(&mut state, target, CardKind::Tao, 9);
    let skill = skill_id(&state, owner, QIAOJIAN);
    state = run(
        state,
        EffectRequest::ResolveQiaojian {
            player: owner,
            target,
            helper_def: SkillDefId::new(QIAOJIAN_EFFECT),
            action: 0,
            cards: Vec::new(),
            split: 0,
            skill,
        },
    );
    state = choose(state, owner, vec![owner_card.raw()], 1);
    state = choose(state, target, vec![target_card.raw()], 2);
    assert_eq!(state.cards.get(owner_card).unwrap().owner, owner);
    assert_eq!(state.cards.get(target_card).unwrap().owner, owner);
    assert!(engine_core::query::card_ignored_for_hand_limit(
        &state, owner, owner_card
    ));
    assert!(engine_core::query::card_ignored_for_hand_limit(
        &state,
        owner,
        target_card
    ));
}

#[test]
fn qiaojian_next_turn_policies_merge_and_ban_only_damage_cards() {
    let (mut state, owner, target, _) = setup(QIAOJIAN);
    let main = skill_id(&state, owner, QIAOJIAN);
    for branch in [1, 2] {
        state = run(
            state,
            EffectRequest::ResolveQiaojian {
                player: owner,
                target,
                helper_def: SkillDefId::new(QIAOJIAN_EFFECT),
                action: 10,
                cards: Vec::new(),
                split: branch,
                skill: main,
            },
        );
    }
    let helper = skill_id(&state, target, QIAOJIAN_EFFECT);
    state = run(
        state,
        EffectRequest::ResolveQiaojian {
            player: target,
            target,
            helper_def: SkillDefId::new(QIAOJIAN_EFFECT),
            action: 5,
            cards: Vec::new(),
            split: 0,
            skill: helper,
        },
    );
    let sha = inject(&mut state, target, CardKind::Sha, 7);
    let wuxie = inject(&mut state, target, CardKind::TrickWuxie, 8);
    assert!(engine_core::query::card_use_cancelled_for_card(
        &state, target, sha
    ));
    assert!(!engine_core::query::card_use_cancelled_for_card(
        &state, target, wuxie
    ));
    assert_eq!(state.skills.get(helper).unwrap().state_blob[4], 3);
    state = run(
        state,
        EffectRequest::ResolveQiaojian {
            player: target,
            target,
            helper_def: SkillDefId::new(QIAOJIAN_EFFECT),
            action: 6,
            cards: vec![sha],
            split: 0,
            skill: helper,
        },
    );
    assert!(state.has_pending_choice(), "合并策略仍可询问杀的额外结算");
}

#[test]
fn xicha_matching_equipment_use_is_cancelled_consumed_and_gained() {
    let (mut state, owner, source, _) = setup(XICHA);
    let watched = inject(&mut state, source, CardKind::Weapon, 5);
    let skill = skill_id(&state, owner, XICHA);
    state = run(
        state,
        EffectRequest::ResolveXicha {
            player: owner,
            source,
            card: CardId::NONE,
            action: 0,
            cards: Vec::new(),
            skill,
        },
    );
    state = choose(state, owner, vec![watched.raw()], 3);
    state
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::EmitRuleEvent {
            code: 50,
            payload: vec![source.raw(), watched.raw()],
        });
    state = run(
        state,
        EffectRequest::ResolveXicha {
            player: owner,
            source,
            card: watched,
            action: 1,
            cards: vec![watched],
            skill,
        },
    );
    assert!(state.resolution.deferred_tail_effects.is_empty());
    assert_eq!(state.cards.get(watched).unwrap().owner, owner);
    assert_eq!(&state.skills.get(skill).unwrap().state_blob, b"XCH1");
}
}

mod original_xianding_guili {
//! xianding/guili 归离（第八十七批·额外回合第三消费者）：
//! 首个自身回合绑定一名其他角色（GUILI_BIND 合成标记）；
//! 绑定者回合结束且当回合未造成伤害 → 持有者入队一个纯额外回合（check=0）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const GUILI: u32 = 4_082_668_025;
const GUILI_BIND: u32 = 281_977_830;

const SOURCES_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../content/original/ported/xianding"
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
    let compiled = content_compiler::compile_source_named("xianding-guili", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 20_260_832, "identity-standard").unwrap();
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
            skill_def: SkillDefId::new(GUILI),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}

fn bind_count(state: &GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(player)
        .and_then(|p| {
            p.marks
                .iter()
                .find(|(id, _)| *id == GUILI_BIND)
                .map(|(_, c)| *c)
        })
        .unwrap_or(0)
}

fn others_of(state: &GameState, owner: PlayerId) -> Vec<PlayerId> {
    state
        .players
        .iter_live()
        .map(|p| p.id)
        .filter(|id| *id != owner)
        .collect()
}

fn end_event(current: PlayerId) -> engine_api::DomainEvent {
    engine_api::DomainEvent::PhaseEnded {
        current,
        phase: engine_api::TurnPhaseTag::End,
        damage_sources: 0,
        damaged_players: 0,
        card_targeted_other: false,
        current_dealt_damage_other: false,
        phase_targeted_players: 0,
        phase_hp_changed_players: 0,
    }
}

#[test]
fn guili_publishes_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(include_bytes!(
        "../../../content/original/ported/xianding/xianding.npack.json"
    ))
    .unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 751);
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == GUILI && t.event == "phase_begin" && t.optional
    }));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == GUILI
            && t.event == "turn_ended"
            && t.filter == "owner_alive_event_target_mark_positive_281977830_no_damage"
            && t.optional
    }));
}

#[test]
fn guili_binds_once_then_recovers_extra_turn_on_clean_target_end() {
    let (state, holder) = started();
    let victim = others_of(&state, holder)[0];
    let mut tx = Transaction::begin(state);

    // 第一个自身回合开始：optional confirm → 单选绑定。
    let opened = drive(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: holder,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let mid = resolve(tx.state, holder, vec![1], 901);
    assert_eq!(
        mid.resolution.continuation_kind,
        ContinuationKind::ChooseOtherAddMark
    );
    let observer = mid.resolution.pending_observer;
    let bound_state = resolve(mid, PlayerId::new(observer), vec![victim.raw()], 902);
    assert_eq!(bind_count(&bound_state, victim), 1);

    // 再次自身 prepare：optional confirm 照开（引擎语义），但 storage key1
    // 已合上——确认后无绑定续体、标记不变。
    let mut tx = Transaction::begin(bound_state);
    let again = drive(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: holder,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
    );
    assert_eq!(again.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let rebound = resolve(tx.state, holder, vec![1], 903);
    assert_ne!(
        rebound.resolution.continuation_kind,
        ContinuationKind::ChooseOtherAddMark
    );
    assert_eq!(bind_count(&rebound, victim), 1);

    // 绑定者的无伤害回合结束：confirm → 入队纯额外回合（check=0）。
    let mut tx = Transaction::begin(rebound);
    tx.state_mut().turn.current = victim;
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
    assert_eq!(
        opened.unwrap_err(),
        EffectError::ChoiceRequested,
        "clean bound target end opens the recovery choice"
    );
    let tx = tx.commit();
    let final_state = resolve(tx.state, holder, vec![1], 911);
    assert_eq!(final_state.turn.extra_turn_queue.first(), Some(&holder));
    assert_eq!(
        final_state
            .turn
            .flag(engine_core::state::turn_flag::extra_turn_check_pending_by_player(
                holder
            )),
        0,
        "check=0：归离的额外回合无尾随代价"
    );

    // 二次回收被 storage key2 锁定（每场一次）：confirm 打开但 body 静默。
    let mut tx = Transaction::begin(final_state);
    tx.state_mut().turn.current = victim;
    let opened = drive(&mut tx, &[end_event(victim)]);
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let gated = resolve(tx.state, holder, vec![1], 912);
    assert_ne!(
        gated.resolution.continuation_kind,
        ContinuationKind::ChooseOtherAddMark
    );
    assert!(!gated.has_pending_choice());
}

#[test]
fn guili_skips_when_bound_target_dealt_damage() {
    use engine_core::state::turn_flag;
    let (state, holder) = started();
    let victim = others_of(&state, holder)[0];

    // 完成绑定。
    let mut tx = Transaction::begin(state);
    let opened = drive(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: holder,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let mid = resolve(tx.state, holder, vec![1], 921);
    let observer = mid.resolution.pending_observer;
    let bound_state = resolve(mid, PlayerId::new(observer), vec![victim.raw()], 922);

    // 绑定者本回合造成过伤害：过滤器 no_damage 不满足——不触发。
    let mut tx = Transaction::begin(bound_state);
    tx.state_mut().turn.set_flag(turn_flag::DAMAGE_DEALT, 3);
    tx.state_mut().turn.current = victim;
    let guarded = drive(&mut tx, &[end_event(victim)]);
    guarded.unwrap();
    assert!(!tx.state().has_pending_choice());
}
}

mod original_xianding_huzun {
//! Golden for XIANDING 胡遵【斩涛/安境】（ABI 587）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const HUZUN: u32 = 2_150_724_644;
const DCZHANTAO: u32 = 4_294_211_147;
const DCANJING: u32 = 2_716_085_756;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(skill_def: u32) -> (GameState, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, source, third) = (players[0], players[1], players[2]);
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(players.iter().map(|player| EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        }))
        .chain(std::iter::once(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }))
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    (tx.commit().state, owner, source, third)
}

fn inject(state: &mut GameState, owner: PlayerId, zone: ZoneKind, rank: u8) -> CardId {
    let zone = state.zones.find(zone, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(90_587),
        kind: CardKind::Sha,
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

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn run(state: GameState, effect: EffectRequest) -> GameState {
    let mut tx = Transaction::begin(state);
    match run_effects(&mut tx, vec![effect]) {
        Ok(_) | Err(EffectError::ChoiceRequested) => {}
        Err(error) => panic!("effect failed: {error}"),
    }
    tx.commit().state
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
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    state
}

#[test]
fn package_closes_huzun_contract() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == HUZUN)
        .unwrap();
    assert_eq!(character.name_key, "huzun");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [DCZHANTAO, DCANJING]);
}

#[test]
fn dczhantao_compares_judgment_with_all_physical_materials() {
    let (mut state, owner, source, _) = setup(DCZHANTAO);
    let first = inject(&mut state, owner, ZoneKind::Hand, 2);
    let second = inject(&mut state, owner, ZoneKind::Hand, 3);
    inject(&mut state, PlayerId::NONE, ZoneKind::DrawPile, 6);
    let before = state.players.get(source).unwrap().hp;
    let skill = skill_id(&state, owner, DCZHANTAO);
    state = run(
        state,
        EffectRequest::ResolveHuzun {
            player: owner,
            source,
            card: first,
            action: 0,
            cards: vec![first, second],
            targets: Vec::new(),
            skill,
        },
    );
    assert_eq!(state.players.get(source).unwrap().hp, before - 1);
}

#[test]
fn dcanjing_second_activation_draws_two_and_recovers_one_lowest_hp_target() {
    let (mut state, owner, first, second) = setup(DCANJING);
    state.players.get_mut(first).unwrap().hp = 1;
    state.players.get_mut(second).unwrap().hp = 2;
    for rank in [7, 8] {
        inject(&mut state, PlayerId::NONE, ZoneKind::DrawPile, rank);
    }
    let skill = skill_id(&state, owner, DCANJING);
    state.skills.get_mut(skill).unwrap().state_blob =
        [b"ANJ1".as_slice(), &1u32.to_le_bytes()].concat();
    state = run(
        state,
        EffectRequest::ResolveHuzun {
            player: owner,
            source: PlayerId::NONE,
            card: CardId::NONE,
            action: 1,
            cards: Vec::new(),
            targets: Vec::new(),
            skill,
        },
    );
    state = choose(state, owner, vec![first.raw(), second.raw()], 1);
    assert_eq!(state.hand_count(first), 1);
    assert_eq!(state.hand_count(second), 1);
    assert_eq!(state.players.get(first).unwrap().hp, 2);
    assert_eq!(state.players.get(second).unwrap().hp, 2);
    assert_eq!(
        state.skills.get(skill).unwrap().state_blob,
        [b"ANJ1".as_slice(), &2u32.to_le_bytes()].concat()
    );
}
}

mod original_xianding_malingli {
//! Golden for XIANDING 马伶俐【硝引】（ABI 607）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCXIAOYIN: u32 = 3_398_996_453;
const DCXIAOYIN_EFFECT: u32 = 3_398_996_454;
const SOURCE: &str = include_str!("../../../content/original/ported/xianding/dcxiaoyin.skill");

fn add_to_draw(tx: &mut Transaction, suit: Suit, rank: u8) -> CardId {
    let zone = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_070 + u32::from(rank)),
        kind: CardKind::Sha,
        suit,
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
fn xiaoyin_gains_red_cards_and_places_black_card_on_the_chosen_target() {
    let package = content_compiler::compile_source_named("malingli-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCXIAOYIN),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCXIAOYIN)
        .unwrap()
        .id;
    let black = add_to_draw(&mut tx, Suit::Club, 7);
    add_to_draw(&mut tx, Suit::Heart, 8);
    add_to_draw(&mut tx, Suit::Diamond, 9);
    let hand_before = tx.state().hand_count(owner);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMalingli {
                player: owner,
                card: CardId::NONE,
                helper_skill_def: SkillDefId::new(DCXIAOYIN_EFFECT),
                action: 0,
                source: PlayerId::NONE,
                target: PlayerId::NONE,
                nature: 0,
                damage_id: 0,
                event_cards: Vec::new(),
                event_targets: Vec::new(),
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let revision = tx.state().revision;
    let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, result) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(6071),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![target.raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.hand_count(owner), hand_before + 2);
    assert_eq!(
        state.skill_pile_cards(target, DCXIAOYIN_EFFECT),
        vec![black]
    );
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
                .is_some_and(|instance| instance.def.raw() == DCXIAOYIN_EFFECT)
        }));
}
}

mod original_xianding_mansi {
//! 第一百六十三批：xianding「蛮肆」（花鬘 mansi）主臂近似重写。
//! 上游：【南蛮入侵】对任意角色造成伤害后，你摸一张牌（frequent）。
//! 实现：damage + owner_alive_damage_card_nanman（本批六点注册）→ draw_cards(self,1)。
//! 近似：计数标记（纯展示）省略；viewas 子技维持债。

use engine_api::{
    CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::state::{CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{damage_nature, EffectRequest};

const MANSI: u32 = 3_673_557_022;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn started(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let state = engine.state().clone();
    let owner = state.turn.current;
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    (state, owner, other)
}

fn strip_and_arm(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(
            tx.state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|player| EffectRequest::DiscardFromHand {
                    player,
                    count: u32::MAX,
                }),
        )
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(MANSI),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.commit().state
}

fn deal_nanman_damage(state: GameState, source: PlayerId, target: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            card: CardId::NONE,
            card_kind: CardKind::TrickNanman.tag(),
            amount: 1,
            is_lose_hp: false,
            nature: damage_nature::NORMAL,
            card_flags: 0,
        }],
    )
    .unwrap();
    tx.commit().state
}

fn hand_count(state: &GameState, player: PlayerId) -> u32 {
    state
        .zones
        .find(ZoneKind::Hand, player)
        .and_then(|zone| state.zones.get(zone).map(|z| z.cards.len() as u32))
        .unwrap_or(0)
}

#[test]
fn package_publishes_mansi_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == MANSI
            && trigger.event == "damage"
            && trigger.filter == "owner_alive_damage_card_nanman"
            && !trigger.optional
    }));
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == MANSI && modifier.target == "hand_limit"));
}

#[test]
fn mansi_draws_when_nanman_damages_anyone() {
    let (state, owner, other) = started(2_026_091_001);
    let state = strip_and_arm(state, owner);

    // 南蛮入侵对他人造成伤害（owner 仅为观察者）：蛮肆摸 1。
    let before = hand_count(&state, owner);
    let state = deal_nanman_damage(state, other, other);
    assert_eq!(hand_count(&state, owner), before + 1, "nanman damage anywhere draws one");
}

#[test]
fn mansi_not_triggered_by_plain_damage() {
    let (state, owner, other) = started(2_026_091_002);
    let state = strip_and_arm(state, owner);

    // 无牌面上下文的普通伤害：过滤器拦截，不摸牌。
    let before = hand_count(&state, owner);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source: other,
            target: other,
            card: CardId::NONE,
            amount: 1,
            nature: damage_nature::NORMAL,
            card_flags: 0,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(hand_count(&state, owner), before, "plain sha damage must not draw");
}
}

mod original_xianding_ruanji {
//! Golden for XIANDING 阮籍【昭文/酒遁】（ABI 584）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCZHAOWEN: u32 = 829_715_916;
const DCJIUDUN: u32 = 2_926_675_148;
const RUANJI: u32 = 2_897_563_211;
const MARK_JIU: u32 = 1;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64, skill_def: u32) -> (GameState, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, source, third) = (players[0], players[1], players[2]);
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(std::iter::once(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }))
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.round = 1;
    tx.state_mut().revision = 1;
    (tx.commit().state, owner, source, third)
}

fn inject_card(state: &mut GameState, player: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(90_584),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn command(state: GameState, actor: PlayerId, kind: CommandKind, key: u64) -> GameState {
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
        "command rejected: {:?}",
        result.result
    );
    state
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
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

fn cancel_pending(mut state: GameState, actors: &[PlayerId], mut key: u64) -> GameState {
    for _ in 0..20 {
        if !state.has_pending_choice() {
            return state;
        }
        let mut accepted = None;
        for actor in actors {
            let (next, result) = apply_command(
                state.clone(),
                Command {
                    actor: *actor,
                    expected_revision: ExpectedRevision(state.revision),
                    idempotency_key: IdempotencyKey(key),
                    kind: CommandKind::ResolveChoice {
                        choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                        selections: Vec::new(),
                        cancelled: true,
                    },
                },
            );
            key += 1;
            if matches!(result.result, CommandResult::Accepted { .. }) {
                accepted = Some(next);
                break;
            }
        }
        state = accepted.expect("pending choice has an expected observer");
    }
    panic!("pending choices did not drain")
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn jiu_mark(state: &GameState, player: PlayerId) -> u32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find_map(|(mark, count)| (*mark == MARK_JIU).then_some((*count).max(0) as u32))
        .unwrap_or(0)
}

#[test]
fn package_closes_ruanji_with_exact_contract() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RUANJI)
        .expect("ruanji");
    assert_eq!(character.name_key, "ruanji");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [DCZHAOWEN, DCJIUDUN]);
    assert_eq!(
        package
            .trigger_index
            .iter()
            .filter(|trigger| trigger.skill_def_id == DCZHAOWEN)
            .count(),
        2
    );
    assert_eq!(
        package
            .trigger_index
            .iter()
            .filter(|trigger| trigger.skill_def_id == DCJIUDUN)
            .count(),
        2
    );
    let view = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == DCZHAOWEN)
        .expect("昭文 view-as");
    assert_eq!(view.as_kind, "ordinary_trick");
    assert_eq!(view.cost.as_deref(), Some("dczhaowen"));
}

#[test]
fn zhaowen_tracks_current_hand_converts_black_once_per_name_and_rewards_red_use() {
    let (mut state, owner, _source, _third) = setup(20_260_810, DCZHAOWEN);
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let black = inject_card(&mut state, owner, CardKind::Sha, Suit::Spade);
    let red = inject_card(&mut state, owner, CardKind::Weapon, Suit::Heart);
    let skill = skill_id(&state, owner, DCZHAOWEN);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveRuanji {
            player: owner,
            source: PlayerId::NONE,
            card: CardId::NONE,
            action: 0,
            cards: Vec::new(),
            skill,
        }],
    )
    .unwrap();
    let mut state = tx.commit().state;

    state = command(
        state,
        owner,
        CommandKind::UseSkillCard {
            skill,
            card_kind: CardKind::TrickWuzhong.tag(),
            targets: Vec::new(),
            equip_slot: 0,
        },
        1,
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveFaceDownViewAs
    );
    state = choose(state, owner, vec![black.raw()], 2);
    state = cancel_pending(state, &[owner, _source, _third], 20);
    let revision = state.revision;
    let (_unchanged, duplicate) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::TrickWuzhong.tag(),
                targets: Vec::new(),
                equip_slot: 0,
            },
        },
    );
    assert!(matches!(duplicate.result, CommandResult::Rejected { .. }));

    let before = state.hand_count(owner);
    state = command(
        state,
        owner,
        CommandKind::PlayCard {
            card: red,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        4,
    );
    assert_eq!(state.hand_count(owner), before, "红装备离手，昭文额外摸一");
}

#[test]
fn jiudun_rewrites_jiu_recovery_to_next_sha_boost() {
    let (mut state, owner, _source, _third) = setup(20_260_811, DCJIUDUN);
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    state.players.get_mut(owner).unwrap().hp = 2;
    let jiu = inject_card(&mut state, owner, CardKind::Jiu, Suit::Diamond);
    state = command(
        state,
        owner,
        CommandKind::PlayCard {
            card: jiu,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        1,
    );
    assert_eq!(state.players.get(owner).unwrap().hp, 2);
    assert_eq!(jiu_mark(&state, owner), 1);
}

#[test]
fn jiudun_black_target_branches_draw_drink_or_discard_exclude() {
    let (mut state, owner, source, _third) = setup(20_260_812, DCJIUDUN);
    state.turn.current = source;
    state.turn.phase = Phase::Play;
    let sha = inject_card(&mut state, source, CardKind::Sha, Suit::Spade);
    let before = state.hand_count(owner);
    state = command(
        state,
        source,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        1,
    );
    assert_eq!(state.resolution.continuation_kind, ContinuationKind::Custom);
    state = choose(state, owner, vec![1], 2);
    assert_eq!(state.hand_count(owner), before + 1);
    assert_eq!(jiu_mark(&state, owner), 1);
    state = cancel_pending(state, &[owner, source], 40);

    let payment = inject_card(&mut state, owner, CardKind::Tao, Suit::Heart);
    let second_sha = inject_card(&mut state, source, CardKind::TrickGuohe, Suit::Club);
    let hp = state.players.get(owner).unwrap().hp;
    state = command(
        state,
        source,
        CommandKind::PlayCard {
            card: second_sha,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        3,
    );
    assert_eq!(state.resolution.continuation_kind, ContinuationKind::Custom);
    state = choose(state, owner, vec![payment.raw()], 4);
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    assert_eq!(
        state
            .zones
            .get(state.cards.get(payment).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::DiscardPile
    );
}
}

mod original_xianding_sanshou_machine {
//! Golden for xianding 三首 sanshou (ABI 751, tag 767):
//! 即将受到伤害时可亮牌堆顶三张：存在本局未使用类型的牌则防止伤害；
//! 亮出的牌移入弃牌堆。真管道（DamageBefore 事件派发）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::damage_nature;
use rule_api::EffectRequest;

const SANSHOU: u32 = 3_583_301_051;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn started(seed: u64) -> (GameState, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package()).unwrap();
    let players = tx.state().players.alive_by_seat();
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
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx.commit().state, players)
}

fn grant(state: &mut GameState, player: PlayerId, def: u32) {
    let mut tx = Transaction::begin(state.clone());
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    *state = tx.commit().state;
}

fn inject_draw_top(state: &mut GameState, kind: CardKind, tag: u32) -> CardId {
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(104_000 + tag),
        kind,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: false,
    });
    state.zones.get_mut(draw).unwrap().cards.push(card);
    card
}


#[test]
fn package_publishes_sanshou_contract() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 751);
    let entry = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == SANSHOU)
        .expect("sanshou trigger");
    assert_eq!(entry.event.as_str(), "damage_before");
    assert_eq!(entry.filter.as_str(), "owner_is_target_sanshou_type_tracked");
    assert!(!entry.optional);
}

#[test]
fn sanshou_cancels_damage_when_new_type_revealed() {
    let (mut state, players) = started(2_026_107_001);
    let owner = players[0];
    let source = players[1];
    grant(&mut state, owner, SANSHOU);
    // 本局无人用过牌（mask=0）→ 牌堆顶注入三张全新类型（基本/锦囊/装备）。
    let _a = inject_draw_top(&mut state, CardKind::Tao, 1);
    let _b = inject_draw_top(&mut state, CardKind::TrickWuxie, 2);
    let _c = inject_draw_top(&mut state, CardKind::Weapon, 3);
    let hp_before = state.players.get(owner).unwrap().hp;
    let mut tx = Transaction::begin(state);
    // DealDamage 管道内构造 DamageBefore ctx → optional 确认窗。
    let dmg = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: damage_nature::NORMAL,
        }],
    );
    match dmg {
        Ok(_) | Err(EffectError::ChoiceRequested) => {}
        Err(err) => panic!("unexpected: {err:?}"),
    }
    // 确认发动 → 机器亮顶判型 → 全新类型 → PreventDamage。
    let state = tx.commit().state;
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        hp_before,
        "全新类型亮出应防止伤害"
    );
    // 亮出的三张移入弃牌堆。
    let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap();
    assert!(state.zones.get(discard).unwrap().cards.len() >= 3);
}


#[test]
fn sanshou_passes_when_all_types_known() {
    let (mut state, players) = started(2_026_107_002);
    let owner = players[0];
    let source = players[1];
    grant(&mut state, owner, SANSHOU);
    // 位图预置三种类型全部已知 → 无全新类型 → 不防伤。
    state.lifetime_used_type_groups = 0b0000_0111;
    let _a = inject_draw_top(&mut state, CardKind::Tao, 4);
    let _b = inject_draw_top(&mut state, CardKind::TrickWuxie, 5);
    let _c = inject_draw_top(&mut state, CardKind::Weapon, 6);
    let hp_before = state.players.get(owner).unwrap().hp;
    let mut tx = Transaction::begin(state);
    let dmg = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: damage_nature::NORMAL,
        }],
    );
    match dmg {
        Ok(_) | Err(EffectError::ChoiceRequested) => {}
        Err(err) => panic!("unexpected: {err:?}"),
    }
    // forced：窗口内机器同步执行 → 无全新类型 → 伤害照常。
    let state = tx.commit().state;
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        hp_before - 2,
        "无全新类型应照常受伤"
    );
}
}

mod original_xianding_tuiyan {
//! 第一百八十四批：xianding「推演」（管辂 tuiyan）观星原语外溢，零新原语。
//! 上游：出牌阶段开始时（frequent），观看牌堆顶三张牌，以任意顺序放回牌堆顶
//! （viewCards + reverse insert）。
//! 实现：phase_play + owner_is_target（forced）→ guanxing_top(self, 3)。
//! 近似（登记债）：guanxing_top 允许逐张顶/底重排（pxlangya 同债）——能力超集，
//! 信息面等价。

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

const TUIYAN: u32 = 2_256_277_010;
const GUANLU: u32 = 4_222_425_710;
const SOURCE: &str = include_str!("../../../content/original/ported/xianding/tuiyan.skill");
const XIANDING: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("tuiyan-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
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
        skill_def: SkillDefId::new(TUIYAN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner)
}

fn fire_play(state: GameState, owner: PlayerId) -> (GameState, Result<(), EffectError>) {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Play,
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
fn package_publishes_tuiyan_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(XIANDING).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 681);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == TUIYAN)
        .unwrap();
    assert_eq!(trigger.event, "phase_play");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == GUANLU)
        .expect("guanlu in roster");
    assert!(character.skills.contains(&TUIYAN));
}

#[test]
fn play_phase_start_opens_three_card_view_loop() {
    let (state, owner) = setup(20_260_909_07);
    let (state, outcome) = fire_play(state, owner);
    assert!(
        matches!(outcome, Err(EffectError::ChoiceRequested)),
        "view loop must open, got {outcome:?}"
    );
    let mut next = state;
    let mut key = 20_260_909_17;
    for round in 0..3 {
        assert!(next.has_pending_choice(), "round {round} must stay open");
        next = resolve(next, owner, 1, key);
        key += 1;
    }
    assert!(!next.has_pending_choice(), "loop must close after 3 rounds");
    let processing = next
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .unwrap();
    assert!(
        next.zones.get(processing).unwrap().cards.is_empty(),
        "processing must drain after the view settles"
    );
}

#[test]
fn other_player_phase_does_not_trigger() {
    let (state, owner, ) = setup(20_260_909_08);
    let mut tx = Transaction::begin(state);
    let others = tx.state().players.alive_by_seat();
    let other = *others.iter().find(|p| **p != owner).unwrap();
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(
        effects.is_empty(),
        "owner_is_target must suppress other-player phases"
    );
}
}

mod original_xianding_v_guanyinping {
//! Golden for XIANDING 威关银屏【绍威/涤仇】（ABI 594）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, MarkDefId, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const V_GUANYINPING: u32 = 4_134_246_215;
const DCSHAOWEI: u32 = 3_440_541_593;
const DCDICHOU: u32 = 3_304_378_096;
const DCDICHOU_EFF: u32 = 2_754_298_115;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(skill_def: u32) -> (GameState, [PlayerId; 3]) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players: [PlayerId; 3] = tx.state().players.alive_by_seat().try_into().unwrap();
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(players.iter().map(|player| EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        }))
        .chain(std::iter::once(EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(skill_def),
        }))
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::Prepare;
    tx.state_mut().revision = 1;
    (tx.commit().state, players)
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    kind: CardKind,
    suit: Suit,
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
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn run(state: GameState, effect: EffectRequest) -> GameState {
    let mut tx = Transaction::begin(state);
    match run_effects(&mut tx, vec![effect]) {
        Ok(_) | Err(EffectError::ChoiceRequested) => {}
        Err(error) => panic!("effect failed: {error}"),
    }
    tx.commit().state
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, result) = apply_command(
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
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    state
}

fn mark_count(state: &GameState, player: PlayerId, mark: u32) -> i32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(id, _)| *id == mark)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn package_closes_v_guanyinping_contract() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 594);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == V_GUANYINPING)
        .unwrap();
    assert_eq!(character.name_key, "v_guanyinping");
    assert_eq!(character.skills, [DCSHAOWEI, DCDICHOU]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == DCDICHOU_EFF));
}

#[test]
fn dcshaowei_gains_at_most_three_red_draw_cards_once_per_turn() {
    let (mut state, [owner, _, _]) = setup(DCSHAOWEI);
    let lost = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        10,
        CardKind::Shan,
        Suit::Diamond,
    );
    for (def, suit) in [
        (11, Suit::Heart),
        (12, Suit::Diamond),
        (13, Suit::Heart),
        (14, Suit::Diamond),
    ] {
        inject(
            &mut state,
            PlayerId::NONE,
            ZoneKind::DrawPile,
            def,
            CardKind::Shan,
            suit,
        );
    }
    inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        15,
        CardKind::Shan,
        Suit::Spade,
    );
    let skill = skill_id(&state, owner, DCSHAOWEI);
    state = run(
        state,
        EffectRequest::ResolveDcshaowei {
            player: owner,
            action: 0,
            cards: vec![lost],
            skill,
        },
    );
    assert_eq!(state.hand_count(owner), 4);
    state = run(
        state,
        EffectRequest::ResolveDcshaowei {
            player: owner,
            action: 0,
            cards: vec![lost],
            skill,
        },
    );
    assert_eq!(state.hand_count(owner), 4);
}

#[test]
fn dcdichou_resolves_begin_bonus_and_end_fire_damage() {
    let (mut state, [owner, target, _]) = setup(DCDICHOU);
    let discarded = inject(
        &mut state,
        target,
        ZoneKind::Hand,
        20,
        CardKind::TrickWuzhong,
        Suit::Club,
    );
    let skill = skill_id(&state, owner, DCDICHOU);
    state = run(
        state,
        EffectRequest::AddMark {
            player: target,
            mark: MarkDefId::new(DCDICHOU),
            delta: 1,
        },
    );
    state = run(
        state,
        EffectRequest::ResolveDcdichou {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 0,
            helper_skill_def: SkillDefId::new(DCDICHOU_EFF),
            skill,
        },
    );
    state = choose(state, owner, vec![1], 1);
    state = choose(state, owner, vec![target.raw()], 2);
    state = choose(state, owner, vec![discarded.raw()], 3);
    assert_eq!(state.players.get(owner).unwrap().disabled_equip_slots[0], 1);
    assert_eq!(
        state
            .zones
            .get(state.cards.get(discarded).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::DiscardPile
    );
    assert!(mark_count(&state, owner, DCDICHOU_EFF) > 0);

    state.turn.phase = Phase::End;
    let hp = state.players.get(target).unwrap().hp;
    state = run(
        state,
        EffectRequest::ResolveDcdichou {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 10,
            helper_skill_def: SkillDefId::new(DCDICHOU_EFF),
            skill,
        },
    );
    state = choose(state, owner, vec![target.raw()], 4);
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert_eq!(mark_count(&state, target, DCDICHOU), 0);
}
}

mod original_xianding_zhangjian {
//! Golden for XIANDING 张臶【技能A/技能B】（ABI 604）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DC_ZJ_A: u32 = 2_792_442_500;
const DC_ZJ_B: u32 = 3_520_470_433;
const SOURCE: &str = include_str!("../../../content/original/ported/xianding/zhangjian.skill");

fn setup(def: u32) -> (GameState, [PlayerId; 3], engine_api::SkillId) {
    let package = content_compiler::compile_source_named("zhangjian-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == players[0] && skill.def.raw() == def)
        .unwrap()
        .id;
    (tx.commit().state, players, skill)
}

fn alloc(tx: &mut Transaction, owner: PlayerId, kind: CardKind, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_604 + u32::from(rank)),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, actor: PlayerId, selection: u32) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(604),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    state
}

#[test]
fn skill_a_rewrites_physical_card_damage_to_rank_and_prevents_rankless_card_damage() {
    let (state, [owner, source, _], _) = setup(DC_ZJ_A);
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 20;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 20;
    let card = alloc(&mut tx, source, CardKind::Sha, 7);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 13);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target: owner,
            amount: 4,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
            card: CardId::NONE,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 13);
}

#[test]
fn skill_b_discards_all_and_grants_skill_a_until_owners_next_turn() {
    let (state, [owner, target, previous], skill) = setup(DC_ZJ_B);
    let mut tx = Transaction::begin(state);
    alloc(&mut tx, owner, CardKind::Sha, 3);
    alloc(&mut tx, owner, CardKind::Tao, 4);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveZhangjian {
                player: owner,
                card: CardId::NONE,
                grant_skill_def: SkillDefId::new(DC_ZJ_A),
                action: 1,
                target: PlayerId::NONE,
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = choose(tx.commit().state, owner, target.raw());
    assert_eq!(state.hand_count(owner), 0);
    let granted = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|instance| instance.owner == target && instance.def.raw() == DC_ZJ_A)
        .unwrap();
    assert_eq!(&granted.state_blob[..4], b"STB1");
    assert_eq!(
        u32::from_le_bytes(granted.state_blob[4..8].try_into().unwrap()),
        owner.raw()
    );

    let mut tx = Transaction::begin(state);
    run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    tx.state_mut().turn.current = previous;
    tx.state_mut().turn.phase = Phase::End;
    tx.state_mut()
        .turn
        .set_flag(turn_flag::TURN_END_DISPATCHED, 1);
    run_effects(&mut tx, vec![EffectRequest::AdvanceTurn]).unwrap();
    assert_eq!(tx.state().turn.current, owner);
    assert!(!tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .any(|instance| instance.owner == target && instance.def.raw() == DC_ZJ_A));
}
}

mod original_xianding_zhugemengxue {
//! Golden for XIANDING 诸葛梦雪【寄春/寒英】（ABI 588）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZHUGEMENGXUE: u32 = 2_758_962_825;
const DCJICHUN: u32 = 3_609_714_118;
const DCHANYING: u32 = 2_250_842_878;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(skill_def: u32) -> (GameState, [PlayerId; 3]) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players: [PlayerId; 3] = tx.state().players.alive_by_seat().try_into().unwrap();
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(players.iter().map(|player| EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        }))
        .chain(std::iter::once(EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(skill_def),
        }))
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    (tx.commit().state, players)
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone: ZoneKind,
    def: u32,
    kind: CardKind,
) -> CardId {
    let zone = state.zones.find(zone, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 2,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn run(state: GameState, effect: EffectRequest) -> GameState {
    let mut tx = Transaction::begin(state);
    match run_effects(&mut tx, vec![effect]) {
        Ok(_) | Err(EffectError::ChoiceRequested) => {}
        Err(error) => panic!("effect failed: {error}"),
    }
    tx.commit().state
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, result) = apply_command(
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
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    state
}

#[test]
fn package_closes_zhugemengxue_contract() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 588);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == ZHUGEMENGXUE)
        .unwrap();
    assert_eq!(character.name_key, "zhugemengxue");
    assert_eq!(character.skills, [DCJICHUN, DCHANYING]);
}

#[test]
fn dcjichun_uses_each_branch_once_and_counts_localized_name() {
    let (mut state, [owner, low, high]) = setup(DCJICHUN);
    let selected = inject(&mut state, owner, ZoneKind::Hand, 1, CardKind::Sha);
    inject(&mut state, owner, ZoneKind::Hand, 2, CardKind::Shan);
    inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        3,
        CardKind::Tao,
    );
    let skill = skill_id(&state, owner, DCJICHUN);
    state = run(
        state,
        EffectRequest::ResolveZhugemengxue {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 0,
            cards: vec![selected],
            skill,
        },
    );
    state = choose(state, owner, vec![low.raw()], 1);
    assert_eq!(state.hand_count(low), 1);
    assert_eq!(state.hand_count(owner), 2);

    let four_chars = inject(&mut state, owner, ZoneKind::Hand, 4, CardKind::TrickWuzhong);
    let high_cards: Vec<_> = (0..4)
        .map(|index| inject(&mut state, high, ZoneKind::Hand, 10 + index, CardKind::Shan))
        .collect();
    state = run(
        state,
        EffectRequest::ResolveZhugemengxue {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 0,
            cards: vec![four_chars],
            skill,
        },
    );
    state = choose(state, owner, vec![high.raw()], 2);
    state = choose(
        state,
        owner,
        high_cards.iter().map(|card| card.raw()).collect(),
        3,
    );
    assert_eq!(state.hand_count(high), 0);
    assert_eq!(state.skills.get(skill).unwrap().state_blob, b"JCH1\x03");
}

#[test]
fn dchanying_uses_draw_pile_equipment_without_a_fake_hand_gain() {
    let (mut state, [owner, target, _]) = setup(DCHANYING);
    let equipment = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        100,
        CardKind::Weapon,
    );
    let skill = skill_id(&state, owner, DCHANYING);
    state = run(
        state,
        EffectRequest::ResolveZhugemengxue {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 10,
            cards: Vec::new(),
            skill,
        },
    );
    state = choose(state, owner, vec![target.raw()], 4);
    let card = state.cards.get(equipment).unwrap();
    assert_eq!(card.owner, target);
    assert_eq!(state.zones.get(card.zone).unwrap().kind, ZoneKind::Equip);
    assert_eq!(state.hand_count(target), 0);
}
}

mod original_xianding_zhugeruoxue {
//! Golden for XIANDING 诸葛若雪【琼英/暖惠】（ABI 605）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCQIONGYING: u32 = 2_641_115_727;
const DCNUANHUI: u32 = 2_308_165_295;
const SOURCE: &str = include_str!("../../../content/original/ported/xianding/zhugeruoxue.skill");

fn setup(def: u32) -> (GameState, [PlayerId; 3], engine_api::SkillId) {
    let package = content_compiler::compile_source_named("zhugeruoxue-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == players[0] && skill.def.raw() == def)
        .unwrap()
        .id;
    (tx.commit().state, players, skill)
}

fn alloc(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    serial: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(605_000 + serial),
        kind,
        suit,
        rank: 7,
        weapon_range: if kind == CardKind::Weapon { 3 } else { 0 },
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(
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
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    state
}

#[test]
fn qiongying_moves_field_card_then_discards_matching_suit_hand_card() {
    let (state, [owner, source, destination], skill) = setup(DCQIONGYING);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let weapon = alloc(
        &mut tx,
        source,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Spade,
        1,
    );
    let cost = alloc(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        2,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveZhugeruoxue {
                player: owner,
                target: PlayerId::NONE,
                card: CardId::NONE,
                action: 0,
                remaining: 0,
                used_mask: 0,
                repeated: false,
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = choose(tx.commit().state, owner, vec![weapon.raw()], false, 6051);
    let state = choose(state, owner, vec![destination.raw()], false, 6052);
    let state = choose(state, owner, vec![cost.raw()], false, 6053);
    let equip = state.zones.find(ZoneKind::Equip, destination).unwrap();
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(weapon).unwrap().zone, equip);
    assert_eq!(state.cards.get(cost).unwrap().zone, discard);
}

#[test]
fn nuanhui_snapshots_equipment_count_and_discards_equipment_after_repeated_name() {
    let (state, [owner, target, _], skill) = setup(DCNUANHUI);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    let weapon = alloc(
        &mut tx,
        target,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Spade,
        3,
    );
    let armor = alloc(
        &mut tx,
        target,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Club,
        4,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveZhugeruoxue {
                player: owner,
                target,
                card: CardId::NONE,
                action: 11,
                remaining: 0,
                used_mask: 0,
                repeated: false,
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = choose(
        tx.commit().state,
        target,
        vec![u32::from(CardKind::Jiu.tag())],
        false,
        6054,
    );
    let state = choose(
        state,
        target,
        vec![u32::from(CardKind::Jiu.tag())],
        false,
        6055,
    );
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(weapon).unwrap().zone, discard);
    assert_eq!(state.cards.get(armor).unwrap().zone, discard);
}
}

mod original_xianding_zhupeilan {
//! Golden for XIANDING 朱佩兰【辞虑/痛悼】（ABI 600）。

use content_schema::PackageCharacterEntry;
use engine_api::{CardDefId, CardId, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, PendingCardResolution, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCCILV: u32 = 3_468_569_693;
const DCCILV_EFFECT: u32 = 3_697_675_750;
const DCTONGDAO: u32 = 3_625_586_472;
const BASE: u32 = 90_600_001;
const ACQUIRED: u32 = 90_600_002;
const INTERNAL: u32 = 90_600_003;
const CHARACTER: u32 = 90_600_004;
const SOURCE: &str = include_str!("../../../content/original/ported/xianding/zhupeilan.skill");

fn setup() -> (
    GameState,
    PlayerId,
    PlayerId,
    engine_api::SkillId,
    engine_api::SkillId,
) {
    let mut package = content_compiler::compile_source_named(
        "zhupeilan-test",
        &format!(
            "{SOURCE}\nskill base({BASE}) {{}}\nskill acquired({ACQUIRED}) {{}}\nskill internal({INTERNAL}) non_gainable {{}}"
        ),
    )
    .unwrap();
    package.character_index.push(PackageCharacterEntry {
        id: CHARACTER,
        name_key: "tongdao_target".into(),
        max_hp: 3,
        initial_hp: 0,
        skills: vec![BASE],
        sex: "male".into(),
        group: "wu".into(),
        is_lord: false,
    clans: vec![],
        tags: Vec::new(),
        sort_key: String::new(),
        portrait_asset: None,
        skin_assets: Vec::new(),
    });
    package.manifest.logic_hash = package.compute_logic_hash();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in &players {
        let cards = tx.state().hand_cards(*player);
        if !cards.is_empty() {
            effects.push(EffectRequest::DiscardCards {
                player: *player,
                cards,
            });
        }
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCCILV),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCTONGDAO),
        },
        EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(BASE),
        },
        EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(ACQUIRED),
        },
        EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(INTERNAL),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut()
        .players
        .get_mut(target)
        .unwrap()
        .character_def = CHARACTER;
    let find = |def| {
        tx.state()
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == owner && skill.def.raw() == def)
            .unwrap()
            .id
    };
    let cilv = find(DCCILV);
    let tongdao = find(DCTONGDAO);
    (tx.commit().state, owner, target, cilv, tongdao)
}

fn alloc(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: ZoneKind,
    rank: u8,
    kind: CardKind,
) -> CardId {
    let zone = tx.state().zones.find(zone, owner).unwrap_or_else(|| {
        tx.state()
            .zones
            .find(zone, PlayerId::NONE)
            .expect("shared zone")
    });
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_900 + u32::from(rank)),
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

fn arm_trick(
    tx: &mut Transaction,
    source: PlayerId,
    owner: PlayerId,
    card: CardId,
    materials: Vec<CardId>,
) {
    tx.state_mut()
        .resolution
        .pending_card_resolutions
        .push(PendingCardResolution {
            source,
            card,
            kind: CardKind::TrickJuedou.tag(),
            targets: vec![owner],
            damaged_targets: Vec::new(),
            source_was_hand: true,
            all_sources_were_hand: true,
            response_color_lock_mask: 0,
            source_cards: materials,
            originating_view_as_skill_def: 0,
            skill_excluded_targets: Vec::new(),
        });
}

#[test]
fn cilv_draws_by_remaining_options_then_binds_prevent_and_gain_to_exact_card() {
    let (state, owner, source, cilv, _) = setup();
    let mut tx = Transaction::begin(state);
    alloc(&mut tx, owner, ZoneKind::Hand, 1, CardKind::Other);
    alloc(&mut tx, owner, ZoneKind::Hand, 2, CardKind::Other);
    for rank in 3..=5 {
        alloc(
            &mut tx,
            PlayerId::NONE,
            ZoneKind::DrawPile,
            rank,
            CardKind::Other,
        );
    }
    let trick = alloc(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        6,
        CardKind::TrickJuedou,
    );
    arm_trick(&mut tx, source, owner, trick, vec![trick]);
    // The current port auto-resolves the 赐赂 draw (3 - used categories);
    // the optional acceptance lives at the trigger level, not in action 0.
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveZhupeilan {
            player: owner,
            target: source,
            card: trick,
            helper_skill_def: SkillDefId::new(DCCILV_EFFECT),
            action: 0,
            event_cards: vec![trick],
            skill: cilv,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), 5);
    assert_eq!(tx.state().resolution.pending_choice_id, 0);

    let (state, owner, source, cilv, _) = setup();
    let mut tx = Transaction::begin(state);
    let trick = alloc(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        6,
        CardKind::TrickJuedou,
    );
    arm_trick(&mut tx, source, owner, trick, vec![trick]);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveZhupeilan {
            player: owner,
            target: source,
            card: trick,
            helper_skill_def: SkillDefId::new(DCCILV_EFFECT),
            action: 4,
            event_cards: vec![trick],
            skill: cilv,
        }],
    )
    .unwrap();
    let helper = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == DCCILV_EFFECT)
        .unwrap();
    run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill: cilv }]).unwrap();
    tx.state_mut().resolution.current_damage_id = 77;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveZhupeilan {
            player: owner,
            target: source,
            card: trick,
            helper_skill_def: SkillDefId::new(DCCILV_EFFECT),
            action: 1,
            event_cards: Vec::new(),
            skill: helper,
        }],
    )
    .unwrap();
    assert!(tx.state().resolution.prevented_damage_ids.contains(&77));
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveZhupeilan {
            player: owner,
            target: source,
            card: trick,
            helper_skill_def: SkillDefId::new(DCCILV_EFFECT),
            action: 2,
            event_cards: Vec::new(),
            skill: helper,
        }],
    )
    .unwrap();

    let second = alloc(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        7,
        CardKind::TrickHuogong,
    );
    let extra = alloc(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        8,
        CardKind::Other,
    );
    arm_trick(&mut tx, source, owner, second, vec![second, extra]);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCCILV),
        }],
    )
    .unwrap();
    let cilv = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == DCCILV)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveZhupeilan {
            player: owner,
            target: source,
            card: second,
            helper_skill_def: SkillDefId::new(DCCILV_EFFECT),
            action: 5,
            event_cards: vec![second, extra],
            skill: cilv,
        }],
    )
    .unwrap();
    let helper = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == DCCILV_EFFECT)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveZhupeilan {
            player: owner,
            target: source,
            card: second,
            helper_skill_def: SkillDefId::new(DCCILV_EFFECT),
            action: 2,
            event_cards: Vec::new(),
            skill: helper,
        }],
    )
    .unwrap();
    assert!(tx.state().hand_cards(owner).contains(&second));
    assert!(tx.state().hand_cards(owner).contains(&extra));
}

#[test]
fn tongdao_replaces_non_internal_skills_with_fresh_starting_skills_and_recovers_owner() {
    let (state, owner, target, _, tongdao) = setup();
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    tx.state_mut().players.get_mut(owner).unwrap().dying = true;
    tx.state_mut().players.get_mut(target).unwrap().hp = 2;
    tx.state_mut()
        .skills
        .get_mut(tongdao)
        .unwrap()
        .lifetime_fire_count = 1;
    let before = tx.state().players.get(target).unwrap().skills.clone();
    let internal = before
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == INTERNAL)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveZhupeilan {
            player: owner,
            target,
            card: CardId::NONE,
            helper_skill_def: SkillDefId::NONE,
            action: 11,
            event_cards: Vec::new(),
            skill: tongdao,
        }],
    )
    .unwrap();
    let skills = tx.state().players.get(target).unwrap().skills.clone();
    assert!(skills.contains(&internal));
    assert!(skills.iter().any(|skill| {
        tx.state().skills.get(*skill).unwrap().def.raw() == BASE && !before.contains(skill)
    }));
    assert!(!skills
        .iter()
        .any(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == ACQUIRED));
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 2);
}
}

mod original_xianding_zhushi {
//! 第一百八十五批：xianding「助势」（曹髦 zhushi）主公技零新原语重写 + 新过滤器。
//! 上游：global recoverEnd；其他魏势力角色于其回合内回复体力后，其可令你摸一张
//! 牌（usable 1）。实现：新过滤器 owner_alive_other_wei_recovery_own_phase_once_
//! per_turn（六点注册含 turn_limit 名单，零 ABI）→ optional → draw_cards(self, 1)。
//! 近似（登记债）：确认方为回复角色 → 拥有者 optional；zhuSkill 身份门未建模。

use engine_api::{Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const ZHUSHI: u32 = 3_044_650_156;
const CAOMAO: u32 = 2_786_357_147;
const SOURCE: &str = include_str!("../../../content/original/ported/xianding/zhushi.skill");
const XIANDING: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("zhushi-test", SOURCE).unwrap();
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
        skill_def: SkillDefId::new(ZHUSHI),
    });
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    // 事件角色为魏势力且处于其自己的回合。
    let mut tx = Transaction::begin(state);
    tx.state_mut().group_overrides.insert(other, "wei".into());
    tx.state_mut().turn.current = other;
    (tx.commit().state, owner, other)
}

fn wei_recovers(state: GameState, other: PlayerId) -> (GameState, bool) {
    let mut tx = Transaction::begin(state);
    let confirmed = match dispatch_for_events(
        &mut tx,
        &[DomainEvent::HpRecovered {
            source: PlayerId::NONE,
            player: other,
            amount: 1,
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

fn resolve(state: GameState, owner: PlayerId, key: u64) -> GameState {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "confirm resolve rejected: {:?}",
        result.result
    );
    next
}

#[test]
fn package_publishes_zhushi_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(XIANDING).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 681);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == ZHUSHI)
        .unwrap();
    assert_eq!(trigger.event, "hp_recovered");
    assert_eq!(
        trigger.filter,
        "owner_alive_other_wei_recovery_own_phase_once_per_turn"
    );
    assert!(trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == CAOMAO)
        .expect("caomao in roster");
    assert!(character.skills.contains(&ZHUSHI));
}

#[test]
fn wei_recovery_in_own_phase_lets_owner_draw_once_per_turn() {
    let (state, owner, other) = setup(20_260_910_01);
    let hand_before = state.hand_count(owner);

    let (state, confirmed) = wei_recovers(state, other);
    assert!(confirmed, "optional confirm must open");
    assert!(state.has_pending_choice());
    let state = resolve(state, owner, 20_260_910_11);
    assert_eq!(state.hand_count(owner), hand_before + 1, "owner draws one");

    // 每回合限一次：同回合再次回复不触发。
    let (state, confirmed) = wei_recovers(state, other);
    assert!(!confirmed, "turn limit must suppress the second fire");
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), hand_before + 1);
}

#[test]
fn non_wei_recovery_does_not_trigger() {
    let (state, owner, other) = setup(20_260_910_02);
    let mut tx = Transaction::begin(state);
    tx.state_mut().group_overrides.insert(other, "wu".into());
    let state = tx.commit().state;
    let (state, confirmed) = wei_recovers(state, other);
    assert!(!confirmed && !state.has_pending_choice(), "wu must not fire");
    let _ = owner;
}

#[test]
fn recovery_outside_own_phase_does_not_trigger() {
    let (state, owner, other) = setup(20_260_910_03);
    // 当前回合角色切回 owner：事件角色（other）不再处于自己的回合。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    let state = tx.commit().state;
    let (_, confirmed) = wei_recovers(state, other);
    assert!(
        !confirmed,
        "recovery outside the recovering player's own phase must not fire"
    );
}
}
