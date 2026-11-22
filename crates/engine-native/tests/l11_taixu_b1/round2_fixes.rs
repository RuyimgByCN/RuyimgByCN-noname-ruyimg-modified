use std::collections::BTreeMap;
use std::path::Path;

use content_native_loader::{
    prepare_locked_content_with_bridge, start_locked_engine, validate_and_freeze_raw_package,
    LockedStartMode,
};
use engine_native::catalog::{CardKind, Faction, Suit};
use engine_native::engine::RejectReason;
use engine_native::event::DamageNature;
use engine_native::rvm_bridge::modifiers::eval_card_target_canceled;
use engine_native::state::Role;
use engine_native::{ChoiceOption, CommandOutcome, Phase};

use super::common::*;

/// Finding #1 (`extension.js:4151-4195`, CDP P6):
/// `cxyFengYing` protects the ally `target` with strictly unique lowest HP, NOT an enemy attacker.
#[test]
fn r2_fix01_cxy_fengying_protects_unique_min_ally_not_unique_min_enemy() {
    let mut e = start_versus(0x2101, &[&[0, 1], &[2, 3]], &[Faction::Wei, Faction::Shu]);
    e.debug_grant(0, "cxyFengYing");

    // Unique-min enemy (seat 2 hp=1) CAN target high-HP ally (seat 1 hp=6)
    e.debug_set_hp(2, 1);
    let sha_2a = place_kind_suit(&mut e, 2, CardKind::Sha, Suit::Spade);
    assert!(
        !eval_card_target_canceled(
            e.state_core(),
            e.catalog(),
            2,
            1,
            sha_2a,
            CardKind::Sha,
            DamageNature::Normal,
        ),
        "unique-min enemy attacker must NOT be blocked from high-HP ally"
    );
    e.debug_set_turn(2, Phase::Play, 1);
    let out_ok = play_card(&mut e, 2, sha_2a, vec![1]);
    assert!(
        matches!(out_ok, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "unique-min enemy targeting high-HP ally must succeed: {out_ok:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 5);

    // Unique-min ally (seat 1 hp=1, all others >= 3) CANNOT be targeted by enemy seat 2
    e.debug_set_hp(2, 3);
    e.debug_set_hp(3, 4);
    e.debug_set_hp(0, 6);
    e.debug_set_hp(1, 1);
    let sha_2b = place_kind_suit(&mut e, 2, CardKind::Sha, Suit::Club);
    e.debug_set_turn(2, Phase::Play, 2);
    assert!(
        eval_card_target_canceled(
            e.state_core(),
            e.catalog(),
            2,
            1,
            sha_2b,
            CardKind::Sha,
            DamageNature::Normal,
        ),
        "enemy must NOT target unique-min ally"
    );
    assert!(matches!(
        play_card(&mut e, 2, sha_2b, vec![1]),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ));

    // Tied min HP (seat 3 also hp=1) lifts protection on seat 1
    e.debug_set_hp(3, 1);
    assert!(!eval_card_target_canceled(
        e.state_core(),
        e.catalog(),
        2,
        1,
        sha_2b,
        CardKind::Sha,
        DamageNature::Normal,
    ));
}

/// Finding #2 (`extension.js:2053-2086`, CDP P4):
/// `txhj_ruiSkill1` filters `hand < owner.hand` and burns enemies with `hand <= owner.hand` regardless of HP.
#[test]
fn r2_fix02_rui_skill1_checks_hand_count_not_hp() {
    let mut e = start_versus(
        0x2102,
        &[&[0], &[1, 2, 3]],
        &[Faction::Wei, Faction::Shu],
    );
    e.debug_grant(0, "txhj_ruiSkill1");
    e.debug_set_hp(0, 1);
    e.debug_set_hp(1, 6);
    e.debug_set_hp(2, 1);
    e.debug_set_hp(3, 6);

    for _ in 0..3 {
        let _ = place_kind(&mut e, 0, CardKind::Sha);
    }
    let _ = place_kind(&mut e, 1, CardKind::Sha); // hand 1 < 3 -> triggers filter & burned
    for _ in 0..5 {
        let _ = place_kind(&mut e, 2, CardKind::Shan); // hand 5 > 3 -> spared
    }
    for _ in 0..3 {
        let _ = place_kind(&mut e, 3, CardKind::Tao); // hand 3 == 3 -> burned once triggered
    }

    let _ = next_phase(&mut e);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 5, "enemy with fewer hand cards takes 1 Fire");
    assert_eq!(e.debug_hp(3), 5, "enemy with equal hand cards takes 1 Fire once filter passes");
    assert_eq!(e.debug_hp(2), 1, "low-HP enemy with MORE hand cards is spared");

    // When all enemies have hand >= owner.hand, filter must NOT trigger
    let mut e_eq = start_versus(0x2103, &[&[0], &[1]], &[Faction::Wei, Faction::Shu]);
    e_eq.debug_grant(0, "txhj_ruiSkill1");
    e_eq.debug_set_hp(0, 6);
    e_eq.debug_set_hp(1, 2);
    for _ in 0..2 {
        let _ = place_kind(&mut e_eq, 0, CardKind::Sha);
        let _ = place_kind(&mut e_eq, 1, CardKind::Sha);
    }
    let _ = next_phase(&mut e_eq);
    drain(&mut e_eq);
    assert_eq!(e_eq.debug_hp(1), 2, "must not trigger without enemy strictly fewer hand cards");

    // Skipped Play phase (`skip_play_phase` / `乐不思蜀` and `dqtianzi`) must NEVER trigger `txhj_ruiSkill1`
    let mut e_skip = start_versus(0x211b, &[&[0], &[1]], &[Faction::Wei, Faction::Shu]);
    e_skip.debug_grant(0, "txhj_ruiSkill1");
    for _ in 0..3 {
        let _ = place_kind(&mut e_skip, 0, CardKind::Sha);
    }
    e_skip.debug_set_turn(0, Phase::Draw, 1);
    e_skip.state_core_mut().turn.skip_play_phase = true;
    let _ = next_phase(&mut e_skip);
    drain(&mut e_skip);
    assert_eq!(e_skip.debug_hp(1), 6, "skipped Phase::Play (乐不思蜀) must NOT trigger txhj_ruiSkill1");

    let mut e_dq = start_versus(0x211c, &[&[0], &[1]], &[Faction::Wei, Faction::Shu]);
    e_dq.debug_grant(0, "txhj_ruiSkill1");
    for _ in 0..3 {
        let _ = place_kind(&mut e_dq, 0, CardKind::Sha);
    }
    e_dq.debug_set_turn(0, Phase::Draw, 1);
    e_dq.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "l11_dqtianzi_skip_play_next".to_string()), 1);
    let _ = next_phase(&mut e_dq);
    drain(&mut e_dq);
    assert_eq!(e_dq.debug_hp(1), 6, "tx_dqtianzi skipped Phase::Play must NOT trigger txhj_ruiSkill1");

    // Extra `Phase::Play` (`extra_phases`) fires `txhj_ruiSkill1` at Play #1 end AND Play #2 end
    let mut e_ex = start_versus(0x211d, &[&[0], &[1]], &[Faction::Wei, Faction::Shu]);
    e_ex.debug_grant(0, "txhj_ruiSkill1");
    for _ in 0..3 {
        let _ = place_kind(&mut e_ex, 0, CardKind::Sha);
    }
    let _ = place_kind(&mut e_ex, 1, CardKind::Shan);
    e_ex.state_core_mut().turn.extra_phases.push(2); // extra Play phase after Play #1
    let _ = next_phase(&mut e_ex);
    drain(&mut e_ex);
    assert_eq!(e_ex.state_core().turn.phase, Phase::Play, "should enter Play #2");
    assert_eq!(e_ex.debug_hp(1), 5, "Play #1 end must fire txhj_ruiSkill1 before Play #2");
    let _ = next_phase(&mut e_ex);
    drain(&mut e_ex);
    assert_eq!(e_ex.debug_hp(1), 4, "Play #2 end must fire txhj_ruiSkill1 before Discard");
}

/// Finding #3 (`extension.js:2824-2847`, CDP P5):
/// `txhj_tengsheSkill2` only triggers on owner's turn end (`phaseAfter`) and checks `enemies.len() == 1`.
#[test]
fn r2_fix03_tengshe_skill2_owner_turn_only_and_total_enemy_count() {
    let mut e = start_versus(0x2104, &[&[0], &[1, 2]], &[Faction::Wei, Faction::Shu]);
    e.debug_grant(0, "txhj_tengsheSkill2");
    e.debug_set_hp(1, 4); // injured enemy

    // Enemy turn end must NOT trigger owner's tengsheSkill2
    e.debug_set_turn(1, Phase::End, 1);
    let _ = next_phase(&mut e);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 4, "enemy TurnEndWindow must not trigger owner tengsheSkill2");
    assert_eq!(e.debug_hp(2), 6);

    // Owner turn end with 2 alive enemies (1 damaged, 1 full) -> 1 Thunder each (NOT 2!)
    e.debug_set_turn(0, Phase::End, 2);
    let _ = next_phase(&mut e);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 3);
    assert_eq!(e.debug_hp(2), 5);

    // Kill seat 2 -> now only 1 alive enemy -> deals 2 Thunder on owner turn end
    e.state_core_mut().player_mut(2).alive = false;
    e.debug_set_turn(0, Phase::End, 3);
    let _ = next_phase(&mut e);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 1, "single alive enemy takes 2 Thunder");
}

/// Finding #4 (`extension_card.js:775-795, 1897`, CDP P1):
/// Nullifying `txhj_huoshaolianying` stops the entire chain, and both `huoshaolianying` and `jiuchiroulin`
/// preserve `card` on damage so `txhj_chongyingshenfu` records and prevents repeat damage.
#[test]
fn r2_fix04_huoshaolianying_wuxie_stops_chain_and_card_ref_preserved() {
    // 1. Nullifying seat 1 stops chain to friend seat 2 (and asks only once)
    let mut e_wx = start_versus(0x2105, &[&[0, 3], &[1, 2]], &[Faction::Wei, Faction::Shu]);
    let hs = e_wx.debug_move_named_card_to_hand(0, "txhj_huoshaolianying");
    let wx = place_kind_suit(&mut e_wx, 1, CardKind::Nullify, Suit::Spade);
    let _ = play_card(&mut e_wx, 0, hs, Vec::new());
    let c = e_wx.open_choice().expect("single wuxie choice").clone();
    let _ = answer(&mut e_wx, &c, vec![ChoiceOption::Card(wx)]);
    assert!(e_wx.open_choice().is_none(), "must NOT open another wuxie choice");
    drain(&mut e_wx);
    assert_eq!(e_wx.debug_hp(1), 6);
    assert_eq!(e_wx.debug_hp(2), 6);

    // 2. Unnullified huoshaolianying chain carries card ref onto seat 2 (chongyingshenfu blocks 2nd burn)
    let mut e_cy = start_versus(0x2106, &[&[0, 3], &[1, 2]], &[Faction::Wei, Faction::Shu]);
    let _ = equip_named(&mut e_cy, 2, "txhj_chongyingshenfu");
    let hs1 = e_cy.debug_move_named_card_to_hand(0, "txhj_huoshaolianying");
    let _ = play_card(&mut e_cy, 0, hs1, Vec::new());
    drain(&mut e_cy);
    assert_eq!(e_cy.debug_hp(1), 5);
    assert_eq!(e_cy.debug_hp(2), 5);
    let hs2 = e_cy.debug_move_named_card_to_hand(0, "txhj_huoshaolianying");
    let _ = play_card(&mut e_cy, 0, hs2, Vec::new());
    drain(&mut e_cy);
    assert_eq!(e_cy.debug_hp(1), 4);
    assert_eq!(e_cy.debug_hp(2), 5, "chongyingshenfu must recognize card name across chain");

    // 3. txhj_jiuchiroulin damage carries card ref (chongyingshenfu blocks 2nd hit)
    let mut e_jc = start_taixu(0x2107, 2);
    let _ = equip_named(&mut e_jc, 1, "txhj_chongyingshenfu");
    let jc1 = e_jc.debug_move_named_card_to_hand(0, "txhj_jiuchiroulin");
    let _ = play_card(&mut e_jc, 0, jc1, Vec::new());
    drain(&mut e_jc);
    assert_eq!(e_jc.debug_hp(1), 5);
    let jc2 = e_jc.debug_move_named_card_to_hand(0, "txhj_jiuchiroulin");
    let _ = play_card(&mut e_jc, 0, jc2, Vec::new());
    drain(&mut e_jc);
    assert_eq!(e_jc.debug_hp(1), 5, "chongyingshenfu must prevent 2nd jiuchiroulin damage");
}

/// Finding #5 (`extension_card.js:1639-1671`, CDP P2):
/// `txhj_fangtianhuaji` checks attack range, forces [1, 2] extra targets when alive >= 3,
/// skips when alive < 3, and preserves Fire/Thunder nature on Sha.
#[test]
fn r2_fix05_fangtianhuaji_range_forced_nature_and_two_player_skip() {
    // 6-seat ring: range 4 reaches 1 (dist 1) & 5 (dist 1), does NOT reach 3 with juechenjinge (+2 -> dist 5)
    let mut e = start_taixu(0x2108, 6);
    let _ = equip_named(&mut e, 0, "txhj_fangtianhuaji"); // attack range 4
    let _ = equip_named(&mut e, 3, "txhj_juechenjinge"); // distance +2 => 3+2 = 5 > 4
    let fsha = place_kind(&mut e, 0, CardKind::FireSha);
    let _ = play_card(&mut e, 0, fsha, vec![1]);
    let c_ft = expect_pending(&e, "txhj_fangtianhuaji:extra_targets");
    assert!(!c_ft.spec.allow_cancel, "fangtianhuaji must be forced when alive >= 3");
    assert_eq!(c_ft.spec.min, 1);
    assert!(c_ft.spec.options.contains(&ChoiceOption::Player(5)));
    assert!(
        !c_ft.spec.options.contains(&ChoiceOption::Player(3)),
        "out-of-range seat 3 (dist 5 > range 4) must be excluded from fangtian candidates"
    );
    let _ = answer(&mut e, &c_ft, vec![ChoiceOption::Player(5)]);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 5);
    assert_eq!(e.debug_hp(5), 5);
    assert_eq!(e.debug_hp(3), 6);

    // 2a. `txminguangkai` (`useCardToTarget`) cancels extra `FireSha` target added by `fangtianhuaji` (`useCard2`)
    let mut e_nat = start_taixu(0x2109, 3);
    let _ = equip_named(&mut e_nat, 0, "txhj_fangtianhuaji");
    let _ = equip_named(&mut e_nat, 2, "txminguangkai"); // blocks FireSha at CardUsed priority 8 < 12!
    let fs2 = place_kind(&mut e_nat, 0, CardKind::FireSha);
    let _ = play_card(&mut e_nat, 0, fs2, vec![1]);
    let c_nat = expect_pending(&e_nat, "txhj_fangtianhuaji:extra_targets");
    let _ = answer(&mut e_nat, &c_nat, vec![ChoiceOption::Player(2)]);
    drain(&mut e_nat);
    assert_eq!(e_nat.debug_hp(1), 5, "primary target takes 1 Fire damage");
    assert_eq!(e_nat.debug_hp(2), 6, "txminguangkai cancels FireSha extra target added by fangtianhuaji");

    // 2b. `eval_card_target_canceled` excludes `zglkongcheng` seat from `fangtianhuaji:extra_targets`
    let mut e_kc = start_taixu(0x2119, 3);
    let _ = equip_named(&mut e_kc, 0, "txhj_fangtianhuaji");
    e_kc.debug_grant(2, "zglkongcheng"); // empty hand immune to Sha/Duel
    let fs_kc = place_kind(&mut e_kc, 0, CardKind::FireSha);
    let _ = play_card(&mut e_kc, 0, fs_kc, vec![1]);
    assert!(
        e_kc.open_choice().is_none(),
        "zglkongcheng empty-hand target must be excluded from fangtianhuaji extra candidates"
    );
    drain(&mut e_kc);
    assert_eq!(e_kc.debug_hp(1), 5);
    assert_eq!(e_kc.debug_hp(2), 6);

    // 2-player game skips fangtianhuaji cleanly
    let mut e_2p = start_taixu(0x210a, 2);
    let _ = equip_named(&mut e_2p, 0, "txhj_fangtianhuaji");
    let s2p = place_kind_suit(&mut e_2p, 0, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut e_2p, 0, s2p, vec![1]);
    assert!(e_2p.open_choice().is_none());
    drain(&mut e_2p);
    assert_eq!(e_2p.debug_hp(1), 5);
}

/// Finding #6 (`extension_card.js:233-288, 1356-1431`, CDP P3):
/// `txhj_zhijizhibi` draws 0 when target ends up with 0 hand cards and no unseen generals;
/// BOTH `txhj_zhijizhibi` and `txhj_shengdong` support `chongzhu: true` recast (both direct & `_chongzhu`).
#[test]
fn r2_fix06_zhijizhibi_empty_target_zero_draw_and_chongzhu_recast() {
    // 1. Target has 1 card (Nullify), spends it to cancel zhibi, owner counter-nullifies ->
    // zhibi resolves un-nullified against now-empty target -> 0 draw!
    let mut e = start_taixu(0x210b, 2);
    let zb = e.debug_move_named_card_to_hand(0, "txhj_zhijizhibi");
    let wx0 = place_kind_suit(&mut e, 0, CardKind::Nullify, Suit::Spade);
    let wx1 = place_kind_suit(&mut e, 1, CardKind::Nullify, Suit::Club);
    let _ = play_card(&mut e, 0, zb, vec![1]);
    let c1 = e.open_choice().expect("seat 1 wuxie").clone();
    let _ = answer(&mut e, &c1, vec![ChoiceOption::Card(wx1)]);
    let c0 = e.open_choice().expect("seat 0 counter-wuxie").clone();
    let _ = answer(&mut e, &c0, vec![ChoiceOption::Card(wx0)]);
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), 0, "0-hand target at resolution yields 0 draw");
    assert_eq!(flag(&e, 0, "l11_zhijizhibi_peeked"), 0);

    // 2. txhj_shengdong recast via PlayCard with 0 targets
    let mut e_sd = start_taixu(0x210c, 2);
    let sd = e_sd.debug_move_named_card_to_hand(0, "txhj_shengdong");
    let _ = play_card(&mut e_sd, 0, sd, Vec::new());
    drain(&mut e_sd);
    assert_eq!(e_sd.debug_hand(0).len(), 1, "shengdong recast drew 1");
    assert!(e_sd.state_core().discard.contains(&sd));

    // 3. Combined locked pack `_chongzhu` accepts both txhj_zhijizhibi and txhj_shengdong
    let freeze_pkg = |dir_name: &str| {
        let pkg_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions")
            .join(dir_name);
        let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
            .expect("read package.npack.json")
            .replace("\r\n", "\n");
        let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
        let mut assets = BTreeMap::new();
        if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
            for rel_path in paths.values().filter_map(|v| v.as_str()) {
                let bytes = std::fs::read(pkg_dir.join(rel_path))
                    .expect("read asset")
                    .into_iter()
                    .filter(|&b| b != b'\r')
                    .collect::<Vec<u8>>();
                assets.insert(rel_path.to_string(), bytes);
            }
        }
        validate_and_freeze_raw_package(raw_json, assets).expect("freeze package")
    };
    let verified_both = prepare_locked_content_with_bridge(vec![
        freeze_pkg("taixu-upstream-liuli"),
        freeze_pkg("mark-supplement-upstream-liuli"),
    ])
    .expect("bridge combined taixu + mark-supplement");
    let mut e_cz = start_locked_engine(0x210d, 2, LockedStartMode::Prototype, &verified_both)
        .expect("combined locked engine");
    e_cz.debug_clear_hand(0);
    e_cz.debug_clear_hand(1);
    e_cz.debug_set_turn(0, Phase::Play, 1);
    e_cz.debug_grant(0, "_chongzhu");
    let zb_cz = e_cz.debug_move_named_card_to_hand(0, "txhj_zhijizhibi");
    let sd_cz = e_cz.debug_move_named_card_to_hand(0, "txhj_shengdong");
    let _ = activate_skill(&mut e_cz, 0, "_chongzhu");
    let c_cz = expect_pending(&e_cz, "_chongzhu:choose_card");
    assert!(c_cz.spec.options.contains(&ChoiceOption::Card(zb_cz)));
    assert!(c_cz.spec.options.contains(&ChoiceOption::Card(sd_cz)));
    let _ = answer(&mut e_cz, &c_cz, vec![ChoiceOption::Card(zb_cz)]);
    drain(&mut e_cz);
    assert!(e_cz.state_core().discard.contains(&zb_cz));
    assert_eq!(e_cz.debug_hand(0).len(), 2, "1 remaining sd_cz + 1 recast draw");
}

/// Finding #7 (`extension_card.js:2339-2381`):
/// Unequipping `txhj_chongyingshenfu` revokes `txhj_chongyingshenfu_effect` too.
#[test]
fn r2_fix07_chongyingshenfu_unequip_revokes_effect_skill() {
    let mut e = start_taixu(0x210e, 2);
    let _ = equip_named(&mut e, 1, "txhj_chongyingshenfu");
    e.debug_grant(0, "咆哮");
    let s1 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut e, 0, s1, vec![1]);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 5);
    assert!(e.debug_has_skill(1, "txhj_chongyingshenfu_effect"));

    // Replace armor with Renwang -> both chongyingshenfu skills must be gone
    let _ = equip_named(&mut e, 1, "renwang");
    assert!(!e.debug_has_skill(1, "txhj_chongyingshenfu"));
    assert!(!e.debug_has_skill(1, "txhj_chongyingshenfu_effect"));

    // Red Sha now hits seat 1 cleanly (Renwang only blocks black Sha)
    let s2 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Heart);
    let _ = play_card(&mut e, 0, s2, vec![1]);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 4, "chongyingshenfu_effect must not linger after unequip");
}

/// Finding #8 (`extension.js:2764-2783`, CDP P7):
/// `txhj_jiuweiSkill2` cancels each different opponent's 1st non-trick card in owner's turn.
#[test]
fn r2_fix08_jiuwei_skill2_cancels_each_opponents_first_nontrick() {
    let mut e = start_taixu(0x210f, 3);
    e.debug_grant(0, "txhj_jiuweiSkill2");
    e.debug_grant(0, "咆哮");
    e.debug_set_hp(1, 1);
    e.debug_set_hp(2, 1);

    let s1 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    let tao1a = place_kind(&mut e, 1, CardKind::Tao);
    let tao1b = place_kind(&mut e, 1, CardKind::Tao);
    let _ = play_card(&mut e, 0, s1, vec![1]);
    let c1a = answer_pass_until(&mut e, "dying_save");
    let _ = answer(&mut e, &c1a, vec![ChoiceOption::Card(tao1a)]);
    let c1b = answer_pass_until(&mut e, "dying_save");
    assert_eq!(e.debug_hp(1), 0, "opponent 1's 1st Tao cancelled");
    let _ = answer(&mut e, &c1b, vec![ChoiceOption::Card(tao1b)]);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 1, "opponent 1's 2nd Tao succeeds");

    // In the SAME turn of seat 0, opponent 2's 1st Tao must ALSO be cancelled!
    let s2 = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Heart);
    let tao2a = place_kind(&mut e, 2, CardKind::Tao);
    let tao2b = place_kind(&mut e, 2, CardKind::Tao);
    let _ = play_card(&mut e, 0, s2, vec![2]);
    let c2a = answer_pass_until(&mut e, "dying_save");
    let _ = answer(&mut e, &c2a, vec![ChoiceOption::Card(tao2a)]);
    let c2b = answer_pass_until(&mut e, "dying_save");
    assert_eq!(e.debug_hp(2), 0, "opponent 2's 1st Tao ALSO cancelled in same turn");
    let _ = answer(&mut e, &c2b, vec![ChoiceOption::Card(tao2b)]);
    drain(&mut e);
    assert_eq!(e.debug_hp(2), 1);
}

/// Finding #9 (`extension.js:3171-3192`, CDP P10):
/// `txhj_qiaoqiaoSkill1` has `usable: 1` per turn and triggers across multiple other seats' turns.
#[test]
fn r2_fix09_qiaoqiao_skill1_triggers_across_other_turns_in_same_round() {
    let mut e = start_versus(0x2110, &[&[0], &[1, 2]], &[Faction::Wei, Faction::Shu]);
    e.debug_grant(0, "txhj_qiaoqiaoSkill1");

    // Seat 1 Play phase places Le on seat 2 -> qiaoqiaoSkill1 drains 1 HP from an enemy
    e.debug_set_turn(1, Phase::Play, 1);
    let le1 = place_kind_suit(&mut e, 1, CardKind::Le, Suit::Heart);
    let _ = play_card(&mut e, 1, le1, vec![2]);
    drain(&mut e);
    let enemy_hp_sum_1 = e.debug_hp(1) + e.debug_hp(2);
    assert_eq!(enemy_hp_sum_1, 11);

    // Seat 2 Play phase (same round 1!) places Bing on seat 1 -> qiaoqiaoSkill1 MUST trigger again!
    e.debug_set_turn(2, Phase::Play, 1);
    let bing2 = place_kind_suit(&mut e, 2, CardKind::Bing, Suit::Club);
    let _ = play_card(&mut e, 2, bing2, vec![1]);
    drain(&mut e);
    assert_eq!(
        e.debug_hp(1) + e.debug_hp(2),
        10,
        "qiaoqiaoSkill1 must re-arm for every new turn"
    );
}

/// Finding #10 (`extension.js:3566-3592`, CDP P8):
/// `txclanxieshu` uses Chinese card name lengths (`FireSha=1`, `Duel/huogong=2`, `txhj_huoshaolianying=4`).
#[test]
fn r2_fix10_txclanxieshu_chinese_name_lengths() {
    // 1. FireSha -> len 1
    let mut e1 = start_taixu(0x2111, 2);
    e1.debug_grant(0, "txclanxieshu");
    for _ in 0..4 {
        let _ = place_kind(&mut e1, 0, CardKind::Shan);
    }
    let fs = place_kind(&mut e1, 0, CardKind::FireSha);
    let _ = play_card(&mut e1, 0, fs, vec![1]);
    let c1 = expect_pending(&e1, "txclanxieshu:discard_cards");
    assert_eq!(c1.spec.min, 1, "FireSha Chinese name len must be 1");

    // 2. Duel -> len 2
    let mut e2 = start_taixu(0x2112, 2);
    e2.debug_grant(0, "txclanxieshu");
    for _ in 0..4 {
        let _ = place_kind(&mut e2, 0, CardKind::Shan);
    }
    let dl = place_kind_suit(&mut e2, 0, CardKind::Duel, Suit::Spade);
    let _ = play_card(&mut e2, 0, dl, vec![1]);
    let c2 = expect_pending(&e2, "txclanxieshu:discard_cards");
    assert_eq!(c2.spec.min, 2, "Duel Chinese name len must be 2");

    // 3. txhj_huoshaolianying -> len 4
    let mut e4 = start_taixu(0x2113, 2);
    e4.debug_grant(0, "txclanxieshu");
    for _ in 0..4 {
        let _ = place_kind(&mut e4, 0, CardKind::Shan);
    }
    let hs = e4.debug_move_named_card_to_hand(0, "txhj_huoshaolianying");
    let _ = play_card(&mut e4, 0, hs, Vec::new());
    let c4 = expect_pending(&e4, "txclanxieshu:discard_cards");
    assert_eq!(c4.spec.min, 4, "txhj_huoshaolianying Chinese name len must be 4");
}

/// Finding #11 (`extension.js:3207-3229, 3471-3488`, CDP P8):
/// `txlieji` is direct (`direct: true`) and ignores delayed tricks (`Le`/`Bing`),
/// while `tx_mojian` & `tx_yushou` are optional (`confirm` + decline cancels).
#[test]
fn r2_fix11_txlieji_direct_no_delay_and_mojian_yushou_optional() {
    // 1. Delayed trick (`Le`) must NOT trigger `txlieji`
    let mut e_lj = start_taixu(0x2114, 2);
    e_lj.debug_grant(0, "txlieji");
    let le = place_kind_suit(&mut e_lj, 0, CardKind::Le, Suit::Heart);
    let _ = play_card(&mut e_lj, 0, le, vec![1]);
    assert!(e_lj.open_choice().is_none(), "txlieji must not prompt and must skip delayed trick");
    drain(&mut e_lj);
    assert_eq!(e_lj.debug_hand(0).len(), 0, "delayed trick must NOT draw from txlieji");

    // Normal trick (`Dismantle`) triggers `txlieji` directly (0 confirm prompt), draws 1 & replays once
    let _d_target_1 = place_kind(&mut e_lj, 1, CardKind::Shan);
    let _d_target_2 = place_kind(&mut e_lj, 1, CardKind::Tao);
    let gh = place_kind_suit(&mut e_lj, 0, CardKind::Dismantle, Suit::Spade);
    let _ = play_card(&mut e_lj, 0, gh, vec![1]);
    assert!(e_lj.open_choice().is_none(), "direct txlieji + Dismantle needs no confirm");
    drain(&mut e_lj);
    assert_eq!(e_lj.debug_hand(0).len(), 1, "txlieji drew 1 directly without confirm");
    assert_eq!(e_lj.debug_hand(1).len(), 0, "replayed Dismantle stripped 2nd card");

    // 2. Optional tx_mojian & tx_yushou can be declined cleanly and save/restore mid-confirm
    let mut e_mj = start_versus(0x2115, &[&[0], &[1]], &[Faction::Wei, Faction::Shu]);
    e_mj.debug_grant(0, "tx_mojian");
    e_mj.debug_set_turn(0, Phase::Draw, 1);
    let _ = next_phase(&mut e_mj);
    let c_mj = expect_pending(&e_mj, "tx_mojian:confirm");
    assert!(c_mj.spec.allow_cancel);
    let e_mj_ok = save_restore_and_compare_pending(&e_mj, &c_mj, vec![ChoiceOption::Confirm]);
    assert_eq!(e_mj_ok.debug_hp(1), 5);
    let _ = answer(&mut e_mj, &c_mj, Vec::new());
    drain(&mut e_mj);
    assert_eq!(e_mj.debug_hp(1), 6, "declining tx_mojian deals 0 damage");

    let mut e_ys = start_versus(0x2116, &[&[0], &[1]], &[Faction::Wei, Faction::Shu]);
    e_ys.debug_grant(0, "tx_yushou");
    e_ys.debug_set_turn(0, Phase::Draw, 1);
    let _ = next_phase(&mut e_ys);
    let c_ys = expect_pending(&e_ys, "tx_yushou:confirm");
    assert!(c_ys.spec.allow_cancel);
    let _ = answer(&mut e_ys, &c_ys, Vec::new());
    drain(&mut e_ys);
    assert_eq!(e_ys.debug_hp(1), 6, "declining tx_yushou deals 0 damage");
}

/// Finding #12 (`extension.js:3909-3960`, CDP P10):
/// `cxyKuangXi` also seals for the turn when owner self-enters dying from step-0 `loseHp()`.
#[test]
fn r2_fix12_cxy_kuangxi_owner_self_dying_seals_skill() {
    let mut e = start_taixu(0x2117, 2);
    e.debug_grant(0, "cxyKuangXi");
    e.debug_set_hp(0, 1);
    let tao0 = place_kind_suit(&mut e, 0, CardKind::Tao, Suit::Heart);
    let _ = activate_skill(&mut e, 0, "cxyKuangXi");
    let c_tgt = expect_pending(&e, "cxyKuangXi:choose_target");
    let _ = answer(&mut e, &c_tgt, vec![ChoiceOption::Player(1)]);
    // Owner enters dying from step-0 loseHp(1) and saves self with Tao
    let c_save = answer_pass_until(&mut e, "dying_save");
    let _ = answer(&mut e, &c_save, vec![ChoiceOption::Card(tao0)]);
    drain(&mut e);
    assert_eq!(e.debug_hp(0), 1);
    assert_eq!(e.debug_hp(1), 5, "step-1 damage still resolves after owner survives");
    assert!(
        matches!(
            activate_skill(&mut e, 0, "cxyKuangXi"),
            CommandOutcome::Rejected(RejectReason::SkillDisabled | RejectReason::InvalidChoice | RejectReason::SkillAlreadyUsed)
        ),
        "owner self-dying during cxyKuangXi must seal cxyKuangXi for rest of turn"
    );

    // Next turn of seat 0 lifts turn seal
    e.debug_set_turn(0, Phase::Play, 2);
    e.debug_set_hp(0, 4);
    assert!(matches!(
        activate_skill(&mut e, 0, "cxyKuangXi"),
        CommandOutcome::Pending { .. }
    ));
}

#[allow(dead_code)]
fn _keep_role_import_used(_r: Role) {}
