//! G1 玩点论杀纵向样本（ADR 0776）：卡牌切片 → 真实包 → 导入 →
//! 宿主行为条件增益 → 内容锁。
//!
//! 切片 14 张实体牌（基础牌 7 + 望梅止渴 2 + 诱敌深入 3 + 三尖 1 +
//! 铁索 1）；暂缓（包数据登记 reason）：调兵遣将×2（多人方向选择多步
//! 结算）。武将批 6 名元数据经角色包管线装载，思继已映射，其余技能登记暂缓。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use engine_native::catalog::CardKind;
use engine_native::content::extension::{ExtensionDef, ExtensionPack};
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase, RejectReason,
};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn options() -> InstallOptions {
    InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

/// 装载（ADR 0777/0778）：包元数据、三尖射程与装备技都写在 cards.json，
/// 由 content-native-loader 统一桥接，测试不含逐包映射。
fn bridge(json: &str) -> ExtensionPack {
    content_native_loader::bridge_pack(json)
        .expect("cards.json 可解析")
        .pack
}

/// 测试脚手架：导入 + 桥接 + 停在 0 号出牌阶段（无挂起、全手牌清空）。
fn setup() -> (Engine, ExtensionPack) {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(discovered
        .iter()
        .any(|d| d.name == "wudian-upstream-noname"));
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    assert!(report.installed.iter().any(|m| m.package_id == "wudian"));
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "wudian")
        .unwrap()
        .clone();
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .expect("cards.json 可读");
    assert!(json.contains("host:wangmeizhike_card"), "宿主映射声明");
    assert!(json.contains("diaobingqianjiang"), "暂缓条目登记");
    let pack = bridge(&json);
    assert_eq!(
        (pack.def.name, pack.def.version),
        ("wudian-upstream-noname", 2),
        "包元数据来自 cards.json"
    );
    assert_eq!(pack.card_count(), 14, "切片 14 张实体牌（+诱敌深入 3）");
    let mut e = Engine::new_with_extensions(3501, 3, &[&pack]).expect("建局");
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    assert_eq!(e.catalog().cards.len(), base_cards + 14);
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let cid = c.id;
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            let obs = c.observer;
            match submit(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            ) {
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                other => panic!("answer: {other:?}"),
            }
            continue;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    (e, pack)
}

fn wangmei_id(e: &Engine, n: usize) -> u32 {
    let mut seen = 0;
    for (i, c) in e.catalog().cards.iter().enumerate() {
        if c.name.starts_with("wd_wangmeizhike") {
            if seen == n {
                return i as u32;
            }
            seen += 1;
        }
    }
    panic!("望梅止渴不足");
}

fn pass_nullify(e: &mut Engine) {
    let c = e.open_choice().expect("无懈窗口");
    let cid = c.id;
    let obs = c.observer;
    assert_eq!(c.spec.prompt, "nullify_window");
    submit(
        e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Pass],
        },
    );
}

fn advance_loop(e: &mut Engine, player: u8) {
    for _ in 0..64 {
        let t = e.turn();
        if t.current == player && t.phase == Phase::Play && e.open_choice().is_none() {
            return;
        }
        if let Some(c) = e.open_choice().cloned() {
            let cid = c.id;
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            let obs = c.observer;
            match submit(
                e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: sel,
                },
            ) {
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                other => panic!("answer: {other:?}"),
            }
            continue;
        }
        let actor = e.turn().current;
        match submit(e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    panic!("no play phase");
}

fn assert_zone_conservation_wudian(e: &Engine) {
    let snap = e.save();
    let mut ids: Vec<u32> = Vec::new();
    for p in &snap.core.players {
        ids.extend(p.hand.iter().copied());
        ids.extend(p.equips.iter().copied());
        ids.extend(p.judge.iter().copied());
    }
    ids.extend(snap.core.deck.iter().copied());
    ids.extend(snap.core.discard.iter().copied());
    ids.extend(snap.core.table.iter().copied());
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), ids.len(), "实体牌重复归属");
}

// ---------------------------------------------------------------------------
// U05 ow 纵切批九（规则 v126）：卢西奥三技——扩音（弃 1 张→乐动目标数
// +2 / 弃 2 张→摸回量 +1）、呼唤（弃 2 张→回复模式）、乐动（出牌阶段
// 结束选目标摸/回）。回合级存储三角（TurnState.yuedong_*，存档 v7）。
// ---------------------------------------------------------------------------

/// 打开挂起选择并以给定选择应答。
fn answer_open(e: &mut Engine, selection: Vec<ChoiceOption>) -> CommandOutcome {
    let c = e.open_choice().cloned().expect("存在挂起选择");
    let (cid, obs) = (c.id, c.observer);
    drop(c);
    submit(
        e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection,
        },
    )
}

/// 暴雪弃牌堆断言辅助：黑牌均在弃牌堆。
fn is_black_card_discarded(e: &Engine, cards: &[u32], black: usize) -> bool {
    let mut n = 0;
    for c in cards {
        let isb = matches!(
            e.catalog().cards[*c as usize].suit,
            engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
        );
        if isb && e.state_core().discard.contains(c) {
            n += 1;
        }
    }
    n == black
}

// ---------------------------------------------------------------------------
// U05 ow 纵切批十一（规则 v128）：路霸（ow_luba）双技——链钩（liangou，
// 出牌阶段限一次：弃 1 张 he+判定非红桃→钩住一名其他角色至回合结束：
// 路霸算到其距离恒 1+其首次受伤 +1）与吸阳（xiyang，回合结束确认：
// 受伤且未翻面→翻面+回复 2）。TurnState 两字段（存档 v8→v9）。
// ---------------------------------------------------------------------------

/// 构造判定牌：把指定花色的一张牌移到牌堆顶（下次 draw_judge_card 摸到）。
fn rig_next_judge(
    e: &Engine,
    snap: &mut engine_native::StateSnapshot,
    want: engine_native::catalog::Suit,
) {
    let pos = (0..snap.core.deck.len())
        .rev()
        .find(|i| e.catalog().cards[snap.core.deck[*i] as usize].suit == want)
        .unwrap_or_else(|| panic!("牌堆中无目标花色"));
    let c = snap.core.deck.remove(pos);
    snap.core.deck.push(c);
}

// ---------------------------------------------------------------------------
// U05 ow 纵切批十二（规则 v129）：源氏（ow_yuanshi）飞刃（feiren，锁定
// 被动）——黑桃杀伤害 +1（非铁索传导，DamageData.from_chain 新字段）+
// 杀无视距离 + 梅花杀额外目标上限 +1；lianpo/zhanlong 登记暂缓。
// ---------------------------------------------------------------------------

/// 从手牌中找指定花色的一张牌。
fn pick_suit(e: &Engine, suit: engine_native::catalog::Suit) -> Option<u32> {
    e.debug_hand(0)
        .into_iter()
        .find(|c| e.catalog().cards[*c as usize].suit == suit)
}

fn hand_len_w(e: &Engine, p: u8) -> usize {
    e.state_core().player(p).hand.len()
}

/// 推进到指定玩家出牌阶段（空/首选应答排空）。
fn advance_to_play(e: &mut Engine, player: u8) {
    for _ in 0..200 {
        if e.turn().current == player && e.turn().phase == Phase::Play {
            return;
        }
        assert!(e.outcome().is_none(), "对局提前结束");
        let actor = e.turn().current;
        submit(e, actor, GameAction::NextPhase);
        if let Some(c) = e.open_choice() {
            let sel: Vec<ChoiceOption> = c
                .spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .take(c.spec.min.max(1))
                .collect();
            let (cid, obs) = (c.id, c.observer);
            if !sel.is_empty() {
                submit(
                    e,
                    obs,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: sel,
                    },
                );
            } else if !c.spec.options.is_empty() {
                submit(
                    e,
                    obs,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: vec![c.spec.options[0].clone()],
                    },
                );
            }
        }
    }
    panic!("200 步未到 p{player} 出牌阶段");
}

/// 可选确认门排空（首选项应答）。
fn settle_optional_confirm(e: &mut Engine) {
    for _ in 0..8 {
        let Some(c) = e.open_choice() else { return };
        let (cid, obs) = (c.id, c.observer);
        if c.spec.options.is_empty() {
            return;
        }
        let sel = vec![c.spec.options[0].clone()];
        submit(
            e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
    }
}

#[path = "g1_wudian/ana.rs"]
mod ana;
#[path = "g1_wudian/anu.rs"]
mod anu;
#[path = "g1_wudian/banzang.rs"]
mod banzang;
#[path = "g1_wudian/baolei.rs"]
mod baolei;
#[path = "g1_wudian/changqing.rs"]
mod changqing;
#[path = "g1_wudian/chanyata.rs"]
mod chanyata;
#[path = "g1_wudian/dva.rs"]
mod dva;
#[path = "g1_wudian/falaozhiying.rs"]
mod falaozhiying;
#[path = "g1_wudian/feishi.rs"]
mod feishi;
#[path = "g1_wudian/g1.rs"]
mod g1;
#[path = "g1_wudian/hanlingsha.rs"]
mod hanlingsha;
#[path = "g1_wudian/hanlong.rs"]
mod hanlong;
#[path = "g1_wudian/heibaihe.rs"]
mod heibaihe;
#[path = "g1_wudian/heiying.rs"]
mod heiying;
#[path = "g1_wudian/jiangcheng.rs"]
mod jiangcheng;
#[path = "g1_wudian/jiangyunfan.rs"]
mod jiangyunfan;
#[path = "g1_wudian/jingtian.rs"]
mod jingtian;
#[path = "g1_wudian/jushifang.rs"]
mod jushifang;
#[path = "g1_wudian/kuangshu.rs"]
mod kuangshu;
#[path = "g1_wudian/laiyinhate.rs"]
mod laiyinhate;
#[path = "g1_wudian/leiyuange.rs"]
mod leiyuange;
#[path = "g1_wudian/linyueru.rs"]
mod linyueru;
#[path = "g1_wudian/liufu.rs"]
mod liufu;
#[path = "g1_wudian/liumengli.rs"]
mod liumengli;
#[path = "g1_wudian/liuyan.rs"]
mod liuyan;
#[path = "g1_wudian/liuzan.rs"]
mod liuzan;
#[path = "g1_wudian/lixiaoyao.rs"]
mod lixiaoyao;
#[path = "g1_wudian/longkui.rs"]
mod longkui;
#[path = "g1_wudian/longyou.rs"]
mod longyou;
#[path = "g1_wudian/luba.rs"]
mod luba;
#[path = "g1_wudian/luozhaoyan.rs"]
mod luozhaoyan;
#[path = "g1_wudian/luxiao.rs"]
mod luxiao;
#[path = "g1_wudian/maikelei.rs"]
mod maikelei;
#[path = "g1_wudian/mei.rs"]
mod mei;
#[path = "g1_wudian/mingxiu.rs"]
mod mingxiu;
#[path = "g1_wudian/moshou.rs"]
mod moshou;
#[path = "g1_wudian/muchanglan.rs"]
mod muchanglan;
#[path = "g1_wudian/murongziying.rs"]
mod murongziying;
#[path = "g1_wudian/nangonghuang.rs"]
mod nangonghuang;
#[path = "g1_wudian/orisa.rs"]
mod orisa;
#[path = "g1_wudian/ow.rs"]
mod ow;
#[path = "g1_wudian/sanjian.rs"]
mod sanjian;
#[path = "g1_wudian/shenqishuang.rs"]
mod shenqishuang;
#[path = "g1_wudian/sishen.rs"]
mod sishen;
#[path = "g1_wudian/sumei.rs"]
mod sumei;
#[path = "g1_wudian/swd.rs"]
mod swd;
#[path = "g1_wudian/tangyurou.rs"]
mod tangyurou;
#[path = "g1_wudian/tianshi.rs"]
mod tianshi;
#[path = "g1_wudian/tianyu.rs"]
mod tianyu;
#[path = "g1_wudian/tuobiang.rs"]
mod tuobiang;
#[path = "g1_wudian/u07.rs"]
mod u07;
#[path = "g1_wudian/wangmeizhike.rs"]
mod wangmeizhike;
#[path = "g1_wudian/wangpengxu.rs"]
mod wangpengxu;
#[path = "g1_wudian/wangxiaohu.rs"]
mod wangxiaohu;
#[path = "g1_wudian/wenhui.rs"]
mod wenhui;
#[path = "g1_wudian/wensidun.rs"]
mod wensidun;
#[path = "g1_wudian/wudian.rs"]
mod wudian;
#[path = "g1_wudian/xia.rs"]
mod xia;
#[path = "g1_wudian/xiahoujinxuan.rs"]
mod xiahoujinxuan;
#[path = "g1_wudian/xianjian.rs"]
mod xianjian;
#[path = "g1_wudian/xianqing.rs"]
mod xianqing;
#[path = "g1_wudian/xiaoman.rs"]
mod xiaoman;
#[path = "g1_wudian/xingxuan.rs"]
mod xingxuan;
#[path = "g1_wudian/xuanxiao.rs"]
mod xuanxiao;
#[path = "g1_wudian/xuejian.rs"]
mod xuejian;
#[path = "g1_wudian/youdishenru.rs"]
mod youdishenru;
#[path = "g1_wudian/yuanshi.rs"]
mod yuanshi;
#[path = "g1_wudian/yuejinzhao.rs"]
mod yuejinzhao;
#[path = "g1_wudian/yueqi.rs"]
mod yueqi;
#[path = "g1_wudian/yuntianhe.rs"]
mod yuntianhe;
#[path = "g1_wudian/zhaliya.rs"]
mod zhaliya;
#[path = "g1_wudian/zhaoliner.rs"]
mod zhaoliner;
#[path = "g1_wudian/zhixuzhiguang.rs"]
mod zhixuzhiguang;
#[path = "g1_wudian/zixuan.rs"]
mod zixuan;
