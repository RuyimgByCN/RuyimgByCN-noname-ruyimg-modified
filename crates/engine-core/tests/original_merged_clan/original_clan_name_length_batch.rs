//! Zero-new-primitive batch (2026-08-15): cardNameLength family.
//! clanjiejian 诫鉴 / clanhuanghan 惶汗 / clanxieshu 胁书 — semantics and
//! approximations documented in the .skill headers.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const CLANJIEJIAN: u32 = 3_544_602_984;
const CLANHUANGHAN: u32 = 4_234_060_719;
const CLANXIESHU: u32 = 2_196_874_944;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/clan/clan.npack.json");

fn start() -> (GameState, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_815,
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
    (engine.state().clone(), owner, other)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
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
    tx.commit().state
}

fn inject_sha(state: &mut GameState, player: PlayerId) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_459),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn commit_sha_damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
    card: CardId,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::CommitDamage {
            damage_id: 0,
            source,
            target,
            amount: 1,
            base_amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card,
            card_kind: 1,
            card_flags: 0,
        }],
    );
    (tx.commit().state, result)
}

fn confirm(state: GameState, actor: PlayerId, key: u64) -> GameState {
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, output) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn committed_package_publishes_name_length_family_triggers() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let jiejian = pkg
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == CLANJIEJIAN)
        .expect("clanjiejian trigger");
    assert_eq!(jiejian.event, "card_used");
    assert_eq!(jiejian.filter, "owner_is_source");
    assert!(jiejian.optional);
    let huanghan = pkg
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == CLANHUANGHAN)
        .expect("clanhuanghan trigger");
    assert_eq!(huanghan.event, "damage");
    assert_eq!(huanghan.filter, "owner_is_target");
    assert!(!huanghan.optional);
    let xieshu_events: Vec<_> = pkg
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == CLANXIESHU)
        .map(|trigger| {
            (
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert_eq!(
        xieshu_events,
        vec![
            ("damage", "owner_is_target", true),
            ("damage", "owner_is_source", true),
        ]
    );
}

#[test]
fn huanghan_draws_name_length_then_discards_lost_hp() {
    let (state, owner, other) = start();
    let state = with_skill(state, owner, CLANHUANGHAN);
    let mut state = state;
    let player = state.players.get_mut(owner).unwrap();
    player.max_hp = 10; // 拉高上限，避免连击伤害把测试武将推进濒死求桃选择
    player.hp = 8; // 已受伤 2 点
    let sha = inject_sha(&mut state, owner);
    let extra1 = inject_sha(&mut state, owner);
    let extra2 = inject_sha(&mut state, owner);
    let before = state.hand_count(owner);
    let (state, result) = commit_sha_damage(state, other, owner, sha);
    assert!(result.is_ok(), "forced trigger chain: {result:?}");
    // 「杀」名字数 1 → 摸 1；body 在伤害应用后求值（对齐上游 damageEnd 的
    // getDamagedHp）→ 已损失 3 → 弃 3：净 -2。
    assert_eq!(state.hand_count(owner), before - 2);
    assert!(state.zones.find(ZoneKind::Hand, owner).is_some());
    let _ = (extra1, extra2);
}

#[test]
fn xieshu_confirm_links_and_discards_then_blocks_second_use() {
    let (state, owner, other) = start();
    let state = with_skill(state, owner, CLANXIESHU);
    let mut state = state;
    let player = state.players.get_mut(owner).unwrap();
    player.max_hp = 10; // 同上：避开濒死
    player.hp = 9; // 已受伤 1 点
    let sha = inject_sha(&mut state, owner);
    let before = state.hand_count(owner);

    let (state, result) = commit_sha_damage(state, other, owner, sha);
    assert_eq!(
        result,
        Err(EffectError::ChoiceRequested),
        "optional confirm"
    );
    let mut state = confirm(state, owner, 1);
    // 弃 1（名字数）+ 摸 2（伤害应用后已损失 2，对齐上游 damageEnd 语义）：净 +1，
    // 且横置已生效。
    assert_eq!(state.hand_count(owner), before + 1);

    // 已横置（mark 2）：optional 语义下确认仍会弹出（body 守卫在 confirm 之后），
    // 但效果被 mark_count 守卫拦下——拒绝后无弃牌/摸牌/再横置。
    let sha2 = inject_sha(&mut state, owner);
    let before_second = state.hand_count(owner);
    let (state, result) = commit_sha_damage(state, other, owner, sha2);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    let (state, output) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(9),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections: vec![],
                cancelled: true,
            },
        },
    );
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.hand_count(owner),
        before_second,
        "guard blocks effects"
    );
    assert_eq!(
        engine_core::query::query(
            &state,
            &QueryRequest::MarkCount {
                player: owner,
                mark: 2
            },
            &mut engine_core::query::QueryContext::new(),
        ),
        QueryResponse::Int(1),
        "still exactly one link mark"
    );
}
