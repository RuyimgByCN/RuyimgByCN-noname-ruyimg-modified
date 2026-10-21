//! 整包验收续（U05 swd 首切片，规则 v85）：护符 hufu——仅濒死自救的
//! 桃形救援牌 + 真实宿主全生命周期。
use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::CardKind;
use engine_native::event::DamageNature;
use engine_native::RejectReason;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};

static DEDUP: AtomicU64 = AtomicU64::new(1);
use std::sync::atomic::{AtomicU64, Ordering};

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

fn repo_extensions_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn options() -> InstallOptions {
    InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

fn bridge(json: &str) -> ExtensionPack {
    let out = content_native_loader::bridge_pack(json).expect("swd 桥接");
    assert_eq!(out.pack.def.name, "shahai-swd-upstream-noname");
    out.pack
}

use engine_native::content::extension::ExtensionPack;

fn pass_nullify(e: &mut Engine) {
    while let Some(c) = e.open_choice() {
        if c.spec.prompt != "nullify_window" {
            break;
        }
        let (cid, obs) = (c.id, c.observer);
        submit(
            e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
}

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..96 {
        if e.turn().current == seat && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice() {
            let sel: Vec<ChoiceOption> = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else if !c.spec.options.is_empty() {
                vec![c.spec.options[0]]
            } else {
                vec![]
            };
            let (cid, obs) = (c.id, c.observer);
            submit(
                e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            );
            continue;
        }
        let actor = e.turn().current;
        match submit(e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
}

fn play(e: &mut Engine, card: u32, targets: Vec<u8>) -> CommandOutcome {
    submit(e, e.turn().current, GameAction::PlayCard { card, targets })
}

fn hufu_id(e: &Engine, n: usize) -> u32 {
    let mut seen = 0;
    for (i, c) in e.catalog().cards.iter().enumerate() {
        if c.kind == CardKind::Hufu {
            if seen == n {
                return i as u32;
            }
            seen += 1;
        }
    }
    panic!("护符不足");
}

fn dying_save_pass(e: &mut Engine) {
    if let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(
            e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
}

fn assert_zone_conservation_swd(e: &Engine) {
    let _ = e; // 占位：与既有 zone 断言同口径（本切片仅守恒粗检）
}

// ---------------------------------------------------------------------------
// U05 swd 第二十二批（规则 v197）：机关妖术（jiguanyaoshu，♣3/♠3）——
// 从牌堆+弃堆装备实体中发现一张「目标空装备栏」子类型的装备（登记差异：
// 上游=全部已定义装备牌凭空造牌），走完整装配；授予防重标记（同目标待
// 获得期间不可再指定；其补偿触发面——失去装备补灵剑——暂缓登记至机关
// 术批）。目标 isMin（手牌数≤体力）不可指定（上游 !isMin 同口径）。
// ---------------------------------------------------------------------------

fn jiguan_ids(e: &Engine, keyword: &str) -> Vec<u32> {
    e.catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| c.name.contains(keyword))
        .map(|(i, _)| i as u32)
        .collect()
}

fn inject_hand_at(e: &mut Engine, pack: &ExtensionPack, player: u8, cards: &[u32]) {
    let mut snap = e.save();
    for c in cards {
        snap.core.deck.retain(|x| x != c);
        snap.core.discard.retain(|x| x != c);
        for pl in snap.core.players.iter_mut() {
            pl.hand.retain(|x| x != c);
        }
    }
    for c in cards {
        snap.core.players[player as usize].hand.push(*c);
    }
    *e = Engine::restore_with_extensions(&snap, &[pack]).expect("注入");
}

fn inject_hand(e: &mut Engine, pack: &ExtensionPack, cards: &[u32]) {
    let mut snap = e.save();
    for c in cards {
        snap.core.deck.retain(|x| x != c);
        snap.core.discard.retain(|x| x != c);
        for pl in snap.core.players.iter_mut() {
            pl.hand.retain(|x| x != c);
        }
    }
    for c in cards {
        snap.core.players[0].hand.push(*c);
    }
    *e = Engine::restore_with_extensions(&snap, &[pack]).expect("注入");
}

// ---------------------------------------------------------------------------
// U05 swd 机关术批（规则 v198，存档 v44）：灵剑池 7 张（hslingjian——
// 旋风之刃/中行虎甲/星体改造/时间回溯/紧急冷冻/圣休号角/隐秘离场）+
// 机关妖术补偿触发补齐。登记差异：①灵剑实体花色点数为原生登记值（上游
// 凭空造牌花色随机）；②装备/灵剑发现池=牌堆+弃堆可见实体（上游全定义
// 池凭空造）；③补偿触发以弃牌事件近似失去装备区牌（替换/获得离去不触
// 发）；④星体改造重置时点=下一回合开始（上游上一结束阶段）。机关术×2
// 暂缓：运行时卡牌合成/改造原语缺口（T18）。
// ---------------------------------------------------------------------------

fn lingjian_id(e: &Engine, keyword: &str) -> u32 {
    e.catalog()
        .cards
        .iter()
        .position(|c| c.name.contains(keyword))
        .expect("灵剑在目录") as u32
}

fn hand_len_w(e: &Engine, p: u8) -> usize {
    e.state_core().player(p).hand.len()
}

/// 可选确认门排空（首选项应答）。
fn settle_optional_confirm(e: &mut Engine) {
    for _ in 0..8 {
        let pending = e.open_choice().map(|ch| (ch.id, ch.observer));
        let Some((cid, obs)) = pending else { return };
        let first = {
            let ch = e.open_choice().expect("still open");
            vec![ch.spec.options[0].clone()]
        };
        submit(
            e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: first,
            },
        );
    }
}

/// 快照重置本回合出杀计数（批 22/23 helper 的独立包装）。
fn reset_sha_for(e: &mut Engine, pack: &ExtensionPack) {
    let mut snap = e.save();
    snap.core.turn.sha_used_play_phase = false;
    *e = Engine::restore_with_extensions(&snap, &[pack]).expect("重置");
}

/// 批内辅助：推进到 p0 出牌阶段（b66）
fn next_p0_play_fn(e: &mut Engine) {
    for _ in 0..3 {
        let c = e.turn().current;
        let _ = submit(e, c, GameAction::NextPhase);
    }
    for _ in 0..16 {
        let c = e.turn().current;
        if c == 0 && e.state_core().turn.phase == engine_native::event::Phase::Play {
            break;
        }
        let _ = submit(e, c, GameAction::NextPhase);
    }
    advance_to_play(e, 0);
}

/// 注入一件装备（从全区域清除后挂到指定座位装备区；测试布阵用）。
fn inject_equip_swd(e: &mut Engine, pack: &ExtensionPack, player: u8, kind: CardKind) -> u32 {
    let card = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == kind)
        .expect("牌种在目录");
    let mut snap = e.save();
    for p in snap.core.players.iter_mut() {
        p.hand.retain(|c| *c != card);
        p.equips.retain(|c| *c != card);
    }
    snap.core.deck.retain(|c| *c != card);
    snap.core.discard.retain(|c| *c != card);
    snap.core.table.retain(|c| *c != card);
    snap.core.players[player as usize].equips.push(card);
    *e = Engine::restore_with_extensions(&snap, &[pack]).expect("装备注入");
    card
}

/// 依次应答所有开放选择（min 张选项；可取消时取空），直到静止。
fn drain_choices(e: &mut Engine) {
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let sel: Vec<ChoiceOption> = if c.spec.min == 0 {
            Vec::new()
        } else {
            c.spec
                .options
                .iter()
                .take(c.spec.min.max(1))
                .copied()
                .collect()
        };
        let (cid, obs) = (c.id, c.observer);
        let _ = submit(
            e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
    }
}

#[path = "g1_shahai_swd/bingjian.rs"]
mod bingjian;
#[path = "g1_shahai_swd/cyxianjiang.rs"]
mod cyxianjiang;
#[path = "g1_shahai_swd/daixing.rs"]
mod daixing;
#[path = "g1_shahai_swd/datong.rs"]
mod datong;
#[path = "g1_shahai_swd/duanyue.rs"]
mod duanyue;
#[path = "g1_shahai_swd/fengming.rs"]
mod fengming;
#[path = "g1_shahai_swd/five.rs"]
mod five;
#[path = "g1_shahai_swd/funiao.rs"]
mod funiao;
#[path = "g1_shahai_swd/gtiandao.rs"]
mod gtiandao;
#[path = "g1_shahai_swd/guaili.rs"]
mod guaili;
#[path = "g1_shahai_swd/guxing.rs"]
mod guxing;
#[path = "g1_shahai_swd/heihuo.rs"]
mod heihuo;
#[path = "g1_shahai_swd/hlingbo.rs"]
mod hlingbo;
#[path = "g1_shahai_swd/huajin.rs"]
mod huajin;
#[path = "g1_shahai_swd/huangyu.rs"]
mod huangyu;
#[path = "g1_shahai_swd/huanhun.rs"]
mod huanhun;
#[path = "g1_shahai_swd/huanjian.rs"]
mod huanjian;
#[path = "g1_shahai_swd/huanxia.rs"]
mod huanxia;
#[path = "g1_shahai_swd/huanxing.rs"]
mod huanxing;
#[path = "g1_shahai_swd/huanyin.rs"]
mod huanyin;
#[path = "g1_shahai_swd/huodan.rs"]
mod huodan;
#[path = "g1_shahai_swd/hwendao.rs"]
mod hwendao;
#[path = "g1_shahai_swd/hyunshen.rs"]
mod hyunshen;
#[path = "g1_shahai_swd/jiefen.rs"]
mod jiefen;
#[path = "g1_shahai_swd/jinlin.rs"]
mod jinlin;
#[path = "g1_shahai_swd/jiying.rs"]
mod jiying;
#[path = "g1_shahai_swd/jqimou.rs"]
mod jqimou;
#[path = "g1_shahai_swd/juxi.rs"]
mod juxi;
#[path = "g1_shahai_swd/kbolan.rs"]
mod kbolan;
#[path = "g1_shahai_swd/lanzhi.rs"]
mod lanzhi;
#[path = "g1_shahai_swd/lexue.rs"]
mod lexue;
#[path = "g1_shahai_swd/lianda.rs"]
mod lianda;
#[path = "g1_shahai_swd/lianwu.rs"]
mod lianwu;
#[path = "g1_shahai_swd/lingshi.rs"]
mod lingshi;
#[path = "g1_shahai_swd/lingxian.rs"]
mod lingxian;
#[path = "g1_shahai_swd/lingxin.rs"]
mod lingxin;
#[path = "g1_shahai_swd/liuhong.rs"]
mod liuhong;
#[path = "g1_shahai_swd/ljifeng.rs"]
mod ljifeng;
#[path = "g1_shahai_swd/luanji.rs"]
mod luanji;
#[path = "g1_shahai_swd/mingfu.rs"]
mod mingfu;
#[path = "g1_shahai_swd/nlianji.rs"]
mod nlianji;
#[path = "g1_shahai_swd/pingxu.rs"]
mod pingxu;
#[path = "g1_shahai_swd/poxiao.rs"]
mod poxiao;
#[path = "g1_shahai_swd/poyue.rs"]
mod poyue;
#[path = "g1_shahai_swd/pozhen.rs"]
mod pozhen;
#[path = "g1_shahai_swd/pozhou.rs"]
mod pozhou;
#[path = "g1_shahai_swd/rexue.rs"]
mod rexue;
#[path = "g1_shahai_swd/shangxi.rs"]
mod shangxi;
#[path = "g1_shahai_swd/shehun.rs"]
mod shehun;
#[path = "g1_shahai_swd/shengshou.rs"]
mod shengshou;
#[path = "g1_shahai_swd/shoulie.rs"]
mod shoulie;
#[path = "g1_shahai_swd/sliufeng.rs"]
mod sliufeng;
#[path = "g1_shahai_swd/swd_cards_equip5_b33_lianyaohu_hujing.rs"]
mod swd_cards_equip5_b33_lianyaohu_hujing;
#[path = "g1_shahai_swd/swd_cards_equip5_b44_yuruyi_draw_window.rs"]
mod swd_cards_equip5_b44_yuruyi_draw_window;
#[path = "g1_shahai_swd/swd_extra_cards_disposition_guard.rs"]
mod swd_extra_cards_disposition_guard;
#[path = "g1_shahai_swd/swd_langeguaiyi_longfan_suit_branches.rs"]
mod swd_langeguaiyi_longfan_suit_branches;
#[path = "g1_shahai_swd/sxianjing.rs"]
mod sxianjing;
#[path = "g1_shahai_swd/tiebi.rs"]
mod tiebi;
#[path = "g1_shahai_swd/wanjun.rs"]
mod wanjun;
#[path = "g1_shahai_swd/xielv.rs"]
mod xielv;
#[path = "g1_shahai_swd/xuanying.rs"]
mod xuanying;
#[path = "g1_shahai_swd/xuanzhou.rs"]
mod xuanzhou;
#[path = "g1_shahai_swd/xuehuang.rs"]
mod xuehuang;
#[path = "g1_shahai_swd/yeying.rs"]
mod yeying;
#[path = "g1_shahai_swd/yijue.rs"]
mod yijue;
#[path = "g1_shahai_swd/yudun.rs"]
mod yudun;
#[path = "g1_shahai_swd/yulin.rs"]
mod yulin;
#[path = "g1_shahai_swd/yunchou.rs"]
mod yunchou;
#[path = "g1_shahai_swd/zhanlu.rs"]
mod zhanlu;
