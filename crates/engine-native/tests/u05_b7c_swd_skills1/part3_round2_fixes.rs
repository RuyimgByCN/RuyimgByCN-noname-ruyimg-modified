use super::common::*;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::DamageNature;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, LogEvent};

fn move_to_discard(e: &mut Engine, cid: u32) {
    let core = e.state_core_mut();
    core.deck.retain(|x| *x != cid);
    core.discard.retain(|x| *x != cid);
    core.table.retain(|x| *x != cid);
    for pl in &mut core.players {
        pl.hand.retain(|x| *x != cid);
        pl.equips.retain(|x| *x != cid);
        pl.judge.retain(|x| *x != cid);
        pl.off_court.retain(|x| *x != cid);
    }
    core.discard.push(cid);
}

#[test]
fn r2_p0_xuanying_use_offturn_cards_lost_and_no_damage_counterattack() {
    let (pack, mut e) = swd_engine(705_201);
    e.assign_general_named(0, "swd_duanmeng").unwrap();
    assert!(e.catalog().skill_by_name("xuanying_use").is_some());
    assert!(e.state_core().has_skill(0, e.catalog().skill_by_name("xuanying_use").unwrap()));
    if let Some(d_old) = e.catalog().skill_by_name("xuanying_damage") {
        assert!(!e.state_core().has_skill(0, d_old), "duanmeng 不得授予 xuanying_damage");
    }

    // 1) 空牌堆激活 xuanying：先横置，Yield 后发射 card: None 虚拟杀
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let drained: Vec<u32> = e.state_core_mut().deck.drain(..).collect();
    e.state_core_mut().discard.extend(drained);
    let xy_def = e.catalog().skill_by_name("xuanying").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: xy_def });
    let tgt = e.open_choice().expect("xuanying_target");
    let (cid, obs) = (tgt.id, tgt.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(2)] });
    drain_pass_or_first(&mut e);
    assert!(e.state_core().player(0).linked, "xuanying 必须先横置");
    assert_eq!(e.debug_hp(2), 3, "空牌堆下 xuanying 虚拟杀仍命中远位 p2");

    // 2) 己方回合内失牌不得触发 xuanying_use
    let own_t = find_card(&e, |k, _| k == CardKind::Tao);
    let p0_lose = find_card(&e, |k, _| k == CardKind::Shan);
    move_card_to_hand(&mut e, 0, p0_lose);
    move_card_to_hand(&mut e, 0, own_t);
    let d3 = find_card(&e, |k, _| k == CardKind::Dismantle);
    move_card_to_hand(&mut e, 1, d3);

    // 3) 他人回合内拆 p0 手牌（CardsLost off-turn while linked）→ xuanying_use_target + 存读档 + 不计次虚拟杀
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    move_card_to_hand(&mut e, 0, p0_lose);
    move_card_to_hand(&mut e, 1, d3);
    e.state_core_mut().players[0].linked = true;
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: d3, targets: vec![0] });
    for _ in 0..8 {
        let Some(ch) = e.open_choice() else { break };
        if ch.spec.prompt == "xuanying_use_target" {
            break;
        }
        let sel = if ch.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else {
            ch.spec.options.iter().take(ch.spec.min.max(1)).copied().collect()
        };
        let (id, o) = (ch.id, ch.observer);
        let _ = submit(&mut e, o, GameAction::Answer { choice_id: id, selection: sel });
    }
    let ch = e.open_choice().expect("回合外失牌触发 xuanying_use_target");
    assert_eq!((ch.observer, ch.spec.prompt), (0, "xuanying_use_target"));
    let snap = decode_archive(&encode_archive(&e.save())).unwrap();
    let mut restored = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();
    assert_eq!(e.open_choice(), restored.open_choice());
    let (id, o) = (ch.id, ch.observer);
    let _ = submit(&mut e, o, GameAction::Answer { choice_id: id, selection: vec![ChoiceOption::Player(1)] });
    let _ = submit(&mut restored, o, GameAction::Answer { choice_id: id, selection: vec![ChoiceOption::Player(1)] });
    drain_pass_or_first(&mut e);
    drain_pass_or_first(&mut restored);
    assert_eq!(e.state_hash(), restored.state_hash());
    assert!(!e.state_core().player(0).linked, "xuanying_use 解除横置");
    assert_eq!(e.debug_hp(1), 3, "xuanying_use 虚拟杀命中 p1");

    // 4) 仅受伤不得触发反击
    e.state_core_mut().players[0].linked = true;
    let hp1 = e.debug_hp(1);
    let _ = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(e.open_choice().is_none(), "受伤不得触发 xuanying_damage");
    assert_eq!(e.debug_hp(1), hp1);
    assert!(e.state_core().player(0).linked);
}

#[test]
fn r2_p0_jilve_playable_filters_aoe_jiedao_and_table_carrier() {
    let (pack, mut e) = swd_engine(705_202);
    e.debug_grant(0, "jilve");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // 1) top-3 = [Shan, Nullify, Jiu] + jiu_used=true → 无可出牌（不得把 Shan/Nullify/已用酒纳入候选）
    let shan = find_card(&e, |k, _| k == CardKind::Shan);
    let nul = find_card(&e, |k, _| k == CardKind::Nullify);
    let jiu = find_card(&e, |k, _| k == CardKind::Jiu);
    push_top_deck(&mut e, shan);
    push_top_deck(&mut e, nul);
    push_top_deck(&mut e, jiu);
    e.state_core_mut().turn.jiu_used = true;
    let jl = e.catalog().skill_by_name("jilve").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: jl });
    assert!(e.open_choice().is_none(), "Shan/Nullify/已限次之 Jiu 不得出现在极略候选中");

    // 2) sha_used_play_phase=true 排除 Sha，包含 AOE (SavageAttack)；AOE 正确传 targets 命中全体他人且牌进 table→discard（不经手牌）
    e.state_core_mut().turn.sha_used_play_phase = true;
    let sha = find_card(&e, |k, _| k == CardKind::Sha);
    let nanman = find_card(&e, |k, _| k == CardKind::SavageAttack);
    push_top_deck(&mut e, shan);
    push_top_deck(&mut e, sha);
    push_top_deck(&mut e, nanman);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: jl });
    let ch = e.open_choice().expect("jilve_card");
    assert_eq!(ch.spec.prompt, "jilve_card");
    assert_eq!(ch.spec.options, vec![ChoiceOption::Card(nanman)], "已出杀后候选只剩 SavageAttack");
    let snap = decode_archive(&encode_archive(&e.save())).unwrap();
    let restored = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();
    assert_eq!(e.open_choice(), restored.open_choice());
    let (cid, obs) = (ch.id, ch.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(nanman)] });
    assert_eq!(e.debug_hand(0).len(), 0, "极略用牌进 table，不得先入 owner 手牌");
    drain_pass_or_first(&mut e);
    assert_eq!((e.debug_hp(1), e.debug_hp(2), e.debug_hp(3)), (3, 3, 3), "极略南蛮 AOE 命中全部他人");
    assert!(e.state_core().discard.contains(&nanman));

    // 3) 借刀杀人两段目标询问（jilve_targets → jilve_jiedao_victim）及挂起存读档
    let w = weapon_with_range(&e, 2);
    equip_slot_card(&mut e, 1, w);
    let jiedao = find_card(&e, |k, _| k == CardKind::Jiedao);
    push_top_deck(&mut e, shan);
    push_top_deck(&mut e, nul);
    push_top_deck(&mut e, jiedao);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: jl });
    let ch = e.open_choice().expect("jilve_card jiedao");
    let (cid, obs) = (ch.id, ch.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(jiedao)] });
    let t1 = e.open_choice().expect("jilve_targets weapon holder");
    assert_eq!(t1.spec.prompt, "jilve_targets");
    let (cid, obs) = (t1.id, t1.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    let t2 = e.open_choice().expect("jilve_jiedao_victim");
    assert_eq!(t2.spec.prompt, "jilve_jiedao_victim");
    let snap2 = decode_archive(&encode_archive(&e.save())).unwrap();
    let mut restored2 = Engine::restore_with_extensions(&snap2, &[&pack]).unwrap();
    assert_eq!(e.open_choice(), restored2.open_choice());
    let (cid, obs) = (t2.id, t2.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(2)] });
    let _ = submit(&mut restored2, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(2)] });
    for branch in [&mut e, &mut restored2] {
        for _ in 0..8 {
            let Some(ch) = branch.open_choice() else { break };
            let (cid, obs, p) = (ch.id, ch.observer, ch.spec.prompt);
            let sel = if p == "jiedao_sha_or_give" {
                vec![ChoiceOption::Tag(1)]
            } else {
                vec![ChoiceOption::Pass]
            };
            let _ = submit(branch, obs, GameAction::Answer { choice_id: cid, selection: sel });
        }
    }
    assert_eq!(e.state_hash(), restored2.state_hash());
    assert!(
        !e.state_core().player(1).equips.contains(&w) && e.state_core().player(2).equips.contains(&w),
        "p1 无杀选择 Tag(1) 交武器给 p2"
    );
}

#[test]
fn r2_p0_lianda_and_jianji_no_deck_carrier_and_real_equip() {
    let (pack, mut e) = swd_engine(705_203);
    e.debug_grant(0, "lianda");
    e.debug_grant(0, "jianji");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // 1) lianda 空牌堆：杀被闪后弃 1 发虚拟杀（card: None），不崩且命中 p1
    let drained: Vec<u32> = e.state_core_mut().deck.drain(..).collect();
    e.state_core_mut().discard.extend(drained);
    let sha = find_card(&e, |k, _| k == CardKind::Sha);
    let shan = find_card(&e, |k, _| k == CardKind::Shan);
    let disc = find_card(&e, |k, _| k == CardKind::Tao);
    move_card_to_hand(&mut e, 0, sha);
    move_card_to_hand(&mut e, 0, disc);
    move_card_to_hand(&mut e, 1, shan);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    for _ in 0..8 {
        let Some(ch) = e.open_choice() else { break };
        let (cid, obs, p) = (ch.id, ch.observer, ch.spec.prompt);
        let sel = if p == "respond_shan" {
            vec![ChoiceOption::Card(shan)]
        } else if p == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
        } else if p == "lianda_discard" {
            vec![ChoiceOption::Card(disc)]
        } else {
            vec![ChoiceOption::Pass]
        };
        let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
    }
    assert_eq!(e.debug_hp(1), 3, "空牌堆下连打虚拟杀命中 p1");

    // 2) jianji 空牌堆+无视防具+实体装备作载体（红装破仁王+空牌堆安全+不占杀次数）
    let red_eq = (0..e.catalog().cards.len() as u32)
        .find(|c| {
            let d = &e.catalog().cards[*c as usize];
            d.kind.is_equip() && matches!(d.suit, Suit::Heart | Suit::Diamond)
        })
        .expect("红装备");
    let renwang = find_card(&e, |k, _| k == CardKind::Renwang);
    equip_slot_card(&mut e, 1, renwang);
    move_card_to_hand(&mut e, 0, red_eq);
    e.state_core_mut().turn.sha_used_play_phase = true;
    let jj = e.catalog().skill_by_name("jianji").unwrap();
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: jj });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "已出杀仍可发动剑击(addCount:false): {out:?}");
    let ch = e.open_choice().expect("jianji_pick");
    let (cid, obs) = (ch.id, ch.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(red_eq)] });
    let tgt = e.open_choice().expect("jianji_target");
    let snap = decode_archive(&encode_archive(&e.save())).unwrap();
    let restored = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();
    assert_eq!(e.open_choice(), restored.open_choice());
    let (cid, obs) = (tgt.id, tgt.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    drain_pass_or_first(&mut e);
    assert_eq!(e.debug_hp(1), 2, "红装剑击作杀穿透且空牌堆不崩溃");
    assert!(e.state_core().discard.contains(&red_eq), "载体红装进入弃牌堆");
}

#[test]
fn r2_p0_nlianji_ask_restore_and_tie_discards_throw_card() {
    let (pack, mut e) = swd_engine(705_204);
    e.debug_grant(0, "nlianji");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let throw_c = find_card(&e, |k, _| k == CardKind::Tao);
    let r4 = (0..e.catalog().cards.len() as u32).find(|c| e.catalog().cards[*c as usize].rank == 4).unwrap();
    let r9 = (0..e.catalog().cards.len() as u32).find(|c| e.catalog().cards[*c as usize].rank == 9).unwrap();
    move_card_to_hand(&mut e, 0, throw_c);
    move_card_to_hand(&mut e, 1, r4);
    move_card_to_hand(&mut e, 2, r9);

    let nlj = e.catalog().skill_by_name("nlianji").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: nlj });
    // 验证全五步挂起窗口的 save -> restore -> open_choice 均与真实流程一致
    for (prompt, sel) in [
        ("nlianji_throw", ChoiceOption::Card(throw_c)),
        ("nlianji_target0", ChoiceOption::Player(1)),
        ("nlianji_target1", ChoiceOption::Player(2)),
        ("nlianji_t0_pick", ChoiceOption::Card(r4)),
        ("nlianji_t1_pick", ChoiceOption::Card(r9)),
    ] {
        let ch = e.open_choice().expect(prompt);
        assert_eq!(ch.spec.prompt, prompt);
        let snap = decode_archive(&encode_archive(&e.save())).unwrap();
        let restored = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();
        assert_eq!(e.open_choice(), restored.open_choice(), "pending mismatch at {prompt}");
        let (cid, obs) = (ch.id, ch.observer);
        let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![sel] });
    }
    assert!(e.debug_hand(2).contains(&throw_c), "胜者 p2 获得 throw_c");
    assert_eq!(e.debug_hp(1), 3, "败者 p1 受胜者 p2 的 1 点伤害");
}

#[test]
fn r2_p0_gongshen_cancels_card_to_discard_not_hand() {
    let (_pack, mut e) = swd_engine(705_205);
    e.debug_grant(2, "gongshen");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let nanman = find_card(&e, |k, _| k == CardKind::SavageAttack);
    let eq = find_card(&e, |k, _| k.equip_slot().is_some());
    move_card_to_hand(&mut e, 0, nanman);
    equip_slot_card(&mut e, 2, eq);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: nanman, targets: vec![] });
    for _ in 0..6 {
        let Some(ch) = e.open_choice() else { break };
        let (cid, obs, p) = (ch.id, ch.observer, ch.spec.prompt);
        let sel = if p == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
        } else if p == "gongshen_discard" {
            vec![ChoiceOption::Card(eq)]
        } else {
            vec![ChoiceOption::Pass]
        };
        let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
    }
    assert!(!e.debug_hand(0).contains(&nanman), "被工神取消的南蛮不得回使用者手牌");
    assert!(e.state_core().discard.contains(&nanman), "被工神取消的南蛮必须进弃牌堆");
    assert_eq!((e.debug_hp(1), e.debug_hp(2), e.debug_hp(3)), (4, 4, 4), "无人受南蛮伤");
}

#[test]
fn r2_p1_pozhou2_duishi_huodan_shenyan_mingfu_senluo_repeatable() {
    use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
    use engine_native::content::host::{HostBehavior, HostSkillDecl};
    let pack = swd_pack();
    let moshou_pack = ExtensionPack {
        def: ExtensionDef {
            name: "r2-moshou",
            version: 1,
            description: "测试墨守延时锦囊目标排除",
        },
        generals: vec![],
        cards: vec![],
        vm_skills: vec![],
        host_skills: vec![HostSkillDecl {
            name: "moshou",
            behavior: HostBehavior::MoshouImmunity,
            presentation_binding: None,
        }],
        init_hook: InitHook::None,
        enabled: true,
    };
    let mut e = Engine::new_with_extensions(705_206, 4, &[&pack, &moshou_pack]).expect("建局");
    e.debug_grant(0, "pozhou2");
    e.debug_grant(0, "duishi");
    e.debug_grant(0, "huodan");
    e.debug_grant(0, "shenyan");
    e.debug_grant(0, "mingfu");
    e.debug_grant(0, "sxianjing");
    e.debug_grant(0, "cyxianjiang");
    e.debug_grant(1, "moshou");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // 1) mingfu 排除自己与墨守目标
    let club = find_card(&e, |k, s| s == Suit::Club && k == CardKind::Sha);
    move_card_to_hand(&mut e, 0, club);
    let mf = e.catalog().skill_by_name("mingfu").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: mf });
    let ch = e.open_choice().unwrap();
    let (cid, obs) = (ch.id, ch.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(club)] });
    let tgt = e.open_choice().unwrap();
    assert!(!tgt.spec.options.contains(&ChoiceOption::Player(0)), "mingfu 不得包含自己");
    assert!(!tgt.spec.options.contains(&ChoiceOption::Player(1)), "mingfu 不得包含墨守持有者 p1");
    let (cid, obs) = (tgt.id, tgt.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: Vec::new() });

    // 2) sxianjing 可同回合置入两张不同花色陷阱（无 once_per_turn）
    let sp = find_card(&e, |k, s| s == Suit::Spade && k == CardKind::Sha);
    let ht = find_card(&e, |k, s| s == Suit::Heart && k == CardKind::Tao);
    move_card_to_hand(&mut e, 0, sp);
    move_card_to_hand(&mut e, 0, ht);
    let sxj = e.catalog().skill_by_name("sxianjing").unwrap();
    for trap_c in [sp, ht] {
        let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: sxj });
        assert!(matches!(out, CommandOutcome::Pending { .. }), "陷阱可同回合多次放置: {out:?}");
        let ch = e.open_choice().unwrap();
        let (cid, obs) = (ch.id, ch.observer);
        let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(trap_c)] });
    }
    assert_eq!(e.state_core().player(0).sxianjing_traps.len(), 2);

    // 3) huodan 红装备 + 先 LoseHp 后 Yield 伤害 + 2 目标按序受伤
    let red_eq = (0..e.catalog().cards.len() as u32)
        .find(|c| {
            let d = &e.catalog().cards[*c as usize];
            d.kind.is_equip() && matches!(d.suit, Suit::Heart | Suit::Diamond)
        })
        .unwrap();
    equip_slot_card(&mut e, 0, red_eq);
    let hd = e.catalog().skill_by_name("huodan").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: hd });
    let hp_pick = e.open_choice().unwrap();
    assert!(hp_pick.spec.options.contains(&ChoiceOption::Card(red_eq)), "huodan 必须接受红色装备");
    let (cid, obs) = (hp_pick.id, hp_pick.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(red_eq)] });
    let h_tgt = e.open_choice().unwrap();
    let (cid, obs) = (h_tgt.id, h_tgt.observer);
    let log_start = e.log().len();
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(2), ChoiceOption::Player(3)] });
    drain_pass_or_first(&mut e);
    assert_eq!((e.debug_hp(0), e.debug_hp(2), e.debug_hp(3)), (3, 3, 3));
    let dmg_seats: Vec<u8> = e.log()[log_start..]
        .iter()
        .filter_map(|ev| match ev {
            LogEvent::DamageDealt { target, .. } => Some(*target),
            _ => None,
        })
        .collect();
    assert_eq!(dmg_seats, vec![2, 3], "huodan 多目标伤害按选择序先后结算");

    // 4) duishi 只准弃置“手牌”同花色（装备同花色不得作弃置候选）+ pending 存读档 min=1
    let sp_eq = (0..e.catalog().cards.len() as u32)
        .find(|c| e.catalog().cards[*c as usize].kind.is_equip() && e.catalog().cards[*c as usize].suit == Suit::Spade)
        .unwrap();
    let p1_d = find_card(&e, |k, s| s == Suit::Diamond && k == CardKind::Shan);
    let p0_sp = find_card(&e, |k, s| s == Suit::Spade && k == CardKind::Dismantle);
    move_card_to_hand(&mut e, 0, p0_sp);
    equip_slot_card(&mut e, 1, sp_eq);
    move_card_to_hand(&mut e, 1, p1_d);
    let ds = e.catalog().skill_by_name("duishi").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: ds });
    let c1 = e.open_choice().unwrap();
    let (cid, obs) = (c1.id, c1.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(p0_sp)] });
    let c2 = e.open_choice().unwrap();
    let (cid, obs) = (c2.id, c2.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    let gain = e.open_choice().expect("p1 仅装备有黑桃，无同花色手牌应直入 duishi_gain");
    assert_eq!(gain.spec.prompt, "duishi_gain");
    let snap = decode_archive(&encode_archive(&e.save())).unwrap();
    let restored = Engine::restore_with_extensions(&snap, &[&pack, &moshou_pack]).unwrap();
    assert_eq!(e.open_choice(), restored.open_choice());
    let (cid, obs) = (gain.id, gain.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(sp_eq)] });
    assert!(e.debug_hand(0).contains(&sp_eq), "duishi_gain 偷得 p1 装备");

    // 5) shenyan 蔓延伤害 source = Some(owner) 且座位序
    let _ = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Fire);
    for _ in 0..8 {
        let Some(ch) = e.open_choice() else { break };
        let (cid, obs, p) = (ch.id, ch.observer, ch.spec.prompt);
        let sel = if p == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
        } else {
            vec![ChoiceOption::Pass]
        };
        let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
    }
    assert!(
        e.log().iter().any(|ev| matches!(ev, LogEvent::DamageDealt { source: Some(0), target: 2, .. })),
        "神焰蔓延来源必须是 Some(0)"
    );
}

#[test]
fn r2_p2_touxi_zhanxing_kbolan2_lingxian_huangyu_lmazui_cleanups() {
    let (_pack, mut e) = swd_engine(705_207);
    e.debug_grant(0, "lmazui");
    e.debug_grant(0, "huangyu");
    e.debug_grant(0, "zhanxing");
    e.debug_grant(1, "lingxian");
    e.debug_grant(0, "touxi");
    e.debug_grant(0, "鬼才");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // 1) huangyu 取消退 once_used + 空牌堆发虚拟炽羽袭
    let r1 = find_card(&e, |k, s| k == CardKind::Tao && s == Suit::Heart);
    let r2 = find_card(&e, |k, s| k == CardKind::Shan && s == Suit::Diamond);
    move_card_to_hand(&mut e, 0, r1);
    move_card_to_hand(&mut e, 0, r2);
    let hy = e.catalog().skill_by_name("huangyu").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: hy });
    let ch = e.open_choice().unwrap();
    let (cid, obs) = (ch.id, ch.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: Vec::new() });
    assert!(!e.state_core().skills.get(&(0, hy)).unwrap().once_used, "huangyu 取消必退 once_used");
    let drained: Vec<u32> = e.state_core_mut().deck.drain(..).collect();
    e.state_core_mut().discard.extend(drained);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: hy });
    let ch = e.open_choice().unwrap();
    let (cid, obs) = (ch.id, ch.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(r1), ChoiceOption::Card(r2)] });
    drain_pass_or_first(&mut e);
    assert_eq!(e.debug_hp(1), 3, "空牌堆 huangyu 虚拟炽羽袭仍伤人");
    assert_eq!(e.debug_hand(0), Vec::<u32>::new(), "huangyu 结算后 p0 手牌应为空");

    // 2) lmazui 可指定自身
    let blk = find_card(&e, |k, s| k == CardKind::Dismantle && s == Suit::Spade);
    move_card_to_hand(&mut e, 0, blk);
    let lm = e.catalog().skill_by_name("lmazui").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: lm });
    let ch = e.open_choice().unwrap();
    let (cid, obs) = (ch.id, ch.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(blk)] });
    let tgt = e.open_choice().unwrap();
    assert!(tgt.spec.options.contains(&ChoiceOption::Player(0)), "lmazui 必须允许选择自身");
    let (cid, obs) = (tgt.id, tgt.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(0)] });
    assert!(e.state_core().player(0).off_court.contains(&blk));

    // 3) zhanxing：方块+黑桃先摸 2 再弹出可取消的 zhanxing_target
    let d_card = find_card(&e, |k, s| s == Suit::Diamond && k == CardKind::Sha);
    let s_card = find_card(&e, |k, s| s == Suit::Spade && k == CardKind::Jiu);
    let c1 = find_card(&e, |k, _| k == CardKind::Tao);
    let c2 = find_card(&e, |k, _| k == CardKind::Shan);
    move_to_discard(&mut e, c1);
    move_to_discard(&mut e, c2);
    push_top_deck(&mut e, c2);
    push_top_deck(&mut e, c1);
    push_top_deck(&mut e, s_card);
    push_top_deck(&mut e, d_card);
    let z_d1 = find_card(&e, |k, _| k == CardKind::Duel);
    let z_d2 = find_card(&e, |k, _| k == CardKind::Wuzhong);
    move_card_to_hand(&mut e, 0, z_d1);
    move_card_to_hand(&mut e, 0, z_d2);
    let zx = e.catalog().skill_by_name("zhanxing").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: zx });
    let ch = e.open_choice().unwrap();
    let (cid, obs) = (ch.id, ch.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(z_d1), ChoiceOption::Card(z_d2)] });
    let zt = e.open_choice().expect("zhanxing_target");
    assert_eq!(zt.spec.prompt, "zhanxing_target");
    assert!(zt.spec.allow_cancel, "zhanxing_target 必须 allow_cancel = true");
    assert_eq!(e.debug_hand(0).len(), 2, "打开 zhanxing_target 前方块摸 2 须已落手");
    let (cid, obs) = (zt.id, zt.observer);
    let hp1_before = e.debug_hp(1);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: Vec::new() });
    assert_eq!(e.debug_hp(1), hp1_before, "取消 zhanxing_target 无雷伤");

    // 4) lingxian 先己后目标摸牌序 + AI 对 xuanying_use_target / yeying / yuchen 公开信息
    let log_l = e.log().len();
    let sha1 = find_card(&e, |k, _| k == CardKind::Sha);
    let shan1 = find_card(&e, |k, _| k == CardKind::Shan);
    push_top_deck(&mut e, r2);
    push_top_deck(&mut e, r1);
    move_card_to_hand(&mut e, 0, sha1);
    move_card_to_hand(&mut e, 1, shan1);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha1, targets: vec![1] });
    for _ in 0..8 {
        let Some(ch) = e.open_choice() else { break };
        let (cid, obs, p) = (ch.id, ch.observer, ch.spec.prompt);
        let sel = if p == "respond_shan" {
            vec![ChoiceOption::Card(shan1)]
        } else if p == "lingxian_target" {
            vec![ChoiceOption::Player(3)]
        } else {
            vec![ChoiceOption::Pass]
        };
        let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
    }
    let drawn_seats: Vec<u8> = e.log()[log_l..]
        .iter()
        .filter_map(|ev| match ev {
            LogEvent::CardsDrawn { player, .. } => Some(*player),
            _ => None,
        })
        .collect();
    assert_eq!(drawn_seats, vec![1, 3], "灵弦必须先 owner(1) 后 target(3) 摸牌");

    // 5) touxi 经 OrdinaryJudge 可被鬼才改判，且红牌时 touxi_discard 允许取消
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sp_j = find_card(&e, |k, s| s == Suit::Spade && k == CardKind::Sha);
    let ht_j = find_card(&e, |k, s| s == Suit::Heart && k == CardKind::Tao);
    move_card_to_hand(&mut e, 0, sp_j);
    push_top_deck(&mut e, ht_j);
    for _ in 0..4 {
        if e.open_choice().is_some() {
            break;
        }
        let cur = e.turn().current;
        let _ = submit(&mut e, cur, GameAction::NextPhase);
    }
    assert!(e.open_choice().is_some(), "p1 结束应触发 touxi 询问");
    let mut saw_guicai = false;
    for _ in 0..10 {
        let Some(ch) = e.open_choice() else { break };
        let (cid, obs, p) = (ch.id, ch.observer, ch.spec.prompt);
        let sel = if p == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
        } else if p == "gui_dao_rewrite" {
            saw_guicai = true;
            vec![ChoiceOption::Card(sp_j)]
        } else {
            vec![ChoiceOption::Pass]
        };
        let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
    }
    assert!(saw_guicai, "touxi 必须经 OrdinaryJudge 触发鬼才改判");

    let (tx_pack, mut tx_e) = swd_engine(705_212);
    tx_e.debug_grant(0, "touxi");
    advance_to_play(&mut tx_e, 1);
    for p in 0..4 {
        tx_e.debug_clear_hand(p);
    }
    let tx_keep = find_card(&tx_e, |k, s| s == Suit::Club && k == CardKind::Sha);
    let tx_red = find_card(&tx_e, |k, s| s == Suit::Heart && k == CardKind::Tao);
    move_card_to_hand(&mut tx_e, 0, tx_keep);
    push_top_deck(&mut tx_e, tx_red);
    for _ in 0..4 {
        if tx_e.open_choice().is_some() {
            break;
        }
        let cur = tx_e.turn().current;
        let _ = submit(&mut tx_e, cur, GameAction::NextPhase);
    }
    let mut saw_touxi_disc = false;
    for _ in 0..6 {
        let Some(ch) = tx_e.open_choice() else { break };
        let (cid, obs, p) = (ch.id, ch.observer, ch.spec.prompt);
        if p == "touxi_discard" {
            saw_touxi_disc = true;
            assert_eq!(obs, 1, "touxi_discard 询问当前回合者 p1");
            assert!(ch.spec.allow_cancel, "touxi_discard 必须 allow_cancel = true");
            let snap = decode_archive(&encode_archive(&tx_e.save())).unwrap();
            let restored = Engine::restore_with_extensions(&snap, &[&tx_pack]).unwrap();
            assert_eq!(tx_e.open_choice(), restored.open_choice());
            let _ = submit(&mut tx_e, obs, GameAction::Answer { choice_id: cid, selection: Vec::new() });
            break;
        }
        let sel = if p == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
        } else {
            vec![ChoiceOption::Pass]
        };
        let _ = submit(&mut tx_e, obs, GameAction::Answer { choice_id: cid, selection: sel });
    }
    assert!(saw_touxi_disc, "红牌判定必打开可取消的 touxi_discard");
    assert!(tx_e.debug_hand(0).contains(&tx_keep), "取消 touxi_discard 不得弃掉 p0 手牌");
}

#[test]
fn r2_extra_viewas_huanxing_kbolan_cyxianjiang_pozhou_and_ai_public() {
    let (pack, mut e) = swd_engine(705_208);
    set_roles_lord_rebel(&mut e); // p0=Lord, p1=Rebel, p2=Loyalist, p3=Traitor
    e.debug_grant(0, "guiying");
    e.debug_grant(0, "fengze");
    e.debug_grant(0, "cyxianjiang");
    e.debug_grant(0, "pozhou2");
    e.debug_grant(0, "yuchen");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // 1) guiying 不限次 + pending save/restore + fengze once_per_turn + 禁止 PlayCard 劫持黑南蛮
    let blk_nm = find_card(&e, |k, s| k == CardKind::SavageAttack && matches!(s, Suit::Spade | Suit::Club));
    move_card_to_hand(&mut e, 0, blk_nm);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: blk_nm, targets: vec![] });
    drain_pass_or_first(&mut e);
    assert_eq!((e.debug_hp(1), e.debug_hp(2), e.debug_hp(3)), (3, 3, 3), "黑南蛮经 PlayCard 正常结算，不得被 fengze 劫持");
    for p in 1..4 {
        e.debug_set_hp(p, 4);
    }

    let blk1 = find_card(&e, |k, s| k == CardKind::Sha && s == Suit::Spade);
    let blk2 = find_card(&e, |k, s| k == CardKind::Sha && s == Suit::Club);
    let own_extra = find_card(&e, |k, _| k == CardKind::Jiu);
    let p1_c = find_card(&e, |k, _| k == CardKind::Dismantle);
    move_card_to_hand(&mut e, 0, blk1);
    move_card_to_hand(&mut e, 0, blk2);
    move_card_to_hand(&mut e, 0, own_extra);
    move_card_to_hand(&mut e, 1, p1_c);
    let gy = e.catalog().skill_by_name("guiying").unwrap();
    for b_card in [blk1, blk2] {
        let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: gy });
        assert!(matches!(out, CommandOutcome::Pending { .. }), "guiying 可同回合连续发动: {out:?}");
        let ch = e.open_choice().unwrap();
        let (cid, obs) = (ch.id, ch.observer);
        let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(b_card)] });
        let tgt = e.open_choice().unwrap();
        assert_eq!(tgt.spec.prompt, "guiying_target");
        let snap = decode_archive(&encode_archive(&e.save())).unwrap();
        let restored = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();
        assert_eq!(e.open_choice(), restored.open_choice());
        let (cid, obs) = (tgt.id, tgt.observer);
        let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
        drain_pass_or_first(&mut e);
    }
    let fz = e.catalog().skill_by_name("fengze").unwrap();
    let blk_dism = find_card(&e, |k, s| k == CardKind::Dismantle && matches!(s, Suit::Spade | Suit::Club));
    move_card_to_hand(&mut e, 0, blk_dism);
    e.debug_set_hp(1, 2);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: fz });
    let ch = e.open_choice().unwrap();
    assert_eq!(ch.spec.prompt, "fengze_pick");
    let snap = decode_archive(&encode_archive(&e.save())).unwrap();
    let restored = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();
    assert_eq!(e.open_choice(), restored.open_choice());
    let (cid, obs) = (ch.id, ch.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(blk_dism)] });
    drain_pass_or_first(&mut e);
    assert_eq!(e.debug_hp(1), 3, "fengze 桃园回 1");
    assert!(matches!(submit(&mut e, 0, GameAction::ActivateSkill { skill: fz }), CommandOutcome::Rejected(_)), "fengze usable:1");

    // 2) cyxianjiang 同回合对不同目标多次触发（无 once_per_turn）
    let dup_pair = |slot| -> (u32, u32) {
        (0..e.catalog().cards.len() as u32)
            .find_map(|c| {
                if e.catalog().card_kind(c).equip_slot() == Some(slot) {
                    (0..e.catalog().cards.len() as u32)
                        .find(|c2| *c2 != c && e.catalog().cards[*c2 as usize].name == e.catalog().cards[c as usize].name)
                        .map(|c2| (c, c2))
                } else {
                    None
                }
            })
            .expect("存在重名装备对")
    };
    let (eq_a, eq_a_dup) = dup_pair(engine_native::catalog::EquipSlot::Weapon);
    let (eq_b, eq_b_dup) = dup_pair(engine_native::catalog::EquipSlot::Armor);
    equip_slot_card(&mut e, 1, eq_a);
    equip_slot_card(&mut e, 3, eq_b);
    push_top_deck(&mut e, eq_a_dup);
    push_top_deck(&mut e, eq_b_dup);
    let d1 = find_card(&e, |k, _| k == CardKind::Duel);
    let d2 = find_card(&e, |k, _| k == CardKind::Dismantle);
    move_card_to_hand(&mut e, 0, d1);
    move_card_to_hand(&mut e, 0, d2);
    for (card_use, t_seat, copy_eq) in [(d1, 1, eq_a), (d2, 3, eq_b)] {
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: card_use, targets: vec![t_seat] });
        let mut confirmed = false;
        for _ in 0..8 {
            let Some(ch) = e.open_choice() else { break };
            let (cid, obs, p) = (ch.id, ch.observer, ch.spec.prompt);
            let sel = if p == "trigger_confirm" && obs == 0 {
                confirmed = true;
                vec![ChoiceOption::Confirm]
            } else if p == "cyxianjiang_pick" {
                vec![ChoiceOption::Card(copy_eq)]
            } else {
                vec![ChoiceOption::Pass]
            };
            let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
        }
        assert!(confirmed, "cyxianjiang 应在同一回合对目标 {t_seat} 触发");
    }

    // 3) pozhou2 排除已被封印的目标（pozhou_target 不含 p1）
    e.state_core_mut().players[0].pozhou_count = 2;
    let pz2 = e.catalog().skill_by_name("pozhou2").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: pz2 });
    let pc = e.open_choice().unwrap();
    let (cid, obs) = (pc.id, pc.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: pz2 });
    let pc2 = e.open_choice().unwrap();
    assert!(!pc2.spec.options.contains(&ChoiceOption::Player(1)), "pozhou2 不得再选已被封印的 p1");
    let (cid, obs) = (pc2.id, pc2.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: Vec::new() });

    // 4) kbolan2 置顶触发回合外玄影 xuanying_use（证明 CardsLost(Hand) 已发帧 + note_cards_lost）
    let (_p2, mut e2) = swd_engine(705_209);
    e2.assign_general_named(1, "swd_duanmeng").unwrap();
    e2.debug_grant(1, "kbolan");
    e2.debug_grant(1, "lingxian");
    advance_to_play(&mut e2, 0);
    for p in 0..4 {
        e2.debug_clear_hand(p);
    }
    e2.state_core_mut().players[1].linked = true;
    let kb_h = find_card(&e2, |k, _| k == CardKind::Tao);
    let kb_sha = find_card(&e2, |k, _| k == CardKind::Sha);
    let kb_shan = find_card(&e2, |k, _| k == CardKind::Shan);
    e2.debug_grant(1, "yishan");
    move_card_to_hand(&mut e2, 1, kb_h);
    move_card_to_hand(&mut e2, 1, kb_shan);
    move_card_to_hand(&mut e2, 0, kb_sha);
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: kb_sha, targets: vec![1] });
    let mut saw_kbolan2 = false;
    let mut saw_xuanying_use = false;
    for _ in 0..16 {
        let Some(ch) = e2.open_choice() else { break };
        let (cid, obs, p) = (ch.id, ch.observer, ch.spec.prompt);
        let sel = if p == "respond_shan" {
            vec![ChoiceOption::Card(kb_shan)]
        } else if p == "lingxian_target" {
            vec![ChoiceOption::Player(3)]
        } else if p == "kbolan_confirm" {
            vec![ChoiceOption::Confirm]
        } else if p == "kbolan2_top" {
            saw_kbolan2 = true;
            vec![ChoiceOption::Card(kb_h)]
        } else if p == "xuanying_use_target" {
            if saw_kbolan2 {
                saw_xuanying_use = true;
                vec![ChoiceOption::Player(2)]
            } else {
                Vec::new()
            }
        } else {
            vec![ChoiceOption::Pass]
        };
        let _ = submit(&mut e2, obs, GameAction::Answer { choice_id: cid, selection: sel });
    }
    let yishan_recorded = e2
        .state_core()
        .player(1)
        .yishan_lost
        .contains(&kb_h);
    assert!(yishan_recorded, "kbolan2 置顶必须记录 note_cards_lost");
    assert!(saw_xuanying_use, "kbolan2 置顶必须发射 CardsLost(Hand) 触发回合外 xuanying_use");

    // 5) yuchen AI 选敌方公开装备而不弃自己手牌（仅用 PolicyInput.view）
    let (_p3, mut e3) = swd_engine(705_210);
    set_roles_lord_rebel(&mut e3);
    e3.debug_grant(0, "yuchen");
    e3.debug_grant(0, "龙胆");
    advance_to_play(&mut e3, 1);
    for p in 0..4 {
        e3.debug_clear_hand(p);
    }
    let bsha = find_card(&e3, |k, s| k == CardKind::Sha && s == Suit::Spade);
    let p1_sha = find_card(&e3, |k, s| k == CardKind::Sha && s == Suit::Heart);
    let p1_eq = find_card(&e3, |k, _| k == CardKind::Renwang);
    let p1_h = find_card(&e3, |k, _| k == CardKind::Tao);
    move_card_to_hand(&mut e3, 0, bsha);
    move_card_to_hand(&mut e3, 1, p1_sha);
    move_card_to_hand(&mut e3, 1, p1_h);
    equip_slot_card(&mut e3, 1, p1_eq);
    let _ = submit(&mut e3, 1, GameAction::PlayCard { card: p1_sha, targets: vec![0] });
    let mut saw_yuchen_disc = false;
    for _ in 0..8 {
        let Some(ch) = e3.open_choice() else { break };
        let (cid, obs, p) = (ch.id, ch.observer, ch.spec.prompt);
        if p == "yuchen_discard" {
            saw_yuchen_disc = true;
            assert_eq!(obs, 0, "yuchen_discard 询问者为 p0");
            let act_prompt = ai_answer_once(&mut e3);
            assert_eq!(act_prompt.as_deref(), Some("yuchen_discard"));
            break;
        }
        let sel = if p == "shan_ask" || p == "respond_shan" {
            vec![ChoiceOption::Card(bsha)]
        } else if p == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
        } else if p == "yuchen_target" {
            vec![ChoiceOption::Player(1)]
        } else {
            vec![ChoiceOption::Pass]
        };
        let _ = submit(&mut e3, obs, GameAction::Answer { choice_id: cid, selection: sel });
    }
    assert!(saw_yuchen_disc, "yuchen_discard 必须打开");
    assert!(!e3.debug_equips(1).contains(&p1_eq), "yuchen AI 优先弃敌方公开装备 p1_eq");

    // 6) huanxing 二次发动替换前次授予技能且更新 prev_gid；受伤触发 SwdHuanxing2 清除授予与 prev_gid
    let (_p4, mut e4) = swd_engine(705_211);
    e4.debug_grant(0, "huanxing");
    e4.debug_set_general(1, "张飞");
    e4.debug_set_general(2, "曹操");
    advance_to_play(&mut e4, 0);
    for p in 0..4 {
        e4.debug_clear_hand(p);
    }
    let hx_d1 = find_card(&e4, |k, _| k == CardKind::Tao);
    let hx_d2 = find_card(&e4, |k, _| k == CardKind::Shan);
    let hx_d3 = find_card(&e4, |k, _| k == CardKind::Jiu);
    move_card_to_hand(&mut e4, 0, hx_d1);
    move_card_to_hand(&mut e4, 0, hx_d2);
    move_card_to_hand(&mut e4, 0, hx_d3);
    let keep = [hx_d1, hx_d2, hx_d3];
    let opts1 = pick_huanxing_round(&mut e4, 1, hx_d1, &keep);
    assert!(opts1.contains(&ChoiceOption::Player(1)), "首轮可选张飞 p1");
    assert!(e4.debug_has_skill(0, "咆哮"), "首轮幻形获得咆哮");
    advance_to_play(&mut e4, 0);
    let opts2 = pick_huanxing_round(&mut e4, 2, hx_d2, &keep);
    assert!(!opts2.contains(&ChoiceOption::Player(1)), "未受伤前次目标 p1 受 prev_gid 限制");
    assert!(opts2.contains(&ChoiceOption::Player(2)), "次轮可选曹操 p2");
    assert!(!e4.debug_has_skill(0, "咆哮"), "幻形二次发动必须替换前次授予技能");
    assert!(e4.debug_has_skill(0, "奸雄"), "次轮幻形获得奸雄");
    let _ = e4.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    drain_pass_or_first(&mut e4);
    assert!(!e4.debug_has_skill(0, "奸雄"), "受伤触发 huanxing2 移除授予技能");
    advance_to_play(&mut e4, 0);
    assert!(e4.debug_hand(0).contains(&hx_d3), "before opts3 hand={:?}", e4.debug_hand(0));
    let opts3 = pick_huanxing_round(&mut e4, 2, hx_d3, &keep);
    assert!(!opts3.is_empty(), "opts3 empty! turn=({},{:?}) open={:?}",
        e4.turn().current,
        e4.turn().phase,
        e4.open_choice().map(|c| (c.observer, c.spec.prompt)));
    assert!(opts3.contains(&ChoiceOption::Player(2)), "huanxing2 清除 prev_gid 后可再次选中 p2: {opts3:?}");
}

fn pick_huanxing_round(e4: &mut Engine, target_seat: u8, disc_card: u32, keep: &[u32]) -> Vec<ChoiceOption> {
    for _ in 0..80 {
        if let Some(ch) = e4.open_choice() {
            let (cid, obs, p) = (ch.id, ch.observer, ch.spec.prompt);
            let sel = if p == "trigger_confirm" && obs == 0 {
                vec![ChoiceOption::Confirm]
            } else if p == "huanxing_discard" {
                vec![ChoiceOption::Card(disc_card)]
            } else if p == "huanxing_target" {
                let target_opts = ch.spec.options.clone();
                let _ = submit(e4, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(target_seat)] });
                return target_opts;
            } else if p.starts_with("discard") {
                ch.spec
                    .options
                    .iter()
                    .filter(|o| !matches!(o, ChoiceOption::Card(c) if keep.contains(c)))
                    .take(ch.spec.min)
                    .cloned()
                    .collect()
            } else {
                vec![ChoiceOption::Pass]
            };
            let _ = submit(e4, obs, GameAction::Answer { choice_id: cid, selection: sel });
        } else {
            for p_seat in 1..4 {
                tx_clear_if_discard(e4, p_seat);
            }
            let cur = e4.turn().current;
            let _ = submit(e4, cur, GameAction::NextPhase);
        }
    }
    Vec::new()
}

fn tx_clear_if_discard(e: &mut Engine, p: u8) {
    if e.turn().current == p && e.turn().phase == engine_native::event::Phase::Discard {
        e.debug_clear_hand(p);
    }
}

