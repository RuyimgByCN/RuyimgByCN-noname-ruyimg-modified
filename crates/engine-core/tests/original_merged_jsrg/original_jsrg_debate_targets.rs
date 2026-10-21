//! Golden coverage for the first targeted-debate consumer (ADR 0644 P2):
//! jsrgshanzheng consumes start_debate_targets (ABI 656, Cont 272).

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SHANZHENG: u32 = 3_264_543_475;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/jsrg/jsrg.npack.json");
const SRC: &str = include_str!("../../../../content/original/ported/jsrg/jsrgshanzheng.skill");

fn inject_hand(tx: &mut Transaction, player: PlayerId, suit: Suit) {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(333_101),
        kind: CardKind::Shan,
        suit,
        rank: 6,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
}

#[test]
fn shanzheng_npack_consumes_656() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert!(package
        .active_index
        .iter()
        .any(|a| a.skill_def_id == SHANZHENG));
    assert!(package.trigger_index.iter().any(|t| {
        t.skill_def_id == SHANZHENG
            && t.event == "debate_resolved"
            && t.filter == "debate_owner_is_initiator_result_red"
    }));
}

#[test]
fn shanzheng_targeted_debate_red_branch_damages() {
    let compiled = content_compiler::compile_source_named("szz", SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, 41, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let mut others = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|p| *p != owner);
    let a = others.next().unwrap();
    let b = others.next().unwrap();
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    for player in tx.state().players.alive_by_seat() {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardFromHand {
                player,
                count: u32::MAX,
            }],
        )
        .unwrap();
    }
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SHANZHENG),
        }],
    )
    .unwrap();
    inject_hand(&mut tx, a, Suit::Heart); // 参与者 a 红意见
    let hp_b = tx.state().players.get(b).unwrap().hp;
    let skill_id = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == SHANZHENG)
        .unwrap()
        .id;
    let (state, pipe) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill: skill_id,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    // 链：Cont272 选参与者（a）→ DebateResolved（红）→ 红分支 confirm →
    // choose_other_damage 选目标（b）。
    let mut state = state;
    let mut key = 10;
    let mut step = 0;
    while state.resolution.pending_choice_id != 0 && step < 5 {
        let actor = engine_api::PlayerId::new(state.resolution.pending_observer);
        let sel = match step {
            0 => vec![u32::from(a.raw())], // 选参与者
            1 => vec![1],                  // 红分支 confirm
            _ => vec![u32::from(b.raw())], // 伤害目标
        };
        state = resolve(state, actor, sel, key);
        key += 1;
        step += 1;
    }
    assert_eq!(
        state.players.get(b).unwrap().hp,
        hp_b - 1,
        "red branch damages 1"
    );
    let _ = EffectError::ChoiceRequested;
    let _ = DomainEvent::DebateResolved {
        initiator: owner,
        red: vec![a],
        black: vec![],
        result: 1,
    };
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, pipe) = apply_command(
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
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}
