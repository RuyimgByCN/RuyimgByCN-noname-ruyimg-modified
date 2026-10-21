use super::*;

#[test]
fn probe_tashui2_tmp() {
    let mut e = engine_native::Engine::new_boss_stage(83002, 4, 1, 3, 6, 4, 0);
    let zj_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 40) as u16;
    e.debug_set_general_def(1, zj_def);
    e.debug_set_hp(1, 4);
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    let c = place_sha_of_color(&mut e, 1, true);
    eprintln!(
        "CARD suit={:?} inhand1={}",
        e.catalog().cards[c as usize].suit,
        e.debug_hand(1).contains(&c)
    );
    let out = play(&mut e, c, vec![0]);
    eprintln!("PLAY {out:?}");
}
