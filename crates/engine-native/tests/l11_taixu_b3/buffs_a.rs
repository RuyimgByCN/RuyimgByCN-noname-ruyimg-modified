//! Batch L11-3 Group A (`extension_buff.js:2..230`, `extension_skill.js:1633..2035`) real-path tests:
//! `aozhan`, `yanhuo`, `jinglei`, `xihuo`, `dinglei`, `wuzhongshengsha`, `wuzhongshengshan`,
//! `fengyushengcai`, `jipodizhen`, `wenhezhenlue`, `wenhejibei`, `yueyingdezhihui`,
//! `changbingguangren`, `fuzu`, `xiaojideyongwu`, `xiongziyingfa`, `wuxiangdexinxue`,
//! `juguzhidao`, `pozhenzhifeng`, `yanrenzhinu`.

use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::engine::Engine;
use engine_native::{ChoiceOption, CommandOutcome, Phase};

fn put_all_kind_in_discard(e: &mut Engine, kind: CardKind) {
    let moved: Vec<u32> = e
        .state_core()
        .deck
        .iter()
        .copied()
        .filter(|&c| e.catalog().card_kind(c) == kind)
        .collect();
    e.state_core_mut().deck.retain(|c| !moved.contains(c));
    e.state_core_mut().discard.extend(moved);
}

#[test]
fn l11_b3_aozhan_and_juguzhidao_no_bogus_combat_modifiers() {
    // `extension_buff.js:2` (`buff_txhj_aozhan`): post-battle HP no-heal ONLY (no +1 hand limit).
    // `extension_buff.js:189` (`buff_txhj_juguzhidao`): shop fee helper ONLY (no +1 hujia damage).
    let mut e = start_versus(301, &[&[0], &[1]]);
    grant_skill(&mut e, 0, "buff_txhj_aozhan");
    grant_skill(&mut e, 0, "buff_txhj_juguzhidao");
    e.debug_set_hp(0, 3);
    assert_eq!(
        e.state_core().hand_limit(e.catalog(), 0),
        3,
        "buff_txhj_aozhan must NOT grant phantom +1 combat hand limit"
    );

    e.debug_set_hp(1, 5);
    e.state_core_mut().player_mut(1).hujia = 2;
    e.debug_set_turn(0, Phase::Play, 1);
    let sha = place_kind(&mut e, 0, CardKind::Sha);
    assert!(matches!(
        play_card(&mut e, 0, sha, vec![1]),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    drain(&mut e);
    assert_eq!(
        e.state_core().player(1).hujia,
        1,
        "buff_txhj_juguzhidao must deal normal 1 Sha damage (consume 1 hujia, not 2)"
    );
    assert_eq!(e.state_core().player(1).hp, 5);
}

#[test]
fn l11_b3_yanhuo_jinglei_xihuo_dinglei_and_chain_not_link() {
    // `extension_skill.js:1633..1706`:
    // `_buff_txhj_yanhuo` / `_buff_txhj_jinglei` (DamageBegin1 +1 non-chain Fire/Thunder);
    // `_buff_txhj_xihuo` / `_buff_txhj_dinglei` (DamageBegin3 -1 Fire/Thunder, 2->1 and 1->0).
    let mut e = start_versus(302, &[&[0], &[1, 2]]);
    grant_skill(&mut e, 0, "buff_txhj_yanhuo");
    grant_skill(&mut e, 0, "_buff_txhj_yanhuo"); // paired dedup: must NOT +2
    grant_skill(&mut e, 0, "_buff_txhj_jinglei");
    grant_skill(&mut e, 1, "_buff_txhj_xihuo");
    grant_skill(&mut e, 2, "_buff_txhj_dinglei");

    // 1. FireSha on seat 1 (linked to seat 2):
    // Seat 1: base 1 + yanhuo 1 (once, deduped!) - xihuo 1 = 1 damage;
    // Seat 2 (from_chain!): chain propagates 1 fire dmg WITHOUT another yanhuo +1!
    e.state_core_mut().player_mut(1).linked = true;
    e.state_core_mut().player_mut(2).linked = true;
    e.debug_set_turn(0, Phase::Play, 1);
    let fsha = place_kind(&mut e, 0, CardKind::FireSha);
    assert!(matches!(
        play_card(&mut e, 0, fsha, vec![1]),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 5);
    assert_eq!(
        e.state_core().player(2).hp,
        5,
        "chained damage must NOT re-trigger yanhuo (+1)"
    );

    // 2. Normal Sha does NOT trigger yanhuo/jinglei or xihuo/dinglei:
    e.debug_set_turn(0, Phase::Play, 2);
    e.state_core_mut().turn.sha_used_play_phase = false;
    let nsha = place_kind(&mut e, 0, CardKind::Sha);
    let _ = play_card(&mut e, 0, nsha, vec![1]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 4);

    // 3. LeiSha on seat 2: base 1 + jinglei 1 - dinglei 1 = 1 (not 0 for 2->1 reduction!)
    e.debug_set_turn(0, Phase::Play, 3);
    e.state_core_mut().turn.sha_used_play_phase = false;
    let lsha = place_kind(&mut e, 0, CardKind::LeiSha);
    let _ = play_card(&mut e, 0, lsha, vec![2]);
    drain(&mut e);
    assert_eq!(
        e.state_core().player(2).hp,
        4,
        "2 thunder damage reduced by dinglei (-1) must become 1 (not 0)"
    );

    // 4. Unboosted 1 thunder damage onto seat 2 (`dinglei` reduces 1 -> 0 prevented):
    grant_skill(&mut e, 1, "_buff_txhj_dinglei");
    e.debug_set_turn(1, Phase::Play, 4);
    e.state_core_mut().turn.sha_used_play_phase = false;
    let lsha2 = place_kind(&mut e, 1, CardKind::LeiSha);
    let _ = play_card(&mut e, 1, lsha2, vec![2]);
    drain(&mut e);
    assert_eq!(e.state_core().player(2).hp, 4);
}

#[test]
fn l11_b3_wuzhongshengsha_shan_and_fengyushengcai_phases() {
    // `extension_skill.js:1709..1775`:
    // `_buff_txhj_wuzhongshengsha` (Prepare): gain Sha from deck ONLY;
    // `_buff_txhj_wuzhongshengshan` (End): gain Shan from deck ONLY;
    // `_buff_txhj_fengyushengcai` (RoundStart / TurnBegin): draw 1 once per round if healthy.
    let mut e = start_versus(303, &[&[0], &[1]]);
    grant_skill(&mut e, 0, "_buff_txhj_wuzhongshengsha");
    grant_skill(&mut e, 0, "_buff_txhj_wuzhongshengshan");
    grant_skill(&mut e, 0, "_buff_txhj_fengyushengcai");

    e.debug_set_hp(0, 6);
    e.debug_set_max_hp(0, 6);
    e.debug_set_turn(1, Phase::End, 0);
    advance_until_turn_phase(&mut e, 0, Phase::Play);
    assert!(
        e.state_core()
            .player(0)
            .hand
            .iter()
            .any(|&c| e.catalog().card_kind(c).is_sha_like()),
        "wuzhongshengsha must gain a Sha from deck on Prepare phase"
    );
    let hand_before_seat1 = e.state_core().player(0).hand.len();

    // Advance to seat 0 End phase -> gains Shan from deck, NOT fengyushengcai again
    advance_until_turn_phase(&mut e, 0, Phase::Discard);
    let shan_before = e
        .state_core()
        .player(0)
        .hand
        .iter()
        .filter(|&&c| e.catalog().card_kind(c) == CardKind::Shan)
        .count();
    let _ = next_phase(&mut e); // Phase::End
    drain(&mut e);
    let shan_after = e
        .state_core()
        .player(0)
        .hand
        .iter()
        .filter(|&&c| e.catalog().card_kind(c) == CardKind::Shan)
        .count();
    assert!(
        shan_after >= shan_before + 1,
        "wuzhongshengshan must gain Shan on Phase::End"
    );
    let _ = hand_before_seat1;

    // Next round while injured with empty Sha in deck:
    // `wuzhongshengsha` MUST fall back to discard (`get.cardPile`) and gain 1 Sha while `fengyushengcai` skips when injured!
    e.debug_set_hp(0, 4);
    put_all_kind_in_discard(&mut e, CardKind::Sha);
    put_all_kind_in_discard(&mut e, CardKind::FireSha);
    put_all_kind_in_discard(&mut e, CardKind::LeiSha);
    let hand_injured = e.state_core().player(0).hand.len();
    e.debug_set_turn(1, Phase::End, 1);
    advance_until_turn_phase(&mut e, 0, Phase::Play);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        hand_injured + 3, // 1 Sha from discard fallback + 2 Draw phase (no fengyushengcai!)
        "injured holder gets no fengyushengcai, and empty-deck wuzhongshengsha falls back to discard"
    );

    // Same discard fallback for `wuzhongshengshan`: empty deck Shan -> gains Shan from discard on Phase::End:
    put_all_kind_in_discard(&mut e, CardKind::Shan);
    advance_until_turn_phase(&mut e, 0, Phase::Discard);
    let shan_before_fb = e
        .state_core()
        .player(0)
        .hand
        .iter()
        .filter(|&&c| e.catalog().card_kind(c) == CardKind::Shan)
        .count();
    let _ = next_phase(&mut e); // Phase::End
    drain(&mut e);
    let shan_after_fb = e
        .state_core()
        .player(0)
        .hand
        .iter()
        .filter(|&&c| e.catalog().card_kind(c) == CardKind::Shan)
        .count();
    assert!(
        shan_after_fb >= shan_before_fb + 1,
        "empty-deck wuzhongshengshan must fall back to discard"
    );

    // Negative check: when NEITHER deck NOR discard holds any Shan, Phase::End gains 0 cards:
    advance_until_turn_phase(&mut e, 0, Phase::Discard);
    let shan_cards: Vec<u32> = e
        .state_core()
        .deck
        .iter()
        .chain(e.state_core().discard.iter())
        .copied()
        .filter(|&c| e.catalog().card_kind(c) == CardKind::Shan)
        .collect();
    e.state_core_mut()
        .deck
        .retain(|c| !shan_cards.contains(c));
    e.state_core_mut()
        .discard
        .retain(|c| !shan_cards.contains(c));
    let hand_no_shan = e.state_core().player(0).hand.len();
    let _ = next_phase(&mut e); // Phase::End
    drain(&mut e);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        hand_no_shan,
        "when both deck and discard lack Shan, wuzhongshengshan draws 0"
    );
}

#[test]
fn l11_b3_jipodizhen_and_wenhezhenlue_ordinary_vs_delay_tricks() {
    // `extension_skill.js:1778..1817`:
    // `_buff_txhj_jipodizhen` (DamageEnd randomDiscard 'he');
    // `_buff_txhj_wenhezhenlue` (`get.type(card) == 'trick'` nowuxie; delay tricks still wuxie-able!).
    let mut e = start_versus(304, &[&[0], &[1]]);
    grant_skill(&mut e, 0, "_buff_txhj_jipodizhen");
    grant_skill(&mut e, 0, "_buff_txhj_wenhezhenlue");

    e.debug_set_turn(0, Phase::Play, 1);
    let _shun = place_kind(&mut e, 0, CardKind::Snatch);
    let target_junk = place_kind(&mut e, 1, CardKind::Tao);
    let target_wuxie = place_kind(&mut e, 1, CardKind::Nullify);

    // Duel vs seat 1 who holds Wuxie: wenhezhenlue makes Duel bypass NullifyWindow -> no Wuxie ask, deals 1 dmg + jipodizhen discards 1!
    let duel = place_kind(&mut e, 0, CardKind::Duel);
    let out = play_card(&mut e, 0, duel, vec![1]);
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    // Must prompt Duel sha response directly (NOT nullify_window!), verifying Wuxie skipped
    if let Some(c) = e.open_choice() {
        assert_ne!(
            c.spec.prompt, "nullify_window",
            "ordinary trick under wenhezhenlue must NOT ask Wuxie"
        );
    }
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 5);
    assert_eq!(
        e.state_core().player(1).hand.len(),
        1,
        "jipodizhen must discard 1 card from damaged target (leaving 1 of [Tao, Wuxie])"
    );
    let _ = (target_junk, target_wuxie);

    // Delayed trick `Le` is NOT `'trick'` type (upstream `'delay'`), so it CAN still be targeted & resolved separately from `wenhezhenlue`:
    grant_skill(&mut e, 0, "buff_txhj_yueyingdezhihui");
    let le = place_kind(&mut e, 0, CardKind::Le);
    assert!(matches!(
        play_card(&mut e, 0, le, vec![1]),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    drain(&mut e);
    assert_eq!(e.state_core().player(1).judge.len(), 1);
}

#[test]
fn l11_b3_wenhejibei_yueying_changbing_fuzu_xiaoji_xiongzi_wuxiang() {
    // 1. `wenhejibei` blocks delayed tricks & wraps lightning past holder;
    // 2. `yueyingdezhihui` unlimited trick/delay range; `changbingguangren` +3 attack range;
    // 3. `fuzu` draws up to max_hp once; `xiaojideyongwu` draws 2 per lost equip (!xiaoji);
    // 4. `xiongziyingfa` draws +1 & sets maxHandcardBase = max_hp BEFORE modifiers;
    // 5. `wuxiangdexinxue` +3 hand limit.
    let mut e = start_versus(305, &[&[0, 1], &[2, 3]]);
    grant_skill(&mut e, 0, "_buff_txhj_yueyingdezhihui");
    grant_skill(&mut e, 0, "_buff_txhj_changbingguangren");
    grant_skill(&mut e, 0, "_buff_txhj_fuzu");
    grant_skill(&mut e, 0, "_buff_txhj_xiaojideyongwu");
    grant_skill(&mut e, 0, "_buff_txhj_xiongziyingfa");
    grant_skill(&mut e, 0, "_buff_txhj_wuxiangdexinxue");
    grant_skill(&mut e, 1, "_buff_txhj_wenhejibei");

    // Seat 0 TurnBegin triggers gameStart helpers & fuzu draw to max_hp (=6):
    e.debug_set_max_hp(0, 6);
    e.debug_set_hp(0, 2);
    e.debug_set_turn(3, Phase::End, 0);
    advance_until_turn_phase(&mut e, 0, Phase::Play);
    assert!(has_skill(&e, 0, "txhj_yueyingdezhihui"));
    assert!(has_skill(&e, 0, "txhj_changbingguangren"));
    assert!(has_skill(&e, 0, "txhj_xiongziyingfa"));
    assert!(has_skill(&e, 0, "txhj_wuxiangdexinxue"));
    // Fuzu drew 6 at TurnBegin + DrawPhase drew (2 + 1 xiongzi) = 9
    assert_eq!(e.state_core().player(0).hand.len(), 9);
    // Attack range = base 1 + changbing 3 = 4
    assert_eq!(e.state_core().attack_range(e.catalog(), 0), 4);
    // Hand limit = maxHandcardBase(max_hp=6 via xiongziyingfa!) + 3 (wuxiangdexinxue) = 9 (NOT hp=2 + 3!)
    assert_eq!(e.state_core().hand_limit(e.catalog(), 0), 9);
    // Apply -1 base modifier (`xingtigaizao_count = 1`) -> proves maxHandcardBase runs BEFORE `-1`/`+3`: `6 - 1 + 3 = 8` (NOT `limit.max(6)`)!
    e.state_core_mut().player_mut(0).xingtigaizao_count = 1;
    assert_eq!(
        e.state_core().hand_limit(e.catalog(), 0),
        8,
        "txhj_xiongziyingfa sets maxHandcardBase BEFORE +/- modifiers (6 - 1 + 3 = 8)"
    );
    e.state_core_mut().player_mut(0).xingtigaizao_count = 0;

    // Distance-2 trick (`Snatch` on seat 2, distance 2!) succeeds via yueyingdezhihui:
    let eq2 = equip_kind(&mut e, 2, CardKind::Armor);
    let snatch = place_kind(&mut e, 0, CardKind::Snatch);
    assert!(matches!(
        play_card(&mut e, 0, snatch, vec![2]),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    drain(&mut e);
    assert!(
        e.state_core().player(0).hand.contains(&eq2),
        "yueyingdezhihui allows Snatch at distance 2"
    );

    // Seat 1 has `wenhejibei` -> `Le`, `ShanDian`, `HongShui`, and `Guiyoujie` targeting seat 1 are ALL REJECTED!
    let le = place_kind(&mut e, 0, CardKind::Le);
    let bad_le = play_card(&mut e, 0, le, vec![1]);
    assert!(matches!(bad_le, CommandOutcome::Rejected { .. }));

    // Self `ShanDian` on seat 1 with `wenhejibei` is REJECTED:
    e.debug_set_turn(1, Phase::Play, 2);
    let sd_self = place_kind(&mut e, 1, CardKind::ShanDian);
    assert!(
        matches!(
            play_card(&mut e, 1, sd_self, Vec::new()),
            CommandOutcome::Rejected { .. }
        ),
        "wenhejibei must block self ShanDian"
    );
    e.debug_set_turn(0, Phase::Play, 2);

    // Multi-pack (`taixu` + `shahai-swd` + `shahai-yunchou`) check for `HongShui` & `Guiyoujie` with `wenhejibei`:
    {
        use content_native_loader::{
            prepare_locked_content_with_bridge, start_locked_engine, validate_and_freeze_raw_package,
            LockedStartMode,
        };
        let load_pkg = |dir_name: &str| {
            let pkg_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../packages/extensions")
                .join(dir_name);
            let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
                .expect("read npack")
                .replace("\r\n", "\n");
            let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
            let mut assets = std::collections::BTreeMap::new();
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
            validate_and_freeze_raw_package(raw_json, assets).expect("freeze")
        };
        let locked = prepare_locked_content_with_bridge(vec![
            load_pkg("taixu-upstream-liuli"),
            load_pkg("shahai-swd-upstream-noname"),
            load_pkg("shahai-yunchou-upstream-noname"),
        ])
        .expect("prepare combined locked");
        let mut ew = start_locked_engine(3051, 3, LockedStartMode::Prototype, &locked)
            .expect("start combined");
        for s in 0..3 {
            ew.debug_clear_hand(s);
            ew.debug_set_max_hp(s, 6);
            ew.debug_set_hp(s, 6);
        }
        let give_or_place_kind = |eng: &mut Engine, seat: u8, kind: CardKind| -> u32 {
            if let Some(cid) = eng
                .state_core()
                .deck
                .iter()
                .chain(eng.state_core().discard.iter())
                .copied()
                .find(|&c| eng.catalog().card_kind(c) == kind)
            {
                eng.state_core_mut().deck.retain(|&c| c != cid);
                eng.state_core_mut().discard.retain(|&c| c != cid);
                eng.state_core_mut().player_mut(seat).hand.push(cid);
                cid
            } else {
                let cid = (0..eng.catalog().cards.len() as u32)
                    .find(|&i| eng.catalog().cards[i as usize].kind == kind)
                    .unwrap_or_else(|| panic!("no catalog card for {kind:?}"));
                eng.state_core_mut().player_mut(seat).hand.push(cid);
                cid
            }
        };
        grant_skill(&mut ew, 1, "_buff_txhj_wenhejibei");
        // Baseline without wenhejibei (seat 2 / seat 0): HongShui + Guiyoujie targeting seat 2 succeed;
        // Seat 1 WITH wenhejibei: HongShui on self (seat 1) AND Guiyoujie targeting seat 1 are REJECTED!
        ew.debug_set_turn(0, Phase::Play, 1);
        let gyj_ok = give_or_place_kind(&mut ew, 0, CardKind::Guiyoujie);
        assert!(matches!(
            play_card(&mut ew, 0, gyj_ok, vec![2]),
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ));
        drain(&mut ew);
        let gyj_bad = give_or_place_kind(&mut ew, 0, CardKind::Guiyoujie);
        assert!(
            matches!(
                play_card(&mut ew, 0, gyj_bad, vec![1]),
                CommandOutcome::Rejected { .. }
            ),
            "wenhejibei must block Guiyoujie targeting holder"
        );
        let hs_ok = give_or_place_kind(&mut ew, 0, CardKind::HongShui);
        assert!(matches!(
            play_card(&mut ew, 0, hs_ok, Vec::new()),
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ));
        drain(&mut ew);
        ew.debug_set_turn(1, Phase::Play, 2);
        let hs_bad = give_or_place_kind(&mut ew, 1, CardKind::HongShui);
        assert!(
            matches!(
                play_card(&mut ew, 1, hs_bad, Vec::new()),
                CommandOutcome::Rejected { .. }
            ),
            "wenhejibei must block self HongShui"
        );
    }

    // `fuzu` trim test (`max_hp == 3`, opening hand == 4 -> silently returns excess 1 card to deck without discard triggers!):
    {
        let mut ef = start_versus(3052, &[&[0], &[1]]);
        grant_skill(&mut ef, 0, "_buff_txhj_fuzu");
        grant_skill(&mut ef, 0, "_buff_txhj_yingjichuoshijia"); // Armor discard trigger MUST NOT fire on fuzu trim!
        ef.debug_set_max_hp(0, 3);
        ef.debug_set_hp(0, 2); // injured so cuoshijia WOULD notice any discard trigger!
        let _h0 = place_kind(&mut ef, 0, CardKind::Armor);
        let _h1 = place_kind(&mut ef, 0, CardKind::Sha);
        let _h2 = place_kind(&mut ef, 0, CardKind::Shan);
        let _h3 = place_kind(&mut ef, 0, CardKind::Tao);
        assert_eq!(ef.state_core().player(0).hand.len(), 4);
        let deck_before = ef.state_core().deck.len();
        ef.debug_set_turn(1, Phase::End, 0);
        // Trigger TurnBegin on seat 0 (Reach Prepare, BEFORE DrawPhase draws 2):
        let _ = next_phase(&mut ef);
        drain(&mut ef);
        assert!(ef.open_choice().is_none(), "fuzu trim must NOT open discard triggers");
        assert_eq!(ef.state_core().player(0).hp, 2, "fuzu trim must NOT trigger armor heal");
        assert_eq!(ef.state_core().player(0).hand.len(), 3, "fuzu must trim 4 opening hand down to max_hp 3");
        assert_eq!(ef.state_core().deck.len(), deck_before + 1, "trimmed card must return to deck cleanly");
    }
    e.debug_set_turn(0, Phase::Play, 2);

    // Seat 0 loses an equip -> xiaojideyongwu draws 2 (whereas with `xiaoji` does not double-fire)
    e.debug_clear_hand(0);
    equip_std_kind(&mut e, 0, CardKind::Weapon);
    let h_before = e.state_core().player(0).hand.len();
    assert_eq!(h_before, 0);
    let dismantle = place_std_kind(&mut e, 2, CardKind::Dismantle);
    e.debug_set_turn(2, Phase::Play, 2);
    let _ = play_card(&mut e, 2, dismantle, vec![0]);
    drain(&mut e);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        h_before + 2,
        "xiaojideyongwu must draw 2 on losing equip"
    );

    // Negative check: when `xiaoji` (`枭姬`) is held, `_buff_txhj_xiaojideyongwu` must NOT fire on equip loss (`CardsLost`), leaving hand at 0
    e.debug_clear_hand(0);
    grant_skill(&mut e, 0, engine_native::vocabulary::names::XIAO_JI);
    equip_std_kind(&mut e, 0, CardKind::Armor);
    drain(&mut e);
    e.debug_clear_hand(0);
    let dismantle2 = place_std_kind(&mut e, 2, CardKind::Dismantle);
    let out2 = play_card(&mut e, 2, dismantle2, vec![0]);
    assert!(matches!(out2, CommandOutcome::Accepted { .. }), "{out2:?}");
    assert_eq!(
        e.state_core().player(0).hand.len(),
        0,
        "when player already holds xiaoji, _buff_txhj_xiaojideyongwu must NOT fire"
    );
}

#[test]
fn l11_b3_pozhenzhifeng_and_yanrenzhinu_gating() {
    // `extension_skill.js:1976..2035` + `3346..3372`:
    // `pozhenzhifeng`: ONLY Play phase + enemy + weapon/feilong ignores armor (NOT infinite range, NOT ally, NOT unarmed);
    // `yanrenzhinu`: ONLY 2nd Sha in Play phase + weapon is directHit (NOT 1st or 3rd, NOT unarmed).
    let mut e = start_versus(306, &[&[0], &[1]]);
    grant_skill(&mut e, 0, "_buff_txhj_pozhenzhifeng");
    grant_skill(&mut e, 0, "_buff_txhj_yanrenzhinu");
    grant_skill(&mut e, 0, engine_native::vocabulary::names::PAO_XIAO); // allows >1 Sha to distinguish 1st vs 2nd vs 3rd!

    // Give target Bagua (Armor: Red judge dodges) + Shan in hand
    equip_std_kind(&mut e, 1, CardKind::Armor);
    e.debug_set_turn(1, Phase::End, 0);
    advance_until_turn_phase(&mut e, 0, Phase::Play);

    // Without weapon: neither pozhenzhifeng nor yanrenzhinu applies -> 1st & 2nd Sha ARE blocked by Bagua heart!
    assert!(!has_skill(&e, 0, "txhj_pozhenzhifeng"));
    let sha1 = place_kind(&mut e, 0, CardKind::Sha);
    top_deck_suit(&mut e, Suit::Heart); // Bagua succeeds
    let _ = play_card(&mut e, 0, sha1, vec![1]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 6, "unarmed must not bypass Bagua");

    let sha2_unarmed = place_kind(&mut e, 0, CardKind::Sha);
    top_deck_suit(&mut e, Suit::Heart);
    let _ = play_card(&mut e, 0, sha2_unarmed, vec![1]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 6, "unarmed 2nd Sha must NOT trigger yanrenzhinu");

    // Next turn, equip Weapon DURING Play -> _buff_txhj_pozhenzhifeng dynamically grants `txhj_pozhenzhifeng`!
    e.debug_set_turn(0, Phase::Play, 2);
    // Reset sha count for clean round 2 play phase
    e.state_core_mut().turn.sha_used_play_phase = false;
    e.state_core_mut()
        .vm_aux
        .flags
        .remove(&(0, "l11_phase_sha_count".to_string()));
    let _wep = equip_std_kind(&mut e, 0, CardKind::Weapon);
    assert!(
        has_skill(&e, 0, "txhj_pozhenzhifeng"),
        "equipping weapon in Play phase must activate txhj_pozhenzhifeng"
    );

    // 1st Sha WITH weapon: Bagua ignored (pozhenzhifeng), Shan STILL usable -> target plays Shan & survives!
    let _shan1 = place_kind(&mut e, 1, CardKind::Shan);
    let sha_a = place_kind(&mut e, 0, CardKind::Sha);
    let _ = play_card(&mut e, 0, sha_a, vec![1]);
    let ch_shan = expect_pending(&e, "respond_shan");
    assert!(ch_shan.spec.options.iter().any(|o| matches!(o, ChoiceOption::Card(_))));
    let _ = answer(&mut e, &ch_shan, vec![ChoiceOption::Card(_shan1)]); // dodge with Shan
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 6);

    // 2nd Sha WITH weapon: BOTH pozhenzhifeng (no Bagua) AND yanrenzhinu (directHit -> NO Shan prompt!) hit!
    let _shan2 = place_kind(&mut e, 1, CardKind::Shan);
    let sha_b = place_kind(&mut e, 0, CardKind::Sha);
    let out_b = play_card(&mut e, 0, sha_b, vec![1]);
    assert!(
        matches!(out_b, CommandOutcome::Accepted { .. }),
        "2nd Sha with yanrenzhinu must direct-hit without opening respond_shan: {out_b:?}"
    );
    assert_eq!(e.state_core().player(1).hp, 5);

    // 3rd Sha WITH weapon: pozhenzhifeng still ignores Bagua, yanrenzhinu does NOT apply -> Shan prompt opens!
    let sha_c = place_kind(&mut e, 0, CardKind::Sha);
    let _ = play_card(&mut e, 0, sha_c, vec![1]);
    let ch_shan3 = expect_pending(&e, "respond_shan");
    let _ = answer(&mut e, &ch_shan3, vec![ChoiceOption::Pass]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 4);

    // Leaving Play phase revokes temporary `txhj_pozhenzhifeng`!
    advance_until_turn_phase(&mut e, 0, Phase::Discard);
    assert!(
        !has_skill(&e, 0, "txhj_pozhenzhifeng"),
        "txhj_pozhenzhifeng must be removed after Phase::Play ends"
    );

    // Unarmed 1st Sha -> equip Weapon -> 2nd Sha with BOTH `yanrenzhinu` + `ziyideyongzhan`:
    // 1st unarmed Sha increments kernel `l11_phase_sha_count` to 1 (without firing either skill),
    // and equipped 2nd Sha reads count == 2 for BOTH without double-incrementing!
    let mut e_comb = start_versus(3061, &[&[0], &[1]]);
    grant_skill(&mut e_comb, 0, "_buff_txhj_yanrenzhinu");
    grant_skill(&mut e_comb, 0, "_buff_txhj_ziyideyongzhan");
    grant_skill(&mut e_comb, 0, engine_native::vocabulary::names::PAO_XIAO);
    e_comb.debug_set_turn(0, Phase::Play, 1);
    let s1_unarmed = place_kind(&mut e_comb, 0, CardKind::Sha);
    let _shan_s1 = place_kind(&mut e_comb, 1, CardKind::Shan);
    let _ = play_card(&mut e_comb, 0, s1_unarmed, vec![1]);
    let c_s1 = expect_pending(&e_comb, "respond_shan");
    let _ = answer(&mut e_comb, &c_s1, vec![ChoiceOption::Card(_shan_s1)]);
    drain(&mut e_comb);
    assert_eq!(e_comb.state_core().player(1).hp, 6);

    let _ = equip_std_kind(&mut e_comb, 0, CardKind::Weapon);
    let _shan_s2 = place_kind(&mut e_comb, 1, CardKind::Shan);
    let s2_armed = place_kind(&mut e_comb, 0, CardKind::Sha);
    let out_s2 = play_card(&mut e_comb, 0, s2_armed, vec![1]);
    assert!(
        matches!(out_s2, CommandOutcome::Accepted { .. }),
        "2nd Sha after unarmed 1st Sha with both yanrenzhinu+ziyideyongzhan must direct-hit for 2 dmg: {out_s2:?}"
    );
    assert_eq!(
        e_comb.state_core().player(1).hp,
        4,
        "both yanrenzhinu (directHit) and ziyideyongzhan (+1 dmg = 2) must trigger on 2nd Sha without double-incrementing"
    );
}
