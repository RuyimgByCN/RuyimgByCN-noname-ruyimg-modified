//! Golden coverage for Standard 救援 rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{CardKind, GameState, Identity, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JIUYUAN: u32 = 3_165_045_964;
const SOURCE: &str = include_str!("../../../content/standard/jiuyuan.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    responder: PlayerId,
    tao: engine_api::CardId,
}

fn setup(owner_identity: Identity, responder_character: u32, hp: i32) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_725_185, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let [owner, responder, other] = players.as_slice() else {
        panic!("expected three players");
    };
    let (owner, responder, other) = (*owner, *responder, *other);
    for (player, character) in [(owner, 15), (responder, responder_character), (other, 8)] {
        tx.state_mut()
            .players
            .get_mut(player)
            .unwrap()
            .character_def = character;
    }
    tx.state_mut().players.get_mut(owner).unwrap().identity = owner_identity;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JIUYUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in [owner, responder, other] {
        for card in tx.state().hand_cards(player) {
            tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
        }
    }
    let tao = tx.state().hand_cards(responder)[0];
    tx.state_mut().cards.get_mut(tao).unwrap().kind = CardKind::Tao;
    {
        let player = tx.state_mut().players.get_mut(owner).unwrap();
        player.hp = hp;
        player.dying = true;
    }
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ContinueDyingSave {
                player: owner,
                source: other,
                responders: vec![responder],
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    Setup {
        state: tx.commit().state,
        owner,
        responder,
        tao,
    }
}

fn save(setup: Setup) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(setup.state.resolution.pending_choice_id);
    apply_command(
        setup.state,
        Command {
            actor: setup.responder,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_closes_sunquan_with_locked_lord_skill() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let sunquan = package
        .character_index
        .iter()
        .find(|character| character.id == 15)
        .unwrap();
    assert_eq!(sunquan.skills, [10, JIUYUAN]);
    let skill = package
        .skill_index
        .iter()
        .find(|skill| skill.skill_def_id == JIUYUAN)
        .unwrap();
    assert!(skill.locked);
    assert!(skill.lord_only);
    let compiled = content_compiler::compile_source_named("standard-jiuyuan", SOURCE).unwrap();
    assert_eq!(
        compiled.trigger_index[0].filter,
        "owner_dying_other_tao_responder_group_wu"
    );
}

#[test]
fn other_wu_tao_recovers_dying_lord_twice_with_responder_as_source() {
    let setup = setup(Identity::Lord, 16, -1);
    let owner = setup.owner;
    let responder = setup.responder;
    let tao = setup.tao;
    let (state, result) = save(setup);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?} events={:?}",
        result.result,
        result.events
    );
    assert!(result.choice.is_none());
    assert_eq!(state.players.get(owner).unwrap().hp, 1);
    assert!(!state.players.get(owner).unwrap().dying);
    assert_eq!(
        state.cards.get(tao).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert_eq!(
        result
            .events
            .iter()
            .filter(|event| matches!(
                event,
                DomainEvent::HpRecovered {
                    source,
                    player,
                    amount: 1,
                } if *source == responder && *player == owner
            ))
            .count(),
        2
    );
}

#[test]
fn non_wu_or_non_lord_tao_only_performs_the_base_recovery() {
    for (identity, responder_character) in [(Identity::Lord, 8), (Identity::Rebel, 16)] {
        let setup = setup(identity, responder_character, 0);
        let owner = setup.owner;
        let (state, result) = save(setup);
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        assert_eq!(state.players.get(owner).unwrap().hp, 1);
        assert!(!state.players.get(owner).unwrap().dying);
        assert_eq!(
            result
                .events
                .iter()
                .filter(|event| matches!(
                    event,
                    DomainEvent::HpRecovered {
                        player,
                        amount: 1,
                        ..
                    } if *player == owner
                ))
                .count(),
            1
        );
    }
}
