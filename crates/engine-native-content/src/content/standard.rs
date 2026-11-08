//! 一组覆盖复杂机制的代表技能，用于验证
//! 「只读目录 + 紧凑状态 + 原生技能暂停恢复」链路：
//!
//! | 技能 | 机制覆盖 |
//! |---|---|
//! | 散财 | 可选触发（确认）→ 摸牌子事件 → 连续两个选择（嵌套选择链）|
//! | 移伤 | 伤害前置窗口内改写帧载荷（伤害转移）|
//! | 碎盾 | 每回合限一次的防止型技能 |
//! | 肃清 | 在同一触发窗口内禁用他人技能（结算中途失效）|
//! | 授艺 | 失去体力后动态获得临时技能（回合结束到期）|

use crate::catalog::{
    CardDef, CardKind, Catalog, QueryCtx, SkillBehavior, SkillDef, SkillLimits, SkillMachine, Suit,
    TriggerMeta,
};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::snap::{
    FanJianStageSnap, GuanXingStageSnap, JiJiStageSnap, JiJieStageSnap, JieYinStageSnap,
    JijiangStageSnap, KuRouStageSnap, LiJianStageSnap, MachineSnapshot, QingNangStageSnap,
    RendeStageSnap, SanCaiStageSnap, ShuShenStageSnap, TongJiStageSnap, TuXiStageSnap,
    XiaoGuoStageSnap, XunXunStageSnap, YaoWuStageSnap, YiShangStageSnap, ZhangbaStageSnap,
    ZhiHengStageSnap,
};

/// 标准测试目录的规则版本：目录语义（牌表/技能表/规则机制）变化时递增。
/// v29：护驾/激将逐人询问重构（Sha/Jijiang 机器快照字段与状态编码变化）、
/// 仁德分次累计（TurnState.rende_given 入状态）、主动技阶段门控、
/// 牌归属完整核算与字节存档 v2（ChoiceKind 显式编码）——ADR 0776。
/// v30：G1 cardpile 前置——新增【酒】/【雷杀】牌种（出牌/濒死/杀结算
/// 规则与 TurnState.jiu_used/jiu_active 状态），ADR 0776 G1。
/// v31：G1 规则宿主原型——扩展机器载荷（MachineSnapshot::Host）、
/// 手里剑牌种、宿主技能目录注册。
/// v32：G1 wuxing——五行属性入状态（PlayerState.wunature /
/// core.card_natures）、PickTag/Tag 选择原语、WuxingPan 宝物栏牌种。
/// v33：G1 3D精选——角色包管线（catalog.extension_generals）、
/// 精彀宿主行为、装备技能生命周期（装备授予/离场撤销 + 恢复一致性）。
/// v34：G1 玩点论杀——望梅止渴牌种（CardKind::WangmeiZhike）与
/// 同名宿主行为（最小体力/最小手牌条件增益）。
/// v35：G1 玩点论杀武将批——思继宿主原语（弃杀数×2 摸牌，
/// HostMachine.cards 关联牌组入快照）。
/// v36：G1 玩点论杀原语增补——三尖武器技（弃牌+目标两段选择）
/// 与墨守目标免疫（延时锦囊命令边界过滤）。
/// v37：G1 运筹帷幄原语增补——水攻（多目标弃装备）与釜底抽薪
/// （拼点比较两段选择）；HostMachine.targets 多目标入快照。
/// v38：G1 运筹帷幄切片扩展——趁火打劫（获得目标手牌）与索链甲
/// （防具 + link_on_equip 横置钩子）。
/// v39：G1 运筹帷幄切片扩展——偷梁换柱（换牌双选）与洪水
/// （自-target 延时锦囊判定：梅花 2–9 → 3 点雷伤）。
/// v40：G1 杀海拾遗·炉石子包切片——灵魂之火/照明弹/急耗卡
/// （CardKind 三种 + 宿主行为；shenenshu 等复杂牌暂缓）。
/// v41：G1 玩点论杀余量原语——诱敌深入（杀响应转化：
/// 打出后杀无效，来源受 1 点反噬；登记差异：无连用循环）。
/// v42：G1 运筹帷幄余量原语——隔岸观火（双目标拼点链，
/// 赢方获得输方一张手牌，平局目标一弃牌）。
/// v43：G1 运筹帷幄余量原语——草船借箭（目标分支选择：
/// 交出所有手牌杀并受反噬杀 / 展示手牌并被弃一张）。
/// v44：G1 运筹帷幄收官——舌战群儒（确定性首轮拼点：
/// 与所有有手牌的其他角色各拼一次，大者摸 1；登记差异：
/// 简化上游多轮 Bool 响应链+随机牌池）。
/// v45：G1 杀海拾遗·古剑子包切片——舒筋散（目标弃任意张手牌
/// 摸等量）/面里金针（摸 1 后受 1 点伤害，登记差异：失去体力
/// 简化为伤害）/飞镖（弃黑色手牌或受 1 点伤害，登记差异同前）。
/// v46：G1 古剑续切片——金莲烛（伤害前置事件取消原语：持有者打出
/// 防止此伤害、来源摸 1；登记差异：chooseToUse 选牌简化为确认后
/// 确定性弃置首张）。
/// v47：G1 古剑食物计时状态原语——摊黄冻（出牌阶段结束摸 1）/
/// 有更（阶段开始已受伤且体力最少回 1，登记差异：原生无准备阶段，
/// 挂判定阶段开始）；SkillInstance 增 rounds 倒计时字段（入哈希/
/// 存档），效果触发点扣减、归零移除。
/// v48：古剑食物效果面续补——青团（回合内首杀摸 1，once_per_turn）/
/// 元宝肉（额外一张杀，杀次数校验钩子）/咕咚羹（受杀伤害 -1，一次
/// 性移除）/麻婆豆腐（结束阶段弃牌，登记差异：随机改确定性）/
/// 春饼（手牌上限 +1，弃牌校验钩子）。
/// v49：护甲原语（PlayerState.hujia：伤害 1:1 先行抵扣，入哈希/存档；
/// LogEvent::HujiaAbsorbed）——古剑食物鲤鱼汤（阶段开始体力最少得
/// 1 护甲）/麻茶（黑色牌目标限制——出牌使用路径过滤）/菱角（摸牌
/// 阶段额外摸 1 后弃 1，选择机）。
/// v50：古剑食物簇收官——蜜链藕（红桃手牌当桃：出牌阶段 view-as
/// 转化 + 濒死响应选项并入 is_tao_like）/卤玉羹（出牌阶段限一次
/// 弃基本牌获得牌堆顶；登记差异：上游发现牌随机五选一）。
/// v51：古剑机关簇——冰魄通（逐目标首张同色比对，同色弃+伤害；
/// 登记差异：确定性首张；失去体力为规则 v77 原语）/乾坤镖（目标及邻座
/// 各弃首张，登记差异：随机改确定性）/龙须钩（获得目标首张装备）/
/// 神火飞鸦（目标及邻座需闪否则 1 火伤——逐目标响应循环）。
/// v52：古剑余簇——紫阳丹（+3 护甲，结束阶段 -1 衰减；登记差异：
/// 上游每阶段开始衰减）/玉女元神（濒死一次性回复标记）/石会粉
/// （需闪否则禁用对其他角色用牌至其回合结束；登记差异：上游为
/// 他人出牌阶段响应使用，原生为主动使用）/六风散（获得两张实体
/// 闪；登记差异：虚拟牌生成改实体牌获得）。
/// v53：古剑最后一批——砂糖（火伤+护甲）/黑龙鳞片（+1 护甲；登记
/// 差异：防御距离临时技暂缓）/硫头阁（自己摸 1/他人弃首张；登记
/// 差异：随机效果池简化为确定性）/木头面具（装备技：出牌阶段限一次
/// 任意手牌当杀——view-as 装备技+once_used）。
/// v54：古剑收官双卡——帝女玄霜（濒死特殊牌：is_tao_like 按桃
/// 救回 + 主动使用回复+rider 弃任意摸等量；登记差异：濒死救援
/// 骑乘暂缓）/玉衡（主动装备技确定性取黑桃；登记差异：升级机制
/// +随机敌目标暂缓）。
/// v55：炉石余量 6 种 9 张——盾牌格挡（+1 护甲摸 1 弃 1，登记差异：
/// 确定性首张）/治疗波（判定红回 1 否则+1 护甲）/元素毁灭（逐目标
/// 弃 1–2 每少弃一张受 1 雷伤）/星界沟通（体力上限+1 回 1 弃全部
/// 手牌）/传送门（获得牌堆顶；登记差异：立即使用链暂缓）/坦蛇至人
/// （座位序链式弃首张；登记差异：随机改确定性）。
/// v56：炉石收官——世界树（牌堆自顶搜索两张不同槽位装备，目标
/// 依次穿戴，同槽旧装备替换进弃牌堆并撤销/授予装备技）。
/// v57：gwent 首切片 5 种 9 张——燕子药水（最小手牌数摸 2 否则 1）/
/// 奥祖之雷（1 雷伤后摸 1）/瘟疫（弃首张或受伤；登记差异：随机改
/// 首张、失去体力为规则 v77 原语）/置换军（反复失去体力上限）/暴雪药水
/// （弃前两张摸 1；登记差异：确定性前两张）。
/// v58：gwent 天气状态群——倾盆大雨（手牌上限 -1 至弃牌阶段结束，
/// 手牌上限钩子读取）/碧日浓雾（不能使用杀至出牌阶段结束，杀次数
/// 校验钩子读取）/刺骨寒霜（摸牌阶段结束弃首张；登记差异：等价净
/// -1）——目标+距离 1 邻座，授予机跳过已持有者。
/// v59：gwent 收官 3 种 4 张——昆恩法印（防止普通伤害，按防止次数
/// 计数，初始 min(5, 存活数)；登记差异：上游按回合计数）/山崩术
/// （弃其他角色前两件装备；登记差异：随机改座位序前二）/破晓 2
/// （分支：清除天气+判定区牌 / 获得牌堆首张天气法术；登记差异：
/// 随机改牌堆序首张）。
/// v60：gwent 收官——祖墨手铠（封印：目标技能全部失效至其任意阶段
/// 结束；登记差异：原生无锁定技区分，全部技能失效）。
/// v61：E03 翻面原语——PlayerState 朝向状态（face_down）与
/// TurnOverBefore/TurnOvered 事件对；欢乐卡牌【漩涡面具】（装备者武将牌
/// 不能被翻面，TurnOverBefore 强制取消）。
/// v324：B7——rule-vm 挂起机快照补齐声明技能名、触发事件原文、限次
/// 清单与命中计数旗标（归档尾部追加并进入规范摘要）。此前恢复从模块
/// 名重建技能名并把触发语境/限次/命中重置为默认，续行的 STORAGE_SET
/// 与 once/hits 键、故障身份偏离未中断执行。旧（v323 及更早）存档按
/// 版本策略拒绝：普通档 RuleVersionMismatch；含旧格式 Rvm 机的档在
/// 解码期即失败（不能可靠恢复）。存档格式 v105 与全局原生 ABI 不变。
/// v325：NC07——炼妖壶壶仓 `lianyaohu_store` 顺序进入规范摘要（紧随
/// 筑城牌区，无条件写入长度与牌序）。此前两种壶序的存档摘要相同，但炼
/// 化选项与丢弃序按壶序生成（GT-012 反例）。v324 及更早存档按版本策略
/// 拒绝（RuleVersionMismatch）；存档格式 v105 与全局原生 ABI 753 不变。
/// v326：NC04——`cards_given` 给牌触发帧改为**每次给牌动作一帧、携带全部牌**
/// （`give_hand_cards` 批量原语；仁德/蝶舞一次给 N 张只触发监听技一次，与
/// 旧线 `give_cards` 单事件同口径；此前按张发帧触发 N 次）。同时追认
/// `d7b46ea6` 新帧使事件 id 计数进入摘要域而未升号的位移。v325 及更早存档
/// 按版本策略拒绝（RuleVersionMismatch）；存档格式 v105、全局原生 ABI 753、
/// CardsLost/CardsMoved 逐张口径不变。
/// v327：NC07（裁决第 11 项）——字段审计所列 10 个「入档未入摘要」字段全部
/// 入摘要：PlayerState 的 `jiguanyuan_cards`／`jileng_active`／`chaofeng_active`
/// ／`xingtigaizao_count`／`yinmi_active`／`xuehuang_used`／`xuejibingbao`
/// ／`nigong_lv`／`card_mods` 与 TurnState 的 `minwan_targets`（编码序与
/// state_encode.rs 一致）。此前只改这些语义字段的两份存档摘要相同（GT-012
/// 逐字段反例）；排除名单为空（十个字段均在规则路径被读取）。结构守卫
/// `nc07_archive_hash_guard` 要求入档字段要么入摘要、要么登记排除理由。
/// v326 及更早存档按版本策略拒绝（RuleVersionMismatch）；存档格式 v105 与
/// 全局原生 ABI 753 不变。
/// v328：ADR 0786 J1（裁决第 13 项）——VM 提交期选择窗挂起：`Halted` 后逐
/// 效果落账遇 `discard_hand_equip_count(N>0)` 不再内联「确定性首 N」近似，
/// 而是暂停落账、向开窗座位发真实选牌窗（恰好 N 张手牌/装备，不足以可用
/// 数封顶、无牌空过），应答后按所选弃置并继续提交剩余队列。选择作为普通
/// Answer 命令进入命令流（可回放、跨端一致）；`MachineSnapshot::Rvm` 尾部
/// 追加 `commit_pending`/`commit_pick`（归档与规范摘要同步扩展）。方案①：
/// 不新增 DSL 原语、`RULE_VM_ABI_VERSION` 不变、N=0 首发拒绝照旧。v327 及
/// 更早存档按版本策略拒绝；存档格式 v105、全局原生 ABI 753、Worker／WS
/// 协议不变。
/// v329: ADR0786 J2 active bridge and content-owned effect choices (ABI758); v328 archives reject.
/// Archive v105 keeps the existing Rvm tail mechanism and command/choice variants.
/// v330: ADR0787 explicit host timeout answers; v329 archives reject.
/// v331: ADR0788 player trigger ordering, shared Damage window, and content schema 5 (NGSA 106 / WS7).
/// v332: GaiMing cancel returns candidate card to draw pile before re-drawing (Decision 24).
/// v333: R05 tiqi/baoshu/dcbihuo compounds, real TurnBegin/Phase::Prepare/TurnPhaseAfter windows,
/// async D/R/G draw/gain family with detached_cards, and VerifiedLockedContent raw-byte lock chain
/// (NGSA 107 / ABI 759 / WS8; Decisions 28/29/31).
pub const STANDARD_RULESET_VERSION: u64 = 333;

pub use crate::vocabulary::generals::*;
pub use crate::vocabulary::names::*;
mod skills;
use skills::*;
pub(crate) use skills::{LeizhuMachine, RenaoMachine};
mod boss_skills;
mod cards;
mod standard_skills;

/// 由内容机器快照重建机器实例（`MachineSnapshot::restore` 的内容侧分派）。
/// 返回 None 表示快照不属于本目录的机器类型。
pub fn restore_content_machine(
    s: &MachineSnapshot,
    catalog: &Catalog,
) -> Option<Box<dyn SkillMachine>> {
    match s {
        MachineSnapshot::FanKui {
            owner,
            source,
            asked,
        } => Some(Box::new(FanKuiMachine {
            owner: *owner,
            source: *source,
            asked: *asked,
        })),
        MachineSnapshot::JianXiong { owner, card, asked } => Some(Box::new(JianXiongMachine {
            owner: *owner,
            card: *card,
            asked: *asked,
        })),
        MachineSnapshot::BiYue { owner } => Some(Box::new(BiYueMachine { owner: *owner })),
        MachineSnapshot::BossBuchun {
            owner,
            stage,
            targets,
        } => Some(Box::new(BuchunMachine {
            owner: *owner,
            stage: *stage,
            targets: targets.clone(),
        })),
        MachineSnapshot::BossXingxia {
            owner,
            stage,
            actor,
        } => Some(Box::new(XingxiaMachine {
            owner: *owner,
            stage: *stage,
            actor: *actor,
        })),
        MachineSnapshot::JiZhi { owner } => Some(Box::new(JiZhiMachine { owner: *owner })),
        MachineSnapshot::XiaoJi { owner } => Some(Box::new(XiaoJiMachine { owner: *owner })),
        MachineSnapshot::QingNang {
            owner,
            stage,
            healed,
        } => Some(Box::new(QingNangMachine {
            owner: *owner,
            stage: match stage {
                QingNangStageSnap::TargetAsk => QingNangStage::TargetAsk,
                QingNangStageSnap::TargetRead => QingNangStage::TargetRead,
                QingNangStageSnap::CardRead => QingNangStage::CardRead,
            },
            healed: *healed,
        })),
        MachineSnapshot::KuRou { owner, stage } => Some(Box::new(KuRouMachine {
            owner: *owner,
            stage: match stage {
                KuRouStageSnap::Lose => KuRouStage::Lose,
                KuRouStageSnap::Draw => KuRouStage::Draw,
            },
        })),
        MachineSnapshot::ZhiHeng { owner, stage } => Some(Box::new(ZhiHengMachine {
            owner: *owner,
            stage: match stage {
                ZhiHengStageSnap::Select => ZhiHengStage::Select,
                ZhiHengStageSnap::Draw => ZhiHengStage::Draw,
            },
        })),
        MachineSnapshot::Dismantle {
            card,
            source,
            target,
            asked,
        } => Some(Box::new(crate::kernel::DismantleMachine {
            card: *card,
            source: *source,
            target: *target,
            asked: *asked,
        })),
        MachineSnapshot::MonkeySteal {
            card,
            user,
            holders,
            asked,
        } => Some(Box::new(crate::kernel::MonkeyStealMachine {
            card: *card,
            user: *user,
            holders: holders.clone(),
            asked: *asked,
        })),
        MachineSnapshot::LuoShen { owner } => Some(Box::new(LuoShenMachine { owner: *owner })),
        MachineSnapshot::YingZi { owner } => Some(Box::new(YingZiMachine { owner: *owner })),
        MachineSnapshot::JiJi {
            owner,
            stage,
            target,
            given,
        } => Some(Box::new(JiJiMachine {
            owner: *owner,
            stage: match stage {
                JiJiStageSnap::Draw => JiJiStage::Draw,
                JiJiStageSnap::Target => JiJiStage::Target,
                JiJiStageSnap::Retarget => JiJiStage::Target,
                JiJiStageSnap::GiveCard => JiJiStage::GiveCard,
                JiJiStageSnap::GiveChooseCard => JiJiStage::GiveChooseCard,
            },
            target: *target,
            given: *given,
        })),
        MachineSnapshot::SanCai {
            owner,
            stage,
            target,
        } => Some(Box::new(SanCaiMachine {
            owner: *owner,
            stage: match stage {
                SanCaiStageSnap::Draw => SanCaiStage::Draw,
                SanCaiStageSnap::Target => SanCaiStage::Target,
                SanCaiStageSnap::GiveCard => SanCaiStage::GiveCard,
                SanCaiStageSnap::Finish => SanCaiStage::Finish,
            },
            target: *target,
        })),
        MachineSnapshot::YiShang { owner, stage } => Some(Box::new(YiShangMachine {
            owner: *owner,
            stage: match stage {
                YiShangStageSnap::Target => YiShangStage::Target,
                YiShangStageSnap::Apply => YiShangStage::Apply,
            },
        })),
        MachineSnapshot::SuiDun => Some(Box::new(SuiDunMachine)),
        MachineSnapshot::SuQing { target } => Some(Box::new(SuQingMachine { target: *target })),
        MachineSnapshot::ShouYi { owner } => Some(Box::new(ShouYiMachine { owner: *owner })),
        MachineSnapshot::GangLie {
            owner,
            source,
            asked,
        } => Some(Box::new(GangLieMachine {
            owner: *owner,
            source: *source,
            asked: *asked,
        })),
        MachineSnapshot::TuXi { owner, stage } => Some(Box::new(TuXiMachine {
            owner: *owner,
            stage: match stage {
                TuXiStageSnap::Ask => TuXiStage::Ask,
                TuXiStageSnap::Read => TuXiStage::Read,
            },
        })),
        MachineSnapshot::LuoYi { owner } => Some(Box::new(LuoYiMachine {
            owner: *owner,
            asked: false,
        })),
        MachineSnapshot::GuanXing { owner, stage, top } => Some(Box::new(GuanXingMachine {
            owner: *owner,
            stage: match stage {
                GuanXingStageSnap::Arrange => GuanXingStage::Arrange,
                GuanXingStageSnap::Place => GuanXingStage::Place,
            },
            top: top.clone(),
        })),
        MachineSnapshot::XunXun { owner, stage, top } => Some(Box::new(XunXunMachine {
            owner: *owner,
            stage: match stage {
                XunXunStageSnap::Undo => XunXunStage::Undo,
                XunXunStageSnap::Arrange => XunXunStage::Arrange,
            },
            top: top.clone(),
        })),
        MachineSnapshot::JiJie { owner, stage, card } => Some(Box::new(JiJieMachine {
            owner: *owner,
            stage: match stage {
                JiJieStageSnap::Ask => JiJieStage::Ask,
                JiJieStageSnap::Read => JiJieStage::Read,
            },
            card: *card,
        })),
        MachineSnapshot::LianYing { owner } => Some(Box::new(LianYingMachine {
            owner: *owner,
            asked: false,
        })),
        MachineSnapshot::JieYin {
            owner,
            stage,
            cards,
        } => Some(Box::new(JieYinMachine {
            owner: *owner,
            stage: match stage {
                JieYinStageSnap::CardsAsk => JieYinStage::CardsAsk,
                JieYinStageSnap::TargetAsk => JieYinStage::TargetAsk,
                JieYinStageSnap::TargetRead => JieYinStage::TargetRead,
            },
            cards: cards.clone(),
        })),
        MachineSnapshot::FanJian {
            owner,
            stage,
            target,
            chosen_suit,
        } => Some(Box::new(FanJianMachine {
            owner: *owner,
            stage: match stage {
                FanJianStageSnap::TargetAsk => FanJianStage::TargetAsk,
                FanJianStageSnap::SuitAsk => FanJianStage::SuitAsk,
                FanJianStageSnap::PickAsk => FanJianStage::PickAsk,
                FanJianStageSnap::Consume => FanJianStage::Consume,
            },
            target: *target,
            chosen_suit: *chosen_suit,
        })),
        MachineSnapshot::LiJian {
            owner,
            stage,
            card,
            inner,
        } => Some(Box::new(LiJianMachine {
            owner: *owner,
            stage: match stage {
                LiJianStageSnap::DiscardAsk => LiJianStage::DiscardAsk,
                LiJianStageSnap::TargetsAsk => LiJianStage::TargetsAsk,
                LiJianStageSnap::Launch => LiJianStage::Launch,
            },
            card: *card,
            inner: match inner {
                Some(m) => match m.restore(catalog) {
                    Ok(r) => Some(r),
                    Err(_) => return None,
                },
                None => None,
            },
        })),
        MachineSnapshot::Rende {
            owner,
            stage,
            cards,
        } => Some(Box::new(RendeMachine {
            owner: *owner,
            stage: match stage {
                RendeStageSnap::SelectCardsAsk => RendeStage::SelectCardsAsk,
                RendeStageSnap::TargetAsk => RendeStage::TargetAsk,
                RendeStageSnap::TargetRead => RendeStage::TargetRead,
            },
            cards: cards.clone(),
        })),
        MachineSnapshot::Jijiang {
            owner,
            stage,
            queue,
            helper,
            sha,
            inner,
        } => Some(Box::new(JijiangMachine {
            owner: *owner,
            stage: match stage {
                JijiangStageSnap::HelperAsk => JijiangStage::HelperAsk,
                JijiangStageSnap::TargetAsk => JijiangStage::TargetAsk,
            },
            queue: queue.clone(),
            helper: *helper,
            sha: *sha,
            inner: match inner {
                Some(m) => match m.restore(catalog) {
                    Ok(r) => Some(r),
                    Err(_) => return None,
                },
                None => None,
            },
        })),
        MachineSnapshot::Snatch {
            card,
            source,
            target,
            asked,
        } => Some(Box::new(crate::kernel::SnatchMachine {
            card: *card,
            source: *source,
            target: *target,
            asked: *asked,
        })),
        _ => None,
    }
}

/// 标准测试目录：基础牌 × 各 12 张 + 五个代表技能。
pub fn standard_catalog() -> Catalog {
    let (cards, mut skills) = standard_catalog_parts();
    native_damage::append_invocation_helpers(&mut skills);
    let mut catalog = Catalog::build(cards, skills);
    catalog.hooks = super::hooks::CONTENT_HOOKS;
    catalog.ruleset_version = STANDARD_RULESET_VERSION;
    catalog
}

/// 带扩展的标准目录（E03 内容锁接线）：启用扩展的卡牌表经冲突校验
/// 后合并；目录版本 = 标准版本与启用扩展集合指纹的组合——存档/恢复
/// 以同一扩展集合校验，集合不同即恢复拒绝（内容锁绑定）。
/// 返回 (目录, 错误)——扩展内容冲突时返回可处理错误。
pub fn standard_catalog_with_extensions(
    packs: &[&crate::content::extension::ExtensionPack],
) -> Result<Catalog, crate::content::extension::ExtensionError> {
    use crate::content::extension::{content_lock_id, merge_extension_cards, ExtensionError};
    use crate::content::host::host_skill_def;
    let (cards, skills) = standard_catalog_parts();
    let mut ext_cards: Vec<CardDef> = Vec::new();
    let mut base_names: std::collections::BTreeSet<&str> = skills.iter().map(|s| s.name).collect();
    base_names.insert(
        crate::event::NativeDamageEffect::JidingRecover
            .helper_name()
            .unwrap(),
    );
    let mut host_names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    base_names.insert("native:liuli/guozhan/jubao");
    let mut ext_skills: Vec<crate::catalog::SkillDef> = Vec::new();
    for pack in packs.iter().filter(|p| p.enabled) {
        ext_cards.extend(pack.cards.iter().cloned());
        for decl in &pack.host_skills {
            if !base_names.insert(decl.name) {
                return Err(ExtensionError::DuplicateSkill {
                    name: decl.name.to_string(),
                });
            }
            if !host_names.insert(decl.name.to_string()) {
                return Err(ExtensionError::DuplicateSkill {
                    name: decl.name.to_string(),
                });
            }
            let mut def = host_skill_def(decl.name.to_string(), decl.behavior);
            def.presentation_binding = decl.presentation_binding.clone();
            ext_skills.push(def);
        }
        for decl in &pack.vm_skills {
            let name = crate::content::names::intern(decl.name.clone());
            if !base_names.insert(name) {
                return Err(ExtensionError::DuplicateSkill {
                    name: decl.name.clone(),
                });
            }
            if !host_names.insert(decl.name.clone()) {
                return Err(ExtensionError::DuplicateSkill {
                    name: decl.name.clone(),
                });
            }
            // PA06：装载期校验一次——畸形 VM 模块在建目录时拒绝（原为
            // 首次触发时执行期校验）。
            let program =
                crate::rvm_bridge::VmSkillProgram::new(decl.module.clone(), decl.name.clone())
                    .map_err(|_| ExtensionError::InvalidVmModule {
                        name: decl.name.clone(),
                    })?;
            ext_skills.push(crate::catalog::SkillDef {
                name,
                trigger: Some(decl.trigger.clone()),
                limits: Default::default(),
                behavior: Box::new(crate::rvm_bridge::VmSkillBehavior {
                    program: std::sync::Arc::new(program),
                    filter: decl.filter.clone(),
                }),
                original_source: None,
                presentation_binding: decl.presentation_binding.clone(),
            });
        }
    }
    if ext_cards
        .iter()
        .any(|c| c.kind == crate::catalog::CardKind::Chenhuodajie)
        && base_names.insert(crate::content::host::WD_CHENHUO_KEY)
        && host_names.insert(crate::content::host::WD_CHENHUO_KEY.to_string())
    {
        let mut def = host_skill_def(
            crate::content::host::WD_CHENHUO_KEY.to_string(),
            crate::content::host::HostBehavior::ChenhuodajieCard,
        );
        def.trigger = Some(crate::catalog::TriggerMeta {
            event: crate::event::EventKind::Damage,
            timing: crate::event::Timing::Normal,
            priority: 0,
            order_selectable: false,
            optional: true,
            extra_events: Vec::new(),
        });
        ext_skills.push(def);
    }
    let cards = merge_extension_cards(cards, &ext_cards)?;
    let mut skills = skills;
    let ext_skill_start = skills.len();
    let ext_skill_len = ext_skills.len();
    skills.extend(ext_skills);
    native_damage::append_invocation_helpers(&mut skills);
    let mut catalog = Catalog::build(cards, skills);
    catalog.hooks = super::hooks::CONTENT_HOOKS;
    catalog.extension_skill_range = ext_skill_start..(ext_skill_start + ext_skill_len);
    catalog.extension_generals = packs
        .iter()
        .filter(|p| p.enabled)
        .flat_map(|p| p.generals.iter().cloned())
        .collect();
    catalog.extension_global_skills = catalog
        .skills
        .iter()
        .enumerate()
        .filter_map(|(id, def)| {
            let behavior = def.behavior.as_host_behavior()?;
            (behavior == crate::content::host::HostBehavior::ChenhuodajieCard
                || packs.iter().any(|p| {
                    p.enabled
                        && (p.init_hook.grants_global_skill(behavior)
                            || (behavior == crate::content::host::HostBehavior::JinlianzhuCancel
                                && p.host_skills.iter().any(|d| d.behavior == behavior)))
                }))
            .then_some(id as crate::ids::SkillDefId)
        })
        .collect();
    // R05-H（§8）：装载器随派生 helper 下传的公开逻辑身份并入目录
    // （helper def → I）；非 helper 声明（`r05_identity=None`）不入表。
    for decl in packs
        .iter()
        .filter(|p| p.enabled)
        .flat_map(|p| p.vm_skills.iter())
    {
        if let Some(identity) = &decl.r05_identity {
            if let Some(def) = catalog.skill_by_name(&decl.name) {
                catalog.r05_identities.insert(def, identity.clone());
            }
        }
    }
    let lock = content_lock_id(packs);
    catalog.ruleset_version = if lock == 0 {
        STANDARD_RULESET_VERSION
    } else {
        STANDARD_RULESET_VERSION ^ (lock | (1 << 63))
    };
    Ok(catalog)
}

fn standard_catalog_parts() -> (Vec<CardDef>, Vec<SkillDef>) {
    let cards = cards::cards();
    // Append in the original order: SkillDefId values enter snapshots and hashes.
    let mut skills = standard_skills::skills();
    skills.extend(boss_skills::skills());
    native_damage::configure(&mut skills);
    (cards, skills)
}

mod native_damage;
