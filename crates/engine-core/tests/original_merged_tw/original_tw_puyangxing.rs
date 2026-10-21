//! Golden coverage for TW 濮阳兴「征建/众斥」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWZHENGJIAN: u32 = 3_256_371_970;
const TWZHONGCHI_EFFECT: u32 = 3_469_371_163;
const SOURCE: &str = include_str!("../../../../content/original/ported/tw/twpuyangxing.skill");

fn inject(tx: &mut Transaction, owner: PlayerId, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit: Suit::Diamond,
        rank: 8,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn distinct_tribute_sources_awaken_zhongchi_then_enable_damage() {
    let package = content_compiler::compile_source_named("tw-puyangxing-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 381);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 4, 2_026_080_122, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second, third) = (players[0], players[1], players[2], players[3]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWZHENGJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == TWZHENGJIAN)
        .unwrap()
        .id;
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwzhengjian {
                player: owner,
                target: owner,
                helper_skill_def: TWZHONGCHI_EFFECT,
                action: 0,
                stage: 0,
                card: CardId::NONE,
                branch: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut state = choose(tx.commit().state, owner, vec![0], 1);

    for (index, target) in [first, second].into_iter().enumerate() {
        let mut tx = Transaction::begin(state);
        let card = inject(&mut tx, target, 93_810 + index as u32);
        assert_eq!(
            run_effects(
                &mut tx,
                vec![EffectRequest::ResolveTwzhengjian {
                    player: owner,
                    target,
                    helper_skill_def: TWZHONGCHI_EFFECT,
                    action: 1,
                    stage: 0,
                    card: CardId::NONE,
                    branch: 0,
                    skill,
                }],
            ),
            Err(EffectError::ChoiceRequested)
        );
        state = choose(
            tx.commit().state,
            target,
            vec![card.raw()],
            10 + index as u64 * 2,
        );
        state = choose(state, owner, vec![0], 11 + index as u64 * 2);
    }

    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert!(state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == TWZHONGCHI_EFFECT)
        }));
    let target_hp = state.players.get(third).unwrap().hp;
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwzhengjian {
                player: owner,
                target: third,
                helper_skill_def: TWZHONGCHI_EFFECT,
                action: 1,
                stage: 0,
                card: CardId::NONE,
                branch: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = choose(tx.commit().state, owner, vec![1], 20);
    assert_eq!(state.players.get(third).unwrap().hp, target_hp - 1);
}
