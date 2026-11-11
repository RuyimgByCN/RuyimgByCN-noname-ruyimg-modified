//! U07 `玩点论杀`（`archive/upstream/noname/character/wudian.js` + `card/wudian.js`）
//! Item 3 六维真实对局事件路径测试：
//! ① 触发时机与否定条件
//! ② 目标/选牌过滤与数量上限
//! ③ 玩家选择分支（目标 A vs B、卡牌 X vs Y、`allow_cancel: true` 取消分支）
//! ④ 多步状态变更（体力、手牌/装备/判定/标记/牌堆、技能得失、阶段提前结束、距离与目标上限修正）
//! ⑤ 挂起窗口（`PendingChoice`）中途存档编码/解码恢复后继续应答的 `state_hash` 与事件日志一致性
//! ⑥ AI `plan_action` 在挂起窗口内不读取未公开信息完成合法决策

use super::*;
use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardDef, EquipSlot, Faction};
use engine_native::machine::LogEvent;
use engine_native::state::{GameMode, Role};

/// 构造包含全部 36 项（9 将 24 技 3 卡）的 U07 玩点论杀完整扩展包。
fn wudian_full_pack() -> ExtensionPack {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "wudian")
        .expect("wudian installed")
        .clone();
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .expect("cards.json readable");
    let mut pack = bridge(&json);
    content_native_loader::attach_formal_package_upstream_rules(
        "upstream-noname",
        "wudian",
        &mut pack,
        &[],
    );
    if !pack.cards.iter().any(|c| c.name == "diaobingqianjiang") {
        pack.cards
            .push(CardDef::basic("diaobingqianjiang", CardKind::Wuzhong));
    }
    pack
}

fn setup_u07(players: u8) -> (Engine, ExtensionPack) {
    let pack = wudian_full_pack();
    let mut e =
        Engine::new_with_extensions(7701, players as usize, &[&pack]).expect("u07 engine");
    for _ in 0..80 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            submit(
                &mut e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            );
            continue;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
    }
    for p in 0..players {
        e.debug_clear_hand(p);
    }
    (e, pack)
}

fn find_deck_card_by_kind(e: &Engine, kind: CardKind, nth: usize) -> u32 {
    let snap = e.save();
    snap.core
        .deck
        .iter()
        .copied()
        .filter(|c| e.catalog().card_kind(*c) == kind)
        .nth(nth)
        .unwrap_or_else(|| panic!("no deck card of kind {kind:?} #{nth}"))
}

fn find_deck_card_by_slot(e: &Engine, slot: EquipSlot, nth: usize) -> u32 {
    let snap = e.save();
    snap.core
        .deck
        .iter()
        .copied()
        .filter(|c| e.catalog().card_slot_of(&snap.core, *c) == Some(slot))
        .nth(nth)
        .unwrap_or_else(|| panic!("no deck card of slot {slot:?} #{nth}"))
}

fn find_deck_card_by_name_substr(e: &Engine, substr: &str, nth: usize) -> u32 {
    let snap = e.save();
    snap.core
        .deck
        .iter()
        .copied()
        .filter(|c| e.catalog().cards[*c as usize].name.contains(substr))
        .nth(nth)
        .unwrap_or_else(|| panic!("no deck card matching {substr} #{nth}"))
}

/// 维度⑤ 通用断言：在挂起窗口 `PendingChoice` 中途执行 `encode_archive` -> `decode_archive` ->
/// `Engine::restore_with_extensions`，对原局与恢复局提交相同应答，断言 `state_hash()` 与 `log()` 完全一致。
fn assert_archive_roundtrip_and_answer(
    e: &mut Engine,
    pack: &ExtensionPack,
    selection: Vec<ChoiceOption>,
) {
    let c = e.open_choice().cloned().expect("pending choice open");
    let log_before = e.log().len();
    let raw = encode_archive(&e.save());
    let decoded = decode_archive(&raw).expect("decode archive");
    let mut restored = Engine::restore_with_extensions(&decoded, &[pack]).expect("restore archive");
    assert_eq!(e.state_hash(), restored.state_hash(), "恢复前 state_hash 一致");
    let out_orig = submit(
        e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection: selection.clone(),
        },
    );
    let out_rest = submit(
        &mut restored,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection,
        },
    );
    assert_eq!(
        std::mem::discriminant(&out_orig),
        std::mem::discriminant(&out_rest),
        "原局与恢复局提交结果类型一致"
    );
    assert_eq!(
        e.state_hash(),
        restored.state_hash(),
        "挂起窗口存档恢复后应答 state_hash 一致"
    );
    assert_eq!(
        &e.log()[log_before..],
        restored.log(),
        "挂起窗口存档恢复后应答事件序列一致"
    );
}

/// 维度⑥ 通用断言：AI 在当前 `PendingChoice` 窗口下完成 `plan_action`，且翻转第三方隐藏身份不影响决策。
fn assert_ai_pending_choice_no_leak(
    e: &Engine,
    pack: &ExtensionPack,
    hidden_seat: u8,
) -> Vec<ChoiceOption> {
    let c = e.open_choice().expect("pending choice for AI");
    let obs = c.observer;
    let planner = BoundedPlanner::new(16);
    let cmd_a = e.plan_action(obs, &planner);
    let GameAction::Answer {
        choice_id,
        selection: sel_a,
    } = cmd_a.action
    else {
        panic!("expected GameAction::Answer from AI, got {:?}", cmd_a.action);
    };
    assert_eq!(choice_id, c.id);

    // 翻转未公开牌堆顺序（以及身份局下非主公且非观察者的隐藏身份），验证 AI 不依赖未公开暗信息
    let mut snap_b = e.save();
    snap_b.core.deck.reverse();
    if let GameMode::Identity { ref mut roles } = snap_b.core.mode {
        let hidden: Vec<usize> = (1..roles.len())
            .filter(|&s| s != obs as usize && s >= hidden_seat as usize)
            .collect();
        if hidden.len() >= 2 {
            roles.swap(hidden[0], hidden[1]);
        }
    }
    let e_b = Engine::restore_with_extensions(&snap_b, &[pack]).expect("restore hidden role flip");
    let cmd_b = e_b.plan_action(obs, &planner);
    let GameAction::Answer {
        selection: sel_b, ..
    } = cmd_b.action
    else {
        panic!("expected Answer");
    };
    assert_eq!(
        sel_a, sel_b,
        "AI 决策不得读取第三方未公开身份信息 (prompt={})",
        c.spec.prompt
    );
    sel_a
}

// ===========================================================================
// 1. 夺气（duoqi）六维真实对局测试
// ===========================================================================

#[test]
fn u07_duoqi_six_dimension_real_match() {
    let (mut e, pack) = setup_u07(4);
    e.debug_grant(1, "zhucheng");
    e.debug_grant(1, "duoqi");
    e.debug_grant(0, "choudu");
    let choudu_def = e.catalog().skill_by_name("choudu").unwrap();

    let zc_a = find_deck_card_by_kind(&e, CardKind::Sha, 0);
    let zc_b = find_deck_card_by_kind(&e, CardKind::Shan, 0);
    let p1_discard = find_deck_card_by_kind(&e, CardKind::Tao, 0);
    let p0_discard1 = find_deck_card_by_kind(&e, CardKind::Sha, 1);
    let p0_discard2 = find_deck_card_by_kind(&e, CardKind::Sha, 2);

    // ① 否定条件 A：筑城牌堆为空时，其他角色在出牌阶段弃置牌不触发夺气
    {
        let mut snap = e.save();
        snap.core.turn.current = 0;
        snap.core.turn.phase = Phase::Play;
        snap.core.deck.retain(|c| *c != p0_discard1);
        snap.core.player_mut(0).hand.push(p0_discard1);
        let mut e_empty = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();
        submit(
            &mut e_empty,
            0,
            GameAction::ActivateSkill { skill: choudu_def },
        );
        answer_open(&mut e_empty, vec![ChoiceOption::Card(p0_discard1)]);
        answer_open(&mut e_empty, vec![ChoiceOption::Player(0)]);
        if let Some(c) = e_empty.open_choice() {
            assert_ne!(c.spec.prompt, "wudian_duoqi_card");
        }
    }

    // ① 否定条件 B：拥有者（1 号）自己在弃牌阶段弃牌，不触发夺气
    {
        let mut snap = e.save();
        snap.core.turn.current = 1;
        snap.core.turn.phase = Phase::Play;
        snap.core.deck.retain(|c| {
            *c != zc_a && *c != zc_b && *c != p1_discard && *c != p0_discard1
        });
        snap.core.player_mut(1).zhucheng = vec![zc_a, zc_b];
        snap.core.player_mut(1).hp = 1;
        snap.core.player_mut(1).hand = vec![p1_discard, p0_discard1];
        let mut e_self = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();
        submit(&mut e_self, 1, GameAction::NextPhase);
        let c = e_self.open_choice().expect("discard choice");
        assert_eq!(c.spec.prompt, "discard_to_limit");
        answer_open(&mut e_self, vec![ChoiceOption::Card(p1_discard)]);
        if let Some(after_c) = e_self.open_choice() {
            assert_ne!(after_c.spec.prompt, "wudian_duoqi_card");
        }
    }

    // ②/③/④/⑤/⑥ 正向全路径：1 号（反贼）有 2 张「筑」牌 [zc_a, zc_b]，0 号（主公，公开身份）在出牌阶段发动筹度弃置 p0_discard1
    e.debug_set_identity_roles(vec![Role::Lord, Role::Rebel, Role::Loyalist, Role::Traitor]);
    let mut snap = e.save();
    snap.core.turn.current = 0;
    snap.core.turn.phase = Phase::Play;
    snap.core.deck.retain(|c| {
        *c != zc_a && *c != zc_b && *c != p0_discard1 && *c != p0_discard2
    });
    snap.core.player_mut(1).zhucheng = vec![zc_a, zc_b];
    snap.core.player_mut(0).hand = vec![p0_discard1, p0_discard2];
    e = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();

    submit(&mut e, 0, GameAction::ActivateSkill { skill: choudu_def });
    answer_open(&mut e, vec![ChoiceOption::Card(p0_discard1)]);
    answer_open(&mut e, vec![ChoiceOption::Player(0)]);

    // 此时 0 号弃置 p0_discard1 触发 1 号夺气挂起窗口 wudian_duoqi_card
    let c = e.open_choice().expect("duoqi choice").clone();
    assert_eq!(c.spec.prompt, "wudian_duoqi_card");
    assert_eq!(c.observer, 1);
    assert!(c.spec.allow_cancel, "夺气必须允许取消");
    assert_eq!(
        c.spec.options,
        vec![ChoiceOption::Card(zc_a), ChoiceOption::Card(zc_b)],
        "② 选牌过滤严格等于筑城牌堆"
    );

    // ⑥ AI 在挂起窗口内决策：1 号为反贼、0 号为主公（敌方），AI 选择移去一张筑牌终止其出牌阶段；且不依赖 2/3 号隐藏身份
    let ai_sel = assert_ai_pending_choice_no_leak(&e, &pack, 2);
    assert_eq!(ai_sel, vec![ChoiceOption::Card(zc_a)]);

    // ⑥ AI 友方分支：若 1 号为忠臣（与 0 号主公同阵营），AI 选择取消（vec![]）保留筑牌且不打断主公出牌阶段
    {
        let mut e_ally = Engine::restore_with_extensions(&e.save(), &[&pack]).unwrap();
        e_ally.debug_set_identity_roles(vec![
            Role::Lord,
            Role::Loyalist,
            Role::Rebel,
            Role::Traitor,
        ]);
        let planner = BoundedPlanner::new(16);
        let cmd_ally = e_ally.plan_action(1, &planner);
        assert_eq!(
            cmd_ally.action,
            GameAction::Answer {
                choice_id: c.id,
                selection: vec![],
            }
        );
    }

    // ③ 取消分支（selection: vec![]）：保留两张筑牌，不设置 skip_play_phase
    {
        let mut e_cancel = Engine::restore_with_extensions(&e.save(), &[&pack]).unwrap();
        answer_open(&mut e_cancel, vec![]);
        assert_eq!(e_cancel.state_core().player(1).zhucheng, vec![zc_a, zc_b]);
        assert!(!e_cancel.state_core().turn.skip_play_phase);
    }

    // ③ 选牌 B 分支（Card(zc_b)）：移去 zc_b，保留 zc_a，设置 skip_play_phase
    {
        let mut e_pick_b = Engine::restore_with_extensions(&e.save(), &[&pack]).unwrap();
        answer_open(&mut e_pick_b, vec![ChoiceOption::Card(zc_b)]);
        assert_eq!(e_pick_b.state_core().player(1).zhucheng, vec![zc_a]);
        assert!(e_pick_b.state_core().discard.contains(&zc_b));
        assert!(e_pick_b.state_core().turn.skip_play_phase);
    }

    // ⑤ 挂起窗口中途存档恢复 + 选牌 A 分支（Card(zc_a)）
    assert_archive_roundtrip_and_answer(&mut e, &pack, vec![ChoiceOption::Card(zc_a)]);
    assert_eq!(e.state_core().player(1).zhucheng, vec![zc_b]);
    assert!(e.state_core().discard.contains(&zc_a));
    assert!(e.state_core().turn.skip_play_phase, "④ 出牌阶段被夺气提前终止");
}

// ===========================================================================
// 2. 诱战（youzhan）与 诱敌深入（youdishenru / g_youdishenru）六维真实对局测试
// ===========================================================================

#[test]
fn u07_youzhan_and_youdishenru_six_dimension_real_match() {
    let (mut e, pack) = setup_u07(5);
    e.debug_grant(0, "youzhan");

    let sha1 = find_deck_card_by_kind(&e, CardKind::Sha, 0);
    let sha2 = find_deck_card_by_kind(&e, CardKind::Sha, 1);
    let shan_target = find_deck_card_by_kind(&e, CardKind::Shan, 0);
    let sanjian = find_deck_card_by_name_substr(&e, "sanjian", 0);
    let armor_hand = find_deck_card_by_slot(&e, EquipSlot::Armor, 0);

    // ① 否定条件 A：5 人局中 1 号杀相邻 2 号（0 号到 2 号距离为 2 > 1），0 号诱战不触发，直接进入 2 号闪响应
    {
        let mut snap = e.save();
        snap.core.turn.current = 1;
        snap.core.turn.phase = Phase::Play;
        snap.core
            .deck
            .retain(|c| *c != sha1 && *c != armor_hand && *c != shan_target);
        snap.core.player_mut(0).hand.push(armor_hand);
        snap.core.player_mut(1).hand.push(sha1);
        snap.core.player_mut(2).hand.push(shan_target);
        let mut e_far = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();
        submit(
            &mut e_far,
            1,
            GameAction::PlayCard {
                card: sha1,
                targets: vec![2],
            },
        );
        let c = e_far.open_choice().expect("respond_shan on seat 2");
        assert_eq!(c.spec.prompt, "respond_shan", "距离 > 1 时诱战不得触发");
        assert_eq!(c.observer, 2);
    }

    // ① 否定条件 B：0 号手牌和装备区均无装备牌时，1 号杀 0 号不触发诱战
    {
        let mut snap = e.save();
        snap.core.turn.current = 1;
        snap.core.turn.phase = Phase::Play;
        snap.core.deck.retain(|c| *c != sha1 && *c != shan_target);
        snap.core.player_mut(0).hand.push(shan_target);
        snap.core.player_mut(1).hand.push(sha1);
        let mut e_no_eq = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();
        submit(
            &mut e_no_eq,
            1,
            GameAction::PlayCard {
                card: sha1,
                targets: vec![0],
            },
        );
        let c = e_no_eq.open_choice().expect("respond_shan on seat 0");
        assert_eq!(c.spec.prompt, "respond_shan", "无装备牌时诱战不得触发");
    }

    // ②/③/④/⑤/⑥ 诱战正向全路径：0 号装备区有 sanjian（带装备技 wd_sanjian），手牌有 armor_hand 与 shan_target；
    // 1 号手牌有 sha1 与备用 sha2，对 0 号出杀
    e.debug_grant(0, "wd_sanjian");
    let sanjian_skill = e.catalog().skill_by_name("wd_sanjian").expect("wd_sanjian");
    let mut snap = e.save();
    snap.core.turn.current = 1;
    snap.core.turn.phase = Phase::Play;
    snap.core.deck.retain(|c| {
        *c != sha1 && *c != sha2 && *c != sanjian && *c != armor_hand && *c != shan_target
    });
    snap.core.player_mut(0).equips.push(sanjian);
    snap.core.player_mut(0).hand = vec![armor_hand, shan_target];
    snap.core.player_mut(1).hand = vec![sha1, sha2];
    e = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();

    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![0],
        },
    );
    let c = e.open_choice().expect("wudian_youzhan_discard");
    assert_eq!(c.spec.prompt, "wudian_youzhan_discard");
    assert_eq!(c.observer, 0);
    assert!(c.spec.allow_cancel);
    assert_eq!(
        c.spec.options,
        vec![ChoiceOption::Card(armor_hand), ChoiceOption::Card(sanjian)],
        "② 诱战可选手牌装备与装备区装备（he），过滤非装备手牌"
    );

    // ⑥ Stage 1 AI 决策与不读暗信息验证
    let ai_sel_stage1 = assert_ai_pending_choice_no_leak(&e, &pack, 3);
    assert!(
        ai_sel_stage1 == vec![ChoiceOption::Card(armor_hand)]
            || ai_sel_stage1 == vec![ChoiceOption::Card(sanjian)]
    );

    // ③ Stage 1 取消分支（vec![]）：杀继续正常结算，进入 respond_shan
    {
        let mut e_cancel = Engine::restore_with_extensions(&e.save(), &[&pack]).unwrap();
        answer_open(&mut e_cancel, vec![]);
        let next_c = e_cancel.open_choice().expect("respond_shan after cancel");
        assert_eq!(next_c.spec.prompt, "respond_shan");
        assert_eq!(e_cancel.state_core().player(0).equips, vec![sanjian]);
    }

    // ③ Stage 1 手牌装备分支（Card(armor_hand)）：弃手牌护具，保留已装备的 sanjian
    {
        let mut e_hand_eq = Engine::restore_with_extensions(&e.save(), &[&pack]).unwrap();
        answer_open(&mut e_hand_eq, vec![ChoiceOption::Card(armor_hand)]);
        assert!(e_hand_eq.state_core().discard.contains(&armor_hand));
        assert_eq!(e_hand_eq.state_core().player(0).equips, vec![sanjian]);
        let next_c = e_hand_eq.open_choice().expect("wudian_youdishenru_sha");
        assert_eq!(next_c.spec.prompt, "wudian_youdishenru_sha");
        assert_eq!(next_c.observer, 1);
    }

    // ⑤ Stage 1 挂起窗口存档恢复 + 弃置装备区 sanjian（验证卸装同步卸载装备技）
    assert_archive_roundtrip_and_answer(&mut e, &pack, vec![ChoiceOption::Card(sanjian)]);
    assert!(e.state_core().player(0).equips.is_empty());
    assert!(
        !e.state_core().has_skill(0, sanjian_skill),
        "④ 弃置装备区武器同步卸载其装备技"
    );
    assert!(e.state_core().discard.contains(&sanjian));
    assert!(e.state_core().discard.contains(&sha1));

    // Stage 2: 1 号面临 wudian_youdishenru_sha（手牌有 sha2，可选打出反击杀或取消受 1 点伤害）
    let c2 = e.open_choice().expect("wudian_youdishenru_sha");
    assert_eq!(c2.spec.prompt, "wudian_youdishenru_sha");
    assert_eq!(c2.observer, 1);
    assert!(c2.spec.allow_cancel);
    assert_eq!(c2.spec.options, vec![ChoiceOption::Card(sha2)]);

    // ⑥ Stage 2 AI 决策验证
    let ai_sel_stage2 = assert_ai_pending_choice_no_leak(&e, &pack, 3);
    assert_eq!(ai_sel_stage2, vec![ChoiceOption::Card(sha2)]);

    // ③ Stage 2 取消分支（vec![]）：0 号对 1 号造成 1 点伤害
    {
        let mut e_cancel2 = Engine::restore_with_extensions(&e.save(), &[&pack]).unwrap();
        let hp_before = e_cancel2.debug_hp(1);
        answer_open(&mut e_cancel2, vec![]);
        assert_eq!(e_cancel2.debug_hp(1), hp_before - 1, "取消反击杀受 1 点伤害");
    }

    // ⑤ Stage 2 挂起窗口存档恢复 + 选择打出反击杀 Card(sha2)
    assert_archive_roundtrip_and_answer(&mut e, &pack, vec![ChoiceOption::Card(sha2)]);
    // 0 号手牌仍有 armor_hand，反击杀再次触发 0 号诱战询问；0 号选择取消后进入 respond_shan
    let c_yz2 = e.open_choice().expect("youzhan on counter sha");
    assert_eq!(c_yz2.spec.prompt, "wudian_youzhan_discard");
    answer_open(&mut e, vec![]);
    let c3 = e.open_choice().expect("counter sha triggers respond_shan on seat 0");
    assert_eq!(c3.spec.prompt, "respond_shan");
    assert_eq!(c3.observer, 0);

    // 诱敌深入实体牌（CardKind::Youdishenru / g_youdishenru）在 respond_shan 中打出后的 Stage 2 分支验证
    {
        let (mut e_yd, _) = setup_u07(3);
        let s1 = find_deck_card_by_kind(&e_yd, CardKind::Sha, 0);
        let s2 = find_deck_card_by_kind(&e_yd, CardKind::Sha, 1);
        let shan1 = find_deck_card_by_kind(&e_yd, CardKind::Shan, 0);
        let yd = find_deck_card_by_kind(&e_yd, CardKind::Youdishenru, 0);
        let mut snap_yd = e_yd.save();
        snap_yd
            .core
            .deck
            .retain(|c| *c != s1 && *c != s2 && *c != shan1 && *c != yd);
        snap_yd.core.player_mut(0).hand = vec![s1, s2];
        snap_yd.core.player_mut(1).hand = vec![yd, shan1];
        e_yd = Engine::restore_with_extensions(&snap_yd, &[&pack]).unwrap();

        submit(
            &mut e_yd,
            0,
            GameAction::PlayCard {
                card: s1,
                targets: vec![1],
            },
        );
        let cr = e_yd.open_choice().expect("respond_shan with youdishenru");
        assert!(cr.spec.options.contains(&ChoiceOption::Card(yd)));
        assert_archive_roundtrip_and_answer(&mut e_yd, &pack, vec![ChoiceOption::Card(yd)]);

        let c_sha = e_yd.open_choice().expect("wudian_youdishenru_sha after card");
        assert_eq!(c_sha.spec.prompt, "wudian_youdishenru_sha");
        assert_eq!(c_sha.observer, 0);
        let _ = assert_ai_pending_choice_no_leak(&e_yd, &pack, 2);

        // 分支 A：0 号取消 -> 1 号对 0 号造成 1 点伤害
        let mut e_yd_cancel = Engine::restore_with_extensions(&e_yd.save(), &[&pack]).unwrap();
        let hp0 = e_yd_cancel.debug_hp(0);
        answer_open(&mut e_yd_cancel, vec![]);
        assert_eq!(e_yd_cancel.debug_hp(0), hp0 - 1);

        // 分支 B：0 号存档恢复后打出 s2 反击 1 号
        assert_archive_roundtrip_and_answer(&mut e_yd, &pack, vec![ChoiceOption::Card(s2)]);
        let c_after = e_yd.open_choice().expect("1 号再次响应反击杀");
        assert_eq!(c_after.spec.prompt, "respond_shan");
        assert_eq!(c_after.observer, 1);
    }
}

// ===========================================================================
// 3. 筹度（choudu）与 调兵遣将（diaobingqianjiang）六维真实对局测试
// ===========================================================================

#[test]
fn u07_choudu_and_diaobingqianjiang_six_dimension_real_match() {
    let (mut e, pack) = setup_u07(3);
    let choudu_def = e.catalog().skill_by_name("choudu").expect("choudu");
    e.debug_grant(0, "choudu");

    // ① 否定条件 A：0 号无手牌且无装备时发动筹度被拒绝
    assert!(matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: choudu_def }),
        CommandOutcome::Rejected(_)
    ));

    let hand_c = find_deck_card_by_kind(&e, CardKind::Sha, 0);
    let worn_eq = find_deck_card_by_name_substr(&e, "sanjian", 0);
    let p1_h1 = find_deck_card_by_kind(&e, CardKind::Shan, 0);
    let p1_h2 = find_deck_card_by_kind(&e, CardKind::Tao, 0);
    let p2_h1 = find_deck_card_by_kind(&e, CardKind::Sha, 1);
    let sanjian_skill = e.catalog().skill_by_name("wd_sanjian").unwrap();
    e.debug_grant(0, "wd_sanjian");

    let mut snap = e.save();
    snap.core
        .deck
        .retain(|c| *c != hand_c && *c != worn_eq && *c != p1_h1 && *c != p1_h2 && *c != p2_h1);
    snap.core.player_mut(0).hand = vec![hand_c];
    snap.core.player_mut(0).equips = vec![worn_eq];
    snap.core.player_mut(1).hand = vec![p1_h1, p1_h2];
    snap.core.player_mut(2).hand = vec![p2_h1];
    e = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();

    // Stage 1: wudian_choudu_card
    submit(&mut e, 0, GameAction::ActivateSkill { skill: choudu_def });
    let c1 = e.open_choice().expect("wudian_choudu_card");
    assert_eq!(c1.spec.prompt, "wudian_choudu_card");
    assert!(c1.spec.allow_cancel);
    assert_eq!(
        c1.spec.options,
        vec![ChoiceOption::Card(hand_c), ChoiceOption::Card(worn_eq)],
        "② 筹度可选手牌与装备区（he）"
    );
    let _ = assert_ai_pending_choice_no_leak(&e, &pack, 2);

    // ③ Stage 1 取消分支
    {
        let mut e_cancel1 = Engine::restore_with_extensions(&e.save(), &[&pack]).unwrap();
        answer_open(&mut e_cancel1, vec![]);
        assert!(e_cancel1.open_choice().is_none());
        assert_eq!(e_cancel1.state_core().player(0).equips, vec![worn_eq]);
    }

    // ⑤ Stage 1 存档恢复 + 选择弃置装备区 worn_eq
    assert_archive_roundtrip_and_answer(&mut e, &pack, vec![ChoiceOption::Card(worn_eq)]);

    // Stage 2: wudian_choudu_target
    let c2 = e.open_choice().expect("wudian_choudu_target");
    assert_eq!(c2.spec.prompt, "wudian_choudu_target");
    assert!(c2.spec.allow_cancel);
    assert_eq!(
        c2.spec.options,
        vec![
            ChoiceOption::Player(0),
            ChoiceOption::Player(1),
            ChoiceOption::Player(2)
        ]
    );
    let _ = assert_ai_pending_choice_no_leak(&e, &pack, 2);

    // ③ Stage 2 取消分支：不弃置 worn_eq
    {
        let mut e_cancel2 = Engine::restore_with_extensions(&e.save(), &[&pack]).unwrap();
        answer_open(&mut e_cancel2, vec![]);
        assert!(e_cancel2.open_choice().is_none());
        assert_eq!(e_cancel2.state_core().player(0).equips, vec![worn_eq]);
    }

    // ③ Stage 2 目标分支 B（选择 Player(2) vs Player(1)）
    {
        let mut e_t2 = Engine::restore_with_extensions(&e.save(), &[&pack]).unwrap();
        answer_open(&mut e_t2, vec![ChoiceOption::Player(2)]);
        let cs = e_t2.open_choice().expect("wudian_diaobing_swap for seat 2");
        assert_eq!(cs.spec.prompt, "wudian_diaobing_swap");
        assert_eq!(cs.observer, 2);
    }

    // ⑤ Stage 2 存档恢复 + 选择目标 Player(1)
    assert_archive_roundtrip_and_answer(&mut e, &pack, vec![ChoiceOption::Player(1)]);
    assert!(
        !e.state_core().has_skill(0, sanjian_skill),
        "④ 筹度弃置装备区武器同步卸载其装备技"
    );
    assert!(e.state_core().discard.contains(&worn_eq));
    // 3 名存活角色 -> 亮出 ceil(3/2) == 2 张牌，且 1 号先摸 1 张牌（手牌数由 2 变 3）
    assert_eq!(e.state_core().player(1).hand.len(), 3);
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(1, "u07_diaobing_resolved".to_string()))
            .copied(),
        Some(2),
        "② 亮出牌堆顶 ceil(alive/2)=2 张牌"
    );

    // Stage 3: wudian_diaobing_swap（observer == 1）
    let c3 = e.open_choice().expect("wudian_diaobing_swap");
    assert_eq!(c3.spec.prompt, "wudian_diaobing_swap");
    assert_eq!(c3.observer, 1);
    assert!(c3.spec.allow_cancel);
    assert_eq!(c3.spec.options.len(), 3);
    let _ = assert_ai_pending_choice_no_leak(&e, &pack, 2);

    // ③ Stage 3 取消换牌分支（vec![]）：手牌不变，亮出牌全部进弃牌堆
    {
        let mut e_no_swap = Engine::restore_with_extensions(&e.save(), &[&pack]).unwrap();
        let hand_before = e_no_swap.state_core().player(1).hand.clone();
        answer_open(&mut e_no_swap, vec![]);
        assert!(e_no_swap.open_choice().is_none());
        assert_eq!(e_no_swap.state_core().player(1).hand, hand_before);
    }

    // ③ Stage 3 选牌分支 Y（换 p1_h2）vs 选牌分支 X（换 p1_h1）
    {
        let mut e_swap_y = Engine::restore_with_extensions(&e.save(), &[&pack]).unwrap();
        answer_open(&mut e_swap_y, vec![ChoiceOption::Card(p1_h2)]);
        assert!(!e_swap_y.state_core().player(1).hand.contains(&p1_h2));
        assert!(e_swap_y.state_core().discard.contains(&p1_h2));
    }

    // ⑤ Stage 3 存档恢复 + 选择换出 p1_h1
    assert_archive_roundtrip_and_answer(&mut e, &pack, vec![ChoiceOption::Card(p1_h1)]);
    assert!(e.open_choice().is_none());
    assert!(!e.state_core().player(1).hand.contains(&p1_h1));
    assert!(e.state_core().discard.contains(&p1_h1));
    assert_eq!(e.state_core().player(1).hand.len(), 3);

    // ① 否定条件 B：同回合再次发动筹度（usable: 1）被拒绝
    assert!(matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: choudu_def }),
        CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)
    ));

    // 直接使用实体锦囊牌「调兵遣将」（diaobingqianjiang）指定多目标 [1, 2]
    let diaobing_card = find_deck_card_by_name_substr(&e, "diaobingqianjiang", 0);
    let mut snap_card = e.save();
    snap_card.core.deck.retain(|c| *c != diaobing_card);
    snap_card.core.player_mut(0).hand.push(diaobing_card);
    snap_card.core.player_mut(2).alive = false;
    let mut e_dead = Engine::restore_with_extensions(&snap_card, &[&pack]).unwrap();
    // ① 否定条件 C：指定已阵亡角色被拒绝 IllegalTargets
    assert!(matches!(
        submit(
            &mut e_dead,
            0,
            GameAction::PlayCard {
                card: diaobing_card,
                targets: vec![2],
            }
        ),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ));

    snap_card.core.player_mut(2).alive = true;
    e = Engine::restore_with_extensions(&snap_card, &[&pack]).unwrap();
    let p0_hand_before = e.state_core().player(0).hand.len();
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: diaobing_card,
            targets: vec![1, 2],
        },
    );
    while e
        .open_choice()
        .is_some_and(|c| c.spec.prompt == "nullify_window")
    {
        pass_nullify(&mut e);
    }
    // 使用者 0 号先摸 1 张牌（打出 1 张摸 1 张，手牌数不变），依次向目标 1、2 打开换牌窗口
    assert_eq!(e.state_core().player(0).hand.len(), p0_hand_before);
    let c_t1 = e.open_choice().expect("swap for target 1");
    assert_eq!(c_t1.observer, 1);
    let pick_t1 = c_t1.spec.options[0];
    assert_archive_roundtrip_and_answer(&mut e, &pack, vec![pick_t1]);

    let c_t2 = e.open_choice().expect("swap for target 2");
    assert_eq!(c_t2.observer, 2);
    assert_archive_roundtrip_and_answer(&mut e, &pack, vec![ChoiceOption::Card(p2_h1)]);
    assert!(e.open_choice().is_none());
}

// ===========================================================================
// 4. 割据（geju）与 思计（siji）六维真实对局测试
// ===========================================================================

#[test]
fn u07_geju_and_siji_six_dimension_real_match() {
    let (mut e, pack) = setup_u07(4);
    e.debug_grant(0, "geju");

    // 4 人局座位距离：1 号到 0 号距离 1，2 号到 0 号距离 2，3 号到 0 号距离 1
    let wei = engine_native::content::standard::STANDARD_GENERALS
        .iter()
        .find(|g| g.faction == Faction::Wei)
        .map(|g| g.name.to_string())
        .unwrap();
    let shu = engine_native::content::standard::STANDARD_GENERALS
        .iter()
        .find(|g| g.faction == Faction::Shu)
        .map(|g| g.name.to_string())
        .unwrap();
    e.debug_assemble_general(1, &wei);
    e.debug_assemble_general(2, &shu);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    let sanjian = find_deck_card_by_name_substr(&e, "sanjian", 0);
    let snap = e.save();

    // 分支 A：2 号（Shu，距离 2）无武器（攻击范围 1 < 2，攻击范围内不含 0 号）；1 号（Wei，距离 1）攻击范围含 0 号
    // 割据在 0 号下一回合开始时仅统计 Shu 1 个势力 -> 摸 1 张牌 + 摸牌阶段 2 张 = 3 张
    {
        let mut e_a = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();
        let r0 = e_a.turn().round;
        for _ in 0..64 {
            if e_a.turn().current == 0 && e_a.turn().phase == Phase::Play && e_a.turn().round > r0 {
                break;
            }
            if let Some(c) = e_a.open_choice().cloned() {
                let sel = if c.spec.min > 0 {
                    c.spec.options.iter().take(c.spec.min).copied().collect()
                } else {
                    vec![ChoiceOption::Pass]
                };
                answer_open(&mut e_a, sel);
                continue;
            }
            let actor = e_a.turn().current;
            let _ = submit(&mut e_a, actor, GameAction::NextPhase);
        }
        assert_eq!(
            e_a.state_core().player(0).hand.len(),
            3,
            "2 号攻击范围不含 0 号，割据摸 1 张 + 摸牌阶段 2 张 = 3 张"
        );
    }

    // ① 否定条件：2 号（Shu，距离 2）装备三尖两刃刀（攻击范围 3 >= 2，攻击范围包含 0 号）
    // 此时全场所有其他存活角色攻击范围均包含 0 号 -> 割据不触发（摸 0 张）
    {
        let mut e_b = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();
        e_b.debug_grant(2, "wd_sanjian");
        let mut s_b = e_b.save();
        s_b.core.deck.retain(|c| *c != sanjian);
        s_b.core.player_mut(2).equips.push(sanjian);
        e_b = Engine::restore_with_extensions(&s_b, &[&pack]).unwrap();
        let r0 = e_b.turn().round;
        for _ in 0..64 {
            if e_b.turn().current == 0 && e_b.turn().phase == Phase::Play && e_b.turn().round > r0 {
                break;
            }
            if let Some(c) = e_b.open_choice().cloned() {
                let sel = if c.spec.min > 0 {
                    c.spec.options.iter().take(c.spec.min).copied().collect()
                } else {
                    vec![ChoiceOption::Pass]
                };
                answer_open(&mut e_b, sel);
                continue;
            }
            let actor = e_b.turn().current;
            let _ = submit(&mut e_b, actor, GameAction::NextPhase);
        }
        assert_eq!(
            e_b.state_core().player(0).hand.len(),
            2,
            "2 号装备长武器后攻击范围覆盖 0 号，割据不摸牌（仅摸牌阶段 2 张）"
        );
    }

    // 思计（siji）六维验证：
    // ① 否定条件：出牌阶段弃置杀（如通过筹度）不触发思计
    let (mut e_siji, pack_siji) = setup_u07(3);
    e_siji.debug_grant(0, "siji");
    e_siji.debug_grant(0, "choudu");
    let choudu_def = e_siji.catalog().skill_by_name("choudu").unwrap();
    let sha_a = find_deck_card_by_kind(&e_siji, CardKind::Sha, 0);
    let sha_b = find_deck_card_by_kind(&e_siji, CardKind::Sha, 1);
    let shan_c = find_deck_card_by_kind(&e_siji, CardKind::Shan, 0);
    let mut s_siji = e_siji.save();
    s_siji
        .core
        .deck
        .retain(|c| *c != sha_a && *c != sha_b && *c != shan_c);
    s_siji.core.player_mut(0).hand = vec![sha_a, sha_b, shan_c];
    s_siji.core.player_mut(0).hp = 1;
    e_siji = Engine::restore_with_extensions(&s_siji, &[&pack_siji]).unwrap();

    // 在出牌阶段发动筹度弃置 sha_a -> 思计不得触发
    {
        let mut e_play = Engine::restore_with_extensions(&e_siji.save(), &[&pack_siji]).unwrap();
        submit(
            &mut e_play,
            0,
            GameAction::ActivateSkill { skill: choudu_def },
        );
        answer_open(&mut e_play, vec![ChoiceOption::Card(sha_a)]);
        answer_open(&mut e_play, vec![ChoiceOption::Player(1)]);
        if let Some(c) = e_play.open_choice() {
            assert_ne!(c.spec.prompt, "trigger_confirm", "出牌阶段弃杀不得触发思计");
        }
    }

    // ②/③/④/⑤/⑥ 正向：推进到 0 号弃牌阶段，弃置 2 张杀（sha_a, sha_b）触发思计 trigger_confirm
    submit(&mut e_siji, 0, GameAction::NextPhase);
    let cd = e_siji.open_choice().expect("discard in Phase::Discard");
    assert_eq!(cd.spec.prompt, "discard_to_limit");
    answer_open(
        &mut e_siji,
        vec![ChoiceOption::Card(sha_a), ChoiceOption::Card(sha_b)],
    );

    let cs = e_siji.open_choice().expect("siji trigger_confirm");
    assert_eq!(cs.spec.prompt, "trigger_confirm");
    assert_eq!(cs.observer, 0);
    let _ = assert_ai_pending_choice_no_leak(&e_siji, &pack_siji, 2);

    // ③ 取消分支（Pass）：不摸牌，手牌仅剩 shan_c（1 张）
    {
        let mut e_cancel = Engine::restore_with_extensions(&e_siji.save(), &[&pack_siji]).unwrap();
        answer_open(&mut e_cancel, vec![ChoiceOption::Pass]);
        assert_eq!(e_cancel.state_core().player(0).hand.len(), 1);
    }

    // ⑤ 存档恢复 + 确认分支（Confirm）：摸 2 * 2 = 4 张牌，手牌变为 1 + 4 = 5 张
    assert_archive_roundtrip_and_answer(&mut e_siji, &pack_siji, vec![ChoiceOption::Confirm]);
    assert_eq!(e_siji.state_core().player(0).hand.len(), 5);
}

// ===========================================================================
// 5. 率言（nsshuaiyan）与 亢音（kangyin / kangyin2）六维真实对局测试
// ===========================================================================

#[test]
fn u07_nsshuaiyan_and_kangyin_six_dimension_real_match() {
    let (mut e, pack) = setup_u07(5);
    e.debug_grant(0, "nsshuaiyan");

    let tao0 = find_deck_card_by_kind(&e, CardKind::Tao, 0);
    let tao1 = find_deck_card_by_kind(&e, CardKind::Tao, 1);
    let hand_c = find_deck_card_by_kind(&e, CardKind::Sha, 0);
    let worn_eq = find_deck_card_by_name_substr(&e, "sanjian", 0);
    let sanjian_skill = e.catalog().skill_by_name("wd_sanjian").unwrap();

    // ① 否定条件：0 号（费诗）自己回合内自己吃桃回复体力，不触发率言
    {
        let mut snap0 = e.save();
        snap0.core.deck.retain(|c| *c != tao0);
        snap0.core.player_mut(0).hp = 2;
        snap0.core.player_mut(0).hand = vec![tao0];
        let mut e0 = Engine::restore_with_extensions(&snap0, &[&pack]).unwrap();
        submit(
            &mut e0,
            0,
            GameAction::PlayCard {
                card: tao0,
                targets: vec![],
            },
        );
        assert!(e0.open_choice().is_none(), "费诗自己回合内回复不得触发率言");
    }

    // ②/③/④/⑤/⑥ 率言正向：1 号回合内吃桃回复，1 号持有手牌 hand_c 与装备区 worn_eq
    e.debug_grant(1, "wd_sanjian");
    let mut snap = e.save();
    snap.core.turn.current = 1;
    snap.core.turn.phase = Phase::Play;
    snap.core
        .deck
        .retain(|c| *c != tao1 && *c != hand_c && *c != worn_eq);
    snap.core.player_mut(1).hp = 2;
    snap.core.player_mut(1).hand = vec![tao1, hand_c];
    snap.core.player_mut(1).equips = vec![worn_eq];
    e = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();

    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: tao1,
            targets: vec![],
        },
    );
    // Stage 1: wudian_shuaiyan_choice（observer == 1，Tag(0)=令费诗摸1张，Tag(1)=自己弃置1张牌）
    let c1 = e.open_choice().expect("wudian_shuaiyan_choice");
    assert_eq!(c1.spec.prompt, "wudian_shuaiyan_choice");
    assert_eq!(c1.observer, 1);
    let _ = assert_ai_pending_choice_no_leak(&e, &pack, 3);

    // ③ 分支 Tag(0)：0 号摸 1 张牌，1 号不弃牌
    {
        let mut e_tag0 = Engine::restore_with_extensions(&e.save(), &[&pack]).unwrap();
        answer_open(&mut e_tag0, vec![ChoiceOption::Tag(0)]);
        assert_eq!(e_tag0.state_core().player(0).hand.len(), 1);
        assert_eq!(e_tag0.state_core().player(1).equips, vec![worn_eq]);
    }

    // ⑤ Stage 1 存档恢复 + 选择 Tag(1) 进入弃牌窗口
    assert_archive_roundtrip_and_answer(&mut e, &pack, vec![ChoiceOption::Tag(1)]);

    // Stage 2: wudian_shuaiyan_discard（可选手牌 hand_c 与装备区 worn_eq）
    let c2 = e.open_choice().expect("wudian_shuaiyan_discard");
    assert_eq!(c2.spec.prompt, "wudian_shuaiyan_discard");
    assert_eq!(
        c2.spec.options,
        vec![ChoiceOption::Card(hand_c), ChoiceOption::Card(worn_eq)],
        "② 率言分支 1 必须包含手牌与装备区牌（he）"
    );
    let _ = assert_ai_pending_choice_no_leak(&e, &pack, 3);

    // ⑤ Stage 2 存档恢复 + 选择弃置装备区 worn_eq（验证卸装同步卸载装备技）
    assert_archive_roundtrip_and_answer(&mut e, &pack, vec![ChoiceOption::Card(worn_eq)]);
    assert!(e.state_core().player(1).equips.is_empty());
    assert!(!e.state_core().has_skill(1, sanjian_skill));
    assert!(e.state_core().discard.contains(&worn_eq));

    // -----------------------------------------------------------------------
    // 亢音（kangyin / kangyin2）六维测试
    // -----------------------------------------------------------------------
    let (mut ek, pack_k) = setup_u07(5);
    let kangyin_def = ek.catalog().skill_by_name("kangyin").unwrap();
    ek.debug_grant(0, "kangyin");

    // ① 否定条件：全场其他角色均无牌时不可发动亢音
    assert!(matches!(
        submit(&mut ek, 0, GameAction::ActivateSkill { skill: kangyin_def }),
        CommandOutcome::Rejected(_)
    ));

    let basic_card = find_deck_card_by_kind(&ek, CardKind::Shan, 0);
    let equip_card = find_deck_card_by_slot(&ek, EquipSlot::Armor, 0);
    let sha_multi = find_deck_card_by_kind(&ek, CardKind::Sha, 0);

    let mut sk = ek.save();
    sk.core
        .deck
        .retain(|c| *c != basic_card && *c != equip_card && *c != sha_multi);
    // 0 号体力 3/4，发动亢音失去 1 点体力后 hp=2, max_hp=4 -> 已损失体力值 x = 2
    sk.core.player_mut(0).hp = 3;
    sk.core.player_mut(0).max_hp = 4;
    sk.core.player_mut(0).hand = vec![sha_multi];
    sk.core.player_mut(1).hand = vec![basic_card];
    sk.core.player_mut(2).equips = vec![equip_card];
    ek = Engine::restore_with_extensions(&sk, &[&pack_k]).unwrap();

    submit(&mut ek, 0, GameAction::ActivateSkill { skill: kangyin_def });
    let ck1 = ek.open_choice().expect("wudian_kangyin_target");
    assert_eq!(ck1.spec.prompt, "wudian_kangyin_target");
    assert_eq!(
        ck1.spec.options,
        vec![ChoiceOption::Player(1), ChoiceOption::Player(2)],
        "② 仅过滤有 he 牌的其他存活角色"
    );
    let _ = assert_ai_pending_choice_no_leak(&ek, &pack_k, 4);

    // 分支 A：选 1 号（弃置基本牌 basic_card）-> 打开 wudian_kangyin_draw（allow_cancel: true, max: 2）
    {
        let mut ek_basic = Engine::restore_with_extensions(&ek.save(), &[&pack_k]).unwrap();
        assert_archive_roundtrip_and_answer(&mut ek_basic, &pack_k, vec![ChoiceOption::Player(1)]);
        let ckd = ek_basic.open_choice().expect("wudian_kangyin_discard");
        assert_eq!(ckd.observer, 1);
        let _ = assert_ai_pending_choice_no_leak(&ek_basic, &pack_k, 4);
        assert_archive_roundtrip_and_answer(
            &mut ek_basic,
            &pack_k,
            vec![ChoiceOption::Card(basic_card)],
        );

        let ck_draw = ek_basic.open_choice().expect("wudian_kangyin_draw");
        assert_eq!(ck_draw.spec.prompt, "wudian_kangyin_draw");
        assert!(ck_draw.spec.allow_cancel, "③ 亢音摸牌必须支持取消（可令至多 X 名角色摸牌）");
        assert_eq!(ck_draw.spec.max, 2);
        let _ = assert_ai_pending_choice_no_leak(&ek_basic, &pack_k, 4);

        // ③ 取消摸牌分支（vec![]）：无人摸牌
        let mut ek_draw_cancel =
            Engine::restore_with_extensions(&ek_basic.save(), &[&pack_k]).unwrap();
        answer_open(&mut ek_draw_cancel, vec![]);
        assert_eq!(ek_draw_cancel.state_core().player(0).hand.len(), 1);
        assert_eq!(ek_draw_cancel.state_core().player(3).hand.len(), 0);

        // ⑤ 存档恢复 + 选择 2 名角色 [Player(0), Player(3)] 各摸 1 张牌
        assert_archive_roundtrip_and_answer(
            &mut ek_basic,
            &pack_k,
            vec![ChoiceOption::Player(0), ChoiceOption::Player(3)],
        );
        assert_eq!(ek_basic.state_core().player(0).hand.len(), 2);
        assert_eq!(ek_basic.state_core().player(3).hand.len(), 1);
    }

    // 分支 B：选 2 号（弃置非基本牌 equip_card）-> 获得 kangyin2（距离 -2、出杀目标上限 +2）
    assert_archive_roundtrip_and_answer(&mut ek, &pack_k, vec![ChoiceOption::Player(2)]);
    assert_archive_roundtrip_and_answer(&mut ek, &pack_k, vec![ChoiceOption::Card(equip_card)]);
    assert_eq!(
        ek.state_core().turn.kangyin2,
        Some((0, 2)),
        "④ 弃置非基本牌获得 kangyin2(x=2)"
    );
    // 5 人局中 0 号到 2 号原始距离为 2，经 kangyin2 距离 -2 修正后距离变为 1（最小为 1）
    assert_eq!(
        ek.state_core().computed_distance(ek.catalog(), 0, 2),
        1,
        "④ kangyin2 使 0 号计算与其他角色的距离 -X"
    );
    // 且 0 号无武器可对距离 2 的 2 号与 3 号同时出杀（最多 1 + 2 = 3 名目标）
    let out_sha = submit(
        &mut ek,
        0,
        GameAction::PlayCard {
            card: sha_multi,
            targets: vec![1, 2, 3],
        },
    );
    assert!(
        matches!(out_sha, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "④ kangyin2 允许无武器杀距离 2 的角色且目标上限 +2: {out_sha:?}"
    );
}

// ===========================================================================
// 6. 复仇（fuchou/fuchou2）、立断（liduan）、良策（liangce）、坚壁（jianbi）、屯军（diyjuntun）六维真实对局测试
// ===========================================================================

#[test]
fn u07_fuchou_liduan_liangce_jianbi_diyjuntun_six_dimension_real_match() {
    // -----------------------------------------------------------------------
    // 6A. 复仇（fuchou + fuchou2）：支持交出手牌或装备区牌（he）取消杀，回合结束摸牌反击
    // -----------------------------------------------------------------------
    let (mut e, pack) = setup_u07(3);
    e.debug_grant(1, "fuchou");
    e.debug_grant(1, "fuchou2");
    e.debug_grant(1, "wd_sanjian");

    let sha_c = find_deck_card_by_kind(&e, CardKind::Sha, 0);
    let shan_p0 = find_deck_card_by_kind(&e, CardKind::Shan, 1);
    let p1_hand = find_deck_card_by_kind(&e, CardKind::Shan, 0);
    let p1_eq = find_deck_card_by_name_substr(&e, "sanjian", 0);
    let sanjian_skill = e.catalog().skill_by_name("wd_sanjian").unwrap();

    let mut snap = e.save();
    snap.core
        .deck
        .retain(|c| *c != sha_c && *c != shan_p0 && *c != p1_hand && *c != p1_eq);
    snap.core.player_mut(0).hand = vec![sha_c, shan_p0];
    snap.core.player_mut(1).hand = vec![p1_hand];
    snap.core.player_mut(1).equips = vec![p1_eq];
    e = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();

    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_c,
            targets: vec![1],
        },
    );
    let cf = e.open_choice().expect("wudian_fuchou_cancel");
    assert_eq!(cf.spec.prompt, "wudian_fuchou_cancel");
    assert_eq!(cf.observer, 1);
    assert!(cf.spec.allow_cancel);
    assert_eq!(
        cf.spec.options,
        vec![ChoiceOption::Card(p1_hand), ChoiceOption::Card(p1_eq)],
        "② 复仇必须可选手牌与装备区（he）"
    );
    let _ = assert_ai_pending_choice_no_leak(&e, &pack, 2);

    // ③ 取消分支（vec![]）：进入普通闪响应，不记录复仇标记
    {
        let mut e_fc_cancel = Engine::restore_with_extensions(&e.save(), &[&pack]).unwrap();
        answer_open(&mut e_fc_cancel, vec![]);
        assert_eq!(e_fc_cancel.open_choice().unwrap().spec.prompt, "respond_shan");
        assert!(e_fc_cancel.state_core().player(1).fuchou_marked.is_empty());
    }

    // ⑤ 存档恢复 + 选择交出装备区武器 p1_eq：卸载装备技、0 号获得 p1_eq 与 sha_c、记录复仇标记
    assert_archive_roundtrip_and_answer(&mut e, &pack, vec![ChoiceOption::Card(p1_eq)]);
    assert!(e.state_core().player(1).equips.is_empty());
    assert!(!e.state_core().has_skill(1, sanjian_skill));
    assert!(e.state_core().player(0).hand.contains(&p1_eq));
    assert!(e.state_core().player(0).hand.contains(&sha_c));
    assert_eq!(e.state_core().player(1).fuchou_marked, vec![0]);

    // 推进至回合结束窗口（Play -> Discard -> End -> TurnEndWindow），触发 fuchou2
    let p1_hand_before_settle = e.state_core().player(1).hand.len();
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    let c_counter = e.open_choice().expect("virtual sha from fuchou2");
    assert_eq!(c_counter.spec.prompt, "respond_shan");
    assert_eq!(c_counter.observer, 0);
    assert_eq!(
        e.state_core().player(1).hand.len(),
        p1_hand_before_settle + 1,
        "④ fuchou2 结算令 1 号摸 1 张牌并对 0 号视为出杀"
    );
    assert!(e.state_core().player(1).fuchou_marked.is_empty());

    // -----------------------------------------------------------------------
    // 6B. 立断（liduan）：其他角色于其回合外获得装备牌进入手牌时，可选交出 1 张手牌或取消并装备之
    // -----------------------------------------------------------------------
    let (mut el, pack_l) = setup_u07(3);
    el.debug_grant(0, "liduan");

    let c_discard = find_deck_card_by_kind(&el, CardKind::Sha, 0);
    let eq_gain = find_deck_card_by_name_substr(&el, "sanjian", 0);
    let p1_give = find_deck_card_by_kind(&el, CardKind::Tao, 0);
    let mut sl = el.save();
    sl.core
        .deck
        .retain(|c| *c != c_discard && *c != eq_gain && *c != p1_give);
    sl.core.player_mut(0).hand = vec![c_discard, eq_gain];
    sl.core.player_mut(1).hand = vec![p1_give];
    el = Engine::restore_with_extensions(&sl, &[&pack_l]).unwrap();

    // ① 负向边界 A：获得非装备牌（c_discard）不触发立断
    el.debug_give_hand_card(0, 1, c_discard);
    assert!(
        el.open_choice().is_none(),
        "① 获得非装备牌进入手牌不得触发立断"
    );
    // 将 c_discard 移回弃牌堆，恢复 1 号手牌仅含 [p1_give]
    el.debug_give_hand_card(1, 0, c_discard);

    // ① 负向边界 B：当前回合角色（0 号）获得装备牌不触发立断
    el.state_core_mut().players[0].hand.retain(|c| *c != eq_gain);
    el.state_core_mut().players[1].hand.push(eq_gain);
    el.debug_give_hand_card(1, 0, eq_gain);
    assert!(
        el.open_choice().is_none(),
        "① 当前回合角色获得装备牌不得触发立断"
    );

    // 正向触发：1 号于其回合外（0 号回合中）获得装备牌 eq_gain 进入手牌 -> 触发 0 号立断 trigger_confirm
    el.debug_give_hand_card(0, 1, eq_gain);
    let cl_confirm = el.open_choice().expect("liduan trigger_confirm");
    assert_eq!(cl_confirm.spec.prompt, "trigger_confirm");
    assert_eq!(cl_confirm.observer, 0);
    assert_archive_roundtrip_and_answer(&mut el, &pack_l, vec![ChoiceOption::Confirm]);

    // 打开 wudian_liduan_give（observer == 1, allow_cancel: true）
    let cl_give = el.open_choice().expect("wudian_liduan_give");
    assert_eq!(cl_give.spec.prompt, "wudian_liduan_give");
    assert_eq!(cl_give.observer, 1);
    assert!(cl_give.spec.allow_cancel);
    assert_eq!(
        cl_give.spec.options,
        vec![ChoiceOption::Card(p1_give), ChoiceOption::Card(eq_gain)]
    );
    let _ = assert_ai_pending_choice_no_leak(&el, &pack_l, 2);

    // ③ 分支 A（选择交出手牌 Card(p1_give)）：0 号获得 p1_give，eq_gain 留在 1 号手牌
    {
        let mut el_give = Engine::restore_with_extensions(&el.save(), &[&pack_l]).unwrap();
        answer_open(&mut el_give, vec![ChoiceOption::Card(p1_give)]);
        assert!(el_give.state_core().player(0).hand.contains(&p1_give));
        assert!(el_give.state_core().player(1).hand.contains(&eq_gain));
        assert!(el_give.state_core().player(1).equips.is_empty());
    }

    // ⑤ 存档恢复 + 分支 B（取消交牌 vec![]）：1 号令其使用 eq_gain（装备到 1 号装备栏并授予装备技）
    assert_archive_roundtrip_and_answer(&mut el, &pack_l, vec![]);
    assert_eq!(
        el.state_core().player(1).equips,
        vec![eq_gain],
        "④ 取消交牌后该装备牌被直接装备到 1 号装备区"
    );
    assert!(el.state_core().has_skill(1, sanjian_skill));

    // -----------------------------------------------------------------------
    // 6C. 良策（liangce + WudianLiangceRemain）：基本牌当五谷丰登使用，且剩余未选牌归良策拥有者
    // -----------------------------------------------------------------------
    let (mut eg, pack_g) = setup_u07(3);
    eg.debug_grant(0, "liangce");

    let b1 = find_deck_card_by_kind(&eg, CardKind::Sha, 0);
    let b2 = find_deck_card_by_kind(&eg, CardKind::Shan, 0);
    let mut sg = eg.save();
    sg.core.deck.retain(|c| *c != b1 && *c != b2);
    sg.core.player_mut(0).hand = vec![b1, b2];
    eg = Engine::restore_with_extensions(&sg, &[&pack_g]).unwrap();

    // ① 负向边界：未持有良策的角色不能将闪（b2）在出牌阶段以无目标转化为五谷丰登
    {
        let mut eg_no_lc = setup_u07(3).0;
        let shan_no = find_deck_card_by_kind(&eg_no_lc, CardKind::Shan, 0);
        let mut s_no = eg_no_lc.save();
        s_no.core.deck.retain(|c| *c != shan_no);
        s_no.core.player_mut(0).hand = vec![shan_no];
        eg_no_lc = Engine::restore_with_extensions(&s_no, &[&pack_g]).unwrap();
        assert!(matches!(
            submit(
                &mut eg_no_lc,
                0,
                GameAction::PlayCard {
                    card: shan_no,
                    targets: vec![]
                }
            ),
            CommandOutcome::Rejected(_)
        ));
    }

    // 正向：0 号持有良策，将基本牌 b2（闪）以无目标打出转化为五谷丰登
    let out_lc = submit(
        &mut eg,
        0,
        GameAction::PlayCard {
            card: b2,
            targets: vec![],
        },
    );
    assert!(matches!(
        out_lc,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    while eg
        .open_choice()
        .is_some_and(|c| c.spec.prompt == "nullify_window")
    {
        pass_nullify(&mut eg);
    }
    let cw0 = eg.open_choice().expect("wugu_pick seat 0");
    assert_eq!(cw0.spec.prompt, "wugu_pick");
    assert_eq!(
        eg.catalog().wugu_remain_beneficiary(eg.state_core()),
        Some(0),
        "④ 良策设置 wugu_remain_beneficiary = Some(0)"
    );

    // 模拟 2 号在选牌前离场，使得五谷丰登结算完毕时余下 1 张亮出牌，验证余牌进入 0 号手牌而非弃牌堆！
    eg.state_core_mut().players[2].alive = false;
    let pick0 = eg.open_choice().unwrap().spec.options[0];
    answer_open(&mut eg, vec![pick0]);
    let pick1 = eg.open_choice().unwrap().spec.options[0];
    let ChoiceOption::Card(rem_card) = eg.open_choice().unwrap().spec.options[1] else {
        panic!("expected second wugu card");
    };
    answer_open(&mut eg, vec![pick1]);
    assert!(
        eg.state_core().player(0).hand.contains(&rem_card),
        "④ 五谷丰登结算后剩余亮出牌由良策角色获得进入手牌"
    );
    assert!(
        eg.log().iter().any(|ev| matches!(
            ev,
            LogEvent::CardsGiven {
                from: 0,
                to: 0,
                cards
            } if cards == &vec![rem_card]
        )),
        "④ 记录 CardsGiven 事件"
    );

    // -----------------------------------------------------------------------
    // 6D. 坚壁（jianbi）与 屯军（diyjuntun）
    // -----------------------------------------------------------------------
    let (mut ej, pack_j) = setup_u07(3);
    // 验证西真夕红（base+6）与造之仁君（base+7）武将目录已绑定 jianbi 与 diyjuntun
    let gen_base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let xizhen = ej.catalog().general_meta(gen_base + 6).expect("西真夕红");
    assert!(xizhen.2.contains(&"jianbi"));
    let zaozhi = ej.catalog().general_meta(gen_base + 7).expect("造之仁君");
    assert!(zaozhi.2.contains(&"liangce"));
    assert!(zaozhi.2.contains(&"diyjuntun"));

    let diyjuntun_def = ej.catalog().skill_by_name("diyjuntun").unwrap();
    ej.debug_grant(1, "jianbi");
    ej.debug_grant(0, "diyjuntun");
    ej.debug_grant(0, "wd_sanjian");

    // 坚壁：0 号使用南蛮入侵（savage），1 号（有坚壁）被自动排除，仅 2 号需响应
    let savage = ej.debug_place(0, "savage");
    let sha_p2 = find_deck_card_by_kind(&ej, CardKind::Sha, 0);
    let eq_hand = find_deck_card_by_slot(&ej, EquipSlot::Armor, 0);
    let eq_worn = find_deck_card_by_name_substr(&ej, "sanjian", 0);
    let mut sj = ej.save();
    sj.core
        .deck
        .retain(|c| *c != sha_p2 && *c != eq_hand && *c != eq_worn);
    sj.core.player_mut(0).hand = vec![savage, eq_hand];
    sj.core.player_mut(0).equips = vec![eq_worn];
    sj.core.player_mut(2).hand = vec![sha_p2];
    ej = Engine::restore_with_extensions(&sj, &[&pack_j]).unwrap();

    let hp1_before = ej.debug_hp(1);
    let hp2_before = ej.debug_hp(2);
    submit(
        &mut ej,
        0,
        GameAction::PlayCard {
            card: savage,
            targets: vec![],
        },
    );
    let csav = ej.open_choice().expect("aoe_response on seat 2 only");
    assert_eq!(
        csav.observer, 2,
        "②/④ 坚壁角色（1 号）不成为南蛮入侵的目标，直接跳到 2 号响应"
    );
    answer_open(&mut ej, vec![ChoiceOption::Pass]);
    assert_eq!(ej.debug_hp(1), hp1_before, "1 号不受南蛮入侵伤害");
    assert_eq!(ej.debug_hp(2), hp2_before - 1, "2 号受 1 点南蛮入侵伤害");

    // 屯军（diyjuntun）：重铸一张装备牌并随机获得一张装备牌
    submit(
        &mut ej,
        0,
        GameAction::ActivateSkill {
            skill: diyjuntun_def,
        },
    );
    let cdj = ej.open_choice().expect("wudian_diyjuntun_recast");
    assert_eq!(cdj.spec.prompt, "wudian_diyjuntun_recast");
    assert!(cdj.spec.allow_cancel);
    assert_eq!(
        cdj.spec.options,
        vec![ChoiceOption::Card(eq_worn), ChoiceOption::Card(eq_hand)]
    );
    let _ = assert_ai_pending_choice_no_leak(&ej, &pack_j, 2);

    // ③ 取消分支
    {
        let mut ej_cancel = Engine::restore_with_extensions(&ej.save(), &[&pack_j]).unwrap();
        answer_open(&mut ej_cancel, vec![]);
        assert_eq!(ej_cancel.state_core().player(0).equips, vec![eq_worn]);
    }

    // ⑤ 存档恢复 + 重铸装备区武器 eq_worn：卸载装备技并从牌堆获得 1 张装备牌
    assert_archive_roundtrip_and_answer(&mut ej, &pack_j, vec![ChoiceOption::Card(eq_worn)]);
    assert!(ej.state_core().player(0).equips.is_empty());
    assert!(!ej.state_core().has_skill(0, sanjian_skill));
    assert!(ej.state_core().discard.contains(&eq_worn));
    assert_eq!(
        ej.state_core().player(0).hand.len(),
        2,
        "手牌由 1 张（eq_hand）增加 1 张新获得的装备牌变为 2 张"
    );
    assert!(
        ej.state_core()
            .player(0)
            .hand
            .iter()
            .all(|c| ej.catalog().card_kind(*c).is_equip()),
        "新获得的牌必须为装备牌"
    );
    // ① 负向边界：手牌与装备区均无装备牌时，发动屯军被拒
    ej.debug_clear_hand(0);
    assert!(matches!(
        submit(
            &mut ej,
            0,
            GameAction::ActivateSkill {
                skill: diyjuntun_def,
            }
        ),
        CommandOutcome::Rejected(_)
    ));
}

// ===========================================================================
// 7. 其余已迁移 U07 技能与卡牌（筑城/筑城2、决刀、禁言、望梅止渴、三尖两刃刀）挂起窗口存档恢复与 AI 补齐测试
// ===========================================================================

#[test]
fn u07_existing_skills_and_cards_six_dimension_complement() {
    let (mut e, pack) = setup_u07(3);
    e.debug_grant(0, "zhucheng");
    e.debug_grant(0, "zhucheng2");
    e.debug_grant(0, "juedao");
    e.debug_grant(0, "jinyan");
    e.debug_grant(0, "wd_sanjian");
    let juedao_def = e.catalog().skill_by_name("juedao").unwrap();

    let zc_card = find_deck_card_by_kind(&e, CardKind::Shan, 0);
    let sha_c = find_deck_card_by_kind(&e, CardKind::Sha, 0);
    let sanjian = find_deck_card_by_name_substr(&e, "sanjian", 0);
    let wangmei = find_deck_card_by_name_substr(&e, "wangmeizhike", 0);

    let mut snap = e.save();
    snap.core
        .deck
        .retain(|c| *c != zc_card && *c != sha_c && *c != sanjian && *c != wangmei);
    snap.core.player_mut(0).zhucheng = vec![zc_card];
    snap.core.player_mut(0).equips = vec![sanjian];
    snap.core.player_mut(0).hand = vec![wangmei];
    snap.core.player_mut(0).hp = 2;
    snap.core.player_mut(1).hand = vec![sha_c];
    e = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();

    // 7A. 禁言（jinyan）：体力 ≤ 2 时黑色锦囊牌被拒 CardLimit，基本牌不受限；体力 > 2 放行
    let le = e.debug_place(0, "le");
    let tao_jy = e.debug_place(0, "tao");
    assert!(matches!(
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: le,
                targets: vec![1],
            }
        ),
        CommandOutcome::Rejected(RejectReason::CardLimit)
    ));
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: tao_jy,
            targets: vec![],
        },
    );
    assert_eq!(e.debug_hp(0), 3, "基本牌桃不受禁言限制，回复至 3 点体力");

    // 7B. 望梅止渴（wangmeizhike）：0 号体力最小且已受伤（hp=3 < 4），且打出后手牌最小 -> 回复 1 点体力并摸 2 张牌
    let out_wm = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: wangmei,
            targets: vec![0],
        },
    );
    assert!(matches!(
        out_wm,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    while e
        .open_choice()
        .is_some_and(|c| c.spec.prompt == "nullify_window")
    {
        pass_nullify(&mut e);
    }
    assert_eq!(e.debug_hp(0), 4, "望梅止渴令最小体力且受伤目标回复 1 点体力");

    // 体力 > 2 后，禁言不再封锁黑色锦囊牌 le
    let out_le = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: le,
            targets: vec![1],
        },
    );
    assert!(matches!(
        out_le,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));

    // 7C. 决刀：发动决刀令 0 号横置
    let out_jd = submit(&mut e, 0, GameAction::ActivateSkill { skill: juedao_def });
    assert!(matches!(
        out_jd,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    assert!(e.state_core().player(0).linked);

    // 7D. 三尖两刃刀（wd_sanjian）：0 号杀 2 号命中后触发 wd_sanjian_discard 与 wd_sanjian_target 两段挂起窗口存档恢复 + AI 决策
    let sha_sj = e.debug_place(0, "sha");
    let pay_sj = e.debug_place(0, "tao");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_sj,
            targets: vec![2],
        },
    );
    let csj1 = e.open_choice().expect("wd_sanjian_discard");
    assert_eq!(csj1.spec.prompt, "wd_sanjian_discard");
    let _ = assert_ai_pending_choice_no_leak(&e, &pack, 2);
    assert_archive_roundtrip_and_answer(&mut e, &pack, vec![ChoiceOption::Card(pay_sj)]);
    let csj2 = e.open_choice().expect("wd_sanjian_target");
    assert_eq!(csj2.spec.prompt, "wd_sanjian_target");
    let _ = assert_ai_pending_choice_no_leak(&e, &pack, 2);
    assert_archive_roundtrip_and_answer(&mut e, &pack, vec![ChoiceOption::Player(1)]);

    // 7E. 筑城2（wudian_zhucheng_guard_ask）：1 号对有筑牌的 0 号出杀，测试挂起窗口存档恢复 + AI 决策
    let mut snap2 = e.save();
    snap2.core.turn.current = 1;
    snap2.core.turn.phase = Phase::Play;
    snap2.core.turn.sha_used_play_phase = false;
    // 先验证 0 号处于决刀横置状态时，无武器 1 号距离 0 号为 2，出杀被拒
    e = Engine::restore_with_extensions(&snap2, &[&pack]).unwrap();
    assert!(matches!(
        submit(
            &mut e,
            1,
            GameAction::PlayCard {
                card: sha_c,
                targets: vec![0],
            }
        ),
        CommandOutcome::Rejected(_)
    ));
    // 解除 0 号横置后，1 号对有筑牌的 0 号出杀触发 wudian_zhucheng_guard_ask
    snap2.core.player_mut(0).linked = false;
    e = Engine::restore_with_extensions(&snap2, &[&pack]).unwrap();
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha_c,
            targets: vec![0],
        },
    );
    let cz2 = e.open_choice().expect("wudian_zhucheng_guard_ask");
    assert_eq!(
        cz2.spec.prompt,
        engine_native::content::host::WUDIAN_ZHUCHENG_GUARD_ASK
    );
    let _ = assert_ai_pending_choice_no_leak(&e, &pack, 2);
    assert_archive_roundtrip_and_answer(&mut e, &pack, vec![ChoiceOption::Pass]);
    assert!(e.state_core().player(1).hand.contains(&sha_c));
}
