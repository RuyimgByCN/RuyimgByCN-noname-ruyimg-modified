use engine_native::archive::{decode_archive, encode_archive, ARCHIVE_FORMAT_VERSION};
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};

#[test]
fn capture_real_v324_archives() {
    let out = std::path::PathBuf::from(std::env::var("NC07_CAPTURE_DIR").unwrap());
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let json = std::fs::read_to_string(
        root.join("../../packages/extensions/shahai-swd-upstream-noname/assets/cards.json"),
    ).unwrap();
    let pack = content_native_loader::bridge_pack(&json).unwrap().pack;
    let plain = Engine::new(3505, 4);
    assert_eq!(plain.ruleset_version(), 324);
    let plain_bytes = encode_archive(&plain.save());
    assert_eq!(ARCHIVE_FORMAT_VERSION, 105);
    assert!(Engine::restore(&decode_archive(&plain_bytes).unwrap()).is_ok());
    std::fs::write(out.join("native-v324.ngsa"), &plain_bytes).unwrap();

    let mut jar = Engine::new_with_extensions(3504, 4, &[&pack]).unwrap();
    for _ in 0..8 {
        if jar.turn().phase == Phase::Play { break; }
        let out = jar.submit(GameCommand {
            actor: 0,
            expected_revision: jar.revision(),
            dedup_id: jar.revision() + 1,
            action: GameAction::NextPhase,
        });
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    }
    assert_eq!(jar.turn().phase, Phase::Play);
    jar.debug_grant(0, "lianyaohu_lianhua");
    let first = jar.state_core_mut().deck.pop().unwrap();
    let second = jar.state_core_mut().deck.pop().unwrap();
    jar.state_core_mut().players[0].lianyaohu_store = vec![first, second];
    let snapshot = jar.save();
    let bytes = encode_archive(&snapshot);
    let decoded = decode_archive(&bytes).unwrap();
    assert_eq!(decoded, snapshot);
    assert_eq!(Engine::restore_with_extensions(&decoded, &[&pack]).unwrap().state_hash(), jar.state_hash());
    std::fs::write(out.join("lianyaohu-v324.ngsa"), &bytes).unwrap();
    println!("base=324 archive={} extension_rules={} jar={:?} extension_archive={}",
        plain_bytes.len(), snapshot.ruleset_version, snapshot.core.players[0].lianyaohu_store, bytes.len());
}
