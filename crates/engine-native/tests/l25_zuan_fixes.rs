//! Batch Q2-L25 规则修复与 counterexample 回归测试（`L25 祖安武将` 18 项收口）。
//!
//! 对照上游 `/Users/ruyimg/Documents/Claude/sanguosha/archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js`：
//! - `cards:ssr_jinfan` (`:6535-6613`)
//! - `configs:ssr_luoyi` (`:15031-15108`)
//! - `configs:ssr_lieren` (`:22337-22560` + `character/xianding.js:13530-13537`)
//! - `ui_hooks:_anshaskill` (`:42659-42742`), `_seadreammodeskill` (`:42815-42888`),
//!   `_relifeskill` (`:43402-43449`), `_haoyunlai` (`:47584-47660`)
//! - `skills:_guanmodeskill` (`:42978-43024` + `guanfu:1215` + `guifu_nei:1633`),
//!   `wumou` (`:48975-49031`), `new_guixin` (`:49077-49155`), `gongxin` (`:49506-49558` + `ssr_gongxin_break:13429-13460`),
//!   `_zuan_damageSendEgg` (`:40324`), `_newZuanPhaseTX` (`:41709`), `_zuanidentityshowskill` (`:42132`),
//!   `_zuanCHANGEHPsayskill` (`:46389`), `_fromsay` (`:47046`),
//!   `_bingletexiao1` (`:48747`, within `/* ... */` `:47983-48948`), `_bingletexiao2` (`:48768`, within `/* ... */` `:47983-48948`)

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::{
    decision34_pack9_extension_pack, prepare_locked_content_with_bridge, restore_locked_engine,
    start_locked_engine, validate_and_freeze_raw_package, LockedStartMode, VerifiedLockedContent,
    ZuanModeConfig, zuan_apply_engine_config,
};
use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind, Suit};
use engine_native::content::extension::ExtensionPack;
use engine_native::event::Phase;
use engine_native::state::{GameMode, Role};
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, LogEvent, OpenChoice,
};

static DEDUP: AtomicU64 = AtomicU64::new(25_900_000);

fn p9() -> ExtensionPack {
    decision34_pack9_extension_pack()
}

fn formal_zuan_verified() -> VerifiedLockedContent {
    let pkg_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/zuan-upstream-liuli");
    let raw_json = std::fs::read_to_string(pkg_dir.join("package.npack.json"))
        .expect("read zuan package.npack.json")
        .replace("\r\n", "\n");
    let parsed: serde_json::Value = serde_json::from_str(&raw_json).unwrap();
    let mut assets = BTreeMap::new();
    if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
        for rel_path in paths.values().filter_map(|v| v.as_str()) {
            let bytes = std::fs::read(pkg_dir.join(rel_path))
                .expect("read zuan asset")
                .into_iter()
                .filter(|&b| b != b'\r')
                .collect::<Vec<u8>>();
            assets.insert(rel_path.to_string(), bytes);
        }
    }
    let frozen = validate_and_freeze_raw_package(raw_json, assets).expect("freeze zuan");
    prepare_locked_content_with_bridge(vec![frozen]).expect("prepare zuan")
}

fn formal_zuan_engine(seed: u64, seats: usize) -> Engine {
    let verified = formal_zuan_verified();
    start_locked_engine(seed, seats, LockedStartMode::Prototype, &verified)
        .expect("start formal zuan engine")
}

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

fn is_rejected(out: &CommandOutcome) -> bool {
    matches!(out, CommandOutcome::Rejected(_))
}

fn clear_zone(e: &mut Engine, s: u8) {
    let h: Vec<u32> = e.state_core_mut().player_mut(s).hand.drain(..).collect();
    let eq: Vec<u32> = e.state_core_mut().player_mut(s).equips.drain(..).collect();
    let jd: Vec<u32> = e.state_core_mut().player_mut(s).judge.drain(..).collect();
    e.state_core_mut()
        .discard
        .extend(h.into_iter().chain(eq).chain(jd));
}

fn fresh_identity(p: &ExtensionPack, seed: u64, skills_by_seat: &[&[&str]]) -> Engine {
    let mut e = Engine::new_with_extensions(seed, skills_by_seat.len(), &[p]).expect("build");
    for s in 0..skills_by_seat.len() as u8 {
        e.state_core_mut().skills.retain(|&(o, _), _| o != s);
        clear_zone(&mut e, s);
        e.state_core_mut().player_mut(s).max_hp = 4;
        e.state_core_mut().player_mut(s).hp = 4;
    }
    e.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor]
            .into_iter()
            .take(skills_by_seat.len())
            .collect(),
    };
    for (s, sks) in skills_by_seat.iter().enumerate() {
        for &sk in *sks {
            e.debug_grant(s as u8, sk);
        }
    }
    e
}

fn flag(e: &Engine, p: u8, key: &str) -> u32 {
    e.state_core()
        .vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

fn set_flag(e: &mut Engine, p: u8, key: &str, val: u32) {
    if val == 0 {
        e.state_core_mut().vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        e.state_core_mut()
            .vm_aux
            .flags
            .insert((p, key.to_string()), val);
    }
}

fn has_skill(e: &Engine, p: u8, sk: &str) -> bool {
    e.catalog()
        .skill_by_name(sk)
        .is_some_and(|def| e.state_core().has_skill(p, def))
}

fn take_card(e: &mut Engine, pred: impl Fn(&engine_native::catalog::CardDef) -> bool) -> u32 {
    let cid = e
        .state_core()
        .deck
        .iter()
        .chain(e.state_core().discard.iter())
        .copied()
        .find(|&id| pred(&e.catalog().cards[id as usize]))
        .expect("matching card in deck or discard");
    e.state_core_mut().deck.retain(|&x| x != cid);
    e.state_core_mut().discard.retain(|&x| x != cid);
    cid
}

fn give_card(
    e: &mut Engine,
    seat: u8,
    pred: impl Fn(&engine_native::catalog::CardDef) -> bool,
) -> u32 {
    let c = take_card(e, pred);
    e.state_core_mut().player_mut(seat).hand.push(c);
    c
}

fn give_kind(e: &mut Engine, seat: u8, kind: CardKind) -> u32 {
    give_card(e, seat, |cd| cd.kind == kind)
}

fn play(e: &mut Engine, seat: u8, card: u32, targets: Vec<u8>) -> CommandOutcome {
    let out = submit(e, seat, GameAction::PlayCard { card, targets });
    assert!(!is_rejected(&out), "play rejected: {out:?}");
    out
}

fn activate(e: &mut Engine, seat: u8, sk: &str) -> CommandOutcome {
    let def = e.catalog().skill_by_name(sk).expect("skill def");
    submit(e, seat, GameAction::ActivateSkill { skill: def })
}

fn open(e: &Engine, prompt: &str) -> OpenChoice {
    let c = e.open_choice().cloned().expect("open choice");
    assert_eq!(c.spec.prompt, prompt, "unexpected prompt");
    c
}

fn answer(e: &mut Engine, prompt: &str, sel: Vec<ChoiceOption>) -> CommandOutcome {
    let c = open(e, prompt);
    let out = submit(
        e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection: sel,
        },
    );
    assert!(!is_rejected(&out), "answer rejected for {prompt}: {out:?}");
    out
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = submit(
            e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        );
    }
}

fn advance_to_end_stop_on_choice(e: &mut Engine, target_seat: u8) {
    for _ in 0..8 {
        if e.open_choice().is_some()
            || (e.turn().current == target_seat && e.turn().phase == Phase::End)
        {
            return;
        }
        let actor = e.turn().current;
        let _ = submit(e, actor, GameAction::NextPhase);
    }
}

fn advance_to(e: &mut Engine, target_seat: u8, target_phase: Phase) {
    for _ in 0..32 {
        drain(e);
        if e.turn().current == target_seat && e.turn().phase == target_phase {
            return;
        }
        let actor = e.turn().current;
        let _ = submit(e, actor, GameAction::NextPhase);
    }
    panic!("failed to reach ({target_seat}, {target_phase:?}), got {:?}", e.turn());
}

fn roundtrip_bytes(e: &Engine, packs: &[&ExtensionPack]) -> Engine {
    let snap = e.save();
    let bytes = encode_archive(&snap);
    let dec = decode_archive(&bytes).expect("decode archive");
    let restored = Engine::restore_with_extensions(&dec, packs).expect("restore");
    assert_eq!(restored.state_hash(), e.state_hash());
    restored
}

fn snapshot_roundtrip_ai(e: &Engine, packs: &[&ExtensionPack], sel: Vec<ChoiceOption>) {
    if let Some(c) = e.open_choice() {
        let planned = e.plan_action(c.observer, &BoundedPlanner::new(4));
        assert!(
            matches!(planned.action, GameAction::Answer { choice_id, .. } if choice_id == c.id),
            "AI must answer open {} cleanly, got {:?}",
            c.spec.prompt,
            planned.action
        );
        let mut ai_probe = roundtrip_bytes(e, packs);
        assert!(!is_rejected(&submit(&mut ai_probe, c.observer, planned.action)));
    }
    let mut restored = roundtrip_bytes(e, packs);
    if let Some(c) = restored.open_choice().cloned() {
        assert!(!is_rejected(&submit(
            &mut restored,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        )));
    }
}

fn put_deck_top(e: &mut Engine, cards_top_first: &[u32]) {
    for &c in cards_top_first {
        e.state_core_mut().deck.retain(|&x| x != c);
        e.state_core_mut().discard.retain(|&x| x != c);
    }
    for &c in cards_top_first.iter().rev() {
        e.state_core_mut().deck.push(c);
    }
}

// =============================================================================
// 1. `cards:ssr_jinfan` + skill `ssr_jinfan` (`祖安武将/extension.js:6535-6613`)
// =============================================================================

#[test]
fn l25_fix_ssr_jinfan_no_fake_card_per_target_distance_armor_and_onremove() {
    let p = p9();
    let mut e = fresh_identity(&p, 25901, &[&["ssr_jinfan", "咆哮"], &[], &[], &[]]);
    assert!(
        !e.catalog().cards.iter().any(|c| c.name == "ssr_jinfan"),
        "上游无 card.card.ssr_jinfan (`extension.js:6535` 为武将技), 原生不得虚构同名实体牌"
    );

    e.debug_set_turn(0, Phase::Play, 1);
    let blk_sha = give_card(&mut e, 0, |c| c.kind == CardKind::Sha && c.suit == Suit::Spade);
    let sha1 = give_card(&mut e, 0, |c| c.kind == CardKind::Sha && c.suit == Suit::Spade);
    let sha2 = give_card(&mut e, 0, |c| c.kind == CardKind::Sha && c.suit == Suit::Spade);
    let renwang = give_kind(&mut e, 2, CardKind::Renwang);
    // Equip Renwang on distance-2 Seat 2
    e.state_core_mut().player_mut(2).hand.retain(|&c| c != renwang);
    e.state_core_mut().player_mut(2).equips.push(renwang);

    // Seat 1 has 2 cards (`>= 2` ling); Seat 2 has 1 card (`< 2` ling); Seat 3 has 0 cards
    let s1a = give_kind(&mut e, 1, CardKind::Shan);
    let s1b = give_kind(&mut e, 1, CardKind::Tao);
    let s2a = give_kind(&mut e, 2, CardKind::Tao);
    let _ = (s1a, s1b, s2a);

    assert!(!is_rejected(&activate(&mut e, 0, "ssr_jinfan")));
    snapshot_roundtrip_ai(&e, &[&p], vec![ChoiceOption::Card(blk_sha)]);
    answer(&mut e, "ssr_jinfan:discard_card", vec![ChoiceOption::Card(blk_sha)]);
    drain(&mut e);
    assert_eq!(e.state_core().vm_aux.piles.get(&(0, "ssr_jinfan".into())).map(Vec::len), Some(2));

    // Distance 2 Seat 2 (hand len 1 < 2): infinite distance + ignores Renwang (black Sha still hits!)
    assert!(
        !is_rejected(&play(&mut e, 0, sha1, vec![2])),
        "ling=2 > Seat 2 手牌数 1，应无视距离与防具（仁王盾）命中"
    );
    drain(&mut e);
    assert_eq!(e.state_core().player(2).hp, 3);

    // Give Seat 2 a 2nd card (`hand.len == 2`, not `< 2`), even while Seat 3 still has 0 cards!
    let _s2b = give_kind(&mut e, 2, CardKind::Tao);
    assert!(
        is_rejected(&submit(&mut e, 0, GameAction::PlayCard { card: sha2, targets: vec![2] })),
        "Seat 2 手牌数=2 不小于 ling=2，即使 Seat 3 手牌=0 也不得对 Seat 2 无视距离"
    );

    // Verify Seat 2's armor is ALSO NOT bypassed when Seat 2 holds >= ling cards (`eval_ignore_armor` false for Seat 2)
    assert!(!engine_native::rvm_bridge::modifiers::eval_ignore_armor(
        e.state_core(),
        e.catalog(),
        0,
        2,
        Some(sha2),
        engine_native::event::DamageNature::Normal,
    ));
    // And after ssr_jinfan killed Seat 2 armor wasn't permanently disabled against other attackers:
    assert!(!engine_native::rvm_bridge::modifiers::eval_ignore_armor(
        e.state_core(),
        e.catalog(),
        1,
        2,
        Some(sha2),
        engine_native::event::DamageNature::Normal,
    ));

    // `onremove` (`extension.js:6568-6571`): removing `ssr_jinfan` moves expansion pile to discard
    let pile_cards = e.state_core().vm_aux.piles.get(&(0, "ssr_jinfan".into())).cloned().unwrap();
    e.debug_remove_skill(0, "ssr_jinfan");
    assert!(e.state_core().vm_aux.piles.get(&(0, "ssr_jinfan".into())).is_none());
    for cid in pile_cards {
        assert!(e.state_core().discard.contains(&cid), "失去 ssr_jinfan 时铃牌须移入弃牌堆");
    }
}

// =============================================================================
// 2. `configs:ssr_luoyi` (`祖安武将/extension.js:15031-15108`)
// =============================================================================

#[test]
fn l25_fix_ssr_luoyi_disable_armor_trick_to_duel_and_double_shan_sha() {
    let p = p9();
    let mut e = fresh_identity(&p, 25902, &[&["ssr_luoyi"], &[], &[], &[]]);
    e.debug_set_turn(0, Phase::Play, 1);

    let armor0 = give_kind(&mut e, 0, CardKind::Armor);
    let wuzhong0 = give_kind(&mut e, 0, CardKind::Wuzhong);
    let le0 = give_kind(&mut e, 0, CardKind::Le);
    let nul0 = give_kind(&mut e, 0, CardKind::Nullify);
    let sha0 = give_kind(&mut e, 0, CardKind::Sha);

    // 1) `disableEquip('equip2')`: Armor equip rejected & absent from legal_actions
    assert!(is_rejected(&submit(&mut e, 0, GameAction::PlayCard { card: armor0, targets: vec![] })));
    assert!(!e.legal_actions(0).iter().any(|a| matches!(a, GameAction::PlayCard { card, .. } if *card == armor0)));

    // 2) `mod.cardname`: tricks & delayed tricks act as Duel (`juedou`), NOT as Wuzhong / Le / Nullify!
    // Play delayed trick `Le` as `Duel` against Seat 1; Seat 1 needs 2 Sha per Duel step!
    let s1a = give_kind(&mut e, 1, CardKind::Sha);
    let s1b = give_kind(&mut e, 1, CardKind::Sha);
    let sh0 = give_kind(&mut e, 0, CardKind::Sha);
    play(&mut e, 0, le0, vec![1]);
    answer(&mut e, "duel_sha", vec![ChoiceOption::Card(s1a)]);
    answer(&mut e, "duel_sha", vec![ChoiceOption::Card(s1b)]); // 2nd Sha required!
    answer(&mut e, "duel_sha", vec![ChoiceOption::Card(sh0)]); // Seat 0 (target=0) only needs 1 Sha
    drain(&mut e); // Seat 1 has no more Sha -> takes ONLY 1 damage (not +1 damage!)
    assert_eq!(e.state_core().player(1).hp, 3, "ssr_luoyi 决斗仅造成 1 点伤害且令对手每次出 2 张杀");

    // Wuzhong ALSO acts as Duel, AND Nullify cannot nullify enemy's trick while `ssr_luoyi` turns all tricks into Duel!
    assert!(is_rejected(&submit(&mut e, 0, GameAction::PlayCard { card: wuzhong0, targets: vec![] })));
    assert!(is_rejected(&submit(&mut e, 0, GameAction::PlayCard { card: nul0, targets: vec![] })));

    // 3) `subSkill.sha`: `shanRequired = 2`, base damage = 1 (NOT 2!)
    let shan1a = give_kind(&mut e, 1, CardKind::Shan);
    let shan1b = give_kind(&mut e, 1, CardKind::Shan);
    play(&mut e, 0, sha0, vec![1]);
    answer(&mut e, "respond_shan", vec![ChoiceOption::Card(shan1a)]);
    // Between Shan #1 and Shan #2, verify pending save -> restore -> Answer
    snapshot_roundtrip_ai(&e, &[&p], vec![ChoiceOption::Card(shan1b)]);
    answer(&mut e, "respond_shan", vec![ChoiceOption::Card(shan1b)]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 3, "两张闪才可完全闪避 ssr_luoyi 的杀");

    // Seat 1's turn: Seat 1 uses `Duel` targeting Seat 0 (`useCardToTargeted`) -> Seat 1 must ALSO play 2 Sha!
    advance_to(&mut e, 1, Phase::Play);
    let duel1 = give_kind(&mut e, 1, CardKind::Duel);
    let s0_back = give_kind(&mut e, 0, CardKind::Sha);
    let s1_only_one = give_kind(&mut e, 1, CardKind::Sha);
    // Seat 0 holds `nul0`, which MUST NOT trigger `nullify_window` because `ssr_luoyi` turns `Nullify` into `Duel`!
    play(&mut e, 1, duel1, vec![0]);
    answer(&mut e, "duel_sha", vec![ChoiceOption::Card(s0_back)]); // Seat 0 answers 1 Sha -> now Seat 1 needs 2!
    answer(&mut e, "duel_sha", vec![ChoiceOption::Card(s1_only_one)]);
    // Seat 1 has no 2nd Sha -> loses immediately and takes 1 damage!
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 2, "他家对 ssr_luoyi 决斗同样须每次打 2 张杀，且 ssr_luoyi 无法出无懈");

    // 4) Removing `ssr_luoyi` re-enables equipping Armor (`equip2`)
    e.debug_remove_skill(0, "ssr_luoyi");
    advance_to(&mut e, 0, Phase::Play);
    assert!(!is_rejected(&play(&mut e, 0, armor0, vec![])));
}

// =============================================================================
// 3. `configs:ssr_lieren` (`祖安武将/extension.js:22337-22560`)
// =============================================================================

#[test]
fn l25_fix_ssr_lieren_multi_use_pyzhuren_diamond_and_temp_exclusion() {
    let p = p9();
    let mut e = fresh_identity(&p, 25903, &[&["ssr_lieren"], &[], &[], &[]]);
    e.debug_set_turn(3, Phase::End, 1);
    advance_to(&mut e, 0, Phase::Play);

    // 1) Start/equip creates `pyzhuren_diamond` weapon into Seat 0 hand
    let py = e
        .debug_hand(0)
        .iter()
        .copied()
        .find(|&cid| e.catalog().cards[cid as usize].name == "pyzhuren_diamond")
        .expect("开局须获得衍生武器 pyzhuren_diamond");
    let extra_wp = give_kind(&mut e, 0, CardKind::Zhuge);
    let non_wp = give_kind(&mut e, 0, CardKind::Sha);

    // 2) Can activate `ssr_lieren` MULTIPLE times in the same Play phase! Non-weapon excluded.
    assert!(!is_rejected(&activate(&mut e, 0, "ssr_lieren")));
    let ask_c1 = open(&e, "ssr_lieren:choose_card");
    assert!(!ask_c1.spec.options.contains(&ChoiceOption::Card(non_wp)), "非武器牌不得作为烈刃置入牌");
    assert!(ask_c1.spec.options.contains(&ChoiceOption::Card(py)));
    snapshot_roundtrip_ai(&e, &[&p], vec![ChoiceOption::Card(py)]);
    answer(&mut e, "ssr_lieren:choose_card", vec![ChoiceOption::Card(py)]);
    answer(&mut e, "ssr_lieren:choose_target", vec![ChoiceOption::Player(1)]);
    drain(&mut e);

    assert!(!is_rejected(&activate(&mut e, 0, "ssr_lieren")), "烈刃无 usable:1 限制，同一出牌阶段可再发动");
    let ask_t2_card = open(&e, "ssr_lieren:choose_card");
    assert!(ask_t2_card.spec.options.contains(&ChoiceOption::Card(extra_wp)));
    answer(&mut e, "ssr_lieren:choose_card", vec![ChoiceOption::Card(extra_wp)]);
    let ask_t2_tgt = open(&e, "ssr_lieren:choose_target");
    assert!(!ask_t2_tgt.spec.options.contains(&ChoiceOption::Player(1)), "已有 ssr_lieren_en 标记角色不可重选");
    answer(&mut e, "ssr_lieren:choose_target", vec![ChoiceOption::Player(2)]);
    drain(&mut e);

    // 3) Next Prepare: Seat 0 reclaims `py` & `extra_wp` -> Seat 1 answers Shan (no dmg), Seat 2 declines (1 dmg),
    //    AND during THAT turn's Play phase `py`/`extra_wp` are in `ssr_lieren_temp` so CANNOT be re-placed!
    let mut e_rec = fresh_identity(&p, 25904, &[&["ssr_lieren"], &[], &[], &[]]);
    e_rec.debug_set_turn(3, Phase::End, 1);
    advance_to(&mut e_rec, 0, Phase::Play);
    let py_r = e_rec
        .debug_hand(0)
        .iter()
        .copied()
        .find(|&cid| e_rec.catalog().cards[cid as usize].name == "pyzhuren_diamond")
        .unwrap();
    let wp2_r = give_kind(&mut e_rec, 0, CardKind::Zhuge);
    let shan_r1 = give_kind(&mut e_rec, 1, CardKind::Shan);
    let _shan_r2 = give_kind(&mut e_rec, 2, CardKind::Shan);
    activate(&mut e_rec, 0, "ssr_lieren");
    answer(&mut e_rec, "ssr_lieren:choose_card", vec![ChoiceOption::Card(py_r)]);
    answer(&mut e_rec, "ssr_lieren:choose_target", vec![ChoiceOption::Player(1)]);
    activate(&mut e_rec, 0, "ssr_lieren");
    answer(&mut e_rec, "ssr_lieren:choose_card", vec![ChoiceOption::Card(wp2_r)]);
    answer(&mut e_rec, "ssr_lieren:choose_target", vec![ChoiceOption::Player(2)]);
    advance_to(&mut e_rec, 3, Phase::End);
    let _ = submit(&mut e_rec, 3, GameAction::NextPhase);
    // Seat 1 answers Shan, Seat 2 declines -> Seat 2 takes 1 damage
    answer(&mut e_rec, "ssr_lieren:target_response", vec![ChoiceOption::Card(shan_r1)]);
    answer(&mut e_rec, "ssr_lieren:target_response", vec![]);
    drain(&mut e_rec);
    assert_eq!(e_rec.state_core().player(1).hp, 4);
    assert_eq!(e_rec.state_core().player(2).hp, 3);
    assert!(e_rec.debug_hand(0).contains(&py_r) && e_rec.debug_hand(0).contains(&wp2_r));
    assert!(
        is_rejected(&activate(&mut e_rec, 0, "ssr_lieren")),
        "ssr_lieren_temp 内的刚收回武器本回合内不得再用于发动烈刃"
    );
    // Next turn clears `ssr_lieren_temp` -> usable again!
    advance_to(&mut e_rec, 1, Phase::Play);
    advance_to(&mut e_rec, 0, Phase::Play);
    assert!(!is_rejected(&activate(&mut e_rec, 0, "ssr_lieren")));
    let _ = (extra_wp, py);
}

// =============================================================================
// 4. `ui_hooks`: `_anshaskill`, `_seadreammodeskill`, `_relifeskill`, `_guanmodeskill`
// =============================================================================

#[test]
fn l25_fix_ui_hooks_ansha_seadream_relife_and_guanmode() {
    // 4a. `_anshaskill` (`:42659`) via formal `zuan-upstream-liuli` package
    let mut ea = formal_zuan_engine(25905, 4);
    ea.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor],
    };
    zuan_apply_engine_config(
        &mut ea,
        &ZuanModeConfig { mode: 8, balance: true, goodluck: false, ..Default::default() },
    );
    ea.debug_grant(0, "_dietime");
    ea.debug_grant(0, "_anshaskill");
    ea.debug_grant(1, "ansha_zhong");
    ea.debug_grant(2, "ansha_zhong");
    ea.debug_grant(3, "ansha_nei");
    set_flag(&mut ea, 0, "mark:_dietime", 2);
    // Lord Phase::End #1: mark:_dietime 2 -> 1 (nobody dies yet)
    ea.debug_set_turn(0, Phase::Discard, 1);
    let _ = submit(&mut ea, 0, GameAction::NextPhase);
    drain(&mut ea);
    assert_eq!(flag(&ea, 0, "mark:_dietime"), 1);
    assert!(ea.state_core().alive_players().count() == 4);

    // Lord Phase::End #2: mark:_dietime == 1 -> kills 1 `ansha_zhong`!
    ea.debug_set_turn(0, Phase::Discard, 2);
    let _ = submit(&mut ea, 0, GameAction::NextPhase);
    drain(&mut ea);
    assert_eq!(ea.state_core().alive_players().count(), 3);

    // Lord Phase::End #3 (`ansha_zhong` count == 1 <= 1, `zhudie == 0`):
    // kills last `ansha_zhong`, reveals `ansha_nei` as `Rebel`, sets `_dietime = 3`, `zhudie = 1`!
    ea.debug_set_turn(0, Phase::Discard, 3);
    let _ = submit(&mut ea, 0, GameAction::NextPhase);
    drain(&mut ea);
    assert_eq!(flag(&ea, 0, "l25_zhudie"), 1);
    assert_eq!(flag(&ea, 0, "mark:_dietime"), 3);
    if let GameMode::Identity { roles } = &ea.state_core().mode {
        assert_eq!(roles[3], Role::Rebel);
    }

    // Countdown `zhudie == 1` to expiry -> Lord dies!
    set_flag(&mut ea, 0, "mark:_dietime", 1);
    ea.debug_set_turn(0, Phase::Discard, 6);
    let _ = submit(&mut ea, 0, GameAction::NextPhase);
    drain(&mut ea);
    assert!(!ea.state_core().player(0).alive, "zhudie!=0 且夜幕再度降临时主公死亡/败北");

    // 4b. `_seadreammodeskill` (`:42815`) + `huanhaicls` (`:1905`)
    // Negative case: `!balance` + non-Diamond judge -> NO `huanhaicls`, keeps `咆哮`!
    let mut es_miss = formal_zuan_engine(25905, 4);
    es_miss.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor],
    };
    zuan_apply_engine_config(
        &mut es_miss,
        &ZuanModeConfig { mode: 7, balance: false, goodluck: false, ..Default::default() },
    );
    es_miss.debug_grant(0, "_seadreammodeskill");
    es_miss.debug_grant(1, "咆哮");
    let non_diamond = take_card(&mut es_miss, |c| c.suit != Suit::Diamond);
    put_deck_top(&mut es_miss, &[non_diamond]);
    es_miss.debug_set_turn(3, Phase::End, 1);
    advance_to(&mut es_miss, 0, Phase::Play);
    assert!(!has_skill(&es_miss, 1, "huanhaicls"), "非方片判定不得授予 huanhaicls");
    assert!(has_skill(&es_miss, 1, "咆哮"), "未授予 huanhaicls 时 咆哮 保持启用");

    // Positive case 1: `!balance` + Diamond judge -> swaps seat, grants `huanhaicls` sealing `咆哮`, clears on next TurnBegin
    let mut es = formal_zuan_engine(25906, 4);
    es.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor],
    };
    zuan_apply_engine_config(
        &mut es,
        &ZuanModeConfig { mode: 7, balance: false, goodluck: false, ..Default::default() },
    );
    es.debug_grant(0, "_seadreammodeskill");
    es.debug_grant(1, "咆哮");
    let diamond = take_card(&mut es, |c| c.suit == Suit::Diamond);
    put_deck_top(&mut es, &[diamond]);
    es.debug_set_turn(3, Phase::End, 1);
    advance_to(&mut es, 0, Phase::Play);
    assert!(has_skill(&es, 1, "huanhaicls"), "方片判定须赋予 huanhaicls 暂时失效非 charlotte 技");
    assert!(!has_skill(&es, 1, "咆哮"), "huanhaicls 须失效非 charlotte 技 咆哮");
    // Returning to Seat 0 next Phase::Prepare (non-Diamond judge) clears `huanhaicls` and restores `咆哮`
    advance_to(&mut es, 1, Phase::Play);
    let club_next = take_card(&mut es, |c| c.suit == Suit::Club);
    put_deck_top(&mut es, &[club_next]);
    advance_to(&mut es, 0, Phase::Play);
    assert!(!has_skill(&es, 1, "huanhaicls"));
    assert!(has_skill(&es, 1, "咆哮"));

    // Positive case 2: `balance: true` -> swap happens unconditionally; draws 1 before dice; discards ALL rank==roll hej cards; grants `huanhaicls` (incl. Lord!) ONLY on roll == 6!
    let mut es_bal = formal_zuan_engine(25906, 4);
    es_bal.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor],
    };
    zuan_apply_engine_config(
        &mut es_bal,
        &ZuanModeConfig { mode: 7, balance: true, goodluck: false, ..Default::default() },
    );
    es_bal.debug_grant(0, "_seadreammodeskill");
    es_bal.debug_grant(1, "咆哮");
    // Also give Seat 1 one judge card per rank 1..=6 to confirm `'hej'` discard includes Judge zone for ALL rolls 1..=6!
    let rank_cards: Vec<(u8, u32)> = (1u8..=6)
        .map(|r| {
            let cid = take_card(&mut es_bal, |c| c.rank == r);
            es_bal.state_core_mut().player_mut(1).hand.push(cid);
            (r, cid)
        })
        .collect();
    let judge_rank_cards: Vec<(u8, u32)> = (1u8..=6)
        .map(|r| {
            let cid = take_card(&mut es_bal, |c| c.rank == r);
            es_bal.state_core_mut().player_mut(1).judge.push(cid);
            (r, cid)
        })
        .collect();
    es_bal.debug_set_turn(3, Phase::End, 1);
    advance_to(&mut es_bal, 0, Phase::Play);
    let rolled = flag(&es_bal, 0, "l25_seadream_last_dice");
    assert!((1..=6).contains(&rolled), "balance 模式须通过真实 RNG 投掷 1..=6 点骰子");
    assert!(flag(&es_bal, 0, "l25_seadream_swap_runs") >= 1, "无论骰点/判定结果，step 0 须先交换座位");
    let (_, matched) = rank_cards.iter().find(|&&(r, _)| r as u32 == rolled).unwrap();
    let (_, j_matched) = judge_rank_cards.iter().find(|&&(r, _)| r as u32 == rolled).unwrap();
    assert!(!es_bal.state_core().player(1).hand.contains(matched), "任何骰点 1..=6 均须弃置匹配点数手牌");
    assert!(!es_bal.state_core().player(1).judge.contains(j_matched), "hej 弃置须包含判定区牌");
    if rolled == 6 {
        assert!(has_skill(&es_bal, 1, "huanhaicls"));
        assert!(has_skill(&es_bal, 0, "huanhaicls"), "huanhaicls 须同时授予主公在内全体存活角色");
    } else {
        assert!(!has_skill(&es_bal, 1, "huanhaicls"), "balance 骰点 != 6 不得授予 huanhaicls");
        assert!(!has_skill(&es_bal, 0, "huanhaicls"));
    }

    // 4c. `_relifeskill` (`:43402`) on `roundStart` for Identity Lord (mode 2 AND mode 4!)
    let mut er = formal_zuan_engine(25907, 4);
    er.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor],
    };
    zuan_apply_engine_config(
        &mut er,
        &ZuanModeConfig { mode: 2, balance: false, goodluck: false, ..Default::default() },
    );
    er.debug_grant(0, "_relifeskill");
    clear_zone(&mut er, 2);
    er.state_core_mut().player_mut(2).alive = false;
    er.state_core_mut().player_mut(2).hp = 0;
    er.state_core_mut().player_mut(2).max_hp = 5;
    set_flag(&mut er, 2, "l25_relife_timer", 2);
    set_flag(&mut er, 2, "l25_level", 3); // draw ceil(3*0.5)=2, next timer ceil(3*0.4+0.1)=2
    er.debug_set_turn(3, Phase::End, 1);
    advance_to(&mut er, 0, Phase::Play); // Round 2 TurnBegin: timer 2 -> 1
    assert!(!er.state_core().player(2).alive);
    assert_eq!(flag(&er, 2, "l25_relife_timer"), 1);
    er.debug_set_turn(3, Phase::End, 2);
    advance_to(&mut er, 0, Phase::Play); // Round 3 TurnBegin: timer 1 -> 0 -> revives at 5/5 & draws 2!
    assert!(er.state_core().player(2).alive);
    assert_eq!(er.state_core().player(2).hp, 5);
    assert_eq!(er.debug_hand(2).len(), 2);
    assert_eq!(flag(&er, 2, "l25_relife_timer"), 2);
    assert_eq!(flag(&er, 2, "l25_zuan_hudun"), 0, "无 zuan_hudun 技能者复活不得凭空补护盾标记");

    // 4d. `_guanmodeskill` (`:42978`) + `guanfu` (`:1215`) + `guifu_nei` (`:1633`) + mode 4 `_relifeskill`
    let mut eg = formal_zuan_engine(25908, 4);
    eg.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor],
    };
    zuan_apply_engine_config(
        &mut eg,
        &ZuanModeConfig { mode: 4, balance: false, goodluck: false, ..Default::default() },
    );
    // Non-Lord Seat 1 holding `_guanmodeskill` MUST NOT trigger it!
    eg.debug_grant(1, "_guanmodeskill");
    eg.debug_set_turn(0, Phase::Discard, 1);
    advance_to(&mut eg, 1, Phase::Play);
    assert_eq!(flag(&eg, 1, "l25_guanmode_init"), 0, "非主公不得触发 _guanmodeskill");
    assert!(!has_skill(&eg, 0, "guanfu"));
    eg.debug_grant(0, "_guanmodeskill");
    eg.debug_set_turn(3, Phase::End, 1);
    advance_to(&mut eg, 0, Phase::Play);
    assert!(has_skill(&eg, 0, "guanfu"), "L25 _guanmodeskill 须授主公 guanfu");
    assert!(has_skill(&eg, 3, "guifu_nei"), "L25 _guanmodeskill 须授内奸 guifu_nei");
    // When `guanfu` active and `!balance`, ANY alive recovering player at hp > 0 (global!) cancels recover & draws 1 instead!
    eg.debug_set_hp(1, 3);
    let tao1 = give_kind(&mut eg, 1, CardKind::Tao);
    let h1 = eg.debug_hand(1).len();
    eg.debug_set_turn(1, Phase::Play, 2);
    play(&mut eg, 1, tao1, vec![]);
    drain(&mut eg);
    assert_eq!(eg.state_core().player(1).hp, 3, "guanfu_disrecover 全局对非主公同样生效：取消回血改摸 1");
    assert_eq!(eg.debug_hand(1).len(), h1, "耗桃 1 张再摸 1 张");
}

// =============================================================================
// 5. `ui_hooks:_haoyunlai` (`祖安武将/extension.js:47584-47660`)
// =============================================================================

#[test]
fn l25_fix_ui_hook_haoyunlai_draw_begin_and_config_gating() {
    let mut e_on = formal_zuan_engine(25909, 4);
    let mut e_off = formal_zuan_engine(25909, 4);
    for (eng, gl) in [(&mut e_on, true), (&mut e_off, false)] {
        for s in 0..4u8 { clear_zone(eng, s); }
        eng.debug_grant(0, "_haoyunlai");
        eng.debug_grant(1, "_haoyunlai");
        zuan_apply_engine_config(
            eng,
            &ZuanModeConfig { mode: 1, balance: false, goodluck: gl, ..Default::default() },
        );
    }

    // Seed exact identical deck in both engines: top is low-value Sha, deeper (within top 20) is high-value Tao!
    let setup_deck = |eng: &mut Engine| {
        let top_sha = take_card(eng, |c| c.kind == CardKind::Sha);
        let mid_shan1 = take_card(eng, |c| c.kind == CardKind::Shan);
        let mid_shan2 = take_card(eng, |c| c.kind == CardKind::Shan);
        let deep_tao = take_card(eng, |c| c.kind == CardKind::Tao);
        put_deck_top(eng, &[top_sha, mid_shan1, mid_shan2, deep_tao]);
        (top_sha, deep_tao)
    };
    let _ = setup_deck(&mut e_on);
    let _ = setup_deck(&mut e_off);
    e_on.debug_set_hp(0, 2);
    e_off.debug_set_hp(0, 2);

    // 1) Seat 1 (AI, != game.me) draws: even with `goodluck: true`, AI must NOT bias deck or consume RNG!
    e_on.debug_set_turn(1, Phase::Judge, 1);
    e_off.debug_set_turn(1, Phase::Judge, 1);
    advance_to(&mut e_on, 1, Phase::Play);
    advance_to(&mut e_off, 1, Phase::Play);
    assert_eq!(e_on.debug_hand(1), e_off.debug_hand(1));
    assert_eq!(e_on.state_core().rng, e_off.state_core().rng, "非 game.me 不得触发好运来且不得消耗 RNG");

    // 2) Seat 0 (`game.me`) draws: `goodluck: false` draws `top_off`, whereas `goodluck: true` promotes `tao_on`!
    // Re-seed top cleanly after Seat 1's draw so Seat 0 directly observes `top` vs promoted `tao`.
    let (top_on, tao_on) = setup_deck(&mut e_on);
    let (top_off, tao_off) = setup_deck(&mut e_off);
    e_on.debug_set_turn(0, Phase::Judge, 1);
    e_off.debug_set_turn(0, Phase::Judge, 1);
    advance_to(&mut e_on, 0, Phase::Play);
    advance_to(&mut e_off, 0, Phase::Play);
    assert!(e_off.debug_hand(0).contains(&top_off) && !e_off.debug_hand(0).contains(&tao_off));
    assert_eq!(
        e_on.debug_hand(0).first().copied(),
        Some(tao_on),
        "开启 goodluck 后 _haoyunlai 须为 game.me 从牌堆前 20 张挑高价值牌(桃)置顶首张摸取 (top={top_on})"
    );
}

// =============================================================================
// 6. `skills`: `wumou` (`:48975`), `new_guixin` (`:49077`), `gongxin` (`:49506`)
// =============================================================================

#[test]
fn l25_fix_wumou_new_guixin_and_l25_gongxin() {
    let p = p9();

    // 6a. `wumou` (`:48975-49031`): once at `phaseJieshuBegin` ONLY when normal trick used in `phaseUse`!
    let mut ew = fresh_identity(&p, 25910, &[&["wumou"], &[], &[], &[]]);
    set_flag(&mut ew, 0, "mark:baonu", 2);
    ew.debug_set_turn(0, Phase::Play, 1);
    let wz0 = give_kind(&mut ew, 0, CardKind::Wuzhong);
    let dis0 = give_kind(&mut ew, 0, CardKind::Dismantle);
    let le0 = give_kind(&mut ew, 0, CardKind::Le);
    let dummy1 = give_kind(&mut ew, 1, CardKind::Shan);
    let _ = dummy1;
    // Play delayed trick (`Le`) first, then 2 normal tricks (`Wuzhong` + `Dismantle`)
    play(&mut ew, 0, le0, vec![2]); drain(&mut ew);
    play(&mut ew, 0, wz0, vec![]); drain(&mut ew);
    play(&mut ew, 0, dis0, vec![1]); drain(&mut ew);
    assert_eq!(flag(&ew, 0, "mark:baonu"), 2, "使用锦囊牌当场不得立刻结算 wumou");

    // Discard -> End (`phaseJieshuBegin`): triggers ONCE! Choice 0 spends 1 mark; second trick didn't double-charge!
    advance_to_end_stop_on_choice(&mut ew, 0);
    let wm_ask = open(&ew, "wumou:choose_mode");
    assert!(!wm_ask.spec.allow_cancel);
    snapshot_roundtrip_ai(&ew, &[&p], vec![ChoiceOption::Tag(0)]);
    answer(&mut ew, "wumou:choose_mode", vec![ChoiceOption::Tag(0)]);
    drain(&mut ew);
    assert_eq!((flag(&ew, 0, "mark:baonu"), ew.state_core().player(0).hp), (1, 4), "两张普通锦囊只触发一次 wumou");

    // Delayed trick ONLY or skipped `phaseUse`: NEVER triggers `wumou`!
    ew.debug_set_turn(0, Phase::Play, 2);
    let le2 = give_kind(&mut ew, 0, CardKind::Le);
    play(&mut ew, 0, le2, vec![1]); drain(&mut ew);
    advance_to_end_stop_on_choice(&mut ew, 0);
    assert!(ew.open_choice().is_none(), "仅使用延时锦囊不得触发 wumou");
    assert_eq!(flag(&ew, 0, "mark:baonu"), 1);

    // 6b. `new_guixin` (`:49077-49154`): confirm + zone priority + random pick + ONLY `gains > 2` turns over + multi-damage loop!
    let mut eg2 = fresh_identity(&p, 25912, &[&["new_guixin"], &[], &[], &[]]);
    let _h1 = give_kind(&mut eg2, 1, CardKind::Shan);
    let eq1 = give_kind(&mut eg2, 1, CardKind::Armor);
    eg2.state_core_mut().player_mut(1).hand.retain(|&c| c != eq1);
    eg2.state_core_mut().player_mut(1).equips.push(eq1);
    let j2 = give_kind(&mut eg2, 2, CardKind::Le);
    eg2.state_core_mut().player_mut(2).hand.retain(|&c| c != j2);
    eg2.state_core_mut().player_mut(2).judge.push(j2);
    eg2.debug_set_turn(1, Phase::Play, 1);
    let sha_a = give_kind(&mut eg2, 1, CardKind::Sha);
    play(&mut eg2, 1, sha_a, vec![0]);
    answer(&mut eg2, "new_guixin:confirm", vec![ChoiceOption::Tag(1)]);
    snapshot_roundtrip_ai(&eg2, &[&p], vec![ChoiceOption::Tag(1)]);
    answer(&mut eg2, "new_guixin:choose_zone", vec![ChoiceOption::Tag(1)]); // Equip priority `['e', 'h', 'j']`
    drain(&mut eg2);
    assert!(eg2.debug_hand(0).contains(&eq1), "优先选装备区须先拿 Seat 1 装备");
    assert!(eg2.debug_hand(0).contains(&j2), "Seat 2 无装备无手牌顺延判定区拿取");
    assert!(!eg2.state_core().player(0).face_down, "仅获得 2 张牌 (`<= 2`) 不得翻面");

    // Now 3 alive other players all hold cards -> gains == 3 (> 2) -> DOES flip face_down!
    let _c1 = give_kind(&mut eg2, 1, CardKind::Sha);
    let _c2 = give_kind(&mut eg2, 2, CardKind::Shan);
    let _c3 = give_kind(&mut eg2, 3, CardKind::Tao);
    advance_to(&mut eg2, 3, Phase::Play);
    let sha_b = give_kind(&mut eg2, 3, CardKind::Sha);
    play(&mut eg2, 3, sha_b, vec![0]);
    answer(&mut eg2, "new_guixin:confirm", vec![ChoiceOption::Tag(1)]);
    answer(&mut eg2, "new_guixin:choose_zone", vec![ChoiceOption::Tag(0)]);
    drain(&mut eg2);
    assert!(eg2.state_core().player(0).face_down, "获得 3 张牌 (`> 2`) 须翻面");

    // 6c. `gongxin` (L25 `:49506-49558` + `ssr_gongxin_break :13429-13460`) via formal `zuan-upstream-liuli`
    let mut ez = formal_zuan_engine(25913, 4);
    for s in 0..4u8 { clear_zone(&mut ez, s); }
    ez.debug_grant(0, "gongxin");
    ez.debug_set_turn(0, Phase::Play, 1);
    let h1_heart = give_card(&mut ez, 1, |c| c.suit == Suit::Heart && c.kind == CardKind::Shan);
    let h2_heart = give_card(&mut ez, 2, |c| c.suit == Suit::Heart && c.kind == CardKind::Sha);
    let h2_diamond_tao = give_card(&mut ez, 2, |c| c.suit == Suit::Diamond && c.kind == CardKind::Tao);
    let h2_diamond_shan = give_card(&mut ez, 2, |c| c.suit == Suit::Diamond && c.kind == CardKind::Shan);
    let h1_extra = give_card(&mut ez, 1, |c| c.suit == Suit::Club && c.kind == CardKind::Sha);
    let _ = h1_extra;

    // Use #1 on Seat 1 -> choose `0` (获得此牌)
    assert!(!is_rejected(&activate(&mut ez, 0, "gongxin")));
    answer(&mut ez, "gongxin_target", vec![ChoiceOption::Player(1)]);
    answer(&mut ez, "gongxin_card", vec![ChoiceOption::Card(h1_heart)]);
    answer(&mut ez, "gongxin_choice", vec![ChoiceOption::Tag(0)]);
    assert!(ez.debug_hand(0).contains(&h1_heart), "L25 攻心选择 0 须获得该红桃牌到手牌");

    // Use #2 (`usable: 2`!): cannot re-target Seat 1 (`gongxin_temp`), MUST allow Seat 2 -> choose `1` (`ssr_gongxin_break`!)
    assert!(!is_rejected(&activate(&mut ez, 0, "gongxin")), "L25 攻心为出牌阶段限 2 次");
    let gx_t2 = open(&ez, "gongxin_target");
    assert!(!gx_t2.spec.options.contains(&ChoiceOption::Player(1)), "同一出牌阶段内已目标角色(gongxin_temp)不可重复指定");
    answer(&mut ez, "gongxin_target", vec![ChoiceOption::Player(2)]);
    answer(&mut ez, "gongxin_card", vec![ChoiceOption::Card(h2_heart)]);
    answer(&mut ez, "gongxin_choice", vec![ChoiceOption::Tag(1)]);
    assert!(is_rejected(&activate(&mut ez, 0, "gongxin")), "L25 攻心每阶段限 2 次");

    // Seat 2 has `ssr_gongxin_break`: CANNOT use/respond/save with ANY Red card (Heart OR Diamond)!
    ez.debug_set_turn(2, Phase::Play, 1);
    assert!(
        is_rejected(&submit(&mut ez, 2, GameAction::PlayCard { card: h2_heart, targets: vec![1] })),
        "封印红牌后不可使用红桃杀"
    );
    ez.debug_set_hp(2, 2);
    assert!(
        is_rejected(&submit(&mut ez, 2, GameAction::PlayCard { card: h2_diamond_tao, targets: vec![] })),
        "封印红牌须同时封禁方块(所有红色牌), 不得仅限红桃"
    );
    let sha1 = give_card(&mut ez, 1, |c| c.kind == CardKind::Sha && c.suit == Suit::Spade);
    ez.debug_set_turn(1, Phase::Play, 1);
    play(&mut ez, 1, sha1, vec![2]);
    drain(&mut ez); // `h2_diamond_shan` CANNOT respond -> takes damage cleanly!
    assert_eq!(ez.state_core().player(2).hp, 1);
    assert!(ez.debug_hand(2).contains(&h2_diamond_shan));

    // When Seat 0's next TurnBegin arrives, `ssr_gongxin_break` expires!
    ez.debug_set_turn(3, Phase::End, 1);
    advance_to(&mut ez, 0, Phase::Play);
    ez.debug_set_turn(2, Phase::Play, 2);
    assert!(!is_rejected(&submit(&mut ez, 2, GameAction::PlayCard { card: h2_diamond_tao, targets: vec![] })));
}

// =============================================================================
// 7. `skills`: 7 个祖安展示/语音/身份钩子与 Block-Comment 死亡/判定清理钩子
// =============================================================================

#[test]
fn l25_fix_seven_zuan_fx_speech_and_identity_hooks() {
    let p = p9();
    let mut e = fresh_identity(
        &p,
        25914,
        &[
            &["_newZuanPhaseTX", "_zuanCHANGEHPsayskill", "_fromsay", "_bingletexiao1", "_bingletexiao2"],
            &["_zuan_damageSendEgg", "_zuanidentityshowskill"],
            &["_zuan_damageSendEgg"],
            &[],
        ],
    );
    zuan_apply_engine_config(
        &mut e,
        &ZuanModeConfig {
            mode: 1,
            balance: false,
            goodluck: false,
            zuantype_new: true,
            zuantype_old: true,
            zuan_emotion: true,
            ransay: 100,
            newhuihe_new: true,
        },
    );

    // 1) `_fromsay`: fires when OTHER player uses a card (`u.player != owner`)!
    let sha1 = give_kind(&mut e, 1, CardKind::Sha);
    e.debug_set_turn(1, Phase::Play, 1);
    play(&mut e, 1, sha1, vec![2]);
    drain(&mut e);
    assert!(flag(&e, 0, "l25_ui_hook:_fromsay") >= 1, "_fromsay 须在他家使用牌时触发");
    // Seat 1 (Rebel) damaging Seat 2 (Loyalist) has negative attitude -> NO `_zuan_damageSendEgg`!
    assert_eq!(flag(&e, 2, "l25_ui_hook:_zuan_damageSendEgg"), 0, "敌方伤害不得触发砸蛋");
    // `_zuanCHANGEHPsayskill` fires ONCE per Damage even if repeated
    let hp_say_after_dmg = flag(&e, 0, "l25_ui_hook:_zuanCHANGEHPsayskill");
    assert_eq!(hp_say_after_dmg, 1, "单次 Damage 仅触发一次 _zuanCHANGEHPsayskill");

    // 2) `_zuanidentityshowskill` (`:42132`): Hidentity active on AI Seat 1 transforms disguised role; revealed on Dying!
    set_flag(&mut e, 1, "l25_hidentity", 1);
    set_flag(&mut e, 1, "l25_sidentity", 3); // true = Rebel (`fan`)
    set_flag(&mut e, 1, "l25_zhu_att", (-10i32) as u32);
    advance_to(&mut e, 2, Phase::Play);
    advance_to(&mut e, 1, Phase::Play);
    assert!(flag(&e, 1, "l25_shown_role") > 0, "Hidentity 开启时回合开始应更新伪装身份");
    assert_eq!(flag(&e, 1, "l25_ui_hook:_zuanidentityshowskill"), 1, "单回合准备阶段只触发一次");
    e.debug_set_hp(1, 1);
    e.debug_grant(1, "_zuanidentitydyingskill");
    advance_to(&mut e, 0, Phase::Play);
    let sha0 = give_kind(&mut e, 0, CardKind::Sha);
    play(&mut e, 0, sha0, vec![1]);
    drain(&mut e);
    assert_eq!(flag(&e, 1, "l25_shown_role"), 3, "濒死时显露真实 Sidentity=3 (反贼)");

    // Ally damage (Seat 0 Lord -> Seat 2 Loyalist, positive attitude, owner!=0, !maixie) DOES trigger `_zuan_damageSendEgg`!
    clear_zone(&mut e, 2);
    let _sha0_duel = give_kind(&mut e, 0, CardKind::Sha);
    let duel_ally = give_kind(&mut e, 0, CardKind::Duel);
    play(&mut e, 0, duel_ally, vec![2]);
    drain(&mut e);
    assert_eq!(flag(&e, 2, "l25_ui_hook:_zuan_damageSendEgg"), 1, "友方对非卖出 AI 造成伤害须触发 _zuan_damageSendEgg");

    // 3) `_bingletexiao1` & `_bingletexiao2` (`extension.js:47983-48948` block comment): STRICTLY INERT!
    set_flag(&mut e, 0, "l25_unshow:hpunshow", 1);
    set_flag(&mut e, 0, "l25_unshow:thunderunshow", 1);
    let le0 = give_kind(&mut e, 0, CardKind::Le);
    let shandian0 = give_kind(&mut e, 0, CardKind::ShanDian);
    let safe_j1 = take_card(&mut e, |c| c.suit == Suit::Heart);
    let safe_j2 = take_card(&mut e, |c| c.suit == Suit::Heart);
    e.state_core_mut().player_mut(0).hand.retain(|&c| c != le0 && c != shandian0);
    e.state_core_mut().player_mut(0).judge.push(le0);
    e.state_core_mut().player_mut(0).judge.push(shandian0);
    e.state_core_mut().deck.push(safe_j1);
    e.state_core_mut().deck.push(safe_j2);
    let prev_phase_tx = flag(&e, 0, "l25_ui_hook:_newZuanPhaseTX");
    e.debug_set_turn(3, Phase::End, 2);
    advance_to(&mut e, 0, Phase::Play);
    assert_eq!(flag(&e, 0, "l25_ui_hook:_newZuanPhaseTX"), prev_phase_tx + 1, "_newZuanPhaseTX 每回合仅触发 1 次");
    assert_eq!(flag(&e, 0, "l25_ui_hook:_bingletexiao2"), 0, "块注释内 _bingletexiao2 不得触发");
    assert_eq!(flag(&e, 0, "l25_unshow:thunderunshow"), 1, "块注释内 _bingletexiao2 不得清除 mask");

    for s in 0..4u8 { clear_zone(&mut e, s); }
    e.debug_set_hp(0, 1);
    e.debug_set_turn(2, Phase::Play, 3);
    let sha2 = give_kind(&mut e, 2, CardKind::Sha);
    play(&mut e, 2, sha2, vec![0]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "l25_ui_hook:_bingletexiao1"), 0, "块注释内 _bingletexiao1 不得触发");
    assert_eq!(flag(&e, 0, "l25_unshow:hpunshow"), 1, "块注释内 _bingletexiao1 不得清除 mask");

    // 4) `_zuanCHANGEHPsayskill` fires EXACTLY ONCE (+1, never +2!) on CardsDrawn / Recover / LoseHp!
    let mut esay = fresh_identity(&p, 25930, &[&["_zuanCHANGEHPsayskill", "wumou"], &[], &[], &[]]);
    zuan_apply_engine_config(
        &mut esay,
        &ZuanModeConfig {
            mode: 1,
            balance: false,
            goodluck: false,
            zuantype_new: true,
            zuantype_old: false,
            zuan_emotion: false,
            ransay: 100,
            newhuihe_new: false,
        },
    );
    esay.debug_set_turn(0, Phase::Play, 1);
    let wz0 = give_kind(&mut esay, 0, CardKind::Wuzhong);
    play(&mut esay, 0, wz0, vec![]);
    drain(&mut esay);
    assert_eq!(
        flag(&esay, 0, "l25_ui_hook:_zuanCHANGEHPsayskill"),
        1,
        "单次 CardsDrawn 仅触发一次 _zuanCHANGEHPsayskill"
    );
    esay.debug_set_hp(0, 2);
    let tao0 = give_kind(&mut esay, 0, CardKind::Tao);
    play(&mut esay, 0, tao0, vec![]);
    drain(&mut esay);
    assert_eq!(
        flag(&esay, 0, "l25_ui_hook:_zuanCHANGEHPsayskill"),
        2,
        "单次 Recover 仅触发一次 _zuanCHANGEHPsayskill"
    );
    set_flag(&mut esay, 0, "mark:baonu", 1);
    let dis_wm = give_kind(&mut esay, 0, CardKind::Dismantle);
    let _dummy_c3 = give_kind(&mut esay, 3, CardKind::Sha);
    play(&mut esay, 0, dis_wm, vec![3]);
    drain(&mut esay);
    assert_eq!(flag(&esay, 0, "l25_ui_hook:_zuanCHANGEHPsayskill"), 2);
    advance_to_end_stop_on_choice(&mut esay, 0);
    answer(&mut esay, "wumou:choose_mode", vec![ChoiceOption::Tag(1)]);
    drain(&mut esay);
    assert_eq!(
        flag(&esay, 0, "l25_ui_hook:_zuanCHANGEHPsayskill"),
        3,
        "单次 LoseHp 仅触发一次 _zuanCHANGEHPsayskill"
    );
}

// =============================================================================
// 8. Batch Q2-L25 Round 2 Reject Counterexample Tests (Findings #1–#10)
// =============================================================================

#[test]
fn l25_r2_jinfan_non_sha_and_luoyi_armor_discard_and_direct_hit() {
    let p = p9();

    // Finding #1: `ssr_jinfan` MUST NOT pierce armor on non-Sha (e.g. SavageAttack) or when sealed by Tieji!
    let mut ej = fresh_identity(&p, 25920, &[&["ssr_jinfan"], &[], &[], &[]]);
    ej.debug_set_turn(0, Phase::Play, 1);
    let cost0 = give_kind(&mut ej, 0, CardKind::Sha);
    let savage0 = give_kind(&mut ej, 0, CardKind::SavageAttack);
    let sha_far = give_kind(&mut ej, 0, CardKind::Sha);
    let _s1_keep = give_kind(&mut ej, 1, CardKind::Shan);
    let _s1_keep2 = give_kind(&mut ej, 1, CardKind::Tao);
    let s2_renwang = give_kind(&mut ej, 2, CardKind::Renwang);
    ej.state_core_mut().player_mut(2).hand.retain(|&c| c != s2_renwang);
    ej.state_core_mut().player_mut(2).equips.push(s2_renwang);
    activate(&mut ej, 0, "ssr_jinfan");
    answer(&mut ej, "ssr_jinfan:discard_card", vec![ChoiceOption::Card(cost0)]);
    drain(&mut ej);
    // Seat 2 has 0 hand cards (< 1 ling): Sha ignores armor, SavageAttack and `None` MUST NOT!
    assert!(engine_native::rvm_bridge::modifiers::eval_ignore_armor(
        ej.state_core(),
        ej.catalog(),
        0,
        2,
        Some(sha_far),
        engine_native::event::DamageNature::Normal,
    ));
    assert!(!engine_native::rvm_bridge::modifiers::eval_ignore_armor(
        ej.state_core(),
        ej.catalog(),
        0,
        2,
        Some(savage0),
        engine_native::event::DamageNature::Normal,
    ));
    assert!(!engine_native::rvm_bridge::modifiers::eval_ignore_armor(
        ej.state_core(),
        ej.catalog(),
        0,
        2,
        None,
        engine_native::event::DamageNature::Normal,
    ));
    // Sealed `ssr_jinfan` (`fz_new_tieji_target_sealed`) MUST NOT pierce armor or ignore distance!
    set_flag(&mut ej, 0, "fz_new_tieji_target_sealed", 1);
    assert!(!engine_native::rvm_bridge::modifiers::eval_ignore_armor(
        ej.state_core(),
        ej.catalog(),
        0,
        2,
        Some(sha_far),
        engine_native::event::DamageNature::Normal,
    ));
    assert!(is_rejected(&submit(
        &mut ej,
        0,
        GameAction::PlayCard { card: sha_far, targets: vec![2] }
    )));

    // Finding #2: `ssr_luoyi` discards pre-equipped Armor on acquire, and skips +1 Shan on directHit Sha!
    let mut el = fresh_identity(&p, 25921, &[&["yxs_fanji2"], &[], &[], &[]]);
    let eq_armor = give_kind(&mut el, 0, CardKind::Renwang);
    el.state_core_mut().player_mut(0).hand.retain(|&c| c != eq_armor);
    el.state_core_mut().player_mut(0).equips.push(eq_armor);
    el.debug_grant(0, "ssr_luoyi");
    assert!(el.state_core().player(0).equips.is_empty(), "获得 ssr_luoyi 时须弃置已有防具");
    assert!(el.state_core().discard.contains(&eq_armor));
    assert_eq!(flag(&el, 0, "l25_luoyi_dmg_boost"), 0);
    // Red Sha + `yxs_fanji2` (`directHit`) hits directly WITHOUT asking Shan (`ssr_luoyi` skips directHit)!
    el.debug_set_turn(0, Phase::Play, 1);
    let red_sha = give_card(&mut el, 0, |c| c.kind == CardKind::Sha && c.suit == Suit::Heart);
    let shan1a = give_kind(&mut el, 1, CardKind::Shan);
    let shan1b = give_kind(&mut el, 1, CardKind::Shan);
    play(&mut el, 0, red_sha, vec![1]);
    assert!(el.open_choice().is_none(), "directHit 杀不得触发 ssr_luoyi 多出闪");
    assert_eq!(el.state_core().player(1).hp, 3);
    assert!(el.debug_hand(1).contains(&shan1a) && el.debug_hand(1).contains(&shan1b));
}

#[test]
fn l25_r2_ssr_lieren_and_pyzhuren_diamond_fixes() {
    let p = p9();
    // Seat 1 (NOT Seat 0!) has `ssr_lieren`: Seat 0's TurnBegin STILL equips `pyzhuren_diamond` into Seat 1 hand!
    let mut e = fresh_identity(&p, 25922, &[&[], &["ssr_lieren"], &[], &[]]);
    e.debug_set_turn(3, Phase::End, 1);
    advance_to(&mut e, 0, Phase::Play);
    let py = e
        .debug_hand(1)
        .iter()
        .copied()
        .find(|&cid| e.catalog().cards[cid as usize].name == "pyzhuren_diamond")
        .expect("非先手位 ssr_lieren 开局须获得 pyzhuren_diamond");

    // Verify `ssr_lieren:target_response` respects `ducai3` (`yxs_ducai_forbidden_suit`) AND emits `CardResponded`!
    advance_to(&mut e, 1, Phase::Play);
    let wp2 = give_kind(&mut e, 1, CardKind::Weapon);
    let wp3 = give_kind(&mut e, 1, CardKind::Weapon);
    clear_zone(&mut e, 2);
    clear_zone(&mut e, 3);
    let shan2_heart = give_card(&mut e, 2, |c| c.kind == CardKind::Shan && c.suit == Suit::Heart);
    let shan2_diamond = give_card(&mut e, 2, |c| c.kind == CardKind::Shan && c.suit == Suit::Diamond);
    let shan3 = give_kind(&mut e, 3, CardKind::Shan);
    activate(&mut e, 1, "ssr_lieren");
    answer(&mut e, "ssr_lieren:choose_card", vec![ChoiceOption::Card(wp2)]);
    answer(&mut e, "ssr_lieren:choose_target", vec![ChoiceOption::Player(2)]);
    activate(&mut e, 1, "ssr_lieren");
    answer(&mut e, "ssr_lieren:choose_card", vec![ChoiceOption::Card(wp3)]);
    answer(&mut e, "ssr_lieren:choose_target", vec![ChoiceOption::Player(3)]);
    advance_to(&mut e, 3, Phase::Play);
    e.debug_grant(3, "ducai");
    let d_heart = give_card(&mut e, 3, |c| c.suit == Suit::Heart);
    activate(&mut e, 3, "ducai");
    answer(&mut e, "ducai:choose_card", vec![ChoiceOption::Card(d_heart)]);
    advance_to(&mut e, 0, Phase::End);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let ask_r2 = open(&e, "ssr_lieren:target_response");
    assert!(
        !ask_r2.spec.options.contains(&ChoiceOption::Card(shan2_heart))
            && ask_r2.spec.options.contains(&ChoiceOption::Card(shan2_diamond)),
        "ducai 禁红桃后 ssr_lieren:target_response 不得允许红桃闪但应保留方片闪"
    );
    answer(&mut e, "ssr_lieren:target_response", vec![]);
    let ask_r3 = open(&e, "ssr_lieren:target_response");
    assert!(ask_r3.spec.options.contains(&ChoiceOption::Card(shan3)));
    answer(&mut e, "ssr_lieren:target_response", vec![ChoiceOption::Card(shan3)]);
    drain(&mut e);
    assert!(
        e.log().iter().any(|ev| matches!(
            ev,
            LogEvent::CardResponded { player: 3, card, .. } if *card == shan3
        )),
        "ssr_lieren:target_response 出闪须记录 CardResponded 事件"
    );
    assert_eq!(e.state_core().player(2).hp, 3);
    assert_eq!(e.state_core().player(3).hp, 4);

    // Place `py` onto Seat 2, then Seat 2 dies via fatal Sha -> `ssr_lieren_back` reclaims WITHOUT `ssr_lieren_temp`!
    advance_to(&mut e, 1, Phase::Play);
    activate(&mut e, 1, "ssr_lieren");
    answer(&mut e, "ssr_lieren:choose_card", vec![ChoiceOption::Card(py)]);
    answer(&mut e, "ssr_lieren:choose_target", vec![ChoiceOption::Player(2)]);
    e.debug_set_hp(2, 1);
    let fatal_sha = give_kind(&mut e, 1, CardKind::Sha);
    play(&mut e, 1, fatal_sha, vec![2]);
    drain(&mut e);
    assert!(!e.state_core().player(2).alive);
    assert!(e.debug_hand(1).contains(&py), "死者持有烈刃牌须收回手牌");
    assert_eq!(
        flag(&e, 1, &format!("ssr_lieren_temp:{py}")),
        0,
        "ssr_lieren_back 不得写入 ssr_lieren_temp"
    );
    // Equip `py` cleanly on Seat 1 (`CardAction::PlayCard` puts weapon into equip slot)
    e.debug_grant(1, "咆哮");
    play(&mut e, 1, py, vec![]);
    drain(&mut e);

    // `pyzhuren_diamond` weapon `damageBegin1`: usable 2, excludes self weapon, cancel refunds usable, +1 dmg on Sha!
    let sha_a = give_kind(&mut e, 1, CardKind::Sha);
    let sha_b = give_kind(&mut e, 1, CardKind::Sha);
    let sha_c = give_kind(&mut e, 1, CardKind::Sha);
    let c1 = give_kind(&mut e, 1, CardKind::Tao);
    let c2 = give_kind(&mut e, 1, CardKind::Tao);
    let c3 = give_kind(&mut e, 1, CardKind::Tao);
    e.debug_set_hp(3, 6);
    e.state_core_mut().player_mut(3).max_hp = 6;
    // Sha #1 -> cancel `pyzhuren_diamond:discard_card` (refunds usable!) -> deals 1 dmg
    play(&mut e, 1, sha_a, vec![3]);
    let ask_py = open(&e, "pyzhuren_diamond:discard_card");
    assert!(!ask_py.spec.options.contains(&ChoiceOption::Card(py)), "不能弃置装备中的毗卢遮那宝钻本体");
    assert!(ask_py.spec.options.contains(&ChoiceOption::Card(c1)));
    snapshot_roundtrip_ai(&e, &[&p], vec![ChoiceOption::Card(c1)]);
    answer(&mut e, "pyzhuren_diamond:discard_card", vec![]); // cancel refunds!
    drain(&mut e);
    assert_eq!(e.state_core().player(3).hp, 5);
    // Sha #2 -> discard `c1` -> deals 2 dmg (hp 5 -> 3)
    play(&mut e, 1, sha_b, vec![3]);
    answer(&mut e, "pyzhuren_diamond:discard_card", vec![ChoiceOption::Card(c1)]);
    drain(&mut e);
    assert_eq!(e.state_core().player(3).hp, 3);
    // Duel -> NEVER triggers `pyzhuren_diamond` (parent is NOT `sha`)!
    let duel1 = give_kind(&mut e, 1, CardKind::Duel);
    let _s1_d = give_kind(&mut e, 1, CardKind::Sha);
    play(&mut e, 1, duel1, vec![3]);
    assert!(e.open_choice().is_none_or(|c| c.spec.prompt != "pyzhuren_diamond:discard_card"));
    drain(&mut e);
    assert_eq!(e.state_core().player(3).hp, 2);
    // Sha #3 -> uses 2nd `usable: 2` charge (because #1 was cancelled!) -> discard `c2` -> deals 2 dmg!
    play(&mut e, 1, sha_c, vec![3]);
    answer(&mut e, "pyzhuren_diamond:discard_card", vec![ChoiceOption::Card(c2)]);
    drain(&mut e);
    assert_eq!(e.state_core().player(3).hp, 0);
    let _ = c3;
}

#[test]
fn l25_r2_mode_hooks_ansha_guanfu_guifu_wumou_guixin_and_gongxin_fixes() {
    let p = p9();

    // Finding #4: `_dietime` game-start auto-init + `_anshaskill` when `zhongs.len() == 1` kills zhong AND enters `zhudie`!
    let mut ea = formal_zuan_engine(25923, 4);
    ea.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor],
    };
    zuan_apply_engine_config(
        &mut ea,
        &ZuanModeConfig { mode: 8, balance: true, goodluck: false, ..Default::default() },
    );
    ea.debug_grant(0, "_dietime");
    ea.debug_grant(0, "_anshaskill");
    ea.debug_set_turn(3, Phase::End, 1);
    advance_to(&mut ea, 0, Phase::Play);
    assert_eq!(flag(&ea, 0, "mark:_dietime"), 2, "_dietime 开局须初始化 2 枚倒计时");
    assert!(has_skill(&ea, 3, "ansha_nei"));
    assert!(has_skill(&ea, 1, "ansha_zhong") && has_skill(&ea, 2, "ansha_zhong"));
    // Advance countdown 2 -> 1 -> trigger with 1 zhong left
    ea.debug_set_turn(0, Phase::Discard, 1);
    let _ = submit(&mut ea, 0, GameAction::NextPhase);
    drain(&mut ea);
    assert_eq!(flag(&ea, 0, "mark:_dietime"), 1);
    // Kill Seat 2 manually so only 1 `ansha_zhong` (Seat 1) remains, then trigger countdown `1 -> 0`!
    ea.state_core_mut().player_mut(2).alive = false;
    ea.debug_set_turn(0, Phase::Discard, 2);
    let _ = submit(&mut ea, 0, GameAction::NextPhase);
    drain(&mut ea);
    assert!(!ea.state_core().player(1).alive, "最后一名 ansha_zhong 须死亡");
    assert_eq!(flag(&ea, 0, "l25_zhudie"), 1, "zhongs.len() <= 1 当轮即须进入 zhudie");
    assert_eq!(flag(&ea, 0, "mark:_dietime"), 3);
    if let GameMode::Identity { roles } = &ea.state_core().mode {
        assert_eq!(roles[3], Role::Rebel, "ansha_nei 须翻为反贼");
    }

    // Finding #7: `guanfu` balance Diamond judge + `guanfu_fandie` rebel reward cancel + `guifu_nei` -> `guifu_fire` + `guifu_dam`!
    let mut eg = formal_zuan_engine(25924, 4);
    eg.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor],
    };
    zuan_apply_engine_config(
        &mut eg,
        &ZuanModeConfig { mode: 4, balance: true, goodluck: false, ..Default::default() },
    );
    eg.debug_grant(0, "_guanmodeskill");
    eg.debug_set_turn(3, Phase::End, 1);
    advance_to(&mut eg, 0, Phase::Play);
    assert!(has_skill(&eg, 0, "guanfu") && has_skill(&eg, 3, "guifu_nei"));
    // Balance `guanfu_disrecover`: Diamond #1 fails -> stored in pile & heal cancelled; Diamond #2 passes & discards stored!
    eg.debug_set_hp(1, 2);
    let d1 = take_card(&mut eg, |c| c.suit == Suit::Diamond);
    let d2 = take_card(&mut eg, |c| c.suit == Suit::Diamond);
    put_deck_top(&mut eg, &[d1, d2]);
    let tao_a = give_kind(&mut eg, 1, CardKind::Tao);
    let tao_b = give_kind(&mut eg, 1, CardKind::Tao);
    eg.debug_set_turn(1, Phase::Play, 2);
    play(&mut eg, 1, tao_a, vec![]);
    drain(&mut eg);
    assert_eq!(eg.state_core().player(1).hp, 2, "balance 首次方片判定失败不回血");
    assert_eq!(
        eg.state_core().vm_aux.piles.get(&(1, "guanfu_disrecover".into())).map(Vec::len),
        Some(1)
    );
    play(&mut eg, 1, tao_b, vec![]);
    drain(&mut eg);
    assert_eq!(eg.state_core().player(1).hp, 3, "balance 同花色方片再判成功回血并弃置原牌");
    assert!(eg.state_core().vm_aux.piles.get(&(1, "guanfu_disrecover".into())).is_none());
    // Seat 1 (Rebel) kills Traitor Seat 3 (`guifu_nei`) -> Seat 1 gets `guifu_fire` (3) + `guifu_dam` (1) + hujia(1) + swaps role with Loyalist Seat 2!
    let hm1 = give_kind(&mut eg, 1, CardKind::HorseMinus);
    play(&mut eg, 1, hm1, vec![]);
    clear_zone(&mut eg, 3);
    eg.debug_set_hp(3, 1);
    let sha_k = give_kind(&mut eg, 1, CardKind::Sha);
    play(&mut eg, 1, sha_k, vec![3]);
    drain(&mut eg);
    assert!(has_skill(&eg, 1, "guifu_fire") && has_skill(&eg, 1, "guifu_dam"));
    assert_eq!(flag(&eg, 1, "mark:guifu_fire"), 3);
    assert_eq!(flag(&eg, 1, "mark:guifu_dam"), 1);
    assert_eq!(eg.state_core().player(1).hujia, 1);
    if let GameMode::Identity { roles } = &eg.state_core().mode {
        assert_eq!(roles[1], Role::Rebel, "击杀方反贼 Seat 1 身份保持 Rebel");
        assert_eq!(roles[2], Role::Traitor, "原忠臣 Seat 2 与阵亡 guifu_nei 交换身份成为 Traitor");
        assert_eq!(roles[3], Role::Loyalist, "阵亡 guifu_nei Seat 3 交换身份成为 Loyalist");
    }
    // Activate `guifu_dam` (`save -> restore -> Answer` via formal locked pack) deals 1 Fire dmg to Seat 0!
    activate(&mut eg, 1, "guifu_dam");
    let verified_zuan = formal_zuan_verified();
    let open_gd = open(&eg, "guifu_dam_target");
    let planned = eg.plan_action(open_gd.observer, &BoundedPlanner::new(4));
    assert!(matches!(planned.action, GameAction::Answer { choice_id, .. } if choice_id == open_gd.id));
    let dec_gd = decode_archive(&encode_archive(&eg.save())).expect("decode guifu_dam");
    let mut restored_gd = restore_locked_engine(&dec_gd, &verified_zuan).expect("restore guifu_dam");
    assert_eq!(restored_gd.state_hash(), eg.state_hash());
    assert!(!is_rejected(&submit(
        &mut restored_gd,
        open_gd.observer,
        GameAction::Answer {
            choice_id: open_gd.id,
            selection: vec![ChoiceOption::Player(0)],
        }
    )));
    answer(&mut eg, "guifu_dam_target", vec![ChoiceOption::Player(0)]);
    drain(&mut eg);
    assert_eq!(eg.state_core().player(0).hp, 3);
    assert_eq!(flag(&eg, 1, "mark:guifu_dam"), 0);
    // `guifu_fire` grants +1 draw on each of 3 Draw phases, and removes `guifu_fire` + `guifu_dam` when reaching 0!
    for round_rem in [2u32, 1, 0] {
        advance_to(&mut eg, 0, Phase::End);
        let hand_before_draw = eg.debug_hand(1).len();
        advance_to(&mut eg, 1, Phase::Play);
        assert_eq!(
            eg.debug_hand(1).len() - hand_before_draw,
            3,
            "guifu_fire 回合摸牌阶段须多摸 1 张 (共 3 张)"
        );
        assert_eq!(flag(&eg, 1, "mark:guifu_fire"), round_rem);
    }
    assert!(!has_skill(&eg, 1, "guifu_fire") && !has_skill(&eg, 1, "guifu_dam"));
    // `guanfu_fandie` suppresses 3-card rebel reward (Lord kills Rebel Seat 1) & increments `mark:zuan_guanbing`!
    clear_zone(&mut eg, 1);
    eg.state_core_mut().player_mut(1).hujia = 0;
    eg.debug_set_hp(1, 1);
    eg.debug_set_turn(0, Phase::Play, 6);
    let h_before_kill = eg.debug_hand(0).len();
    let sha_l = give_kind(&mut eg, 0, CardKind::Sha);
    play(&mut eg, 0, sha_l, vec![1]);
    drain(&mut eg);
    assert!(!eg.state_core().player(1).alive);
    assert_eq!(eg.debug_hand(0).len(), h_before_kill, "guanfu_fandie 须取消击杀反贼摸 3 牌奖赏");
    assert!(flag(&eg, 0, "mark:zuan_guanbing") >= 1);

    // `_relifeskill` in `mode == 4` revives dead seat when `timer == 1` (`1 - 1 < 1`) WITHOUT inventing `zuan_hudun`!
    let mut er4 = formal_zuan_engine(25928, 4);
    er4.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor],
    };
    zuan_apply_engine_config(
        &mut er4,
        &ZuanModeConfig { mode: 4, balance: false, goodluck: false, ..Default::default() },
    );
    er4.debug_grant(0, "_relifeskill");
    clear_zone(&mut er4, 2);
    er4.state_core_mut().player_mut(2).alive = false;
    er4.state_core_mut().player_mut(2).hp = 0;
    er4.state_core_mut().player_mut(2).max_hp = 4;
    set_flag(&mut er4, 2, "l25_relife_timer", 1);
    er4.debug_set_turn(3, Phase::End, 1);
    advance_to(&mut er4, 0, Phase::Play);
    assert!(er4.state_core().player(2).alive, "mode == 4 的 _relifeskill 也须触发复活");
    assert_eq!(flag(&er4, 2, "l25_zuan_hudun"), 0);

    // Findings #8 & #9 & minor `gongxin_temp`: `wumou` Nullify out-of-turn & TurnEnded clear (even skipping Phase::End); `new_guixin` TurnOverBefore; `gongxin_temp` persists across Phase::Play end!
    let mut ew = fresh_identity(&p, 25925, &[&["wumou"], &["gongxin"], &[], &[]]);
    set_flag(&mut ew, 0, "mark:baonu", 1);
    ew.debug_set_turn(1, Phase::Play, 1);
    let atk1 = give_kind(&mut ew, 1, CardKind::SavageAttack);
    let _sha0 = give_kind(&mut ew, 0, CardKind::Sha);
    let nul0 = give_kind(&mut ew, 0, CardKind::Nullify);
    play(&mut ew, 1, atk1, vec![]);
    answer(&mut ew, "nullify_window", vec![ChoiceOption::Card(nul0)]);
    drain(&mut ew);
    assert_eq!(flag(&ew, 0, "l25_wumou_trick_in_play"), 0, "他家出牌阶段打无懈不得计入 wumou");
    // Seat 1 uses L25 `gongxin` (`formal_zuan_engine`): `l25_gongxin_temp` persists after Phase::Play -> Phase::Discard, clears on TurnEnded!
    let mut egx_l25 = formal_zuan_engine(25929, 4);
    for s in 0..4u8 { clear_zone(&mut egx_l25, s); }
    egx_l25.debug_grant(1, "gongxin");
    egx_l25.debug_set_turn(1, Phase::Play, 1);
    let heart2 = give_card(&mut egx_l25, 2, |c| c.suit == Suit::Heart);
    activate(&mut egx_l25, 1, "gongxin");
    answer(&mut egx_l25, "gongxin_target", vec![ChoiceOption::Player(2)]);
    answer(&mut egx_l25, "gongxin_card", vec![ChoiceOption::Card(heart2)]);
    answer(&mut egx_l25, "gongxin_choice", vec![ChoiceOption::Tag(1)]);
    drain(&mut egx_l25);
    assert_eq!(flag(&egx_l25, 2, "l25_gongxin_temp"), 1);
    let _ = submit(&mut egx_l25, 1, GameAction::NextPhase); // Play -> Discard
    drain(&mut egx_l25);
    assert_eq!(flag(&egx_l25, 2, "l25_gongxin_temp"), 1, "出牌阶段结束后攻心封牌应持续至回合结束");
    advance_to(&mut egx_l25, 2, Phase::Play);
    assert_eq!(flag(&egx_l25, 2, "l25_gongxin_temp"), 0, "回合结束须清除 l25_gongxin_temp");
    // Even when `wumou` is skipped at Phase::End (e.g. `baonu == 0`), TurnEnded STILL clears `l25_wumou_trick_in_play`!
    advance_to(&mut ew, 0, Phase::Play);
    set_flag(&mut ew, 0, "mark:baonu", 0);
    let wz0 = give_kind(&mut ew, 0, CardKind::Wuzhong);
    play(&mut ew, 0, wz0, vec![]);
    drain(&mut ew);
    assert_eq!(flag(&ew, 0, "l25_wumou_trick_in_play"), 1);
    advance_to(&mut ew, 1, Phase::Play);
    assert_eq!(flag(&ew, 0, "l25_wumou_trick_in_play"), 0, "未在结束阶段结算 wumou 时 TurnEnded 仍须清理 wumou 标记");
    // TurnOverBefore BEFORE 2nd `new_guixin:confirm` on multi-damage!
    let mut egx = fresh_identity(&p, 25926, &[&["new_guixin"], &[], &[], &[]]);
    for s in 1..4u8 {
        let _ = give_kind(&mut egx, s, CardKind::Sha);
        let _ = give_kind(&mut egx, s, CardKind::Shan);
    }
    egx.debug_set_turn(1, Phase::Play, 1);
    let jiu1 = give_kind(&mut egx, 1, CardKind::Jiu);
    let sha1g = give_kind(&mut egx, 1, CardKind::Sha);
    play(&mut egx, 1, jiu1, vec![]);
    play(&mut egx, 1, sha1g, vec![0]);
    answer(&mut egx, "new_guixin:confirm", vec![ChoiceOption::Tag(1)]);
    answer(&mut egx, "new_guixin:choose_zone", vec![ChoiceOption::Tag(0)]);
    assert!(
        egx.state_core().player(0).face_down,
        "第一次新归心获牌 > 2 后须先翻面再弹出第二次询问"
    );
    let _ = open(&egx, "new_guixin:confirm");
    answer(&mut egx, "new_guixin:confirm", vec![]);
    drain(&mut egx);

    // Finding #10 config gates: `_fromsay`, `_zuanCHANGEHPsayskill`, `_newZuanPhaseTX`
    let mut ecfg = fresh_identity(
        &p,
        25927,
        &[&["_fromsay", "_zuanCHANGEHPsayskill", "_newZuanPhaseTX"], &[], &[], &[]],
    );
    zuan_apply_engine_config(
        &mut ecfg,
        &ZuanModeConfig {
            mode: 1,
            balance: false,
            goodluck: false,
            zuantype_new: true,
            zuantype_old: false,
            zuan_emotion: false,
            ransay: 100,
            newhuihe_new: false,
        },
    );
    ecfg.debug_set_turn(1, Phase::Play, 1);
    let sha1c = give_kind(&mut ecfg, 1, CardKind::Sha);
    play(&mut ecfg, 1, sha1c, vec![2]);
    drain(&mut ecfg);
    assert_eq!(flag(&ecfg, 0, "l25_ui_hook:_fromsay"), 0, "zuantype=new 不得触发 _fromsay");
    assert_eq!(flag(&ecfg, 0, "l25_ui_hook:_zuanCHANGEHPsayskill"), 1, "zuantype_new+ransay 无需 zuan_emotion 即可触发 _zuanCHANGEHPsayskill");
    advance_to(&mut ecfg, 0, Phase::Play);
    assert_eq!(flag(&ecfg, 0, "l25_ui_hook:_newZuanPhaseTX"), 0, "newhuihe_new=false 时不得因 zuantype_new 触发 _newZuanPhaseTX");
}
