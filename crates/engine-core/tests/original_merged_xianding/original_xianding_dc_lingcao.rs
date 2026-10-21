//! Golden for XIANDING 凌操【独锋】（ABI 595）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, MarkDefId,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DC_LINGCAO: u32 = 2_455_294_139;
const DCDUFENG: u32 = 2_523_630_693;
const DCDUFENG_EFFECT: u32 = 226_166_961;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup() -> (GameState, PlayerId, engine_api::SkillId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package())
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(
            tx.state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|player| EffectRequest::DiscardFromHand {
                    player,
                    count: u32::MAX,
                }),
        )
        .chain(std::iter::once(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(DCDUFENG),
        }))
        .collect();
    run_effects(&mut tx, effects).unwrap();
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.max_hp = 5;
    participant.hp = 4;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCDUFENG)
        .unwrap()
        .id;
    (tx.commit().state, owner, skill)
}

fn run(state: GameState, effect: EffectRequest) -> GameState {
    let mut tx = Transaction::begin(state);
    match run_effects(&mut tx, vec![effect]) {
        Ok(_) | Err(EffectError::ChoiceRequested) => {}
        Err(error) => panic!("effect failed: {error}"),
    }
    tx.commit().state
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>) -> GameState {
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    state
}

fn mark_count(state: &GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == MarkDefId::new(DCDUFENG_EFFECT).raw())
        .map_or(0, |(_, count)| *count)
}

#[test]
fn package_closes_dc_lingcao_contract() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 595);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == DC_LINGCAO)
        .unwrap();
    assert_eq!(character.name_key, "dc_lingcao");
    assert_eq!((character.initial_hp, character.max_hp), (4, 5));
    assert_eq!(character.skills, [DCDUFENG]);
    let modifiers: Vec<_> = package
        .modifier_index
        .iter()
        .filter(|modifier| modifier.skill_def_id == DCDUFENG_EFFECT)
        .map(|modifier| modifier.target.as_str())
        .collect();
    assert_eq!(modifiers, ["attack_range", "sha_use_limit"]);
}

#[test]
fn dcdufeng_can_pay_both_costs_then_draw_and_gain_exact_turn_bonus() {
    let (mut state, owner, skill) = setup();
    let base_sha_limit = engine_core::query::sha_use_limit_for(&state, owner);
    state = run(
        state,
        EffectRequest::ResolveDcdufeng {
            player: owner,
            action: 0,
            helper_skill_def: SkillDefId::new(DCDUFENG_EFFECT),
            skill,
        },
    );
    state = choose(state, owner, vec![0, 1]);

    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert_eq!(state.players.get(owner).unwrap().disabled_equip_slots[0], 1);
    assert_eq!(state.hand_count(owner), 3);
    assert_eq!(mark_count(&state, owner), 2);
    assert_eq!(
        engine_core::query::sha_use_limit_for(&state, owner),
        base_sha_limit + 2
    );
}

#[test]
fn dcdufeng_requires_at_least_one_cost() {
    let (state, owner, skill) = setup();
    let state = run(
        state,
        EffectRequest::ResolveDcdufeng {
            player: owner,
            action: 0,
            helper_skill_def: SkillDefId::new(DCDUFENG_EFFECT),
            skill,
        },
    );
    let (_, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: Vec::new(),
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
}
