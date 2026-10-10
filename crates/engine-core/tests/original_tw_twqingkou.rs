//! Golden coverage for TW 冯习「清口」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, CardKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TWQINGKOU: u32 = 2_492_727_906;
const TW_FENGXI: u32 = 3_711_625_118;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twqingkou.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn setup(seed: u64, target_has_sha: bool) -> (Transaction, PlayerId, PlayerId, Option<u32>) {
    let package = content_compiler::compile_source_named("twqingkou-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 396);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWQINGKOU),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in players {
        for card in tx.state().hand_cards(player) {
            tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
        }
    }
    let response = target_has_sha.then(|| {
        let card = tx.state().hand_cards(target)[0];
        tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Sha;
        card.raw()
    });
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    (tx, owner, target, response)
}

fn fire(tx: &mut Transaction, owner: PlayerId) -> Result<(), EffectError> {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )?;
    run_effects(tx, effects).map(|_| ())
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
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
                cancelled: false,
            },
        },
    )
}

#[test]
fn duel_winner_draws_and_only_owner_win_skips_judge_and_discard() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == TW_FENGXI)
            .unwrap()
            .skills,
        vec![TWQINGKOU]
    );

    let (mut owner_wins, owner, target, _) = setup(2_026_080_201, false);
    let owner_hand = owner_wins.state().hand_count(owner);
    let target_hp = owner_wins.state().players.get(target).unwrap().hp;
    assert_eq!(
        fire(&mut owner_wins, owner),
        Err(EffectError::ChoiceRequested)
    );
    // 当前实现：清口触发后直接打开虚拟决斗目标选择（无独立 optional 确认段）。
    let (state, used) = resolve(owner_wins.commit().state, owner, vec![target.raw()], 1);
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    assert_eq!(state.hand_count(owner), owner_hand + 1);
    assert!(state.pending_judge_skips.contains(&owner));
    assert_ne!(state.turn.flag(turn_flag::SKIP_DISCARD), 0);

    let (mut target_wins, owner, target, response) = setup(2_026_080_202, true);
    let owner_hp = target_wins.state().players.get(owner).unwrap().hp;
    let target_hand = target_wins.state().hand_count(target);
    assert_eq!(
        fire(&mut target_wins, owner),
        Err(EffectError::ChoiceRequested)
    );
    let (state, _) = resolve(target_wins.commit().state, owner, vec![1], 3);
    let (state, _) = resolve(state, owner, vec![target.raw()], 4);
    let (state, answered) = resolve(state, target, vec![response.unwrap()], 5);
    assert!(matches!(answered.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
    assert_eq!(state.hand_count(target), target_hand);
    assert!(!state.pending_judge_skips.contains(&owner));
    assert_eq!(state.turn.flag(turn_flag::SKIP_DISCARD), 0);
}
