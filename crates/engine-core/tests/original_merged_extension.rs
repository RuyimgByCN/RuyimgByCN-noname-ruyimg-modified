//! O2 全量合并批次 1（2026-10-14）：家族 extension 的 33 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_extension_ddd_baosanniang {
//! Golden for 3D精选 鲍三娘【枝连/赍剑】（ABI 629）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZHILIAN: u32 = 3_056_536_406;
const JIJIAN: u32 = 887_278_272;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/baosanniang.skill");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-baosanniang", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 629);
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&standard).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    tx.state_mut()
        .players
        .get_mut(players[0])
        .unwrap()
        .character_def = 25;
    tx.state_mut()
        .players
        .get_mut(players[1])
        .unwrap()
        .character_def = 1;
    tx.state_mut()
        .players
        .get_mut(players[2])
        .unwrap()
        .character_def = 2;
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
    effects.extend([ZHILIAN, JIJIAN].map(|skill_def| EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(skill_def),
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.entered_discard_cards.clear();
    (tx.commit().state, players)
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
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

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    serial: u32,
) -> CardId {
    let zone_owner = if matches!(zone_kind, ZoneKind::DiscardPile) {
        PlayerId::NONE
    } else {
        owner
    };
    let zone = state.zones.find(zone_kind, zone_owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_629_000 + serial),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: if zone_owner.is_some() {
            owner
        } else {
            PlayerId::NONE
        },
        face_up: zone_owner.is_none(),
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(
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

fn arm_zhilian(state: &mut GameState, owner: PlayerId, male: PlayerId) {
    state.turn.phase = Phase::End;
    state.turn.current = male;
    state
        .turn
        .set_flag(turn_flag::card_action_count_by_player(owner), 1);
    state
        .turn
        .set_flag(turn_flag::card_action_count_by_player(male), 1);
}

#[test]
fn zhilian_draws_both_then_gains_all_turn_entered_sha_still_in_discard() {
    let (mut state, players) = setup();
    let (owner, male, third) = (players[0], players[1], players[2]);
    arm_zhilian(&mut state, owner, male);
    let sha = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        CardKind::ShaThunder,
        Suit::Spade,
        1,
    );
    state.turn.entered_discard_cards.insert(sha);
    let owner_before = state.hand_count(owner);
    let male_before = state.hand_count(male);
    let skill = skill_id(&state, owner, ZHILIAN);
    state
        .turn
        .set_flag(turn_flag::card_action_count_by_player(third), 1);
    state.players.get_mut(third).unwrap().alive = false;
    let mut tx = Transaction::begin(state);
    assert!(run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddBaosanniang {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            action: 0,
            cards: Vec::new(),
            targets: Vec::new(),
            used_kind: 0,
            skill,
        }],
    )
    .unwrap()
    .is_empty());
    let mut state = tx.commit().state;
    state.players.get_mut(third).unwrap().alive = true;
    state
        .turn
        .set_flag(turn_flag::card_action_count_by_player(third), 0);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddBaosanniang {
                player: owner,
                target: PlayerId::NONE,
                card: CardId::NONE,
                action: 0,
                cards: Vec::new(),
                targets: Vec::new(),
                used_kind: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let (state, result) = choose(state, owner, vec![0], 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_before + 2);
    assert_eq!(state.hand_count(male), male_before + 1);
    assert!(state.hand_cards(owner).contains(&sha));
}

#[test]
fn zhilian_discard_branch_recovers_the_sha_owner_just_discarded() {
    let (mut state, players) = setup();
    let (owner, male) = (players[0], players[1]);
    arm_zhilian(&mut state, owner, male);
    let owner_sha = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Heart,
        10,
    );
    let male_card = inject(
        &mut state,
        male,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Club,
        11,
    );
    let skill = skill_id(&state, owner, ZHILIAN);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddBaosanniang {
                player: owner,
                target: PlayerId::NONE,
                card: CardId::NONE,
                action: 0,
                cards: Vec::new(),
                targets: Vec::new(),
                used_kind: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = choose(tx.commit().state, owner, vec![1], 20);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let (state, result) = choose(state, male, vec![male_card.raw()], 21);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let (state, result) = choose(state, owner, vec![owner_sha.raw()], 22);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(owner).contains(&owner_sha));
    assert!(!state.hand_cards(male).contains(&male_card));
}

#[test]
fn jijian_gives_sha_reveals_same_color_and_blocks_second_sha_name() {
    let (mut state, players) = setup();
    let (owner, male, victim) = (players[0], players[1], players[2]);
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let gift = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        20,
    );
    let fire_sha = inject(
        &mut state,
        male,
        ZoneKind::Hand,
        CardKind::ShaFire,
        Suit::Heart,
        21,
    );
    let wuzhong = inject(
        &mut state,
        male,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        Suit::Diamond,
        22,
    );
    let skill = skill_id(&state, owner, JIJIAN);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddBaosanniang {
                player: owner,
                target: male,
                card: gift,
                action: 1,
                cards: vec![gift],
                targets: vec![male],
                used_kind: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    assert!(state.hand_cards(male).contains(&gift));
    let (state, result) = choose(state, male, vec![fire_sha.raw(), wuzhong.raw()], 30);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let choice = result.choice.as_ref().unwrap();
    assert_eq!(choice.observer, owner);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == u32::from(CardKind::ShaFire.tag())));
    let (state, result) = choose(state, owner, vec![u32::from(CardKind::ShaFire.tag())], 31);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let choice = result.choice.as_ref().unwrap();
    assert_eq!(choice.observer, owner);
    let (state, result) = choose(state, owner, vec![victim.raw()], 32);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let choice = result.choice.as_ref().unwrap();
    assert_eq!(choice.observer, male);
    let options: Vec<_> = choice.options.iter().map(|option| option.token).collect();
    assert_eq!(options, vec![u32::from(CardKind::TrickWuzhong.tag())]);
    let (state, result) = choose(
        state,
        male,
        vec![u32::from(CardKind::TrickWuzhong.tag())],
        33,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert!(state.resolution.pending_choice_id == 0);
}
}

mod original_extension_ddd_caomao {
//! Golden for 3D精选 曹髦【太思/驱车/潜龙】（ABI 624）。

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DDDTAISI: u32 = 632_127_901;
const DDDQUCHE: u32 = 2_669_792_322;
const DDDQUCHE_EFFECT: u32 = 3_815_717_800;
const DDDQIANLONG: u32 = 3_550_796_651;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/caomao.skill");

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    serial: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_624_000 + serial),
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

fn setup(skill_defs: &[u32]) -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-caomao", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    tx.state_mut().players.get_mut(players[0]).unwrap().identity = Identity::Lord;
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
    effects.extend(skill_defs.iter().map(|skill_def| EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(*skill_def),
    }));
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, players)
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
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

#[test]
fn package_exposes_abi624_active_modifier_and_three_triggers() {
    let package = content_compiler::compile_source_named("ddd-caomao", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 624);
    assert_eq!(package.skill_index.len(), 4);
    assert_eq!(package.trigger_index.len(), 3);
    assert_eq!(package.active_index.len(), 1);
    assert_eq!(package.modifier_index.len(), 1);
}

#[test]
fn taisi_gives_a_turn_entered_discard_card_and_draws_for_its_damage_source() {
    let (mut state, players) = setup(&[DDDTAISI]);
    let (owner, source, target) = (players[0], players[1], players[2]);
    let card = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        CardKind::Other,
        Suit::Club,
        1,
    );
    inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Other,
        Suit::Spade,
        2,
    );
    inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Other,
        Suit::Spade,
        3,
    );
    state.turn.record_hp_change(owner);
    state.turn.record_entered_discard_cards([card]);
    state.turn.record_damage_pair(source, owner);
    let skill = skill_id(&state, owner, DDDTAISI);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddCaomao {
                player: owner,
                action: 0,
                cards: Vec::new(),
                targets: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let (state, card_offer) = resolve(state, owner, vec![source.raw()], false, 1);
    assert_eq!(card_offer.choice.unwrap().kind, ChoiceKind::SelectCards);
    let (state, done) = resolve(state, owner, vec![card.raw()], false, 2);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(card).unwrap().owner, source);
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.hand_count(target), 0);
}

#[test]
fn quche_gives_red_uses_black_sha_without_count_and_can_stop_before_next_round() {
    let (mut state, players) = setup(&[DDDQUCHE, DDDQUCHE_EFFECT]);
    let (owner, gift_target, sha_target) = (players[0], players[1], players[2]);
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let red = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Heart,
        10,
    );
    let black = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Club,
        11,
    );
    let next_red = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Diamond,
        12,
    );
    let hp = state.players.get(sha_target).unwrap().hp;
    let skill = skill_id(&state, owner, DDDQUCHE);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddCaomao {
                player: owner,
                action: 10,
                cards: vec![red],
                targets: vec![gift_target],
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    assert_eq!(state.cards.get(red).unwrap().owner, gift_target);
    let (state, target_offer) = resolve(state, owner, vec![black.raw()], false, 10);
    assert_eq!(target_offer.choice.unwrap().kind, ChoiceKind::SelectTargets);
    let (state, red_offer) = resolve(state, owner, vec![sha_target.raw()], false, 11);
    assert_eq!(red_offer.choice.unwrap().kind, ChoiceKind::SelectCards);
    assert_eq!(state.players.get(sha_target).unwrap().hp, hp - 1);
    assert_eq!(engine_core::query::sha_play_phase_use_count(&state), 0);
    assert_eq!(
        state
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .find(|(mark, _)| *mark == DDDQUCHE_EFFECT)
            .unwrap()
            .1,
        2
    );
    let (state, done) = resolve(state, owner, Vec::new(), true, 12);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.cards.get(next_red).unwrap().owner, owner);
}

#[test]
fn qianlong_lord_draws_three_after_killing_another_player() {
    let (mut state, players) = setup(&[DDDQIANLONG]);
    let owner = players[0];
    for serial in 20..23 {
        inject(
            &mut state,
            PlayerId::NONE,
            ZoneKind::DrawPile,
            CardKind::Other,
            Suit::Spade,
            serial,
        );
    }
    let qianlong = skill_id(&state, owner, DDDQIANLONG);
    assert!(state.skill_effectively_enabled(state.skills.get(qianlong).unwrap()));
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PlayerKilled {
            player: players[1],
            killer: owner,
            hand_equip_cards: Vec::new(),
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert_eq!(state.hand_count(owner), before + 3);
}
}

mod original_extension_ddd_caoshuang {
//! Golden for 3D精选 曹爽【专摄/危秋】（ABI 635）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const ZHUANSHE: u32 = 902_778_848;
const WEIQIU: u32 = 208_021_574;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/caoshuang.skill");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-caoshuang", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 635);
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&standard).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 4, 20_260_803, "identity-standard").unwrap();
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
    effects.extend([ZHUANSHE, WEIQIU].map(|skill_def| EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(skill_def),
    }));
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, players)
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
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

fn inject(state: &mut GameState, owner: PlayerId, serial: u32, kind: CardKind) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_635_000 + serial),
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

#[test]
fn zhuanshe_gifts_matching_sha_and_adds_a_distance_ignored_target() {
    let (mut state, players) = setup();
    let owner = players[0];
    let actor = players[1];
    let first = players[2];
    let extra = players[3];
    let skill = skill_id(&state, owner, ZHUANSHE);
    let sha = inject(&mut state, owner, 1, CardKind::Sha);
    state.turn.current = actor;
    state.turn.phase = Phase::Play;

    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddCaoshuang {
                player: owner,
                actor,
                card: CardId::NONE,
                action: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (mut state, _) = choose(tx.commit().state, owner, vec![sha.raw()], false, 1);
    assert!(state.hand_cards(actor).contains(&sha));

    assert!(!state.skills.get(skill).unwrap().state_blob.is_empty());
    state.resolution.resume_kind = ContinuationKind::ResumePlayedSha;
    state.resolution.resume_payload = vec![actor.raw(), first.raw(), 1, 0, 1, sha.raw()];
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: actor,
            card: sha,
            targets: vec![first],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let commit = tx.commit();
    let choice = commit.output.choice.as_ref().unwrap();
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == extra.raw()));
    let (mut state, result) = choose(commit.state, owner, vec![extra.raw()], false, 2);
    assert!(result.choice.is_none());
    assert_eq!(state.skills.get(skill).unwrap().state_blob.last(), Some(&1));
    state.resolution.resume_kind = ContinuationKind::None;
    state.resolution.resume_payload.clear();
    state.resolution.clear_choice();

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddCaoshuang {
            player: owner,
            actor,
            card: CardId::NONE,
            action: 2,
            skill,
        }],
    )
    .unwrap();
    assert!(tx.state().skills.get(skill).unwrap().state_blob.is_empty());

    let tao = inject(tx.state_mut(), owner, 2, CardKind::Tao);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddCaoshuang {
                player: owner,
                actor,
                card: CardId::NONE,
                action: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, _) = choose(tx.commit().state, owner, vec![tao.raw()], false, 3);
    let actor_hp = state.players.get(actor).unwrap().hp;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddCaoshuang {
                player: owner,
                actor,
                card: CardId::NONE,
                action: 2,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, _) = choose(tx.commit().state, owner, vec![1], false, 4);
    assert_eq!(state.players.get(actor).unwrap().hp, actor_hp - 1);
}

#[test]
fn weiqiu_cancels_recovery_before_hp_changes_and_draws_for_each_empty_owner() {
    let (mut state, players) = setup();
    let owner = players[0];
    let target = players[1];
    state.players.get_mut(target).unwrap().hp -= 2;
    let hp = state.players.get(target).unwrap().hp;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source: target,
            player: target,
            amount: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(target).unwrap().hp, hp);
    assert_eq!(tx.state().hand_count(owner), 1);
    assert_eq!(tx.state().resolution.current_recovery_id, 0);
    assert!(tx.state().resolution.prevented_recovery_ids.is_empty());
}
}

mod original_extension_ddd_dingfeng {
//! Golden for 3D精选 丁奉【短兵】（ABI 621）。

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DDDDUANBING: u32 = 2_748_520_735;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/dingfeng.skill");

fn inject(state: &mut GameState, owner: PlayerId, zone: ZoneKind, serial: u32) -> CardId {
    let zone = state.zones.find(zone, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_621_000 + serial),
        kind: CardKind::Other,
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

fn setup() -> (GameState, PlayerId, PlayerId, CardId) {
    let package = content_compiler::compile_source_named("ddd-dingfeng", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(DDDDUANBING),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let cost = inject(tx.state_mut(), owner, ZoneKind::Hand, 1);
    inject(tx.state_mut(), PlayerId::NONE, ZoneKind::DrawPile, 2);
    inject(tx.state_mut(), PlayerId::NONE, ZoneKind::DrawPile, 3);
    (tx.commit().state, owner, target, cost)
}

fn skill(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
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
                .is_some_and(|skill| skill.def.raw() == DDDDUANBING)
        })
        .unwrap()
}

fn submit(
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

fn choose(
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

#[test]
fn package_exposes_exact_abi_and_optional_card_damage_trigger() {
    let package = content_compiler::compile_source_named("ddd-dingfeng", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 621);
    assert_eq!(package.skill_index.len(), 1);
    assert_eq!(package.active_index.len(), 1);
    assert_eq!(package.trigger_index.len(), 1);
    let trigger = &package.trigger_index[0];
    assert_eq!(trigger.event, "damage");
    assert_eq!(
        trigger.filter,
        "owner_source_card_damage_target_can_receive_owner_bingliang"
    );
    assert!(trigger.optional);
}

#[test]
fn duanbing_places_bingliang_draws_two_uses_sha_and_transfers_it() {
    let (state, owner, target, cost) = setup();
    let hp = state.players.get(target).unwrap().hp;
    let active = skill(&state, owner);
    let (state, opened) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        1,
    );
    assert_eq!(opened.choice.unwrap().kind, ChoiceKind::SelectCards);

    let (state, sha_target) = choose(state, owner, vec![cost.raw()], 2);
    assert_eq!(sha_target.choice.unwrap().kind, ChoiceKind::SelectTargets);
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(
        state.cards.get(cost).unwrap().kind,
        CardKind::TrickBingliang
    );

    let (state, trigger) = choose(state, owner, vec![target.raw()], 3);
    assert_eq!(trigger.choice.unwrap().kind, ChoiceKind::Confirm);
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);

    let (state, result) = choose(state, owner, vec![1], 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let owner_judge = state.zones.find(ZoneKind::Judge, owner).unwrap();
    let target_judge = state.zones.find(ZoneKind::Judge, target).unwrap();
    assert!(!state.zones.get(owner_judge).unwrap().cards.contains(&cost));
    assert!(state.zones.get(target_judge).unwrap().cards.contains(&cost));
}
}

mod original_extension_ddd_guanning {
//! Golden for 3D精选 管宁【游学/承经】（ABI 634）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const YOUXUE: u32 = 2_449_657_606;
const CHENGJING: u32 = 4_114_032_104;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/guanning.skill");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-guanning", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 634);
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&standard).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 4, 20_260_803, "identity-standard").unwrap();
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
    effects.extend(
        [YOUXUE, CHENGJING].map(|skill_def| EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, players)
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
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

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    serial: u32,
    kind: CardKind,
    zone_kind: ZoneKind,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_634_000 + serial),
        kind,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: if kind == CardKind::Weapon { 2 } else { 0 },
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
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

#[test]
fn youxue_starts_on_round_one_moves_unique_mark_and_draws_distance() {
    let (mut state, players) = setup();
    let owner = players[0];
    let target = players[2];
    let skill = skill_id(&state, owner, YOUXUE);
    state.turn.round = 1;
    state.turn.current = owner;
    state.turn.phase = Phase::Prepare;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddGuanning {
                player: owner,
                actor: owner,
                action: 0,
                card: CardId::NONE,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = choose(tx.commit().state, owner, vec![target.raw()], false, 1);
    assert!(result.choice.is_none());
    assert_eq!(state.hand_count(owner), 2);
    assert!(!state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, _)| *mark == YOUXUE));
    assert!(state
        .players
        .get(target)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| { *mark == YOUXUE && *count == 1 }));
}

#[test]
fn chengjing_keeps_last_eligible_use_then_uses_equipment_and_offers_gift() {
    let (mut state, players) = setup();
    let owner = players[0];
    let actor = players[1];
    let recipient = players[2];
    let skill = skill_id(&state, owner, CHENGJING);
    let first = inject(&mut state, actor, 1, CardKind::TrickWuzhong, ZoneKind::Hand);
    let last = inject(&mut state, actor, 2, CardKind::TrickWuzhong, ZoneKind::Hand);
    state.turn.current = actor;
    state.turn.phase = Phase::Play;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddMark {
                player: actor,
                mark: engine_api::MarkDefId::new(YOUXUE),
                delta: 1,
            },
            EffectRequest::UseCard {
                player: actor,
                card: first,
                targets: Vec::new(),
            },
            EffectRequest::UseCard {
                player: actor,
                card: last,
                targets: Vec::new(),
            },
        ],
    )
    .unwrap();
    assert_eq!(
        tx.state().turn.ddd_chengjing_eligible_uses,
        vec![first, last]
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddGuanning {
            player: owner,
            actor,
            action: 1,
            card: CardId::NONE,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().skill_pile_cards(owner, CHENGJING), vec![last]);

    let material = inject(tx.state_mut(), owner, 3, CardKind::Weapon, ZoneKind::Equip);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let revision = state.revision;
    let (state, opened) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    );
    assert!(opened.choice.is_some());
    let (state, gift) = choose(state, owner, vec![material.raw()], false, 3);
    assert!(gift.choice.is_some(), "{gift:?}");
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(material).unwrap().zone, discard);
    let (state, result) = choose(state, owner, vec![1], false, 4);
    assert!(result.choice.is_none());
    assert!(state.hand_cards(recipient).contains(&material));
}
}

mod original_extension_ddd_handang {
//! Golden for 3D精选 韩当【险袭】（ABI 617）。

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DDDXIANXI: u32 = 3_400_504_078;
const STATE_MAGIC: u32 = 0xDDD0_6170;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/handang.skill");

fn setup(seed: u64) -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-handang", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(DDDXIANXI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, players)
}

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, serial: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_000 + serial),
        kind,
        suit: Suit::Club,
        rank: 8,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn skill(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
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
                .is_some_and(|skill| skill.def.raw() == DDDXIANXI)
        })
        .unwrap()
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

fn choose(
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

fn track(
    state: &mut GameState,
    owner: PlayerId,
    card: CardId,
    targets: &[PlayerId],
) -> engine_api::SkillId {
    let skill = skill(state, owner);
    let blob = &mut state.skills.get_mut(skill).unwrap().state_blob;
    blob.clear();
    blob.extend_from_slice(&STATE_MAGIC.to_le_bytes());
    blob.extend_from_slice(&card.raw().to_le_bytes());
    blob.extend_from_slice(&(targets.len() as u32).to_le_bytes());
    for target in targets {
        blob.extend_from_slice(&target.raw().to_le_bytes());
    }
    skill
}

#[test]
fn package_exposes_exact_abi_and_trigger_contract() {
    let package = content_compiler::compile_source_named("ddd-handang", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 617);
    assert_eq!(package.skill_index.len(), 1);
    assert_eq!(package.trigger_index.len(), 2);
}

#[test]
fn xianxi_adds_multiple_sha_targets_while_ignoring_distance() {
    let (mut state, players) = setup(20_260_803);
    let owner = players[0];
    let sha = inject(&mut state, owner, CardKind::Sha, 1);
    let hp: Vec<_> = players
        .iter()
        .map(|player| state.players.get(*player).unwrap().hp)
        .collect();
    let (state, result) = command(
        state,
        owner,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![players[1]],
            card_targets: Vec::new(),
        },
        10,
    );
    assert_eq!(result.choice.unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![1], false, 11);
    let choice = result.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == players[2].raw()));
    let (state, result) = choose(
        state,
        owner,
        vec![players[2].raw(), players[3].raw()],
        false,
        12,
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert!(result.choice.is_none());
    assert!(state
        .skills
        .get(skill(&state, owner))
        .unwrap()
        .state_blob
        .is_empty());
    assert_eq!(state.players.get(players[1]).unwrap().hp, hp[1] - 1);
    assert_eq!(state.players.get(players[2]).unwrap().hp, hp[2] - 1);
    assert_eq!(state.players.get(players[3]).unwrap().hp, hp[3] - 1);
}

#[test]
fn xianxi_settlement_supports_discard_damage_and_cancel_draw_loss() {
    let (mut state, players) = setup(20_260_804);
    let owner = players[0];
    let card = inject(&mut state, owner, CardKind::Sha, 2);
    let cost = inject(&mut state, owner, CardKind::Shan, 3);
    let tracked = [players[2]];
    let skill = track(&mut state, owner, card, &tracked);
    let hp = state.players.get(players[2]).unwrap().hp;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddXianxi {
                player: owner,
                card,
                action: 1,
                event_targets: tracked.to_vec(),
                damaged_targets: Vec::new(),
                skill,
            }],
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let (state, result) = choose(state, owner, vec![cost.raw()], false, 20);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.players.get(players[2]).unwrap().hp, hp - 1);
    assert!(!state.hand_cards(owner).contains(&cost));

    let (mut state, players) = setup(20_260_805);
    let owner = players[0];
    let card = inject(&mut state, owner, CardKind::Sha, 4);
    inject(&mut state, owner, CardKind::Shan, 5);
    let tracked = [players[1], players[2]];
    let skill = track(&mut state, owner, card, &tracked);
    let hp = state.players.get(owner).unwrap().hp;
    let hand = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddXianxi {
                player: owner,
                card,
                action: 1,
                event_targets: tracked.to_vec(),
                damaged_targets: Vec::new(),
                skill,
            }],
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let (state, result) = choose(state, owner, Vec::new(), true, 30);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
    assert_eq!(state.hand_count(owner), hand + 2);
}
}

mod original_extension_ddd_jianshuo {
//! Golden for 3D精选 蹇硕【分野/逝潮】（ABI 633）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const FENYE: u32 = 1_190_748_172;
const SHICHAO: u32 = 219_219_058;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/jianshuo.skill");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-jianshuo", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 633);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == FENYE)
        .unwrap();
    assert_eq!(active.uses_per_play_phase, Some(1));
    assert!(active.targets.as_ref().unwrap().can_compare);

    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&standard).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 4, 20_260_803, "identity-standard").unwrap();
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
    effects.extend([FENYE, SHICHAO].map(|skill_def| EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(skill_def),
    }));
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, players)
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
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

fn inject(state: &mut GameState, owner: PlayerId, serial: u32, rank: u8) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_633_000 + serial),
        kind: CardKind::Other,
        suit: Suit::Heart,
        rank,
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

#[test]
fn fenye_uses_joined_cards_average_and_orders_winners_by_seat() {
    let (mut state, players) = setup();
    let owner = players[0];
    let skill = skill_id(&state, owner, FENYE);
    let cards = [
        inject(&mut state, players[0], 1, 10),
        inject(&mut state, players[1], 2, 4),
        inject(&mut state, players[2], 3, 12),
        inject(&mut state, players[3], 4, 2),
    ];
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(100),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![players[1]],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(result.choice.is_some());
    let (state, _) = choose(state, players[0], vec![cards[0].raw()], false, 1);
    let (state, _) = choose(state, players[1], vec![cards[1].raw()], false, 2);
    let (state, _) = choose(state, players[2], vec![1], false, 3);
    let (state, _) = choose(state, players[2], vec![cards[2].raw()], false, 4);
    let (state, _) = choose(state, players[3], vec![2], false, 5);
    let (state, result) = choose(state, players[3], vec![cards[3].raw()], false, 6);
    assert_eq!(
        choice_tokens(&result),
        vec![players[1].raw(), players[3].raw()]
    );
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(cards
        .iter()
        .all(|card| state.cards.get(*card).unwrap().zone == discard));

    let loser_hp = state.players.get(players[1]).unwrap().hp;
    let (state, result) = choose(state, players[0], vec![players[1].raw()], false, 7);
    assert_eq!(state.players.get(players[1]).unwrap().hp, loser_hp - 1);
    assert_eq!(choice_tokens(&result), vec![players[3].raw()]);
    let (state, result) = choose(state, players[2], Vec::new(), true, 8);
    assert!(result.choice.is_none());
    let processing = state
        .zones
        .find(ZoneKind::Processing, PlayerId::NONE)
        .unwrap();
    assert!(state.zones.get(processing).unwrap().cards.is_empty());
}

#[test]
fn shichao_adjusts_to_rank_and_source_can_prevent_then_advance_rank() {
    let (mut state, players) = setup();
    let owner = players[0];
    let skill = skill_id(&state, owner, SHICHAO);
    inject(&mut state, players[1], 10, 8);
    inject(&mut state, players[1], 11, 9);
    inject(&mut state, players[1], 12, 10);
    inject(&mut state, players[2], 13, 6);
    inject(&mut state, players[2], 14, 7);
    inject(&mut state, players[3], 15, 5);

    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddJianshuo {
                player: owner,
                target: owner,
                action: 1,
                cards: Vec::new(),
                targets: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = choose(tx.commit().state, owner, vec![players[1].raw()], false, 20);
    assert!(result.choice.is_none());
    assert_eq!(state.hand_count(owner), 3);

    let mut state = state;
    state.resolution.current_damage_id = 633;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddJianshuo {
                player: owner,
                target: players[1],
                action: 2,
                cards: Vec::new(),
                targets: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, _) = choose(tx.commit().state, players[1], vec![1], false, 21);
    assert!(state.resolution.prevented_damage_ids.contains(&633));
    assert_eq!(state.skills.get(skill).unwrap().mark, 1);

    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddJianshuo {
                player: owner,
                target: owner,
                action: 1,
                cards: Vec::new(),
                targets: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = choose(tx.commit().state, owner, vec![players[2].raw()], false, 22);
    let discard = choice_tokens(&result)[0];
    let (state, _) = choose(state, owner, vec![discard], false, 23);
    assert_eq!(state.hand_count(owner), 2);
}
}

mod original_extension_ddd_liangxi {
//! Golden for 3D精选 梁习【仝御】（ABI 640）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TONGYU: u32 = 2_302_761_270;
const TONGYU_BLOCKER: u32 = 3_861_572_400;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/liangxi.skill");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-liangxi", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 640);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap())
        .unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_804, "identity-standard").unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(TONGYU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, players)
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    serial: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_640_000 + serial),
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

fn skill(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
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

fn submit(
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

fn choose(
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
fn tongyu_rejects_duplicate_suits() {
    let (mut state, players) = setup();
    let owner = players[0];
    let first = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        1,
    );
    let second = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Spade,
        2,
    );
    let tongyu = skill(&state, owner, TONGYU);
    let (_state, result) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill: tongyu,
            targets: Vec::new(),
            card_targets: vec![first, second],
        },
        1,
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
}

#[test]
fn tongyu_adds_material_count_rejects_target_and_can_give_leftovers() {
    let (mut state, players) = setup();
    let owner = players[0];
    let rejector = players[1];
    let material_a = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        10,
    );
    let material_b = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Heart,
        11,
    );
    let revealed: Vec<_> = (0..5)
        .map(|index| {
            inject(
                &mut state,
                PlayerId::NONE,
                ZoneKind::DrawPile,
                CardKind::Other,
                Suit::Club,
                20 + index,
            )
        })
        .collect();
    let tongyu = skill(&state, owner, TONGYU);
    let (mut state, result) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill: tongyu,
            targets: Vec::new(),
            card_targets: vec![material_a, material_b],
        },
        10,
    );
    let mut next_choice = result.choice.unwrap();
    assert_eq!(next_choice.observer, owner);

    // Counterclockwise direction.
    let (next, result) = choose(state, owner, vec![1], false, 11);
    state = next;
    next_choice = result.choice.unwrap();
    // Every target chooses: owner accepts, rejector rejects, last target accepts.
    for (index, target) in [owner, players[2], rejector].into_iter().enumerate() {
        assert_eq!(next_choice.observer, target);
        let selection = u32::from(target == rejector);
        let (next, result) = choose(state, target, vec![selection], false, 12 + index as u64);
        state = next;
        next_choice = result.choice.unwrap();
    }
    assert!(state.players.get(owner).unwrap().skills.iter().any(|id| {
        state
            .skills
            .get(*id)
            .is_some_and(|skill| skill.def.raw() == TONGYU_BLOCKER)
    }));

    // Two effective targets each take one card from the five-card pool.
    for (index, target) in [owner, players[2]].into_iter().enumerate() {
        assert_eq!(next_choice.observer, target);
        let card = next_choice.options[0].token;
        let (next, result) = choose(state, target, vec![card], false, 20 + index as u64);
        state = next;
        next_choice = result.choice.unwrap();
    }
    // Optional remained trigger, then choose the rejecting target.
    assert_eq!(next_choice.observer, owner);
    let (next, result) = choose(state, owner, vec![1], false, 30);
    state = next;
    next_choice = result.choice.unwrap();
    assert_eq!(next_choice.observer, owner);
    (state, _) = choose(state, owner, vec![rejector.raw()], false, 31);

    assert_eq!(state.hand_count(rejector), 3);
    assert!(
        revealed
            .iter()
            .filter(|card| state.hand_cards(rejector).contains(card))
            .count()
            == 3
    );
}
}

mod original_extension_ddd_liuba {
//! Golden for 3D精选 刘巴【丰政/玉律】（ABI 632）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const FENGZHENG: u32 = 986_583_704;
const FENGZHENG_GLOBAL: u32 = 1_253_882_292;
const YULV: u32 = 694_263_366;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/liuba.skill");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-liuba", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 632);
    let global = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == FENGZHENG_GLOBAL)
        .unwrap();
    assert_eq!(global.uses_per_play_phase, Some(1));
    assert_eq!(
        global
            .selection
            .as_ref()
            .unwrap()
            .sum_effective_number_exact,
        Some(13)
    );

    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&standard).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
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
    effects.extend([FENGZHENG, YULV].map(|skill_def| EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(skill_def),
    }));
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, players)
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
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

fn inject(state: &mut GameState, owner: PlayerId, serial: u32, rank: u8) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_632_000 + serial),
        kind: CardKind::Other,
        suit: Suit::Heart,
        rank,
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

#[test]
fn fengzheng_installs_global_and_uses_exact_thirteen_as_complete_wuzhong() {
    let (mut state, players) = setup();
    for player in &players {
        assert!(state
            .players
            .get(*player)
            .unwrap()
            .skills
            .iter()
            .any(|skill| { state.skills.get(*skill).unwrap().def.raw() == FENGZHENG_GLOBAL }));
    }

    let user = players[1];
    let six = inject(&mut state, user, 1, 6);
    let seven = inject(&mut state, user, 2, 7);
    state.turn.current = user;
    state.turn.phase = Phase::Play;
    let global = skill_id(&state, user, FENGZHENG_GLOBAL);
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddLiuba {
            player: user,
            actor: user,
            action: 0,
            cards: vec![six, seven],
            targets: Vec::new(),
            skill: global,
        }],
    );
    assert!(matches!(result, Ok(_) | Err(EffectError::ChoiceRequested)));
    let state = tx.commit().state;
    assert_eq!(
        state.skills.get(global).unwrap().round_fire_round,
        state.turn.round
    );
    assert!(!state.hand_cards(user).contains(&six));
    assert!(!state.hand_cards(user).contains(&seven));
}

#[test]
fn fengzheng_counts_distinct_previous_round_users_and_loses_on_three_to_one() {
    let (mut state, players) = setup();
    let owner = players[0];
    let fengzheng = skill_id(&state, owner, FENGZHENG);
    for player in &players {
        let global = skill_id(&state, *player, FENGZHENG_GLOBAL);
        state.skills.get_mut(global).unwrap().round_fire_round = 1;
    }
    state.turn.round = 2;
    state.turn.current = owner;
    state.turn.phase = Phase::Prepare;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddLiuba {
                player: owner,
                actor: owner,
                action: 1,
                cards: Vec::new(),
                targets: Vec::new(),
                skill: fengzheng,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = choose(tx.commit().state, owner, vec![1], false, 10);
    let subset: Vec<_> = result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(subset.len(), 3);
    let (state, _) = choose(state, owner, subset, false, 11);
    let (state, result) = choose(state, owner, vec![players[1].raw()], false, 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(players[1]), 3);
    assert!(!state
        .players
        .get(owner)
        .unwrap()
        .skills
        .contains(&fengzheng));
}

#[test]
fn yulv_matches_physical_rank_and_exchanges_after_second_trigger_this_turn() {
    let (state, players) = setup();
    let owner = players[0];
    let actor = players[1];
    let yulv = skill_id(&state, owner, YULV);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddLiuba {
                player: owner,
                actor: owner,
                action: 3,
                cards: Vec::new(),
                targets: Vec::new(),
                skill: yulv,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    let initial = committed.output.choice.as_ref().unwrap().options[0].token;
    let (mut state, _) = choose(committed.state, owner, vec![initial], false, 20);
    let old_yulv = state.skill_pile_cards(owner, YULV)[0];
    let rank = state.cards.get(old_yulv).unwrap().rank;
    state.turn.current = actor;
    state.turn.phase = Phase::Play;

    for key in [21u64, 24] {
        let card = inject(&mut state, actor, key as u32, rank);
        let mut tx = Transaction::begin(state);
        assert_eq!(
            run_effects(
                &mut tx,
                vec![EffectRequest::DiscardCards {
                    player: actor,
                    cards: vec![card],
                }],
            ),
            Err(EffectError::ChoiceRequested)
        );
        let (next, result) = choose(tx.commit().state, owner, vec![0], false, key + 1);
        state = next;
        if key == 24 {
            assert!(result.choice.is_some(), "第二次玉律后应继续要求交换");
            let replacement = result.choice.as_ref().unwrap().options[0].token;
            let (next, result) = choose(state, owner, vec![replacement], false, key + 2);
            assert!(matches!(result.result, CommandResult::Accepted { .. }));
            state = next;
        }
    }
    assert_ne!(state.skill_pile_cards(owner, YULV)[0], old_yulv);
    assert!(state.hand_cards(owner).contains(&old_yulv));
}
}

mod original_extension_ddd_liuchong {
//! Golden for 3D精选 刘宠【精彀】【末焱】（ABI 619）。

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DDDJINGGOU: u32 = 3_419_786_472;
const DDDMOYAN: u32 = 4_038_206_557;
const STATE_MAGIC: u32 = 0xDDD0_6190;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/liuchong.skill");

fn setup(seed: u64) -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-liuchong", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
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
    effects.extend(
        [DDDJINGGOU, DDDMOYAN].map(|skill_def| EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::Prepare;
    tx.state_mut().turn.round = 1;
    (tx.commit().state, players)
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    serial: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_200 + serial),
        kind,
        suit: Suit::Club,
        rank: 8,
        weapon_range: u8::from(kind == CardKind::Weapon) * 5,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::Equip,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
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

fn choose(
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

#[test]
fn package_exposes_exact_abi_and_trigger_contracts() {
    let package = content_compiler::compile_source_named("ddd-liuchong", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 619);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.trigger_index.len(), 3);
}

#[test]
fn jinggou_weapon_with_maximum_range_damages_another_player() {
    let (mut state, players) = setup(20_260_809);
    let owner = players[0];
    let target = players[1];
    let weapon = inject(&mut state, owner, ZoneKind::Equip, CardKind::Weapon, 1);
    let instance = skill(&state, owner, DDDJINGGOU);
    let hp = state.players.get(target).unwrap().hp;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddLiuchong {
                player: owner,
                actor: owner,
                card: weapon,
                action: 2,
                cards: Vec::new(),
                skill: instance,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseOtherDamage
    );
    let (state, result) = choose(state, owner, vec![target.raw()], 10);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn moyan_gives_selected_cards_and_increases_damage_at_its_threshold() {
    let (state, players) = setup(20_260_810);
    let owner = players[0];
    let recipient = players[1];
    let source = players[2];
    let instance = skill(&state, owner, DDDMOYAN);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddLiuchong {
                player: owner,
                actor: owner,
                card: CardId::NONE,
                action: 0,
                cards: Vec::new(),
                skill: instance,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let (state, result) = choose(state, owner, vec![recipient.raw()], 20);
    let choice = result.choice.unwrap();
    assert_eq!(choice.observer, recipient);
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    let selected: Vec<_> = choice
        .options
        .iter()
        .take(2)
        .map(|option| option.token)
        .collect();
    let (state, result) = choose(state, recipient, selected, 21);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(recipient), 2);
    let blob = &state.skills.get(instance).unwrap().state_blob;
    assert_eq!(
        u32::from_le_bytes(blob[0..4].try_into().unwrap()),
        STATE_MAGIC
    );
    assert_eq!(
        u32::from_le_bytes(blob[8..12].try_into().unwrap()),
        recipient.raw()
    );
    assert_eq!(u32::from_le_bytes(blob[12..16].try_into().unwrap()), 2);

    let hp = state.players.get(owner).unwrap().hp;
    let mut tx = Transaction::begin(state);
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
    let mut state = tx.commit().state;
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 2);

    inject(&mut state, recipient, ZoneKind::Hand, CardKind::Shan, 2);
    let hp = state.players.get(owner).unwrap().hp;
    let mut tx = Transaction::begin(state);
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
}
}

mod original_extension_ddd_liuhong {
//! Golden for 3D精选 刘宏【失兴/党锢/覆宗】（ABI 637）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SHIXING: u32 = 42_478_118;
const DANGGU: u32 = 1_734_030_406;
const FUZONG: u32 = 1_289_813_886;
const XIANGLE: u32 = 2_880_679_348;
const JIUSHI: u32 = 4_293_489_189;
const JIUSHI1: u32 = 3_647_393_232;
const JIUSHI3: u32 = 4_156_803_269;
const REZHIHENG: u32 = 2_455_708_599;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/liuhong.skill");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");
const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const REFRESH: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-liuhong", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let mut tx = Transaction::begin(GameState::empty());
    for bytes in [STANDARD, SHENHUA, YIJIANG, REFRESH] {
        let package = content_schema::ContentPackage::from_json_bytes(bytes).unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    }
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 4, 20_260_804, "identity-standard").unwrap();
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
    effects.extend(
        [SHIXING, DANGGU, FUZONG].map(|skill_def| EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.character_def = 1;
        participant.max_hp = 4;
        participant.hp = 4;
    }
    let female = tx.state_mut().players.get_mut(players[2]).unwrap();
    female.character_def = 25;
    female.max_hp = 3;
    female.hp = 3;
    tx.state_mut().players.get_mut(players[0]).unwrap().identity = Identity::Lord;
    tx.state_mut()
        .group_overrides
        .insert(players[0], "qun".into());
    tx.state_mut()
        .group_overrides
        .insert(players[1], "qun".into());
    tx.state_mut()
        .group_overrides
        .insert(players[2], "wei".into());
    tx.state_mut()
        .group_overrides
        .insert(players[3], "shu".into());
    (tx.commit().state, players)
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
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

fn has_def(state: &GameState, owner: PlayerId, def: u32) -> bool {
    state.players.get(owner).is_some_and(|player| {
        player.skills.iter().any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
    })
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    serial: u32,
    kind: CardKind,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_637_000 + serial),
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

fn dispatch(state: GameState, event: DomainEvent) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new()).unwrap();
    match run_effects(&mut tx, effects) {
        Ok(_) | Err(EffectError::ChoiceRequested) => tx.commit().state,
        Err(error) => panic!("dispatch failed: {error:?}"),
    }
}

fn choose(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
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
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn shixing_switches_existing_derived_skills_and_removes_its_own_grants() {
    let (state, players) = setup();
    let owner = players[0];
    let female = players[2];
    let shixing = skill_id(&state, owner, SHIXING);
    let state = dispatch(state, DomainEvent::GameStarted { player_count: 4 });
    for def in [XIANGLE, JIUSHI, JIUSHI1, JIUSHI3] {
        assert!(has_def(&state, owner, def), "失兴授予 {def}");
    }
    assert!(!has_def(&state, owner, REZHIHENG));

    let mut state = state;
    let trick = inject(&mut state, owner, ZoneKind::Hand, 1, CardKind::TrickJuedou);
    let state = dispatch(
        state,
        DomainEvent::CardUsed {
            player: owner,
            card: trick,
            targets: vec![players[1]],
        },
    );
    assert!(has_def(&state, owner, XIANGLE));
    for def in [JIUSHI, JIUSHI1, JIUSHI3, REZHIHENG] {
        assert!(!has_def(&state, owner, def));
    }

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: female,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(!has_def(tx.state(), owner, XIANGLE));
    assert!(has_def(tx.state(), owner, REZHIHENG));

    run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill: shixing }]).unwrap();
    assert!(!has_def(tx.state(), owner, REZHIHENG));
}

#[test]
fn danggu_links_at_least_the_qun_shortfall_and_counts_other_linked_equipment() {
    let (mut state, players) = setup();
    let owner = players[0];
    state.turn.current = owner;
    state.turn.phase = Phase::End;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::End,
        },
    );
    assert!(state.has_pending_choice());
    let (state, result) = choose(state, owner, vec![owner.raw(), players[2].raw()], 1);
    assert!(result.choice.is_none());
    assert!(!engine_core::query::discard_phase_includes_equip(
        &state, owner
    ));
    assert!(engine_core::query::discard_phase_includes_equip(
        &state, players[2]
    ));
    assert!(!engine_core::query::discard_phase_includes_equip(
        &state, players[1]
    ));
}

#[test]
fn fuzong_draws_for_living_qun_and_dead_qun_chooses_one_owner_card_to_discard() {
    let (mut state, players) = setup();
    let owner = players[0];
    let dead = players[1];
    let skill = skill_id(&state, owner, FUZONG);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddLiuhong {
            player: owner,
            actor: PlayerId::NONE,
            action: 4,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), 2);
    state = tx.commit().state;
    let chosen = inject(&mut state, owner, ZoneKind::Hand, 2, CardKind::Sha);
    state.players.get_mut(dead).unwrap().alive = false;
    let state = dispatch(
        state,
        DomainEvent::PlayerKilled {
            player: dead,
            killer: PlayerId::NONE,
            hand_equip_cards: Vec::new(),
        },
    );
    assert!(state.has_pending_choice());
    assert_eq!(state.resolution.pending_observer, dead.raw());
    let (state, result) = choose(state, dead, vec![chosen.raw()], 2);
    assert!(result.choice.is_none());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(chosen).unwrap().zone, discard);
}

#[test]
fn shixing_granted_jiushi_uses_source_free_jiu_and_turns_back_after_damage() {
    let (state, players) = setup();
    let owner = players[0];
    let state = dispatch(state, DomainEvent::GameStarted { player_count: 4 });
    let skill = skill_id(&state, owner, JIUSHI1);
    let mut state = state;
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(10),
            kind: CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::Jiu.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.players.get(owner).unwrap().turned_over);

    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(11),
            kind: CommandKind::UseSkillCard {
                skill,
                card_kind: CardKind::Jiu.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        },
    );
    assert!(!matches!(result.result, CommandResult::Accepted { .. }));

    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: players[1],
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = tx.commit().state;
    assert!(state.has_pending_choice());
    let (state, result) = choose(state, owner, vec![1], 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.players.get(owner).unwrap().turned_over);
}
}

mod original_extension_ddd_liuye {
//! Golden for 3D精选 刘晔【察势/齐策】（ABI 628）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const CHASHI: u32 = 2_811_969_013;
const QICE: u32 = 4_207_915_436;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/liuye.skill");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-liuye", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 628);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
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
    effects.extend([CHASHI, QICE].map(|skill_def| EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(skill_def),
    }));
    run_effects(&mut tx, effects).unwrap();
    let mut state = tx.commit().state;
    state.turn.current = players[1];
    state.turn.phase = Phase::Play;
    (state, players)
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
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

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    serial: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_628_000 + serial),
        kind,
        suit,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 3,
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

fn arm_chashi(
    mut state: GameState,
    owner: PlayerId,
    actor: PlayerId,
    discarded: CardId,
) -> GameState {
    let skill = skill_id(&state, owner, CHASHI);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddLiuye {
                player: owner,
                actor,
                card: CardId::NONE,
                action: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    state = tx.commit().state;
    let (state, result) = choose(state, owner, vec![discarded.raw()], 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn chashi_waits_for_first_matching_card_then_draws_for_both() {
    let (mut state, players) = setup();
    let (owner, actor) = (players[0], players[1]);
    let cost = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Weapon,
        Suit::Heart,
        1,
    );
    let mismatch = inject(
        &mut state,
        actor,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        2,
    );
    let match_by_category = inject(
        &mut state,
        actor,
        ZoneKind::Hand,
        CardKind::Armor,
        Suit::Club,
        3,
    );
    let state = arm_chashi(state, owner, actor, cost);
    let skill = skill_id(&state, owner, CHASHI);
    let owner_before = state.hand_count(owner);
    let actor_before = state.hand_count(actor);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddLiuye {
            player: owner,
            actor,
            card: mismatch,
            action: 1,
            skill,
        }],
    )
    .unwrap();
    assert!(tx
        .state()
        .skills
        .get(skill)
        .unwrap()
        .state_blob
        .starts_with(b"DCS1"));
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddLiuye {
            player: owner,
            actor,
            card: match_by_category,
            action: 1,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), owner_before + 1);
    assert_eq!(tx.state().hand_count(actor), actor_before + 1);
    assert!(tx.state().skills.get(skill).unwrap().state_blob.is_empty());
}

#[test]
fn chashi_pending_comparison_expires_with_the_play_phase() {
    let (mut state, players) = setup();
    let (owner, actor) = (players[0], players[1]);
    let cost = inject(
        &mut state,
        owner,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Diamond,
        10,
    );
    let matching = inject(
        &mut state,
        actor,
        ZoneKind::Hand,
        CardKind::Weapon,
        Suit::Diamond,
        11,
    );
    let state = arm_chashi(state, owner, actor, cost);
    let skill = skill_id(&state, owner, CHASHI);
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::ResolveDddLiuye {
                player: owner,
                actor,
                card: CardId::NONE,
                action: 4,
                skill,
            },
            EffectRequest::ResolveDddLiuye {
                player: owner,
                actor,
                card: matching,
                action: 1,
                skill,
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before);
}

#[test]
fn qice_stays_locked_across_attempts_until_owner_uses_a_trick() {
    let (mut state, players) = setup();
    let owner = players[0];
    state.turn.current = owner;
    state.turn.phase = Phase::Prepare;
    let basic = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        20,
    );
    let trick = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::TrickGuohe,
        Suit::Club,
        21,
    );
    let skill = skill_id(&state, owner, QICE);
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddLiuye {
            player: owner,
            actor: owner,
            card: CardId::NONE,
            action: 2,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 2);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::ResolveDddLiuye {
                player: owner,
                actor: owner,
                card: CardId::NONE,
                action: 2,
                skill,
            },
            EffectRequest::ResolveDddLiuye {
                player: owner,
                actor: owner,
                card: basic,
                action: 3,
                skill,
            },
            EffectRequest::ResolveDddLiuye {
                player: owner,
                actor: owner,
                card: CardId::NONE,
                action: 2,
                skill,
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 2);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::ResolveDddLiuye {
                player: owner,
                actor: owner,
                card: trick,
                action: 3,
                skill,
            },
            EffectRequest::ResolveDddLiuye {
                player: owner,
                actor: owner,
                card: CardId::NONE,
                action: 2,
                skill,
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 4);
}
}

mod original_extension_ddd_luoxian {
//! Golden for 3D精选 罗宪【示烈】（ABI 620）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DDDSHILIE: u32 = 3_904_237_061;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/luoxian.skill");

fn setup(seed: u64) -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-luoxian", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(DDDSHILIE),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, players)
}

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, rank: u8, serial: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_300 + serial),
        kind,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn inject_draw(state: &mut GameState, rank: u8, serial: u32) -> CardId {
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_300 + serial),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone: draw,
        owner: PlayerId::NONE,
        face_up: false,
    });
    state.zones.get_mut(draw).unwrap().cards.push(card);
    card
}

fn skill(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
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
                .is_some_and(|skill| skill.def.raw() == DDDSHILIE)
        })
        .unwrap()
}

fn submit(
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

fn choose(
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

#[test]
fn package_exposes_exact_abi_and_source_free_contract() {
    let package = content_compiler::compile_source_named("ddd-luoxian", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 620);
    assert_eq!(package.skill_index.len(), 1);
    assert_eq!(package.view_as_index.len(), 1);
    assert_eq!(package.view_as_index[0].cost.as_deref(), Some("dddshilie"));
}

#[test]
fn shilie_exact_sum_supplies_shan_keeps_cards_public_and_draws() {
    let (mut state, players) = setup(20_260_811);
    let owner = players[0];
    let attacker = players[1];
    state.turn.current = attacker;
    let sha = inject(&mut state, attacker, CardKind::Sha, 7, 1);
    let threshold =
        (state.players.get(owner).unwrap().hp + state.players.get(attacker).unwrap().hp) as u8;
    let shown = inject(&mut state, owner, CardKind::Other, threshold, 2);
    inject_draw(&mut state, 3, 20);
    let hp = state.players.get(owner).unwrap().hp;
    let (state, offered) = submit(
        state,
        attacker,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        10,
    );
    let hand: std::collections::BTreeSet<_> = state
        .hand_cards(owner)
        .into_iter()
        .map(CardId::raw)
        .collect();
    let conversion = offered
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .find(|token| !hand.contains(token) && !matches!(*token, 0 | 1))
        .unwrap();
    let (state, sources) = choose(state, owner, vec![conversion], 11);
    assert_eq!(sources.choice.unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![shown.raw()], 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    assert!(state.hand_cards(owner).contains(&shown));
    assert!(state.cards.get(shown).unwrap().face_up);
    assert_eq!(state.hand_count(owner), 2);
}

#[test]
fn shilie_above_threshold_uses_virtual_sha_without_draw_and_is_once_per_turn() {
    let (mut state, players) = setup(20_260_812);
    let owner = players[0];
    let target = players[1];
    let threshold = (state.players.get(owner).unwrap().hp * 2) as u8;
    let shown = inject(&mut state, owner, CardKind::Other, threshold + 1, 3);
    let instance = skill(&state, owner);
    let hp = state.players.get(target).unwrap().hp;
    let (state, choice) = submit(
        state,
        owner,
        CommandKind::UseSkillCard {
            skill: instance,
            card_kind: CardKind::Sha.tag(),
            targets: vec![target],
            equip_slot: 0,
        },
        20,
    );
    assert_eq!(choice.choice.unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![shown.raw()], 21);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert_eq!(state.hand_cards(owner), vec![shown]);
    assert!(state.cards.get(shown).unwrap().face_up);
    assert_eq!(state.skills.get(instance).unwrap().use_count, 1);

    let (.., repeated) = submit(
        state,
        owner,
        CommandKind::UseSkillCard {
            skill: instance,
            card_kind: CardKind::Sha.tag(),
            targets: vec![target],
            equip_slot: 0,
        },
        22,
    );
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
}
}

mod original_extension_ddd_wuzhi {
//! Golden for 3D精选 吴质【灵涌/虚孝】（ABI 622）。

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DDDLINGYONG: u32 = 2_946_205_848;
const DDDXUXIAO: u32 = 3_539_713_105;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/wuzhi.skill");

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    serial: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_622_000 + serial),
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

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-wuzhi", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
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
    effects.extend(
        [DDDLINGYONG, DDDXUXIAO].map(|skill_def| EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, players)
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

#[test]
fn package_exposes_exact_abi_and_two_trigger_contracts() {
    let package = content_compiler::compile_source_named("ddd-wuzhi", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 622);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.trigger_index.len(), 2);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.event == "phase_any_skipped" && trigger.filter == "owner_alive" && trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.event == "cards_moved"
            && trigger.filter == "owner_alive_discarded_black_basic_to_discard"
            && !trigger.optional
    }));
}

#[test]
fn lingyong_observes_non_draw_skips_repeats_after_non_sha_and_stops_on_sha() {
    let (mut state, players) = setup();
    let owner = players[0];
    let sha = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Sha,
        Suit::Spade,
        1,
    );
    let other = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Other,
        Suit::Heart,
        2,
    );
    let mut tx = Transaction::begin(state);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::PhaseSkipped {
                current: players[1],
                phase: TurnPhaseTag::Prepare,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    assert_eq!(committed.output.choice.unwrap().kind, ChoiceKind::Confirm);
    let state = committed.state;

    let (state, use_offer) = resolve(state, owner, vec![1], false, 1);
    assert_eq!(use_offer.choice.unwrap().kind, ChoiceKind::Confirm);
    let (state, repeat_offer) = resolve(state, owner, vec![0], false, 2);
    assert_eq!(repeat_offer.choice.unwrap().kind, ChoiceKind::Confirm);
    let (state, finished) = resolve(state, owner, vec![1], false, 3);
    assert!(matches!(finished.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(other).unwrap().zone, discard);
    assert_eq!(state.cards.get(sha).unwrap().zone, discard);
}

#[test]
fn xuxiao_observes_another_players_discard_and_directly_places_bingliang_then_draws_two() {
    let (mut state, players) = setup();
    let owner = players[0];
    let discarder = players[1];
    let recipient = players[2];
    let black = inject(
        &mut state,
        discarder,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Club,
        3,
    );
    let before = state.hand_count(recipient);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: discarder,
                cards: vec![black],
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    assert_eq!(
        committed.output.choice.unwrap().kind,
        ChoiceKind::SelectCards
    );
    let state = committed.state;
    let (state, target_offer) = resolve(state, owner, vec![black.raw()], false, 4);
    assert_eq!(target_offer.choice.unwrap().kind, ChoiceKind::SelectTargets);
    let (state, result) = resolve(state, owner, vec![recipient.raw()], false, 5);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(recipient), before + 2);
    let judge = state.zones.find(ZoneKind::Judge, recipient).unwrap();
    assert_eq!(state.cards.get(black).unwrap().zone, judge);
    assert_eq!(
        state.cards.get(black).unwrap().kind,
        CardKind::TrickBingliang
    );
}
}

mod original_extension_ddd_xiahouxuan {
//! Golden for 3D精选 夏侯玄【朗怀/玄论】（ABI 638）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const LANGHUAI: u32 = 4_025_119_557;
const XUANLUN: u32 = 3_478_177_951;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/xiahouxuan.skill");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-xiahouxuan", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 638);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap())
        .unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_804, "identity-standard").unwrap();
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
    effects.extend(
        [LANGHUAI, XUANLUN].map(|skill_def| EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, players)
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
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

fn inject(state: &mut GameState, owner: PlayerId, serial: u32, suit: Suit) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_638_000 + serial),
        kind: CardKind::Other,
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

fn dispatch(state: GameState, event: DomainEvent) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new()).unwrap();
    match run_effects(&mut tx, effects) {
        Ok(_) | Err(EffectError::ChoiceRequested) => tx.commit().state,
        Err(error) => panic!("dispatch failed: {error:?}"),
    }
}

fn choose(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
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
                selections,
                cancelled: false,
            },
        },
    )
}

fn begin_xuanlun_settlement(state: GameState, owner: PlayerId) -> GameState {
    let skill = skill_id(&state, owner, XUANLUN);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddXiahouxuan {
            player: owner,
            action: 1,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), 4);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddXiahouxuan {
                player: owner,
                action: 2,
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    tx.commit().state
}

#[test]
fn langhuai_draws_present_then_missing_suit_counts_and_toggles() {
    let (mut state, players) = setup();
    let owner = players[0];
    inject(&mut state, owner, 1, Suit::Spade);
    inject(&mut state, owner, 2, Suit::Heart);
    inject(&mut state, owner, 3, Suit::Heart);
    state.turn.current = owner;
    state.turn.phase = Phase::Draw;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Draw,
        },
    );
    assert!(state.has_pending_choice());
    let (state, result) = choose(state, owner, vec![1], 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 5);
    assert_eq!(state.turn.draw_phase_fixed_count(), Some(0));
    assert_eq!(
        state
            .skills
            .get(skill_id(&state, owner, LANGHUAI))
            .unwrap()
            .mark,
        1
    );

    let mut state = state;
    state.turn.clear_draw_phase_fixed_count();
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Draw,
        },
    );
    let before = state.hand_count(owner);
    let mut suits = 0u8;
    for card in state.hand_cards(owner) {
        let card = state.cards.get(card).unwrap();
        suits |= match engine_core::query::effective_card_suit(&state, owner, card) {
            Suit::Spade => 1,
            Suit::Heart => 2,
            Suit::Club => 4,
            Suit::Diamond => 8,
            Suit::None => 0,
        };
    }
    let missing = 4 - suits.count_ones();
    let (state, result) = choose(state, owner, vec![1], 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + missing);
    assert_eq!(
        state
            .skills
            .get(skill_id(&state, owner, LANGHUAI))
            .unwrap()
            .mark,
        0
    );
}

#[test]
fn xuanlun_can_force_the_next_langhuai_then_forced_activation_clears_it() {
    let (state, players) = setup();
    let owner = players[0];
    let state = begin_xuanlun_settlement(state, owner);
    let (mut state, result) = choose(state, owner, vec![2], 10);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let xuanlun = skill_id(&state, owner, XUANLUN);
    assert_eq!(state.skills.get(xuanlun).unwrap().mark & 2, 2);

    state.turn.current = owner;
    state.turn.phase = Phase::Draw;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Draw,
        },
    );
    assert!(!state.has_pending_choice());
    assert_eq!(state.skills.get(xuanlun).unwrap().mark & 2, 0);
    assert_eq!(state.turn.draw_phase_fixed_count(), Some(0));
}

#[test]
fn xuanlun_places_four_selected_cards_on_top_and_bottom_in_requested_order() {
    let (state, players) = setup();
    let owner = players[0];
    let state = begin_xuanlun_settlement(state, owner);
    let (state, result) = choose(state, owner, vec![1], 20);
    assert!(result.choice.is_some());
    let cards = state.hand_cards(owner);
    assert_eq!(cards.len(), 4);
    let order = vec![
        cards[0].raw(),
        cards[1].raw(),
        0,
        cards[2].raw(),
        cards[3].raw(),
    ];
    let (state, result) = choose(state, owner, order, 21);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let pile = &state.zones.get(draw).unwrap().cards;
    assert_eq!(&pile[..2], &[cards[2], cards[3]]);
    assert_eq!(&pile[pile.len() - 2..], &[cards[1], cards[0]]);
}
}

mod original_extension_ddd_xianglang {
//! Golden for 3D精选 向朗【恰化/恂恂/腹笥/拓籍】（ABI 626）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const QIAHUA: u32 = 891_363_178;
const XUNXUN: u32 = 1_673_829_014;
const FUSI: u32 = 1_632_230_116;
const FUSI_GLOBAL: u32 = 2_198_174_483;
const TUOJI: u32 = 1_700_500_735;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/xianglang.skill");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-xianglang", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 626);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
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
    effects.extend([QIAHUA, FUSI, TUOJI].map(|def| EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(def),
    }));
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, players)
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
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

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, serial: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_626_000 + serial),
        kind,
        suit: Suit::Heart,
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

fn choose(
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

#[test]
fn qiahua_reveals_exact_hp_cards_and_grants_xunxun_for_turn() {
    let (mut state, players) = setup();
    let (owner, actor) = (players[0], players[1]);
    state.players.get_mut(actor).unwrap().hp = 2;
    state.turn.current = actor;
    let cards = [
        inject(&mut state, owner, CardKind::Shan, 1),
        inject(&mut state, owner, CardKind::Tao, 2),
    ];
    let qiahua = skill(&state, owner, QIAHUA);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddXianglang {
                player: owner,
                actor,
                action: 0,
                skill: qiahua,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = choose(
        tx.commit().state,
        owner,
        cards.iter().map(|card| card.raw()).collect(),
        1,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(cards
        .iter()
        .all(|card| state.cards.get(*card).unwrap().face_up));
    assert_eq!(
        state
            .skills
            .get(skill(&state, actor, XUNXUN))
            .unwrap()
            .temporary_until_turn_end_of,
        actor
    );
}

#[test]
fn fusi_confirmed_foreign_shown_sha_is_used_and_tuoji_draws_three() {
    let (mut state, players) = setup();
    let (owner, borrower, target) = (players[0], players[1], players[2]);
    let shown = inject(&mut state, owner, CardKind::Sha, 10);
    state.cards.get_mut(shown).unwrap().face_up = true;
    let fusi = skill(&state, owner, FUSI);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddXianglang {
            player: owner,
            actor: PlayerId::NONE,
            action: 1,
            skill: fusi,
        }],
    )
    .unwrap();
    let mut state = tx.commit().state;
    state.turn.current = borrower;
    state.turn.phase = Phase::Play;
    let helper = skill(&state, borrower, FUSI_GLOBAL);
    let target_hp = state.players.get(target).unwrap().hp;
    let (state, source_choice) = submit(
        state,
        borrower,
        CommandKind::UseSkillCard {
            skill: helper,
            card_kind: CardKind::Sha.tag(),
            targets: vec![target],
            equip_slot: 0,
        },
        10,
    );
    assert_eq!(source_choice.choice.unwrap().observer, borrower);
    let (state, approval) = choose(state, borrower, vec![shown.raw()], 11);
    assert_eq!(approval.choice.unwrap().observer, owner);
    let (state, tuoji) = choose(state, owner, vec![0], 12);
    assert_eq!(
        tuoji.choice.as_ref().map(|choice| choice.observer),
        Some(owner),
        "result={:?} continuation={:?}",
        tuoji.result,
        state.resolution.continuation_kind
    );
    let before = state.hand_count(owner);
    let (state, done) = choose(state, owner, vec![1], 13);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert_eq!(state.hand_count(owner), before + 3);
    assert!(!state.hand_cards(owner).contains(&shown));
}

#[test]
fn fusi_face_up_cards_ignore_limit_and_all_shown_owner_turn_locks_other_hands() {
    let (mut state, players) = setup();
    let (owner, other) = (players[0], players[1]);
    let shown = inject(&mut state, owner, CardKind::Tao, 20);
    state.cards.get_mut(shown).unwrap().face_up = true;
    state.turn.current = owner;
    assert!(engine_core::query::card_ignored_for_hand_limit(
        &state, owner, shown
    ));
    assert!(engine_core::query::hand_card_use_response_cancelled_for(
        &state, other
    ));
}
}

mod original_extension_ddd_xinxianying {
//! Golden for 3D精选 辛宪英【洞察/智解】（ABI 625）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DONGCHA: u32 = 3_036_718_178;
const ZHIJIE: u32 = 1_168_942_044;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/xinxianying.skill");

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    kind: CardKind,
    suit: Suit,
    serial: u32,
) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_625_000 + serial),
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

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-xinxianying", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
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
    effects.extend([DONGCHA, ZHIJIE].map(|def| EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(def),
    }));
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, players)
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
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

fn submit(
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

fn place_two_jian(
    mut state: GameState,
    owner: PlayerId,
    actor: PlayerId,
    first: CardId,
    second: CardId,
) -> GameState {
    let skill = skill_id(&state, owner, DONGCHA);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddXinxianying {
                player: owner,
                actor,
                action: 0,
                skill,
            }]
        ),
        Err(EffectError::ChoiceRequested)
    );
    state = tx.commit().state;
    let (state, _) = resolve(state, owner, vec![first.raw()], 1);
    let (state, done) = resolve(state, actor, vec![second.raw()], 2);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn dongcha_stores_then_owner_and_turn_player_take_one_jian_each() {
    let (mut state, players) = setup();
    let (owner, actor) = (players[0], players[1]);
    let first = inject(&mut state, owner, CardKind::Other, Suit::Heart, 1);
    let second = inject(&mut state, actor, CardKind::Other, Suit::Club, 2);
    let state = place_two_jian(state, owner, actor, first, second);
    assert_eq!(state.skill_pile_cards(owner, DONGCHA).len(), 2);
    let skill = skill_id(&state, owner, DONGCHA);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddXinxianying {
                player: owner,
                actor,
                action: 1,
                skill,
            }]
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let (state, _) = resolve(state, owner, vec![first.raw()], 3);
    let (state, done) = resolve(state, actor, vec![second.raw()], 4);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert!(state.skill_pile_cards(owner, DONGCHA).is_empty());
    assert_eq!(state.cards.get(first).unwrap().owner, owner);
    assert_eq!(state.cards.get(second).unwrap().owner, actor);
}

#[test]
fn zhijie_same_color_jian_respond_as_shan_and_draw_two() {
    let (mut state, players) = setup();
    let (owner, actor) = (players[0], players[1]);
    let first = inject(&mut state, owner, CardKind::Other, Suit::Heart, 10);
    let second = inject(&mut state, actor, CardKind::Other, Suit::Diamond, 11);
    let attack = inject(&mut state, actor, CardKind::Sha, Suit::Spade, 12);
    for serial in 20..22 {
        let draw = state
            .zones
            .find(ZoneKind::DrawPile, PlayerId::NONE)
            .unwrap();
        let card = state.cards.alloc(Card {
            id: CardId::NONE,
            def: CardDefId::new(99_625_000 + serial),
            kind: CardKind::Other,
            suit: Suit::Club,
            rank: 7,
            weapon_range: 0,
            zone: draw,
            owner: PlayerId::NONE,
            face_up: true,
        });
        state.zones.get_mut(draw).unwrap().cards.push(card);
    }
    let mut state = place_two_jian(state, owner, actor, first, second);
    state.turn.current = actor;
    state.turn.phase = Phase::Play;
    let hp = state.players.get(owner).unwrap().hp;
    let (state, response) = submit(
        state,
        actor,
        CommandKind::PlayCard {
            card: attack,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        10,
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::RespondToSha
    );
    let conversion = response
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .find(|token| !matches!(*token, 0 | 1))
        .unwrap();
    let (state, source_choice) = resolve(state, owner, vec![conversion], 11);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::MultiCardViewAsResponse
    );
    let offered: Vec<_> = source_choice
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(offered, vec![first.raw(), second.raw()]);
    let (state, done) = resolve(state, owner, vec![first.raw(), second.raw()], 12);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    assert_eq!(state.hand_count(owner), 2);
    assert!(state.skill_pile_cards(owner, DONGCHA).is_empty());
}
}

mod original_extension_ddd_xuelingyun {
//! Golden for 3D精选 薛灵芸【涟洏/暗织】（ABI 636）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const LIANER: u32 = 1_083_711_762;
const ANZHI: u32 = 1_654_153_098;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/xuelingyun.skill");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-xuelingyun", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 636);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.trigger_index.len(), 7);
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&standard).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 4, 20_260_803, "identity-standard").unwrap();
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
    effects.extend([LIANER, ANZHI].map(|skill_def| EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(skill_def),
    }));
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, players)
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
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

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    serial: u32,
    kind: CardKind,
    suit: Suit,
    rank: u8,
) -> CardId {
    let zone_owner = if matches!(zone_kind, ZoneKind::Hand) {
        owner
    } else {
        PlayerId::NONE
    };
    let zone = state.zones.find(zone_kind, zone_owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_636_000 + serial),
        kind,
        suit,
        rank,
        weapon_range: 0,
        zone,
        owner: zone_owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(
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

#[test]
fn lianer_reclaims_red_material_and_enforces_rank_ceiling_until_turn_end() {
    let (mut state, players) = setup();
    let owner = players[0];
    let skill = skill_id(&state, owner, LIANER);
    let resolved = inject(
        &mut state,
        owner,
        ZoneKind::DiscardPile,
        1,
        CardKind::Sha,
        Suit::Heart,
        7,
    );
    let lower = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        2,
        CardKind::Shan,
        Suit::Club,
        6,
    );
    let equal = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        3,
        CardKind::Tao,
        Suit::Diamond,
        7,
    );
    let higher = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        4,
        CardKind::Sha,
        Suit::Spade,
        8,
    );
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddXuelingyun {
            player: owner,
            actor: owner,
            card: resolved,
            cards: vec![resolved],
            action: 0,
            skill,
        }],
    )
    .unwrap();
    assert!(tx.state().hand_cards(owner).contains(&resolved));
    assert_eq!(tx.state().skills.get(skill).unwrap().mark, 7);
    assert!(!engine_core::query::card_use_cancelled_for_card(
        tx.state(),
        owner,
        lower
    ));
    assert!(engine_core::query::card_use_cancelled_for_card(
        tx.state(),
        owner,
        equal
    ));
    assert!(engine_core::query::card_use_cancelled_for_card(
        tx.state(),
        owner,
        higher
    ));

    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddXuelingyun {
            player: owner,
            actor: owner,
            card: CardId::NONE,
            cards: Vec::new(),
            action: 1,
            skill,
        }],
    )
    .unwrap();
    assert!(!engine_core::query::card_use_cancelled_for_card(
        tx.state(),
        owner,
        higher
    ));
}

#[test]
fn anzhi_excludes_only_the_first_black_card_target_in_the_armed_turn() {
    let (mut state, players) = setup();
    let owner = players[0];
    let actor = players[1];
    let target = players[2];
    let skill = skill_id(&state, owner, ANZHI);
    let black = inject(
        &mut state,
        actor,
        ZoneKind::Hand,
        5,
        CardKind::Sha,
        Suit::Spade,
        9,
    );
    state.turn.current = actor;
    state.turn.phase = Phase::Play;

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddXuelingyun {
            player: owner,
            actor,
            card: black,
            cards: Vec::new(),
            action: 2,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddXuelingyun {
                player: owner,
                actor,
                card: CardId::NONE,
                cards: Vec::new(),
                action: 3,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (mut state, result) = choose(tx.commit().state, owner, vec![target.raw()], 1);
    assert!(result.choice.is_none());
    state.resolution.resume_kind = ContinuationKind::ResumePlayedSha;
    state.resolution.resume_payload = vec![actor.raw(), target.raw(), 1, 0, 1, black.raw()];

    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: actor,
            card: black,
            targets: vec![target],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(
        tx.state().resolution.resume_payload.get(6),
        Some(&target.raw())
    );
    assert_eq!(tx.state().skills.get(skill).unwrap().state_blob.len(), 13);
    assert_eq!(
        &tx.state().skills.get(skill).unwrap().state_blob[5..],
        &[0; 8]
    );

    tx.state_mut().resolution.resume_payload.truncate(6);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: actor,
            card: black,
            targets: vec![target],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().resolution.resume_payload.len(), 6);
}
}

mod original_extension_ddd_xujing {
//! Golden for 3D精选 许靖【虚誉/实荐】（ABI 623）。

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DDDXUYU: u32 = 3_210_825_357;
const DDDSHIJIAN: u32 = 2_262_569_305;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/xujing.skill");

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    serial: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_623_000 + serial),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-xujing", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
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
    effects.extend(
        [DDDXUYU, DDDSHIJIAN].map(|skill_def| EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, players)
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

fn resolved(source: PlayerId, card: CardId, targets: Vec<PlayerId>) -> DomainEvent {
    DomainEvent::CardResolved {
        source,
        card,
        kind: CardKind::Other.tag(),
        targets,
        damaged_targets: Vec::new(),
        source_cards: vec![card],
        all_sources_were_hand: true,
    }
}

#[test]
fn package_exposes_exact_abi_and_six_trigger_contracts() {
    let package = content_compiler::compile_source_named("ddd-xujing", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 623);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.trigger_index.len(), 6);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.event == "card_resolved"
            && trigger.filter
                == "owner_alive_other_source_second_play_card_resolved_hand_equip_nonempty"
            && trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.event == "card_used"
            && trigger.filter == "ddd_shijian_watched_source_card_used_targeted"
            && !trigger.optional
    }));
}

#[test]
fn xuyu_draws_then_forces_one_discard_after_the_next_resolved_card() {
    let (mut state, players) = setup();
    let owner = players[0];
    let first = inject(&mut state, owner, ZoneKind::Hand, CardKind::Other, 1);
    let second = inject(&mut state, owner, ZoneKind::Hand, CardKind::Other, 2);
    inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Other,
        3,
    );
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[resolved(owner, first, Vec::new())],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let (state, accepted) = resolve(state, owner, vec![1], 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 1);

    let mut tx = Transaction::begin(state);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[resolved(owner, second, Vec::new())],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    assert_eq!(
        committed.output.choice.unwrap().kind,
        ChoiceKind::SelectCards
    );
    let (state, next_offer) = resolve(committed.state, owner, vec![first.raw()], 2);
    assert_eq!(next_offer.choice.unwrap().kind, ChoiceKind::Confirm);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(first).unwrap().zone, discard);
    let (state, done) = resolve(state, owner, vec![0], 3);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
}

#[test]
fn shijian_gifts_after_second_play_card_and_replays_next_targeted_sha() {
    let (mut state, players) = setup();
    let (owner, source, target) = (players[0], players[1], players[2]);
    state.turn.current = source;
    state.turn.phase = Phase::Play;
    let gift = inject(&mut state, owner, ZoneKind::Hand, CardKind::Other, 4);
    let first = inject(&mut state, source, ZoneKind::Hand, CardKind::Other, 5);
    let second = inject(&mut state, source, ZoneKind::Hand, CardKind::Other, 6);
    let sha = inject(&mut state, source, ZoneKind::Hand, CardKind::Sha, 7);
    let mut tx = Transaction::begin(state);
    dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: source,
            card: first,
            targets: Vec::new(),
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: source,
            card: second,
            targets: Vec::new(),
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[resolved(source, second, Vec::new())],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let (state, card_offer) = resolve(state, owner, vec![1], 10);
    assert_eq!(card_offer.choice.unwrap().kind, ChoiceKind::SelectCards);
    let (state, armed) = resolve(state, owner, vec![gift.raw()], 11);
    assert!(matches!(armed.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(gift).unwrap().owner, source);

    let hp = state.players.get(target).unwrap().hp;
    let revision = state.revision;
    let (state, offered) = apply_command(
        state,
        Command {
            actor: source,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(12),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert_eq!(offered.choice.unwrap().kind, ChoiceKind::Custom);
    let (state, result) = resolve(state, owner, vec![0], 13);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 2);
}
}

mod original_extension_ddd_yujin {
//! Golden for 3D精选 于禁【整军】（ABI 627）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZHENGJUN: u32 = 2_986_915_915;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/yujin.skill");

fn setup() -> (GameState, Vec<PlayerId>, engine_api::SkillId) {
    let package = content_compiler::compile_source_named("ddd-yujin", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 627);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(ZHENGJUN),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx.state().players.get(players[0]).unwrap().skills[0];
    let mut state = tx.commit().state;
    state.turn.current = players[0];
    state.turn.phase = Phase::Play;
    (state, players, skill)
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    serial: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_627_000 + serial),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 3,
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

#[test]
fn hp_category_recovers_matching_player_and_is_recorded() {
    let (mut state, players, skill) = setup();
    let (owner, actor) = (players[0], players[1]);
    state.players.get_mut(owner).unwrap().hp = 3;
    state.players.get_mut(actor).unwrap().hp = 3;
    state.players.get_mut(actor).unwrap().max_hp = 4;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddYujin {
                player: owner,
                actor,
                target: PlayerId::NONE,
                move_mask: 0,
                action: 0,
                candidates: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = choose(tx.commit().state, owner, vec![0], 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(actor).unwrap().hp, 4);
    assert_eq!(state.skills.get(skill).unwrap().play_phase_category_mask, 1);
}

#[test]
fn one_move_event_can_resolve_hand_then_equipment_categories() {
    let (mut state, players, skill) = setup();
    let (owner, actor, destination) = (players[0], players[1], players[2]);
    inject(&mut state, owner, ZoneKind::Hand, CardKind::Shan, 1);
    inject(&mut state, actor, ZoneKind::Hand, CardKind::Tao, 2);
    inject(&mut state, owner, ZoneKind::Equip, CardKind::Armor, 3);
    let weapon = inject(&mut state, actor, ZoneKind::Equip, CardKind::Weapon, 4);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddYujin {
                player: owner,
                actor,
                target: actor,
                move_mask: 1 | 8,
                action: 1,
                candidates: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, _) = choose(tx.commit().state, owner, vec![actor.raw()], 10);
    let (state, _) = choose(state, owner, vec![0], 11); // actor draws one
    assert_eq!(state.resolution.pending_observer, owner.raw());
    let (state, _) = choose(state, owner, vec![actor.raw()], 12);
    let (state, _) = choose(state, owner, vec![weapon.raw()], 13);
    let (state, result) = choose(state, owner, vec![destination.raw()], 14);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let destination_equip = state.zones.find(ZoneKind::Equip, destination).unwrap();
    assert_eq!(state.cards.get(weapon).unwrap().zone, destination_equip);
    assert_eq!(
        state.skills.get(skill).unwrap().play_phase_category_mask,
        2 | 4
    );
}

#[test]
fn hand_discard_branch_is_chosen_by_owner_but_paid_by_changed_player() {
    let (mut state, players, skill) = setup();
    let (owner, actor) = (players[0], players[1]);
    inject(&mut state, owner, ZoneKind::Hand, CardKind::Shan, 20);
    let actor_card = inject(&mut state, actor, ZoneKind::Hand, CardKind::Tao, 21);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddYujin {
                player: owner,
                actor,
                target: PlayerId::NONE,
                move_mask: 1,
                action: 1,
                candidates: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, _) = choose(tx.commit().state, owner, vec![actor.raw()], 20);
    let (state, branch) = choose(state, owner, vec![1], 21);
    assert_eq!(branch.choice.unwrap().observer, actor);
    let (state, result) = choose(state, actor, vec![actor_card.raw()], 22);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.hand_cards(actor).contains(&actor_card));
    assert_eq!(state.skills.get(skill).unwrap().play_phase_category_mask, 2);
}

#[test]
fn cards_moved_trigger_captures_destination_owner_and_both_zone_categories() {
    let (mut state, players, _) = setup();
    let (owner, actor) = (players[0], players[1]);
    inject(&mut state, owner, ZoneKind::Hand, CardKind::Shan, 30);
    inject(&mut state, actor, ZoneKind::Hand, CardKind::Tao, 31);
    inject(&mut state, owner, ZoneKind::Equip, CardKind::Armor, 32);
    let weapon = inject(&mut state, actor, ZoneKind::Hand, CardKind::Weapon, 33);
    let from = state.zones.find(ZoneKind::Hand, actor).unwrap();
    let to = state.zones.find(ZoneKind::Equip, actor).unwrap();
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: vec![weapon],
                to,
                from: Some(from),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, accepted) = choose(tx.commit().state, owner, vec![1], 30);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    let choice = accepted.choice.unwrap();
    assert_eq!(choice.observer, owner);
    assert_eq!(
        choice
            .options
            .into_iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        vec![actor.raw()]
    );
    assert!(state.has_pending_choice());
}
}

mod original_extension_ddd_zhangkai {
//! Golden for 3D精选 张闿【劫行/拜泪】（ABI 639）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const JIEXING: u32 = 3_377_566_778;
const BAILEI: u32 = 2_689_073_031;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/zhangkai.skill");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-zhangkai", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 639);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap())
        .unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_804, "identity-standard").unwrap();
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
    effects.extend([JIEXING, BAILEI].map(|skill_def| EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(skill_def),
    }));
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.max_hp = 4;
        participant.hp = 3;
    }
    (tx.commit().state, players)
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
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

fn choose(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
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
                selections,
                cancelled,
            },
        },
    )
}

fn inject(state: &mut GameState, owner: PlayerId, zone: ZoneKind, kind: CardKind) -> CardId {
    let zone = state.zones.find(zone, owner).unwrap_or_else(|| {
        state
            .zones
            .find(ZoneKind::Processing, PlayerId::NONE)
            .unwrap()
    });
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_639_000),
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

#[test]
fn jiexing_steals_recovery_then_only_a_later_recovery_resets_it() {
    let (mut state, players) = setup();
    let owner = players[0];
    let other = players[1];
    state.players.get_mut(owner).unwrap().hp = 1;
    state.players.get_mut(other).unwrap().hp = 1;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::RecoverHp {
                source: other,
                player: other,
                amount: 2,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = tx.commit().state;
    let (state, result) = choose(state, owner, vec![1], false, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert_eq!(state.players.get(other).unwrap().hp, 1);
    let jiexing = skill_id(&state, owner, JIEXING);
    assert_eq!(state.skills.get(jiexing).unwrap().lifetime_fire_count, 1);
    assert_eq!(state.skills.get(jiexing).unwrap().content_flags, 0);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::RecoverHp {
            source: owner,
            player: owner,
            amount: 1,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().skills.get(jiexing).unwrap().lifetime_fire_count,
        0
    );
}

#[test]
fn jiexing_cancels_equipment_use_equips_material_and_later_use_resets() {
    let (mut state, players) = setup();
    let owner = players[0];
    let other = players[1];
    let old = inject(&mut state, owner, ZoneKind::Equip, CardKind::Armor);
    let card = inject(&mut state, other, ZoneKind::Hand, CardKind::Armor);
    let jiexing = skill_id(&state, owner, JIEXING);
    state.turn.current = other;
    state.turn.phase = Phase::Play;
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: other,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(10),
            kind: CommandKind::PlayCard {
                card,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    let (state, result) = choose(state, owner, vec![1], false, 11);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state.zones.get(equip).unwrap().cards.contains(&card));
    assert!(state.zones.get(discard).unwrap().cards.contains(&old));
    assert_eq!(state.skills.get(jiexing).unwrap().lifetime_fire_count, 1);

    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardResolved {
            source: owner,
            card,
            kind: CardKind::Armor.tag(),
            targets: Vec::new(),
            damaged_targets: Vec::new(),
            source_cards: vec![card],
            all_sources_were_hand: true,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(
        tx.state().skills.get(jiexing).unwrap().lifetime_fire_count,
        0
    );
}

#[test]
fn bailei_damage_that_kills_removes_bailei() {
    let (mut state, players) = setup();
    let owner = players[0];
    let target = players[1];
    state.players.get_mut(target).unwrap().hp = 1;
    let bailei = skill_id(&state, owner, BAILEI);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddZhangkai {
                player: owner,
                actor: owner,
                card: CardId::NONE,
                cards: Vec::new(),
                amount: 0,
                action: 4,
                skill: bailei,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let state = tx.commit().state;
    let (state, result) = choose(state, owner, vec![target.raw()], false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.players.get(target).unwrap().alive);
    assert!(state.skills.get(bailei).is_none());
}

#[test]
fn bailei_gains_from_the_unique_most_equipped_target() {
    let (mut state, players) = setup();
    let owner = players[0];
    let target = players[1];
    let equipment = inject(&mut state, target, ZoneKind::Equip, CardKind::Weapon);
    let bailei = skill_id(&state, owner, BAILEI);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddZhangkai {
                player: owner,
                actor: owner,
                card: CardId::NONE,
                cards: Vec::new(),
                amount: 0,
                action: 4,
                skill: bailei,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, result) = choose(tx.commit().state, owner, vec![target.raw()], false, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    let (state, result) = choose(state, owner, vec![equipment.raw()], false, 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(owner).contains(&equipment));
    assert!(state.skills.get(bailei).is_some());
}
}

mod original_extension_ddd_zhaoang {
//! Golden for 3D精选 赵昂【奋计】（ABI 631）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const FENJI: u32 = 782_265_781;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/zhaoang.skill");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup(phase: Phase) -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-zhaoang", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 631);
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&standard).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(FENJI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = phase;
    (tx.commit().state, players)
}

fn skill_id(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == FENJI)
        .unwrap()
}

fn choose(
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

fn begin(state: GameState, owner: PlayerId, action: u8) -> (GameState, Vec<u32>) {
    let skill = skill_id(&state, owner);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddZhaoang {
                player: owner,
                action,
                stage: 0,
                card: CardId::NONE,
                kind: 0,
                count: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    let options = committed
        .output
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    (committed.state, options)
}

fn inject(state: &mut GameState, owner: PlayerId, serial: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_631_000 + serial),
        kind: CardKind::Other,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

#[test]
fn draw_phase_wanjian_replaces_normal_draw_with_total_damage() {
    let (state, players) = setup(Phase::Draw);
    let owner = players[0];
    let hp_before = [
        state.players.get(players[1]).unwrap().hp,
        state.players.get(players[2]).unwrap().hp,
    ];
    let (state, _) = begin(state, owner, 0);
    let (state, result) = choose(state, owner, vec![CardKind::TrickWanjian.tag().into()], 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.draw_phase_fixed_count(), Some(0));
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.players.get(players[1]).unwrap().hp, hp_before[0] - 1);
    assert_eq!(state.players.get(players[2]).unwrap().hp, hp_before[1] - 1);
}

#[test]
fn used_trick_name_is_permanently_removed_from_later_choices() {
    let (state, players) = setup(Phase::Draw);
    let owner = players[0];
    let (state, _) = begin(state, owner, 0);
    let (mut state, result) = choose(state, owner, vec![CardKind::TrickWuzhong.tag().into()], 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state.turn.clear_draw_phase_fixed_count();
    let (_, options) = begin(state, owner, 0);
    assert!(!options.contains(&u32::from(CardKind::TrickWuzhong.tag())));
}

#[test]
fn discard_phase_uses_declared_target_count_and_skips_normal_discard() {
    let (mut state, players) = setup(Phase::Discard);
    let owner = players[0];
    for serial in 0..4 {
        inject(&mut state, owner, serial);
    }
    let (state, _) = begin(state, owner, 1);
    let (state, result) = choose(state, owner, vec![CardKind::TrickTaoyuan.tag().into()], 3);
    let choice = result
        .choice
        .as_ref()
        .expect("奋计应要求弃置三个目标数的牌");
    assert_eq!(choice.min_count, 3);
    assert_eq!(choice.max_count, 3);
    let cards = choice
        .options
        .iter()
        .take(3)
        .map(|option| option.token)
        .collect();
    let (state, result) = choose(state, owner, cards, 4);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 1);
    assert_eq!(state.turn.flag(turn_flag::SKIP_DISCARD), 1);
}
}

mod original_extension_ddd_zhenji {
//! Golden for 3D精选 甄宓【淼形/浮世】（ABI 630）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const MIAOXING: u32 = 505_834_181;
const MIAOXING_1: u32 = 919_529_013;
const MIAOXING_2: u32 = 2_949_101_455;
const FUSHI: u32 = 1_483_827_339;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/zhenji.skill");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-zhenji", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 630);
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&standard).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    tx.state_mut()
        .players
        .get_mut(players[0])
        .unwrap()
        .character_def = 25;
    tx.state_mut()
        .players
        .get_mut(players[1])
        .unwrap()
        .character_def = 1;
    tx.state_mut()
        .players
        .get_mut(players[2])
        .unwrap()
        .character_def = 2;
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
    effects.extend([MIAOXING, FUSHI].map(|skill_def| EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(skill_def),
    }));
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, players)
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
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

fn inject(state: &mut GameState, owner: PlayerId, serial: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_630_000 + serial),
        kind: CardKind::Other,
        suit: Suit::Heart,
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

fn initialize(mut state: GameState, owner: PlayerId, count: u32) -> (GameState, Vec<CardId>) {
    let cards: Vec<_> = (0..count)
        .map(|serial| inject(&mut state, owner, serial))
        .collect();
    let skill = skill_id(&state, owner, MIAOXING);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddZhenji {
                player: owner,
                target: PlayerId::NONE,
                action: 0,
                stage: 0,
                piles: Vec::new(),
                cards: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = choose(tx.commit().state, owner, vec![0], 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    (state, cards)
}

#[test]
fn miaoxing_triples_initial_hand_then_keeps_one_group_and_hides_two() {
    let (state, players) = setup();
    let owner = players[0];
    assert_eq!(
        engine_core::query::initial_hand_count_for(&state, owner, 4),
        12
    );
    let (state, cards) = initialize(state, owner, 12);
    assert_eq!(state.hand_cards(owner), cards[..4]);
    assert_eq!(
        state.hidden_skill_pile_cards(owner, MIAOXING_1),
        cards[4..8]
    );
    assert_eq!(state.hidden_skill_pile_cards(owner, MIAOXING_2), cards[8..]);
}

#[test]
fn miaoxing_balances_both_selected_piles_to_current_hand_count() {
    let (state, players) = setup();
    let owner = players[0];
    let (mut state, _) = initialize(state, owner, 9);
    let pile_1 = state.skill_pile_cards(owner, MIAOXING_1);
    let pile_2_zone = state
        .skill_piles
        .iter()
        .find(|pile| pile.owner == owner && pile.skill_def == MIAOXING_2)
        .unwrap()
        .zone;
    for card in &pile_1[..2] {
        let from = state.cards.get(*card).unwrap().zone;
        state
            .zones
            .get_mut(from)
            .unwrap()
            .cards
            .retain(|candidate| candidate != card);
        state.zones.get_mut(pile_2_zone).unwrap().cards.push(*card);
        state.cards.get_mut(*card).unwrap().zone = pile_2_zone;
    }
    let skill = skill_id(&state, owner, MIAOXING);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddZhenji {
                player: owner,
                target: PlayerId::NONE,
                action: 1,
                stage: 0,
                piles: Vec::new(),
                cards: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, result) = choose(tx.commit().state, owner, vec![1, 2], 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let discard = result.choice.as_ref().unwrap();
    assert_eq!(discard.observer, owner);
    assert_eq!(discard.options.len(), 5);
    let selected: Vec<_> = discard
        .options
        .iter()
        .take(2)
        .map(|option| option.token)
        .collect();
    let (state, result) = choose(state, owner, selected, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.skill_pile_cards(owner, MIAOXING_1).len(), 3);
    assert_eq!(state.skill_pile_cards(owner, MIAOXING_2).len(), 3);
}

#[test]
fn fushi_exchanges_owner_then_optional_male_hand_with_the_chosen_water() {
    let (state, players) = setup();
    let (owner, male) = (players[0], players[1]);
    let (mut state, cards) = initialize(state, owner, 6);
    let male_cards = vec![inject(&mut state, male, 20), inject(&mut state, male, 21)];
    let skill = skill_id(&state, owner, FUSHI);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddZhenji {
                player: owner,
                target: PlayerId::NONE,
                action: 2,
                stage: 0,
                piles: Vec::new(),
                cards: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, owner_target) = choose(tx.commit().state, owner, vec![1], 4);
    assert_eq!(state.hand_cards(owner), cards[2..4]);
    assert_eq!(state.skill_pile_cards(owner, MIAOXING_1), cards[..2]);
    assert_eq!(owner_target.choice.as_ref().unwrap().observer, owner);
    let (state, male_pile) = choose(state, owner, vec![male.raw()], 5);
    assert_eq!(male_pile.choice.as_ref().unwrap().observer, male);
    let (state, done) = choose(state, male, vec![1], 6);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_cards(male), cards[..2]);
    assert_eq!(state.skill_pile_cards(owner, MIAOXING_1), male_cards);
}
}

mod original_extension_ddd_zhouchu {
//! Golden for 3D精选 周处【侠横】（ABI 618）。

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DDDXIAHENG: u32 = 3_492_764_267;
const STATE_MAGIC: u32 = 0xDDD0_6180;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/zhouchu.skill");

fn setup(seed: u64) -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-zhouchu", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(DDDXIAHENG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, players)
}

fn inject(state: &mut GameState, owner: PlayerId, serial: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_100 + serial),
        kind: CardKind::Shan,
        suit: Suit::Club,
        rank: 8,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn skill(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
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
                .is_some_and(|skill| skill.def.raw() == DDDXIAHENG)
        })
        .unwrap()
}

fn choose(
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

fn start(state: GameState, owner: PlayerId) -> GameState {
    let instance = skill(&state, owner);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddXiaheng {
                player: owner,
                discard_target: PlayerId::NONE,
                damage_target: PlayerId::NONE,
                action: 0,
                skill: instance,
            }],
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    );
    tx.commit().state
}

#[test]
fn package_exposes_exact_abi_and_locked_trigger() {
    let package = content_compiler::compile_source_named("ddd-zhouchu", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 618);
    assert_eq!(package.skill_index.len(), 1);
    assert_eq!(package.trigger_index.len(), 1);
    assert!(!package.trigger_index[0].optional);
}

#[test]
fn xiaheng_forces_target_discard_then_damage_and_max_hp_loss() {
    let (mut state, players) = setup(20_260_806);
    let owner = players[0];
    let discard_target = players[1];
    let damage_target = players[2];
    let cards = [
        inject(&mut state, discard_target, 1),
        inject(&mut state, discard_target, 2),
        inject(&mut state, discard_target, 3),
    ];
    let old_hp = state.players.get(damage_target).unwrap().hp;
    let old_max_hp = state.players.get(owner).unwrap().max_hp;

    let state = start(state, owner);
    let (state, result) = choose(state, owner, vec![discard_target.raw()], 10);
    let choice = result.choice.unwrap();
    assert_eq!(choice.observer, discard_target);
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    let (state, result) = choose(
        state,
        discard_target,
        vec![cards[0].raw(), cards[1].raw()],
        11,
    );
    let choice = result.choice.unwrap();
    assert_eq!(choice.observer, owner);
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    let (state, result) = choose(state, owner, vec![damage_target.raw()], 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(discard_target), 1);
    assert_eq!(state.players.get(damage_target).unwrap().hp, old_hp - 1);
    assert_eq!(state.players.get(owner).unwrap().max_hp, old_max_hp - 1);
}

#[test]
fn xiaheng_same_target_removes_the_skill_after_damage() {
    let (mut state, players) = setup(20_260_807);
    let owner = players[0];
    let target = players[1];
    let cards = [inject(&mut state, target, 4), inject(&mut state, target, 5)];
    let old_hp = state.players.get(target).unwrap().hp;

    let state = start(state, owner);
    let (state, _) = choose(state, owner, vec![target.raw()], 20);
    let (state, _) = choose(state, target, vec![cards[0].raw(), cards[1].raw()], 21);
    let (state, result) = choose(state, owner, vec![target.raw()], 22);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, old_hp - 1);
    assert!(state.players.get(owner).unwrap().skills.iter().all(|id| {
        state
            .skills
            .get(*id)
            .is_none_or(|instance| instance.def.raw() != DDDXIAHENG)
    }));
}

#[test]
fn third_distinct_damage_target_deletes_future_penalties() {
    let (mut state, players) = setup(20_260_808);
    let owner = players[0];
    let instance = skill(&state, owner);
    let blob = &mut state.skills.get_mut(instance).unwrap().state_blob;
    blob.extend_from_slice(&STATE_MAGIC.to_le_bytes());
    blob.push(0);
    blob.extend_from_slice(&2u32.to_le_bytes());
    blob.extend_from_slice(&players[1].raw().to_le_bytes());
    blob.extend_from_slice(&players[2].raw().to_le_bytes());
    let old_max_hp = state.players.get(owner).unwrap().max_hp;

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddXiaheng {
            player: owner,
            discard_target: players[1],
            damage_target: players[3],
            action: 2,
            skill: instance,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(state.players.get(owner).unwrap().max_hp, old_max_hp - 1);
    assert_eq!(state.skills.get(instance).unwrap().state_blob[4], 1);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddXiaheng {
            player: owner,
            discard_target: players[2],
            damage_target: players[2],
            action: 2,
            skill: instance,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(state.players.get(owner).unwrap().max_hp, old_max_hp - 1);
    assert!(state.skills.get(instance).is_some());
}
}

mod original_extension_wandianlunsha_feishi {
//! Golden for archived extension 玩点论杀 费诗【率言】（ABI 608）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NSSHUAIYAN: u32 = 2_978_279_193;
const SOURCE: &str = include_str!("../../../content/original/ported/wandianlunsha/feishi.skill");

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

fn open_shuaiyan(
    tx: &mut Transaction,
    owner: PlayerId,
    recovered: PlayerId,
    skill: engine_api::SkillId,
) {
    assert_eq!(
        run_effects(
            tx,
            vec![EffectRequest::ResolveNsshuaiyan {
                player: owner,
                recovered_player: recovered,
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
}

#[test]
fn recovered_player_chooses_owner_draw_or_owner_discards_their_card() {
    let package = content_compiler::compile_source_named("wandianlunsha-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let recovered = players[1];
    tx.state_mut().turn.current = players[2];
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NSSHUAIYAN),
        }],
    )
    .unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == NSSHUAIYAN)
        .unwrap()
        .id;

    let hand_before = tx.state().hand_count(owner);
    open_shuaiyan(&mut tx, owner, recovered, skill);
    let state = choose(tx.commit().state, recovered, vec![0], 6081);
    assert_eq!(state.hand_count(owner), hand_before + 1);

    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, recovered).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_081),
        kind: CardKind::Sha,
        suit: Suit::Club,
        rank: 8,
        weapon_range: 0,
        zone: hand,
        owner: recovered,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    open_shuaiyan(&mut tx, owner, recovered, skill);
    let state = choose(tx.commit().state, recovered, vec![1], 6082);
    assert!(state.has_pending_choice());
    let state = choose(state, owner, vec![card.raw()], 6083);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state.zones.get(discard).unwrap().cards.contains(&card));
}
}

mod original_extension_wandianlunsha_hanlong {
//! Golden for archived extension 玩点论杀 韩龙【伺机/刺酋】（ABI 609）。

use engine_api::{CardDefId, CardId, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SIJI: u32 = 2_867_665_051;
const CIQIU: u32 = 3_777_188_862;
const CIQIU_DYING: u32 = 3_613_978_472;
const SOURCE: &str = include_str!("../../../content/original/ported/wandianlunsha/hanlong.skill");

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: ZoneKind,
    kind: CardKind,
    serial: u32,
) -> CardId {
    let zone_id = tx.state().zones.find(zone, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_090 + serial),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: zone_id,
        owner,
        face_up: zone != ZoneKind::Hand,
    });
    tx.state_mut()
        .zones
        .get_mut(zone_id)
        .unwrap()
        .cards
        .push(card);
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

#[test]
fn siji_draws_twice_discarded_sha_count_and_ciqiu_kills_lethal_healthy_target() {
    let package = content_compiler::compile_source_named("wandianlunsha-hanlong", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(SIJI),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(CIQIU),
            },
        ],
    )
    .unwrap();

    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_091),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: discard,
        owner: PlayerId::NONE,
        face_up: true,
    });
    let shan = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_092),
        kind: CardKind::Shan,
        suit: Suit::Heart,
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
        .extend([sha, shan]);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::DISCARD_PHASE_LOST_BASE, 1);
    tx.state_mut().turn.record_card_loss(owner, sha);
    tx.state_mut().turn.record_card_loss(owner, shan);
    tx.state_mut().turn.record_discarded_cards([sha, shan]);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    let hand_before = tx.state().hand_count(owner);
    let siji = skill(&tx, owner, SIJI);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveWdbHanlong {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            helper_skill_def: SkillDefId::new(CIQIU_DYING),
            action: 0,
            skill: siji,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand_before + 2);

    tx.state_mut().turn.phase = Phase::Play;
    let target_state = tx.state_mut().players.get_mut(target).unwrap();
    target_state.max_hp = 2;
    target_state.hp = 2;
    let card = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 3);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source: owner,
            target,
            card,
            amount: 1,
            nature: rule_api::damage_nature::NORMAL,
            card_flags: 0,
        }],
    )
    .unwrap();
    assert!(!tx.state().players.get(target).unwrap().alive);
    assert!(tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .all(|skill| tx.state().skills.get(*skill).unwrap().def.raw() != CIQIU));
}
}

mod original_extension_wandianlunsha_liufu {
//! Golden for archived extension 玩点论杀 刘馥【筑城/夺气】（ABI 610）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const ZHUCHENG: u32 = 2_400_116_759;
const DUOQI: u32 = 2_671_845_513;
const SOURCE: &str = include_str!("../../../content/original/ported/wandianlunsha/liufu.skill");

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, serial: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_100 + serial),
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

fn skill(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
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
        "{result:?}"
    );
    state
}

#[test]
fn zhucheng_stores_lost_hp_top_cards_defends_sha_and_duoqi_ends_play() {
    let package = content_compiler::compile_source_named("wandianlunsha-liufu", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 610);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let attacker = players[1];
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
            skill_def: SkillDefId::new(ZHUCHENG),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DUOQI),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.max_hp = 3;
    participant.hp = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    let zhucheng = skill(tx.state(), owner, ZHUCHENG);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveWdbLiufu {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            zhucheng_skill_def: SkillDefId::new(ZHUCHENG),
            action: 0,
            skill: zhucheng,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().skill_pile_cards(owner, ZHUCHENG).len(), 2);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;

    tx.state_mut().turn.current = attacker;
    tx.state_mut().turn.phase = Phase::Play;
    let sha = inject(tx.state_mut(), attacker, CardKind::Sha, 1);
    let payment = inject(tx.state_mut(), attacker, CardKind::Shan, 2);
    let payment_two = inject(tx.state_mut(), attacker, CardKind::Tao, 3);
    let hp = tx.state().players.get(owner).unwrap().hp;
    let state = tx.commit().state;
    let (state, result) = apply_command(
        state,
        Command {
            actor: attacker,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![owner],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = choose(state, owner, vec![1], false, 2);
    assert_eq!(state.resolution.continuation_kind, ContinuationKind::Custom);
    let state = choose(
        state,
        attacker,
        vec![payment.raw(), payment_two.raw()],
        false,
        3,
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = choose(state, owner, vec![0], false, 4);
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
    assert_eq!(state.skill_pile_cards(owner, ZHUCHENG).len(), 2);

    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = attacker;
    tx.state_mut().turn.phase = Phase::Play;
    let duoqi = skill(tx.state(), owner, DUOQI);
    let pile_card = tx.state().skill_pile_cards(owner, ZHUCHENG)[0];
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveWdbLiufu {
                player: owner,
                target: attacker,
                card: CardId::NONE,
                zhucheng_skill_def: SkillDefId::new(ZHUCHENG),
                action: 2,
                skill: duoqi,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![pile_card.raw()], false, 5);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = choose(state, owner, vec![0], false, 6);
    assert_ne!(state.turn.phase, Phase::Play);
    assert_eq!(state.skill_pile_cards(owner, ZHUCHENG).len(), 1);
}
}

mod original_extension_wandianlunsha_liuyan {
//! Golden for archived extension 玩点论杀 刘焉【绝道/割据】（ABI 611）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JUEDAO: u32 = 3_304_739_063;
const GEJU: u32 = 4_049_779_435;
const SOURCE: &str = include_str!("../../../content/original/ported/wandianlunsha/liuyan.skill");

fn inject_hand(tx: &mut Transaction, owner: PlayerId) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_111),
        kind: CardKind::Sha,
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
fn juedao_links_with_bidirectional_distance_and_geju_counts_whole_groups() {
    let package = content_compiler::compile_source_named("wandianlunsha-liuyan", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 611);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let adjacent = players[1];
    let opposite = players[2];
    let other_adjacent = players[3];
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
            skill_def: SkillDefId::new(JUEDAO),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(GEJU),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().group_overrides.insert(owner, "qun".into());
    tx.state_mut()
        .group_overrides
        .insert(adjacent, "wei".into());
    tx.state_mut()
        .group_overrides
        .insert(opposite, "shu".into());
    tx.state_mut()
        .group_overrides
        .insert(other_adjacent, "wei".into());
    assert!(engine_core::query::player_in_attack_range(
        tx.state(),
        owner,
        adjacent
    ));
    assert!(engine_core::query::player_in_attack_range(
        tx.state(),
        adjacent,
        owner
    ));

    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let cost = inject_hand(&mut tx, owner);
    let juedao = skill(tx.state(), owner, JUEDAO);
    let state = tx.commit().state;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: juedao,
                targets: Vec::new(),
                card_targets: vec![cost],
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert!(!state.hand_cards(owner).contains(&cost));
    assert!(!engine_core::query::player_in_attack_range(
        &state, owner, adjacent
    ));
    assert!(!engine_core::query::player_in_attack_range(
        &state, adjacent, owner
    ));

    let mut tx = Transaction::begin(state);
    let second_cost = inject_hand(&mut tx, owner);
    let state = tx.commit().state;
    let (state, rejected) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ActivateSkill {
                skill: juedao,
                targets: Vec::new(),
                card_targets: vec![second_cost],
            },
        },
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert!(state.hand_cards(owner).contains(&second_cost));

    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Prepare;
    let hand_before = tx.state().hand_count(owner);
    let geju = skill(tx.state(), owner, GEJU);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveWdbLiuyan {
            player: owner,
            card: CardId::NONE,
            action: 1,
            skill: geju,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand_before + 2);
}
}

mod original_extension_wandianlunsha_liuzan {
//! Golden for archived extension 玩点论杀 留赞【亢音】（ABI 612）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const KANGYIN: u32 = 2_250_380_873;
const KANGYIN2: u32 = 2_524_514_324;
const SOURCE: &str = include_str!("../../../content/original/ported/wandianlunsha/liuzan.skill");

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, serial: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_120 + serial),
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

fn skill(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
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

fn setup(kind: CardKind, hp: i32) -> (GameState, Vec<PlayerId>, engine_api::SkillId, CardId) {
    let package = content_compiler::compile_source_named("wandianlunsha-liuzan", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 612);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_803, "identity-standard").unwrap();
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
        skill_def: SkillDefId::new(KANGYIN),
    });
    run_effects(&mut tx, effects).unwrap();
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.max_hp = 4;
    participant.hp = hp;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let card = inject(tx.state_mut(), target, kind, 1);
    let kangyin = skill(tx.state(), owner, KANGYIN);
    (tx.commit().state, players, kangyin, card)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> GameState {
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: Vec::new(),
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
fn kangyin_basic_draws_optional_targets_and_nonbasic_grants_turn_modifiers() {
    let (state, players, kangyin, basic) = setup(CardKind::Shan, 2);
    let owner = players[0];
    let target = players[1];
    let state = activate(state, owner, target, kangyin);
    assert_eq!(state.players.get(owner).unwrap().hp, 1);
    let state = choose(state, owner, vec![basic.raw()], 2);
    assert!(state.has_pending_choice());
    let owner_hand = state.hand_count(owner);
    let recipient_hand = state.hand_count(players[2]);
    let state = choose(state, owner, vec![owner.raw(), players[2].raw()], 3);
    assert_eq!(state.hand_count(owner), owner_hand + 1);
    assert_eq!(state.hand_count(players[2]), recipient_hand + 1);

    let (state, players, kangyin, trick) = setup(CardKind::TrickWuzhong, 4);
    let owner = players[0];
    let target = players[1];
    let opposite = players[2];
    let base_target_limit = engine_core::query::sha_target_limit_for(&state, owner);
    assert!(!engine_core::query::player_in_attack_range(
        &state, owner, opposite
    ));
    let state = activate(state, owner, target, kangyin);
    let state = choose(state, owner, vec![trick.raw()], 4);
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert_eq!(
        engine_core::query::sha_target_limit_for(&state, owner),
        base_target_limit + 1
    );
    assert!(engine_core::query::player_in_attack_range(
        &state, owner, opposite
    ));
    let helper = skill(&state, owner, KANGYIN2);
    let instance = state.skills.get(helper).unwrap();
    assert_eq!(instance.state_blob, 1u32.to_le_bytes());
    assert_eq!(instance.temporary_until_turn_end_of, owner);
}
}

mod original_extension_wandianlunsha_tianyu {
//! Golden for archived extension 玩点论杀 田豫【车阵/诱战】与【诱敌深入】（ABI 613）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{query_explain, QueryContext};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const CHEZHEN: u32 = 2_288_694_956;
const YOUZHAN: u32 = 2_340_355_119;
const SOURCE: &str = include_str!("../../../content/original/ported/wandianlunsha/tianyu.skill");

fn setup() -> (GameState, [PlayerId; 4]) {
    let package = content_compiler::compile_source_named("wandianlunsha-tianyu", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 613);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_803, "identity-standard").unwrap();
    let players: [PlayerId; 4] = tx.state().players.alive_by_seat().try_into().unwrap();
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
            skill_def: SkillDefId::new(CHEZHEN),
        },
        EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(YOUZHAN),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[2];
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, players)
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    serial: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(96_130 + serial),
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

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
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
        "{result:?}"
    );
    (state, result)
}

fn choose(
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

fn distance(state: &GameState, from: PlayerId, to: PlayerId) -> i32 {
    let mut context = QueryContext::new();
    match query_explain(state, &QueryRequest::Distance { from, to }, &mut context).response {
        QueryResponse::Int(value) => value,
        response => panic!("unexpected distance response: {response:?}"),
    }
}

fn begin_youzhan(mut state: GameState, players: [PlayerId; 4], key: u64) -> (GameState, CardId) {
    let owner = players[0];
    let protected = players[1];
    let source = players[2];
    let cost = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Weapon,
        key as u32,
    );
    let incoming = inject(
        &mut state,
        source,
        ZoneKind::Hand,
        CardKind::Sha,
        key as u32 + 1,
    );
    let (state, result) = submit(
        state,
        source,
        CommandKind::PlayCard {
            card: incoming,
            targets: vec![protected],
            card_targets: Vec::new(),
        },
        key,
    );
    assert_eq!(result.choice.unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![1], false, key + 1);
    assert_eq!(result.choice.unwrap().observer, owner);
    let (state, _) = choose(state, owner, vec![cost.raw()], false, key + 2);
    (state, cost)
}

#[test]
fn package_and_chezhen_keep_the_equipment_test_boolean() {
    let package = content_compiler::compile_source_named("wandianlunsha-tianyu", SOURCE).unwrap();
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == YOUZHAN
            && trigger.filter == "owner_alive_sha_target_distance_le_one_has_equip_card"));
    let (mut state, players) = setup();
    let owner = players[0];
    let far = players[2];
    assert_eq!(distance(&state, owner, far), 2);
    assert_eq!(distance(&state, far, owner), 3);
    inject(&mut state, owner, ZoneKind::Equip, CardKind::Weapon, 1);
    inject(&mut state, owner, ZoneKind::Equip, CardKind::Armor, 2);
    inject(&mut state, owner, ZoneKind::Equip, CardKind::Treasure, 3);
    assert_eq!(distance(&state, owner, far), 1);
    assert_eq!(distance(&state, far, owner), 2);
}

#[test]
fn youzhan_cancels_the_incoming_sha_and_refusal_damages_its_source() {
    let (state, players) = setup();
    let owner = players[0];
    let protected = players[1];
    let source = players[2];
    let protected_hp = state.players.get(protected).unwrap().hp;
    let source_hp = state.players.get(source).unwrap().hp;
    let (state, cost) = begin_youzhan(state, players, 10);
    assert!(!state.hand_cards(owner).contains(&cost));
    assert_eq!(state.players.get(protected).unwrap().hp, protected_hp);
    assert_eq!(state.players.get(source).unwrap().hp, source_hp - 1);
}

#[test]
fn youdishenru_is_wuxieable_and_then_the_original_sha_continues() {
    let (mut state, players) = setup();
    let protected = players[1];
    let source = players[2];
    let wuxie_owner = players[3];
    let wuxie = inject(
        &mut state,
        wuxie_owner,
        ZoneKind::Hand,
        CardKind::TrickWuxie,
        20,
    );
    let protected_hp = state.players.get(protected).unwrap().hp;
    let source_hp = state.players.get(source).unwrap().hp;
    let (state, _) = begin_youzhan(state, players, 21);
    assert_eq!(state.resolution.pending_observer, wuxie_owner.raw());
    let (state, _) = choose(state, wuxie_owner, vec![wuxie.raw()], false, 24);
    assert_eq!(state.players.get(protected).unwrap().hp, protected_hp - 1);
    assert_eq!(state.players.get(source).unwrap().hp, source_hp);
}

#[test]
fn a_dodged_forced_sha_repeats_until_the_source_refuses() {
    let (mut state, players) = setup();
    let protected = players[1];
    let source = players[2];
    let forced = inject(&mut state, source, ZoneKind::Hand, CardKind::Sha, 30);
    let forced_again = inject(&mut state, source, ZoneKind::Hand, CardKind::Sha, 31);
    let shan = inject(&mut state, protected, ZoneKind::Hand, CardKind::Shan, 32);
    let protected_hp = state.players.get(protected).unwrap().hp;
    let source_hp = state.players.get(source).unwrap().hp;
    let (state, _) = begin_youzhan(state, players, 32);
    assert_eq!(state.resolution.pending_observer, source.raw());
    let (state, result) = choose(state, source, vec![forced.raw()], false, 35);
    assert_eq!(result.choice.unwrap().observer, protected);
    let (state, result) = choose(state, protected, vec![shan.raw()], false, 36);
    assert_eq!(result.choice.unwrap().observer, source);
    assert!(state.hand_cards(source).contains(&forced_again));
    let (state, _) = choose(state, source, Vec::new(), true, 37);
    assert_eq!(state.players.get(protected).unwrap().hp, protected_hp);
    assert_eq!(state.players.get(source).unwrap().hp, source_hp - 1);
}

#[test]
fn a_forced_sha_that_deals_damage_ends_the_loop() {
    let (mut state, players) = setup();
    let protected = players[1];
    let source = players[2];
    let forced = inject(&mut state, source, ZoneKind::Hand, CardKind::Sha, 38);
    let protected_hp = state.players.get(protected).unwrap().hp;
    let source_hp = state.players.get(source).unwrap().hp;
    let (state, _) = begin_youzhan(state, players, 39);
    let (state, result) = choose(state, source, vec![forced.raw()], false, 42);
    assert!(result.choice.is_none());
    assert_eq!(state.players.get(protected).unwrap().hp, protected_hp - 1);
    assert_eq!(state.players.get(source).unwrap().hp, source_hp);
}

#[test]
fn a_physical_youdishenru_uses_the_same_response_pipeline() {
    let (mut state, players) = setup();
    let protected = players[1];
    let source = players[2];
    let youdi = inject(
        &mut state,
        protected,
        ZoneKind::Hand,
        CardKind::TrickYoudishenru,
        40,
    );
    let incoming = inject(&mut state, source, ZoneKind::Hand, CardKind::Sha, 41);
    let source_hp = state.players.get(source).unwrap().hp;
    let (state, result) = submit(
        state,
        source,
        CommandKind::PlayCard {
            card: incoming,
            targets: vec![protected],
            card_targets: Vec::new(),
        },
        40,
    );
    let result = result.choice.unwrap();
    assert_eq!(result.observer, protected);
    assert!(result
        .options
        .iter()
        .any(|option| option.token == youdi.raw()));
    let (state, _) = choose(state, protected, vec![youdi.raw()], false, 41);
    assert_eq!(state.players.get(source).unwrap().hp, source_hp - 1);
    assert!(!state.hand_cards(protected).contains(&youdi));
}
}

mod original_extension_wandianlunsha_xizhenxihong {
//! Golden for archived extension 玩点论杀 习珍习宏【负仇/噤言】（ABI 614）。

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceRequest, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const FUCHOU: u32 = 3_890_431_595;
const JINYAN: u32 = 3_563_375_486;
const SOURCE: &str =
    include_str!("../../../content/original/ported/wandianlunsha/xizhenxihong.skill");

fn setup() -> (GameState, [PlayerId; 4]) {
    let package =
        content_compiler::compile_source_named("wandianlunsha-xizhenxihong", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 614);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_803, "identity-standard").unwrap();
    let players: [PlayerId; 4] = tx.state().players.alive_by_seat().try_into().unwrap();
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
            skill_def: SkillDefId::new(FUCHOU),
        },
        EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(JINYAN),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[1];
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, players)
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    serial: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(97_000 + serial),
        kind,
        suit,
        rank: 7,
        weapon_range: if kind == CardKind::Weapon { 3 } else { 0 },
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

fn choose(
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

fn fuchou_skill(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
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
                .is_some_and(|skill| skill.def.raw() == FUCHOU)
        })
        .unwrap()
}

fn jinyan_skill(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
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
                .is_some_and(|skill| skill.def.raw() == JINYAN)
        })
        .unwrap()
}

fn record_fuchou(
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    gift: CardId,
    incoming: CardId,
    key: u64,
) -> GameState {
    let (state, result) = submit(
        state,
        source,
        CommandKind::PlayCard {
            card: incoming,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        key,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(result.choice.unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![1], false, key + 1);
    assert_eq!(result.choice.unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![gift.raw()], false, key + 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn end_turn(state: GameState, current: PlayerId) -> (GameState, Option<ChoiceRequest>) {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current,
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
    let effect_result = run_effects(&mut tx, effects);
    assert!(matches!(
        effect_result,
        Ok(_) | Err(EffectError::ChoiceRequested)
    ));
    let committed = tx.commit();
    (committed.state, committed.output.choice)
}

#[test]
fn package_exposes_exact_filters_modifiers_and_view_as_contract() {
    let package =
        content_compiler::compile_source_named("wandianlunsha-xizhenxihong", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 614);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == FUCHOU && trigger.filter == "owner_alive_wdb_fuchou_pending_living"
    }));
    assert!(package.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == JINYAN
            && modifier.target == "card_response_cancel"
            && modifier.selector.as_deref() == Some("black_trick")
    }));
    assert_eq!(
        package
            .view_as_index
            .iter()
            .filter(
                |view| view.skill_def_id == JINYAN && view.filter == "owner_hp_le_two_black_trick"
            )
            .count(),
        2
    );
}

#[test]
fn jinyan_blocks_the_original_black_trick_but_can_use_it_as_sha_at_two_hp() {
    let (mut state, players) = setup();
    let owner = players[0];
    let target = players[1];
    state.turn.current = owner;
    state.players.get_mut(owner).unwrap().hp = 2;
    let card = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        Suit::Spade,
        1,
    );
    let (state, rejected) = submit(
        state,
        owner,
        CommandKind::PlayCard {
            card,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        10,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    let hp = state.players.get(target).unwrap().hp;
    let skill = jinyan_skill(&state, owner);
    let (state, result) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![card],
        },
        11,
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn jinyan_responds_with_black_wuxie_as_sha_instead_of_wuxie() {
    let (mut state, players) = setup();
    let owner = players[0];
    let source = players[1];
    state.players.get_mut(owner).unwrap().hp = 2;
    let black_wuxie = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::TrickWuxie,
        Suit::Club,
        61,
    );
    let duel = inject(
        &mut state,
        source,
        ZoneKind::Hand,
        CardKind::TrickJuedou,
        Suit::Heart,
        62,
    );
    let source_hp = state.players.get(source).unwrap().hp;
    let (state, result) = submit(
        state,
        source,
        CommandKind::PlayCard {
            card: duel,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        12,
    );
    let choice = result.choice.unwrap();
    assert_eq!(choice.observer, owner);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == black_wuxie.raw()));
    let (state, result) = choose(state, owner, vec![black_wuxie.raw()], false, 13);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.players.get(source).unwrap().hp, source_hp - 1);
}

#[test]
fn fuchou_gives_the_card_cancels_sha_and_records_each_source_once() {
    let (mut state, players) = setup();
    let owner = players[0];
    let source = players[1];
    let gift = inject(
        &mut state,
        owner,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Heart,
        2,
    );
    let incoming = inject(
        &mut state,
        source,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Club,
        3,
    );
    let hp = state.players.get(owner).unwrap().hp;
    let mut state = record_fuchou(state, owner, source, gift, incoming, 20);
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    assert!(state.hand_cards(source).contains(&gift));
    state
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 0);
    state
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
    let second_gift = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        Suit::Diamond,
        63,
    );
    let second_sha = inject(
        &mut state,
        source,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        64,
    );
    let state = record_fuchou(state, owner, source, second_gift, second_sha, 23);
    assert!(state.hand_cards(source).contains(&second_gift));
    assert_eq!(
        state
            .skills
            .get(fuchou_skill(&state, owner))
            .unwrap()
            .state_blob
            .len(),
        8
    );
}

#[test]
fn fuchou_draws_then_loses_hp_when_no_sha_is_available() {
    let (mut state, players) = setup();
    let owner = players[0];
    let source = players[1];
    let gift = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Heart,
        4,
    );
    let incoming = inject(
        &mut state,
        source,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Club,
        5,
    );
    let state = record_fuchou(state, owner, source, gift, incoming, 30);
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let mut state = state;
    state.zones.get_mut(draw).unwrap().cards.clear();
    inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::TrickWuzhong,
        Suit::Heart,
        50,
    );
    let hp = state.players.get(owner).unwrap().hp;
    let hand = state.hand_count(owner);
    let (state, result) = end_turn(state, source);
    assert!(result.is_none());
    assert_eq!(state.hand_count(owner), hand + 1);
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
    assert_eq!(
        state
            .skills
            .get(fuchou_skill(&state, owner))
            .unwrap()
            .state_blob,
        b"WFU1"
    );
}

#[test]
fn fuchou_can_use_sha_at_distance_two_and_continues_after_it_resolves() {
    let (mut state, players) = setup();
    let owner = players[0];
    let source = players[2];
    state.turn.current = source;
    state
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED, 0);
    state
        .turn
        .set_flag(engine_core::state::turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
    inject(
        &mut state,
        source,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Heart,
        60,
    );
    let gift = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::TrickWuzhong,
        Suit::Heart,
        6,
    );
    let retaliation = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Diamond,
        7,
    );
    let incoming = inject(
        &mut state,
        source,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Club,
        8,
    );
    let state = record_fuchou(state, owner, source, gift, incoming, 40);
    let hp = state.players.get(source).unwrap().hp;
    let (state, result) = end_turn(state, source);
    let choice = result.unwrap();
    assert_eq!(choice.observer, owner);
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == retaliation.raw()));
    let (state, result) = choose(state, owner, vec![retaliation.raw()], false, 43);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.players.get(source).unwrap().hp, hp - 1);
    assert!(result.choice.is_none());
}
}

mod original_extension_wandianlunsha_yangyi {
//! Golden for archived extension 玩点论杀 杨仪【筹度/立断】与【调兵遣将】（ABI 615）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const CHOUDU: u32 = 4_166_492_495;
const LIDUAN: u32 = 2_934_618_094;
const SOURCE: &str = include_str!("../../../content/original/ported/wandianlunsha/yangyi.skill");

fn setup(players_count: u8) -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("wandianlunsha-yangyi", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 615);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, players_count, 20_260_803, "identity-standard").unwrap();
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
    effects.extend([
        EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(CHOUDU),
        },
        EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(LIDUAN),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[2.min(players.len() - 1)];
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, players)
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    serial: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000 + serial),
        kind,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: if kind == CardKind::Weapon { 3 } else { 0 },
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
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
                .is_some_and(|skill| skill.def.raw() == def)
        })
        .unwrap()
}

fn submit(
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

fn choose(
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

fn gain_one(
    state: GameState,
    target: PlayerId,
    card: CardId,
) -> (GameState, Option<engine_api::ChoiceRequest>) {
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let from = tx.state().cards.get(card).unwrap().zone;
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::MoveCardsWithReason {
            cards: vec![card],
            to: hand,
            from: Some(from),
            reason: engine_api::CardMoveReason::Unspecified,
        }],
    );
    assert!(matches!(result, Err(EffectError::ChoiceRequested)));
    let committed = tx.commit();
    (committed.state, committed.output.choice)
}

#[test]
fn package_exposes_exact_abi_active_and_gain_filter() {
    let package = content_compiler::compile_source_named("wandianlunsha-yangyi", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 615);
    assert_eq!(package.active_index.len(), 1);
    assert_eq!(package.trigger_index.len(), 1);
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_alive_other_gained_single_equip_to_hand_outside_turn_usable"
    );
}

#[test]
fn liduan_recipient_can_give_any_hand_card_to_yangyi() {
    let (mut state, players) = setup(3);
    let owner = players[0];
    let recipient = players[1];
    let gift = inject(&mut state, recipient, ZoneKind::Hand, CardKind::Sha, 1);
    let equipment = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Weapon,
        2,
    );
    let (state, choice) = gain_one(state, recipient, equipment);
    assert_eq!(choice.unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![1], false, 10);
    assert_eq!(result.choice.unwrap().observer, recipient);
    let (state, result) = choose(state, recipient, vec![gift.raw()], false, 11);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(owner).contains(&gift));
    assert!(state.hand_cards(recipient).contains(&equipment));
}

#[test]
fn liduan_cancel_forces_the_gained_equipment_to_be_used() {
    let (mut state, players) = setup(3);
    let owner = players[0];
    let recipient = players[1];
    let equipment = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Armor,
        3,
    );
    let (state, choice) = gain_one(state, recipient, equipment);
    assert_eq!(choice.unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![1], false, 20);
    assert_eq!(result.choice.unwrap().observer, recipient);
    let (state, result) = choose(state, recipient, Vec::new(), true, 21);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    let equip = state.zones.find(ZoneKind::Equip, recipient).unwrap();
    assert!(state.zones.get(equip).unwrap().cards.contains(&equipment));
}

#[test]
fn diaobing_swaps_revealed_card_then_preserves_chosen_top_order() {
    let (mut state, players) = setup(3);
    let actor = players[2];
    let target = players[1];
    state.turn.current = actor;
    let card = inject(
        &mut state,
        actor,
        ZoneKind::Hand,
        CardKind::TrickDiaobingqianjiang,
        10,
    );
    let replacement = inject(&mut state, target, ZoneKind::Hand, CardKind::Sha, 11);
    let pool_first = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Armor,
        12,
    );
    let pool_second = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::TrickWuzhong,
        13,
    );
    let drawn = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Shan,
        14,
    );
    let (state, result) = submit(
        state,
        actor,
        CommandKind::PlayCard {
            card,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        30,
    );
    assert_eq!(result.choice.unwrap().observer, actor);
    assert!(state.hand_cards(actor).contains(&drawn));
    let (state, result) = choose(state, actor, Vec::new(), true, 31);
    assert_eq!(result.choice.unwrap().observer, target);
    let (state, result) = choose(state, target, vec![pool_second.raw()], false, 32);
    assert_eq!(result.choice.unwrap().observer, target);
    let (state, result) = choose(state, target, vec![replacement.raw()], false, 33);
    assert_eq!(result.choice.unwrap().observer, actor);
    let (state, result) = choose(state, actor, vec![replacement.raw()], false, 34);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert!(state.hand_cards(target).contains(&pool_second));
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(
        state.zones.get(draw).unwrap().cards.last(),
        Some(&replacement)
    );
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state
        .zones
        .get(discard)
        .unwrap()
        .cards
        .contains(&pool_first));
}

#[test]
fn diaobing_opens_an_independent_wuxie_window_for_each_target() {
    let (mut state, players) = setup(3);
    let actor = players[2];
    let responder = players[0];
    state.turn.current = actor;
    let card = inject(
        &mut state,
        actor,
        ZoneKind::Hand,
        CardKind::TrickDiaobingqianjiang,
        15,
    );
    let wuxie = inject(
        &mut state,
        responder,
        ZoneKind::Hand,
        CardKind::TrickWuxie,
        16,
    );
    inject(&mut state, responder, ZoneKind::Hand, CardKind::Other, 19);
    inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Armor,
        17,
    );
    inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Sha,
        18,
    );
    let (state, result) = submit(
        state,
        actor,
        CommandKind::PlayCard {
            card,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        35,
    );
    assert_eq!(result.choice.unwrap().observer, responder);
    let (mut state, mut result) = choose(state, responder, vec![wuxie.raw()], false, 36);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    let mut key = 37;
    let choice = loop {
        let choice = result.choice.unwrap();
        if choice.kind == engine_api::ChoiceKind::SelectCards {
            break choice;
        }
        (state, result) = choose(state, choice.observer, Vec::new(), true, key);
        key += 1;
    };
    assert_eq!(choice.observer, responder);
    assert_eq!(choice.kind, engine_api::ChoiceKind::SelectCards);
}

#[test]
fn choudu_discards_cost_and_makes_the_chosen_player_use_diaobing() {
    let (mut state, players) = setup(3);
    let owner = players[0];
    let target = players[1];
    state.turn.current = owner;
    let cost = inject(&mut state, owner, ZoneKind::Hand, CardKind::Sha, 20);
    let drawn = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Shan,
        21,
    );
    let choudu = skill(&state, owner, CHOUDU);
    let (state, result) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill: choudu,
            targets: Vec::new(),
            card_targets: vec![cost],
        },
        40,
    );
    assert_eq!(result.choice.unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![target.raw()], false, 41);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(result.choice.unwrap().observer, target);
    assert!(!state.hand_cards(owner).contains(&cost));
    assert!(state.hand_cards(target).contains(&drawn));
}
}

mod original_extension_wandianlunsha_zaozhirenjun {
//! Golden for 玩点论杀 枣祗任峻【粮策/坚壁/军屯】、【五谷丰登】与【望梅止渴】（ABI 616）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LIANGCE: u32 = 3_473_735_644;
const JIANBI: u32 = 3_452_664_776;
const DIYJUNTUN: u32 = 2_307_382_183;
const SOURCE: &str =
    include_str!("../../../content/original/ported/wandianlunsha/zaozhirenjun.skill");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package =
        content_compiler::compile_source_named("wandianlunsha-zaozhirenjun", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 616);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
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
    effects.extend(
        [LIANGCE, JIANBI, DIYJUNTUN].map(|skill_def| EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = players[0];
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, players)
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    serial: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_000 + serial),
        kind,
        suit: Suit::Club,
        rank: 8,
        weapon_range: if kind == CardKind::Weapon { 3 } else { 0 },
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
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
                .is_some_and(|skill| skill.def.raw() == def)
        })
        .unwrap()
}

fn submit(
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

fn choose(
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
fn package_exposes_exact_abi_and_three_skill_surfaces() {
    let package =
        content_compiler::compile_source_named("wandianlunsha-zaozhirenjun", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 616);
    assert_eq!(package.skill_index.len(), 3);
    assert_eq!(package.trigger_index.len(), 2);
    assert_eq!(package.active_index.len(), 1);
    assert_eq!(package.view_as_index.len(), 1);
    assert_eq!(package.view_as_index[0].as_kind, "wugu");
}

#[test]
fn diyjuntun_recasts_an_equipment_from_the_equip_zone() {
    let (mut state, players) = setup();
    let owner = players[0];
    let equipment = inject(&mut state, owner, ZoneKind::Equip, CardKind::Weapon, 1);
    let replacement = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Sha,
        2,
    );
    let diyjuntun = skill(&state, owner, DIYJUNTUN);
    let (state, result) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill: diyjuntun,
            targets: Vec::new(),
            card_targets: vec![equipment],
        },
        10,
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert!(state.hand_cards(owner).contains(&replacement));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(state.zones.get(discard).unwrap().cards.contains(&equipment));
}

#[test]
fn jianbi_excludes_one_selected_tiesuo_target() {
    let (mut state, players) = setup();
    let owner = players[0];
    let actor = players[1];
    let other = players[2];
    state.turn.current = actor;
    let card = inject(&mut state, actor, ZoneKind::Hand, CardKind::TrickTiesuo, 10);
    let (state, result) = submit(
        state,
        actor,
        CommandKind::PlayCard {
            card,
            targets: vec![owner, other],
            card_targets: Vec::new(),
        },
        20,
    );
    assert_eq!(result.choice.as_ref().unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![1], false, 21);
    assert_eq!(result.choice.as_ref().unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![other.raw()], false, 22);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    let chained = |player| {
        state
            .players
            .get(player)
            .unwrap()
            .marks
            .iter()
            .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0)
    };
    assert!(chained(owner));
    assert!(!chained(other));
}

#[test]
fn wugu_reveals_one_per_target_and_each_target_takes_one() {
    let (mut state, players) = setup();
    let actor = players[1];
    state.turn.current = actor;
    let card = inject(&mut state, actor, ZoneKind::Hand, CardKind::TrickWugu, 20);
    for serial in 21..=23 {
        inject(
            &mut state,
            PlayerId::NONE,
            ZoneKind::DrawPile,
            CardKind::Sha,
            serial,
        );
    }
    let (mut state, mut result) = submit(
        state,
        actor,
        CommandKind::PlayCard {
            card,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        30,
    );
    let mut key = 31;
    loop {
        let Some(choice) = result.choice.clone() else {
            break;
        };
        if choice.kind == engine_api::ChoiceKind::SelectCards {
            (state, result) = choose(
                state,
                choice.observer,
                vec![choice.options[0].token],
                false,
                key,
            );
        } else {
            // Do not activate 坚壁 in this test.
            (state, result) = choose(state, choice.observer, Vec::new(), true, key);
        }
        key += 1;
    }
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert!(players.iter().all(|player| state.hand_count(*player) == 1));
}

#[test]
fn jianbi_excludes_a_wugu_target_without_reducing_the_revealed_count() {
    let (mut state, players) = setup();
    let owner = players[0];
    let actor = players[1];
    let excluded = players[2];
    state.turn.current = actor;
    let card = inject(&mut state, actor, ZoneKind::Hand, CardKind::TrickWugu, 24);
    for serial in 25..=27 {
        inject(
            &mut state,
            PlayerId::NONE,
            ZoneKind::DrawPile,
            CardKind::Sha,
            serial,
        );
    }
    let (state, result) = submit(
        state,
        actor,
        CommandKind::PlayCard {
            card,
            targets: Vec::new(),
            card_targets: Vec::new(),
        },
        35,
    );
    assert_eq!(result.choice.as_ref().unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![1], false, 36);
    assert!(result.choice.is_some(), "{result:?}");
    assert_eq!(
        result.choice.as_ref().unwrap().kind,
        engine_api::ChoiceKind::SelectTargets
    );
    let (mut state, mut result) = choose(state, owner, vec![excluded.raw()], false, 37);
    let mut key = 38;
    loop {
        let Some(choice) = result.choice.clone() else {
            break;
        };
        if choice.kind == engine_api::ChoiceKind::SelectCards {
            (state, result) = choose(
                state,
                choice.observer,
                vec![choice.options[0].token],
                false,
                key,
            );
        } else {
            // Decline 粮策 after the single remaining revealed card reaches discard.
            (state, result) = choose(state, choice.observer, Vec::new(), true, key);
        }
        key += 1;
    }
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.hand_count(owner), 1);
    assert_eq!(state.hand_count(actor), 1);
    assert_eq!(state.hand_count(excluded), 0);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(
        state
            .zones
            .get(discard)
            .unwrap()
            .cards
            .iter()
            .filter(|card| state.cards.get(**card).unwrap().def.raw() >= 99_025)
            .count(),
        1
    );
}

#[test]
fn liangce_can_give_all_wugu_remainders_still_in_discard() {
    let (mut state, players) = setup();
    let owner = players[0];
    let recipient = players[2];
    let left = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::Processing,
        CardKind::Sha,
        30,
    );
    let right = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::Processing,
        CardKind::Shan,
        31,
    );
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::ContinueWugu {
            action: 2,
            actor: players[1],
            source_card: CardId::NONE,
            targets: Vec::new(),
            excluded: Vec::new(),
            revealed: vec![left, right],
        }],
    );
    assert!(matches!(result, Err(EffectError::ChoiceRequested)));
    let committed = tx.commit();
    assert_eq!(committed.output.choice.unwrap().observer, owner);
    let (state, result) = choose(committed.state, owner, vec![1], false, 40);
    assert_eq!(result.choice.as_ref().unwrap().observer, owner);
    let (state, result) = choose(state, owner, vec![recipient.raw()], false, 41);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert!(state.hand_cards(recipient).contains(&left));
    assert!(state.hand_cards(recipient).contains(&right));
}

#[test]
fn wangmeizhike_recomputes_min_hand_after_recovery() {
    let (mut state, players) = setup();
    let actor = players[1];
    let target = players[2];
    state.turn.current = actor;
    state.players.get_mut(target).unwrap().hp = 2;
    state.players.get_mut(target).unwrap().max_hp = 3;
    for player in [players[0], actor] {
        inject(
            &mut state,
            player,
            ZoneKind::Hand,
            CardKind::Other,
            40 + player.raw(),
        );
    }
    let card = inject(
        &mut state,
        actor,
        ZoneKind::Hand,
        CardKind::TrickWangmeizhike,
        50,
    );
    let drawn = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Sha,
        51,
    );
    let (state, result) = submit(
        state,
        actor,
        CommandKind::PlayCard {
            card,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        50,
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    assert_eq!(state.hand_count(target), 1);
    assert!(state.hand_cards(target).contains(&drawn));
}

#[test]
fn wangmeizhike_draws_two_when_it_does_not_recover() {
    let (mut state, players) = setup();
    let actor = players[1];
    let target = players[2];
    state.turn.current = actor;
    state.players.get_mut(target).unwrap().hp = 3;
    state.players.get_mut(target).unwrap().max_hp = 3;
    for player in [players[0], actor] {
        inject(
            &mut state,
            player,
            ZoneKind::Hand,
            CardKind::Other,
            60 + player.raw(),
        );
    }
    let card = inject(
        &mut state,
        actor,
        ZoneKind::Hand,
        CardKind::TrickWangmeizhike,
        70,
    );
    let first = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Sha,
        71,
    );
    let second = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Shan,
        72,
    );
    let (state, result) = submit(
        state,
        actor,
        CommandKind::PlayCard {
            card,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        60,
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    assert_eq!(state.hand_count(target), 2);
    assert!(state.hand_cards(target).contains(&first));
    assert!(state.hand_cards(target).contains(&second));
}

#[test]
fn wuxie_cancels_wangmeizhike_before_recovery_and_draw() {
    let (mut state, players) = setup();
    let responder = players[0];
    let actor = players[1];
    let target = players[2];
    state.turn.current = actor;
    state.players.get_mut(target).unwrap().hp = 2;
    state.players.get_mut(target).unwrap().max_hp = 3;
    let wuxie = inject(
        &mut state,
        responder,
        ZoneKind::Hand,
        CardKind::TrickWuxie,
        80,
    );
    let card = inject(
        &mut state,
        actor,
        ZoneKind::Hand,
        CardKind::TrickWangmeizhike,
        81,
    );
    inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Sha,
        82,
    );
    let (state, result) = submit(
        state,
        actor,
        CommandKind::PlayCard {
            card,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        70,
    );
    assert_eq!(result.choice.as_ref().unwrap().observer, responder);
    let (state, result) = choose(state, responder, vec![wuxie.raw()], false, 71);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    assert_eq!(state.players.get(target).unwrap().hp, 2);
    assert_eq!(state.hand_count(target), 0);
}
}
