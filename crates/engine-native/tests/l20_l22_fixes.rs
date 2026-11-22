//! Batch Q2-L20+L22 规则修复与 counterexample 回归测试（`L20 标记补充` + `L22 段位制裁`）。
//!
//! 对照上游：
//! - `L20 mibei` (`archive/upstream/noname-liuli/noname/extension/标记补充/extension.js:4592-4674` + `character/shiji.js:2406-2457`)
//! - `L20 wengua` / `wengua5` (`archive/upstream/noname-liuli/noname/extension/标记补充/extension.js:6141-6154` + `character/yijiang.js:4426-4559`)
//! - `L20 kuanshi` / `kuanshi2` (`archive/upstream/noname-liuli/noname/extension/标记补充/extension.js:5299-5372`)
//! - `L20 tspowei` (`archive/upstream/noname-liuli/noname/extension/标记补充/extension.js:4366-4531`)
//! - `L20 fz_liegong` (`archive/upstream/noname-liuli/noname/extension/标记补充/extension.js:5160-5197` + `character/shenhua.js:6456-6475`)
//! - `L22 _eloDraw` (`archive/upstream/noname-liuli/noname/extension/段位制裁/extension.js:117-138, 181-263`)
//! - `L22 _timeKiller` (`archive/upstream/noname-liuli/noname/extension/段位制裁/extension.js:288-458`)

use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::{
    decision34_pack9_extension_pack, elo_apply_engine_config, elo_sanction_extension_pack,
    EloConfig, EloQuery,
};
use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind, Suit};
use engine_native::content::extension::{
    content_lock_id, ExtensionDef, ExtensionPack, InitHook, VmSkillDecl,
};
use engine_native::content::host::{HostBehavior, HostSkillDecl};
use engine_native::event::{EventKind, Phase, Timing};
use engine_native::kernel::{
    elo_get_player_scalar, elo_judge_pref_for, elo_set_global_scalar, elo_set_player_scalar,
};
use engine_native::state::{GameMode, Role};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, OpenChoice};

static DEDUP: AtomicU64 = AtomicU64::new(20_900_000);

fn l20_pack() -> ExtensionPack {
    decision34_pack9_extension_pack()
}

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

fn is_rejected(out: CommandOutcome) -> bool {
    matches!(out, CommandOutcome::Rejected(_))
}

fn fresh(p: &ExtensionPack, seats: usize, seed: u64, skills_by_seat: &[&[&str]]) -> Engine {
    let mut e = Engine::new_with_extensions(seed, seats, &[p]).expect("build engine");
    for s in 0..seats as u8 {
        e.state_core_mut().skills.retain(|&(o, _), _| o != s);
        clear_zone(&mut e, s);
        e.state_core_mut().player_mut(s).max_hp = 4;
        e.state_core_mut().player_mut(s).hp = 4;
    }
    for (s, sks) in skills_by_seat.iter().enumerate() {
        for &sk in *sks {
            e.debug_grant(s as u8, sk);
        }
    }
    e
}

fn clear_zone(e: &mut Engine, s: u8) {
    let h: Vec<u32> = e.state_core_mut().player_mut(s).hand.drain(..).collect();
    let eq: Vec<u32> = e.state_core_mut().player_mut(s).equips.drain(..).collect();
    let jd: Vec<u32> = e.state_core_mut().player_mut(s).judge.drain(..).collect();
    e.state_core_mut()
        .discard
        .extend(h.into_iter().chain(eq).chain(jd));
}

fn flag(e: &Engine, p: u8, key: &str) -> u32 {
    e.state_core()
        .vm_aux
        .flags
        .get(&(p, key.to_string()))
        .copied()
        .unwrap_or(0)
}

fn set_flag(e: &mut Engine, p: u8, key: &str, v: u32) {
    if v == 0 {
        e.state_core_mut().vm_aux.flags.remove(&(p, key.to_string()));
    } else {
        e.state_core_mut()
            .vm_aux
            .flags
            .insert((p, key.to_string()), v);
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

fn give_card(e: &mut Engine, seat: u8, pred: impl Fn(&engine_native::catalog::CardDef) -> bool) -> u32 {
    let c = take_card(e, pred);
    e.state_core_mut().player_mut(seat).hand.push(c);
    c
}

fn give_kind(e: &mut Engine, seat: u8, kind: CardKind) -> u32 {
    give_card(e, seat, |cd| cd.kind == kind)
}

fn play(e: &mut Engine, seat: u8, card: u32, targets: Vec<u8>) -> CommandOutcome {
    let out = submit(e, seat, GameAction::PlayCard { card, targets });
    assert!(!is_rejected(out.clone()), "play rejected: {out:?}");
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
    assert!(!is_rejected(out.clone()), "answer rejected for {prompt}: {out:?}");
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

fn advance_to(e: &mut Engine, target_seat: u8, target_phase: Phase) {
    for _ in 0..32 {
        drain(e);
        if e.turn().current == target_seat && e.turn().phase == target_phase {
            return;
        }
        let actor = e.turn().current;
        let _ = submit(e, actor, GameAction::NextPhase);
    }
    panic!("failed to reach seat {target_seat} phase {target_phase:?}, got {:?}", e.turn());
}

fn roundtrip_bytes(e: &Engine, packs: &[&ExtensionPack]) -> Engine {
    let snap = e.save();
    let bytes = encode_archive(&snap);
    let dec = decode_archive(&bytes).expect("decode archive");
    let restored = Engine::restore_with_extensions(&dec, packs).expect("restore");
    assert_eq!(restored.state_hash(), e.state_hash());
    restored
}

fn snapshot_roundtrip(e: &Engine, packs: &[&ExtensionPack], sel: Vec<ChoiceOption>) {
    if let Some(c) = e.open_choice() {
        let planned = e.plan_action(c.observer, &BoundedPlanner::new(4));
        assert!(
            matches!(planned.action, GameAction::Answer { choice_id, .. } if choice_id == c.id),
            "AI must answer open {} cleanly, got {:?}",
            c.spec.prompt,
            planned.action
        );
        let mut ai_probe = roundtrip_bytes(e, packs);
        assert!(!is_rejected(submit(&mut ai_probe, c.observer, planned.action)));
    }
    let mut restored = roundtrip_bytes(e, packs);
    if let Some(c) = restored.open_choice().cloned() {
        assert!(!is_rejected(submit(
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
// 1. L20 `mibei`（标记补充/extension.js:4592-4674 + character/shiji.js:2406-2457）
// =============================================================================

#[test]
fn l20_fix_mibei_dedup_effective_name_use_as_and_delay_exclusion() {
    let p = l20_pack();
    // Grant `mibei` + `paoxiao` + `武圣` (red card -> Sha) on Seat 0
    let mut e = fresh(&p, 4, 20901, &[&["mibei", "咆哮", "武圣"], &[], &[], &[]]);
    e.debug_set_hp(1, 10);
    e.debug_set_turn(0, Phase::Play, 1);

    // 1) Duplicate same-name uses MUST NOT double-count (2x Sha, 2x Wuzhong, 2x HorsePlus -> only 1/1/1).
    let sha1 = give_kind(&mut e, 0, CardKind::Sha);
    let sha2 = give_kind(&mut e, 0, CardKind::Sha);
    let fire_sha = give_kind(&mut e, 0, CardKind::FireSha);
    play(&mut e, 0, sha1, vec![1]);
    drain(&mut e);
    play(&mut e, 0, sha2, vec![1]);
    drain(&mut e);
    // Upstream `fire_sha` also has `card.name == 'sha'`!
    play(&mut e, 0, fire_sha, vec![1]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "mibei_basic_cnt"), 1, "重复使用杀/火杀不得反复累计数");
    assert_eq!(flag(&e, 0, "xingqi_count"), 1);

    // Delayed tricks (`Le`, `Bing`) must NOT enter `xingqi` or `mibei_trick_cnt`.
    let le = give_kind(&mut e, 0, CardKind::Le);
    play(&mut e, 0, le, vec![2]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "mibei_trick_cnt"), 0, "延时锦囊乐不思蜀不得计入星启/秘备");
    assert_eq!(flag(&e, 0, "xingqi_count"), 1);

    // 2) `use_as` conversion (`武圣` converts red `Shan` into `Sha`): logical card is basic `sha`
    // (NOT `"shan"`), and because `sha` is already recorded, counts remain (1, 0, 0).
    let red_shan_1 = give_card(&mut e, 0, |cd| {
        cd.kind == CardKind::Shan && matches!(cd.suit, Suit::Heart | Suit::Diamond)
    });
    play(&mut e, 0, red_shan_1, vec![1]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "mibei_basic_cnt"), 1, "红闪当杀不得重复记杀");
    assert_eq!(flag(&e, 0, "mibei_trick_cnt"), 0);
    assert_eq!(flag(&e, 0, "xingqi_card:shan"), 0, "转化为杀使用的红闪不得记成闪");
    let ws = e.catalog().skill_by_name("武圣").unwrap();
    e.state_core_mut().skills.remove(&(0, ws));

    // Now use two Wuzhong normally + two identical-name Armor/Equip: still only 1 trick + 1 equip!
    let wz1 = give_card(&mut e, 0, |cd| cd.kind == CardKind::Wuzhong && cd.name == "wuzhong");
    let wz2 = give_card(&mut e, 0, |cd| cd.kind == CardKind::Wuzhong && cd.name == "wuzhong");
    play(&mut e, 0, wz1, vec![]);
    drain(&mut e);
    play(&mut e, 0, wz2, vec![]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "mibei_trick_cnt"), 1, "两次无中生有只能记 1 个锦囊名");

    let eq1 = give_card(&mut e, 0, |cd| cd.kind == CardKind::HorsePlus && cd.name == "horse_plus");
    let eq2 = give_card(&mut e, 0, |cd| cd.kind == CardKind::HorsePlus && cd.name == "horse_plus");
    play(&mut e, 0, eq1, vec![]);
    drain(&mut e);
    play(&mut e, 0, eq2, vec![]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "mibei_equip_cnt"), 1, "两张同名防御马只能记 1 个装备名");
    assert_eq!(flag(&e, 0, "mibei_resolved"), 0, "仅 (1,1,1) 不得提前完成秘备");

    // Put a delayed trick near top of deck to ensure `mibei` trick reward skips delayed tricks!
    let top_le = take_card(&mut e, |cd| cd.kind == CardKind::Le);
    put_deck_top(&mut e, &[top_le]);

    // 3) Complete distinct second names: `Tao` (basic #2), `Dismantle` (trick #2), `Zhuge` (equip #2)
    e.debug_set_hp(0, 3);
    let tao = give_kind(&mut e, 0, CardKind::Tao);
    play(&mut e, 0, tao, vec![]);
    drain(&mut e);
    let dis = give_card(&mut e, 0, |cd| cd.kind == CardKind::Dismantle && cd.name == "dismantle");
    let dummy_s1 = give_kind(&mut e, 1, CardKind::Shan);
    play(&mut e, 0, dis, vec![1]);
    drain(&mut e);
    let _ = dummy_s1;
    assert_eq!(flag(&e, 0, "mibei_resolved"), 0, "差 1 种装备时不得提前成功");

    let zhuge_eq2 = give_kind(&mut e, 0, CardKind::Zhuge);
    let hand_before = e.debug_hand(0).len();
    play(&mut e, 0, zhuge_eq2, vec![]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "mibei_resolved"), 1);
    assert_eq!(flag(&e, 0, "mibei_succeeded"), 1);
    assert_eq!(flag(&e, 0, "xinmouli_granted"), 1);
    assert_eq!(e.debug_hand(0).len(), hand_before - 1 + 3);
    assert!(!e.debug_hand(0).contains(&top_le), "秘备所摸锦囊不得摸成延时锦囊乐不思蜀");

    // 4) Prepare without empty xingqi does not fail at End
    let _ = roundtrip_bytes(&e, &[&p]);
}

// =============================================================================
// 2. L20 `wengua` / `wengua5`（标记补充/extension.js:6141-6154 + yijiang.js:4426-4559）
// =============================================================================

#[test]
fn l20_fix_wengua_no_independent_active_and_self_other_he_via_wengua5() {
    let p = l20_pack();
    // Grant ONLY `wengua` to Seat 0 (L20 overrides wengua = { group: ["wengua5"] }, NO enable: 'phaseUse'!).
    let mut e = fresh(&p, 4, 20902, &[&["wengua"], &[], &[], &[]]);

    // 1) Direct Play-phase `ActivateSkill("wengua")` MUST be rejected and absent from legal_actions & AI.
    e.debug_set_turn(0, Phase::Play, 1);
    let c0_hand = give_kind(&mut e, 0, CardKind::Sha);
    let wg_def = e.catalog().skill_by_name("wengua").unwrap();
    assert!(!e
        .legal_actions(0)
        .iter()
        .any(|a| matches!(a, GameAction::ActivateSkill { skill } if *skill == wg_def)));
    assert!(is_rejected(activate(&mut e, 0, "wengua")));
    let planned = e.plan_action(0, &BoundedPlanner::new(4));
    assert!(!matches!(planned.action, GameAction::ActivateSkill { skill } if skill == wg_def));

    // 2) Entering `wengua` owner Seat 0's own `Phase::Prepare` triggers `wengua` -> `wengua5` (`global: phaseZhunbeiBegin`),
    // granting Seat 0 temporary `wengua2` (`wengua5` active entry) so Seat 0 can self-wengua ONCE!
    e.debug_set_turn(3, Phase::End, 1);
    advance_to(&mut e, 0, Phase::Play);
    assert_eq!(flag(&e, 0, "wengua2_active"), 1, "自己准备阶段也须触发 wengua5 授临时 wengua2");
    assert_eq!(e.turn().phase, Phase::Play);
    assert!(is_rejected(activate(&mut e, 0, "wengua")), "wengua 自身无 phaseUse 主动");

    // Self-wengua using an equipped `HorsePlus` (`position: 'he'`) -> non-cancelable choose_side (top), draws ONLY 1 card from bottom!
    let horse_eq = give_kind(&mut e, 0, CardKind::HorsePlus);
    play(&mut e, 0, horse_eq, vec![]);
    drain(&mut e);
    let hand_before_self = e.debug_hand(0).len();
    assert!(!is_rejected(activate(&mut e, 0, "wengua5")));
    let ask_give = open(&e, "wengua5:give_card");
    assert!(ask_give.spec.options.contains(&ChoiceOption::Card(horse_eq)), "自己问卦也可选装备区牌");
    assert!(ask_give.spec.options.contains(&ChoiceOption::Card(c0_hand)));
    snapshot_roundtrip(&e, &[&p], vec![ChoiceOption::Card(horse_eq)]);
    answer(&mut e, "wengua5:give_card", vec![ChoiceOption::Card(horse_eq)]);

    let ask_side = open(&e, "wengua5:choose_side");
    assert_eq!(ask_side.observer, 0);
    assert!(!ask_side.spec.allow_cancel, "上游 event.target == player 时 chooseControlList(..., true) 不可取消");
    let bot_card = e.state_core().deck[0];
    answer(&mut e, "wengua5:choose_side", vec![ChoiceOption::Tag(0)]);
    drain(&mut e);
    assert_eq!(e.state_core().deck.last().copied(), Some(horse_eq));
    assert!(e.state_core().player(0).equips.is_empty());
    assert_eq!(e.debug_hand(0).len(), hand_before_self + 1, "自我问卦仅自己摸 1 张（绝不得摸 2 张）");
    assert!(e.debug_hand(0).contains(&bot_card));
    assert!(is_rejected(activate(&mut e, 0, "wengua5")), "同回合限一次");

    // 3) Advance to Seat 1's turn: Seat 1's `Phase::Prepare` gets temporary `wengua5`,
    // equips `Armor`, gives `Armor` via `he` `wengua5`, Seat 0 chooses bottom (cancelable) -> BOTH draw 1 from top!
    advance_to(&mut e, 1, Phase::Play);
    assert_eq!(flag(&e, 1, "wengua2_active"), 1);
    assert_eq!((e.turn().current, e.turn().phase), (1, Phase::Play));
    let armor1 = give_kind(&mut e, 1, CardKind::Armor);
    play(&mut e, 1, armor1, vec![]);
    drain(&mut e);
    let h0_before = e.debug_hand(0).len();
    let h1_before = e.debug_hand(1).len();
    assert!(!is_rejected(activate(&mut e, 1, "wengua5")));
    answer(&mut e, "wengua5:give_card", vec![ChoiceOption::Card(armor1)]);
    let side_other = open(&e, "wengua5:choose_side");
    assert_eq!(side_other.observer, 0, "须由 wengua 持有者 Seat 0 选置顶/置底");
    assert!(side_other.spec.allow_cancel, "他家给牌时 wengua 持有者可取消直接留牌");
    snapshot_roundtrip(&e, &[&p], vec![ChoiceOption::Tag(1)]);
    answer(&mut e, "wengua5:choose_side", vec![ChoiceOption::Tag(1)]);
    drain(&mut e);
    assert_eq!(e.state_core().deck.first().copied(), Some(armor1));
    assert!(e.state_core().player(1).equips.is_empty());
    assert_eq!(e.debug_hand(0).len(), h0_before + 1);
    assert_eq!(e.debug_hand(1).len(), h1_before + 1);
}

// =============================================================================
// 3. L20 `kuanshi` & `kuanshi2`（标记补充/extension.js:5299-5372）
// =============================================================================

#[test]
fn l20_fix_kuanshi_post_mod_damage_and_multi_protector_cleanup() {
    let mut p = l20_pack();
    // VM modifier simulating post-mod `damage_amount +1` (e.g. xunmeng/luoyi/tengjia+fire)
    // and mitigation (`global_damage_taken_final_amount -1` / `damage_taken_cancel`).
    let src = "\
skill kuanshi_dmg_plus1(420090011) locked { modifier damage_amount(sha) add 1; }\n\
skill kuanshi_dmg_cancel_fire(420090012) locked { modifier damage_taken_cancel(fire) override 1; }";
    let pkg = content_compiler::compile_source(src).unwrap();
    p.vm_skills
        .extend(content_native_loader::bridge_vm_skill_decls(&pkg).unwrap().decls);

    // Test 1: Base 1 Sha boosted to 2 (`kuanshi_dmg_plus1`) MUST trigger `kuanshi2` (`effective_damage_amount > 1`);
    // canceled fire damage (`kuanshi_dmg_cancel_fire`, even with base 2 via Jiu+FireSha) MUST NOT trigger `kuanshi2`.
    let mut e = fresh(
        &p,
        4,
        20903,
        &[&["kuanshi"], &[], &["kuanshi_dmg_cancel_fire"], &["kuanshi_dmg_plus1"]],
    );
    e.debug_set_hp(0, 4);
    e.debug_set_hp(2, 4);
    // Enter Seat 0 Phase::End via Discard -> End
    e.debug_set_turn(0, Phase::Discard, 1);
    submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "kuanshi:choose_target", vec![ChoiceOption::Player(2)]);
    drain(&mut e);
    assert!(has_skill(&e, 2, "kuanshi2"));

    // Seat 2 has `kuanshi_dmg_cancel_fire` -> Seat 1 uses `Jiu + FireSha` (amount=2, but `is_damage_taken_canceled` == true):
    // kuanshi2 MUST NOT intercept or mark `kuanshi_skip_draw`!
    advance_to(&mut e, 1, Phase::Play);
    let jiu1 = give_kind(&mut e, 1, CardKind::Jiu);
    let fsha1 = give_kind(&mut e, 1, CardKind::FireSha);
    play(&mut e, 1, jiu1, vec![]);
    drain(&mut e);
    play(&mut e, 1, fsha1, vec![2]);
    drain(&mut e);
    assert_eq!(e.state_core().player(2).hp, 4);
    assert_eq!(flag(&e, 0, "kuanshi_skip_draw"), 0, "已被完全免疫的伤害不得白耗宽释保护");
    assert!(has_skill(&e, 2, "kuanshi2"));

    // Seat 3 (`kuanshi_dmg_plus1`: +1 damage on Sha) attacks Seat 2 with `Sha` (base 1 -> effective 2):
    advance_to(&mut e, 3, Phase::Play);
    let sha3 = give_kind(&mut e, 3, CardKind::Sha);
    play(&mut e, 3, sha3, vec![2]);
    drain(&mut e);
    assert_eq!(e.state_core().player(2).hp, 4, "1 点基础杀经伤害修正增伤为 2 时 kuanshi2 须拦截");
    assert_eq!(flag(&e, 0, "kuanshi_skip_draw"), 1);
    assert!(!has_skill(&e, 2, "kuanshi2"));

    // Test 2: Multi-protector overwrite (Seat 0 protects Seat 3, then Seat 1 also protects Seat 3):
    // When Seat 0's Prepare clears Seat 0's state, Seat 1's active `kuanshi2` on Seat 3 MUST survive!
    let mut m = fresh(&p, 4, 20904, &[&["kuanshi"], &["kuanshi"], &[], &[]]);
    m.debug_set_turn(0, Phase::Discard, 1);
    submit(&mut m, 0, GameAction::NextPhase);
    answer(&mut m, "kuanshi:choose_target", vec![ChoiceOption::Player(3)]);
    drain(&mut m);
    advance_to(&mut m, 1, Phase::Discard);
    submit(&mut m, 1, GameAction::NextPhase);
    answer(&mut m, "kuanshi:choose_target", vec![ChoiceOption::Player(3)]);
    drain(&mut m);
    assert_eq!(flag(&m, 3, "kuanshi_protected_by"), 2, "后保护者 Seat 1 覆盖 Seat 3 保护来源");

    // Advance into Seat 0's Play (past Seat 0's Prepare): Seat 0's cleanup must NOT remove Seat 1's `kuanshi2` on Seat 3!
    advance_to(&mut m, 0, Phase::Play);
    assert!(has_skill(&m, 3, "kuanshi2"), "先保护者过期清理不得误删后保护者的 kuanshi2");
    assert_eq!(flag(&m, 3, "kuanshi_protected_by"), 2);

    // Give Seat 0 `RedHorseMinus` or `Sha`? Seat 0 -> Seat 3 distance is 1 in 4-seat circle!
    // 2 damage via Jiu + Sha from Seat 0 against Seat 3 is still prevented by Seat 1's protection (and Seat 1 skips draw)
    let jiu0 = give_kind(&mut m, 0, CardKind::Jiu);
    let sha0 = give_kind(&mut m, 0, CardKind::Sha);
    play(&mut m, 0, jiu0, vec![]);
    drain(&mut m);
    play(&mut m, 0, sha0, vec![3]);
    drain(&mut m);
    assert_eq!(m.state_core().player(3).hp, 4);
    assert_eq!(flag(&m, 1, "kuanshi_skip_draw"), 1);
    assert_eq!(flag(&m, 0, "kuanshi_skip_draw"), 0);
}

// =============================================================================
// 4. L20 `tspowei`（标记补充/extension.js:4366-4531）
// =============================================================================

#[test]
fn l20_fix_tspowei_dulie_stacking_move_achieve_and_options() {
    let p = l20_pack();
    let mut e = fresh(&p, 4, 20905, &[&["tspowei"], &[], &[], &[]]);
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 3);
    e.debug_set_hp(2, 3);
    e.debug_set_hp(3, 3);

    // Round 1: Seat 0 starts -> `tspowei_init` gives `dulie = 1` to Seat 1, 2, 3
    e.debug_set_turn(3, Phase::End, 1);
    submit(&mut e, 3, GameAction::NextPhase);
    drain(&mut e);
    assert_eq!((flag(&e, 1, "dulie"), flag(&e, 2, "dulie"), flag(&e, 3, "dulie")), (1, 1, 1));

    // Put 2 dulie on Seat 3 and clear Seat 2 (`tspowei_wei = 0, dulie = 0`), then test cyclic move at Seat 0 TurnBegin:
    // Seat 1 (1) -> Seat 2 (1); Seat 3 (2) skips Seat 0 -> Seat 1 (2)!
    set_flag(&mut e, 2, "tspowei_wei", 0);
    set_flag(&mut e, 2, "dulie", 0);
    set_flag(&mut e, 3, "dulie", 2);
    e.debug_set_turn(3, Phase::End, 2);
    submit(&mut e, 3, GameAction::NextPhase);
    drain(&mut e);
    assert_eq!(flag(&e, 1, "dulie"), 2, "顺移跳过 owner 须保留原标记层数 (2)");
    assert_eq!(flag(&e, 2, "dulie"), 1);
    assert_eq!(flag(&e, 3, "dulie"), 0);

    // Clear Seat 2, and test Seat 1 TurnBegin option 1 (gain card from Seat 1 + `inRange`)
    set_flag(&mut e, 2, "tspowei_wei", 0);
    set_flag(&mut e, 2, "dulie", 0);
    let s1_a = give_kind(&mut e, 1, CardKind::Sha);
    let s1_b = give_kind(&mut e, 1, CardKind::Shan);
    e.debug_set_turn(0, Phase::End, 2);
    submit(&mut e, 0, GameAction::NextPhase);
    let c = open(&e, "tspowei:choose_action");
    assert!(c.spec.allow_cancel);
    snapshot_roundtrip(&e, &[&p], vec![ChoiceOption::Tag(1)]);
    answer(&mut e, "tspowei:choose_action", vec![ChoiceOption::Tag(1)]);
    answer(&mut e, "tspowei:gain_card", vec![ChoiceOption::Card(s1_a)]);
    drain(&mut e);
    assert!(e.debug_hand(0).contains(&s1_a));
    assert_eq!(e.debug_hand(1), vec![s1_b]);
    assert_eq!(flag(&e, 0, "tspowei_in_range"), 2);

    // Advance via natural turns to Seat 0 Play: during Seat 0's TurnBegin, Seat 1's stacked `dulie = 2`
    // cyclic-moves to Seat 2 (`dulie = 2`). Then Seat 0's `Duel` damages Seat 2, clearing ALL `2 -> 0` marks,
    // and natural advance into next Seat 0 Prepare achieves `shenzhu`!
    advance_to(&mut e, 0, Phase::Play);
    assert_eq!(flag(&e, 2, "dulie"), 2);
    let duel_dmg = give_kind(&mut e, 0, CardKind::Duel);
    play(&mut e, 0, duel_dmg, vec![2]);
    drain(&mut e);
    assert_eq!(flag(&e, 2, "dulie"), 0);
    assert_eq!(flag(&e, 2, "tspowei_wei"), 0);
    advance_to(&mut e, 0, Phase::Judge);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "tspowei_achieved"), 1);
    assert_eq!(flag(&e, 0, "shenzhu_granted"), 1);
}

// =============================================================================
// 5. L20 `fz_liegong`（标记补充/extension.js:5160-5197）运行时 (Normal, 0, true) 与锁不变
// =============================================================================

#[test]
fn l20_fix_fz_liegong_runtime_order_selectable_and_lock_preserved() {
    let mut p = l20_pack();
    // Verify static content_lock_id of `decision34_pack9_extension_pack()` reflects Batch Q2-L25's
    // authentic `ssr_jinfan` card removal + `pyzhuren_diamond`/mode skill additions (`1_761_060_662_268_120_978`);
    // frozen `u01-dddanzhi-pending-v107.ngsa` coexistence lock (`9_097_112_637_251_845_257`) remains untouched.
    let base_rest = decision34_pack9_extension_pack();
    assert_eq!(content_lock_id(&[&base_rest]), 1_761_060_662_268_120_978);

    // Add a dummy CardUsed (Normal, priority=0, order_selectable=true) VM skill to verify `trigger_order` prompt!
    let source = "skill l20_sha_peer(420090001) { trigger card_used owner_is_source { effect storage_set(self_skill, 1, 1); } }";
    let pkg = content_compiler::compile_source(source).unwrap();
    let vm_decls: Vec<VmSkillDecl> = content_native_loader::bridge_vm_skill_decls(&pkg)
        .unwrap()
        .decls;
    p.vm_skills.extend(vm_decls);

    // Also add standalone `HostBehavior::FzLiegong` alias to check both bindings.
    p.host_skills.push(HostSkillDecl {
        name: "fz_liegong_standalone",
        behavior: HostBehavior::FzLiegong,
        presentation_binding: None,
    });

    let mut e = fresh(&p, 4, 20906, &[&["fz_liegong", "l20_sha_peer"], &[], &[], &[]]);
    for name in ["fz_liegong", "fz_liegong_standalone"] {
        let def = e.catalog().skill_by_name(name).unwrap();
        let meta = e.catalog().skills[def as usize]
            .behavior
            .kind_trigger_meta(EventKind::CardUsed)
            .expect("kind_trigger_meta(CardUsed)");
        assert_eq!(
            (meta.timing, meta.priority, meta.order_selectable),
            (Timing::Normal, 0, true),
            "{name} runtime trigger must match upstream (Normal, 0, true)"
        );
    }

    e.debug_set_turn(0, Phase::Play, 1);
    let sha = give_kind(&mut e, 0, CardKind::Sha);
    let shan1 = give_kind(&mut e, 1, CardKind::Shan);
    // Seat 1 hand len = 1 <= atk_range(0) = 1 -> fz_liegong qualifies AND l20_sha_peer qualifies -> opens `trigger_order`!
    play(&mut e, 0, sha, vec![1]);
    let order_c = open(&e, "trigger_order");
    assert_eq!(order_c.observer, 0);
    let liegong_def = e.catalog().skill_by_name("fz_liegong").unwrap();
    let lg_opt = ChoiceOption::Skill(liegong_def);
    assert!(order_c.spec.options.contains(&lg_opt));
    snapshot_roundtrip(&e, &[&p], vec![lg_opt]);
    answer(&mut e, "trigger_order", vec![lg_opt]);
    answer(&mut e, "fz_liegong:confirm", vec![ChoiceOption::Confirm]);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).hp, 3, "烈弓强命后目标即使手牌有闪也不能响应");
    assert!(e.debug_hand(1).contains(&shan1));

    // Also check standalone `HostBehavior::FzLiegong` via `mark-supplement-upstream-liuli` pack pattern:
    let mut sp = ExtensionPack {
        def: ExtensionDef {
            name: "fz-lg-probe",
            version: 1,
            description: "probe",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![HostSkillDecl {
            name: "fz_liegong",
            behavior: HostBehavior::FzLiegong,
            presentation_binding: None,
        }],
        vm_skills: vec![],
        init_hook: InitHook::None,
        enabled: true,
    };
    let e_standalone = Engine::new_with_extensions(20907, 4, &[&sp]).unwrap();
    let d_st = e_standalone.catalog().skill_by_name("fz_liegong").unwrap();
    let meta_st = e_standalone.catalog().skills[d_st as usize]
        .behavior
        .kind_trigger_meta(EventKind::CardUsed)
        .unwrap();
    assert_eq!((meta_st.timing, meta_st.priority, meta_st.order_selectable), (Timing::Normal, 0, true));
    sp.enabled = true;
}

// =============================================================================
// 6. L22 `_eloDraw`（段位制裁/extension.js:222-263）：自定义技能判定偏好
// =============================================================================

fn setup_l22(seed: u64, diff: &str, tk: &str) -> (ExtensionPack, Engine) {
    let pack = elo_sanction_extension_pack();
    let mut engine = Engine::new_with_extensions(seed, 4, &[&pack]).expect("build L22 engine");
    for s in 0..4u8 {
        clear_zone(&mut engine, s);
    }
    engine.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor],
    };
    elo_apply_engine_config(
        &mut engine,
        &EloConfig {
            source: "noname-liuli".to_string(),
            elo_players: true,
            difficulty: diff.to_string(),
            time_killer: tk.to_string(),
        },
        &EloQuery {
            online: true,
            mode: "identity".to_string(),
            paiwei_level: None,
        },
    );
    (pack, engine)
}

#[test]
fn l22_fix_elo_draw_custom_skill_judge_prefs_luoshen_ganglie_bagua_and_ordinary() {
    // Counterexample (`#85`):
    // 1) `洛神` wants Black (Spade/Club):
    //    - under `ehard` (eh = -1, diff = 4), `_eloDraw` MUST put Red (Heart) on `top[0]` to fizzle Seat 0's `洛神`!
    //    - under `eeasy` (eh = +1, diff = 4), `_eloDraw` MUST put Black on `top[0]` so Seat 0 gains it!
    for (diff, want_red_top) in [("ehard", true), ("eeasy", false)] {
        let (_pack, mut e) = setup_l22(22901, diff, "none");
        e.debug_grant(0, "洛神");
        let sp = take_card(&mut e, |c| c.suit == Suit::Spade && c.kind == CardKind::Sha);
        let ht = take_card(&mut e, |c| c.suit == Suit::Heart && c.kind == CardKind::Shan);
        let cl = take_card(&mut e, |c| c.suit == Suit::Club && c.kind == CardKind::Sha);
        let dm = take_card(&mut e, |c| c.suit == Suit::Diamond && c.kind == CardKind::Sha);
        put_deck_top(&mut e, &[sp, ht, cl, dm]);
        e.debug_set_turn(3, Phase::End, 1);
        submit(&mut e, 3, GameAction::NextPhase);
        answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]);
        drain(&mut e);
        if want_red_top {
            assert!(e.debug_hand(0).is_empty(), "ehard 须把红牌调至牌堆顶令 Seat 0 洛神立即断掉");
        } else {
            assert!(!e.debug_hand(0).is_empty(), "eeasy 须把黑牌调至牌堆顶令 Seat 0 洛神摸到黑牌");
        }
    }

    // 2) `刚烈` wants `!Heart`:
    //    - under `ehard`, `_eloDraw` MUST put `Heart` at `top[0]` so Seat 0's `刚烈` fails!
    //    - under `eeasy`, `_eloDraw` MUST put `!Heart` at `top[0]` so Seat 0's `刚烈` triggers!
    for (diff, want_heart_top) in [("ehard", true), ("eeasy", false)] {
        let (_pack, mut e) = setup_l22(22902, diff, "none");
        e.debug_grant(0, "刚烈");
        let ht = take_card(&mut e, |c| c.suit == Suit::Heart && c.kind == CardKind::Tao);
        let sp = take_card(&mut e, |c| c.suit == Suit::Spade && c.kind == CardKind::Sha);
        let cl = take_card(&mut e, |c| c.suit == Suit::Club && c.kind == CardKind::Sha);
        let dm = take_card(&mut e, |c| c.suit == Suit::Diamond && c.kind == CardKind::Sha);
        if want_heart_top {
            put_deck_top(&mut e, &[sp, ht, cl, dm]);
        } else {
            put_deck_top(&mut e, &[ht, sp, cl, dm]);
        }
        e.debug_set_turn(1, Phase::Play, 1);
        let atk = give_kind(&mut e, 1, CardKind::Sha);
        let hp1_before = e.state_core().player(1).hp;
        play(&mut e, 1, atk, vec![0]);
        let c = open(&e, "trigger_confirm");
        assert_eq!(c.observer, 0);
        answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]);
        drain(&mut e);
        if want_heart_top {
            assert_eq!(e.state_core().player(1).hp, hp1_before, "ehard 放红桃顶使刚烈失败");
        } else {
            assert_eq!(e.state_core().player(1).hp, hp1_before - 1, "eeasy 放非红桃使刚烈造成 1 伤");
        }
    }

    // 3) `八卦阵` (`sha/armor.rs:64` checks `j_suit.is_heart()`) wants `Heart` OVER `Diamond` (`HeartPositive`):
    // with `[Spade, Diamond, Club, Heart]` at top, `eeasy` MUST move `Heart` above `Diamond` to `top[0]`!
    let (_pack, mut e_bagua) = setup_l22(22903, "eeasy", "none");
    let bagua = take_card(&mut e_bagua, |c| c.kind == CardKind::Armor && c.name == "armor");
    e_bagua.state_core_mut().player_mut(0).equips.push(bagua);
    let sp = take_card(&mut e_bagua, |c| c.suit == Suit::Spade);
    let dm = take_card(&mut e_bagua, |c| c.suit == Suit::Diamond);
    let cl = take_card(&mut e_bagua, |c| c.suit == Suit::Club);
    let ht = take_card(&mut e_bagua, |c| c.suit == Suit::Heart);
    put_deck_top(&mut e_bagua, &[sp, dm, cl, ht]);
    e_bagua.debug_set_turn(1, Phase::Play, 1);
    let atk = give_kind(&mut e_bagua, 1, CardKind::Sha);
    play(&mut e_bagua, 1, atk, vec![0]);
    drain(&mut e_bagua);
    assert_eq!(e_bagua.state_core().player(0).hp, 4, "eeasy 应把红桃置于方块前移至牌堆顶令八卦阵判定成功");

    // 4) OrdinaryJudge (`JudgePhaseMachine` with `trick == None`): `"极点"` (`BOSS_JIDIAN`) judges target Seat 0
    // (Seat 0 wants non-black, so Diamond avoids 1 Thunder damage whereas old `CardKind::Le` missed Diamond!)
    let (_pack_jd, mut e_jd) = setup_l22(22909, "eeasy", "none");
    e_jd.debug_grant(1, "极点");
    let sp = take_card(&mut e_jd, |c| c.suit == Suit::Spade);
    let cl = take_card(&mut e_jd, |c| c.suit == Suit::Club);
    let sp2 = take_card(&mut e_jd, |c| c.suit == Suit::Spade);
    let dm = take_card(&mut e_jd, |c| c.suit == Suit::Diamond);
    put_deck_top(&mut e_jd, &[sp, cl, sp2, dm]);
    e_jd.debug_set_turn(1, Phase::Play, 1);
    let sha_jd = give_kind(&mut e_jd, 1, CardKind::Sha);
    play(&mut e_jd, 1, sha_jd, vec![2]);
    // Drain until Jidian target prompt or complete; select Seat 0 as Jidian target
    for _ in 0..8 {
        let Some(c) = e_jd.open_choice().cloned() else {
            break;
        };
        if c.spec.options.contains(&ChoiceOption::Player(0)) {
            submit(
                &mut e_jd,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Player(0)],
                },
            );
        } else {
            drain(&mut e_jd);
        }
    }
    assert_eq!(e_jd.state_core().player(0).hp, 4, "eeasy 须在极点普通判定把方块调顶使 Seat 0 免受雷伤");

    // 5) Unmapped OrdinaryJudge must return `None` (MUST NOT fall back to `CardKind::Le`)!
    assert_eq!(
        elo_judge_pref_for(
            e_jd.state_core(),
            e_jd.resolution(),
            e_jd.catalog(),
            None,
            Some(999_999)
        ),
        None,
        "无明确偏好的普通判定绝不得伪造 CardKind::Le 偏好"
    );
}

#[test]
fn l22_fix_time_killer_restricts_to_use_respond_discard_without_extra_rng() {
    // Pair of identical engines (Seat 0 = `game.me`, Seat 1/2/3 = AI):
    // `a` has `_timeKiller = heavy` (`elo_time_killer_val = 3`), `b` has `none` (`0`).
    // Unhooked prompts (`trigger_confirm`, `wugu_pick`, Dismantle/Snatch `choose_player_card`, `fire_attack_show`)
    // MUST NOT trigger `_timeKiller` or consume any extra RNG (`a.rng == b.rng`).
    // Fresh pair testing ONLY unhooked prompts on AI Seat 1 (no use/respond/discard!):
    // (a) `fanjian` (`fanjian_target`, `fanjian_suit`, `fanjian_pick`) -> `遗计` (`trigger_confirm`)
    // (b) `wugu_pick` (`五谷丰登`)
    // (c) `Dismantle` (`choose_player_card`) by AI Seat 1
    // (d) `Snatch` (`choose_player_card`) by AI Seat 1
    let (pack, mut a) = setup_l22(22905, "normal", "heavy");
    let (_, mut b) = setup_l22(22905, "normal", "none");
    for eng in [&mut a, &mut b] {
        eng.debug_grant(0, "反间");
        eng.debug_grant(1, "遗计");
        eng.debug_set_turn(0, Phase::Play, 1);
        let give_club = give_card(eng, 0, |c| c.suit == Suit::Club && c.kind == CardKind::Sha);
        activate(eng, 0, "反间");
        answer(eng, "fanjian_target", vec![ChoiceOption::Player(1)]);
        answer(eng, "fanjian_suit", vec![ChoiceOption::Tag(0)]); // Heart != Club -> 1 damage to Seat 1
        answer(eng, "fanjian_pick", vec![ChoiceOption::Card(give_club)]);
        answer(eng, "trigger_confirm", vec![ChoiceOption::Confirm]);
        drain(eng);

        let wugu = give_kind(eng, 0, CardKind::Wugu);
        play(eng, 0, wugu, vec![]);
        let wc0 = open(eng, "wugu_pick").spec.options[0];
        answer(eng, "wugu_pick", vec![wc0]);
        let wc1 = open(eng, "wugu_pick").spec.options[0];
        answer(eng, "wugu_pick", vec![wc1]);
        drain(eng);

        // AI Seat 1 Dismantle + Snatch (`choose_player_card` MUST NOT invoke `_timeKiller`)
        eng.debug_set_turn(1, Phase::Play, 1);
        let _h0 = give_kind(eng, 0, CardKind::Sha);
        let d1 = give_kind(eng, 1, CardKind::Dismantle);
        let sn1 = give_kind(eng, 1, CardKind::Snatch);
        play(eng, 1, d1, vec![0]);
        drain(eng);
        play(eng, 1, sn1, vec![0]);
        drain(eng);
    }
    assert_eq!(elo_get_player_scalar(a.state_core(), 1, "elo_time_killer_triggers"), None, "非 use/respond/discard 询问绝不得触发 _timeKiller");
    assert_eq!(elo_get_player_scalar(a.state_core(), 1, "elo_time_killer_delay_ms"), None);
    assert_eq!(a.state_core().rng, b.state_core().rng, "非目标询问绝不得额外消耗 RNG！");

    // Positive checks: genuine `chooseToRespondBefore` (`respond_shan`), `chooseToUseBefore` (`Phase::Play`),
    // and `chooseToDiscardBefore` (`discard_to_limit`) on AI Seat 1 DO trigger `_timeKiller`!
    elo_set_global_scalar(a.state_core_mut(), "elo_zuan_enable", 1);
    elo_set_player_scalar(a.state_core_mut(), 1, "elo_time_killer_ran_permille", 850);
    a.debug_set_turn(0, Phase::Play, 2);
    let _shan1 = give_kind(&mut a, 1, CardKind::Shan);
    let sha_hit = give_kind(&mut a, 0, CardKind::Sha);
    play(&mut a, 0, sha_hit, vec![1]);
    let r = open(&a, "respond_shan");
    assert_eq!(r.observer, 1);
    assert_eq!(elo_get_player_scalar(a.state_core(), 1, "elo_time_killer_triggers"), Some(1));
    assert_eq!(elo_get_player_scalar(a.state_core(), 1, "elo_time_killer_delay_ms"), Some(3 * 1500 * 850 / 1000));
    let _ = roundtrip_bytes(&a, &[&pack]);

    // Negative: game.me (Seat 0) does NOT trigger `_timeKiller` on `respond_shan`
    elo_set_player_scalar(a.state_core_mut(), 0, "elo_time_killer_ran_permille", 850);
    answer(&mut a, "respond_shan", vec![ChoiceOption::Pass]);
    drain(&mut a);
    // Advance into Seat 1 Play (`chooseToUseBefore`) -> triggers = 2, then Seat 1 Discard with excess hand -> triggers = 3!
    advance_to(&mut a, 1, Phase::Play);
    assert_eq!(elo_get_player_scalar(a.state_core(), 1, "elo_time_killer_triggers"), Some(2));
    for _ in 0..6 {
        let _ = give_kind(&mut a, 1, CardKind::Shan);
    }
    submit(&mut a, 1, GameAction::NextPhase);
    let disc = open(&a, "discard_to_limit");
    assert_eq!(disc.observer, 1);
    assert_eq!(elo_get_player_scalar(a.state_core(), 1, "elo_time_killer_triggers"), Some(3));
    drain(&mut a);

    a.debug_set_turn(1, Phase::Play, 2);
    let _shan0 = give_kind(&mut a, 0, CardKind::Shan);
    let sha_to_me = give_kind(&mut a, 1, CardKind::Sha);
    play(&mut a, 1, sha_to_me, vec![0]);
    let _ = open(&a, "respond_shan");
    assert_eq!(elo_get_player_scalar(a.state_core(), 0, "elo_time_killer_triggers"), None);
}

// =============================================================================
// 7. Round 2 Counterexamples (M1/M2, W1/W2/W3, T1, K1/K2, F1/F2, E1/E2, TK1)
// =============================================================================

#[test]
fn l20_fix_mibei_no_discard_fallback_and_turn_end_mark_cleanup() {
    let p = l20_pack();
    // M1: `get.cardPile2` searches `deck` ONLY, NEVER `discard` (`标记补充/extension.js:4620`, `game.js:63223-63225`).
    let mut e = fresh(&p, 4, 20921, &[&["mibei", "咆哮"], &[], &[], &[]]);
    e.debug_set_hp(1, 10);
    e.debug_set_turn(0, Phase::Play, 1);

    // Seed deck = [Sha, Wuzhong, Le] (no equip), discard = [Armor] (`bagua`).
    // Take cards BEFORE clearing deck/discard so all needed IDs still exist in deck/discard!
    let d_sha = take_card(&mut e, |c| c.kind == CardKind::Sha);
    let d_wuz = take_card(&mut e, |c| c.kind == CardKind::Wuzhong);
    let d_le = take_card(&mut e, |c| c.kind == CardKind::Le);
    let disc_bagua = take_card(&mut e, |c| c.kind == CardKind::Armor);
    let s1 = take_card(&mut e, |c| c.kind == CardKind::Sha);
    let t1 = take_card(&mut e, |c| c.kind == CardKind::Tao);
    let duel1 = take_card(&mut e, |c| c.kind == CardKind::Duel);
    let dis = take_card(&mut e, |c| c.kind == CardKind::Dismantle);
    let eq1 = take_card(&mut e, |c| c.kind == CardKind::HorsePlus);
    let eq2 = take_card(&mut e, |c| c.kind == CardKind::Zhuge);
    let dummy1 = take_card(&mut e, |c| c.kind == CardKind::Shan);
    e.state_core_mut().deck = vec![d_le, d_wuz, d_sha];
    e.state_core_mut().discard = vec![disc_bagua];
    e.state_core_mut().player_mut(0).hand = vec![s1, t1, duel1, dis, eq1, eq2];
    e.state_core_mut().player_mut(1).hand = vec![dummy1];
    e.debug_set_hp(0, 3);

    play(&mut e, 0, s1, vec![1]); drain(&mut e);
    play(&mut e, 0, t1, vec![]); drain(&mut e);
    play(&mut e, 0, duel1, vec![1]); drain(&mut e);
    play(&mut e, 0, dis, vec![1]); drain(&mut e);
    play(&mut e, 0, eq1, vec![]); drain(&mut e);
    let h_before = e.debug_hand(0).len();
    play(&mut e, 0, eq2, vec![]); drain(&mut e);
    assert_eq!(flag(&e, 0, "mibei_succeeded"), 1);
    assert!(e.debug_hand(0).contains(&d_sha));
    assert!(e.debug_hand(0).contains(&d_wuz));
    assert!(!e.debug_hand(0).contains(&d_le), "秘备锦囊不得摸延时锦囊");
    assert!(!e.debug_hand(0).contains(&disc_bagua), "秘备不得从弃牌堆捞装备");
    assert!(e.state_core().discard.contains(&disc_bagua), "弃牌堆装备必须保持原位");
    assert_eq!(e.debug_hand(0).len(), h_before - 1 + 2, "缺装备时仅摸基本+普通锦囊共 2 张");

    // M2: `mibei_mark` expires on `TurnEnded`/`TurnPhaseAfter` even if `Phase::End` was skipped (`extension.js:4642`).
    let mut e2 = fresh(&p, 4, 20922, &[&["mibei"], &[], &[], &[]]);
    e2.debug_set_turn(3, Phase::End, 1);
    advance_to(&mut e2, 0, Phase::Play);
    assert_eq!(flag(&e2, 0, "mibei_mark"), 1);
    // Simulate skipping `Phase::End` begin directly to turn end cleanup (`End` finish -> `TurnEnded` -> `TurnEndWindow`).
    e2.debug_set_turn(0, Phase::End, 1);
    submit(&mut e2, 0, GameAction::NextPhase);
    drain(&mut e2);
    assert_eq!(flag(&e2, 0, "mibei_mark"), 0, "跳过结束阶段时 TurnEnded 仍须清掉 mibei_mark");
    assert_eq!(e2.state_core().player(0).max_hp, 4, "未触发 phaseJieshuBegin 不得误扣体力上限");
}

#[test]
fn l20_fix_wengua_draw_order_single_deck_phase_use_end_and_multi_holder() {
    let p = l20_pack();
    // W1: Verify `giver` draws FIRST and `wg_owner` draws SECOND (`yijiang.js:4526-4543`).
    let mut e = fresh(&p, 4, 20923, &[&["wengua"], &[], &[], &[]]);
    let tao = take_card(&mut e, |c| c.kind == CardKind::Tao && c.suit == Suit::Heart);
    let shan = take_card(&mut e, |c| c.kind == CardKind::Shan && c.suit == Suit::Diamond);
    let sha = take_card(&mut e, |c| c.kind == CardKind::Sha && c.suit == Suit::Club);
    let wuz = take_card(&mut e, |c| c.kind == CardKind::Wuzhong && c.suit == Suit::Heart);
    let duel = take_card(&mut e, |c| c.kind == CardKind::Duel && c.suit == Suit::Spade);

    // Put Top (`Tag(0)`): deck top->bottom = [tao, shan, sha, wuz], given `duel` ->
    // `giver` (Seat 1) draws `wuz` (bottom) FIRST, `wg_owner` (Seat 0) draws `sha` SECOND!
    advance_to(&mut e, 1, Phase::Play);
    clear_zone(&mut e, 0);
    clear_zone(&mut e, 1);
    e.state_core_mut().deck = vec![wuz, sha, shan, tao]; // [bottom .. top]
    e.state_core_mut().player_mut(1).hand = vec![duel];
    assert!(!is_rejected(activate(&mut e, 1, "wengua5")));
    answer(&mut e, "wengua5:give_card", vec![ChoiceOption::Card(duel)]);
    answer(&mut e, "wengua5:choose_side", vec![ChoiceOption::Tag(0)]);
    drain(&mut e);
    assert_eq!(e.debug_hand(1), vec![wuz], "Put Top 时给牌者先从牌堆底摸 wuz");
    assert_eq!(e.debug_hand(0), vec![sha], "Put Top 时问卦持有者后从牌堆底摸 sha");
    assert_eq!(e.state_core().deck.last().copied(), Some(duel));

    // Put Bottom (`Tag(1)`): deck top->bottom = [tao, shan, sha, wuz], given `duel` ->
    // `giver` (Seat 1) draws `tao` (top) FIRST, `wg_owner` (Seat 0) draws `shan` SECOND!
    let mut e_bot = fresh(&p, 4, 20924, &[&["wengua"], &[], &[], &[]]);
    let tao = take_card(&mut e_bot, |c| c.kind == CardKind::Tao && c.suit == Suit::Heart);
    let shan = take_card(&mut e_bot, |c| c.kind == CardKind::Shan && c.suit == Suit::Diamond);
    let sha = take_card(&mut e_bot, |c| c.kind == CardKind::Sha && c.suit == Suit::Club);
    let wuz = take_card(&mut e_bot, |c| c.kind == CardKind::Wuzhong && c.suit == Suit::Heart);
    let duel = take_card(&mut e_bot, |c| c.kind == CardKind::Duel && c.suit == Suit::Spade);
    advance_to(&mut e_bot, 1, Phase::Play);
    clear_zone(&mut e_bot, 0);
    clear_zone(&mut e_bot, 1);
    e_bot.state_core_mut().deck = vec![wuz, sha, shan, tao];
    e_bot.state_core_mut().player_mut(1).hand = vec![duel];
    assert!(!is_rejected(activate(&mut e_bot, 1, "wengua5")));
    answer(&mut e_bot, "wengua5:give_card", vec![ChoiceOption::Card(duel)]);
    answer(&mut e_bot, "wengua5:choose_side", vec![ChoiceOption::Tag(1)]);
    drain(&mut e_bot);
    assert_eq!(e_bot.debug_hand(1), vec![tao], "Put Bottom 时给牌者先从牌堆顶摸 tao");
    assert_eq!(e_bot.debug_hand(0), vec![shan], "Put Bottom 时问卦持有者后从牌堆顶摸 shan");
    assert_eq!(e_bot.state_core().deck.first().copied(), Some(duel));

    // 1-card deck (`ui.cardPile.childElementCount == 1` after placing `duel`): ONLY `giver` draws `duel`, `wg_owner` draws 0!
    let mut e_one = fresh(&p, 4, 20925, &[&["wengua"], &[], &[], &[]]);
    advance_to(&mut e_one, 1, Phase::Play);
    clear_zone(&mut e_one, 0);
    clear_zone(&mut e_one, 1);
    let duel_one = take_card(&mut e_one, |c| c.kind == CardKind::Duel);
    let arm_disc = take_card(&mut e_one, |c| c.kind == CardKind::Armor);
    e_one.state_core_mut().deck.clear();
    e_one.state_core_mut().discard = vec![arm_disc];
    e_one.state_core_mut().player_mut(1).hand = vec![duel_one];
    assert!(!is_rejected(activate(&mut e_one, 1, "wengua5")));
    answer(&mut e_one, "wengua5:give_card", vec![ChoiceOption::Card(duel_one)]);
    answer(&mut e_one, "wengua5:choose_side", vec![ChoiceOption::Tag(0)]);
    drain(&mut e_one);
    assert_eq!(e_one.debug_hand(1), vec![duel_one], "牌堆仅 1 张时仅给牌者摸回");
    assert!(e_one.debug_hand(0).is_empty(), "牌堆仅 1 张时问卦者不得再摸或触发洗牌");
    assert_eq!(e_one.state_core().discard, vec![arm_disc]);

    // W2 (`phaseUseEnd` reset): second `Phase::Play` in same turn can use `wengua5` again!
    let mut e_w2 = fresh(&p, 4, 20926, &[&["wengua"], &[], &[], &[]]);
    e_w2.debug_set_turn(3, Phase::End, 1);
    advance_to(&mut e_w2, 0, Phase::Play);
    let c0a = give_kind(&mut e_w2, 0, CardKind::Sha);
    let c0b = give_kind(&mut e_w2, 0, CardKind::Shan);
    e_w2.state_core_mut().turn.extra_phases.push(2); // extra Phase::Play
    assert!(!is_rejected(activate(&mut e_w2, 0, "wengua5")));
    answer(&mut e_w2, "wengua5:give_card", vec![ChoiceOption::Card(c0a)]);
    answer(&mut e_w2, "wengua5:choose_side", vec![ChoiceOption::Tag(0)]);
    drain(&mut e_w2);
    assert!(is_rejected(activate(&mut e_w2, 0, "wengua5")), "同一出牌阶段内不得再用");
    submit(&mut e_w2, 0, GameAction::NextPhase); // Play #1 End -> extra Play #2!
    drain(&mut e_w2);
    assert_eq!((e_w2.turn().current, e_w2.turn().phase), (0, Phase::Play));
    assert!(e_w2.debug_hand(0).contains(&c0b));
    assert!(!is_rejected(activate(&mut e_w2, 0, "wengua5")), "phaseUseEnd 重置 wengua3 后新出牌阶段可再用");
    answer(&mut e_w2, "wengua5:give_card", vec![ChoiceOption::Card(c0b)]);
    answer(&mut e_w2, "wengua5:choose_side", vec![ChoiceOption::Tag(1)]);
    drain(&mut e_w2);

    // W3: (a) Losing `wengua` between Prepare and Play disables `wengua5`; (b) >=2 holders prompts `wengua5:choose_target`!
    let wg_def = e_w2.catalog().skill_by_name("wengua").unwrap();
    advance_to(&mut e_w2, 1, Phase::Play);
    let _c1 = give_kind(&mut e_w2, 1, CardKind::Sha);
    e_w2.state_core_mut().skills.remove(&(0, wg_def));
    assert!(is_rejected(activate(&mut e_w2, 1, "wengua5")), "场上已无 wengua 持有者时不可发动 wengua5");

    let mut e_multi = fresh(&p, 4, 20927, &[&["wengua"], &[], &["wengua"], &[]]);
    advance_to(&mut e_multi, 1, Phase::Play);
    let cg = give_kind(&mut e_multi, 1, CardKind::Sha);
    assert!(!is_rejected(activate(&mut e_multi, 1, "wengua5")));
    answer(&mut e_multi, "wengua5:give_card", vec![ChoiceOption::Card(cg)]);
    let tgt_ask = open(&e_multi, "wengua5:choose_target");
    assert!(!tgt_ask.spec.allow_cancel);
    assert!(tgt_ask.spec.options.contains(&ChoiceOption::Player(0)));
    assert!(tgt_ask.spec.options.contains(&ChoiceOption::Player(2)));
    snapshot_roundtrip(&e_multi, &[&p], vec![ChoiceOption::Player(2)]);
    answer(&mut e_multi, "wengua5:choose_target", vec![ChoiceOption::Player(2)]);
    let side_ask = open(&e_multi, "wengua5:choose_side");
    assert_eq!(side_ask.observer, 2);
    answer(&mut e_multi, "wengua5:choose_side", vec![ChoiceOption::Tag(1)]);
    drain(&mut e_multi);
}

#[test]
fn l20_fix_tspowei_one_way_in_range_without_distance_mutation() {
    let p = l20_pack();
    // T1 (`extension.js:4434-4442`): `tspowei_in_range` gives `in_attack_range(target, owner) == true` ONLY,
    // without mutating `computed_distance` (`Snatch` at dist 2 must fail!) and without reversing `owner -> target`.
    let mut e = fresh(&p, 4, 20928, &[&["tspowei"], &[], &[], &[]]);
    advance_to(&mut e, 2, Phase::Play);
    // Clear marks on Seat 1 and re-trigger Seat 2's TurnBegin with mark/hand setup
    e.state_core_mut().vm_aux.flags.clear();
    set_flag(&mut e, 2, "dulie", 1);
    set_flag(&mut e, 2, "tspowei_wei", 1);
    let _c0 = give_kind(&mut e, 0, CardKind::Tao);
    let sha2 = give_kind(&mut e, 2, CardKind::Sha);
    let sn2 = give_kind(&mut e, 2, CardKind::Snatch);
    e.debug_set_turn(1, Phase::End, 2);
    submit(&mut e, 1, GameAction::NextPhase);
    answer(&mut e, "tspowei:choose_action", vec![ChoiceOption::Tag(1)]); // Option 1 (no discard) sets tspowei_in_range
    drain(&mut e);
    advance_to(&mut e, 2, Phase::Play);
    assert_eq!(e.state_core().computed_distance(e.catalog(), 2, 0), 2, "物理距离不得被改写为 1");
    assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 2);
    assert!(e.state_core().in_attack_range(e.catalog(), 2, 0), "当前回合角色视为可将 owner 计入攻击范围");
    assert!(!e.state_core().in_attack_range(e.catalog(), 0, 2), "inRangeOf 仅单向生效，不得反向扩距");
    assert!(is_rejected(submit(&mut e, 2, GameAction::PlayCard { card: sn2, targets: vec![0] })), "顺手牵羊看距离<=1仍必须拒绝");
    let out_sha = submit(&mut e, 2, GameAction::PlayCard { card: sha2, targets: vec![0] });
    assert!(!is_rejected(out_sha), "距离 2 的当前回合角色借助 tspowei_inRange 可对 owner 用杀");
    drain(&mut e);
}

#[test]
fn l20_fix_kuanshi_only_kuanshi2_damage_before_and_no_judge_cleanup() {
    let p = l20_pack();
    // K1: If target's `kuanshi2` is disabled or `tieji`-sealed while `kuanshi` remains active on protector,
    // `kuanshi` MUST NOT duplicate `DamageBefore`: full 2 damage lands!
    let mut e = fresh(&p, 4, 20929, &[&["kuanshi"], &[], &[], &[]]);
    e.debug_set_turn(0, Phase::Discard, 1);
    submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "kuanshi:choose_target", vec![ChoiceOption::Player(2)]);
    drain(&mut e);
    assert!(has_skill(&e, 2, "kuanshi2"));

    // Disable `kuanshi2` on Seat 2 (and verify `kuanshi2` is sealable in Catalog!).
    let k2_def = e.catalog().skill_by_name("kuanshi2").unwrap();
    assert!(!engine_native::catalog::Catalog::is_locked_skill_name("kuanshi2", None));
    e.state_core_mut().skills.get_mut(&(2, k2_def)).unwrap().enabled = false;
    advance_to(&mut e, 1, Phase::Play);
    let jiu1 = give_kind(&mut e, 1, CardKind::Jiu);
    let sha1 = give_kind(&mut e, 1, CardKind::Sha);
    play(&mut e, 1, jiu1, vec![]); drain(&mut e);
    play(&mut e, 1, sha1, vec![2]); drain(&mut e);
    assert_eq!(e.state_core().player(2).hp, 2, "kuanshi2 失效/被封时 kuanshi 不得代为防止伤害");
    assert_eq!(flag(&e, 0, "kuanshi_skip_draw"), 0);

    // K2: `Phase::Judge` must NOT clean up active `kuanshi2` (`extension.js:5357` only cleans at `phaseZhunbeiBegin`/`dieAfter`).
    let mut e2 = fresh(&p, 4, 20930, &[&["kuanshi"], &[], &[], &[]]);
    e2.debug_set_turn(0, Phase::Discard, 1);
    submit(&mut e2, 0, GameAction::NextPhase);
    answer(&mut e2, "kuanshi:choose_target", vec![ChoiceOption::Player(2)]);
    drain(&mut e2);
    advance_to(&mut e2, 1, Phase::Judge);
    assert!(has_skill(&e2, 2, "kuanshi2"), "他家 Judge 阶段不得提前清除 kuanshi2");
    e2.debug_set_turn(0, Phase::Prepare, 2);
    submit(&mut e2, 0, GameAction::NextPhase); // Prepare -> Judge (begin: true)
    drain(&mut e2);
    assert!(has_skill(&e2, 2, "kuanshi2"), "Judge 阶段不得提前清除 kuanshi2");
    advance_to(&mut e2, 1, Phase::Play);
    advance_to(&mut e2, 0, Phase::Play); // Wraps through Seat 0 next Phase::Prepare { begin: true } -> cleaned!
    assert!(!has_skill(&e2, 2, "kuanshi2"), "经过保护者下一准备阶段后须清除 kuanshi2");
}

#[test]
fn l20_fix_fz_liegong_shouyue_zhu_and_virtual_sha() {
    let mut p = l20_pack();
    let sy_pkg = content_compiler::compile_source(
        "skill shouyue(420090013) locked { modifier damage_amount(sha) add 0; }",
    )
    .unwrap();
    p.vm_skills
        .extend(content_native_loader::bridge_vm_skill_decls(&sy_pkg).unwrap().decls);
    // F1 (`extension.js:5187`, `game.js:61049-61093`): `shouyue` +1 range checks `get.zhu(player, 'shouyue')`.
    let mut e = fresh(&p, 4, 20931, &[&[], &["shouyue"], &["fz_liegong"], &[]]);
    e.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor],
    };
    assert_eq!(e.state_core().attack_range(e.catalog(), 2), 1, "反贼持有守约不得使主公技范围 +1");
    e.debug_grant(0, "shouyue");
    assert_eq!(e.state_core().attack_range(e.catalog(), 2), 2, "主公持有守约时 fz_liegong 攻击范围 +1");
    e.state_core_mut().mode = GameMode::Prototype;
    assert_eq!(e.state_core().attack_range(e.catalog(), 2), 1, "Prototype 非主公模式无 get.zhu 守约加成");

    // F2 (`mark_sha_unrespondable` when `u.card == None`, e.g., virtual Sha via `"兵先"`/`use_virtual_card`).
    let mut e2 = fresh(&p, 4, 20932, &[&["fz_liegong", "兵先"], &[], &[], &[]]);
    let shan1 = give_kind(&mut e2, 1, CardKind::Shan);
    e2.debug_set_turn(1, Phase::Discard, 1);
    submit(&mut e2, 1, GameAction::NextPhase); // Seat 1 enters Phase::End -> `兵先` emits virtual `Sha` (`u.card == None`)!
    answer(&mut e2, "fz_liegong:confirm", vec![ChoiceOption::Confirm]);
    drain(&mut e2);
    assert_eq!(e2.state_core().player(1).hp, 3, "虚拟杀 (u.card == None) 命中 fz_liegong 强命时目标不可出闪");
    assert!(e2.debug_hand(1).contains(&shan1));
}

#[test]
fn l22_fix_elo_draw_zj_leiji_and_direct_judge_skills() {
    // E1: `"雷击·张角"` (`zj_leiji.rs`) judges `Spade | Club` (`BlackPositive`): `eeasy` picks `Club` over `Heart`!
    for (diff, want_hit) in [("eeasy", true), ("ehard", false)] {
        let (_pk, mut e) = setup_l22(22921, diff, "none");
        e.debug_grant(0, "雷击·张角");
        let ht = take_card(&mut e, |c| c.suit == Suit::Heart);
        let dm = take_card(&mut e, |c| c.suit == Suit::Diamond);
        let ht2 = take_card(&mut e, |c| c.suit == Suit::Heart);
        let cl = take_card(&mut e, |c| c.suit == Suit::Club);
        put_deck_top(&mut e, &[ht, dm, ht2, cl]);
        e.debug_set_turn(1, Phase::Play, 1);
        let sh = give_kind(&mut e, 0, CardKind::Shan);
        let sha1 = give_kind(&mut e, 1, CardKind::Sha);
        play(&mut e, 1, sha1, vec![0]);
        answer(&mut e, "respond_shan", vec![ChoiceOption::Card(sh)]);
        if e.open_choice().is_some_and(|c| c.spec.prompt == "trigger_confirm") {
            answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]);
        }
        drain(&mut e); // triggers ZjLeiji
        let target_hp = e.state_core().player(1).hp;
        if want_hit {
            assert_eq!(target_hp, 3, "eeasy 应把梅花调至牌堆顶令雷击·张角判定黑色成功");
        } else {
            assert_eq!(target_hp, 4, "ehard 应把红牌放顶令雷击·张角判定失败");
        }
    }

    // E2: Direct judge skill (`busi` -> `NonSpadePositive` + `new_wuhun` -> `TaoOrTaoyuanPositive`)
    let mut pack_elo = elo_sanction_extension_pack();
    pack_elo.host_skills.push(HostSkillDecl {
        name: "busi",
        behavior: HostBehavior::SwdBusi,
        presentation_binding: None,
    });
    let pack_d9 = l20_pack();
    let mut e_busi = Engine::new_with_extensions(22922, 4, &[&pack_elo, &pack_d9]).unwrap();
    for s in 0..4u8 { clear_zone(&mut e_busi, s); }
    e_busi.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor],
    };
    elo_apply_engine_config(
        &mut e_busi,
        &EloConfig { source: "noname-liuli".into(), elo_players: true, difficulty: "eeasy".into(), time_killer: "none".into() },
        &EloQuery { online: true, mode: "identity".into(), paiwei_level: None },
    );
    e_busi.debug_grant(0, "busi");
    e_busi.debug_set_hp(0, 1);
    let sp = take_card(&mut e_busi, |c| c.suit == Suit::Spade);
    let cl = take_card(&mut e_busi, |c| c.suit == Suit::Club);
    let sp2 = take_card(&mut e_busi, |c| c.suit == Suit::Spade);
    let ht = take_card(&mut e_busi, |c| c.suit == Suit::Heart);
    put_deck_top(&mut e_busi, &[sp, sp2, cl, ht]);
    e_busi.debug_set_turn(1, Phase::Play, 1);
    let sha_b = give_kind(&mut e_busi, 1, CardKind::Sha);
    play(&mut e_busi, 1, sha_b, vec![0]);
    for _ in 0..8 {
        if e_busi.open_choice().is_some_and(|c| c.spec.prompt == "trigger_confirm") {
            answer(&mut e_busi, "trigger_confirm", vec![ChoiceOption::Confirm]);
        } else {
            break;
        }
    }
    drain(&mut e_busi);
    assert_eq!(e_busi.state_core().player(0).hp, 1, "busi 经 _eloDraw (eeasy) 须挑非黑桃救活");

    // E2b: `new_wuhun` (`TaoOrTaoyuanPositive`): when Seat 0 (`game.me`) kills Seat 1 (`new_wuhun`),
    // `eeasy` gives `game.me` `Tao` on top so Seat 0 survives, while `ehard` gives non-Tao so Seat 0 dies!
    for (diff, me_survives) in [("eeasy", true), ("ehard", false)] {
        let mut e_wh = Engine::new_with_extensions(22924, 4, &[&pack_elo, &pack_d9]).unwrap();
        for s in 0..4u8 { clear_zone(&mut e_wh, s); }
        e_wh.state_core_mut().mode = GameMode::Identity {
            roles: vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor],
        };
        elo_apply_engine_config(
            &mut e_wh,
            &EloConfig { source: "noname-liuli".into(), elo_players: true, difficulty: diff.into(), time_killer: "none".into() },
            &EloQuery { online: true, mode: "identity".into(), paiwei_level: None },
        );
        e_wh.debug_grant(1, "new_wuhun");
        e_wh.debug_set_hp(1, 1);
        let wh_sha = take_card(&mut e_wh, |c| c.kind == CardKind::Sha);
        let wh_shan = take_card(&mut e_wh, |c| c.kind == CardKind::Shan);
        let wh_wuz = take_card(&mut e_wh, |c| c.kind == CardKind::Wuzhong);
        let wh_tao = take_card(&mut e_wh, |c| c.kind == CardKind::Tao);
        put_deck_top(&mut e_wh, &[wh_sha, wh_shan, wh_wuz, wh_tao]);
        e_wh.debug_set_turn(0, Phase::Play, 1);
        let kill_sha = give_kind(&mut e_wh, 0, CardKind::Sha);
        play(&mut e_wh, 0, kill_sha, vec![1]);
        drain(&mut e_wh);
        assert!(!e_wh.state_core().player(1).alive);
        assert_eq!(
            e_wh.state_core().player(0).alive,
            me_survives,
            "new_wuhun 经 _eloDraw ({diff}) 应按 TaoOrTaoyuanPositive 调顶牌"
        );
    }
}

#[test]
fn l22_fix_time_killer_additional_use_respond_discard_prompts() {
    // TK1: Verify newly added `jiedao_sha_or_give` (`chooseToUse`) & `guanshi_force_damage` (`chooseToDiscard`) trigger `_timeKiller`.
    let (_pack, mut a) = setup_l22(22923, "normal", "heavy");
    elo_set_player_scalar(a.state_core_mut(), 1, "elo_time_killer_ran_permille", 600);
    a.debug_set_turn(0, Phase::Play, 1);
    let weap1 = take_card(&mut a, |c| c.kind == CardKind::Weapon);
    a.state_core_mut().player_mut(1).equips.push(weap1);
    let jd0 = give_kind(&mut a, 0, CardKind::Jiedao);
    play(&mut a, 0, jd0, vec![1, 2]);
    let jd_ask = open(&a, "jiedao_sha_or_give");
    assert_eq!(jd_ask.observer, 1);
    assert_eq!(elo_get_player_scalar(a.state_core(), 1, "elo_time_killer_triggers"), Some(1));
    drain(&mut a);

    // Seat 1 attacks Seat 0 with Guanshi equipped; Seat 0 plays Shan -> AI Seat 1 gets `guanshi_force_damage`!
    a.debug_set_turn(1, Phase::Play, 2);
    let gs = take_card(&mut a, |c| c.kind == CardKind::Guanshi);
    a.state_core_mut().player_mut(1).equips = vec![gs];
    let _g_extra1 = give_kind(&mut a, 1, CardKind::Tao);
    let _g_extra2 = give_kind(&mut a, 1, CardKind::Tao);
    let sha1 = give_kind(&mut a, 1, CardKind::Sha);
    let shan0 = give_kind(&mut a, 0, CardKind::Shan);
    play(&mut a, 1, sha1, vec![0]);
    answer(&mut a, "respond_shan", vec![ChoiceOption::Card(shan0)]);
    let gs_ask = open(&a, "guanshi_force_damage");
    assert_eq!(gs_ask.observer, 1);
    assert_eq!(elo_get_player_scalar(a.state_core(), 1, "elo_time_killer_triggers"), Some(2));
    drain(&mut a);
}
