//! Emit trusted archives and server expectations for the generated WASM32 parity check.
//! cargo run -p game-server --example emit_projection_fixture -- /tmp/projection.json
#[path = "../tests/support/nc09_projection_fixtures.rs"]
mod fixtures;
#[path = "../src/native_view.rs"]
mod native_view;

use engine_api::PlayerId;
use native_view::{observer_snapshot_for, RoomRole};
use serde_json::json;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = std::env::args().nth(1).ok_or("output JSON path required")?;
    let mut cases = Vec::new();
    for (name, engine) in fixtures::scenarios() {
        let bytes = engine_native::archive::encode_archive(&engine.save());
        let archive_hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        let restored = engine_native::Engine::restore(
            &engine_native::archive::decode_archive(&bytes).expect("decode generated archive"),
        )
        .expect("restore generated archive");
        let seats: Vec<_> = (0..engine.state_core().players.len())
            .map(|seat| {
                let id = Some(PlayerId::new(seat as u32 + 1));
                let view = observer_snapshot_for(&engine, RoomRole::Player, id);
                assert_eq!(view, observer_snapshot_for(&restored, RoomRole::Player, id));
                view
            })
            .collect();
        cases.push(json!({
            "name": name,
            "archiveHex": archive_hex,
            "seats": seats,
            "spectator": observer_snapshot_for(&engine, RoomRole::Spectator, Some(PlayerId::new(2))),
            "hands": engine.state_core().players.iter().map(|p| &p.hand).collect::<Vec<_>>(),
            "deck": engine.state_core().deck,
        }));
    }
    std::fs::write(output, serde_json::to_vec_pretty(&cases)?)?;
    println!("emitted {} same-archive projection cases", cases.len());
    Ok(())
}
