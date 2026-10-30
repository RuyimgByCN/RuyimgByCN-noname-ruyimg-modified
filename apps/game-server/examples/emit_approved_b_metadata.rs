//! Actual archives and authorized server expectations for approved B cross-entry acceptance.
#[path = "../src/native_view.rs"]
mod native_view;
#[path = "../../../crates/engine-native/tests/support/nc09_machine_windows.rs"]
mod windows;
use engine_api::PlayerId;
use engine_native::{Engine, GameAction, Phase};
use native_view::{choice_for, observer_snapshot_for, RoomRole};
use serde_json::{json, Value};

fn submit(engine: &mut Engine, action: GameAction) {
    windows::submit(engine, action);
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args().nth(1).ok_or("output JSON required")?;
    let mut cases: Vec<(String, Engine, String)> = Vec::new();
    let engine = Engine::new_identity(1515, 4);
    let mut snapshot = engine.save();
    for player in &mut snapshot.core.players[..2] {
        player.general_slots = [Some(0), Some(1)];
        player.revealed_generals = [false, false];
    }
    cases.push(("ordinary-dual-def-zero".into(), Engine::restore(&snapshot).unwrap(), "[]".into()));
    snapshot.core.mode = engine_native::GameMode::Kingdoms;
    for (name, revealed) in [("kingdoms-dark", [false,false]), ("kingdoms-primary-revealed", [true,false]), ("kingdoms-both-revealed", [true,true])] {
        for player in &mut snapshot.core.players[..2] { player.revealed_generals = revealed; }
        cases.push((name.into(), Engine::restore(&snapshot).unwrap(), "[]".into()));
    }
    let mut engine = Engine::new(804, 3);
    engine.debug_grant(1, "突袭");
    engine.debug_set_turn(1, Phase::Judge, 1);
    submit(&mut engine, GameAction::NextPhase);
    cases.push(("confirm".into(), Engine::restore(&engine.save()).unwrap(), "[]".into()));
    let choice_id = engine.open_choice().unwrap().id;
    submit(&mut engine, GameAction::Answer { choice_id, selection: vec![engine_native::ChoiceOption::Confirm] });
    cases.push(("select_players".into(), engine, "[]".into()));
    let mut engine = Engine::new(1515, 4);
    engine.debug_set_turn(1, Phase::Play, 1);
    engine.debug_set_hp(1, 1);
    submit(&mut engine, GameAction::NextPhase);
    cases.push(("select_cards".into(), engine, "[]".into()));
    let mut engine = Engine::new(2, 2);
    engine.debug_set_turn(0, Phase::Play, 1);
    let card = engine.debug_place(0, "sha");
    engine.debug_place(1, "shan");
    submit(&mut engine, GameAction::PlayCard { card, targets: vec![1] });
    cases.push(("respond_card".into(), engine, "[]".into()));
    let mut engine = Engine::new(42, 3);
    engine.debug_set_turn(0, Phase::Play, 1);
    engine.debug_grant(0, "反间");
    let skill = engine.catalog().skill_by_name("反间").unwrap();
    submit(&mut engine, GameAction::ActivateSkill { skill });
    let choice_id = engine.open_choice().unwrap().id;
    submit(&mut engine, GameAction::Answer { choice_id, selection: vec![engine_native::ChoiceOption::Player(1)] });
    cases.push(("pick_tag-fanjian".into(), engine, "[]".into()));
    for window in [windows::Window::XunXun, windows::Window::JiJie, windows::Window::GaiMing, windows::Window::HaoTianTa, windows::Window::Liaoyuan] {
        let fixture = windows::fixture(window);
        let directory = match window {
            windows::Window::GaiMing => Some("boss-zhugeliang-upstream-liuli"),
            windows::Window::HaoTianTa | windows::Window::Liaoyuan => Some("shahai-swd-upstream-noname"),
            _ => None,
        };
        let packs: Vec<Value> = directory.map(|directory| {
            let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions").join(directory).join("assets/cards.json");
            serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
        }).into_iter().collect();
        cases.push((format!("machine-{window:?}"), fixture.engine, serde_json::to_string(&packs)?));
    }
    let mut result = Vec::new();
    for (name, engine, packs_json) in cases {
        let before = engine.save();
        let bytes = engine_native::archive::encode_archive(&before);
        let hex: String = bytes.iter().map(|byte| format!("{byte:02x}")).collect();
        let seats: Vec<_> = (0..engine.state_core().players.len()).map(|seat| {
            let id = Some(PlayerId::new(seat as u32 + 1));
            json!({"view": observer_snapshot_for(&engine, RoomRole::Player, id), "choice": choice_for(&engine, RoomRole::Player, id)})
        }).collect();
        result.push(json!({"name":name,"archiveHex":hex,"packsJson":packs_json,"seats":seats,
            "referee": {"view": observer_snapshot_for(&engine, RoomRole::Referee, None), "choice": choice_for(&engine, RoomRole::Referee, None)},
            "spectator": {"view": observer_snapshot_for(&engine, RoomRole::Spectator, Some(PlayerId::new(1))), "choice": choice_for(&engine, RoomRole::Spectator, Some(PlayerId::new(1)))}}));
        assert_eq!(engine.save(), before);
    }
    std::fs::write(output, serde_json::to_vec_pretty(&result)?)?;
    println!("emitted {} actual same-archive B cases", result.len());
    Ok(())
}
