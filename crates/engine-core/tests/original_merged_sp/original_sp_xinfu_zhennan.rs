//! 第一百五十二批：sp「镇南」（xinfu_zhennan）零新原语重写。
//! 上游：当你成为【南蛮入侵】的目标时，你可以对一名其他角色造成 1-3 点随机伤害。
//! 实现：card_targeted + owner_targeted_by_nanman（本批注册）→ choose_other_damage。
//! 近似：随机 1-3 点伤害定为固定 1 点。

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

const XINFU_ZHENNAN: u32 = 4_152_984_845;
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

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
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

#[test]
fn package_publishes_zhennan_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == XINFU_ZHENNAN
            && trigger.event == "card_targeted"
            && trigger.filter == "owner_targeted_by_nanman"
            && trigger.optional
    }));
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == XINFU_ZHENNAN && modifier.target == "hand_limit"));
}

#[test]
fn zhennan_deals_damage_to_chosen_other_when_targeted_by_nanman() {
    let (state, owner, others) = started(2_026_090_401);
    let state = strip_skills_and_hands(state);
    let state = add_skill(state, owner, XINFU_ZHENNAN);
    let owner_hp = hp_of(&state, owner);
    let other_hp = hp_of(&state, others[0]);

    // 他人使用南蛮入侵（owner 为目标之一）。
    let (state, played) = play_nanman(state, others[1], others.to_vec(), 1);
    assert!(matches!(played.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    // 确认后选择伤害目标：选 others[0]。
    let state = resolve(state, owner, vec![1], 2);
    let state = resolve(state, owner, vec![others[0].raw()], 3);
    let _ = state.has_pending_choice();

    // 近似口径：镇南固定 1 点伤害落在被选目标上；owner 另受南蛮 1 点（免除臂未建模，
    // 南蛮对其照常结算）。
    assert_eq!(
        hp_of(&state, others[0]),
        other_hp - 2,
        "chosen target takes zhennan 1 + nanman 1"
    );
    let _ = owner_hp;
}
