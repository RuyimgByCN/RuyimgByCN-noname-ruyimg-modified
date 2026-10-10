//! Golden for SP 巧织 olqiaozhi (ABI 740, tag 761, Cont 330):
//! 出牌阶段，弃一张牌 → 亮牌堆顶两张 → 获得其中一张（打「巧织」标，
//! 未选入弃）；手牌有巧织牌时不可发动。active 全管道驱动。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLQIAOZHI: u32 = 3_591_197_180;
const OLQIAOZHI_TAG: u32 = 1_909_178_934;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

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

fn inject(state: &mut GameState, owner: PlayerId, zone: ZoneKind, kind: CardKind, tag: u32) -> CardId {
    let zone = state.zones.find(zone, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(97_000 + tag),
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
fn package_publishes_olqiaozhi_active_contract() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == OLQIAOZHI)
        .expect("olqiaozhi active entry");
    assert!(active.hand_nonempty);
    // 批 301：命令级 gaintag gate。
    assert_eq!(active.hand_missing_gaintag, Some(1_909_178_934));
}

#[test]
fn olqiaozhi_discards_reveals_and_gains_tagged_card() {
    let (mut state, players) = started(2_026_096_001);
    let owner = players[0];
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let skill = add_skill(&mut state, owner, OLQIAOZHI);
    let cost = inject(&mut state, owner, ZoneKind::Hand, CardKind::Tao, 1);
    let top_a = inject(&mut state, PlayerId::NONE, ZoneKind::DrawPile, CardKind::Shan, 2);
    let top_b = inject(&mut state, PlayerId::NONE, ZoneKind::DrawPile, CardKind::Other, 3);
    let hand_before = state.hand_count(owner);

    // 发动 → stage 0：开弃成本窗口。
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
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveOlqiaozhiStage
    );

    // 弃成本 → 亮顶 2 选 1。
    let (state, result) = resolve(state, owner, vec![cost.raw()], false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveOlqiaozhiStage
    );
    // 选择窗口选项 = 亮出的两张。
    let choice = result.choice.as_ref().expect("reveal choice open");
    let options: Vec<u32> = choice.options.iter().map(|option| option.token).collect();
    assert_eq!(options.len(), 2);
    assert!(options.contains(&top_a.raw()) && options.contains(&top_b.raw()));

    // 选 top_a → 打标入手牌；top_b 入弃牌堆。
    let (state, result) = resolve(state, owner, vec![top_a.raw()], false, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.hand_cards(owner).contains(&top_a));
    assert!(state
        .content_card_tags
        .get(&top_a)
        .is_some_and(|tags| tags.contains(&OLQIAOZHI_TAG)));
    let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap();
    assert!(state.zones.get(discard).unwrap().cards.contains(&top_b));
    // 净手牌：-1 成本 +1 所得。
    assert_eq!(state.hand_count(owner), hand_before);
    assert!(!state.has_pending_choice());
}

#[test]
fn olqiaozhi_gate_blocks_reactivation_while_tagged_card_in_hand() {
    let (mut state, players) = started(2_026_096_002);
    let owner = players[0];
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let skill = add_skill(&mut state, owner, OLQIAOZHI);
    inject(&mut state, owner, ZoneKind::Hand, CardKind::Tao, 4);
    // 手牌预置一张「巧织」牌。
    let tagged = inject(&mut state, owner, ZoneKind::Hand, CardKind::Other, 5);
    state.content_card_tags.entry(tagged).or_default().insert(OLQIAOZHI_TAG);

    let (state, result) = submit(
        state,
        owner,
        CommandKind::ActivateSkill {
            skill,
            targets: vec![],
            card_targets: Vec::new(),
        },
        1,
    );
    // 批 301：命令级 gaintag gate → 发动被拒。
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert!(!state.has_pending_choice());
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
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
