//! O2 全量合并批次 1（2026-10-14）：家族 bingshi 的 33 个独立测试文件逐字包裹为模块——#[test] 函数名不变、测试数量不变；测试全名获得确定性前缀（模块名=原文件名，子串过滤行为不变）；include_bytes/str 相对路径不受影响（同目录）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_bingshi_guoyuan {
//! Golden coverage for BINGSHI 国渊「清蹈」「修耕」「陈赦」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const GUOYUAN: u32 = 2_786_098_854;
const MBQINGDAO: u32 = 2_885_098_893;
const MBXIUGENG: u32 = 3_797_581_537;
const MBCHENSHE: u32 = 3_265_153_315;
const MBXIUGENG_EFFECT: u32 = 4_053_015_127;
const MBXIUGENG_HANDCARD: u32 = 2_492_312_223;
const MBQUNCHONG: u32 = 3_613_650_732;
const MBQUNCHONG_EFFECT: u32 = 3_254_288_451;
const MBRENXING: u32 = 3_029_832_657;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(skill_def: u32, player_count: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(u64::from(skill_def)),
            kind: CommandKind::StartGame {
                player_count,
                rng_seed: u64::from(skill_def),
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, others)
}

fn skill(state: &GameState, player: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == player && skill.def.raw() == def)
        .unwrap()
        .id
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let cards = tx.state().hand_cards(player);
    if !cards.is_empty() {
        run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }]).unwrap();
    }
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, offset: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(503_000 + offset),
        kind,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(state: GameState, selections: Vec<u32>, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let (state, output) = apply_command(
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
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_contains_guoyuan_at_abi509() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    let guoyuan = package
        .character_index
        .iter()
        .find(|character| character.id == GUOYUAN)
        .unwrap();
    assert_eq!(guoyuan.skills, [MBQINGDAO, MBXIUGENG, MBCHENSHE]);
}

#[test]
fn qingdao_waits_for_the_damage_card_to_finish() {
    let (mut tx, owner, others) = setup(MBQINGDAO, 2);
    let attacker = others[0];
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, attacker);
    let sha = inject(&mut tx, attacker, CardKind::Sha, 1);
    tx.state_mut().turn.current = attacker;
    tx.state_mut().turn.phase = Phase::Play;
    let hp = tx.state().players.get(owner).unwrap().hp;
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: attacker,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![owner],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);
    assert!(state.has_pending_choice());
    assert_eq!(state.resolution.continuation_kind, ContinuationKind::Custom);
    let state = resolve(state, vec![1], 2);
    assert!(!state.has_pending_choice());
}

#[test]
fn xiugeng_equal_hand_count_applies_both_results() {
    let (mut tx, owner, others) = setup(MBXIUGENG, 2);
    let target = others[0];
    clear_hand(&mut tx, target);
    let owner_skill = skill(tx.state(), owner, MBXIUGENG);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbxiugeng {
                player: owner,
                helper_def: SkillDefId::new(MBXIUGENG_EFFECT),
                hand_limit_def: SkillDefId::new(MBXIUGENG_HANDCARD),
                stage: 0,
                targets: Vec::new(),
                skill: owner_skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, vec![target.raw()], 3);
    let helper = skill(&state, target, MBXIUGENG_EFFECT);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbxiugeng {
            player: target,
            helper_def: SkillDefId::new(MBXIUGENG_EFFECT),
            hand_limit_def: SkillDefId::new(MBXIUGENG_HANDCARD),
            stage: 1,
            targets: Vec::new(),
            skill: helper,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(target), 2);
    assert!(tx.state().skills.get(helper).is_none());
    assert!(tx
        .state()
        .players
        .get(target)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == MBXIUGENG_HANDCARD && *count == 1));
}

#[test]
fn chenshe_same_suit_recovers_to_max_and_removes_itself() {
    let (mut tx, owner, others) = setup(MBCHENSHE, 3);
    let dying = others[0];
    let source = others[1];
    for player in [owner, dying, source] {
        clear_hand(&mut tx, player);
    }
    let cards = [
        inject(&mut tx, owner, CardKind::Sha, 2),
        inject(&mut tx, dying, CardKind::Shan, 3),
        inject(&mut tx, source, CardKind::Tao, 4),
    ];
    let max_hp = tx.state().players.get(dying).unwrap().max_hp;
    let dying_state = tx.state_mut().players.get_mut(dying).unwrap();
    dying_state.hp = 0;
    dying_state.dying = true;
    let skill = skill(tx.state(), owner, MBCHENSHE);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbchenshe {
                player: owner,
                dying,
                source,
                stage: 0,
                roles: Vec::new(),
                suits: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = tx.commit().state;
    for (key, card) in cards.into_iter().enumerate() {
        state = resolve(state, vec![card.raw()], 10 + key as u64);
    }
    assert_eq!(state.players.get(dying).unwrap().hp, max_hp as i32);
    assert!(state.skills.get(skill).is_none());
    assert!(!state.has_pending_choice());
}

#[test]
fn quanchong_discards_all_and_queues_penalized_extra_turn() {
    let (mut tx, owner, others) = setup(MBQUNCHONG, 2);
    let hp = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .hp
        .min(tx.state().players.get(others[0]).unwrap().hp);
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp;
    tx.state_mut().players.get_mut(others[0]).unwrap().hp = hp;
    let skill = skill(tx.state(), owner, MBQUNCHONG);
    assert!(!tx.state().hand_cards(owner).is_empty());
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbquanchong {
            player: owner,
            helper_def: SkillDefId::new(MBQUNCHONG_EFFECT),
            skill,
        }],
    )
    .unwrap();
    assert!(tx.state().hand_cards(owner).is_empty());
    assert_eq!(tx.state().turn.extra_turn_queue, [owner]);
    assert!(tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .any(|instance| instance.owner == owner && instance.def.raw() == MBQUNCHONG_EFFECT));
}

#[test]
fn renxing_draw_branch_counts_against_the_round_limit() {
    let (mut tx, owner, _) = setup(MBRENXING, 2);
    let skill = skill(tx.state(), owner, MBRENXING);
    let before = tx.state().hand_count(owner);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbrenxing {
                player: owner,
                stage: 0,
                target: PlayerId::NONE,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, vec![1], 30);
    assert_eq!(state.hand_count(owner), before + 1);
    assert_eq!(
        u32::from_le_bytes(
            state.skills.get(skill).unwrap().state_blob[8..12]
                .try_into()
                .unwrap()
        ),
        1
    );
}
}

mod original_bingshi_huangzu {
//! Golden coverage for BINGSHI 势黄祖「鸱张」「断鞅」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const MBCHIZHANG: u32 = 3_712_476_166;
const MBDUANYANG: u32 = 2_631_325_253;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(u64::from(skill_def)),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: u64::from(skill_def),
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    for player in [owner, target] {
        let cards = tx.state().hand_cards(player);
        if !cards.is_empty() {
            run_effects(&mut tx, vec![EffectRequest::DiscardCards { player, cards }]).unwrap();
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
    (tx, owner, target)
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, n: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(505_000 + n),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(state: GameState, selections: Vec<u32>, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let (state, output) = apply_command(
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
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn chizhang_locks_matching_response_color_and_duanyang_stores_ownerless_sha() {
    let (mut tx, owner, target) = setup(MBCHIZHANG);
    let sha = inject(&mut tx, owner, CardKind::Sha, Suit::Spade, 1);
    let red_cost = inject(&mut tx, owner, CardKind::Tao, Suit::Heart, 2);
    let red_shan = inject(&mut tx, target, CardKind::Shan, Suit::Diamond, 3);
    let black_shan = inject(&mut tx, target, CardKind::Shan, Suit::Club, 4);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    let state = resolve(state, vec![1], 2);
    let state = resolve(state, vec![red_cost.raw()], 3);
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let response = |selection| Command {
        actor: target,
        expected_revision: ExpectedRevision(revision),
        idempotency_key: IdempotencyKey(u64::from(selection)),
        kind: CommandKind::ResolveChoice {
            choice_id,
            selections: vec![selection],
            cancelled: false,
        },
    };
    let (_, red_output) = apply_command(state.clone(), response(red_shan.raw()));
    assert!(matches!(red_output.result, CommandResult::Rejected { .. }));
    let (_, black_output) = apply_command(state, response(black_shan.raw()));
    assert!(matches!(
        black_output.result,
        CommandResult::Accepted { .. }
    ));

    let (mut tx, owner, _) = setup(MBDUANYANG);
    let stored = inject(&mut tx, owner, CardKind::Sha, Suit::Heart, 5);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    tx.state_mut()
        .zones
        .get_mut(hand)
        .unwrap()
        .cards
        .retain(|card| *card != stored);
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(stored);
    let card = tx.state_mut().cards.get_mut(stored).unwrap();
    card.zone = discard;
    card.owner = PlayerId::NONE;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == MBDUANYANG)
        .unwrap()
        .id;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbduanyang {
            player: owner,
            target: PlayerId::NONE,
            card: CardId::NONE,
            mode: 0,
            stage: 0,
            cards: vec![stored],
            skill,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().skill_pile_cards(owner, MBDUANYANG), [stored]);
    assert!(tx.state().playable_hand_cards(owner).contains(&stored));
}
}

mod original_bingshi_luyusheng {
//! Golden coverage for BINGSHI 势陆郁生「润微」「霜怀」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, schedule_damage, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const MBRUNWEI: u32 = 3_010_472_386;
const MBSHUANGHUAI: u32 = 4_109_631_346;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(u64::from(skill_def)),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: u64::from(skill_def),
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let target = engine
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
    (tx, owner, target)
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

fn resolve(state: GameState, selections: Vec<u32>, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let (state, output) = apply_command(
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
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

fn inject(tx: &mut Transaction, owner: PlayerId) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(506_001),
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

#[test]
fn runwei_resets_after_qualified_loss_and_cleans_own_gain() {
    let (mut tx, owner, _) = setup(MBRUNWEI);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = skill(tx.state(), owner, MBRUNWEI);
    tx.state_mut()
        .skills
        .get_mut(skill)
        .unwrap()
        .play_phase_use_count = 1;
    let filler = inject(&mut tx, owner);
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top = *tx.state().zones.get(draw).unwrap().cards.last().unwrap();
    let color: u32 = match tx.state().cards.get(top).unwrap().suit {
        Suit::Heart | Suit::Diamond => 1,
        Suit::Spade | Suit::Club => 2,
        Suit::None => 0,
    };
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbrunwei {
                player: owner,
                target: PlayerId::NONE,
                stage: 0,
                value: 0,
                cards: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, vec![1], 1);
    let state = resolve(state, vec![color], 2);
    let state = resolve(state, vec![owner.raw()], 3);
    assert!(state.hand_cards(owner).contains(&top));
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: owner,
            cards: vec![filler],
        }],
    )
    .unwrap();
    let blob = &tx.state().skills.get(skill).unwrap().state_blob;
    assert_eq!(u32::from_le_bytes(blob[8..12].try_into().unwrap()), 1);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbrunwei {
            player: owner,
            target: PlayerId::NONE,
            stage: 5,
            value: 0,
            cards: Vec::new(),
            skill,
        }],
    )
    .unwrap();
    assert!(!tx.state().hand_cards(owner).contains(&top));
}

#[test]
fn shuanghuai_prevents_the_bound_damage() {
    let (mut tx, owner, target) = setup(MBSHUANGHUAI);
    let hp = tx.state().players.get(target).unwrap().hp;
    assert_eq!(
        schedule_damage(&mut tx, owner, target, 1),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, vec![1], 4);
    let state = resolve(state, vec![1], 5);
    assert_eq!(state.players.get(target).unwrap().hp, hp);
    let skill = skill(&state, owner, MBSHUANGHUAI);
    assert_eq!(&state.skills.get(skill).unwrap().state_blob[..4], b"SHH1");
}
}

mod original_bingshi_mbtanfeng {
//! Golden tests for BINGSHI「探锋」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{move_one_card, run_effects, EffectError};
use engine_core::hash::canonical_state_hash;
use engine_core::state::{turn_flag, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const MBTANFENG: u32 = 4_191_284_417;
const XIAHOUSHANG: u32 = 2_359_253_130;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/bingshi/guoyuan.skill"),
    include_str!("../../../content/original/ported/bingshi/kunfen_pot_weiyan.skill"),
    include_str!("../../../content/original/ported/bingshi/mb_chenzhi.skill"),
    include_str!("../../../content/original/ported/bingshi/mb_huangzu.skill"),
    include_str!("../../../content/original/ported/bingshi/mb_luyusheng.skill"),
    include_str!("../../../content/original/ported/bingshi/mb_sunjun.skill"),
    include_str!("../../../content/original/ported/bingshi/mb_tianfeng.skill"),
    include_str!("../../../content/original/ported/bingshi/mb_yanghong.skill"),
    include_str!("../../../content/original/ported/bingshi/mb_zhangyan.skill"),
    include_str!("../../../content/original/ported/bingshi/mbtanfeng.skill"),
    include_str!("../../../content/original/ported/bingshi/pangxi.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_caozhen.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_chendao.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_chengpu.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_chenjiao.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_chenqun.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_dengai.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_dongzhao.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_huanjie.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_lougui.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_lusu.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_sunchen.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_xiahouba.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_zanghong.skill"),
    include_str!("../../../content/original/ported/bingshi/pot_zhangren.skill"),
    include_str!("../../../content/original/ported/bingshi/sunshao.skill")
);
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn started_state(seed: u64) -> GameState {
    let mut engine = Engine::new();
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
    engine.state().clone()
}

fn resolve(state: GameState, selections: Vec<u32>, cancelled: bool) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: PlayerId::new(state.resolution.pending_observer),
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    )
}

fn start_effect(tx: &mut Transaction, owner: PlayerId) -> Result<(), EffectError> {
    run_effects(
        tx,
        vec![EffectRequest::ChooseDiscardOrFireDamageSkipPhases {
            player: owner,
            max_discard: 2,
            damage: 1,
            selected: 0,
            step: 0,
            target: PlayerId::NONE,
        }],
    )
    .map(|_| ())
}

#[test]
fn package_contains_complete_xiahoushang() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == MBTANFENG));
    // 33 名上游武将 + ABI 521 孙韶（敢决/筑墼，上游 bingshi/skill.js 定义、角色条目
    // 随 ABI 521 落地）＝ 34。
    assert_eq!(committed.character_index.len(), 36);
    let xiahoushang = committed
        .character_index
        .iter()
        .find(|character| character.id == XIAHOUSHANG)
        .unwrap();
    assert_eq!(xiahoushang.skills, vec![MBTANFENG]);
    assert!(committed
        .character_index
        .iter()
        .any(|character| character.name_key == "sunshao"));
}

#[test]
fn both_branches_discard_hand_and_equip_deal_fire_damage_and_skip_phases() {
    let state = started_state(2_026_071_504);
    let owner = state.turn.current;
    let targets: Vec<_> = state
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let discard_target = targets[0];
    let damage_target = targets[1];
    let mut tx = Transaction::begin(state);

    let hand = tx
        .state()
        .zones
        .find(ZoneKind::Hand, discard_target)
        .unwrap();
    let equip = tx
        .state()
        .zones
        .find(ZoneKind::Equip, discard_target)
        .unwrap();
    let equip_card = tx.state().hand_cards(discard_target)[0];
    tx.state_mut().cards.get_mut(equip_card).unwrap().kind = CardKind::Weapon;
    move_one_card(&mut tx, equip_card, hand, equip, discard_target, true).unwrap();
    let hand_card = tx.state().hand_cards(discard_target)[0];

    assert_eq!(
        start_effect(&mut tx, owner),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::DiscardOrFireDamageSkipPhases
    );
    let state = tx.commit().state;

    let (cancelled_state, cancelled) = resolve(state.clone(), Vec::new(), true);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert!(!cancelled_state.has_pending_choice());
    assert_eq!(cancelled_state.turn.flag(turn_flag::SKIP_DRAW), 0);
    assert_eq!(cancelled_state.turn.flag(turn_flag::SKIP_PLAY), 0);

    let before_forgery = canonical_state_hash(&state);
    let (unchanged, rejected) = resolve(state.clone(), vec![3], false);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&unchanged), before_forgery);

    let (state, _) = resolve(state, vec![1, 2], false);
    let (state, _) = resolve(state, vec![discard_target.raw()], false);
    let (state, discarded) = resolve(state, vec![hand_card.raw(), equip_card.raw()], false);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.flag(turn_flag::SKIP_DRAW), 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::DiscardOrFireDamageSkipPhases
    );
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(hand_card).unwrap().zone, discard);
    assert_eq!(state.cards.get(equip_card).unwrap().zone, discard);

    let hp = state.players.get(damage_target).unwrap().hp;
    let (state, damaged) = resolve(state, vec![damage_target.raw()], false);
    assert!(matches!(damaged.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(damage_target).unwrap().hp, hp - 1);
    assert_eq!(state.turn.flag(turn_flag::SKIP_PLAY), 1);
    assert!(damaged.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target,
            amount: 1,
            nature: 1,
            ..
        } if *source == owner && *target == damage_target
    )));
}
}

mod original_bingshi_pangxi {
//! Golden coverage for BINGSHI 势庞羲「蓄业」「狂想」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const MBXUYE: u32 = 4_127_729_199;
const MBKUANGXIANG: u32 = 3_758_195_005;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup() -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(514),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 514,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let order = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([MBXUYE, MBKUANGXIANG].map(|def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, order)
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

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let cards = tx.state().hand_cards(player);
    if cards.is_empty() {
        return;
    }
    let from = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let to = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        tx,
        vec![EffectRequest::MoveCards {
            cards,
            to,
            from: Some(from),
        }],
    )
    .unwrap();
}

fn inject(tx: &mut Transaction, owner: PlayerId, zone_kind: ZoneKind, serial: u32) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(514_000 + serial),
        kind: CardKind::Shan,
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

fn choose(state: GameState, selection: u32) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_pangxi_and_hand_lte_target_contract_at_abi514() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pangxi")
            .unwrap()
            .skills,
        [MBXUYE, MBKUANGXIANG]
    );
    assert!(
        package
            .active_index
            .iter()
            .find(|active| active.skill_def_id == MBKUANGXIANG)
            .unwrap()
            .targets
            .as_ref()
            .unwrap()
            .hand_lte_owner
    );
}

#[test]
fn xuye_draws_then_places_target_hej_card_on_draw_pile_top() {
    let (mut tx, owner, order) = setup();
    let target = order.iter().copied().find(|role| *role != owner).unwrap();
    for role in order.iter().copied() {
        clear_hand(&mut tx, role);
    }
    for role in order.iter().copied().filter(|role| *role != target) {
        inject(&mut tx, role, ZoneKind::Hand, 10 + role.raw());
    }
    let field = inject(&mut tx, target, ZoneKind::Equip, 1);
    let xuye = skill(tx.state(), owner, MBXUYE);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbxuye {
                player: owner,
                target,
                stage: 0,
                was_max: false,
                skill: xuye,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, field.raw());
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.zones.get(draw).unwrap().cards.last(), Some(&field));
}

#[test]
fn kuangxiang_swaps_whole_hands_and_each_received_set_fires_once() {
    let (mut tx, owner, order) = setup();
    let target = order.into_iter().find(|role| *role != owner).unwrap();
    let third = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|role| *role != owner && *role != target)
        .unwrap();
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, target);
    clear_hand(&mut tx, third);
    for serial in 20..25 {
        inject(&mut tx, third, ZoneKind::Hand, serial);
    }
    let owner_card = inject(&mut tx, owner, ZoneKind::Hand, 2);
    let target_card = inject(&mut tx, target, ZoneKind::Hand, 3);
    let kuangxiang = skill(tx.state(), owner, MBKUANGXIANG);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbkuangxiang {
            player: owner,
            target,
            stage: 0,
            skill: kuangxiang,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_cards(owner), [target_card]);
    assert_eq!(tx.state().hand_cards(target), [owner_card]);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let target_hand = tx.state().zones.find(ZoneKind::Hand, target).unwrap();
    let before = tx.state().hand_count(owner);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: vec![owner_card],
                to: discard,
                from: Some(target_hand),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, 1);
    assert_eq!(state.hand_count(owner), before + 2);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbkuangxiang {
            player: owner,
            target,
            stage: 1,
            skill: kuangxiang,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 2);
}
}

mod original_bingshi_pot_caoshuang {
//! Golden coverage for BINGSHI 势曹爽「典易/奢权/渐专/溺尊」.

use engine_api::{
    CardDefId, CardId, CardMoveReason, ChoiceId, Command, CommandKind, CommandResult,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DIANYI: u32 = 4_200_000_573;
const SHEQUAN: u32 = 4_200_000_574;
const JIANZHUAN: u32 = 4_200_000_575;
const NIZUN: u32 = 4_200_000_576;
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/pot_caoshuang.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

#[test]
fn package_publishes_complete_pot_caoshuang_at_abi568() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    // 定稿 npack（2026-08-04 16:23 重打包）＝ 33 名上游武将 + ABI 521 孙韶 = 34；
    // 技能 99 项；上游 bingshi/card.js 无卡牌定义，card_index 为 0。
    assert_eq!(package.character_index.len(), 36);
    assert_eq!(package.skill_index.len(), 103);
    assert_eq!(package.card_index.len(), 1);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_caoshuang")
            .unwrap()
            .skills,
        [DIANYI, SHEQUAN, JIANZHUAN]
    );
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NIZUN));
}

fn setup(seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 568);
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
    let order = engine.state().players.alive_by_seat();
    let owner = order[0];
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        [DIANYI, SHEQUAN, JIANZHUAN].map(|def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    clear_hands(&mut tx);
    tx.state_mut().turn.round = 1;
    tx.state_mut().players.get_mut(owner).unwrap().max_hp = 4;
    tx.state_mut().players.get_mut(owner).unwrap().hp = 4;
    (tx.commit().state, owner, order)
}

fn clear_hands(tx: &mut Transaction) {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let effects: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter_map(|player| {
            let cards = tx.state().hand_cards(player);
            (!cards.is_empty()).then(|| EffectRequest::MoveCards {
                cards,
                to: discard,
                from: tx.state().zones.find(ZoneKind::Hand, player),
            })
        })
        .collect();
    run_effects(tx, effects).unwrap();
}

fn inject(tx: &mut Transaction, owner: PlayerId, zone_kind: ZoneKind, serial: u32) -> CardId {
    let zone_owner = if matches!(
        zone_kind,
        ZoneKind::Hand | ZoneKind::Equip | ZoneKind::Judge
    ) {
        owner
    } else {
        PlayerId::NONE
    };
    let zone = tx.state().zones.find(zone_kind, zone_owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(568_000 + serial),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: zone_owner,
        face_up: zone_kind != ZoneKind::Hand,
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

fn request(
    player: PlayerId,
    card: CardId,
    action: u8,
    skill: engine_api::SkillId,
) -> EffectRequest {
    EffectRequest::ResolvePotcaoshuang {
        player,
        target: PlayerId::NONE,
        card,
        action,
        skill,
    }
}

fn resolve(state: GameState, selections: Vec<u32>, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: PlayerId::new(state.resolution.pending_observer),
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

fn assign(mut state: GameState, owner: PlayerId, action: u8, def: u32, key: u64) -> GameState {
    let jianzhuan = skill(&state, owner, JIANZHUAN);
    let mut tx = Transaction::begin(state);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![request(owner, CardId::NONE, action, jianzhuan)]
        ),
        Err(EffectError::ChoiceRequested)
    ));
    state = tx.commit().state;
    let (state, result) = resolve(state, vec![def], key);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn jianzhuan_assigns_all_timings_then_awakens_once() {
    let (state, owner, _) = setup(5680);
    let state = assign(state, owner, 0, DIANYI, 5681);
    let state = assign(state, owner, 1, SHEQUAN, 5682);
    let mut state = assign(state, owner, 2, DIANYI, 5683);
    state.players.get_mut(owner).unwrap().hp = 3;
    let jianzhuan = skill(&state, owner, JIANZHUAN);
    let mut tx = Transaction::begin(state);
    assert!(matches!(
        run_effects(&mut tx, vec![request(owner, CardId::NONE, 0, jianzhuan)]),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, result) = resolve(tx.commit().state, vec![DIANYI], 5684);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state
        .skills
        .sorted_live()
        .iter()
        .all(|instance| instance.owner != owner || instance.def.raw() != DIANYI));
    assert!(state
        .skills
        .sorted_live()
        .iter()
        .any(|instance| instance.owner == owner && instance.def.raw() == NIZUN));
    assert_eq!(state.players.get(owner).unwrap().max_hp, 5);
    assert_eq!(state.players.get(owner).unwrap().hp, 4);
}

#[test]
fn shequan_freezes_private_choices_and_rewrites_hand_numbers() {
    let (state, owner, order) = setup(5685);
    let state = assign(state, owner, 0, SHEQUAN, 5686);
    let mut tx = Transaction::begin(state);
    let first = inject(&mut tx, order[1], ZoneKind::Hand, 1);
    let second = inject(&mut tx, order[2], ZoneKind::Hand, 2);
    let shequan = skill(tx.state(), owner, SHEQUAN);
    assert!(matches!(
        run_effects(&mut tx, vec![request(owner, CardId::NONE, 6, shequan)]),
        Err(EffectError::ChoiceRequested)
    ));
    let pending = tx.commit().state;
    let first_observer = PlayerId::new(pending.resolution.pending_observer);
    let first_card = if first_observer == order[1] {
        first
    } else {
        second
    };
    let (pending, _) = resolve(pending, vec![first_card.raw()], 5687);
    let second_observer = PlayerId::new(pending.resolution.pending_observer);
    let second_card = if second_observer == order[1] {
        first
    } else {
        second
    };
    let (mut state, _) = resolve(pending, vec![second_card.raw()], 5688);
    assert!(state.card_has_content_tag(first, SHEQUAN));
    assert!(state.card_has_content_tag(second, SHEQUAN));
    assert_eq!(
        engine_core::query::effective_card_number(
            &state,
            order[1],
            state.cards.get(first).unwrap()
        ),
        1
    );
    let owner_hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let old_hand = state.zones.find(ZoneKind::Hand, order[1]).unwrap();
    state
        .zones
        .get_mut(old_hand)
        .unwrap()
        .cards
        .retain(|card| *card != first);
    state.zones.get_mut(owner_hand).unwrap().cards.push(first);
    let card = state.cards.get_mut(first).unwrap();
    card.zone = owner_hand;
    card.owner = owner;
    assert_eq!(
        engine_core::query::effective_card_number(&state, owner, state.cards.get(first).unwrap()),
        13
    );
}

#[test]
fn dianyi_uses_deterministic_random_gain_count_and_hp_threshold() {
    let (state, owner, order) = setup(5689);
    let state = assign(state, owner, 0, DIANYI, 5690);
    let state = assign(state, owner, 1, DIANYI, 5691);
    let state = assign(state, owner, 2, DIANYI, 5692);
    let mut tx = Transaction::begin(state);
    for serial in 10..=15 {
        let zone = if serial % 2 == 0 {
            ZoneKind::DiscardPile
        } else {
            ZoneKind::Hand
        };
        let card = inject(&mut tx, order[1], zone, serial);
        tx.state_mut().add_content_card_tag(card, SHEQUAN);
    }
    let dianyi = skill(tx.state(), owner, DIANYI);
    for action in 3..=5 {
        run_effects(&mut tx, vec![request(owner, CardId::NONE, action, dianyi)]).unwrap();
    }
    assert_eq!(tx.state().hand_count(owner), 6);
    assert_eq!(
        tx.state().players.get(owner).unwrap().hp,
        3,
        "典易状态={:?}",
        tx.state().skills.get(dianyi).unwrap().state_blob
    );
}

#[test]
fn nizun_collects_tagged_discard_once_per_turn_and_increases_damage() {
    let (state, owner, order) = setup(5693);
    let mut tx = Transaction::begin(state);
    let dianyi_skill = skill(tx.state(), owner, DIANYI);
    let jianzhuan_skill = skill(tx.state(), owner, JIANZHUAN);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::RemoveSkill {
                skill: dianyi_skill,
            },
            EffectRequest::RemoveSkill {
                skill: jianzhuan_skill,
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(NIZUN),
            },
        ],
    )
    .unwrap();
    let first = inject(&mut tx, PlayerId::NONE, ZoneKind::DiscardPile, 20);
    let second = inject(&mut tx, PlayerId::NONE, ZoneKind::DiscardPile, 21);
    tx.state_mut().add_content_card_tag(first, SHEQUAN);
    tx.state_mut().add_content_card_tag(second, SHEQUAN);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::card_used_by_player(owner), 1);
    tx.state_mut().turn.current = order[1];
    tx.state_mut().turn.phase = Phase::End;
    let nizun = skill(tx.state(), owner, NIZUN);
    run_effects(&mut tx, vec![request(owner, CardId::NONE, 9, nizun)]).unwrap();
    assert!(tx.state().playable_hand_cards(owner).contains(&first));
    let pile = tx
        .state()
        .skill_piles
        .iter()
        .find(|pile| pile.owner == owner && pile.skill_def == NIZUN)
        .unwrap()
        .zone;
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCardsWithReason {
            cards: vec![first],
            to: discard,
            from: Some(pile),
            reason: CardMoveReason::Use,
        }],
    )
    .unwrap();
    run_effects(&mut tx, vec![request(owner, first, 11, nizun)]).unwrap();
    assert!(!tx.state().playable_hand_cards(owner).contains(&second));
    let hp = tx.state().players.get(owner).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: order[1],
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 2);
}
}

mod original_bingshi_pot_caozhen {
//! Golden coverage for BINGSHI 势曹真「伺锋」.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const POTSIFENG: u32 = 3_809_208_030;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, engine_api::SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(517),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 517,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|role| *role != owner)
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
        skill_def: SkillDefId::new(POTSIFENG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == POTSIFENG)
        .unwrap()
        .id;
    (tx.commit().state, owner, target, skill)
}

fn choose(state: GameState, selections: Vec<u32>, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let revision = state.revision;
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, output) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

fn distribute_all(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
) -> GameState {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotsifeng {
                player: owner,
                target: owner,
                card: CardId::NONE,
                stage: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let cards = tx.state().resolution.continuation_payload[4..].to_vec();
    let state = choose(tx.commit().state, cards, 1);
    choose(state, vec![target.raw()], 2)
}

#[test]
fn package_publishes_complete_pot_caozhen_at_abi517() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_caozhen")
            .unwrap()
            .skills,
        [POTSIFENG]
    );
}

#[test]
fn sifeng_distributes_in_selection_order_consumes_first_and_forces_mismatched_discard() {
    let (state, owner, target, skill) = setup();
    let state = distribute_all(state, owner, target, skill);
    let pile = state.skill_pile_cards(target, POTSIFENG);
    assert_eq!(pile.len(), 3);
    let first = pile[0];
    let event_card = state.hand_cards(target)[0];
    let mut tx = Transaction::begin(state);
    let first_red = matches!(
        tx.state().cards.get(first).unwrap().suit,
        Suit::Heart | Suit::Diamond
    );
    tx.state_mut().cards.get_mut(event_card).unwrap().suit =
        if first_red { Suit::Spade } else { Suit::Heart };
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotsifeng {
                player: owner,
                target,
                card: event_card,
                stage: 3,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(tx.state().skill_pile_cards(target, POTSIFENG).len(), 2);
    let discard = tx
        .state()
        .hand_cards(target)
        .into_iter()
        .find(|card| *card != event_card)
        .unwrap_or(event_card);
    let state = choose(tx.commit().state, vec![discard.raw()], 3);
    assert_eq!(state.hand_count(target), 3);
}

#[test]
fn sifeng_phase_end_can_gain_all_or_discard_all_then_damage() {
    for (branch, gains) in [(1, true), (0, false)] {
        let (state, owner, target, skill) = setup();
        let state = distribute_all(state, owner, target, skill);
        let owner_hand = state.hand_count(owner);
        let hp = state.players.get(target).unwrap().hp;
        let mut tx = Transaction::begin(state);
        assert_eq!(
            run_effects(
                &mut tx,
                vec![EffectRequest::ResolvePotsifeng {
                    player: owner,
                    target,
                    card: CardId::NONE,
                    stage: 4,
                    skill,
                }],
            ),
            Err(EffectError::ChoiceRequested)
        );
        let state = choose(tx.commit().state, vec![branch], 4 + branch as u64);
        assert!(state.skill_pile_cards(target, POTSIFENG).is_empty());
        if gains {
            assert_eq!(state.hand_count(owner), owner_hand + 3);
            assert_eq!(state.players.get(target).unwrap().hp, hp);
        } else {
            assert_eq!(state.hand_count(owner), owner_hand);
            assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
        }
    }
}
}

mod original_bingshi_pot_chendao {
//! Golden coverage for BINGSHI 势陈到「往烈」「弘毅」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::query;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const POTWANGLIE: u32 = 3_694_841_232;
const POTWANGLIE_DEBUFF: u32 = 3_694_841_233;
const POTHONGYI: u32 = 3_733_552_053;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(skill_def: u32) -> (engine_core::state::GameState, PlayerId, engine_api::SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(519),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 519,
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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == skill_def)
        .unwrap()
        .id;
    (tx.commit().state, owner, skill)
}

fn choose(
    state: engine_core::state::GameState,
    selections: Vec<u32>,
) -> engine_core::state::GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(5_190),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

fn mark_count(state: &engine_core::state::GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(player)
        .and_then(|player| player.marks.iter().find(|(mark, _)| *mark == POTHONGYI))
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[test]
fn package_publishes_complete_pot_chendao_at_abi519() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    let character = package
        .character_index
        .iter()
        .find(|character| character.name_key == "pot_chendao")
        .unwrap();
    assert_eq!(character.skills, vec![POTWANGLIE, POTHONGYI]);
}

#[test]
fn wanglie_arms_only_the_chosen_card_then_enables_self_target_lock() {
    let (state, owner, skill) = setup(POTWANGLIE);
    let card = state.hand_cards(owner)[0];
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotwanglie {
                player: owner,
                card: engine_api::CardId::NONE,
                helper_skill_def: SkillDefId::new(POTWANGLIE_DEBUFF),
                stage: 0,
                skill,
            }],
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, vec![card.raw()]);
    assert!(query::card_distance_unlimited_for_card(&state, owner, card));
    let physical = state.cards.get(card).unwrap();
    assert!(query::card_use_unrespondable_for_card(
        &state,
        owner,
        card,
        query::effective_card_kind(&state, owner, physical),
        query::effective_card_suit(&state, owner, physical),
    ));

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePotwanglie {
            player: owner,
            card,
            helper_skill_def: SkillDefId::new(POTWANGLIE_DEBUFF),
            stage: 1,
            skill,
        }],
    )
    .unwrap();
    assert!(query::card_targets_self_only_for(tx.state(), owner));
    assert!(!query::card_distance_unlimited_for_card(
        tx.state(),
        owner,
        card
    ));
}

#[test]
fn hongyi_caps_at_five_and_runs_the_opposite_end_phase_branch() {
    let (state, owner, skill) = setup(POTHONGYI);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePothongyi {
            player: owner,
            stage: 0,
            skill,
        }],
    )
    .unwrap();
    for _ in 0..3 {
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePothongyi {
                player: owner,
                stage: 1,
                skill,
            }],
        )
        .unwrap();
    }
    assert_eq!(mark_count(tx.state(), owner), 5);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePothongyi {
                player: owner,
                stage: 2,
                skill,
            }],
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    );
    let before = tx.state().hand_count(owner);
    let state = choose(tx.commit().state, vec![0]);
    assert_eq!(state.hand_count(owner), before + 5);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePothongyi {
            player: owner,
            stage: 3,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(mark_count(tx.state(), owner), 0);

    let (state, owner, skill) = setup(POTHONGYI);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePothongyi {
            player: owner,
            stage: 0,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePothongyi {
                player: owner,
                stage: 2,
                skill,
            }],
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, vec![1]);
    assert_eq!(mark_count(&state, owner), 0);
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePothongyi {
            player: owner,
            stage: 3,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 4);
}
}

mod original_bingshi_pot_chengpu {
//! Golden coverage for BINGSHI 势程普「督作」「避憾」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const POTDUZUO: u32 = 3_906_103_687;
const POTBIHAN: u32 = 4_060_934_534;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(515),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 515,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|role| *role != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([POTDUZUO, POTBIHAN].map(|def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, target)
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

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    serial: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(515_000 + serial),
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

fn choose(state: GameState, selections: Vec<u32>, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let (state, output) = apply_command(
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
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_complete_pot_chengpu_at_abi517() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_chengpu")
            .unwrap()
            .skills,
        [POTDUZUO, POTBIHAN]
    );
}

#[test]
fn duzuo_moves_the_topmost_fire_sha_to_the_chosen_role() {
    let (mut tx, owner, target) = setup();
    let fire_sha = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::ShaFire,
        1,
    );
    let duzuo = skill(tx.state(), owner, POTDUZUO);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotchengpu {
                player: owner,
                target: PlayerId::NONE,
                stage: 0,
                damage_id: 0,
                skill: duzuo,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, vec![target.raw()], 1);
    assert!(state.hand_cards(target).contains(&fire_sha));
}

#[test]
fn bihan_discards_to_hp_then_reduces_the_bound_sha_damage() {
    let (mut tx, owner, target) = setup();
    clear_hand(&mut tx, target);
    tx.state_mut().players.get_mut(target).unwrap().hp = 2;
    let cards: Vec<_> = (10..14)
        .map(|serial| inject(&mut tx, target, ZoneKind::Hand, CardKind::Shan, serial))
        .collect();
    let bihan = skill(tx.state(), owner, POTBIHAN);
    tx.state_mut().resolution.current_damage_id = 515;
    tx.state_mut()
        .resolution
        .deferred_tail_effects
        .push(EffectRequest::CommitDamage {
            damage_id: 515,
            source: owner,
            target,
            amount: 2,
            base_amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: CardId::NONE,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        });
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotchengpu {
                player: owner,
                target,
                stage: 2,
                damage_id: 515,
                skill: bihan,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, vec![target.raw()], 2);
    let state = choose(state, cards[..2].iter().map(|card| card.raw()).collect(), 3);
    assert_eq!(state.hand_count(target), 2);
    assert_eq!(state.players.get(target).unwrap().hp, 1);
}
}

mod original_bingshi_pot_chenjiao {
//! Golden coverage for BINGSHI 势陈矫「清严」「策断」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const POTQINGYAN: u32 = 4_232_147_498;
const POTCEDUAN: u32 = 4_257_587_443;
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/pot_chenjiao.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(def: u32, seed: u64, players: u8) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let order = engine.state().players.alive_by_seat();
    let owner = order[0];
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
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    clear_hands(&mut tx);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, order)
}

fn clear_hands(tx: &mut Transaction) {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let effects = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter_map(|player| {
            let cards = tx.state().hand_cards(player);
            (!cards.is_empty()).then(|| EffectRequest::MoveCards {
                cards,
                to: discard,
                from: tx.state().zones.find(ZoneKind::Hand, player),
            })
        })
        .collect();
    run_effects(tx, effects).unwrap();
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

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: ZoneKind,
    kind: CardKind,
    suit: Suit,
    range: u8,
    serial: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(544_000 + serial),
        kind,
        suit,
        rank: 7,
        weapon_range: range,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
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

fn resolve(state: GameState, selection: Vec<u32>, key: u64) -> (GameState, PipelineResult) {
    let actor = PlayerId::new(state.resolution.pending_observer);
    submit(
        state.clone(),
        actor,
        CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(state.resolution.pending_choice_id),
            selections: selection,
            cancelled: false,
        },
        key,
    )
}

#[test]
fn package_publishes_chenjiao_at_abi544() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(package.character_index.len(), 36);
    assert_eq!(package.skill_index.len(), 103);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_chenjiao")
            .unwrap()
            .skills,
        [POTQINGYAN, POTCEDUAN]
    );
}

#[test]
fn qingyan_keeps_shown_cards_locks_then_increases_round_cost() {
    let (state, _first, order) = setup(POTQINGYAN, 5441, 3);
    let attacker = order[0];
    let owner = order[1];
    let mut tx = Transaction::begin(state);
    let old_skill = skill(tx.state(), attacker, POTQINGYAN);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::RemoveSkill { skill: old_skill },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(POTQINGYAN),
            },
        ],
    )
    .unwrap();
    tx.state_mut().turn.current = attacker;
    tx.state_mut().turn.phase = Phase::Play;
    let attack = inject(
        &mut tx,
        attacker,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        0,
        1,
    );
    for serial in 2..=4 {
        inject(
            &mut tx,
            owner,
            ZoneKind::Hand,
            CardKind::Other,
            Suit::Heart,
            0,
            serial,
        );
    }
    let state = tx.commit().state;
    let (state, response) = submit(
        state,
        attacker,
        CommandKind::PlayCard {
            card: attack,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        5442,
    );
    let hand: std::collections::BTreeSet<_> = state
        .hand_cards(owner)
        .into_iter()
        .map(CardId::raw)
        .collect();
    let conversion = response
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .find(|token| !hand.contains(token) && !matches!(*token, 0 | 1))
        .unwrap();
    let (state, sources) = resolve(state, vec![conversion], 5443);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::MultiCardViewAsResponse
    );
    let choice = sources.choice.unwrap();
    assert_eq!((choice.min_count, choice.max_count), (1, 1));
    let shown = choice.options[0].token;
    let (state, result) = resolve(state, vec![shown], 5444);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(owner).contains(&CardId::new(shown)));

    let mut tx = Transaction::begin(state);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![CardId::new(shown)],
            to: discard,
            from: hand,
        }],
    )
    .unwrap();
    tx.state_mut().turn.set_flag(turn_flag::SHA_PLAYED, 0);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 0);
    let attack = inject(
        &mut tx,
        attacker,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Club,
        0,
        5,
    );
    let (state, response) = submit(
        tx.commit().state,
        attacker,
        CommandKind::PlayCard {
            card: attack,
            targets: vec![owner],
            card_targets: Vec::new(),
        },
        5445,
    );
    let hand: std::collections::BTreeSet<_> = state
        .hand_cards(owner)
        .into_iter()
        .map(CardId::raw)
        .collect();
    let conversion = response
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .find(|token| !hand.contains(token) && !matches!(*token, 0 | 1))
        .unwrap();
    let (state, sources) = resolve(state, vec![conversion], 5446);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::MultiCardViewAsResponse
    );
    assert_eq!(
        (
            sources.choice.as_ref().unwrap().min_count,
            sources.choice.unwrap().max_count
        ),
        (2, 2)
    );
}

#[test]
fn ceduan_reveals_range_players_uses_all_majority_cards_and_draws_on_damage() {
    let (state, owner, order) = setup(POTCEDUAN, 5447, 4);
    let target = order[2];
    let mut tx = Transaction::begin(state);
    inject(
        &mut tx,
        target,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Spade,
        3,
        10,
    );
    for serial in 11..=12 {
        inject(
            &mut tx,
            owner,
            ZoneKind::Hand,
            CardKind::Other,
            Suit::Heart,
            0,
            serial,
        );
    }
    inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Club,
        0,
        13,
    );
    for (index, candidate) in order
        .iter()
        .copied()
        .filter(|candidate| *candidate != owner)
        .enumerate()
    {
        inject(
            &mut tx,
            candidate,
            ZoneKind::Hand,
            CardKind::Other,
            Suit::Diamond,
            0,
            20 + index as u32,
        );
    }
    let ceduan = skill(tx.state(), owner, POTCEDUAN);
    let target_hp = tx.state().players.get(target).unwrap().hp;
    let (mut state, mut output) = submit(
        tx.commit().state,
        owner,
        CommandKind::ActivateSkill {
            skill: ceduan,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        5448,
    );
    let mut key = 5449;
    while state.resolution.continuation_kind == ContinuationKind::Custom {
        let choice = output.choice.take().unwrap();
        let selected = choice
            .options
            .iter()
            .map(|option| CardId::new(option.token))
            .find(|card| {
                state
                    .cards
                    .get(*card)
                    .is_some_and(|card| matches!(card.suit, Suit::Heart | Suit::Diamond))
            })
            .unwrap();
        (state, output) = resolve(state, vec![selected.raw()], key);
        key += 1;
    }
    while state.has_pending_choice() {
        let choice = output.choice.take().unwrap();
        let pass = choice
            .options
            .iter()
            .find(|option| option.token == 0)
            .map_or(choice.options[0].token, |option| option.token);
        (state, output) = resolve(state, vec![pass], key);
        key += 1;
    }
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert_eq!(
        state.hand_count(owner),
        2,
        "两张红牌作为杀，黑牌保留，造成伤害后摸一"
    );
}
}

mod original_bingshi_pot_chenqun {
//! Golden coverage for BINGSHI 势陈群「法恩」「定品」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{turn_flag, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const POTFAEN: u32 = 4_194_455_770;
const POTDINGPIN: u32 = 2_518_587_448;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(skill_def: u32) -> (GameState, PlayerId, Vec<PlayerId>, engine_api::SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(518),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 518,
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
        .collect::<Vec<_>>();
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
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == skill_def)
        .unwrap()
        .id;
    (tx.commit().state, owner, others, skill)
}

fn choose(state: GameState, selections: Vec<u32>, cancelled: bool, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let (state, output) = apply_command(
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
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_complete_pot_chenqun_at_abi518() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_chenqun")
            .unwrap()
            .skills,
        [POTFAEN, POTDINGPIN]
    );
}

#[test]
fn faen_remembers_previous_user_and_applies_the_opposite_to_the_next_user() {
    let (state, owner, others, skill) = setup(POTFAEN);
    let target = others[0];
    let next = others[1];
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePotchenqun {
            player: owner,
            target: owner,
            stage: 0,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotchenqun {
                player: owner,
                target,
                stage: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let before_draw = tx.state().hand_count(target);
    let state = choose(tx.commit().state, vec![1], false, 1);
    assert_eq!(state.hand_count(target), before_draw + 1);

    let before_discard = state.hand_count(next);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotchenqun {
                player: owner,
                target: next,
                stage: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(PlayerId::new(tx.state().resolution.pending_observer), next);
    let card = tx.state().resolution.continuation_payload[6];
    let state = choose(tx.commit().state, vec![card], false, 2);
    assert_eq!(state.hand_count(next), before_discard - 1);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePotchenqun {
            player: owner,
            target: owner,
            stage: 0,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotchenqun {
                player: owner,
                target,
                stage: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, vec![2], false, 3);
    let discard = state.resolution.continuation_payload[5];
    let state = choose(state, vec![discard], false, 4);
    let before_draw = state.hand_count(next);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePotchenqun {
            player: owner,
            target: next,
            stage: 0,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(next), before_draw + 1);
}

#[test]
fn dingpin_queues_a_real_draw_phase_for_a_maximum_loss_target() {
    let (state, owner, others, skill) = setup(POTDINGPIN);
    let target = others[0];
    let mut tx = Transaction::begin(state);
    let lost = tx.state().hand_cards(target)[0];
    tx.state_mut().turn.record_card_loss(target, lost);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotchenqun {
                player: owner,
                target: owner,
                stage: 1,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, vec![target.raw()], false, 3);
    assert_eq!(state.turn.extra_draw_phase_queue, [target]);
    let before = state.hand_count(target);
    let mut tx = Transaction::begin(state);
    run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
    assert_eq!(tx.state().turn.extra_draw_phase_queue, [target]);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::TURN_END_DISPATCHED, 1);
    run_effects(&mut tx, vec![EffectRequest::AdvanceTurn]).unwrap();
    assert_eq!(tx.state().turn.current, target);
    assert_eq!(tx.state().turn.phase, Phase::Draw);
    assert!(tx.state().turn.extra_draw_phase_active);
    assert_eq!(tx.state().hand_count(target), before + 2);
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: target,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(4),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(!state.turn.extra_draw_phase_active);
    assert_eq!(state.turn.phase, Phase::Prepare);
}
}

mod original_bingshi_pot_dengai {
//! Golden coverage for BINGSHI 势邓艾「屯田」「凿险」「急袭」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const POTTUNTIAN: u32 = 2_435_155_097;
const POTZAOXIAN: u32 = 2_551_307_508;
const POTJIXI: u32 = 4_126_984_108;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(defs: &[u32], seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let mut engine = Engine::new();
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
    let owner = engine.state().turn.current;
    let order = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(defs.iter().map(|def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(*def),
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, order)
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

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone: ZoneKind,
    kind: CardKind,
    suit: Suit,
    serial: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(543_000 + serial),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: PlayerId::new(state.resolution.pending_observer),
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

fn charge(state: &GameState, owner: PlayerId) -> i32 {
    state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == POTTUNTIAN)
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[test]
fn package_publishes_dengai_at_abi543() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(package.character_index.len(), 36);
    assert_eq!(package.skill_index.len(), 103);
    assert_eq!(package.card_index.len(), 1);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_dengai")
            .unwrap()
            .skills,
        [POTTUNTIAN, POTZAOXIAN, POTJIXI]
    );
}

#[test]
fn tuntian_grows_charges_spends_and_applies_zaoxian() {
    let (state, owner, order) = setup(&[POTTUNTIAN, POTZAOXIAN], 5431);
    let targets: Vec<_> = order
        .into_iter()
        .filter(|target| *target != owner)
        .collect();
    let mut tx = Transaction::begin(state);
    let tuntian = skill(tx.state(), owner, POTTUNTIAN);
    let zaoxian = skill(tx.state(), owner, POTZAOXIAN);
    let before_draw = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePottuntian {
            player: owner,
            action: 1,
            amount: 0,
            targets: Vec::new(),
            zaoxian_def: SkillDefId::new(POTZAOXIAN),
            skill: tuntian,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before_draw + 1);
    for action in [0, 1, 0] {
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePottuntian {
                player: owner,
                action,
                amount: 0,
                targets: Vec::new(),
                zaoxian_def: SkillDefId::new(POTZAOXIAN),
                skill: tuntian,
            }],
        )
        .unwrap();
    }
    assert_eq!(charge(tx.state(), owner), 2);
    let discard_owner = PlayerId::NONE;
    inject(
        &mut tx,
        discard_owner,
        ZoneKind::DiscardPile,
        CardKind::TrickWuzhong,
        Suit::Spade,
        1,
    );
    for (index, target) in targets.iter().enumerate() {
        inject(
            &mut tx,
            PlayerId::NONE,
            ZoneKind::DrawPile,
            CardKind::Other,
            Suit::Heart,
            10 + index as u32,
        );
        assert!(tx.state().players.get(*target).unwrap().alive);
    }
    let target_counts: Vec<_> = targets
        .iter()
        .map(|target| tx.state().hand_count(*target))
        .collect();
    let wuzhong_before = tx
        .state()
        .hand_cards(owner)
        .into_iter()
        .filter(|card| tx.state().cards.get(*card).unwrap().kind == CardKind::TrickWuzhong)
        .count();
    assert!(tx.state().skills.get(zaoxian).is_some());
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePottuntian {
                player: owner,
                action: 2,
                amount: 0,
                targets: targets.clone(),
                zaoxian_def: SkillDefId::new(POTZAOXIAN),
                skill: tuntian,
            }],
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    let (state, output) = resolve(tx.commit().state, vec![2], false, 5432);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(charge(&state, owner), 0);
    for (target, before) in targets.iter().zip(target_counts) {
        assert_eq!(state.hand_count(*target), before + 1);
    }
    assert_eq!(
        state
            .hand_cards(owner)
            .into_iter()
            .filter(|card| state.cards.get(*card).unwrap().kind == CardKind::TrickWuzhong)
            .count(),
        wuzhong_before + 1
    );
}

#[test]
fn jixi_records_targets_discards_then_starts_multi_target_shunshou() {
    let (state, owner, order) = setup(&[POTJIXI], 5433);
    let targets: Vec<_> = order
        .into_iter()
        .filter(|target| *target != owner)
        .collect();
    let mut tx = Transaction::begin(state);
    let jixi = skill(tx.state(), owner, POTJIXI);
    let discard = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Club,
        20,
    );
    for (index, target) in targets.iter().enumerate() {
        inject(
            &mut tx,
            *target,
            ZoneKind::Hand,
            CardKind::Other,
            Suit::Diamond,
            30 + index as u32,
        );
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotjixi {
                player: owner,
                target: *target,
                action: 1,
                targets: Vec::new(),
                skill: jixi,
            }],
        )
        .unwrap();
    }
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotjixi {
                player: owner,
                target: owner,
                action: 2,
                targets: Vec::new(),
                skill: jixi,
            }],
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    let (state, output) = resolve(tx.commit().state, vec![discard.raw()], false, 5434);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    let (state, output) = resolve(
        state,
        targets.iter().map(|target| target.raw()).collect(),
        false,
        5435,
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(
        state.has_pending_choice(),
        "虚拟顺手牵羊应进入独立无懈/获得结算队列"
    );
}
}

mod original_bingshi_pot_dongzhao {
//! Golden coverage for BINGSHI 势董昭「妙略」「迎驾」与「瞒天过海」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::query::card_ignored_for_hand_limit;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SPMIAOLVE: u32 = 2_347_949_240;
const SPYINGJIA: u32 = 3_012_325_144;
const SPYINGJIA_DRAW: u32 = 3_612_952_432;
const MANTIANGUOHAI: u32 = 3_442_665_819;
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/pot_dongzhao.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(def: u32, seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
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
    let owner = engine.state().turn.current;
    let order = engine.state().players.alive_by_seat();
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
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    clear_hands(&mut tx);
    (tx.commit().state, owner, order)
}

fn clear_hands(tx: &mut Transaction) {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let effects = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter_map(|player| {
            let cards = tx.state().hand_cards(player);
            (!cards.is_empty()).then(|| EffectRequest::MoveCards {
                cards,
                to: discard,
                from: tx.state().zones.find(ZoneKind::Hand, player),
            })
        })
        .collect();
    run_effects(tx, effects).unwrap();
}

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, serial: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(539_000 + serial),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
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

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let revision = state.revision;
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_dongzhao_and_mantianguohai_at_abi539() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(package.character_index.len(), 36);
    assert_eq!(package.skill_index.len(), 103);
    assert_eq!(package.card_index.len(), 1);
    assert_eq!(package.card_index[0].card_def_id, MANTIANGUOHAI);
    assert_eq!(package.card_index[0].kind, "mantianguohai");
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_dongzhao")
            .unwrap()
            .skills,
        [SPMIAOLVE, SPYINGJIA]
    );
}

#[test]
fn miaolve_creates_distinct_suits_and_resolves_damage_reward() {
    let (state, owner, _) = setup(SPMIAOLVE, 5391);
    let mut tx = Transaction::begin(state);
    let skill = skill(tx.state(), owner, SPMIAOLVE);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpmiaolve {
            player: owner,
            action: 0,
            skill,
        }],
    )
    .unwrap();
    let cards: Vec<_> = tx
        .state()
        .hand_cards(owner)
        .into_iter()
        .filter(|card| tx.state().cards.get(*card).unwrap().kind == CardKind::TrickMantianguohai)
        .collect();
    assert_eq!(cards.len(), 2);
    assert_ne!(
        tx.state().cards.get(cards[0]).unwrap().suit,
        tx.state().cards.get(cards[1]).unwrap().suit
    );
    assert!(cards
        .iter()
        .all(|card| card_ignored_for_hand_limit(tx.state(), owner, *card)));
    let before = tx.state().hand_count(owner);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveSpmiaolve {
                player: owner,
                action: 1,
                skill,
            }],
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    let (state, output) = resolve(tx.commit().state, vec![0], false, 5392);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 2);
}

#[test]
fn yingjia_requires_a_repeated_trick_then_queues_one_draw_turn() {
    let (state, owner, order) = setup(SPYINGJIA, 5393);
    let target = order.into_iter().find(|player| *player != owner).unwrap();
    let mut tx = Transaction::begin(state);
    let cost = inject(&mut tx, owner, CardKind::Other, 1);
    let trick = inject(&mut tx, owner, CardKind::TrickGuohe, 2);
    let owner_skill = skill(tx.state(), owner, SPYINGJIA);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::ResolveSpyingjia {
                player: owner,
                action: 0,
                helper_def: SkillDefId::new(SPYINGJIA_DRAW),
                card: CardId::NONE,
                card_kind: 0,
                target: PlayerId::NONE,
                skill: owner_skill,
            },
            EffectRequest::ResolveSpyingjia {
                player: owner,
                action: 1,
                helper_def: SkillDefId::new(SPYINGJIA_DRAW),
                card: trick,
                card_kind: CardKind::TrickGuohe.tag(),
                target: PlayerId::NONE,
                skill: owner_skill,
            },
            EffectRequest::ResolveSpyingjia {
                player: owner,
                action: 1,
                helper_def: SkillDefId::new(SPYINGJIA_DRAW),
                card: trick,
                card_kind: CardKind::TrickGuohe.tag(),
                target: PlayerId::NONE,
                skill: owner_skill,
            },
        ],
    )
    .unwrap();
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveSpyingjia {
                player: owner,
                action: 2,
                helper_def: SkillDefId::new(SPYINGJIA_DRAW),
                card: CardId::NONE,
                card_kind: 0,
                target: PlayerId::NONE,
                skill: owner_skill,
            }],
        ),
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));
    let (state, _) = resolve(tx.commit().state, vec![cost.raw()], false, 5394);
    let (state, output) = resolve(state, vec![target.raw()], false, 5395);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.extra_turn_queue, [target]);
    assert!(!state.hand_cards(owner).contains(&cost));
    let helper = skill(&state, target, SPYINGJIA_DRAW);
    let before = state.hand_count(target);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSpyingjia {
            player: target,
            action: 3,
            helper_def: SkillDefId::new(SPYINGJIA_DRAW),
            card: CardId::NONE,
            card_kind: 0,
            target: PlayerId::NONE,
            skill: helper,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(target), before + 2);
    assert!(tx.state().skills.get(helper).is_none());
}

#[test]
fn mantianguohai_gains_then_gives_each_successful_target_in_seat_order() {
    let (state, actor, order) = setup(SPMIAOLVE, 5396);
    let targets: Vec<_> = order
        .into_iter()
        .filter(|player| *player != actor)
        .collect();
    let mut tx = Transaction::begin(state);
    let card = inject(&mut tx, actor, CardKind::TrickMantianguohai, 10);
    let gift_a = inject(&mut tx, actor, CardKind::Other, 11);
    let gift_b = inject(&mut tx, actor, CardKind::Other, 12);
    let take_a = inject(&mut tx, targets[0], CardKind::Other, 13);
    let take_b = inject(&mut tx, targets[1], CardKind::Other, 14);
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(5397),
            kind: CommandKind::PlayCard {
                card,
                targets: targets.clone(),
                card_targets: Vec::new(),
            },
        },
    );
    assert!(state.has_pending_choice(), "{:?}", output.result);
    let (state, _) = resolve(state, vec![take_a.raw()], false, 5398);
    let (state, _) = resolve(state, vec![take_b.raw()], false, 5399);
    let (state, _) = resolve(state, vec![gift_a.raw()], false, 5400);
    let (state, output) = resolve(state, vec![gift_b.raw()], false, 5401);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(actor).contains(&take_a));
    assert!(state.hand_cards(actor).contains(&take_b));
    assert!(state.hand_cards(targets[0]).contains(&gift_a));
    assert!(state.hand_cards(targets[1]).contains(&gift_b));
}
}

mod original_bingshi_pot_huanjie {
//! Golden coverage for BINGSHI 势桓阶「共谋」「正朔」 and derived「奇策」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const POTGONGMOU: u32 = 3_894_451_488;
const POTZHENGSHUO: u32 = 2_346_184_405;
const QICE: u32 = 3_567_101_194;
const KANPO: u32 = 2_834_927_312;
const HUANJIE: u32 = 2_873_796_786;
const BINGSHI: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");
const YIJIANG: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");
const SHENHUA: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

fn setup(defs: &[u32]) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let mut engine = Engine::new();
    for bytes in [YIJIANG, SHENHUA, BINGSHI] {
        engine
            .load_package(&content_schema::ContentPackage::from_json_bytes(bytes).unwrap())
            .unwrap();
    }
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(529),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 529,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let order = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(defs.iter().copied().map(|def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, order)
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

fn inject(tx: &mut Transaction, owner: PlayerId, serial: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(529_000 + serial),
        kind: CardKind::Shan,
        suit: Suit::Heart,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, selection: u32, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let revision = state.revision;
    let choice_id = state.resolution.pending_choice_id;
    let (state, output) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice_id),
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "{:?}",
        output.result
    );
    state
}

#[test]
fn package_publishes_huanjie_and_cross_package_derivations_at_abi529() {
    let package = content_schema::ContentPackage::from_json_bytes(BINGSHI).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == HUANJIE)
            .unwrap()
            .skills,
        [POTGONGMOU, POTZHENGSHUO]
    );
    assert!(package
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "yijiang"));
    assert!(package
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "shenhua"));
}

#[test]
fn gongmou_swaps_whole_hands_and_grants_qice_and_kanpo_until_turn_end() {
    let (mut tx, owner, order) = setup(&[POTGONGMOU]);
    let target = order.into_iter().find(|role| *role != owner).unwrap();
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, target);
    let owner_card = inject(&mut tx, owner, 1);
    let target_card = inject(&mut tx, target, 2);
    let gongmou = skill(tx.state(), owner, POTGONGMOU);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePothuanjie {
                player: owner,
                target: PlayerId::NONE,
                stage: 0,
                qice_def: SkillDefId::new(QICE),
                kanpo_def: SkillDefId::new(KANPO),
                skill: gongmou,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, target.raw(), 1);
    assert_eq!(state.hand_cards(owner), [target_card]);
    assert_eq!(state.hand_cards(target), [owner_card]);
    assert_eq!(skill(&state, owner, QICE).raw() != 0, true);
    assert_eq!(skill(&state, target, KANPO).raw() != 0, true);
}

#[test]
fn zhengshuo_discards_every_hand_then_reshuffles_and_deals_four() {
    let (mut tx, owner, order) = setup(&[POTZHENGSHUO]);
    let before = tx.state().draw_reshuffle_count;
    let zhengshuo = skill(tx.state(), owner, POTZHENGSHUO);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePothuanjie {
            player: owner,
            target: PlayerId::NONE,
            stage: 2,
            qice_def: SkillDefId::new(QICE),
            kanpo_def: SkillDefId::new(KANPO),
            skill: zhengshuo,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().draw_reshuffle_count, before + 1);
    assert!(order
        .into_iter()
        .all(|player| tx.state().hand_count(player) == 4));
}

#[test]
fn qice_consumes_every_original_hand_card_as_one_ordinary_trick() {
    let (mut tx, owner, _) = setup(&[QICE]);
    clear_hand(&mut tx, owner);
    let materials = [
        inject(&mut tx, owner, 10),
        inject(&mut tx, owner, 11),
        inject(&mut tx, owner, 12),
    ];
    let qice = skill(tx.state(), owner, QICE);
    let state = tx.commit().state;
    let revision = state.revision;
    let (mut state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(20),
            kind: CommandKind::UseSkillCard {
                skill: qice,
                card_kind: CardKind::TrickTaoyuan.tag(),
                targets: Vec::new(),
                equip_slot: 0,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let mut key = 21;
    while state.has_pending_choice() {
        let actor = PlayerId::new(state.resolution.pending_observer);
        let revision = state.revision;
        let choice_id = state.resolution.pending_choice_id;
        let (next, output) = apply_command(
            state,
            Command {
                actor,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: ChoiceId::new(choice_id),
                    selections: Vec::new(),
                    cancelled: true,
                },
            },
        );
        assert!(matches!(output.result, CommandResult::Accepted { .. }));
        state = next;
        key += 1;
    }
    assert_eq!(
        materials
            .iter()
            .map(|card| {
                state
                    .zones
                    .get(state.cards.get(*card).unwrap().zone)
                    .unwrap()
                    .kind
            })
            .collect::<Vec<_>>(),
        vec![ZoneKind::DiscardPile; materials.len()]
    );
    assert_eq!(state.hand_count(owner), 0);
}
}

mod original_bingshi_pot_lougui {
//! Golden coverage for BINGSHI 势娄圭「灌沙」「急御」.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const POTGUANSHA: u32 = 2_542_568_623;
const POTGUANSHA_HAND: u32 = 2_542_568_624;
const POTJIYU: u32 = 3_103_425_312;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(skill_def: u32) -> (engine_core::state::GameState, PlayerId, engine_api::SkillId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(520),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 520,
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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == skill_def)
        .unwrap()
        .id;
    (tx.commit().state, owner, skill)
}

fn hand_limit(state: &engine_core::state::GameState, player: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::HandLimit { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected hand-limit response: {other:?}"),
    }
}

fn category(
    state: &engine_core::state::GameState,
    player: PlayerId,
    card: engine_api::CardId,
) -> u8 {
    let kind =
        engine_core::query::effective_card_kind(state, player, state.cards.get(card).unwrap());
    if kind.is_basic() {
        1
    } else if kind.is_trick() {
        2
    } else if kind.is_equip() {
        3
    } else {
        0
    }
}

#[test]
fn package_publishes_complete_pot_lougui_at_abi520() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    let character = package
        .character_index
        .iter()
        .find(|character| character.name_key == "pot_lougui")
        .unwrap();
    assert_eq!(character.skills, vec![POTGUANSHA, POTJIYU]);
}

#[test]
fn guansha_replaces_every_hand_card_with_basics_and_adds_equal_hand_limit() {
    let (state, owner, skill) = setup(POTGUANSHA);
    let before_limit = hand_limit(&state, owner);
    let replaced = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePotguansha {
            player: owner,
            helper_skill_def: SkillDefId::new(POTGUANSHA_HAND),
            skill,
        }],
    )
    .unwrap();
    let gained = tx.state().hand_cards(owner);
    assert_eq!(gained.len() as u32, replaced);
    assert!(gained
        .iter()
        .all(|card| category(tx.state(), owner, *card) == 1));
    assert_eq!(
        hand_limit(tx.state(), owner),
        before_limit + replaced as i32
    );
}

#[test]
fn jiyu_gains_other_categories_and_only_resets_twice() {
    let (state, owner, skill) = setup(POTJIYU);
    let mut state = state;
    for expected_reset in [true, true, false] {
        let selected = state.hand_cards(owner)[0];
        let selected_category = category(&state, owner, selected);
        let before = state.hand_cards(owner);
        let mut tx = Transaction::begin(state);
        tx.state_mut().skills.get_mut(skill).unwrap().use_count = 1;
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotjiyu {
                player: owner,
                card: selected,
                stage: 0,
                skill,
            }],
        )
        .unwrap();
        let gained: Vec<_> = tx
            .state()
            .hand_cards(owner)
            .into_iter()
            .filter(|card| !before.contains(card))
            .collect();
        assert!(!gained.is_empty());
        assert!(gained
            .iter()
            .all(|card| category(tx.state(), owner, *card) != selected_category));
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: owner,
                cards: gained,
            }],
        )
        .unwrap();
        assert_eq!(
            tx.state().skills.get(skill).unwrap().use_count == 0,
            expected_reset
        );
        state = tx.commit().state;
    }
}
}

mod original_bingshi_pot_lusu {
//! Golden coverage for BINGSHI 势鲁肃「好施/缔盟」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::projection::project_player;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const POTDIMENG: u32 = 2_916_721_654;
const POTHAOSHI: u32 = 3_050_076_876;
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/pot_lusu.skill");

fn setup(seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 536);
    assert!(package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == POTDIMENG)
        .unwrap()
        .targets
        .as_ref()
        .is_some_and(|targets| targets.pair_hand_difference_lte_three));
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
    let owner = engine.state().turn.current;
    let order = engine.state().players.alive_by_seat();
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
        skill_def: SkillDefId::new(POTDIMENG),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
    owner_state.max_hp = 4;
    owner_state.hp = 2;
    clear_hands(&mut tx);
    (tx.commit().state, owner, order)
}

fn clear_hands(tx: &mut Transaction) {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut effects = Vec::new();
    for player in tx.state().players.alive_by_seat() {
        let cards = tx.state().hand_cards(player);
        if !cards.is_empty() {
            effects.push(EffectRequest::MoveCards {
                cards,
                to: discard,
                from: tx.state().zones.find(ZoneKind::Hand, player),
            });
        }
    }
    run_effects(tx, effects).unwrap();
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, serial: u32) -> CardId {
    inject_hand_kind(tx, player, serial, CardKind::Other)
}

fn inject_hand_kind(tx: &mut Transaction, player: PlayerId, serial: u32, kind: CardKind) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(535_000 + serial),
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

fn skill(state: &GameState, owner: PlayerId) -> engine_api::SkillId {
    skill_by_def(state, owner, POTDIMENG)
}

fn skill_by_def(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let revision = state.revision;
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections,
                cancelled,
            },
        },
    )
}

fn grant_pothaoshi(state: GameState, owner: PlayerId, borrower: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let skill = skill_by_def(tx.state(), owner, POTHAOSHI);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePothaoshi {
            player: owner,
            target: PlayerId::NONE,
            action: 0,
            skill,
        }],
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let (state, output) = resolve(tx.commit().state, vec![borrower.raw()], false, 5360);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

fn setup_pothaoshi(seed: u64) -> (GameState, PlayerId, PlayerId, PlayerId) {
    let (state, owner, order) = setup(seed);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(POTHAOSHI),
        }],
    )
    .unwrap();
    let others: Vec<_> = order
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let borrower = others[0];
    let third = others[1];
    (
        grant_pothaoshi(tx.commit().state, owner, borrower),
        owner,
        borrower,
        third,
    )
}

#[test]
fn potdimeng_revalidates_pair_swaps_hands_and_settles_both_choices() {
    let (state, owner, order) = setup(5350);
    let target = order.into_iter().find(|player| *player != owner).unwrap();
    let mut tx = Transaction::begin(state);
    for serial in 20..=23 {
        inject_hand(&mut tx, target, serial);
    }
    let state = tx.commit().state;
    let (_, output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::ActivateSkill {
                skill: skill(&state, owner),
                targets: vec![owner, target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(!matches!(output.result, CommandResult::Accepted { .. }));

    let (state, owner, order) = setup(5351);
    let target = order.into_iter().find(|player| *player != owner).unwrap();
    let mut tx = Transaction::begin(state);
    inject_hand(&mut tx, owner, 1);
    for serial in 2..=5 {
        inject_hand(&mut tx, target, serial);
    }
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: skill(&state, owner),
                targets: vec![owner, target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 4);
    assert_eq!(state.hand_count(target), 1);
    let (state, output) = resolve(state, Vec::new(), true, 2);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 4);
    assert_eq!(state.hand_count(target), 3);

    let (state, owner, order) = setup(5352);
    let target = order.into_iter().find(|player| *player != owner).unwrap();
    let mut tx = Transaction::begin(state);
    inject_hand(&mut tx, owner, 10);
    for serial in 11..=14 {
        inject_hand(&mut tx, target, serial);
    }
    let state = tx.commit().state;
    let (state, _) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ActivateSkill {
                skill: skill(&state, owner),
                targets: vec![owner, target],
                card_targets: Vec::new(),
            },
        },
    );
    let cards: Vec<_> = state.hand_cards(owner)[..2]
        .iter()
        .map(|card| card.raw())
        .collect();
    let (state, output) = resolve(state, cards, false, 4);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.hand_count(target), 1);
}

#[test]
fn pothaoshi_borrows_visible_hand_refills_twice_and_clears() {
    let (state, owner, borrower, third) = setup_pothaoshi(5361);
    let mut tx = Transaction::begin(state);
    let first = inject_hand_kind(&mut tx, owner, 101, CardKind::Jiu);
    tx.state_mut().turn.current = borrower;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;

    assert!(state.playable_hand_cards(borrower).contains(&first));
    assert!(project_player(&state, borrower)
        .known_cards
        .iter()
        .any(|card| card.id == first && card.def.is_some()));
    assert!(!project_player(&state, third)
        .known_cards
        .iter()
        .any(|card| card.id == first && card.def.is_some()));

    let mut state = state;
    for (index, serial) in [101, 102, 103].into_iter().enumerate() {
        if index > 0 {
            let mut tx = Transaction::begin(state);
            let discard = tx
                .state()
                .zones
                .find(ZoneKind::DiscardPile, PlayerId::NONE)
                .unwrap();
            let hand = tx.state().zones.find(ZoneKind::Hand, owner);
            let cards = tx.state().hand_cards(owner);
            if !cards.is_empty() {
                run_effects(
                    &mut tx,
                    vec![EffectRequest::MoveCards {
                        cards,
                        to: discard,
                        from: hand,
                    }],
                )
                .unwrap();
            }
            inject_hand_kind(&mut tx, owner, serial, CardKind::Jiu);
            state = tx.commit().state;
        }
        let card = state.hand_cards(owner)[0];
        let (next, output) = apply_command(
            state.clone(),
            Command {
                actor: borrower,
                expected_revision: ExpectedRevision(state.revision),
                idempotency_key: IdempotencyKey(5362 + index as u64),
                kind: CommandKind::PlayCard {
                    card,
                    targets: Vec::new(),
                    card_targets: Vec::new(),
                },
            },
        );
        assert!(matches!(output.result, CommandResult::Accepted { .. }));
        assert_eq!(next.hand_count(owner), if index < 2 { 3 } else { 0 });
        state = next;
    }

    let mut tx = Transaction::begin(state);
    let skill = skill_by_def(tx.state(), owner, POTHAOSHI);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePothaoshi {
            player: owner,
            target: PlayerId::NONE,
            action: 1,
            skill,
        }],
    )
    .unwrap();
    let after_clear = inject_hand_kind(&mut tx, owner, 104, CardKind::Jiu);
    assert!(!tx
        .state()
        .playable_hand_cards(borrower)
        .contains(&after_clear));
}

#[test]
fn pothaoshi_borrowed_shan_uses_the_response_pipeline() {
    let (state, owner, borrower, attacker) = setup_pothaoshi(5365);
    let mut tx = Transaction::begin(state);
    let shan = inject_hand_kind(&mut tx, owner, 201, CardKind::Shan);
    let sha = inject_hand_kind(&mut tx, attacker, 202, CardKind::Sha);
    tx.state_mut().turn.current = attacker;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: attacker,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(5366),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![borrower],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(PlayerId::new(state.resolution.pending_observer), borrower);
    let (state, output) = resolve(state, vec![shan.raw()], false, 5367);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 3);
    assert!(state
        .players
        .get(borrower)
        .is_some_and(|player| player.hp == player.max_hp as i32));
}
}

mod original_bingshi_pot_lvyi {
//! Golden coverage for BINGSHI 势吕壹「隳律/颂谳/势施」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const HUILV: u32 = 4_200_000_570;
const SONGYAN: u32 = 4_200_000_571;
const SHISHI: u32 = 4_200_000_572;
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/pot_lvyi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 564);
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
    let order = engine.state().players.alive_by_seat();
    let owner = order[0];
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([HUILV, SONGYAN, SHISHI].map(|def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    }));
    run_effects(&mut tx, effects).unwrap();
    clear_hands(&mut tx);
    (tx.commit().state, owner, order)
}

fn clear_hands(tx: &mut Transaction) {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let effects: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter_map(|player| {
            let cards = tx.state().hand_cards(player);
            (!cards.is_empty()).then(|| EffectRequest::MoveCards {
                cards,
                to: discard,
                from: tx.state().zones.find(ZoneKind::Hand, player),
            })
        })
        .collect();
    run_effects(tx, effects).unwrap();
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    serial: u32,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let zone_owner = if matches!(
        zone_kind,
        ZoneKind::Hand | ZoneKind::Equip | ZoneKind::Expansion
    ) {
        owner
    } else {
        PlayerId::NONE
    };
    let zone = tx.state().zones.find(zone_kind, zone_owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(564_000 + serial),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: zone_owner,
        face_up: zone_kind != ZoneKind::Hand,
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

fn request(
    player: PlayerId,
    target: PlayerId,
    card: CardId,
    action: u8,
    event_cards: Vec<CardId>,
    skill: engine_api::SkillId,
) -> EffectRequest {
    EffectRequest::ResolvePotlvyi {
        player,
        target,
        card,
        action,
        event_cards,
        targets: Vec::new(),
        skill,
    }
}

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: PlayerId::new(state.resolution.pending_observer),
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
fn package_publishes_complete_pot_lvyi_at_abi564() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(package.character_index.len(), 36);
    assert_eq!(package.skill_index.len(), 103);
    assert_eq!(package.card_index.len(), 1);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_lvyi")
            .unwrap()
            .skills,
        [HUILV, SONGYAN, SHISHI]
    );
}

#[test]
fn huilv_rejects_a_pending_choice_after_the_skill_is_removed() {
    let (state, owner, order) = setup(5640);
    let actor = order[1];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        0,
        CardKind::Sha,
        Suit::Heart,
    );
    let huilv = skill(tx.state(), owner, HUILV);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![request(owner, actor, CardId::NONE, 0, Vec::new(), huilv)]
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let mut pending = tx.commit().state;
    pending.skills.get_mut(huilv).unwrap().enabled = false;
    let (_, result) = resolve(pending, vec![1], false, 5641);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
}

#[test]
fn huilv_tracks_the_exact_used_card_and_damages_the_phase_actor() {
    let (state, owner, order) = setup(5641);
    let actor = order[1];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    let tracked = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        1,
        CardKind::Sha,
        Suit::Heart,
    );
    let huilv = skill(tx.state(), owner, HUILV);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![request(owner, actor, CardId::NONE, 0, Vec::new(), huilv)]
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, result) = resolve(tx.commit().state, vec![1], false, 5642);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(actor).contains(&tracked));

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![request(owner, actor, tracked, 1, Vec::new(), huilv)],
    )
    .unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::MoveCardsWithReason {
            cards: vec![tracked],
            to: discard,
            from: Some(hand),
            reason: engine_api::CardMoveReason::Use,
        }],
    )
    .unwrap();
    let hp_before = tx.state().players.get(actor).unwrap().hp;
    run_effects(
        &mut tx,
        vec![request(owner, actor, CardId::NONE, 2, Vec::new(), huilv)],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(actor).unwrap().hp, hp_before - 1);
    assert_eq!(tx.state().resolution.pending_choice_id, 0);
}

#[test]
fn huilv_card_outside_discard_causes_unreal_damage_then_can_be_given_away() {
    let (state, owner, order) = setup(5643);
    let actor = order[1];
    let recipient = order[2];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    let tracked = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        2,
        CardKind::Tao,
        Suit::Diamond,
    );
    let huilv = skill(tx.state(), owner, HUILV);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![request(owner, actor, CardId::NONE, 0, Vec::new(), huilv)]
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, _) = resolve(tx.commit().state, vec![1], false, 5644);
    let hp_before = state.players.get(owner).unwrap().hp;
    let mut tx = Transaction::begin(state);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![request(owner, actor, CardId::NONE, 2, Vec::new(), huilv)]
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let pending_songyan = tx.commit().state;
    assert_eq!(
        pending_songyan.players.get(owner).unwrap().hp,
        hp_before - 1
    );
    let (pending_gift, result) = resolve(pending_songyan, Vec::new(), true, 5645);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_ne!(pending_gift.resolution.pending_choice_id, 0);
    let (state, result) = resolve(pending_gift, vec![recipient.raw()], false, 5646);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(recipient).contains(&tracked));
    assert!(!state.hand_cards(actor).contains(&tracked));
}

#[test]
fn songyan_allows_matching_round_discard_color_as_wuxie() {
    let (state, owner, order) = setup(5646);
    let responder = order[1];
    let mut tx = Transaction::begin(state);
    let discarded = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        3,
        CardKind::Sha,
        Suit::Spade,
    );
    let response = inject(
        &mut tx,
        responder,
        ZoneKind::Hand,
        4,
        CardKind::Tao,
        Suit::Club,
    );
    let songyan = skill(tx.state(), owner, SONGYAN);
    run_effects(
        &mut tx,
        vec![request(
            owner,
            responder,
            CardId::NONE,
            3,
            vec![discarded],
            songyan,
        )],
    )
    .unwrap();
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![request(owner, owner, CardId::NONE, 4, Vec::new(), songyan)]
        ),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(
        PlayerId::new(tx.state().resolution.pending_observer),
        responder
    );
    let (state, result) = resolve(tx.commit().state, vec![response.raw()], false, 5648);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.hand_cards(responder).contains(&response));
}

#[test]
fn shishi_freezes_each_card_then_stops_recording_after_first_response() {
    let (state, owner, order) = setup(5647);
    let source = order[1];
    let mut tx = Transaction::begin(state);
    let wuxie = inject(
        &mut tx,
        source,
        ZoneKind::Hand,
        5,
        CardKind::TrickWuxie,
        Suit::Spade,
    );
    let shishi = skill(tx.state(), owner, SHISHI);
    run_effects(
        &mut tx,
        vec![request(owner, source, CardId::NONE, 6, Vec::new(), shishi)],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::UseVirtualCardOn {
            player: owner,
            kind: CardKind::TrickWuzhong.tag(),
            target: owner,
        }],
    )
    .unwrap();
    assert!(tx.state().hand_cards(source).contains(&wuxie));
    assert_eq!(tx.state().resolution.pending_choice_id, 0);

    run_effects(
        &mut tx,
        vec![request(owner, source, wuxie, 8, Vec::new(), shishi)],
    )
    .unwrap();
    clear_hands(&mut tx);
    let wuxie = inject(
        &mut tx,
        source,
        ZoneKind::Hand,
        6,
        CardKind::TrickWuxie,
        Suit::Club,
    );
    run_effects(
        &mut tx,
        vec![request(owner, source, CardId::NONE, 6, Vec::new(), shishi)],
    )
    .unwrap();
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::UseVirtualCardOn {
                player: owner,
                kind: CardKind::TrickWuzhong.tag(),
                target: owner,
            }]
        ),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(
        PlayerId::new(tx.state().resolution.pending_observer),
        source
    );
    assert!(tx.state().hand_cards(source).contains(&wuxie));
}
}

mod original_bingshi_pot_sunchen {
//! Golden coverage for BINGSHI 势孙綝「逆固」「戮连」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const POTNIGU: u32 = 3_220_807_381;
const POTLULIAN: u32 = 2_734_921_313;
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/pot_sunchen.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(def: u32, seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
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
    let owner = engine.state().turn.current;
    let order = engine.state().players.alive_by_seat();
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
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, order)
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

fn clear_hands(tx: &mut Transaction) {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut effects = Vec::new();
    for player in tx.state().players.alive_by_seat() {
        let cards = tx.state().hand_cards(player);
        if !cards.is_empty() {
            effects.push(EffectRequest::MoveCards {
                cards,
                to: discard,
                from: tx.state().zones.find(ZoneKind::Hand, player),
            });
        }
    }
    run_effects(tx, effects).unwrap();
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    serial: u32,
    weapon_range: u8,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(532_000 + serial),
        kind,
        suit,
        rank: 7,
        weapon_range,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let revision = state.revision;
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn potnigu_collects_optional_gifts_and_buffs_exactly_each_refused_damage() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_sunchen")
            .unwrap()
            .skills,
        [POTNIGU, POTLULIAN]
    );
    let (state, owner, order) = setup(POTNIGU, 5321);
    let others: Vec<_> = order
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let mut tx = Transaction::begin(state);
    clear_hands(&mut tx);
    let cost_a = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Spade,
        1,
        0,
    );
    let cost_b = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Heart,
        2,
        0,
    );
    inject(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Club,
        3,
        8,
    );
    let gift = inject(
        &mut tx,
        others[0],
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Diamond,
        4,
        0,
    );
    inject(
        &mut tx,
        others[1],
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Club,
        5,
        0,
    );
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: skill(&state, owner, POTNIGU),
                targets: Vec::new(),
                card_targets: vec![cost_a, cost_b],
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    let first_observer = PlayerId::new(state.resolution.pending_observer);
    let first_gift = if first_observer == others[0] {
        gift
    } else {
        state.hand_cards(first_observer)[0]
    };
    let (state, output) = resolve(state, vec![first_gift.raw()], false, 2);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    let (state, output) = resolve(state, Vec::new(), true, 3);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert!(state.hand_cards(owner).contains(&first_gift));

    let before = state.players.get(others[0]).unwrap().hp;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: others[0],
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(others[0]).unwrap().hp, before - 2);
}

#[test]
fn potlulian_links_targets_draws_and_forces_fire_damage_after_hand_card_resolves() {
    let (state, owner, order) = setup(POTLULIAN, 5322);
    let others: Vec<_> = order
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let target = others[0];
    let minimum = others[1];
    let mut tx = Transaction::begin(state);
    clear_hands(&mut tx);
    tx.state_mut().players.get_mut(target).unwrap().hp = 3;
    tx.state_mut().players.get_mut(minimum).unwrap().hp = 1;
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        10,
        0,
    );
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(10),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    assert!(state
        .players
        .get(target)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| { *mark == engine_core::content::MARK_CHAIN && *count > 0 }));
    assert_eq!(state.hand_count(owner), 1);
    let options: Vec<_> = output
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert!(options.contains(&target.raw()));
    assert!(!options.contains(&minimum.raw()));
    let before = state.players.get(target).unwrap().hp;
    let (state, output) = resolve(state, vec![target.raw()], false, 11);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, before - 1);
}
}

mod original_bingshi_pot_taishici {
//! Golden coverage for BINGSHI 势太史慈「酣战」「战烈」「振锋」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const HANZHAN: u32 = 2_573_360_121;
const ZHANLIE: u32 = 3_829_642_123;
const ZHENFENG: u32 = 2_197_448_518;
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/pot_taishici.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let order = engine.state().players.alive_by_seat();
    let owner = order[0];
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        [HANZHAN, ZHANLIE, ZHENFENG].map(|skill_def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let clears: Vec<_> = order
        .iter()
        .copied()
        .filter_map(|player| {
            let cards = tx.state().hand_cards(player);
            (!cards.is_empty()).then(|| EffectRequest::MoveCards {
                cards,
                to: discard,
                from: tx.state().zones.find(ZoneKind::Hand, player),
            })
        })
        .collect();
    run_effects(&mut tx, clears).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, order)
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

fn inject_hand(tx: &mut Transaction, owner: PlayerId, serial: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(548_000 + serial),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let actor = PlayerId::new(state.resolution.pending_observer);
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
fn package_publishes_complete_pot_taishici_at_abi548() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(package.character_index.len(), 36);
    assert_eq!(package.skill_index.len(), 103);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_taishici")
            .unwrap()
            .skills,
        [HANZHAN, ZHANLIE, ZHENFENG]
    );
}

#[test]
fn hanzhan_draws_each_side_to_max_hp_cap_three_then_duels() {
    let (state, owner, order) = setup(5481);
    let target = order[1];
    let target_hp = state.players.get(target).unwrap().hp;
    let (mut state, mut output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(5482),
            kind: CommandKind::ActivateSkill {
                skill: skill(&state, owner, HANZHAN),
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert_eq!(state.hand_count(owner), 3);
    assert_eq!(state.hand_count(target), 3);
    let mut key = 5483;
    while state.has_pending_choice() {
        (state, output) = resolve(state, vec![0], false, key);
        key += 1;
    }
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "{:?}",
        output.result
    );
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
}

#[test]
fn zhanlie_spends_six_marks_for_damage_response_cost_and_post_sha_draw() {
    let (state, owner, order) = setup(5484);
    let target = order[1];
    let target_hp = state.players.get(target).unwrap().hp;
    let mut tx = Transaction::begin(state);
    let discard_card = inject_hand(&mut tx, target, 1);
    let zhanlie = skill(tx.state(), owner, ZHANLIE);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: owner,
            mark: engine_api::MarkDefId::new(ZHANLIE),
            delta: 6,
        }],
    )
    .unwrap();
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePottaishici {
                player: owner,
                target: owner,
                action: 2,
                cards: Vec::new(),
                hanzhan_def: SkillDefId::new(HANZHAN),
                zhanlie_def: SkillDefId::new(ZHANLIE),
                skill: zhanlie,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, output) = resolve(tx.commit().state, vec![target.raw()], false, 5485);
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "target: {:?}",
        output.result
    );
    let (state, output) = resolve(state, vec![2, 4, 8], false, 5486);
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "options: {:?}",
        output.result
    );
    let (mut state, mut output) = resolve(state, vec![discard_card.raw()], false, 5487);
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "cost: {:?}",
        output.result
    );
    let mut key = 5488;
    while state.has_pending_choice() {
        (state, output) = resolve(state, Vec::new(), false, key);
        key += 1;
    }
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "{:?}",
        output.result
    );
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 2);
    assert_eq!(state.hand_count(owner), 3);
    assert_eq!(state.hand_count(target), 0);
}

#[test]
fn zhenfeng_persists_independent_x_sources_for_both_skills() {
    let (state, owner, _) = setup(5490);
    let mut tx = Transaction::begin(state);
    let zhenfeng = skill(tx.state(), owner, ZHENFENG);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePottaishici {
                player: owner,
                target: owner,
                action: 4,
                cards: Vec::new(),
                hanzhan_def: SkillDefId::new(HANZHAN),
                zhanlie_def: SkillDefId::new(ZHANLIE),
                skill: zhenfeng,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, _) = resolve(tx.commit().state, vec![1], false, 5491);
    let (state, output) = resolve(state, vec![11, 23], false, 5492);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state
            .skills
            .get(skill(&state, owner, HANZHAN))
            .unwrap()
            .state_blob[0],
        1
    );
    assert_eq!(
        state
            .skills
            .get(skill(&state, owner, ZHANLIE))
            .unwrap()
            .state_blob[0],
        3
    );
}
}

mod original_bingshi_pot_weiyan {
//! Golden coverage for BINGSHI 势魏延「忠傲/壮誓/饮战/狂骨」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const POTZHONGAO: u32 = 4_200_000_559;
const POTZHUANGSHI: u32 = 4_200_000_560;
const POTYINZHAN: u32 = 4_200_000_561;
const DIRECT: u32 = 4_200_000_563;
const LIMIT: u32 = 4_200_000_564;
const KUNFEN: u32 = 3_536_332_483;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId) {
    let source = format!(
        "{}\n{}",
        include_str!("../../../content/original/ported/bingshi/pot_weiyan.skill"),
        include_str!("../../../content/original/ported/bingshi/kunfen_pot_weiyan.skill")
    );
    let package = content_compiler::compile_source(&source).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 558);
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
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        [POTZHONGAO, POTZHUANGSHI, POTYINZHAN].map(|def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner)
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> Option<engine_api::SkillId> {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .map(|skill| skill.id)
}

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: PlayerId::new(state.resolution.pending_observer),
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

fn inject_hand(tx: &mut Transaction, player: PlayerId) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(558_001),
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
fn package_publishes_complete_pot_weiyan_at_abi558() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(package.character_index.len(), 36);
    assert_eq!(package.skill_index.len(), 103);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_weiyan")
            .unwrap()
            .skills,
        [POTZHONGAO, POTZHUANGSHI, POTYINZHAN]
    );
}

#[test]
fn mission_failure_replaces_zhuangshi_with_kunfen() {
    let (state, owner) = setup(5581);
    let mut tx = Transaction::begin(state);
    let mission = skill(tx.state(), owner, POTZHONGAO).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePotweiyan {
            player: owner,
            target: owner,
            card: CardId::NONE,
            action: 3,
            skill: mission,
        }],
    )
    .unwrap();
    assert!(skill(tx.state(), owner, POTZHUANGSHI).is_none());
    assert!(skill(tx.state(), owner, KUNFEN).is_some());
}

#[test]
fn zhuangshi_discards_and_loses_hp_into_first_n_helpers() {
    let (state, owner) = setup(5582);
    let mut tx = Transaction::begin(state);
    let card = inject_hand(&mut tx, owner);
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let mission = skill(tx.state(), owner, POTZHONGAO).unwrap();
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotweiyan {
                player: owner,
                target: owner,
                card: CardId::NONE,
                action: 2,
                skill: mission,
            }]
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, output) = resolve(tx.commit().state, vec![card.raw()], false, 5583);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    let (state, output) = resolve(state, vec![2], false, 5584);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, hp_before - 2);
    assert_eq!(
        state
            .skills
            .get(skill(&state, owner, DIRECT).unwrap())
            .unwrap()
            .mark,
        1
    );
    assert_eq!(
        state
            .skills
            .get(skill(&state, owner, LIMIT).unwrap())
            .unwrap()
            .mark,
        2
    );
}
}

mod original_bingshi_pot_xiahouba {
//! Golden coverage for BINGSHI 势夏侯霸「砺兵」「破袭」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const POTLIBING: u32 = 2_358_132_416;
const POTPOXI: u32 = 4_156_018_278;
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/pot_xiahouba.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(seed: u64, skill_def: u32) -> (GameState, PlayerId, SkillId) {
    let package = content_compiler::compile_source_named("pot_xiahouba_test", SOURCE).unwrap();
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
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == skill_def)
        .unwrap()
        .id;
    (tx.commit().state, owner, skill)
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

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(522_000 + u32::from(rank)),
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

fn choose(state: GameState, selections: Vec<u32>, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let (state, output) = apply_command(
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
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

fn attack_range(state: &GameState, player: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::AttackRange { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected attack-range response: {other:?}"),
    }
}

#[test]
fn package_publishes_complete_pot_xiahouba_at_abi522() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_xiahouba")
            .unwrap()
            .skills,
        [POTLIBING, POTPOXI]
    );
}

#[test]
fn libing_cycles_range_draw_and_next_sha_damage_then_resets() {
    let (state, owner, skill) = setup(5221, POTLIBING);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    clear_hand(&mut tx, target);
    let non_damage = inject(&mut tx, owner, CardKind::Tao, Suit::Heart, 2);
    let sha = inject(&mut tx, owner, CardKind::Sha, Suit::Spade, 3);
    let base_range = attack_range(tx.state(), owner);
    for _ in 0..3 {
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotlibing {
                player: owner,
                card: non_damage,
                stage: 0,
                skill,
            }],
        )
        .unwrap();
    }
    assert_eq!(attack_range(tx.state(), owner), base_range + 1);
    let hp = tx.state().players.get(target).unwrap().hp;
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 2);
    assert_eq!(attack_range(&state, owner), base_range);
}

fn discard_sha_for_poxi(mut tx: Transaction, owner: PlayerId, sha: CardId) -> GameState {
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: owner,
                cards: vec![sha],
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    tx.commit().state
}

#[test]
fn poxi_direct_hits_targets_with_no_more_hand_cards() {
    let (state, owner, _) = setup(5222, POTPOXI);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, target);
    let sha = inject(&mut tx, owner, CardKind::Sha, Suit::Club, 4);
    let hp = tx.state().players.get(target).unwrap().hp;
    let state = discard_sha_for_poxi(tx, owner, sha);
    let state = choose(state, vec![target.raw()], 2);
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn poxi_allows_a_target_with_more_hand_cards_to_respond() {
    let (state, owner, _) = setup(5223, POTPOXI);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, target);
    let sha = inject(&mut tx, owner, CardKind::Sha, Suit::Diamond, 5);
    let shan = inject(&mut tx, target, CardKind::Shan, Suit::Heart, 6);
    let hp = tx.state().players.get(target).unwrap().hp;
    let state = discard_sha_for_poxi(tx, owner, sha);
    let state = choose(state, vec![target.raw()], 3);
    assert!(state.has_pending_choice());
    let state = choose(state, vec![shan.raw()], 4);
    assert_eq!(state.players.get(target).unwrap().hp, hp);
}
}

mod original_bingshi_pot_xinxianying {
//! Golden coverage for BINGSHI 势辛宪英「诫节/清识」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{card_use_unlimited_for_card, query, QueryContext};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse, RelationTag};

const JIEJIE: u32 = 3_788_885_537;
const QINGSHI: u32 = 2_678_891_040;
const USED: u32 = 4_200_000_554;
const SUIT_DEFS: [u32; 4] = [4_200_000_555, 4_200_000_556, 4_200_000_557, 4_200_000_558];
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/pot_xinxianying.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 554);
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let order = engine.state().players.alive_by_seat();
    let owner = order[0];
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([JIEJIE, QINGSHI].map(|def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    }));
    run_effects(&mut tx, effects).unwrap();
    clear_hands(&mut tx);
    (tx.commit().state, owner, order)
}

fn clear_hands(tx: &mut Transaction) {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let effects: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter_map(|player| {
            let cards = tx.state().hand_cards(player);
            (!cards.is_empty()).then(|| EffectRequest::MoveCards {
                cards,
                to: discard,
                from: tx.state().zones.find(ZoneKind::Hand, player),
            })
        })
        .collect();
    run_effects(tx, effects).unwrap();
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, serial: u32, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(554_000 + serial),
        kind: CardKind::Sha,
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

fn skill(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: PlayerId::new(state.resolution.pending_observer),
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
fn package_publishes_complete_pot_xinxianying_at_abi558() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(package.character_index.len(), 36);
    assert_eq!(package.skill_index.len(), 103);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_xinxianying")
            .unwrap()
            .skills,
        [JIEJIE, QINGSHI]
    );
}

#[test]
fn jiejie_discards_other_suits_and_grants_turn_suit_limit() {
    let (state, owner, order) = setup(5541);
    let actor = order[1];
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = actor;
    tx.state_mut().turn.phase = Phase::Play;
    let spade = inject_hand(&mut tx, actor, 1, Suit::Spade);
    inject_hand(&mut tx, actor, 2, Suit::Heart);
    let jiejie = skill(tx.state(), owner, JIEJIE);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotjiejie {
                player: owner,
                actor,
                used_def: SkillDefId::new(USED),
                suit_defs: SUIT_DEFS.map(SkillDefId::new),
                qingshi_def: SkillDefId::new(QINGSHI),
                skill: jiejie,
            }]
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, output) = resolve(tx.commit().state, vec![0], false, 5542);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_cards(actor), [spade]);
    assert!(card_use_unlimited_for_card(&state, actor, spade));
    assert!(state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == actor && skill.def.raw() == USED));
    assert_ne!(
        state.resolution.pending_choice_id, 0,
        "unique maximum invokes 清识"
    );
    let (_, output) = resolve(state, Vec::new(), true, 5543);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
}

#[test]
fn qingshi_resolves_ally_draw_and_enemy_discards() {
    let (state, owner, order) = setup(5544);
    let qingshi = skill(&state, owner, QINGSHI);
    let mut tx = Transaction::begin(state);
    let before = tx.state().hand_count(owner);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotqingshi {
                player: owner,
                skill: qingshi,
            }]
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, output) = resolve(tx.commit().state, vec![owner.raw()], false, 5545);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 2);

    let enemy = order
        .into_iter()
        .find(|target| {
            matches!(
                query(
                    &state,
                    &QueryRequest::Relation {
                        a: owner,
                        b: *target,
                    },
                    &mut QueryContext::new(),
                ),
                QueryResponse::Relation(RelationTag::Enemy)
            )
        })
        .unwrap();
    let mut tx = Transaction::begin(state);
    clear_hands(&mut tx);
    let owner_card = inject_hand(&mut tx, owner, 3, Suit::Club);
    let enemy_card = inject_hand(&mut tx, enemy, 4, Suit::Diamond);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePotqingshi {
            player: owner,
            skill: qingshi,
        }],
    )
    .unwrap_err();
    let (state, _) = resolve(tx.commit().state, vec![enemy.raw()], false, 5546);
    let (state, _) = resolve(state, vec![owner_card.raw()], false, 5547);
    let (state, output) = resolve(state, vec![enemy_card.raw()], false, 5548);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(owner).is_empty());
    assert!(state.hand_cards(enemy).is_empty());
}
}

mod original_bingshi_pot_yuji {
//! Golden coverage for BINGSHI 势于吉「道转/符济」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DAOZHUAN: u32 = 4_200_000_577;
const FUJI: u32 = 4_200_000_578;
const FUJI_EFFECT: u32 = 4_200_000_579;
const FUJI_SHA: u32 = 4_200_000_580;
const FUJI_SHAN: u32 = 4_200_000_581;
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/pot_yuji.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 570);
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
    let order = engine.state().players.alive_by_seat();
    let owner = order[0];
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([DAOZHUAN, FUJI].map(|def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    }));
    run_effects(&mut tx, effects).unwrap();
    clear_hands(&mut tx);
    tx.state_mut().turn.round = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, order)
}

fn clear_hands(tx: &mut Transaction) {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let effects: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter_map(|player| {
            let cards = tx.state().hand_cards(player);
            (!cards.is_empty()).then(|| EffectRequest::MoveCards {
                cards,
                to: discard,
                from: tx.state().zones.find(ZoneKind::Hand, player),
            })
        })
        .collect();
    run_effects(tx, effects).unwrap();
}

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    serial: u32,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let zone_owner = if matches!(zone_kind, ZoneKind::Hand | ZoneKind::Equip) {
        owner
    } else {
        PlayerId::NONE
    };
    let zone = tx.state().zones.find(zone_kind, zone_owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(570_000 + serial),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: zone_owner,
        face_up: zone_kind != ZoneKind::Hand,
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

fn has_skill(state: &GameState, owner: PlayerId, def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == def)
}

fn choose(state: GameState, selections: Vec<u32>, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: PlayerId::new(state.resolution.pending_observer),
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
fn package_publishes_complete_pot_yuji_at_abi570() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(package.character_index.len(), 36);
    assert_eq!(package.skill_index.len(), 103);
    assert_eq!(package.card_index.len(), 1);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_yuji")
            .unwrap()
            .skills,
        [FUJI, DAOZHUAN]
    );
}

#[test]
fn daozhuan_discards_cost_and_enforces_turn_and_round_names() {
    let (state, owner, order) = setup(5700);
    let target = order[1];
    let mut tx = Transaction::begin(state);
    let first = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        1,
        CardKind::Tao,
        Suit::Heart,
    );
    let second = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        2,
        CardKind::Shan,
        Suit::Club,
    );
    let daozhuan = skill(tx.state(), owner, DAOZHUAN);
    let hp = tx.state().players.get(target).unwrap().hp;
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(5701),
            kind: CommandKind::UseSkillCard {
                skill: daozhuan,
                card_kind: CardKind::Sha.tag(),
                targets: vec![target],
                equip_slot: 0,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    let (state, output) = choose(state, vec![first.raw()], 5702);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert!(!state.hand_cards(owner).contains(&first));

    let (mut state, repeated) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(5703),
            kind: CommandKind::UseSkillCard {
                skill: daozhuan,
                card_kind: CardKind::Jiu.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        },
    );
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
    state.skills.get_mut(daozhuan).unwrap().turn_fire_count = 0;
    let (_, repeated_name) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(5704),
            kind: CommandKind::UseSkillCard {
                skill: daozhuan,
                card_kind: CardKind::ShaFire.tag(),
                targets: vec![target],
                equip_slot: 0,
            },
        },
    );
    assert!(matches!(
        repeated_name.result,
        CommandResult::Rejected { .. }
    ));
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(5705),
            kind: CommandKind::UseSkillCard {
                skill: daozhuan,
                card_kind: CardKind::Jiu.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    let (_, output) = choose(state, vec![second.raw()], 5706);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
}

#[test]
fn fuji_pairs_cards_rewards_minimum_and_tagged_sha_gains_and_deals_extra_damage() {
    let (state, owner, order) = setup(5710);
    let first = order[1];
    let second = order[2];
    let mut tx = Transaction::begin(state);
    let first_gift = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        10,
        CardKind::Sha,
        Suit::Heart,
    );
    let second_gift = inject(
        &mut tx,
        owner,
        ZoneKind::Equip,
        11,
        CardKind::Tao,
        Suit::Club,
    );
    inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        12,
        CardKind::Tao,
        Suit::Heart,
    );
    let fuji = skill(tx.state(), owner, FUJI);
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(5711),
            kind: CommandKind::ActivateSkill {
                skill: fuji,
                targets: vec![first, second],
                card_targets: vec![first_gift, second_gift],
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(first_gift).unwrap().owner, first);
    assert_eq!(state.cards.get(second_gift).unwrap().owner, second);
    assert!(state.card_has_content_tag(first_gift, FUJI));
    assert!(state.card_has_content_tag(second_gift, FUJI));
    assert!(has_skill(&state, first, FUJI_EFFECT));
    assert!(has_skill(&state, second, FUJI_EFFECT));
    assert!(has_skill(&state, owner, FUJI_SHA));
    assert!(has_skill(&state, owner, FUJI_SHAN));
    assert_eq!(state.hand_count(owner), 2);

    let mut state = state;
    state.turn.current = first;
    state.turn.phase = Phase::Play;
    let hp = state.players.get(second).unwrap().hp;
    let before_hand = state.hand_count(first);
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: first,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(5712),
            kind: CommandKind::PlayCard {
                card: first_gift,
                targets: vec![second],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(second).unwrap().hp, hp - 2);
    assert_eq!(state.hand_count(first), before_hand);
    assert!(!state.card_has_content_tag(first_gift, FUJI));
}

#[test]
fn fuji_one_time_shan_reward_draws_then_expires() {
    let (state, owner, _) = setup(5720);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(FUJI_SHAN),
        }],
    )
    .unwrap();
    let shan = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        20,
        CardKind::Shan,
        Suit::Diamond,
    );
    inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        21,
        CardKind::Tao,
        Suit::Diamond,
    );
    let helper = skill(tx.state(), owner, FUJI_SHAN);
    let before = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::ResolvePotyuji {
                player: owner,
                target: owner,
                card: shan,
                action: 5,
                cards: vec![shan],
                targets: vec![],
                skill: helper,
            },
            EffectRequest::ResolvePotyuji {
                player: owner,
                target: owner,
                card: shan,
                action: 7,
                cards: vec![shan],
                targets: vec![],
                skill: helper,
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 2);
    assert!(!has_skill(tx.state(), owner, FUJI_SHAN));
}
}

mod original_bingshi_pot_zanghong {
//! Golden coverage for BINGSHI 势臧洪「烈志」「拒孤」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{CardKind, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const POT_LIEZHI: u32 = 3_956_687_454;
const POT_JUGU: u32 = 4_277_593_099;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(defs: &[u32]) -> (engine_core::state::GameState, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(516),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 516,
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
    effects.extend(defs.iter().map(|def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(*def),
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner)
}

fn skill(state: &engine_core::state::GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn choose(
    state: engine_core::state::GameState,
    selections: Vec<u32>,
    key: u64,
) -> engine_core::state::GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let revision = state.revision;
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, output) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_complete_pot_zanghong_at_abi517() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_zanghong")
            .unwrap()
            .skills,
        [POT_LIEZHI, POT_JUGU]
    );
}

#[test]
fn liezhi_uses_source_free_jiu_loses_max_hp_and_shares_one_turn_limit() {
    let (state, owner) = setup(&[POT_LIEZHI]);
    let liezhi = skill(&state, owner, POT_LIEZHI);
    let before = state.players.get(owner).unwrap().max_hp;
    let revision = state.revision;
    let (state, used) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::UseSkillCard {
                skill: liezhi,
                card_kind: CardKind::Jiu.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        },
    );
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().max_hp, before - 1);
    assert_eq!(state.skills.get(liezhi).unwrap().use_count, 1);
    let revision = state.revision;
    let (_, repeated) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::UseSkillCard {
                skill: liezhi,
                card_kind: CardKind::Tao.tag(),
                targets: vec![],
                equip_slot: 0,
            },
        },
    );
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
}

#[test]
fn jugu_discards_one_then_two_across_successive_turn_fires() {
    let (state, owner) = setup(&[POT_JUGU]);
    let jugu = skill(&state, owner, POT_JUGU);
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotjugu {
                player: owner,
                skill: jugu,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let first = tx.state().hand_cards(owner)[0];
    let state = choose(tx.commit().state, vec![first.raw()], 3);
    assert_eq!(state.hand_count(owner), before + 1);

    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotjugu {
                player: owner,
                skill: jugu,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let cards: Vec<_> = tx.state().hand_cards(owner)[..2]
        .iter()
        .map(|card| card.raw())
        .collect();
    let state = choose(tx.commit().state, cards, 4);
    assert_eq!(state.hand_count(owner), before + 1);
    assert_eq!(state.skills.get(jugu).unwrap().turn_fire_count, 2);
}
}

mod original_bingshi_pot_zhangren {
//! Golden coverage for BINGSHI 势张任「伏暗」「引弦」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const POTFUAN: u32 = 2_955_537_231;
const POTFUAN_RANGE: u32 = 2_955_537_232;
const POTYINXIAN: u32 = 2_595_028_433;
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/pot_zhangren.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, SkillId, SkillId) {
    let package = content_compiler::compile_source_named("pot_zhangren_test", SOURCE).unwrap();
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
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        [POTFUAN, POTYINXIAN].map(|skill_def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let find = |tx: &Transaction, def| {
        tx.state()
            .skills
            .sorted_live()
            .into_iter()
            .find(|skill| skill.owner == owner && skill.def.raw() == def)
            .unwrap()
            .id
    };
    let fuan = find(&tx, POTFUAN);
    let yinxian = find(&tx, POTYINXIAN);
    (tx.commit().state, owner, fuan, yinxian)
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

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, rank: u8) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(525_000 + u32::from(rank)),
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

fn choose(state: GameState, selections: Vec<u32>, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let (state, output) = apply_command(
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
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

fn attack_range(state: &GameState, player: PlayerId) -> i32 {
    match query(
        state,
        &QueryRequest::AttackRange { player },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected attack-range response: {other:?}"),
    }
}

#[test]
fn package_publishes_complete_pot_zhangren_at_abi525() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_zhangren")
            .unwrap()
            .skills,
        [POTFUAN, POTYINXIAN]
    );
}

#[test]
fn yinxian_boosts_farthest_sha_and_grows_range_outside_turn() {
    let (state, owner, _, yinxian) = setup(5251);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    clear_hand(&mut tx, target);
    let sha = inject(&mut tx, owner, CardKind::Sha, Suit::Spade, 1);
    let hp = tx.state().players.get(target).unwrap().hp;
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 2);
    let base = attack_range(&state, owner);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = target;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePotyinxian {
            player: owner,
            card: sha,
            stage: 1,
            skill: yinxian,
        }],
    )
    .unwrap();
    assert_eq!(attack_range(tx.state(), owner), base + 1);
}

#[test]
fn fuan_records_suit_uses_virtual_sha_and_can_silence() {
    let (state, owner, fuan, yinxian) = setup(5252);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    clear_hand(&mut tx, owner);
    clear_hand(&mut tx, target);
    let heart = inject(&mut tx, owner, CardKind::Tao, Suit::Heart, 2);
    tx.state_mut().turn.current = target;
    for _ in 0..3 {
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotyinxian {
                player: owner,
                card: heart,
                stage: 1,
                skill: yinxian,
            }],
        )
        .unwrap();
    }
    tx.state_mut().turn.current = owner;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotfuan {
                player: owner,
                card: CardId::NONE,
                stage: 0,
                range_helper_def: SkillDefId::new(POTFUAN_RANGE),
                yinxian_def: SkillDefId::new(POTYINXIAN),
                skill: fuan,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, vec![2], 2);
    assert_eq!(attack_range(&state, owner), 0);

    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = target;
    let target_heart = inject(&mut tx, target, CardKind::Shan, Suit::Heart, 3);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: target,
            cards: vec![target_heart],
        }],
    )
    .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(POTFUAN),
        }],
    )
    .unwrap();
    let hp = tx.state().players.get(target).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolvePotfuan {
                player: owner,
                card: CardId::NONE,
                stage: 2,
                range_helper_def: SkillDefId::new(POTFUAN_RANGE),
                yinxian_def: SkillDefId::new(POTYINXIAN),
                skill: fuan,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, vec![target.raw()], 3);
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    assert!(state.has_pending_choice());
    let state = choose(state, vec![2], 4);
    let target_skill = state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == target && skill.def.raw() == POTFUAN)
        .unwrap();
    assert!(!state.skill_effectively_enabled(target_skill));
}
}

mod original_bingshi_pot_zhouyu {
//! Golden coverage for BINGSHI 势周瑜「炽沄/焰洄/焚涛/雄姿」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const CHIYUN: u32 = 4_200_000_566;
const YANHUI: u32 = 4_200_000_567;
const FENTAO: u32 = 4_200_000_568;
const XIONGZI: u32 = 4_200_000_569;
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/pot_zhouyu.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 561);
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
    let order = engine.state().players.alive_by_seat();
    let owner = order[0];
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        [CHIYUN, YANHUI, FENTAO, XIONGZI].map(|def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    clear_hands(&mut tx);
    (tx.commit().state, owner, order)
}

fn clear_hands(tx: &mut Transaction) {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let effects: Vec<_> = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter_map(|player| {
            let cards = tx.state().hand_cards(player);
            (!cards.is_empty()).then(|| EffectRequest::MoveCards {
                cards,
                to: discard,
                from: tx.state().zones.find(ZoneKind::Hand, player),
            })
        })
        .collect();
    run_effects(tx, effects).unwrap();
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, serial: u32, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(561_000 + serial),
        kind: CardKind::Sha,
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

fn skill(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

fn request(
    player: PlayerId,
    target: PlayerId,
    action: u8,
    targets: Vec<PlayerId>,
    skill: engine_api::SkillId,
) -> EffectRequest {
    EffectRequest::ResolvePotzhouyu {
        player,
        target,
        card: CardId::NONE,
        action,
        card_flags: 0,
        targets,
        skill,
    }
}

fn resolve(
    state: GameState,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: PlayerId::new(state.resolution.pending_observer),
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
fn package_publishes_complete_pot_zhouyu_at_abi561() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(package.character_index.len(), 36);
    assert_eq!(package.skill_index.len(), 103);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "pot_zhouyu")
            .unwrap()
            .skills,
        [CHIYUN, YANHUI, FENTAO, XIONGZI]
    );
}

#[test]
fn chiyun_gives_cards_then_draws_two_and_links_target_once_per_phase() {
    let (state, owner, order) = setup(5611);
    let target = order[1];
    let mut tx = Transaction::begin(state);
    let card = inject_hand(&mut tx, owner, 1, Suit::Heart);
    let chiyun = skill(tx.state(), owner, CHIYUN);
    assert!(matches!(
        run_effects(&mut tx, vec![request(owner, owner, 0, Vec::new(), chiyun)]),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, _) = resolve(tx.commit().state, vec![card.raw()], false, 5612);
    let (state, _) = resolve(state, vec![target.raw()], false, 5613);
    let (state, result) = resolve(state, vec![2], false, 5614);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 2);
    assert!(state
        .players
        .get(target)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| { *mark == engine_core::content::MARK_CHAIN && *count > 0 }));

    let mut tx = Transaction::begin(state);
    inject_hand(&mut tx, owner, 2, Suit::Spade);
    run_effects(&mut tx, vec![request(owner, owner, 0, Vec::new(), chiyun)]).unwrap();
    assert_eq!(tx.state().resolution.pending_choice_id, 0);
}

#[test]
fn yanhui_phase_damage_is_limited_to_a_role_whose_card_was_discarded() {
    let (state, owner, order) = setup(5615);
    let shown = order[1];
    let other = order[2];
    let mut tx = Transaction::begin(state);
    let card = inject_hand(&mut tx, shown, 3, Suit::Club);
    let yanhui = skill(tx.state(), owner, YANHUI);

    for key in [5616, 5618] {
        assert!(matches!(
            run_effects(
                &mut tx,
                vec![request(owner, shown, 1, vec![shown, other], yanhui)]
            ),
            Err(EffectError::ChoiceRequested)
        ));
        let (state, _) = resolve(tx.commit().state, vec![shown.raw()], false, key);
        let (state, _) = resolve(state, vec![card.raw()], false, key + 1);
        tx = Transaction::begin(state);
    }
    assert!(!tx.state().hand_cards(shown).contains(&card));
    assert!(matches!(
        run_effects(&mut tx, vec![request(owner, owner, 2, Vec::new(), yanhui)]),
        Err(EffectError::ChoiceRequested)
    ));
    let pending = tx.commit().state;
    let (_, rejected) = resolve(pending.clone(), vec![other.raw()], false, 5620);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    let hp_before = pending.players.get(shown).unwrap().hp;
    let (state, accepted) = resolve(pending, vec![shown.raw()], false, 5621);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(shown).unwrap().hp, hp_before - 1);
}

#[test]
fn xiongzi_keeps_one_branch_for_all_three_skills_and_draws_two() {
    let (state, owner, _) = setup(5622);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let xiongzi = skill(tx.state(), owner, XIONGZI);
    assert!(matches!(
        run_effects(&mut tx, vec![request(owner, owner, 6, Vec::new(), xiongzi)]),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, result) = resolve(tx.commit().state, vec![1], false, 5623);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.skills.get(xiongzi).unwrap().lifetime_fire_count, 1);
    for def in [CHIYUN, YANHUI, FENTAO] {
        assert_eq!(
            state
                .skills
                .get(skill(&state, owner, def))
                .unwrap()
                .content_flags
                & 4,
            4
        );
    }
}

#[test]
fn fentao_discard_branch_relinks_after_the_fire_damage() {
    let (state, owner, order) = setup(5624);
    let target = order[1];
    let mut tx = Transaction::begin(state);
    let first = inject_hand(&mut tx, target, 4, Suit::Diamond);
    inject_hand(&mut tx, target, 5, Suit::Spade);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: target,
            mark: engine_api::MarkDefId::new(engine_core::content::MARK_CHAIN),
            delta: 1,
        }],
    )
    .unwrap();
    let fentao = skill(tx.state(), owner, FENTAO);
    assert!(matches!(
        run_effects(&mut tx, vec![request(owner, target, 3, Vec::new(), fentao)]),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, result) = resolve(tx.commit().state, vec![first.raw()], false, 5625);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let mut tx = Transaction::begin(state);
    tx.state_mut()
        .players
        .get_mut(target)
        .unwrap()
        .marks
        .retain(|(mark, _)| *mark != engine_core::content::MARK_CHAIN);
    run_effects(&mut tx, vec![request(owner, target, 4, Vec::new(), fentao)]).unwrap();
    assert!(tx
        .state()
        .players
        .get(target)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| { *mark == engine_core::content::MARK_CHAIN && *count > 0 }));
}

#[test]
fn pending_choice_rejects_after_the_originating_skill_is_removed() {
    let (state, owner, order) = setup(5626);
    let mut tx = Transaction::begin(state);
    let card = inject_hand(&mut tx, owner, 6, Suit::Heart);
    let chiyun = skill(tx.state(), owner, CHIYUN);
    assert!(matches!(
        run_effects(&mut tx, vec![request(owner, owner, 0, Vec::new(), chiyun)]),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, _) = resolve(tx.commit().state, vec![card.raw()], false, 5627);
    let mut tx = Transaction::begin(state);
    tx.state_mut().skills.remove(chiyun);
    let (_, result) = resolve(tx.commit().state, vec![order[1].raw()], false, 5628);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
}
}

mod original_bingshi_potyinhui_gain {
//! §7 技能夺取设施 S0（第一百四十三批，ABI 675）：potyinhui 音洄增益臂——
//! `choose_gain_other_skill` 两段选择链（选目标 → 选其可获得技能 → AddSkill）。

use engine_api::{
    ChoiceId, Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const POTYINHUI: u32 = 3_278_616_702;
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/potyinhui.skill");

fn started(source: &str, helper_defs: &[u32]) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("bingshi-potyinhui-test", source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_843, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    // 第二个包提供「可被夺取」的技能 def（锁定技不可获得）。
    if !helper_defs.is_empty() {
        let helper = format!(
            "skill test_helper_a(3900000001) {{ trigger phase_end owner_is_target optional {{ effect draw_cards(self, 1); }} }}\nskill test_helper_locked(3900000002) locked non_gainable {{ modifier hand_limit add 0; }}\n"
        );
        let _ = helper_defs;
        let compiled2 =
            content_compiler::compile_source_named("bingshi-potyinhui-helpers", &helper).unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled2).unwrap();
    }
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|p| *p != owner)
        .unwrap()
        .raw();
    let clear: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, clear).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(POTYINHUI),
            },
            EffectRequest::AddSkill {
                player: PlayerId::new(other),
                skill_def: SkillDefId::new(3900000001),
            },
            EffectRequest::AddSkill {
                player: PlayerId::new(other),
                skill_def: SkillDefId::new(3900000002),
            },
        ],
    )
    .unwrap();
    (tx.commit().state, owner, PlayerId::new(other))
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = state.resolution.pending_choice_id;
    let (next, _pipeline) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    next
}

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .iter()
        .any(|s| s.owner == player && s.def.raw() == def)
}

#[test]
fn potyinhui_offers_then_gains_target_skill_via_staged_choice() {
    let source = SOURCE.to_string();
    let (state, owner, other) = started(&source, &[1]);
    // 驱动回合开始触发（optional confirm）。
    let mut tx = Transaction::begin(state);
    let _ = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .map(|effects| run_effects(&mut tx, effects).unwrap_or_default());
    let asked = tx.state().has_pending_choice();
    let committed = tx.commit();
    assert!(asked, "optional potyinhui asks");
    // 段0 confirm（触发 optional）。
    let state = resolve(committed.state, owner, vec![1], 7_001);
    // 段1 选目标（options=其他存活者 raw；只有 other 一个候选）。
    let state = resolve(state, owner, vec![other.raw()], 7_002);
    // 段2 名单选技（只有 helper_a 可获得；locked non_gainable 已被过滤）。
    let state = resolve(state, owner, vec![0], 7_003);
    assert!(
        has_skill(&state, owner, 3900000001),
        "owner gained helper_a"
    );
    assert!(
        !has_skill(&state, owner, 3900000002),
        "locked non_gainable skill is not offered"
    );
}

#[test]
fn potyinhui_lose_arm_offers_and_removes_a_gained_skill() {
    let source = SOURCE.to_string();
    let (state, owner, other) = started(&source, &[1]);
    let mut tx = Transaction::begin(state);
    // 第一轮：confirm → 选目标 → 选技能 → 获得 helper_a。
    let _ = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .map(|effects| run_effects(&mut tx, effects).unwrap_or_default());
    let committed = tx.commit();
    assert!(committed.state.has_pending_choice(), "first chain asks");
    let state = resolve(committed.state, owner, vec![1], 8_101);
    let state = resolve(state, owner, vec![other.raw()], 8_102);
    let state = resolve(state, owner, vec![0], 8_103);
    assert!(has_skill(&state, owner, 3_900_000_001), "gained helper_a");
    assert!(
        state
            .facility_gained_skills
            .get(&owner.raw())
            .is_some_and(|v| v.contains(&3_900_000_001)),
        "gain registered in facility list"
    );

    // 直接驱动 S1 失去臂：清单列出 helper_a；确认后 RemoveSkill+出清登记。
    let potyinhui_instance = state
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.owner == owner && s.def.raw() == POTYINHUI)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(state);
    let _ = run_effects(
        &mut tx,
        vec![EffectRequest::LoseGainedSkill {
            player: owner,
            skill: potyinhui_instance,
        }],
    );
    assert!(tx.state().has_pending_choice(), "lose arm offers list");
    let committed = tx.commit();
    let state = resolve(committed.state, owner, vec![0], 8_104);
    assert!(
        !has_skill(&state, owner, 3_900_000_001),
        "gained skill removed"
    );
    assert!(
        state.facility_gained_skills.get(&owner.raw()).is_none(),
        "facility registration cleared"
    );
}

#[test]
fn potyinhui_once_per_round_lock_engages() {
    let source = SOURCE.to_string();
    let (state, owner, _other) = started(&source, &[1]);
    let mut tx = Transaction::begin(state);
    let _ = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .map(|effects| run_effects(&mut tx, effects).unwrap_or_default());
    let asked = tx.state().has_pending_choice();
    let _committed = tx.commit();
    assert!(asked, "first round asks");
}

// 第一百四十五批：合韵延伸（S2）——potheyun 三段链。
const POTHEYUN: u32 = 2_439_155_128;
const POTHEYUN_SOURCE: &str = include_str!("../../../content/original/ported/bingshi/potheyun.skill");

fn started_potheyun() -> (GameState, PlayerId, PlayerId) {
    let compiled =
        content_compiler::compile_source_named("bingshi-potheyun-test", POTHEYUN_SOURCE).unwrap();
    let helper = "skill shared_a(3900000011) { trigger phase_end owner_is_target optional { effect draw_cards(self, 1); } }\nskill shared_b(3900000012) { modifier hand_limit add 0; }\nskill locked_c(3900000013) locked { modifier hand_limit add 0; }\n";
    let compiled2 =
        content_compiler::compile_source_named("bingshi-potheyun-helpers", helper).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_844, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled2).unwrap();
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|p| *p != owner)
        .unwrap();
    let clear: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, clear).unwrap();
    // 双方共享 shared_a/shared_b；owner 独有 locked_c（不可共享/不可失去）。
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(POTHEYUN),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(3_900_000_011),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(3_900_000_013),
            },
            EffectRequest::AddSkill {
                player: other,
                skill_def: SkillDefId::new(3_900_000_011),
            },
            EffectRequest::AddSkill {
                player: other,
                skill_def: SkillDefId::new(3_900_000_012),
            },
        ],
    )
    .unwrap();
    (tx.commit().state, owner, other)
}

#[test]
fn potheyun_loses_shared_skill_and_target_draws_two() {
    let (state, owner, other) = started_potheyun();
    let mut tx = Transaction::begin(state);
    let other_defs_before = tx.state().hand_count(other);
    let potheyun_instance = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.owner == owner && s.def.raw() == POTHEYUN)
        .unwrap()
        .id;
    let _ = run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePotheyun {
            player: owner,
            skill: potheyun_instance,
        }],
    );
    assert!(tx.state().has_pending_choice(), "stage0 target choice");
    let committed = tx.commit();
    let state = resolve(committed.state, owner, vec![other.raw()], 9_101);
    let state = resolve(state, owner, vec![0], 9_102);
    // 共同技能只有 shared_a（locked_c 被过滤、shared_b 是对方独有）。
    assert!(
        !has_skill(&state, owner, 3_900_000_011),
        "owner lost shared_a"
    );
    assert!(
        has_skill(&state, owner, 3_900_000_013),
        "locked skill untouched"
    );
    assert_eq!(
        state.hand_count(other),
        other_defs_before + 2,
        "target drew 2"
    );
}
}

mod original_bingshi_sp_zhonghui {
//! Golden coverage for BINGSHI 势钟会「俟资」「挟志」「狱难」「克昌」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{query, sha_use_limit_for, QueryContext};
use engine_core::state::{turn_flag, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const SIZI: u32 = 2_212_391_997;
const XIEZHI: u32 = 3_797_490_396;
const YUNAN: u32 = 3_604_534_562;
const KECHANG: u32 = 2_942_773_382;
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/sp_zhonghui.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source(SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let order = engine.state().players.alive_by_seat();
    let owner = order[0];
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(
        [SIZI, XIEZHI, YUNAN, KECHANG].map(|skill_def| EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, order)
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

fn resolve(state: GameState, selection: u32, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor: PlayerId::new(state.resolution.pending_observer),
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![selection],
                cancelled: false,
            },
        },
    )
}

fn effect(
    player: PlayerId,
    target: PlayerId,
    action: u8,
    amount: u32,
    skill: engine_api::SkillId,
) -> EffectRequest {
    EffectRequest::ResolveMbzhonghui {
        player,
        target,
        action,
        amount,
        mbsizi_def: SkillDefId::new(SIZI),
        mbkechang_def: SkillDefId::new(KECHANG),
        skill,
    }
}

#[test]
fn package_publishes_complete_sp_zhonghui_at_abi553() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(package.character_index.len(), 36);
    assert_eq!(package.skill_index.len(), 103);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "sp_zhonghui")
            .unwrap()
            .skills,
        [SIZI, XIEZHI, YUNAN, KECHANG]
    );
}

#[test]
fn sizi_spends_charge_and_settles_sha_users() {
    let (state, owner, order) = setup(5531);
    let user = order[1];
    let sizi = skill(&state, owner, SIZI);
    let mut tx = Transaction::begin(state);
    run_effects(&mut tx, vec![effect(owner, owner, 1, 0, sizi)]).unwrap();
    assert!(matches!(
        run_effects(&mut tx, vec![effect(owner, owner, 0, 0, sizi)]),
        Err(EffectError::ChoiceRequested)
    ));
    let (state, output) = resolve(tx.commit().state, 2, 5532);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    let mut tx = Transaction::begin(state);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::sha_used_by_player(user), 1);
    let hp = tx.state().players.get(user).unwrap().hp;
    let hand = tx.state().hand_count(owner);
    run_effects(&mut tx, vec![effect(owner, owner, 2, 0, sizi)]).unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 2);
    assert_eq!(tx.state().players.get(user).unwrap().hp, hp - 1);
    assert_eq!(tx.state().skills.get(sizi).unwrap().state_blob[0], 1);
}

#[test]
fn xiezhi_overflow_grows_both_limits_and_yunan_upgrades_kechang() {
    let (state, owner, order) = setup(5533);
    let mut tx = Transaction::begin(state);
    let sizi = skill(tx.state(), owner, SIZI);
    let xiezhi = skill(tx.state(), owner, XIEZHI);
    let yunan = skill(tx.state(), owner, YUNAN);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: owner,
            mark: engine_api::MarkDefId::new(SIZI),
            delta: 3,
        }],
    )
    .unwrap();
    let base_sha = sha_use_limit_for(tx.state(), owner);
    let base_hand = match query(
        tx.state(),
        &QueryRequest::HandLimit { player: owner },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Int(value) => value,
        other => panic!("unexpected {other:?}"),
    };
    run_effects(&mut tx, vec![effect(owner, owner, 4, 3, xiezhi)]).unwrap();
    assert_eq!(sha_use_limit_for(tx.state(), owner), base_sha + 1);
    assert_eq!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: owner },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(base_hand + 1)
    );
    assert_eq!(
        tx.state()
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .find(|(mark, _)| *mark == SIZI)
            .unwrap()
            .1,
        4
    );

    run_effects(
        &mut tx,
        vec![
            effect(owner, order[1], 5, 0, yunan),
            effect(owner, order[2], 6, 0, yunan),
        ],
    )
    .unwrap();
    assert_eq!(tx.state().player_group(owner), Some("qun"));
    assert_eq!(
        tx.state()
            .skills
            .get(skill(tx.state(), owner, KECHANG))
            .unwrap()
            .state_blob,
        [1]
    );
    assert_eq!(tx.state().skills.get(sizi).unwrap().def.raw(), SIZI);
}
}

mod original_bingshi_sunjun {
//! Golden coverage for BINGSHI 势孙峻「凶图」「险帅」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, schedule_damage, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const MBXIONGTU: u32 = 2_558_604_057;
const MBXIANSHUAI: u32 = 3_336_756_615;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(skill_def: u32) -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(u64::from(skill_def)),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: u64::from(skill_def),
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let target = engine
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
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, target)
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

fn inject(tx: &mut Transaction, owner: PlayerId, suit: Suit) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(507_001 + suit as u32),
        kind: CardKind::Sha,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(state: GameState, selections: Vec<u32>, cancelled: bool, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let (state, output) = apply_command(
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
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn xianshuai_only_unlimits_the_first_hand_card_of_each_suit() {
    let (mut tx, owner, _) = setup(MBXIANSHUAI);
    let first = inject(&mut tx, owner, Suit::Spade);
    let second = inject(&mut tx, owner, Suit::Spade);
    assert!(engine_core::query::card_use_unlimited_for_card(
        tx.state(),
        owner,
        first
    ));
    tx.state_mut()
        .turn
        .remember_hand_card_suit_used(owner, Suit::Spade);
    assert!(!engine_core::query::card_use_unlimited_for_card(
        tx.state(),
        owner,
        second
    ));
}

#[test]
fn xiongtu_damage_arms_only_after_itself_then_external_damage_enables_second_use() {
    let (mut tx, owner, target) = setup(MBXIONGTU);
    let skill = skill(tx.state(), owner, MBXIONGTU);
    tx.state_mut()
        .skills
        .get_mut(skill)
        .unwrap()
        .play_phase_use_count = 1;
    tx.state_mut()
        .turn
        .set_flag(turn_flag::DISCARD_PILE_SUIT_MASK, 0b1111);
    let revealed = inject(&mut tx, target, Suit::Heart);
    let hp = tx.state().players.get(target).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbxiongtu {
                player: owner,
                target,
                stage: 0,
                card: CardId::NONE,
                cards: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, vec![revealed.raw()], false, 1);
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
    let flags = u32::from_le_bytes(
        state.skills.get(skill).unwrap().state_blob[4..8]
            .try_into()
            .unwrap(),
    );
    assert_eq!(flags, 0b001);

    let hand_before = state.hand_cards(owner).len();
    let mut tx = Transaction::begin(state);
    schedule_damage(&mut tx, owner, target, 1).unwrap();
    let flags = u32::from_le_bytes(
        tx.state().skills.get(skill).unwrap().state_blob[4..8]
            .try_into()
            .unwrap(),
    );
    assert_eq!(flags, 0b010);
    assert_eq!(tx.state().hand_cards(owner).len(), hand_before + 1);

    tx.state_mut()
        .skills
        .get_mut(skill)
        .unwrap()
        .play_phase_use_count = 2;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbxiongtu {
                player: owner,
                target,
                stage: 0,
                card: CardId::NONE,
                cards: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
}

#[test]
fn xiongtu_cancelled_cost_discards_the_revealed_hand_card_and_arms() {
    let (mut tx, owner, target) = setup(MBXIONGTU);
    let skill = skill(tx.state(), owner, MBXIONGTU);
    tx.state_mut()
        .skills
        .get_mut(skill)
        .unwrap()
        .play_phase_use_count = 1;
    let revealed = inject(&mut tx, target, Suit::Diamond);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbxiongtu {
                player: owner,
                target,
                stage: 0,
                card: CardId::NONE,
                cards: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, vec![revealed.raw()], false, 2);
    let state = resolve(state, Vec::new(), true, 3);
    assert!(!state.hand_cards(target).contains(&revealed));
    assert_eq!(&state.skills.get(skill).unwrap().state_blob[..4], b"XTG1");
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbxiongtu {
            player: owner,
            target,
            stage: 6,
            card: CardId::NONE,
            cards: Vec::new(),
            skill,
        }],
    )
    .unwrap();
    assert_eq!(
        u32::from_le_bytes(
            tx.state().skills.get(skill).unwrap().state_blob[4..8]
                .try_into()
                .unwrap()
        ),
        0b001
    );
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbxiongtu {
            player: owner,
            target,
            stage: 7,
            card: CardId::NONE,
            cards: Vec::new(),
            skill,
        }],
    )
    .unwrap();
    assert!(tx.state().skills.get(skill).unwrap().state_blob.is_empty());
}
}

mod original_bingshi_sunshao {
//! Golden coverage for BINGSHI 孙韶「敢决」「筑墼」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const MBGANJUE: u32 = 3_789_971_232;
const MBZHUJI: u32 = 2_468_204_018;
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/sunshao.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(seed: u64, players: u8, skill_def: u32) -> (GameState, PlayerId, SkillId) {
    let package = content_compiler::compile_source_named("sunshao_test", SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: seed,
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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == skill_def)
        .unwrap()
        .id;
    (tx.commit().state, owner, skill)
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
        def: CardDefId::new(521_000 + u32::from(rank)),
        kind,
        suit,
        rank,
        weapon_range: u8::from(kind == CardKind::Weapon) + 1,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, selections: Vec<u32>, cancelled: bool, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let revision = state.revision;
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, output) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_sunshao_at_abi521() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "sunshao")
            .unwrap()
            .skills,
        [MBGANJUE, MBZHUJI]
    );
    let view = package
        .view_as_index
        .iter()
        .find(|view| view.skill_def_id == MBGANJUE)
        .unwrap();
    assert!(view.ignore_distance && view.no_use_count);
    assert!(view.target_lacks_source_suit_unrespondable);
}

#[test]
fn ganjue_ignores_distance_and_sha_count_but_same_suit_hand_can_respond() {
    let (state, owner, skill) = setup(5211, 4, MBGANJUE);
    let seats = state.players.alive_by_seat();
    let owner_index = seats.iter().position(|player| *player == owner).unwrap();
    let far = seats[(owner_index + 2) % seats.len()];
    let mut tx = Transaction::begin(state);
    clear_hand(&mut tx, far);
    let source = inject(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Weapon,
        Suit::Heart,
        7,
    );
    let shan = inject(&mut tx, far, ZoneKind::Hand, CardKind::Shan, Suit::Heart, 8);
    tx.state_mut().turn.set_flag(turn_flag::SHA_PLAYED, 1);
    let hp = tx.state().players.get(far).unwrap().hp;
    let (state, output) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![far],
                card_targets: vec![source],
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    let state = choose(state, vec![shan.raw()], false, 2);
    assert_eq!(state.players.get(far).unwrap().hp, hp);
    assert_eq!(state.turn.flag(turn_flag::SHA_PLAYED), 1);
}

#[test]
fn ganjue_target_without_matching_suit_cannot_respond() {
    let (state, owner, skill) = setup(5212, 2, MBGANJUE);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    clear_hand(&mut tx, target);
    let source = inject(
        &mut tx,
        owner,
        ZoneKind::Equip,
        CardKind::Armor,
        Suit::Spade,
        9,
    );
    inject(
        &mut tx,
        target,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Club,
        10,
    );
    let hp = tx.state().players.get(target).unwrap().hp;
    let (state, output) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![source],
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn zhuji_discards_exact_suit_uses_shared_equip_and_hujia_absorbs_damage() {
    let (state, owner, skill) = setup(5213, 2, MBZHUJI);
    let source = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(state);
    clear_hand(&mut tx, owner);
    let heart_a = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Heart,
        3,
    );
    let heart_b = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Tao,
        Suit::Heart,
        4,
    );
    let club = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Club,
        5,
    );
    let weapon = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Weapon,
        Suit::Heart,
        6,
    );
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbzhuji {
                player: owner,
                choice: 0,
                stage: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, vec![2], false, 4);
    assert!(state.has_pending_choice());
    assert!(!state.hand_cards(owner).contains(&heart_a));
    assert!(!state.hand_cards(owner).contains(&heart_b));
    assert!(state.hand_cards(owner).contains(&club));
    let equip = state.zones.find(ZoneKind::Equip, owner).unwrap();
    assert!(state.zones.get(equip).unwrap().cards.contains(&weapon));
    let state = choose(state, vec![3], false, 5);
    let hp = state.players.get(owner).unwrap().hp;
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
    let state = tx.commit().state;
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    assert!(!state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_HUJIA && *count > 0));
}
}

mod original_bingshi_tianfeng {
//! Golden coverage for BINGSHI 势田丰「刚鲠」「死谏」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const MBGANGGENG: u32 = 2_184_967_024;
const MBSIJIAN: u32 = 2_168_498_663;
const MBSIJIAN_DISCARD: u32 = 2_654_683_932;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

#[test]
fn package_publishes_complete_tianfeng_at_abi509() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    let character = package
        .character_index
        .iter()
        .find(|character| character.name_key == "mb_tianfeng")
        .unwrap();
    assert_eq!(character.skills, [MBGANGGENG, MBSIJIAN]);
}

fn setup(skill_def: u32) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(u64::from(skill_def)),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: u64::from(skill_def),
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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, others[0], others[1])
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

fn inject(tx: &mut Transaction, owner: PlayerId, serial: u32) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(508_000 + serial),
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

fn resolve(state: GameState, selections: Vec<u32>, cancelled: bool, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let (state, output) = apply_command(
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
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn ganggeng_gives_selected_cards_and_draws_when_recipient_has_max_hand() {
    let (mut tx, owner, target, _) = setup(MBGANGGENG);
    let skill = skill(tx.state(), owner, MBGANGGENG);
    let cards = vec![inject(&mut tx, owner, 1), inject(&mut tx, owner, 2)];
    let before = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbganggeng {
            player: owner,
            target,
            stage: 0,
            cards: cards.clone(),
            skill,
        }],
    )
    .unwrap();
    assert!(cards
        .iter()
        .all(|card| tx.state().hand_cards(target).contains(card)));
    assert_eq!(
        &tx.state().skills.get(skill).unwrap().state_blob[..4],
        b"GGG1"
    );
    let hand_after_gift = tx.state().hand_count(owner);
    assert_eq!(hand_after_gift, before - 2);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbganggeng {
            player: owner,
            target: owner,
            stage: 1,
            cards: Vec::new(),
            skill,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand_after_gift + 1);
    assert!(tx.state().skills.get(skill).unwrap().state_blob.is_empty());
}

#[test]
fn ganggeng_active_command_forwards_selected_hand_cards() {
    let (mut tx, owner, target, _) = setup(MBGANGGENG);
    let skill = skill(tx.state(), owner, MBGANGGENG);
    let cards = vec![inject(&mut tx, owner, 40), inject(&mut tx, owner, 41)];
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(40),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: cards.clone(),
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert!(cards
        .iter()
        .all(|card| state.hand_cards(target).contains(card)));
}

#[test]
fn ganggeng_discards_from_recipient_when_another_player_has_more_hand() {
    let (mut tx, owner, target, leader) = setup(MBGANGGENG);
    let skill = skill(tx.state(), owner, MBGANGGENG);
    let cards = vec![inject(&mut tx, owner, 3), inject(&mut tx, owner, 4)];
    for serial in 10..20 {
        inject(&mut tx, leader, serial);
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbganggeng {
            player: owner,
            target,
            stage: 0,
            cards,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbganggeng {
                player: owner,
                target: owner,
                stage: 1,
                cards: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let selected = tx.state().hand_cards(target)[0];
    let state = resolve(tx.commit().state, vec![selected.raw()], false, 1);
    assert!(!state.hand_cards(target).contains(&selected));
}

#[test]
fn sijian_both_is_cumulative_and_grants_one_shot_discard() {
    let (mut tx, owner, target, _) = setup(MBSIJIAN);
    let skill = skill(tx.state(), owner, MBSIJIAN);
    let hp = tx.state().players.get(owner).unwrap().hp;
    let hand = tx.state().hand_count(owner);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbsijian {
                player: owner,
                target: PlayerId::NONE,
                stage: 0,
                choice: 0,
                helper_def: SkillDefId::new(MBSIJIAN_DISCARD),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, vec![3], false, 2);
    let state = resolve(state, vec![target.raw()], false, 3);
    assert_eq!(state.players.get(owner).unwrap().hp, hp);
    assert_eq!(state.hand_count(owner), hand + 2);
    assert!(state
        .skills
        .sorted_live()
        .into_iter()
        .any(|instance| { instance.owner == target && instance.def.raw() == MBSIJIAN_DISCARD }));

    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbsijian {
                player: owner,
                target: PlayerId::NONE,
                stage: 0,
                choice: 0,
                helper_def: SkillDefId::new(MBSIJIAN_DISCARD),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, vec![3], false, 4);
    let state = resolve(state, vec![target.raw()], false, 5);
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);

    let mut tx = Transaction::begin(state);
    let used = inject(&mut tx, target, 30);
    let discardable = inject(&mut tx, target, 31);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: target,
                card: used,
                targets: Vec::new(),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, vec![discardable.raw()], false, 6);
    assert!(!state
        .skills
        .sorted_live()
        .into_iter()
        .any(|instance| { instance.owner == target && instance.def.raw() == MBSIJIAN_DISCARD }));
    assert!(!state.hand_cards(target).contains(&discardable));
}
}

mod original_bingshi_yanghong {
//! Golden coverage for BINGSHI 势杨弘「间计」「远谟」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const MBJIANJI: u32 = 3_226_006_705;
const MBYUANMO: u32 = 4_112_615_395;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup(skill_def: u32, player_count: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(u64::from(skill_def)),
            kind: CommandKind::StartGame {
                player_count,
                rng_seed: u64::from(skill_def),
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, others)
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

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
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

fn inject(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    rank: u8,
    serial: u32,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(509_000 + serial),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: if kind == CardKind::Weapon { 3 } else { 0 },
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(state: GameState, selections: Vec<u32>, cancelled: bool, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
    let (state, output) = apply_command(
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
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_complete_yanghong_at_abi509() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    let character = package
        .character_index
        .iter()
        .find(|character| character.name_key == "mb_yanghong")
        .unwrap();
    assert_eq!(character.skills, [MBJIANJI, MBYUANMO]);
}

#[test]
fn jianji_shared_sha_compares_once_then_damages_its_chooser() {
    let (mut tx, owner, others) = setup(MBJIANJI, 3);
    let first = others[0];
    let second = others[1];
    for player in [owner, first, second] {
        clear_hand(&mut tx, player);
    }
    let shared = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 8, 1);
    let losing = inject(&mut tx, second, ZoneKind::Hand, CardKind::Shan, 5, 2);
    let first_hp = tx.state().players.get(first).unwrap().hp;
    let second_hp = tx.state().players.get(second).unwrap().hp;
    let skill = skill(tx.state(), owner, MBJIANJI);
    let state = tx.commit().state;
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![first, second],
                card_targets: vec![shared],
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    let state = resolve(state, vec![shared.raw()], false, 2);
    let state = resolve(state, vec![losing.raw()], false, 3);
    assert_eq!(state.players.get(first).unwrap().hp, first_hp - 1);
    assert_eq!(state.players.get(second).unwrap().hp, second_hp - 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(shared).unwrap().zone, discard);
}

#[test]
fn jianji_both_participants_can_share_the_same_physical_card() {
    let (mut tx, owner, others) = setup(MBJIANJI, 3);
    let first = others[0];
    let second = others[1];
    for player in [owner, first, second] {
        clear_hand(&mut tx, player);
    }
    let shared = inject(&mut tx, owner, ZoneKind::Hand, CardKind::Sha, 9, 20);
    let first_hp = tx.state().players.get(first).unwrap().hp;
    let second_hp = tx.state().players.get(second).unwrap().hp;
    let skill = skill(tx.state(), owner, MBJIANJI);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbjianji {
                player: owner,
                source: first,
                target: second,
                stage: 0,
                card: shared,
                cards: Vec::new(),
                targets: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, vec![shared.raw()], false, 20);
    let state = resolve(state, vec![shared.raw()], false, 21);
    assert_eq!(state.players.get(first).unwrap().hp, first_hp - 1);
    assert_eq!(state.players.get(second).unwrap().hp, second_hp - 1);
}

#[test]
fn yuanmo_draws_for_attack_range_population_lost_by_moving_weapon() {
    let (mut tx, owner, others) = setup(MBYUANMO, 4);
    let source = others[0];
    let destination = others[1];
    let weapon = inject(&mut tx, source, ZoneKind::Equip, CardKind::Weapon, 7, 3);
    let before = tx.state().hand_count(source);
    let skill = skill(tx.state(), owner, MBYUANMO);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbyuanmo {
                player: owner,
                target: PlayerId::NONE,
                stage: 0,
                card: CardId::NONE,
                value: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = resolve(tx.commit().state, vec![weapon.raw()], false, 3);
    let state = resolve(state, vec![destination.raw()], false, 4);
    assert!(state.resolution.pending_choice_id != 0);
    let state = resolve(state, vec![1], false, 5);
    assert_eq!(state.hand_count(source), before + 1);
    let equip = state.zones.find(ZoneKind::Equip, destination).unwrap();
    assert_eq!(state.cards.get(weapon).unwrap().zone, equip);
}
}

mod original_bingshi_zhangyan {
//! Golden coverage for BINGSHI 势张燕「飞径」「骁戈」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const MBFEIJING: u32 = 2_916_762_074;
const MBXIAOGE: u32 = 3_431_894_905;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/bingshi/bingshi.npack.json");

fn setup() -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(510),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 510,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let order = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([MBFEIJING, MBXIAOGE].map(|def| EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, order)
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

fn inject(
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
        def: CardDefId::new(510_000 + serial),
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

fn choose(state: GameState, selections: Vec<u32>, key: u64) -> GameState {
    let actor = PlayerId::new(state.resolution.pending_observer);
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
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_zhangyan_and_damage_trick_view_as_at_abi510() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 677);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.name_key == "mb_zhangyan")
            .unwrap()
            .skills,
        [MBFEIJING, MBXIAOGE]
    );
    assert_eq!(
        package
            .view_as_index
            .iter()
            .filter(|view| view.skill_def_id == MBFEIJING && view.filter == "damage_trick")
            .count(),
        2
    );
}

#[test]
fn feijing_discards_on_chosen_path_and_adds_matching_color_target() {
    let (mut tx, owner, order) = setup();
    for player in &order {
        clear_hand(&mut tx, *player);
    }
    let owner_index = order.iter().position(|player| *player == owner).unwrap();
    let middle = order[(owner_index + 1) % 4];
    let target = order[(owner_index + 2) % 4];
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        1,
    );
    let red = inject(
        &mut tx,
        middle,
        ZoneKind::Hand,
        CardKind::Shan,
        Suit::Heart,
        2,
    );
    let middle_hp = tx.state().players.get(middle).unwrap().hp;
    tx.state_mut().resolution.resume_kind = ContinuationKind::ResumePlayedSha;
    tx.state_mut().resolution.resume_payload = vec![owner.raw(), target.raw(), 1, 0, 1, sha.raw()];
    let feijing = skill(tx.state(), owner, MBFEIJING);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMbfeijing {
                player: owner,
                target,
                card: sha,
                stage: 0,
                targets: Vec::new(),
                cards: Vec::new(),
                skill: feijing,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, vec![1], 1);
    let state = choose(state, vec![red.raw()], 2);
    let state = choose(state, vec![1], 3);
    let owner_hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    assert_eq!(state.cards.get(red).unwrap().zone, owner_hand);
    assert_eq!(state.players.get(middle).unwrap().hp, middle_hp);
}

#[test]
fn xiaoge_prevents_bound_damage_recovers_and_reclaims_discard() {
    let (mut tx, owner, order) = setup();
    let target = order.into_iter().find(|player| *player != owner).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        3,
    );
    let selected = inject(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        CardKind::Shan,
        Suit::Heart,
        4,
    );
    let feijing = skill(tx.state(), owner, MBFEIJING);
    let xiaoge = skill(tx.state(), owner, MBXIAOGE);
    let mut blob = b"FJG1".to_vec();
    blob.extend_from_slice(&sha.raw().to_le_bytes());
    blob.extend_from_slice(&1u32.to_le_bytes());
    blob.extend_from_slice(&target.raw().to_le_bytes());
    blob.extend_from_slice(&selected.raw().to_le_bytes());
    tx.state_mut().skills.get_mut(feijing).unwrap().state_blob = blob;
    tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;
    let hp = tx.state().players.get(owner).unwrap().hp;
    tx.state_mut().resolution.current_damage_id = 77;
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbxiaoge {
            player: owner,
            target,
            card: sha,
            stage: 0,
            targets: Vec::new(),
            skill: xiaoge,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp + 1);
    assert!(tx.state().resolution.prevented_damage_ids.contains(&77));
    assert_ne!(tx.state().cards.get(selected).unwrap().zone, discard);
    assert_eq!(tx.state().skills.get(feijing).unwrap().state_blob.len(), 12);
}

#[test]
fn xiaoge_follows_the_original_single_target_sha_with_duel() {
    let (mut tx, owner, order) = setup();
    let target = order
        .iter()
        .copied()
        .find(|player| *player != owner)
        .unwrap();
    for player in order {
        clear_hand(&mut tx, player);
    }
    let sha = inject(
        &mut tx,
        owner,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Spade,
        5,
    );
    let hp = tx.state().players.get(target).unwrap().hp;
    let xiaoge = skill(tx.state(), owner, MBXIAOGE);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveMbxiaoge {
            player: owner,
            target: owner,
            card: sha,
            stage: 1,
            targets: vec![target],
            skill: xiaoge,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(target).unwrap().hp, hp - 1);
}
}
