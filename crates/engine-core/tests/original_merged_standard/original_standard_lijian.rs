//! Golden coverage for Standard 离间 rewritten from the legacy JavaScript semantics.

use engine_api::{
    CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const LIJIAN: u32 = 2_292_478_631;
const DIAOCHAN: u32 = 25;
const JUE_DOU_LOCK: u32 = 2_000_000_183;
const SOURCE: &str = include_str!("../../../../content/standard/lijian.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/standard/standard.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    user: PlayerId,
    observer: PlayerId,
    skill: SkillId,
    cost: CardId,
    wuxie: CardId,
}

fn setup(lock_user: bool) -> Setup {
    let standard = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&standard)
        .unwrap();
    bootstrap_match(&mut tx, 4, 20_260_725_183, "identity-standard").unwrap();
    if lock_user {
        let lock = content_compiler::compile_source_named(
            "standard-lijian-lock-test",
            &format!(
                "skill juedou_lock({JUE_DOU_LOCK}) locked {{
                    modifier card_use_cancel(juedou) override 1;
                }}"
            ),
        )
        .unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content)
            .load_package(&lock)
            .unwrap();
    }
    let players = tx.state().players.alive_by_seat();
    let [owner, target, user, observer] = players.as_slice() else {
        panic!("expected four players");
    };
    let (owner, target, user, observer) = (*owner, *target, *user, *observer);
    tx.state_mut().players.get_mut(owner).unwrap().character_def = DIAOCHAN;
    tx.state_mut()
        .players
        .get_mut(target)
        .unwrap()
        .character_def = 1;
    tx.state_mut().players.get_mut(user).unwrap().character_def = 2;
    tx.state_mut()
        .players
        .get_mut(observer)
        .unwrap()
        .character_def = 7;

    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LIJIAN),
    });
    if lock_user {
        effects.push(EffectRequest::AddSkill {
            player: user,
            skill_def: SkillDefId::new(JUE_DOU_LOCK),
        });
    }
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    for player in [owner, target, user, observer] {
        for card in tx.state().hand_cards(player) {
            tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Other;
        }
    }
    let cost = tx.state().hand_cards(owner)[0];
    let wuxie = tx.state().hand_cards(observer)[0];
    tx.state_mut().cards.get_mut(wuxie).unwrap().kind = CardKind::TrickWuxie;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == LIJIAN)
        .unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        target,
        user,
        observer,
        skill,
        cost,
        wuxie,
    }
}

fn activate(setup: &Setup, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        setup.state.clone(),
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(setup.state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill: setup.skill,
                targets: vec![setup.target, setup.user],
                card_targets: vec![setup.cost],
            },
        },
    )
}

#[test]
fn package_closes_diaochan_and_declares_the_ordered_duel_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let diaochan = package
        .character_index
        .iter()
        .find(|character| character.id == DIAOCHAN)
        .unwrap();
    assert_eq!(diaochan.skills, [LIJIAN, 4]);
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == LIJIAN)
        .unwrap();
    let targets = active.targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max), (2, 2));
    assert!(!targets.allow_self);
    assert!(targets.male);
    assert!(targets.second_can_use_juedou_on_first);
    let compiled = content_compiler::compile_source_named("standard-lijian", SOURCE).unwrap();
    assert!(compiled
        .modules
        .iter()
        .all(|expected| package.modules.iter().any(
            |actual| actual.name == expected.name && actual.logic_hash == expected.logic_hash
        )));
}

#[test]
fn second_target_uses_unwuxieable_duel_on_first_after_the_cost_is_discarded() {
    let setup = setup(false);
    let target_hp = setup.state.players.get(setup.target).unwrap().hp;
    let (state, result) = activate(&setup, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.choice.is_none());
    assert_eq!(state.players.get(setup.target).unwrap().hp, target_hp - 1);
    assert_eq!(
        state.cards.get(setup.cost).unwrap().zone,
        state
            .zones
            .find(ZoneKind::DiscardPile, PlayerId::NONE)
            .unwrap()
    );
    assert!(state.hand_cards(setup.observer).contains(&setup.wuxie));
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardUsed {
            player,
            targets,
            ..
        } if *player == setup.user && targets == &[setup.target]
    )));
}

#[test]
fn an_illegal_second_user_is_rejected_before_the_discard_cost() {
    let setup = setup(true);
    let (same, result) = activate(&setup, 2);
    assert!(matches!(result.result, CommandResult::Rejected { .. }));
    assert_eq!(same, setup.state);
    assert!(same.hand_cards(setup.owner).contains(&setup.cost));
    assert_eq!(same.skills.get(setup.skill).unwrap().use_count, 0);
}
