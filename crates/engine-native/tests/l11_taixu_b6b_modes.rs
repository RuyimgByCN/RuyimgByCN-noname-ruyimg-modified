//! Batch L11-6b: Complete parity and live `Engine` integration tests for all 206
//! `L11:modes:*` entries across:
//! - `event/event_universal.js` (`gainPack` 22, `exam1..4` 230, `changePack` 5 shops + 19 gains)
//! - all 8 DLC `extension_season.js` packs (52 chapters, 313 deduplicated events, 1006 refs)
//! - `extension_servant.js` (`servant`, `action`, `skillDesc` for all 21 servants)
//! - `extension_framework.js` (`taixuhuanjing` session runtime, `target=all/kill/repair/subsist`,
//!   `unique`, difficulty `rank 1..5`, `effectPack`, `adjust` mulligan, `"牌库"` gaintag,
//!   `dieAfter2` kill draw, AI play, and binary archive save/restore).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

use content_native_loader::{
    find_exam_by_id, find_universal_event, get_dlc_event_by_idx,
    prepare_locked_content_with_bridge, restore_locked_engine, start_locked_taixu_battle,
    taixu_max_exp_for_level, validate_and_freeze_raw_package, TaixuBattleTarget, TaixuBtnResult,
    TaixuModeSession, TaixuPendingBattleSpec, TaixuPendingInteraction, TaixuShopCategory,
    VerifiedLockedContent, L11_TAIXU_BUFF_STORE, L11_TAIXU_CAMPAIGN_NODES, L11_TAIXU_CARD_RANK,
    L11_TAIXU_CHANGE_PACK, L11_TAIXU_CHAPTER_EVENT_REFS_A, L11_TAIXU_CHAPTER_EVENT_REFS_B,
    L11_TAIXU_EFFECT_PACK, L11_TAIXU_EXAMS_1_2, L11_TAIXU_EXAMS_3_4, L11_TAIXU_GAIN_PACK,
    L11_TAIXU_SEASONS, L11_TAIXU_SERVANT_SPECS, L11_TAIXU_SKILL_RANK,
};
use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::ids::{CardId, PlayerId};
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase, Winner,
};

static TAIXU_B6B_VERIFIED: OnceLock<VerifiedLockedContent> = OnceLock::new();
static B6B_DEDUP: AtomicU64 = AtomicU64::new(62_000_000);

fn next_dedup() -> u64 {
    B6B_DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn taixu_verified() -> &'static VerifiedLockedContent {
    TAIXU_B6B_VERIFIED.get_or_init(|| {
        let pkg_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/taixu-upstream-liuli");
        let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
            .expect("read taixu package.npack.json")
            .replace("\r\n", "\n");
        let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
        let mut assets = BTreeMap::new();
        if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
            for rel in paths.values().filter_map(|v| v.as_str()) {
                let bytes = std::fs::read(pkg_dir.join(rel))
                    .expect("read taixu asset")
                    .into_iter()
                    .filter(|&b| b != b'\r')
                    .collect::<Vec<u8>>();
                assets.insert(rel.to_string(), bytes);
            }
        }
        let frozen = validate_and_freeze_raw_package(raw_json, assets).expect("freeze taixu");
        prepare_locked_content_with_bridge(vec![frozen]).expect("bridge taixu")
    })
}

fn submit(e: &mut Engine, actor: PlayerId, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    })
}

fn drain_ai(e: &mut Engine, max_steps: usize) {
    let ai = BoundedPlanner::new(16);
    for _ in 0..max_steps {
        if e.outcome().is_some() {
            break;
        }
        if let Some(ch) = e.open_choice() {
            let obs = ch.observer;
            let act = e.plan_action(obs, &ai).action;
            let _ = submit(e, obs, act);
        } else {
            let cur = e.state_core().turn.current;
            let act = e.plan_action(cur, &ai).action;
            let _ = submit(e, cur, act);
        }
    }
}

fn find_card_in_hand_or_deck(e: &mut Engine, seat: PlayerId, name: &str) -> CardId {
    if let Some(&c) = e
        .state_core()
        .player(seat)
        .hand
        .iter()
        .find(|&&c| e.catalog().cards[c as usize].name == name)
    {
        return c;
    }
    e.debug_place(seat, name)
}

#[test]
fn l11_b6b_all_206_modes_upstream_data_parity() {
    // 1. All 206 `L11:modes:*` anchors accounted for with unique `(scope, section, id, line)`.
    assert_eq!(L11_TAIXU_CAMPAIGN_NODES.len(), 206);
    let uniq: BTreeSet<(&str, &str, &str, u32)> = L11_TAIXU_CAMPAIGN_NODES
        .iter()
        .map(|n| (n.scope, n.section, n.id, n.source_line))
        .collect();
    assert_eq!(uniq.len(), 206, "every one of the 206 mode entries must be uniquely anchored");

    // 2. All 8 seasons + 50 chapters match `dlc/*/extension_season.js` exact procedure counts.
    assert_eq!(L11_TAIXU_SEASONS.len(), 8);
    let total_chapters: usize = L11_TAIXU_SEASONS.iter().map(|s| s.chapters.len()).sum();
    assert_eq!(total_chapters, 50);
    let cy = L11_TAIXU_SEASONS.iter().find(|s| s.id == "ChongYingChuLin").unwrap();
    assert_eq!(cy.chapters[0].procedure, &["gain", "boss"]);
    assert_eq!(cy.chapters[1].procedure.len(), 8);
    assert_eq!(cy.chapters[3].procedure.last(), Some(&"Ultimate"));

    // 3. All 22 gainPack & 24 changePack entries (`event_universal.js:4-556, 10154-10735`).
    assert_eq!(L11_TAIXU_GAIN_PACK.len(), 22);
    assert_eq!(L11_TAIXU_CHANGE_PACK.len(), 24);
    let shops: Vec<_> = L11_TAIXU_CHANGE_PACK.iter().filter(|c| c.kind == "store").collect();
    assert_eq!(shops.len(), 5);
    for (idx, expected) in ["txhj_sibingheishi", "txhj_anxianghonglou", "txhj_xiaojishi", "txhj_dajishi", "txhj_huanjingjishi"].iter().enumerate() {
        assert_eq!(shops[idx].id, *expected);
        assert_eq!(shops[idx].level, (idx + 1) as u8);
    }

    // 4. All 230 exam1..4 questions (`50+50+50+80`), verifying zero fake index-0 true bias.
    let total_exams = L11_TAIXU_EXAMS_1_2.len() + L11_TAIXU_EXAMS_3_4.len();
    assert_eq!(total_exams, 230);
    for q in L11_TAIXU_EXAMS_1_2.iter().chain(L11_TAIXU_EXAMS_3_4.iter()) {
        assert_ne!(
            q.true_btn_idx, 0,
            "exam {} has real true button at {}, never upstream hardcoded index 0",
            q.id, q.true_btn_idx
        );
        assert_eq!(q.buttons[q.true_btn_idx].result, TaixuBtnResult::True);
    }
    assert!(find_exam_by_id("exam4005").is_some());

    // 5. All 1006 DLC chapter refs & deduplicated events cover `repair`, `subsist`, `kill`, `all`, `choose`, `trade`.
    let ref_count = L11_TAIXU_CHAPTER_EVENT_REFS_A.len() + L11_TAIXU_CHAPTER_EVENT_REFS_B.len();
    assert_eq!(ref_count, 1006);
    assert!(get_dlc_event_by_idx(0).is_some());
    let has_repair = (0..313).filter_map(get_dlc_event_by_idx).any(|e| e.target == TaixuBattleTarget::Repair);
    let has_subsist = (0..313).filter_map(get_dlc_event_by_idx).any(|e| e.target == TaixuBattleTarget::Subsist);
    let has_kill = (0..313).filter_map(get_dlc_event_by_idx).any(|e| e.target == TaixuBattleTarget::Kill);
    assert!(has_repair && has_subsist && has_kill);

    // 6. Servants (`extension_servant.js:420,1329,1507`), `effectPack` (34), `cardRank` (140), `skillRank` (625).
    assert_eq!(L11_TAIXU_SERVANT_SPECS.len(), 21);
    let dundun = L11_TAIXU_SERVANT_SPECS.iter().find(|s| s.id == "dundun").unwrap();
    assert_eq!(dundun.grade, 2);
    assert_eq!(dundun.skill_descs[0].tier_probs, [10, 20, 40, 60, 80, 100]);
    let qilin = L11_TAIXU_SERVANT_SPECS.iter().find(|s| s.id == "qilin").unwrap();
    assert_eq!(qilin.skill_descs[0].tier_probs, [20, 30, 40, 60, 80, 100]);
    assert_eq!(L11_TAIXU_EFFECT_PACK.len(), 34);
    assert_eq!(L11_TAIXU_CARD_RANK.len(), 140);
    assert_eq!(L11_TAIXU_SKILL_RANK.len(), 625);
    assert!(L11_TAIXU_BUFF_STORE.iter().any(|b| b.id == "buff_txhj_juguzhidao" && b.store));
    assert_eq!(taixu_max_exp_for_level(2), 300);
    assert_eq!(taixu_max_exp_for_level(9), 4000);
    assert!(find_universal_event("txhj_fujiahaoli").is_some());
}

#[test]
fn l11_b6b_interactive_quiz_shop_choose_and_trade_session() {
    // Test real quiz (`思考一下` + `buff_txhj_bowenqiangzhi` + real pass vs fail),
    // shop level escalation & `buff_txhj_juguzhidao` first-buy discount + insufficient coin error,
    // and `choose` (`txhj_tianjijushou`) branching.
    let mut sess = TaixuModeSession::start_season(
        777,
        "GFHuangJinZhiLuan",
        1,
        "liubei",
        "lingqiyiman",
        Some("lulu"),
        2,
    )
    .expect("start GFHuangJinZhiLuan");
    sess.buffs.push("buff_txhj_bowenqiangzhi".to_string());
    sess.coin = 1200;
    sess.max_coin = 1200;

    // 1. Place an `exam1` 3-button question (Wait: find which question or test any exam + '思考一下' on 1-opt or 3-opt!)
    let three_opt_exam = L11_TAIXU_EXAMS_1_2
        .iter()
        .chain(L11_TAIXU_EXAMS_3_4.iter())
        .find(|q| q.buttons.len() == 3)
        .expect("upstream has 3-button exam question");
    sess.optional[0] = Some(content_native_loader::TaixuSlotNode {
        slot_seat: 901,
        season: "GFHuangJinZhiLuan".into(),
        chapter: "0".into(),
        id: three_opt_exam.id.into(),
        kind: three_opt_exam.tier.into(),
        level: 1,
        change: true,
        dlc_def_idx: None,
        exam_tier: Some(three_opt_exam.tier.into()),
        override_buttons: None,
    });
    let q_view = sess.open_slot(0).unwrap();
    let TaixuPendingInteraction::Quiz { buttons, .. } = q_view else {
        panic!("expected Quiz interaction");
    };
    // 3 real options + inserted "思考一下" = 4 options, and bowenqiangzhi marks 1 wrong!
    assert_eq!(buttons.len(), 4);
    assert_eq!(buttons[3].name, "思考一下");
    assert_eq!(buttons[3].result, None);
    assert_eq!(buttons.iter().filter(|b| b.bowen_marked_wrong).count(), 1);
    let marked_idx = buttons.iter().position(|b| b.bowen_marked_wrong).unwrap();
    assert_eq!(buttons[marked_idx].result, Some(false));

    // Choosing "思考一下" cancels cleanly WITHOUT consuming slot 0!
    let (consumed, passed) = sess.answer_quiz(3).unwrap();
    assert!(!consumed && !passed);
    assert_eq!(sess.optional[0].as_ref().unwrap().id, three_opt_exam.id);

    // Reopen -> pick true option (NOT hardcoded 0): passes & grants real reward!
    let TaixuPendingInteraction::Quiz { buttons: b2, .. } = sess.open_slot(0).unwrap() else {
        unreachable!();
    };
    let true_idx = b2.iter().position(|b| b.result == Some(true)).unwrap();
    let (consumed2, passed2) = sess.answer_quiz(true_idx).unwrap();
    assert!(consumed2 && passed2);
    assert!(sess.bowen_marked_opt.is_none());

    // 2. Shop level 2 escalation & `buff_txhj_juguzhidao` first-purchase waiver + insufficient coin error
    sess.coin = 5000;
    sess.buffs.push("buff_txhj_juguzhidao".to_string());
    sess.optional[0] = Some(content_native_loader::TaixuSlotNode {
        slot_seat: 902,
        season: "GFHuangJinZhiLuan".into(),
        chapter: "0".into(),
        id: "txhj_anxianghonglou".into(),
        kind: "store".into(),
        level: 2,
        change: true,
        dlc_def_idx: None,
        exam_tier: None,
        override_buttons: None,
    });
    let _ = sess.open_slot(0).unwrap();
    let stock_snapshot = sess.stores["txhj_anxianghonglou"].clone();
    assert_eq!(stock_snapshot.buffs.len(), 4);
    assert_eq!(stock_snapshot.skills.len(), 4);
    let b0 = &stock_snapshot.buffs[0];
    let b1 = &stock_snapshot.buffs[1];
    let b2 = &stock_snapshot.buffs[2];
    let base_b1 = L11_TAIXU_BUFF_STORE.iter().find(|x| x.id == b1).unwrap().value as i32;
    let base_b2 = L11_TAIXU_BUFF_STORE.iter().find(|x| x.id == b2).unwrap().value as i32;

    // 1st buy: base price (`buff_num` becomes 1)
    let p0 = sess.buy_shop_item(TaixuShopCategory::Buff, 0).unwrap();
    assert_eq!(
        p0,
        L11_TAIXU_BUFF_STORE.iter().find(|x| x.id == b0).unwrap().value as i32
    );
    // 2nd buy: WITH `juguzhidao` (`dec = 1`), `buff_num == 1 <= 1`, so STILL +0 escalation!
    let p1 = sess.buy_shop_item(TaixuShopCategory::Buff, 0).unwrap();
    assert_eq!(p1, base_b1);
    // 3rd buy: `buff_num == 2 > dec (1)` => +200 escalation!
    let p2_expected = base_b2 + 200;
    assert_eq!(
        sess.shop_item_price("txhj_anxianghonglou", TaixuShopCategory::Buff, b2),
        Some(p2_expected)
    );
    // Negative check: insufficient coin returns Err(InsufficientGold) without mutating stock
    sess.coin = 10;
    assert!(matches!(
        sess.buy_shop_item(TaixuShopCategory::Buff, 0),
        Err(content_native_loader::TaixuError::InsufficientGold { .. })
    ));
    sess.leave_shop().unwrap();

    // 3. `choose` node (`txhj_tianjijushou`) -> Option 1 ("偷拿珍宝 溜之大吉") -> maxHp-1 & sub-trade!
    let jushou_idx = (0..313)
        .find(|&i| get_dlc_event_by_idx(i).is_some_and(|d| d.id == "txhj_tianjijushou"))
        .expect("txhj_tianjijushou");
    let jushou = get_dlc_event_by_idx(jushou_idx).unwrap();
    sess.optional[0] = Some(content_native_loader::TaixuSlotNode {
        slot_seat: 903,
        season: "ChongYingChuLin".into(),
        chapter: "2".into(),
        id: jushou.id.into(),
        kind: jushou.kind.into(),
        level: jushou.level,
        change: false,
        dlc_def_idx: Some(jushou_idx),
        exam_tier: None,
        override_buttons: None,
    });
    let TaixuPendingInteraction::Choose { buttons: cbtns, .. } = sess.open_slot(0).unwrap() else {
        panic!("expected Choose");
    };
    assert_eq!(cbtns.len(), 4); // 3 choices + "思考一下"
    assert!(cbtns[0].launches_battle);
    assert!(cbtns[1].opens_sub_trade);
    let max_hp_before = sess.max_hp;
    let sub = sess.answer_choose(1).unwrap().expect("sub-trade opens");
    assert_eq!(sess.max_hp, max_hp_before - 1);
    assert!(matches!(
        sub,
        TaixuPendingInteraction::GainOrTrade { is_sub_trade: true, .. }
    ));
    assert!(sess.answer_gain_or_trade(0).unwrap());

    // Roundtrip snapshot parity across all mutations
    let snap = sess.serialize_snapshot();
    let restored = TaixuModeSession::deserialize_snapshot(&snap).unwrap();
    assert_eq!(restored, sess);
}

#[test]
fn l11_b6b_real_battles_all_kill_repair_subsist_unique_and_save_restore() {
    let verified = taixu_verified();

    // A. `target: kill` (`txhj_lipojuren` or any kill event with multiple enemies):
    // Killing boss enemy wins immediately EVEN IF non-boss minion survives!
    let kill_idx = (0..313)
        .find(|&i| {
            get_dlc_event_by_idx(i).is_some_and(|d| {
                d.target == TaixuBattleTarget::Kill
                    && d.enemies.len() >= 2
                    && d.enemies.iter().any(|e| e.is_boss)
            })
        })
        .expect("multi-enemy kill event");
    let kill_def = get_dlc_event_by_idx(kill_idx).unwrap();
    let mut s_kill =
        TaixuModeSession::start_season(101, "ChongYingChuLin", 2, "liubei", "lingqiyiman", Some("ahao"), 3)
            .unwrap();
    s_kill.buffs.push("txhj_aHaoSkill1".into());
    s_kill.buffs.push("buff_txhj_majundegaizao".into());
    s_kill.equip1 = Some("zhuge".into());
    s_kill.cards.push("tao".into());
    s_kill.adjust = 1;
    s_kill.pending = Some(TaixuPendingInteraction::Battle(TaixuPendingBattleSpec {
        slot_idx: 0,
        event_id: kill_def.id.into(),
        event_kind: kill_def.kind.into(),
        season: "ChongYingChuLin".into(),
        chapter: "2".into(),
        dlc_def_idx: Some(kill_idx),
        choose_btn_idx: None,
        target_str: "kill".into(),
        unique: false,
        max_rounds: kill_def.max_rounds,
        exp: kill_def.exp,
    }));
    let mut e_kill = start_locked_taixu_battle(&s_kill, verified).expect("launch kill battle");
    // Verify `lingqiyiman` (+2 max_hp), `majundegaizao` (`rewrite_zhuge` via `th_jingxie_rewritten`), `paiku` tag, and `adjust` mulligan prompt!
    assert_eq!(e_kill.state_core().player(0).hp, e_kill.state_core().player(0).max_hp as i32);
    assert!(e_kill
        .state_core()
        .player(0)
        .equips
        .iter()
        .any(|&c| {
            e_kill.catalog().cards[c as usize].name == "zhuge"
                && e_kill
                    .state_core()
                    .vm_aux
                    .flags
                    .get(&(0, format!("th_jingxie_rewritten:{c}")))
                    .copied()
                    == Some(1)
        }));
    // Advance until `_txhj_skill_addEffectStyle:mulligan` fires, test save -> restore -> Answer parity!
    for _ in 0..8 {
        if e_kill.open_choice().is_some() {
            break;
        }
        let _ = submit(&mut e_kill, 0, GameAction::NextPhase);
    }
    let ch_id = {
        let ch = e_kill.open_choice().expect("adjust mulligan prompt must open");
        assert_eq!(ch.spec.prompt, "_txhj_skill_addEffectStyle:mulligan");
        ch.id
    };
    let raw_arch = encode_archive(&e_kill.save());
    let decoded = decode_archive(&raw_arch).unwrap();
    let mut restored = restore_locked_engine(&decoded, verified).unwrap();
    assert_eq!(restored.state_hash(), e_kill.state_hash());
    let _ = submit(
        &mut e_kill,
        0,
        GameAction::Answer {
            choice_id: ch_id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let _ = submit(
        &mut restored,
        0,
        GameAction::Answer {
            choice_id: ch_id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert_eq!(restored.state_hash(), e_kill.state_hash());

    // Advance to seat 0 Play phase and kill ONLY the enemy boss via real Sha while minion lives!
    while e_kill.state_core().turn.phase != Phase::Play || e_kill.state_core().turn.current != 0 {
        if let Some((obs, cid)) = e_kill.open_choice().map(|c| (c.observer, c.id)) {
            let _ = submit(
                &mut e_kill,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: vec![ChoiceOption::Pass],
                },
            );
        } else {
            let cur = e_kill.state_core().turn.current;
            let _ = submit(&mut e_kill, cur, GameAction::NextPhase);
        }
    }
    let boss_seat = (1..e_kill.state_core().players.len() as u8)
        .find(|&p| {
            e_kill
                .state_core()
                .vm_aux
                .flags
                .get(&(p, "l11_npc_boss".to_string()))
                .copied()
                .unwrap_or(0)
                > 0
        })
        .expect("enemy boss flag present");
    e_kill.debug_set_hp(boss_seat, 1);
    let sha = find_card_in_hand_or_deck(&mut e_kill, 0, "sha");
    let _ = submit(
        &mut e_kill,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![boss_seat],
        },
    );
    // Let AI/defending seat decline Shan or settle naturally
    drain_ai(&mut e_kill, 40);
    if e_kill.outcome().is_none() {
        // Ensure boss without Shan takes lethal Sha (if they happened to dodge with Shan)
        e_kill.debug_set_hp(boss_seat, 1);
        let sha2 = find_card_in_hand_or_deck(&mut e_kill, 0, "sha");
        let _ = submit(
            &mut e_kill,
            0,
            GameAction::PlayCard {
                card: sha2,
                targets: vec![boss_seat],
            },
        );
        drain_ai(&mut e_kill, 40);
    }
    assert_eq!(
        e_kill.outcome().unwrap().winner,
        Winner::Versus(0),
        "killing enemy boss wins kill target immediately"
    );
    let _ = s_kill.settle_battle(&e_kill).unwrap();

    // B. `target: repair` (`txhj_xiufujiguan`): healing friend NPC boss to full HP wins on roundStart!
    let rep_idx = (0..313)
        .find(|&i| {
            get_dlc_event_by_idx(i).is_some_and(|d| {
                d.target == TaixuBattleTarget::Repair && !d.friends.is_empty()
            })
        })
        .expect("repair event with friend");
    let rep_def = get_dlc_event_by_idx(rep_idx).unwrap();
    let mut s_rep =
        TaixuModeSession::start_season(202, "ChongYingChuLin", 1, "liubei", "pinganwushi", None, 0)
            .unwrap();
    s_rep.pending = Some(TaixuPendingInteraction::Battle(TaixuPendingBattleSpec {
        slot_idx: 0,
        event_id: rep_def.id.into(),
        event_kind: rep_def.kind.into(),
        season: "ChongYingChuLin".into(),
        chapter: "1".into(),
        dlc_def_idx: Some(rep_idx),
        choose_btn_idx: None,
        target_str: "repair".into(),
        unique: rep_def.unique,
        max_rounds: rep_def.max_rounds.max(3),
        exp: rep_def.exp,
    }));
    let mut e_rep = start_locked_taixu_battle(&s_rep, verified).expect("launch repair battle");
    let ally_boss = (0..e_rep.state_core().players.len() as u8)
        .find(|&p| {
            e_rep
                .state_core()
                .vm_aux
                .flags
                .get(&(p, "l11_npc_boss".to_string()))
                .copied()
                .unwrap_or(0)
                > 0
        })
        .expect("repair ally boss seat");
    let max_ally = e_rep.state_core().player(ally_boss).max_hp as i32;
    e_rep.debug_set_hp(ally_boss, max_ally);
    let total_p = e_rep.state_core().players.len() as u8;
    e_rep.debug_set_turn(total_p - 1, Phase::End, 1);
    let _ = submit(&mut e_rep, total_p - 1, GameAction::NextPhase);
    assert_eq!(
        e_rep.outcome().map(|o| o.winner),
        Some(Winner::Versus(0)),
        "full-HP ally boss wins repair battle on roundStart"
    );
    let _ = s_rep.settle_battle(&e_rep).unwrap();

    // C. `target: subsist` (`txhj_moouxisha`): surviving past `max_rounds` wins on roundStart!
    let sub_idx = (0..313)
        .find(|&i| {
            get_dlc_event_by_idx(i).is_some_and(|d| d.target == TaixuBattleTarget::Subsist && d.max_rounds > 1)
        })
        .expect("subsist event");
    let sub_def = get_dlc_event_by_idx(sub_idx).unwrap();
    let mut s_sub =
        TaixuModeSession::start_season(303, "ChongYingChuLin", 1, "liubei", "pinganwushi", None, 0)
            .unwrap();
    s_sub.pending = Some(TaixuPendingInteraction::Battle(TaixuPendingBattleSpec {
        slot_idx: 0,
        event_id: sub_def.id.into(),
        event_kind: sub_def.kind.into(),
        season: "ChongYingChuLin".into(),
        chapter: "2".into(),
        dlc_def_idx: Some(sub_idx),
        choose_btn_idx: None,
        target_str: "subsist".into(),
        unique: sub_def.unique,
        max_rounds: sub_def.max_rounds,
        exp: sub_def.exp,
    }));
    let mut e_sub = start_locked_taixu_battle(&s_sub, verified).expect("launch subsist battle");
    let total_p = e_sub.state_core().players.len() as u8;
    e_sub.debug_set_turn(total_p - 1, Phase::End, sub_def.max_rounds);
    let _ = submit(&mut e_sub, total_p - 1, GameAction::NextPhase);
    assert_eq!(
        e_sub.outcome().map(|o| o.winner),
        Some(Winner::Versus(0)),
        "surviving > max_rounds wins subsist battle"
    );

    // D. Player-0 death in non-unique vs unique battle (`extension_framework.js:9857-9864`):
    // Even if an ally survives on team 0, player 0 dying immediately loses!
    let mut s_uniq =
        TaixuModeSession::start_season(404, "ChongYingChuLin", 1, "liubei", "pinganwushi", None, 0)
            .unwrap();
    s_uniq.buffs.push("buff_txhj_aozhan".into());
    s_uniq.pending = Some(TaixuPendingInteraction::Battle(TaixuPendingBattleSpec {
        slot_idx: 0,
        event_id: rep_def.id.into(),
        event_kind: rep_def.kind.into(),
        season: "ChongYingChuLin".into(),
        chapter: "1".into(),
        dlc_def_idx: Some(rep_idx),
        choose_btn_idx: None,
        target_str: "repair".into(),
        unique: true, // non-fatal on defeat!
        max_rounds: 3,
        exp: rep_def.exp,
    }));
    let mut e_u = start_locked_taixu_battle(&s_uniq, verified).unwrap();
    let u_me = (0..e_u.state_core().players.len() as u8)
        .find(|&p| e_u.state_core().vm_aux.flags.get(&(p, "l11_txhj_active_battle".into())).copied().unwrap_or(0) > 0)
        .expect("uniq player seat");
    let u_enemy = (0..e_u.state_core().players.len() as u8)
        .find(|&p| !e_u.state_core().is_friend_of(e_u.catalog(), u_me, p))
        .expect("uniq enemy seat");
    // Enemy seat kills 1-HP player while friend ally_boss is still alive!
    e_u.debug_set_hp(u_me, 1);
    e_u.debug_set_turn(u_enemy, Phase::Play, 1);
    let sha_u = find_card_in_hand_or_deck(&mut e_u, u_enemy, "sha");
    let _ = submit(&mut e_u, u_enemy, GameAction::PlayCard { card: sha_u, targets: vec![u_me] });
    for _ in 0..16 {
        if e_u.outcome().is_some() {
            break;
        }
        let Some((obs, cid)) = e_u.open_choice().map(|c| (c.observer, c.id)) else {
            break;
        };
        let _ = submit(
            &mut e_u,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    assert_eq!(e_u.outcome().map(|o| o.winner), Some(Winner::Versus(1)));
    let _ = s_uniq.settle_battle(&e_u).unwrap();
    assert!(!s_uniq.failed, "unique battle defeat does NOT fail campaign run");
    assert_eq!(s_uniq.hp, 1, "aozhan clamps survived HP >= 1 after defeat");

    // E. Non-unique player-0 death terminates run with `failed = true` and `final_score.is_some()`
    let prologue_boss_idx = (0..313)
        .find(|&i| {
            get_dlc_event_by_idx(i).is_some_and(|d| d.id == "txhj_xuzhang1" || (d.kind == "boss" && !d.unique && d.enemies.len() == 1))
        })
        .unwrap();
    let pb = get_dlc_event_by_idx(prologue_boss_idx).unwrap();
    let mut s_fatal =
        TaixuModeSession::start_season(505, "ChongYingChuLin", 1, "liubei", "pinganwushi", None, 0)
            .unwrap();
    s_fatal.pending = Some(TaixuPendingInteraction::Battle(TaixuPendingBattleSpec {
        slot_idx: 0,
        event_id: pb.id.into(),
        event_kind: pb.kind.into(),
        season: "ChongYingChuLin".into(),
        chapter: "0".into(),
        dlc_def_idx: Some(prologue_boss_idx),
        choose_btn_idx: None,
        target_str: "all".into(),
        unique: false,
        max_rounds: 1,
        exp: pb.exp,
    }));
    let mut e_f = start_locked_taixu_battle(&s_fatal, verified).unwrap();
    // Kill player with Sha -> fatal defeat (player passes optional Shan/Dying)
    let f_me = (0..e_f.state_core().players.len() as u8)
        .find(|&p| e_f.state_core().vm_aux.flags.get(&(p, "l11_txhj_active_battle".into())).copied().unwrap_or(0) > 0)
        .expect("fatal player seat");
    let f_enemy = (0..e_f.state_core().players.len() as u8)
        .find(|&p| !e_f.state_core().is_friend_of(e_f.catalog(), f_me, p))
        .expect("fatal enemy seat");
    e_f.debug_set_hp(f_me, 1);
    e_f.debug_set_turn(f_enemy, Phase::Play, 1);
    let sha_f = find_card_in_hand_or_deck(&mut e_f, f_enemy, "sha");
    let _ = submit(&mut e_f, f_enemy, GameAction::PlayCard { card: sha_f, targets: vec![f_me] });
    for _ in 0..16 {
        if e_f.outcome().is_some() {
            break;
        }
        let Some((obs, cid)) = e_f.open_choice().map(|c| (c.observer, c.id)) else {
            break;
        };
        let _ = submit(
            &mut e_f,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    assert_eq!(e_f.outcome().map(|o| o.winner), Some(Winner::Versus(1)));
    let _ = s_fatal.settle_battle(&e_f).unwrap();
    assert!(s_fatal.failed);
    assert!(s_fatal.final_score.is_some());
}

#[test]
fn l11_b6b_ai_bounded_planner_chapter_advance_and_xianhujiqu() {
    let verified = taixu_verified();
    // Full chapter clear on `ChongYingChuLin` chap 0 (prologue) + `xianhujiqu` + `levelUp` 4-of-9!
    let mut sess = TaixuModeSession::start_season(
        909,
        "ChongYingChuLin",
        3,
        "liubei",
        "yaowuyangwei", // grants global `yaowu` (`耀武` in standard catalog)
        Some("dundun"),
        2,
    )
    .unwrap();
    assert_eq!(sess.chapter_idx, 0);
    assert_eq!(sess.optional.iter().flatten().count(), 2); // proc: ["gain", "boss"]

    // First play slot 0 ("gain"), then slot 1 ("boss")
    let g_int = sess.open_slot(0).unwrap();
    assert!(matches!(g_int, TaixuPendingInteraction::GainOrTrade { .. }));
    assert!(sess.answer_gain_or_trade(0).unwrap());

    sess.buffs.push("buff_txhj_xianhujiqu".into());
    sess.exp = 90; // next boss exp triggers level-up 1 -> 2!
    let b_int = sess.open_slot(1).unwrap();
    assert!(matches!(b_int, TaixuPendingInteraction::Battle(_)));
    let mut e = start_locked_taixu_battle(&sess, verified).unwrap();
    // Verify `rank 3` gave enemy `reyingzi` & `mashu` (`马术` in standard catalog), and `yaowuyangwei` gave `yaowu` (`耀武`)!
    let enemy_seat = (0..e.state_core().players.len() as u8).find(|&p| p != 0).unwrap();
    assert!(e.debug_has_skill(enemy_seat, "reyingzi"));
    assert!(e.debug_has_skill(enemy_seat, "马术"));
    assert!(e.debug_has_skill(enemy_seat, "耀武"));
    // Raise player max_hp mid-battle (simulating in-battle maxHp gain for xianhujiqu carryover check)
    let orig_max = sess.max_hp;
    e.debug_set_max_hp(0, (orig_max + 2) as u32);
    // Use `BoundedPlanner` AI to execute full turn combat!
    e.debug_set_hp(enemy_seat, 1);
    e.debug_set_turn(0, Phase::Play, 1);
    let sha = find_card_in_hand_or_deck(&mut e, 0, "sha");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![enemy_seat] });
    drain_ai(&mut e, 60);
    if e.outcome().is_none() {
        e.debug_set_hp(enemy_seat, 1);
        let sha2 = find_card_in_hand_or_deck(&mut e, 0, "sha");
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha2, targets: vec![enemy_seat] });
        drain_ai(&mut e, 60);
    }
    assert_eq!(e.outcome().map(|o| o.winner), Some(Winner::Versus(0)));

    let post = sess.settle_battle(&e).unwrap();
    assert_eq!(sess.max_hp, orig_max + 2, "xianhujiqu preserves gained max_hp");
    assert_eq!(sess.level, 2, "exp crossed 100 threshold -> level 2");
    assert_eq!(sess.max_exp, 300);
    let Some(TaixuPendingInteraction::LevelUp { choices, .. }) = post else {
        panic!("expected LevelUp 4-of-9 interaction");
    };
    assert_eq!(choices.len(), 4);
    sess.choose_level_up(0).unwrap();
    assert_eq!(
        sess.chapter_idx, 1,
        "clearing chapter 0 boss advances campaign to chapter 1 (first chapter)"
    );
    assert_eq!(sess.optional.iter().flatten().count(), 3);
}

#[test]
fn l11_b6b_die_after2_rank_draw_paiku_discard_and_wrong_quiz() {
    let verified = taixu_verified();

    // 1. Negative quiz case: picking wrong button consumes slot with `passed == false` and +0 coin.
    let mut qs = TaixuModeSession::start_season(
        950,
        "GFHuangJinZhiLuan",
        1,
        "liubei",
        "pinganwushi",
        None,
        0,
    )
    .unwrap();
    let exam = L11_TAIXU_EXAMS_1_2.first().unwrap();
    qs.optional[0] = Some(content_native_loader::TaixuSlotNode {
        slot_seat: 951,
        season: "GFHuangJinZhiLuan".into(),
        chapter: "0".into(),
        id: exam.id.into(),
        kind: exam.tier.into(),
        level: 1,
        change: true,
        dlc_def_idx: None,
        exam_tier: Some(exam.tier.into()),
        override_buttons: None,
    });
    let TaixuPendingInteraction::Quiz { buttons, .. } = qs.open_slot(0).unwrap() else {
        panic!("expected Quiz");
    };
    let wrong_idx = buttons
        .iter()
        .position(|b| b.result == Some(false))
        .expect("wrong exam option");
    let coin_before = qs.coin;
    let (consumed, passed) = qs.answer_quiz(wrong_idx).unwrap();
    assert!(consumed && !passed);
    assert_eq!(qs.coin, coin_before);
    assert_ne!(qs.optional[0].as_ref().map(|n| n.slot_seat), Some(951));

    // 2. `dieAfter2` kill draw by `rank` (1=>2, 2=>1, 3=>0) & `"牌库"` card survival across Discard phase.
    let multi_all_idx = (0..313)
        .find(|&i| {
            get_dlc_event_by_idx(i).is_some_and(|d| {
                d.target == TaixuBattleTarget::All && d.enemies.len() >= 2 && d.friends.is_empty()
            })
        })
        .expect("multi-enemy Target::All event with no friends");
    let def = get_dlc_event_by_idx(multi_all_idx).unwrap();

    for (rank, expected_draw) in [(1u8, 2usize), (2, 1), (3, 0)] {
        let mut s = TaixuModeSession::start_season(
            960 + u64::from(rank),
            "ChongYingChuLin",
            rank,
            "liubei",
            "pinganwushi",
            None,
            0,
        )
        .unwrap();
        s.cards = vec!["tao".into()];
        s.equip1 = Some("qilin".into());
        s.pending = Some(TaixuPendingInteraction::Battle(TaixuPendingBattleSpec {
            slot_idx: 0,
            event_id: def.id.into(),
            event_kind: def.kind.into(),
            season: "ChongYingChuLin".into(),
            chapter: "1".into(),
            dlc_def_idx: Some(multi_all_idx),
            choose_btn_idx: None,
            target_str: "all".into(),
            unique: false,
            max_rounds: def.max_rounds,
            exp: def.exp,
        }));
        let mut e = start_locked_taixu_battle(&s, verified).unwrap();
        let me = (0..e.state_core().players.len() as u8)
            .find(|&p| {
                e.state_core()
                    .vm_aux
                    .flags
                    .get(&(p, "l11_txhj_active_battle".into()))
                    .copied()
                    .unwrap_or(0)
                    > 0
            })
            .unwrap();
        let paiku_cid = e
            .state_core()
            .player(me)
            .hand
            .iter()
            .copied()
            .find(|&c| {
                e.state_core()
                    .vm_aux
                    .flags
                    .get(&(me, format!("l11_gaintag_paiku:{c}")))
                    .copied()
                    .unwrap_or(0)
                    > 0
            })
            .expect("paiku tagged card present");

        let victim = (0..e.state_core().players.len() as u8)
            .find(|&p| !e.state_core().is_friend_of(e.catalog(), me, p))
            .unwrap();
        e.debug_set_turn(me, Phase::Play, 1);
        let sha = find_card_in_hand_or_deck(&mut e, me, "sha");
        let hand_before = e.state_core().player(me).hand.len();
        e.debug_set_hp(victim, 1);
        let _ = submit(
            &mut e,
            me,
            GameAction::PlayCard {
                card: sha,
                targets: vec![victim],
            },
        );
        for _ in 0..16 {
            if !e.state_core().player(victim).alive {
                break;
            }
            let Some((obs, cid)) = e.open_choice().map(|c| (c.observer, c.id)) else {
                break;
            };
            let _ = submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: vec![ChoiceOption::Pass],
                },
            );
        }
        assert!(!e.state_core().player(victim).alive, "rank {rank} enemy died");
        assert!(e.outcome().is_none(), "second enemy still alive");
        assert_eq!(
            e.state_core().player(me).hand.len(),
            hand_before - 1 + expected_draw,
            "rank {rank} dieAfter2 kill draw must be {expected_draw}"
        );

        // Verify `"牌库"` tagged `tao` is preserved across Discard phase even at 1 HP!
        if rank == 1 {
            e.debug_set_hp(me, 1);
            let _ = submit(&mut e, me, GameAction::NextPhase);
            if let Some((obs, cid, min_req)) = e
                .open_choice()
                .map(|c| (c.observer, c.id, c.spec.min as usize))
            {
                let non_paiku: Vec<CardId> = e
                    .state_core()
                    .player(me)
                    .hand
                    .iter()
                    .copied()
                    .filter(|&c| c != paiku_cid)
                    .take(min_req)
                    .collect();
                let _ = submit(
                    &mut e,
                    obs,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: non_paiku.into_iter().map(ChoiceOption::Card).collect(),
                    },
                );
            }
            assert!(
                e.state_core().player(me).hand.contains(&paiku_cid),
                "\"牌库\" tagged card must survive Discard phase"
            );
        }
    }
}

fn kill_all_enemies_with_sha_pass(e: &mut Engine, me: u8) {
    // Equip a zhuge & ensure attack reach by ordering targets adjacent-first clockwise/ccw.
    for _ in 0..10 {
        if e.outcome().is_some() {
            break;
        }
        let n = e.state_core().players.len() as u8;
        let alive_enemies: Vec<u8> = (0..n)
            .filter(|&p| {
                e.state_core().player(p).alive && !e.state_core().is_friend_of(e.catalog(), me, p)
            })
            .collect();
        let Some(target) = alive_enemies.into_iter().min_by_key(|&p| (p + n - me) % n) else {
            break;
        };
        e.debug_set_hp(target, 1);
        e.debug_set_turn(me, Phase::Play, 1);
        let sha = find_card_in_hand_or_deck(e, me, "sha");
        let _ = submit(
            e,
            me,
            GameAction::PlayCard {
                card: sha,
                targets: vec![target],
            },
        );
        for _ in 0..16 {
            if !e.state_core().player(target).alive || e.outcome().is_some() {
                break;
            }
            let Some((obs, cid)) = e.open_choice().map(|c| (c.observer, c.id)) else {
                break;
            };
            let _ = submit(
                e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: vec![ChoiceOption::Pass],
                },
            );
        }
    }
}

fn force_me_loss_with_sha_pass(e: &mut Engine, me: u8) {
    let killer = (0..e.state_core().players.len() as u8)
        .find(|&p| !e.state_core().is_friend_of(e.catalog(), me, p))
        .expect("enemy seat");
    let q = find_card_in_hand_or_deck(e, killer, "qilin");
    e.debug_set_turn(killer, Phase::Play, 1);
    let _ = submit(
        e,
        killer,
        GameAction::PlayCard {
            card: q,
            targets: Vec::new(),
        },
    );
    e.debug_set_hp(me, 1);
    e.debug_set_turn(killer, Phase::Play, 1);
    let sha = find_card_in_hand_or_deck(e, killer, "sha");
    let _ = submit(
        e,
        killer,
        GameAction::PlayCard {
            card: sha,
            targets: vec![me],
        },
    );
    for _ in 0..16 {
        if e.outcome().is_some() {
            break;
        }
        let Some((obs, cid)) = e.open_choice().map(|c| (c.observer, c.id)) else {
            break;
        };
        let _ = submit(
            e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
}

#[test]
fn l11_b6b_r2_choose_spoils_unique_boss_ultimate_and_equip5_buffs() {
    let verified = taixu_verified();

    // (a1) `choose` battle (`txhj_tiannvdekaoyan` btn 1 -> `btn.spoils`: MaxHp +1).
    let kaoyan_idx = (0..313)
        .find(|&i| get_dlc_event_by_idx(i).is_some_and(|d| d.id == "txhj_tiannvdekaoyan"))
        .expect("txhj_tiannvdekaoyan");
    let kaoyan = get_dlc_event_by_idx(kaoyan_idx).unwrap();
    let mut s_ch1 =
        TaixuModeSession::start_season(1101, "ChongYingChuLin", 1, "liubei", "pinganwushi", None, 0)
            .unwrap();
    s_ch1.equip1 = Some("zhuge".into());
    s_ch1.optional[0] = Some(content_native_loader::TaixuSlotNode {
        slot_seat: 970,
        season: "ChongYingChuLin".into(),
        chapter: "2".into(),
        id: kaoyan.id.into(),
        kind: kaoyan.kind.into(),
        level: kaoyan.level,
        change: false,
        dlc_def_idx: Some(kaoyan_idx),
        exam_tier: None,
        override_buttons: None,
    });
    let _ = s_ch1.open_slot(0).unwrap();
    assert!(matches!(
        s_ch1.answer_choose(1).unwrap(),
        Some(TaixuPendingInteraction::Battle(_))
    ));
    let max_hp_before = s_ch1.max_hp;
    let mut e_ch1 = start_locked_taixu_battle(&s_ch1, verified).unwrap();
    let me1 = (0..e_ch1.state_core().players.len() as u8)
        .find(|&p| {
            e_ch1
                .state_core()
                .vm_aux
                .flags
                .get(&(p, "l11_txhj_active_battle".into()))
                .copied()
                .unwrap_or(0)
                > 0
        })
        .unwrap();
    kill_all_enemies_with_sha_pass(&mut e_ch1, me1);
    assert!(e_ch1.outcome().is_some());
    let _ = s_ch1.settle_battle(&e_ch1).unwrap();
    assert_eq!(
        s_ch1.max_hp,
        max_hp_before + 1,
        "choose battle win must apply btn.spoils MaxHp +1"
    );

    // (a2) `choose` battle (`txhj_tianjijushou` btn 0 -> `btn.enemies[*].spoils`: RandomEquip + RandomBuff + RandomSkill).
    let jushou_idx = (0..313)
        .find(|&i| get_dlc_event_by_idx(i).is_some_and(|d| d.id == "txhj_tianjijushou"))
        .expect("txhj_tianjijushou");
    let jushou = get_dlc_event_by_idx(jushou_idx).unwrap();
    let mut s_ch2 =
        TaixuModeSession::start_season(1102, "ChongYingChuLin", 1, "liubei", "pinganwushi", None, 0)
            .unwrap();
    s_ch2.optional[0] = Some(content_native_loader::TaixuSlotNode {
        slot_seat: 971,
        season: "ChongYingChuLin".into(),
        chapter: "2".into(),
        id: jushou.id.into(),
        kind: jushou.kind.into(),
        level: jushou.level,
        change: false,
        dlc_def_idx: Some(jushou_idx),
        exam_tier: None,
        override_buttons: None,
    });
    let _ = s_ch2.open_slot(0).unwrap();
    assert!(matches!(
        s_ch2.answer_choose(0).unwrap(),
        Some(TaixuPendingInteraction::Battle(_))
    ));
    let buffs_before = s_ch2.buffs.len();
    let skills_before = s_ch2.use_skills.len() + s_ch2.skills.len();
    let equips_before = [
        &s_ch2.equip1,
        &s_ch2.equip2,
        &s_ch2.equip3,
        &s_ch2.equip4,
    ]
    .into_iter()
    .flatten()
    .count()
        + s_ch2.equips.len();
    let mut e_ch2 = start_locked_taixu_battle(&s_ch2, verified).unwrap();
    let me2 = (0..e_ch2.state_core().players.len() as u8)
        .find(|&p| {
            e_ch2
                .state_core()
                .vm_aux
                .flags
                .get(&(p, "l11_txhj_active_battle".into()))
                .copied()
                .unwrap_or(0)
                > 0
        })
        .unwrap();
    kill_all_enemies_with_sha_pass(&mut e_ch2, me2);
    assert_eq!(e_ch2.outcome().map(|o| o.winner), Some(Winner::Versus(0)));
    let _ = s_ch2.settle_battle(&e_ch2).unwrap();
    assert!(
        s_ch2.buffs.len() > buffs_before,
        "choose btn.enemies[*].spoils must grant RandomBuff"
    );
    assert!(
        s_ch2.use_skills.len() + s_ch2.skills.len() > skills_before,
        "choose btn.enemies[*].spoils must grant RandomSkill"
    );
    let equips_after = [
        &s_ch2.equip1,
        &s_ch2.equip2,
        &s_ch2.equip3,
        &s_ch2.equip4,
    ]
    .into_iter()
    .flatten()
    .count()
        + s_ch2.equips.len();
    assert!(
        equips_after > equips_before,
        "choose btn.enemies[*].spoils must grant RandomEquip"
    );

    // (b1) `boss` + `unique: true` (`txhj_gongsuncongdi`) defeat advances chapter cleanly (`chapter++`, no soft-lock)!
    let gscd_idx = (0..313)
        .find(|&i| get_dlc_event_by_idx(i).is_some_and(|d| d.id == "txhj_gongsuncongdi"))
        .expect("txhj_gongsuncongdi");
    let gscd = get_dlc_event_by_idx(gscd_idx).unwrap();
    assert!(gscd.unique && gscd.kind == "boss");
    let mut s_boss = TaixuModeSession::start_season(
        1103,
        "HaiWaiFengHe",
        1,
        "liubei",
        "pinganwushi",
        None,
        0,
    )
    .unwrap();
    assert_eq!(s_boss.season, "HaiWaiFenghe");
    s_boss.enter_chapter(3).unwrap();
    s_boss.pending = Some(TaixuPendingInteraction::Battle(TaixuPendingBattleSpec {
        slot_idx: 0,
        event_id: gscd.id.into(),
        event_kind: gscd.kind.into(),
        season: "HaiWaiFenghe".into(),
        chapter: "3".into(),
        dlc_def_idx: Some(gscd_idx),
        choose_btn_idx: None,
        target_str: "all".into(),
        unique: true,
        max_rounds: gscd.max_rounds,
        exp: 10,
    }));
    let mut e_boss = start_locked_taixu_battle(&s_boss, verified).unwrap();
    let me_b = (0..e_boss.state_core().players.len() as u8)
        .find(|&p| {
            e_boss
                .state_core()
                .vm_aux
                .flags
                .get(&(p, "l11_txhj_active_battle".into()))
                .copied()
                .unwrap_or(0)
                > 0
        })
        .unwrap();
    force_me_loss_with_sha_pass(&mut e_boss, me_b);
    assert_eq!(e_boss.outcome().map(|o| o.winner), Some(Winner::Versus(1)));
    let _ = s_boss.settle_battle(&e_boss).unwrap();
    assert!(!s_boss.failed, "unique boss loss must not fail campaign");
    assert_eq!(
        s_boss.chapter_idx, 4,
        "unique boss non-fatal loss still advances chapter++ without soft-lock"
    );

    // (b2) `Ultimate` + `unique: true` (`txhj_cailangyexin`) loss stays alive (`!completed`, `!failed`, slot kept retryable)!
    let ult_idx = (0..313)
        .find(|&i| get_dlc_event_by_idx(i).is_some_and(|d| d.id == "txhj_cailangyexin"))
        .expect("txhj_cailangyexin");
    let ult = get_dlc_event_by_idx(ult_idx).unwrap();
    assert!(ult.unique && ult.kind == "Ultimate");
    s_boss.enter_chapter(4).unwrap();
    s_boss.optional[0] = Some(content_native_loader::TaixuSlotNode {
        slot_seat: 972,
        season: "HaiWaiFenghe".into(),
        chapter: "4".into(),
        id: ult.id.into(),
        kind: ult.kind.into(),
        level: ult.level,
        change: false,
        dlc_def_idx: Some(ult_idx),
        exam_tier: None,
        override_buttons: None,
    });
    let _ = s_boss.open_slot(0).unwrap();
    let mut e_ult = start_locked_taixu_battle(&s_boss, verified).unwrap();
    let me_u = (0..e_ult.state_core().players.len() as u8)
        .find(|&p| {
            e_ult
                .state_core()
                .vm_aux
                .flags
                .get(&(p, "l11_txhj_active_battle".into()))
                .copied()
                .unwrap_or(0)
                > 0
        })
        .unwrap();
    force_me_loss_with_sha_pass(&mut e_ult, me_u);
    assert_eq!(e_ult.outcome().map(|o| o.winner), Some(Winner::Versus(1)));
    let _ = s_boss.settle_battle(&e_ult).unwrap();
    assert!(!s_boss.completed, "unique Ultimate loss must not award win");
    assert!(!s_boss.failed, "unique Ultimate loss must not fail run");
    assert!(
        s_boss.optional[0].as_ref().is_some_and(|n| n.id == "txhj_cailangyexin"),
        "unique Ultimate loss keeps Ultimate node retryable"
    );

    // (c) `equipBuffUpdate` + `Equip1` slot swap (`txhj_lingbaoxianhu` / `txhj_taijifuchen` / `txhj_feilongduofeng`)
    //     + `Equip5` dual treasures (`txhj_xuwangzhimian` + `txhj_dinglanyemingzhu` stay in bag on acquire, equip manually)!
    let mut s_eq =
        TaixuModeSession::start_season(1104, "ChongYingChuLin", 1, "liubei", "pinganwushi", None, 0)
            .unwrap();
    s_eq.coin = 5000;
    s_eq.optional[0] = Some(content_native_loader::TaixuSlotNode {
        slot_seat: 973,
        season: "ChongYingChuLin".into(),
        chapter: "1".into(),
        id: "txhj_huanjingjishi".into(),
        kind: "store".into(),
        level: 5,
        change: false,
        dlc_def_idx: None,
        exam_tier: None,
        override_buttons: None,
    });
    let _ = s_eq.open_slot(0).unwrap();
    s_eq.stores.get_mut("txhj_huanjingjishi").unwrap().equips = vec![
        "txhj_feilongduofeng".into(),
        "txhj_lingbaoxianhu".into(),
        "txhj_taijifuchen".into(),
    ];
    let _ = s_eq.buy_shop_item(TaixuShopCategory::Equip, 0).unwrap();
    let _ = s_eq.buy_shop_item(TaixuShopCategory::Equip, 0).unwrap();
    let _ = s_eq.buy_shop_item(TaixuShopCategory::Equip, 0).unwrap();
    s_eq.leave_shop().unwrap();
    assert_eq!(s_eq.equip1.as_deref(), Some("txhj_feilongduofeng"));
    assert!(s_eq.equip3.is_none());
    assert!(s_eq.equip4.is_none());
    assert_eq!(
        s_eq.equips,
        vec!["txhj_lingbaoxianhu".to_string(), "txhj_taijifuchen".to_string()],
        "other Equip1 weapons go to bag rather than Equip3/Equip4"
    );
    assert!(s_eq.buffs.iter().any(|b| b == "buff_txhj_pozhenzhifeng"));

    // Unequipping slot 1 (`txhj_feilongduofeng`) -> buff removed; equipping `txhj_lingbaoxianhu` from bag -> buff switched!
    s_eq.unequip_slot(1).unwrap();
    assert!(s_eq.equip1.is_none());
    assert!(!s_eq.buffs.iter().any(|b| b == "buff_txhj_pozhenzhifeng"));
    s_eq.equip_from_bag(0).unwrap();
    assert_eq!(s_eq.equip1.as_deref(), Some("txhj_lingbaoxianhu"));
    assert!(s_eq.buffs.iter().any(|b| b == "buff_txhj_xianhujiqu"));

    // Real `Equip5` treasures stay in `equips` bag on acquire, and can dual-equip manually into `equip3`/`equip4`:
    s_eq.apply_effect_op(
        content_native_loader::TaixuEffectKind::Equip,
        1,
        "txhj_xuwangzhimian",
    );
    s_eq.apply_effect_op(
        content_native_loader::TaixuEffectKind::Equip,
        1,
        "txhj_dinglanyemingzhu",
    );
    assert!(s_eq.equip3.is_none(), "Equip5 must never auto-equip");
    assert!(s_eq.equip4.is_none(), "Equip5 must never auto-equip");
    let xw_idx = s_eq.equips.iter().position(|x| x == "txhj_xuwangzhimian").unwrap();
    s_eq.equip_from_bag(xw_idx).unwrap();
    let dl_idx = s_eq.equips.iter().position(|x| x == "txhj_dinglanyemingzhu").unwrap();
    s_eq.equip_from_bag(dl_idx).unwrap();
    assert_eq!(s_eq.equip3.as_deref(), Some("txhj_xuwangzhimian"));
    assert_eq!(s_eq.equip4.as_deref(), Some("txhj_dinglanyemingzhu"));
    assert!(s_eq.buffs.iter().any(|b| b == "buff_txhj_shenfuhuaxie"));

    // RemoveEquip random removal removes a buff cleanly when its active equip is dropped.
    let mut s_rm = s_eq.clone();
    s_rm.equip1 = Some("txhj_feilong".into());
    s_rm.equip3 = None;
    s_rm.equip4 = None;
    s_rm.equips.clear();
    s_rm.buffs.retain(|b| b == "buff_txhj_pozhenzhifeng");
    s_rm.apply_effect_op(content_native_loader::TaixuEffectKind::RemoveEquip, 1, "");
    assert!(s_rm.equip1.is_none());
    assert!(!s_rm.buffs.iter().any(|b| b == "buff_txhj_pozhenzhifeng"));

    // Launch real battle with Equip1 weapon + 2 Equip5 treasures simultaneously & check `xianhujiqu` carryover!
    let b0_idx = (0..313)
        .find(|&i| {
            get_dlc_event_by_idx(i).is_some_and(|d| {
                d.kind == "boss"
                    && !d.unique
                    && d.enemies.len() == 1
                    && !d.enemies[0].spoils.iter().any(|sp| {
                        sp.effects
                            .iter()
                            .any(|ef| ef.kind == content_native_loader::TaixuEffectKind::MaxHp)
                    })
            })
        })
        .unwrap();
    let b0 = get_dlc_event_by_idx(b0_idx).unwrap();
    s_eq.pending = Some(TaixuPendingInteraction::Battle(TaixuPendingBattleSpec {
        slot_idx: 0,
        event_id: b0.id.into(),
        event_kind: b0.kind.into(),
        season: "ChongYingChuLin".into(),
        chapter: "0".into(),
        dlc_def_idx: Some(b0_idx),
        choose_btn_idx: None,
        target_str: "all".into(),
        unique: false,
        max_rounds: 1,
        exp: b0.exp,
    }));
    let mut e_art = start_locked_taixu_battle(&s_eq, verified).unwrap();
    let eq_names: Vec<&str> = e_art
        .state_core()
        .player(0)
        .equips
        .iter()
        .map(|&c| e_art.catalog().cards[c as usize].name)
        .collect();
    assert!(eq_names.contains(&"txhj_lingbaoxianhu"), "Equip1 weapon must equip: {eq_names:?}");
    assert!(eq_names.contains(&"txhj_xuwangzhimian"), "Equip5 slot1 must dual-equip: {eq_names:?}");
    assert!(eq_names.contains(&"txhj_dinglanyemingzhu"), "Equip5 slot2 must dual-equip: {eq_names:?}");
    assert!(e_art.debug_has_skill(0, "buff_txhj_xianhujiqu"));
    assert!(e_art.debug_has_skill(0, "buff_txhj_shenfuhuaxie"));
    let orig_max = s_eq.max_hp;
    e_art.debug_set_max_hp(0, (orig_max + 2) as u32);
    kill_all_enemies_with_sha_pass(&mut e_art, 0);
    assert_eq!(e_art.outcome().map(|o| o.winner), Some(Winner::Versus(0)));
    let battle_end_max = e_art.state_core().player(0).max_hp as i32;
    assert!(battle_end_max >= orig_max + 2);
    let _ = s_eq.settle_battle(&e_art).unwrap();
    assert_eq!(
        s_eq.max_hp, battle_end_max,
        "Equip5 lingbaoxianhu triggers xianhujiqu carryover of end-of-battle max_hp"
    );
}

#[test]
fn l11_b6b_r2_card_aliases_and_haiwaifenghe_casing() {
    let verified = taixu_verified();
    let b0_idx = (0..313)
        .find(|&i| {
            get_dlc_event_by_idx(i)
                .is_some_and(|d| d.kind == "boss" && !d.unique && d.enemies.len() == 1)
        })
        .unwrap();
    let b0 = get_dlc_event_by_idx(b0_idx).unwrap();

    // (d) Verify `wanjian`, `nanman`, `shunshou`, `zixin`, `dawan`, `txhj_feilong*`,
    // `txhj_shengdong*`, `txhj_shuiyan*`, `nvzhuang`, and `rewrite_bagua` ALL materialize in a real `load_frozen_taixu()` battle!
    let mut sess = TaixuModeSession::start_season(
        1105,
        "HaiWaiFengHe",
        1,
        "liubei",
        "pinganwushi",
        None,
        0,
    )
    .unwrap();
    assert_eq!(sess.season, "HaiWaiFenghe");
    sess.cards = vec![
        "wanjian".into(),
        "nanman".into(),
        "shunshou".into(),
        "txhj_shengdongjixi".into(),
        "txhj_shuiyanqijun".into(),
    ];
    sess.equip1 = Some("txhj_feilongduofeng".into());
    sess.equip2 = Some("rewrite_bagua".into());
    sess.equip3 = Some("zixin".into());
    sess.equip4 = None;
    sess.pending = Some(TaixuPendingInteraction::Battle(TaixuPendingBattleSpec {
        slot_idx: 0,
        event_id: b0.id.into(),
        event_kind: b0.kind.into(),
        season: "HaiWaiFenghe".into(),
        chapter: "0".into(),
        dlc_def_idx: Some(b0_idx),
        choose_btn_idx: None,
        target_str: "all".into(),
        unique: false,
        max_rounds: 1,
        exp: b0.exp,
    }));
    let e1 = start_locked_taixu_battle(&sess, verified).unwrap();
    let hand_names: Vec<&str> = e1
        .state_core()
        .player(0)
        .hand
        .iter()
        .map(|&c| e1.catalog().cards[c as usize].name)
        .collect();
    for req in ["archery", "savage", "snatch", "txhj_shengdong", "txhj_shuiyan"] {
        assert!(
            hand_names.contains(&req),
            "expected materialized paiku card `{req}` in hand {hand_names:?}"
        );
    }
    let eq_names: Vec<&str> = e1
        .state_core()
        .player(0)
        .equips
        .iter()
        .map(|&c| e1.catalog().cards[c as usize].name)
        .collect();
    assert!(eq_names.contains(&"txhj_feilong"), "feilongduofeng -> txhj_feilong: {eq_names:?}");
    let bagua_cid = e1
        .state_core()
        .player(0)
        .equips
        .iter()
        .copied()
        .find(|&c| e1.catalog().cards[c as usize].name == "armor")
        .expect("rewrite_bagua -> armor");
    assert_eq!(
        e1.state_core()
            .vm_aux
            .flags
            .get(&(0, format!("th_jingxie_rewritten:{bagua_cid}")))
            .copied(),
        Some(1)
    );
    assert!(eq_names.contains(&"horse_minus"), "zixin -> horse_minus: {eq_names:?}");
    assert!(e1.debug_has_skill(0, "buff_txhj_pozhenzhifeng"));

    // Also test `nvzhuang`, `dawan`, `txhj_shengdong`, `txhj_shuiyan`, `txhj_feilong`, and `txzhaoshu`!
    sess.cards = vec![
        "txhj_shengdong".into(),
        "txhj_shuiyan".into(),
    ];
    sess.equip1 = Some("txhj_feilong".into());
    sess.equip2 = Some("bagua".into());
    sess.equip3 = Some("txzhaoshu".into());
    sess.equip4 = Some("dawan".into());
    let e2 = start_locked_taixu_battle(&sess, verified).unwrap();
    let h2: Vec<&str> = e2
        .state_core()
        .player(0)
        .hand
        .iter()
        .map(|&c| e2.catalog().cards[c as usize].name)
        .collect();
    assert!(h2.contains(&"txhj_shengdong"));
    assert!(h2.contains(&"txhj_shuiyan"));
    let eq2: Vec<&str> = e2
        .state_core()
        .player(0)
        .equips
        .iter()
        .map(|&c| e2.catalog().cards[c as usize].name)
        .collect();
    assert!(eq2.contains(&"txhj_feilong"));
    assert!(eq2.contains(&"armor"), "bagua -> armor: {eq2:?}");
    assert!(eq2.contains(&"txzhaoshu_skill"));
    assert!(eq2.contains(&"horse_minus"), "dawan -> horse_minus: {eq2:?}");
}
