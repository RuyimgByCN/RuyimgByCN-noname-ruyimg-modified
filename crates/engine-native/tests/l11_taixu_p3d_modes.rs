//! P3-D L11 太虚幻境实局与模式回归测试（零 `debug_execute_effect`、零 `debug_resolve`）：
//! 1. `bind_locked_taixu_catalog` / `bind_available_catalog` 对 `skillRank` 与 `cardRank` 的动态筛选（含商店、随机卡池、`huanjingcaoge` 与向后兼容快照）；
//! 2. `event_universal.js` 真实交互流程（`exam1`/`exam3`/`exam4`/`exam4001..4016` 随机技能、5 阶商店 `+200` 与 `juguzhidao` `dec=1`、`gain`/`trade`）；
//! 3. 全 8 赛季 DLC 实局建局（含全部 DLC/扩展装备、`nvzhuang`/`zhuge` 战利品、`guosi`/`lijue`/`zhangji` 原生武将与公开信息集 AI）；
//! 4. 5 件新实装装备（`yajiaoqiang`、`nvzhuang`、`txhj_xieshenmianju`、`txhj_shengguangbaiyi`、`txhj_zhenhunqin`）真实 `PlayCard`/出牌回合内实测、反例守卫与 NGSA v107 存档恢复。

use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::OnceLock;

use content_native_loader::{
    bind_locked_taixu_catalog, find_exam_by_id, find_universal_event, get_dlc_event_by_idx,
    prepare_locked_content_with_bridge, restore_locked_engine, start_locked_taixu_battle,
    start_locked_taixu_season, validate_and_freeze_raw_package, TaixuBattleTarget, TaixuBtnResult,
    TaixuEffectKind, TaixuModeSession, TaixuPendingBattleSpec, TaixuPendingInteraction,
    TaixuShopCategory, TaixuSlotNode, VerifiedLockedContent, L11_TAIXU_CARD_RANK,
    L11_TAIXU_CHANGE_PACK, L11_TAIXU_CHAPTER_EVENT_REFS_A, L11_TAIXU_CHAPTER_EVENT_REFS_B,
    L11_TAIXU_GAIN_PACK, L11_TAIXU_SEASONS,
};
use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive, ARCHIVE_FORMAT_VERSION};
use engine_native::catalog::{CardKind, Suit};
use engine_native::ids::{CardId, PlayerId};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};

static TAIXU_P3D_VERIFIED: OnceLock<VerifiedLockedContent> = OnceLock::new();
static P3D_DEDUP: AtomicU64 = AtomicU64::new(83_000_000);

fn next_dedup() -> u64 {
    P3D_DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn taixu_verified() -> &'static VerifiedLockedContent {
    TAIXU_P3D_VERIFIED.get_or_init(|| {
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

fn is_accepted(outcome: &CommandOutcome) -> bool {
    matches!(
        outcome,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    )
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

fn decline_open_choice_if_matching(e: &mut Engine, prompt: &str) {
    if e.open_choice().is_some_and(|c| c.spec.prompt == prompt) {
        let c = e.open_choice().unwrap().clone();
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else {
            Vec::new()
        };
        let out = submit(
            e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        );
        assert!(is_accepted(&out), "decline {prompt} rejected: {out:?}");
    }
}

fn clear_seat_skills(e: &mut Engine, seat: PlayerId) {
    let skills: Vec<_> = e
        .state_core()
        .skills
        .keys()
        .filter(|&&(p, _)| p == seat)
        .copied()
        .collect();
    for k in skills {
        e.state_core_mut().skills.remove(&k);
    }
}

fn drain_to_play_phase(e: &mut Engine, actor: PlayerId) {
    let ai = BoundedPlanner::new(8);
    for _ in 0..256 {
        assert!(e.outcome().is_none(), "match ended early before seat {actor} Play phase: {:?}", e.outcome());
        if let Some(c) = e.open_choice().cloned() {
            let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if c.spec.allow_cancel && c.spec.min == 0 {
                Vec::new()
            } else {
                match e.plan_action(c.observer, &ai).action {
                    GameAction::Answer { selection, .. } => selection,
                    _ => c
                        .spec
                        .options
                        .iter()
                        .take(c.spec.min.max(1))
                        .copied()
                        .collect(),
                }
            };
            let out = submit(
                e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            );
            assert!(is_accepted(&out), "drain answer failed on {}: {out:?}", c.spec.prompt);
            continue;
        }
        let cur = e.state_core().turn.current;
        let phase = e.state_core().turn.phase;
        if cur == actor && phase == Phase::Play {
            break;
        }
        let out = submit(e, cur, GameAction::NextPhase);
        assert!(is_accepted(&out), "drain NextPhase failed at ({cur}, {phase:?}): {out:?}");
    }
    assert_eq!(e.state_core().turn.current, actor);
    assert_eq!(e.state_core().turn.phase, Phase::Play);
}

fn advance_to_next_play_phase(e: &mut Engine, actor: PlayerId) {
    let cur = e.state_core().turn.current;
    if cur == actor && e.state_core().turn.phase == Phase::Play {
        let _ = submit(e, cur, GameAction::NextPhase);
    }
    drain_to_play_phase(e, actor);
}

fn move_hand_to_discard(e: &mut Engine, seat: PlayerId) {
    let drained: Vec<CardId> = e.state_core_mut().player_mut(seat).hand.drain(..).collect();
    e.state_core_mut().discard.extend(drained);
}

fn claim_named_card(e: &mut Engine, name: &str) -> CardId {
    if let Some(pos) = e
        .state_core()
        .deck
        .iter()
        .position(|&c| e.catalog().cards[c as usize].name == name)
    {
        return e.state_core_mut().deck.remove(pos);
    }
    if let Some(pos) = e
        .state_core()
        .discard
        .iter()
        .position(|&c| e.catalog().cards[c as usize].name == name)
    {
        return e.state_core_mut().discard.remove(pos);
    }
    if let Some(pos) = e
        .state_core()
        .detached_cards
        .iter()
        .position(|&(c, _)| e.catalog().cards[c as usize].name == name)
    {
        return e.state_core_mut().detached_cards.remove(pos).0;
    }
    let seats = e.state_core().players.len();
    for s in 0..seats {
        if let Some(pos) = e
            .state_core()
            .players[s]
            .hand
            .iter()
            .position(|&c| e.catalog().cards[c as usize].name == name)
        {
            return e.state_core_mut().players[s].hand.remove(pos);
        }
    }
    panic!("missing card `{name}` in deck/discard/detached/hands");
}

fn claim_card_matching(
    e: &mut Engine,
    pred: impl Fn(&engine_native::catalog::CardDef) -> bool,
) -> CardId {
    let cid = {
        let cat = e.catalog();
        e.state_core()
            .deck
            .iter()
            .copied()
            .chain(e.state_core().discard.iter().copied())
            .chain(e.state_core().players.iter().flat_map(|p| p.hand.iter().copied()))
            .find(|&c| pred(&cat.cards[c as usize]))
            .expect("matching card in deck/discard/hands")
    };
    e.state_core_mut().deck.retain(|&c| c != cid);
    e.state_core_mut().discard.retain(|&c| c != cid);
    for pl in &mut e.state_core_mut().players {
        pl.hand.retain(|&c| c != cid);
    }
    cid
}

fn equip_real_card(e: &mut Engine, seat: PlayerId, name: &str) -> CardId {
    let cid = claim_named_card(e, name);
    e.state_core_mut().player_mut(seat).hand.push(cid);
    let play = submit(
        e,
        seat,
        GameAction::PlayCard {
            card: cid,
            targets: Vec::new(),
        },
    );
    assert!(is_accepted(&play), "equip {name} via PlayCard failed: {play:?}");
    cid
}

fn build_taixu_dlc_battle(
    sess: &TaixuModeSession,
    dlc_def_idx: usize,
    verified: &VerifiedLockedContent,
) -> Engine {
    let def = get_dlc_event_by_idx(dlc_def_idx).expect("dlc def exists");
    let mut bs = sess.clone();
    bs.pending = Some(TaixuPendingInteraction::Battle(TaixuPendingBattleSpec {
        slot_idx: 0,
        event_id: def.id.to_string(),
        event_kind: def.kind.to_string(),
        season: sess.season.clone(),
        chapter: "0".to_string(),
        dlc_def_idx: Some(dlc_def_idx),
        choose_btn_idx: (!def.buttons.is_empty()).then_some(0),
        target_str: match def.target {
            TaixuBattleTarget::All => "all",
            TaixuBattleTarget::Kill => "kill",
            TaixuBattleTarget::Repair => "repair",
            TaixuBattleTarget::Subsist => "subsist",
        }
        .to_string(),
        unique: def.unique,
        max_rounds: def.max_rounds,
        exp: def.exp,
    }));
    start_locked_taixu_battle(&bs, verified)
        .unwrap_or_else(|err| panic!("launch {} failed: {err:?}", def.id))
}

#[test]
fn p3d_catalog_filter_shop_price_and_event_universal() {
    let verified = taixu_verified();

    // 1. `start_locked_taixu_season` / `bind_available_catalog` filters both `skillRank` and `cardRank` against live `Catalog`.
    let mut sess = start_locked_taixu_season(
        2026_1124,
        "ChongYingChuLin",
        3,
        "liubei",
        "huanjingcaoge",
        Some("jiuwei"),
        3,
        verified,
    )
    .expect("start_locked_taixu_season");

    let card_flt: BTreeSet<String> = sess
        .available_cards_filter
        .as_ref()
        .expect("available_cards_filter must be populated")
        .iter()
        .cloned()
        .collect();
    let skill_flt: BTreeSet<String> = sess
        .available_skills_filter
        .as_ref()
        .expect("available_skills_filter must be populated")
        .iter()
        .cloned()
        .collect();

    assert!(card_flt.contains("sha"));
    for resolved_eq in [
        "txhj_zhenhunqin",
        "txhj_xieshenmianju",
        "txhj_shengguangbaiyi",
        "yajiaoqiang",
        "nvzhuang",
        "yitianjian",
        "wushuangfangtianji",
        "guilongzhanyuedao",
        "wuxinghelingshan",
        "rewrite_bagua",
        "rewrite_baiyin",
        "rewrite_tengjia",
        "lanyinjia",
        "rewrite_renwang",
        "rewrite_zhuge",
    ] {
        assert!(
            card_flt.contains(resolved_eq),
            "resolved equip {resolved_eq} must be present in available_cards_filter"
        );
    }
    for dead_card in ["caoyao", "du", "wy_meirenji", "dagongche", "zheji", "yinfengjia"] {
        assert!(
            !card_flt.contains(dead_card),
            "unloaded card {dead_card} must be filtered out from available_cards_filter"
        );
    }
    for eq in [&sess.equip1, &sess.equip2, &sess.equip3, &sess.equip4]
        .into_iter()
        .flatten()
    {
        assert!(
            card_flt.contains(eq),
            "huanjingcaoge equip {eq} must satisfy available_cards_filter"
        );
    }

    // 2. All 5 shop levels honor both `available_cards_filter` and `available_skills_filter`,
    //    and escalate +200 per purchase (`buff_txhj_juguzhidao` gives dec=1 discount).
    let shop_ids = [
        ("txhj_sibingheishi", 1u8),
        ("txhj_anxianghonglou", 2u8),
        ("txhj_xiaojishi", 3u8),
        ("txhj_dajishi", 4u8),
        ("txhj_huanjingjishi", 5u8),
    ];
    for (shop_id, _lvl) in shop_ids {
        sess.coin = 50_000;
        let shop_node = find_universal_event(shop_id).expect("universal shop");
        sess.optional[0] = Some(TaixuSlotNode {
            slot_seat: 900 + u16::from(shop_node.level),
            season: sess.season.clone(),
            chapter: "0".into(),
            id: shop_id.into(),
            kind: "store".into(),
            level: shop_node.level,
            change: shop_node.change,
            dlc_def_idx: None,
            exam_tier: None,
            override_buttons: None,
        });
        let opened = sess.open_slot(0).expect("open shop slot");
        let TaixuPendingInteraction::Shop { shop_id: sid, .. } = opened else {
            panic!("expected shop interaction for {shop_id}");
        };
        let stock = sess.stores.get(&sid).expect("shop stock").clone();
        for c in stock.cards.iter().chain(stock.equips.iter()) {
            assert!(
                card_flt.contains(c),
                "shop {shop_id} leaked unfiltered card/equip `{c}`"
            );
        }
        for sk in &stock.skills {
            assert!(
                skill_flt.contains(sk),
                "shop {shop_id} leaked unfiltered skill `{sk}`"
            );
        }
        if stock.equips.len() >= 2 {
            let p0 = sess
                .shop_item_price(&sid, TaixuShopCategory::Equip, &stock.equips[0])
                .unwrap();
            let paid0 = sess.buy_shop_item(TaixuShopCategory::Equip, 0).unwrap();
            assert_eq!(paid0, p0);
            let next_eq = &sess.stores.get(&sid).unwrap().equips[0].clone();
            let p1 = sess
                .shop_item_price(&sid, TaixuShopCategory::Equip, next_eq)
                .unwrap();
            let has_jugu = sess.buffs.iter().any(|b| b == "buff_txhj_juguzhidao");
            if !has_jugu {
                let p_base_next = L11_TAIXU_CARD_RANK
                    .iter()
                    .find(|c| c.card_id == next_eq)
                    .unwrap()
                    .value as i32;
                assert_eq!(p1, p_base_next + 200, "store price must escalate +200 per item");
            }
        }
        sess.leave_shop().expect("leave shop");
    }

    // Explicit `buff_txhj_juguzhidao` dec=1 discount check:
    sess.stores.clear();
    sess.buffs.retain(|b| b != "buff_txhj_juguzhidao");
    sess.buffs.push("buff_txhj_juguzhidao".into());
    sess.coin = 50_000;
    sess.optional[0] = Some(TaixuSlotNode {
        slot_seat: 999,
        season: sess.season.clone(),
        chapter: "0".into(),
        id: "txhj_dajishi".into(),
        kind: "store".into(),
        level: 4,
        change: true,
        dlc_def_idx: None,
        exam_tier: None,
        override_buttons: None,
    });
    let _ = sess.open_slot(0).unwrap();
    let st = sess.stores.get("txhj_dajishi").unwrap().clone();
    assert!(st.equips.len() >= 3, "level 4 store must stock >= 3 equips");
    let eq0_val = L11_TAIXU_CARD_RANK
        .iter()
        .find(|c| c.card_id == st.equips[0])
        .unwrap()
        .value as i32;
    let eq1_val = L11_TAIXU_CARD_RANK
        .iter()
        .find(|c| c.card_id == st.equips[1])
        .unwrap()
        .value as i32;
    let eq2_val = L11_TAIXU_CARD_RANK
        .iter()
        .find(|c| c.card_id == st.equips[2])
        .unwrap()
        .value as i32;
    assert_eq!(sess.buy_shop_item(TaixuShopCategory::Equip, 0).unwrap(), eq0_val);
    assert_eq!(sess.buy_shop_item(TaixuShopCategory::Equip, 0).unwrap(), eq1_val);
    assert_eq!(
        sess.buy_shop_item(TaixuShopCategory::Equip, 0).unwrap(),
        eq2_val + 200
    );
    sess.leave_shop().unwrap();

    // 3. `apply_effect_op` & `event_universal.js` `exam1`/`exam3`/`exam4`/`exam4001..4016` & snapshot backwards-compat
    sess.cards.clear();
    sess.equips.clear();
    for (op, n) in [
        (TaixuEffectKind::RandomCard, 3),
        (TaixuEffectKind::Basic, 2),
        (TaixuEffectKind::Trick, 2),
        (TaixuEffectKind::RandomEquip, 2),
        (TaixuEffectKind::Equip1, 2),
        (TaixuEffectKind::Equip2, 2),
        (TaixuEffectKind::Equip5, 2),
        (TaixuEffectKind::RandomUnique, 2),
    ] {
        sess.apply_effect_op(op, n, "");
    }
    assert!(!sess.cards.is_empty());
    for c in &sess.cards {
        assert!(card_flt.contains(c), "apply_effect_op produced filtered card `{c}`");
    }
    for eq in sess
        .equips
        .iter()
        .chain([&sess.equip1, &sess.equip2, &sess.equip3, &sess.equip4].into_iter().flatten())
    {
        assert!(card_flt.contains(eq), "apply_effect_op produced filtered equip `{eq}`");
    }

    // All 16 `exam4001..exam4016` questions plus `exam1`/`exam3` open cleanly & grant catalog-valid skills!
    for idx in 1..=16 {
        let qid = format!("exam40{idx:02}");
        let q = find_exam_by_id(&qid).expect("exam4001..4016 exists");
        assert_eq!(q.buttons[q.true_btn_idx].result, TaixuBtnResult::True);
        assert!(
            q.buttons[q.true_btn_idx]
                .effects
                .iter()
                .any(|e| e.kind == TaixuEffectKind::RandomSkill),
            "{qid} true button awards RandomSkill"
        );
        let skills_before = sess.use_skills.len() + sess.skills.len();
        sess.optional[0] = Some(TaixuSlotNode {
            slot_seat: 1100 + idx as u16,
            season: sess.season.clone(),
            chapter: "0".into(),
            id: qid.clone(),
            kind: q.tier.into(),
            level: 1,
            change: true,
            dlc_def_idx: None,
            exam_tier: Some(q.tier.into()),
            override_buttons: None,
        });
        let opened = sess.open_slot(0).unwrap();
        let TaixuPendingInteraction::Quiz { buttons, .. } = opened else {
            panic!("expected quiz for {qid}");
        };
        let disp_pos = buttons
            .iter()
            .position(|b| b.result == Some(true) && b.orig_idx == Some(q.true_btn_idx))
            .expect("true quiz option present");
        let (passed, _) = sess.answer_quiz(disp_pos).unwrap();
        assert!(passed);
        assert!(sess.use_skills.len() + sess.skills.len() > skills_before);
    }
    for sk in sess.use_skills.iter().chain(sess.skills.iter()) {
        assert!(skill_flt.contains(sk), "acquired skill {sk} must be in available_skills_filter");
    }
    assert!(L11_TAIXU_GAIN_PACK.len() == 22 && L11_TAIXU_CHANGE_PACK.len() == 24);
    assert!(find_exam_by_id("exam1001").is_some() && find_exam_by_id("exam3001").is_some());

    // Snapshot backwards compat without `available_cards_filter` JSON field + rebinding:
    let json = sess.serialize_snapshot();
    let restored = TaixuModeSession::deserialize_snapshot(&json).unwrap();
    assert_eq!(restored, sess);
    let mut val: serde_json::Value = serde_json::from_str(&json).unwrap();
    val.as_object_mut().unwrap().remove("available_cards_filter");
    let mut legacy = TaixuModeSession::deserialize_snapshot(&val.to_string()).unwrap();
    assert_eq!(legacy.available_cards_filter, None);
    bind_locked_taixu_catalog(&mut legacy, verified).unwrap();
    assert_eq!(legacy.available_cards_filter, sess.available_cards_filter);
}

#[test]
fn p3d_all_8_dlc_seasons_launch_real_battles_with_dlc_equips_and_non_txhj_generals() {
    let verified = taixu_verified();

    // 1. Verify all 8 DLC seasons resolve and launch real `Engine` battles.
    assert_eq!(L11_TAIXU_SEASONS.len(), 8);
    let refs = L11_TAIXU_CHAPTER_EVENT_REFS_A
        .iter()
        .chain(L11_TAIXU_CHAPTER_EVENT_REFS_B.iter())
        .collect::<Vec<_>>();
    for (i, season) in L11_TAIXU_SEASONS.iter().enumerate() {
        let sess = start_locked_taixu_season(
            8800 + i as u64,
            season.id,
            2,
            "liubei",
            "pinganwushi",
            Some("lulu"),
            2,
            verified,
        )
        .unwrap();
        let battle_ref = refs
            .iter()
            .find(|r| {
                r.season == season.id
                    && r.def_idx >= 0
                    && get_dlc_event_by_idx(r.def_idx as usize)
                        .is_some_and(|d| !d.enemies.is_empty())
            })
            .expect("each DLC season must have battle events");
        let e = build_taixu_dlc_battle(&sess, battle_ref.def_idx as usize, verified);
        assert!(
            e.state_core().alive_count() >= 2,
            "{} must spawn >= 2 seats",
            season.id
        );
    }

    // 2. Verify every DLC hardcoded equip & `extension_card.js` equip & `rewrite_*` resolves physically + skills:
    let mut sess = start_locked_taixu_season(
        9901,
        "ChongYingChuLin",
        1,
        "liubei",
        "pinganwushi",
        Some("jiuwei"),
        3,
        verified,
    )
    .unwrap();
    let b_ref = refs
        .iter()
        .find(|r| {
            r.season == "ChongYingChuLin"
                && r.def_idx >= 0
                && get_dlc_event_by_idx(r.def_idx as usize).is_some_and(|d| !d.enemies.is_empty())
        })
        .unwrap()
        .def_idx as usize;

    for (equip_name, expected_card, expected_skill) in [
        ("yajiaoqiang", "yajiaoqiang", Some("yajiaoqiang_skill")),
        ("nvzhuang", "nvzhuang", Some("nvzhuang")),
        ("yitianjian", "yitianjian", Some("yitianjian")),
        ("wushuangfangtianji", "wushuangfangtianji", Some("wushuangfangtianji_skill")),
        ("guilongzhanyuedao", "guilongzhanyuedao", Some("guilongzhanyuedao")),
        ("wuxinghelingshan", "wuxinghelingshan", Some("wuxinghelingshan_skill")),
        ("txhj_zhenhunqin", "txhj_zhenhunqin", Some("txhj_zhenhunqin")),
        ("txhj_xieshenmianju", "txhj_xieshenmianju", Some("txhj_xieshenmianju")),
        ("txhj_shengguangbaiyi", "txhj_shengguangbaiyi", Some("txhj_shengguangbaiyi")),
        ("txhj_chiyanzhenhunqin", "txhj_chiyanzhenhunqin", Some("txhj_chiyanzhenhunqin")),
        ("chitu", "horse_minus", None),
        ("rewrite_bagua", "armor", None),
        ("rewrite_baiyin", "baiyin", None),
        ("rewrite_tengjia", "tengjia", None),
        ("rewrite_lanyinjia", "lanyinjia", None),
        ("rewrite_renwang", "renwang", None),
        ("rewrite_zhuge", "zhuge", None),
    ] {
        let is_weapon = matches!(
            equip_name,
            "yajiaoqiang"
                | "yitianjian"
                | "wushuangfangtianji"
                | "guilongzhanyuedao"
                | "wuxinghelingshan"
                | "txhj_zhenhunqin"
                | "txhj_chiyanzhenhunqin"
                | "rewrite_zhuge"
        );
        let is_horse = equip_name == "chitu";
        sess.equip1 = if is_weapon { Some(equip_name.into()) } else { None };
        sess.equip2 = if !is_weapon && !is_horse {
            Some(equip_name.into())
        } else {
            None
        };
        sess.equip3 = None;
        sess.equip4 = if is_horse { Some(equip_name.into()) } else { None };
        let e = build_taixu_dlc_battle(&sess, b_ref, verified);
        let eq_names: Vec<&str> = e
            .debug_equips(0)
            .into_iter()
            .map(|c| e.catalog().cards[c as usize].name)
            .collect();
        assert!(
            eq_names.contains(&expected_card),
            "equip `{equip_name}` should materialize as `{expected_card}`, got {eq_names:?}"
        );
        if let Some(sk) = expected_skill {
            assert!(
                e.debug_has_skill(0, sk),
                "equip `{equip_name}` must attach skill `{sk}`"
            );
        }
    }

    // 3. `ChongYingChuLin` (`txhj_yingjiuhandi`) resolves `lijue`, `guosi`, `zhangji` with authentic stats/skills + AI play:
    let yingjiu_idx = (0usize..)
        .map_while(get_dlc_event_by_idx)
        .position(|d| d.id == "txhj_yingjiuhandi")
        .expect("txhj_yingjiuhandi DLC event");
    let mut e = build_taixu_dlc_battle(&sess, yingjiu_idx, verified);
    let mut found = Vec::new();
    for s in 0..e.state_core().players.len() as PlayerId {
        if let Some(g) = e.state_core().player(s).general_slots[0] {
            if let Some((name, _, _, _, _)) = e.catalog().general_meta(g) {
                if matches!(name, "lijue" | "guosi" | "zhangji") {
                    found.push((name.to_string(), e.state_core().player(s).max_hp, s));
                }
            }
        }
    }
    assert_eq!(
        found.len(),
        3,
        "expected all 3 non-txhj generals in txhj_yingjiuhandi: {found:?}"
    );
    let ai = BoundedPlanner::new(16);
    for _ in 0..12 {
        if e.outcome().is_some() {
            break;
        }
        let actor = e.open_choice().map_or(e.state_core().turn.current, |c| c.observer);
        let act = e.plan_action(actor, &ai).action;
        let out = submit(&mut e, actor, act);
        assert!(is_accepted(&out), "public-info AI action rejected: {out:?}");
    }
}

#[test]
fn p3d_real_combat_yajiaoqiang_and_nvzhuang_parity() {
    let verified = taixu_verified();
    let sess = start_locked_taixu_season(
        4201,
        "ChongYingChuLin",
        1,
        "liubei",
        "pinganwushi",
        None,
        0,
        verified,
    )
    .unwrap();
    let boss_idx = (0usize..)
        .map_while(get_dlc_event_by_idx)
        .position(|d| d.enemies.len() == 1 && d.friends.is_empty())
        .unwrap();

    // --- Part A: `yajiaoqiang` (`yajiaoqiang_skill`, zhulu.js:360-381,824-838) ---
    let mut ey = build_taixu_dlc_battle(&sess, boss_idx, verified);
    clear_seat_skills(&mut ey, 0);
    clear_seat_skills(&mut ey, 1);
    drain_to_play_phase(&mut ey, 0);

    let yjq = claim_named_card(&mut ey, "yajiaoqiang");
    let duel1 = claim_named_card(&mut ey, "duel");
    let wuzhong1 = claim_named_card(&mut ey, "wuzhong");
    let wuzhong2 = claim_named_card(&mut ey, "wuzhong");
    let black_sha1 = claim_card_matching(&mut ey, |cd| {
        cd.kind == CardKind::Sha && matches!(cd.suit, Suit::Spade | Suit::Club)
    });
    let black_sha2 = claim_card_matching(&mut ey, |cd| {
        cd.kind == CardKind::Sha && matches!(cd.suit, Suit::Spade | Suit::Club)
    });
    let red_shan = claim_card_matching(&mut ey, |cd| {
        cd.kind == CardKind::Shan && matches!(cd.suit, Suit::Heart | Suit::Diamond)
    });
    let red_sha = claim_card_matching(&mut ey, |cd| {
        cd.kind == CardKind::Sha && matches!(cd.suit, Suit::Heart | Suit::Diamond)
    });
    let black_wuxie1 = claim_card_matching(&mut ey, |cd| {
        cd.kind == CardKind::Nullify && matches!(cd.suit, Suit::Spade | Suit::Club)
    });
    let black_wuxie2 = claim_card_matching(&mut ey, |cd| {
        cd.kind == CardKind::Nullify && matches!(cd.suit, Suit::Spade | Suit::Club)
    });

    // Advance cleanly to P1's Play phase, equip yajiaoqiang via PlayCard:
    move_hand_to_discard(&mut ey, 0);
    move_hand_to_discard(&mut ey, 1);
    ey.state_core_mut().player_mut(1).hand.extend([yjq, black_sha1]);
    drain_to_play_phase(&mut ey, 1);
    assert!(is_accepted(&submit(
        &mut ey,
        1,
        GameAction::PlayCard {
            card: yjq,
            targets: Vec::new(),
        },
    )));
    assert_eq!(
        ey.state_core().attack_range(ey.catalog(), 1),
        3,
        "yajiaoqiang attackFrom:-2 sets weapon attack range to 3"
    );
    // Black card in OWN turn must NOT trigger yajiaoqiang_skill!
    ey.state_core_mut().player_mut(1).max_hp = 6;
    ey.state_core_mut().player_mut(1).hp = 6;
    ey.state_core_mut().player_mut(0).max_hp = 6;
    ey.state_core_mut().player_mut(0).hp = 6;
    assert!(is_accepted(&submit(
        &mut ey,
        1,
        GameAction::PlayCard {
            card: black_sha1,
            targets: vec![0],
        },
    )));
    assert!(ey.open_choice().is_none(), "no yajiaoqiang_skill prompt on own turn");
    assert_eq!(ey.state_core().player(0).hp, 5, "own-turn Sha hits P0 6 -> 5");

    // Advance to P0's turn; set up hands cleanly:
    drain_to_play_phase(&mut ey, 0);
    move_hand_to_discard(&mut ey, 0);
    move_hand_to_discard(&mut ey, 1);
    ey.state_core_mut()
        .player_mut(1)
        .hand
        .extend([black_sha2, red_shan, black_wuxie1, black_wuxie2]);
    ey.state_core_mut()
        .player_mut(0)
        .hand
        .extend([duel1, wuzhong1, wuzhong2]);

    // 1. Negative (Duel respond): P1 responds BLACK Sha in Duel -> `respond` (NOT `useCard`), must NOT trigger or consume black `useCard`!
    assert!(is_accepted(&submit(
        &mut ey,
        0,
        GameAction::PlayCard {
            card: duel1,
            targets: vec![1],
        },
    )));
    decline_open_choice_if_matching(&mut ey, "nullify_window");
    assert_eq!(ey.open_choice().map(|c| (c.observer, c.spec.prompt)), Some((1, "duel_sha")));
    assert!(is_accepted(&answer(&mut ey, 1, vec![ChoiceOption::Card(black_sha2)])));
    assert_eq!(
        ey.open_choice().map(|c| (c.observer, c.spec.prompt)),
        None,
        "Duel black Sha respond must NOT trigger yajiaoqiang_skill"
    );
    assert_eq!(ey.state_core().player(0).hp, 4, "Duel hits P0 5 -> 4");

    // 2. Negative (Red useCard): P0 plays red_sha -> P1 dodges with RED Shan (`useCard`) -> must NOT trigger or consume black `useCard`!
    ey.state_core_mut().player_mut(0).hand.push(red_sha);
    assert!(is_accepted(&submit(
        &mut ey,
        0,
        GameAction::PlayCard {
            card: red_sha,
            targets: vec![1],
        },
    )));
    assert_eq!(ey.open_choice().map(|c| (c.observer, c.spec.prompt)), Some((1, "respond_shan")));
    assert!(is_accepted(&answer(&mut ey, 1, vec![ChoiceOption::Card(red_shan)])));
    assert!(ey.open_choice().is_none(), "red Shan must NOT trigger yajiaoqiang_skill");

    // 3. Positive (1st black useCard): P0 plays `wuzhong1` -> P1 uses FIRST black Nullify (`black_wuxie1`)!
    assert!(is_accepted(&submit(
        &mut ey,
        0,
        GameAction::PlayCard {
            card: wuzhong1,
            targets: Vec::new(),
        },
    )));
    assert_eq!(
        ey.open_choice().map(|c| (c.observer, c.spec.prompt)),
        Some((1, "nullify_window"))
    );
    assert!(is_accepted(&answer(&mut ey, 1, vec![ChoiceOption::Card(black_wuxie1)])));
    decline_open_choice_if_matching(&mut ey, "nullify_window");
    assert_eq!(
        ey.open_choice().map(|c| (c.observer, c.spec.prompt)),
        Some((1, "yajiaoqiang_skill:confirm")),
        "first black card used in P0 turn must prompt yajiaoqiang_skill:confirm for P1"
    );

    // NGSA v107 save/restore mid `yajiaoqiang_skill:confirm`:
    let saved_bytes = encode_archive(&ey.save());
    assert_eq!(
        u16::from_le_bytes(saved_bytes[4..6].try_into().unwrap()),
        ARCHIVE_FORMAT_VERSION
    );
    assert_eq!(ARCHIVE_FORMAT_VERSION, 107);
    let hdr = decode_archive(&saved_bytes).expect("decode NGSA 107 archive");
    let mut ey_restored = restore_locked_engine(&hdr, verified).expect("restore NGSA");
    assert!(is_accepted(&answer(&mut ey_restored, 1, vec![ChoiceOption::Confirm])));
    assert!(is_accepted(&answer(&mut ey, 1, vec![ChoiceOption::Confirm])));
    assert_eq!(ey.state_hash(), ey_restored.state_hash());
    assert!(
        ey.state_core().player(1).hand.contains(&black_wuxie1),
        "confirming yajiaoqiang_skill recovers black_wuxie1 into P1 hand"
    );
    decline_open_choice_if_matching(&mut ey, "nullify_window");

    // 4. Negative (2nd black useCard in same turn): P0 plays `wuzhong2` -> P1 uses `black_wuxie2` -> MUST NOT trigger `yajiaoqiang_skill` again!
    assert!(is_accepted(&submit(
        &mut ey,
        0,
        GameAction::PlayCard {
            card: wuzhong2,
            targets: Vec::new(),
        },
    )));
    assert_eq!(
        ey.open_choice().map(|c| (c.observer, c.spec.prompt)),
        Some((1, "nullify_window"))
    );
    assert!(is_accepted(&answer(&mut ey, 1, vec![ChoiceOption::Card(black_wuxie2)])));
    decline_open_choice_if_matching(&mut ey, "nullify_window");
    assert_ne!(
        ey.open_choice().map(|c| c.spec.prompt),
        Some("yajiaoqiang_skill:confirm"),
        "2nd black card in same turn must NOT trigger yajiaoqiang"
    );
    assert!(ey.open_choice().is_none());

    // --- Part B: `nvzhuang` (yongjian.js:220-253,360-390) ---
    let mut en = build_taixu_dlc_battle(&sess, boss_idx, verified);
    clear_seat_skills(&mut en, 0);
    clear_seat_skills(&mut en, 1);
    drain_to_play_phase(&mut en, 1);
    equip_real_card(&mut en, 1, "nvzhuang");

    let male_liubei = (0..en.catalog().general_count())
        .find_map(|i| {
            let idx = i as u16;
            en.catalog().general_meta(idx).is_some_and(|m| m.0 == "刘备").then_some(idx)
        })
        .expect("standard 刘备");
    en.state_core_mut().players[1].general_slots[0] = Some(male_liubei);

    // Red card on top of deck -> verify nvzhuang does NOT act as Bagua Shield, AND red judge keeps Sha damage = 1!
    let top_red = claim_card_matching(&mut en, |cd| matches!(cd.suit, Suit::Heart | Suit::Diamond));
    let top_black = claim_card_matching(&mut en, |cd| matches!(cd.suit, Suit::Spade | Suit::Club));
    let shan_n = claim_named_card(&mut en, "shan");
    en.state_core_mut().player_mut(1).max_hp = 6;
    en.state_core_mut().player_mut(1).hp = 6;
    move_hand_to_discard(&mut en, 1);
    en.state_core_mut().player_mut(1).hand.push(shan_n);
    drain_to_play_phase(&mut en, 0);
    en.state_core_mut().deck.push(top_red);
    let s1 = claim_card_matching(&mut en, |cd| cd.kind == CardKind::Sha);
    move_hand_to_discard(&mut en, 0);
    en.state_core_mut().player_mut(0).hand.push(s1);
    assert!(is_accepted(&submit(
        &mut en,
        0,
        GameAction::PlayCard {
            card: s1,
            targets: vec![1],
        },
    )));
    assert_eq!(en.open_choice().map(|c| c.spec.prompt), Some("respond_shan"));
    assert!(is_accepted(&answer(&mut en, 1, vec![ChoiceOption::Pass])));
    assert_eq!(en.state_core().player(1).hp, 5, "red nvzhuang judge keeps Sha dmg = 1 & no Bagua");

    // Next Sha with BLACK card on top of deck -> nvzhuang adds +1 damage (deals 2 damage total)!
    advance_to_next_play_phase(&mut en, 0);
    en.state_core_mut().deck.push(top_black);
    let s2 = claim_card_matching(&mut en, |cd| cd.kind == CardKind::Sha);
    en.state_core_mut().player_mut(0).hand.push(s2);
    assert!(is_accepted(&submit(
        &mut en,
        0,
        GameAction::PlayCard {
            card: s2,
            targets: vec![1],
        },
    )));
    assert_eq!(en.open_choice().map(|c| c.spec.prompt), Some("respond_shan"));
    assert!(is_accepted(&answer(&mut en, 1, vec![ChoiceOption::Pass])));
    assert_eq!(en.state_core().player(1).hp, 3, "black nvzhuang judge on male adds +1 Sha dmg (5 -> 3)");

    // Female general (`甄姬`) wearing `nvzhuang` + black top card takes only 1 dmg!
    let std_zhenji = (0..en.catalog().general_count())
        .find_map(|i| {
            let idx = i as u16;
            en.catalog().general_meta(idx).is_some_and(|m| m.0 == "甄姬").then_some(idx)
        })
        .expect("standard 甄姬");
    en.state_core_mut().players[1].general_slots[0] = Some(std_zhenji);
    advance_to_next_play_phase(&mut en, 0);
    move_hand_to_discard(&mut en, 1);
    let shan_n3 = claim_named_card(&mut en, "shan");
    en.state_core_mut().player_mut(1).hand.push(shan_n3);
    let top_black2 = claim_card_matching(&mut en, |cd| matches!(cd.suit, Suit::Spade | Suit::Club));
    en.state_core_mut().deck.push(top_black2);
    let s3 = claim_card_matching(&mut en, |cd| cd.kind == CardKind::Sha);
    en.state_core_mut().player_mut(0).hand.push(s3);
    assert!(is_accepted(&submit(
        &mut en,
        0,
        GameAction::PlayCard {
            card: s3,
            targets: vec![1],
        },
    )));
    assert_eq!(en.open_choice().map(|c| c.spec.prompt), Some("respond_shan"));
    assert!(is_accepted(&answer(&mut en, 1, vec![ChoiceOption::Pass])));
    assert_eq!(en.state_core().player(1).hp, 2, "female wearing nvzhuang takes normal 1 dmg");

    // Male with source wearing `qinggang` ignores `nvzhuang` armor -> 1 dmg even with black top card!
    en.state_core_mut().players[1].general_slots[0] = Some(male_liubei);
    advance_to_next_play_phase(&mut en, 0);
    move_hand_to_discard(&mut en, 1);
    let shan_n4 = claim_named_card(&mut en, "shan");
    en.state_core_mut().player_mut(1).hand.push(shan_n4);
    equip_real_card(&mut en, 0, "qinggang");
    let top_black3 = claim_card_matching(&mut en, |cd| matches!(cd.suit, Suit::Spade | Suit::Club));
    en.state_core_mut().deck.push(top_black3);
    let s4 = claim_card_matching(&mut en, |cd| cd.kind == CardKind::Sha);
    en.state_core_mut().player_mut(0).hand.push(s4);
    assert!(is_accepted(&submit(
        &mut en,
        0,
        GameAction::PlayCard {
            card: s4,
            targets: vec![1],
        },
    )));
    assert_eq!(en.open_choice().map(|c| c.spec.prompt), Some("respond_shan"));
    assert!(is_accepted(&answer(&mut en, 1, vec![ChoiceOption::Pass])));
    assert_eq!(en.state_core().player(1).hp, 1, "qinggang pierces nvzhuang -> normal 1 dmg");
}

#[test]
fn p3d_real_combat_txhj_xieshenmianju_shengguangbaiyi_and_zhenhunqin() {
    let verified = taixu_verified();
    let sess = start_locked_taixu_season(
        5201,
        "ChongYingChuLin",
        1,
        "liubei",
        "pinganwushi",
        None,
        0,
        verified,
    )
    .unwrap();
    let three_seat_idx = (0usize..)
        .map_while(get_dlc_event_by_idx)
        .position(|d| d.enemies.len() == 2 && d.friends.is_empty())
        .unwrap();

    // --- Part A: `txhj_xieshenmianju` (`turnOverBefore` cancel + `damage >= 2` reduced by 1) ---
    let mut ex = build_taixu_dlc_battle(&sess, three_seat_idx, verified);
    for p in 0..3 {
        clear_seat_skills(&mut ex, p);
    }
    drain_to_play_phase(&mut ex, 1);
    equip_real_card(&mut ex, 1, "txhj_xieshenmianju");
    ex.state_core_mut().player_mut(1).max_hp = 8;
    ex.state_core_mut().player_mut(1).hp = 8;
    move_hand_to_discard(&mut ex, 1);

    drain_to_play_phase(&mut ex, 0);
    // Play `txhj_diaohu` targeting Seat 1: Seat 1's `txhj_xieshenmianju` blocks `TurnOverBefore`!
    let diaohu = claim_named_card(&mut ex, "txhj_diaohu");
    ex.state_core_mut().player_mut(0).hand.push(diaohu);
    assert!(is_accepted(&submit(
        &mut ex,
        0,
        GameAction::PlayCard {
            card: diaohu,
            targets: vec![1],
        },
    )));
    assert!(!ex.state_core().player(1).face_down, "txhj_xieshenmianju cancels TurnOverBefore");

    // Drink `Jiu` + play `Sha` -> 2 base dmg reduced to 1 on Seat 1 (`8 -> 7`)!
    let shan_x = claim_named_card(&mut ex, "shan");
    ex.state_core_mut().player_mut(1).hand.push(shan_x);
    let jiu = claim_card_matching(&mut ex, |cd| cd.kind == CardKind::Jiu);
    let sha1 = claim_card_matching(&mut ex, |cd| cd.kind == CardKind::Sha);
    ex.state_core_mut().player_mut(0).hand.extend([jiu, sha1]);
    assert!(is_accepted(&submit(
        &mut ex,
        0,
        GameAction::PlayCard {
            card: jiu,
            targets: Vec::new(),
        },
    )));
    assert!(is_accepted(&submit(
        &mut ex,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
        },
    )));
    assert_eq!(ex.open_choice().map(|c| c.spec.prompt), Some("respond_shan"));
    assert!(is_accepted(&answer(&mut ex, 1, vec![ChoiceOption::Pass])));
    assert_eq!(ex.state_core().player(1).hp, 7, "txhj_xieshenmianju reduces 2 dmg -> 1");

    // Unbuffed 1 dmg `Sha` stays 1 (`num <= 1` untouched: `7 -> 6`)!
    advance_to_next_play_phase(&mut ex, 0);
    let sha2 = claim_card_matching(&mut ex, |cd| cd.kind == CardKind::Sha);
    ex.state_core_mut().player_mut(0).hand.push(sha2);
    assert!(is_accepted(&submit(
        &mut ex,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    )));
    assert_eq!(ex.open_choice().map(|c| c.spec.prompt), Some("respond_shan"));
    assert!(is_accepted(&answer(&mut ex, 1, vec![ChoiceOption::Pass])));
    assert_eq!(ex.state_core().player(1).hp, 6, "txhj_xieshenmianju preserves 1-point damage");

    // --- Part B: `txhj_shengguangbaiyi` (immune to red Sha + maxHandcard +2) ---
    let mut es = build_taixu_dlc_battle(&sess, three_seat_idx, verified);
    for p in 0..3 {
        clear_seat_skills(&mut es, p);
    }
    drain_to_play_phase(&mut es, 1);
    equip_real_card(&mut es, 1, "txhj_shengguangbaiyi");
    es.state_core_mut().player_mut(1).max_hp = 4;
    es.state_core_mut().player_mut(1).hp = 2;
    move_hand_to_discard(&mut es, 1);
    let shan_s = claim_named_card(&mut es, "shan");
    es.state_core_mut().player_mut(1).hand.push(shan_s);
    for _ in 0..3 {
        let c = es.state_core_mut().deck.pop().unwrap();
        es.state_core_mut().player_mut(1).hand.push(c);
    }
    // Hand limit with hp=2 + shengguangbaiyi(+2) = 4: Ending play phase discards 0!
    assert!(is_accepted(&submit(&mut es, 1, GameAction::NextPhase)));
    assert!(es.open_choice().is_none(), "shengguangbaiyi +2 hand limit keeps 4 cards at hp=2");
    assert_eq!(es.state_core().player(1).hand.len(), 4);

    // P0 plays RED Sha -> cancelled by txhj_shengguangbaiyi (0 damage, no Shan prompt);
    // P0 plays BLACK Sha -> hits normally (`2 -> 1`)!
    drain_to_play_phase(&mut es, 0);
    let zhuge = claim_named_card(&mut es, "zhuge");
    let red_sha = claim_card_matching(&mut es, |cd| {
        cd.kind == CardKind::Sha && matches!(cd.suit, Suit::Heart | Suit::Diamond)
    });
    let black_sha = claim_card_matching(&mut es, |cd| {
        cd.kind == CardKind::Sha && matches!(cd.suit, Suit::Spade | Suit::Club)
    });
    es.state_core_mut().player_mut(0).hand.extend([zhuge, red_sha, black_sha]);
    assert!(is_accepted(&submit(
        &mut es,
        0,
        GameAction::PlayCard {
            card: zhuge,
            targets: Vec::new(),
        },
    )));
    assert!(is_accepted(&submit(
        &mut es,
        0,
        GameAction::PlayCard {
            card: red_sha,
            targets: vec![1],
        },
    )));
    assert!(es.open_choice().is_none(), "red Sha negated by txhj_shengguangbaiyi");
    assert_eq!(es.state_core().player(1).hp, 2);

    assert!(is_accepted(&submit(
        &mut es,
        0,
        GameAction::PlayCard {
            card: black_sha,
            targets: vec![1],
        },
    )));
    assert_eq!(es.open_choice().map(|c| c.spec.prompt), Some("respond_shan"));
    assert!(is_accepted(&answer(&mut es, 1, vec![ChoiceOption::Pass])));
    assert_eq!(es.state_core().player(1).hp, 1, "black Sha still hits through txhj_shengguangbaiyi");

    // --- Part C: `txhj_zhenhunqin` (range 4 weapon + normal Sha -> Thunder + NGSA save/restore + chain conduction) ---
    let mut ez = build_taixu_dlc_battle(&sess, three_seat_idx, verified);
    for p in 0..3 {
        clear_seat_skills(&mut ez, p);
        ez.state_core_mut().player_mut(p).max_hp = 5;
        ez.state_core_mut().player_mut(p).hp = 5;
    }
    drain_to_play_phase(&mut ez, 0);
    equip_real_card(&mut ez, 0, "txhj_zhenhunqin");
    assert_eq!(
        ez.state_core().attack_range(ez.catalog(), 0),
        4,
        "txhj_zhenhunqin attackFrom:-3 sets weapon attack range to 4"
    );
    let ch1 = claim_named_card(&mut ez, "chain");
    let ch2 = claim_named_card(&mut ez, "chain");
    let shan_z = claim_named_card(&mut ez, "shan");
    let norm_sha = claim_card_matching(&mut ez, |cd| cd.kind == CardKind::Sha);
    move_hand_to_discard(&mut ez, 1);
    move_hand_to_discard(&mut ez, 2);
    ez.state_core_mut().player_mut(1).hand.push(shan_z);
    ez.state_core_mut().player_mut(0).hand.extend([ch1, ch2, norm_sha]);
    assert!(is_accepted(&submit(
        &mut ez,
        0,
        GameAction::PlayCard {
            card: ch1,
            targets: vec![1],
        },
    )));
    assert!(is_accepted(&submit(
        &mut ez,
        0,
        GameAction::PlayCard {
            card: ch2,
            targets: vec![2],
        },
    )));
    assert!(ez.state_core().player(1).linked && ez.state_core().player(2).linked);

    assert!(is_accepted(&submit(
        &mut ez,
        0,
        GameAction::PlayCard {
            card: norm_sha,
            targets: vec![1],
        },
    )));
    assert_eq!(
        ez.open_choice().map(|c| c.spec.prompt),
        Some("txhj_zhenhunqin:confirm"),
        "normal Sha opens txhj_zhenhunqin:confirm"
    );

    // NGSA v107 archive save/restore mid `txhj_zhenhunqin:confirm`:
    let zq_bytes = encode_archive(&ez.save());
    assert_eq!(
        u16::from_le_bytes(zq_bytes[4..6].try_into().unwrap()),
        ARCHIVE_FORMAT_VERSION
    );
    let zq_snap = decode_archive(&zq_bytes).expect("decode NGSA mid txhj_zhenhunqin");
    let mut ez_restored = restore_locked_engine(&zq_snap, verified).expect("restore");
    let mut ez_declined = restore_locked_engine(&zq_snap, verified).expect("restore decline");

    assert!(is_accepted(&answer(&mut ez_restored, 0, vec![ChoiceOption::Confirm])));
    assert!(is_accepted(&answer(&mut ez, 0, vec![ChoiceOption::Confirm])));
    assert_eq!(ez.open_choice().map(|c| c.spec.prompt), Some("respond_shan"));
    assert!(is_accepted(&answer(&mut ez, 1, vec![ChoiceOption::Pass])));
    assert!(is_accepted(&answer(&mut ez_restored, 1, vec![ChoiceOption::Pass])));
    assert_eq!(ez.state_hash(), ez_restored.state_hash());
    // Thunder damage conducts to chained P2!
    assert_eq!(ez.state_core().player(1).hp, 4, "P1 takes 1 thunder damage");
    assert_eq!(
        ez.state_core().player(2).hp,
        4,
        "P2 takes 1 chained thunder damage from txhj_zhenhunqin"
    );

    // Negative 1: declining `txhj_zhenhunqin:confirm` keeps Sha Normal (does NOT conduct to P2!).
    assert!(is_accepted(&answer(&mut ez_declined, 0, Vec::new())));
    assert_eq!(ez_declined.open_choice().map(|c| c.spec.prompt), Some("respond_shan"));
    assert!(is_accepted(&answer(&mut ez_declined, 1, vec![ChoiceOption::Pass])));
    assert_eq!(ez_declined.state_core().player(1).hp, 4, "P1 takes 1 Normal damage");
    assert_eq!(
        ez_declined.state_core().player(2).hp,
        5,
        "declining txhj_zhenhunqin must NOT conduct Thunder damage to P2"
    );

    // Negative 2: FireSha while `txhj_zhenhunqin` is STILL equipped must NOT trigger `txhj_zhenhunqin:confirm`!
    advance_to_next_play_phase(&mut ez, 0);
    let fire_sha = claim_card_matching(&mut ez, |cd| cd.kind == CardKind::FireSha);
    ez.state_core_mut().player_mut(0).hand.push(fire_sha);
    assert!(is_accepted(&submit(
        &mut ez,
        0,
        GameAction::PlayCard {
            card: fire_sha,
            targets: vec![1],
        },
    )));
    assert_ne!(
        ez.open_choice().map(|c| c.spec.prompt),
        Some("txhj_zhenhunqin:confirm"),
        "elemental FireSha must NOT trigger txhj_zhenhunqin"
    );
    decline_open_choice_if_matching(&mut ez, "respond_shan");
}

#[test]
fn p3d_yajiaoqiang_wanjian_and_cancelled_usecard_parity() {
    let verified = taixu_verified();
    let sess = start_locked_taixu_season(
        2026_1124,
        "ChongYingChuLin",
        1,
        "liubei",
        "pinganwushi",
        None,
        0,
        verified,
    )
    .unwrap();
    let boss_idx = (0usize..)
        .map_while(get_dlc_event_by_idx)
        .position(|d| d.enemies.len() == 1 && d.friends.is_empty())
        .unwrap();

    let grant_skill = |e: &mut Engine, seat: PlayerId, name: &str| {
        let def = e
            .catalog()
            .skill_by_name(name)
            .unwrap_or_else(|| panic!("missing skill `{name}`"));
        e.state_core_mut().skills.insert(
            (seat, def),
            engine_native::state::SkillInstance {
                owner: seat,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
    };

    // Part A: `ArcheryAttack` (`Wanjian`) `Shan` respond MUST NOT trigger or consume `yajiaoqiang_skill`,
    // whereas `Wanjian` `Nullify` (`Wuxie`), `Sha` black `Shan` dodge, and `qingguo` black dodge DO!
    let mut ey = build_taixu_dlc_battle(&sess, boss_idx, verified);
    clear_seat_skills(&mut ey, 0);
    clear_seat_skills(&mut ey, 1);
    grant_skill(&mut ey, 1, engine_native::content::standard::QING_GUO);
    drain_to_play_phase(&mut ey, 1);

    let yjq = claim_named_card(&mut ey, "yajiaoqiang");
    ey.state_core_mut().player_mut(1).hand.push(yjq);
    assert!(is_accepted(&submit(
        &mut ey,
        1,
        GameAction::PlayCard {
            card: yjq,
            targets: Vec::new(),
        },
    )));

    ey.state_core_mut().player_mut(0).max_hp = 6;
    ey.state_core_mut().player_mut(0).hp = 6;
    ey.state_core_mut().player_mut(1).max_hp = 6;
    ey.state_core_mut().player_mut(1).hp = 6;

    let archery1 = claim_card_matching(&mut ey, |cd| cd.kind == CardKind::ArcheryAttack);
    let archery2 = claim_card_matching(&mut ey, |cd| cd.kind == CardKind::ArcheryAttack);
    let sha1 = claim_card_matching(&mut ey, |cd| cd.kind == CardKind::Sha);
    let sha2 = claim_card_matching(&mut ey, |cd| cd.kind == CardKind::Sha);
    let black_as_shan1 = claim_card_matching(&mut ey, |cd| {
        cd.kind == CardKind::Sha && matches!(cd.suit, Suit::Spade | Suit::Club)
    });
    let black_as_shan2 = claim_card_matching(&mut ey, |cd| {
        cd.kind == CardKind::Dismantle && matches!(cd.suit, Suit::Spade | Suit::Club)
    });
    let black_sha_qg = claim_card_matching(&mut ey, |cd| {
        cd.kind == CardKind::Sha && matches!(cd.suit, Suit::Spade | Suit::Club)
    });
    let black_wuxie = claim_card_matching(&mut ey, |cd| {
        cd.kind == CardKind::Nullify && matches!(cd.suit, Suit::Spade | Suit::Club)
    });

    drain_to_play_phase(&mut ey, 0);
    move_hand_to_discard(&mut ey, 0);
    move_hand_to_discard(&mut ey, 1);
    ey.state_core_mut()
        .declared_kinds
        .insert(black_as_shan1, CardKind::Shan);
    ey.state_core_mut()
        .declared_kinds
        .insert(black_as_shan2, CardKind::Shan);
    ey.state_core_mut()
        .player_mut(1)
        .hand
        .extend([black_as_shan1, black_as_shan2, black_sha_qg, black_wuxie]);
    ey.state_core_mut().player_mut(0).hand.extend([archery1, sha1]);

    // 1) P0 plays Wanjian -> P1 responds BLACK Shan (`respond`, NOT `useCard`) -> NO yajiaoqiang prompt or consumption!
    assert!(is_accepted(&submit(
        &mut ey,
        0,
        GameAction::PlayCard {
            card: archery1,
            targets: Vec::new(),
        },
    )));
    decline_open_choice_if_matching(&mut ey, "nullify_window");
    decline_open_choice_if_matching(&mut ey, "nullify_window");
    assert_eq!(ey.open_choice().map(|c| (c.observer, c.spec.prompt)), Some((1, "aoe_response")));
    assert!(is_accepted(&answer(&mut ey, 1, vec![ChoiceOption::Card(black_as_shan1)])));
    assert_eq!(
        ey.open_choice().map(|c| (c.observer, c.spec.prompt)),
        None,
        "Wanjian black Shan respond must NOT trigger yajiaoqiang_skill"
    );
    assert_eq!(ey.state_core().player(1).hp, 6, "Wanjian Shan dodged damage");
    assert!(!ey.state_core().player(1).hand.contains(&black_as_shan1));

    // 2) Same P0 turn: P0 plays Sha -> P1 dodges with BLACK Shan (`useCard`) -> MUST prompt & recover!
    assert!(is_accepted(&submit(
        &mut ey,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
        },
    )));
    assert_eq!(ey.open_choice().map(|c| (c.observer, c.spec.prompt)), Some((1, "respond_shan")));
    assert!(is_accepted(&answer(&mut ey, 1, vec![ChoiceOption::Card(black_as_shan2)])));
    assert_eq!(
        ey.open_choice().map(|c| (c.observer, c.spec.prompt)),
        Some((1, "yajiaoqiang_skill:confirm")),
        "Sha black Shan dodge on same turn after Wanjian must still trigger yajiaoqiang_skill"
    );
    assert!(is_accepted(&answer(&mut ey, 1, vec![ChoiceOption::Confirm])));
    assert_eq!(
        ey.state_core().player(1).hand.iter().filter(|&&c| c == black_as_shan2).count(),
        1,
        "confirming yajiaoqiang_skill recovers black_as_shan2 once into P1 hand"
    );

    // 3) Next P0 turn: `qingguo` black non-Shan card (`black_sha_qg`) dodging `Sha` triggers yajiaoqiang_skill!
    advance_to_next_play_phase(&mut ey, 0);
    ey.state_core_mut().player_mut(0).hand.push(sha2);
    assert!(is_accepted(&submit(
        &mut ey,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    )));
    assert_eq!(ey.open_choice().map(|c| (c.observer, c.spec.prompt)), Some((1, "respond_shan")));
    assert!(is_accepted(&answer(&mut ey, 1, vec![ChoiceOption::Card(black_sha_qg)])));
    assert_eq!(
        ey.open_choice().map(|c| (c.observer, c.spec.prompt)),
        Some((1, "yajiaoqiang_skill:confirm")),
        "qingguo black card dodging Sha must trigger yajiaoqiang_skill"
    );
    assert!(is_accepted(&answer(&mut ey, 1, vec![ChoiceOption::Confirm])));
    assert!(ey.state_core().player(1).hand.contains(&black_sha_qg));

    // 4) Next P0 turn: playing black `Nullify` (`Wuxie`) inside `Wanjian` IS `useCard` -> MUST trigger yajiaoqiang_skill!
    advance_to_next_play_phase(&mut ey, 0);
    move_hand_to_discard(&mut ey, 0);
    move_hand_to_discard(&mut ey, 1);
    let black_wuxie = claim_card_matching(&mut ey, |cd| {
        cd.kind == CardKind::Nullify && matches!(cd.suit, Suit::Spade | Suit::Club)
    });
    ey.state_core_mut().player_mut(1).hand.push(black_wuxie);
    ey.state_core_mut().player_mut(0).hand.push(archery2);
    assert!(is_accepted(&submit(
        &mut ey,
        0,
        GameAction::PlayCard {
            card: archery2,
            targets: Vec::new(),
        },
    )));
    // Only decline nullify_window if another seat (e.g. P0) was prompted before P1:
    while ey.open_choice().is_some_and(|c| c.observer != 1 && c.spec.prompt == "nullify_window") {
        decline_open_choice_if_matching(&mut ey, "nullify_window");
    }
    assert_eq!(ey.open_choice().map(|c| (c.observer, c.spec.prompt)), Some((1, "nullify_window")));
    assert!(is_accepted(&answer(&mut ey, 1, vec![ChoiceOption::Card(black_wuxie)])));
    decline_open_choice_if_matching(&mut ey, "nullify_window");
    assert_eq!(
        ey.open_choice().map(|c| (c.observer, c.spec.prompt)),
        Some((1, "yajiaoqiang_skill:confirm")),
        "black Nullify inside Wanjian must still trigger yajiaoqiang_skill"
    );
    assert!(is_accepted(&answer(&mut ey, 1, vec![ChoiceOption::Confirm])));
    assert!(ey.state_core().player(1).hand.contains(&black_wuxie));
    assert!(ey.open_choice().is_none(), "nullified Wanjian finishes cleanly");

    // Part B: `u.cancelled == true` off-turn `CardUsed` (`useCardAfter`) MUST trigger `yajiaoqiang_skill`!
    let mut ey2 = build_taixu_dlc_battle(&sess, boss_idx, verified);
    clear_seat_skills(&mut ey2, 0);
    clear_seat_skills(&mut ey2, 1);
    grant_skill(&mut ey2, 0, "txhj_jiuweiSkill2");
    drain_to_play_phase(&mut ey2, 1);

    let yjq2 = claim_named_card(&mut ey2, "yajiaoqiang");
    ey2.state_core_mut().player_mut(1).hand.push(yjq2);
    assert!(is_accepted(&submit(
        &mut ey2,
        1,
        GameAction::PlayCard {
            card: yjq2,
            targets: Vec::new(),
        },
    )));

    let kill_sha = claim_card_matching(&mut ey2, |cd| cd.kind == CardKind::Sha);
    let black_jiu = claim_card_matching(&mut ey2, |cd| {
        cd.kind == CardKind::Jiu && matches!(cd.suit, Suit::Spade | Suit::Club)
    });
    let red_tao = claim_card_matching(&mut ey2, |cd| {
        cd.kind == CardKind::Tao && matches!(cd.suit, Suit::Heart | Suit::Diamond)
    });
    let wuzhong_after = claim_named_card(&mut ey2, "wuzhong");
    let black_wuxie_after = claim_card_matching(&mut ey2, |cd| {
        cd.kind == CardKind::Nullify && matches!(cd.suit, Suit::Spade | Suit::Club)
    });

    drain_to_play_phase(&mut ey2, 0);
    move_hand_to_discard(&mut ey2, 0);
    move_hand_to_discard(&mut ey2, 1);
    ey2.state_core_mut().player_mut(1).max_hp = 4;
    ey2.state_core_mut().player_mut(1).hp = 1;
    ey2.state_core_mut()
        .player_mut(1)
        .hand
        .extend([black_jiu, red_tao, black_wuxie_after]);
    ey2.state_core_mut().player_mut(0).hand.extend([kill_sha, wuzhong_after]);

    assert!(is_accepted(&submit(
        &mut ey2,
        0,
        GameAction::PlayCard {
            card: kill_sha,
            targets: vec![1],
        },
    )));
    assert_eq!(ey2.open_choice().map(|c| (c.observer, c.spec.prompt)), Some((1, "dying_save")));
    assert!(is_accepted(&answer(&mut ey2, 1, vec![ChoiceOption::Card(black_jiu)])));
    // P0's `txhj_jiuweiSkill2` cancels P1's `black_jiu` `CardUsed` (`u.cancelled = true`, hp remains 0).
    // `DyingMachine` already parked at depth D resumes first for `red_tao` rescue, and once `Dying`
    // completes at depth D, `useCardAfter` for the cancelled `black_jiu` still offers `yajiaoqiang_skill:confirm`!
    assert_eq!(ey2.state_core().player(1).hp, 0, "txhj_jiuweiSkill2 cancelled black_jiu heal");
    assert_eq!(ey2.open_choice().map(|c| (c.observer, c.spec.prompt)), Some((1, "dying_save")));
    assert!(is_accepted(&answer(&mut ey2, 1, vec![ChoiceOption::Card(red_tao)])));
    assert_eq!(ey2.state_core().player(1).hp, 1, "red Tao saves P1 out of dying");
    assert_eq!(
        ey2.open_choice().map(|c| (c.observer, c.spec.prompt)),
        Some((1, "yajiaoqiang_skill:confirm")),
        "cancelled black CardUsed must still trigger yajiaoqiang_skill useCardAfter"
    );
    assert!(is_accepted(&answer(&mut ey2, 1, vec![ChoiceOption::Confirm])));
    assert_eq!(
        ey2.state_core().player(1).hand.iter().filter(|&&c| c == black_jiu).count(),
        1,
        "yajiaoqiang recovers cancelled black_jiu into P1 hand without duplicating"
    );

    // 2nd black card used by P1 in the SAME P0 turn must NOT re-trigger `yajiaoqiang_skill`!
    assert!(is_accepted(&submit(
        &mut ey2,
        0,
        GameAction::PlayCard {
            card: wuzhong_after,
            targets: Vec::new(),
        },
    )));
    assert_eq!(ey2.open_choice().map(|c| (c.observer, c.spec.prompt)), Some((1, "nullify_window")));
    assert!(is_accepted(&answer(&mut ey2, 1, vec![ChoiceOption::Card(black_wuxie_after)])));
    decline_open_choice_if_matching(&mut ey2, "nullify_window");
    assert!(ey2.open_choice().is_none(), "once-per-turn consumed by cancelled first black card");

    // Part C: Nested `CardUsed` frames (`txhj_jiedao` outer frame F0 -> forced `Sha` inner frame F1 -> `qingguo` black dodge):
    // `frames.iter().rev().find(...)` MUST inspect innermost `F1` (`l11_sha_dodge_ok:{F1.id}:...`), NOT outer `F0`!
    let three_seat_idx = (0usize..)
        .map_while(get_dlc_event_by_idx)
        .position(|d| d.enemies.len() >= 2 && d.friends.is_empty())
        .expect("3-seat battle");
    let mut en3 = build_taixu_dlc_battle(&sess, three_seat_idx, verified);
    for p in 0..3 {
        clear_seat_skills(&mut en3, p);
        en3.state_core_mut().player_mut(p).max_hp = 5;
        en3.state_core_mut().player_mut(p).hp = 5;
    }
    grant_skill(&mut en3, 1, engine_native::content::standard::QING_GUO);
    drain_to_play_phase(&mut en3, 1);
    equip_real_card(&mut en3, 1, "yajiaoqiang");
    drain_to_play_phase(&mut en3, 2);
    equip_real_card(&mut en3, 2, "zhuge");

    let jiedao = claim_named_card(&mut en3, "txhj_jiedao");
    let sha_forced = claim_card_matching(&mut en3, |cd| cd.kind == CardKind::Sha);
    let black_dismantle_nested = claim_card_matching(&mut en3, |cd| {
        cd.kind == CardKind::Dismantle && matches!(cd.suit, Suit::Spade | Suit::Club)
    });

    drain_to_play_phase(&mut en3, 0);
    move_hand_to_discard(&mut en3, 0);
    move_hand_to_discard(&mut en3, 1);
    move_hand_to_discard(&mut en3, 2);
    en3.state_core_mut().player_mut(0).hand.push(jiedao);
    en3.state_core_mut().player_mut(2).hand.push(sha_forced);
    en3.state_core_mut().player_mut(1).hand.push(black_dismantle_nested);

    assert!(is_accepted(&submit(
        &mut en3,
        0,
        GameAction::PlayCard {
            card: jiedao,
            targets: vec![2, 1],
        },
    )));
    while en3.open_choice().is_some_and(|c| c.spec.prompt == "nullify_window") {
        decline_open_choice_if_matching(&mut en3, "nullify_window");
    }
    assert_eq!(
        en3.open_choice().map(|c| (c.observer, c.spec.prompt)),
        Some((2, "txhj_jiedao:choose_sha"))
    );
    assert!(is_accepted(&answer(&mut en3, 2, vec![ChoiceOption::Card(sha_forced)])));
    assert_eq!(
        en3.open_choice().map(|c| (c.observer, c.spec.prompt)),
        Some((1, "respond_shan"))
    );
    assert!(is_accepted(&answer(&mut en3, 1, vec![ChoiceOption::Card(black_dismantle_nested)])));
    assert_eq!(
        en3.open_choice().map(|c| (c.observer, c.spec.prompt)),
        Some((1, "yajiaoqiang_skill:confirm")),
        "nested txhj_jiedao -> Sha -> qingguo black dodge must resolve innermost Sha frame for yajiaoqiang_skill"
    );
    assert!(is_accepted(&answer(&mut en3, 1, vec![ChoiceOption::Confirm])));
    assert_eq!(
        en3.state_core().player(1).hand.iter().filter(|&&c| c == black_dismantle_nested).count(),
        1,
        "yajiaoqiang recovers nested black dodge card once into P1 hand"
    );
}
