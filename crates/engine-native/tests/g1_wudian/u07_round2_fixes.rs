//! U07 玩点论杀 Round-2 独立复核退回项针对性回归测试（2026-11-20）。
//!
//! 覆盖全部 6 组退回根因（均经真实对局事件路径触发并对照上游 file:line）：
//! 1. `fuchou2` / `youzhan` / 物理 `youdishenru` 反击使用火杀/雷杀保留 `DamageNature`
//!    （玩点论杀/character/skill.js:569-571,445; card/card.js:187）；
//! 2. `youzhan` 视为使用 `youdishenru` 开启无懈可击窗口，被无懈不取消原杀且反无懈可恢复，
//!    并清理 `yd_pending_sha_frame` / `yd_pending_source`（character/skill.js:445）；
//! 3. 物理 `youdishenru` 0 杀/取消/闪后反伤 `DamageDealt.card == Some(yd)` 且青龙追杀重开
//!    `fuchou` 与 `g_youdishenru`（card/card.js:199; card/skill.js:16-36）；
//! 4. `jinyan` 在 `fz_new_tieji` 封印后 `hp <= 2` 仍可正常出黑色锦囊且不转化为【杀】
//!    （character/skill.js:448-507）；
//! 5. `choudu` + `diaobingqianjiang` 16 牌转正、触发 `jizhi`、`jileng_active` 排除、
//!    部分目标拒批、`versus` 模式顺逆方向弹窗/AI/存档恢复、无懈窗口无伪 `SkillToggled`
//!    （character/skill.js:586-615; card/card.js:4-142）；
//! 6. `diyjuntun` 重铸触发 `player.recast` 从牌堆顶摸任意牌而非按装备过滤发现
//!    （character/skill.js:776-811）。

use super::*;
use content_native_loader::{bridge_pack, decision34_pack9_for_coexistence};
use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::Suit;
use engine_native::content::host::WUDIAN_DIAOBING_KEY;
use engine_native::machine::LogEvent;
use engine_native::state::GameMode;
use engine_native::ChoiceKind;

fn load_u07_pack() -> ExtensionPack {
    let cards_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/wudian-upstream-noname/assets/cards.json");
    let json = std::fs::read_to_string(cards_path).expect("read wudian cards.json");
    let mut pack = bridge_pack(&json).expect("bridge wudian").pack;
    content_native_loader::attach_formal_package_upstream_rules(
        "upstream-noname",
        "wudian",
        &mut pack,
        &[],
    );
    pack
}

fn drain_nullify(e: &mut Engine) {
    while e
        .open_choice()
        .is_some_and(|c| c.spec.prompt == "nullify_window")
    {
        pass_nullify(e);
    }
}

fn setup_r2(seed: u64, players: u8) -> (Engine, ExtensionPack) {
    let pack = load_u07_pack();
    let mut e = Engine::new_with_extensions(seed, players as usize, &[&pack]).expect("u07 r2 init");
    advance_to_play(&mut e, 0);
    e.state_core_mut().skills.clear();
    for p in 0..players {
        e.debug_clear_hand(p);
    }
    (e, pack)
}

fn deck_card_by_name(e: &Engine, substr: &str, nth: usize) -> u32 {
    e.save()
        .core
        .deck
        .iter()
        .chain(e.save().core.discard.iter())
        .copied()
        .filter(|&c| e.catalog().cards[c as usize].name.contains(substr))
        .nth(nth)
        .unwrap_or_else(|| panic!("missing deck card matching '{substr}' #{nth}"))
}

fn deck_kind_suit(e: &Engine, kind: CardKind, suit: Suit) -> u32 {
    e.save()
        .core
        .deck
        .iter()
        .chain(e.save().core.discard.iter())
        .copied()
        .find(|&c| e.catalog().card_kind(c) == kind && e.catalog().cards[c as usize].suit == suit)
        .unwrap_or_else(|| panic!("missing deck card {kind:?} of {suit:?}"))
}

fn give_card(e: &mut Engine, player: u8, card: u32) {
    let mut snap = e.save();
    for p in &mut snap.core.players {
        p.hand.retain(|&c| c != card);
        p.equips.retain(|&c| c != card);
        p.judge.retain(|&c| c != card);
    }
    snap.core.deck.retain(|&c| c != card);
    snap.core.discard.retain(|&c| c != card);
    snap.core.players[player as usize].hand.push(card);
    *e = Engine::restore_with_extensions(&snap, &[&load_u07_pack()]).expect("restore give_card");
}

fn give_card_with_pack(e: &mut Engine, packs: &[&ExtensionPack], player: u8, card: u32) {
    let mut snap = e.save();
    for p in &mut snap.core.players {
        p.hand.retain(|&c| c != card);
        p.equips.retain(|&c| c != card);
        p.judge.retain(|&c| c != card);
    }
    snap.core.deck.retain(|&c| c != card);
    snap.core.discard.retain(|&c| c != card);
    snap.core.players[player as usize].hand.push(card);
    *e = Engine::restore_with_extensions(&snap, packs).expect("restore give_card_with_pack");
}

fn has_flag(e: &Engine, p: u8, key: &str) -> bool {
    e.state_core().vm_aux.flags.contains_key(&(p, key.to_string()))
}

// ===========================================================================
// Fix 1: fuchou2 / youzhan / youdishenru 反击火杀/雷杀保留 DamageNature
// ===========================================================================

#[test]
fn u07_r2_fix1_fuchou2_youzhan_youdishenru_preserve_fire_and_thunder_sha_nature() {
    // 1a-fire) fuchou2 反击火杀 -> 触发 0 号【火神】防止火伤并回 1 血（Normal/Thunder 不会触发火神）
    {
        let (mut e, _pack) = setup_r2(2026112001, 3);
        e.debug_grant(1, "fuchou");
        e.debug_grant(1, "fuchou2");
        e.debug_grant(0, "火神");
        e.debug_set_hp(0, 3);
        let sha = find_deck_kind_w(&e, CardKind::Sha);
        let h1 = deck_card_by_name(&e, "tao", 0);
        let fire_sha = deck_card_by_name(&e, "fire_sha", 0);
        give_card(&mut e, 0, sha);
        give_card(&mut e, 1, h1);
        give_card(&mut e, 1, fire_sha);

        submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        assert_eq!(e.open_choice().unwrap().spec.prompt, "wudian_fuchou_cancel");
        answer_open(&mut e, vec![ChoiceOption::Card(h1)]);
        // fuchou2 挂在 global:phaseAfter（Play 点 -> Discard -> End -> TurnPhaseAfter 触发）
        let log_start = e.log().len();
        submit(&mut e, 0, GameAction::NextPhase); // Discard
        submit(&mut e, 0, GameAction::NextPhase); // End
        submit(&mut e, 0, GameAction::NextPhase); // TurnPhaseAfter -> fuchou2
        assert_eq!(e.open_choice().unwrap().spec.prompt, "wudian_fuchou2_sha");
        answer_open(&mut e, vec![ChoiceOption::Card(fire_sha)]);
        assert_eq!(
            e.debug_hp(0),
            4,
            "fuchou2 反击火杀应保留 Fire 属性触发【火神】免疫并回复 1 血(3->4)"
        );
        assert!(
            e.log()[log_start..]
                .iter()
                .any(|ev| matches!(ev, LogEvent::DamagePrevented { target: 0, .. })),
            "fuchou2 火杀必须作为 Fire 伤害被火神防止"
        );
    }

    // 1a-thunder) fuchou2 反击雷杀 -> 对铁索连环角色传导雷电伤害
    {
        let (mut e, _pack) = setup_r2(2026112091, 3);
        e.debug_grant(1, "fuchou");
        e.debug_grant(1, "fuchou2");
        let chain1 = deck_card_by_name(&e, "chain", 0);
        let sha = find_deck_kind_w(&e, CardKind::Sha);
        let h1 = deck_card_by_name(&e, "tao", 0);
        let leisha = deck_card_by_name(&e, "leisha", 0);
        give_card(&mut e, 0, chain1);
        give_card(&mut e, 0, sha);
        give_card(&mut e, 1, h1);
        give_card(&mut e, 1, leisha);

        submit(&mut e, 0, GameAction::PlayCard { card: chain1, targets: vec![0, 2] });
        drain_nullify(&mut e);
        let hp0_before = e.debug_hp(0);
        let hp2_before = e.debug_hp(2);
        submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        answer_open(&mut e, vec![ChoiceOption::Card(h1)]);
        submit(&mut e, 0, GameAction::NextPhase);
        submit(&mut e, 0, GameAction::NextPhase);
        submit(&mut e, 0, GameAction::NextPhase);
        assert_eq!(e.open_choice().unwrap().spec.prompt, "wudian_fuchou2_sha");
        answer_open(&mut e, vec![ChoiceOption::Card(leisha)]);
        assert_eq!(e.debug_hp(0), hp0_before - 1);
        assert_eq!(
            e.debug_hp(2),
            hp2_before - 1,
            "fuchou2 反击雷杀须保留 Thunder 属性并传导至连环 2 号"
        );
    }

    // 1b) youzhan 反击雷杀 -> 对铁索连环角色传导雷电伤害
    {
        let (mut e, _pack) = setup_r2(2026112002, 3);
        e.debug_grant(1, "youzhan");
        let chain1 = deck_card_by_name(&e, "chain", 0);
        let sha = find_deck_kind_w(&e, CardKind::Sha);
        let eq = find_deck_kind_w(&e, CardKind::Zhuge);
        let leisha = deck_card_by_name(&e, "leisha", 0);
        give_card(&mut e, 0, chain1);
        give_card(&mut e, 0, sha);
        give_card(&mut e, 0, leisha);
        give_card(&mut e, 1, eq);

        // 连环 0 号和 2 号，然后 0 对 2 出杀，1 号发动诱战，0 号以雷杀反击 2 号
        submit(&mut e, 0, GameAction::PlayCard { card: chain1, targets: vec![0, 2] });
        drain_nullify(&mut e);
        assert!(e.state_core().player(0).linked && e.state_core().player(2).linked);
        let hp0_before = e.debug_hp(0);
        let hp2_before = e.debug_hp(2);
        submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![2] });
        assert_eq!(e.open_choice().unwrap().spec.prompt, "wudian_youzhan_discard");
        answer_open(&mut e, vec![ChoiceOption::Card(eq)]);
        drain_nullify(&mut e);
        assert_eq!(e.open_choice().unwrap().spec.prompt, "wudian_youdishenru_sha");
        answer_open(&mut e, vec![ChoiceOption::Card(leisha)]);
        assert_eq!(e.debug_hp(2), hp2_before - 1, "youzhan 反击雷杀直接伤害 2 号");
        assert_eq!(
            e.debug_hp(0),
            hp0_before - 1,
            "youzhan 反击雷杀须保留 Thunder 属性从而铁索传导伤害 0 号"
        );
    }

    // 1c) 物理 youdishenru 反击火杀 -> 命中回合外【拟态】造成 2 点火焰伤害
    {
        let (mut e, _pack) = setup_r2(2026112003, 3);
        e.debug_grant(1, "拟态");
        let sha = find_deck_kind_w(&e, CardKind::Sha);
        let fire_sha = deck_card_by_name(&e, "fire_sha", 0);
        let yd = deck_card_by_name(&e, "youdishenru", 0);
        give_card(&mut e, 0, sha);
        give_card(&mut e, 0, fire_sha);
        give_card(&mut e, 1, yd);

        let hp1_before = e.debug_hp(1);
        submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        assert_eq!(e.open_choice().unwrap().spec.prompt, "wudian_youdishenru_use");
        answer_open(&mut e, vec![ChoiceOption::Card(yd)]);
        drain_nullify(&mut e);
        assert_eq!(e.open_choice().unwrap().spec.prompt, "wudian_youdishenru_sha");
        answer_open(&mut e, vec![ChoiceOption::Card(fire_sha)]);
        assert_eq!(
            e.debug_hp(1),
            hp1_before - 2,
            "物理诱敌深入反击火杀命中【拟态】应造成 2 点火焰伤害"
        );
    }
}

fn find_deck_kind_w(e: &Engine, kind: CardKind) -> u32 {
    e.save()
        .core
        .deck
        .iter()
        .chain(e.save().core.discard.iter())
        .copied()
        .find(|&c| e.catalog().card_kind(c) == kind)
        .unwrap_or_else(|| panic!("missing kind {kind:?}"))
}

// ===========================================================================
// Fix 2: youzhan 视为使用诱敌深入经无懈窗口 + 反无懈 + 标志清理
// ===========================================================================

fn setup_r2_pair(seed: u64) -> Engine {
    setup_r2(seed, 3).0
}

#[test]
fn u07_r2_fix2_youzhan_virtual_youdishenru_nullify_window_and_counter_nullify() {
    // 2a) 诱战虚拟诱敌深入被无懈：原杀不被取消仍命中，且清理 yd_pending_* 标志
    {
        let (mut e, pack) = setup_r2(2026112004, 3);
        e.debug_grant(1, "youzhan");
        let sha = find_deck_kind_w(&e, CardKind::Sha);
        let eq = find_deck_kind_w(&e, CardKind::Zhuge);
        let wx = find_deck_kind_w(&e, CardKind::Nullify);
        give_card(&mut e, 0, sha);
        give_card(&mut e, 0, wx);
        give_card(&mut e, 1, eq);

        let hp2_before = e.debug_hp(2);
        submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![2] });
        assert_eq!(e.open_choice().unwrap().spec.prompt, "wudian_youzhan_discard");
        answer_open(&mut e, vec![ChoiceOption::Card(eq)]);

        // 必须开启 nullify_window 供 0 号无懈（支持存档恢复）
        let c_wx = e.open_choice().expect("virtual youdishenru must open nullify_window");
        assert_eq!(c_wx.spec.prompt, "nullify_window");
        assert_eq!(c_wx.observer, 0);
        let bytes = encode_archive(&e.save());
        e = Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &[&pack])
            .expect("restore mid youzhan nullify_window");
        answer_open(&mut e, vec![ChoiceOption::Card(wx)]);

        // 被无懈后原杀继续生效（2 号掉 1 血，不问 wudian_youdishenru_sha），标志无残留
        assert_eq!(e.debug_hp(2), hp2_before - 1);
        assert!(e.open_choice().is_none());
        assert!(!has_flag(&e, 2, "yd_pending_sha_frame"));
        assert!(!has_flag(&e, 2, "yd_pending_source"));
    }

    // 2b) 反无懈分支：0 号无懈虚拟诱敌深入、2 号再出无懈反抵消 -> resolution 不误清 yd_pending_*，原杀被取消
    {
        let (mut e, _pack) = setup_r2(2026112005, 3);
        e.debug_grant(1, "youzhan");
        let sha = find_deck_kind_w(&e, CardKind::Sha);
        let eq = find_deck_kind_w(&e, CardKind::Zhuge);
        let wx0 = deck_kind_suit(&e, CardKind::Nullify, Suit::Spade);
        let wx2 = deck_kind_suit(&e, CardKind::Nullify, Suit::Club);
        give_card(&mut e, 0, sha);
        give_card(&mut e, 0, wx0);
        give_card(&mut e, 1, eq);
        give_card(&mut e, 2, wx2);

        let hp0_before = e.debug_hp(0);
        let hp2_before = e.debug_hp(2);
        submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![2] });
        answer_open(&mut e, vec![ChoiceOption::Card(eq)]);
        // 问询队自 defender=2 起始：[2, 0, 1] -> 2 先 Pass，0 出 wx0，随后 2 再出 wx2 反无懈
        let c_wx2_first = e.open_choice().expect("defender 2 asked first in nullify_window");
        assert_eq!(c_wx2_first.spec.prompt, "nullify_window");
        assert_eq!(c_wx2_first.observer, 2);
        answer_open(&mut e, vec![ChoiceOption::Pass]);
        let c_wx0 = e.open_choice().expect("attacker 0 asked next");
        assert_eq!(c_wx0.observer, 0);
        answer_open(&mut e, vec![ChoiceOption::Card(wx0)]);
        // 2 号反无懈
        let c_wx2 = e.open_choice().expect("counter nullify prompt");
        assert_eq!(c_wx2.observer, 2);
        answer_open(&mut e, vec![ChoiceOption::Card(wx2)]);

        // 诱敌深入生效 -> 原杀对 2 号取消 -> 0 号无反击杀直接受 1 点反伤（来源为 2）
        assert!(e.open_choice().is_none(), "2b 结算完成应无残余选择");
        assert_eq!(e.debug_hp(2), hp2_before);
        assert_eq!(e.debug_hp(0), hp0_before - 1);
        assert!(!has_flag(&e, 2, "yd_pending_sha_frame"));
        assert!(!has_flag(&e, 2, "yd_pending_source"));
    }

    // 2c) 物理诱敌深入被无懈同样清理 yd_pending_*
    {
        let (mut e, _pack) = setup_r2(2026112006, 3);
        let sha = find_deck_kind_w(&e, CardKind::Sha);
        let yd = deck_card_by_name(&e, "youdishenru", 0);
        let wx = find_deck_kind_w(&e, CardKind::Nullify);
        give_card(&mut e, 0, sha);
        give_card(&mut e, 0, wx);
        give_card(&mut e, 1, yd);

        let hp1_before = e.debug_hp(1);
        submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        answer_open(&mut e, vec![ChoiceOption::Card(yd)]);
        assert_eq!(e.open_choice().unwrap().spec.prompt, "nullify_window");
        answer_open(&mut e, vec![ChoiceOption::Card(wx)]);
        assert_eq!(e.debug_hp(1), hp1_before - 1);
        assert!(!has_flag(&e, 1, "yd_pending_sha_frame"));
        assert!(!has_flag(&e, 1, "yd_pending_source"));
    }
}

// ===========================================================================
// Fix 3: 物理诱敌深入 Damage.card == Some(yd) + 青龙 Sha#2 重置标记
// ===========================================================================

#[test]
fn u07_r2_fix3_physical_youdishenru_backlash_card_and_qinglong_sha_replay() {
    // 3a/3b/3c) 物理诱敌深入 0 杀、取消、闪后停止三条反伤路径均携带 card == Some(yd)
    for branch in 0..3 {
        let (mut e, _pack) = setup_r2(2026112010 + branch, 3);
        let sha = find_deck_kind_w(&e, CardKind::Sha);
        let yd = deck_card_by_name(&e, "youdishenru", 0);
        give_card(&mut e, 0, sha);
        give_card(&mut e, 1, yd);
        if branch >= 1 {
            let c_sha = deck_kind_suit(&e, CardKind::Sha, Suit::Heart);
            give_card(&mut e, 0, c_sha);
        }
        if branch == 2 {
            let shan = find_deck_kind_w(&e, CardKind::Shan);
            let c_sha2 = deck_kind_suit(&e, CardKind::Sha, Suit::Diamond);
            give_card(&mut e, 1, shan);
            give_card(&mut e, 0, c_sha2);
        }

        let log_start = e.log().len();
        submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        answer_open(&mut e, vec![ChoiceOption::Card(yd)]);
        drain_nullify(&mut e);
        if branch == 1 {
            assert_eq!(e.open_choice().unwrap().spec.prompt, "wudian_youdishenru_sha");
            answer_open(&mut e, Vec::new());
        } else if branch == 2 {
            let ask = e.open_choice().unwrap();
            assert_eq!(ask.spec.prompt, "wudian_youdishenru_sha");
            let first_sha = ask.spec.options[0];
            answer_open(&mut e, vec![first_sha]);
            let shan_ask = e.open_choice().unwrap();
            assert_eq!(shan_ask.spec.prompt, "respond_shan");
            let shan_opt = shan_ask.spec.options[1];
            answer_open(&mut e, vec![shan_opt]);
            assert_eq!(e.open_choice().unwrap().spec.prompt, "wudian_youdishenru_sha");
            answer_open(&mut e, Vec::new());
        }
        let dmg_cards: Vec<Option<u32>> = e.log()[log_start..]
            .iter()
            .filter_map(|ev| match ev {
                LogEvent::DamageDealt { target: 0, card, .. } => Some(*card),
                _ => None,
            })
            .collect();
        assert_eq!(
            dmg_cards,
            vec![Some(yd)],
            "物理 youdishenru branch={branch} 反伤必须携带 Some(yd)"
        );
    }

    // 3d) 青龙偃月刀 Sha#2 重演必须重新允许 1 号的 fuchou 与 g_youdishenru
    let (mut e, _pack) = setup_r2(2026112015, 3);
    e.debug_grant(1, "fuchou");
    let ql = find_deck_kind_w(&e, CardKind::Qinglong);
    let sha1 = deck_kind_suit(&e, CardKind::Sha, Suit::Spade);
    let sha2 = deck_kind_suit(&e, CardKind::Sha, Suit::Club);
    let sha3 = deck_kind_suit(&e, CardKind::Sha, Suit::Diamond);
    let shan = find_deck_kind_w(&e, CardKind::Shan);
    let h_extra = deck_card_by_name(&e, "tao", 0);
    let yd = deck_card_by_name(&e, "youdishenru", 0);
    give_card(&mut e, 0, ql);
    give_card(&mut e, 0, sha1);
    give_card(&mut e, 0, sha2);
    give_card(&mut e, 0, sha3);
    give_card(&mut e, 1, shan);
    give_card(&mut e, 1, h_extra);
    give_card(&mut e, 1, yd);

    submit(&mut e, 0, GameAction::PlayCard { card: ql, targets: vec![] });
    submit(&mut e, 0, GameAction::PlayCard { card: sha1, targets: vec![1] });
    // Sha#1：先婉拒 fuchou 与 g_youdishenru，再用闪抵消
    assert_eq!(e.open_choice().unwrap().spec.prompt, "wudian_fuchou_cancel");
    answer_open(&mut e, Vec::new());
    assert_eq!(e.open_choice().unwrap().spec.prompt, "wudian_youdishenru_use");
    answer_open(&mut e, Vec::new());
    assert_eq!(e.open_choice().unwrap().spec.prompt, "respond_shan");
    answer_open(&mut e, vec![ChoiceOption::Card(shan)]);

    // 0 号发动青龙对 1 号再出 sha2
    let ql_ask = e.open_choice().expect("qinglong replay prompt");
    assert_eq!(ql_ask.spec.prompt, "qinglong_replay_sha");
    answer_open(&mut e, vec![ChoiceOption::Card(sha2)]);

    // Sha#2：fuchou 与 g_youdishenru 必须再次触发
    assert_eq!(
        e.open_choice().unwrap().spec.prompt,
        "wudian_fuchou_cancel",
        "青龙 Sha#2 必须重置 asking_fuchou 并再次询问 fuchou"
    );
    answer_open(&mut e, Vec::new());
    assert_eq!(
        e.open_choice().unwrap().spec.prompt,
        "wudian_youdishenru_use",
        "青龙 Sha#2 必须清除 yd_tried 并再次询问 g_youdishenru"
    );
    answer_open(&mut e, vec![ChoiceOption::Card(yd)]);
    drain_nullify(&mut e);
    answer_open(&mut e, Vec::new());
}

// ===========================================================================
// Fix 4: jinyan 在 fz_new_tieji 封印下（hp <= 2）可用黑色锦囊且不转作杀
// ===========================================================================

#[test]
fn u07_r2_fix4_jinyan_tieji_seal_allows_black_trick_disables_conversion() {
    let wudian_pack = load_u07_pack();
    let d34_pack = decision34_pack9_for_coexistence(&[&wudian_pack]);
    let packs = [&wudian_pack, &d34_pack];
    let mut e = Engine::new_with_extensions(2026112020, 3, &packs).expect("init fix4");
    advance_to_play(&mut e, 0);
    e.state_core_mut().skills.clear();
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "fz_new_tieji");
    e.debug_grant(0, "yxs_fanji");
    e.debug_grant(1, "jinyan");

    let sha0 = deck_kind_suit(&e, CardKind::Sha, Suit::Spade);
    let sha1 = deck_kind_suit(&e, CardKind::Sha, Suit::Club);
    let sha_counter = deck_kind_suit(&e, CardKind::Sha, Suit::Heart);
    let wuzhong1 = e
        .save()
        .core
        .deck
        .iter()
        .copied()
        .filter(|&c| e.catalog().card_kind(c) == CardKind::Wuzhong)
        .nth(0)
        .unwrap();
    let wuzhong2 = e
        .save()
        .core
        .deck
        .iter()
        .copied()
        .filter(|&c| e.catalog().card_kind(c) == CardKind::Wuzhong)
        .nth(1)
        .unwrap();
    let wx_black = deck_kind_suit(&e, CardKind::Nullify, Suit::Spade);
    let nanman_spade = deck_kind_suit(&e, CardKind::SavageAttack, Suit::Spade);
    let le_spade = deck_kind_suit(&e, CardKind::Le, Suit::Spade);
    let nanman_club = deck_kind_suit(&e, CardKind::SavageAttack, Suit::Club);

    // 4a) 回合外无懈面：1 号 hp==2 未封印时黑无懈被禁言屏蔽；0 号用 fz_new_tieji 封印 1 号后黑无懈解禁
    e.debug_set_hp(1, 2);
    give_card_with_pack(&mut e, &packs, 0, wuzhong1);
    give_card_with_pack(&mut e, &packs, 1, wx_black);
    let out_w1 = submit(&mut e, 0, GameAction::PlayCard { card: wuzhong1, targets: vec![] });
    assert!(matches!(out_w1, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out_w1:?}");
    assert!(
        e.open_choice().is_none(),
        "未封印时 hp<=2 的 jinyan 须屏蔽黑色【无懈可击】响应"
    );

    e.debug_set_hp(1, 3);
    give_card_with_pack(&mut e, &packs, 0, sha0);
    give_card_with_pack(&mut e, &packs, 0, wuzhong2);
    let out_s0 = submit(&mut e, 0, GameAction::PlayCard { card: sha0, targets: vec![1] });
    assert!(matches!(out_s0, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out_s0:?}");
    assert_eq!(e.open_choice().unwrap().spec.prompt, "fz_new_tieji:confirm");
    answer_open(&mut e, vec![ChoiceOption::Confirm]);
    if e.open_choice().is_some_and(|c| c.spec.prompt == "fz_new_tieji:target_discard") {
        answer_open(&mut e, Vec::new());
    }
    if e.open_choice().is_some_and(|c| c.spec.prompt == "respond_shan") {
        answer_open(&mut e, vec![ChoiceOption::Pass]);
    }
    assert_eq!(e.debug_hp(1), 2, "open={:?} log={:?}", e.open_choice(), &e.log()[e.log().len().saturating_sub(6)..]);
    let out_w2 = submit(&mut e, 0, GameAction::PlayCard { card: wuzhong2, targets: vec![] });
    assert!(matches!(out_w2, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "wuzhong2 rejected: {out_w2:?}");
    let wx_ask = e.open_choice().unwrap_or_else(|| panic!(
        "tieji 封印下 jinyan 失效，应开放黑无懈响应; hand1={:?} sealed={} hp1={} log={:?}",
        e.debug_hand(1),
        has_flag(&e, 1, "fz_new_tieji_target_sealed"),
        e.debug_hp(1),
        &e.log()[e.log().len().saturating_sub(8)..]
    ));
    assert_eq!(wx_ask.spec.prompt, "nullify_window");
    assert_eq!(wx_ask.observer, 1);
    answer_open(&mut e, vec![ChoiceOption::Card(wx_black)]);

    // 4b) 回合内 PlayCard 面：切到 1 号出牌阶段（hp=3），1 号杀 0 号触发 0 号反击+fz_new_tieji 封印 1 号并将 hp 降至 2
    advance_to_play(&mut e, 1);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 3);
    give_card_with_pack(&mut e, &packs, 0, sha_counter);
    give_card_with_pack(&mut e, &packs, 1, sha1);
    give_card_with_pack(&mut e, &packs, 1, nanman_spade);
    give_card_with_pack(&mut e, &packs, 1, le_spade);
    give_card_with_pack(&mut e, &packs, 1, nanman_club);

    submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![0] });
    if e
        .open_choice()
        .is_some_and(|c| c.spec.prompt == "trigger_confirm" || c.spec.prompt == "yxs_fanji")
    {
        answer_open(&mut e, vec![ChoiceOption::Confirm]);
    }
    assert_eq!(e.open_choice().unwrap().spec.prompt, "yxs_fanji_sha");
    answer_open(&mut e, vec![ChoiceOption::Card(sha_counter)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "fz_new_tieji:confirm");
    answer_open(&mut e, vec![ChoiceOption::Confirm]);
    if e.open_choice().is_some_and(|c| c.spec.prompt == "fz_new_tieji:target_discard") {
        answer_open(&mut e, Vec::new());
    }
    if e.open_choice().is_some_and(|c| c.spec.prompt == "respond_shan") {
        answer_open(&mut e, vec![ChoiceOption::Pass]);
    }
    assert_eq!(e.debug_hp(1), 2, "反击+铁骑后 1 号 hp 降至 2 且已封印");
    assert_eq!(e.turn().current, 1);
    assert_eq!(e.turn().phase, Phase::Play);

    // 此时 1 号 hp==2 且被铁骑封印：可直接出黑南蛮入侵及黑乐不思蜀（入判定区而非视为杀）
    let out_nm = submit(&mut e, 1, GameAction::PlayCard { card: nanman_spade, targets: vec![] });
    assert!(
        matches!(out_nm, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "铁骑封印下 jinyan 失效，黑南蛮入侵必须可正常使用: {out_nm:?}"
    );
    while e.open_choice().is_some() {
        let opts = e.open_choice().unwrap().spec.options.clone();
        answer_open(&mut e, vec![opts[0]]);
    }
    let out_le = submit(&mut e, 1, GameAction::PlayCard { card: le_spade, targets: vec![0] });
    assert!(
        matches!(out_le, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "铁骑封印下黑乐不思蜀不得视为杀或禁牌: {out_le:?}"
    );
    assert!(
        e.state_core().player(0).judge.contains(&le_spade),
        "黑乐不思蜀必须以真乐置入 0 号判定区而非转化为杀"
    );

    // 结束 1 号本回合（TurnEnded 清除铁骑封印）后再轮到 1 号（仍 hp<=2），黑南蛮再度受禁 CardLimit
    submit(&mut e, 1, GameAction::NextPhase);
    advance_to_play(&mut e, 1);
    if e.debug_hp(1) > 2 {
        e.debug_set_hp(1, 2);
    }
    give_card_with_pack(&mut e, &packs, 1, nanman_club);
    let out_blocked = submit(&mut e, 1, GameAction::PlayCard { card: nanman_club, targets: vec![] });
    assert!(
        matches!(out_blocked, CommandOutcome::Rejected(RejectReason::CardLimit)),
        "解除铁骑封印后 hp<=2 黑锦囊须恢复 CardLimit 禁用: {out_blocked:?}"
    );
}

// ===========================================================================
// Fix 5: choudu + diaobingqianjiang 完整等价回归
// ===========================================================================

#[test]
fn u07_r2_fix5_choudu_and_diaobingqianjiang_upstream_equivalence() {
    // 5.1) wudian-upstream-noname 加载 16 牌，含红桃4/方块4 调兵遣将，deferred 为空
    let cards_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/wudian-upstream-noname/assets/cards.json");
    let raw = std::fs::read_to_string(cards_path).unwrap();
    let bridged = bridge_pack(&raw).expect("bridge wudian");
    assert_eq!(bridged.pack.card_count(), 16);
    assert!(bridged.deferred.is_empty());
    let db_defs: Vec<_> = bridged
        .pack
        .cards
        .iter()
        .filter(|c| c.name.contains("diaobingqianjiang"))
        .collect();
    assert_eq!(db_defs.len(), 2);
    assert_eq!((db_defs[0].suit, db_defs[0].rank), (Suit::Heart, 4));
    assert_eq!((db_defs[1].suit, db_defs[1].rank), (Suit::Diamond, 4));

    // 5.2 & 5.3) choudu 发射 CardUsed(Wuzhong) 触发 1 号集智、wudian_choudu_targets 排除 jileng_active、
    //             且实体 diaobingqianjiang 部分目标被 IllegalTargets 拒绝
    {
        let (mut e, _pack) = setup_r2(2026112030, 3);
        let _ = setup_r2_pair;
        e.debug_grant(0, "choudu");
        e.debug_grant(1, "集智");
        let choudu_def = e.catalog().skill_by_name("choudu").unwrap();
        let h0 = find_deck_kind_w(&e, CardKind::Tao);
        let h1 = deck_card_by_name(&e, "sha", 0);
        let h2 = deck_card_by_name(&e, "shan", 0);
        let db = deck_card_by_name(&e, "diaobingqianjiang", 0);
        give_card(&mut e, 0, h0);
        give_card(&mut e, 0, db);
        give_card(&mut e, 1, h1);
        give_card(&mut e, 2, h2);

        // 5.3 部分目标 [0, 1]（2 号有手牌却遗漏）必拒 IllegalTargets
        let partial = submit(&mut e, 0, GameAction::PlayCard { card: db, targets: vec![0, 1] });
        assert!(matches!(partial, CommandOutcome::Rejected(RejectReason::IllegalTargets)));

        // jileng_active 角色从 wudian_choudu_target 选项中排除（真实激活流程检查），之后取消重开
        e.state_core_mut().players[2].jileng_active = true;
        submit(&mut e, 0, GameAction::ActivateSkill { skill: choudu_def });
        assert_eq!(e.open_choice().unwrap().spec.prompt, "wudian_choudu_card");
        answer_open(&mut e, vec![ChoiceOption::Card(h0)]);
        let tgt_ask = e.open_choice().expect("wudian_choudu_target");
        assert_eq!(tgt_ask.spec.prompt, "wudian_choudu_target");
        assert!(
            !tgt_ask.spec.options.contains(&ChoiceOption::Player(2)),
            "jileng_active 角色不可出现在 wudian_choudu_target 选项中: {:?}",
            tgt_ask.spec.options
        );
        answer_open(&mut e, Vec::new()); // 取消筹度，未消耗 once 限次
        e.state_core_mut().players[2].jileng_active = false;

        // 5.2 筹度指定 1 号 -> 必须产生 VirtualCardPlayed(Wuzhong, player=1) 并触发 1 号集智
        let log_start = e.log().len();
        submit(&mut e, 0, GameAction::ActivateSkill { skill: choudu_def });
        assert_eq!(e.open_choice().unwrap().spec.prompt, "wudian_choudu_card");
        answer_open(&mut e, vec![ChoiceOption::Card(h0)]);
        assert_eq!(e.open_choice().unwrap().spec.prompt, "wudian_choudu_target");
        answer_open(&mut e, vec![ChoiceOption::Player(1)]);
        if e.open_choice().is_some_and(|c| c.spec.prompt == "集智" || c.spec.prompt == "jizhi") {
            answer_open(&mut e, vec![ChoiceOption::Confirm]);
        }
        assert!(
            e.log()[log_start..].iter().any(|ev| matches!(
                ev,
                LogEvent::VirtualCardPlayed { kind: CardKind::Wuzhong, player: 1, .. }
            )),
            "choudu 必须发射真实 CardUsed(Wuzhong) 日志事件"
        );
        assert!(
            e.log()[log_start..].iter().any(|ev| matches!(
                ev,
                LogEvent::SkillTriggered { skill: "集智", owner: 1, .. }
            )),
            "choudu 发射的虚拟调兵遣将必须触发 1 号集智"
        );
    }

    // 5.4 & 5.5) GameMode::Versus 方向弹窗（Tag(0)=顺时针 [0,3,2,1] vs Tag(1)=逆时针 [0,1,2,3]）、
    //             AI 按下家阵营抉择、无伪 SkillToggled、mid-NullifyWindow 存档恢复
    for dir_tag in [0u8, 1u8] {
        let (mut e, pack) = setup_r2(2026112040 + u64::from(dir_tag), 4);
        e.state_core_mut().mode = GameMode::Versus {
            teams: if dir_tag == 0 {
                vec![vec![0, 2], vec![1, 3]]
            } else {
                vec![vec![0, 1], vec![2, 3]]
            },
            substitutes: vec![vec![], vec![]],
        };
        let db = deck_card_by_name(&e, "diaobingqianjiang", 0);
        let wx3 = find_deck_kind_w(&e, CardKind::Nullify);
        let h1 = deck_card_by_name(&e, "tao", 0);
        let h2 = deck_card_by_name(&e, "sha", 0);
        give_card(&mut e, 0, db);
        give_card(&mut e, 1, h1);
        give_card(&mut e, 2, h2);
        give_card(&mut e, 3, wx3);

        let log_start = e.log().len();
        submit(&mut e, 0, GameAction::PlayCard { card: db, targets: Vec::new() });
        let dir_ask = e.open_choice().expect("versus mode must ask wudian_diaobing_dir");
        assert_eq!(dir_ask.spec.prompt, "wudian_diaobing_dir");
        assert_eq!(dir_ask.spec.kind, ChoiceKind::PickTag);
        assert!(!dir_ask.spec.allow_cancel);
        // 验证 mid-wudian_diaobing_dir 存档恢复 + AI 取向
        let planner = BoundedPlanner::new(8);
        let GameAction::Answer { selection: ai_sel, .. } = e.plan_action(0, &planner).action else {
            panic!("expected AI Answer for wudian_diaobing_dir");
        };
        assert_eq!(
            ai_sel,
            vec![ChoiceOption::Tag(dir_tag)],
            "Versus AI 下家敌对(dir_tag=0)应选顺时针Tag(0)，同队(dir_tag=1)应选逆时针Tag(1)"
        );
        let dir_bytes = encode_archive(&e.save());
        e = Engine::restore_with_extensions(&decode_archive(&dir_bytes).unwrap(), &[&pack])
            .expect("restore mid wudian_diaobing_dir");
        answer_open(&mut e, vec![ChoiceOption::Tag(dir_tag)]);

        let mut pick_observers = Vec::new();
        while let Some(c) = e.open_choice().cloned() {
            match c.spec.prompt {
                "nullify_window" => {
                    let bytes = encode_archive(&e.save());
                    e = Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &[&pack])
                        .expect("restore mid NO_FRAME nullify_window");
                    answer_open(&mut e, vec![ChoiceOption::Pass]);
                }
                "wudian_diaobing_pick" => {
                    pick_observers.push(c.observer);
                    answer_open(&mut e, Vec::new());
                }
                "wudian_diaobing_top" => {
                    answer_open(&mut e, Vec::new());
                }
                other => panic!("unexpected prompt {other}"),
            }
        }
        let expected_order = if dir_tag == 0 {
            vec![0, 3, 2, 1]
        } else {
            vec![0, 1, 2, 3]
        };
        assert_eq!(pick_observers, expected_order);
        assert!(
            !e.log()[log_start..]
                .iter()
                .any(|ev| matches!(ev, LogEvent::SkillToggled { .. })),
            "diaobingqianjiang 逐目标无懈不得插入伪 SkillToggled"
        );
        assert!(!has_flag(&e, 0, "diaobing_wuxie_active"));
        let _ = WUDIAN_DIAOBING_KEY;
    }
}

// ===========================================================================
// Fix 6: diyjuntun 重铸装备后摸牌堆顶任意牌（含非装备桃）
// ===========================================================================

#[test]
fn u07_r2_fix6_diyjuntun_recast_draws_top_non_equip() {
    let (mut e, _pack) = setup_r2(2026112050, 3);
    e.debug_grant(0, "diyjuntun");
    let diyjuntun_def = e.catalog().skill_by_name("diyjuntun").unwrap();
    let eq = find_deck_kind_w(&e, CardKind::Zhuge);
    give_card(&mut e, 0, eq);
    let top_tao = e.debug_set_deck_top("tao");

    submit(&mut e, 0, GameAction::ActivateSkill { skill: diyjuntun_def });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "wudian_diyjuntun_recast");
    answer_open(&mut e, vec![ChoiceOption::Card(eq)]);
    assert_eq!(
        e.state_core().player(0).hand,
        vec![top_tao],
        "diyjuntun 重铸装备必须走 player.recast 摸起牌堆顶非装备牌【桃】"
    );
    assert!(e.state_core().discard.contains(&eq));
}

// ===========================================================================
// Round-3 Fix Verification (diaobingqianjiang CardsLost / youzhan Duoqi / diyjuntun Equip Lose)
// ===========================================================================

#[test]
fn u07_r3_diaobing_cards_lost_settles_before_gain_next_nullify_and_content_after() {
    let wudian_pack = load_u07_pack();
    let d34_pack = decision34_pack9_for_coexistence(&[&wudian_pack]);
    let packs = [&wudian_pack, &d34_pack];

    // Part A: 单手牌目标 0（持 dddzhengjun，2 号 0 手牌，1 号持无懈）替换唯一手牌：
    // CardsLost 期间 0 号尚未获得 market_c（hand.is_empty() == 0 == 2 号手牌数）且早于 1 号无懈窗触发。
    {
        let mut e = Engine::new_with_extensions(2026112101, 3, &packs).expect("init r3 diaobing A");
        advance_to_play(&mut e, 0);
        e.state_core_mut().skills.clear();
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        e.debug_grant(0, "dddzhengjun");
        let db = deck_card_by_name(&e, "diaobingqianjiang", 0);
        let h1 = deck_card_by_name(&e, "tao", 0);
        let wx1 = find_deck_kind_w(&e, CardKind::Nullify);
        give_card_with_pack(&mut e, &packs, 0, db);
        give_card_with_pack(&mut e, &packs, 1, h1);
        give_card_with_pack(&mut e, &packs, 1, wx1);

        submit(&mut e, 0, GameAction::PlayCard { card: db, targets: Vec::new() });
        // 0 号摸 1 牌：上游 dddzhengjun 允许自身相等，先取消这次摸牌触发（不消耗 hs 次数）再进无懈窗
        if e.open_choice().is_some_and(|c| c.spec.prompt == "dddzhengjun_hs_choice") {
            answer_open(&mut e, Vec::new());
        }
        // 0 号摸 1 牌后对 0 号开无懈窗（1 号 Pass）
        assert_eq!(e.open_choice().unwrap().spec.prompt, "nullify_window");
        answer_open(&mut e, vec![ChoiceOption::Pass]);
        let pick0 = e.open_choice().expect("wudian_diaobing_pick for 0");
        assert_eq!(pick0.observer, 0);
        let market_0 = pick0.spec.options[0];
        let ChoiceOption::Card(market_0_id) = market_0 else { panic!() };
        answer_open(&mut e, vec![market_0]);
        let swap0 = e.open_choice().expect("wudian_diaobing_swap for 0");
        let sole_card = swap0.spec.options[0];
        answer_open(&mut e, vec![sole_card]);

        // 此时 CardsLost 正处于结算中，且早于 0 号获得 market_0 与 1 号无懈窗！
        let zj_ask = e
            .open_choice()
            .expect("dddzhengjun_hs_choice must open during CardsLost before gain/next NullifyWindow");
        assert_eq!(zj_ask.spec.prompt, "dddzhengjun_hs_choice");
        assert_eq!(zj_ask.observer, 0);
        assert!(
            e.debug_hand(0).is_empty(),
            "CardsLost 结算期间 market_0 不得提前入 0 号手牌: {:?}",
            e.debug_hand(0)
        );
        // 挂起于 CardsLost（WudianDiaobingCard.cards 含全量亮出牌+hand_c）存档恢复全区守恒
        let raw = encode_archive(&e.save());
        e = Engine::restore_with_extensions(&decode_archive(&raw).unwrap(), &packs)
            .expect("restore mid CardsLost inside WudianDiaobingCard");
        assert_eq!(e.open_choice().unwrap().spec.prompt, "dddzhengjun_hs_choice");
        answer_open(&mut e, Vec::new());

        // CardsLost 结算完：0 号现已获得 market_0，然后才开启下一目标 1 号的无懈窗
        assert_eq!(e.debug_hand(0), vec![market_0_id]);
        let wx_next = e.open_choice().expect("target 1 nullify_window after CardsLost");
        assert_eq!(wx_next.spec.prompt, "nullify_window");
        assert_eq!(wx_next.observer, 1);
    }

    // Part B: 末位目标替换手牌 -> CardsLost 必须早于 wudian_diaobing_top（contentAfter）结算
    {
        let mut e = Engine::new_with_extensions(2026112102, 3, &packs).expect("init r3 diaobing B");
        advance_to_play(&mut e, 0);
        e.state_core_mut().skills.clear();
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        e.debug_grant(0, "dddzhengjun");
        let db = deck_card_by_name(&e, "diaobingqianjiang", 0);
        give_card_with_pack(&mut e, &packs, 0, db);

        // 1 号/2 号均 0 手牌，0 号为唯一也是末位目标
        submit(&mut e, 0, GameAction::PlayCard { card: db, targets: Vec::new() });
        if e.open_choice().is_some_and(|c| c.spec.prompt == "dddzhengjun_hs_choice") {
            answer_open(&mut e, Vec::new());
        }
        let pick0 = e.open_choice().unwrap();
        let market_0 = pick0.spec.options[0];
        answer_open(&mut e, vec![market_0]);
        let swap0 = e.open_choice().unwrap();
        let sole = swap0.spec.options[0];
        answer_open(&mut e, vec![sole]);

        assert_eq!(
            e.open_choice().unwrap().spec.prompt,
            "dddzhengjun_hs_choice",
            "末位目标换牌 CardsLost 必须先于 wudian_diaobing_top 触发"
        );
        answer_open(&mut e, Vec::new());
        assert_eq!(
            e.open_choice().unwrap().spec.prompt,
            "wudian_diaobing_top",
            "CardsLost 结算完成后才进入 contentAfter 置于牌堆顶询问"
        );
        answer_open(&mut e, Vec::new());
        assert!(e.open_choice().is_none());
    }
}

#[test]
fn u07_r3_youzhan_discard_settles_duoqi_before_virtual_youdishenru() {
    let (mut e, _pack) = setup_r2(2026112103, 3);
    e.debug_grant(1, "youzhan");
    e.debug_grant(2, "duoqi");
    let sha = find_deck_kind_w(&e, CardKind::Sha);
    let eq = find_deck_kind_w(&e, CardKind::Zhuge);
    let zc = find_deck_kind_w(&e, CardKind::Tao);
    let wx0 = find_deck_kind_w(&e, CardKind::Nullify);
    e.state_core_mut().deck.retain(|&c| c != zc);
    e.state_core_mut().player_mut(2).zhucheng = vec![zc];
    give_card(&mut e, 0, sha);
    give_card(&mut e, 0, wx0);
    give_card(&mut e, 1, eq);

    submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![2] });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "wudian_youzhan_discard");
    answer_open(&mut e, vec![ChoiceOption::Card(eq)]);

    // 上游 youzhan 先完成 cost 弃牌 -> 触发 2 号夺气，再进入虚拟诱敌深入无懈窗口
    let duoqi_ask = e
        .open_choice()
        .expect("wudian_duoqi_card must open before virtual youdishenru");
    assert_eq!(duoqi_ask.spec.prompt, "wudian_duoqi_card");
    assert_eq!(duoqi_ask.observer, 2);
    answer_open(&mut e, Vec::new()); // 取消夺气

    let wx_ask = e
        .open_choice()
        .expect("virtual youdishenru nullify_window must follow after duoqi settles");
    assert_eq!(wx_ask.spec.prompt, "nullify_window");
    assert_eq!(wx_ask.observer, 0);
    answer_open(&mut e, vec![ChoiceOption::Pass]);
    // 0 号无第二张杀 -> 取消原杀对 2 号并受 1 点反伤
    assert!(e.open_choice().is_none());
    assert_eq!(e.debug_hp(0), 3);
}

#[test]
fn u07_r3_diyjuntun_equip_recast_emits_cards_lost_and_single_discard_log() {
    let wudian_pack = load_u07_pack();
    let d34_pack = decision34_pack9_for_coexistence(&[&wudian_pack]);
    let packs = [&wudian_pack, &d34_pack];
    let mut e = Engine::new_with_extensions(2026112104, 3, &packs).expect("init r3 diyjuntun");
    advance_to_play(&mut e, 0);
    e.state_core_mut().skills.clear();
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "diyjuntun");
    let diyjuntun_def = e.catalog().skill_by_name("diyjuntun").unwrap();

    // 1) 装备区重铸：清空 declared_slots、触发 Equip CardsLost（先于摸牌）、且仅记 1 条 CardDiscarded 日志
    let eq_worn = find_deck_kind_w(&e, CardKind::Zhuge);
    give_card_with_pack(&mut e, &packs, 0, eq_worn);
    submit(&mut e, 0, GameAction::PlayCard { card: eq_worn, targets: Vec::new() });
    e.debug_grant(0, "dddzhengjun");
    let top_tao = e.debug_set_deck_top("tao");
    assert!(e.state_core().player(0).equips.contains(&eq_worn));

    let log_worn = e.log().len();
    submit(&mut e, 0, GameAction::ActivateSkill { skill: diyjuntun_def });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "wudian_diyjuntun_recast");
    answer_open(&mut e, vec![ChoiceOption::Card(eq_worn)]);

    let es_ask = e
        .open_choice()
        .expect("dddzhengjun_es_confirm must fire on diyjuntun Equip CardsLost before recast draw");
    assert_eq!(es_ask.spec.prompt, "dddzhengjun_es_confirm");
    assert_eq!(es_ask.observer, 0);
    assert!(
        e.debug_hand(0).is_empty(),
        "CardsLost 结算完成前 recast 不得抢先摸 top_tao: {:?}",
        e.debug_hand(0)
    );
    assert!(!e.state_core().declared_slots.contains_key(&eq_worn));
    assert!(e.state_core().player(0).equips.is_empty());
    assert!(e.state_core().discard.contains(&eq_worn));
    answer_open(&mut e, Vec::new());

    assert_eq!(e.debug_hand(0), vec![top_tao]);
    let worn_discard_logs = e.log()[log_worn..]
        .iter()
        .filter(|ev| matches!(ev, LogEvent::CardDiscarded { player: 0, cards } if cards == &vec![eq_worn]))
        .count();
    assert_eq!(worn_discard_logs, 1, "装备区重铸不得缺失或重复记 CardDiscarded");
    if e.open_choice().is_some_and(|c| c.spec.prompt == "dddzhengjun_hs_choice") {
        answer_open(&mut e, Vec::new());
    }
    if let Some(zj_def) = e.catalog().skill_by_name("dddzhengjun") {
        e.state_core_mut().skills.remove(&(0, zj_def));
    }

    // 2) 手牌区装备重铸：仅记 1 条 CardDiscarded（修复重复 log）
    let eq_hand = deck_card_by_name(&e, "sanjian", 0);
    give_card_with_pack(&mut e, &packs, 0, eq_hand);
    let log_hand = e.log().len();
    submit(&mut e, 0, GameAction::ActivateSkill { skill: diyjuntun_def });
    answer_open(&mut e, vec![ChoiceOption::Card(eq_hand)]);
    let hand_discard_logs = e.log()[log_hand..]
        .iter()
        .filter(|ev| matches!(ev, LogEvent::CardDiscarded { player: 0, cards } if cards == &vec![eq_hand]))
        .count();
    assert_eq!(hand_discard_logs, 1, "手牌区重铸不得重复记录 CardDiscarded 日志");
}
