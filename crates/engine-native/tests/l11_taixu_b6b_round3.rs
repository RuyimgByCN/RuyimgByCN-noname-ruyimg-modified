//! Batch L11-6b Round 3: comprehensive real-event regression suite covering ALL 5 findings from #31.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

use content_native_loader::{
    get_dlc_event_by_idx, prepare_locked_content_with_bridge, start_locked_taixu_battle,
    validate_and_freeze_raw_package, TaixuCardCategory, TaixuEffectKind, TaixuModeSession,
    TaixuPendingBattleSpec, TaixuPendingInteraction, TaixuShopCategory, TaixuSlotNode,
    VerifiedLockedContent, L11_TAIXU_CARD_RANK,
};
use engine_native::ai::BoundedPlanner;
use engine_native::catalog::Suit;
use engine_native::ids::{CardId, PlayerId};
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase,
};

static TAIXU_R3_VERIFIED: OnceLock<VerifiedLockedContent> = OnceLock::new();
static R3_DEDUP: AtomicU64 = AtomicU64::new(71_000_000);

fn next_dedup() -> u64 {
    R3_DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn taixu_verified() -> &'static VerifiedLockedContent {
    TAIXU_R3_VERIFIED.get_or_init(|| {
        let pkg_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/taixu-upstream-liuli");
        let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
            .unwrap()
            .replace("\r\n", "\n");
        let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
        let mut assets = BTreeMap::new();
        if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
            for rel_path in paths.values().filter_map(|v| v.as_str()) {
                let bytes = std::fs::read(pkg_dir.join(rel_path))
                    .unwrap()
                    .into_iter()
                    .filter(|&b| b != b'\r')
                    .collect::<Vec<u8>>();
                assets.insert(rel_path.to_string(), bytes);
            }
        }
        let frozen = validate_and_freeze_raw_package(raw_json, assets).unwrap();
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

fn answer(e: &mut Engine, actor: PlayerId, selection: Vec<ChoiceOption>) -> CommandOutcome {
    let cid = e.open_choice().expect("open choice").id;
    submit(
        e,
        actor,
        GameAction::Answer {
            choice_id: cid,
            selection,
        },
    )
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

fn resolve_open_choices(e: &mut Engine) {
    let ai = BoundedPlanner::new(16);
    for _ in 0..32 {
        if e.outcome().is_some() {
            break;
        }
        let Some(ch) = e.open_choice() else {
            break;
        };
        let obs = ch.observer;
        let act = e.plan_action(obs, &ai).action;
        let _ = submit(e, obs, act);
    }
}

fn simple_1v1_boss_idx() -> usize {
    (0..313)
        .find(|&i| get_dlc_event_by_idx(i).is_some_and(|d| d.id == "txhj_huanjingshouhutongzi"))
        .expect("txhj_huanjingshouhutongzi")
}

fn find_me(e: &Engine) -> PlayerId {
    (0..e.state_core().players.len() as u8)
        .find(|&p| {
            e.state_core()
                .vm_aux
                .flags
                .get(&(p, "l11_txhj_active_battle".to_string()))
                .copied()
                .unwrap_or(0)
                > 0
        })
        .expect("l11_txhj_active_battle player")
}

fn attach_simple_boss_battle(sess: &mut TaixuModeSession, max_rounds: u32) {
    let idx = simple_1v1_boss_idx();
    let d = get_dlc_event_by_idx(idx).unwrap();
    sess.pending = Some(TaixuPendingInteraction::Battle(TaixuPendingBattleSpec {
        slot_idx: 0,
        event_id: d.id.into(),
        event_kind: d.kind.into(),
        season: sess.season.clone(),
        chapter: "1".into(),
        dlc_def_idx: Some(idx),
        choose_btn_idx: None,
        target_str: "all".into(),
        unique: false,
        max_rounds,
        exp: d.exp,
    }));
}

fn find_card_in_hand(e: &Engine, seat: PlayerId, name: &str) -> Option<CardId> {
    e.state_core()
        .player(seat)
        .hand
        .iter()
        .copied()
        .find(|&c| e.catalog().cards[c as usize].name == name)
}

fn ensure_hand_card(e: &mut Engine, seat: PlayerId, name: &str) -> CardId {
    if let Some(c) = find_card_in_hand(e, seat, name) {
        c
    } else {
        e.debug_place(seat, name)
    }
}

fn advance_to_my_play(e: &mut Engine, me: PlayerId) {
    let ai = BoundedPlanner::new(16);
    for _ in 0..80 {
        if e.outcome().is_some() {
            break;
        }
        if let Some(ch) = e.open_choice() {
            let obs = ch.observer;
            let act = e.plan_action(obs, &ai).action;
            let _ = submit(e, obs, act);
            continue;
        }
        let st = e.state_core();
        if st.turn.current == me && st.turn.phase == Phase::Play {
            break;
        }
        let cur = st.turn.current;
        if cur == me {
            let _ = submit(e, me, GameAction::NextPhase);
        } else {
            let act = e.plan_action(cur, &ai).action;
            let _ = submit(e, cur, act);
        }
    }
    assert_eq!(e.state_core().turn.current, me);
    assert_eq!(e.state_core().turn.phase, Phase::Play);
    assert!(e.open_choice().is_none());
}

#[test]
fn l11_b6b_r3_effect_pack_rank_difficulty_and_battle_modifiers() {
    let verified = taixu_verified();

    let s_base =
        TaixuModeSession::start_season(3001, "ChongYingChuLin", 1, "liubei", "pinganwushi", None, 0)
            .unwrap();
    let s_man = TaixuModeSession::start_season(
        3001,
        "ChongYingChuLin",
        1,
        "liubei",
        "lingqiyiman",
        None,
        0,
    )
    .unwrap();
    assert_eq!(s_man.max_hp, s_base.max_hp + 2);
    assert_eq!(s_man.hp, s_base.hp + 2);

    let mut s_kui = TaixuModeSession::start_season(
        3001,
        "ChongYingChuLin",
        1,
        "liubei",
        "lingqikuijie",
        None,
        0,
    )
    .unwrap();
    assert_eq!(s_kui.max_hp, s_base.max_hp - 1);
    assert_eq!(s_kui.hp, s_base.hp - 1);
    s_kui.min_hs = -1;
    attach_simple_boss_battle(&mut s_kui, 5);
    let e_kui = start_locked_taixu_battle(&s_kui, verified).unwrap();
    let me_kui = find_me(&e_kui);
    assert_eq!(me_kui, 0);
    assert_eq!(
        e_kui.state_core().player(me_kui).hand.len(),
        4,
        "player min_hs < 0 clamps up to 4 initial hand cards"
    );

    let s_cf = TaixuModeSession::start_season(
        3001,
        "ChongYingChuLin",
        1,
        "liubei",
        "chongfenbeizhan",
        None,
        0,
    )
    .unwrap();
    let s_cc =
        TaixuModeSession::start_season(3001, "ChongYingChuLin", 1, "liubei", "cangcuchuji", None, 0)
            .unwrap();
    assert_eq!(s_cf.min_hs, s_base.min_hs + 2);
    assert_eq!(s_cc.min_hs, s_base.min_hs - 3);

    let s_r3 =
        TaixuModeSession::start_season(3001, "ChongYingChuLin", 3, "liubei", "pinganwushi", None, 0)
            .unwrap();
    assert_eq!(s_r3.hp, s_base.hp - 1);
    assert_eq!(s_r3.max_hp, s_base.max_hp - 1);
    assert!(!s_r3.buffs.iter().any(|b| b == "buff_txhj_aozhan"));

    let s_r5 =
        TaixuModeSession::start_season(3001, "ChongYingChuLin", 5, "liubei", "pinganwushi", None, 0)
            .unwrap();
    assert_eq!(s_r5.hp, s_base.hp - 1);
    assert_eq!(s_r5.max_hp, s_base.max_hp - 1);
    assert!(s_r5.buffs.iter().any(|b| b == "buff_txhj_aozhan"));

    // `xueyan` grants player `kuanggu` in start_season and global_skills apply to NPCs without double-applying to `me`.
    let mut s_xy =
        TaixuModeSession::start_season(3002, "ChongYingChuLin", 1, "liubei", "xueyan", None, 0)
            .unwrap();
    assert!(s_xy.use_skills.iter().any(|s| s == "kuanggu"));
    attach_simple_boss_battle(&mut s_xy, 5);
    let e_xy = start_locked_taixu_battle(&s_xy, verified).unwrap();
    assert!(e_xy.debug_has_skill(0, "kuanggu"));
    assert!(e_xy.debug_has_skill(1, "kuanggu"));
    let kuanggu_id = e_xy.catalog().skill_by_name("kuanggu").unwrap();
    let me_skill_count = e_xy
        .state_core()
        .skills
        .keys()
        .filter(|&&(p, id)| p == 0 && id == kuanggu_id)
        .count();
    assert_eq!(me_skill_count, 1, "effectPack global_skill must not double-apply on me");

    // `huanjingcaoge` equips player in start_season and NPCs in battle.
    let mut s_hj = TaixuModeSession::start_season(
        3003,
        "ChongYingChuLin",
        1,
        "liubei",
        "huanjingcaoge",
        None,
        0,
    )
    .unwrap();
    let total_eq = s_hj.equips.len()
        + [&s_hj.equip1, &s_hj.equip2, &s_hj.equip3, &s_hj.equip4]
            .into_iter()
            .flatten()
            .count();
    assert!(total_eq >= 1, "huanjingcaoge gives player 1 high equip");
    attach_simple_boss_battle(&mut s_hj, 5);
    let e_hj = start_locked_taixu_battle(&s_hj, verified).unwrap();
    assert!(
        !e_hj.state_core().player(1).equips.is_empty(),
        "huanjingcaoge gives NPC high equip in battle"
    );
}

#[test]
fn l11_b6b_r3_effect_pack_skills_kuanggu_hongyan_tanshi_chanyuan_xingshang_hengzheng() {
    let verified = taixu_verified();

    // 1. `kuanggu` (xueyan) + `chanyuan` (xiaoxinxuanya):
    //    at hp=1, chanyuan seals kuanggu; after healing to hp=2 via real Peach, kuanggu recovers HP on Sha damage!
    let mut s_kc = TaixuModeSession::start_season(
        3010,
        "ChongYingChuLin",
        1,
        "liubei",
        "xiaoxinxuanya",
        None,
        0,
    )
    .unwrap();
    s_kc.use_skills.push("kuanggu".into());
    s_kc.use_skills.push("hongyan".into());
    s_kc.max_hp = 4;
    s_kc.hp = 1;
    attach_simple_boss_battle(&mut s_kc, 5);
    let mut e_kc = start_locked_taixu_battle(&s_kc, verified).unwrap();
    advance_to_my_play(&mut e_kc, 0);
    let spade_id = e_kc
        .state_core()
        .deck
        .iter()
        .copied()
        .find(|&c| e_kc.catalog().cards[c as usize].suit == Suit::Spade)
        .expect("spade in deck");
    assert_eq!(
        e_kc.catalog().effective_card_suit(e_kc.state_core(), 0, spade_id),
        Suit::Spade,
        "chanyuan at hp==1 seals hongyan suit conversion"
    );
    e_kc.debug_set_hp(1, 5);
    e_kc.debug_clear_hand(1);
    let sha1 = ensure_hand_card(&mut e_kc, 0, "sha");
    let out1 = submit(
        &mut e_kc,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
        },
    );
    assert!(!matches!(out1, CommandOutcome::Rejected(_)));
    resolve_open_choices(&mut e_kc);
    assert_eq!(
        e_kc.debug_hp(0),
        1,
        "chanyuan at hp==1 seals kuanggu so Sha damage does not heal"
    );
    // Heal via real Peach: hp 1 -> 2 restores chanyuan-sealed skills!
    let peach = ensure_hand_card(&mut e_kc, 0, "tao");
    let out_p = submit(
        &mut e_kc,
        0,
        GameAction::PlayCard {
            card: peach,
            targets: vec![],
        },
    );
    assert!(!matches!(out_p, CommandOutcome::Rejected(_)));
    resolve_open_choices(&mut e_kc);
    assert_eq!(e_kc.debug_hp(0), 2);
    assert_eq!(
        e_kc.catalog().effective_card_suit(e_kc.state_core(), 0, spade_id),
        Suit::Heart,
        "hp > 1 lifts chanyuan seal so hongyan turns Spade into Heart"
    );
    // Advance naturally to Seat 0 Turn 2 PlayPhase (with a Shan buffer against Seat 1 Turn 1) so Sha limit resets!
    let _ = ensure_hand_card(&mut e_kc, 0, "shan");
    let _ = submit(&mut e_kc, 0, GameAction::NextPhase);
    advance_to_my_play(&mut e_kc, 0);
    assert_eq!(e_kc.debug_hp(0), 2);
    let sha2 = ensure_hand_card(&mut e_kc, 0, "sha");
    let out2 = submit(
        &mut e_kc,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    );
    assert!(!matches!(out2, CommandOutcome::Rejected(_)));
    resolve_open_choices(&mut e_kc);
    assert_eq!(
        e_kc.debug_hp(0),
        3,
        "kuanggu recovers 1 HP on real Sha damage once hp > 1"
    );

    // 2. `hongyan` (hongyanhuoshui): real `Le` delayed-trick judge with Spade on deck top is treated as Heart (skipped)!
    let mut s_hy = TaixuModeSession::start_season(
        3011,
        "ChongYingChuLin",
        1,
        "liubei",
        "hongyanhuoshui",
        None,
        0,
    )
    .unwrap();
    attach_simple_boss_battle(&mut s_hy, 5);
    let mut e_hy = start_locked_taixu_battle(&s_hy, verified).unwrap();
    e_hy.debug_set_deck_top_by(|c| c.suit == Suit::Spade);
    let _ = e_hy.debug_place_judge(0, "le");
    for _ in 0..8 {
        if e_hy.state_core().turn.current == 0 && e_hy.state_core().turn.phase == Phase::Play {
            break;
        }
        if e_hy.open_choice().is_some() {
            resolve_open_choices(&mut e_hy);
        } else {
            let _ = submit(&mut e_hy, 0, GameAction::NextPhase);
        }
    }
    assert_eq!(
        e_hy.state_core().turn.phase,
        Phase::Play,
        "hongyan Spade judge resolves as Heart so Le does not skip PlayPhase"
    );

    // 3. `txhj_tanshi` (daodaozhiming): Phase::End opens `txhj_tanshi:discard_card` and discards 1 card.
    let mut s_ts = TaixuModeSession::start_season(
        3012,
        "ChongYingChuLin",
        1,
        "liubei",
        "daodaozhiming",
        None,
        0,
    )
    .unwrap();
    attach_simple_boss_battle(&mut s_ts, 5);
    let mut e_ts = start_locked_taixu_battle(&s_ts, verified).unwrap();
    advance_to_my_play(&mut e_ts, 0);
    let _ = ensure_hand_card(&mut e_ts, 0, "sha");
    let before_ts_hand = e_ts.debug_hand(0).len();
    for _ in 0..4 {
        if e_ts
            .open_choice()
            .is_some_and(|c| c.spec.prompt == "txhj_tanshi:discard_card")
        {
            break;
        }
        if e_ts.open_choice().is_some() {
            drain_ai(&mut e_ts, 1);
        } else {
            let _ = submit(&mut e_ts, 0, GameAction::NextPhase);
        }
    }
    let ch_ts = e_ts.open_choice().expect("tanshi prompt");
    assert_eq!(ch_ts.spec.prompt, "txhj_tanshi:discard_card");
    let drop_c = ch_ts.spec.options[0];
    let _ = answer(&mut e_ts, 0, vec![drop_c]);
    assert!(
        e_ts.debug_hand(0).len() < before_ts_hand,
        "txhj_tanshi discards chosen card"
    );

    // 4. `xingshang` (rulianshi): real kill of enemy with hand card opens `xingshang:confirm` & transfers cards!
    let mut s_xs =
        TaixuModeSession::start_season(3013, "ChongYingChuLin", 1, "liubei", "rulianshi", None, 0)
            .unwrap();
    s_xs.use_skills.push("wushuang".into());
    attach_simple_boss_battle(&mut s_xs, 5);
    // Add a 2nd enemy by checking single kill when `target_kill`? Even simpler: give ally (`txhj_yingjiuhandi` or check 2 enemies)!
    let two_idx = (0..313)
        .find(|&i| {
            get_dlc_event_by_idx(i).is_some_and(|d| {
                d.kind == "main" && d.enemies.len() >= 2 && d.enemies.iter().all(|n| n.seat < 0)
            })
        })
        .unwrap();
    let d2 = get_dlc_event_by_idx(two_idx).unwrap();
    s_xs.pending = Some(TaixuPendingInteraction::Battle(TaixuPendingBattleSpec {
        slot_idx: 0,
        event_id: d2.id.into(),
        event_kind: d2.kind.into(),
        season: "ChongYingChuLin".into(),
        chapter: "1".into(),
        dlc_def_idx: Some(two_idx),
        choose_btn_idx: None,
        target_str: "all".into(),
        unique: false,
        max_rounds: 5,
        exp: d2.exp,
    }));
    let mut e_xs = start_locked_taixu_battle(&s_xs, verified).unwrap();
    let me_xs = find_me(&e_xs);
    let target_xs = if me_xs == 0 { 1 } else { 0 };
    advance_to_my_play(&mut e_xs, me_xs);
    e_xs.debug_set_hp(target_xs, 1);
    e_xs.debug_clear_hand(target_xs);
    let gift = e_xs.debug_place(target_xs, "sha");
    let sha_k = ensure_hand_card(&mut e_xs, me_xs, "sha");
    let _ = submit(
        &mut e_xs,
        me_xs,
        GameAction::PlayCard {
            card: sha_k,
            targets: vec![target_xs],
        },
    );
    let ai_xs = BoundedPlanner::new(16);
    for _ in 0..16 {
        let Some(ch) = e_xs.open_choice() else {
            break;
        };
        if ch.spec.prompt == "xingshang:confirm" {
            break;
        }
        let obs = ch.observer;
        let act = e_xs.plan_action(obs, &ai_xs).action;
        let _ = submit(&mut e_xs, obs, act);
    }
    assert_eq!(
        e_xs.open_choice().map(|c| c.spec.prompt),
        Some("xingshang:confirm")
    );
    let _ = answer(&mut e_xs, me_xs, vec![ChoiceOption::Confirm]);
    assert!(
        e_xs.debug_hand(me_xs).contains(&gift),
        "xingshang steals dead player's hand card"
    );

    // 5. `hengzheng` (hengxingbadao): at DrawPhaseBegin1 when hp==1, skips draw & steals from alive other's zone.
    let mut s_hz = TaixuModeSession::start_season(
        3014,
        "ChongYingChuLin",
        1,
        "liubei",
        "hengxingbadao",
        None,
        0,
    )
    .unwrap();
    s_hz.hp = 1;
    attach_simple_boss_battle(&mut s_hz, 5);
    let mut e_hz = start_locked_taixu_battle(&s_hz, verified).unwrap();
    e_hz.debug_clear_hand(1);
    let loot = e_hz.debug_place(1, "wuzhong");
    for _ in 0..8 {
        if e_hz
            .open_choice()
            .is_some_and(|c| c.spec.prompt == "hengzheng:confirm")
        {
            break;
        }
        if e_hz.open_choice().is_some() {
            resolve_open_choices(&mut e_hz);
        } else {
            let _ = submit(&mut e_hz, 0, GameAction::NextPhase);
        }
    }
    assert_eq!(
        e_hz.open_choice().map(|c| c.spec.prompt),
        Some("hengzheng:confirm")
    );
    let hand_before_hz = e_hz.debug_hand(0).len();
    let _ = answer(&mut e_hz, 0, vec![ChoiceOption::Confirm]);
    resolve_open_choices(&mut e_hz);
    assert!(
        e_hz.debug_hand(0).contains(&loot) && e_hz.debug_hand(0).len() == hand_before_hz + 1,
        "hengzheng skips 2-card draw and steals 1 card per alive other"
    );
}

#[test]
fn l11_b6b_r3_yingjiuhandi_sp2_generals_skills_and_npc_max_hp_bonus() {
    let verified = taixu_verified();

    // 1. `txhj_yingjiuhandi` instantiates `lijue` (4/6), `guosi` (4/4), `zhangji` (4/4) & skips mulligan for seat != 0.
    let yjh_idx = (0..313)
        .find(|&i| get_dlc_event_by_idx(i).is_some_and(|d| d.id == "txhj_yingjiuhandi"))
        .expect("txhj_yingjiuhandi");
    let mut s_yjh =
        TaixuModeSession::start_season(3020, "ChongYingChuLin", 1, "liubei", "pinganwushi", None, 0)
            .unwrap();
    s_yjh.adjust = 2;
    s_yjh.pending = Some(TaixuPendingInteraction::Battle(TaixuPendingBattleSpec {
        slot_idx: 0,
        event_id: "txhj_yingjiuhandi".into(),
        event_kind: "side".into(),
        season: "ChongYingChuLin".into(),
        chapter: "1".into(),
        dlc_def_idx: Some(yjh_idx),
        choose_btn_idx: None,
        target_str: "repair".into(),
        unique: false,
        max_rounds: 5,
        exp: 0,
    }));
    let e_yjh = start_locked_taixu_battle(&s_yjh, verified).unwrap();
    assert!(
        e_yjh
            .open_choice()
            .is_none_or(|c| c.spec.prompt != "_txhj_skill_addEffectStyle:mulligan"),
        "mulligan must NOT fire when player_seat != 0"
    );
    let has_gen = |gname: &str, hp: i32, max_hp: u32, sk: &str| {
        e_yjh.state_core().players.iter().enumerate().any(|(i, p)| {
            p.general_slots[0]
                .and_then(|g| e_yjh.catalog().general_meta(g))
                .is_some_and(|m| m.0 == gname)
                && p.hp == hp
                && p.max_hp == max_hp
                && e_yjh.debug_has_skill(i as u8, sk)
        })
    };
    assert!(has_gen("lijue", 4, 6, "xinfu_yisuan"));
    assert!(has_gen("lijue", 4, 6, "xinfu_langxi"));
    assert!(has_gen("guosi", 4, 4, "xinfu_tanbei"));
    assert!(has_gen("guosi", 4, 4, "xinfu_sidao"));
    assert!(has_gen("zhangji", 4, 4, "xinfu_lveming"));
    assert!(has_gen("zhangji", 4, 4, "xinfu_tunjun"));

    // 2. NPC `max_hp_bonus > 0` (`txhj_huanshenguanyu`: max_hp_bonus = 2, level = 5) raises BOTH n_max_hp & n_hp.
    let hsg_idx = (0..313)
        .find(|&i| {
            get_dlc_event_by_idx(i).is_some_and(|d| {
                d.id == "txhj_huanshenguanyu"
                    && d.enemies.first().is_some_and(|n| n.max_hp_bonus == 2)
            })
        })
        .expect("txhj_huanshenguanyu with max_hp_bonus == 2");
    let mut s_bonus =
        TaixuModeSession::start_season(3021, "ChongYingChuLin", 1, "liubei", "pinganwushi", None, 0)
            .unwrap();
    s_bonus.pending = Some(TaixuPendingInteraction::Battle(TaixuPendingBattleSpec {
        slot_idx: 0,
        event_id: "txhj_huanshenguanyu".into(),
        event_kind: "noname".into(),
        season: "ChongYingChuLin".into(),
        chapter: "1".into(),
        dlc_def_idx: Some(hsg_idx),
        choose_btn_idx: None,
        target_str: "all".into(),
        unique: false,
        max_rounds: 5,
        exp: 100,
    }));
    let e_bonus = start_locked_taixu_battle(&s_bonus, verified).unwrap();
    let npc_seat = (0..e_bonus.state_core().players.len() as u8)
        .find(|&p| {
            e_bonus.state_core().player(p).general_slots[0]
                .and_then(|g| e_bonus.catalog().general_meta(g))
                .is_some_and(|m| m.0 == "txhj_huanshenguanyu")
        })
        .expect("txhj_huanshenguanyu seat");
    let npc = e_bonus.state_core().player(npc_seat);
    let base_max = e_bonus
        .catalog()
        .general_meta(npc.general_slots[0].unwrap())
        .unwrap()
        .3;
    assert_eq!(npc.max_hp, base_max + 2);
    assert_eq!(npc.hp as u32, npc.max_hp, "max_hp_bonus increments both n_max_hp and n_hp");

    // 3. Real active/trigger play of SP2 skills: `xinfu_yisuan`, `xinfu_lveming`, `xinfu_tunjun`, `xinfu_tanbei`.
    let mut s_sp2 =
        TaixuModeSession::start_season(3022, "ChongYingChuLin", 1, "liubei", "pinganwushi", None, 0)
            .unwrap();
    s_sp2.use_skills.push("xinfu_yisuan".into());
    s_sp2.use_skills.push("xinfu_lveming".into());
    s_sp2.use_skills.push("xinfu_tunjun".into());
    s_sp2.use_skills.push("xinfu_tanbei".into());
    s_sp2.equip1 = Some("txhj_feilong".into());
    attach_simple_boss_battle(&mut s_sp2, 5);
    let mut e_sp2 = start_locked_taixu_battle(&s_sp2, verified).unwrap();
    advance_to_my_play(&mut e_sp2, 0);

    // (a) `xinfu_yisuan`: use ordinary trick `wuzhong` -> confirm -> max_hp -1 & trick regained.
    let wz = ensure_hand_card(&mut e_sp2, 0, "wuzhong");
    let max_before = e_sp2.state_core().player(0).max_hp;
    let out_wz = submit(
        &mut e_sp2,
        0,
        GameAction::PlayCard {
            card: wz,
            targets: vec![],
        },
    );
    assert!(!matches!(out_wz, CommandOutcome::Rejected(_)));
    let ai = BoundedPlanner::new(16);
    for _ in 0..8 {
        let Some(ch) = e_sp2.open_choice() else {
            break;
        };
        if ch.spec.prompt == "xinfu_yisuan:confirm" {
            break;
        }
        let obs = ch.observer;
        let act = e_sp2.plan_action(obs, &ai).action;
        let _ = submit(&mut e_sp2, obs, act);
    }
    assert_eq!(
        e_sp2.open_choice().map(|c| c.spec.prompt),
        Some("xinfu_yisuan:confirm")
    );
    let _ = answer(&mut e_sp2, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(e_sp2.state_core().player(0).max_hp, max_before - 1);
    assert!(e_sp2.debug_hand(0).contains(&wz));

    // (b) `xinfu_lveming` -> enemy picks rank matching top card (rank 1) -> 2 dmg, then `xinfu_tunjun` equips!
    e_sp2.debug_clear_hand(1);
    e_sp2.debug_set_hp(1, 5);
    e_sp2.debug_set_deck_top_by(|c| c.rank == 1);
    let lm_id = e_sp2.catalog().skill_by_name("xinfu_lveming").unwrap();
    let _ = submit(&mut e_sp2, 0, GameAction::ActivateSkill { skill: lm_id });
    assert_eq!(
        e_sp2.open_choice().map(|c| c.spec.prompt),
        Some("xinfu_lveming:choose_target")
    );
    let _ = answer(&mut e_sp2, 0, vec![ChoiceOption::Player(1)]);
    assert_eq!(
        e_sp2.open_choice().map(|c| c.spec.prompt),
        Some("xinfu_lveming:choose_rank")
    );
    let _ = answer(&mut e_sp2, 1, vec![ChoiceOption::Tag(0)]); // Tag(0) == rank 1
    resolve_open_choices(&mut e_sp2);
    assert_eq!(e_sp2.debug_hp(1), 3, "lveming exact rank match deals 2 dmg");

    let tj_id = e_sp2.catalog().skill_by_name("xinfu_tunjun").unwrap();
    let _ = submit(&mut e_sp2, 0, GameAction::ActivateSkill { skill: tj_id });
    assert_eq!(
        e_sp2.open_choice().map(|c| c.spec.prompt),
        Some("xinfu_tunjun:choose_target")
    );
    let _ = answer(&mut e_sp2, 0, vec![ChoiceOption::Player(0)]);
    assert!(
        e_sp2.debug_equips(0).len() >= 2,
        "tunjun attaches equip from deck onto target"
    );

    // (c) `xinfu_tanbei` mode 0 steals a card from enemy hand.
    let stolen = e_sp2.debug_place(1, "sha");
    let tb_id = e_sp2.catalog().skill_by_name("xinfu_tanbei").unwrap();
    let _ = submit(&mut e_sp2, 0, GameAction::ActivateSkill { skill: tb_id });
    assert_eq!(
        e_sp2.open_choice().map(|c| c.spec.prompt),
        Some("xinfu_tanbei:choose_target")
    );
    let _ = answer(&mut e_sp2, 0, vec![ChoiceOption::Player(1)]);
    assert_eq!(
        e_sp2.open_choice().map(|c| c.spec.prompt),
        Some("xinfu_tanbei:choose_mode")
    );
    let _ = answer(&mut e_sp2, 1, vec![ChoiceOption::Tag(0)]);
    assert!(e_sp2.debug_hand(0).contains(&stolen));
}

#[test]
fn l11_b6b_r3_equip_slots_card_fallbacks_and_mode_fixes() {
    let verified = taixu_verified();

    // 1. Exact `Equip1..6` categorization in L11_TAIXU_CARD_RANK & no auto-equip for `Equip5`/`Equip6`.
    let cat_of = |id: &str| {
        L11_TAIXU_CARD_RANK
            .iter()
            .find(|c| c.card_id == id)
            .expect(id)
            .category
    };
    assert_eq!(cat_of("txhj_lingbaoxianhu"), TaixuCardCategory::Equip1);
    assert_eq!(cat_of("txhj_taijifuchen"), TaixuCardCategory::Equip1);
    assert_eq!(cat_of("txhj_chongyingshenfu"), TaixuCardCategory::Equip2);
    assert_eq!(cat_of("txhj_juechenjinge"), TaixuCardCategory::Equip3);
    assert_eq!(cat_of("txhj_liulongcanjia"), TaixuCardCategory::Equip6);
    assert_eq!(cat_of("txhj_xuwangzhimian"), TaixuCardCategory::Equip5);
    assert_eq!(cat_of("txhj_dinglanyemingzhu"), TaixuCardCategory::Equip5);

    let mut s_mode =
        TaixuModeSession::start_season(3030, "ChongYingChuLin", 1, "liubei", "pinganwushi", None, 0)
            .unwrap();
    // Shop duplicate card purchase succeeds twice:
    s_mode.coin = 5000;
    s_mode.optional[0] = Some(TaixuSlotNode {
        slot_seat: 991,
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
    let _ = s_mode.open_slot(0).unwrap();
    s_mode.stores.get_mut("txhj_huanjingjishi").unwrap().cards =
        vec!["sha".into(), "sha".into()];
    let cards_before = s_mode.cards.len();
    let _ = s_mode.buy_shop_item(TaixuShopCategory::Card, 0).unwrap();
    let _ = s_mode.buy_shop_item(TaixuShopCategory::Card, 0).unwrap();
    assert_eq!(
        s_mode.cards.len(),
        cards_before + 2,
        "shop must allow buying duplicate cards"
    );
    s_mode.leave_shop().unwrap();

    // `acquire_equip` keeps Equip5 and Equip6 in `equips` bag without auto-equipping into `equip3`/`equip4`:
    s_mode.apply_effect_op(TaixuEffectKind::Equip, 1, "txhj_xuwangzhimian");
    s_mode.apply_effect_op(TaixuEffectKind::Equip, 1, "txhj_liulongcanjia");
    assert!(s_mode.equip3.is_none() && s_mode.equip4.is_none());
    assert!(s_mode.equips.contains(&"txhj_xuwangzhimian".to_string()));
    assert!(s_mode.equips.contains(&"txhj_liulongcanjia".to_string()));

    // `compute_score` excludes bag equips (`self.equips`), counting only `use_skills`, `buffs`, `cards`, `equip1..4`:
    let score_with_bag = s_mode.compute_score(true).value_score;
    s_mode.equips.clear();
    let score_without_bag = s_mode.compute_score(true).value_score;
    assert_eq!(score_with_bag, score_without_bag);

    // `nvzhuang` equips authentically (NOT substituting into `armor`), while invalid card names (`taoyuan`, `shuiyanqijunx`, `yushijufen`) do NOT inject substitute cards:
    let mut s_clean = s_mode.clone();
    s_clean.equip2 = None;
    s_clean.adjust = 2;
    attach_simple_boss_battle(&mut s_clean, 5);
    let e_clean = start_locked_taixu_battle(&s_clean, verified).unwrap();
    let total_cards = |e: &Engine| {
        e.state_core().deck.len() + e.debug_hand(0).len() + e.debug_hand(1).len() + e.debug_equips(0).len()
    };
    let clean_count = total_cards(&e_clean);

    s_mode.cards.push("taoyuan".into());
    s_mode.cards.push("shuiyanqijunx".into());
    s_mode.cards.push("yushijufen".into());
    s_mode.equip2 = Some("nvzhuang".into());
    s_mode.adjust = 2; // Seat 0 Turn 1 starts at Phase::Prepare and opens mulligan!
    attach_simple_boss_battle(&mut s_mode, 5);
    let e_mull = start_locked_taixu_battle(&s_mode, verified).unwrap();
    assert_eq!(e_mull.state_core().turn.current, 0);
    assert_eq!(e_mull.state_core().turn.phase, Phase::Prepare);
    assert_eq!(
        e_mull.open_choice().map(|c| c.spec.prompt),
        Some("_txhj_skill_addEffectStyle:mulligan"),
        "Seat 0 Turn 1 must boot cleanly at TurnBegin -> Phase::Prepare and fire mulligan"
    );
    let eq0: Vec<&str> = e_mull
        .debug_equips(0)
        .into_iter()
        .map(|cid| e_mull.catalog().cards[cid as usize].name)
        .collect();
    assert_eq!(
        eq0,
        vec!["nvzhuang"],
        "nvzhuang must equip its authentic CardDef rather than substituting into armor"
    );
    assert!(e_mull.debug_has_skill(0, "nvzhuang"));
    assert_eq!(
        total_cards(&e_mull),
        clean_count + 1,
        "stripped invalid card names must not inject fake substitute cards (+1 is only authentic off-deck nvzhuang)"
    );

    // `answer_gain_or_trade` runs `run_change_event` BEFORE `consume_slot`:
    let mut s_chg =
        TaixuModeSession::start_season(3031, "ChongYingChuLin", 2, "liubei", "pinganwushi", None, 0)
            .unwrap();
    s_chg.optional[0] = Some(TaixuSlotNode {
        slot_seat: 995,
        season: "ChongYingChuLin".into(),
        chapter: "2".into(),
        id: "txhj_fujiazhili".into(),
        kind: "gain".into(),
        level: 1,
        change: true,
        dlc_def_idx: None,
        exam_tier: None,
        override_buttons: None,
    });
    s_chg.optional[1] = Some(TaixuSlotNode {
        slot_seat: 996,
        season: "ChongYingChuLin".into(),
        chapter: "2".into(),
        id: "txhj_puyuanchuanren".into(),
        kind: "gain".into(),
        level: 1,
        change: true,
        dlc_def_idx: None,
        exam_tier: None,
        override_buttons: None,
    });
    let _ = s_chg.open_slot(0).unwrap();
    let _ = s_chg.answer_gain_or_trade(0).unwrap();
    assert_ne!(
        s_chg.optional[1].as_ref().map(|n| n.slot_seat),
        Some(996),
        "other change=true slot must update via run_change_event"
    );
    if let Some(ref s0) = s_chg.optional[0] {
        assert_ne!(s0.slot_seat, 995);
        assert!(
            s0.slot_seat > s_chg.optional[1].as_ref().map_or(0, |n| n.slot_seat),
            "consume_slot refill allocates seat AFTER run_change_event"
        );
    }
}
