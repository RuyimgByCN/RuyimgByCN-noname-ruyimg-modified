//! R03：冯河/英霸的体力、上限、标记与技能授予；同时验证 `has_skill` 查询。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const YINGBA: u32 = 2_187_983_153;
const PINGHE: u32 = 2_176_317_369;
const MARK: u32 = 3_219_705_664;

const YINGBA_SRC: &str = include_str!("../../../../content/original/ported/shiji/yingba.skill");
const PINGHE_SRC: &str = include_str!("../../../../content/original/ported/shiji/pinghe.skill");

fn started() -> (GameState, PlayerId, PlayerId) {
    let source = format!("{YINGBA_SRC}\n{PINGHE_SRC}");
    let compiled = content_compiler::compile_source_named("shiji-r03", &source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_261_026, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
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
    for def in [YINGBA, PINGHE] {
        run_effects(
            &mut tx,
            vec![EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(def),
            }],
        )
        .unwrap();
    }
    for player in [owner, other] {
        let p = tx.state_mut().players.get_mut(player).unwrap();
        p.hp = 4;
        p.max_hp = 4;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx.commit().state, owner, other)
}

fn inject(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind,
        suit: engine_api::Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn inject_def(tx: &mut Transaction, player: PlayerId, def: u32, kind: CardKind) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(def),
        kind,
        suit: engine_api::Suit::Spade,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.owner == owner && s.def.raw() == def)
        .unwrap()
        .id
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    revision: u64,
) -> (GameState, engine_api::CommandResult) {
    let (next, pipe) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind,
        },
    );
    (next, pipe.result)
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, revision: u64) -> GameState {
    let choice_id = next_choice_id(&state);
    let (next, result) = submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(choice_id),
            selections,
            cancelled: false,
        },
        revision,
    );
    assert!(
        matches!(result, engine_api::CommandResult::Accepted { .. }),
        "{result:?}"
    );
    next
}

fn next_choice_id(state: &GameState) -> u32 {
    state.resolution.pending_choice_id
}

#[test]
fn yingba_loses_hp_then_max_hp_and_grants_mark_to_living_target() {
    let (state, owner, other) = started();
    let revision = state.revision;
    let skill = skill_id(&state, owner, YINGBA);
    let (state, result) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![other],
            card_targets: vec![],
        },
        revision,
    );
    assert!(
        matches!(result, engine_api::CommandResult::Accepted { .. }),
        "{result:?}"
    );
    let p = state.players.get(other).unwrap();
    assert_eq!((p.hp, p.max_hp), (3, 3), "目标先失去1体力，再失去1体力上限");
    assert!(p
        .marks
        .iter()
        .any(|(mark, count)| *mark == MARK && *count == 1));
    assert!(state
        .skills
        .sorted_live()
        .iter()
        .any(|s| s.owner == other && s.def.raw() == MARK));
    assert_eq!(state.players.get(owner).unwrap().max_hp, 3);
    let limit = engine_core::query::query(
        &state,
        &QueryRequest::HandLimit { player: other },
        &mut engine_core::query::QueryContext::new(),
    );
    assert_eq!(limit, QueryResponse::Int(3), "平定标记不再错误增加手牌上限");
}

#[test]
fn pinghe_branch_and_mark_only_with_yingba_owned() {
    for keep_yingba in [true, false] {
        let (state, owner, other) = started();
        let mut tx = Transaction::begin(state);
        if !keep_yingba {
            let skill = skill_id(tx.state(), owner, YINGBA);
            run_effects(&mut tx, vec![EffectRequest::RemoveSkill { skill }]).unwrap();
        }
        let card = inject(&mut tx, owner, CardKind::Sha);
        let max_before = tx.state().players.get(owner).unwrap().max_hp;
        let err = run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: other,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        )
        .unwrap_err();
        assert_eq!(err, EffectError::ChoiceRequested);
        let state = tx.commit().state;
        assert!(state.has_pending_choice());
        let owner_state = state.players.get(owner).unwrap();
        assert_eq!(
            (owner_state.hp, owner_state.max_hp),
            (3, 3),
            "上限-1后体力被夹到新上限；伤害尚未提交"
        );
        assert!(
            state
                .resolution
                .prevented_damage_ids
                .contains(&state.resolution.current_damage_id),
            "冯河已在选择前登记防止本次伤害"
        );
        let revision = state.revision;
        let state = resolve(state, owner, vec![card.raw()], revision);
        assert!(state.has_pending_choice(), "选牌后应继续选目标或重铸");
        let revision = state.revision;
        let state = resolve(state, owner, vec![other.raw()], revision);
        assert!(!state.has_pending_choice());
        let p = state.players.get(owner).unwrap();
        // 本测试经直接 DealDamage 触发；该路径的防伤 + 嵌套选择提交面未在
        // 此断言（见证据边界），这里只覆盖上限/标记/交牌与 has_skill 分支。
        assert_eq!(p.max_hp, max_before - 1, "冯河失去1体力上限");
        let source = state.players.get(other).unwrap();
        let joined = source
            .marks
            .iter()
            .any(|(mark, count)| *mark == MARK && *count == 1);
        assert_eq!(joined, keep_yingba, "仅拥有英霸时来源获平定标记");
        assert_eq!(
            state
                .skills
                .sorted_live()
                .iter()
                .any(|s| s.owner == other && s.def.raw() == MARK),
            keep_yingba
        );
    }
}

#[test]
fn pinghe_recast_branch_consumes_and_redraws_without_giving() {
    let (state, owner, other) = started();
    let mut tx = Transaction::begin(state);
    let card = inject_def(
        &mut tx,
        owner,
        engine_core::content::card_def_id_for_name("tiesuo"),
        CardKind::TrickTiesuo,
    );
    let hand_before = tx.state().hand_count(owner);
    let target_hand_before = tx.state().hand_count(other);
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap_err();
    assert_eq!(err, EffectError::ChoiceRequested);
    let state = tx.commit().state;
    let revision = state.revision;
    let state = resolve(state, owner, vec![card.raw()], revision);
    assert!(state.has_pending_choice());
    let revision = state.revision;
    let state = resolve(state, owner, vec![0], revision);
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), hand_before, "重铸：弃一张再摸一张");
    assert_eq!(state.hand_count(other), target_hand_before, "未交给目标");
    assert_eq!(state.players.get(owner).unwrap().max_hp, 3);
    assert!(
        !state
            .players
            .get(other)
            .unwrap()
            .marks
            .iter()
            .any(|(mark, _)| *mark == MARK),
        "重铸不应授予平定标记"
    );
}

#[test]
fn pinghe_hand_limit_base_uses_max_hp() {
    let (state, owner, _) = started();
    let mut tx = Transaction::begin(state);
    {
        let p = tx.state_mut().players.get_mut(owner).unwrap();
        p.hp = 2;
        p.max_hp = 4;
    }
    let limit = engine_core::query::query(
        tx.state(),
        &QueryRequest::HandLimit { player: owner },
        &mut engine_core::query::QueryContext::new(),
    );
    assert_eq!(limit, QueryResponse::Int(4), "冯河手牌上限基数=最大体力");
}

#[test]
fn yingba_rejects_target_with_max_hp_one() {
    let (state, owner, other) = started();
    let mut tx = Transaction::begin(state);
    {
        let p = tx.state_mut().players.get_mut(other).unwrap();
        p.max_hp = 1;
        p.hp = 1;
    }
    let state = tx.commit().state;
    let revision = state.revision;
    let skill = skill_id(&state, owner, YINGBA);
    let (_state, result) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![other],
            card_targets: vec![],
        },
        revision,
    );
    assert!(
        matches!(result, engine_api::CommandResult::Rejected { .. }),
        "英霸不能选体力上限 1 的目标：{result:?}"
    );
}

#[test]
fn yingba_mark_blocks_early_cards_until_user_used_marks_cards() {
    let (state, owner, other) = started();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(MARK),
            },
            EffectRequest::AddMark {
                player: owner,
                mark: engine_api::MarkDefId::new(MARK),
                delta: 1,
            },
        ],
    )
    .unwrap();
    let card = inject(&mut tx, other, CardKind::Sha);
    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let revision = state.revision;
    let (state, blocked) = submit(
        state,
        other,
        CommandKind::PlayCard {
            card,
            targets: vec![owner],
            card_targets: vec![],
        },
        revision,
    );
    assert!(
        matches!(blocked, engine_api::CommandResult::Rejected { .. }),
        "使用第 1 张牌时不能指定有 1 枚平定的英霸角色：{blocked:?}"
    );

    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.record_turn_card_use(other);
    let state = tx.commit().state;
    let revision = state.revision;
    let (_state, allowed) = submit(
        state,
        other,
        CommandKind::PlayCard {
            card,
            targets: vec![owner],
            card_targets: vec![],
        },
        revision,
    );
    assert!(
        matches!(allowed, engine_api::CommandResult::Accepted { .. }),
        "已使用 1 张牌后应可指定：{allowed:?}"
    );
}

#[test]
fn pinghe_rejects_when_chosen_card_leaves_hand_before_target_step() {
    let (state, owner, other) = started();
    let mut tx = Transaction::begin(state);
    let card = inject(&mut tx, owner, CardKind::Sha);
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap_err();
    assert_eq!(err, EffectError::ChoiceRequested);
    let state = tx.commit().state;
    let revision = state.revision;
    let state = resolve(state, owner, vec![card.raw()], revision);
    assert!(state.has_pending_choice());

    // 第二次选择前把牌移出手牌，解析必须拒绝且不改状态。
    let mut tx = Transaction::begin(state);
    let hand = tx
        .state()
        .zones
        .find(engine_core::state::ZoneKind::Hand, owner)
        .unwrap();
    tx.state_mut()
        .zones
        .get_mut(hand)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    let state = tx.commit().state;
    let before = engine_core::canonical_state_hash(&state);
    let revision = state.revision;
    let choice_id = next_choice_id(&state);
    let (state, result) = submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(choice_id),
            selections: vec![other.raw()],
            cancelled: false,
        },
        revision,
    );
    assert!(
        matches!(result, engine_api::CommandResult::Rejected { .. }),
        "牌已离手应拒绝：{result:?}"
    );
    assert_eq!(engine_core::canonical_state_hash(&state), before);
}

#[test]
fn yingba_mark_turn_use_count_survives_snapshot_roundtrip() {
    let (state, owner, other) = started();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(MARK),
            },
            EffectRequest::AddMark {
                player: owner,
                mark: engine_api::MarkDefId::new(MARK),
                delta: 1,
            },
        ],
    )
    .unwrap();
    tx.state_mut().turn.record_turn_card_use(other);
    let state = tx.commit().state;
    let snapshot = engine_core::snapshot::Snapshot::capture(&state);
    let bytes = snapshot.to_bytes().expect("encode snapshot");
    let decoded = engine_core::snapshot::Snapshot::from_bytes(&bytes).expect("decode snapshot");
    let restored = decoded.restore().expect("restore");
    assert_eq!(
        engine_core::canonical_state_hash(&restored),
        engine_core::canonical_state_hash(&state)
    );
    assert_eq!(restored.turn.turn_card_use_count(other), 1);
}

#[test]
fn yingba_mark_turn_use_count_clears_when_current_player_changes() {
    let (state, owner, other) = started();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(MARK),
            },
            EffectRequest::AddMark {
                player: owner,
                mark: engine_api::MarkDefId::new(MARK),
                delta: 1,
            },
        ],
    )
    .unwrap();
    tx.state_mut().turn.record_turn_card_use(other);
    assert_eq!(tx.state().turn.turn_card_use_count(other), 1);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: other,
            phase: engine_core::state::Phase::Play as u8,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().turn.turn_card_use_count(other),
        0,
        "换人后本回合用牌计数必须清零"
    );
}

#[test]
fn yingba_mark_restriction_releases_after_a_real_card_use() {
    let (state, owner, other) = started();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(MARK),
            },
            EffectRequest::AddMark {
                player: owner,
                mark: engine_api::MarkDefId::new(MARK),
                delta: 1,
            },
        ],
    )
    .unwrap();
    let tao = inject(&mut tx, other, CardKind::Tao);
    let sha = inject(&mut tx, other, CardKind::Sha);
    tx.state_mut().players.get_mut(other).unwrap().hp = 3;
    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let revision = state.revision;
    let (state, healed) = submit(
        state,
        other,
        CommandKind::PlayCard {
            card: tao,
            targets: vec![other],
            card_targets: vec![],
        },
        revision,
    );
    assert!(
        matches!(healed, engine_api::CommandResult::Accepted { .. }),
        "先使用一张桃：{healed:?}"
    );
    assert_eq!(state.turn.turn_card_use_count(other), 1);
    let revision = state.revision;
    let (_state, allowed) = submit(
        state,
        other,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![owner],
            card_targets: vec![],
        },
        revision,
    );
    assert!(
        matches!(allowed, engine_api::CommandResult::Accepted { .. }),
        "已使用 1 张真实牌后应允许指定：{allowed:?}"
    );
}

#[test]
fn yingba_mark_two_marks_require_two_real_cards_before_targeting() {
    let (state, owner, other) = started();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(MARK),
            },
            EffectRequest::AddMark {
                player: owner,
                mark: engine_api::MarkDefId::new(MARK),
                delta: 2,
            },
        ],
    )
    .unwrap();
    let tao1 = inject(&mut tx, other, CardKind::Tao);
    let tao2 = inject(&mut tx, other, CardKind::Tao);
    let sha = inject(&mut tx, other, CardKind::Sha);
    tx.state_mut().players.get_mut(other).unwrap().hp = 2;
    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let revision = state.revision;
    let (state, first) = submit(
        state,
        other,
        CommandKind::PlayCard {
            card: tao1,
            targets: vec![other],
            card_targets: vec![],
        },
        revision,
    );
    assert!(matches!(first, engine_api::CommandResult::Accepted { .. }));
    let revision = state.revision;
    let (state, still_blocked) = submit(
        state,
        other,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![owner],
            card_targets: vec![],
        },
        revision,
    );
    assert!(
        matches!(still_blocked, engine_api::CommandResult::Rejected { .. }),
        "2 枚平定只用了 1 张牌仍应拒绝：{still_blocked:?}"
    );
    let revision = state.revision;
    let (state, second) = submit(
        state,
        other,
        CommandKind::PlayCard {
            card: tao2,
            targets: vec![other],
            card_targets: vec![],
        },
        revision,
    );
    assert!(matches!(second, engine_api::CommandResult::Accepted { .. }));
    assert_eq!(state.turn.turn_card_use_count(other), 2);
    let revision = state.revision;
    let (_state, allowed) = submit(
        state,
        other,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![owner],
            card_targets: vec![],
        },
        revision,
    );
    assert!(
        matches!(allowed, engine_api::CommandResult::Accepted { .. }),
        "2 张牌后应允许：{allowed:?}"
    );
}

#[test]
fn pinghe_does_not_offer_recast_for_non_recastable_card() {
    let (state, owner, other) = started();
    let mut tx = Transaction::begin(state);
    let card = inject(&mut tx, owner, CardKind::Sha);
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap_err();
    assert_eq!(err, EffectError::ChoiceRequested);
    let state = tx.commit().state;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(12),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![card.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(
        result.result,
        engine_api::CommandResult::Accepted { .. }
    ));
    let choice = result.choice.expect("target/recast choice");
    assert!(
        !choice.options.iter().any(|option| option.token == 0),
        "非重铸牌不应出现重铸选项：{:?}",
        choice.options
    );
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (_state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(13),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![0],
                cancelled: false,
            },
        },
    );
    assert!(matches!(
        result.result,
        engine_api::CommandResult::Rejected { .. }
    ));
}

#[test]
fn yingba_mark_counts_cards_used_before_gaining_the_mark() {
    let (state, owner, other) = started();
    let mut tx = Transaction::begin(state);
    let tao = inject(&mut tx, other, CardKind::Tao);
    let sha = inject(&mut tx, other, CardKind::Sha);
    tx.state_mut().players.get_mut(other).unwrap().hp = 3;
    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::Play;
    let state = tx.commit().state;
    let revision = state.revision;
    let (state, healed) = submit(
        state,
        other,
        CommandKind::PlayCard {
            card: tao,
            targets: vec![other],
            card_targets: vec![],
        },
        revision,
    );
    assert!(matches!(healed, engine_api::CommandResult::Accepted { .. }));
    assert_eq!(state.turn.turn_card_use_count(other), 1);

    // 标记在已用 1 张牌后才获得；上游 init 会继承已有用牌数。
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(MARK),
            },
            EffectRequest::AddMark {
                player: owner,
                mark: engine_api::MarkDefId::new(MARK),
                delta: 1,
            },
        ],
    )
    .unwrap();
    let state = tx.commit().state;
    let revision = state.revision;
    let (_state, allowed) = submit(
        state,
        other,
        CommandKind::PlayCard {
            card: sha,
            targets: vec![owner],
            card_targets: vec![],
        },
        revision,
    );
    assert!(
        matches!(allowed, engine_api::CommandResult::Accepted { .. }),
        "标记前已用的牌应计入：{allowed:?}"
    );
}
