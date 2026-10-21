//! Golden coverage for 界廖化「当先」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const XINDANGXIAN: u32 = 3_456_939_183;
const XINFULI: u32 = 3_062_292_376;
const RE_LIAOHUA: u32 = 1_916_827_798;
const YIJIANG_PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");
const REFRESH_PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");
const SOURCE: &str = concat!(
    include_str!("../../../../content/original/ported/yijiang/xindangxian.skill"),
    "\n",
    include_str!("../../../../content/original/ported/yijiang/xinfuli.skill"),
);

struct Setup {
    state: GameState,
    owner: PlayerId,
    previous: PlayerId,
    sha: CardId,
}

fn setup(seed: u64, owns_xinfuli: bool) -> Setup {
    let package = content_compiler::compile_source_named("xindangxian", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 200);
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();

    let seats = tx.state().players.alive_by_seat();
    let previous = tx.state().turn.current;
    let previous_index = seats.iter().position(|player| *player == previous).unwrap();
    let owner = seats[(previous_index + 1) % seats.len()];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(XINDANGXIAN),
    });
    if owns_xinfuli {
        effects.push(EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(XINFULI),
        });
    }
    effects.push(EffectRequest::SetTurnState {
        current: previous,
        phase: Phase::End.to_u8(),
    });
    run_effects(&mut tx, effects).unwrap();

    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(95_001),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: discard,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(sha);
    Setup {
        state: tx.commit().state,
        owner,
        previous,
        sha,
    }
}

fn end_phase(state: GameState, actor: PlayerId, key: u64) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::EndPhase,
        },
    )
}

#[test]
fn committed_packages_publish_xindangxian_and_complete_re_liaohua() {
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG_PACKAGE).unwrap();
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    assert_eq!(yijiang.skill_index.len(), 281);
    assert_eq!(yijiang.trigger_index.len(), 215);
    assert!(yijiang
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == XINDANGXIAN && skill.locked));

    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH_PACKAGE).unwrap();
    assert!(refresh.manifest.rule_vm_abi_version >= 251);
    let character = refresh
        .character_index
        .iter()
        .find(|character| character.id == RE_LIAOHUA)
        .unwrap();
    assert_eq!(character.skills, [XINDANGXIAN, XINFULI]);
}

#[test]
fn unused_xinfuli_forces_hp_loss_then_gains_discard_sha() {
    let setup = setup(2_026_072_601, true);
    let before_hp = setup.state.players.get(setup.owner).unwrap().hp;
    let hand = setup.state.zones.find(ZoneKind::Hand, setup.owner).unwrap();

    let (state, result) = end_phase(setup.state, setup.previous, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.turn.current, setup.owner);
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(state.turn.flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE), 1);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, before_hp - 1);
    assert_eq!(state.cards.get(setup.sha).unwrap().zone, hand);
}

#[test]
fn consumed_xinfuli_makes_cost_optional_and_cancel_preserves_state() {
    let setup = setup(2_026_072_602, false);
    let before_hp = setup.state.players.get(setup.owner).unwrap().hp;
    let discard = setup
        .state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();

    let (state, offered) = end_phase(setup.state, setup.previous, 2);
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, cancelled) = apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(3),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: Vec::new(),
                cancelled: true,
            },
        },
    );
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(setup.owner).unwrap().hp, before_hp);
    assert_eq!(state.cards.get(setup.sha).unwrap().zone, discard);
}

#[test]
fn hp_loss_gain_is_inert_outside_its_originating_extra_phase() {
    let setup = setup(2_026_072_603, true);
    let skill = setup
        .state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == setup.owner && skill.def.raw() == XINDANGXIAN)
        .unwrap()
        .id;
    let before_hp = setup.state.players.get(setup.owner).unwrap().hp;
    let sha_zone = setup.state.cards.get(setup.sha).unwrap().zone;
    let mut tx = Transaction::begin(setup.state);
    tx.state_mut().turn.current = setup.owner;
    tx.state_mut().turn.phase = Phase::Play;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::LoseHpGainDiscardCardForOpeningExtraPlay {
                player: setup.owner,
                kind: CardKind::Sha.tag(),
                mandatory_skill_def: SkillDefId::new(XINFULI),
                skill,
            }]
        ),
        Ok(Vec::new())
    );
    assert_eq!(tx.state().players.get(setup.owner).unwrap().hp, before_hp);
    assert_eq!(tx.state().cards.get(setup.sha).unwrap().zone, sha_zone);
}
