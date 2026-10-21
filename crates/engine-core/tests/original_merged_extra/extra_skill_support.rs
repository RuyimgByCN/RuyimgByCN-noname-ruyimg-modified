use super::*;

pub(super) fn cancel_startup_choices(engine: &mut Engine) {
    let mut key = 90_000;
    while engine.state().resolution.pending_choice_id != 0 {
        let state = engine.state();
        let actor = PlayerId::new(state.resolution.pending_observer);
        let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        assert!(matches!(
            engine.submit(Command {
                actor,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: Vec::new(),
                    cancelled: true,
                },
            }),
            CommandResult::Accepted { .. }
        ));
        key += 1;
    }
}

/// Concatenate every committed extra source in pack order (alphabetical),
/// mirroring `content-cli pack` input concatenation so the logic hash matches
/// the committed npack regardless of future roster growth.
pub(super) fn extra_sources() -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/ported/extra");
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "skill"))
        .map(|entry| entry.path())
        .collect();
    files.sort();
    files
        .iter()
        .map(|path| {
            let mut source = std::fs::read_to_string(path).unwrap();
            source.push('\n');
            source
        })
        .collect()
}

pub(super) fn new_wuhun_death_result(judgment_kind: CardKind) -> bool {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(judgment_kind.tag() as u64),
        kind: CommandKind::StartGame {
            player_count: 4,
            rng_seed: 20_260_726,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&package).unwrap();
    let players = engine.state().players.alive_by_seat();
    let (owner, source, tied) = (players[1], players[2], players[3]);
    let mut tx = Transaction::begin(engine.state().clone());
    for (player, identity) in players.into_iter().zip([
        Identity::Lord,
        Identity::Loyalist,
        Identity::Rebel,
        Identity::Renegade,
    ]) {
        tx.state_mut().players.get_mut(player).unwrap().identity = identity;
    }
    let remove_existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, remove_existing).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NEW_WUHUN),
        }],
    )
    .unwrap();

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let source_marks = tx
        .state()
        .players
        .get(source)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == NEW_WUHUN)
        .map_or(0, |(_, count)| *count);
    assert_eq!(source_marks, 2);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: tied,
            mark: MarkDefId::new(NEW_WUHUN),
            delta: 2,
        }],
    )
    .unwrap();
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top = *tx.state().zones.get(draw).unwrap().cards.last().unwrap();
    tx.state_mut().cards.get_mut(top).unwrap().kind = judgment_kind;

    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::KillPlayer {
                player: owner,
                killer: PlayerId::NONE,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseMaxMarkOtherJudgeKillUnlessTaoOrTaoyuan
    );
    let committed = tx.commit();
    let choice = committed.output.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [source.raw(), tied.raw()]
    );
    let (state, result) = engine_core::command::apply_command(
        committed.state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(committed.state.revision),
            idempotency_key: IdempotencyKey(NEW_WUHUN as u64 + judgment_kind.tag() as u64),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![source.raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{result:?}"
    );
    state.players.get(source).unwrap().alive
}

pub(super) fn extra_active_state(
    skill_def: u32,
    marks: u32,
    player_count: u8,
) -> (engine_core::state::GameState, PlayerId, SkillId) {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD_PACKAGE).unwrap();
    let extra = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&standard).unwrap();
    engine.load_package(&extra).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count,
                rng_seed: skill_def as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    cancel_startup_choices(&mut engine);
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let existing: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, existing).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(skill_def),
            },
            EffectRequest::AddMark {
                player: owner,
                mark: MarkDefId::new(BAONU_MARK),
                delta: marks as i32,
            },
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Play.to_u8(),
            },
        ],
    )
    .unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == skill_def)
        .unwrap();
    (tx.state().clone(), owner, skill)
}

pub(super) fn activate_extra(
    state: engine_core::state::GameState,
    owner: PlayerId,
    skill: SkillId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    let revision = state.revision;
    engine_core::command::apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets,
                card_targets: Vec::new(),
            },
        },
    )
}
