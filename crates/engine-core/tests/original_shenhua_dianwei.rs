//! Golden semantics for SHENHUA 典韦「强袭」rewritten from legacy JavaScript.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const QIANGXIX: u32 = 3_151_452_629;
const DIANWEI: u32 = 2_274_294_572;
const SOURCE: &str = include_str!("../../../content/original/ported/shenhua/qiangxix.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/shenhua/shenhua.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    targets: Vec<PlayerId>,
    skill: SkillId,
    weapon: CardId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("qiangxix-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, u64::from(QIANGXIX), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let targets = players[1..].to_vec();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(QIANGXIX),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.max_hp = 8;
        participant.hp = 8;
    }
    let weapon = tx.state().hand_cards(owner)[0];
    let card = tx.state_mut().cards.get_mut(weapon).unwrap();
    card.kind = CardKind::Weapon;
    card.weapon_range = 2;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == QIANGXIX)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        targets,
        skill,
        weapon,
    }
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn activate(
    setup: &Setup,
    state: GameState,
    target: PlayerId,
    key: u64,
) -> (GameState, PipelineResult) {
    submit(
        state,
        setup.owner,
        CommandKind::ActivateSkill {
            skill: setup.skill,
            targets: vec![target],
            card_targets: Vec::new(),
        },
        key,
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id: choice,
            selections,
            cancelled: false,
        },
        key,
    )
}

#[test]
fn package_publishes_dianwei_and_qiangxix_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 671);
    assert!(package.skill_index.len() >= 75);
    assert!(package.character_index.len() >= 16);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == DIANWEI)
        .unwrap();
    assert_eq!(character.skills, [QIANGXIX]);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == QIANGXIX)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(2));
    assert!(active
        .targets
        .as_ref()
        .is_some_and(|targets| !targets.allow_self && targets.unused_by_skill_this_turn));
}

#[test]
fn weapon_and_hp_costs_damage_distinct_targets_with_two_use_limit() {
    let setup = setup();
    let first_hp = setup.state.players.get(setup.targets[0]).unwrap().hp;
    let second_hp = setup.state.players.get(setup.targets[1]).unwrap().hp;
    let owner_hp = setup.state.players.get(setup.owner).unwrap().hp;

    let (state, activation) = activate(&setup, setup.state.clone(), setup.targets[0], 1);
    assert!(matches!(activation.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardWeaponOrLoseHp
    );
    let (state, branch) = resolve(state, setup.owner, vec![1], 2);
    assert!(matches!(branch.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardWeaponOrLoseHp
    );
    let (state, paid) = resolve(state, setup.owner, vec![setup.weapon.raw()], 3);
    assert!(matches!(paid.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, owner_hp);
    assert_eq!(
        state.players.get(setup.targets[0]).unwrap().hp,
        first_hp - 1
    );
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(setup.weapon).unwrap().zone, discard);

    let (same_state, repeated) = activate(&setup, state.clone(), setup.targets[0], 4);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
    assert_eq!(same_state, state);

    let (state, second) = activate(&setup, state, setup.targets[1], 5);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, owner_hp - 1);
    assert_eq!(
        state.players.get(setup.targets[1]).unwrap().hp,
        second_hp - 1
    );

    let (_, limited) = activate(&setup, state, setup.targets[2], 6);
    assert!(matches!(limited.result, CommandResult::Rejected { .. }));
}
