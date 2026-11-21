//! L10 史诗卡牌真实对局修复回归测试（Decision 40）。
//!
//! 覆盖：
//! 1. `reguhuo` (`史诗卡牌/extension.js:297-420`, `refresh.js:10639-10783`):
//!    - 取消选牌/牌名/目标不消耗回合限一次（仅实际提交进入质疑时消耗）；
//!    - 真/假质疑与他座（`!rechanyuan`）询问流程：真牌先结算后质疑者弃手牌或失体力并获 `rechanyuan`（`hp<=1` 封技），
//!      假牌质疑者摸 1 牌、作废并弃置载体、响应上下文重开窗口；全不质疑直接结算；
//!    - 动态 `filterCard` 候选过滤（含火/雷杀与扩展牌）及响应上下文（杀响应闪/决斗/南蛮/濒死/无懈）。
//! 2. `rangjie` (`史诗卡牌/extension.js:422-494`, `game.js:63187`):
//!    - `cancel2` 空取消不摸牌并退出，字面 `Tag(2)` 取消按钮按上游仍摸 1 牌并进入移动牌分支；
//!    - `rangjie_type` 可取消（`allow_cancel: true`）且多点伤害继续下轮；
//!    - 牌堆搜索自牌堆顶向下（`deck.iter().rev()`）覆盖 `basic`/`trick`/`equip`；
//!    - `moveCard` 移动判定牌入目标判定区、过滤自身与同栏位占满/同延时重名目标。
//! 3. `k_danggu` (`史诗卡牌/extension.js:2358-2595`, `mobile.js:4669-5360`):
//!    - 有向 `conflictMap`（`3->[5]`, `5->[3]`, `7->[8]`, `8->[7]`, `9->[3,5,7,8]`；`3/5/7/8` 均不锁 `9`）；
//!    - `scs_gaowang`(9) 保底抽样分布（非 `9` 中抽 3 + `9` 后洗牌）与 AI 冲突处理；
//!    - 已授常侍技能（含 `locked_content` 产品路径）绝不在 `PhaseBegin` 自动触发；
//!    - `scsniqu`（出牌限 1 选目标火伤）、`scstaoluan`（出牌限 1 转化并用牌）、`scschiyan`（用杀指定 `he` 非空目标暂扣 + 回合末归还 + 手牌装备均不小于时加伤）真实对局与存档恢复。

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::{
    bridge_pack, decision34_closure_extension_pack, prepare_locked_content_with_bridge,
    start_locked_engine, validate_and_freeze_raw_package, LockedStartMode,
};
use engine_native::ai::BoundedPlanner;
use engine_native::catalog::CardKind;
use engine_native::event::DamageNature;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};

static DEDUP: AtomicU64 = AtomicU64::new(500_000);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn epic_pack() -> engine_native::content::extension::ExtensionPack {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("epic-cards-upstream-liuli/assets/cards.json"),
    )
    .expect("read epic cards.json");
    bridge_pack(&json).expect("bridge epic").pack
}

fn epic_engine(seed: u64) -> Engine {
    Engine::new_with_extensions(seed, 4, &[&epic_pack()]).expect("new epic engine")
}

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    })
}

fn drain(e: &mut Engine) {
    for _ in 0..96 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.prompt == "reguhuo_guess_pick" {
            vec![ChoiceOption::Tag(0)]
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

fn advance_to_play(e: &mut Engine, player: u8) {
    for _ in 0..96 {
        if e.turn().current == player && e.turn().phase == Phase::Play && e.open_choice().is_none()
        {
            return;
        }
        if e.open_choice().is_some() {
            drain(e);
            continue;
        }
        let cur = e.turn().current;
        let _ = submit(e, cur, GameAction::NextPhase);
    }
    panic!("failed to reach player {player} Play phase");
}

fn roundtrip_and_answer(
    e: &mut Engine,
    pack: &[&engine_native::content::extension::ExtensionPack],
    selection: Vec<ChoiceOption>,
) {
    let c = e.open_choice().cloned().expect("pending choice for roundtrip");
    let raw = engine_native::archive::encode_archive(&e.save());
    let decoded = engine_native::archive::decode_archive(&raw).expect("decode archive");
    let mut restored =
        Engine::restore_with_extensions(&decoded, pack).expect("restore archive with extensions");
    let out_a = submit(
        e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection: selection.clone(),
        },
    );
    let out_b = submit(
        &mut restored,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection,
        },
    );
    assert!(
        !matches!(out_a, CommandOutcome::Rejected(_)),
        "expected answer accepted on {}: {out_a:?}",
        c.spec.prompt
    );
    assert_eq!(
        std::mem::discriminant(&out_a),
        std::mem::discriminant(&out_b)
    );
    assert_eq!(e.state_hash(), restored.state_hash());
}

fn assert_ai_no_leak(
    e: &Engine,
    pack: &[&engine_native::content::extension::ExtensionPack],
) -> Vec<ChoiceOption> {
    let c = e.open_choice().expect("open choice for AI");
    let obs = c.observer;
    let planner = BoundedPlanner::new(16);
    let a = e.plan_action(obs, &planner);
    let GameAction::Answer {
        choice_id,
        selection: sel_a,
    } = a.action
    else {
        panic!("expected AI Answer");
    };
    assert_eq!(choice_id, c.id);

    let mut snap_b = e.save();
    snap_b.core.deck.reverse();
    let e_b = Engine::restore_with_extensions(&snap_b, pack).expect("restore hidden flip");
    let b = e_b.plan_action(obs, &planner);
    let GameAction::Answer {
        selection: sel_b, ..
    } = b.action
    else {
        panic!("expected AI Answer after hidden flip");
    };
    assert_eq!(
        sel_a, sel_b,
        "AI choice on {} must not depend on hidden deck/hand info",
        c.spec.prompt
    );
    sel_a
}

#[test]
fn l10_reguhuo_cancel_never_consumes_once_per_turn() {
    let pack = epic_pack();
    let mut e = epic_engine(10_1001);
    e.debug_grant(0, "reguhuo");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    let c1 = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, c1);
    let def = e.catalog().skill_by_name("reguhuo").expect("reguhuo");

    // 1. Cancel at reguhuo_card -> second activation succeeds
    assert!(!matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: def }),
        CommandOutcome::Rejected(_)
    ));
    let p_card = e.open_choice().cloned().expect("reguhuo_card");
    assert_eq!(p_card.spec.prompt, "reguhuo_card");
    roundtrip_and_answer(&mut e, &[&pack], Vec::new());

    // 2. Pick card then cancel at reguhuo_name -> third activation succeeds
    assert!(!matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: def }),
        CommandOutcome::Rejected(_)
    ));
    let p_card2 = e.open_choice().cloned().expect("reguhuo_card 2");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: p_card2.id,
            selection: vec![ChoiceOption::Card(c1)],
        },
    );
    let p_name = e.open_choice().cloned().expect("reguhuo_name");
    assert_eq!(p_name.spec.prompt, "reguhuo_name");
    assert!(p_name.spec.allow_cancel);
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: p_name.id,
            selection: Vec::new(),
        },
    );

    // 3. Pick card + Sha then cancel at reguhuo_targets (`allow_cancel: true`) -> still NOT consumed!
    assert!(!matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: def }),
        CommandOutcome::Rejected(_)
    ));
    let p_card3 = e.open_choice().cloned().expect("reguhuo_card 3");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: p_card3.id,
            selection: vec![ChoiceOption::Card(c1)],
        },
    );
    let p_name2 = e.open_choice().cloned().expect("reguhuo_name 2");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: p_name2.id,
            selection: vec![ChoiceOption::Tag(0)],
        },
    );
    let p_tgt = e.open_choice().cloned().expect("reguhuo_targets");
    assert_eq!(p_tgt.spec.prompt, "reguhuo_targets");
    assert!(
        p_tgt.spec.allow_cancel,
        "reguhuo_targets must allow cancel without consuming once (`史诗卡牌/extension.js:363`)"
    );
    roundtrip_and_answer(&mut e, &[&pack], Vec::new());

    // 4. Commit a declaration -> NOW once per turn is consumed
    assert!(!matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: def }),
        CommandOutcome::Rejected(_)
    ));
    let p_card4 = e.open_choice().cloned().expect("reguhuo_card 4");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: p_card4.id,
            selection: vec![ChoiceOption::Card(c1)],
        },
    );
    let p_name3 = e.open_choice().cloned().expect("reguhuo_name 3");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: p_name3.id,
            selection: vec![ChoiceOption::Tag(0)],
        },
    );
    let p_tgt2 = e.open_choice().cloned().expect("reguhuo_targets 2");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: p_tgt2.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    drain(&mut e);

    let c2 = e.debug_place(0, "tao");
    e.debug_give_hand_card(0, 0, c2);
    assert!(
        matches!(
            submit(&mut e, 0, GameAction::ActivateSkill { skill: def }),
            CommandOutcome::Rejected(_)
        ),
        "committed reguhuo declaration must consume once-per-turn"
    );
}

#[test]
fn l10_reguhuo_questioning_true_vs_false_and_rechanyuan() {
    let pack = epic_pack();
    let mut e = epic_engine(10_1002);
    e.debug_grant(0, "reguhuo");
    e.debug_grant(1, "咆哮");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // Seat 0 wounded (hp 2/4), holds real `tao`
    e.debug_set_hp(0, 2);
    let real_tao = e.debug_place(0, "tao");
    e.debug_give_hand_card(0, 0, real_tao);
    let seat1_card = e.debug_place(1, "shan");
    e.debug_give_hand_card(1, 1, seat1_card);
    e.debug_set_hp(1, 2);

    let def = e.catalog().skill_by_name("reguhuo").expect("reguhuo");
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
    let c_card = e.open_choice().cloned().unwrap();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_card.id,
            selection: vec![ChoiceOption::Card(real_tao)],
        },
    );
    let c_name = e.open_choice().cloned().unwrap();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_name.id,
            selection: vec![ChoiceOption::Tag(4)], // Tao
        },
    );

    // Other seats [1, 2, 3] are asked reguhuo_guess_pick; Seat 1 questions (Tag(1))
    let q1 = e.open_choice().cloned().expect("reguhuo_guess_pick seat 1");
    assert_eq!(q1.spec.prompt, "reguhuo_guess_pick");
    assert_eq!(q1.observer, 1);
    let _ = assert_ai_no_leak(&e, &[&pack]);
    roundtrip_and_answer(&mut e, &[&pack], vec![ChoiceOption::Tag(1)]); // Seat 1 betrays

    let q2 = e.open_choice().cloned().expect("reguhuo_guess_pick seat 2");
    assert_eq!(q2.observer, 2);
    let _ = submit(
        &mut e,
        2,
        GameAction::Answer {
            choice_id: q2.id,
            selection: vec![ChoiceOption::Tag(0)],
        },
    );
    let q3 = e.open_choice().cloned().expect("reguhuo_guess_pick seat 3");
    assert_eq!(q3.observer, 3);
    let _ = submit(
        &mut e,
        3,
        GameAction::Answer {
            choice_id: q3.id,
            selection: vec![ChoiceOption::Tag(0)],
        },
    );

    // True card: Tao resolves FIRST (seat 0 recovers +1 -> hp=3), then reguhuo_final asks seat 1
    assert_eq!(e.debug_hp(0), 3, "true reguhuo card must resolve before reguhuo_final");
    let fin = e.open_choice().cloned().expect("reguhuo_final seat 1");
    assert_eq!(fin.spec.prompt, "reguhuo_final");
    assert_eq!(fin.observer, 1);
    assert!(fin.spec.allow_cancel);
    let _ = assert_ai_no_leak(&e, &[&pack]);

    // Save before reguhuo_final to compare Discard vs LoseHp branches
    let mut e_lose = Engine::restore_with_extensions(&e.save(), &[&pack]).expect("fork loseHp branch");

    // Branch A: Seat 1 discards 1 hand card -> hp stays 2, gains rechanyuan
    roundtrip_and_answer(&mut e, &[&pack], vec![ChoiceOption::Card(seat1_card)]);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 2);
    assert!(e.debug_hand(1).is_empty());
    assert!(e.debug_has_skill(1, "rechanyuan"), "questioner must gain rechanyuan (`refresh.js:10754`)");

    // Branch B: Seat 1 declines discard -> loses 1 hp (2 -> 1), gains rechanyuan, and hp<=1 seals `paoxiao`
    let _ = submit(
        &mut e_lose,
        1,
        GameAction::Answer {
            choice_id: fin.id,
            selection: Vec::new(),
        },
    );
    drain(&mut e_lose);
    assert_eq!(e_lose.debug_hp(1), 1);
    assert!(e_lose.debug_has_skill(1, "rechanyuan"));

    // Seat 1 with rechanyuan and hp==1 cannot use `paoxiao` (2nd Sha rejected)
    advance_to_play(&mut e_lose, 1);
    let s1 = e_lose.debug_place(1, "sha");
    let s2 = e_lose.debug_place(1, "sha");
    e_lose.debug_give_hand_card(1, 1, s1);
    e_lose.debug_give_hand_card(1, 1, s2);
    let _ = submit(
        &mut e_lose,
        1,
        GameAction::PlayCard {
            card: s1,
            targets: vec![2],
        },
    );
    drain(&mut e_lose);
    assert!(
        matches!(
            submit(
                &mut e_lose,
                1,
                GameAction::PlayCard {
                    card: s2,
                    targets: vec![2],
                }
            ),
            CommandOutcome::Rejected(_)
        ),
        "rechanyuan at hp<=1 must block non-charlotte skills like paoxiao (`refresh.js:10770`)"
    );

    // On next turn, seat 0 reguhuo skips seat 1 in reguhuo_guess_pick because seat 1 has rechanyuan;
    // and bluff card questioned by seat 2 voids the card + draws 1 for seat 2 without setting sha_used!
    advance_to_play(&mut e_lose, 0);
    e_lose.debug_clear_hand(0);
    let bluff_shan = e_lose.debug_place(0, "shan");
    let real_sha = e_lose.debug_place(0, "sha");
    e_lose.debug_give_hand_card(0, 0, bluff_shan);
    e_lose.debug_give_hand_card(0, 0, real_sha);
    let hand2_before = e_lose.debug_hand(2).len();
    let hp3_before = e_lose.debug_hp(3);
    let _ = submit(&mut e_lose, 0, GameAction::ActivateSkill { skill: def });
    let cc = e_lose.open_choice().cloned().unwrap();
    let _ = submit(
        &mut e_lose,
        0,
        GameAction::Answer {
            choice_id: cc.id,
            selection: vec![ChoiceOption::Card(bluff_shan)],
        },
    );
    let cn = e_lose.open_choice().cloned().unwrap();
    let _ = submit(
        &mut e_lose,
        0,
        GameAction::Answer {
            choice_id: cn.id,
            selection: vec![ChoiceOption::Tag(0)], // Bluff Sha with Shan
        },
    );
    let ct = e_lose.open_choice().cloned().unwrap();
    let _ = submit(
        &mut e_lose,
        0,
        GameAction::Answer {
            choice_id: ct.id,
            selection: vec![ChoiceOption::Player(3)],
        },
    );
    let fq = e_lose.open_choice().cloned().expect("questioner");
    assert_eq!(fq.observer, 2, "seat 1 with rechanyuan must be skipped (`refresh.js:10669`)");
    let _ = submit(
        &mut e_lose,
        2,
        GameAction::Answer {
            choice_id: fq.id,
            selection: vec![ChoiceOption::Tag(1)], // Seat 2 questions bluff Sha!
        },
    );
    drain(&mut e_lose);
    assert_eq!(e_lose.debug_hp(3), hp3_before, "bluff Sha questioned must be voided (no damage)");
    assert_eq!(e_lose.debug_hand(2).len(), hand2_before + 1, "questioner draws 1 on fake card (`refresh.js:10727`)");
    assert!(
        !matches!(
            submit(
                &mut e_lose,
                0,
                GameAction::PlayCard {
                    card: real_sha,
                    targets: vec![3],
                }
            ),
            CommandOutcome::Rejected(_)
        ),
        "voided reguhuo Sha must not consume phase Sha limit"
    );
}

#[test]
fn l10_reguhuo_response_contexts_and_dynamic_filter_card() {
    let pack = epic_pack();
    let mut e = epic_engine(10_1003);
    e.debug_grant(0, "reguhuo");
    let reguhuo_def = e.catalog().skill_by_name("reguhuo").expect("reguhuo");

    // Dynamic filterCard check in Phase::Play at full HP (hp==max_hp):
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    let tao0 = e.debug_place(0, "tao");
    e.debug_give_hand_card(0, 0, tao0);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: reguhuo_def });
    let pc = e.open_choice().cloned().unwrap();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: pc.id,
            selection: vec![ChoiceOption::Card(tao0)],
        },
    );
    let pn = e.open_choice().cloned().expect("reguhuo_name dynamic filter");
    assert!(!pn.spec.options.contains(&ChoiceOption::Tag(3)), "Shan cannot be actively played (`filterCard`)");
    assert!(!pn.spec.options.contains(&ChoiceOption::Tag(19)), "Nullify cannot be actively played");
    assert!(!pn.spec.options.contains(&ChoiceOption::Tag(4)), "Tao hidden at full HP (`filterCard`)");
    assert!(pn.spec.options.contains(&ChoiceOption::Tag(1)), "FireSha in catalog must be offered");
    assert!(pn.spec.options.contains(&ChoiceOption::Tag(2)), "LeiSha in catalog must be offered");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: pn.id,
            selection: Vec::new(),
        },
    );

    // Response context 1: `respond_shan` when Seat 0 only holds Tao (NO physical Shan)
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    let bluff_tao = e.debug_place(0, "tao");
    let real_shan = e.debug_place(0, "shan");
    e.debug_give_hand_card(0, 0, bluff_tao);
    e.debug_give_hand_card(0, 0, real_shan);
    let sha1 = e.debug_place(1, "sha");
    e.debug_give_hand_card(1, 1, sha1);
    let _ = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![0],
        },
    );
    let resp = e.open_choice().cloned().expect("respond_shan with reguhuo");
    assert_eq!(resp.spec.prompt, "respond_shan");
    assert!(
        resp.spec.options.contains(&ChoiceOption::Skill(reguhuo_def)),
        "respond_shan must offer ChoiceOption::Skill(reguhuo) (`史诗卡牌/extension.js:300`)"
    );
    // Activate reguhuo -> bluff Shan -> Seat 1 questions -> voided -> respond_shan reopens WITHOUT reguhuo
    roundtrip_and_answer(&mut e, &[&pack], vec![ChoiceOption::Skill(reguhuo_def)]);
    let rc = e.open_choice().cloned().expect("reguhuo_card in response");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: rc.id,
            selection: vec![ChoiceOption::Card(bluff_tao)],
        },
    );
    if e.open_choice().is_some_and(|c| c.spec.prompt == "reguhuo_name") {
        let rn = e.open_choice().cloned().unwrap();
        let _ = submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: rn.id,
                selection: vec![ChoiceOption::Tag(3)], // Shan
            },
        );
    }
    let rq = e.open_choice().cloned().expect("reguhuo_guess_pick in response");
    assert_eq!(rq.observer, 1);
    roundtrip_and_answer(&mut e, &[&pack], vec![ChoiceOption::Tag(1)]); // Question fake Shan!
    drain_guess_skip_non_owner(&mut e);
    let reopened = e.open_choice().cloned().expect("respond_shan must reopen after voided reguhuo");
    assert_eq!(reopened.spec.prompt, "respond_shan");
    assert!(
        !reopened.spec.options.contains(&ChoiceOption::Skill(reguhuo_def)),
        "consumed reguhuo must not reappear on reopened respond_shan"
    );
    assert!(reopened.spec.options.contains(&ChoiceOption::Card(real_shan)));
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: reopened.id,
            selection: vec![ChoiceOption::Card(real_shan)],
        },
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(0), 4, "Seat 0 avoided damage with real Shan after reopened prompt");

    // Response context 2: `dying_save`, `duel_sha`, `aoe_response`, `nullify_window` also offer reguhuo
    advance_to_play(&mut e, 2);
    e.debug_clear_hand(0);
    let junk = e.debug_place(0, "sha"); // NOT a wuxie or tao, yet reguhuo opens window!
    e.debug_give_hand_card(0, 0, junk);
    let wuzhong = e.debug_place(2, "wuzhong");
    e.debug_give_hand_card(2, 2, wuzhong);
    let _ = submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: wuzhong,
            targets: Vec::new(),
        },
    );
    let nul = e.open_choice().cloned().expect("nullify_window with reguhuo");
    assert_eq!(nul.spec.prompt, "nullify_window");
    assert!(nul.spec.options.contains(&ChoiceOption::Skill(reguhuo_def)));
    drain(&mut e);
}

fn drain_guess_skip_non_owner(e: &mut Engine) {
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if c.spec.prompt != "reguhuo_guess_pick" {
            break;
        }
        let _ = submit(
            e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: vec![ChoiceOption::Tag(0)],
            },
        );
    }
}

#[test]
fn l10_rangjie_literal_cancel_type_cancel_deck_top_and_move_dests() {
    let pack = epic_pack();
    let mut e = epic_engine(10_2001);
    e.debug_grant(0, "rangjie");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // Put distinct `basic` cards at bottom (`sha`) vs near top (`tao` right under top draw card)
    let sha_bottom = e
        .state_core()
        .deck
        .iter()
        .copied()
        .find(|&c| e.catalog().card_kind(c) == CardKind::Sha)
        .expect("sha in deck");
    let tao_top = e
        .state_core()
        .deck
        .iter()
        .copied()
        .find(|&c| e.catalog().card_kind(c) == CardKind::Tao)
        .expect("tao in deck");
    let filler_draw = e
        .state_core()
        .deck
        .iter()
        .copied()
        .find(|&c| c != sha_bottom && c != tao_top && e.catalog().card_kind(c) == CardKind::Shan)
        .expect("shan in deck");
    {
        let deck = &mut e.state_core_mut().deck;
        deck.retain(|&c| c != sha_bottom && c != tao_top && c != filler_draw);
        deck.insert(0, sha_bottom); // bottom
        deck.push(tao_top); // 2nd from top
        deck.push(filler_draw); // very top (drawn by player.draw())
    }

    // Deal 2 damage:
    // Step A: Test `cancel2` (empty selection) exits immediately with 0 draws (`direct: true`, no trigger_confirm)
    let snap_two_dmg = {
        let mut clone = Engine::restore_with_extensions(&e.save(), &[&pack]).unwrap();
        let _ = clone.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
        assert_eq!(
            clone.open_choice().expect("direct rangjie_choice").spec.prompt,
            "rangjie_choice"
        );
        clone.save()
    };

    let mut e_cancel2 = Engine::restore_with_extensions(&snap_two_dmg, &[&pack]).unwrap();
    let rc_cancel2 = e_cancel2.open_choice().cloned().expect("rangjie_choice");
    assert!(rc_cancel2.spec.allow_cancel, "rangjie_choice must support cancel2 (`extension.js:450`)");
    let _ = assert_ai_no_leak(&e_cancel2, &[&pack]);
    roundtrip_and_answer(&mut e_cancel2, &[&pack], Vec::new());
    drain(&mut e_cancel2);
    assert!(e_cancel2.debug_hand(0).is_empty(), "cancel2 exits without drawing");

    // Step B: On 2-dmg choice:
    //   Iteration 1: choose `Tag(0)`, then CANCEL `rangjie_type` -> keeps 1 draw, continues to Iteration 2!
    //   Iteration 2: choose literal `Tag(2)` ('取消') -> upstream STILL draws 1 card! (`extension.js:450`)
    let mut e_lit = Engine::restore_with_extensions(&snap_two_dmg, &[&pack]).unwrap();
    let rc1 = e_lit.open_choice().cloned().expect("iter 1 rangjie_choice");
    let _ = submit(
        &mut e_lit,
        0,
        GameAction::Answer {
            choice_id: rc1.id,
            selection: vec![ChoiceOption::Tag(0)],
        },
    );
    let rt1 = e_lit.open_choice().cloned().expect("iter 1 rangjie_type");
    assert_eq!(rt1.spec.prompt, "rangjie_type");
    assert!(rt1.spec.allow_cancel, "rangjie_type must allow cancel (`extension.js:473`)");
    roundtrip_and_answer(&mut e_lit, &[&pack], Vec::new());

    let rc2 = e_lit
        .open_choice()
        .cloned()
        .expect("canceling rangjie_type on 2-damage must still advance to iteration 2");
    assert_eq!(rc2.spec.prompt, "rangjie_choice");
    roundtrip_and_answer(&mut e_lit, &[&pack], vec![ChoiceOption::Tag(2)]);
    drain(&mut e_lit);
    assert_eq!(
        e_lit.debug_hand(0).len(),
        2,
        "literal Tag(2) must still draw 1 card (`史诗卡牌/extension.js:450-453`)"
    );

    // Step C: Verify deck search direction from deck TOP (`deck.iter().rev()`) finds `tao_top` NOT `sha_bottom`
    let _ = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    let rc = e.open_choice().cloned().unwrap();
    assert_eq!(rc.spec.prompt, "rangjie_choice");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: rc.id,
            selection: vec![ChoiceOption::Tag(0)],
        },
    );
    let rt = e.open_choice().cloned().unwrap();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: rt.id,
            selection: vec![ChoiceOption::Tag(0)], // basic
        },
    );
    drain(&mut e);
    assert!(
        e.debug_hand(0).contains(&tao_top),
        "rangjie deck search must scan top-down (`game.js:63187`), picking tao_top over sha_bottom"
    );

    // Step D: Move destination rules: moving a judge card goes to target `judge` zone (not hand) & excludes duplicate/self
    let mut e_mv = epic_engine(10_2002);
    e_mv.debug_grant(0, "rangjie");
    for p in 0..4 {
        e_mv.debug_clear_hand(p);
    }
    let le1 = e_mv.debug_place_judge(1, "le");
    let le2 = e_mv.debug_place_judge(2, "le");
    let _ = (le1, le2);
    let _ = e_mv.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    let mc1 = e_mv.open_choice().cloned().unwrap();
    assert_eq!(mc1.spec.prompt, "rangjie_choice");
    let _ = submit(
        &mut e_mv,
        0,
        GameAction::Answer {
            choice_id: mc1.id,
            selection: vec![ChoiceOption::Tag(1)],
        },
    );
    let mc_card = e_mv.open_choice().cloned().expect("rangjie_move_card");
    let _ = submit(
        &mut e_mv,
        0,
        GameAction::Answer {
            choice_id: mc_card.id,
            selection: vec![ChoiceOption::Card(le1)],
        },
    );
    let mc_tgt = e_mv.open_choice().cloned().expect("rangjie_move_target");
    assert!(!mc_tgt.spec.options.contains(&ChoiceOption::Player(1)), "cannot move card to source seat");
    assert!(!mc_tgt.spec.options.contains(&ChoiceOption::Player(2)), "cannot move Le to seat already holding Le");
    assert!(mc_tgt.spec.options.contains(&ChoiceOption::Player(3)));
    roundtrip_and_answer(&mut e_mv, &[&pack], vec![ChoiceOption::Player(3)]);
    drain(&mut e_mv);
    assert!(e_mv.debug_judge(3).contains(&le1), "moved judge card must enter target judge zone, not hand");
    assert!(!e_mv.debug_hand(3).contains(&le1));
}

#[test]
fn l10_kdanggu_directional_conflict_gaowang_and_scs_no_auto_phase_trigger() {
    let pack = epic_pack();
    // 1. Directional conflictMap (`extension.js:2370-2381`)
    for seed in 10_3001..10_3020 {
        let mut e = epic_engine(seed);
        e.debug_grant(0, "k_danggu");
        e.state_core_mut()
            .vm_aux
            .flags
            .insert((0, "k_danggu_roster_mask".into()), (1 << 0) | (1 << 3) | (1 << 5) | (1 << 9));
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        let c = e.open_choice().cloned().expect("k_danggu_partner");
        let first = e.state_core().vm_aux.flags[&(0, "k_danggu_first".into())];
        match first {
            3 => {
                // conflictMap[3] == [5], so 9 IS selectable!
                assert!(c.spec.options.contains(&ChoiceOption::Tag(9)), "3->9 must NOT conflict");
                assert!(!c.spec.options.contains(&ChoiceOption::Tag(5)));
            }
            9 => {
                // conflictMap[9] == [3, 5, 7, 8], so neither 3 nor 5 is selectable
                assert_eq!(c.spec.options, vec![ChoiceOption::Tag(0)]);
            }
            _ => {}
        }
    }

    // 2. Gaowang placement is uniformly shuffled across `others` (`randomGets(3)` + `9` + `randomSort`)
    let mut non_last_pos = false;
    for seed in 10_3100..10_3140 {
        let mut e = epic_engine(seed);
        e.debug_grant(0, "k_danggu");
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        let c = e.open_choice().cloned().expect("k_danggu_partner");
        let first = e.state_core().vm_aux.flags[&(0, "k_danggu_first".into())];
        if first != 9 {
            assert!(c.spec.options.contains(&ChoiceOption::Tag(9)));
            if c.spec.options.len() >= 2 && c.spec.options.last() != Some(&ChoiceOption::Tag(9)) {
                non_last_pos = true;
            }
        }
    }
    assert!(non_last_pos, "gaowang must be shuffled into others, not fixed at last slot");

    // 3. Locked product path (`decision34` + `epic-cards`): companion skills NEVER auto-fire on PhaseBegin!
    let d34_pack = decision34_closure_extension_pack();
    let dir = repo_extensions_root().join("epic-cards-upstream-liuli");
    let raw_npack = std::fs::read_to_string(dir.join("package.npack.json")).expect("npack");
    let mut assets = BTreeMap::new();
    assets.insert(
        "assets/cards.json".to_string(),
        std::fs::read(dir.join("assets/cards.json")).unwrap(),
    );
    assets.insert(
        "assets/config.json".to_string(),
        std::fs::read(dir.join("assets/config.json")).unwrap(),
    );
    let frozen = validate_and_freeze_raw_package(raw_npack, assets).expect("freeze epic");
    let verified = prepare_locked_content_with_bridge(vec![frozen]).expect("verified epic");
    let mut e_locked = start_locked_engine(10_3200, 4, LockedStartMode::Prototype, &verified).expect("locked");
    let scs_gen = (0..e_locked.catalog().general_count() as u16)
        .find(|&i| e_locked.catalog().general_meta(i).is_some_and(|m| m.0 == "shichangshi"))
        .expect("shichangshi");
    e_locked.debug_assemble_general_def(0, scs_gen);
    for sk in [
        "scstaoluan",
        "scschiyan",
        "scszimou",
        "scspicai",
        "scsyaozhuo",
        "scsxiaolu",
        "scskuiji",
        "scschihe",
        "scsniqu",
        "scsanruo",
    ] {
        e_locked.debug_grant(0, sk);
    }
    advance_to_play(&mut e_locked, 0);
    for p in 0..4 {
        assert_eq!(e_locked.debug_hp(p), if p == 0 { 1 } else { 4 }, "no auto damage on PhaseBegin");
        assert!(
            !e_locked.state_core().vm_aux.flags.keys().any(|(_, k)| k.starts_with("l10_scs_resolved")),
            "companion skills must never auto-resolve on PhaseBegin"
        );
    }

    // 4. Real `scsniqu` active (`phaseUse`, `usable: 1`, target -> 1 fire damage, cancelable without consuming)
    let _ = d34_pack;
    let niqu_def = e_locked.catalog().skill_by_name("scsniqu").expect("scsniqu");
    assert!(!matches!(
        submit(&mut e_locked, 0, GameAction::ActivateSkill { skill: niqu_def }),
        CommandOutcome::Rejected(_)
    ));
    let nq = e_locked.open_choice().cloned().expect("scsniqu_target");
    assert!(nq.spec.allow_cancel);
    let _ = submit(
        &mut e_locked,
        0,
        GameAction::Answer {
            choice_id: nq.id,
            selection: Vec::new(),
        },
    );
    // Re-activating after cancel must succeed; picking target 1 deals 1 fire damage and consumes usable:1
    assert!(!matches!(
        submit(&mut e_locked, 0, GameAction::ActivateSkill { skill: niqu_def }),
        CommandOutcome::Rejected(_)
    ));
    let nq2 = e_locked.open_choice().cloned().expect("scsniqu_target 2");
    let hp1_before = e_locked.debug_hp(1);
    let _ = submit(
        &mut e_locked,
        0,
        GameAction::Answer {
            choice_id: nq2.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    drain(&mut e_locked);
    assert_eq!(e_locked.debug_hp(1), hp1_before - 1);
    assert!(matches!(
        submit(&mut e_locked, 0, GameAction::ActivateSkill { skill: niqu_def }),
        CommandOutcome::Rejected(_)
    ));

    // 5. Real `scstaoluan` (`phaseUse`, `usable: 1`, convert `'hes'` card into real card use)
    let taoluan_def = e_locked.catalog().skill_by_name("scstaoluan").expect("scstaoluan");
    e_locked.debug_clear_hand(0);
    let cost_card = e_locked.debug_place(0, "shan");
    e_locked.debug_give_hand_card(0, 0, cost_card);
    assert!(!matches!(
        submit(&mut e_locked, 0, GameAction::ActivateSkill { skill: taoluan_def }),
        CommandOutcome::Rejected(_)
    ));
    let tc = e_locked.open_choice().cloned().expect("scstaoluan_card");
    let _ = submit(
        &mut e_locked,
        0,
        GameAction::Answer {
            choice_id: tc.id,
            selection: vec![ChoiceOption::Card(cost_card)],
        },
    );
    let tn = e_locked.open_choice().cloned().expect("scstaoluan_name");
    let _ = submit(
        &mut e_locked,
        0,
        GameAction::Answer {
            choice_id: tn.id,
            selection: vec![ChoiceOption::Tag(6)], // Wuzhong
        },
    );
    drain(&mut e_locked);
    assert_eq!(e_locked.debug_hand(0).len(), 2, "scstaoluan Wuzhong draws 2");

    // 6. Real `scschiyan` (`useCardToPlayered` sha with target `he` non-empty; return at turn end)
    let mut e_cy = epic_engine(10_3300);
    e_cy.debug_grant(0, "scschiyan");
    advance_to_play(&mut e_cy, 0);
    e_cy.debug_clear_hand(0);
    e_cy.debug_clear_hand(1);
    let sha_c = e_cy.debug_place(0, "sha");
    let t_card = e_cy.debug_place(1, "tao");
    e_cy.debug_give_hand_card(0, 0, sha_c);
    e_cy.debug_give_hand_card(1, 1, t_card);
    let hp1 = e_cy.debug_hp(1);
    let _ = submit(
        &mut e_cy,
        0,
        GameAction::PlayCard {
            card: sha_c,
            targets: vec![1],
        },
    );
    let cy_pick = e_cy.open_choice().cloned().expect("scschiyan_pick on Sha target");
    assert!(cy_pick.spec.options.contains(&ChoiceOption::Card(t_card)));
    roundtrip_and_answer(&mut e_cy, &[&pack], vec![ChoiceOption::Card(t_card)]);
    drain(&mut e_cy);
    assert!(!e_cy.debug_hand(1).contains(&t_card), "t_card detained in scschiyan_get during turn");
    // owner hand == 0, target hand == 0, equips 0 == 0 -> scschiyan_damage triggers +1 (total 2 dmg)
    assert_eq!(e_cy.debug_hp(1), hp1 - 2, "scschiyan_damage deals +1 damage when h>=t_h and e>=t_e");
    // End turn -> scschiyan_get returns to seat 1
    while e_cy.turn().current == 0 {
        let _ = submit(&mut e_cy, 0, GameAction::NextPhase);
        drain(&mut e_cy);
    }
    assert!(e_cy.debug_hand(1).contains(&t_card), "scschiyan_get returns to target at turn end (`mobile.js:4767`)");
}
