//! Golden tests for KEY「寻衅」rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{CardKind, GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NODA_XUNXIN: u32 = 3_836_596_548;
const NODA_XUNXIN2: u32 = 2_159_749_553;
const NODA_FENGCHENG: u32 = 2_452_779_390;
const KEY_NODA: u32 = 3_057_217_957;
const SOURCE: &str = include_str!("../../../../content/original/ported/key/noda_xunxin.skill");
const FENGCHENG_SOURCE: &str =
    include_str!("../../../../content/original/ported/key/noda_fengcheng.skill");
const KEY: &[u8] = include_bytes!("../../../../content/original/ported/key/key.npack.json");

fn skill(state: &GameState, owner: PlayerId, def: u32) -> SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn setup(with_fengcheng: bool) -> (GameState, PlayerId, PlayerId, SkillId) {
    let source = if with_fengcheng {
        format!("{SOURCE}\n{FENGCHENG_SOURCE}")
    } else {
        SOURCE.to_owned()
    };
    let package = content_compiler::compile_source_named("noda_xunxin_test", &source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_718, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let target = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(NODA_XUNXIN),
    });
    if with_fengcheng {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NODA_FENGCHENG),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    for player in [owner, target] {
        for card in tx.state().hand_cards(player) {
            tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
        }
    }
    tx.state_mut().players.get_mut(owner).unwrap().hp = 2;
    tx.state_mut().players.get_mut(target).unwrap().hp = 4;
    let active = skill(tx.state(), owner, NODA_XUNXIN);
    (tx.state().clone(), owner, target, active)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    )
}

fn resolve_forced_gift(
    state: GameState,
    giver: PlayerId,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    assert_eq!(state.resolution.pending_observer, giver.raw());
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let card = state.hand_cards(giver)[0].raw();
    apply_command(
        state,
        Command {
            actor: giver,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![card],
                cancelled: false,
            },
        },
    )
}

#[test]
fn committed_package_contains_dynamic_active_and_folded_helper() {
    let committed = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    let compiled = content_compiler::compile_source_named("noda_xunxin", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 13);
    let active = compiled
        .active_index
        .iter()
        .find(|active| active.skill_def_id == NODA_XUNXIN)
        .unwrap();
    assert_eq!(active.uses_per_turn, None);
    assert_eq!(active.uses_per_turn_source.as_deref(), Some("owner_hp"));
    assert!(active.targets.as_ref().unwrap().hp_at_least_owner);
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NODA_XUNXIN2));
    let noda = committed
        .character_index
        .iter()
        .find(|character| character.id == KEY_NODA)
        .expect("key_noda must be published after both skills are ported");
    assert_eq!(noda.skills, [NODA_FENGCHENG, NODA_XUNXIN]);
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
fn target_hp_and_current_hp_use_limit_are_authoritative() {
    let (state, owner, target, active) = setup(false);
    let mut too_low = state.clone();
    too_low.players.get_mut(target).unwrap().hp = 1;
    let (same, rejected) = activate(too_low.clone(), owner, target, active, 1);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(same, too_low);

    let (state, first) = activate(state, owner, target, active, 2);
    assert!(matches!(first.result, CommandResult::Accepted { .. }));
    let (state, gift) = resolve_forced_gift(state, target, 3);
    assert!(matches!(gift.result, CommandResult::Accepted { .. }));
    let (state, second) = activate(state, owner, target, active, 4);
    assert!(matches!(second.result, CommandResult::Accepted { .. }));
    let (state, gift) = resolve_forced_gift(state, target, 5);
    assert!(matches!(gift.result, CommandResult::Accepted { .. }));
    assert_eq!(state.skills.get(active).unwrap().use_count, 2);

    let (same, third) = activate(state.clone(), owner, target, active, 6);
    assert!(matches!(third.result, CommandResult::Rejected { .. }));
    assert_eq!(same, state);
}

#[test]
fn duel_loser_gives_winner_and_fengcheng_draws_for_the_giver() {
    let (state, owner, target, active) = setup(true);
    let owner_before = state.hand_count(owner);
    let target_before = state.hand_count(target);
    let target_hp = state.players.get(target).unwrap().hp;

    let (state, activated) = activate(state, owner, target, active, 10);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, target_hp - 1);
    let (state, gift) = resolve_forced_gift(state, target, 11);
    assert!(matches!(gift.result, CommandResult::Accepted { .. }));

    assert_eq!(state.hand_count(owner), owner_before + 1);
    assert_eq!(state.hand_count(target), target_before);
    assert_eq!(state.resolution.pending_choice_id, 0);
}

#[test]
fn no_gift_is_requested_when_the_original_duel_target_is_dead() {
    let (state, owner, target, _) = setup(false);
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(target).unwrap().alive = false;
    run_effects(
        &mut tx,
        vec![EffectRequest::EmitDuelResolved {
            card: CardId::NONE,
            initiator: owner,
            target,
            winner: owner,
            loser: target,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().resolution.pending_choice_id, 0);
}
