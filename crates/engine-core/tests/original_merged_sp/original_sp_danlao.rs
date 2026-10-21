//! 第一百四十九批：sp 杨修「啖酪」（danlao）零新原语真实语义重写。
//! 上游：当你成为【杀】或普通锦囊牌的目标后，若此牌的目标数大于 1，
//! 你可以摸一张牌，然后令此牌对你无效。
//! 实现：card_targeted 触发 + owner_in_multitarget_sha_or_trick_targets（本批注册）
//! → draw_cards。免除臂为登记债（AOE 队列无挂起期触发面），行为测试按近似口径。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DANLAO: u32 = 3_326_219_495;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
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
    let others: Vec<_> = state
        .players
        .iter_live()
        .filter(|player| player.id != owner)
        .map(|player| player.id)
        .collect();
    (state, owner, others)
}

fn strip_skills_and_hands(state: GameState) -> GameState {
    let mut tx = Transaction::begin(state);
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
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn add_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
    tx.commit().state
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

/// Inject a nanman into `actor`'s hand and play it against everyone else.
fn play_nanman(
    state: GameState,
    actor: PlayerId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().turn.current = actor;
    let zone = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(92_100),
        kind: CardKind::TrickNanman,
        suit: engine_api::Suit::Spade,
        rank: 9,
        weapon_range: 0,
        zone,
        owner: actor,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    let card_id = card;
    let tx = tx.commit();
    apply_command(
        tx.state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card: card_id,
                targets,
                card_targets: Vec::new(),
            },
        },
    )
}

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

fn hand_of(state: &GameState, player: PlayerId) -> u32 {
    state.hand_count(player)
}

#[test]
fn package_publishes_danlao_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == DANLAO
            && trigger.event == "card_targeted"
            && trigger.filter == "owner_in_multitarget_sha_or_trick_targets"
            && trigger.optional
    }));
    // 啖酪不再保留诚实 noop 的 hand_limit +0 modifier。
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == DANLAO && modifier.target == "hand_limit"));
}

#[test]
fn danlao_draws_and_excludes_owner_from_multitarget_trick() {
    let (state, owner, others) = started(2_026_090_301);
    let state = strip_skills_and_hands(state);
    let state = add_skill(state, owner, DANLAO);
    let owner_hp = hp_of(&state, owner);
    let other_hp = hp_of(&state, others[0]);

    // 他人使用南蛮入侵（目标 = 除使用者外全体，含 owner，目标数 > 1）。
    let (state, played) = play_nanman(state, others[1], others.to_vec(), 1);
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    // 可选触发 confirm。
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, _) = resolve(state, owner, vec![1], false, 2);
    // 批 155：免除臂经 deferred_card_exclusions 在 AOE 队列逐目标时消费——
    // owner 不受南蛮伤害。
    assert!(!state.has_pending_choice(), "resolution expected to finish");
    // 摸 1 + cancel_incoming 共用原语（杜达元帅）副作用：获得被免除的南蛮实体牌。
    assert_eq!(
        hand_of(&state, owner),
        2,
        "draw one + gain the excluded card (shared primitive)"
    );
    assert_eq!(
        hp_of(&state, owner),
        owner_hp,
        "owner must be excluded from nanman"
    );
    assert_eq!(hp_of(&state, others[0]), other_hp - 1);
}

#[test]
fn danlao_not_triggered_by_single_target_card() {
    let (state, owner, others) = started(2_026_090_302);
    let state = strip_skills_and_hands(state);
    let state = add_skill(state, owner, DANLAO);
    let owner_hp = hp_of(&state, owner);

    // 单目标【杀】（目标数 = 1）：啖酪过滤不过，无可选触发。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().turn.current = others[1];
    let zone = tx.state().zones.find(ZoneKind::Hand, others[1]).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(92_100),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 9,
        weapon_range: 0,
        zone,
        owner: others[1],
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    let card_id = card;
    let tx = tx.commit();
    let (state, played) = apply_command(
        tx.state,
        Command {
            actor: others[1],
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: card_id,
                targets: vec![owner],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    assert_ne!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger,
        "single-target sha must not open the danlao confirm"
    );
    assert_eq!(hp_of(&state, owner), owner_hp - 1, "sha lands normally");
}
