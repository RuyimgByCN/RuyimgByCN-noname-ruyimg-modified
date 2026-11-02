//! A0 注册表不变量检查（演进提案 §3.A0 第 1/4 点，挂靠主稿 P0 N11）。
//!
//! 1. `MachineSnapshot` 每个变体构造样例，字节编码后机器 tag 互不相同，
//!    且编码→解码往返相等。变体清单经 `variant_index` 的 match 穷尽性
//!    锁定：新增变体而漏登记样例时编译失败；tag 冲突/漏 tag 时本测试
//!    失败（历史实例：tag 29 曾同时分配给 Dismantle 与 XiaoJi，枭姬分支
//!    不可达，2026-09-24 修复为 XiaoJi=43）。
//! 2. `HostBehavior::tag()` 全变体唯一（标签与键一一对应）。
//! 3. `CardKind` 编码双射（archive.rs 的 `card_kind_to_num`/`num_to_card_kind`
//!    为私有函数，双射以本文件同轮的静态核查记录进证据，不在此重复）。
//! 4. 扩展包数据键表（ADR 0778）：`CardKind::key`、`HostBehavior::key` 双射，
//!    牌种伴随技能表键唯一且覆盖固有装备技。

use std::collections::BTreeMap;

use engine_native::archive::{decode_archive, encode_archive};
use engine_native::content::host::HostBehavior;
use engine_native::event::{DamageData, DamageNature, EventData};
use engine_native::resolution::Slot;
use engine_native::snap::{
    FanJianStageSnap, GuanXingStageSnap, JiJiStageSnap, JieYinStageSnap, JijiangStageSnap,
    JudgementProgressSnap, KuRouStageSnap, LiJianStageSnap, MachineSnapshot, ParkSnapshot,
    QingNangStageSnap, RendeStageSnap, SanCaiStageSnap, StateSnapshot, TuXiStageSnap,
    YiShangStageSnap, ZhiHengStageSnap,
};
use engine_native::{Engine, SkillDefId};

/// 每变体一个样例（名称仅用于报错）。字段取最小合法值，嵌套 inner 用
/// `Some(Box::new(Noop))` 验证嵌套编码；OptionalGate 的事件载荷用伤害。
fn all_samples() -> Vec<(&'static str, MachineSnapshot)> {
    let damage = EventData::Damage(DamageData {
        settlement: Default::default(),
        source: Some(1),
        target: 0,
        amount: 1,
        nature: DamageNature::Normal,
        prevented: false,
        redirect_chain: Vec::new(),
        card: None,
        from_chain: false,
    });
    vec![("JudgeContinuation",MachineSnapshot::JudgeContinuation{owner:0,queue:vec![],current:None,ordinary:None,started:false,payment:None}),("NativeInvocation",MachineSnapshot::NativeInvocation{owner:0,def:0,window:engine_native::event::DamageWindowKind::End,progress:engine_native::event::NativeInvocationProgress::Ready}),
        ("DamageFlow", MachineSnapshot::DamageFlow { root: 1 }),
        (
            "NativeDamageSkill",
            MachineSnapshot::NativeDamageSkill {
                owner: 0,
                def: 0,
                window: engine_native::event::DamageWindowKind::Before,
            },
        ),
        (
            "Sha",
            MachineSnapshot::Sha {
                card: 1,
                source: 0,
                target: 1,
                nature: DamageNature::Normal,
                asked: true,
                armor_judged: false,
                dodges_used: 1,
                tieji_red: Some(true),
                liuli_done: true,
                liuli_stage: 2,
                liuli_card: None,
                hujia_queue: Some(vec![2]),
                hujia_holder: Some(2),
                jiu_bonus: true,
                asking_xiandan: false,
                xiandan_done: false,
                xiandan_bonus: false,
                asking_lueying: false,
                lueying_victim: None,
                lueying_declined: false,
                asking_fuchou: false,
                fuchou_card: None,
                asking_sadeng: true,
                asking_sadeng_shan: false,
                shoulie_direct: false,
                asking_qilin: false,
                qilin_horse: None,
                asking_hanbing: false,
                asking_fengxueren: false,
                asking_guanshi: false,
                guanshi_cards: Vec::new(),
                asking_qinglong: false,
                asking_cixiong: false,
                cixiong_done: false,
                extra_targets: Vec::new(),
                hanbing_pending: 2,
            },
        ),
        (
            "DyingSave",
            MachineSnapshot::DyingSave {
                dying: 0,
                source: Some(1),
                responders: vec![1, 2],
                asked: true,
                round_used: false,
                guihan_asked: false,
            },
        ),
        (
            "Duel",
            MachineSnapshot::Duel {
                card: 1,
                source: 0,
                target: 1,
                on_target: true,
                asked: true,
                second_pending: false,
            },
        ),
        (
            "NullifyWindow",
            MachineSnapshot::NullifyWindow {
                card: 1,
                source: 0,
                nullified: false,
                queue: vec![1],
                asked: true,
                window_open: true,
                owns_disposal: true,
                effect: Some(Box::new(MachineSnapshot::Noop)),
            },
        ),
        (
            "JudgePhase",
            MachineSnapshot::JudgePhase {
                owner: 0,
                queue: vec![1],
                current: Some(Box::new(JudgementProgressSnap {
                    trick: Some(1),
                    judge_card: 2,
                    rewriters: vec![1],
                    asked: true,
                    tian_du: vec![2],
                    gained: false,
                    finalized: true,
                    in_rewrite: false,
                    judge_held: false,
                    haotian_holder: Some(1),
                    haotian_other: Some(3),
                    gaiming_cards: None,
                })),
            },
        ),
        ("Noop", MachineSnapshot::Noop),
        (
            "PlaceDelayed",
            MachineSnapshot::PlaceDelayed {
                card: 1,
                target: 1,
                placed: true,
            },
        ),
        (
            "Aoe",
            MachineSnapshot::Aoe {
                card: 1,
                source: 0,
                need: engine_native::CardKind::SavageAttack,
                targets: vec![1],
                idx: 0,
                inner: Some(Box::new(MachineSnapshot::Noop)),
                nature: engine_native::event::DamageNature::Normal,
            },
        ),
        (
            "AoeTarget",
            MachineSnapshot::AoeTarget {
                source: 0,
                target: 1,
                need: engine_native::CardKind::ArcheryAttack,
                asked: true,
                card: 1,
                nature: engine_native::event::DamageNature::Normal,
            },
        ),
        (
            "Chain",
            MachineSnapshot::Chain {
                card: 1,
                source: 0,
                targets: vec![1, 2],
                idx: 1,
                inner: Some(Box::new(MachineSnapshot::Noop)),
                dispatcher: true,
            },
        ),
        (
            "DiscardToLimit",
            MachineSnapshot::DiscardToLimit {
                player: 0,
                need: 1,
                asked: true,
            },
        ),
        (
            "Death",
            MachineSnapshot::Death {
                player: 1,
                killer: Some(0),
            },
        ),
        (
            "OptionalGate",
            MachineSnapshot::OptionalGate {
                owner: 0,
                def: SkillDefId::MAX,
                event: damage,
                asked: true,
                inner: Some(Box::new(MachineSnapshot::Noop)),
            },
        ),
        (
            "SanCai",
            MachineSnapshot::SanCai {
                owner: 0,
                stage: SanCaiStageSnap::Draw,
                target: Some(1),
            },
        ),
        (
            "YiShang",
            MachineSnapshot::YiShang {
                owner: 0,
                stage: YiShangStageSnap::Target,
            },
        ),
        ("SuiDun", MachineSnapshot::SuiDun),
        ("SuQing", MachineSnapshot::SuQing { target: 1 }),
        ("ShouYi", MachineSnapshot::ShouYi { owner: 0 }),
        (
            "FanKui",
            MachineSnapshot::FanKui {
                owner: 0,
                source: Some(1),
                asked: true,
            },
        ),
        (
            "JianXiong",
            MachineSnapshot::JianXiong {
                owner: 0,
                card: 1,
                asked: true,
            },
        ),
        ("BiYue", MachineSnapshot::BiYue { owner: 0 }),
        ("JiZhi", MachineSnapshot::JiZhi { owner: 0 }),
        (
            "ZhiHeng",
            MachineSnapshot::ZhiHeng {
                owner: 0,
                stage: ZhiHengStageSnap::Select,
            },
        ),
        (
            "KuRou",
            MachineSnapshot::KuRou {
                owner: 0,
                stage: KuRouStageSnap::Lose,
            },
        ),
        ("LuoShen", MachineSnapshot::LuoShen { owner: 0 }),
        ("YingZi", MachineSnapshot::YingZi { owner: 0 }),
        (
            "Dismantle",
            MachineSnapshot::Dismantle {
                card: 1,
                source: 0,
                target: 1,
                asked: true,
            },
        ),
        (
            "MonkeySteal",
            MachineSnapshot::MonkeySteal {
                card: 1,
                user: 0,
                holders: vec![1],
                asked: true,
            },
        ),
        ("XiaoJi", MachineSnapshot::XiaoJi { owner: 0 }),
        (
            "QingNang",
            MachineSnapshot::QingNang {
                owner: 0,
                stage: QingNangStageSnap::TargetAsk,
                healed: Some(1),
            },
        ),
        (
            "JiJi",
            MachineSnapshot::JiJi {
                owner: 0,
                stage: JiJiStageSnap::Draw,
                target: Some(1),
                given: 1,
            },
        ),
        (
            "GangLie",
            MachineSnapshot::GangLie {
                owner: 0,
                source: 1,
                asked: true,
            },
        ),
        (
            "TuXi",
            MachineSnapshot::TuXi {
                owner: 0,
                stage: TuXiStageSnap::Ask,
            },
        ),
        ("LuoYi", MachineSnapshot::LuoYi { owner: 0 }),
        (
            "GuanXing",
            MachineSnapshot::GuanXing {
                owner: 0,
                stage: GuanXingStageSnap::Arrange,
                top: vec![1, 2],
            },
        ),
        ("LianYing", MachineSnapshot::LianYing { owner: 0 }),
        (
            "Rende",
            MachineSnapshot::Rende {
                owner: 0,
                stage: RendeStageSnap::SelectCardsAsk,
                cards: vec![1],
            },
        ),
        (
            "Jijiang",
            MachineSnapshot::Jijiang {
                owner: 0,
                stage: JijiangStageSnap::HelperAsk,
                queue: Some(vec![1]),
                helper: Some(1),
                sha: Some(1),
                inner: Some(Box::new(MachineSnapshot::Noop)),
            },
        ),
        (
            "Snatch",
            MachineSnapshot::Snatch {
                card: 1,
                source: 0,
                target: 1,
                asked: true,
            },
        ),
        (
            "JieYin",
            MachineSnapshot::JieYin {
                owner: 0,
                stage: JieYinStageSnap::CardsAsk,
                cards: vec![1],
            },
        ),
        (
            "FanJian",
            MachineSnapshot::FanJian {
                owner: 0,
                stage: FanJianStageSnap::TargetAsk,
                target: Some(1),
                chosen_suit: None,
            },
        ),
        (
            "LiJian",
            MachineSnapshot::LiJian {
                owner: 0,
                stage: LiJianStageSnap::DiscardAsk,
                card: Some(1),
                inner: Some(Box::new(MachineSnapshot::Noop)),
            },
        ),
        (
            "Host",
            MachineSnapshot::Host {
                def_key: String::from("t:demo"),
                behavior: 1,
                stage: 0,
                owner: 0,
                target: Some(1),
                card: Some(1),
                cards: Vec::new(),
                targets: Vec::new(),
            },
        ),
        (
            "XiaoGuo",
            MachineSnapshot::XiaoGuo {
                owner: 0,
                target: 1,
                stage: engine_native::snap::XiaoGuoStageSnap::Read,
            },
        ),
        (
            "KuangFu",
            MachineSnapshot::KuangFu {
                owner: 0,
                target: 1,
            },
        ),
        (
            "YaoWu",
            MachineSnapshot::YaoWu {
                owner: 0,
                source: 1,
                stage: engine_native::snap::YaoWuStageSnap::Ask,
            },
        ),
        (
            "ShuShen",
            MachineSnapshot::ShuShen {
                owner: 0,
                target: Some(1),
                stage: engine_native::snap::ShuShenStageSnap::Read,
            },
        ),
        ("ShenZhi", MachineSnapshot::ShenZhi { owner: 0 }),
        (
            "XunXun",
            MachineSnapshot::XunXun {
                owner: 0,
                stage: engine_native::snap::XunXunStageSnap::Arrange,
                top: vec![1, 2],
            },
        ),
        ("WangXi", MachineSnapshot::WangXi { owner: 0, other: 1 }),
        (
            "JiJie",
            MachineSnapshot::JiJie {
                owner: 0,
                card: Some(1),
                stage: engine_native::snap::JiJieStageSnap::Read,
            },
        ),
        ("JiYuan", MachineSnapshot::JiYuan { owner: 0 }),
        (
            "WangZun",
            MachineSnapshot::WangZun {
                owner: 0,
                target: 1,
            },
        ),
        (
            "TongJi",
            MachineSnapshot::TongJi {
                owner: 0,
                stage: engine_native::snap::TongJiStageSnap::Ask,
            },
        ),
        ("Wuzhong", MachineSnapshot::Wuzhong { owner: 0 }),
        ("Taoyuan", MachineSnapshot::Taoyuan { owner: 0 }),
        (
            "Wugu",
            MachineSnapshot::Wugu {
                owner: 0,
                cards: vec![1, 2],
                pickers: vec![0, 1],
            },
        ),
        (
            "Jiedao",
            MachineSnapshot::Jiedao {
                owner: 0,
                holder: 1,
                victim: 2,
                stage: engine_native::snap::JiedaoStageSnap::Ask,
            },
        ),
        (
            "Zhangba",
            MachineSnapshot::Zhangba {
                owner: 0,
                cards: vec![1],
                stage: engine_native::snap::ZhangbaStageSnap::Cards,
            },
        ),
        ("BossXiangde", MachineSnapshot::BossXiangde { owner: 0 }),
        ("BossFutai", MachineSnapshot::BossFutai { owner: 0 }),
        ("BossLuanchang", MachineSnapshot::BossLuanchang { owner: 0 }),
        ("BossZhaohuo", MachineSnapshot::BossZhaohuo { owner: 0 }),
        ("Shedu", MachineSnapshot::Shedu { owner: 0 }),
        (
            "BossBuchun",
            MachineSnapshot::BossBuchun {
                owner: 0,
                stage: 2,
                targets: vec![1, 2],
            },
        ),
        (
            "BossXingxia",
            MachineSnapshot::BossXingxia {
                owner: 0,
                stage: 1,
                actor: 2,
            },
        ),
        ("BossQizuo", MachineSnapshot::BossQizuo { owner: 0 }),
        ("BossXiaoxiong", MachineSnapshot::BossXiaoxiong { owner: 0 }),
        ("BossLingbo", MachineSnapshot::BossLingbo { owner: 0 }),
        ("BossTashui", MachineSnapshot::BossTashui { owner: 0 }),
        ("BossZjLeiji", MachineSnapshot::BossZjLeiji { owner: 0 }),
        ("BossHeisheng", MachineSnapshot::BossHeisheng { owner: 0 }),
        (
            "BossLeizhu",
            MachineSnapshot::BossLeizhu {
                owner: 0,
                emitted: 2,
            },
        ),
        (
            "BossRenao",
            MachineSnapshot::BossRenao {
                owner: 0,
                emitted: 1,
            },
        ),
        ("BossWanghun", MachineSnapshot::BossWanghun { owner: 0 }),
        (
            "BossZhuxin",
            MachineSnapshot::BossZhuxin {
                owner: 0,
                emitted: 1,
            },
        ),
        (
            "BossPingdeng",
            MachineSnapshot::BossPingdeng {
                owner: 0,
                emitted: 1,
            },
        ),
        (
            "BossHuoxing",
            MachineSnapshot::BossHuoxing {
                owner: 0,
                emitted: 1,
            },
        ),
        ("BossBingfeng", MachineSnapshot::BossBingfeng { owner: 0 }),
        ("BossWangxiang", MachineSnapshot::BossWangxiang { owner: 0 }),
        (
            "Rvm",
            MachineSnapshot::Rvm {
                owner: 0,
                module: Vec::new(),
                paused: None,
                skill_name: "a0_rvm".to_string(),
                event: engine_native::EventData::Phase {
                    player: 0,
                    phase: engine_native::Phase::End,
                    begin: true,
                },
                consume: Vec::new(),
                count_hits: false,
                commit_pending: Vec::new(),
                commit_pick: None,
                following: Vec::new(),
                commit_state: Vec::new(),
                committing: false,
            },
        ),
        (
            "TriggerOrder",
            MachineSnapshot::TriggerOrder {
                frame: 1,
                owner: 0,
                candidates: vec![1, 2],
                asked: true,
                cursor: 1,
                done: vec![(1, 3)],
                owners: vec![0, 1],
                normal_owners: 2,
            },
        ),
    ]
}

/// match 穷尽性守卫：新增 `MachineSnapshot` 变体而未登记样例时编译失败。
fn variant_index(m: &MachineSnapshot) -> u32 {
    use MachineSnapshot as M;
    match m {
        M::Sha { .. } => 0,
        M::DyingSave { .. } => 1,
        M::Duel { .. } => 2,
        M::NullifyWindow { .. } => 3,
        M::JudgePhase { .. } => 4,
        M::Noop => 5,
        M::PlaceDelayed { .. } => 6,
        M::Aoe { .. } => 7,
        M::AoeTarget { .. } => 8,
        M::Chain { .. } => 9,
        M::DiscardToLimit { .. } => 10,
        M::Death { .. } => 11,
        M::OptionalGate { .. } => 12,
        M::SanCai { .. } => 13,
        M::YiShang { .. } => 14,
        M::SuiDun => 15,
        M::SuQing { .. } => 16,
        M::ShouYi { .. } => 17,
        M::FanKui { .. } => 18,
        M::JianXiong { .. } => 19,
        M::BiYue { .. } => 20,
        M::JiZhi { .. } => 21,
        M::ZhiHeng { .. } => 22,
        M::KuRou { .. } => 23,
        M::LuoShen { .. } => 24,
        M::YingZi { .. } => 25,
        M::Dismantle { .. } => 26,
        M::MonkeySteal { .. } => 27,
        M::XiaoJi { .. } => 28,
        M::QingNang { .. } => 29,
        M::JiJi { .. } => 30,
        M::GangLie { .. } => 31,
        M::TuXi { .. } => 32,
        M::LuoYi { .. } => 33,
        M::GuanXing { .. } => 34,
        M::LianYing { .. } => 35,
        M::Rende { .. } => 36,
        M::Jijiang { .. } => 37,
        M::Snatch { .. } => 38,
        M::JieYin { .. } => 39,
        M::FanJian { .. } => 40,
        M::LiJian { .. } => 41,
        M::Host { .. } => 42,
        M::XiaoGuo { .. } => 43,
        M::KuangFu { .. } => 44,
        M::YaoWu { .. } => 45,
        M::ShuShen { .. } => 46,
        M::ShenZhi { .. } => 47,
        M::XunXun { .. } => 48,
        M::WangXi { .. } => 49,
        M::JiJie { .. } => 50,
        M::JiYuan { .. } => 51,
        M::WangZun { .. } => 52,
        M::TongJi { .. } => 53,
        M::Wuzhong { .. } => 54,
        M::Taoyuan { .. } => 55,
        M::Wugu { .. } => 56,
        M::Jiedao { .. } => 57,
        M::Zhangba { .. } => 58,
        M::BossXiangde { .. } => 59,
        M::BossFutai { .. } => 60,
        M::BossLuanchang { .. } => 61,
        M::BossZhaohuo { .. } => 62,
        M::Shedu { .. } => 63,
        M::BossBuchun { .. } => 64,
        M::BossXingxia { .. } => 65,
        M::BossQizuo { .. } => 66,
        M::BossXiaoxiong { .. } => 67,
        M::BossLingbo { .. } => 68,
        M::BossTashui { .. } => 69,
        M::BossZjLeiji { .. } => 70,
        M::BossHeisheng { .. } => 71,
        M::BossLeizhu { .. } => 72,
        M::BossRenao { .. } => 73,
        M::BossWanghun { .. } => 74,
        M::BossZhuxin { .. } => 75,
        M::BossPingdeng { .. } => 76,
        M::BossHuoxing { .. } => 77,
        M::BossBingfeng { .. } => 78,
        M::BossWangxiang { .. } => 79,
        M::Rvm { .. } => 80,
        M::TriggerOrder { .. } => 81,
        M::DamageFlow { .. } => 82,
        M::JudgeContinuation{..}=>85,
        M::NativeInvocation{..}=>84,
        M::NativeDamageSkill { .. } => 83,
    }
}

/// 单挂起机器的最小快照：真实建局后清空 parks 只留一个目标机器。
fn single_park_snapshot(machine: MachineSnapshot) -> StateSnapshot {
    let mut snap = Engine::new(7, 2).save();
    snap.resolution.parks = vec![ParkSnapshot {
        resume_depth: 0,
        slot: Slot::Intrinsic { frame: 0 },
        machine,
    }];
    snap
}

/// 机器 tag 在编码字节流中的位置：剥离外层帧（22 字节头 + 32 字节
/// blake3 校验和）后在 payload 内用两个已知不同 tag 的机器做最长公共
/// 前缀校准（布局变化时校准断言失败提示修正，不静默错位）。
fn calibrate_machine_tag_pos() -> (Vec<Vec<u8>>, usize) {
    let samples = all_samples();
    let by_name =
        |n: &str| single_park_snapshot(samples.iter().find(|(k, _)| *k == n).unwrap().1.clone());
    fn payload(data: &[u8]) -> &[u8] {
        &data[22..data.len() - 32]
    }
    let noop_enc = encode_archive(&by_name("Noop"));
    let death_enc = encode_archive(&by_name("Death"));
    let (noop, death) = (payload(&noop_enc), payload(&death_enc));
    let pos = noop
        .iter()
        .zip(death.iter())
        .position(|(a, b)| a != b)
        .expect("Noop 与 Death 编码必须不同");
    // 自检：校准位置恰是机器 tag 字节（Noop=5、Death=11，源码 tag 表）。
    assert_eq!(noop[pos], 5, "机器 tag 定位失准（Noop）");
    assert_eq!(death[pos], 11, "机器 tag 定位失准（Death）");
    let all = samples
        .into_iter()
        .map(|(_, m)| encode_archive(&single_park_snapshot(m)))
        .collect();
    (all, pos)
}

#[test]
fn machine_snapshot_tags_unique_and_roundtrip() {
    // 剥离外层帧（22 字节头 + 32 字节 blake3 校验和）后的载荷。
    fn payload(data: &[u8]) -> &[u8] {
        &data[22..data.len() - 32]
    }
    let samples = all_samples();
    let (all_data, tag_pos) = calibrate_machine_tag_pos();
    assert_eq!(all_data.len(), samples.len(), "样例数与编码数不一致");
    assert_eq!(
        samples.len(),
        samples
            .iter()
            .map(|(_, machine)| variant_index(machine))
            .max()
            .unwrap() as usize
            + 1,
        "变体清单不完整"
    );

    let mut tags: BTreeMap<u8, Vec<&str>> = BTreeMap::new();
    for ((name, machine), data) in samples.iter().zip(all_data.iter()) {
        let snap = single_park_snapshot(machine.clone());
        let decoded = decode_archive(data).unwrap_or_else(|e| panic!("{name} 往返解码失败: {e:?}"));
        assert_eq!(&decoded, &snap, "{name} 编码→解码往返不等");
        tags.entry(payload(data)[tag_pos]).or_default().push(name);
    }
    let dups: Vec<String> = tags
        .iter()
        .filter(|(_, v)| v.len() > 1)
        .map(|(t, v)| format!("tag {t}: {v:?}"))
        .collect();
    assert!(
        dups.is_empty(),
        "MachineSnapshot 编码 tag 冲突（解码分支不可达）：\n{}",
        dups.join("\n")
    );
}

#[test]
fn host_behavior_tags_unique() {
    // tag() 本身是 host.rs 内的穷尽 match（新增变体漏登记即编译失败）；这里校验
    // ALL 全变体 tag 唯一、from_tag 往返，且 from_tag 可达变体数恰为 ALL（ALL 漏列也失败）。
    let mut seen = std::collections::BTreeMap::new();
    for b in HostBehavior::ALL {
        let t = b.tag();
        if let Some(prev) = seen.insert(t, b) {
            panic!("HostBehavior tag {t} 冲突：{prev:?} 与 {b:?}");
        }
        assert_eq!(HostBehavior::from_tag(t), Some(b), "tag {t} 往返失败");
    }
    let reachable = (0..=u16::MAX).filter_map(HostBehavior::from_tag).count();
    assert_eq!(
        reachable,
        HostBehavior::ALL.len(),
        "from_tag 可达变体数与 ALL 不符"
    );
}
