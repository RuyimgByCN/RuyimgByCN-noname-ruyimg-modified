//! `card/gwent/skill.js`（11 技）、`character/gwent/skill.js`（`_gainspell` 等）与
//! `gw_dieyi_equip1..5` / `gw_dieyi_judge`（`nojudge: true` + `gw_dieyi` 累计）测试。

use super::common::*;
use engine_native::catalog::CardKind;
use engine_native::{ChoiceOption, CommandOutcome, GameAction};

#[test]
fn u05_b5a_dieyi_no_wuxingpan_judge_nojudge_and_dismantle() {
    let (v, e0) = setup_gwent_locked(5540);

    // 1. `gw_dieyi_equip5` 不得继承 `wuxingpan_skill` (`shahai_attach.rs`)
    let equip5 = find_card(&e0, "gw_dieyi_equip5");
    assert!(
        e0.catalog().cards[equip5 as usize].equip_skills.is_empty(),
        "gw_dieyi_equip5 不得带 wuxingpan_skill"
    );

    // 2. `gw_dieyi_judge` (`nojudge: true`): 到其判定阶段直接弃置（不翻牌堆判定）、触发 CardsLost 并累计 `gw_dieyi`
    let dieyi_j = find_card(&e0, "gw_dieyi_judge");
    let sha = card_of_kind(&e0, CardKind::Sha, None);
    let mut e = inject_judge(&e0, &v, dieyi_j, 1);
    e = inject_hand(&e, &v, &[sha], 1);
    let deck_before = e.save().core.deck.len();
    advance_to_seat_play(&mut e, 1);
    assert_eq!(
        e.save().core.deck.len(),
        deck_before - 2,
        "nojudge=true 仅摸牌阶段摸 2 张，判定阶段不消耗牌堆翻判定牌"
    );
    assert!(e.save().core.players[1].judge.is_empty());
    assert!(is_vanished(&e, dieyi_j), "gw_dieyi_judge vanish=true 入 u05_vanished");
    let dieyi_def = e.catalog().skill_by_name("gw_dieyi").unwrap();
    assert!(e.save().core.skills.contains_key(&(1, dieyi_def)), "离开判定区授 gw_dieyi");
    let h1_before_end = e.debug_hand(1).len();
    advance_past_turn(&mut e, 1);
    assert_eq!(e.debug_hand(1).len(), h1_before_end - 1, "回合结束触发 gw_dieyi(1层) 随机弃 1 并移除自身");
    assert!(!e.save().core.skills.contains_key(&(1, dieyi_def)));
    assert_zone_conservation(&e);

    // 3. 过河拆桥拆掉 `gw_dieyi_judge` 不得触发 `gw_dieyi`（仅 phaseJudge 结算 `effect()` 时触发）
    let chai = card_of_kind(&e0, CardKind::Dismantle, None);
    let mut e = inject_hand(&e0, &v, &[chai], 0);
    e = inject_judge(&e, &v, dieyi_j, 1);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: chai, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert!(e.save().core.players[1].judge.is_empty());
    assert!(
        !e.save().core.skills.contains_key(&(1, dieyi_def)),
        "拆 gw_dieyi_judge 不得触发 gw_dieyi（仅 phaseJudge effect() 触发）"
    );
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5a_baobaoshu_nuhaifengbao_leizhoushu_and_gainspell() {
    let (v, e0) = setup_gwent_locked(5541);
    let sha = card_of_kind(&e0, CardKind::Sha, None);
    let yanzi = find_card(&e0, "gw_yanziyaoshui");

    // 1. 雹暴术 (`card/gwent/skill.js:76`): 使用基本/锦囊强制弃 1 `he`（含 save->restore）；使用 Gwent `spell` 不触发
    let baobao = find_card(&e0, "gw_baobaoshu");
    let mut e = inject_hand(&e0, &v, &[baobao], 0);
    e = inject_hand(&e, &v, &[sha, yanzi], 1);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: baobao, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    advance_to_seat_play(&mut e, 1);
    // 1 号先用 gw_yanziyaoshui（type: spell），不得触发雹暴术弃牌
    assert!(matches!(
        submit(&mut e, 1, GameAction::PlayCard { card: yanzi, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert!(
        e.open_choice().is_none_or(|c| c.spec.prompt != "gw_baobaoshu_discard"),
        "type: spell 非基本/锦囊不触发雹暴术"
    );
    // 1 号使用基本牌【杀】，触发强制弃 1
    assert!(matches!(
        submit(&mut e, 1, GameAction::PlayCard { card: sha, targets: vec![0] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    if let Some(c) = e.open_choice().cloned() {
        if c.spec.prompt == "gw_baobaoshu_discard" {
            let pick = c.spec.options[0];
            let (mut e_r, _) = save_restore_answer(e, &v, 1, vec![pick]);
            drain_default(&mut e_r);
            e = e_r;
        } else {
            drain_default(&mut e);
        }
    }
    assert!(e.debug_hand(1).len() <= 2, "雹暴术用基本牌后弃 1");
    assert_zone_conservation(&e);

    // 2. 怒海风暴 (`card/gwent/skill.js:116`): 回合结束随机弃 1 且 rounds=2 -> 1 -> 0 移除
    let nuhai = find_card(&e0, "gw_nuhaifengbao");
    let shas = find_cards_matching(&e0, |d| d.kind == CardKind::Sha, 3);
    let mut e = inject_hand(&e0, &v, &[nuhai], 0);
    e = inject_hand(&e, &v, &shas[..2], 1);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: nuhai, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    let nuhai_def = e.catalog().skill_by_name("gw_nuhaifengbao").unwrap();
    assert_eq!(e.save().core.skills.get(&(1, nuhai_def)).unwrap().rounds, 2);
    advance_to_seat_play(&mut e, 1);
    let h1_play = e.debug_hand(1).len(); // 起手 2 + 摸牌阶段 2 = 4
    advance_past_turn(&mut e, 1);
    assert_eq!(e.debug_hand(1).len(), h1_play - 1, "怒海风暴第 1 次回合结束随机弃 1");
    assert_eq!(e.save().core.skills.get(&(1, nuhai_def)).unwrap().rounds, 1);
    assert_zone_conservation(&e);

    // 3. `_gainspell` (`card/gwent/skill.js:353`): 挂在全局技中且于摸牌阶段开始初始化计数并可将金/银法术置顶
    assert!(
        e0.catalog().skill_by_name("_gainspell").is_some_and(|d| {
            e0.catalog().extension_global_skills.contains(&d)
        }),
        "_gainspell 已注册为 gwent 扩展全局技"
    );
    let mut e_gs = e0.clone();
    advance_past_turn(&mut e_gs, 0);
    advance_to_seat_play(&mut e_gs, 1);
    assert!(
        e_gs.state_core().vm_aux.flags.contains_key(&(1, "gw_spell_gain_init".to_string())),
        "_gainspell 在 Draw 阶段触发初始化倒计时"
    );
    // 手动将 0 号金卡槽位计数置为 0 (OFFSET=1000 => 1000)，1/2 号槽位为 5 (1005)，在 0 号下次摸牌阶段开始触发：金法术被置顶并摸入手
    let mut snap = e_gs.save();
    snap.core.vm_aux.flags.insert((0, "gw_spell_gain:0".to_string()), 1000);
    snap.core.vm_aux.flags.insert((0, "gw_spell_gain:1".to_string()), 1005);
    snap.core.vm_aux.flags.insert((0, "gw_spell_gain:2".to_string()), 1005);
    snap.core.vm_aux.flags.insert((0, "gw_spell_gain_init".to_string()), 1);
    let mut e_gs = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    advance_to_seat_play(&mut e_gs, 0);
    assert!(
        e_gs.debug_hand(0).iter().any(|c| {
            let n = e_gs.catalog().cards[*c as usize].name;
            n.starts_with("gw_") && n != "gw_dieyi"
        }),
        "_gainspell 计数归 0 时将金卡法术置于牌堆顶供摸牌阶段摸入"
    );
    assert_zone_conservation(&e_gs);
}

#[test]
fn u05_b5a_round2_dieyi_distance_and_xianzumaijiu_sha_variants() {
    use engine_native::catalog::Suit;
    let (v, e0) = setup_gwent_locked(5542);

    // #5: `gw_dieyi_equip1` 攻击范围 = 1；`gw_dieyi_equip3`/`4` +0/-0 距离；假标 `gw_dieyi_card:*` 同口径
    let de1 = find_card(&e0, "gw_dieyi_equip1");
    let de3 = find_card(&e0, "gw_dieyi_equip3");
    let de4 = find_card(&e0, "gw_dieyi_equip4");
    let e_rng = inject_equip(&e0, &v, de1, 0);
    assert_eq!(e_rng.state_core().attack_range(e_rng.catalog(), 0), 1, "gw_dieyi_equip1 攻击范围须为 1");
    let e_hp = inject_equip(&e0, &v, de3, 1);
    assert_eq!(e_hp.state_core().computed_distance(e_hp.catalog(), 0, 1), 1, "gw_dieyi_equip3 +0 防御距离");
    let e_hm = inject_equip(&e0, &v, de4, 1);
    assert_eq!(e_hm.state_core().computed_distance(e_hm.catalog(), 1, 3), 2, "gw_dieyi_equip4 -0 进攻距离");

    // 牌库耗尽唯一 gw_dieyi_equip1/3 后蝴蝶舞兜底假标：`qilin`(射程5) 与 HorsePlus 均降回 1 射程 / +0 距离
    let hudiewu = find_card(&e0, "gw_hudiewu");
    let qilin = card_of_kind(&e0, CardKind::Qilin, None);
    let hp_card = card_of_kind(&e0, CardKind::HorsePlus, None);
    let mut e_mask = inject_hand(&e0, &v, &[hudiewu], 0);
    e_mask = inject_equip(&e_mask, &v, qilin, 1);
    e_mask = inject_equip(&e_mask, &v, hp_card, 1);
    let mut snap_m = e_mask.save();
    for dn in ["gw_dieyi_equip1", "gw_dieyi_equip2", "gw_dieyi_equip3", "gw_dieyi_equip4", "gw_dieyi_equip5", "gw_dieyi_judge"] {
        let cid = find_card(&e0, dn);
        snap_m.core.deck.retain(|x| *x != cid);
        snap_m.core.discard.retain(|x| *x != cid);
        snap_m.core.vm_aux.piles.entry((0, "u05_vanished".to_string())).or_default().push(cid);
    }
    let mut e_mask = content_native_loader::restore_locked_engine(&snap_m, &v).unwrap();
    submit(&mut e_mask, 0, GameAction::PlayCard { card: hudiewu, targets: vec![] });
    pass_window(&mut e_mask);
    assert_eq!(e_mask.state_core().attack_range(e_mask.catalog(), 1), 1, "假标 gw_dieyi_card:* 武器射程须为 1");
    assert_eq!(e_mask.state_core().computed_distance(e_mask.catalog(), 0, 1), 1, "假标 gw_dieyi_card:* +1马须 +0 距离");

    // 破晓清理 gw_dieyi_judge 不触发 gw_dieyi
    let dieyi_j = find_card(&e0, "gw_dieyi_judge");
    let px = find_card(&e0, "gw_poxiao");
    let dieyi_def = e0.catalog().skill_by_name("gw_dieyi").unwrap();
    let mut e_px = inject_hand(&e0, &v, &[px], 0);
    e_px = inject_judge(&e_px, &v, dieyi_j, 1);
    submit(&mut e_px, 0, GameAction::PlayCard { card: px, targets: vec![] });
    pass_window(&mut e_px);
    let (e_px, _) = save_restore_answer(e_px, &v, 0, vec![ChoiceOption::Tag(0)]);
    let (mut e_px, _) = save_restore_answer(e_px, &v, 0, vec![ChoiceOption::Player(1)]);
    pass_window(&mut e_px);
    assert!(!e_px.save().core.skills.contains_key(&(1, dieyi_def)), "破晓移除 gw_dieyi_judge 不触发 gw_dieyi");

    // #8(a): `gw_xianzumaijiu` 支持 view-as【杀】（武圣红闪当杀）与虚拟【杀】（罗契）
    let maijiu = find_card(&e0, "gw_xianzumaijiu");
    let red_shan = card_of_kind(&e0, CardKind::Shan, Some(Suit::Heart));
    let mut e_ws = inject_hand(&e0, &v, &[maijiu, red_shan], 0);
    assert!(matches!(
        submit(&mut e_ws, 0, GameAction::PlayCard { card: maijiu, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e_ws);
    e_ws = grant_skill(&e_ws, &v, "武圣", 0);
    let h0_ws = e_ws.debug_hand(0).len();
    let h2_ws = e_ws.debug_hand(2).len();
    assert!(matches!(
        submit(&mut e_ws, 0, GameAction::PlayCard { card: red_shan, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e_ws);
    assert_eq!(e_ws.debug_hp(1), 3, "武圣红闪当杀命中 1 号");
    assert_eq!(e_ws.debug_hand(0).len(), h0_ws, "先祖麦酒命中后 0 号摸 1 (-1+1=+0)");
    assert_eq!(e_ws.debug_hand(2).len(), h2_ws + 1, "先祖麦酒令忠臣 2 号摸 1");
    let xzm_def = e_ws.catalog().skill_by_name("gw_xianzumaijiu").unwrap();
    assert!(!e_ws.save().core.skills.contains_key(&(0, xzm_def)), "触发后移除 gw_xianzumaijiu");

    // 罗契虚拟【杀】同样触发先祖麦酒
    let luoqi = find_card(&e0, "gwmaoxian_luoqi");
    let mut e_lq = inject_hand(&e0, &v, &[maijiu, luoqi], 0);
    submit(&mut e_lq, 0, GameAction::PlayCard { card: maijiu, targets: vec![] });
    pass_window(&mut e_lq);
    let h0_lq = e_lq.debug_hand(0).len();
    let h2_lq = e_lq.debug_hand(2).len();
    submit(&mut e_lq, 0, GameAction::PlayCard { card: luoqi, targets: vec![1] });
    pass_window(&mut e_lq);
    assert_eq!(e_lq.debug_hp(1), 3, "罗契虚拟杀命中 1 号");
    assert_eq!(e_lq.debug_hand(0).len(), h0_lq, "罗契虚拟杀触发先祖麦酒摸 1");
    assert_eq!(e_lq.debug_hand(2).len(), h2_lq + 1, "罗契虚拟杀令友军摸 1");
    assert!(!e_lq.save().core.skills.contains_key(&(0, xzm_def)));

    // #8(b): `gw_kunenfayin` 防住伤害 / `hujia` 吸收为 0 后，先祖麦酒不得摸牌且必须清除（第 2 刀不再触发）
    let sha1 = card_of_kind(&e0, CardKind::Sha, None);
    let sha2 = find_cards_matching(&e0, |d| d.kind == CardKind::Sha, 2)[1];
    let kunen = find_card(&e0, "gw_kunenfayin");
    let mut e_k = grant_skill(&e0, &v, "咆哮", 0);
    e_k = inject_hand(&e_k, &v, &[kunen, maijiu, sha1, sha2], 0);
    submit(&mut e_k, 0, GameAction::PlayCard { card: kunen, targets: vec![1] });
    pass_window(&mut e_k);
    submit(&mut e_k, 0, GameAction::PlayCard { card: maijiu, targets: vec![] });
    pass_window(&mut e_k);
    submit(&mut e_k, 0, GameAction::PlayCard { card: sha1, targets: vec![1] });
    pass_window(&mut e_k);
    assert_eq!(e_k.debug_hp(1), 4, "昆恩防止杀伤");
    assert!(!e_k.save().core.skills.contains_key(&(0, xzm_def)), "昆恩防止杀伤后须清理 gw_xianzumaijiu，不得泄漏到第 2 张杀");
    let h2_before_s2 = e_k.debug_hand(2).len();
    submit(&mut e_k, 0, GameAction::PlayCard { card: sha2, targets: vec![3] });
    pass_window(&mut e_k);
    assert_eq!(e_k.debug_hp(3), 3);
    assert_eq!(e_k.debug_hand(2).len(), h2_before_s2, "泄漏清理后第 2 张杀不得再触发先祖麦酒");

    // 0 伤害（DamageZero）同样不摸牌并清理 gw_xianzumaijiu，第 2 刀不再触发
    let mut e_z = grant_skill(&e0, &v, "咆哮", 0);
    e_z = inject_hand(&e_z, &v, &[maijiu, sha2], 0);
    submit(&mut e_z, 0, GameAction::PlayCard { card: maijiu, targets: vec![] });
    pass_window(&mut e_z);
    let mut core_z = e_z.save().core;
    let mut res_z = engine_native::Resolution::new(128);
    let mut log_z = Vec::new();
    let dmg_fid = engine_native::resolution::push_frame(
        &mut res_z,
        engine_native::EventData::Damage(engine_native::machine::damage_data_with(
            Some(0),
            1,
            0,
            engine_native::event::DamageNature::Normal,
            Some(sha1),
        )),
    );
    core_z.vm_aux.flags.insert((0, format!("sha_damage_frame:{dmg_fid}")), 1);
    let out_z = engine_native::resolution::run(&mut core_z, &mut res_z, e_z.catalog(), &mut log_z, None);
    assert_eq!(out_z, engine_native::resolution::StepOutcome::Done);
    let mut snap_z = e_z.save();
    snap_z.core = core_z;
    let mut e_z = content_native_loader::restore_locked_engine(&snap_z, &v).unwrap();
    assert_eq!(e_z.debug_hp(1), 4);
    assert!(e_z.debug_hand(2).is_empty(), "DamageZero 不触发先祖麦酒摸牌");
    assert!(!e_z.save().core.skills.contains_key(&(0, xzm_def)), "DamageZero 后须清理 gw_xianzumaijiu");
    submit(&mut e_z, 0, GameAction::PlayCard { card: sha2, targets: vec![3] });
    pass_window(&mut e_z);
    assert_eq!(e_z.debug_hp(3), 3);
    assert!(e_z.debug_hand(2).is_empty(), "DamageZero 清理后第 2 张杀不得再触发先祖麦酒");
    assert_zone_conservation(&e_z);
}
