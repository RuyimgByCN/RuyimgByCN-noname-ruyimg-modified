//! Golden coverage for YIJIANG「追忆」and MOBILE 界步练师.

use engine_api::{
    ChoiceId, ChoiceKind, ChoiceRequest, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const REANXU: u32 = 2_398_589_569;
const ZHUIYI: u32 = 3_427_217_715;
const RE_BULIANSHI: u32 = 3_361_488_418;
const SOURCE: &str = include_str!("../../../../content/original/ported/yijiang/zhuiyi.skill");
const YIJIANG: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");
const MOBILE: &[u8] =
    include_bytes!("../../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    killer: PlayerId,
    recipients: Vec<PlayerId>,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("zhuiyi-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 297);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 5, ZHUIYI as u64, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[1];
    let killer = players[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZHUIYI),
    });
    run_effects(&mut tx, effects).unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        killer,
        recipients: players
            .into_iter()
            .filter(|player| *player != owner && *player != killer)
            .collect(),
    }
}

fn kill(state: GameState, owner: PlayerId, killer: PlayerId) -> (GameState, ChoiceRequest) {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::KillPlayer {
                player: owner,
                killer,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    (committed.state, committed.output.choice.unwrap())
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn compiled_contract_and_mobile_cross_package_character_are_closed() {
    let package = content_compiler::compile_source_named("zhuiyi-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 297);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == ZHUIYI)
        .unwrap();
    assert_eq!(trigger.event, "death");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
    let module = package
        .modules
        .iter()
        .find(|module| module.name == trigger.module)
        .unwrap();
    assert_eq!(module.code, [83, 85, 90, 28, 1, 1]);

    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    let mobile = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    assert_eq!(yijiang.manifest.rule_vm_abi_version, 637);
    assert!(yijiang
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == ZHUIYI));
    let bulianshi = mobile
        .character_index
        .iter()
        .find(|character| character.id == RE_BULIANSHI)
        .unwrap();
    assert_eq!(bulianshi.name_key, "re_bulianshi");
    assert_eq!(bulianshi.sex, "female");
    assert_eq!(bulianshi.group, "wu");
    assert_eq!(bulianshi.max_hp, 3);
    assert_eq!(bulianshi.skills, [REANXU, ZHUIYI]);
    assert!(mobile
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "yijiang"));
    assert!(mobile
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == REANXU));
    assert!(!mobile
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == ZHUIYI));
    assert!(bulianshi.skills.iter().all(|wanted| mobile
        .skill_index
        .iter()
        .chain(yijiang.skill_index.iter())
        .any(|skill| skill.skill_def_id == *wanted)));
}

#[test]
fn death_opens_one_cancellable_target_choice_for_owner_and_excludes_killer() {
    let scenario = setup();
    let (state, choice) = kill(scenario.state, scenario.owner, scenario.killer);
    assert!(!state.players.get(scenario.owner).unwrap().alive);
    assert_eq!(choice.observer, scenario.owner);
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!((choice.min_count, choice.max_count), (1, 1));
    assert!(choice.allow_cancel);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ResolveZhuiyi
    );
    assert_eq!(
        state.resolution.continuation_payload,
        [
            scenario.owner.raw(),
            scenario.killer.raw(),
            state
                .skills
                .sorted_live()
                .into_iter()
                .find(|skill| skill.owner == scenario.owner && skill.def.raw() == ZHUIYI)
                .unwrap()
                .id
                .raw(),
        ]
    );
    let offered: Vec<_> = choice.options.iter().map(|option| option.token).collect();
    assert_eq!(
        offered,
        scenario
            .recipients
            .iter()
            .copied()
            .map(PlayerId::raw)
            .collect::<Vec<_>>()
    );
    assert!(!offered.contains(&scenario.owner.raw()));
    assert!(!offered.contains(&scenario.killer.raw()));

    let before: Vec<_> = scenario
        .recipients
        .iter()
        .map(|player| {
            (
                *player,
                state.players.get(*player).unwrap().hp,
                state.hand_count(*player),
            )
        })
        .collect();
    let (state, cancelled) = resolve(state, scenario.owner, Vec::new(), true, 1);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    for (player, hp, hand) in before {
        assert_eq!(state.players.get(player).unwrap().hp, hp);
        assert_eq!(state.hand_count(player), hand);
    }
    assert!(!state.has_pending_choice());

    let no_source = setup();
    let (state, choice) = kill(no_source.state, no_source.owner, PlayerId::NONE);
    assert_eq!(choice.observer, no_source.owner);
    let offered: Vec<_> = choice.options.iter().map(|option| option.token).collect();
    assert_eq!(offered.len(), 4);
    assert!(!offered.contains(&no_source.owner.raw()));
    assert!(offered.contains(&no_source.killer.raw()));
    let (state, cancelled) = resolve(state, no_source.owner, Vec::new(), true, 2);
    assert!(matches!(cancelled.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
}

#[test]
fn injured_target_recovers_before_drawing_and_full_target_still_draws_three() {
    let mut injured = setup();
    let target = injured.recipients[0];
    injured.state.players.get_mut(target).unwrap().hp = 2;
    let (state, _) = kill(injured.state, injured.owner, injured.killer);
    let (state, pipeline) = resolve(state, injured.owner, vec![target.raw()], false, 3);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    assert_eq!(state.hand_count(target), 3);
    let recovered = pipeline
        .events
        .iter()
        .position(|event| {
            matches!(
                event,
                DomainEvent::HpRecovered {
                    source,
                    player,
                    amount: 1,
                } if *source == PlayerId::NONE && *player == target
            )
        })
        .unwrap();
    let drawn = pipeline
        .events
        .iter()
        .position(|event| {
            matches!(
                event,
                DomainEvent::CardsDrawn { player, cards }
                    if *player == target && cards.len() == 3
            )
        })
        .unwrap();
    assert!(recovered < drawn);

    let full = setup();
    let target = full.recipients[0];
    let max_hp = full.state.players.get(target).unwrap().max_hp;
    assert_eq!(full.state.players.get(target).unwrap().hp, max_hp as i32);
    let (state, _) = kill(full.state, full.owner, full.killer);
    let (state, pipeline) = resolve(state, full.owner, vec![target.raw()], false, 4);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, max_hp as i32);
    assert_eq!(state.hand_count(target), 3);
    assert!(!pipeline.events.iter().any(
        |event| matches!(event, DomainEvent::HpRecovered { player, .. } if *player == target)
    ));
    assert!(pipeline.events.iter().any(|event| {
        matches!(
            event,
            DomainEvent::CardsDrawn { player, cards }
                if *player == target && cards.len() == 3
        )
    }));
}
