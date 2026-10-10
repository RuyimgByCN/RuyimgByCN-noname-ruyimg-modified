//! Golden tests for KEY「音爆」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{ContinuationKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{damage_nature, EffectRequest};

const HISAKO_YINBAO: u32 = 3_078_155_020;
const SOURCE: &str = include_str!("../../../content/original/ported/key/hisako_yinbao.skill");
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");

fn started(players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("hisako_yinbao_test", SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_718),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_718,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let others: Vec<_> = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
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
        skill_def: SkillDefId::new(HISAKO_YINBAO),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, others)
}

fn set_judgment_suit(tx: &mut Transaction, suit: Suit) {
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top = tx
        .state()
        .zones
        .get(draw)
        .unwrap()
        .cards
        .last()
        .copied()
        .unwrap();
    tx.state_mut().cards.get_mut(top).unwrap().suit = suit;
}

fn resolve(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (engine_core::state::GameState, PipelineResult) {
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
fn committed_package_contains_both_optional_triggers_and_abi15_module() {
    let committed = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    let compiled = content_compiler::compile_source_named("hisako_yinbao", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 15);
    let triggers: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == HISAKO_YINBAO)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert!(triggers.iter().all(|trigger| trigger.optional));
    assert!(triggers.iter().any(|trigger| trigger.event == "damage"));
    assert!(triggers
        .iter()
        .any(|trigger| trigger.event == "hp_recovered"));
    for expected in &compiled.modules {
        let actual = committed
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn spade_judgment_forces_another_target_and_deals_owner_sourced_thunder_damage() {
    let (mut tx, owner, others) = started(3);
    set_judgment_suit(&mut tx, Suit::Spade);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: others[0],
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );

    let (state, activated) = resolve(tx.commit().state, owner, vec![1], 1);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::JudgeDamageOtherOnSuit
    );
    let offered: Vec<_> = activated
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert!(!offered.contains(&owner.raw()));
    assert!(others.iter().all(|player| offered.contains(&player.raw())));

    let target = others[1];
    let hp_before = state.players.get(target).unwrap().hp;
    let (state, damaged) = resolve(state, owner, vec![target.raw()], 2);
    assert!(matches!(damaged.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp_before - 1);
    assert!(damaged.events.iter().any(|event| matches!(
        event,
        DomainEvent::DamageDealt {
            source,
            target: damaged,
            amount: 1,
            nature: damage_nature::THUNDER,
            ..
        } if *source == owner && *damaged == target
    )));
}

#[test]
fn recovery_also_triggers_but_a_non_spade_result_opens_no_target_choice() {
    let (mut tx, owner, others) = started(2);
    tx.state_mut().players.get_mut(owner).unwrap().hp -= 1;
    set_judgment_suit(&mut tx, Suit::Heart);
    let other_hp = tx.state().players.get(others[0]).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::RecoverHp {
                source: others[0],
                player: owner,
                amount: 1,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, activated) = resolve(tx.commit().state, owner, vec![1], 3);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(state.resolution.pending_choice_id, 0);
    assert_eq!(state.players.get(others[0]).unwrap().hp, other_hp);
}
