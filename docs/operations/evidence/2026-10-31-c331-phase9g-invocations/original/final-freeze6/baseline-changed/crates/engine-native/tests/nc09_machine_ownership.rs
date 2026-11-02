//! NC09 review regression: every existing parked-card window must project and restore.
#[path = "support/nc09_machine_windows.rs"]
mod windows;

use engine_native::snap::{machine_held_cards, MachineSnapshot, SnapshotError, StateSnapshot};
use engine_native::GameAction;
use windows::{fixture, Window};

fn projection(window: Window) {
    let fixture = fixture(window);
    let engine = &fixture.engine;
    let before = engine.save();
    let view = engine.view_referee();
    let mut actual = view.visible_card_ids();
    actual.sort_unstable();
    assert_eq!(
        actual, fixture.physical_ids,
        "{window:?}: exact physical multiset"
    );
    for id in &fixture.held {
        let cards: Vec<_> = view.cards.iter().filter(|card| card.id == *id).collect();
        assert_eq!(cards.len(), 1, "{window:?}: held card {id} exactly once");
        assert_eq!(
            cards[0].zone,
            engine_native::snap::PhysicalCardZone::Processing
        );
        assert_eq!(cards[0].owner, None);
    }
    for seat in 0..engine.state_core().players.len() as u8 {
        let ordinary = engine.view_for(seat);
        assert!(ordinary
            .visible_card_ids()
            .iter()
            .all(|id| !fixture.held.contains(id)));
        assert_eq!(engine.ai_session(seat).view(), ordinary);
    }
    assert!(engine
        .view_spectator()
        .visible_card_ids()
        .iter()
        .all(|id| !fixture.held.contains(id)));
    assert_eq!(
        engine.save(),
        before,
        "projection must not change authority/RNG"
    );
}

fn trace_frame(engine: &engine_native::Engine) -> serde_json::Value {
    let choice = engine.open_choice().map(|open| {
        serde_json::json!({
            "id": open.id, "observer": open.observer, "prompt": open.spec.prompt,
            "native_kind": engine_native::view::choice_kind_name(open.spec.kind),
            "kind": match open.spec.kind {
                engine_native::ChoiceKind::Confirm => "Confirm",
                engine_native::ChoiceKind::SelectCards => "SelectCards",
                engine_native::ChoiceKind::SelectPlayers => "SelectTargets",
                engine_native::ChoiceKind::RespondCard | engine_native::ChoiceKind::PickTag => "Custom",
            },
            "min": open.spec.min, "max": open.spec.max, "allowCancel": open.spec.allow_cancel,
            "options": open.spec.options.iter().map(|option| match option {
                engine_native::ChoiceOption::Confirm => serde_json::json!("confirm"),
                engine_native::ChoiceOption::Pass => serde_json::json!("pass"),
                engine_native::ChoiceOption::Card(id) => serde_json::json!({"card": id}),
                engine_native::ChoiceOption::Player(id) => serde_json::json!({"player": id}),
                engine_native::ChoiceOption::Tag(id) => serde_json::json!({"tag": id}),
                engine_native::ChoiceOption::Skill(id) => serde_json::json!({"skill": engine.catalog().skill_name(*id)}),
                engine_native::ChoiceOption::Kind(id) => serde_json::json!({"kind": id}),
            }).collect::<Vec<_>>(),
        })
    });
    serde_json::json!({
        "hash": engine.state_hash().iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
        "revision": engine.revision(), "choice": choice, "deckCount": engine.state_core().deck.len(),
        "visible": (0..engine.state_core().players.len() as u8).map(|seat| {
            let mut ids = engine.view_for(seat).visible_card_ids(); ids.sort_unstable(); ids
        }).collect::<Vec<_>>(),
    })
}

fn archive_resume(window: Window) {
    let mut fixture = fixture(window);
    let before = fixture.engine.save();
    let bytes = engine_native::archive::encode_archive(&before);
    let decoded = engine_native::archive::decode_archive(&bytes).unwrap();
    assert_eq!(decoded, before);
    let mut restored = fixture
        .restore(&decoded)
        .unwrap_or_else(|error| panic!("{window:?}: actual pending archive: {error:?}"));
    let open = fixture.engine.open_choice().unwrap();
    let expected = engine_native::snap::ask(
        open.observer,
        open.spec.prompt,
        open.spec.kind,
        open.spec.options.clone(),
        open.spec.min,
        open.spec.max,
        open.spec.allow_cancel,
    );
    assert_eq!(
        engine_native::snap::active_ask(
            &before.resolution.parks.last().unwrap().machine,
            &before.core,
            fixture.engine.catalog()
        ),
        Some(expected)
    );
    assert_eq!(restored.view_referee(), fixture.engine.view_referee());
    assert_eq!(restored.open_choice(), fixture.engine.open_choice());
    let mut frames = vec![trace_frame(&fixture.engine)];
    let mut commands = Vec::new();
    for step in 0..7 {
        let action = if step == 0 {
            GameAction::Answer {
                choice_id: fixture.engine.open_choice().unwrap().id,
                selection: fixture.selection.clone(),
            }
        } else if let Some(open) = fixture.engine.open_choice() {
            GameAction::Answer {
                choice_id: open.id,
                selection: windows::default_selection(&fixture.engine),
            }
        } else if fixture.engine.outcome().is_none() {
            GameAction::NextPhase
        } else {
            break;
        };
        let actor = fixture
            .engine
            .open_choice()
            .map_or(fixture.engine.turn().current, |c| c.observer);
        let command = match &action {
            GameAction::Answer {
                choice_id,
                selection,
            } => serde_json::json!({
                "kind": "answer", "actor": actor, "id": choice_id,
                "indices": selection.iter().map(|option| fixture.engine.open_choice().unwrap().spec.options.iter().position(|item| item == option).unwrap()).collect::<Vec<_>>()
            }),
            GameAction::NextPhase => serde_json::json!({"kind": "next", "actor": actor}),
            _ => unreachable!(),
        };
        let left_log = fixture.engine.log().len();
        let right_log = restored.log().len();
        windows::submit(&mut fixture.engine, action.clone());
        windows::submit(&mut restored, action);
        assert_eq!(
            fixture.engine.state_hash(),
            restored.state_hash(),
            "{window:?}: step {step}"
        );
        assert_eq!(fixture.engine.state_core(), restored.state_core());
        assert_eq!(fixture.engine.state_core().rng, restored.state_core().rng);
        assert_eq!(fixture.engine.save().resolution, restored.save().resolution);
        assert_eq!(fixture.engine.open_choice(), restored.open_choice());
        assert_eq!(
            &fixture.engine.log()[left_log..],
            &restored.log()[right_log..],
            "{window:?}: events at {step}"
        );
        commands.push(serde_json::json!({"command": command, "codes": fixture.engine.public_event_codes_from(left_log)}));
        frames.push(trace_frame(&fixture.engine));
        let bytes = engine_native::archive::encode_archive(&restored.save());
        let decoded = engine_native::archive::decode_archive(&bytes).unwrap();
        assert_eq!(decoded, restored.save());
        restored = fixture
            .restore(&decoded)
            .unwrap_or_else(|error| panic!("{window:?}: resumed checkpoint {step}: {error:?}"));
    }
    if let Some(directory) = std::env::var_os("NC09_MACHINE_TRACE_OUT") {
        let directory = std::path::PathBuf::from(directory);
        std::fs::create_dir_all(&directory).unwrap();
        let value = serde_json::json!({
            "window": format!("{window:?}"), "held": fixture.held,
            "archiveHex": bytes.iter().map(|byte| format!("{byte:02x}")).collect::<String>(),
            "frames": frames, "commands": commands,
        });
        std::fs::write(
            directory.join(format!("{window:?}.json")),
            serde_json::to_vec_pretty(&value).unwrap(),
        )
        .unwrap();
    }
}

#[test]
fn xunxun_projects_four_held_cards() {
    projection(Window::XunXun);
}
#[test]
fn jijie_projects_bottom_card() {
    projection(Window::JiJie);
}
#[test]
fn gaiming_projects_seven_candidates_once() {
    projection(Window::GaiMing);
}
#[test]
fn haotianta_projects_both_candidates() {
    projection(Window::HaoTianTa);
}
#[test]
fn xunxun_archive_restores_and_resumes() {
    archive_resume(Window::XunXun);
}
#[test]
fn jijie_archive_restores_and_resumes() {
    archive_resume(Window::JiJie);
}
#[test]
fn gaiming_archive_restores_and_resumes() {
    archive_resume(Window::GaiMing);
}
#[test]
fn haotianta_archive_restores_and_resumes() {
    archive_resume(Window::HaoTianTa);
}

fn window_machine(machine: &mut MachineSnapshot) -> &mut MachineSnapshot {
    match machine {
        MachineSnapshot::OptionalGate {
            inner: Some(inner), ..
        }
        | MachineSnapshot::NullifyWindow {
            effect: Some(inner),
            ..
        }
        | MachineSnapshot::Aoe {
            inner: Some(inner), ..
        }
        | MachineSnapshot::Jijiang {
            inner: Some(inner), ..
        }
        | MachineSnapshot::LiJian {
            inner: Some(inner), ..
        } => window_machine(inner),
        _ => machine,
    }
}

fn change_held(snapshot: &mut StateSnapshot, window: Window, replacement: Option<u32>) {
    let machine = window_machine(&mut snapshot.resolution.parks.last_mut().unwrap().machine);
    match (window, machine) {
        (Window::XunXun, MachineSnapshot::XunXun { top, .. }) => match replacement {
            Some(id) => top[0] = id,
            None => {
                top.remove(0);
            }
        },
        (Window::JiJie, MachineSnapshot::JiJie { card, .. }) => *card = replacement,
        (
            Window::GaiMing,
            MachineSnapshot::JudgePhase {
                current: Some(current),
                ..
            },
        ) => {
            let cards = current.gaiming_cards.as_mut().unwrap();
            match replacement {
                Some(id) => cards[1] = id,
                None => {
                    cards.remove(1);
                }
            }
        }
        (
            Window::HaoTianTa,
            MachineSnapshot::JudgePhase {
                current: Some(current),
                ..
            },
        ) => current.haotian_other = replacement,
        _ => panic!("wrong real window"),
    }
}

fn restore_bytes(
    fixture: &windows::Fixture,
    snapshot: &StateSnapshot,
) -> Result<engine_native::Engine, SnapshotError> {
    let bytes = engine_native::archive::encode_archive(snapshot);
    let decoded = engine_native::archive::decode_archive(&bytes).unwrap();
    assert_eq!(&decoded, snapshot);
    fixture.restore(&decoded)
}

#[test]
fn machine_ownership_still_rejects_duplicate_missing_and_out_of_range_cards() {
    for window in [
        Window::XunXun,
        Window::JiJie,
        Window::GaiMing,
        Window::HaoTianTa,
    ] {
        let fixture = fixture(window);
        let original = fixture.engine.save();
        let mut duplicate = original.clone();
        duplicate.core.deck.push(fixture.held[0]);
        assert!(
            matches!(
                restore_bytes(&fixture, &duplicate),
                Err(SnapshotError::IdOutOfBounds {
                    what: "card_in_multiple_zones",
                    ..
                })
            ),
            "{window:?}: duplicate"
        );
        let mut missing = original.clone();
        change_held(&mut missing, window, None);
        assert!(
            matches!(
                restore_bytes(&fixture, &missing),
                Err(SnapshotError::IdOutOfBounds {
                    what: "card_missing_from_all_zones",
                    ..
                })
            ),
            "{window:?}: missing"
        );
        let mut outside = original.clone();
        change_held(
            &mut outside,
            window,
            Some(fixture.engine.catalog().cards.len() as u32),
        );
        assert!(
            matches!(
                restore_bytes(&fixture, &outside),
                Err(SnapshotError::IdOutOfBounds { what: "card", .. })
            ),
            "{window:?}: out of range"
        );
        if matches!(window, Window::GaiMing | Window::HaoTianTa) {
            let mut alias = original.clone();
            change_held(&mut alias, window, Some(fixture.held[0]));
            assert!(
                matches!(
                    restore_bytes(&fixture, &alias),
                    Err(SnapshotError::IdOutOfBounds {
                        what: "card_in_multiple_zones",
                        ..
                    })
                ),
                "{window:?}: only GaiMing judge_card/first-candidate alias is legitimate"
            );
        }
    }
}

#[test]
fn inactive_storage_and_finalized_judge_references_are_not_owned() {
    for window in [
        Window::XunXun,
        Window::JiJie,
        Window::GaiMing,
        Window::HaoTianTa,
    ] {
        let fixture = fixture(window);
        let mut machine = fixture
            .engine
            .save()
            .resolution
            .parks
            .last()
            .unwrap()
            .machine
            .clone();
        let leaf = window_machine(&mut machine);
        match leaf {
            MachineSnapshot::XunXun { stage, .. } => {
                *stage = engine_native::snap::XunXunStageSnap::Undo
            }
            MachineSnapshot::JiJie { stage, .. } => {
                *stage = engine_native::snap::JiJieStageSnap::Ask
            }
            MachineSnapshot::JudgePhase {
                current: Some(current),
                ..
            } => {
                current.finalized = true;
                current.judge_held = false;
            }
            _ => panic!("window"),
        }
        let mut ids = Vec::new();
        machine_held_cards(&machine, &mut ids);
        assert!(
            ids.is_empty(),
            "{window:?}: inactive references are not physical ownership: {ids:?}"
        );
    }
}

#[test]
fn parked_card_collection_and_expected_asks_recurse_through_all_existing_wrappers() {
    use engine_native::snap::{active_ask, JijiangStageSnap, LiJianStageSnap};
    for window in [
        Window::XunXun,
        Window::JiJie,
        Window::GaiMing,
        Window::HaoTianTa,
    ] {
        let fixture = fixture(window);
        let mut root = fixture
            .engine
            .save()
            .resolution
            .parks
            .last()
            .unwrap()
            .machine
            .clone();
        let mut machine = window_machine(&mut root).clone();
        let mut expected = fixture.held.clone();
        if let MachineSnapshot::JudgePhase {
            current: Some(current),
            queue,
            ..
        } = &machine
        {
            expected.extend(queue);
            expected.push(current.trick);
        }
        expected.sort_unstable();
        let open = fixture.engine.open_choice().unwrap();
        let ask = engine_native::snap::ask(
            open.observer,
            open.spec.prompt,
            open.spec.kind,
            open.spec.options.clone(),
            open.spec.min,
            open.spec.max,
            open.spec.allow_cancel,
        );
        for wrapper in 0..5 {
            machine = match wrapper {
                0 => MachineSnapshot::OptionalGate {
                    owner: 0,
                    def: 0,
                    event: engine_native::event::EventData::Phase {
                        player: 0,
                        phase: engine_native::Phase::Draw,
                        begin: true,
                    },
                    asked: true,
                    inner: Some(Box::new(machine)),
                },
                1 => MachineSnapshot::NullifyWindow {
                    card: 0,
                    source: 0,
                    nullified: false,
                    queue: vec![],
                    asked: false,
                    window_open: false,
                    owns_disposal: false,
                    effect: Some(Box::new(machine)),
                },
                2 => MachineSnapshot::Aoe {
                    card: 0,
                    source: 0,
                    need: engine_native::CardKind::Sha,
                    targets: vec![],
                    idx: 0,
                    nature: engine_native::event::DamageNature::Normal,
                    inner: Some(Box::new(machine)),
                },
                3 => MachineSnapshot::Jijiang {
                    owner: 0,
                    stage: JijiangStageSnap::HelperAsk,
                    queue: None,
                    helper: None,
                    sha: None,
                    inner: Some(Box::new(machine)),
                },
                4 => MachineSnapshot::LiJian {
                    owner: 0,
                    stage: LiJianStageSnap::Launch,
                    card: None,
                    inner: Some(Box::new(machine)),
                },
                _ => unreachable!(),
            };
            let mut actual = Vec::new();
            machine_held_cards(&machine, &mut actual);
            actual.sort_unstable();
            assert_eq!(actual, expected, "{window:?}: nested wrapper {wrapper}");
            assert_eq!(
                active_ask(
                    &machine,
                    fixture.engine.state_core(),
                    fixture.engine.catalog()
                ),
                Some(ask.clone())
            );
        }
    }
}

#[test]
fn archive_encoding_rejects_unknown_prompts_without_changing_existing_layout() {
    for window in [
        Window::XunXun,
        Window::JiJie,
        Window::GaiMing,
        Window::HaoTianTa,
        Window::Liaoyuan,
    ] {
        let fixture = fixture(window);
        let original = fixture.engine.save();
        let bytes = engine_native::archive::encode_archive(&original);
        assert_eq!(u16::from_le_bytes([bytes[4], bytes[5]]), 106);
        if let Some(directory) = std::env::var_os("NC09_MACHINE_ARCHIVE_OUT") {
            let directory = std::path::PathBuf::from(directory);
            std::fs::create_dir_all(&directory).unwrap();
            std::fs::write(directory.join(format!("{window:?}.bin")), &bytes).unwrap();
        }
        let mut unknown = original;
        unknown.resolution.open_choice.as_mut().unwrap().spec.prompt =
            "nc09_unregistered_test_prompt";
        let bytes = engine_native::archive::encode_archive(&unknown);
        assert!(matches!(
            engine_native::archive::decode_archive(&bytes),
            Err(engine_native::archive::ArchiveError::BadTag(
                "prompt_unknown",
                0
            ))
        ));
    }
}

#[test]
fn gaiming_only_first_candidate_aliases_judge_card() {
    let fixture = fixture(Window::GaiMing);
    let mut snapshot = fixture.engine.save();
    let MachineSnapshot::JudgePhase {
        current: Some(current),
        ..
    } = window_machine(&mut snapshot.resolution.parks.last_mut().unwrap().machine)
    else {
        panic!("judge")
    };
    current.judge_card = current.gaiming_cards.as_ref().unwrap()[1];
    assert!(matches!(
        restore_bytes(&fixture, &snapshot),
        Err(SnapshotError::IdOutOfBounds {
            what: "card_in_multiple_zones",
            ..
        })
    ));
}

#[test]
fn pending_choice_metadata_must_match_the_machine_after_byte_decode() {
    for window in [
        Window::XunXun,
        Window::JiJie,
        Window::GaiMing,
        Window::HaoTianTa,
    ] {
        let fixture = fixture(window);
        for mutation in 0..6 {
            let mut snapshot = fixture.engine.save();
            let open = snapshot.resolution.open_choice.as_mut().unwrap();
            match mutation {
                0 => open.spec.options.reverse(),
                1 => open.spec.min += 1,
                2 => open.spec.max += 1,
                3 => open.spec.allow_cancel = !open.spec.allow_cancel,
                4 => open.spec.kind = engine_native::ChoiceKind::Confirm,
                5 => open.observer = (open.observer + 1) % snapshot.core.players.len() as u8,
                _ => unreachable!(),
            }
            assert!(
                matches!(
                    restore_bytes(&fixture, &snapshot),
                    Err(SnapshotError::OpenChoiceMismatch { .. })
                ),
                "{window:?}: metadata mutation {mutation}"
            );
        }
    }
}

fn liaoyuan_fields(snapshot: &StateSnapshot) -> (u8, Option<u32>, Vec<u32>) {
    let mut root = snapshot.resolution.parks.last().unwrap().machine.clone();
    let MachineSnapshot::Host {
        behavior,
        stage,
        card,
        cards,
        ..
    } = window_machine(&mut root)
    else {
        panic!("LiaoYuan host")
    };
    assert_eq!(
        *behavior,
        engine_native::content::host::HostBehavior::Liaoyuan.tag()
    );
    (*stage, *card, cards.clone())
}

#[test]
fn liaoyuan_real_suit_scalar_snapshot_derives_its_ask_and_restores() {
    let fixture = fixture(Window::Liaoyuan);
    let before = fixture.engine.save();
    let (stage, suit, history) = liaoyuan_fields(&before);
    assert_eq!(
        stage,
        engine_native::content::host::HostStage::SelectCard.tag()
    );
    assert!(suit.is_some_and(|suit| suit <= 3));
    assert!(history.is_empty());
    let restored = fixture.restore(&before).unwrap();
    assert_eq!(restored.state_core(), fixture.engine.state_core());
    assert_eq!(restored.save().resolution, before.resolution);
    let open = fixture.engine.open_choice().unwrap();
    let expected: Vec<_> = before
        .core
        .player(0)
        .hand
        .iter()
        .chain(&before.core.player(0).equips)
        .filter(|id| fixture.engine.catalog().cards[**id as usize].suit as u32 == suit.unwrap())
        .copied()
        .map(engine_native::ChoiceOption::Card)
        .collect();
    assert_eq!(open.spec.options, expected);
    assert_eq!(
        (open.spec.min, open.spec.max, open.spec.allow_cancel),
        (1, 1, true)
    );
}

#[test]
fn liaoyuan_byte_archive_restores_and_resumes() {
    archive_resume(Window::Liaoyuan);
}

#[test]
fn liaoyuan_privacy_candidates_and_equipment_history_survive_byte_restores() {
    let fixture = fixture(Window::Liaoyuan);
    for privacy in [false, true] {
        let mut original = fixture.engine.clone();
        let before = original.save();
        if privacy {
            engine_native::ai::redact_opponent_hands(&mut original, 0);
            assert_eq!(original.save().resolution, before.resolution);
            assert_eq!(original.state_core().rng, before.core.rng);
        }
        let scalar = liaoyuan_fields(&original.save()).1;
        let equip = original.state_core().player(0).equips[0];
        let mut restored = restore_bytes(&fixture, &original.save()).unwrap();
        for selection in [vec![engine_native::ChoiceOption::Card(equip)], vec![]] {
            assert_eq!(
                original.open_choice().unwrap().spec.prompt,
                "liaoyuan_discard"
            );
            let left_log = original.log().len();
            let right_log = restored.log().len();
            windows::answer(&mut original, selection.clone());
            windows::answer(&mut restored, selection);
            assert_eq!(original.state_core(), restored.state_core());
            assert_eq!(original.state_hash(), restored.state_hash());
            assert_eq!(original.save().resolution, restored.save().resolution);
            assert_eq!(&original.log()[left_log..], &restored.log()[right_log..]);
            if original
                .open_choice()
                .is_some_and(|choice| choice.spec.prompt == "liaoyuan_discard")
            {
                let (_, after_scalar, history) = liaoyuan_fields(&original.save());
                assert_eq!(after_scalar, scalar, "Host.card is the same suit scalar");
                assert_eq!(
                    history,
                    vec![equip],
                    "Host.cards references the already discarded equipment"
                );
                assert!(original.state_core().discard.contains(&equip));
                let mut physical = original.view_referee().visible_card_ids();
                physical.sort_unstable();
                assert_eq!(
                    physical, fixture.physical_ids,
                    "history does not own a second copy"
                );
            }
            restored = restore_bytes(&fixture, &restored.save()).unwrap();
        }
    }
}

#[test]
fn liaoyuan_invalid_scalar_stage_and_candidate_still_reject() {
    let fixture = fixture(Window::Liaoyuan);
    for invalid in 0..5 {
        let mut snapshot = fixture.engine.save();
        let MachineSnapshot::Host { card, stage, .. } =
            window_machine(&mut snapshot.resolution.parks.last_mut().unwrap().machine)
        else {
            panic!("host")
        };
        match invalid {
            0 => *card = None,
            1 => *card = Some(4),
            2 => *card = Some(256),
            3 => *stage = engine_native::content::host::HostStage::Act.tag(),
            4 => {
                snapshot
                    .resolution
                    .open_choice
                    .as_mut()
                    .unwrap()
                    .spec
                    .options = vec![engine_native::ChoiceOption::Card(
                    snapshot.core.player(1).hand[0],
                )]
            }
            _ => unreachable!(),
        }
        let result = restore_bytes(&fixture, &snapshot);
        if invalid == 2 {
            assert!(
                matches!(
                    result,
                    Err(SnapshotError::IdOutOfBounds {
                        what: "card",
                        got: 256
                    })
                ),
                "existing host payload bounds must reject 256: {:?}",
                result.as_ref().err()
            );
        } else {
            assert!(
                matches!(result, Err(SnapshotError::OpenChoiceMismatch { .. })),
                "LiaoYuan invalid scalar/stage/candidate {invalid}: {:?}",
                result.as_ref().err()
            );
        }
    }
}
