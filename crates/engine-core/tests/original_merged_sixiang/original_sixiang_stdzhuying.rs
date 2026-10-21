//! Golden tests for 四象「驻营」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{damage_nature, EffectRequest};

const SIXIANG: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");
const SOURCE: &str = include_str!("../../../../content/original/ported/sixiang/stdzhuying.skill");
const STDZHUYING: u32 = 2_174_724_909;
const STDZHONGSHI: u32 = 3_843_853_491;
const STD_PANGLIN: u32 = 3_614_379_071;

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(SIXIANG).unwrap();
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
    let players = engine.state().players.alive_by_seat();
    let (owner, source, target) = (players[0], players[1], players[2]);
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
        skill_def: SkillDefId::new(STDZHUYING),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, source, target)
}

fn damage(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    nature: u8,
) -> Result<(), EffectError> {
    run_effects(
        tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature,
        }],
    )
    .map(|_| ())
}

fn linked(tx: &Transaction, player: PlayerId) -> bool {
    tx.state()
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0)
}

fn set_linked(tx: &mut Transaction, player: PlayerId) {
    tx.state_mut()
        .players
        .get_mut(player)
        .unwrap()
        .marks
        .push((engine_core::content::MARK_CHAIN, 1));
}

#[test]
fn committed_package_declares_abi76_and_precise_filter_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(SIXIANG).unwrap();
    let compiled = content_compiler::compile_source_named("stdzhuying", SOURCE).unwrap();

    assert_eq!(compiled.manifest.rule_vm_abi_version, 73);
    let trigger = &compiled.trigger_index[0];
    assert_eq!(trigger.event, "damage_before");
    assert_eq!(
        trigger.filter,
        "owner_alive_other_normal_damage_target_unlinked"
    );
    assert!(trigger.optional);
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDZHUYING && !skill.locked));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_PANGLIN)
        .unwrap();
    assert_eq!(character.name_key, "std_panglin");
    assert_eq!(character.skills, vec![STDZHUYING, STDZHONGSHI]);
}

#[test]
fn normal_damage_to_another_unlinked_player_offers_link_before_damage() {
    let (mut tx, owner, source, target) = setup(20_260_724);
    let hp = tx.state().players.get(target).unwrap().hp;

    assert_eq!(
        damage(&mut tx, source, target, damage_nature::NORMAL),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(tx.state().resolution.pending_observer, owner.raw());
    assert!(!linked(&tx, target));

    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(linked(&Transaction::begin(state.clone()), target));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);
}

#[test]
fn elemental_self_or_already_linked_damage_never_offers_the_skill() {
    let (mut elemental, _owner, source, target) = setup(20_260_725);
    damage(&mut elemental, source, target, damage_nature::FIRE).unwrap();
    assert!(!elemental.state().has_pending_choice());
    assert!(!linked(&elemental, target));

    let (mut self_damage, owner, source, _) = setup(20_260_726);
    damage(&mut self_damage, source, owner, damage_nature::NORMAL).unwrap();
    assert!(!self_damage.state().has_pending_choice());
    assert!(!linked(&self_damage, owner));

    let (mut already_linked, _owner, source, target) = setup(20_260_727);
    set_linked(&mut already_linked, target);
    damage(&mut already_linked, source, target, damage_nature::NORMAL).unwrap();
    assert!(!already_linked.state().has_pending_choice());
    assert!(linked(&already_linked, target));
}
