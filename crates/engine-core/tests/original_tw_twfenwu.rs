//! Golden coverage for TW 张南「奋武」.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{CardKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TWFENWU: u32 = 2_254_911_276;
const TW_ZHANGNAN: u32 = 2_701_918_325;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twfenwu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("twfenwu-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 361);
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
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::DrawCards {
        player: owner,
        count: 2,
    });
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWFENWU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::End;
    (tx, owner, target)
}

fn fire(tx: &mut Transaction, owner: PlayerId) -> Result<(), EffectError> {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )?;
    run_effects(tx, effects).map(|_| ())
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

#[test]
fn twfenwu_keeps_target_first_cost_timing_and_basic_variant_damage() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let zhangnan = committed
        .character_index
        .iter()
        .find(|character| character.id == TW_ZHANGNAN)
        .unwrap();
    assert_eq!(zhangnan.skills, vec![TWFENWU]);

    let (mut cancel, owner, _) = setup(2_026_080_101);
    let hp = cancel.state().players.get(owner).unwrap().hp;
    assert_eq!(fire(&mut cancel, owner), Err(EffectError::ChoiceRequested));
    let optional = cancel.commit();
    // 当前实现：结束阶段触发直接打开“选目标”（可取消）；选中后失 1 体力并使用虚拟杀。
    let (state, accepted) = resolve(optional.state, owner, vec![2], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, hp - 1);

    let (mut damage, owner, target) = setup(2_026_080_102);
    let cards = damage.state().hand_cards(owner);
    assert!(cards.len() >= 2);
    damage.state_mut().cards.get_mut(cards[0]).unwrap().kind = CardKind::Shan;
    damage.state_mut().cards.get_mut(cards[1]).unwrap().kind = CardKind::Tao;
    dispatch_for_events(
        &mut damage,
        &[
            DomainEvent::CardUsed {
                player: owner,
                card: cards[0],
                targets: vec![],
            },
            DomainEvent::CardUsed {
                player: owner,
                card: cards[1],
                targets: vec![],
            },
        ],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(damage.state().turn.distinct_basic_card_variants(owner), 2);
    let owner_hp = damage.state().players.get(owner).unwrap().hp;
    let target_hp = damage.state().players.get(target).unwrap().hp;
    assert_eq!(fire(&mut damage, owner), Err(EffectError::ChoiceRequested));
    let optional = damage.commit();
    let (state, used) = resolve(optional.state, owner, vec![target.raw()], false, 3);
    assert!(matches!(used.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 2);
}
