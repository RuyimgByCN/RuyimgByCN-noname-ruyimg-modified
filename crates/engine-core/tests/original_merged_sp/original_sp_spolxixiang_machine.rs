//! Golden for SP 西向 spolxixiang (ABI 741):
//! owner_basic_uses_plus_one_cards 动态张数原语——转换素材最少
//! 1+本局基本牌使用数张；计数器经真实 CardUsed 事件递增（track 门控）。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SPOLXIXIANG: u32 = 2_531_785_939;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn started(seed: u64) -> (GameState, Vec<PlayerId>) {
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package())
        .unwrap();
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

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, tag: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_000 + tag),
        kind,
        suit: Suit::Club,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
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

#[test]
fn package_publishes_spolxixiang_dynamic_contract() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    let rules: Vec<_> = package
        .view_as_index
        .iter()
        .filter(|view| view.skill_def_id == SPOLXIXIANG)
        .collect();
    assert_eq!(rules.len(), 2);
    for rule in &rules {
        assert!(rule.owner_basic_uses_plus_one_cards);
    }
    for rule in &rules {
        assert!(rule.once_per_turn_per_branch);
    }
    assert!(rules
        .iter()
        .any(|rule| rule.as_kind == "sha" && rule.no_use_count));
    assert!(rules.iter().any(|rule| rule.as_kind == "juedou"));
}

#[test]
fn basic_use_counter_increments_via_real_card_used_events() {
    let (mut state, players) = started(2_026_097_001);
    let owner = players[0];
    let tao_card = inject(&mut state, owner, CardKind::Tao, 1);
    let mut tx = Transaction::begin(state);
    dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player: owner,
            card: tao_card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(tx.state().lifetime_basic_card_uses, 1);
}

#[test]
fn conversion_requires_plus_one_source_cards() {
    let (mut state, players) = started(2_026_097_002);
    let owner = players[0];
    let target = players[1];
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let skill = add_skill(&mut state, owner, SPOLXIXIANG);
    // 本局基本牌使用数 = 1 → 至少 2 张素材。
    state.lifetime_basic_card_uses = 1;
    let c1 = inject(&mut state, owner, CardKind::Tao, 1);
    let c2 = inject(&mut state, owner, CardKind::Shan, 2);
    let c3 = inject(&mut state, owner, CardKind::Other, 3);
    let hp_before = state.players.get(target).unwrap().hp;

    // 1 张素材 → 拒绝。
    let (_, rejected) = submit(
        state.clone(),
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![c1],
        },
        1,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    // 2 张素材 → 接受（转换【决斗】结算：目标掉血）。
    let (state, accepted) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![c2, c3],
        },
        2,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(
        state.players.get(target).unwrap().hp < hp_before,
        "决斗结算应使目标掉血"
    );
}

#[test]
fn each_rule_usable_once_per_turn() {
    let (mut state, players) = started(2_026_097_003);
    let owner = players[0];
    let target = players[1];
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let skill = add_skill(&mut state, owner, SPOLXIXIANG);
    let c1 = inject(&mut state, owner, CardKind::Tao, 1);
    let c2 = inject(&mut state, owner, CardKind::Shan, 2);
    let c3 = inject(&mut state, owner, CardKind::Other, 3);

    // 第一发：sha 规则（index 0，位 0）。
    let (state, first) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![c1],
        },
        1,
    );
    assert!(matches!(first.result, CommandResult::Accepted { .. }));

    // 第二发：sha 位已置跳过，决斗规则（index 1，位 1）接管。
    let (state, second) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![c2],
        },
        2,
    );
    assert!(matches!(second.result, CommandResult::Accepted { .. }));

    // 第三发：两条规则位均已置 → 无匹配规则 → 拒绝。
    let (_, third) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![target],
            card_targets: vec![c3],
        },
        3,
    );
    assert!(matches!(third.result, CommandResult::Rejected { .. }));
}
