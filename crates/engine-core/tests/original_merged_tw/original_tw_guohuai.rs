//! Golden coverage for TW 郭淮「精策/御章」.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWJINGCE: u32 = 3_238_855_457;
const YUZHANG: u32 = 2_473_230_123;
const DONT_USE: u32 = 3_964_351_777;
const SOURCE: &str = include_str!("../../../../content/original/ported/tw/twguohuai.skill");

fn skill(tx: &Transaction, owner: PlayerId, def: u32) -> engine_api::SkillId {
    tx.state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == def)
        .unwrap()
        .id
}

#[test]
fn jingce_earns_mark_and_yuzhang_bans_source() {
    let package = content_compiler::compile_source_named("tw-guohuai-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 388);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_207, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, source) = (players[0], players[1]);
    let remove: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(TWJINGCE),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(YUZHANG),
            },
        ],
    )
    .unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let jingce = skill(&tx, owner, TWJINGCE);
    let card = tx.state().hand_cards(owner)[0];
    let hand_before = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwguohuai {
            player: owner,
            target: owner,
            source: owner,
            card,
            mark_def: YUZHANG,
            helper_skill_def: DONT_USE,
            action: 0,
            stage: 0,
            skill: jingce,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand_before + 2);
    assert!(!tx.state().skills.get(jingce).unwrap().state_blob.is_empty());
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTwguohuai {
            player: owner,
            target: source,
            source: owner,
            card,
            mark_def: YUZHANG,
            helper_skill_def: DONT_USE,
            action: 1,
            stage: 0,
            skill: jingce,
        }],
    )
    .unwrap();
    assert!(tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .contains(&(YUZHANG, 1)));

    let yuzhang = skill(&tx, owner, YUZHANG);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwguohuai {
                player: owner,
                target: owner,
                source,
                card: CardId::NONE,
                mark_def: YUZHANG,
                helper_skill_def: DONT_USE,
                action: 3,
                stage: 0,
                skill: yuzhang,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, _)| *mark == YUZHANG));
    assert!(state
        .players
        .get(source)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|skill| skill.def.raw() == DONT_USE)
        }));

    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    let yuzhang = skill(&tx, owner, YUZHANG);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddMark {
                player: owner,
                mark: engine_api::MarkDefId::new(YUZHANG),
                delta: 1,
            },
            EffectRequest::ResolveTwguohuai {
                player: owner,
                target: owner,
                source: PlayerId::NONE,
                card: CardId::NONE,
                mark_def: YUZHANG,
                helper_skill_def: DONT_USE,
                action: 4,
                stage: 0,
                skill: yuzhang,
            },
        ],
    )
    .unwrap();
    assert_eq!(tx.state().turn.phase, Phase::Judge);
}
