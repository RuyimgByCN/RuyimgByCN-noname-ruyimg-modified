//! NC10: unknown physical cards in VM piles and live response windows.
use engine_native::ai::{redact_opponent_hands, BoundedPlanner};
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};

fn submit(e: &mut Engine, actor: u8, action: GameAction) {
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: e.save().last_dedup + 1,
        action,
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
}

#[test]
fn real_liaoyuan_suit_scalar_is_preserved_in_conservative_host_path() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let json = std::fs::read_to_string(
        root.join("../../packages/extensions/shahai-swd-upstream-noname/assets/cards.json"),
    )
    .unwrap();
    let pack = content_native_loader::bridge_pack(&json).unwrap().pack;
    let mut e = Engine::new_with_extensions(122001, 4, &[&pack]).unwrap();
    e.debug_grant(0, "liaoyuan");
    e.debug_set_turn(0, Phase::Play, 1);
    let sha = e.debug_move_named_card_to_hand(0, "sha");
    let suit = e.catalog().cards[sha as usize].suit;
    for _ in 0..3 {
        let index = e
            .state_core()
            .deck
            .iter()
            .position(|c| e.catalog().cards[*c as usize].suit == suit)
            .unwrap();
        let card = e.state_core_mut().deck.remove(index);
        e.state_core_mut().players[0].hand.push(card);
    }
    eprintln!(
        "LIAOYUAN_COMMAND actor=0 revision={} dedup={} PlayCard card={sha} targets=[1]",
        e.revision(),
        e.save().last_dedup + 1
    );
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    for _ in 0..16 {
        let c = e.open_choice().cloned().unwrap();
        if c.spec.prompt == "liaoyuan_discard" {
            break;
        }
        let selection = if c
            .spec
            .options
            .contains(&engine_native::ChoiceOption::Confirm)
        {
            vec![engine_native::ChoiceOption::Confirm]
        } else {
            c.spec.options.iter().take(c.spec.min).copied().collect()
        };
        eprintln!("LIAOYUAN_COMMAND actor={} revision={} dedup={} Answer choice={} selection={selection:?}", c.observer, e.revision(), e.save().last_dedup + 1, c.id);
        submit(
            &mut e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection,
            },
        );
    }
    assert_eq!(e.open_choice().unwrap().spec.prompt, "liaoyuan_discard");
    let before = e.save();
    let expected: Vec<_> = before
        .resolution
        .parks
        .iter()
        .filter_map(|p| engine_native::snap::active_ask(&p.machine, &before.core, e.catalog()))
        .collect();
    eprintln!("LIAOYUAN_EXPECTED {expected:#?}");
    eprintln!("LIAOYUAN_ACTUAL {:#?}", before.resolution.open_choice);
    eprintln!("LIAOYUAN_SNAPSHOT {before:#?}");
    eprintln!(
        "LIAOYUAN_ARCHIVE {:?}",
        engine_native::archive::encode_archive(&before)
    );
    Engine::restore_with_extensions(&before, &[&pack]).unwrap();
    let scalar = before
        .resolution
        .parks
        .iter()
        .find_map(|park| match &park.machine {
            engine_native::MachineSnapshot::Host {
                behavior,
                stage,
                card,
                ..
            } if *behavior == engine_native::content::host::HostBehavior::Liaoyuan.tag() => {
                assert_eq!(
                    *stage,
                    engine_native::content::host::HostStage::SelectCard.tag()
                );
                *card
            }
            _ => None,
        })
        .expect("real suit scalar");
    assert_eq!(scalar, suit as u32);
    redact_opponent_hands(&mut e, 0);
    assert_eq!(e.save().resolution, before.resolution);
    Engine::restore_with_extensions(&e.save(), &[&pack]).unwrap();
}
