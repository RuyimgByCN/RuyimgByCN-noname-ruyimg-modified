//! U07 玩点论杀队列一规则等价补测（Item 3 六维真实对局事件路径）。
//! 禁调 `debug_execute_effect`；全部走真实 `PlayCard` / `ActivateSkill` / `NextPhase` / `Answer`。

use super::*;
use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::Suit;
use engine_native::machine::LogEvent;
use engine_native::state::GameMode;

fn u07_equiv_pack() -> ExtensionPack {
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
    pack
}

fn setup_equiv(players: u8) -> (Engine, ExtensionPack) {
    let pack = u07_equiv_pack();
    let mut e =
        Engine::new_with_extensions(7713, players as usize, &[&pack]).expect("u07 equiv engine");
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

fn find_deck_kind(e: &Engine, kind: CardKind, nth: usize) -> u32 {
    e.save()
        .core
        .deck
        .iter()
        .chain(e.save().core.discard.iter())
        .copied()
        .filter(|c| e.catalog().card_kind(*c) == kind)
        .nth(nth)
        .unwrap_or_else(|| panic!("missing deck/discard card of kind {kind:?} #{nth}"))
}

fn answer_open(e: &mut Engine, selection: Vec<ChoiceOption>) {
    let c = e.open_choice().cloned().expect("expected open choice");
    submit(
        e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection,
        },
    );
}

fn roundtrip_and_answer(e: &mut Engine, pack: &ExtensionPack, selection: Vec<ChoiceOption>) {
    let c = e.open_choice().cloned().expect("pending choice open");
    let log_before = e.log().len();
    let raw = encode_archive(&e.save());
    let decoded = decode_archive(&raw).expect("decode archive");
    let mut restored = Engine::restore_with_extensions(&decoded, &[pack]).expect("restore archive");
    assert_eq!(e.state_hash(), restored.state_hash());
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
        std::mem::discriminant(&out_rest)
    );
    assert_eq!(e.state_hash(), restored.state_hash());
    assert_eq!(&e.log()[log_before..], restored.log());
}

fn assert_ai_no_leak(e: &Engine, pack: &ExtensionPack, hidden_seat: u8) -> Vec<ChoiceOption> {
    let c = e.open_choice().expect("pending choice for AI");
    let obs = c.observer;
    let planner = BoundedPlanner::new(16);
    let GameAction::Answer {
        choice_id,
        selection: sel_a,
    } = e.plan_action(obs, &planner).action
    else {
        panic!("expected GameAction::Answer from AI");
    };
    assert_eq!(choice_id, c.id);

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
    let GameAction::Answer {
        selection: sel_b, ..
    } = e_b.plan_action(obs, &planner).action
    else {
        panic!("expected Answer");
    };
    assert_eq!(sel_a, sel_b, "AI leaked hidden info on {}", c.spec.prompt);
    sel_a
}

/// Item 1: fuchou 取消杀后原杀入弃牌堆（不回来源手）、同源去重、多目标只取消自身目标。
#[test]
fn u07_fuchou_discards_sha_deduplicates_marks_and_multi_target_only_cancels_self() {
    // (A) 两次同源杀取消：原杀入弃牌堆（非回手）、fuchou_marked 去重只留 1 个 [0]
    let (mut e, pack) = setup_equiv(3);
    e.debug_grant(1, "fuchou");
    e.debug_grant(1, "fuchou2");

    let zhuge = find_deck_kind(&e, CardKind::Zhuge, 0);
    let sha1 = find_deck_kind(&e, CardKind::Sha, 0);
    let sha2 = find_deck_kind(&e, CardKind::Sha, 1);
    let give1 = find_deck_kind(&e, CardKind::Tao, 0);
    let give2 = find_deck_kind(&e, CardKind::Shan, 0);
    let mut snap = e.save();
    snap.core
        .deck
        .retain(|c| *c != zhuge && *c != sha1 && *c != sha2 && *c != give1 && *c != give2);
    snap.core
        .discard
        .retain(|c| *c != zhuge && *c != sha1 && *c != sha2 && *c != give1 && *c != give2);
    snap.core.player_mut(0).equips = vec![zhuge];
    snap.core.player_mut(0).hand = vec![sha1, sha2];
    snap.core.player_mut(1).hand = vec![give1, give2];
    e = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();

    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
        },
    );
    assert_eq!(e.open_choice().unwrap().spec.prompt, "wudian_fuchou_cancel");
    answer_open(&mut e, vec![ChoiceOption::Card(give1)]);

    assert!(
        !e.state_core().player(0).hand.contains(&sha1),
        "fuchou 取消杀后，原【杀】不得退回来源手牌"
    );
    assert!(
        e.save().core.discard.contains(&sha1),
        "fuchou 取消杀后，原【杀】应进入弃牌堆"
    );
    assert!(e.state_core().player(0).hand.contains(&give1));
    assert_eq!(e.state_core().player(1).fuchou_marked, vec![0]);

    // 同一来源再出一张【杀】，再次发动复仇给牌：fuchou_marked 必须去重
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    );
    assert_eq!(e.open_choice().unwrap().spec.prompt, "wudian_fuchou_cancel");
    answer_open(&mut e, vec![ChoiceOption::Card(give2)]);
    assert!(e.save().core.discard.contains(&sha2));
    assert_eq!(
        e.state_core().player(1).fuchou_marked,
        vec![0],
        "同一来源重复触发复仇，fuchou2 标记集应按 .add() 去重"
    );

    // (B) 方天多目标【杀】：仅取消发动复仇的 1 号，2 号照常结算
    let (mut e2, pack2) = setup_equiv(4);
    e2.debug_grant(1, "fuchou");
    let ft = find_deck_kind(&e2, CardKind::Fangtian, 0);
    let sha_m = find_deck_kind(&e2, CardKind::Sha, 0);
    let p1_give = find_deck_kind(&e2, CardKind::Tao, 0);
    let p2_shan = find_deck_kind(&e2, CardKind::Shan, 0);
    let mut s2 = e2.save();
    s2.core
        .deck
        .retain(|c| *c != ft && *c != sha_m && *c != p1_give && *c != p2_shan);
    s2.core.player_mut(0).equips = vec![ft];
    s2.core.player_mut(0).hand = vec![sha_m];
    s2.core.player_mut(1).hand = vec![p1_give];
    s2.core.player_mut(2).hand = vec![p2_shan];
    e2 = Engine::restore_with_extensions(&s2, &[&pack2]).unwrap();

    let hp1_before = e2.debug_hp(1);
    let hp2_before = e2.debug_hp(2);
    submit(
        &mut e2,
        0,
        GameAction::PlayCard {
            card: sha_m,
            targets: vec![1, 2],
        },
    );
    assert_eq!(e2.open_choice().unwrap().spec.prompt, "wudian_fuchou_cancel");
    roundtrip_and_answer(&mut e2, &pack2, vec![ChoiceOption::Card(p1_give)]);
    // 1 号被复仇取消不受伤；2 号无复仇无闪，应继续受此【杀】1 点伤害；sha_m 最终进弃牌堆
    let c2 = e2.open_choice().expect("2 号仍应进入 respond_shan 询问");
    assert_eq!(c2.observer, 2);
    assert_eq!(c2.spec.prompt, "respond_shan");
    answer_open(&mut e2, vec![ChoiceOption::Pass]);
    assert_eq!(e2.debug_hp(1), hp1_before);
    assert_eq!(e2.debug_hp(2), hp2_before - 1);
    assert!(e2.save().core.discard.contains(&sha_m));
}

/// Item 2: fuchou2 全局 phaseAfter 按序结算（死源跳过、活源先摸 1 再判定真实可用【杀】 ChoiceKind / 取消或无杀掉 1 血、六维含恢复与 AI）。
#[test]
fn u07_fuchou2_global_phase_after_real_sha_choice_lose_hp_and_six_dimensions() {
    // 场景 A：① 负向条件——无标记或仅死源不触发，不摸牌不掉血
    let (mut e0, pack0) = setup_equiv(4);
    e0.debug_grant(1, "fuchou2");
    e0.debug_set_hp(2, 0);
    let mut s0 = e0.save();
    s0.core.player_mut(2).alive = false;
    s0.core.player_mut(1).fuchou_marked = vec![2];
    e0 = Engine::restore_with_extensions(&s0, &[&pack0]).unwrap();
    let hp1_a = e0.debug_hp(1);
    let hand1_a = e0.debug_hand(1).len();
    submit(&mut e0, 0, GameAction::NextPhase);
    submit(&mut e0, 0, GameAction::NextPhase);
    submit(&mut e0, 0, GameAction::NextPhase);
    assert!(e0.open_choice().is_none(), "仅死源时 fuchou2 不得触发");
    assert_eq!(e0.debug_hp(1), hp1_a);
    assert_eq!(e0.debug_hand(1).len(), hand1_a);

    // 场景 B：真实出杀与跳过死源（marked = [死 2, 远距活 3(无手闪), 活 0(有手闪)]）
    // 1 号起手无杀，靠第 1 次摸 1 从牌堆顶摸到真杀 sha_top 对远距 3 号（距离 2）使用真实【杀】（无视距离 canUse=false），
    // 第 2 次摸 1 摸到非杀 tao_next（手里无可用杀）→ 直接 loseHp(1)。
    let (mut e, pack) = setup_equiv(5);
    e.debug_grant(1, "fuchou2");
    let sha_top = find_deck_kind(&e, CardKind::Sha, 0);
    let tao_next = find_deck_kind(&e, CardKind::Tao, 0);
    let shan_0 = find_deck_kind(&e, CardKind::Shan, 0);
    let shan_3 = find_deck_kind(&e, CardKind::Shan, 1);
    let p3_filler = find_deck_kind(&e, CardKind::Tao, 1);

    let mut snap = e.save();
    snap.core
        .deck
        .retain(|c| *c != sha_top && *c != tao_next && *c != shan_0 && *c != shan_3 && *c != p3_filler);
    // stack 顶为 last()：先被 3 号标记摸到的是 sha_top，再被 0 号标记摸到的是 tao_next
    snap.core.deck.push(tao_next);
    snap.core.deck.push(sha_top);
    snap.core.player_mut(2).hp = 0;
    snap.core.player_mut(2).alive = false;
    snap.core.player_mut(0).hand = vec![shan_0];
    snap.core.player_mut(1).hand = vec![p3_filler]; // 1 张非杀手牌，保证有对比项
    snap.core.player_mut(1).fuchou_marked = vec![2, 3, 0];
    snap.core.player_mut(3).hand = vec![shan_3];
    e = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();

    let hp1_before = e.debug_hp(1);
    let hp3_before = e.debug_hp(3);
    // 0 号推进到回合后 TurnPhaseAfter，全局触发 1 号的 fuchou2（无需等到 1 号自己回合末！）
    submit(&mut e, 0, GameAction::NextPhase); // Discard
    submit(&mut e, 0, GameAction::NextPhase); // End
    submit(&mut e, 0, GameAction::NextPhase); // TurnEndWindow -> TurnPhaseAfter -> fuchou2

    let ch = e.open_choice().expect("fuchou2 摸到真杀后应询问使用真实【杀】");
    assert_eq!(ch.observer, 1);
    assert_ne!(ch.spec.prompt, "respond_shan", "严禁自动虚拟杀");
    assert!(ch.spec.allow_cancel, "使用【杀】选择必须可取消（allow_cancel: true）");
    assert_eq!(
        ch.spec.options,
        vec![ChoiceOption::Card(sha_top)],
        "仅摸到的真【杀】应作为合法使用候选项"
    );
    // ⑥ AI 无泄漏检查
    let _ = assert_ai_no_leak(&e, &pack, 4);

    // Cancel 分支：若取消使用真杀，1 号保留 sha_top 并直接 loseHp(1) 随后进入下一标记
    {
        let mut e_cancel = Engine::restore_with_extensions(&e.save(), &[&pack]).unwrap();
        answer_open(&mut e_cancel, vec![]);
        assert!(
            e_cancel.state_core().player(1).hand.contains(&sha_top),
            "取消出杀后真【杀】保留在手牌"
        );
        // 第 1 次取消掉 1 血；第 2 个标记（0 号）摸到 tao_next（非杀）后若手上有 sha_top 实际上此时仍可用 sha_top 问 0 号！
        let ch_next = e_cancel
            .open_choice()
            .expect("取消首杀掉 1 血后仍持 sha_top，结算下一标记 0 号时应继续询问可用【杀】");
        assert_eq!(ch_next.observer, 1);
        assert_eq!(e_cancel.debug_hp(1), hp1_before - 1);
    }

    // ⑤ 中途存档恢复后选择真杀 sha_top 对远距 3 号打出（真实消耗 sha_top，绝非虚拟杀）
    roundtrip_and_answer(&mut e, &pack, vec![ChoiceOption::Card(sha_top)]);
    let shan_ask = e.open_choice().expect("3 号应收到真实杀的 respond_shan 询问");
    assert_eq!(shan_ask.observer, 3);
    assert_eq!(shan_ask.spec.prompt, "respond_shan");
    assert!(!e.state_core().player(1).hand.contains(&sha_top));
    answer_open(&mut e, vec![ChoiceOption::Pass]);
    // 3 号掉 1 血，原杀 sha_top 进弃牌堆；随后轮到下一标记 0 号：1 号摸 tao_next 后已无杀，直接 loseHp(1)
    assert_eq!(e.debug_hp(3), hp3_before - 1);
    assert!(e.save().core.discard.contains(&sha_top));
    assert!(
        e.state_core().player(1).fuchou_marked.is_empty(),
        "全部标记应按序结算并清空"
    );
    assert_eq!(
        e.debug_hp(1),
        hp1_before - 1,
        "第二标记 0 号无可用杀，应直接 loseHp(1)"
    );
}

/// Item 3: choudu 每出牌阶段限 1 次 + cardEnabled 目标过滤 + 市场选牌/强制手牌替换/按序置顶/单次弃置日志 + 独立构造包实体调兵(红桃4/方片4)。
#[test]
fn u07_choudu_and_diaobingqianjiang_equivalence_and_physical_pack() {
    // 正式包经 attach_formal_package_upstream_rules 已携带上游红桃 4 / 方片 4 两张实体【调兵遣将】（card/list.js:25-26）
    let pack = u07_equiv_pack();
    let db_cards: Vec<_> = pack
        .cards
        .iter()
        .filter(|c| c.name.contains("diaobingqianjiang"))
        .collect();
    assert_eq!(db_cards.len(), 2, "formal wudian pack must include 2 physical diaobingqianjiang cards");
    assert_eq!((db_cards[0].suit, db_cards[0].rank), (Suit::Heart, 4));
    assert_eq!((db_cards[1].suit, db_cards[1].rank), (Suit::Diamond, 4));

    let mut e = Engine::new_with_extensions(7719, 4, &[&pack]).expect("u07 choudu engine");
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
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "choudu");
    let choudu_def = e.catalog().skill_by_name("choudu").unwrap();

    let cost_c = find_deck_kind(&e, CardKind::Shan, 0);
    let p0_swap = find_deck_kind(&e, CardKind::Tao, 0);
    let p1_swap = find_deck_kind(&e, CardKind::Sha, 0);
    let p2_h = find_deck_kind(&e, CardKind::Sha, 1);
    let m1 = find_deck_kind(&e, CardKind::Duel, 0);
    let m2 = find_deck_kind(&e, CardKind::Wuzhong, 0);
    let draw_1 = find_deck_kind(&e, CardKind::Tao, 1);
    let extra_top = find_deck_kind(&e, CardKind::Shan, 1);

    let mut snap = e.save();
    snap.core.deck.retain(|c| {
        ![cost_c, p0_swap, p1_swap, p2_h, m1, m2, draw_1, extra_top].contains(c)
    });
    // 牌堆顶（last）：1号摸 draw_1，随后亮出 ceil(4/2)=2 张市场牌 [m1, m2]（pop 出 m1 再 pop 出 m2）
    snap.core.deck.push(extra_top);
    snap.core.deck.push(m2);
    snap.core.deck.push(m1);
    snap.core.deck.push(draw_1);
    snap.core.player_mut(0).hand = vec![cost_c, p0_swap];
    snap.core.player_mut(1).hand = vec![p1_swap];
    snap.core.player_mut(2).hand = vec![p2_h];
    // 3 号设为无手牌且被 blocks_all_cards（cardEnabled 拒绝作为 choudu 目标）
    snap.core.player_mut(3).hand.clear();
    snap.core
        .vm_aux
        .flags
        .insert((3, "diaohulishan_active".to_string()), 1);
    e = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();

    // ① 取消 choudu 不消耗每出牌阶段 1 次额度
    submit(&mut e, 0, GameAction::ActivateSkill { skill: choudu_def });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "wudian_choudu_card");
    answer_open(&mut e, vec![]);

    // 正式发动 choudu
    submit(&mut e, 0, GameAction::ActivateSkill { skill: choudu_def });
    answer_open(&mut e, vec![ChoiceOption::Card(cost_c)]);

    let c_tgt = e.open_choice().expect("wudian_choudu_target");
    assert_eq!(c_tgt.spec.prompt, "wudian_choudu_target");
    assert_eq!(
        c_tgt.spec.options,
        vec![
            ChoiceOption::Player(0),
            ChoiceOption::Player(1),
            ChoiceOption::Player(2)
        ],
        "cardEnabled({{diaobingqianjiang}}) 应排除无法用牌的 3 号"
    );
    let log_before = e.log().len();
    roundtrip_and_answer(&mut e, &pack, vec![ChoiceOption::Player(1)]);
    let discard_logs = e.log()[log_before..]
        .iter()
        .filter(|ev| matches!(ev, LogEvent::CardDiscarded { player: 0, cards } if cards == &vec![cost_c]))
        .count();
    assert_eq!(discard_logs, 1, "choudu 弃置 1 张牌不得重复记 CardDiscarded 日志");

    // 1 号先摸 1 张 draw_1，亮出 [m1, m2]；队列按座次从 1 号起：[1, 2, 0]（3 号无手牌跳过）
    assert!(e.state_core().player(1).hand.contains(&draw_1));

    // 目标 1：选市场牌（m1, m2 可选且可取消）— 故意选第 2 张 m2！
    let c_pick1 = e.open_choice().expect("target 1 market pick");
    assert_eq!(c_pick1.observer, 1);
    assert!(c_pick1.spec.allow_cancel);
    assert!(c_pick1.spec.options.contains(&ChoiceOption::Card(m1)));
    assert!(c_pick1.spec.options.contains(&ChoiceOption::Card(m2)));
    let _ = assert_ai_no_leak(&e, &pack, 2);
    roundtrip_and_answer(&mut e, &pack, vec![ChoiceOption::Card(m2)]);

    // 目标 1：强制选 1 张手牌替换 m2
    let c_swap1 = e.open_choice().expect("target 1 hand swap");
    assert_eq!(c_swap1.observer, 1);
    assert_eq!(c_swap1.spec.prompt, "wudian_diaobing_swap");
    assert!(!c_swap1.spec.allow_cancel, "选定市场牌后替换手牌不可取消");
    let _ = assert_ai_no_leak(&e, &pack, 2);
    roundtrip_and_answer(&mut e, &pack, vec![ChoiceOption::Card(p1_swap)]);
    assert!(e.state_core().player(1).hand.contains(&m2));
    assert!(!e.state_core().player(1).hand.contains(&p1_swap));

    // 目标 2：跳过不换
    let c_pick2 = e.open_choice().expect("target 2 market pick");
    assert_eq!(c_pick2.observer, 2);
    answer_open(&mut e, vec![]);

    // 目标 0：选市场里刚被 1 号换入的 p1_swap，用 p0_swap 替换
    let c_pick0 = e.open_choice().expect("target 0 market pick");
    assert_eq!(c_pick0.observer, 0);
    assert!(
        c_pick0.spec.options.contains(&ChoiceOption::Card(p1_swap)),
        "1 号换入市场的 p1_swap 应可被后续目标 0 号选中"
    );
    answer_open(&mut e, vec![ChoiceOption::Card(p1_swap)]);
    let c_swap0 = e.open_choice().expect("target 0 hand swap");
    assert_eq!(c_swap0.observer, 0);
    assert!(!c_swap0.spec.allow_cancel);
    answer_open(&mut e, vec![ChoiceOption::Card(p0_swap)]);
    assert!(e.state_core().player(0).hand.contains(&p1_swap));

    // 末尾使用者 1 号：contentAfter 选择任意张剩余市场牌 [p0_swap] 置顶，其余 m1 弃置
    let c_top = e.open_choice().expect("user contentAfter top-deck choice");
    assert_eq!(c_top.observer, 1);
    assert!(c_top.spec.allow_cancel);
    assert_eq!(c_top.spec.max, 2);
    assert!(c_top.spec.options.contains(&ChoiceOption::Card(m1)));
    assert!(c_top.spec.options.contains(&ChoiceOption::Card(p0_swap)));
    let _ = assert_ai_no_leak(&e, &pack, 2);
    roundtrip_and_answer(&mut e, &pack, vec![ChoiceOption::Card(p0_swap)]);

    assert!(e.open_choice().is_none());
    assert_eq!(
        e.save().core.deck.last(),
        Some(&p0_swap),
        "所选置顶牌必须放在牌堆顶（deck.last()）"
    );
    assert!(
        e.save().core.discard.contains(&m1),
        "未选的剩余市场牌必须进弃牌堆"
    );

    // 同出牌阶段再发动 choudu 应拒绝，但同回合额外 Play 阶段可再次发动！
    assert!(matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: choudu_def }),
        CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)
    ));
    let mut s_extra = e.save();
    s_extra.core.turn.extra_phases.push(2); // 同回合追加一个 Phase::Play
    s_extra.core.vm_aux.flags.remove(&(3, "diaohulishan_active".to_string()));
    // 给 0 号补物理 wd_diaobingqianjiang_1 与手牌再测实体出牌 + 额外阶段筹度
    let db_phys = s_extra
        .core
        .deck
        .iter()
        .copied()
        .find(|&c| e.catalog().cards[c as usize].name.contains("wd_diaobingqianjiang"))
        .expect("physical wd_diaobingqianjiang present");
    s_extra.core.deck.retain(|&c| c != db_phys);
    s_extra.core.player_mut(0).hand.push(db_phys);
    e = Engine::restore_with_extensions(&s_extra, &[&pack]).unwrap();
    // 推进至同回合额外 Play 阶段（phase.rs 在阶段结束弹出 extra_phases，单次 NextPhase 即入额外 Play）
    submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(e.turn().current, 0);
    assert_eq!(e.turn().phase, Phase::Play);

    // 额外出牌阶段再次发动 choudu（本次先取消一次验证可用，再用 db_phys 测实体调兵遣将）
    submit(&mut e, 0, GameAction::ActivateSkill { skill: choudu_def });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "wudian_choudu_card");
    answer_open(&mut e, vec![]);

    // 实体 wd_diaobingqianjiang_1（红桃 4）出牌：targets=[] 自动全选合法目标（座次 0,1,2）并支持多张按序置顶！
    let mut s_phys = e.save();
    let phys_top1 = find_deck_kind(&e, CardKind::Sha, 2);
    let phys_top2 = find_deck_kind(&e, CardKind::Tao, 2);
    let phys_draw = find_deck_kind(&e, CardKind::Duel, 1);
    s_phys
        .core
        .deck
        .retain(|&c| c != phys_top1 && c != phys_top2 && c != phys_draw);
    s_phys.core.deck.push(phys_top2);
    s_phys.core.deck.push(phys_top1);
    s_phys.core.deck.push(phys_draw);
    s_phys.core.player_mut(3).hand.clear();
    e = Engine::restore_with_extensions(&s_phys, &[&pack]).unwrap();

    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: db_phys,
            targets: vec![],
        },
    );
    assert!(e.state_core().player(0).hand.contains(&phys_draw));
    // 所有目标（0、1、2）均取消不换
    for seat in [0u8, 1, 2] {
        let ch = e.open_choice().expect("target pick on physical diaobingqianjiang");
        assert_eq!(ch.observer, seat);
        answer_open(&mut e, vec![]);
    }
    // 末尾使用者 0 号按顺序 [phys_top2, phys_top1] 置顶（先选的 phys_top2 在最顶！）
    let ch_top = e.open_choice().expect("physical contentAfter top");
    assert_eq!(ch_top.observer, 0);
    roundtrip_and_answer(
        &mut e,
        &pack,
        vec![ChoiceOption::Card(phys_top2), ChoiceOption::Card(phys_top1)],
    );
    assert!(e.open_choice().is_none());
    let d = e.save().core.deck;
    assert_eq!(d[d.len() - 1], phys_top2, "先选的应在牌堆最顶");
    assert_eq!(d[d.len() - 2], phys_top1, "后选的紧随其下");
    assert!(
        e.save().core.discard.contains(&db_phys),
        "实体调兵遣将结算完毕后必须离桌入弃牌堆"
    );
}

/// Item 4: youzhan 逐目标 shaBefore 距离<=1（仅取消对应目标）、原杀真正取消不造成额外伤害、
/// 虚拟与实体诱敌深入均改用循环 ChoiceKind 直到某次【杀】造成 shaDamage 方结束（被闪且停止则受1伤）、去重日志。
#[test]
fn u07_youzhan_and_youdishenru_loop_multi_target_and_no_extra_damage() {
    use engine_native::catalog::EquipSlot;

    // -----------------------------------------------------------------------
    // 场景 A：youzhan 原杀真正取消（目标无闪时也不受伤！）+ 循环反击杀（被闪后继续问，中则停，停则受1伤）+ 日志去重
    // -----------------------------------------------------------------------
    let (mut e, pack) = setup_equiv(4);
    e.debug_grant(0, "youzhan");
    let eq_c = e
        .save()
        .core
        .deck
        .iter()
        .copied()
        .find(|&c| e.catalog().card_slot_of(&e.save().core, c) == Some(EquipSlot::Armor))
        .expect("armor");
    let sha1 = find_deck_kind(&e, CardKind::Sha, 0);
    let sha2 = find_deck_kind(&e, CardKind::Sha, 1);
    let sha3 = find_deck_kind(&e, CardKind::Sha, 2);
    let shan0 = find_deck_kind(&e, CardKind::Shan, 0);

    let mut snap = e.save();
    snap.core.turn.current = 1;
    snap.core.turn.phase = Phase::Play;
    snap.core
        .deck
        .retain(|c| ![eq_c, sha1, sha2, sha3, shan0].contains(c));
    snap.core.player_mut(0).hand = vec![eq_c, shan0];
    snap.core.player_mut(1).hand = vec![sha1, sha2, sha3];
    e = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();

    let hp0_start = e.debug_hp(0);
    let hp1_start = e.debug_hp(1);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![0],
        },
    );
    assert_eq!(e.open_choice().unwrap().spec.prompt, "wudian_youzhan_discard");
    let log_disc_start = e.log().len();
    answer_open(&mut e, vec![ChoiceOption::Card(eq_c)]);
    let disc_cnt = e.log()[log_disc_start..]
        .iter()
        .filter(|ev| matches!(ev, LogEvent::CardDiscarded { player: 0, cards } if cards == &vec![eq_c]))
        .count();
    assert_eq!(disc_cnt, 1, "youzhan 弃置装备不得重复记录 CardDiscarded");
    assert!(e.save().core.discard.contains(&sha1));

    // 反击杀询问 #1：1 号打出 sha2（检查单次 CardPlayed 日志）
    let ch_loop1 = e.open_choice().expect("counter sha #1");
    assert_eq!(ch_loop1.spec.prompt, "wudian_youdishenru_sha");
    assert_eq!(ch_loop1.observer, 1);
    let log_play_start = e.log().len();
    roundtrip_and_answer(&mut e, &pack, vec![ChoiceOption::Card(sha2)]);
    let play_cnt = e.log()[log_play_start..]
        .iter()
        .filter(|ev| matches!(ev, LogEvent::CardPlayed { player: 1, card, .. } if *card == sha2))
        .count();
    assert_eq!(play_cnt, 1, "反击杀不得重复记录 CardPlayed");

    // 0 号出闪抵消 sha2（sha2 未造成 shaDamage）
    let ch_shan0 = e.open_choice().expect("0 号响应闪抵消 sha2");
    assert_eq!(ch_shan0.observer, 0);
    assert_eq!(ch_shan0.spec.prompt, "respond_shan");
    answer_open(&mut e, vec![ChoiceOption::Card(shan0)]);

    // 关键等价点：sha2 被闪后，循环不得结束！必须继续询问 1 号下一张【杀】 sha3！
    let ch_loop2 = e
        .open_choice()
        .expect("反击杀被闪未造成 shaDamage，必须循环继续询问来源出下一张杀");
    assert_eq!(ch_loop2.spec.prompt, "wudian_youdishenru_sha");
    assert_eq!(ch_loop2.observer, 1);
    let _ = assert_ai_no_leak(&e, &pack, 2);

    // 分支 A：被闪后停止（取消 vec![]）-> 0 号对 1 号造成 1 点伤害，0 号无伤（原 sha1 也真正取消）
    {
        let mut e_stop = Engine::restore_with_extensions(&e.save(), &[&pack]).unwrap();
        answer_open(&mut e_stop, vec![]);
        assert!(e_stop.open_choice().is_none());
        assert_eq!(e_stop.debug_hp(0), hp0_start, "原杀 sha1 被诱战取消不得对 0 号造成额外伤害");
        assert_eq!(e_stop.debug_hp(1), hp1_start - 1, "反击杀停止后来源受 1 点反噬伤害");
    }

    // 分支 B：存档恢复后继续出 sha3 并命中（0 号无闪自动扣 1 血）-> shaDamage 触发，循环立即结束！
    roundtrip_and_answer(&mut e, &pack, vec![ChoiceOption::Card(sha3)]);
    assert!(e.open_choice().is_none());
    assert_eq!(e.debug_hp(0), hp0_start - 1, "sha3 造成 1 点 shaDamage（原 sha1 无额外伤害）");
    assert_eq!(e.debug_hp(1), hp1_start, "shaDamage 结束诱敌循环，来源不受反噬伤害");

    // -----------------------------------------------------------------------
    // 场景 B：5 人局方天多目标 shaBefore——目标[2, 0]，诱战者 4 号距 2 为 2(>1)、距 0 为 1(<=1)，
    // 4 号仍须对 0 号触发诱战，且只取消 0 号目标，2 号照常结算原杀！
    // -----------------------------------------------------------------------
    let (mut e2, pack2) = setup_equiv(5);
    e2.debug_grant(4, "youzhan");
    let ft = find_deck_kind(&e2, CardKind::Fangtian, 0);
    let sha_m = find_deck_kind(&e2, CardKind::Sha, 0);
    let eq4 = find_deck_kind(&e2, CardKind::Shan, 0); // 找装备换用 find_deck_kind_or_slot:
    let _ = eq4;
    let eq4 = e2
        .save()
        .core
        .deck
        .iter()
        .copied()
        .find(|&c| e2.catalog().card_slot_of(&e2.save().core, c) == Some(EquipSlot::Armor))
        .expect("armor for seat 4");
    let shan2 = find_deck_kind(&e2, CardKind::Shan, 1);

    let mut s2 = e2.save();
    s2.core.turn.current = 1;
    s2.core.turn.phase = Phase::Play;
    s2.core
        .deck
        .retain(|c| ![ft, sha_m, eq4, shan2].contains(c));
    s2.core.player_mut(1).equips = vec![ft];
    s2.core.player_mut(1).hand = vec![sha_m];
    s2.core.player_mut(4).hand = vec![eq4];
    s2.core.player_mut(2).hand = vec![shan2];
    s2.core.player_mut(0).hand.clear();
    e2 = Engine::restore_with_extensions(&s2, &[&pack2]).unwrap();

    let hp0_b = e2.debug_hp(0);
    let hp1_b = e2.debug_hp(1);
    let hp2_b = e2.debug_hp(2);
    submit(
        &mut e2,
        1,
        GameAction::PlayCard {
            card: sha_m,
            targets: vec![2, 0],
        },
    );
    let ch_yz = e2
        .open_choice()
        .expect("u.targets[0]=2 超距但 u.targets[1]=0 在距离1内，4号诱战必须触发");
    assert_eq!(ch_yz.spec.prompt, "wudian_youzhan_discard");
    assert_eq!(ch_yz.observer, 4);
    answer_open(&mut e2, vec![ChoiceOption::Card(eq4)]);
    // 1 号无多余杀 -> 直接受 0 号 1 点反噬；0 号被取消不受伤；随后 2 号必须照常进入原杀 sha_m 的 respond_shan！
    let ch_t2 = e2
        .open_choice()
        .expect("诱战仅取消 0 号，2 号必须照常结算原杀 respond_shan");
    assert_eq!(ch_t2.observer, 2);
    assert_eq!(ch_t2.spec.prompt, "respond_shan");
    answer_open(&mut e2, vec![ChoiceOption::Pass]);
    assert_eq!(e2.debug_hp(0), hp0_b, "0 号被诱战取消不受原杀伤害");
    assert_eq!(e2.debug_hp(1), hp1_b - 1, "1 号受诱敌反噬 1 伤");
    assert_eq!(e2.debug_hp(2), hp2_b - 1, "2 号未取消，照常受原杀 1 伤");
    assert!(e2.save().core.discard.contains(&sha_m));

    // -----------------------------------------------------------------------
    // 场景 C：实体诱敌深入（Youdishenru / g_youdishenru）循环反击直到 shaDamage + 多目标仅取消自身
    // -----------------------------------------------------------------------
    let (mut e3, pack3) = setup_equiv(4);
    let p_sha1 = find_deck_kind(&e3, CardKind::Sha, 0);
    let p_sha2 = find_deck_kind(&e3, CardKind::Sha, 1);
    let p_sha3 = find_deck_kind(&e3, CardKind::Sha, 2);
    let p_yd = find_deck_kind(&e3, CardKind::Youdishenru, 0);
    let p_shan1 = find_deck_kind(&e3, CardKind::Shan, 0);

    let mut s3 = e3.save();
    s3.core
        .deck
        .retain(|c| ![p_sha1, p_sha2, p_sha3, p_yd, p_shan1].contains(c));
    s3.core.player_mut(0).hand = vec![p_sha1, p_sha2, p_sha3];
    s3.core.player_mut(1).hand = vec![p_yd, p_shan1];
    e3 = Engine::restore_with_extensions(&s3, &[&pack3]).unwrap();

    let hp0_c = e3.debug_hp(0);
    let hp1_c = e3.debug_hp(1);
    submit(
        &mut e3,
        0,
        GameAction::PlayCard {
            card: p_sha1,
            targets: vec![1],
        },
    );
    let c_yd0 = e3.open_choice().expect("shaBefore wudian_youdishenru_use");
    assert_eq!(c_yd0.spec.prompt, "wudian_youdishenru_use");
    answer_open(&mut e3, vec![ChoiceOption::Card(p_yd)]);
    let c_loop1 = e3.open_choice().expect("physical yd counter sha #1");
    assert_eq!(c_loop1.spec.prompt, "wudian_youdishenru_sha");
    answer_open(&mut e3, vec![ChoiceOption::Card(p_sha2)]);
    let c_shan1 = e3.open_choice().expect("1 号闪抵消 p_sha2");
    assert_eq!(c_shan1.observer, 1);
    answer_open(&mut e3, vec![ChoiceOption::Card(p_shan1)]);

    let c_loop2 = e3
        .open_choice()
        .expect("实体诱敌深入：p_sha2 被闪后必须继续循环询问 p_sha3");
    assert_eq!(c_loop2.spec.prompt, "wudian_youdishenru_sha");
    assert_eq!(c_loop2.observer, 0);

    // 取消停止分支
    {
        let mut e3_stop = Engine::restore_with_extensions(&e3.save(), &[&pack3]).unwrap();
        answer_open(&mut e3_stop, vec![]);
        assert!(e3_stop.open_choice().is_none());
        assert_eq!(e3_stop.debug_hp(0), hp0_c - 1);
        assert_eq!(e3_stop.debug_hp(1), hp1_c);
    }

    // 命中结束分支（存档恢复）
    roundtrip_and_answer(&mut e3, &pack3, vec![ChoiceOption::Card(p_sha3)]);
    assert!(e3.open_choice().is_none());
    assert_eq!(e3.debug_hp(0), hp0_c);
    assert_eq!(e3.debug_hp(1), hp1_c - 1);
}

/// Item 5A: zhucheng2 任意 num 张 he 牌自选弃置（含装备区、无随机补弃）、完整取消（杀进弃牌堆不回手）、
/// <num 直接取消、多目标仅取消自身、支持火/雷杀、恢复与 AI。
#[test]
fn u07_zhucheng2_he_arbitrary_multi_select_cancel_no_return_and_multi_target() {
    use engine_native::catalog::EquipSlot;

    let (mut e, pack) = setup_equiv(4);
    e.debug_grant(0, "zhucheng2");
    e.debug_grant(0, "wd_sanjian");

    let zc1 = find_deck_kind(&e, CardKind::Shan, 0);
    let zc2 = find_deck_kind(&e, CardKind::Shan, 1);
    let sha1 = find_deck_kind(&e, CardKind::FireSha, 0);
    let sha_m = find_deck_kind(&e, CardKind::Sha, 0);
    let hand_a = find_deck_kind(&e, CardKind::Tao, 0);
    let hand_b = find_deck_kind(&e, CardKind::Duel, 0);
    let eq_1 = e
        .save()
        .core
        .deck
        .iter()
        .copied()
        .find(|&c| e.catalog().card_slot_of(&e.save().core, c) == Some(EquipSlot::Armor))
        .expect("armor for attacker 1");
    let ft = find_deck_kind(&e, CardKind::Fangtian, 0);
    let shan2 = find_deck_kind(&e, CardKind::Shan, 2);

    // 场景 A：1 号 he 牌总数 < zhucheng.len()=2（仅 1 张 hand_a）-> 不询问，直接取消杀且 sha1 进弃牌堆不回手
    let snap_base = e.save();
    let mut snap_a = snap_base.clone();
    snap_a.core.turn.current = 1;
    snap_a.core.turn.phase = Phase::Play;
    snap_a
        .core
        .deck
        .retain(|c| ![zc1, zc2, sha1, hand_a].contains(c));
    snap_a.core.player_mut(0).zhucheng = vec![zc1, zc2];
    snap_a.core.player_mut(1).hand = vec![sha1, hand_a];
    let mut e_less = Engine::restore_with_extensions(&snap_a, &[&pack]).unwrap();
    let hp0_a = e_less.debug_hp(0);
    submit(
        &mut e_less,
        1,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![0],
        },
    );
    let confirm_a = e_less.open_choice().expect("zhucheng2 trigger_confirm");
    assert_eq!(confirm_a.spec.prompt, "trigger_confirm");
    assert_eq!(confirm_a.observer, 0);
    answer_open(&mut e_less, vec![ChoiceOption::Confirm]);
    assert!(e_less.open_choice().is_none(), "确认发动后 he 不足 num 时直接取消，不得再问攻击者");
    assert_eq!(e_less.debug_hp(0), hp0_a);
    assert!(!e_less.state_core().player(1).hand.contains(&sha1));
    assert!(e_less.save().core.discard.contains(&sha1));

    // 场景 B：1 号有 [hand_a, hand_b, eq_1] 共 3 张 he 牌（>=2），使用火杀 sha1 指向 0 号：
    // 必须允许任意自选 2 张（含装备 eq_1）或取消（allow_cancel: true，杀入弃牌堆不回手）
    let mut snap_b = snap_base.clone();
    snap_b.core.turn.current = 1;
    snap_b.core.turn.phase = Phase::Play;
    snap_b
        .core
        .deck
        .retain(|c| ![zc1, zc2, sha1, hand_a, hand_b, eq_1].contains(c));
    snap_b.core.player_mut(0).zhucheng = vec![zc1, zc2];
    snap_b.core.player_mut(1).hand = vec![sha1, hand_a, hand_b];
    snap_b.core.player_mut(1).equips = vec![eq_1];
    let mut e_b = Engine::restore_with_extensions(&snap_b, &[&pack]).unwrap();
    submit(
        &mut e_b,
        1,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![0],
        },
    );
    let confirm_b = e_b.open_choice().expect("zhucheng2 trigger_confirm");
    assert_eq!(confirm_b.spec.prompt, "trigger_confirm");
    assert_eq!(confirm_b.observer, 0);
    answer_open(&mut e_b, vec![ChoiceOption::Confirm]);
    let ch = e_b.open_choice().expect("火杀确认发动筑城后触发 wudian_zhucheng_guard");
    assert_eq!(ch.observer, 1);
    assert_eq!(ch.spec.min, 2);
    assert_eq!(ch.spec.max, 2);
    assert!(ch.spec.allow_cancel);
    assert_eq!(
        ch.spec.options,
        vec![
            ChoiceOption::Card(hand_a),
            ChoiceOption::Card(hand_b),
            ChoiceOption::Card(eq_1),
        ],
        "zhucheng2 必须可选手牌与装备（he）任意 2 张"
    );
    let _ = assert_ai_no_leak(&e_b, &pack, 2);

    // 取消分支：1 号不退 sha1 回手，sha1 入弃牌堆，手牌/装备保持不变
    {
        let mut e_cancel = Engine::restore_with_extensions(&e_b.save(), &[&pack]).unwrap();
        answer_open(&mut e_cancel, vec![]);
        assert!(e_cancel.open_choice().is_none());
        assert_eq!(e_cancel.debug_hp(0), hp0_a);
        assert!(!e_cancel.state_core().player(1).hand.contains(&sha1));
        assert!(e_cancel.save().core.discard.contains(&sha1));
        assert_eq!(e_cancel.state_core().player(1).hand, vec![hand_a, hand_b]);
        assert_eq!(e_cancel.state_core().player(1).equips, vec![eq_1]);
    }

    // 存档恢复 + 自选第 2 张手牌与装备 [hand_b, eq_1]（严禁随机补弃 hand_a!）
    let log_before = e_b.log().len();
    roundtrip_and_answer(
        &mut e_b,
        &pack,
        vec![ChoiceOption::Card(hand_b), ChoiceOption::Card(eq_1)],
    );
    assert!(e_b.open_choice().is_none());
    assert_eq!(
        e_b.state_core().player(1).hand,
        vec![hand_a],
        "自选弃置 [hand_b, eq_1] 后 hand_a 必须保留"
    );
    assert!(e_b.state_core().player(1).equips.is_empty());
    assert_eq!(e_b.debug_hp(0), hp0_a - 1, "放行火杀结算 1 点伤害");
    let disc_b = e_b.log()[log_before..]
        .iter()
        .filter(|ev| matches!(ev, LogEvent::CardDiscarded { player: 1, cards } if cards == &vec![hand_b]))
        .count();
    assert_eq!(disc_b, 1, "不得重复记录 CardDiscarded");

    // 场景 C：方天多目标 [2, 0]（0 号不在首目标）-> 仅对 0 号触发 zhucheng2；取消后仅 0 号取消，2 号仍结算原杀！
    let mut snap_c = snap_base;
    snap_c.core.turn.current = 1;
    snap_c.core.turn.phase = Phase::Play;
    snap_c
        .core
        .deck
        .retain(|c| ![zc1, zc2, sha_m, ft, shan2].contains(c));
    snap_c.core.player_mut(0).zhucheng = vec![zc1, zc2];
    snap_c.core.player_mut(1).equips = vec![ft];
    snap_c.core.player_mut(1).hand = vec![sha_m]; // 出杀后 1 号仅余装备 ft (1 张 < zc=2)
    snap_c.core.player_mut(2).hand = vec![shan2];
    let mut e_c = Engine::restore_with_extensions(&snap_c, &[&pack]).unwrap();
    submit(
        &mut e_c,
        1,
        GameAction::PlayCard {
            card: sha_m,
            targets: vec![2, 0],
        },
    );
    // 先问 0 号 trigger_confirm -> 确认发动，1 号 he 只有 ft (1 < 2)，0 号取消；但 2 号未被取消，必须进入 respond_shan！
    let confirm_c = e_c.open_choice().expect("zhucheng2 trigger_confirm on seat 0");
    assert_eq!(confirm_c.spec.prompt, "trigger_confirm");
    assert_eq!(confirm_c.observer, 0);
    answer_open(&mut e_c, vec![ChoiceOption::Confirm]);
    let c2 = e_c
        .open_choice()
        .expect("多目标 [2,0] 中 0 号筑城取消，2 号仍须结算 respond_shan");
    assert_eq!(c2.observer, 2);
    assert_eq!(c2.spec.prompt, "respond_shan");
    answer_open(&mut e_c, vec![ChoiceOption::Pass]);
    assert_eq!(e_c.debug_hp(0), hp0_a);
    assert_eq!(e_c.debug_hp(2), hp0_a - 1);
    assert!(e_c.save().core.discard.contains(&sha_m));
}

/// Item 5B: jinyan（hp<=2）Play/Respond/DyingSave 黑锦囊禁令 + Play/Respond/fuchou2 转杀分支 + hp>2 恢复。
#[test]
fn u07_jinyan_respond_dying_black_trick_ban_and_sha_conversion_six_dimensions() {
    let (mut e, pack) = setup_equiv(4);
    e.debug_grant(0, "jinyan");
    e.debug_grant(0, "fuchou2");

    // 找到各种黑锦囊（黑乐不思蜀/黑无中或过拆/黑无懈/黑诱敌深入）及红锦囊
    let snap0 = e.save();
    let black_le = snap0
        .core
        .deck
        .iter()
        .copied()
        .find(|&c| {
            e.catalog().card_kind(c) == CardKind::Le
                && matches!(e.catalog().cards[c as usize].suit, Suit::Spade | Suit::Club)
        })
        .expect("black le");
    let black_guohe = snap0
        .core
        .deck
        .iter()
        .copied()
        .find(|&c| {
            e.catalog().card_kind(c) == CardKind::Dismantle
                && matches!(e.catalog().cards[c as usize].suit, Suit::Spade | Suit::Club)
        })
        .expect("black dismantle");
    let black_nullify = snap0
        .core
        .deck
        .iter()
        .copied()
        .find(|&c| {
            e.catalog().card_kind(c) == CardKind::Nullify
                && matches!(e.catalog().cards[c as usize].suit, Suit::Spade | Suit::Club)
        })
        .expect("black nullify");
    let black_yd = find_deck_kind(&e, CardKind::Youdishenru, 0); // assets/cards.json 全为黑花色
    let black_trick_notarget = snap0
        .core
        .deck
        .iter()
        .copied()
        .find(|&c| {
            let k = e.catalog().card_kind(c);
            matches!(k, CardKind::Taoyuan | CardKind::SavageAttack | CardKind::Wugu | CardKind::Wuzhong)
                && matches!(e.catalog().cards[c as usize].suit, Suit::Spade | Suit::Club)
        })
        .expect("black ao/group trick");
    let red_wuzhong = snap0
        .core
        .deck
        .iter()
        .copied()
        .find(|&c| {
            e.catalog().card_kind(c) == CardKind::Wuzhong
                && !e.catalog().cards[c as usize].name.contains("diaobingqianjiang")
                && matches!(e.catalog().cards[c as usize].suit, Suit::Heart | Suit::Diamond)
        })
        .expect("red wuzhong");
    let nanman_1 = snap0
        .core
        .deck
        .iter()
        .copied()
        .find(|&c| e.catalog().card_kind(c) == CardKind::SavageAttack && c != black_trick_notarget)
        .expect("second nanman");
    let duel_1 = find_deck_kind(&e, CardKind::Duel, 0);
    let sha_1 = find_deck_kind(&e, CardKind::Sha, 0);
    let _tao_0 = find_deck_kind(&e, CardKind::Tao, 0);

    let mut snap = snap0.clone();
    snap.core.deck.retain(|c| {
        ![black_trick_notarget, black_le, black_guohe, red_wuzhong, sha_1].contains(c)
    });
    snap.core.player_mut(0).hp = 2;
    snap.core.player_mut(0).hand = vec![black_trick_notarget, black_le, black_guohe, red_wuzhong];
    snap.core.player_mut(1).hand = vec![sha_1];
    e = Engine::restore_with_extensions(&snap, &[&pack]).unwrap();

    // ① Play 面禁令：无目标黑锦囊（不可作杀）直接 CardLimit 拒绝；红锦囊 red_wuzhong 正常可用！
    assert!(matches!(
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: black_trick_notarget,
                targets: vec![],
            }
        ),
        CommandOutcome::Rejected(RejectReason::CardLimit)
    ));
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: red_wuzhong,
            targets: vec![],
        },
    );
    assert!(e.open_choice().is_none());

    // ① Play 面转杀：hp<=2 时，持禁言者可将黑色锦囊（black_guohe / black_le）当作【杀】使用！
    let hp1_start = e.debug_hp(1);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: black_guohe,
            targets: vec![1],
        },
    );
    assert_eq!(e.debug_hp(1), hp1_start - 1, "black_guohe 转化杀对 1 号造成 1 点伤害");
    assert!(e.state_core().player(1).hand.len() == 1, "未执行拆牌效果");
    // 出过杀后，再想用 black_le（作杀受限 ShaLimit、作乐受禁言禁用）必须拒绝！
    assert!(matches!(
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: black_le,
                targets: vec![1],
            }
        ),
        CommandOutcome::Rejected(RejectReason::ShaLimit | RejectReason::CardLimit)
    ));

    // ② 响应面（无懈可击禁止 + 诱敌深入禁止 + 南蛮/决斗黑锦囊转杀）
    let mut s2 = snap0;
    s2.core.turn.current = 1;
    s2.core.turn.phase = Phase::Play;
    s2.core.turn.sha_used_play_phase = false;
    s2.core.deck.retain(|c| {
        ![
            black_nullify,
            black_yd,
            black_le,
            red_wuzhong,
            nanman_1,
            duel_1,
            sha_1,
        ]
        .contains(c)
    });
    s2.core.player_mut(0).hp = 2;
    s2.core.player_mut(0).hand = vec![black_nullify, black_yd, black_le];
    s2.core.player_mut(1).hand = vec![red_wuzhong, nanman_1, duel_1, sha_1];
    let mut e2 = Engine::restore_with_extensions(&s2, &[&pack]).unwrap();

    // 1 号用 red_wuzhong：0 号只有黑无懈 black_nullify（hp<=2），cardRespondable 禁用，不得开 nullify_window！
    submit(
        &mut e2,
        1,
        GameAction::PlayCard {
            card: red_wuzhong,
            targets: vec![],
        },
    );
    assert!(
        e2.open_choice().is_none(),
        "hp<=2 持禁言时黑无懈不可响应，不得弹出 nullify_window"
    );

    // 1 号对 0 号用 sha_1：0 号只有黑诱敌深入 black_yd，respond_shan 中不得出现 black_yd！
    submit(
        &mut e2,
        1,
        GameAction::PlayCard {
            card: sha_1,
            targets: vec![0],
        },
    );
    assert!(e2.open_choice().is_none(), "hp<=2 时黑诱敌深入被禁用，无闪直接命中");
    assert_eq!(e2.debug_hp(0), 1);

    // 1 号用南蛮入侵 nanman_1：0 号有黑锦囊 black_le / black_nullify / black_yd -> aoe_response 必须可选当作【杀】响应并档伤！
    submit(
        &mut e2,
        1,
        GameAction::PlayCard {
            card: nanman_1,
            targets: vec![],
        },
    );
    let ch_aoe = e2.open_choice().expect("aoe_response with jinyan black tricks");
    assert_eq!(ch_aoe.observer, 0);
    assert_eq!(ch_aoe.spec.prompt, "aoe_response");
    assert!(ch_aoe.spec.options.contains(&ChoiceOption::Card(black_le)));
    assert!(ch_aoe.spec.options.contains(&ChoiceOption::Card(black_nullify)));
    roundtrip_and_answer(&mut e2, &pack, vec![ChoiceOption::Card(black_le)]);
    assert_eq!(e2.debug_hp(0), 1, "以黑锦囊当杀响应南蛮不受伤");

    // 1 号用决斗 duel_1 指 0 号：0 号用 black_nullify 当杀打出
    submit(
        &mut e2,
        1,
        GameAction::PlayCard {
            card: duel_1,
            targets: vec![0],
        },
    );
    let ch_duel = e2.open_choice().expect("duel_sha with jinyan black trick");
    assert_eq!(ch_duel.observer, 0);
    assert_eq!(ch_duel.spec.prompt, "duel_sha");
    assert!(ch_duel.spec.options.contains(&ChoiceOption::Card(black_nullify)));
    answer_open(&mut e2, vec![ChoiceOption::Card(black_nullify)]);
    // 1 号无杀直接受 1 点伤害
    assert!(e2.open_choice().is_none());

    // ③ DyingSave 救援面：0 号持「体力<=2 禁用黑锦囊」（含 cardSavable: false）
    // 即便有测试视为技允许黑牌当桃救人，hp<=2 持禁言时黑锦囊仍被 cardSavable 禁用；hp=3 时可救！
    let dsl = r#"
skill test_black_tao(99100501) {
    view_as tao response black from hand;
}
"#;
    let pkg = content_compiler::compile_source_named("test_black_tao", dsl).expect("compile dsl");
    let vm_out = content_native_loader::bridge_vm_skill_decls(&pkg).expect("bridge vm skill");
    let pack_sav = ExtensionPack {
        def: ExtensionDef {
            name: "test-jinyan-black-tao",
            version: 1,
            description: "test black tao response for jinyan cardSavable",
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills: vm_out.decls,
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    let mut e_sav = Engine::new_with_extensions(7731, 3, &[&pack, &pack_sav]).unwrap();
    for _ in 0..80 {
        if e_sav.turn().current == 0 && e_sav.turn().phase == Phase::Play && e_sav.open_choice().is_none() {
            break;
        }
        if let Some(c) = e_sav.open_choice().cloned() {
            let sel = vec![c.spec.options[0]];
            submit(&mut e_sav, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
            continue;
        }
        let act = e_sav.turn().current;
        submit(&mut e_sav, act, GameAction::NextPhase);
    }
    for p in 0..3 {
        e_sav.debug_clear_hand(p);
    }
    e_sav.debug_grant(1, "jinyan");
    e_sav.debug_grant(1, "test_black_tao");
    let sav_sha = find_deck_kind(&e_sav, CardKind::Sha, 0);
    let sav_black_trick = e_sav
        .save()
        .core
        .deck
        .iter()
        .copied()
        .find(|&c| {
            let k = e_sav.catalog().card_kind(c);
            !k.is_basic_kind()
                && k.equip_slot().is_none()
                && k.is_basic_trick_or_equip()
                && matches!(e_sav.catalog().cards[c as usize].suit, Suit::Spade | Suit::Club)
        })
        .expect("black trick");
    let mut ss = e_sav.save();
    ss.core.deck.retain(|c| *c != sav_sha && *c != sav_black_trick);
    ss.core.player_mut(0).hand = vec![sav_sha];
    ss.core.player_mut(1).hp = 2; // <=2: 禁言封锁黑锦囊当桃救人！
    ss.core.player_mut(1).hand = vec![sav_black_trick];
    ss.core.player_mut(2).hp = 1;
    ss.core.player_mut(2).hand.clear();
    let mut e_sav_ban = Engine::restore_with_extensions(&ss, &[&pack, &pack_sav]).unwrap();
    submit(&mut e_sav_ban, 0, GameAction::PlayCard { card: sav_sha, targets: vec![2] });
    // 2 号濒死，但 1 号 hp=2 持禁言，黑锦囊被 cardSavable 禁用，不得打开 dying_save！
    assert!(
        !e_sav_ban.state_core().player(2).alive,
        "hp<=2 持禁言者黑锦囊受 cardSavable 禁用，不得救活濒死者"
    );

    // hp=3 时解除禁言：1 号可用该黑锦囊当桃救活 2 号！
    ss.core.player_mut(1).hp = 3;
    let mut e_sav_ok = Engine::restore_with_extensions(&ss, &[&pack, &pack_sav]).unwrap();
    submit(&mut e_sav_ok, 0, GameAction::PlayCard { card: sav_sha, targets: vec![2] });
    let ch_dying = e_sav_ok.open_choice().expect("hp=3 时黑锦囊可救濒死");
    assert_eq!(ch_dying.observer, 1);
    assert_eq!(ch_dying.spec.prompt, "dying_save");
    assert!(ch_dying.spec.options.contains(&ChoiceOption::Card(sav_black_trick)));
    answer_open(&mut e_sav_ok, vec![ChoiceOption::Card(sav_black_trick)]);
    assert!(e_sav_ok.state_core().player(2).alive);
}

#[test]
fn u07_verifier_five_rejections_fix() {
    fn recycle_hand_to_discard(core: &mut engine_native::state::GameStateCore, p: u8) {
        let old: Vec<u32> = core.player_mut(p).hand.drain(..).collect();
        core.discard.extend(old);
    }
    fn move_to_hand(core: &mut engine_native::state::GameStateCore, p: u8, cid: u32) {
        core.deck.retain(|c| *c != cid);
        core.discard.retain(|c| *c != cid);
        for pl in &mut core.players {
            pl.hand.retain(|c| *c != cid);
        }
        core.player_mut(p).hand.push(cid);
    }

    // 1) zhucheng2: defender gets optional trigger_confirm first; decline resolves Sha without guard prompt
    let (mut e1, pack1) = setup_equiv(3);
    e1.debug_grant(1, "zhucheng2");
    let zc1 = find_deck_kind(&e1, CardKind::Sha, 0);
    let sha1 = find_deck_kind(&e1, CardKind::Sha, 1);
    let filler1 = find_deck_kind(&e1, CardKind::Wuzhong, 0);
    let filler2 = find_deck_kind(&e1, CardKind::Tao, 0);
    let mut s1 = e1.save();
    s1.core.deck.retain(|c| *c != zc1);
    s1.core.player_mut(1).zhucheng = vec![zc1];
    recycle_hand_to_discard(&mut s1.core, 0);
    recycle_hand_to_discard(&mut s1.core, 1);
    move_to_hand(&mut s1.core, 0, sha1);
    move_to_hand(&mut s1.core, 0, filler1);
    move_to_hand(&mut s1.core, 0, filler2);
    let mut e1a = Engine::restore_with_extensions(&s1, &[&pack1]).unwrap();
    submit(&mut e1a, 0, GameAction::PlayCard { card: sha1, targets: vec![1] });
    let tc = e1a.open_choice().expect("zhucheng2 must ask defender trigger_confirm first");
    assert_eq!(tc.spec.prompt, "trigger_confirm");
    assert_eq!(tc.observer, 1);
    // Decline -> Sha hits seat 1 without opening wudian_zhucheng_guard!
    let hp1_before = e1a.debug_hp(1);
    answer_open(&mut e1a, Vec::new());
    assert!(e1a.open_choice().is_none(), "declining zhucheng2 must not open wudian_zhucheng_guard");
    assert_eq!(e1a.debug_hp(1), hp1_before - 1);

    // Confirm -> opens wudian_zhucheng_guard (and preserves save/restore)
    let mut e1b = Engine::restore_with_extensions(&s1, &[&pack1]).unwrap();
    submit(&mut e1b, 0, GameAction::PlayCard { card: sha1, targets: vec![1] });
    assert_eq!(e1b.open_choice().unwrap().spec.prompt, "trigger_confirm");
    answer_open(&mut e1b, vec![ChoiceOption::Confirm]);
    let bytes_zc = encode_archive(&e1b.save());
    let mut e1b_res = Engine::restore_with_extensions(&decode_archive(&bytes_zc).unwrap(), &[&pack1]).unwrap();
    let guard = e1b_res.open_choice().expect("wudian_zhucheng_guard after Confirm");
    assert_eq!(guard.spec.prompt, "wudian_zhucheng_guard");
    assert_eq!(guard.observer, 0);
    answer_open(&mut e1b_res, Vec::new());
    assert_eq!(e1b_res.debug_hp(1), hp1_before);

    // 2) youdishenru / g_youdishenru: shaBefore (before Renwang!) + NullifyWindow (_wuxie) + CardUsed
    let (e2, pack2) = setup_equiv(3);
    let black_sha = e2
        .state_core()
        .deck
        .iter()
        .copied()
        .find(|&c| {
            e2.catalog().card_kind(c) == CardKind::Sha
                && matches!(e2.catalog().cards[c as usize].suit, Suit::Spade | Suit::Club)
        })
        .expect("black sha");
    let renwang = find_deck_kind(&e2, CardKind::Renwang, 0);
    let yd_card = find_deck_kind(&e2, CardKind::Youdishenru, 0);
    let nul_card = find_deck_kind(&e2, CardKind::Nullify, 0);
    let mut s2 = e2.save();
    recycle_hand_to_discard(&mut s2.core, 0);
    recycle_hand_to_discard(&mut s2.core, 1);
    recycle_hand_to_discard(&mut s2.core, 2);
    move_to_hand(&mut s2.core, 0, black_sha);
    move_to_hand(&mut s2.core, 1, yd_card);
    s2.core.deck.retain(|c| *c != renwang);
    s2.core.player_mut(1).equips = vec![renwang];
    move_to_hand(&mut s2.core, 2, nul_card);
    let mut e2_run = Engine::restore_with_extensions(&s2, &[&pack2]).unwrap();
    let out2 = submit(&mut e2_run, 0, GameAction::PlayCard { card: black_sha, targets: vec![1] });
    assert!(matches!(out2, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    let yd_prompt = e2_run.open_choice().expect("g_youdishenru must fire at shaBefore before Renwang!");
    assert_eq!(yd_prompt.spec.prompt, "wudian_youdishenru_use");
    assert_eq!(yd_prompt.observer, 1);
    // AI public-info check on wudian_youdishenru_use
    let ai_yd = e2_run.plan_action(1, &BoundedPlanner::new(64));
    assert!(matches!(ai_yd.action, GameAction::Answer { .. }));
    let bytes_yd = encode_archive(&e2_run.save());
    let mut e2_res = Engine::restore_with_extensions(&decode_archive(&bytes_yd).unwrap(), &[&pack2]).unwrap();
    assert_eq!(e2_res.open_choice().unwrap().spec.prompt, "wudian_youdishenru_use");
    answer_open(&mut e2_res, vec![ChoiceOption::Card(yd_card)]);
    // NullifyWindow (_wuxie) must open for seat 2!
    let wx = e2_res.open_choice().expect("youdishenru must open nullify_window");
    assert_eq!(wx.spec.prompt, "nullify_window");
    assert_eq!(wx.observer, 2);
    // Seat 2 nullifies youdishenru -> original Black Sha continues & gets blocked by Renwang!
    answer_open(&mut e2_res, vec![ChoiceOption::Card(nul_card)]);
    assert!(e2_res.open_choice().is_none());
    assert_eq!(e2_res.debug_hp(1), 4, "Renwang blocks Black Sha after Youdishenru is nullified");
    assert_eq!(e2_res.debug_hp(0), 4, "Nullified Youdishenru deals no counter damage");

    // 2b) Non-nullified Youdishenru emits CardUsed (not CardResponded for yd), cancels Sha, loops/damages attacker
    let mut e2b = Engine::restore_with_extensions(&s2, &[&pack2]).unwrap();
    submit(&mut e2b, 0, GameAction::PlayCard { card: black_sha, targets: vec![1] });
    answer_open(&mut e2b, vec![ChoiceOption::Card(yd_card)]);
    assert_eq!(e2b.open_choice().unwrap().spec.prompt, "nullify_window");
    answer_open(&mut e2b, vec![ChoiceOption::Pass]);
    assert_eq!(e2b.debug_hp(1), 4, "Youdishenru cancels Black Sha");
    assert_eq!(e2b.debug_hp(0), 3, "No Sha -> attacker takes 1 counter damage");
    assert!(
        e2b.log().iter().any(|ev| matches!(ev, LogEvent::CardPlayed { player: 1, card, .. } if *card == yd_card)),
        "Youdishenru must emit CardPlayed (CardUsed) log"
    );
    assert!(
        !e2b.log().iter().any(|ev| matches!(ev, LogEvent::CardResponded { card, .. } if *card == yd_card)),
        "Youdishenru must NOT emit CardResponded"
    );

    // 3) diaobingqianjiang: per-target NullifyWindow (_wuxie) cancels target 1's pick/swap!
    let (e3, pack3) = setup_equiv(3);
    let db_card = e3
        .state_core()
        .deck
        .iter()
        .copied()
        .find(|&c| e3.catalog().cards[c as usize].name.contains("diaobingqianjiang"))
        .expect("diaobing card");
    let nul3 = find_deck_kind(&e3, CardKind::Nullify, 0);
    let h0 = find_deck_kind(&e3, CardKind::Sha, 0);
    let h1 = find_deck_kind(&e3, CardKind::Shan, 0);
    let h2 = find_deck_kind(&e3, CardKind::Tao, 0);
    let mut s3 = e3.save();
    recycle_hand_to_discard(&mut s3.core, 0);
    recycle_hand_to_discard(&mut s3.core, 1);
    recycle_hand_to_discard(&mut s3.core, 2);
    move_to_hand(&mut s3.core, 0, db_card);
    move_to_hand(&mut s3.core, 0, h0);
    move_to_hand(&mut s3.core, 1, h1);
    move_to_hand(&mut s3.core, 2, h2);
    move_to_hand(&mut s3.core, 2, nul3);
    let mut e3_run = Engine::restore_with_extensions(&s3, &[&pack3]).unwrap();
    submit(&mut e3_run, 0, GameAction::PlayCard { card: db_card, targets: vec![0, 1, 2] });
    // Target 0 NullifyWindow: seat 2 passes
    let wx0 = e3_run.open_choice().expect("nullify_window for target 0 of diaobingqianjiang");
    assert_eq!(wx0.spec.prompt, "nullify_window");
    answer_open(&mut e3_run, vec![ChoiceOption::Pass]);
    // Target 0 picks and skips swap
    assert_eq!(e3_run.open_choice().unwrap().spec.prompt, "wudian_diaobing_pick");
    answer_open(&mut e3_run, Vec::new());
    // Target 1 NullifyWindow: seat 2 uses Nullify -> target 1 skipped!
    let wx1 = e3_run.open_choice().expect("nullify_window for target 1 of diaobingqianjiang");
    assert_eq!(wx1.spec.prompt, "nullify_window");
    let bytes_db = encode_archive(&e3_run.save());
    let mut e3_res = Engine::restore_with_extensions(&decode_archive(&bytes_db).unwrap(), &[&pack3]).unwrap();
    assert_eq!(e3_res.open_choice().unwrap().spec.prompt, "nullify_window");
    answer_open(&mut e3_res, vec![ChoiceOption::Card(nul3)]);
    // Target 1 must be skipped -> Target 2 picks and skips swap -> wudian_diaobing_top!
    let c_t2 = e3_res.open_choice().expect("wudian_diaobing_pick for target 2 after target 1 nullified");
    assert_eq!(c_t2.observer, 2);
    assert_eq!(c_t2.spec.prompt, "wudian_diaobing_pick");
    answer_open(&mut e3_res, Vec::new());
    assert_eq!(e3_res.open_choice().unwrap().spec.prompt, "wudian_diaobing_top");
    assert_eq!(e3_res.debug_hand(1), vec![h1], "nullified target 1 did not swap market card");

    // 3b) Virtual choudu -> diaobingqianjiang also opens per-target NullifyWindow (_wuxie)
    let (mut e3v, pack3v) = setup_equiv(3);
    e3v.debug_grant(0, "choudu");
    let cd_def = e3v.catalog().skill_by_name("choudu").unwrap();
    let cd_cost = find_deck_kind(&e3v, CardKind::Sha, 0);
    let cd_h1 = find_deck_kind(&e3v, CardKind::Shan, 0);
    let cd_h2 = find_deck_kind(&e3v, CardKind::Tao, 0);
    let cd_nul = find_deck_kind(&e3v, CardKind::Nullify, 0);
    let mut s3v = e3v.save();
    recycle_hand_to_discard(&mut s3v.core, 0);
    recycle_hand_to_discard(&mut s3v.core, 1);
    recycle_hand_to_discard(&mut s3v.core, 2);
    s3v.core.deck.retain(|c| *c < 1000 && ![cd_cost, cd_h1, cd_h2, cd_nul].contains(c));
    move_to_hand(&mut s3v.core, 0, cd_cost);
    move_to_hand(&mut s3v.core, 1, cd_h1);
    move_to_hand(&mut s3v.core, 2, cd_h2);
    move_to_hand(&mut s3v.core, 2, cd_nul);
    let mut e3v_run = Engine::restore_with_extensions(&s3v, &[&pack3v]).unwrap();
    submit(&mut e3v_run, 0, GameAction::ActivateSkill { skill: cd_def });
    assert_eq!(e3v_run.open_choice().unwrap().spec.prompt, "wudian_choudu_card");
    answer_open(&mut e3v_run, vec![ChoiceOption::Card(cd_cost)]);
    assert_eq!(e3v_run.open_choice().unwrap().spec.prompt, "wudian_choudu_target");
    answer_open(&mut e3v_run, vec![ChoiceOption::Player(1)]);
    // Seat 1 has NullifyWindow for target 1 first -> seat 2 can nullify virtual diaobingqianjiang!
    let vwx = e3v_run.open_choice().expect("virtual choudu diaobingqianjiang must open nullify_window");
    assert_eq!(vwx.spec.prompt, "nullify_window");
    assert_eq!(vwx.observer, 2);
    roundtrip_and_answer(&mut e3v_run, &pack3v, vec![ChoiceOption::Card(cd_nul)]);
    // Target 1 skipped, next alive seat 2 has NullifyWindow or goes straight to pick if no more Nullify!
    while e3v_run
        .open_choice()
        .is_some_and(|c| c.spec.prompt == "nullify_window")
    {
        answer_open(&mut e3v_run, vec![ChoiceOption::Pass]);
    }
    assert_ne!(e3v_run.open_choice().unwrap().observer, 1, "nullified target 1 skipped on virtual diaobingqianjiang");
    assert_eq!(e3v_run.debug_hand(1).len(), 2, "seat 1 kept initial hand + drew 1, did not swap after nullify");

    // 4) wdb_xizhenxihong / 西真夕红 must NOT carry jianbi in formal wudian package (coexistence pack retains frozen u01 lock)
    let formal_wudian = u07_equiv_pack();
    let xzx = formal_wudian
        .generals
        .iter()
        .find(|g| matches!(g.name, "西真夕红" | "wdb_xizhenxihong"))
        .expect("西真夕红 in formal wudian pack");
    assert!(!xzx.skills.contains(&"jianbi"), "formal 西真夕红 must NOT have jianbi: {:?}", xzx.skills);
    assert!(xzx.skills.contains(&"fuchou") && xzx.skills.contains(&"jinyan"));

    // 5) kangyin2 & ciqiu_dying cleared on turn handoff
    let (e5, pack5) = setup_equiv(3);
    let mut s5 = e5.save();
    s5.core.turn.kangyin2 = Some((0, 2));
    s5.core.turn.ciqiu_dying = vec![1];
    let mut e5_run = Engine::restore_with_extensions(&s5, &[&pack5]).unwrap();
    while e5_run.turn().current == 0 {
        while let Some(_) = e5_run.open_choice() {
            answer_open(&mut e5_run, Vec::new());
        }
        let cur = e5_run.turn().current;
        submit(&mut e5_run, cur, GameAction::NextPhase);
    }
    assert!(e5_run.state_core().turn.kangyin2.is_none(), "kangyin2 must clear on turn change");
    assert!(e5_run.state_core().turn.ciqiu_dying.is_empty(), "ciqiu_dying must clear on turn change");
}
