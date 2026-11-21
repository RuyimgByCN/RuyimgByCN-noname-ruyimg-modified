//! L27 选将美化（`选将美化/extension.js`）6 项剩余技能 + 6 项含规则 UI 钩子修复与 6 维实盘闭环测试。
//!
//! 对照上游：`archive/upstream/noname-liuli/noname/extension/选将美化/extension.js`
//! - `skills` 6 项：`xiefang` (:1162)、`new_meibu` (:1674)、`new_zhixi` (:1738)、
//!   `clandianzhan` (:1867)、`reqimou` (:6059)、`bolan` (:7747)
//! - `ui_hooks` 6 项：`mbcmqingzheng` (:3697/:4259)、`sbqingzheng` (:4263/:4825)、
//!   `xingchong` (:6633)、`dcxiaoxi` (:6774)、`twfengji` (:6889)、`twzhouzu` (:7441)
//!
//! 每项均走真实游戏事件链路（无 `debug_execute_effect`），覆盖正例、负例、分支 A/B、
//! 取消路径、AI 公开信息与挂起态存档恢复（`save -> encode_archive -> decode_archive -> restore_with_extensions -> Answer`）。

use std::collections::BTreeSet;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::{
    attach_formal_package_upstream_rules, bridge_pack, decision34_pack9_extension_pack,
};
use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardDef, CardKind, Suit};
use engine_native::content::extension::ExtensionPack;
use engine_native::content::host::{HostBehavior, HostSkillDecl};
use engine_native::event::Phase;
use engine_native::machine::LogEvent;
use engine_native::view::{EventView, HandView};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, OpenChoice};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn pack() -> ExtensionPack {
    let mut p = decision34_pack9_extension_pack();
    for (name, behavior) in [
        ("xiefang", HostBehavior::Xiefang),
        ("sbzhaxiang", HostBehavior::SbZhaXiang),
        ("new_meibu", HostBehavior::NewMeibu),
        ("new_zhixi", HostBehavior::NewZhixi),
        ("new_zhixi_mark2", HostBehavior::NewZhixi),
        ("new_meibu_range", HostBehavior::NewMeibuRange),
        ("jishe", HostBehavior::Jishe),
        ("mbcmqingzheng", HostBehavior::MbcmQingzheng),
        ("bolan", HostBehavior::L27BeautifyBatchB),
        ("bolan_g", HostBehavior::L27BeautifyBatchB),
    ] {
        if !p.host_skills.iter().any(|h| h.name == name) {
            p.host_skills.push(HostSkillDecl {
                name,
                behavior,
                presentation_binding: None,
            });
        }
    }
    p
}

fn select_beautify_formal_pack() -> ExtensionPack {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/select-beautify-upstream-liuli/assets/cards.json");
    let json = std::fs::read_to_string(&path).expect("read L27 cards.json");
    let mut p = bridge_pack(&json).expect("bridge L27").pack;
    attach_formal_package_upstream_rules("upstream-liuli", "select-beautify", &mut p, &[]);
    p
}

fn longxue_formal_pack() -> ExtensionPack {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/longxue-upstream-liuli/assets/cards.json");
    let json = std::fs::read_to_string(&path).expect("read L29 cards.json");
    let mut p = bridge_pack(&json).expect("bridge L29").pack;
    attach_formal_package_upstream_rules("upstream-liuli", "longxue", &mut p, &[]);
    p
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

fn fresh(p: &ExtensionPack, seed: u64, skills_by_seat: &[&[&str]]) -> Engine {
    let mut e = Engine::new_with_extensions(seed, skills_by_seat.len(), &[p]).expect("build");
    for (seat, sks) in skills_by_seat.iter().enumerate() {
        for &sk in *sks {
            e.debug_grant(seat as u8, sk);
        }
    }
    e
}

fn clear_hand(e: &mut Engine, seat: u8) {
    let cards = e.debug_hand(seat);
    for c in cards {
        e.state_core_mut().player_mut(seat).hand.retain(|&x| x != c);
        e.state_core_mut().discard.push(c);
    }
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

fn has_skill(e: &Engine, p: u8, skill: &str) -> bool {
    e.catalog()
        .skill_by_name(skill)
        .is_some_and(|def| e.state_core().has_skill(p, def))
}

fn hand(e: &Engine, p: u8) -> usize {
    e.debug_hand(p).len()
}

fn hp(e: &Engine, p: u8) -> i32 {
    e.state_core().player(p).hp
}

fn mark_of(e: &Engine, seat: u8, skill: &str) -> i32 {
    let Some(def) = e.catalog().skill_by_name(skill) else {
        return 0;
    };
    e.state_core()
        .vm_aux
        .marks
        .get(&(seat, u32::from(def)))
        .copied()
        .unwrap_or(0)
}

fn card_by(e: &Engine, pred: impl Fn(u32, &CardDef) -> bool) -> u32 {
    let core = e.state_core();
    let mut pools = Vec::new();
    pools.extend_from_slice(&core.deck);
    pools.extend_from_slice(&core.discard);
    for pl in &core.players {
        pools.extend_from_slice(&pl.hand);
    }
    pools
        .into_iter()
        .find(|&id| pred(id, &e.catalog().cards[id as usize]))
        .expect("matching card")
}

fn give(e: &mut Engine, p: u8, card: u32) {
    e.state_core_mut().deck.retain(|&x| x != card);
    e.state_core_mut().discard.retain(|&x| x != card);
    for seat in 0..e.state_core().players.len() as u8 {
        e.state_core_mut().player_mut(seat).hand.retain(|&x| x != card);
        e.state_core_mut().player_mut(seat).equips.retain(|&x| x != card);
    }
    e.state_core_mut().player_mut(p).hand.push(card);
}

fn equip(e: &mut Engine, p: u8, card: u32) {
    e.state_core_mut().deck.retain(|&x| x != card);
    e.state_core_mut().discard.retain(|&x| x != card);
    for seat in 0..e.state_core().players.len() as u8 {
        e.state_core_mut().player_mut(seat).hand.retain(|&x| x != card);
        e.state_core_mut().player_mut(seat).equips.retain(|&x| x != card);
    }
    e.state_core_mut().player_mut(p).equips.push(card);
}

fn confirm_triggers(e: &mut Engine) {
    while let Some(c) = e.open_choice().cloned() {
        if c.spec.prompt == "trigger_confirm" {
            let _ = submit(
                e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Confirm],
                },
            );
            continue;
        }
        break;
    }
}

fn activate(e: &mut Engine, p: u8, skill: &str) -> CommandOutcome {
    let def = e.catalog().skill_by_name(skill).expect("skill in catalog");
    submit(e, p, GameAction::ActivateSkill { skill: def })
}

fn open(e: &Engine, prompt: &str) -> OpenChoice {
    let c = e.open_choice().cloned().expect("open choice");
    assert_eq!(c.spec.prompt, prompt, "unexpected prompt");
    c
}

fn answer(e: &mut Engine, prompt: &str, sel: Vec<ChoiceOption>) -> CommandOutcome {
    while let Some(c) = e.open_choice().cloned() {
        if c.spec.prompt == "trigger_confirm" && prompt != "trigger_confirm" {
            let _ = submit(
                e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Confirm],
                },
            );
            continue;
        }
        break;
    }
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

fn tag(t: u8) -> ChoiceOption {
    ChoiceOption::Tag(t)
}

fn pl(p: u8) -> ChoiceOption {
    ChoiceOption::Player(p)
}

fn cd(c: u32) -> ChoiceOption {
    ChoiceOption::Card(c)
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

fn roundtrip_bytes(e: &Engine, packs: &[&ExtensionPack]) -> Engine {
    let bytes = encode_archive(&e.save());
    let snap = decode_archive(&bytes).expect("decode archive");
    let restored = Engine::restore_with_extensions(&snap, packs).expect("restore");
    assert_eq!(restored.state_hash(), e.state_hash());
    restored
}

fn snapshot_roundtrip(e: &Engine, p: &ExtensionPack, sel: Vec<ChoiceOption>) {
    let snap = e.save();
    let bytes = encode_archive(&snap);
    let dec = decode_archive(&bytes).expect("decode archive");
    let mut restored = Engine::restore_with_extensions(&dec, &[p]).expect("restore pending");
    assert_eq!(restored.state_hash(), e.state_hash());
    if let Some(c) = restored.open_choice().cloned() {
        let out = submit(
            &mut restored,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        );
        assert!(!is_rejected(out.clone()), "restored answer rejected: {out:?}");
    }
}

fn hand_turn_to(e: &mut Engine, seat: u8, round: u32) -> CommandOutcome {
    let n = e.state_core().players.len() as u8;
    let prev = (seat + n - 1) % n;
    e.debug_set_turn(prev, Phase::End, round);
    submit(e, prev, GameAction::NextPhase)
}

fn play(e: &mut Engine, p: u8, card: u32, targets: Vec<u8>) -> CommandOutcome {
    let out = submit(e, p, GameAction::PlayCard { card, targets });
    assert!(!is_rejected(out.clone()), "play rejected: {out:?}");
    out
}

// =============================================================================
// 1. xiefang（选将美化/extension.js:1162-1212）：gameDrawEnd / dieEnd / drawEnd / onremove
// =============================================================================

#[test]
fn l27_fix_xiefang_game_draw_end_die_end_and_onremove() {
    let p9 = pack();
    let mut g = Engine::new_with_extensions(27902, 4, &[&p9]).expect("build pack9");
    let xf_def = g.catalog().skill_by_name("xiefang").expect("xiefang");
    assert_eq!(mark_of(&g, 0, "xiefang"), 0, "未持有 xiefang 时不应带标记");

    g.debug_grant(0, "xiefang");
    g.debug_set_general(1, "貂蝉"); // female
    g.debug_set_general(2, "黄月英"); // female
    g.debug_set_general(3, "曹操"); // male

    // 首轮 Prepare -> Judge（早于 Draw end）即刷新 xiefang 标记为 2
    g.debug_set_turn(0, Phase::Prepare, 1);
    let _ = submit(&mut g, 0, GameAction::NextPhase); // Prepare -> Judge
    drain(&mut g);
    assert_eq!(
        mark_of(&g, 0, "xiefang"),
        2,
        "gameDrawEnd/首回合入场即刷新 xiefang 标记为 2（无需等到自己摸牌阶段结束）"
    );
    assert_eq!(
        g.state_core().computed_distance(g.catalog(), 0, 2),
        1,
        "2 名存活女性使距离 -2 后钳位为 1"
    );

    let mut g = roundtrip_bytes(&g, &[&p9]);
    let plan = g.plan_action(0, &BoundedPlanner::new(4));
    assert!(!matches!(plan.action, GameAction::ActivateSkill { skill, .. } if skill == xf_def));

    // 击杀一名女性角色（2 号黄月英）：真实 PlayCard【杀】令其死亡，dieEnd 后标记必须减为 1
    g.debug_set_turn(0, Phase::Play, 1);
    clear_hand(&mut g, 2);
    g.debug_set_hp(2, 1);
    let sha = card_by(&g, |_, c| c.kind == CardKind::Sha);
    give(&mut g, 0, sha);
    play(&mut g, 0, sha, vec![2]);
    drain(&mut g);
    assert!(!g.state_core().player(2).alive);
    assert_eq!(
        mark_of(&g, 0, "xiefang"),
        1,
        "女性角色阵亡 dieEnd 后 xiefang 标记应立即减为 1（不可仍把死者计入 2）"
    );

    // 击杀另一名女性后标记清零
    clear_hand(&mut g, 1);
    g.debug_set_hp(1, 1);
    g.state_core_mut().turn.sha_used_play_phase = false;
    let sha2 = card_by(&g, |id, c| c.kind == CardKind::Sha && id != sha);
    give(&mut g, 0, sha2);
    play(&mut g, 0, sha2, vec![1]);
    drain(&mut g);
    assert!(!g.state_core().player(1).alive);
    assert_eq!(mark_of(&g, 0, "xiefang"), 0, "无存活女性时移除 xiefang 标记");
}

// =============================================================================
// 2 & 3. new_meibu + new_zhixi（选将美化/extension.js:1674-1814）：
//         initial count = 0（未出牌不封锁）+ phaseUseEnd 清理（而非 phaseEnd）
// =============================================================================

#[test]
fn l27_fix_new_meibu_and_new_zhixi_initial_count_and_phase_use_end_cleanup() {
    let p = pack();
    let mut e = fresh(&p, 27903, &[&["new_meibu"][..], &[][..]]);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);
    let red_basic = card_by(&e, |_, c| {
        c.kind == CardKind::Sha && matches!(c.suit, Suit::Heart | Suit::Diamond)
    });
    let sha1 = card_by(&e, |id, c| c.kind == CardKind::Sha && id != red_basic);
    let shan1 = card_by(&e, |_, c| c.kind == CardKind::Shan);
    let trick1 = card_by(&e, |_, c| c.kind == CardKind::Wuzhong);
    give(&mut e, 0, red_basic);
    give(&mut e, 1, sha1);
    give(&mut e, 1, shan1);
    give(&mut e, 1, trick1);
    e.debug_set_hp(1, 2);

    // Seat 1 进入 Play 开始 -> Seat 0 触发 new_meibu，取消分支不赋止息
    e.debug_set_turn(1, Phase::Draw, 1);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    confirm_triggers(&mut e);
    snapshot_roundtrip(&e, &p, Vec::new());
    answer(&mut e, "new_meibu_discard", Vec::new());
    assert!(!has_skill(&e, 1, "new_zhixi"), "取消每步后目标不应获得止息");
    assert!(e.debug_hand(0).contains(&red_basic), "取消每步后手牌不变");

    // 再次进入 Seat 1 Play 开始 -> Seat 0 弃置非杀非锦囊非黑牌以同时赋 new_meibu_range
    clear_hand(&mut e, 0);
    let red_horse = card_by(&e, |_, c| {
        c.kind.equip_slot().is_some() && matches!(c.suit, Suit::Heart | Suit::Diamond)
    });
    give(&mut e, 0, red_horse);
    e.debug_set_turn(1, Phase::Draw, 2);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    answer(&mut e, "new_meibu_discard", vec![cd(red_horse)]);
    drain(&mut e);

    // 正例 1：获得 new_zhixi 后未出任何牌前计数必须为 0（绝对不能初始化为 hp！）
    assert!(has_skill(&e, 1, "new_zhixi"));
    assert!(has_skill(&e, 1, "new_meibu_range"));
    assert_eq!(mark_of(&e, 1, "new_zhixi"), 0, "新得止息初始使用计数必须为 0");
    assert!(
        !e.catalog().blocks_all_cards(e.state_core(), 1),
        "hp=2 且尚未用牌时不得立即被止息封锁"
    );

    // Choice A：Seat 1 正常打出一张【杀】（count=1 < hp=2，仍未封锁）
    play(&mut e, 1, sha1, vec![0]);
    drain(&mut e);
    assert_eq!(mark_of(&e, 1, "new_zhixi"), 1);
    assert!(!e.catalog().blocks_all_cards(e.state_core(), 1));

    // Choice B：Seat 1 再用锦囊牌【无中生有】-> new_zhixi_mark2=1，随后在 Play 阶段内封锁
    play(&mut e, 1, trick1, vec![]);
    drain(&mut e);
    assert_eq!(mark_of(&e, 1, "new_zhixi"), 2);
    assert_eq!(mark_of(&e, 1, "new_zhixi_mark2"), 1);
    assert!(e.catalog().blocks_all_cards(e.state_core(), 1));
    let extra_tao = card_by(&e, |_, c| c.kind == CardKind::Tao);
    give(&mut e, 1, extra_tao);
    assert!(is_rejected(submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: extra_tao,
            targets: vec![]
        }
    )));

    // AI 公开视角看到被止息限制 -> 选择结束出牌阶段
    let ai_act = e.plan_action(1, &BoundedPlanner::new(4));
    assert!(matches!(ai_act.action, GameAction::NextPhase));

    // 正例 2：`phaseUseEnd`（Play -> Discard）结束时即刻清理临时 new_zhixi / new_meibu_range 与计数标记
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    drain(&mut e);
    assert_eq!(e.turn().phase, Phase::Discard);
    assert!(
        !has_skill(&e, 1, "new_zhixi"),
        "phaseUseEnd 后临时止息必须已移除（不等到 phaseEnd）"
    );
    assert!(
        !has_skill(&e, 1, "new_meibu_range"),
        "phaseUseEnd 后每步距离技必须已移除"
    );
    assert_eq!(mark_of(&e, 1, "new_zhixi"), 0);
    assert_eq!(mark_of(&e, 1, "new_zhixi_mark2"), 0);
    assert!(!e.catalog().blocks_all_cards(e.state_core(), 1));
}

// =============================================================================
// 4. clandianzhan（选将美化/extension.js:1867-1927）：useCardAfter、重铸同花色手牌、下一轮重置
// =============================================================================

#[test]
fn l27_fix_clandianzhan_recast_and_next_round_reset() {
    let p = pack();
    let mut e = fresh(&p, 27904, &[&["clandianzhan"][..], &[][..]]);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);

    let sp1 = card_by(&e, |_, c| c.suit == Suit::Spade && c.kind == CardKind::Sha);
    let sp2 = card_by(&e, |id, c| c.suit == Suit::Spade && id != sp1);
    let sp3 = card_by(&e, |id, c| c.suit == Suit::Spade && id != sp1 && id != sp2);
    let h1 = card_by(&e, |_, c| c.suit == Suit::Heart && c.kind == CardKind::Sha);
    let c1 = card_by(&e, |_, c| c.suit == Suit::Club && c.kind == CardKind::Dismantle);
    let d1 = card_by(&e, |_, c| c.suit == Suit::Diamond && c.kind == CardKind::Sha);
    give(&mut e, 0, sp1);
    give(&mut e, 0, sp2);
    give(&mut e, 0, sp3);
    give(&mut e, 0, h1);
    give(&mut e, 0, c1);
    give(&mut e, 0, d1);

    e.debug_set_turn(0, Phase::Play, 1);
    let deck_before = e.state_core().deck.len();
    play(&mut e, 0, sp1, vec![1]);
    drain(&mut e);
    assert!(e.state_core().player(1).linked, "唯一目标 Seat 1 应被横置");
    assert!(
        !e.debug_hand(0).contains(&sp2) && !e.debug_hand(0).contains(&sp3),
        "点盏应重铸手中全部同花色牌（sp2/sp3 离手入弃牌堆）"
    );
    assert!(
        e.state_core().discard.contains(&sp2) && e.state_core().discard.contains(&sp3),
        "被重铸牌应进入弃牌堆"
    );
    assert_eq!(
        deck_before - e.state_core().deck.len(),
        3,
        "重铸 2 张摸 2 张 + 点盏摸 1 张 = 共摸 3 张"
    );

    // 负例：群体牌（targets.len() != 1）不触发点盏
    e.state_core_mut().turn.sha_used_play_phase = false;
    clear_hand(&mut e, 0);
    let aoe = card_by(&e, |_, c| c.kind == CardKind::SavageAttack);
    give(&mut e, 0, aoe);
    let h_before_aoe = hand(&e, 0);
    play(&mut e, 0, aoe, vec![]);
    drain(&mut e);
    assert_eq!(hand(&e, 0), h_before_aoe - 1, "targets.len()!=1 不触发点盏摸牌");

    // 同轮集齐其余三花色后（每次目标未横置且维持存活），同轮第 5 张牌绝对不能触发
    clear_hand(&mut e, 0);
    give(&mut e, 0, h1);
    give(&mut e, 0, c1);
    give(&mut e, 0, d1);
    e.debug_set_hp(1, 4);
    e.state_core_mut().player_mut(1).linked = false;
    play(&mut e, 0, h1, vec![1]);
    drain(&mut e);
    e.debug_set_hp(1, 4);
    e.state_core_mut().player_mut(1).linked = false;
    play(&mut e, 0, c1, vec![1]);
    drain(&mut e);
    e.debug_set_hp(1, 4);
    e.state_core_mut().player_mut(1).linked = false;
    e.state_core_mut().turn.sha_used_play_phase = false;
    play(&mut e, 0, d1, vec![1]);
    drain(&mut e);
    assert_eq!(flag(&e, 0, "clandianzhan_suit_mask") & 0xF, 0xF);

    let sp4 = card_by(&e, |id, c| {
        c.suit == Suit::Spade && c.kind == CardKind::Dismantle && id != sp2 && id != sp3
    });
    give(&mut e, 0, sp4);
    let h_before_5th = hand(&e, 0);
    play(&mut e, 0, sp4, vec![1]);
    drain(&mut e);
    assert_eq!(
        hand(&e, 0),
        h_before_5th - 1,
        "同轮集齐四花色后不得当轮重置再触发点盏"
    );

    // 推进到下一轮（round 2）：花色记录清零，再次使用黑桃牌重新触发点盏
    let _ = hand_turn_to(&mut e, 0, 2);
    drain(&mut e);
    e.debug_set_turn(0, Phase::Play, 2);
    let sp5 = card_by(&e, |id, c| {
        c.suit == Suit::Spade && c.kind == CardKind::Sha && id != sp1
    });
    give(&mut e, 0, sp5);
    let h_before_r2 = hand(&e, 0);
    let _ = roundtrip_bytes(&e, &[&p]);
    play(&mut e, 0, sp5, vec![1]);
    drain(&mut e);
    assert_eq!(hand(&e, 0), h_before_r2, "下一轮重置后首张黑桃重新触发点盏");
}

// =============================================================================
// 5. reqimou（选将美化/extension.js:6059-6214）：选数后可取消 chooseBool 分支 + 失血在摸牌前
// =============================================================================

#[test]
fn l27_fix_reqimou_cancel_after_number_and_lose_hp_before_draw() {
    let p = pack();
    let mut e = fresh(&p, 27905, &[&["reqimou"][..], &[][..]]);
    clear_hand(&mut e, 0);
    e.debug_set_turn(0, Phase::Play, 1);

    // 取消分支 A：数字询问直接取消 -> 不失血、不摸牌、不消耗限定技
    activate(&mut e, 0, "reqimou");
    answer(&mut e, "reqimou:choose_hp", Vec::new());
    drain(&mut e);
    assert_eq!(hp(&e, 0), 4);
    assert_eq!(hand(&e, 0), 0);
    assert_eq!(flag(&e, 0, "reqimou_used"), 0);

    // 取消分支 B：选定数字 X=2 后，在 `reqimou:confirm` (chooseBool) 点取消 -> 同样不失血不消耗
    activate(&mut e, 0, "reqimou");
    snapshot_roundtrip(&e, &p, vec![tag(2)]);
    answer(&mut e, "reqimou:choose_hp", vec![tag(2)]);
    snapshot_roundtrip(&e, &p, Vec::new());
    answer(&mut e, "reqimou:confirm", Vec::new());
    drain(&mut e);
    assert_eq!(hp(&e, 0), 4, "选数字后在 chooseBool 拒绝不得失血");
    assert_eq!(hand(&e, 0), 0, "选数字后在 chooseBool 拒绝不得摸牌");
    assert_eq!(flag(&e, 0, "reqimou_used"), 0, "拒绝后限定技仍可再次发动");

    // 正例 + 濒死优先顺序 + AI confirm
    let tao = card_by(&e, |_, c| c.kind == CardKind::Tao);
    give(&mut e, 1, tao);
    activate(&mut e, 0, "reqimou");
    answer(&mut e, "reqimou:choose_hp", vec![tag(4)]);
    assert!(open(&e, "reqimou:confirm").spec.allow_cancel);
    let ai_confirm = e.plan_action(0, &BoundedPlanner::new(4));
    assert!(matches!(ai_confirm.action, GameAction::Answer { .. }));
    answer(&mut e, "reqimou:confirm", vec![ChoiceOption::Confirm]);
    assert_eq!(hand(&e, 0), 0, "loseHp 结算（含濒死）必须先于 draw(X)");
    while let Some(c) = e.open_choice().cloned() {
        if c.observer == 1 && c.spec.options.contains(&cd(tao)) {
            answer(&mut e, c.spec.prompt, vec![cd(tao)]);
        } else {
            answer(&mut e, c.spec.prompt, Vec::new());
        }
    }
    assert_eq!(hp(&e, 0), 1);
    assert_eq!(hand(&e, 0), 4, "救活后摸 X=4 张牌");
    assert_eq!(flag(&e, 0, "reqimou_used"), 1);
    assert!(is_rejected(activate(&mut e, 0, "reqimou")));
}

// =============================================================================
// 6. bolan / bolan_g（选将美化/extension.js:7747-8016）：动态技能池 + RNG 抽取 + phaseUseEnd 移除
// =============================================================================

#[test]
fn l27_fix_bolan_dynamic_pool_rng_and_phase_use_end_expiry() {
    let p = pack();
    let mut e_neg = fresh(&p, 27906, &[&["bolan"][..], &[][..]]);
    e_neg.debug_set_turn(0, Phase::Play, 1);
    assert!(
        is_rejected(activate(&mut e_neg, 0, "bolan")),
        "bolan 不是出牌阶段主动技，应拒绝直接 ActivateSkill"
    );

    let mut seen_defs: BTreeSet<u32> = BTreeSet::new();
    for seed in [27910u64, 27911, 27912, 27913, 27914] {
        let mut e = fresh(&p, seed, &[&["bolan"][..], &[][..]]);
        e.debug_set_turn(0, Phase::Draw, 1);
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        assert!(open(&e, "bolan:choose_skill").spec.allow_cancel);
        assert!(flag(&e, 0, "bolan_pool_size") > 3, "动态技能池必须大于固定 3 技");
        for i in 0..3 {
            let def_plus1 = flag(&e, 0, &format!("bolan_opt_{i}"));
            assert!(def_plus1 > 0);
            seen_defs.insert(def_plus1 - 1);
        }
        if seed == 27910 {
            snapshot_roundtrip(&e, &p, vec![tag(0)]);
        }
        answer(&mut e, "bolan:choose_skill", vec![tag(0)]);
        drain(&mut e);
        let granted_def = (flag(&e, 0, "bolan_granted_def") - 1) as u16;
        let granted_name = e.catalog().skill_name(granted_def);
        assert!(has_skill(&e, 0, granted_name), "出牌阶段内获授真实技能 {granted_name}");

        let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard
        drain(&mut e);
        assert_eq!(e.turn().phase, Phase::Discard);
        assert!(
            !has_skill(&e, 0, granted_name),
            "phaseUseEnd 后博览所获临时技能 {granted_name} 必须立即移除"
        );
    }
    assert!(
        seen_defs.len() > 3,
        "不同 RNG 种子合计应采样超过固定 3 个技能，实际={:?}",
        seen_defs
    );
}

// =============================================================================
// 7 & 8. mbcmqingzheng（L27 4259 vs L29 隔离） & sbqingzheng（4263-4838）
// =============================================================================

#[test]
fn l27_fix_mbcmqingzheng_isolated_from_l29_and_matches_4259() {
    // 1. L29 龙血玄黄 `HostBehavior::MbcmQingzheng` 仍保持原有 1 点伤害
    let l29_p = longxue_formal_pack();
    let mut e29 = fresh(&l29_p, 27920, &[&[][..], &["mbcmqingzheng"][..]]);
    clear_hand(&mut e29, 0);
    clear_hand(&mut e29, 1);
    let s1 = card_by(&e29, |_, c| c.suit == Suit::Spade);
    let s2 = card_by(&e29, |id, c| c.suit == Suit::Spade && id != s1);
    let h1 = card_by(&e29, |_, c| c.suit == Suit::Heart);
    give(&mut e29, 1, s1);
    give(&mut e29, 1, s2);
    give(&mut e29, 0, h1);
    let hp0_29 = hp(&e29, 0);
    e29.debug_set_turn(1, Phase::Draw, 1);
    let _ = submit(&mut e29, 1, GameAction::NextPhase);
    confirm_triggers(&mut e29);
    let st = open(&e29, "mbcmqingzheng_suit").spec.options[0];
    answer(&mut e29, "mbcmqingzheng_suit", vec![st]);
    answer(&mut e29, "mbcmqingzheng_target", vec![pl(0)]);
    let tst = open(&e29, "mbcmqingzheng_target_suit").spec.options[0];
    answer(&mut e29, "mbcmqingzheng_target_suit", vec![tst]);
    drain(&mut e29);
    assert_eq!(hp(&e29, 0), hp0_29 - 1, "L29 龙血玄黄清正仍造成 1 点伤害");

    // 2. L27 选将美化正式包绑定独立 L27MbcmQingzheng（extension.js:3705 direct:true + step 0 可取消 + CardsDiscarded）
    let l27_p = select_beautify_formal_pack();
    let mut e27 = fresh(&l27_p, 27921, &[&[][..], &["mbcmqingzheng"][..]]);
    clear_hand(&mut e27, 0);
    clear_hand(&mut e27, 1);
    let s1 = card_by(&e27, |_, c| c.suit == Suit::Spade);
    let s2 = card_by(&e27, |id, c| c.suit == Suit::Spade && id != s1);
    let h1 = card_by(&e27, |_, c| c.suit == Suit::Heart);
    give(&mut e27, 1, s1);
    give(&mut e27, 1, s2);
    give(&mut e27, 0, h1);
    let hp0_27 = hp(&e27, 0);
    e27.debug_set_turn(1, Phase::Draw, 1);
    let _ = submit(&mut e27, 1, GameAction::NextPhase);
    let suit_ask_cancel = open(&e27, "mbcmqingzheng_suit");
    assert!(
        suit_ask_cancel.spec.allow_cancel,
        "L27 mbcmqingzheng step 0 必须支持直接取消（无需先经过 trigger_confirm）"
    );
    answer(&mut e27, "mbcmqingzheng_suit", Vec::new());
    drain(&mut e27);
    assert_eq!(hand(&e27, 1), 2, "step 0 取消后手牌不得被弃");

    e27.debug_set_turn(1, Phase::Draw, 2);
    let _ = submit(&mut e27, 1, GameAction::NextPhase);
    let st = open(&e27, "mbcmqingzheng_suit").spec.options[0];
    answer(&mut e27, "mbcmqingzheng_suit", vec![st]);
    answer(&mut e27, "mbcmqingzheng_target", Vec::new());
    drain(&mut e27);
    assert_eq!(hand(&e27, 1), 2, "取消选目标时自身手牌不得被弃");

    e27.debug_set_turn(1, Phase::Draw, 3);
    let _ = submit(&mut e27, 1, GameAction::NextPhase);
    let st = open(&e27, "mbcmqingzheng_suit").spec.options[0];
    answer(&mut e27, "mbcmqingzheng_suit", vec![st]);
    answer(&mut e27, "mbcmqingzheng_target", vec![pl(0)]);
    let ts_choice = open(&e27, "mbcmqingzheng_target_suit");
    assert!(!ts_choice.spec.allow_cancel, "目标花色选择不可取消");
    let tst = ts_choice.spec.options[0];
    snapshot_roundtrip(&e27, &l27_p, vec![tst]);
    answer(&mut e27, "mbcmqingzheng_target_suit", vec![tst]);
    drain(&mut e27);
    assert_eq!(hand(&e27, 1), 0);
    assert_eq!(hand(&e27, 0), 0);
    assert_eq!(
        hp(&e27, 0),
        hp0_27,
        "L27 选将美化 :4259 (discardMeCards > discardHimCards && discardHimCards > 0) 在 DOM 数组下恒 false"
    );

    // 3. 验证 L27MbcmQingzheng 发出 EventData::CardsDiscarded（记录于 vm_aux.lost；pack() 已启用 vm_card_history）
    let mut p_disc = pack();
    for h in &mut p_disc.host_skills {
        if h.name == "mbcmqingzheng" {
            h.behavior = HostBehavior::L27MbcmQingzheng;
        }
    }
    let mut e_disc = fresh(&p_disc, 27929, &[&[][..], &["mbcmqingzheng"][..]]);
    clear_hand(&mut e_disc, 0);
    clear_hand(&mut e_disc, 1);
    let ds1 = card_by(&e_disc, |_, c| c.suit == Suit::Spade);
    let ds2 = card_by(&e_disc, |id, c| c.suit == Suit::Spade && id != ds1);
    let dh1 = card_by(&e_disc, |_, c| c.suit == Suit::Heart);
    give(&mut e_disc, 1, ds1);
    give(&mut e_disc, 1, ds2);
    give(&mut e_disc, 0, dh1);
    e_disc.debug_set_turn(1, Phase::Draw, 1);
    let _ = submit(&mut e_disc, 1, GameAction::NextPhase);
    let dst = open(&e_disc, "mbcmqingzheng_suit").spec.options[0];
    answer(&mut e_disc, "mbcmqingzheng_suit", vec![dst]);
    answer(&mut e_disc, "mbcmqingzheng_target", vec![pl(0)]);
    let dtst = open(&e_disc, "mbcmqingzheng_target_suit").spec.options[0];
    answer(&mut e_disc, "mbcmqingzheng_target_suit", vec![dtst]);
    drain(&mut e_disc);
    let lost1 = e_disc
        .state_core()
        .vm_aux
        .lost
        .get(&1)
        .cloned()
        .unwrap_or_default();
    let lost0 = e_disc
        .state_core()
        .vm_aux
        .lost
        .get(&0)
        .cloned()
        .unwrap_or_default();
    assert!(lost1.contains(&ds1) && lost1.contains(&ds2));
    assert!(lost0.contains(&dh1));
}

#[test]
fn l27_fix_sbqingzheng_discard_order_and_unconditional_gain_mark() {
    let p = pack();
    let mut e = fresh(&p, 27922, &[&[][..], &["sbqingzheng", "sbjianxiong"][..]]);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);
    set_flag(&mut e, 1, "sbjianxiong", 1);
    set_flag(&mut e, 1, "sbjianxiong_marks", 1);

    let sp1 = card_by(&e, |_, c| c.suit == Suit::Spade);
    let c1 = card_by(&e, |_, c| c.suit == Suit::Club);
    let h1 = card_by(&e, |_, c| c.suit == Suit::Heart);
    let h2 = card_by(&e, |id, c| c.suit == Suit::Heart && id != h1);
    give(&mut e, 1, sp1);
    give(&mut e, 1, c1);
    give(&mut e, 0, h1);
    give(&mut e, 0, h2);
    let hp0 = hp(&e, 0);

    e.debug_set_turn(1, Phase::Draw, 1);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    answer(&mut e, "sbqingzheng:choose_suits", vec![tag(0), tag(2)]);
    assert_eq!(hand(&e, 1), 2, "选择己方花色阶段不得提前弃牌");
    answer(&mut e, "sbqingzheng:choose_target", Vec::new());
    drain(&mut e);
    assert_eq!(hand(&e, 1), 2, "取消选目标后己方手牌完整保留");

    e.debug_set_turn(1, Phase::Draw, 2);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    answer(&mut e, "sbqingzheng:choose_suits", vec![tag(0), tag(2)]);
    answer(&mut e, "sbqingzheng:choose_target", vec![pl(0)]);
    let tgt_suit_ask = open(&e, "sbqingzheng:choose_target_suit");
    assert!(!tgt_suit_ask.spec.allow_cancel);
    answer(&mut e, "sbqingzheng:choose_target_suit", vec![tag(1)]);
    assert_eq!(hand(&e, 1), 0);
    assert_eq!(hand(&e, 0), 0);
    assert_eq!(hp(&e, 0), hp0, "L27 :4825 数组比较不造成伤害");
    snapshot_roundtrip(&e, &p, vec![tag(1)]);
    answer(&mut e, "sbqingzheng:gain_mark", vec![tag(1)]);
    drain(&mut e);
    assert_eq!(flag(&e, 1, "sbjianxiong_marks"), 2, "未造成伤害时仍可获 1 枚治世");
}

// =============================================================================
// 9. xingchong（选将美化/extension.js:6633-6771）：公开 showCards + 可见标记 + 摸后选展示 + roundStart 清理
// =============================================================================

#[test]
fn l27_fix_xingchong_visible_mark_public_reveal_draw_then_show_and_round_clear() {
    let p = pack();
    let mut e = fresh(&p, 27923, &[&["xingchong"][..], &[][..]]);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);

    let _ = hand_turn_to(&mut e, 0, 2);
    snapshot_roundtrip(&e, &p, vec![tag(2)]);
    answer(&mut e, "xingchong:choose_draw", vec![tag(2)]);
    let show_ask = open(&e, "xingchong:show_cards");
    assert_eq!(
        show_ask.spec.options.len(),
        2,
        "必须先摸牌再打开 show_cards，否则空手牌无法选刚摸到的牌"
    );
    let shown_opt = show_ask.spec.options[0];
    let ChoiceOption::Card(shown_cid) = shown_opt else {
        panic!("expected Card option");
    };
    answer(&mut e, "xingchong:show_cards", vec![shown_opt]);
    drain(&mut e);

    assert_eq!(mark_of(&e, 0, "xingchong"), 1, "xingchong 可见标记=1");
    let revealed_to_other = e.event_view_for(1).into_iter().any(|ev| {
        matches!(ev, EventView::Public(LogEvent::CardsRevealed { cards, .. }) if cards.contains(&shown_cid))
    });
    assert!(revealed_to_other, "他家公开日志视图必须可见幸宠展示牌");
    let view_other = e.view_for(1);
    assert!(matches!(view_other.players[0].hand, HandView::Hidden));
    assert!(
        view_other.visible_card_ids().contains(&shown_cid),
        "幸宠公开展示的特定手牌在他人 PlayerView.cards 中必须可见"
    );

    let h_before_loss = hand(&e, 0);
    let filler1 = card_by(&e, |id, _| id != shown_cid);
    let filler2 = card_by(&e, |id, _| id != shown_cid && id != filler1);
    let filler3 = card_by(&e, |id, _| id != shown_cid && id != filler1 && id != filler2);
    give(&mut e, 0, filler1);
    give(&mut e, 0, filler2);
    give(&mut e, 0, filler3);
    e.debug_set_hp(0, 4);
    e.debug_set_turn(0, Phase::Play, 2);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard 询问弃牌
    let disc_ask = open(&e, "discard_to_limit");
    assert!(disc_ask.spec.options.contains(&cd(shown_cid)));
    answer(&mut e, "discard_to_limit", vec![cd(shown_cid)]);
    drain(&mut e);
    assert_eq!(mark_of(&e, 0, "xingchong"), 0, "失去展示牌后幸宠标记清零");
    assert_eq!(hand(&e, 0), h_before_loss + 3 - 1 + 2, "失去 1 张幸宠展示牌后摸 2 张");

    // 取消分支：拒绝 choose_draw（不摸牌）时仍可进入 show_cards（extension.js:6693）
    let _ = hand_turn_to(&mut e, 0, 3);
    answer(&mut e, "xingchong:choose_draw", Vec::new());
    let show_r3 = open(&e, "xingchong:show_cards");
    let c_r3 = show_r3.spec.options[0];
    answer(&mut e, "xingchong:show_cards", vec![c_r3]);
    drain(&mut e);
    assert_eq!(mark_of(&e, 0, "xingchong"), 1);

    let _ = hand_turn_to(&mut e, 0, 4);
    assert_eq!(mark_of(&e, 0, "xingchong"), 0, "新轮开始时旧幸宠展示标记应先清零");
}

// =============================================================================
// 10. dcxiaoxi（选将美化/extension.js:6774-6886）：真实 useCard(sha) + gainPlayerCard + 攻击范围
// =============================================================================

#[test]
fn l27_fix_dcxiaoxi_real_use_card_sha_gain_card_and_range() {
    let p = pack();
    let mut e = fresh(&p, 27924, &[&["dcxiaoxi"][..], &[][..]]);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);
    let shan1 = card_by(&e, |_, c| c.kind == CardKind::Shan);
    let shan2 = card_by(&e, |id, c| c.kind == CardKind::Shan && id != shan1);
    give(&mut e, 1, shan1);
    give(&mut e, 1, shan2);
    let hp1 = hp(&e, 1);

    e.debug_set_turn(0, Phase::Draw, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "dcxiaoxi:choose_count", vec![tag(2)]);
    answer(&mut e, "dcxiaoxi:choose_target", vec![pl(1)]);
    snapshot_roundtrip(&e, &p, vec![tag(1)]);
    answer(&mut e, "dcxiaoxi:choose_mode", vec![tag(1)]);
    let _ = open(&e, "respond_shan");
    answer(&mut e, "respond_shan", vec![cd(shan1)]);
    let _ = open(&e, "respond_shan");
    answer(&mut e, "respond_shan", vec![ChoiceOption::Pass]);
    drain(&mut e);
    assert_eq!(hp(&e, 1), hp1 - 1, "2 张虚拟杀：第 1 张被闪，第 2 张命中共伤 1");

    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);
    let hidden_c = card_by(&e, |_, c| c.kind == CardKind::Tao);
    let eq_c = card_by(&e, |_, c| c.kind == CardKind::Armor);
    give(&mut e, 1, hidden_c);
    equip(&mut e, 1, eq_c);
    e.debug_set_hp(0, 2);
    e.debug_set_turn(0, Phase::Draw, 2);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "dcxiaoxi:choose_target", vec![pl(1)]);
    answer(&mut e, "dcxiaoxi:choose_mode", vec![tag(0)]);
    let gain_ask = open(&e, "dcxiaoxi:gain_card");
    assert!(gain_ask.spec.options.iter().any(|o| matches!(o, ChoiceOption::HiddenCard(_))));
    assert!(gain_ask.spec.options.contains(&cd(eq_c)));
    let ai_pick = e.plan_action(0, &BoundedPlanner::new(4));
    assert!(matches!(ai_pick.action, GameAction::Answer { .. }));
    answer(&mut e, "dcxiaoxi:gain_card", vec![cd(eq_c)]);
    drain(&mut e);
    assert!(e.debug_hand(0).contains(&eq_c));

    let mut e4 = fresh(&p, 27925, &[&["dcxiaoxi"][..], &[][..], &[][..], &[][..]]);
    for seat in 0..4 {
        clear_hand(&mut e4, seat);
    }
    let plus1 = card_by(&e4, |_, c| c.kind == CardKind::HorsePlus);
    let plus2 = card_by(&e4, |id, c| c.kind == CardKind::HorsePlus && id != plus1);
    let plus3 = card_by(&e4, |id, c| c.kind == CardKind::HorsePlus && id != plus1 && id != plus2);
    equip(&mut e4, 1, plus1);
    equip(&mut e4, 2, plus2);
    equip(&mut e4, 3, plus3);
    e4.debug_set_turn(0, Phase::Draw, 1);
    let _ = submit(&mut e4, 0, GameAction::NextPhase);
    answer(&mut e4, "dcxiaoxi:choose_count", vec![tag(1)]);
    drain(&mut e4);
    assert_eq!(e4.state_core().player(0).max_hp, 3);
    assert!(e4.open_choice().is_none(), "无攻击范围内目标时不得打开选目标或直接打伤");
}

// =============================================================================
// 11. twfengji（选将美化/extension.js:6889-7043）：按 card.name 搜牌堆，不足即停不补抽
// =============================================================================

#[test]
fn l27_fix_twfengji_card_name_deck_only_and_stop_when_short() {
    let p = pack();
    let mut e = fresh(&p, 27926, &[&["twfengji"][..], &[][..]]);
    clear_hand(&mut e, 0);
    let w2_a = card_by(&e, |_, c| c.name == "weapon2");
    let w2_b = card_by(&e, |_, c| c.name == "weapon2b");
    let w3 = card_by(&e, |_, c| c.name == "weapon3");
    let tao1 = card_by(&e, |_, c| c.kind == CardKind::Tao);
    let tao2 = card_by(&e, |id, c| c.kind == CardKind::Tao && id != tao1);
    let extra_taos: Vec<u32> = e
        .state_core()
        .deck
        .iter()
        .copied()
        .filter(|&id| e.catalog().card_kind(id) == CardKind::Tao && id != tao1 && id != tao2)
        .collect();
    for id in extra_taos {
        e.state_core_mut().deck.retain(|&x| x != id);
        e.state_core_mut().discard.push(id);
    }
    e.state_core_mut().deck.retain(|&x| x != w2_a && x != w2_b && x != w3);
    e.state_core_mut().discard.push(w2_b);
    e.state_core_mut().deck.push(w3);
    e.state_core_mut().deck.retain(|&x| x != tao1);
    if !e.state_core().deck.contains(&tao2) {
        e.state_core_mut().deck.push(tao2);
    }

    give(&mut e, 0, tao1);
    give(&mut e, 0, w2_a);
    e.debug_set_turn(0, Phase::Draw, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "twfengji:choose_card", Vec::new());
    drain(&mut e);
    assert_eq!(hand(&e, 0), 2, "取消蜂集后手牌不入示");

    e.debug_set_turn(0, Phase::Draw, 2);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "twfengji:choose_card", vec![cd(w2_a)]);
    snapshot_roundtrip(&e, &p, vec![tag(2)]);
    answer(&mut e, "twfengji:choose_turns", vec![tag(2)]);
    assert_eq!(hand(&e, 0), 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // 0 End -> 1 Prepare (1/2)
    drain(&mut e);
    let _ = hand_turn_to(&mut e, 0, 3); // 0 Prepare (2/2)
    drain(&mut e);
    assert_eq!(hand(&e, 0), 1, "牌堆无同名 weapon2 时摸 0 张，绝不从弃牌堆/其他武器/随机补牌");
    assert!(!e.debug_hand(0).contains(&w2_b));
    assert!(!e.debug_hand(0).contains(&w3));

    e.debug_set_turn(0, Phase::Draw, 3);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "twfengji:choose_card", vec![cd(tao1)]);
    answer(&mut e, "twfengji:choose_turns", vec![tag(3)]);
    let h_after_exp = hand(&e, 0);
    let _ = hand_turn_to(&mut e, 1, 3);
    drain(&mut e);
    let _ = hand_turn_to(&mut e, 0, 4);
    drain(&mut e);
    let _ = hand_turn_to(&mut e, 1, 4);
    drain(&mut e);
    assert_eq!(hand(&e, 0), h_after_exp + 1, "牌堆只有 1 张桃时仅获该 1 张即停，不补抽 2 张随机牌");
    assert!(e.debug_hand(0).contains(&tao2));
}

// =============================================================================
// 12. twzhouzu（选将美化/extension.js:7441-7564）：受害者自选弃手牌 h（非 he）+ 1 雷伤
// =============================================================================

#[test]
fn l27_fix_twzhouzu_victim_chooses_hand_cards_only_not_equips() {
    let p = pack();
    let mut e = fresh(&p, 27927, &[&["twzhouzu"][..], &[][..]]);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);

    let eq = card_by(&e, |_, c| c.kind == CardKind::Armor);
    let h1 = card_by(&e, |_, c| c.kind == CardKind::Sha);
    let h2 = card_by(&e, |_, c| c.kind == CardKind::Shan);
    let h3 = card_by(&e, |id, c| c.kind == CardKind::Tao && id != h1 && id != h2);
    equip(&mut e, 1, eq);
    give(&mut e, 1, h1);
    give(&mut e, 1, h2);
    give(&mut e, 1, h3);

    e.debug_set_turn(0, Phase::Play, 1);
    activate(&mut e, 0, "twzhouzu");
    answer(&mut e, "twzhouzu:choose_target", vec![pl(1)]);
    answer(&mut e, "twzhouzu:choose_turns", vec![tag(2)]);

    let hp1 = hp(&e, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Seat 0 End end (1/2)
    drain(&mut e);
    e.debug_set_turn(1, Phase::Discard, 1);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Seat 1 End end (2/2)

    let ask = open(&e, "twzhouzu:target_discard");
    assert_eq!(ask.observer, 1, "必须由受害者 Seat 1 自己选弃牌");
    assert!(!ask.spec.allow_cancel);
    assert_eq!(ask.spec.min, 2);
    assert_eq!(ask.spec.max, 2);
    assert!(
        !ask.spec.options.contains(&cd(eq)),
        "extension.js:7542 默认 position='h'，不得将装备放入弃置选项"
    );
    let ai_disc = e.plan_action(1, &BoundedPlanner::new(4));
    assert!(matches!(ai_disc.action, GameAction::Answer { .. }));

    snapshot_roundtrip(&e, &p, vec![cd(h2), cd(h3)]);
    answer(&mut e, "twzhouzu:target_discard", vec![cd(h2), cd(h3)]);
    drain(&mut e);

    assert!(e.debug_hand(1).contains(&h1), "受害者自选弃置 h2/h3，保留了 h1");
    assert!(e.state_core().player(1).equips.contains(&eq), "装备区不得受咒诅弃置");
    assert_eq!(hp(&e, 1), hp1 - 1, "弃牌后受 1 点雷电伤害");

    clear_hand(&mut e, 1);
    let hp1_b = hp(&e, 1);
    e.debug_set_turn(0, Phase::Play, 2);
    activate(&mut e, 0, "twzhouzu");
    answer(&mut e, "twzhouzu:choose_target", vec![pl(1)]);
    answer(&mut e, "twzhouzu:choose_turns", vec![tag(1)]);
    e.debug_set_turn(0, Phase::Discard, 2);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e);
    assert!(e.state_core().player(1).equips.contains(&eq));
    assert_eq!(hp(&e, 1), hp1_b - 1);
}

// =============================================================================
// 13. 第 2 轮复核反例全覆盖回归测试（CDP 探针 1-5 & 8 项退回缺陷闭环）
// =============================================================================

#[test]
fn l27_fix_clandianzhan_already_linked_and_prior_multi_target_suit() {
    let p = pack();

    // Case 1A (`probe1.case1A`, extension.js:1886-1889):
    // 目标已横置且手中无同花色手牌时，`.filter(cardChongzhuable).length == 0` 为真，仍须触发点盏并摸 1
    let mut e1a = fresh(&p, 27940, &[&["clandianzhan"][..], &[][..]]);
    clear_hand(&mut e1a, 0);
    clear_hand(&mut e1a, 1);
    e1a.state_core_mut().player_mut(1).linked = true;
    let sp_sha = card_by(&e1a, |_, c| c.suit == Suit::Spade && c.kind == CardKind::Sha);
    let h_keep = card_by(&e1a, |_, c| c.suit == Suit::Heart);
    give(&mut e1a, 0, sp_sha);
    give(&mut e1a, 0, h_keep);
    e1a.debug_set_turn(0, Phase::Play, 1);
    let _ = roundtrip_bytes(&e1a, &[&p]);
    play(&mut e1a, 0, sp_sha, vec![1]);
    drain(&mut e1a);
    assert_eq!(hand(&e1a, 0), 2, "已横置目标+无同花色手牌仍须触发点盏摸 1");
    assert_eq!(flag(&e1a, 0, "clandianzhan_suit_mask") & 1, 1);

    // Case 1B (`probe1.case1B`, extension.js:1876-1884):
    // 同轮先使用多目标黑桃【南蛮入侵】，再使用单目标黑桃【杀】，同花色本轮已用过不得再触发点盏
    let mut e1b = fresh(
        &p,
        27941,
        &[&["clandianzhan"][..], &[][..], &[][..], &[][..]],
    );
    for seat in 0..4 {
        clear_hand(&mut e1b, seat);
    }
    let sp_nanman = card_by(&e1b, |_, c| {
        c.suit == Suit::Spade && c.kind == CardKind::SavageAttack
    });
    let sp_sha2 = card_by(&e1b, |_, c| c.suit == Suit::Spade && c.kind == CardKind::Sha);
    give(&mut e1b, 0, sp_nanman);
    give(&mut e1b, 0, sp_sha2);
    e1b.debug_set_turn(0, Phase::Play, 1);
    play(&mut e1b, 0, sp_nanman, vec![]);
    drain(&mut e1b);
    assert!(!e1b.state_core().player(1).linked);
    assert_eq!(flag(&e1b, 0, "clandianzhan_suit_mask") & 1, 0);

    play(&mut e1b, 0, sp_sha2, vec![1]);
    drain(&mut e1b);
    assert!(
        !e1b.state_core().player(1).linked,
        "同轮先出多目标黑桃后，单目标黑桃杀不得横置目标"
    );
    assert_eq!(hand(&e1b, 0), 0, "同轮先出多目标黑桃后，单目标黑桃杀不得摸牌");
    assert_eq!(flag(&e1b, 0, "clandianzhan_suit_mask") & 1, 0);
}

#[test]
fn l27_fix_reqimou_attack_range_not_boosted_and_ai_guard() {
    let p = pack();
    let mut e = fresh(&p, 27942, &[&["reqimou"][..], &[][..], &[][..], &[][..]]);
    for seat in 0..4 {
        clear_hand(&mut e, seat);
    }
    let plus = card_by(&e, |_, c| c.kind == CardKind::HorsePlus);
    equip(&mut e, 2, plus);
    e.debug_set_turn(0, Phase::Play, 1);

    // 1. `probe5.reqimouAttackRange`: reqimou(X=1) 只减全局距离，不加 attack_range（refresh.js:9262-9275）
    activate(&mut e, 0, "reqimou");
    answer(&mut e, "reqimou:choose_hp", vec![tag(1)]);
    answer(&mut e, "reqimou:confirm", vec![ChoiceOption::Confirm]);
    drain(&mut e);
    assert_eq!(
        e.state_core().attack_range(e.catalog(), 0),
        1,
        "reqimou 不得增加 attack_range（应保持无武器射程 1）"
    );
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 0, 2),
        2,
        "4 人局跨 1 马到 Seat 2 原距 3 - reqimou_x(1) = 2"
    );
    let sha = card_by(&e, |_, c| c.kind == CardKind::Sha);
    give(&mut e, 0, sha);
    assert!(
        is_rejected(submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: sha,
                targets: vec![2]
            }
        )),
        "攻击范围仍为 1 时，距 2 的 Seat 2 不得成为无武器杀合法目标"
    );

    // 2. AI `reqimou:confirm` 守卫（refresh.js:9246-9258）：hp < 3 或敌人全部超距时取消
    let mut e_low = fresh(&p, 27943, &[&["reqimou"][..], &[][..]]);
    clear_hand(&mut e_low, 0);
    e_low.debug_set_hp(0, 2);
    e_low.debug_set_turn(0, Phase::Play, 1);
    activate(&mut e_low, 0, "reqimou");
    answer(&mut e_low, "reqimou:choose_hp", vec![tag(1)]);
    let ai_low = e_low.plan_action(0, &BoundedPlanner::new(4));
    let GameAction::Answer { selection, .. } = ai_low.action else {
        panic!("expected AI Answer on reqimou:confirm");
    };
    assert!(
        selection != vec![ChoiceOption::Confirm],
        "hp < 3 时 AI 不得确认 reqimou:confirm，实际={selection:?}"
    );
    let req_cid = open(&e_low, "reqimou:confirm").id;
    let _ = submit(
        &mut e_low,
        0,
        GameAction::Answer {
            choice_id: req_cid,
            selection,
        },
    );
    drain(&mut e_low);
    assert_eq!(hp(&e_low, 0), 2);
    assert_eq!(flag(&e_low, 0, "reqimou_used"), 0);
}

#[test]
fn l27_fix_bolan_pool_excludes_triggers_multi_holder_and_preserves_owned() {
    let p = pack();

    // 1. 池化过滤：触发技 / 限定 / 主公 / 觉醒 / 使命技不得进入博览采样池（extension.js:7751-7773）
    let forbidden_in_pool = [
        "xinfu_lingren",
        "shouye",
        "dddchengjing",
        "dddduanbing",
        "rechuhai",
        "th_yeyan",
        "qimou",
        "reqimou",
        "taoluan",
        "jijiang",
        "tuxi",
        "luoyi",
        "bolan",
        "bolan_g",
    ];
    for seed in 27950u64..27962 {
        let mut e = fresh(&p, seed, &[&["bolan"][..], &[][..]]);
        e.debug_set_turn(0, Phase::Draw, 1);
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        for i in 0..3 {
            let def_plus1 = flag(&e, 0, &format!("bolan_opt_{i}"));
            if def_plus1 > 0 {
                let sname = e.catalog().skill_name((def_plus1 - 1) as u16);
                assert!(
                    !forbidden_in_pool.contains(&sname),
                    "触发/限定/主公/非phaseUse-usable:1 技 {sname} 不得进入博览池"
                );
            }
        }
    }

    // 2. `bolan_g` 多名其他存活 `bolan` 持有者时必须弹出 `bolan_g:choose_target`，支持取消/存档恢复/AI
    let mut e_multi = fresh(
        &p,
        27965,
        &[&["bolan_g"][..], &["bolan"][..], &["bolan"][..]],
    );
    e_multi.debug_set_turn(0, Phase::Play, 1);
    activate(&mut e_multi, 0, "bolan_g");
    let t_ask = open(&e_multi, "bolan_g:choose_target");
    assert!(t_ask.spec.allow_cancel);
    assert!(t_ask.spec.options.contains(&pl(1)) && t_ask.spec.options.contains(&pl(2)));
    snapshot_roundtrip(&e_multi, &p, Vec::new());
    answer(&mut e_multi, "bolan_g:choose_target", Vec::new());
    drain(&mut e_multi);
    assert_eq!(hp(&e_multi, 0), 4, "取消 bolan_g:choose_target 不得扣血");
    let bolan_g_def = e_multi.catalog().skill_by_name("bolan_g").unwrap();
    assert!(
        !e_multi
            .state_core()
            .skills
            .get(&(0, bolan_g_def))
            .is_some_and(|i| i.once_used)
    );

    activate(&mut e_multi, 0, "bolan_g");
    let ai_pick = e_multi.plan_action(0, &BoundedPlanner::new(4));
    assert!(matches!(ai_pick.action, GameAction::Answer { .. }));
    snapshot_roundtrip(&e_multi, &p, vec![pl(2)]);
    answer(&mut e_multi, "bolan_g:choose_target", vec![pl(2)]);
    let g_sk = open(&e_multi, "bolan:choose_skill");
    assert_eq!(g_sk.observer, 2, "应由选定的目标 bolan 持有者 Seat 2 选择技能");
    answer(&mut e_multi, "bolan:choose_skill", vec![tag(0)]);
    drain(&mut e_multi);
    let granted2 = (flag(&e_multi, 0, "bolan_granted_def") - 1) as u16;
    let granted2_name = e_multi.catalog().skill_name(granted2);
    assert!(has_skill(&e_multi, 0, granted2_name));

    // 3. 保留已预先持有的技能 & 出牌阶段多技并存，phaseUseEnd 后仅移除新授临时技
    let mut probe = fresh(&p, 27966, &[&["bolan", "bolan_g"][..], &["bolan"][..]]);
    probe.debug_set_turn(0, Phase::Draw, 1);
    let _ = submit(&mut probe, 0, GameAction::NextPhase);
    let pre_owned_def = (flag(&probe, 0, "bolan_opt_0") - 1) as u16;
    let pre_owned_name = probe.catalog().skill_name(pre_owned_def);

    let mut e_own = fresh(
        &p,
        27966,
        &[&["bolan", "bolan_g", pre_owned_name][..], &["bolan"][..]],
    );
    e_own.debug_set_turn(0, Phase::Draw, 1);
    let _ = submit(&mut e_own, 0, GameAction::NextPhase);
    answer(&mut e_own, "bolan:choose_skill", vec![tag(0)]); // 选中本已持有的 pre_owned_name
    drain(&mut e_own);
    // 同出牌阶段再通过 bolan_g 获第 2 个技能
    activate(&mut e_own, 0, "bolan_g");
    answer(&mut e_own, "bolan:choose_skill", vec![tag(0)]);
    drain(&mut e_own);
    let extra_name = e_own
        .catalog()
        .skill_name((flag(&e_own, 0, "bolan_granted_def") - 1) as u16);

    let _ = submit(&mut e_own, 0, GameAction::NextPhase); // Play -> Discard
    drain(&mut e_own);
    assert!(
        has_skill(&e_own, 0, pre_owned_name),
        "phaseUseEnd 不得误删角色原本已持有的技能 {pre_owned_name}"
    );
    if extra_name != pre_owned_name {
        assert!(
            !has_skill(&e_own, 0, extra_name),
            "phaseUseEnd 应移除额外临时获得的技能 {extra_name}"
        );
    }
}

#[test]
fn l27_fix_sbqingzheng_strict_suit_count_and_target_suits() {
    let p = pack();

    // Case 1 (`probe2.sbq2Suits`): 0 治世需恰选 3 种花色，只有 2 种花色时不可缩水，仅可取消
    let mut e1 = fresh(&p, 27967, &[&[][..], &["sbqingzheng", "sbjianxiong"][..]]);
    clear_hand(&mut e1, 0);
    clear_hand(&mut e1, 1);
    let sp = card_by(&e1, |_, c| c.suit == Suit::Spade);
    let ht = card_by(&e1, |_, c| c.suit == Suit::Heart);
    let cl = card_by(&e1, |_, c| c.suit == Suit::Club);
    give(&mut e1, 1, sp);
    give(&mut e1, 1, ht);
    give(&mut e1, 0, cl);
    e1.debug_set_turn(1, Phase::Draw, 1);
    let _ = submit(&mut e1, 1, GameAction::NextPhase);
    let ask = open(&e1, "sbqingzheng:choose_suits");
    assert_eq!(ask.spec.min, 3);
    assert_eq!(ask.spec.max, 3);
    assert!(ask.spec.allow_cancel);
    assert!(
        is_rejected(submit(
            &mut e1,
            ask.observer,
            GameAction::Answer {
                choice_id: ask.id,
                selection: vec![tag(0), tag(1)],
            }
        )),
        "不足 3 花色不得确认部分花色"
    );
    let ai_ans = e1.plan_action(1, &BoundedPlanner::new(4));
    assert_eq!(
        ai_ans.action,
        GameAction::Answer {
            choice_id: ask.id,
            selection: Vec::new(),
        },
        "AI 花色不足时应取消而非阻塞"
    );
    answer(&mut e1, "sbqingzheng:choose_suits", Vec::new());
    drain(&mut e1);
    assert_eq!(hand(&e1, 1), 2);

    // Case 2: 目标花色只展示目标实际持有的花色，缺门花色不得可选
    let mut e2 = fresh(&p, 27968, &[&[][..], &["sbqingzheng"][..]]);
    clear_hand(&mut e2, 0);
    clear_hand(&mut e2, 1);
    let s1 = card_by(&e2, |_, c| c.suit == Suit::Spade);
    let h1 = card_by(&e2, |_, c| c.suit == Suit::Heart);
    let c1 = card_by(&e2, |_, c| c.suit == Suit::Club);
    let th = card_by(&e2, |id, c| c.suit == Suit::Heart && id != h1);
    give(&mut e2, 1, s1);
    give(&mut e2, 1, h1);
    give(&mut e2, 1, c1);
    give(&mut e2, 0, th);
    e2.debug_set_turn(1, Phase::Draw, 1);
    let _ = submit(&mut e2, 1, GameAction::NextPhase);
    answer(
        &mut e2,
        "sbqingzheng:choose_suits",
        vec![tag(0), tag(1), tag(2)],
    );
    answer(&mut e2, "sbqingzheng:choose_target", vec![pl(0)]);
    let ts = open(&e2, "sbqingzheng:choose_target_suit");
    assert_eq!(ts.spec.options, vec![tag(1)], "仅应暴露目标实有花色");
    assert!(
        is_rejected(submit(
            &mut e2,
            ts.observer,
            GameAction::Answer {
                choice_id: ts.id,
                selection: vec![tag(0)],
            }
        )),
        "目标无黑桃时选黑桃必须被拒"
    );
    answer(&mut e2, "sbqingzheng:choose_target_suit", vec![tag(1)]);
    drain(&mut e2);
    assert_eq!(hand(&e2, 0), 0);
    assert_eq!(hand(&e2, 1), 0);
}

#[test]
fn l27_fix_xingchong_triggers_on_play_use_respond_and_equip() {
    let p = pack();

    // Case 1 (`probe3.xcAfterUseCard`): PlayCard 使用幸宠展示的【杀】必须立即触发摸 2 与清标
    let mut e = fresh(&p, 27969, &[&["xingchong"][..], &[][..]]);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);
    let sha = card_by(&e, |_, c| c.kind == CardKind::Sha);
    give(&mut e, 0, sha);
    let _ = hand_turn_to(&mut e, 0, 2);
    answer(&mut e, "xingchong:choose_draw", Vec::new());
    answer(&mut e, "xingchong:show_cards", vec![cd(sha)]);
    drain(&mut e);
    assert_eq!(mark_of(&e, 0, "xingchong"), 1);

    e.debug_set_turn(0, Phase::Play, 2);
    let hp1 = hp(&e, 1);
    play(&mut e, 0, sha, vec![1]);
    drain(&mut e);
    assert_eq!(mark_of(&e, 0, "xingchong"), 0, "使用展示牌后幸宠标记清零");
    assert_eq!(hand(&e, 0), 2, "使用 1 张幸宠展示【杀】后必摸 2 张");
    assert_eq!(hp(&e, 1), hp1 - 1);

    // Case 2 (`Equipped` + `CardResponded`): 装备展示装备牌与响应打出展示【闪】均摸 2
    let mut e2 = fresh(&p, 27970, &[&["xingchong"][..], &[][..]]);
    clear_hand(&mut e2, 0);
    clear_hand(&mut e2, 1);
    let eq = card_by(&e2, |_, c| c.kind == CardKind::Armor);
    let shan = card_by(&e2, |_, c| c.kind == CardKind::Shan);
    give(&mut e2, 0, eq);
    give(&mut e2, 0, shan);
    let _ = hand_turn_to(&mut e2, 0, 2);
    answer(&mut e2, "xingchong:choose_draw", Vec::new());
    answer(&mut e2, "xingchong:show_cards", vec![cd(eq), cd(shan)]);
    drain(&mut e2);
    assert_eq!(mark_of(&e2, 0, "xingchong"), 2);

    e2.debug_set_turn(0, Phase::Play, 2);
    play(&mut e2, 0, eq, vec![]);
    drain(&mut e2);
    assert_eq!(mark_of(&e2, 0, "xingchong"), 1);
    assert_eq!(hand(&e2, 0), 3, "装备展示牌后应摸 2 张（1 张闪 + 摸 2）");

    let enemy_sha = card_by(&e2, |_, c| c.kind == CardKind::Sha);
    give(&mut e2, 1, enemy_sha);
    e2.debug_set_turn(1, Phase::Play, 2);
    play(&mut e2, 1, enemy_sha, vec![0]);
    answer(&mut e2, "respond_shan", vec![cd(shan)]);
    drain(&mut e2);
    assert_eq!(mark_of(&e2, 0, "xingchong"), 0);
    assert_eq!(hand(&e2, 0), 4, "响应打出展示【闪】后应摸 2 张（原 3 - 1 + 2 = 4）");
}

#[test]
fn l27_fix_dcxiaoxi_single_branch_auto_and_empty_deck_virtual_sha() {
    let p = pack();

    // Case 1 (`probe4.dxAfterEmptyDeck`): 空牌堆 + 目标空 he（!bool1 && bool2）跳过 choose_mode 直发虚拟杀
    let mut e1 = fresh(&p, 27971, &[&["dcxiaoxi"][..], &[][..]]);
    clear_hand(&mut e1, 0);
    clear_hand(&mut e1, 1);
    e1.state_core_mut().deck.clear();
    let hp1 = hp(&e1, 1);
    e1.debug_set_turn(0, Phase::Draw, 1);
    let _ = submit(&mut e1, 0, GameAction::NextPhase);
    answer(&mut e1, "dcxiaoxi:choose_count", vec![tag(2)]);
    answer(&mut e1, "dcxiaoxi:choose_target", vec![pl(1)]);
    assert!(
        e1.open_choice().is_none(),
        "目标空 he 时应跳过 dcxiaoxi:choose_mode 并直接结算空牌堆虚拟杀（目标无手牌无需出闪）"
    );
    drain(&mut e1);
    assert_eq!(hp(&e1, 1), hp1 - 2, "空牌堆下 2 张虚拟杀全额结算造成 2 伤");

    // Case 2 (`bool1 && !bool2`): 目标空城+仅装备（不可成为杀目标但有 he 牌）跳过 choose_mode 直进 gain_card
    let mut e2 = fresh(&p, 27972, &[&["dcxiaoxi"][..], &["空城"][..]]);
    clear_hand(&mut e2, 0);
    clear_hand(&mut e2, 1);
    let eq = card_by(&e2, |_, c| c.kind == CardKind::Armor);
    equip(&mut e2, 1, eq);
    e2.debug_set_turn(0, Phase::Draw, 1);
    let _ = submit(&mut e2, 0, GameAction::NextPhase);
    answer(&mut e2, "dcxiaoxi:choose_count", vec![tag(1)]);
    answer(&mut e2, "dcxiaoxi:choose_target", vec![pl(1)]);
    assert_eq!(
        e2.open_choice().map(|c| c.spec.prompt),
        Some("dcxiaoxi:gain_card"),
        "仅获牌合法时必须跳过 dcxiaoxi:choose_mode 直入 gain_card"
    );
    answer(&mut e2, "dcxiaoxi:gain_card", vec![cd(eq)]);
    drain(&mut e2);
    assert!(e2.debug_hand(0).contains(&eq));

    // Case 3 (`!bool1 && !bool2`): 目标空城且无任何牌 -> 不开 choose_mode 直接结束
    let mut e3 = fresh(&p, 27973, &[&["dcxiaoxi"][..], &["空城"][..]]);
    clear_hand(&mut e3, 0);
    clear_hand(&mut e3, 1);
    e3.debug_set_turn(0, Phase::Draw, 1);
    let _ = submit(&mut e3, 0, GameAction::NextPhase);
    answer(&mut e3, "dcxiaoxi:choose_count", vec![tag(1)]);
    answer(&mut e3, "dcxiaoxi:choose_target", vec![pl(1)]);
    drain(&mut e3);
    assert!(e3.open_choice().is_none());
    assert_eq!(hp(&e3, 1), 4);
}

#[test]
fn l27_fix_twfengji_matches_sha_fire_thunder_variants() {
    let p = pack();
    let mut e = fresh(&p, 27974, &[&["twfengji"][..], &[][..]]);
    clear_hand(&mut e, 0);
    let sha = card_by(&e, |_, c| c.kind == CardKind::Sha);
    let fire_sha = card_by(&e, |_, c| c.kind == CardKind::FireSha);
    let lei_sha = card_by(&e, |_, c| c.kind == CardKind::LeiSha);
    let shan = card_by(&e, |_, c| c.kind == CardKind::Shan);

    // 清空牌堆后仅放入 1 张火杀与 1 张雷杀（probe5.fengjiShaNames: 三者 card.name 皆为 "sha"）
    e.state_core_mut().deck.clear();
    e.state_core_mut().deck.push(fire_sha);
    e.state_core_mut().deck.push(lei_sha);
    e.state_core_mut().deck.push(shan);
    give(&mut e, 0, sha);

    e.debug_set_turn(0, Phase::Draw, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "twfengji:choose_card", vec![cd(sha)]);
    answer(&mut e, "twfengji:choose_turns", vec![tag(2)]);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Seat 0 End -> Seat 1 Prepare (1/2)
    drain(&mut e);
    let _ = hand_turn_to(&mut e, 0, 2); // Seat 0 Prepare (2/2)
    drain(&mut e);

    assert!(
        e.debug_hand(0).contains(&fire_sha) && e.debug_hand(0).contains(&lei_sha),
        "蜂集扣置普通【杀】应从牌堆获得同名（sha）火杀与雷杀"
    );
    assert!(!e.debug_hand(0).contains(&shan));
}

#[test]
fn l27_fix_xingchong_snatch_steal_triggers_draw_two_and_clears_mark() {
    let p = pack();

    // 正例（选将美化/extension.js:6720-6771，verifier-l27-edge-cdp-results.json xingchongSnatch）：
    // Seat 0 通过 xingchong 展示 1 张手牌后，Seat 1 使用【顺手牵羊】盗走该展示牌 →
    // 牌进入 Seat 1 手牌，Seat 0 立即摸 2 张且 mark:xingchong / xingchong_card 清零。
    let mut e = fresh(&p, 27975, &[&["xingchong"][..], &[][..]]);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);
    let sha = card_by(&e, |_, c| c.kind == CardKind::Sha);
    let snatch = card_by(&e, |_, c| c.kind == CardKind::Snatch);
    give(&mut e, 0, sha);
    give(&mut e, 1, snatch);

    let _ = hand_turn_to(&mut e, 0, 2);
    answer(&mut e, "xingchong:choose_draw", Vec::new());
    snapshot_roundtrip(&e, &p, vec![cd(sha)]);
    answer(&mut e, "xingchong:show_cards", vec![cd(sha)]);
    drain(&mut e);
    assert_eq!(mark_of(&e, 0, "xingchong"), 1);
    assert_eq!(flag(&e, 0, &format!("xingchong_card:{sha}")), 1);
    assert_eq!(hand(&e, 0), 1);

    e.debug_set_turn(1, Phase::Play, 2);
    play(&mut e, 1, snatch, vec![0]);
    drain(&mut e);

    assert!(e.debug_hand(1).contains(&sha), "顺手牵羊应将展示牌移入 Seat 1 手牌");
    assert_eq!(mark_of(&e, 0, "xingchong"), 0, "顺手牵羊移走展示牌后 xingchong 标记须清零");
    assert_eq!(flag(&e, 0, &format!("xingchong_card:{sha}")), 0);
    assert_eq!(hand(&e, 0), 2, "失去 1 张展示牌应摸 2 张（1 - 1 + 2 = 2）");
    let _ = roundtrip_bytes(&e, &[&p]);

    // 负例：顺手牵羊盗走未展示的非幸宠手牌时不得触发摸 2
    let mut neg = fresh(&p, 27976, &[&["xingchong"][..], &[][..]]);
    clear_hand(&mut neg, 0);
    clear_hand(&mut neg, 1);
    let unshown = card_by(&neg, |_, c| c.kind == CardKind::Shan);
    let shown = card_by(&neg, |_, c| c.kind == CardKind::Sha);
    let snatch2 = card_by(&neg, |_, c| c.kind == CardKind::Snatch);
    give(&mut neg, 0, unshown);
    give(&mut neg, 0, shown);
    give(&mut neg, 1, snatch2);

    let _ = hand_turn_to(&mut neg, 0, 2);
    answer(&mut neg, "xingchong:choose_draw", Vec::new());
    answer(&mut neg, "xingchong:show_cards", vec![cd(shown)]);
    drain(&mut neg);
    assert_eq!(mark_of(&neg, 0, "xingchong"), 1);

    neg.debug_set_turn(1, Phase::Play, 2);
    play(&mut neg, 1, snatch2, vec![0]);
    drain(&mut neg);
    assert!(neg.debug_hand(1).contains(&unshown));
    assert_eq!(mark_of(&neg, 0, "xingchong"), 1, "被盗非展示牌时幸宠标记不变");
    assert_eq!(hand(&neg, 0), 1, "被盗非展示牌时不得摸 2");
}

#[test]
fn l27_fix_new_zhixi_blocks_nullify_window_on_trick_and_hp_cap() {
    let p = pack();

    // Case A（选将美化/extension.js:1739-1748 + card/standard.js:2725-2728，
    // verifier-l27-edge-cdp-results.json newZhixiMods）：
    // Seat 1 经 new_meibu 获得临时 new_zhixi；在自己 Phase::Play 使用锦囊【过河拆桥】
    // 置位 new_zhixi_mark2=1 后，Seat 0 打出【无懈可击】→ 无懈窗口不得再询问持有【无懈可击】的 Seat 1！
    let mut e = fresh(&p, 27977, &[&["new_meibu"][..], &[][..]]);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);
    let disc = card_by(&e, |_, c| c.kind == CardKind::Shan);
    let eq0 = card_by(&e, |_, c| c.kind == CardKind::Armor);
    let wuxie0 = card_by(&e, |_, c| c.kind == CardKind::Nullify);
    let dismantle = card_by(&e, |_, c| c.kind == CardKind::Dismantle);
    let wuxie1 = card_by(&e, |id, c| c.kind == CardKind::Nullify && id != wuxie0);
    let extra_sha = card_by(&e, |_, c| c.kind == CardKind::Sha);
    give(&mut e, 0, disc);
    give(&mut e, 0, wuxie0);
    equip(&mut e, 0, eq0);
    give(&mut e, 1, dismantle);
    give(&mut e, 1, wuxie1);
    give(&mut e, 1, extra_sha);

    e.debug_set_turn(1, Phase::Draw, 1);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    confirm_triggers(&mut e);
    answer(&mut e, "new_meibu_discard", vec![cd(disc)]);
    drain(&mut e);
    assert!(has_skill(&e, 1, "new_zhixi"));

    assert!(!e.catalog().blocks_all_cards(e.state_core(), 1));
    play(&mut e, 1, dismantle, vec![0]);
    assert_eq!(mark_of(&e, 1, "new_zhixi"), 1);
    assert_eq!(mark_of(&e, 1, "new_zhixi_mark2"), 1);
    assert!(e.catalog().blocks_all_cards(e.state_core(), 1));

    let ask = open(&e, "nullify_window");
    assert_eq!(ask.observer, 0, "无懈窗口应正常询问未被止息封锁的 Seat 0");
    snapshot_roundtrip(&e, &p, vec![cd(wuxie0)]);
    answer(&mut e, "nullify_window", vec![cd(wuxie0)]);
    assert!(
        e.open_choice().is_none(),
        "new_zhixi 置位 new_zhixi_mark2=1 后，Seat 1 虽持无懈可击仍不得被 nullify_window 询问"
    );
    drain(&mut e);
    assert!(e.state_core().player(0).equips.contains(&eq0), "拆桥被抵消，装备保留");
    assert!(e.debug_hand(1).contains(&wuxie1), "Seat 1 无法打出无懈可击，仍留手牌");
    assert!(
        is_rejected(submit(
            &mut e,
            1,
            GameAction::PlayCard {
                card: extra_sha,
                targets: vec![0],
            },
        )),
        "止息封锁下不得继续出杀"
    );

    // 负例对照：Seat 0 取消 new_meibu 后，Seat 1 未持 new_zhixi，同局拆桥→Seat 0 无懈→Seat 1 必须获询并可反无懈
    let mut neg = fresh(&p, 27978, &[&["new_meibu"][..], &[][..]]);
    clear_hand(&mut neg, 0);
    clear_hand(&mut neg, 1);
    let n_disc = card_by(&neg, |_, c| c.kind == CardKind::Shan);
    let n_eq0 = card_by(&neg, |_, c| c.kind == CardKind::Armor);
    let n_wx0 = card_by(&neg, |_, c| c.kind == CardKind::Nullify);
    let n_dis = card_by(&neg, |_, c| c.kind == CardKind::Dismantle);
    let n_wx1 = card_by(&neg, |id, c| c.kind == CardKind::Nullify && id != n_wx0);
    give(&mut neg, 0, n_disc);
    give(&mut neg, 0, n_wx0);
    equip(&mut neg, 0, n_eq0);
    give(&mut neg, 1, n_dis);
    give(&mut neg, 1, n_wx1);

    neg.debug_set_turn(1, Phase::Draw, 1);
    let _ = submit(&mut neg, 1, GameAction::NextPhase);
    confirm_triggers(&mut neg);
    answer(&mut neg, "new_meibu_discard", Vec::new());
    drain(&mut neg);
    assert!(!has_skill(&neg, 1, "new_zhixi"));

    play(&mut neg, 1, n_dis, vec![0]);
    assert_eq!(open(&neg, "nullify_window").observer, 1);
    answer(&mut neg, "nullify_window", vec![ChoiceOption::Pass]);
    assert_eq!(open(&neg, "nullify_window").observer, 0);
    answer(&mut neg, "nullify_window", vec![cd(n_wx0)]);
    let counter_ask = open(&neg, "nullify_window");
    assert_eq!(counter_ask.observer, 1, "未持止息时 Seat 1 应正常获得反无懈询问");
    answer(&mut neg, "nullify_window", vec![cd(n_wx1)]);
    drain(&mut neg);
    assert!(
        !neg.debug_hand(0).contains(&n_disc) && neg.state_core().discard.contains(&n_disc),
        "Seat 1 反无懈成功后【过河拆桥】生效，弃置 Seat 0 手牌"
    );

    // Case B（次数达体力上限 count >= hp 且未用锦囊 new_zhixi_mark2 == 0）
    let mut e2 = fresh(&p, 27979, &[&[][..], &["new_zhixi"][..]]);
    clear_hand(&mut e2, 0);
    clear_hand(&mut e2, 1);
    e2.debug_set_hp(1, 2);
    let b_jiu = card_by(&e2, |_, c| c.kind == CardKind::Jiu);
    let b_sha = card_by(&e2, |_, c| c.kind == CardKind::Sha);
    let b_dism = card_by(&e2, |_, c| c.kind == CardKind::Dismantle);
    give(&mut e2, 1, b_jiu);
    give(&mut e2, 1, b_sha);
    give(&mut e2, 1, b_dism);
    e2.debug_set_turn(1, Phase::Play, 1);
    play(&mut e2, 1, b_jiu, vec![]);
    drain(&mut e2);
    assert_eq!(mark_of(&e2, 1, "new_zhixi"), 1);
    assert_eq!(mark_of(&e2, 1, "new_zhixi_mark2"), 0);
    assert!(!e2.catalog().blocks_all_cards(e2.state_core(), 1));

    play(&mut e2, 1, b_sha, vec![0]);
    drain(&mut e2);
    assert_eq!(mark_of(&e2, 1, "new_zhixi"), 2);
    assert_eq!(mark_of(&e2, 1, "new_zhixi_mark2"), 0);
    assert!(
        e2.catalog().blocks_all_cards(e2.state_core(), 1),
        "new_zhixi 次数达体力上限后（未用锦囊）亦须置位全牌封锁"
    );
    assert!(is_rejected(submit(
        &mut e2,
        1,
        GameAction::PlayCard {
            card: b_dism,
            targets: vec![0],
        },
    )));
}
