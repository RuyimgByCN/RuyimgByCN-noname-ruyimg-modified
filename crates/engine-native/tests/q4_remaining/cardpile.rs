use std::collections::BTreeSet;

use content_native_loader::{cardpile_arena_ready_pack, verify_pile_json_buckets};
use engine_native::content::extension::ExtensionDef;
use engine_native::{Engine, GameAction, Phase};

use super::helpers::{
    assert_accepted_or_pending, assert_archive_roundtrip, drain_default, repo_asset, step_to_phase,
    submit,
};

#[test]
fn u03_and_l04_machine_verify_all_29_buckets_scale_rng_ranks_and_combat_play() {
    let noname_pile = repo_asset("packages/extensions/cardpile-upstream-noname/assets/pile.json");
    let liuli_pile = repo_asset("packages/extensions/cardpile-upstream-liuli/assets/pile.json");

    let (spec_n, buckets_n) = verify_pile_json_buckets(&noname_pile).expect("parse noname pile.json");
    let (spec_l, buckets_l) = verify_pile_json_buckets(&liuli_pile).expect("parse liuli pile.json");

    assert_eq!(buckets_n.len(), 29, "all 29 (card, suit) pile buckets in U03");
    assert_eq!(buckets_l.len(), 29, "all (card, suit) pile buckets in L04");
    assert_eq!(spec_n.total, 160);
    assert_eq!(spec_l.total, 160);
    let sum_n: u32 = buckets_n.iter().map(|(_, _, c)| *c).sum();
    assert_eq!(sum_n, 113);

    let all_ones = r#"{"sha":1,"huosha":1,"leisha":1,"shan":1,"jiu":1,"tao":1,"wanjian":1,"nanman":1,"guohe":1,"shunshou":1,"wuxie":1,"tiesuo":1}"#;
    let partial = r#"{"sha":1}"#;
    let def = ExtensionDef {
        name: "cardpile-upstream-noname",
        version: 1,
        description: "arenaReady test pack",
    };

    // Unconfigured / missing key -> skipped
    assert!(
        cardpile_arena_ready_pack(&noname_pile, partial, 207, 11, def.clone(), "u03")
            .unwrap()
            .is_none(),
        "unconfigured factors skip scaling"
    );
    // base_deck_len <= 160 -> skipped
    assert!(
        cardpile_arena_ready_pack(&noname_pile, all_ones, 160, 11, def.clone(), "u03")
            .unwrap()
            .is_none(),
        "deck_len <= 160 skips scaling"
    );

    // Cap at 1000 enforced by scale_plan when extreme deck_len provided
    let capped_pack = cardpile_arena_ready_pack(&noname_pile, all_ones, 10_000, 99, def.clone(), "cap")
        .unwrap()
        .expect("scaled pack");
    assert!(
        capped_pack.card_count() <= 1000 + 12,
        "dn capped near 1000 rounding slack: {}",
        capped_pack.card_count()
    );

    // Normal scaling (deck_len = 207 -> adds 113 cards) with RNG 1..=13 ranks varying by seed
    let pack_seed1 = cardpile_arena_ready_pack(&noname_pile, all_ones, 207, 111, def.clone(), "s1")
        .unwrap()
        .expect("pack s1");
    let pack_seed2 = cardpile_arena_ready_pack(&liuli_pile, all_ones, 207, 222, def, "s2")
        .unwrap()
        .expect("pack s2");
    assert_eq!(pack_seed1.card_count(), 113);
    assert_eq!(pack_seed2.card_count(), 113);
    assert!(pack_seed1.cards.iter().all(|c| (1..=13).contains(&c.rank)));
    let ranks1: Vec<u8> = pack_seed1.cards.iter().map(|c| c.rank).collect();
    let ranks2: Vec<u8> = pack_seed2.cards.iter().map(|c| c.rank).collect();
    assert_ne!(ranks1, ranks2, "different RNG seeds must assign distinct rank sequences");
    let distinct_ranks: BTreeSet<u8> = ranks1.iter().copied().collect();
    assert!(distinct_ranks.len() >= 10, "RNG fill covers 1..=13 spread");

    // Build real Engine with injected cardpile and verify deck growth & real combat card play + restore
    let base_engine = Engine::new(77, 2);
    let base_card_count = base_engine.catalog().cards.len();
    let mut e = Engine::new_with_extensions(77, 2, &[&pack_seed1]).expect("engine with cardpile");
    assert_eq!(e.catalog().cards.len(), base_card_count + 113);

    let jiu_name = pack_seed1
        .cards
        .iter()
        .find(|c| c.kind == engine_native::catalog::CardKind::Jiu)
        .unwrap()
        .name;
    let leisha_name = pack_seed1
        .cards
        .iter()
        .find(|c| c.kind == engine_native::catalog::CardKind::LeiSha)
        .unwrap()
        .name;

    step_to_phase(&mut e, 0, Phase::Play);
    e.debug_clear_hand(1);
    let jiu = e.debug_place(0, jiu_name);
    let leisha = e.debug_place(0, leisha_name);
    assert_accepted_or_pending(&submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: jiu,
            targets: vec![],
        },
    ));
    drain_default(&mut e);
    assert_accepted_or_pending(&submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: leisha,
            targets: vec![1],
        },
    ));
    drain_default(&mut e);
    assert_eq!(e.state_core().player(1).hp, 2, "injected jiu + leisha dealt 2 damage");

    let restored = assert_archive_roundtrip(&e, &[&pack_seed1]);
    assert_eq!(restored.state_core().player(1).hp, 2);
}
