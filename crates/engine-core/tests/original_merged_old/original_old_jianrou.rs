//! Golden coverage for old「剑柔」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JIANROU: u32 = 4_238_996_080;
const SOURCE: &str = include_str!("../../../../content/original/ported/old/jianrou.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/old/old.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    skill: SkillId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("jianrou-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 2_026_072_811, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JIANROU),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    Setup {
        state: tx.commit().state,
        owner,
        source,
        skill,
    }
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind, suit: Suit) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
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
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn prevent_with(mut setup: Setup, first: (CardKind, Suit), second: (CardKind, Suit)) -> GameState {
    let a = inject(&mut setup.state, setup.owner, 99_801, first.0, first.1);
    let b = inject(&mut setup.state, setup.owner, 99_802, second.0, second.1);
    let hp = setup.state.players.get(setup.owner).unwrap().hp;
    let mut tx = Transaction::begin(setup.state);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: setup.source,
                target: setup.owner,
                amount: 1,
                is_lose_hp: false,
                nature: rule_api::damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let state = resolve(tx.commit().state, setup.owner, vec![1], 1);
    let state = resolve(state, setup.owner, vec![a.raw(), b.raw()], 2);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, hp);
    state
}

#[test]
fn prevents_damage_and_only_refunds_round_use_for_same_color_or_type() {
    let refunded = setup();
    let skill = refunded.skill;
    let hand = refunded.state.hand_count(refunded.owner);
    let state = prevent_with(
        refunded,
        (CardKind::Sha, Suit::Spade),
        (CardKind::Tao, Suit::Heart),
    );
    assert_eq!(
        state.hand_count(state.skills.get(skill).unwrap().owner),
        hand + 1
    );
    assert_eq!(state.skills.get(skill).unwrap().round_fire_round, 0);

    let setup = setup();
    let round = setup.state.turn.round;
    let skill = setup.skill;
    let state = prevent_with(
        setup,
        (CardKind::Sha, Suit::Heart),
        (CardKind::Weapon, Suit::Spade),
    );
    assert_eq!(state.skills.get(skill).unwrap().round_fire_round, round);
}

#[test]
fn package_publishes_jianrou_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 275);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JIANROU && skill.name == "jianrou"));
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}
