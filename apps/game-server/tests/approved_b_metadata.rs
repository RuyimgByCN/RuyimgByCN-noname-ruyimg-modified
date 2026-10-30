//! Approved B: metadata must survive the real authorized bridge and archive restore.
#[path = "../src/native_view.rs"]
mod native_view;
use engine_api::PlayerId;
use engine_native::{Engine, GameAction, Phase};
use native_view::{choice_for, observer_snapshot_for, RoomRole};
use serde_json::{json, Value};

fn value(engine: &Engine, role: RoomRole, seat: u32) -> Value {
    serde_json::to_value(observer_snapshot_for(engine, role, Some(PlayerId::new(seat)))).unwrap()
}

#[test]
fn both_slots_include_def_zero_only_when_authorized() {
    let engine = Engine::new_identity(1515, 4);
    let mut archive = engine.save();
    archive.core.players[1].general_slots = [Some(0), Some(1)];
    archive.core.players[1].revealed_generals = [false, false];
    let ordinary_mode = archive.core.mode.clone();
    for kingdoms in [false, true] {
        archive.core.mode = if kingdoms { engine_native::GameMode::Kingdoms } else { ordinary_mode.clone() };
        let bytes = engine_native::archive::encode_archive(&archive);
        let engine = Engine::restore(&engine_native::archive::decode_archive(&bytes).unwrap()).unwrap();
        let before = engine.save();
        for (role, seat, visible) in [(RoomRole::Player, 2, true), (RoomRole::Player, 1, !kingdoms), (RoomRole::Referee, 0, true), (RoomRole::Spectator, 2, !kingdoms)] {
            let view = value(&engine, role, seat);
            let slots = &view["seats"][1]["generals"];
            assert!(slots.is_array(), "two slots missing: {slots}");
            assert_eq!(slots.as_array().unwrap().len(), 2);
            for slot in 0..2 {
                if visible {
                    let meta = engine.catalog().general_meta(slot as u16).unwrap();
                    assert_eq!(slots[slot], json!({"def": slot, "name": meta.0, "faction": format!("{:?}", meta.1).to_lowercase(), "revealed": false}));
                } else { assert_eq!(slots[slot], Value::Null); }
            }
        }
        assert_eq!(engine.save(), before);
    }
}

fn check_choice(engine: &Engine, native: &str, legacy: &str) {
    let open = engine.open_choice().unwrap();
    let expected = serde_json::to_value(choice_for(engine, RoomRole::Referee, None).unwrap()).unwrap();
    assert_eq!(expected["native_kind"], native);
    assert_eq!(expected["kind"], legacy);
    assert_eq!(expected["prompt"], open.spec.prompt);
    for seat in [0, 1, 257, u32::MAX] {
        assert!(choice_for(engine, RoomRole::Spectator, Some(PlayerId::new(seat))).is_none());
    }
    assert_eq!(serde_json::to_value(choice_for(engine, RoomRole::Player, Some(PlayerId::new(open.observer as u32 + 1))).unwrap()).unwrap(), expected);
    let mut session = engine_native_wasm::NativeSession::new(1, 0).unwrap();
    let bytes = engine_native::archive::encode_archive(&engine.save());
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    session.restore_hex(&hex).unwrap_or_else(|error| panic!("{}: {error}", open.spec.prompt));
    let restored_hex = session.save_hex();
    let local: Value = serde_json::from_str(&session.open_choice_json(open.observer).unwrap()).unwrap();
    assert_eq!(local["native_kind"], native);
    assert_eq!(local["kind"], legacy);
    assert_eq!(local["prompt"], expected["prompt"]);
    assert_eq!(session.save_hex(), restored_hex, "metadata projection must not change archive bytes");
    let decoded = engine_native::archive::decode_archive(&bytes).unwrap();
    let mut restored = Engine::restore(&decoded).unwrap();
    let mut direct = Engine::restore(&engine.save()).unwrap();
    let selection = open.spec.options.iter().take(open.spec.min).copied().collect::<Vec<_>>();
    for candidate in [&mut restored, &mut direct] { submit(candidate, GameAction::Answer { choice_id: open.id, selection: selection.clone() }); }
    assert_eq!(restored.save(), direct.save());
    assert_eq!(restored.log(), direct.log());
    let mut unknown = engine.save();
    unknown.resolution.open_choice.as_mut().unwrap().spec.prompt = "unknown_extension_prompt";
    assert!(engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&unknown)).is_err());
}

#[test]
fn real_choices_keep_precise_kind_and_prompt_through_archive() {
    let mut engine = Engine::new(804, 3);
    engine.debug_grant(0, "突袭");
    engine.state_core_mut().turn.phase = Phase::Judge;
    submit(&mut engine, GameAction::NextPhase);
    check_choice(&engine, "confirm", "Confirm");
    let id = engine.open_choice().unwrap().id;
    submit(&mut engine, GameAction::Answer { choice_id: id, selection: vec![engine_native::ChoiceOption::Confirm] });
    check_choice(&engine, "select_players", "SelectTargets");
    let mut discard = Engine::new(1515, 4);
    for _ in 0..3 { submit(&mut discard, GameAction::NextPhase); }
    check_choice(&discard, "select_cards", "SelectCards");
    let mut response = Engine::new(2, 2);
    response.debug_set_turn(0, Phase::Play, 1);
    let sha = response.debug_place(0, "sha");
    response.debug_place(1, "shan");
    submit(&mut response, GameAction::PlayCard { card: sha, targets: vec![1] });
    check_choice(&response, "respond_card", "Custom");
    let mut pick = Engine::new(42, 3);
    pick.debug_set_turn(0, Phase::Play, 1);
    pick.debug_grant(0, "反间");
    let skill = pick.catalog().skill_by_name("反间").unwrap();
    submit(&mut pick, GameAction::ActivateSkill { skill });
    let id = pick.open_choice().unwrap().id;
    submit(&mut pick, GameAction::Answer { choice_id: id, selection: vec![engine_native::ChoiceOption::Player(1)] });
    check_choice(&pick, "pick_tag", "Custom");
}

fn submit(engine: &mut Engine, action: GameAction) {
    let result = engine.submit(engine_native::GameCommand { actor: engine.open_choice().map_or(engine.turn().current, |c| c.observer), expected_revision: engine.revision(), dedup_id: engine.revision() + 1, action });
    assert!(!matches!(result, engine_native::CommandOutcome::Rejected(_)), "{result:?}");
}

#[path = "support/ws5_metadata_readers.rs"]
mod old;
#[test]
fn ws5_old_and_new_rust_readers_accept_both_payload_generations() {
    let engine = Engine::new(1, 3);
    let mut payload = value(&engine, RoomRole::Player, 1)["seats"][0].clone();
    let old: old::OldSeatView = serde_json::from_value(payload.clone()).unwrap();
    let mut previous = payload.clone(); previous.as_object_mut().unwrap().remove("generals");
    let current: engine_api::SeatView = serde_json::from_value(previous).unwrap();
    assert_eq!(serde_json::to_value(&current).unwrap()["generals"], json!([null,null]));
    payload["generals"] = json!([{ "def":0,"name":"extension","faction":"unknown","revealed":false },null]);
    assert_eq!(serde_json::from_value::<old::OldSeatView>(payload.clone()).unwrap(), old);
    for invalid in [json!(null),json!([]),json!([null]),json!([{"def":"0","name":"x","faction":"wei","revealed":false},null])] {
        payload["generals"] = invalid;
        assert!(serde_json::from_value::<engine_api::SeatView>(payload.clone()).is_err());
    }
    let previous = json!({"choice_id":1,"observer":1,"options":[],"min_count":0,"max_count":1,"allow_cancel":true,"timeout_policy":"None","revision":7,"continuation_id":1,"kind":"Custom"});
    let current: engine_api::ChoiceRequest = serde_json::from_value(previous.clone()).unwrap();
    assert_eq!(serde_json::to_value(current).unwrap(), previous);
    let mut payload = previous.clone(); payload["prompt"]=json!("extension.key"); payload["native_kind"]=json!("pick_tag");
    assert_eq!(serde_json::from_value::<old::OldChoiceRequest>(previous).unwrap(), serde_json::from_value::<old::OldChoiceRequest>(payload.clone()).unwrap());
    for (field,invalid) in [("prompt",json!(3)),("native_kind",json!("made_up")),("native_kind",json!(3))] {
        let mut bad=payload.clone();bad[field]=invalid;
        assert!(serde_json::from_value::<engine_api::ChoiceRequest>(bad).is_err());
    }
}

#[test]
fn actual_ws5_snapshot_bytes_round_trip_old_and_new_rust_readers() {
    let mut engine = Engine::new(42, 3);
    engine.debug_set_turn(0, Phase::Play, 1);
    engine.debug_grant(0, "反间");
    let skill = engine.catalog().skill_by_name("反间").unwrap();
    submit(&mut engine, GameAction::ActivateSkill { skill });
    let id = engine.open_choice().unwrap().id;
    submit(&mut engine, GameAction::Answer { choice_id: id, selection: vec![engine_native::ChoiceOption::Player(1)] });
    let message = engine_protocol::ServerMessage::Snapshot {
        header: engine_protocol::MessageHeader::new(909, 1, engine.revision()),
        view: observer_snapshot_for(&engine, RoomRole::Referee, None),
        legal_actions: vec![],
        choice: choice_for(&engine, RoomRole::Referee, None),
    };
    let wire = engine_protocol::encode_s2c(&message).unwrap();
    let old: old::OldServerMessage = serde_json::from_str(&wire).unwrap();
    let old::OldServerMessage::Snapshot { header, choice, .. } = &old;
    assert_eq!(header.protocol_version, 5);
    assert_eq!(choice.as_ref().unwrap().kind, engine_api::ChoiceKind::Custom);
    let old_wire = serde_json::to_string(&old).unwrap();
    let engine_protocol::ServerMessage::Snapshot { view, choice, .. } = engine_protocol::decode_s2c(&old_wire).unwrap() else { panic!("snapshot") };
    assert!(view.seats.iter().all(|seat| seat.generals == [None,None]));
    let choice = choice.unwrap();
    assert!(choice.prompt.is_none() && choice.native_kind.is_none());
}
