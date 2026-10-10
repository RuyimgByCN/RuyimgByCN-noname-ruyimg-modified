//! Golden for SB Diaochan's distinct-damaged-player end-phase draw count.

use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId};
use engine_core::effect::run_effects;
use engine_core::state::{turn_flag, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SBBIYUE: u32 = 4_152_380_059;
const SOURCE: &str = include_str!("../../../content/original/ported/sb/sbbiyue.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sb/sb.npack.json");

fn prepared() -> (Transaction, Vec<PlayerId>) {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let compiled = content_compiler::compile_source_named("sb", SOURCE).unwrap();
    // 全包逻辑哈希含全部界武将技能；此处校验包内登记与编译契约一致。
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == SBBIYUE));
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == SBBIYUE));

    let mut engine = Engine::new();
    engine.load_package(&committed).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 5,
            rng_seed: 20_260_714_379,
            ruleset: "identity-standard".into(),
        },
    });
    let players: Vec<_> = engine
        .state()
        .players
        .iter_live()
        .map(|player| player.id)
        .collect();
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SBBIYUE),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let state = tx.state_mut().players.get_mut(*player).unwrap();
        state.hp = 10;
        state.max_hp = 10;
    }
    (tx, players)
}

#[test]
fn sbbiyue_counts_distinct_damaged_players_and_caps_draw_at_four() {
    for (damaged, expected_draw) in [(vec![], 1), (vec![1, 1, 2], 3), (vec![0, 1, 2, 3, 4], 4)] {
        let (mut tx, players) = prepared();
        let owner = tx.state().turn.current;
        for index in damaged {
            run_effects(
                &mut tx,
                vec![EffectRequest::DealDamage {
                    source: PlayerId::NONE,
                    target: players[index],
                    amount: 1,
                    is_lose_hp: false,
                    nature: 0,
                }],
            )
            .unwrap();
        }
        let hand = tx.state().hand_count(owner);
        run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::End.to_u8(),
            }],
        )
        .unwrap();
        assert_eq!(tx.state().hand_count(owner), hand + expected_draw);
    }
}

#[test]
fn damaged_player_history_resets_when_the_turn_owner_changes() {
    let (mut tx, players) = prepared();
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::NONE,
            target: players[1],
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().turn.damaged_player_count(), 1);

    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: players[1],
            phase: Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().turn.flag(turn_flag::DAMAGED_PLAYERS), 0);
}
