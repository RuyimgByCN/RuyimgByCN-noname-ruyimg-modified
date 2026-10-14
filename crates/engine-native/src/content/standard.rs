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
    CardDef, CardKind, Catalog, Faction, QueryCtx, SkillBehavior, SkillDef, SkillLimits,
    SkillMachine,
    Suit, TriggerMeta,
};
use crate::choice::{ChoiceKind, ChoiceOption, ChoiceSpec};
use crate::event::{EventData, EventKind, Phase, Timing};
use crate::ids::{CardId, PlayerId};
use crate::machine::{Ctx, Flow, LogEvent};
use crate::snap::{
    FanJianStageSnap, GuanXingStageSnap, JiJiStageSnap, JieYinStageSnap, JijiangStageSnap,
    KuRouStageSnap, LiJianStageSnap, MachineSnapshot, QingNangStageSnap, RendeStageSnap,
    JiJieStageSnap, SanCaiStageSnap, ShuShenStageSnap, TongJiStageSnap, TuXiStageSnap, XiaoGuoStageSnap, XunXunStageSnap,
    YaoWuStageSnap, YiShangStageSnap, ZhangbaStageSnap,
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
pub const STANDARD_RULESET_VERSION: u64 = 287;

// ---------------------------------------------------------------------------
// 散财：受到伤害后，可以摸两张牌，然后将一张手牌交给另一名玩家。
// ---------------------------------------------------------------------------

struct SanCaiBehavior;

impl SkillBehavior for SanCaiBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d)
            if d.target == owner && !d.prevented && d.amount > 0)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(SanCaiMachine {
            owner,
            stage: SanCaiStage::Draw,
            target: None,
        })
    }}

#[derive(Clone)]
enum SanCaiStage {
    /// 摸两张（子事件结算后继续）。
    Draw,
    /// 询问交给谁（恢复后读目标）。
    Target,
    /// 已知目标，询问交哪张牌（恢复后读牌）。
    GiveCard,
    /// 已知目标与牌，执行给予。
    Finish,
}

#[derive(Clone)]
struct SanCaiMachine {
    owner: PlayerId,
    stage: SanCaiStage,
    target: Option<PlayerId>,
}

impl SkillMachine for SanCaiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::SanCai {
            owner: self.owner,
            stage: match self.stage {
                SanCaiStage::Draw => SanCaiStageSnap::Draw,
                SanCaiStage::Target => SanCaiStageSnap::Target,
                SanCaiStage::GiveCard => SanCaiStageSnap::GiveCard,
                SanCaiStage::Finish => SanCaiStageSnap::Finish,
            },
            target: self.target,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            SanCaiStage::Draw => {
                ctx.draw_cards(self.owner, 2);
                self.stage = SanCaiStage::Target;
                Flow::Yield
            }
            SanCaiStage::Target => {
                let others: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| *p != self.owner)
                    .map(ChoiceOption::Player)
                    .collect();
                if others.is_empty() || ctx.core.player(self.owner).hand.is_empty() {
                    return Flow::Done;
                }
                self.stage = SanCaiStage::GiveCard;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "sancai_target",
                        options: others,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            SanCaiStage::GiveCard => {
                match ctx.answer_first() {
                    ChoiceOption::Player(p) => self.target = Some(p),
                    _ => return Flow::Done,
                }
                let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return Flow::Done;
                }
                self.stage = SanCaiStage::Finish;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "sancai_card",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            SanCaiStage::Finish => {
                if let (Some(target), ChoiceOption::Card(c)) = (self.target, ctx.answer_first()) {
                    ctx.give_hand_card(self.owner, target, c);
                }
                Flow::Done
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 星夏：出牌阶段主动技——对场上焰灵造成 2 点火焰伤害；随后可选一名敌
// 方：其弃一张红色牌（手牌+装备）或受 1 点火焰伤害（可放弃追击）。
// 间隔 ≥2 轮。祝融（规则 v242）。
// ---------------------------------------------------------------------------

/// 星夏阶段码（0=损焰灵已结算、1=敌方询问挂起、2=敌方已选、3=弃牌已答
/// 待结算——弃红询问由阶段 2 直接挂出，不经独立码）。
const XINGXIA_STAGE_DAMAGE: u8 = 0;
const XINGXIA_STAGE_ASK_ENEMY: u8 = 1;
const XINGXIA_STAGE_ENEMY_DONE: u8 = 2;
const XINGXIA_STAGE_DISCARD_DONE: u8 = 3;

struct XingxiaBehavior;

impl SkillBehavior for XingxiaBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false
    }
    fn start(&self, _owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        unreachable!("星夏为主动技，不经触发总线")
    }
    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        let used = ctx.state.players[owner as usize].xingxia_used_round;
        let round = ctx.state.turn.round;
        if used != 0 && round.saturating_sub(used) < 2 {
            return None;
        }
        // 焰灵定位：BOSS_GENERALS 段内按名查目录索引（表序重排稳健）
        let base = (ctx.catalog.general_count()
            - crate::content::standard::BOSS_GENERALS.len())
            as u16;
        let yanling = (base..ctx.catalog.general_count() as u16)
            .find(|&d| ctx.catalog.general_meta(d).map(|g| g.0) == Some("焰灵"))?;
        let seats = ctx.state.players.len() as u8;
        let target = (0..seats).find(|&t| {
            ctx.state.players[t as usize].alive
                && ctx.state.players[t as usize].general_slots[0] == Some(yanling)
        })?;
        Some(Box::new(XingxiaMachine {
            owner,
            stage: XINGXIA_STAGE_DAMAGE,
            actor: target as u32,
        }))
    }
}

/// 星夏机器：损焰灵线→追击线（敌方弃红或 1 火伤）。
#[derive(Clone)]
struct XingxiaMachine {
    owner: PlayerId,
    stage: u8,
    actor: u32,
}

impl XingxiaMachine {
    /// 焦点敌方的红色牌（手牌+装备）选项。
    fn red_options(ctx: &Ctx<'_>, t: PlayerId) -> Vec<ChoiceOption> {
        let mut options: Vec<ChoiceOption> = ctx.core.player(t)
            .hand
            .iter()
            .copied()
            .filter(|c| {
                matches!(
                    ctx.catalog.cards[*c as usize].suit,
                    crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
                )
            })
            .map(ChoiceOption::Card)
            .collect();
        for c in ctx.core.player(t).equips.iter().copied() {
            if matches!(
                ctx.catalog.cards[c as usize].suit,
                crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond
            ) {
                options.push(ChoiceOption::Card(c));
            }
        }
        options
    }
}

impl SkillMachine for XingxiaMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossXingxia {
            owner: self.owner,
            stage: self.stage,
            actor: self.actor,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            XINGXIA_STAGE_DAMAGE => {
                let t = self.actor as PlayerId;
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(self.owner),
                    t,
                    2,
                    crate::event::DamageNature::Fire,
                    None,
                )));
                ctx.core.player_mut(self.owner).xingxia_used_round =
                    ctx.core.turn.round;
                // 无敌方可追击 → 仅损焰灵即止
                let seats = ctx.core.players.len() as u8;
                let has_enemy = (0..seats).any(|t| {
                    ctx.core.player(t).alive
                        && ctx.core.is_boss_side(t) != ctx.core.is_boss_side(self.owner)
                });
                if !has_enemy {
                    return Flow::Done;
                }
                self.stage = XINGXIA_STAGE_ASK_ENEMY;
                Flow::Yield
            }
            XINGXIA_STAGE_ASK_ENEMY => {
                let seats = ctx.core.players.len() as u8;
                let candidates: Vec<ChoiceOption> = (0..seats)
                    .filter(|&t| {
                        ctx.core.player(t).alive
                            && ctx.core.is_boss_side(t)
                                != ctx.core.is_boss_side(self.owner)
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                self.stage = XINGXIA_STAGE_ENEMY_DONE;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "xingxia_enemy",
                        options: candidates,
                        min: 0,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            XINGXIA_STAGE_ENEMY_DONE => match ctx.answer().first().copied() {
                Some(ChoiceOption::Player(t)) => {
                    self.actor = t as u32;
                    let options = Self::red_options(ctx, t);
                    self.stage = XINGXIA_STAGE_DISCARD_DONE;
                    Flow::WaitChoice {
                        observer: t,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::SelectCards,
                            prompt: "xingxia_discard",
                            options,
                            min: 0,
                            max: 1,
                            allow_cancel: true,
                        },
                    }
                }
                _ => Flow::Done,
            },
            XINGXIA_STAGE_DISCARD_DONE => {
                let t = self.actor as PlayerId;
                match ctx.answer().first().copied() {
                    Some(ChoiceOption::Card(c)) => {
                        // he 位：手牌优先，装备区移除并撤技能
                        if ctx.core.player(t).hand.contains(&c) {
                            ctx.discard_from_hand(t, c);
                        } else {
                            ctx.core.player_mut(t).equips.retain(|x| *x != c);
                            crate::kernel::revoke_equip_skills(
                                ctx.core,
                                ctx.catalog,
                                t,
                                c,
                            );
                            ctx.to_discard(c);
                        }
                    }
                    _ => {
                        // 放弃弃红 → 1 点火焰伤害
                        ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                            Some(self.owner),
                            t,
                            1,
                            crate::event::DamageNature::Fire,
                            None,
                        )));
                    }
                }
                Flow::Done
            }
            _ => Flow::Done,
        }
    }
}

// ---------------------------------------------------------------------------
// 不臣：出牌阶段主动技两线——场上有已死亡的书精 → 失去 1 点体力并把
// 它们全部复活（回复至 1 点、摸 2）；否则令一名己方受伤角色回复 2 点
// 体力。发动后登记轮次（间隔 ≥2 轮可再发动）。勾芒（规则 v240）。
// ---------------------------------------------------------------------------

/// 不臣阶段码（快照显式编码；0/1=复活线，2/3=恢复线）。
const BUCHUN_STAGE_LOSE_HP: u8 = 0;
const BUCHUN_STAGE_REVIVE_NEXT: u8 = 1;
const BUCHUN_STAGE_ASK_TARGET: u8 = 2;
const BUCHUN_STAGE_WAIT_TARGET: u8 = 3;

struct BuchunBehavior;

impl SkillBehavior for BuchunBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        // 主动技：不经触发总线（ActivateSkill 直调 activate）
        false
    }
    fn start(&self, _owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        unreachable!("不臣为主动技，不经触发总线")
    }
    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        // 间隔门控：距上次发动 <2 轮 → 不可发动（上游 storage+roundNumber
        // 过滤同口径）
        let used = ctx.state.players[owner as usize].buchun_used_round;
        let round = ctx.state.turn.round;
        if used != 0 && round.saturating_sub(used) < 2 {
            return None;
        }
        // 书精将定位：BOSS_GENERALS 段内按名查目录索引（表序重排稳健）
        let base = (ctx.catalog.general_count()
            - crate::content::standard::BOSS_GENERALS.len()) as u16;
        let shujing = (base..ctx.catalog.general_count() as u16)
            .find(|&d| ctx.catalog.general_meta(d).map(|g| g.0) == Some("书精"))?;
        let seats = ctx.state.players.len() as u8;
        let dead_shujings: Vec<u32> = (0..seats)
            .filter(|&t| {
                let p = &ctx.state.players[t as usize];
                !p.alive && p.general_slots[0] == Some(shujing)
            })
            .map(|t| t as u32)
            .collect();
        if !dead_shujings.is_empty() {
            // 复活线：失 1 体 → 全部复活（回 1 体、各摸 2）
            return Some(Box::new(BuchunMachine {
                owner,
                stage: BUCHUN_STAGE_LOSE_HP,
                targets: dead_shujings,
            }));
        }
        // 恢复线：须存在同阵营受伤角色（上游 filterTarget isFriendOf+
        // isDamaged 同口径；is_boss_side 异侧=敌）
        let has_damaged_friend = (0..seats).any(|t| {
            let p = &ctx.state.players[t as usize];
            p.alive
                && p.hp < p.max_hp as i32
                && ctx.state.is_boss_side(t) == ctx.state.is_boss_side(owner)
        });
        if has_damaged_friend {
            return Some(Box::new(BuchunMachine {
                owner,
                stage: BUCHUN_STAGE_ASK_TARGET,
                targets: Vec::new(),
            }));
        }
        None
    }
}

/// 不臣机器：复活线两步（失体→逐个复活）、恢复线两步（询问→回 2）。
#[derive(Clone)]
struct BuchunMachine {
    owner: PlayerId,
    stage: u8,
    targets: Vec<u32>,
}

impl SkillMachine for BuchunMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossBuchun {
            owner: self.owner,
            stage: self.stage,
            targets: self.targets.clone(),
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            BUCHUN_STAGE_LOSE_HP => {
                ctx.emit(EventData::LoseHp {
                    player: self.owner,
                    amount: 1,
                });
                self.stage = BUCHUN_STAGE_REVIVE_NEXT;
                Flow::Yield
            }
            BUCHUN_STAGE_REVIVE_NEXT => {
                while let Some(t) = self.targets.pop() {
                    let t = t as PlayerId;
                    if ctx.core.player(t).alive {
                        continue;
                    }
                    // 复活（上游 revive(1)+draw(2) 同口径；重整计数清零
                    // 与重整复活臂同款；轮次登记）
                    {
                        let p = ctx.core.player_mut(t);
                        p.alive = true;
                        p.dying = false;
                        p.hp = 1;
                        p.boss_chongzheng = 0;
                    }
                    ctx.log(LogEvent::BossRevived { player: t });
                    ctx.draw_cards(t, 2);
                    ctx.core.player_mut(self.owner).buchun_used_round =
                        ctx.core.turn.round;
                    return Flow::Yield;
                }
                Flow::Done
            }
            BUCHUN_STAGE_ASK_TARGET => {
                let seats = ctx.core.players.len() as u8;
                let candidates: Vec<ChoiceOption> = (0..seats)
                    .filter(|&t| {
                        let p = ctx.core.player(t);
                        p.alive
                            && p.hp < p.max_hp as i32
                            && ctx.core.is_boss_side(t) == ctx.core.is_boss_side(self.owner)
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                self.stage = BUCHUN_STAGE_WAIT_TARGET;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "buchun_target",
                        options: candidates,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            BUCHUN_STAGE_WAIT_TARGET => {
                if let Some(ChoiceOption::Player(t)) = ctx.answer().first().copied() {
                    ctx.recover(t, 2, None);
                    ctx.core.player_mut(self.owner).buchun_used_round =
                        ctx.core.turn.round;
                }
                Flow::Done
            }
            _ => Flow::Done,
        }
    }
}

// ---------------------------------------------------------------------------
// 移伤：当你即将受到伤害时，你可以将其转移给另一名存活玩家。
// ---------------------------------------------------------------------------

/// 招祸（Boss 毕方，规则 v227）：DamageBefore 一步直改——持有者造成的
/// 伤害视为火属性（nature=Fire）。
struct ZhaoHuoBehavior;

impl SkillBehavior for ZhaoHuoBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d)
            if d.source == Some(owner) && d.target != owner && !d.prevented)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(ZhaoHuoMachine { owner })
    }
}

/// 招祸机器：无挂起，启动即改当前伤害帧 nature。
#[derive(Clone)]
struct ZhaoHuoMachine {
    owner: PlayerId,
}

impl SkillMachine for ZhaoHuoMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossZhaohuo { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let d = ctx.current_damage_mut();
        d.nature = crate::event::DamageNature::Fire;
        Flow::Done
    }
}

/// 覆泰（Boss 穷奇，规则 v212）：准备阶段（判定阶段开始锚点，神智同
/// 口径）——所有受伤角色按座位序各回复 1 点体力。
struct FutaiBehavior;

impl SkillBehavior for FutaiBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase { player, phase: Phase::Judge, begin: true } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(FutaiMachine { owner })
    }
}

/// 乱常（Boss 梼杌，规则 v218）：准备阶段（判定阶段开始锚点）视为使
/// 用【南蛮入侵】（虚拟牌，目标=全部其他存活角色）；结束阶段视为用
/// 【万箭齐发】（kernel Phase::End 臂查询式）。触发计划层过滤 disabled，
/// 激活门控=SkillInstance.enabled 翻转。
struct LuanChangBehavior;

impl SkillBehavior for LuanChangBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase { player, phase: Phase::Judge, begin: true } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(LuanChangMachine { owner })
    }
}

/// 乱常机器：无挂起，启动即虚拟南蛮（use_virtual_card 先例：虎煞
/// 虚拟南蛮同款，目标=全部其他存活角色）。
#[derive(Clone)]
struct LuanChangMachine {
    owner: PlayerId,
}

impl SkillMachine for LuanChangMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossLuanchang { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let targets: Vec<PlayerId> = ctx
            .core
            .seat_order_from(self.owner)
            .into_iter()
            .filter(|p| *p != self.owner)
            .collect();
        ctx.use_virtual_card(self.owner, CardKind::SavageAttack, targets);
        Flow::Done
    }
}

/// 奇佐（Boss 郭嘉，规则 v259，群雄乱入）：你使用锦囊牌结算结束后，你
/// 使用一张同名牌。触发挂 CardUsed AfterTriggers 窗口（上游 useCardAfter
/// →前置近似：副本效果先于原牌效果，登记）；无懈不可复制；铁索不复制
///（上游 AI check 排除同口径）；副本载体槽守卫递归（上游 event.parent
/// name 守卫同口径）。
struct QizuoBehavior;

/// 奇佐可复制的锦囊种（标准锦囊集；G1 扩展锦囊不入 boss 场）：
/// 延时锦囊含判定区同名去重门（上游 canUse 同口径）。
fn qizuo_copyable(kind: CardKind) -> bool {
    matches!(
        kind,
        CardKind::SavageAttack
            | CardKind::ArcheryAttack
            | CardKind::Duel
            | CardKind::Le
            | CardKind::Bing
            | CardKind::ShanDian
            | CardKind::Dismantle
            | CardKind::Snatch
            | CardKind::Wuzhong
            | CardKind::Taoyuan
            | CardKind::Wugu
            | CardKind::Jiedao
    )
}

impl SkillBehavior for QizuoBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        let crate::event::EventData::CardUsed(u) = event else {
            return false;
        };
        if u.cancelled || u.player != owner {
            return false;
        }
        // 副本载体自身的使用不再触发（上游 event.parent name 守卫同口径）
        if ctx.state.turn.qizuo_copy == Some(u.card) {
            return false;
        }
        let kind = u.use_as.unwrap_or_else(|| ctx.catalog.card_kind(u.card));
        if !qizuo_copyable(kind) {
            return false;
        }
        // 延时锦囊：目标判定区无同名（上游 canUse 同口径）
        if matches!(kind, CardKind::Le | CardKind::Bing | CardKind::ShanDian) {
            for t in &u.targets {
                if ctx
                    .state
                    .player(*t)
                    .judge
                    .iter()
                    .any(|c| ctx.catalog.card_kind(*c) == kind)
                {
                    return false;
                }
            }
        }
        // 原目标全体存活（上游 isIn 同口径）
        u.targets.iter().all(|t| ctx.state.player(*t).alive)
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        let crate::event::EventData::CardUsed(u) = event else {
            unreachable!("qizuo trigger event");
        };
        Box::new(QizuoMachine {
            owner,
            targets: u.targets.clone(),
        })
    }
}

/// 奇佐机器：无挂起一步机——以牌堆底实体为载体按原牌种复用（use_
/// virtual_card 完整管线），载体实体入副本槽供其 AfterTriggers 过滤。
#[derive(Clone)]
struct QizuoMachine {
    owner: PlayerId,
    targets: Vec<PlayerId>,
}

impl SkillMachine for QizuoMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossQizuo { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 牌堆空无载体（上游凭空造牌不依赖实体，登记）；过滤后目标离场
        //（同窗口其他触发先行结算所致）→ 不发动
        if ctx.core.deck.is_empty()
            || !self
                .targets
                .iter()
                .all(|t| ctx.core.player(*t).alive)
        {
            return Flow::Done;
        }
        let u = ctx
            .current_card_use()
            .expect("qizuo machine: card use frame");
        let kind = u.use_as.unwrap_or_else(|| ctx.catalog.card_kind(u.card));
        let carrier = ctx.use_virtual_card(self.owner, kind, self.targets.clone());
        ctx.core.turn.qizuo_copy = Some(carrier);
        Flow::Done
    }
}

/// 枭雄（Boss 刘备，规则 v261，群雄乱入）：其他角色使用基本牌或锦囊牌
/// 结算结束后，你获得该牌的副本。触发挂 CardUsed AfterTriggers 窗口
///（上游 global useCardAfter 同口径；副本入手不 emit CardUsed=无递归）。
struct XiaoxiongBehavior;

/// 枭雄可复制的牌种：基本牌+锦囊（含延时——上游 get.type(card,"trick")
/// 延时算 trick；装备排除）。
fn xiaoxiong_copyable(kind: CardKind) -> bool {
    matches!(
        kind,
        CardKind::Sha
            | CardKind::Shan
            | CardKind::Tao
            | CardKind::Jiu
            | CardKind::FireSha
            | CardKind::LeiSha
            | CardKind::ShouLiJian
            | CardKind::SavageAttack
            | CardKind::ArcheryAttack
            | CardKind::Duel
            | CardKind::Le
            | CardKind::Bing
            | CardKind::ShanDian
            | CardKind::Dismantle
            | CardKind::Snatch
            | CardKind::Wuzhong
            | CardKind::Taoyuan
            | CardKind::Wugu
            | CardKind::Jiedao
    )
}

impl SkillBehavior for XiaoxiongBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        let crate::event::EventData::CardUsed(u) = event else {
            return false;
        };
        if u.cancelled || u.player == owner {
            return false;
        }
        let kind = u.use_as.unwrap_or_else(|| ctx.catalog.card_kind(u.card));
        xiaoxiong_copyable(kind)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(XiaoxiongMachine { owner })
    }
}

/// 枭雄机器：无挂起一步机——牌堆底实体入手（上游 createCard 凭空造牌→
/// 载体实体入手登记差异：花色点数随载体+消耗库顶实体；牌堆空静默）。
#[derive(Clone)]
struct XiaoxiongMachine {
    owner: PlayerId,
}

impl SkillMachine for XiaoxiongMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossXiaoxiong { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let Some(carrier) = ctx.core.deck.first().copied() else {
            return Flow::Done;
        };
        ctx.core.deck.remove(0);
        ctx.core.player_mut(self.owner).hand.push(carrier);
        ctx.log(LogEvent::CardsGiven {
            from: self.owner,
            to: self.owner,
            cards: vec![carrier],
        });
        Flow::Done
    }
}

/// 凌波（Boss 甄姬，规则 v262，群雄乱入）：你使用或打出【闪】后摸两张。
/// 触发挂 CardResponded+CardUsed（extra）；filter=声明/实体牌种为闪。
struct LingboBehavior;

impl SkillBehavior for LingboBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        match event {
            crate::event::EventData::CardResponded { card, player } => {
                *player == owner && ctx.catalog.card_kind(*card) == CardKind::Shan
            }
            crate::event::EventData::CardUsed(u) => {
                u.player == owner
                    && !u.cancelled
                    && u.use_as.unwrap_or_else(|| ctx.catalog.card_kind(u.card))
                        == CardKind::Shan
            }
            _ => false,
        }
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(LingboMachine { owner })
    }
}

/// 凌波机器：无挂起一步机——摸 2。
#[derive(Clone)]
struct LingboMachine {
    owner: PlayerId,
}

impl SkillMachine for LingboMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossLingbo { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        ctx.draw_cards(self.owner, 2);
        Flow::Done
    }
}

/// 踏水（Boss 甄姬，规则 v262，群雄乱入）：你使用或打出黑色牌后，将一名
/// 其他角色翻面。触发挂 CardResponded+CardUsed（extra）；filter=实体花
/// 色黑（上游 get.color(event.card) 实体同口径）。
struct TashuiBehavior;

impl SkillBehavior for TashuiBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        let card = match event {
            crate::event::EventData::CardResponded { card, player } => {
                if *player != owner {
                    return false;
                }
                *card
            }
            crate::event::EventData::CardUsed(u) => {
                if u.player != owner || u.cancelled {
                    return false;
                }
                u.card
            }
            _ => return false,
        };
        matches!(
            ctx.catalog.cards[card as usize].suit,
            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
        )
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(TashuiMachine { owner })
    }
}

/// 踏水机器：无挂起一步机——座位序首个其他存活角色翻面（chooseTarget
/// AI 态度面→首个确定性近似登记；经 TurnOverBefore 走翻面提交，虚失同
/// 款通道；无其他存活静默）。
#[derive(Clone)]
struct TashuiMachine {
    owner: PlayerId,
}

impl SkillMachine for TashuiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossTashui { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let seats = ctx.core.players.len() as PlayerId;
        if let Some(t) = (0..seats)
            .find(|&t| t != self.owner && ctx.core.player(t).alive)
        {
            ctx.emit(EventData::TurnOverBefore {
                player: t,
                cancelled: false,
            });
        }
        Flow::Done
    }
}

/// 雷击·张角（Boss 张角，规则 v263，群雄乱入）：你使用或打出【闪】后，
/// 一名其他角色判定：黑则其受到 1 点雷伤且你摸一张牌。触发挂 CardResponded
/// +CardUsed（extra）；chooseTarget AI 态度面→座位序首个确定性近似。
struct ZjLeijiBehavior;

impl SkillBehavior for ZjLeijiBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        match event {
            crate::event::EventData::CardResponded { card, player } => {
                *player == owner && ctx.catalog.card_kind(*card) == CardKind::Shan
            }
            crate::event::EventData::CardUsed(u) => {
                u.player == owner
                    && !u.cancelled
                    && u.use_as.unwrap_or_else(|| ctx.catalog.card_kind(u.card))
                        == CardKind::Shan
            }
            _ => false,
        }
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(ZjLeijiMachine { owner })
    }
}

/// 雷击·张角机器：无挂起一步机——座位序首个其他存活角色判定（内联确定
/// 性口径=铁骑先例，无改判窗）：黑则 1 雷伤+持有者摸 1；红色无伤。
#[derive(Clone)]
struct ZjLeijiMachine {
    owner: PlayerId,
}

impl SkillMachine for ZjLeijiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossZjLeiji { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let seats = ctx.core.players.len() as PlayerId;
        if let Some(t) = (0..seats).find(|&t| t != self.owner && ctx.core.player(t).alive)
        {
            let judged = ctx.draw_judge_card();
            let black = matches!(
                ctx.catalog.cards[judged as usize].suit,
                crate::catalog::Suit::Spade | crate::catalog::Suit::Club
            );
            ctx.to_discard(judged);
            if black {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(self.owner),
                    t,
                    1,
                    crate::event::DamageNature::Thunder,
                    None,
                )));
                ctx.draw_cards(self.owner, 1);
            }
        }
        Flow::Done
    }
}

/// 黑绳（十殿阎罗宋帝王，规则 v266，群雄乱入）：你死亡时，全部其他角色
/// 横置。Death 前置窗口触发（changan 氏遗言同款；forceDie 死后面=时序
/// 不敏感；死亡被取消仍已横置=登记差异）。
struct HeishengBehavior;

impl SkillBehavior for HeishengBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Death { player, .. } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(HeishengMachine { owner })
    }
}

/// 黑绳机器：无挂起一步机——全部其他存活横置（索命直写同款）。
#[derive(Clone)]
struct HeishengMachine {
    owner: PlayerId,
}

impl SkillMachine for HeishengMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossHeisheng { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let seats = ctx.core.players.len() as PlayerId;
        for t in 0..seats {
            if t != self.owner && ctx.core.player(t).alive && !ctx.core.player(t).linked {
                ctx.core.player_mut(t).linked = true;
                ctx.log(LogEvent::Linked { player: t, linked: true });
            }
        }
        Flow::Done
    }
}

/// 雷诛（十殿阎罗卞城王，规则 v267）：你死亡时，按座位序对全部其他角色
/// 各造成 1 点雷伤。Death 前置窗口触发（黑绳同位；forceDie 死后面=时序
/// 不敏感；死亡被取消仍已触发=登记差异）。
struct LeizhuBehavior;

impl SkillBehavior for LeizhuBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Death { player, .. } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(LeizhuMachine { owner, emitted: 0 })
    }
}

/// 雷诛机器：逐目标雷伤（伤害走 Damage 帧，铁索逐发同款 Yield——
/// 伤害可致死并级联遗言，逐发让管线消化窗口）。
/// 可挂起（目标濒死询问期间停机），故入快照重建表。
#[derive(Clone)]
pub(crate) struct LeizhuMachine {
    owner: PlayerId,
    emitted: u8,
}

impl LeizhuMachine {
    pub(crate) fn new(owner: PlayerId, emitted: u8) -> Self {
        Self { owner, emitted }
    }
}

impl SkillMachine for LeizhuMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossLeizhu {
            owner: self.owner,
            emitted: self.emitted,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let seats = ctx.core.players.len() as PlayerId;
        while self.emitted < seats {
            let t = self.emitted;
            self.emitted += 1;
            if t == self.owner || !ctx.core.player(t).alive {
                continue;
            }
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                Some(self.owner),
                t,
                1,
                crate::event::DamageNature::Thunder,
                None,
            )));
            return Flow::Yield;
        }
        Flow::Done
    }
}

/// 热恼（十殿阎罗泰山王，规则 v268）：你死亡时，随机一名其他角色受 3
/// 点火焰伤害。Death 前置窗口触发（雷诛同位；死亡被取消仍已触发=登记
/// 差异；randomGet→座位序首个确定性近似登记）。
struct RenaoBehavior;

impl SkillBehavior for RenaoBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Death { player, .. } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(RenaoMachine { owner, emitted: 0 })
    }
}

/// 热恼机器：单发 3 火伤（Yield 让濒死窗口消化；可挂起入重建表）。
#[derive(Clone)]
pub(crate) struct RenaoMachine {
    owner: PlayerId,
    emitted: u8,
}

impl RenaoMachine {
    pub(crate) fn new(owner: PlayerId, emitted: u8) -> Self {
        Self { owner, emitted }
    }
}

impl SkillMachine for RenaoMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossRenao {
            owner: self.owner,
            emitted: self.emitted,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if self.emitted == 0 {
            self.emitted = 1;
            let seats = ctx.core.players.len() as PlayerId;
            if let Some(t) =
                (0..seats).find(|&t| t != self.owner && ctx.core.player(t).alive)
            {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(self.owner),
                    t,
                    3,
                    crate::event::DamageNature::Fire,
                    None,
                )));
                return Flow::Yield;
            }
        }
        Flow::Done
    }
}

/// 忘魂（十殿阎罗孟婆，规则 v270）：你死亡时，清全场往事封禁，随机两
/// 名敌方角色各随机失去一个技能。Death 前置窗口触发（黑绳同位直写一
/// 步机；登记差异：随机敌将/随机技→座位序前两个其他存活各首个启用技
/// ——skills 为 BTreeMap 按 (座位,def 序) 确定性；敌方面→全体其他；回
/// 魂牌堆注入与回收面未落）。
struct WanghunBehavior;

impl SkillBehavior for WanghunBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Death { player, .. } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(WanghunMachine { owner })
    }
}

/// 忘魂机器：直写一步机（禁用=SkillInstance.enabled 直写+SkillToggled，
/// 隐贼同款；无子事件不挂起）。
#[derive(Clone)]
struct WanghunMachine {
    owner: PlayerId,
}

impl SkillMachine for WanghunMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossWanghun { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // ① 清全场往事封禁（上游敌方 removeSkill boss_wangshi2 同口径）
        for p in ctx.core.players.iter_mut() {
            p.wangshi_ban = false;
        }
        // ② 座位序前两个其他存活各禁用首个启用技
        let seats = ctx.core.players.len() as PlayerId;
        let mut disabled = 0u8;
        for t in 0..seats {
            if disabled >= 2 {
                break;
            }
            if t == self.owner || !ctx.core.player(t).alive {
                continue;
            }
            let first_enabled = ctx
                .core
                .skills
                .iter()
                .find(|((o, _), i)| *o == t && i.enabled)
                .map(|((_, dd), _)| *dd);
            if let Some(dd) = first_enabled {
                if let Some(inst) = ctx.core.skills.get_mut(&(t, dd)) {
                    inst.enabled = false;
                }
                ctx.log(LogEvent::SkillToggled {
                    player: t,
                    skill: ctx.catalog.skill_name(dd),
                    enabled: false,
                });
                disabled += 1;
            }
        }
        Flow::Done
    }
}

/// 诛心（十殿阎罗阎罗王，规则 v271）：你死亡时，令场上血量最少的一名
/// 其他角色受到 2 点伤害。Death 前置窗口触发（热恼同位直写一步机；
/// 登记差异：上游 chooseTarget 最低体力池自选→座位序首个最低体力）。
struct ZhuxinBehavior;

impl SkillBehavior for ZhuxinBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Death { player, .. } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(ZhuxinMachine {
            owner,
            emitted: 0,
        })
    }
}

/// 诛心机器：直写一步机（emit+Yield 一次，emitted 防重入；热恼同构）。
#[derive(Clone)]
struct ZhuxinMachine {
    owner: PlayerId,
    emitted: u8,
}

impl SkillMachine for ZhuxinMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossZhuxin {
            owner: self.owner,
            emitted: self.emitted,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if self.emitted == 0 {
            self.emitted = 1;
            let seats = ctx.core.players.len() as PlayerId;
            // 座位序首个最低体力（严格更小才替换=并列取首）
            let mut best: Option<PlayerId> = None;
            for t in 0..seats {
                if t == self.owner || !ctx.core.player(t).alive {
                    continue;
                }
                let hp = ctx.core.player(t).hp;
                if best.map(|b| hp < ctx.core.player(b).hp).unwrap_or(true) {
                    best = Some(t);
                }
            }
            if let Some(t) = best {
                ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                    Some(self.owner),
                    t,
                    2,
                    crate::event::DamageNature::Normal,
                    None,
                )));
                return Flow::Yield;
            }
        }
        Flow::Done
    }
}

/// 平等（十殿阎罗平等王，规则 v272）：你死亡时——最高体力其他存活
///（座位序首个）先受 2 点火焰伤害、重算后再受 1 点火焰伤害。Death 前置
/// 窗口触发（诛心同位直写两波机；登记差异：上游两波 randomGet×
/// lib.linked randomGet→座位序首个+恒定火）。
struct PingdengBehavior;

impl SkillBehavior for PingdengBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Death { player, .. } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(PingdengMachine {
            owner,
            emitted: 0,
        })
    }
}

/// 平等机器：直写两波机（emit+Yield 两次，emitted 防重入；诛心同构）。
#[derive(Clone)]
struct PingdengMachine {
    owner: PlayerId,
    emitted: u8,
}

impl SkillMachine for PingdengMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossPingdeng {
            owner: self.owner,
            emitted: self.emitted,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if self.emitted >= 2 {
            return Flow::Done;
        }
        let seats = ctx.core.players.len() as PlayerId;
        // 最高体力池=体力等于其他存活最大者；取座位序首个
        let mut max_hp: Option<i32> = None;
        for t in 0..seats {
            if t == self.owner || !ctx.core.player(t).alive {
                continue;
            }
            let hp = ctx.core.player(t).hp;
            if max_hp.map(|m| hp > m).unwrap_or(true) {
                max_hp = Some(hp);
            }
        }
        let target = (0..seats).find(|&t| {
            t != self.owner
                && ctx.core.player(t).alive
                && Some(ctx.core.player(t).hp) == max_hp
        });
        self.emitted += 1;
        if let Some(t) = target {
            let amount = if self.emitted == 1 { 2 } else { 1 };
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                None,
                t,
                amount,
                crate::event::DamageNature::Fire,
                None,
            )));
            return Flow::Yield;
        }
        Flow::Done
    }
}

/// 火刑（十殿阎罗都市王，规则 v273）：你死亡时，全部其他存活角色按座
/// 位序各受 1 点火焰伤害（热恼同位直写逐座机；emitted=下一待扫描座位
/// 索引，中途死亡由存活过滤跳过）。
struct HuoxingBehavior;

impl SkillBehavior for HuoxingBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Death { player, .. } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(HuoxingMachine {
            owner,
            emitted: 0,
        })
    }
}

/// 火刑机器：直写逐座机（emit+Yield 逐目标，emitted=下一座位索引）。
#[derive(Clone)]
struct HuoxingMachine {
    owner: PlayerId,
    emitted: u8,
}

impl SkillMachine for HuoxingMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossHuoxing {
            owner: self.owner,
            emitted: self.emitted,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let seats = ctx.core.players.len() as PlayerId;
        while self.emitted < seats {
            let t = self.emitted;
            self.emitted += 1;
            if t == self.owner || !ctx.core.player(t).alive {
                continue;
            }
            ctx.emit(EventData::Damage(crate::machine::damage_data_with(
                None,
                t,
                1,
                crate::event::DamageNature::Fire,
                None,
            )));
            return Flow::Yield;
        }
        Flow::Done
    }
}

/// 冰封（十殿阎罗楚江王，规则 v275）：你死亡时，伤害来源存活且未翻面
/// 则将其翻面。Death 前置窗口触发（忘魂同位直写一步机；无来源/已翻面
/// 不重复翻）。
struct BingfengBehavior;

impl SkillBehavior for BingfengBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Death { player, .. } if *player == owner)
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        let killer = match event {
            EventData::Death { killer, .. } => *killer,
            _ => None,
        };
        Box::new(BingfengMachine { owner, killer })
    }
}

/// 冰封机器：直写一步机（翻面写+TurnOvered；风雪刃确认臂同款写法）。
#[derive(Clone)]
struct BingfengMachine {
    owner: PlayerId,
    killer: Option<PlayerId>,
}

impl SkillMachine for BingfengMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossBingfeng { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 来源=死亡事件的 killer（behavior.start 时捕获）
        if let Some(src) = self.killer {
            if src != self.owner
                && ctx.core.player(src).alive
                && !ctx.core.player(src).face_down
            {
                let flipped = {
                    let p = ctx.core.player_mut(src);
                    p.face_down = true;
                    p.face_down
                };
                ctx.emit(EventData::TurnOvered {
                    player: src,
                    face_down: flipped,
                });
            }
        }
        Flow::Done
    }
}

/// 望乡（十殿阎罗秦广王，规则 v276）：你死亡时，全部有装备的其他角色
/// 装备全弃（撤技能+入弃堆；绝红拆装管线同款直写一步机）。
struct WangxiangBehavior;

impl SkillBehavior for WangxiangBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Death { player, .. } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(WangxiangMachine { owner })
    }
}

/// 望乡机器：直写一步机（逐目标装备全弃；绝红同款 revoke+入弃堆）。
#[derive(Clone)]
struct WangxiangMachine {
    owner: PlayerId,
}

impl SkillMachine for WangxiangMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossWangxiang { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let seats = ctx.core.players.len() as PlayerId;
        for t in 0..seats {
            if t == self.owner || !ctx.core.player(t).alive {
                continue;
            }
            let equips: Vec<crate::ids::CardId> =
                ctx.core.player(t).equips.iter().copied().collect();
            for c in equips {
                ctx.core.player_mut(t).equips.retain(|x| *x != c);
                crate::kernel::revoke_equip_skills(
                    &mut ctx.core,
                    ctx.catalog,
                    t,
                    c,
                );
                ctx.to_discard(c);
            }
        }
        Flow::Done
    }
}

/// 蛇毒标记结算（Boss 相柳毒妒授予，规则 v223）：持有标记>0 的角色回
/// 合开始——「弃置 X 张牌，或失去 X 点体力」（挂起 SelectCards；取消/
/// 空选=失 X）→ 标记-1；X=当前标记数。
struct ShedUBehavior;

impl SkillBehavior for ShedUBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase { player, phase: Phase::Judge, begin: true }
            if *player == owner
                && ctx.state.player(owner).shedu > 0)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(ShedUMachine { owner, asked: false })
    }
}

#[derive(Clone)]
struct ShedUMachine {
    owner: PlayerId,
    asked: bool,
}

impl SkillMachine for ShedUMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::Shedu { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if self.asked {
            // 消费：非空选=弃所选牌（各-1 张？逐张去手）；空选=失 X 体力
            let x = ctx.core.player(self.owner).shedu as usize;
            let chosen: Vec<crate::ids::CardId> = ctx
                .answer()
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
            if chosen.is_empty() {
                ctx.emit(EventData::LoseHp {
                    player: self.owner,
                    amount: x as u32,
                });
            } else {
                for c in chosen {
                    ctx.discard_from_hand(self.owner, c);
                }
            }
            let p = ctx.core.player_mut(self.owner);
            p.shedu = p.shedu.saturating_sub(1);
            return Flow::Done;
        }
        self.asked = true;
        let x = ctx.core.player(self.owner).shedu as usize;
        if x == 0 {
            return Flow::Done;
        }
        let options: Vec<ChoiceOption> = ctx
            .core
            .player(self.owner)
            .hand
            .iter()
            .copied()
            .map(ChoiceOption::Card)
            .collect();
        Flow::WaitChoice {
            observer: self.owner,
            spec: ChoiceSpec {
                kind: ChoiceKind::SelectCards,
                prompt: "shedu_discard_or_lose",
                options,
                min: 0,
                max: x,
                allow_cancel: true, // 空选=失去 X 点体力
            },
        }
    }
}

/// 覆泰机器：无挂起，启动即按座位序回血。
#[derive(Clone)]
struct FutaiMachine {
    owner: PlayerId,
}

impl SkillMachine for FutaiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossFutai { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        for p in ctx.core.seat_order_from(self.owner) {
            let pl = ctx.core.player(p);
            if pl.alive && pl.hp < pl.max_hp as i32 {
                ctx.recover(p, 1, None);
            }
        }
        Flow::Done
    }
}

/// 相德（Boss 混沌，规则 v210）：DamageBefore 一步直改——来源装备区
/// 有武器且非持有者本人时伤害 +1。
struct XiangDeBehavior;

impl SkillBehavior for XiangDeBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d)
            if d.target == owner
                && !d.prevented
                && d.source.is_some()
                && d.source != Some(owner)
                && ctx
                    .state
                    .player(d.source.expect("checked"))
                    .equips
                    .iter()
                    .any(|c| ctx.catalog.cards[*c as usize].kind == crate::catalog::CardKind::Weapon))
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(XiangDeMachine { owner })
    }
}

/// 相德机器：无挂起，启动即直改当前伤害帧。
#[derive(Clone)]
struct XiangDeMachine {
    owner: PlayerId,
}

impl SkillMachine for XiangDeMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BossXiangde { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let d = ctx.current_damage_mut();
        d.amount += 1;
        Flow::Done
    }
}

struct YiShangBehavior;

impl SkillBehavior for YiShangBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d) if d.target == owner && !d.prevented)
    }


    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(YiShangMachine {
            owner,
            stage: YiShangStage::Target,
        })
    }
}

#[derive(Clone)]
enum YiShangStage {
    Target,
    Apply,
}

#[derive(Clone)]
struct YiShangMachine {
    owner: PlayerId,
    stage: YiShangStage,
}

impl SkillMachine for YiShangMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::YiShang {
            owner: self.owner,
            stage: match self.stage {
                YiShangStage::Target => YiShangStageSnap::Target,
                YiShangStage::Apply => YiShangStageSnap::Apply,
            },
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            YiShangStage::Target => {
                let others: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| *p != self.owner)
                    .map(ChoiceOption::Player)
                    .collect();
                if others.is_empty() {
                    return Flow::Done;
                }
                self.stage = YiShangStage::Apply;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "yishang_target",
                        options: others,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            YiShangStage::Apply => {
                if let ChoiceOption::Player(p) = ctx.answer_first() {
                    let d = ctx.current_damage_mut();
                    d.redirect_chain.push(d.target);
                    d.target = p;
                }
                Flow::Done
            }
        }
    }
}

// ---------------------------------------------------------------------------
// 反馈（司马懿）：受到 1 点伤害后，可获得伤害来源的一张牌（手牌优先，
// 其次装备区）。来源死亡/无牌或无来源时不触发。
#[derive(Clone)]
struct FanKuiBehavior {
    _p: (),
}

impl SkillBehavior for FanKuiBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d)
            if d.target == owner && !d.prevented && d.amount > 0)
            && event_damage_source_stealable(ctx, event)
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        let source = match event {
            EventData::Damage(d) => d.source,
            _ => None,
        };
        Box::new(FanKuiMachine {
            owner,
            source,
            asked: false,
        })
    }
}

/// 来源存活且持有至少一张可取的牌。
fn event_damage_source_stealable(ctx: &QueryCtx<'_>, event: &EventData) -> bool {
    let Some(src) = (match event {
        EventData::Damage(d) => d.source,
        _ => None,
    }) else {
        return false;
    };
    let p = ctx.state.player(src);
    p.alive && (!p.hand.is_empty() || !p.equips.is_empty())
}

#[derive(Clone)]
struct FanKuiMachine {
    owner: PlayerId,
    source: Option<PlayerId>,
    asked: bool,
}

impl SkillMachine for FanKuiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::FanKui {
            owner: self.owner,
            source: self.source,
            asked: self.asked,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 可选触发由 OptionalGate 的 trigger_confirm 询问确认；
        // 本机器被启动（gate 已确认）后直接执行取牌。
        if !self.asked {
            self.asked = true;
            if let Some(src) = self.source {
                if ctx.core.player(src).alive {
                    let p = ctx.core.player(src);
                    let card = p.hand.first().copied().or_else(|| p.equips.first().copied());
                    if let Some(c) = card {
                        ctx.give_hand_card(src, self.owner, c);
                    }
                }
            }
        }
        Flow::Done
    }
}

// 奸雄（曹操）：受到伤害后，可获得造成此伤害的牌（牌已进弃牌堆）。
#[derive(Clone)]
struct JianXiongBehavior;

impl SkillBehavior for JianXiongBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d)
            if d.target == owner && !d.prevented && d.amount > 0 && d.card.is_some())
            && d_card_in_discard(ctx, event)
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        let card = match event {
            EventData::Damage(d) => d.card,
            _ => None,
        };
        Box::new(JianXiongMachine {
            owner,
            card: card.expect("filtered"),
            asked: false,
        })
    }
}

fn d_card_in_discard(ctx: &QueryCtx<'_>, event: &EventData) -> bool {
    let Some(card) = (match event {
        EventData::Damage(d) => d.card,
        _ => None,
    }) else {
        return false;
    };
    ctx.state.discard.contains(&card)
}

#[derive(Clone)]
struct JianXiongMachine {
    owner: PlayerId,
    card: CardId,
    asked: bool,
}

impl SkillMachine for JianXiongMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::JianXiong {
            owner: self.owner,
            card: self.card,
            asked: self.asked,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // OptionalGate 已确认；牌仍在弃牌堆时收入手牌（竞态防御：不在则跳过）
        if !self.asked {
            self.asked = true;
            if let Some(pos) = ctx.core.discard.iter().position(|c| *c == self.card) {
                ctx.core.discard.remove(pos);
                ctx.core.player_mut(self.owner).hand.push(self.card);
            }
        }
        Flow::Done
    }
}

// 刚烈（夏侯惇）：受到伤害后，可进行判定：若判定牌不为红桃，
// 伤害来源受到你造成的 1 点伤害（递归由确认闸门天然限流）。
#[derive(Clone)]
struct GangLieBehavior;

impl SkillBehavior for GangLieBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d)
            if d.target == owner && !d.prevented && d.amount > 0
                && d.source.is_some() && d.source != Some(owner))
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        let source = match event {
            EventData::Damage(d) => d.source.expect("filtered"),
            _ => unreachable!(),
        };
        Box::new(GangLieMachine { owner, source, asked: false })
    }
}

#[derive(Clone)]
struct GangLieMachine {
    owner: PlayerId,
    source: PlayerId,
    asked: bool,
}

impl SkillMachine for GangLieMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::GangLie {
            owner: self.owner,
            source: self.source,
            asked: self.asked,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if !self.asked {
            self.asked = true;
            let card = ctx.draw_judge_card();
            let heart = ctx.catalog.cards[card as usize].suit.is_heart();
            ctx.to_discard(card);
            if !heart && ctx.core.player(self.source).alive {
                ctx.emit(EventData::Damage(crate::machine::damage_data(
                    Some(self.owner),
                    self.source,
                    1,
                )));
            }
        }
        Flow::Done
    }
}

// 骁果（乐进，P2/N08 标准缺失将 2026-10-07）：其他角色的结束阶段
// 开始时，可弃一张基本手牌，对其造成 1 点伤害（登记差异：上游
// chooseToDiscard 自选，原生为确认门 + 手牌基本牌选择）。
#[derive(Clone)]
struct XiaoGuoBehavior;

impl SkillBehavior for XiaoGuoBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase { player, phase: Phase::End, begin: true }
            if *player != owner
                && ctx.state.player(*player).alive
                && ctx.state.player(owner).alive
                && ctx.state.player(owner).hand.iter().any(|c| {
                    matches!(ctx.catalog.card_kind(*c), crate::catalog::CardKind::Sha | crate::catalog::CardKind::Shan | crate::catalog::CardKind::Tao | crate::catalog::CardKind::Jiu)
                }))
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        let target = match event {
            EventData::Phase { player, .. } => *player,
            _ => unreachable!(),
        };
        Box::new(XiaoGuoMachine { owner, target, stage: XiaoGuoStage::Ask })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum XiaoGuoStage {
    /// 弃牌选择（挂起 = xiaoguo_discard_basic）。
    Ask,
    /// 恢复读应答：弃置所选基本牌并对目标造成 1 点伤害。
    Read,
}

#[derive(Clone)]
struct XiaoGuoMachine {
    owner: PlayerId,
    target: PlayerId,
    stage: XiaoGuoStage,
}

impl SkillMachine for XiaoGuoMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::XiaoGuo {
            owner: self.owner,
            target: self.target,
            stage: match self.stage {
                XiaoGuoStage::Ask => XiaoGuoStageSnap::Ask,
                XiaoGuoStage::Read => XiaoGuoStageSnap::Read,
            },
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            XiaoGuoStage::Ask => {
                if !ctx.core.player(self.target).alive {
                    return Flow::Done;
                }
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .filter(|c| {
                        matches!(
                            ctx.catalog.card_kind(*c),
                            crate::catalog::CardKind::Sha
                                | crate::catalog::CardKind::Shan
                                | crate::catalog::CardKind::Tao
                                | crate::catalog::CardKind::Jiu
                        )
                    })
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return Flow::Done;
                }
                self.stage = XiaoGuoStage::Read;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: crate::choice::ChoiceSpec {
                        kind: crate::choice::ChoiceKind::SelectCards,
                        prompt: "xiaoguo_discard_basic",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            XiaoGuoStage::Read => {
                if let ChoiceOption::Card(card) = ctx.answer_first() {
                    if ctx.core.player(self.owner).hand.contains(&card) {
                        ctx.discard_from_hand(self.owner, card);
                        ctx.emit(EventData::Damage(crate::machine::damage_data(
                            Some(self.owner),
                            self.target,
                            1,
                        )));
                    }
                }
                Flow::Done
            }
        }
    }
}

// 狂斧（潘凤，P2/N08 标准缺失将 2026-10-07）：出牌阶段用【杀】对目标
// 造成伤害后触发（锁定）：目标体力值小于你则你摸两张牌，否则你失去
// 1 点体力。每回合限一次（上游为出牌阶段限一次——temp 技能到阶段结束；
// 因触发仅在自身出牌阶段，两者等价）。
#[derive(Clone)]
struct KuangFuBehavior;

impl SkillBehavior for KuangFuBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d)
            if d.source == Some(owner)
                && d.target != owner
                && !d.prevented
                && d.amount > 0
                && d.card.map(|c| ctx.catalog.card_kind(c) == crate::catalog::CardKind::Sha).unwrap_or(false)
                && ctx.state.turn.current == owner
                && ctx.state.turn.phase == Phase::Play)
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        let target = match event {
            EventData::Damage(d) => d.target,
            _ => unreachable!(),
        };
        Box::new(KuangFuMachine { owner, target })
    }
}

#[derive(Clone)]
struct KuangFuMachine {
    owner: PlayerId,
    target: PlayerId,
}

impl SkillMachine for KuangFuMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::KuangFu { owner: self.owner, target: self.target }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if !ctx.core.player(self.target).alive {
            return Flow::Done;
        }
        if ctx.core.player(self.target).hp < ctx.core.player(self.owner).hp {
            ctx.draw_cards(self.owner, 2);
        } else {
            // 失去 1 点体力（苦肉同款：濒死检查 + Dying 子事件）。
            let p = ctx.core.player_mut(self.owner);
            p.hp -= 1;
            let entering = p.hp <= 0 && !p.dying;
            if entering {
                p.dying = true;
                ctx.log(crate::LogEvent::DyingEntered { player: self.owner });
                ctx.emit(EventData::Dying { player: self.owner, source: None });
                return Flow::Yield;
            }
        }
        Flow::Done
    }
}

// 耀武（华雄，P2/N08 标准缺失将 2026-10-07）：锁定技——你受到红色
// 【杀】的伤害时，伤害来源须选择摸一张牌或回复 1 点体力（满血时无
// 回复选项；登记差异：上游 chooseDrawRecover 由 AI/玩家确认，原生为
// 强制二选）。
#[derive(Clone)]
struct YaoWuBehavior;

impl SkillBehavior for YaoWuBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d)
            if d.target == owner
                && !d.prevented
                && d.amount > 0
                && d.source.is_some()
                && d.source != Some(owner)
                && ctx.state.player(d.source.unwrap()).alive
                && d.card.map(|c| {
                    let def = &ctx.catalog.cards[c as usize];
                    matches!(ctx.catalog.card_kind(c), crate::catalog::CardKind::Sha | crate::catalog::CardKind::FireSha | crate::catalog::CardKind::LeiSha)
                        && matches!(def.suit, crate::catalog::Suit::Heart | crate::catalog::Suit::Diamond)
                }).unwrap_or(false))
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        let source = match event {
            EventData::Damage(d) => d.source.expect("filtered"),
            _ => unreachable!(),
        };
        Box::new(YaoWuMachine { owner, source, stage: YaoWuStage::Ask })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum YaoWuStage {
    /// 来源二选（挂起 = yaowu_draw_or_recover）。
    Ask,
    /// 恢复读应答：摸一张或回复 1 点体力。
    Read,
}

#[derive(Clone)]
struct YaoWuMachine {
    owner: PlayerId,
    source: PlayerId,
    stage: YaoWuStage,
}

impl SkillMachine for YaoWuMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::YaoWu {
            owner: self.owner,
            source: self.source,
            stage: match self.stage {
                YaoWuStage::Ask => YaoWuStageSnap::Ask,
                YaoWuStage::Read => YaoWuStageSnap::Read,
            },
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            YaoWuStage::Ask => {
                let p = ctx.core.player(self.source);
                let mut options = vec![ChoiceOption::Tag(0)];
                if (p.hp as u32) < p.max_hp {
                    options.push(ChoiceOption::Tag(1));
                }
                self.stage = YaoWuStage::Read;
                Flow::WaitChoice {
                    observer: self.source,
                    spec: crate::choice::ChoiceSpec {
                        kind: crate::choice::ChoiceKind::PickTag,
                        prompt: "yaowu_draw_or_recover",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            YaoWuStage::Read => {
                match ctx.answer_first() {
                    ChoiceOption::Tag(1) => {
                        ctx.recover(self.source, 1, None);
                    }
                    _ => {
                        ctx.draw_cards(self.source, 1);
                    }
                }
                Flow::Done
            }
        }
    }
}

// 淑慎（甘夫人，P2/N08 标准缺失将第二批 2026-10-07）：回复体力后，
// 可令一名其他角色摸牌——有手牌摸 1，无手牌摸 2。
#[derive(Clone)]
struct ShuShenBehavior;

impl SkillBehavior for ShuShenBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Recover { player, amount, .. }
            if *player == owner && *amount > 0 && ctx.state.player(owner).alive)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(ShuShenMachine { owner, target: None, stage: ShuShenStage::Ask })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ShuShenStage {
    /// 目标选择（挂起 = shushen_target）。
    Ask,
    /// 恢复读应答：目标摸 1（有手牌）或 2（无手牌）。
    Read,
}

#[derive(Clone)]
struct ShuShenMachine {
    owner: PlayerId,
    target: Option<PlayerId>,
    stage: ShuShenStage,
}

impl SkillMachine for ShuShenMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::ShuShen {
            owner: self.owner,
            target: self.target,
            stage: match self.stage {
                ShuShenStage::Ask => ShuShenStageSnap::Ask,
                ShuShenStage::Read => ShuShenStageSnap::Read,
            },
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            ShuShenStage::Ask => {
                let candidates: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| *p != self.owner)
                    .map(ChoiceOption::Player)
                    .collect();
                if candidates.is_empty() {
                    return Flow::Done;
                }
                self.stage = ShuShenStage::Read;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: crate::choice::ChoiceSpec {
                        kind: crate::choice::ChoiceKind::SelectPlayers,
                        prompt: "shushen_target",
                        options: candidates,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            ShuShenStage::Read => {
                if let ChoiceOption::Player(target) = ctx.answer_first() {
                    self.target = Some(target);
                    if ctx.core.player(target).alive {
                        let n = if ctx.core.player(target).hand.is_empty() { 2 } else { 1 };
                        ctx.draw_cards(target, n);
                    }
                }
                Flow::Done
            }
        }
    }
}

// 神智（甘夫人，P2/N08 标准缺失将第二批 2026-10-07）：准备阶段可弃置
// 全部手牌，若数量不少于体力值则回复 1 点体力（登记差异：原生无准备
// 阶段，以判定阶段开始为回合开始锚点——先于一切阶段结算）。
#[derive(Clone)]
struct ShenZhiBehavior;

impl SkillBehavior for ShenZhiBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase { player, phase: Phase::Judge, begin: true }
            if *player == owner && !ctx.state.player(owner).hand.is_empty())
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(ShenZhiMachine { owner })
    }
}

#[derive(Clone)]
struct ShenZhiMachine {
    owner: PlayerId,
}

impl SkillMachine for ShenZhiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::ShenZhi { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let hand: Vec<crate::ids::CardId> = ctx.core.player(self.owner).hand.clone();
        if hand.is_empty() {
            return Flow::Done;
        }
        let enough = hand.len() as i32 >= ctx.core.player(self.owner).hp;
        for card in hand {
            ctx.discard_from_hand(self.owner, card);
        }
        if enough && ctx.core.player(self.owner).alive {
            ctx.recover(self.owner, 1, None);
        }
        Flow::Done
    }
}

// 恂恂（李典，P2/N08 标准缺失将第二批 2026-10-07）：摸牌阶段开始时
// 观看牌堆顶四张，选两张置顶（先选先摸）其余置底，再正常摸牌。
// 阶段机器在固定摸牌之后执行：先把固定摸的 2 张还回牌堆顶（恢复
// 摸牌前牌序），此后与上游观看对象一致（净效果等价）。
#[derive(Clone)]
struct XunXunBehavior;

impl SkillBehavior for XunXunBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase { player, phase: Phase::Draw, begin: true }
            if *player == owner && ctx.state.deck.len() >= 4)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(XunXunMachine { owner, top: Vec::new(), stage: XunXunStage::Undo })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum XunXunStage {
    /// 还回固定摸牌并观看顶四张（打开选择）。
    Undo,
    /// 恢复读应答：两张置顶（先选先摸）其余置底，然后摸 2。
    Arrange,
}

#[derive(Clone)]
struct XunXunMachine {
    owner: PlayerId,
    top: Vec<crate::ids::CardId>,
    stage: XunXunStage,
}

impl SkillMachine for XunXunMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::XunXun {
            owner: self.owner,
            stage: match self.stage {
                XunXunStage::Undo => XunXunStageSnap::Undo,
                XunXunStage::Arrange => XunXunStageSnap::Arrange,
            },
            top: self.top.iter().copied().map(|c| c as u32).collect(),
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            XunXunStage::Undo => {
                // 还回固定摸的 2 张（手牌末尾=刚摸的牌；按弹出序压回牌堆
                // 即恢复摸牌前牌序，观星同款：牌库顶在末尾）。
                for _ in 0..2 {
                    if let Some(c) = ctx.core.player_mut(self.owner).hand.pop() {
                        ctx.core.deck.push(c);
                    }
                }
                let take = 4.min(ctx.core.deck.len());
                let new_len = ctx.core.deck.len() - take;
                self.top = ctx.core.deck.split_off(new_len);
                let options: Vec<ChoiceOption> =
                    self.top.iter().copied().map(ChoiceOption::Card).collect();
                self.stage = XunXunStage::Arrange;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: crate::choice::ChoiceSpec {
                        kind: crate::choice::ChoiceKind::SelectCards,
                        prompt: "xunxun_arrange",
                        options,
                        min: 2,
                        max: 2,
                        allow_cancel: false,
                    },
                }
            }
            XunXunStage::Arrange => {
                let chosen: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                let rest: Vec<CardId> = self
                    .top
                    .iter()
                    .copied()
                    .filter(|c| !chosen.contains(c))
                    .collect();
                // 未选置底（原序前插）；选中的置顶（先选先摸 → 逆序压栈）。
                for c in &rest {
                    ctx.core.deck.insert(0, *c);
                }
                for c in chosen.iter().rev() {
                    ctx.core.deck.push(*c);
                }
                ctx.draw_cards(self.owner, 2);
                Flow::Done
            }
        }
    }
}

// 忘隙（李典，P2/N08 标准缺失将第二批 2026-10-07）：对其他角色造成
// 伤害或受到其伤害后，可与该角色各摸一张牌（登记差异：摸牌顺序为
// 座位号升序，上游按座位序自当前回合角色起算）。
#[derive(Clone)]
struct WangXiBehavior;

impl SkillBehavior for WangXiBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d)
            if d.amount > 0 && !d.prevented
                && d.source.is_some() && d.source != Some(d.target)
                && (d.target == owner || d.source == Some(owner))
                && ctx.state.player(d.target).alive
                && ctx.state.player(d.source.unwrap()).alive)
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        let other = match event {
            EventData::Damage(d) => {
                if d.target == owner { d.source.expect("filtered") } else { d.target }
            }
            _ => unreachable!(),
        };
        Box::new(WangXiMachine { owner, other })
    }
}

#[derive(Clone)]
struct WangXiMachine {
    owner: PlayerId,
    other: PlayerId,
}

impl SkillMachine for WangXiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::WangXi { owner: self.owner, other: self.other }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 各摸 1（座位号升序；目标已亡则只摸自己的）。
        let mut order = vec![self.owner, self.other];
        order.sort();
        order.dedup();
        for p in order {
            if ctx.core.player(p).alive {
                ctx.draw_cards(p, 1);
            }
        }
        Flow::Done
    }
}

// 机捷（伊籍，P2/N08 标准缺失将第四批 2026-10-07）：出牌阶段限一次
// 主动技——取牌堆底一张牌交给一名任意存活角色（上游 chooseTarget 无
// 过滤器，AI 含自引加成，原生同允许自选）。
#[derive(Clone)]
struct JiJieBehavior;

impl SkillBehavior for JiJieBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(JiJieMachine { owner, card: None, stage: JiJieStage::Ask })
    }

    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        if !ctx.state.player(owner).alive || ctx.state.deck.is_empty() {
            return None; // 无牌可授
        }
        Some(Box::new(JiJieMachine { owner, card: None, stage: JiJieStage::Ask }))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum JiJieStage {
    /// 目标选择（挂起 = jijie_give_target；牌堆底牌已取入手上）。
    Ask,
    /// 恢复读应答：交牌。
    Read,
}

#[derive(Clone)]
struct JiJieMachine {
    owner: PlayerId,
    card: Option<crate::ids::CardId>,
    stage: JiJieStage,
}

impl SkillMachine for JiJieMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::JiJie {
            owner: self.owner,
            card: self.card,
            stage: match self.stage {
                JiJieStage::Ask => JiJieStageSnap::Ask,
                JiJieStage::Read => JiJieStageSnap::Read,
            },
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            JiJieStage::Ask => {
                let candidates: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .map(ChoiceOption::Player)
                    .collect();
                if candidates.is_empty() {
                    return Flow::Done;
                }
                // 取牌堆底（deck 首位）。
                self.card = Some(ctx.core.deck.remove(0));
                self.stage = JiJieStage::Read;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: crate::choice::ChoiceSpec {
                        kind: crate::choice::ChoiceKind::SelectPlayers,
                        prompt: "jijie_give_target",
                        options: candidates,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            JiJieStage::Read => {
                if let (ChoiceOption::Player(target), Some(card)) =
                    (ctx.answer_first(), self.card)
                {
                    if ctx.core.player(target).alive {
                        ctx.core.player_mut(target).hand.push(card);
                        ctx.out.push(crate::LogEvent::CardsDrawn {
                            player: target,
                            cards: vec![card],
                        });
                    } else {
                        ctx.to_discard(card);
                    }
                }
                self.card = None;
                Flow::Done
            }
        }
    }
}

// 急援（伊籍，P2/N08 标准缺失将第四批 2026-10-07）：一名角色进入濒死
// 状态时，可令其摸一张牌（登记差异：仅濒死臂）。
#[derive(Clone)]
struct JiYuanBehavior;

impl SkillBehavior for JiYuanBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Dying { .. }) && ctx.state.player(owner).alive
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(JiYuanMachine { owner })
    }
}

#[derive(Clone)]
struct JiYuanMachine {
    owner: PlayerId,
}

impl SkillMachine for JiYuanMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::JiYuan { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 濒死者为事件目标（任一角色濒死时其摸 1）。
        if let EventData::Dying { player, .. } =
            ctx.res.stack.last().expect("dying frame").data
        {
            if ctx.core.player(player).dying {
                ctx.draw_cards(player, 1);
            }
        }
        Flow::Done
    }
}

// 妄尊（袁术，P2/N08 标准缺失将收尾批 2026-10-07）：锁定技——其他
// 角色的回合开始（判定阶段开始锚点）时，若其体力值大于你，你摸一张；
// 该角色为主公时再摸一张并给其「妄尊印」（手牌上限 -1 到其回合结束）。
// 主公资格沿激将 ADR 0776 惯例（原型模式视为真）。
#[derive(Clone)]
struct WangZunBehavior;

impl SkillBehavior for WangZunBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase { player, phase: Phase::Judge, begin: true }
            if *player != owner
                && ctx.state.player(*player).alive
                && ctx.state.player(owner).alive
                && ctx.state.player(*player).hp > ctx.state.player(owner).hp)
    }

    fn start(&self, owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        let target = match event {
            EventData::Phase { player, .. } => *player,
            _ => unreachable!(),
        };
        Box::new(WangZunMachine { owner, target })
    }
}

#[derive(Clone)]
struct WangZunMachine {
    owner: PlayerId,
    target: PlayerId,
}

impl SkillMachine for WangZunMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::WangZun { owner: self.owner, target: self.target }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        ctx.draw_cards(self.owner, 1);
        if ctx.core.is_lord(self.target) {
            ctx.draw_cards(self.owner, 1);
            if let Some(mark) = ctx.catalog.skill_by_name(WANG_ZUN_MARK) {
                ctx.core.skills.insert(
                    (self.target, mark),
                    crate::state::SkillInstance {
                        owner: self.target,
                        enabled: true,
                        once_used: false,
                        expires_end_of_turn: true,
                        rounds: 0,
                    },
                );
            }
        }
        Flow::Done
    }
}

// 同疾（袁术，P2/N08 标准缺失将收尾批 2026-10-07）：锁定技——其他
// 角色使用【杀】指定唯一目标（你不在目标内、目标对你 inRange）时，
// 目标须选择弃置一张牌或取消：弃置则该杀转移指向你（改写 CardUsed
// 帧载荷，sha 结算机器在 Intrinsic 阶段才读取目标）。
#[derive(Clone)]
struct TongJiBehavior;

impl SkillBehavior for TongJiBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        let matches_sha = matches!(event, EventData::CardUsed(u)
            if {
                let kind = u.use_as.unwrap_or_else(|| ctx.catalog.card_kind(u.card));
                matches!(kind, crate::catalog::CardKind::Sha | crate::catalog::CardKind::FireSha | crate::catalog::CardKind::LeiSha)
                    && u.targets.len() == 1
                    && u.player != owner
                    && !u.targets.contains(&owner)
            });
        if !matches_sha {
            return false;
        }
        let EventData::CardUsed(u) = event else { return false };
        let target = u.targets[0];
        let owner_alive = ctx.state.player(owner).alive;
        let target_ok = ctx.state.player(target).alive
            && (!ctx.state.player(target).hand.is_empty()
                || !ctx.state.player(target).equips.is_empty());
        let in_range = ctx
            .state
            .computed_distance(ctx.catalog, target, owner)
            <= ctx.state.attack_range(ctx.catalog, target);
        owner_alive && target_ok && in_range
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(TongJiMachine { owner, stage: TongJiStage::Ask })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TongJiStage {
    /// 目标弃牌/取消（挂起 = tongji_transfer_discard）。
    Ask,
    /// 恢复读应答：弃置则转移目标。
    Read,
}

#[derive(Clone)]
struct TongJiMachine {
    owner: PlayerId,
    stage: TongJiStage,
}

impl SkillMachine for TongJiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::TongJi { owner: self.owner, stage: match self.stage {
            TongJiStage::Ask => TongJiStageSnap::Ask,
            TongJiStage::Read => TongJiStageSnap::Read,
        } }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            TongJiStage::Ask => {
                let Ok(u) = ctx.current_card_use() else { return Flow::Done };
                let target = u.targets[0];
                let mut options: Vec<ChoiceOption> = ctx
                    .core
                    .player(target)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                options.extend(
                    ctx.core.player(target).equips.iter().copied().map(ChoiceOption::Card),
                );
                if options.is_empty() {
                    return Flow::Done;
                }
                self.stage = TongJiStage::Read;
                Flow::WaitChoice {
                    observer: target,
                    spec: crate::choice::ChoiceSpec {
                        kind: crate::choice::ChoiceKind::SelectCards,
                        prompt: "tongji_transfer_discard",
                        options,
                        min: 0,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            TongJiStage::Read => {
                let Ok(u) = ctx.current_card_use() else { return Flow::Done };
                let target = u.targets[0];
                // 空应答 = 目标取消（不弃牌不转移）。
                if let Some(ChoiceOption::Card(card)) = ctx.answer().first().copied() {
                    let hand = ctx.core.player(target).hand.contains(&card);
                    if hand {
                        ctx.discard_from_hand(target, card);
                    } else if let Some(pos) = ctx
                        .core
                        .player(target)
                        .equips
                        .iter()
                        .position(|c| *c == card)
                    {
                        let equip = ctx.core.player_mut(target).equips.remove(pos);
                        crate::kernel::revoke_equip_skills(ctx.core, ctx.catalog, target, equip);
                        ctx.to_discard(equip);
                    } else {
                        return Flow::Done;
                    }
                    // 转移：改写 CardUsed 帧载荷目标（sha 结算尚未开始）。
                    let u = ctx.current_card_use_mut();
                    u.targets[0] = self.owner;
                }
                Flow::Done
            }
        }
    }
}

// 丈八蛇矛（配套主动技，批③-b 第七把）：两张手牌当【杀】使用——
// 出牌阶段限一次不限（上游同款）；Stage 1 选两张手牌，Stage 2 选目标
//（射程内其他角色），首张作 declared-Sha 载荷进入杀结算，次张弃置。
#[derive(Clone)]
struct ZhangbaBehavior;

impl SkillBehavior for ZhangbaBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(ZhangbaMachine {
            owner,
            cards: Vec::new(),
            stage: ZhangbaStage::Cards,
        })
    }

    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        // 资格：两张手牌 + 攻击范围内有其他角色。
        if !ctx.state.player(owner).alive || ctx.state.player(owner).hand.len() < 2 {
            return None;
        }
        let in_range = ctx.state.alive_players().any(|p| {
            p != owner
                && ctx.state.computed_distance(ctx.catalog, owner, p)
                    <= ctx.state.attack_range(ctx.catalog, owner)
        });
        if !in_range {
            return None;
        }
        Some(Box::new(ZhangbaMachine {
            owner,
            cards: Vec::new(),
            stage: ZhangbaStage::Cards,
        }))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum ZhangbaStage {
    /// 选两张手牌（打开询问）。
    Cards,
    /// 选目标（打开询问）。
    Target,
    /// 读目标应答 → 发射 declared-Sha CardUsed。
    TargetRead,
}

#[derive(Clone)]
struct ZhangbaMachine {
    owner: PlayerId,
    cards: Vec<crate::ids::CardId>,
    stage: ZhangbaStage,
}

impl SkillMachine for ZhangbaMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::Zhangba {
            owner: self.owner,
            cards: self.cards.iter().copied().map(|c| c as u32).collect(),
            stage: match self.stage {
                ZhangbaStage::Cards => ZhangbaStageSnap::Cards,
                ZhangbaStage::Target => ZhangbaStageSnap::Target,
                ZhangbaStage::TargetRead => ZhangbaStageSnap::TargetRead,
            },
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            ZhangbaStage::Cards => {
                let options: Vec<ChoiceOption> = ctx
                    .core
                    .player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                if options.len() < 2 {
                    return Flow::Done;
                }
                self.stage = ZhangbaStage::Target;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "zhangba_pick_two",
                        options,
                        min: 2,
                        max: 2,
                        allow_cancel: false,
                    },
                }
            }
            ZhangbaStage::Target => {
                // 恢复：读两张手牌应答 → 打开目标询问。
                let picked: Vec<crate::ids::CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if picked.len() != 2 {
                    return Flow::Done;
                }
                self.cards = picked;
                let candidates: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| {
                        *p != self.owner
                            && ctx
                                .core
                                .computed_distance(ctx.catalog, self.owner, *p)
                                <= ctx.core.attack_range(ctx.catalog, self.owner)
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                if candidates.is_empty() {
                    return Flow::Done;
                }
                self.stage = ZhangbaStage::TargetRead;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "zhangba_sha_target",
                        options: candidates,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            ZhangbaStage::TargetRead => {
                if let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied()
                {
                    if ctx.core.player(target).alive {
                        let [c1, c2] = self.cards.as_slice() else {
                            return Flow::Done;
                        };
                        let (c1, c2) = (*c1, *c2);
                        // 次张弃为转化素材；首张作 declared-Sha 载荷进入
                        // 杀结算（出闪/贯石斧/麒麟弓/青龙全流程重走）。
                        ctx.discard_from_hand(self.owner, c2);
                        if ctx.core.player(self.owner).hand.contains(&c1) {
                            ctx.core.declared_kinds.insert(c1, CardKind::Sha);
                            ctx.emit(EventData::CardUsed(
                                crate::machine::card_use_as(
                                    c1,
                                    self.owner,
                                    vec![target],
                                    Some(CardKind::Sha),
                                ),
                            ));
                        }
                    }
                }
                Flow::Done
            }
        }
    }
}

// 突袭（张辽）：摸牌阶段开始时，可放弃摸牌，改为获得至多两名
// 其他角色的各一张手牌（各区首张，Dismantle 同款确定性取牌；
// 目标询问弃权 = 不发动，保留摸牌）。
#[derive(Clone)]
struct TuXiBehavior;
impl SkillBehavior for TuXiBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase {
            player,
            phase: Phase::Draw,
            begin: true,
        } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(TuXiMachine {
            owner,
            stage: TuXiStage::Ask,
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum TuXiStage {
    /// 挂起询问 = tuxi_target（选择 1–2 名有手牌的其他角色）。
    Ask,
    /// 恢复读应答：弃固定摸牌并取牌。
    Read,
}

#[derive(Clone)]
struct TuXiMachine {
    owner: PlayerId,
    stage: TuXiStage,
}

impl SkillMachine for TuXiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::TuXi {
            owner: self.owner,
            stage: match self.stage {
                TuXiStage::Ask => TuXiStageSnap::Ask,
                TuXiStage::Read => TuXiStageSnap::Read,
            },
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            TuXiStage::Ask => {
                let candidates: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| {
                        *p != self.owner && !ctx.core.player(*p).hand.is_empty()
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                if candidates.is_empty() {
                    return Flow::Done;
                }
                self.stage = TuXiStage::Read;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "tuxi_target",
                        options: candidates,
                        min: 1,
                        max: 2,
                        allow_cancel: true,
                    },
                }
            }
            TuXiStage::Read => {
                let targets: Vec<PlayerId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    })
                    .collect();
                if targets.is_empty() {
                    return Flow::Done; // 弃权：保留本阶段摸牌
                }
                // 放弃摸牌：弃置本阶段固定摸到的牌（手牌末尾至多 2 张，
                // 触发机器在阶段固定摸牌之后执行，末尾即刚摸的牌）
                let mut dropped = Vec::new();
                {
                    let p = ctx.core.player_mut(self.owner);
                    for _ in 0..2 {
                        if let Some(c) = p.hand.pop() {
                            dropped.push(c);
                        }
                    }
                }
                if !dropped.is_empty() {
                    for c in &dropped {
                        ctx.to_discard(*c);
                    }
                    ctx.log(LogEvent::CardDiscarded {
                        player: self.owner,
                        cards: dropped,
                    });
                }
                let mut gained = Vec::new();
                for t in targets {
                    if t == self.owner || !ctx.core.player(t).alive {
                        continue;
                    }
                    if let Some(c) = ctx.core.player(t).hand.first().copied() {
                        ctx.give_hand_card(t, self.owner, c);
                        gained.push(c);
                    }
                }
                if !gained.is_empty() {
                    ctx.log(LogEvent::CardsDrawn {
                        player: self.owner,
                        cards: gained,
                    });
                }
                Flow::Done
            }
        }
    }
}

// 裸衣（许褚）：摸牌阶段开始时，可少摸一张牌，然后本回合
// 【杀】或【决斗】造成的伤害 +1。加成旗标复用 once_per_turn 的
// `once_used`（确认时置位；下回合判定阶段开始时随
// `reset_turn_limits` 清除），伤害结算侧由 kernel `luoyi_bonus` 读取。
#[derive(Clone)]
struct LuoYiBehavior;

impl SkillBehavior for LuoYiBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase {
            player,
            phase: Phase::Draw,
            begin: true,
        } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(LuoYiMachine { owner, asked: false })
    }
}

#[derive(Clone)]
struct LuoYiMachine {
    owner: PlayerId,
    asked: bool,
}

impl SkillMachine for LuoYiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::LuoYi { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if !self.asked {
            self.asked = true;
            // 少摸一张：弃置本阶段固定摸到的最后一张（手牌末尾）
            let dropped = ctx.core.player_mut(self.owner).hand.pop();
            if let Some(c) = dropped {
                ctx.to_discard(c);
                ctx.log(LogEvent::CardDiscarded {
                    player: self.owner,
                    cards: vec![c],
                });
            }
        }
        Flow::Done
    }
}

// 仁德（刘备）：出牌阶段限一次的主动技——将任意张手牌交给一名
// 其他角色；给出的牌不少于两张时回复 1 点体力（体力已满则无效）。
#[derive(Clone)]
struct RendeBehavior;

impl SkillBehavior for RendeBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false // 主动技：不随事件触发
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(RendeMachine {
            owner,
            stage: RendeStage::SelectCardsAsk,
            cards: Vec::new(),
        })
    }

    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        let p = ctx.state.player(owner);
        if !p.alive || p.hand.is_empty() {
            return None; // 无手牌可给：无法发动
        }
        Some(Box::new(RendeMachine {
            owner,
            stage: RendeStage::SelectCardsAsk,
            cards: Vec::new(),
        }))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum RendeStage {
    /// 挂起询问 = rende_cards（选择给出的手牌，空应答 = 放弃）。
    SelectCardsAsk,
    /// 消费 rende_cards 应答并挂起 rende_target 询问。
    TargetAsk,
    /// 消费 rende_target 应答并交付（≥2 张则回复 1 点体力）。
    TargetRead,
}

#[derive(Clone)]
struct RendeMachine {
    owner: PlayerId,
    stage: RendeStage,
    /// SelectCards 应答选中的牌（待交付）。
    cards: Vec<CardId>,
}

impl SkillMachine for RendeMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::Rende {
            owner: self.owner,
            stage: match self.stage {
                RendeStage::SelectCardsAsk => RendeStageSnap::SelectCardsAsk,
                RendeStage::TargetAsk => RendeStageSnap::TargetAsk,
                RendeStage::TargetRead => RendeStageSnap::TargetRead,
            },
            cards: self.cards.clone(),
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            RendeStage::SelectCardsAsk => {
                if ctx.core.player(self.owner).hand.is_empty() {
                    return Flow::Done;
                }
                let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                let max_cards = options.len();
                self.stage = RendeStage::TargetAsk;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "rende_cards",
                        options,
                        min: 0,
                        max: max_cards,
                        allow_cancel: true,
                    },
                }
            }
            RendeStage::TargetAsk => {
                self.cards = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if self.cards.is_empty() {
                    return Flow::Done; // 空应答：放弃发动
                }
                let candidates: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| *p != self.owner)
                    .map(ChoiceOption::Player)
                    .collect();
                if candidates.is_empty() {
                    return Flow::Done;
                }
                self.stage = RendeStage::TargetRead;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "rende_target",
                        options: candidates,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            RendeStage::TargetRead => {
                let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                    return Flow::Done; // 弃权：不交付
                };
                if !ctx.core.player(target).alive {
                    return Flow::Done;
                }
                // 仁德累计（ADR 0776 对齐上游 rende1/content）：按本
                // 出牌阶段累计给出数判定——此前 <2 且累计后 ≥2 时回复
                // 1 点（每次发动都可给牌，无每回合一次限制）。
                let num_before = ctx.core.turn.rende_given;
                for c in &self.cards {
                    ctx.give_hand_card(self.owner, target, *c);
                }
                let given = self.cards.len() as u8;
                ctx.core.turn.rende_given = num_before.saturating_add(given);
                if num_before < 2 && num_before + given > 1 {
                    ctx.recover(self.owner, 1, None);
                }
                Flow::Done
            }
        }
    }
}

// 激将（刘备）：主公技，出牌阶段限一次——请求一名已声明蜀势力的
// 其他角色打出一张杀（其确认后置入你的手牌，视为由你使用；
// 原生简化：获得后走正常出杀流程与次数校验，见 ADR 0760）。
#[derive(Clone)]
struct JijiangBehavior;

impl SkillBehavior for JijiangBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(JijiangMachine {
            owner,
            stage: JijiangStage::HelperAsk,
            queue: None,
            helper: None,
            sha: None,
            inner: None,
        })
    }

    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        // 资格（ADR 0776 对齐上游 zhuSkill + filter）：主公资格
        // （身份局须主公身份）+ 存在蜀势力角色。**不以暗手牌预筛**——
        // 有无杀由被询问者逐人应答时才知道。
        if !ctx.state.is_lord(owner) {
            return None;
        }
        let has_shu = ctx.state.alive_players().any(|p| {
            p != owner && ctx.state.effective_faction(ctx.catalog, p) == Some(crate::catalog::Faction::Shu)
        });
        if !has_shu {
            return None; // 无可请求的蜀势力角色
        }
        Some(Box::new(JijiangMachine {
            owner,
            stage: JijiangStage::HelperAsk,
            queue: None,
            helper: None,
            sha: None,
            inner: None,
        }))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum JijiangStage {
    /// 逐人询问蜀势力角色（挂起 = jijiang_respond；helper 记当前被问者）。
    HelperAsk,
    /// 消费 jijiang_target 应答：借牌直结算（内嵌 ShaResolve，视为使用）。
    TargetAsk,
}

struct JijiangMachine {
    owner: PlayerId,
    stage: JijiangStage,
    /// 逐人询问队列（ADR 0776）：`None` = 未开始；`Some(q)` = 剩余待问
    /// 的蜀势力角色（owner 下家起按座位序）；`Some(空)` = 全部拒绝。
    /// 不以暗手牌预筛。
    queue: Option<Vec<PlayerId>>,
    /// 当前被询问/已响应的蜀势力角色。
    helper: Option<PlayerId>,
    /// 借出的杀（TargetAsk 起生效，置于桌面）。
    sha: Option<CardId>,
    /// 直结算内嵌机器（Launch 后委托）。
    inner: Option<Box<dyn SkillMachine>>,
}

impl JijiangMachine {
    /// 询问队列：owner 下家起按座位序的蜀势力存活角色。
    fn build_queue(&self, ctx: &Ctx<'_>) -> Vec<PlayerId> {
        ctx.core
            .seat_order_from(ctx.core.next_alive(self.owner))
            .into_iter()
            .filter(|p| {
                *p != self.owner
                    && ctx.core.player(*p).alive
                    && ctx.core.effective_faction(ctx.catalog, *p) == Some(crate::catalog::Faction::Shu)
            })
            .collect()
    }
}

impl SkillMachine for JijiangMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(Self {
            owner: self.owner,
            stage: self.stage,
            queue: self.queue.clone(),
            helper: self.helper,
            sha: self.sha,
            inner: self.inner.as_ref().map(|m| m.fork()),
        })
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::Jijiang {
            owner: self.owner,
            stage: match self.stage {
                JijiangStage::HelperAsk => JijiangStageSnap::HelperAsk,
                JijiangStage::TargetAsk => JijiangStageSnap::TargetAsk,
            },
            queue: self.queue.clone(),
            helper: self.helper,
            sha: self.sha,
            inner: self.inner.as_ref().map(|m| Box::new(m.snapshot())),
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 内嵌直结算机器存在即全权委托（OptionalGate 同款）
        if let Some(inner) = self.inner.as_mut() {
            let flow = inner.step(ctx);
            if matches!(flow, Flow::Done) {
                self.inner = None;
                return Flow::Done;
            }
            return flow;
        }
        match self.stage {
            JijiangStage::HelperAsk => {
                // 本窗口首次进入：构建逐人询问队列
                if self.queue.is_none() {
                    self.queue = Some(self.build_queue(ctx));
                }
                // 消费当前被询问者应答（若有）
                if let Some(helper) = self.helper.take() {
                    let confirmed = ctx.answer().first() == Some(&ChoiceOption::Confirm);
                    let alive = ctx.core.player(helper).alive;
                    let has_sha = alive
                        && ctx
                            .core
                            .player(helper)
                            .hand
                            .iter()
                            .any(|c| ctx.catalog.card_kind(*c) == CardKind::Sha);
                    if confirmed && has_sha {
                        // 响应：借杀上桌面并询问目标（同 ADR 0760 直结算）
                        self.helper = Some(helper);
                        let sha = ctx
                            .core
                            .player(helper)
                            .hand
                            .iter()
                            .copied()
                            .find(|c| ctx.catalog.card_kind(*c) == CardKind::Sha)
                            .expect("has_sha checked above");
                        {
                            let p = ctx.core.player_mut(helper);
                            p.hand.retain(|x| *x != sha);
                        }
                        ctx.core.table.push(sha);
                        self.sha = Some(sha);
                        // 目标候选：其他存活且距离 ≤ 射程（借牌仍守目标合法性）
                        let candidates: Vec<ChoiceOption> = ctx
                            .core
                            .alive_players()
                            .filter(|p| {
                                *p != self.owner
                                    && ctx
                                        .core
                                        .computed_distance(ctx.catalog, self.owner, *p)
                                        <= ctx.core.attack_range(ctx.catalog, self.owner)
                            })
                            .map(ChoiceOption::Player)
                            .collect();
                        if candidates.is_empty() {
                            // 无合法目标：借牌归还 helper，问下一位
                            let c = self.sha.take().expect("sha set above");
                            ctx.core.table.retain(|x| *x != c);
                            ctx.core.player_mut(helper).hand.push(c);
                        } else {
                            self.stage = JijiangStage::TargetAsk;
                            return Flow::WaitChoice {
                                observer: self.owner,
                                spec: ChoiceSpec {
                                    kind: ChoiceKind::SelectPlayers,
                                    prompt: "jijiang_target",
                                    options: candidates,
                                    min: 1,
                                    max: 1,
                                    allow_cancel: true,
                                },
                            };
                        }
                    }
                    // Pass / 确认但无杀：继续问下一位
                }
                // 逐人询问下一位
                let queue = self.queue.as_mut().expect("queue built above");
                if let Some(pos) = queue.iter().position(|p| ctx.core.player(*p).alive) {
                    let next = queue.remove(pos);
                    self.helper = Some(next);
                    return Flow::WaitChoice {
                        observer: next,
                        spec: ChoiceSpec {
                            kind: ChoiceKind::Confirm,
                            prompt: "jijiang_respond",
                            options: vec![ChoiceOption::Confirm, ChoiceOption::Pass],
                            min: 1,
                            max: 1,
                            allow_cancel: false,
                        },
                    };
                }
                // 全部拒绝：取消本次发动（上游语义——无人响应则使用取消，
                // 不消耗每回合一次）
                let jj = ctx.catalog.skill_by_name(crate::content::standard::JI_JIANG);
                if let Some(d) = jj {
                    if let Some(inst) = ctx.core.skills.get_mut(&(self.owner, d)) {
                        inst.once_used = false;
                    }
                }
                Flow::Done
            }
            JijiangStage::TargetAsk => {
                // 消费 jijiang_target：借牌直结算（视为刘备使用杀）
                let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                    // 弃权：借牌归还 helper
                    if let Some(helper) = self.helper {
                        if let Some(c) = self.sha.take() {
                            ctx.core.table.retain(|x| *x != c);
                            ctx.core.player_mut(helper).hand.push(c);
                        }
                    }
                    return Flow::Done;
                };
                let sha = self.sha.take().expect("sha set at TargetAsk");
                // card, source, target, nature, asked, armor_judged,
                // dodges_used, tieji_red, liuli_done, liuli_stage, liuli_card,
                // hujia_queue, hujia_holder, jiu_bonus, asking_qilin,
                // qilin_horse, asking_hanbing, asking_guanshi, guanshi_cards,
                // asking_qinglong, asking_cixiong, cixiong_done,
                // extra_targets, hanbing_pending
                // card, source, target, nature, asked, armor_judged,
                // dodges, tieji, liuli_done, liuli_stage, liuli_card,
                // hujia_queue, hujia_holder, jiu_bonus, asking_qilin,
                // qilin_horse, asking_hanbing, asking_fengxueren,
                // asking_guanshi, guanshi_cards, asking_qinglong,
                // asking_cixiong, cixiong_done, extra_targets,
                // hanbing_pending
                let machine = crate::kernel::ShaResolveMachine::new(
                    sha,
                    self.owner,
                    target,
                    crate::event::DamageNature::Normal,
                    false,
                    false,
                    0,
                    None,
                    false,
                    0,
                    None,
                    None,
                    None,
                    false,
                    false,
                    None,
                    false,
                    false,
                    false,
                    Vec::new(),
                    false,
                    false,
                    false,
                    Vec::new(),
                    0,
                    false,
                    false,
                    false,
                    false,
                    None,
                    false,
                    false,
                    None,
                );
                self.inner = Some(Box::new(machine));
                let flow = self.inner.as_mut().expect("just set").step(ctx);
                if matches!(flow, Flow::Done) {
                    self.inner = None;
                    return Flow::Done;
                }
                flow
            }
        }
    }
}


/// 性别门控助手：指定座位的双将槽中任一武将为男性。
fn seat_is_male(ctx: &Ctx<'_>, p: PlayerId) -> bool {
    slots_is_male(&ctx.core.player(p).general_slots)
}

/// 将槽性别判定（非国战局无将槽 → 门控不适用，默认 true）。
fn slots_is_male(slots: &[Option<u16>; 2]) -> bool {
    if slots.iter().all(|s| s.is_none()) {
        return true;
    }
    slots.iter().flatten().any(|g| {
        crate::content::standard::general_gender(*g)
            == Some(crate::content::standard::Gender::Male)
    })
}

// 结姻（孙尚香）：出牌阶段限一次——弃两张手牌，与一名**受伤的**
// 其他男性角色各回复 1 点体力（v202 对齐上游 filterTarget：male +
// hp<maxHp + 非自己；上游无「已婚」「装备区有牌」条件，旧登记撤销）。
#[derive(Clone)]
struct JieYinBehavior;

impl SkillBehavior for JieYinBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(JieYinMachine { owner, stage: JieYinStage::CardsAsk, cards: Vec::new() })
    }

    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        let p = ctx.state.player(owner);
        if !p.alive || p.hand.len() < 2 {
            return None;
        }
        if !ctx.state.alive_players().any(|t| {
            t != owner
                && slots_is_male(&ctx.state.player(t).general_slots)
                && ctx.state.player(t).hp < ctx.state.player(t).max_hp as i32
        }) {
            return None;
        }
        Some(Box::new(JieYinMachine { owner, stage: JieYinStage::CardsAsk, cards: Vec::new() }))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum JieYinStage {
    /// 挂起询问 = jieyin_cards（弃两张手牌；不足两枚或取消 = 放弃）。
    CardsAsk,
    /// 消费 jieyin_cards 并挂起 jieyin_target 询问。
    TargetAsk,
    /// 消费 jieyin_target：双方各回复 1 点体力。
    TargetRead,
}

#[derive(Clone)]
struct JieYinMachine {
    owner: PlayerId,
    stage: JieYinStage,
    cards: Vec<CardId>,
}

impl SkillMachine for JieYinMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::JieYin {
            owner: self.owner,
            stage: match self.stage {
                JieYinStage::CardsAsk => JieYinStageSnap::CardsAsk,
                JieYinStage::TargetAsk => JieYinStageSnap::TargetAsk,
                JieYinStage::TargetRead => JieYinStageSnap::TargetRead,
            },
            cards: self.cards.clone(),
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            JieYinStage::CardsAsk => {
                let hand = ctx.core.player(self.owner).hand.clone();
                if hand.len() < 2 {
                    return Flow::Done;
                }
                let options: Vec<ChoiceOption> =
                    hand.iter().copied().map(ChoiceOption::Card).collect();
                self.stage = JieYinStage::TargetAsk;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "jieyin_cards",
                        options,
                        min: 2,
                        max: 2,
                        allow_cancel: true,
                    },
                }
            }
            JieYinStage::TargetAsk => {
                self.cards = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                if self.cards.len() < 2 {
                    return Flow::Done;
                }
                // 目标门控（v202 对齐上游 filterTarget）：其他受伤男性
                let candidates: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|t| {
                        *t != self.owner
                            && seat_is_male(ctx, *t)
                            && ctx.core.player(*t).hp < ctx.core.player(*t).max_hp as i32
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                if candidates.is_empty() {
                    return Flow::Done;
                }
                self.stage = JieYinStage::TargetRead;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "jieyin_target",
                        options: candidates,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            JieYinStage::TargetRead => {
                let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                if !ctx.core.player(target).alive {
                    return Flow::Done;
                }
                for c in &self.cards {
                    ctx.discard_from_hand(self.owner, *c);
                }
                ctx.recover(self.owner, 1, None);
                ctx.recover(target, 1, None);
                Flow::Done
            }
        }
    }
}

// 反间（周瑜）：出牌阶段限一次——一名其他角色先选一种花色（v203 对齐
// 上游 chooseControl：♥/♦/♣/♠），再从周瑜手牌中任选一张获得；花色与所
// 选不符时受到周瑜造成的 1 点伤害（上游条件 damage，花色相符无事）。
#[derive(Clone)]
struct FanJianBehavior;

impl SkillBehavior for FanJianBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(FanJianMachine { owner, stage: FanJianStage::TargetAsk, target: None, chosen_suit: None })
    }

    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        let p = ctx.state.player(owner);
        if !p.alive || p.hand.is_empty() {
            return None;
        }
        if !ctx.state.alive_players().any(|t| t != owner) {
            return None;
        }
        Some(Box::new(FanJianMachine { owner, stage: FanJianStage::TargetAsk, target: None, chosen_suit: None }))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FanJianStage {
    /// 挂起询问 = fanjian_target（选择一名其他角色；取消 = 放弃）。
    TargetAsk,
    /// 挂起询问 = fanjian_suit（目标选一种花色；v203 对齐上游）。
    SuitAsk,
    /// 挂起询问 = fanjian_pick（目标选周瑜一张手牌获得）。
    PickAsk,
    /// 消费 fanjian_pick：交牌并按花色比对条件伤害。
    Consume,
}

#[derive(Clone)]
struct FanJianMachine {
    owner: PlayerId,
    stage: FanJianStage,
    target: Option<PlayerId>,
    /// 目标所选花色 tag（0=♥ 1=♦ 2=♣ 3=♠，上游 controls 序）。
    chosen_suit: Option<u8>,
}

/// 反间花色 tag：0=♥ 1=♦ 2=♣ 3=♠（上游 chooseControl 序）。
fn fanjian_suit_tag(s: crate::catalog::Suit) -> u8 {
    match s {
        crate::catalog::Suit::Heart => 0,
        crate::catalog::Suit::Diamond => 1,
        crate::catalog::Suit::Club => 2,
        crate::catalog::Suit::Spade => 3,
    }
}

impl SkillMachine for FanJianMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::FanJian {
            owner: self.owner,
            stage: match self.stage {
                FanJianStage::TargetAsk => FanJianStageSnap::TargetAsk,
                FanJianStage::SuitAsk => FanJianStageSnap::SuitAsk,
                FanJianStage::PickAsk => FanJianStageSnap::PickAsk,
                FanJianStage::Consume => FanJianStageSnap::Consume,
            },
            target: self.target,
            chosen_suit: self.chosen_suit,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            FanJianStage::TargetAsk => {
                let candidates: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|t| *t != self.owner)
                    .map(ChoiceOption::Player)
                    .collect();
                if candidates.is_empty() {
                    return Flow::Done;
                }
                self.stage = FanJianStage::SuitAsk;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "fanjian_target",
                        options: candidates,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            FanJianStage::SuitAsk => {
                // 消费 fanjian_target 应答
                let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                if !ctx.core.player(target).alive {
                    return Flow::Done;
                }
                self.target = Some(target);
                self.stage = FanJianStage::PickAsk;
                Flow::WaitChoice {
                    observer: target,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::PickTag,
                        prompt: "fanjian_suit",
                        options: (0..=3).map(ChoiceOption::Tag).collect(),
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            FanJianStage::PickAsk => {
                let Some(ChoiceOption::Tag(suit)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                self.chosen_suit = Some(suit);
                let Some(target) = self.target else {
                    return Flow::Done;
                };
                let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return Flow::Done;
                }
                self.stage = FanJianStage::Consume;
                Flow::WaitChoice {
                    observer: target,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "fanjian_pick",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: false,
                    },
                }
            }
            FanJianStage::Consume => {
                let Some(target) = self.target else {
                    return Flow::Done;
                };
                if let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() {
                    let suit_matches = self
                        .chosen_suit
                        .is_some_and(|t| fanjian_suit_tag(ctx.catalog.cards[c as usize].suit) == t);
                    ctx.give_hand_card(self.owner, target, c);
                    // v203 对齐上游：花色相符无事；不符才受 1 点伤害
                    if !suit_matches {
                        ctx.emit(EventData::Damage(crate::machine::damage_data(
                            Some(self.owner),
                            target,
                            1,
                        )));
                    }
                }
                Flow::Done
            }
        }
    }
}

// 离间（貂蝉）：出牌阶段限一次——弃一张手牌置于桌面作为载体，令两名
// 其他角色决斗（登记差异：不限男性角色）；借牌直结算（内嵌决斗机器）。
#[derive(Clone)]
struct LiJianBehavior;

impl SkillBehavior for LiJianBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(LiJianMachine {
            owner,
            stage: LiJianStage::DiscardAsk,
            card: None,
            inner: None,
        })
    }

    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        let p = ctx.state.player(owner);
        if !p.alive || p.hand.is_empty() {
            return None;
        }
        if ctx.state.alive_players().count() < 3 {
            return None;
        }
        Some(Box::new(LiJianMachine {
            owner,
            stage: LiJianStage::DiscardAsk,
            card: None,
            inner: None,
        }))
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LiJianStage {
    /// 挂起询问 = lijian_discard（弃一张手牌作为离间载体；取消 = 放弃）。
    DiscardAsk,
    /// 消费 lijian_discard 并挂起 lijian_targets（两名其他角色）。
    TargetsAsk,
    /// 消费 lijian_targets：载体牌置桌并内嵌决斗直结算。
    Launch,
}

struct LiJianMachine {
    owner: PlayerId,
    stage: LiJianStage,
    /// 载体牌（弃置的手牌，置于桌面参与决斗结算）。
    card: Option<CardId>,
    inner: Option<Box<dyn SkillMachine>>,
}

impl SkillMachine for LiJianMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(Self {
            owner: self.owner,
            stage: self.stage,
            card: self.card,
            inner: self.inner.as_ref().map(|m| m.fork()),
        })
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::LiJian {
            owner: self.owner,
            stage: match self.stage {
                LiJianStage::DiscardAsk => LiJianStageSnap::DiscardAsk,
                LiJianStage::TargetsAsk => LiJianStageSnap::TargetsAsk,
                LiJianStage::Launch => LiJianStageSnap::Launch,
            },
            card: self.card,
            inner: self.inner.as_ref().map(|m| Box::new(m.snapshot())),
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 内嵌决斗存在即全权委托
        if let Some(inner) = self.inner.as_mut() {
            let flow = inner.step(ctx);
            if matches!(flow, Flow::Done) {
                self.inner = None;
                return Flow::Done;
            }
            return flow;
        }
        match self.stage {
            LiJianStage::DiscardAsk => {
                if ctx.core.player(self.owner).hand.is_empty() {
                    return Flow::Done;
                }
                let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                self.stage = LiJianStage::TargetsAsk;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "lijian_discard",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            LiJianStage::TargetsAsk => {
                let Some(ChoiceOption::Card(c)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                self.card = Some(c);
                // 性别门控：离间限男性角色
                let candidates: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| {
                        *p != self.owner && seat_is_male(ctx, *p)
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                if candidates.len() < 2 {
                    return Flow::Done;
                }
                self.stage = LiJianStage::Launch;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "lijian_targets",
                        options: candidates,
                        min: 2,
                        max: 2,
                        allow_cancel: true,
                    },
                }
            }
            LiJianStage::Launch => {
                let Some(card) = self.card else {
                    return Flow::Done;
                };
                let picks: Vec<PlayerId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Player(p) => Some(*p),
                        _ => None,
                    })
                    .collect();
                if picks.len() < 2 {
                    // 弃权：载体牌离手入弃牌堆
                    if ctx.core.player(self.owner).hand.contains(&card) {
                        ctx.discard_from_hand(self.owner, card);
                    }
                    return Flow::Done;
                }
                // 载体牌从手牌置于桌面，作为决斗结算的牌
                ctx.core.player_mut(self.owner).hand.retain(|x| *x != card);
                ctx.core.table.push(card);
                let duel = crate::kernel::DuelMachine::new(
                    card, picks[0], picks[1], true, false, false,
                );
                self.inner = Some(Box::new(duel));
                let flow = self.inner.as_mut().expect("just set").step(ctx);
                if matches!(flow, Flow::Done) {
                    self.inner = None;
                    return Flow::Done;
                }
                flow
            }
        }
    }
}

// 观星（诸葛亮）：回合开始（判定阶段占位）时，可观看牌库顶 X 张
// （X = 存活角色数且至多 5）：所选牌按应答顺序置于牌库顶（先选先摸），
// 未选牌按原序置于牌库底。
#[derive(Clone)]
struct GuanXingBehavior;

impl SkillBehavior for GuanXingBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase {
            player,
            phase: Phase::Judge,
            begin: true,
        } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(GuanXingMachine {
            owner,
            stage: GuanXingStage::Arrange,
            top: Vec::new(),
        })
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GuanXingStage {
    /// 挂起询问 = guanxing_arrange（从已展示的顶牌中选任意张）。
    Arrange,
    /// 恢复读应答并落位。
    Place,
}

#[derive(Clone)]
struct GuanXingMachine {
    owner: PlayerId,
    stage: GuanXingStage,
    /// Arrange 阶段取出的牌库顶牌（原序，顶部在末尾）。
    top: Vec<CardId>,
}

impl SkillMachine for GuanXingMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::GuanXing {
            owner: self.owner,
            stage: match self.stage {
                GuanXingStage::Arrange => GuanXingStageSnap::Arrange,
                GuanXingStage::Place => GuanXingStageSnap::Place,
            },
            top: self.top.clone(),
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            GuanXingStage::Arrange => {
                let x = ctx.core.alive_count().min(5);
                let take = x.min(ctx.core.deck.len());
                if take == 0 {
                    return Flow::Done;
                }
                let new_len = ctx.core.deck.len() - take;
                self.top = ctx.core.deck.split_off(new_len);
                let options: Vec<ChoiceOption> =
                    self.top.iter().copied().map(ChoiceOption::Card).collect();
                self.stage = GuanXingStage::Place;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "guanxing_arrange",
                        options,
                        min: 0,
                        max: take,
                        allow_cancel: false,
                    },
                }
            }
            GuanXingStage::Place => {
                // 应答中的牌 = 置顶（按应答顺序，先选先摸：牌库顶在末尾，
                // 故逆序压栈）；未选的按原序置于牌库底（前插）。
                let chosen: Vec<CardId> = ctx
                    .answer()
                    .iter()
                    .filter_map(|o| match o {
                        ChoiceOption::Card(c) => Some(*c),
                        _ => None,
                    })
                    .collect();
                let rest: Vec<CardId> = self
                    .top
                    .iter()
                    .copied()
                    .filter(|c| !chosen.contains(c))
                    .collect();
                for c in &rest {
                    ctx.core.deck.insert(0, *c);
                }
                for c in chosen.iter().rev() {
                    ctx.core.deck.push(*c);
                }
                Flow::Done
            }
        }
    }
}

// 连营（陆逊）：失去最后一张手牌后，可摸一张牌。触发覆盖出牌
// （CardUsed）、响应（CardResponded）、给出（CardsMoved）三条路径；
// 弃牌阶段的批量弃置不产生事件，暂不触发（挂账覆盖清单）。
#[derive(Clone)]
struct LianYingBehavior;

impl SkillBehavior for LianYingBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        if ctx.state.player(owner).hand.len() > 1 {
            return false;
        }
        match event {
            EventData::CardUsed(u) => u.player == owner,
            EventData::CardResponded { player, .. } => *player == owner,
            EventData::CardsMoved { from, .. } => *from == owner,
            EventData::CardsDiscarded { player, .. } => *player == owner,
            _ => false,
        }
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(LianYingMachine { owner, asked: false })
    }
}

#[derive(Clone)]
struct LianYingMachine {
    owner: PlayerId,
    asked: bool,
}

impl SkillMachine for LianYingMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::LianYing { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        if !self.asked {
            self.asked = true;
            if ctx.core.player(self.owner).alive && ctx.core.player(self.owner).hand.is_empty()
            {
                ctx.draw_cards(self.owner, 1);
            }
        }
        Flow::Done
    }
}

// 闭月（貂蝉）：结束阶段开始时摸一张牌。
#[derive(Clone)]
struct BiYueBehavior;

impl SkillBehavior for BiYueBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase {
            player,
            phase: Phase::End,
            begin: true,
        } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(BiYueMachine { owner })
    }
}

#[derive(Clone)]
struct BiYueMachine {
    owner: PlayerId,
}

impl SkillMachine for BiYueMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::BiYue { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        ctx.draw_cards(self.owner, 1);
        Flow::Done
    }
}

// 英姿（周瑜）：摸牌阶段摸牌时额外摸一张牌。
#[derive(Clone)]
struct YingZiBehavior;

impl SkillBehavior for YingZiBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase {
            player,
            phase: Phase::Draw,
            begin: true,
        } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(YingZiMachine { owner })
    }
}

#[derive(Clone)]
struct YingZiMachine {
    owner: PlayerId,
}

impl SkillMachine for YingZiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::YingZi { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        ctx.draw_cards(self.owner, 1);
        Flow::Done
    }
}

// 枭姬（孙尚香）：失去装备区里的一张牌（替换/弃置）时摸两张牌。
#[derive(Clone)]
struct XiaoJiBehavior;

impl SkillBehavior for XiaoJiBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Equipped {
            player,
            replaced: Some(_),
            ..
        } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(XiaoJiMachine { owner })
    }
}

#[derive(Clone)]
struct XiaoJiMachine {
    owner: PlayerId,
}

impl SkillMachine for XiaoJiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::XiaoJi { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        ctx.draw_cards(self.owner, 2);
        Flow::Done
    }
}

// 青囊（华佗）：出牌阶段限一次，弃一张手牌并令一名已受伤的其他角色回复
// 1 点体力（Target → Discard 两阶段主动技）。
#[derive(Clone)]
struct QingNangBehavior;

impl SkillBehavior for QingNangBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false // 主动技
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(QingNangMachine {
            owner,
            stage: QingNangStage::TargetAsk,
            healed: None,
        })
    }

    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        let p = ctx.state.player(owner);
        if !p.alive || p.hand.is_empty() {
            return None;
        }
        let wounded = ctx
            .state
            .players
            .iter()
            .enumerate()
            .any(|(i, p)| {
                p.alive && (p.hp as i64) < (p.max_hp as i64) && i as PlayerId != owner
            });
        if !wounded {
            return None;
        }
        Some(Box::new(QingNangMachine {
            owner,
            stage: QingNangStage::TargetAsk,
            healed: None,
        }))
    }
}

#[derive(Clone)]
enum QingNangStage {
    /// 挂起询问 = qingnang_target（恢复读目标）。
    TargetAsk,
    /// 挂起询问 = qingnang_card（恢复读牌并治疗）。
    TargetRead,
    /// 挂起询问 = qingnang_card（恢复读牌并给出）。
    CardRead,
}

#[derive(Clone)]
struct QingNangMachine {
    owner: PlayerId,
    stage: QingNangStage,
    healed: Option<PlayerId>,
}

impl SkillMachine for QingNangMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::QingNang {
            owner: self.owner,
            stage: match self.stage {
                QingNangStage::TargetAsk => QingNangStageSnap::TargetAsk,
                QingNangStage::TargetRead => QingNangStageSnap::TargetRead,
                QingNangStage::CardRead => QingNangStageSnap::CardRead,
            },
            healed: self.healed,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            QingNangStage::TargetAsk => {
                // 构建 qingnang_target 询问（已受伤的其他角色）
                let wounded: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| {
                        *p != self.owner
                            && {
                                let pl = ctx.core.player(*p);
                                pl.alive
                                    && (pl.hp as i64) < (pl.max_hp as i64)
                            }
                    })
                    .map(ChoiceOption::Player)
                    .collect();
                if wounded.is_empty() {
                    return Flow::Done;
                }
                self.stage = QingNangStage::TargetRead;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "qingnang_target",
                        options: wounded,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            QingNangStage::TargetRead => {
                // 恢复：qingnang_target 的应答（Player 目标 / 空应答弃权）
                let Some(ChoiceOption::Player(target)) = ctx.answer().first().copied() else {
                    return Flow::Done;
                };
                self.healed = Some(target);
                let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return Flow::Done;
                }
                self.stage = QingNangStage::CardRead;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "qingnang_card",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            QingNangStage::CardRead => {
                // 恢复：qingnang_card 的应答——Card 则弃置并治疗
                if let Some(&ChoiceOption::Card(c)) = ctx.answer().first() {
                    ctx.discard_from_hand(self.owner, c);
                    if let Some(target) = self.healed {
                        ctx.recover(target, 1, None);
                    }
                }
                Flow::Done
            }
        }
    }
}




// 洛神（甄姬）：判定阶段开始时可进行判定——黑色则获得此判定牌并再次判定
// （循环至翻出红色；简化契约：整链一次确认，不分次放弃）。
#[derive(Clone)]
struct LuoShenBehavior;

impl SkillBehavior for LuoShenBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase {
            player,
            phase: Phase::Judge,
            begin: true,
        } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(LuoShenMachine { owner })
    }
}

#[derive(Clone)]
struct LuoShenMachine {
    owner: PlayerId,
}

impl SkillMachine for LuoShenMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::LuoShen { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 单次判定：黑桃/梅花 → 获得此牌并再次判定（Yield 续行）；
        // 红桃/方块 → 判定牌弃置，循环结束。
        let card = ctx.draw_judge_card();
        let black = matches!(
            ctx.catalog.cards[card as usize].suit,
            crate::catalog::Suit::Spade | crate::catalog::Suit::Club
        );
        if black {
            ctx.core.player_mut(self.owner).hand.push(card);
            ctx.log(LogEvent::CardsDrawn {
                player: self.owner,
                cards: vec![card],
            });
            Flow::Yield
        } else {
            ctx.to_discard(card);
            Flow::Done
        }
    }
}

// 集智（黄月英）：使用普通锦囊时摸一张牌。
#[derive(Clone)]
struct JiZhiBehavior;

impl SkillBehavior for JiZhiBehavior {
    fn filter(&self, ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::CardUsed(u)
            if u.player == owner
                && matches!(
                    u.use_as.unwrap_or_else(|| ctx.catalog.card_kind(u.card)),
                    crate::catalog::CardKind::SavageAttack
                        | crate::catalog::CardKind::ArcheryAttack
                        | crate::catalog::CardKind::Duel
                        | crate::catalog::CardKind::Chain
                        | crate::catalog::CardKind::Nullify
                ))
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(JiZhiMachine { owner })
    }
}

#[derive(Clone)]
struct JiZhiMachine {
    owner: PlayerId,
}

impl SkillMachine for JiZhiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::JiZhi { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        ctx.draw_cards(self.owner, 1);
        Flow::Done
    }
}

// 苦肉（黄盖）：出牌阶段主动技——失去 1 点体力，摸两张牌（每阶段限一次）。
#[derive(Clone)]
struct KuRouBehavior;

impl SkillBehavior for KuRouBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false // 主动技：不随事件触发
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(KuRouMachine {
            owner,
            stage: KuRouStage::Lose,
        })
    }

    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        let p = ctx.state.player(owner);
        if !p.alive || p.dying {
            return None;
        }
        Some(Box::new(KuRouMachine {
            owner,
            stage: KuRouStage::Lose,
        }))
    }
}

#[derive(Clone)]
enum KuRouStage {
    /// 失去 1 点体力（含濒死内联检查）。
    Lose,
    /// 摸两张牌。
    Draw,
}

#[derive(Clone)]
struct KuRouMachine {
    owner: PlayerId,
    stage: KuRouStage,
}

impl SkillMachine for KuRouMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::KuRou {
            owner: self.owner,
            stage: match self.stage {
                KuRouStage::Lose => KuRouStageSnap::Lose,
                KuRouStage::Draw => KuRouStageSnap::Draw,
            },
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            KuRouStage::Lose => {
                let p = ctx.core.player_mut(self.owner);
                p.hp -= 1;
                let entering = p.hp <= 0 && !p.dying;
                if entering {
                    p.dying = true;
                    ctx.log(LogEvent::DyingEntered { player: self.owner });
                    ctx.emit(EventData::Dying {
                        player: self.owner,
                        source: None,
                    });
                }
                self.stage = KuRouStage::Draw;
                Flow::Yield
            }
            KuRouStage::Draw => {
                ctx.draw_cards(self.owner, 2);
                Flow::Done
            }
        }
    }
}

// 制衡（孙权）：出牌阶段主动技——弃置任意张手牌，然后摸等量的牌
// （每回合限一次；空选择 = 不发动效果直接结束）。
#[derive(Clone)]
struct ZhiHengBehavior;

impl SkillBehavior for ZhiHengBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false // 主动技：不随事件触发
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(ZhiHengMachine {
            owner,
            stage: ZhiHengStage::Select,
        })
    }

    fn activate(&self, ctx: &QueryCtx<'_>, owner: PlayerId) -> Option<Box<dyn SkillMachine>> {
        let p = ctx.state.player(owner);
        if !p.alive || p.hand.is_empty() {
            return None; // 无手牌可弃：无法发动
        }
        Some(Box::new(ZhiHengMachine {
            owner,
            stage: ZhiHengStage::Select,
        }))
    }
}

#[derive(Clone)]
enum ZhiHengStage {
    /// 询问弃置哪些手牌（空应答 = 放弃发动）。
    Select,
    /// 弃置后摸等量（同一次提交内完成，无需挂起）。
    Draw,
}

#[derive(Clone)]
struct ZhiHengMachine {
    owner: PlayerId,
    stage: ZhiHengStage,
}

impl SkillMachine for ZhiHengMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::ZhiHeng {
            owner: self.owner,
            stage: match self.stage {
                ZhiHengStage::Select => ZhiHengStageSnap::Select,
                ZhiHengStage::Draw => ZhiHengStageSnap::Draw,
            },
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            ZhiHengStage::Select => {
                let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                let max = options.len();
                // 挂起询问 = zhiheng_select；恢复由 Draw 分支执行弃置与摸牌
                self.stage = ZhiHengStage::Draw;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: crate::choice::ChoiceKind::SelectCards,
                        prompt: "zhiheng_select",
                        options,
                        min: 0,
                        max,
                        allow_cancel: true,
                    },
                }
            }
            ZhiHengStage::Draw => {
                let answer = ctx.answer().clone();
                let n = answer.len();
                for c in answer {
                    if let ChoiceOption::Card(card) = c {
                        ctx.discard_from_hand(self.owner, card);
                    }
                }
                if n > 0 {
                    ctx.draw_cards(self.owner, n);
                }
                Flow::Done
            }
        }
    }
}

// 遗计（郭嘉）：受到 1 点伤害后，摸两张牌，然后可将至多两张手牌交给
// 一至两名其他玩家（每张给牌均可放弃）。
#[derive(Clone)]
struct JiJiBehavior;

impl SkillBehavior for JiJiBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d)
            if d.target == owner && !d.prevented && d.amount > 0)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(JiJiMachine {
            owner,
            stage: JiJiStage::Draw,
            target: None,
            given: 0,
        })
    }
}

#[derive(Clone)]
enum JiJiStage {
    /// 摸两张（子事件结算后继续）。
    Draw,
    /// 挂起询问 = jiji_target（恢复读目标）。
    Target,
    /// 给牌后回到目标询问前的中转（行为同 Target：重新询问目标）。
    /// 不再构造，但保留变体：v2 字节的 jiji 挂起可能携带 Retarget=2，
    /// 恢复时映射回 Target；删除会破坏已落盘字节。
    #[allow(dead_code)]
    Retarget,
    /// 挂起询问 = jiji_card（恢复读牌并给出）。
    GiveCard,
    /// 挂起询问 = jiji_card 的给牌（恢复执行给出）。
    GiveChooseCard,
}

#[derive(Clone)]
struct JiJiMachine {
    owner: PlayerId,
    stage: JiJiStage,
    target: Option<PlayerId>,
    given: u8,
}

impl SkillMachine for JiJiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::JiJi {
            owner: self.owner,
            stage: match self.stage {
                JiJiStage::Draw => JiJiStageSnap::Draw,
                JiJiStage::Target => JiJiStageSnap::Target,
                JiJiStage::Retarget => JiJiStageSnap::Retarget,
                JiJiStage::GiveCard => JiJiStageSnap::GiveCard,
                JiJiStage::GiveChooseCard => JiJiStageSnap::GiveChooseCard,
            },
            target: self.target,
            given: self.given,
        }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        match self.stage {
            JiJiStage::Draw => {
                ctx.draw_cards(self.owner, 2);
                self.stage = JiJiStage::Target;
                Flow::Yield
            }
            JiJiStage::Target | JiJiStage::Retarget => {
                // 至多两张；无其他存活玩家或手牌已空则结束
                if self.given >= 2 || ctx.core.player(self.owner).hand.is_empty() {
                    return Flow::Done;
                }
                let others: Vec<ChoiceOption> = ctx
                    .core
                    .alive_players()
                    .filter(|p| *p != self.owner)
                    .map(ChoiceOption::Player)
                    .collect();
                if others.is_empty() {
                    return Flow::Done;
                }
                // 挂起询问 = jiji_target；恢复由 GiveCard 分支读取目标
                self.stage = JiJiStage::GiveCard;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectPlayers,
                        prompt: "jiji_target",
                        options: others,
                        min: 0,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            JiJiStage::GiveCard => {
                // 恢复：jiji_target 的应答（Player 目标 / 空应答弃权）
                let to = match ctx.answer().first() {
                    Some(ChoiceOption::Player(p)) => *p,
                    _ => return Flow::Done,
                };
                self.target = Some(to);
                let options: Vec<ChoiceOption> = ctx.core.player(self.owner)
                    .hand
                    .iter()
                    .copied()
                    .map(ChoiceOption::Card)
                    .collect();
                if options.is_empty() {
                    return Flow::Done;
                }
                // 挂起询问 = jiji_card；恢复由 GiveChooseCard 分支读牌并给出
                self.stage = JiJiStage::GiveChooseCard;
                Flow::WaitChoice {
                    observer: self.owner,
                    spec: ChoiceSpec {
                        kind: ChoiceKind::SelectCards,
                        prompt: "jiji_card",
                        options,
                        min: 1,
                        max: 1,
                        allow_cancel: true,
                    },
                }
            }
            JiJiStage::GiveChooseCard => {
                // 恢复：jiji_card 的应答——Card 则给出并计入；空应答则放弃并结束
                if let Some(&ChoiceOption::Card(c)) = ctx.answer().first() {
                    let Some(to) = self.target else {
                        return Flow::Done;
                    };
                    ctx.give_hand_card(self.owner, to, c);
                    self.given += 1;
                } else {
                    return Flow::Done;
                }
                if self.given >= 2 || ctx.core.player(self.owner).hand.is_empty() {
                    return Flow::Done;
                }
                // 回到目标询问（第二轮，至多两张）
                self.stage = JiJiStage::Target;
                Flow::Yield
            }
        }
    }
}

// 碎盾：每回合限一次，当你即将受到伤害时，防止之。
// ---------------------------------------------------------------------------

struct SuiDunBehavior;

impl SkillBehavior for SuiDunBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d) if d.target == owner && !d.prevented)
    }

    fn start(&self, _owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(SuiDunMachine)
    }
}

#[derive(Clone)]
struct SuiDunMachine;

impl SkillMachine for SuiDunMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::SuiDun
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        ctx.current_damage_mut().prevented = true;
        Flow::Done
    }
}

// ---------------------------------------------------------------------------
// 肃清：当其他玩家即将受到伤害时，其「碎盾」技能直到回合结束失效。
// （在同一个伤害前置窗口内先行结算，验证队列运行前的重查。）
// ---------------------------------------------------------------------------

struct SuQingBehavior;

impl SkillBehavior for SuQingBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d) if d.target != owner)
    }

    fn start(&self, _owner: PlayerId, event: &EventData) -> Box<dyn SkillMachine> {
        let target = match event {
            EventData::Damage(d) => d.target,
            _ => unreachable!("filter guarantees damage"),
        };
        Box::new(SuQingMachine { target })
    }
}

#[derive(Clone)]
struct SuQingMachine {
    target: PlayerId,
}

impl SkillMachine for SuQingMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::SuQing { target: self.target }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let suidun = ctx.catalog.skill_by_name(SUI_DUN).expect("碎盾 in catalog");
        ctx.set_skill_enabled(self.target, suidun, false);
        Flow::Done
    }
}

// ---------------------------------------------------------------------------
// 曹仁「据守」：结束阶段，你可以翻面并摸一张牌。
// 登记差异：翻面被【漩涡面具】等取消时摸牌仍执行（效果独立）。
// ---------------------------------------------------------------------------

struct JushouBehavior;

impl SkillBehavior for JushouBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Phase {
            player,
            phase: Phase::End,
            begin: true,
        } if *player == owner)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(JushouMachine { owner })
    }
}

#[derive(Clone)]
struct JushouMachine {
    owner: PlayerId,
}

impl SkillMachine for JushouMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::ShouYi { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        // 确认门（OptionalGate）通过后执行：翻面经 TurnOverBefore 子事件
        // （面具等可取消）；摸牌独立执行（登记差异：取消翻面不影响摸牌）。
        ctx.emit(EventData::TurnOverBefore {
            player: self.owner,
            cancelled: false,
        });
        ctx.draw_cards(self.owner, 1);
        Flow::Done
    }
}

// ---------------------------------------------------------------------------
// 授艺：当你受到伤害后，你获得「移伤」直到回合结束。
// ---------------------------------------------------------------------------

struct ShouYiBehavior;

impl SkillBehavior for ShouYiBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, owner: PlayerId, event: &EventData) -> bool {
        matches!(event, EventData::Damage(d)
            if d.target == owner && !d.prevented && d.amount > 0)
    }

    fn start(&self, owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(ShouYiMachine { owner })
    }
}

#[derive(Clone)]
struct ShouYiMachine {
    owner: PlayerId,
}

impl SkillMachine for ShouYiMachine {
    fn fork(&self) -> Box<dyn SkillMachine> {
        Box::new(self.clone())
    }
    fn snapshot(&self) -> MachineSnapshot {
        MachineSnapshot::ShouYi { owner: self.owner }
    }
    fn step(&mut self, ctx: &mut Ctx<'_>) -> Flow {
        let yishang = ctx.catalog.skill_by_name(YI_SHANG).expect("移伤 in catalog");
        ctx.add_skill_until_turn_end(self.owner, yishang);
        Flow::Done
    }
}

// ---------------------------------------------------------------------------
// 目录
// ---------------------------------------------------------------------------

/// 权限型能力（无触发时机）的空行为：判定流程经 skill_by_name 查询持有者，
/// 该行为永不进入触发队列与挂起槽。
struct NoopBehavior;

impl SkillBehavior for NoopBehavior {
    fn filter(&self, _ctx: &QueryCtx<'_>, _owner: PlayerId, _event: &EventData) -> bool {
        false
    }
    fn start(&self, _owner: PlayerId, _event: &EventData) -> Box<dyn SkillMachine> {
        Box::new(crate::kernel::NoopMachine::new())
    }
}

// ---------------------------------------------------------------------------
// 首组官方技能（N08，ADR 0712）：判定主题——张角「鬼道」+ 郭嘉「天妒」
// 规则来源：标准包规则文本；鬼道改判流程对齐旧线 `query/mod.rs`
// `judgment_black_rewrite_cards`（黑色手牌/装备）与 `command/judge.rs`
// `judgment_rewriters`（自判定阶段拥有者起座位序、逐人一次）——
// 原生本批改判资格仅取黑色手牌（装备区牌打出未接，差异登记）。
// 天妒：旧线无实现（仅 `characters.rs::TIANDU` 常量），按标准包规则文本
// 「每当一名角色的判定牌生效后，你可以将之收入手牌」实现。
// ---------------------------------------------------------------------------

/// 张角「鬼道」：判定牌生效前，可打出一张黑色手牌代替之。
pub const GUI_DAO: &str = "鬼道";
/// 郭嘉「天妒」：一名角色的判定牌生效后，可将之收入手牌。
pub const TIAN_DU: &str = "天妒";

pub const SAN_CAI: &str = "散财";
pub const YI_SHANG: &str = "移伤";
pub const SUI_DUN: &str = "碎盾";
pub const SU_QING: &str = "肃清";
pub const SHOU_YI: &str = "授艺";
pub const JU_SHOU: &str = "据守";
/// 乐进「骁果」：其他角色的结束阶段开始时，可弃一张基本手牌，对其
/// 造成 1 点伤害（P2/N08 标准缺失将，2026-10-07）。
pub const XIAO_GUO: &str = "骁果";
/// 潘凤「狂斧」：出牌阶段用【杀】对目标造成伤害后，若其体力值小于你，
/// 你摸两张牌，否则你失去 1 点体力（每回合限一次）。
pub const KUANG_FU: &str = "狂斧";
/// 华雄「耀武」：锁定技，你受到红色【杀】的伤害时，伤害来源选择摸一
/// 张牌或回复 1 点体力。
pub const YAO_WU: &str = "耀武";
/// 甘夫人「淑慎」：回复体力后，可令一名其他角色摸牌（有手牌摸 1，
/// 无手牌摸 2）（P2/N08 标准缺失将第二批，2026-10-07）。
pub const SHU_SHEN: &str = "淑慎";
/// 甘夫人「神智」：准备阶段（原生锚点：判定阶段开始）可弃置全部手牌，
/// 若数量不少于体力值则回复 1 点体力。
pub const SHEN_ZHI: &str = "神智";
/// 李典「恂恂」：摸牌阶段开始时，观看牌堆顶四张，将其中两张置于牌堆顶
/// （先选先摸）其余置于牌堆底，然后正常摸牌（P2/N08 标准缺失将第二批）。
pub const XUN_XUN: &str = "恂恂";
/// 李典「忘隙」：对其他角色造成伤害或受到其伤害后，可与该角色各摸一张牌。
pub const WANG_XI: &str = "忘隙";
/// 公孙瓒「义从」（refresh 改版，P2/N08 标准缺失将第三批）：锁定技——
/// 你计算与其他角色的距离 -1；其他角色计算与你的距离 +1（你的体力值
/// 不大于 2 时）。
pub const YI_CONG: &str = "义从";
/// 伊籍「机捷」：出牌阶段限一次，取牌堆底一张牌交给一名角色
/// （P2/N08 标准缺失将第四批）。
pub const JI_JIE: &str = "机捷";
/// 伊籍「急援」：一名角色进入濒死状态时，可令其摸一张牌（登记差异：
/// 仅濒死臂——上游赠牌/失去牌臂需要 giver 跟踪面，未建模）。
pub const JI_YUAN: &str = "急援";
/// 丈八蛇矛（配套主动技，批③-b 第七把）：两张手牌当【杀】使用。
pub const ZHANGBA: &str = "丈八蛇矛";
/// 雌雄双股剑（批③-b 第八把，equip1 随牌种接线）：对异性目标出杀后
/// 目标弃一张手牌或令使用者摸一张牌。
pub const CI_XIONG: &str = "雌雄双股剑";
/// 袁术「妄尊」（P2/N08 标准缺失将收尾批）：锁定技——其他角色的回合
/// 开始时，若其体力值大于你，你摸一张牌；若该角色为主公，你再摸一张
/// 且其手牌上限 -1 直到回合结束（印记技「妄尊印」承载）。
pub const WANG_ZUN: &str = "妄尊";
/// 袁术「妄尊」的手牌上限印记（隐藏技能，弃牌阶段结算 -1）。
pub const WANG_ZUN_MARK: &str = "妄尊印";
/// 袁术「同疾」：锁定技——其他角色使用【杀】指定唯一目标时（你不在
/// 目标内、你在其攻击范围内），其须选择弃置一张牌或取消：弃置则该杀
/// 改为指向你（登记差异：目标合法域按上游 inRange=目标对你的攻击范围）。
pub const TONG_JI: &str = "同疾";

/// 司马懿「鬼才」：一名角色的判定牌生效前，可打出一张**任意花色**手牌代替之
/// （区别于鬼道的黑色手牌/装备牌）。
pub const GUI_CAI: &str = "鬼才";
/// 司马懿「反馈」：受到 1 点伤害后，可获得伤害来源的一张牌（手牌或装备）。
pub const FAN_KUI: &str = "反馈";
/// 曹操「奸雄」：受到伤害后，可获得造成此伤害的牌。
pub const JIAN_XIONG: &str = "奸雄";
/// 貂蝉「闭月」：结束阶段开始时，摸一张牌。
pub const BI_YUE: &str = "闭月";
/// 甄姬「洛神」：判定阶段开始时，可进行判定——黑色则获得此牌并再次判定
/// （循环至红桃或放弃）。
pub const LUO_SHEN: &str = "洛神";
/// 关羽「武圣」：可将任意红色手牌当杀使用。
pub const WU_SHENG: &str = "武圣";
/// 赵云「龙胆」：可将杀当闪、闪当杀使用。
pub const LONG_DAN: &str = "龙胆";
/// 大乔「国色」：可将方块牌当【乐不思蜀】使用。
pub const GUO_SE: &str = "国色";
/// 吕布「无双」：你的杀需连续两张闪抵消；与你决斗的角色每次需打出两张杀。
pub const WU_SHUANG: &str = "无双";
/// 周瑜「英姿」：摸牌阶段摸牌时额外摸一张牌。
pub const YING_ZI: &str = "英姿";
/// 孙尚香「枭姬」：失去装备区里的一张牌时，摸两张牌。
pub const XIAO_JI: &str = "枭姬";
/// 华佗「青囊」：出牌阶段限一次，弃一张手牌并令一名已受伤的其他角色回复 1 点体力。
pub const QING_NANG: &str = "青囊";
/// 黄月英「集智」：使用普通锦囊时摸一张牌。
pub const JI_ZHI: &str = "集智";
/// 吕蒙「克己」：若本回合未使用杀，则无需弃牌。
pub const KE_JI: &str = "克己";
/// 张飞「咆哮」：出牌阶段使用杀没有次数限制。
pub const PAO_XIAO: &str = "咆哮";
/// 甘宁「奇袭」：可将黑色牌当【过河拆桥】使用。
pub const QI_XI: &str = "奇袭";
/// 黄盖「苦肉」：出牌阶段，可失去 1 点体力，摸两张牌（每阶段限一次）。
pub const KU_ROU: &str = "苦肉";
/// 孙权「制衡」：出牌阶段限一次，可弃置任意张手牌，然后摸等量的牌。
pub const ZHI_HENG: &str = "制衡";
/// 郭嘉「遗计」：受到 1 点伤害后，摸两张牌，然后可将至多两张手牌交给其他玩家。
pub const JI_JI: &str = "遗计";
/// 夏侯惇「刚烈」：受到伤害后，可进行判定：若判定牌不为红桃，伤害来源受到你造成的 1 点伤害。
pub const GANG_LIE: &str = "刚烈";
/// 张辽「突袭」：摸牌阶段开始时，可放弃摸牌，改为获得至多两名其他角色的各一张手牌。
pub const TU_XI: &str = "突袭";
/// 许褚「裸衣」：摸牌阶段开始时，可少摸一张牌，然后本回合【杀】或【决斗】造成的伤害 +1。
pub const LUO_YI: &str = "裸衣";
/// 甄姬「倾国」：可将装备区里的黑色牌当【闪】使用（响应杀）。
pub const QING_GUO: &str = "倾国";
/// 黄月英「奇才」：锁定技，使用任何锦囊牌无距离限制。
pub const QI_CAI: &str = "奇才";
/// 孙权「救援」：主公技，锁定技，处于濒死状态时，其他角色对你使用的【桃】回复量 +1。
pub const JIU_YUAN: &str = "救援";
/// 华佗「急救」：锁定技，濒死结算时可将红色手牌当【桃】使用。
pub const JI_JIU: &str = "急救";
/// 曹操「护驾」：主公技，需要出闪时，其他存活角色的闪可代出（原生简化：跨玩家借闪响应）。
pub const HU_JIA: &str = "护驾";
/// 大乔「流离」：成为【杀】的目标时，可弃一张手牌将此杀转移给另一名其他角色。
pub const LIU_LI: &str = "流离";
/// 孙尚香「结姻」：出牌阶段限一次，弃两张手牌，与一名受伤的其他男性角色各回复 1 点体力
/// （v202 对齐上游 filterTarget；旧「已婚=装备区有牌」登记撤销）。
pub const JIE_YIN: &str = "结姻";
/// 周瑜「反间」：出牌阶段限一次，一名其他角色选择你的一张手牌获得之，然后受到你造成的 1 点伤害
/// （v203 对齐上游：目标先选花色，花色不符才受伤；旧登记撤销）。
pub const FAN_JIAN: &str = "反间";
/// 貂蝉「离间」：出牌阶段限一次，弃一张手牌，令两名其他角色决斗
/// （原生简化：不限男性角色，见 ADR 0770 登记表）。
pub const LI_JIAN: &str = "离间";
/// 诸葛亮「观星」：回合开始（判定阶段）时，可观看牌库顶 X 张（X=存活数且至多 5），
/// 任意张以任意顺序置于牌库顶，其余置于牌库底。
pub const GUAN_XING: &str = "观星";
/// 诸葛亮「空城」：锁定技，没有手牌时不能成为【杀】和【决斗】的目标。
pub const KONG_CHENG: &str = "空城";
/// 陆逊「谦逊」：锁定技，不能成为【乐不思蜀】的目标（上游无【顺手牵羊】，挂账清单）。
pub const QIAN_XUN: &str = "谦逊";
/// 陆逊「连营」：失去最后一张手牌后（出牌/响应/给出路径），可摸一张牌。
pub const LIAN_YING: &str = "连营";
/// 刘备「仁德」：出牌阶段限一次，可将任意张手牌交给一名其他角色，
/// 若给出的牌不少于两张，你回复 1 点体力。
pub const REN_DE: &str = "仁德";
/// 刘备「激将」主公技：出牌阶段限一次，可请求一名已声明蜀势力的其他角色
/// 打出一张杀（经其确认后置入你的手牌，视为由你使用——原生简化：获得后
/// 走正常出杀流程，见 ADR 0760）。
pub const JI_JIANG: &str = "激将";
/// 马超「马术」：锁定技，计算与其他角色的距离时始终 -1。
pub const MA_SHU: &str = "马术";
/// 凶兽（Boss 模式共有技，规则 v208）：锁定技三面——你使用【杀】对体力
/// 值小于你的角色伤害 +1；你与其他角色距离 -1；你不能被翻面。
pub const BOSS_XIONGSHOU: &str = "凶兽";
/// 无脏（Boss 混沌，规则 v209）：锁定技两面——摸牌阶段的摸牌基数为
/// max(5, 体力值一半向上取整前的整除值)（即 2+max(5,floor(hp/2))-2）；
/// 手牌上限基数为 0（体力贡献清零，保留其余加成）。
pub const BOSS_WUZANG: &str = "无脏";
/// 相德（Boss 混沌，规则 v210）：锁定技——其他角色（装备区有武器）对
/// 你造成伤害时，此伤害 +1。
pub const BOSS_XIANGDE: &str = "相德";
/// 隐贼（Boss 混沌，规则 v210）：锁定技，体力值首次减至一半或更少时激
/// 活；你无手牌时，其他角色对你造成伤害后弃置其一张手牌。
pub const BOSS_YINZEI: &str = "隐贼";
/// 逐厄（Boss 穷奇，规则 v211）：锁定技——任何人受到伤害后（来源存活
/// 且非你），你与来源各摸一张牌。
pub const BOSS_ZHUE: &str = "逐厄";
/// 覆泰（Boss 穷奇，规则 v212）：锁定技——准备阶段（原生锚点：判定阶
/// 段开始，神智同口径）开始时，所有受伤角色按座位序各回复 1 点体力。
pub const BOSS_FUTAI: &str = "覆泰";
/// 延妒（Boss 穷奇，规则 v213）：锁定技，体力值首次减至一半或更少时激
/// 活；其他角色结束阶段开始时，若其本回合未造成伤害且有牌，你获得其
/// 一张手牌。
pub const BOSS_YANDU: &str = "延妒";
/// 贪欲（Boss 饕餮，规则 v215）：锁定技两面——跳过你的弃牌阶段；结束
/// 阶段若你的手牌数为全场最多（不少于任何其他存活角色），失去 1 点体
/// 力。
pub const BOSS_TANYU: &str = "贪欲";
/// 藏目（Boss 饕餮，规则 v216）：锁定技——摸牌阶段的摸牌基数改为存活
/// 角色数（固定摸 2 后补额外 存活数-2）。
pub const BOSS_CANGMU: &str = "藏目";
/// 积财（Boss 饕餮，规则 v216）：锁定技，体力值首次减至一半或更少时激
/// 活；一名角色回复体力后，你与其各摸一张牌（自己回复则自己摸两张）。
pub const BOSS_JICAI: &str = "积财";
/// 拟态（Boss 梼杌，规则 v217）：锁定技两面——防止你于回合内受到的伤
/// 害；你于回合外受到的火属性伤害 +1。
pub const BOSS_NITAI: &str = "拟态";
/// 冥顽（Boss 梼杌，规则 v219）：锁定技——你于回合内使用牌对其他角色
/// 造成伤害后，你于此回合内使用牌只能指定你与这些角色为目标，且你每
/// 使用一张牌，摸一张牌。
pub const BOSS_MINWAN: &str = "冥顽";
/// 乱常（Boss 梼杌，规则 v218）：锁定技，体力值首次减至一半或更少时激
/// 活；回合开始（判定阶段开始锚点）视为使用【南蛮入侵】，回合结束视为
/// 使用【万箭齐发】（目标=全部其他存活角色）。
pub const BOSS_LUANCHANG: &str = "乱常";
/// 催毒（Boss 书精，规则 v220）：锁定技——你对敌方角色造成伤害后，若
/// 其没有「中毒」，你令其获得「中毒」（登记：敌方以受伤者≠持技者近似，
/// 木神勾芒联动不在场跳过）。
pub const BOSS_CUIDUI: &str = "催毒";
/// 中毒（Boss 书精授技，规则 v220）：锁定技——回合开始（判定阶段开始
/// 锚点）判定，非红桃受 1 点无来源伤害，非黑桃则失去此技。
pub const BOSS_ZHONGDU: &str = "中毒";
/// 妖兽（Boss 神兽共有技，规则 v221）：锁定技——你与其他角色计算距离
/// 时 -2（上游 boss_yaoshou mod globalFrom distance-2 同口径）。
pub const BOSS_YAOSHOU: &str = "妖兽";
/// 毒妒（Boss 相柳，规则 v223）：锁定技——你受到伤害时，伤害来源获得
/// 1 枚「蛇毒」标记（你自身不会拥有）；【桃】视为【杀】（mod 面暂缓
/// 登记）。蛇毒标记：其回合开始弃 X 张牌或失去 X 点体力后递减。
pub const BOSS_DUQU: &str = "毒妒";
/// 邪酬（Boss 相柳，规则 v223）：锁定技，体力值首次减至一半或更少时激
/// 活；除你之外的角色使用「桃」或「酒」时，获得 1 枚「蛇毒」标记。
pub const BOSS_ECHOU: &str = "邪酬";
/// 蛇毒标记结算（毒妒/邪酬授予的标记，规则 v223）：锁定技——回合开始
/// 弃 X 张牌或失去 X 点体力后标记-1（X=标记数）。
pub const BOSS_SHEDU: &str = "蛇毒";
/// 兽九（Boss 相柳，规则 v222）：锁定技——手牌上限恒为 9；出牌阶段与
/// 回合结束时手牌补至 9；跳过摸牌阶段（摸牌基数 0）。
pub const BOSS_JIUSHOU: &str = "兽九";
/// 兵先（Boss 朱厌，规则 v224）：锁定技——其他角色的回合结束时，若其
/// 本回合内没有使用【杀】（登记：出牌阶段口径），视为对其使用一张
/// 【杀】。
pub const BOSS_BINGXIAN: &str = "兵先";
/// 巨源（Boss 朱厌，规则 v226）：锁定技——体力上限+5（由 boss_zhuyan
/// maxHp30 表意，登记同穷奇）；出牌阶段体力低于上次回合结束时体力时，
/// 【杀】可额外指定 1 个目标。
pub const BOSS_JUYUAN: &str = "巨源";
/// 招祸（Boss 毕方，规则 v227）：锁定技两面——你造成的所有伤害均视为
/// 火属性；你免疫所有火属性伤害（登记差异：你的回合中其他角色防具无
/// 效面挂点分散，未建模）。
pub const BOSS_ZHAOHUO: &str = "招祸";
/// 封冻（Boss 应昭，规则 v228）：锁定技——你的回合开始时，所有其他角
/// 色获得「封印」（本回合无法使用手牌，回合结束解除）。
pub const BOSS_FENGDONG: &str = "封冻";
/// 巡游（Boss 应昭，规则 v231）：锁定技——其他角色回合开始时，你获得
/// 座位序首名有手牌的其他角色的一张手牌（登记：hej 全区域/随机以座位
/// 序+手牌首张确定性近似）；若为装备牌则你自动使用之。
pub const BOSS_XUNYOU: &str = "巡游";
/// 魔道（Boss 夜叉，规则 v253）：锁定技——准备阶段摸 2 张牌。
pub const BOSS_MODAO: &str = "魔道";
/// 魔剑（Boss 夜叉，规则 v253）：锁定技——出牌阶段开始视为对全部敌方
/// 使用【万箭齐发】（虚拟牌，机器直发不经 PlayCard 校验；上游 useCard
/// canUse 过滤同口径=异侧存活）。
pub const BOSS_MOJIAN: &str = "魔剑";
/// 鬼计（Boss 黑无常，规则 v256）：锁定技——准备阶段（判定阶段锚）
/// 弃置自己判定区一张牌（随机→首张确定性近似登记）。
pub const BOSS_GUIJI: &str = "鬼计";
/// 太平（Boss 黑无常，规则 v256）：锁定技——摸牌阶段额外摸 2 张牌。
pub const BOSS_TAIPING: &str = "太平";
/// 索命（Boss 黑无常，规则 v256）：锁定技——结束阶段横置（链接）全部
/// 未链接的其他存活角色（[1,num] 自选→全部确定性近似登记）。
pub const BOSS_SUOMING: &str = "索命";
/// 吸行（Boss 黑无常，规则 v256）：锁定技——准备阶段对一名已链接的其
/// 他角色造成 1 点雷伤并回复 1 点体力（目标自选→座位序首个已链接确定
/// 性近似登记）。
pub const BOSS_XIXING: &str = "吸行";
/// 爆敛（Boss 白无常/牛头，规则 v255）：锁定技——结束阶段摸 2 张牌。
pub const BOSS_BAOLIAN: &str = "爆敛";
/// 强征（Boss 白无常，规则 v255）：锁定技——结束阶段从每名其他有手牌
/// 角色处随机获得一张手牌（随机→手牌首张确定性近似登记）。
pub const BOSS_QIANGZHENG: &str = "强征";
/// 醉酒（Boss 白无常，规则 v255）：锁定技——你使用的【杀】造成的伤害
/// +1（上游 damageBegin1 sha+shaParent 同口径）。
pub const BOSS_ZUIJIU: &str = "醉酒";
/// 枭首（Boss 牛头，规则 v255）：锁定技——结束阶段对一名体力值不小于你
/// 的其他角色造成 3 点伤害（目标自选可放弃→恒发动座位序首个 hp≥self 其
/// 他存活确定性近似登记）。
pub const BOSS_XIAOSHOU: &str = "枭首";
/// 御兽（Boss 罗刹，规则 v254）：锁定技——出牌阶段开始视为对全部敌方
/// 使用【南蛮入侵】（虚拟牌机器直发；异侧存活全目标）。
pub const BOSS_YUSHOU: &str = "御兽";
/// 魔炎（Boss 罗刹，规则 v254）：锁定技——非当前回合失去牌后判定红色 →
/// 对一名其他角色造成 2 点火焰伤害（目标自选→座位序首个其他存活确定性
/// 近似登记；loseEnd 全牌损失面→CardsDiscarded 面登记）。
pub const BOSS_MOYANY: &str = "魔炎";
/// 胆术（Boss 夜叉，规则 v252）：锁定技——失去手牌后（非当前回合且已
/// 受伤）判定红色回 1（CardsDiscarded 面；gain/addJudge/equip 等其余卡
/// 牌移动面登记差异）。
pub const BOSS_DANSHU: &str = "胆术";
/// 精觉（年兽精觉形态，规则 v252）：锁定技——继承胆术（同内容共用
/// commit_cards_discarded 臂，按名双查）。
pub const BOSS_JINGJUE: &str = "精觉";
/// 绝策（Boss 马面，规则 v258，一将包李儒旧绝策）：锁定技——自己回合
/// 内其他角色失去最后一手牌（弃后空手）→ 对其造成 1 点伤害（chooseBool
/// AI→恒发动确定性近似；lose/equip/gain 全事件族→CardsDiscarded 面登记）。
pub const BOSS_JUECE: &str = "绝策";
/// 酒池（Boss 董卓，规则 v258，群雄乱入）：锁定技——黑桃手牌可当【酒】
/// 使用（viewAs 转化：奇袭/武圣同族 PlayCard kind 改写臂；与奇袭[黑→拆]
/// 上游同持时由玩家择技——native if/else 链奇袭优先，登记）。
pub const BOSS_JIUCHI: &str = "酒池";
/// 鬼谋（Boss 郭嘉，规则 v259，群雄乱入）：锁定技——结束阶段随机一名
/// 其他未混乱角色进入混乱状态，其回合结束时解除（随机→座位序首个确定性
/// 近似登记；上游 mad 为纯标记技+AI 态度面，原生规则面消费=鬼谋去重过滤）。
pub const BOSS_GUIMOU: &str = "鬼谋";
/// 远策（Boss 郭嘉，规则 v259，群雄乱入）：锁定技——每当其他角色对另一
/// 其他角色造成伤害后，你选择一项：①判定，黑则受伤角色失 1 体力否则来
/// 源失 1 体力；②判定，红则受伤角色回 1 体力否则来源回 1 体力（chooseControl
/// AI 态度面+rejudge stop 检查→恒选选项一确定性近似登记；判定修正回调
/// 为 AI 选牌面不落，判定=翻牌取色）。
pub const BOSS_YUANCE: &str = "远策";
/// 奇佐（Boss 郭嘉，规则 v259，群雄乱入）：锁定技——你使用锦囊牌结算
/// 结束后，你使用一张同名牌（不可用则不发动）——副本以牌堆底实体为载体
/// 走 use_virtual_card 完整管线（v169 原语；上游凭空造牌→载体弃置登记）；
/// 触发时机 useCardAfter→CardUsed AfterTriggers 窗口近似（副本效果先于
/// 原牌效果）；铁索不复制（上游 AI check 排除同口径）；无懈不可复制。
pub const BOSS_QIZUO: &str = "奇佐";
/// 绝情（Boss 张春华，规则 v260，群雄乱入；一将包张春华旧绝情）：锁定
/// 技——你造成的伤害均改为目标失去等量体力（damageBefore cancel+loseHp
/// (num)；num=增幅前原始值——原生 commit 入口直读取 d.amount 同口径；
/// 取消后伤害事件不再结算=受击/加成类不触发）。
pub const JUE_QING: &str = "绝情";
/// 无心（Boss 张春华，规则 v260，群雄乱入）：锁定技两半——①你不能使用
/// 延时锦囊指定其他角色为目标（mod targetEnabled；原生乐/兵路由本就只
/// 能置他人→等价于持有者不可用乐/兵；闪电自置不受限）；②你受到的伤害
/// 改为失去 1 点体力（damageBefore priority 10 cancel+loseHp() 无参=1）。
pub const BOSS_WUXIN: &str = "无心";
/// 伤逝（Boss 张春华，规则 v260，群雄乱入）：锁定技——每当你的牌失去后
/// 或体力变化后，若手牌数小于 4，你将手牌补至 4 张（loseEnd/changeHp 双
/// 触发→原生 CardsDiscarded/Damage/LoseHp/Recover 四 commit 臂）；子技
/// shangshix2（group）：结束阶段若你的体力大于 1，全体存活角色（含自己）
/// 各失去 1 点体力。
pub const BOSS_SHANGSHIX: &str = "伤逝";
/// 枭雄（Boss 刘备，规则 v261，群雄乱入）：锁定技两半——①其他角色使用
/// 基本牌或锦囊牌结算结束后，你获得该牌的副本（上游 createCard 凭空造牌
/// →牌堆底实体入手登记差异：花色点数随载体+消耗库顶实体）；②子技
/// xiaoxiong_damage（group）：其他角色的结束阶段，若其本回合未使用过牌
///（上游 countUsed()==0→原生出牌阶段用牌计数==0 登记差异：响应/装备不
/// 计），你对其造成 1 点伤害。
pub const BOSS_XIAOXIONG: &str = "枭雄";
/// 章武（Boss 刘备，规则 v261，群雄乱入）：锁定技——你受到伤害后（来源
/// 存活且非自己），你弃置一张牌，来源选择：弃置一张牌，或受到 1 点伤害
///（上游刘备 [1,∞] 自选+来源 chooseToDiscard/取消双 AI 态度面→双方恒弃
/// 首张确定性近似登记；上游 num=刘备弃牌数可多张登记）。
pub const BOSS_ZHANGWU: &str = "章武";
/// 凌波（Boss 甄姬，规则 v262，群雄乱入）：锁定技——你使用或打出【闪】
/// 后摸两张牌（frequent 无询问；上游 [respond, useCard] 双触发→原生
/// CardResponded 触发+CardUsed extra 触发；viewAs 闪按声明牌种）。
pub const BOSS_LINGBO: &str = "凌波";
/// 娇孝（Boss 甄姬，规则 v262，群雄乱入）：锁定技——你成为红色牌的目标
/// 后摸一张牌（frequent；上游 target useCardToTargeted→原生 CardUsed
/// commit 臂 targets 含持有者；红色=实体花色 Heart/Diamond）。
pub const BOSS_JIAOXIA: &str = "娇孝";
/// 芳华（Boss 甄姬，规则 v262，群雄乱入）：锁定技——结束阶段若存在翻面
/// 角色，全部翻面角色各失去 1 点体力。
pub const BOSS_FANGHUA: &str = "芳华";
/// 踏水（Boss 甄姬，规则 v262，群雄乱入）：锁定技——你使用或打出黑色牌
/// 后，将一名其他角色翻面（chooseTarget AI 态度面→座位序首个其他存活
/// 确定性近似登记；经 TurnOverBefore 走翻面提交=虚失同款通道）。
pub const BOSS_TASHUI: &str = "踏水";
/// 火神（Boss 周瑜，规则 v263，群雄乱入）：锁定技——火属性伤害对你无效，
/// 你回复 1 点体力（上游 damageBegin1 cancel+recover() 无参=1 同口径）。
pub const BOSS_HUOSHEN: &str = "火神";
/// 红莲·周瑜（Boss 周瑜，规则 v263，群雄乱入；上游 boss_honglian 与毕方
/// boss_honglianx 显示名同为「红莲」，按迅影批改名映射先例以「红莲·周瑜」
/// 入目录登记）：锁定技——结束阶段你摸两张牌，并对所有敌人各造成 1 点
/// 火焰伤害。
pub const BOSS_ZYHONGLIAN: &str = "红莲·周瑜";
/// 仙音（Boss 周瑜，规则 v263，群雄乱入）：锁定技——非你的回合内，你每
/// 失去一次牌后进行判定：红色则一名其他角色失去 1 点体力（chooseTarget
/// AI 态度面→座位序首个其他存活确定性近似登记）。
pub const BOSS_XIANYIN: &str = "仙音";
/// 天道（Boss 张角，规则 v263，群雄乱入）：锁定技——判定牌确定前，你可
/// 打出一张手牌或装备牌替换判定牌，并获得原判定牌（改判机 rewriters 扩
/// 展：hes 任意牌，鬼道/鬼才同族）。
pub const BOSS_TIANDAO: &str = "天道";
/// 雷击·张角（Boss 张角，规则 v263，群雄乱入；上游 boss_leiji 与青龙
/// boss_releiji 显示名同为「雷击」，按迅影批改名映射先例以「雷击·张角」
/// 入目录登记）：锁定技——你使用或打出【闪】后，一名其他角色判定：黑则
/// 其受到 1 点雷伤且你摸一张牌（chooseTarget AI→座位序首个确定性近似）。
pub const BOSS_ZJLEIJI: &str = "雷击·张角";
/// 极点（Boss 张角，规则 v263，群雄乱入）：锁定技——你造成伤害后，距离
/// 受伤者 1 以内的一名其他角色判定：黑则其受到 1 点雷伤（chooseTarget
/// AI→座位序首个确定性近似；不自我递归=链挡旗标登记差异）。
pub const BOSS_JIDIAN: &str = "极点";
/// 枳枉（十殿阎罗五官王，规则 v265，群雄乱入）：锁定技——其他角色获得
/// 牌结算后（非摸牌阶段摸牌），其随机弃置一张手牌（随机→首张确定性近
/// 似登记；隐藏彩蛋分支 planetarian 孙策化不落登记）。
pub const BOSS_ZHIWANG: &str = "枳枉";
/// 公正（十殿阎罗五官王，规则 v265，群雄乱入）：锁定技——准备阶段（原生
/// 判定阶段开始锚点）若你的判定区有牌，你随机弃置其中一张（随机→首张
/// 确定性近似登记；鬼计同款臂）。
pub const BOSS_GONGZHENG: &str = "公正";
/// 血池（十殿阎罗五官王，规则 v265，群雄乱入）：锁定技——结束阶段随机
/// 一名其他角色失去 2 点体力（随机→座位序首个确定性近似登记）。
pub const BOSS_XUECHI: &str = "血池";
/// 黑绳（十殿阎罗宋帝王，规则 v266，群雄乱入）：锁定技——你死亡时，全
/// 部其他角色横置（上游 die forceDie 逐个 link() 同口径；死亡前置窗口触
/// 发=changan 氏遗言同款；若死亡被取消仍已横置=登记差异）。
pub const BOSS_HEISHENG: &str = "黑绳";
/// 胜负（十殿阎罗宋帝王，规则 v266，群雄乱入）：锁定技——结束阶段，一
/// 名装备区有坐骑牌的其他角色弃置其中一张（随机目标/随机牌→首个有牌者
/// /首张确定性近似登记；宝物 equip6 面随宝物栏机制登记）。
pub const BOSS_SHENGFU: &str = "胜负";
/// 雷舟（十殿阎罗卞城王，规则 v267，群雄乱入）：锁定技——准备阶段，随
/// 机一名其他角色受 1 点雷伤（randomGet→座位序首个确定性近似登记）。
pub const BOSS_LEIZHOU: &str = "雷舟";
/// 雷斧（十殿阎罗卞城王，规则 v267，群雄乱入）：锁定技——结束阶段，随
/// 机一名其他角色横置态切换（上游 link() 非恒链同口径；randomGet→首个
/// 确定性近似登记）。
pub const BOSS_LEIFU: &str = "雷斧";
/// 雷诛（十殿阎罗卞城王，规则 v267，群雄乱入）：锁定技——你死亡时，按
/// 座位序对全部其他角色各造成 1 点雷伤（die forceDie 遗言面，黑绳同位）。
pub const BOSS_LEIZHU: &str = "雷诛";
/// 服毒（十殿阎罗泰山王，规则 v268，群雄乱入）：锁定技——其他角色使用
/// 【桃】时，随机另一名其他角色失去 1 点体力（randomGet→座位序首个确
/// 定性近似登记；勘误——勘验存量误记「复读」）。
pub const BOSS_FUDU: &str = "服毒";
/// 苦酒（十殿阎罗泰山王，规则 v268，群雄乱入）：锁定技——其他角色准备
/// 阶段，其失去 1 点体力，然后视为使用一张【酒】（jiu_active 置位，不
/// 占 jiu_used；无 useCard 事件面=登记差异）。
pub const BOSS_KUJIU: &str = "苦酒";
/// 热恼（十殿阎罗泰山王，规则 v268，群雄乱入）：锁定技——你死亡时，随
/// 机一名其他角色受 3 点火焰伤害（die forceDie 遗言面，雷诛同位）。
pub const BOSS_RENAO: &str = "热恼";
/// 轮回（十殿阎罗转轮王，规则 v269，群雄乱入）：锁定技——准备阶段，若
/// 你的体力不大于 2，你与体力最高且大于 2 的一名其他角色交换体力值
///（randomGet→座位序首个确定性近似登记；交换直写 min(maxHp) 封顶）。
pub const BOSS_LUNHUI: &str = "轮回";
/// 往生（十殿阎罗转轮王，规则 v269，群雄乱入）：锁定技——出牌阶段开始
/// 时，视为随机使用一张【南蛮入侵】/【万箭齐发】（randomGet→恒定取南
/// 蛮近似登记；use_virtual_card 载体管线，目标=全部其他存活）。
pub const BOSS_WANGSHENG: &str = "往生";
/// 反噬（十殿阎罗转轮王，规则 v269，群雄乱入）：锁定技——每回合第一次
/// 受伤后，若再次受到伤害，对随机一名其他角色造成 1 点伤害（上游
/// addTempSkill boss_zlfanshi_terra 交替同口径；随机→首个确定性近似）。
pub const BOSS_ZLFANSHI: &str = "反噬";
/// 拾忧（十殿阎罗孟婆，规则 v270，群雄乱入）：锁定化近似（上游可选技
/// 恒发动确定性近似登记）——其他角色弃牌阶段弃置牌后，获得其中花色各
/// 不相同的牌（随机/自选→按弃牌序贪心取首个未见花色确定性近似；显示
/// 名勘误——勘验存量误记「拾又」）。
pub const BOSS_SHIYOU: &str = "拾忧";
/// 往事（十殿阎罗孟婆，规则 v270，群雄乱入）：锁定技——其他角色的回合
/// 开始时，其本回合不能使用或打出一种类型的牌（上游 randomGet 基本锦
/// 囊装备→恒定基本牌确定性近似登记；敌方面→全部其他近似；打出面响应
/// 窗未落=登记差异；显示名勘误——勘验存量误记「忘世」）。
pub const BOSS_WANGSHI: &str = "往事";
/// 忘魂（十殿阎罗孟婆，规则 v270，群雄乱入）：锁定遗言（Death 触发）——
/// 清全场往事封禁，随机两名敌方角色各随机失去一个技能（随机敌将/随机
/// 技→座位序前两个其他存活各首个启用技确定性近似登记；回魂牌堆注入与
/// 回收面、shidianyanluo 模式旗未落随 boss 模式扩展批）。
pub const BOSS_WANGHUN: &str = "忘魂";
/// 铁面（十殿阎罗阎罗王，规则 v271）：锁定技——防具区没有牌时，视为
/// 装备【仁王盾】（sha 机仁王块扩展：持有者防具槽空 → 黑杀无效；青釭/
/// 铡刀穿透天然覆盖）。
pub const BOSS_TIEMIAN: &str = "铁面";
/// 铡刀（十殿阎罗阎罗王，规则 v271）：锁定技——你使用【杀】指定目标后，
/// 令目标角色防具无效（上游 inherit qinggang_skill；sha 机青釭穿透链
/// 扩展：来源持有者即穿透）。
pub const BOSS_ZHADAO: &str = "铡刀";
/// 诛心（十殿阎罗阎罗王，规则 v271）：锁定遗言（Death 触发）——你死亡
/// 时，令场上血量最少的一名其他角色受到 2 点伤害（上游 chooseTarget
/// 最低体力池自选→座位序首个最低体力确定性近似登记）。
pub const BOSS_ZHUXIN: &str = "诛心";
/// 锁足（十殿阎罗平等王，规则 v272）：锁定技——准备阶段，全部其他存活
/// 角色横置（上游 boss_suozu content link 同口径；不分阵营）。
pub const BOSS_SUOZU: &str = "锁足";
/// 阿鼻（十殿阎罗平等王，规则 v272）：锁定技——你受到有来源的伤害后，
/// 来源受 1 点火焰伤害（上游 randomGet fire/thunder→恒定火确定性近似
/// 登记；无来源伤害不触发）。
pub const BOSS_ABI: &str = "阿鼻";
/// 平等（十殿阎罗平等王，规则 v272）：锁定遗言（Death 触发）——两波：
/// 最高体力其他存活（座位序首个）先受 2 点火焰伤害、重算后再受 1 点火
/// 焰伤害（上游两波 randomGet×lib.linked randomGet→座位序首个+恒定火
/// 确定性近似登记）。
pub const BOSS_PINGDENG: &str = "平等";
/// 热闷（十殿阎罗都市王，规则 v273）：锁定技——防具区没有牌时，【杀】
///（无属性，plain Sha）/【南蛮入侵】/【万箭齐发】对其无效（上游
/// useCardToBefore cancel 同口径；PlayCard 拒绝位+AOE 机窗口前跳过；
/// 属性杀不覆盖）。
pub const BOSS_REMEN: &str = "热闷";
/// 炙焚（十殿阎罗都市王，规则 v273）：锁定技——准备阶段随机一名其他
/// 角色获得其一张随机手牌（若有）并受 1 点火焰伤害（上游
/// gainPlayerCard randomGet→座位序首个+首张确定性近似登记；伤害无来
/// 源同上游）。
pub const BOSS_ZHIFEN: &str = "炙焚";
/// 火刑（十殿阎罗都市王，规则 v273）：锁定遗言（Death 触发）——全部
/// 其他存活角色按座位序各受 1 点火焰伤害（上游 sort seat 逐发同口径；
/// 中途死亡由存活过滤跳过）。
pub const BOSS_HUOXING: &str = "火刑";
/// 不佛（十殿阎罗地藏王，规则 v274）：锁定技两面——受到大于 1 点的伤
/// 害时，改为其中 1 点（上游 damageBegin4 num>1→num-1 同口径，灵渠入
/// 口同区）；准备阶段距离 1 以内的其他存活角色受 1 点火焰伤害（有来源
/// 同上游）。
pub const BOSS_BUFO: &str = "不佛";
/// 无量（十殿阎罗地藏王，规则 v274）：锁定技——结束阶段摸 2 张牌；准
/// 备阶段体力小于 3 时回复至 3 点（上游 phaseJieshu draw 2 + recover
/// 同口径）。建局摸 3 面（gameDrawAfter/enterGame）未落随装配批登记。
pub const BOSS_WULIANG: &str = "无量";
/// 谛听（十殿阎罗地藏王，规则 v274）：锁定技——你计算与其他角色的距离
/// -1；其他角色计算与你的距离 +1（上游 mod globalFrom/-1+globalTo/+1
/// 同口径；义从同位）。
pub const BOSS_DITING: &str = "谛听";
/// 帷幕（十殿阎罗楚江王，规则 v275，共用新落）：锁定技——黑色锦囊牌对
/// 你无效（显式目标黑锦囊 PlayCard 拒绝位+黑色 AOE 机窗口前跳过；无指
/// 向锦囊无指向面天然不适用=登记；红杀/属性杀不属锦囊不覆盖）。
pub const BOSS_WEIMU: &str = "帷幕";
/// 冰封（十殿阎罗楚江王，规则 v275）：锁定遗言（Death 触发）——伤害来
/// 源存活且未翻面时，将其翻面（上游 turnOver 同口径；已翻面不重复翻）。
pub const BOSS_BINGFENG: &str = "冰封";
/// 判官（十殿阎罗秦广王，规则 v276）：锁定技——延时锦囊对你无效
///（上游 mod targetEnabled delay→目标不可指定近似登记；乐/闪电；兵粮
/// 寸断不在原生目录）。
pub const BOSS_PANGUAN: &str = "判官";
/// 拘魂（十殿阎罗秦广王，规则 v276）：锁定技——结束阶段随机一名其他
/// 角色翻面或横置（上游 ["turnOver","link"] randomGet→恒定翻面+座位序
/// 首个确定性近似登记）。
pub const BOSS_JUHUN: &str = "拘魂";
/// 望乡（十殿阎罗秦广王，规则 v276）：锁定遗言（Death 触发）——全部
/// 有装备的其他角色装备全弃（撤技能+入弃堆；绝红同款拆装管线）。
pub const BOSS_WANGXIANG: &str = "望乡";
/// 天狱（群雄乱入 Boss 庞统[涅槃凤雏]，规则 v277）：锁定技——结束阶段
/// 自己已横置则先解除，再将全部未横置的其他存活横置（上游
/// ["turnOver"...boss_tianyu link 同口径；炼狱/锁足同款臂）。
pub const BOSS_TIANYU: &str = "天狱";
/// 栖梧（群雄乱入 Boss 庞统，规则 v277）：锁定技——使用梅花牌后若已受
/// 伤则回复 1 点体力（上游 useCard club+hp<maxHp recover 同口径；事件面
/// 与娇孝臂同族）。
pub const BOSS_QIWU: &str = "栖梧";
/// 炼狱（Boss 马面，规则 v257）：锁定技——结束阶段对全部其他存活角色
/// 各造成 1 点火焰伤害（无询问直伤）。
pub const BOSS_LIANYU: &str = "炼狱";
/// 守翼（年兽守翼形态，规则 v251）：锁定技——你使用所有牌无距离限制
///（上游 mod targetInRange恒 true 同口径；sha/顺手两距离链追加）。
pub const BOSS_SHOUYI: &str = "守翼";
/// 睿智（年兽睿智形态，规则 v250）：锁定技——准备阶段其余角色（手牌
/// >1 或装备 >1）须保留一张手牌和一张装备区内的牌，弃置其余（保留哪张
/// 由目标自选——原生保留首张确定性近似登记）。
pub const BOSS_RUIZHI: &str = "睿智";
/// 形态轮换（Boss 年兽系，规则 v249）：锁定技——回合开始交替轮换至另
/// 一「形态轮换」携带将（assembly 重挂+hp/手牌/装备/判定区保留；首次置
/// 位不换将）。随机序列→确定性互斥轮转登记。
pub const BOSS_NBIANSHEN: &str = "形态轮换";
/// 忍性（年兽忍性形态，规则 v249）：锁定技——持有者非当前回合角色时，
/// 任意角色受到伤害或回复后其摸 1 张牌。
pub const BOSS_RENXING: &str = "忍性";
/// 年瑞（Boss 年兽，规则 v248）：锁定技——摸牌阶段额外摸 2 张牌。
pub const BOSS_NIANRUI: &str = "年瑞";
/// 猛态（Boss 年兽，规则 v248）：锁定技——准备阶段置位双旗标：本回合
/// 未进入摸牌阶段 → 结束阶段摸 3；未进入出牌阶段 → 跳过弃牌阶段
/// （TurnState.mengtai_draw_pending/use_pending 承载）。形态轮换
/// （boss_nbianshen）未落随下批。
pub const BOSS_MENGTAI: &str = "猛态";
/// 自润（Boss 玄冥，规则 v247）：锁定技——准备阶段全体角色按座位序各
/// 摸牌：有装备者摸 2、无装备者摸 1。
pub const BOSS_ZIRUN: &str = "自润";
/// 造异（Boss 颛顼，规则 v246）：锁定技——玄冥/共工死亡且场上无存活
/// 同名将 → 摸 4 并获「造异异变」（运行时授予技：回合开始敌方中体力最
/// 低者全体失去全部体力）；玄冥在场 → 敌方不可为锦囊目标、共工在场 →
/// 敌方不可为基本牌目标（targetEnabled 双门控；玄冥未入库随自润批）。
pub const BOSS_ZAOYI: &str = "造异";
/// 造异异变（造异运行时授予技，规则 v246）：回合开始——敌方中体力最低
/// 者全体失去全部体力。
pub const BOSS_ZAOYI_HP: &str = "造异异变";
/// 庆祝（Boss 明星，规则 v245）：锁定技两面——跳过弃牌阶段；自己回合
/// 内出牌阶段不能用【杀】，直到获得急叮（上游 mod cardEnabled 同口径）。
pub const BOSS_QINGZHU: &str = "庆祝";
/// 夹击（Boss 明星，规则 v245）：锁定技——准备阶段相邻（环上下家）敌方
/// 中有坐骑（+1/-1 马槽）者 → 坐骑全弃。
pub const BOSS_JIAZU: &str = "夹击";
/// 急叮（Boss 明星，规则 v245）：锁定技——友方（同侧非己）受敌方来源伤
/// 害后，对来源用虚拟雷杀（机器直发不经 PlayCard 校验；蓐收加成尾面登
/// 记差异未建模）。**授予面**：由星囚（蓐收）addSkills 授予——不随明星
/// 条目装配（庆祝禁杀窗口依赖初始缺席）。
pub const BOSS_JIDING: &str = "急叮";
/// 绝红（Boss 共工，规则 v244）：锁定技——准备阶段敌方全体逐个：有牌
/// 者，有装备 → 装备全弃（撤技能+入弃堆）；无装备 → 随机弃一张手牌
/// （随机→手牌首张确定性近似登记先例族）。
pub const BOSS_JUEHONG: &str = "绝红";
/// 星夏（Boss 祝融，规则 v242）：出牌阶段主动技——对场上焰灵造成 2 点
/// 火焰伤害；随后可选一名敌方：其弃一张红色牌（手牌+装备）或受 1 点火
/// 焰伤害（可放弃追击）。间隔 ≥2 轮可再发动
/// （PlayerState.xingxia_used_round 登记）。
pub const BOSS_XINGXIA: &str = "星夏";
/// 回火（Boss 焰灵，规则 v241）：锁定技两面——【杀】使用上限 +1（每
/// 回合一次额外额度，SkillInstance.once_used 随本人回合开始重置）；死
/// 亡反噬——死亡结算完成后其敌方全体各受 3 点无来源火伤（上游
/// boss_huihuo2 global dieAfter 同口径）。
pub const BOSS_HUIHUO: &str = "回火";
/// 复燃（Boss 焰灵，规则 v241）：焰灵濒死时其敌方可用红色牌当【桃】
/// 救（上游 boss_furan2 dying chooseToUse viewAs tao 同口径）。
pub const BOSS_FURAN: &str = "复燃";
/// 不臣（Boss 勾芒，规则 v240）：出牌阶段主动技两线——场上有已死亡的
/// 书精 → 失去 1 点体力并把它们全部复活（回 1 体、各摸 2）；否则令一
/// 名己方受伤角色回复 2 点体力。间隔 ≥2 轮可再发动
/// （PlayerState.buchun_used_round 登记）。
pub const BOSS_BUCHUN: &str = "不臣";
/// 星囚（Boss 蓐收，规则 v239）：锁定技——出牌阶段开始交替横置全部敌
/// 方（每回合开始开关取反，true 回合把未横置的敌方全部横置；只链不解，
/// 已横置保持）。对 boss_mingxingzhu 授 boss_jiding 面依赖武将未入库，
/// 登记差异。
pub const BOSS_XINGQIU: &str = "星囚";
/// 神恩（Boss 太昊/炎帝 光环，规则 v238）：全局光环——boss 方用牌无视
/// 距离；挑战者侧手牌上限 +1、摸牌阶段 +1（上游 boss_shenen2 global
/// 三 mod 同口径：targetInRange side→true / maxHandcard !side→num+1 /
/// phaseDrawBegin !side→num++；光环激活=场上存在神恩持有者）。
pub const BOSS_SHENEN: &str = "神恩";
/// 灵渠（Boss 玄武，规则 v237）：锁定技两面——每次受到伤害后摸 1 张牌
/// 且受击计数 +1（手牌上限 +X，跨回合持久）；受到的大于 1 点的伤害视为
/// 0 点（上游 boss_lingqu cancel damageBegin4 num>1 → num=0 同口径，
/// 0 点伤害仍完成事件→摸 1/计数照常）。boss_xuanlin2 玄麟=关卡流程面，
/// 登记不落内核。
pub const BOSS_LINGQU: &str = "灵渠";
/// 焚天（Boss 朱雀，规则 v236）：锁定技多面——你造成的伤害视为火属性
/// （与招祸同构，复用 ZhaoHuo 机器/快照变体）；红色牌使用无次数限制且
/// 无距离限制（上游 mod cardUsable Infinity + targetInRange=true）；红
/// 色杀无视防具（boss_fentian2 directHit 全场——原生防具=八卦阵，跳防
/// 具判定承载）。wuxieRespondable 红色牌禁他人无懈面未挂钩，登记差异。
pub const BOSS_FENTIAN: &str = "焚天";
/// 狂笑（Boss 白虎，规则 v235）：锁定技两面——你的【杀】无距离限制且
/// 目标数上限无限（上游 mod targetInRange=true + selectTarget
/// range[0..1]=-1/-1 同口径）；不能指定友方为杀目标（playerEnabled）——
/// boss 侧同队关系网未入原生 PlayCard 校验，登记差异。
pub const BOSS_KUANGXIAO: &str = "狂笑";
/// 雷击（Boss 青龙，规则 v234，上游 releiji 界雷击）：你使用/打出闪后
/// 选一名其他角色判定——黑桃其受 2 点雷伤、梅花你回 1 其受 1 点雷伤、
/// 其他无事（原生=杀/万箭两闪消费位内联臂；目标选择以座位序首名确定
/// 性近似登记；判定=铁骑内联口径无鬼道/天妒改判窗；闪无主动使用面，
/// useCard 触发半面由模型承载）。
pub const BOSS_RELEIJI: &str = "雷击";
/// 神裔（Boss 青龙，四象系共有技，规则 v233）：锁定技两面——判定阶段
/// 的延时锦囊判定恒不生效（上游 mod judge `event.type=="phase"`：bool
/// true→false、false→null，对延时锦囊消费面等价恒失效；牌去向不变）；
/// 武将牌不能翻至背面（上游 turnOverBefore filter !isTurnedOver——仅
/// 正→背取消，背→正照常翻回）。洪荒之力禁用交互（honghuangzhili）未
/// 落，登记差异。
pub const BOSS_SHENYI: &str = "神裔";
/// 司圃（Boss 应昭，规则 v232）：锁定技，体力值首次减至一半或更少时激
/// 活；你的出牌阶段内已使用牌数≤2 期间，其他角色无法使用牌。cardEnabled/
/// cardUsable 使用面在原生回合模型下由「仅回合玩家可提交使用」天然承载；
/// cardRespondable/cardSavable 按响应/救选项排除建模（无懈等其余响应窗
/// 未纳入禁令，登记差异）。
pub const BOSS_SIPU: &str = "司圃";
/// 红莲（Boss 毕方，规则 v228）：锁定技两面——你的红色牌不计入手牌上
/// 限（弃牌阶段红色牌不可弃）；回合开始随机获得牌堆 0–3 张红色牌（确
/// 定性近似：顺序取牌堆前缀红牌至多 3），并随机对 3–0 名其他角色各造
/// 成 1 点火属性伤害（确定性近似：座位序前至多 3 名）。
pub const BOSS_HONGLIANX: &str = "红莲";
/// 炎狱（Boss 毕方，规则 v229）：锁定技，体力值首次减至一半或更少时激
/// 活；其他角色回合开始（判定阶段开始锚点）判定，红色 → 其受 1 点来
/// 自你的火伤（每回合至多 3 次）。
pub const BOSS_YANYU: &str = "炎狱";
/// 虚失（Boss 朱厌，规则 v225）：锁定技，体力值首次减至一半或更少时激
/// 活；你的出牌阶段结束时翻面；武将牌翻回正面时对所有其他角色各造成
/// 1 点伤害（上游 [1,2] 随机——恒 1 登记差异）。
pub const BOSS_XUSHI: &str = "虚失";
/// 马超「铁骑」：锁定式判定——使用杀指定目标后进行判定，若判定牌为红色，
/// 该角色不能使用闪响应此杀。
pub const TIE_JI: &str = "铁骑";

/// 完整武将授予（N08 内容扩面）：武将名 → 技能集。
/// 授予武将即同时授予其全部技能。「张角」为官方技能迁移；
/// 「样例」前缀条目为 ADR 0712 机制样例组合（不计官方技能迁移）。
/// 武将性别（离间/结姻等性别门控用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gender {
    Male,
    Female,
}

pub const STANDARD_GENERALS: &[(&str, Faction, &[&str], u32, Gender)] = &[
    // 四元组：名称 / 势力 / 技能集 / 体力上限（N11 将领装配，ADR 0769）
    ("样例·张角组合", Faction::Qun, &[GUI_DAO, TIAN_DU], 3, Gender::Male),
    ("司马懿", Faction::Wei, &[GUI_CAI, FAN_KUI], 3, Gender::Male),
    ("曹操", Faction::Wei, &[JIAN_XIONG, HU_JIA], 4, Gender::Male),
    ("貂蝉", Faction::Qun, &[BI_YUE, LI_JIAN], 3, Gender::Female),
    ("甄姬", Faction::Wei, &[LUO_SHEN, QING_GUO], 3, Gender::Female),
    ("关羽", Faction::Shu, &[WU_SHENG], 4, Gender::Male),
    ("赵云", Faction::Shu, &[LONG_DAN], 4, Gender::Male),
    ("吕布", Faction::Qun, &[WU_SHUANG], 4, Gender::Male),
    ("黄月英", Faction::Shu, &[JI_ZHI, QI_CAI], 3, Gender::Female),
    ("大乔", Faction::Wu, &[GUO_SE, LIU_LI], 3, Gender::Female),
    ("吕蒙", Faction::Wu, &[KE_JI], 4, Gender::Male),
    ("甘宁", Faction::Wu, &[QI_XI], 4, Gender::Male),
    ("张飞", Faction::Shu, &[PAO_XIAO], 4, Gender::Male),
    ("周瑜", Faction::Wu, &[YING_ZI, FAN_JIAN], 3, Gender::Male),
    ("孙尚香", Faction::Wu, &[XIAO_JI, JIE_YIN], 3, Gender::Female),
    ("华佗", Faction::Qun, &[QING_NANG, JI_JIU], 3, Gender::Male),
    ("黄盖", Faction::Wu, &[KU_ROU], 4, Gender::Male),
    ("孙权", Faction::Wu, &[ZHI_HENG, JIU_YUAN], 4, Gender::Male),
    ("郭嘉", Faction::Wei, &[TIAN_DU, JI_JI], 3, Gender::Male),
    ("夏侯惇", Faction::Wei, &[GANG_LIE], 4, Gender::Male),
    ("诸葛亮", Faction::Shu, &[GUAN_XING, KONG_CHENG], 3, Gender::Male),
    ("陆逊", Faction::Wu, &[QIAN_XUN, LIAN_YING], 3, Gender::Male),
    ("刘备", Faction::Shu, &[REN_DE, JI_JIANG], 4, Gender::Male),
    ("马超", Faction::Shu, &[MA_SHU, TIE_JI], 4, Gender::Male),
    ("张辽", Faction::Wei, &[TU_XI], 4, Gender::Male),
    ("许褚", Faction::Wei, &[LUO_YI], 4, Gender::Male),
    ("曹仁", Faction::Wei, &[JU_SHOU], 4, Gender::Male),
    ("样例·策援", Faction::Qun, &[YI_SHANG, SHOU_YI], 4, Gender::Male),
    ("样例·守财", Faction::Qun, &[SUI_DUN, SAN_CAI, SU_QING], 4, Gender::Female),
    // P2/N08 标准缺失将补齐（2026-10-07，上游 standard/character.js 对齐）：
    ("乐进", Faction::Wei, &[XIAO_GUO], 4, Gender::Male),
    ("潘凤", Faction::Qun, &[KUANG_FU], 4, Gender::Male),
    ("华雄", Faction::Qun, &[YAO_WU], 6, Gender::Male),
    ("甘夫人", Faction::Shu, &[SHU_SHEN, SHEN_ZHI], 3, Gender::Female),
    ("李典", Faction::Wei, &[XUN_XUN, WANG_XI], 3, Gender::Male),
    ("公孙瓒", Faction::Qun, &[YI_CONG], 4, Gender::Male),
    ("伊籍", Faction::Shu, &[JI_JIE, JI_YUAN], 3, Gender::Male),
    ("袁术", Faction::Qun, &[WANG_ZUN, TONG_JI], 4, Gender::Male),
];

/// Boss 模式内置武将（上游 mode/boss.js characterPack.mode_boss 首批：
/// 规则 v208 烛阴；追加于 STANDARD 索引空间尾——既有将索引不受扰动）。
pub const BOSS_GENERALS: &[(&str, Faction, &[&str], u32, Gender)] = &[
    ("烛阴", Faction::Qun, &[BOSS_XIONGSHOU], 4, Gender::Male),
    // 混沌（上游 boss_hundun hp25）：隐贼以「体力≤一半条件内生效」近似
    // 上游 switch 激活门控（首次激活永久语义登记差异）
    ("混沌", Faction::Qun, &[BOSS_XIONGSHOU, BOSS_WUZANG, BOSS_XIANGDE, BOSS_YINZEI], 25, Gender::Male),
    // 穷奇（上游 boss_qiongqi hp20/maxHp25）：逐厄单技本批；覆泰(+全场
    // 禁桃)/延妒(+switch) 未落随下批（maxHp 差异登记：max_hp 暂=hp=20）
    // 延妒已落（v213，激活门控以 SkillInstance.enabled 同隐贼先例）；
    // 覆泰2 全场禁桃未落随下批
    ("穷奇", Faction::Qun, &[BOSS_XIONGSHOU, BOSS_ZHUE, BOSS_FUTAI, BOSS_YANDU], 20, Gender::Male),
    // 饕餮四技全齐（v215 贪欲；v216 藏目+集财——集财激活以
    // SkillInstance.enabled 同隐贼/延妒先例）
    ("饕餮", Faction::Qun, &[BOSS_XIONGSHOU, BOSS_TANYU, BOSS_CANGMU, BOSS_JICAI], 20, Gender::Male),
    // 梼杌四技全齐（v217 拟态；v218 乱常；v219 冥顽——目标锁定集以
    // TurnState.minwan_targets 回合级承载）
    ("梼杌", Faction::Qun, &[BOSS_XIONGSHOU, BOSS_NITAI, BOSS_LUANCHANG, BOSS_MINWAN], 25, Gender::Male),
    // 毕方四技全齐（v227 招祸；v228 红莲；v229 炎狱——激活门控以
    // SkillInstance.enabled 同隐贼先例；每回合至多 3 以 TurnState 计数器）
    ("毕方", Faction::Qun, &[BOSS_YAOSHOU, BOSS_ZHAOHUO, BOSS_HONGLIANX, BOSS_YANYU], 25, Gender::Male),
    // 书精（上游 boss_shujing hp2 女神）：催毒+中毒全技（中毒为运行时
    // 授技面，表列仅催毒——授技入实例同 catalog skill 空间）
    ("书精", Faction::Shu, &[BOSS_CUIDUI], 2, Gender::Female),
    // 相柳四技全齐（v221 妖兽；v222 兽九；v223 毒妒+邪酬——蛇毒标记
    // 以 PlayerState.shedu 承载）
    ("相柳", Faction::Qun, &[BOSS_YAOSHOU, BOSS_JIUSHOU, BOSS_DUQU, BOSS_ECHOU], 25, Gender::Male),
    // 应昭四技全齐（v228 封冻；v231 巡游；v232 司圃使用面——响应四面
    // 登记差异；激活门控同隐贼先例）
    ("应昭", Faction::Qun, &[BOSS_YAOSHOU, BOSS_FENGDONG, BOSS_XUNYOU, BOSS_SIPU], 25, Gender::Male),
    // 朱厌四技全齐（v224 兵先；v225 虚失；v226 巨源——体力上限+5 由
    // maxHp30 表意[登记同穷奇]；记录=PlayerState.juyuan_storage）
    ("朱厌", Faction::Qun, &[BOSS_YAOSHOU, BOSS_BINGXIAN, BOSS_XUSHI, BOSS_JUYUAN], 25, Gender::Male),
    // 青龙（v233 神裔、v234 雷击；boss_qingmu2 青木=关卡流程面登记不落）
    ("青龙", Faction::Qun, &[BOSS_SHENYI, BOSS_RELEIJI], 4, Gender::Male),
    // 白虎（v235 狂笑；神裔共用 v233；boss_baimang2 白芒=关卡流程面登记不落；
    // 上游势力 shen→群目录近似[ Faction 枚举无神势力，烛阴空势力→群先例]）
    ("白虎", Faction::Qun, &[BOSS_SHENYI, BOSS_KUANGXIAO], 4, Gender::Male),
    // 朱雀（v236 焚天；神裔共用 v233；boss_chiyan2 赤炎未落随下批；
    // 势力 shen→群目录近似；上游 female）
    ("朱雀", Faction::Qun, &[BOSS_SHENYI, BOSS_FENTIAN], 4, Gender::Female),
    // 玄武（v237 灵渠；神裔共用 v233；boss_xuanlin2 玄麟=关卡流程面登记不落；
    // 势力 shen→群目录近似）
    ("玄武", Faction::Qun, &[BOSS_SHENYI, BOSS_LINGQU], 4, Gender::Male),
    // 太昊（v238 神恩光环；神裔共用 v233；boss_qingyi 青翼未落随下批；
    // 上游 names 风|伏羲、shen→群目录近似）
    ("太昊", Faction::Qun, &[BOSS_SHENYI, BOSS_SHENEN], 6, Gender::Male),
    // 蓐收（v239 星囚；神裔共用 v233；boss_baimang3 白芒3=关卡流程面登记不落；
    // 上游 names 嬴|该、shen→群目录近似）
    ("蓐收", Faction::Qun, &[BOSS_SHENYI, BOSS_XINGQIU], 5, Gender::Male),
    // 勾芒（v240 不臣；神裔共用 v233；boss_qingmu3 青木3=已处置关卡面；
    // 上游 names 姬|重、shen→群目录近似）
    ("勾芒", Faction::Qun, &[BOSS_SHENYI, BOSS_BUCHUN], 5, Gender::Male),
    // 焰灵（v241 回火+复燃；上游技能表无神裔——陪将不在神裔名单；
    // shen→群目录近似）
    ("焰灵", Faction::Qun, &[BOSS_HUIHUO, BOSS_FURAN], 4, Gender::Male),
    // 祝融（v242 星夏；神裔共用 v233；boss_chiyan3 赤炎3=已处置关卡面；
    // 上游 names 姜|重黎、shen→群目录近似）
    ("祝融", Faction::Qun, &[BOSS_SHENYI, BOSS_XINGXIA], 5, Gender::Male),
    // 炎帝（v243 纯入库：神裔共用 v233+神恩光环共用 v238；boss_chiyi 赤
    // 翼=青翼同族已处置；上游 names 姜|轨、shen→群目录近似）
    ("炎帝", Faction::Qun, &[BOSS_SHENYI, BOSS_SHENEN], 6, Gender::Male),
    // 少昊（v243 纯入库：神裔共用 v233+神恩光环共用 v238；boss_baiyi 白
    // 翼=青翼同族已处置；上游 names 嬴|挚、shen→群目录近似）
    ("少昊", Faction::Qun, &[BOSS_SHENYI, BOSS_SHENEN], 6, Gender::Male),
    // 共工（v244 绝红；神裔共用 v233；boss_xuanlin3 玄麟3=已处置关卡面；
    // 上游 names 姜|共工、shen→群目录近似）
    ("共工", Faction::Qun, &[BOSS_SHENYI, BOSS_JUEHONG], 5, Gender::Male),
    // 明星（v245 庆祝+夹击随条目；急叮=星囚授予技不随条目装配[上游
    // boss_jiding 由蓐收 boss_xingquiaaddSkills 授予——庆祝禁杀窗口依赖
    // 此初始缺席]；上游 female、shen→群目录近似）
    ("明星", Faction::Qun, &[BOSS_QINGZHU, BOSS_JIAZU], 3, Gender::Female),
    // 颛顼（v246 造异；神裔共用 v233+神恩光环共用 v238；造异异变=运行
    // 时授予技不随条目；上游 names 姬|颛顼、shen→群目录近似）
    ("颛顼", Faction::Qun, &[BOSS_SHENYI, BOSS_SHENEN, BOSS_ZAOYI], 6, Gender::Male),
    // 玄冥（v247 自润；神裔共用 v233；boss_xuanlin3 玄麟3=已处置关卡面；
    // 上游 female、names 嬴|禺强、shen→群目录近似——入库激活造异玄冥半门）
    ("玄冥", Faction::Qun, &[BOSS_SHENYI, BOSS_ZIRUN], 5, Gender::Female),
    // 年兽合体（v248 年瑞+猛态；v249 形态轮换随条目[轮换载体]；
    // shen→群目录近似）
    ("年兽", Faction::Qun, &[BOSS_NIANRUI, BOSS_MENGTAI, BOSS_NBIANSHEN], 12, Gender::Male),
    // 忍性年兽（v249 忍性+形态轮换；年瑞/猛态共用 v248；shen→群目录近似）
    ("忍性年兽", Faction::Qun, &[BOSS_NIANRUI, BOSS_MENGTAI, BOSS_RENXING, BOSS_NBIANSHEN], 12, Gender::Male),
    // 睿智年兽（v250 睿智+形态轮换；年瑞/猛态共用 v248；shen→群目录近似）
    ("睿智年兽", Faction::Qun, &[BOSS_NIANRUI, BOSS_MENGTAI, BOSS_RUIZHI, BOSS_NBIANSHEN], 12, Gender::Male),
    // 守翼年兽（v251 守翼+形态轮换；年瑞/猛态共用 v248；上游形态表含
    // boss_nbaonu[暴怒,未落随批]——条目只列已转换技；shen→群目录近似）
    ("守翼年兽", Faction::Qun, &[BOSS_NIANRUI, BOSS_MENGTAI, BOSS_SHOUYI, BOSS_NBIANSHEN], 12, Gender::Male),
    // 精觉年兽（v252 精觉[继承胆术]+形态轮换；年瑞/猛态共用 v248；shen→群目录近似）
    ("精觉年兽", Faction::Qun, &[BOSS_NIANRUI, BOSS_MENGTAI, BOSS_JINGJUE, BOSS_NBIANSHEN], 12, Gender::Male),
    // 夜叉（v253 魔道+魔剑+胆术复用 v252；bazhen=标准装备八卦阵由 Armor
    // 判定机承载不入条目；shen→群目录近似）
    ("夜叉", Faction::Qun, &[BOSS_MODAO, BOSS_MOJIAN, BOSS_DANSHU], 11, Gender::Male),
    // 罗刹（v254 魔道共用+御兽+魔炎；yizhong=一将包防御技未落登记；
    // 上游 female、shen→群目录近似）
    ("罗刹", Faction::Qun, &[BOSS_MODAO, BOSS_YUSHOU, BOSS_MOYANY], 12, Gender::Female),
    // 白无常（v255 爆敛+强征+醉酒；juece 界绝策[refresh 包]混入未落登记；
    // shen→群目录近似）
    ("白无常", Faction::Qun, &[BOSS_BAOLIAN, BOSS_QIANGZHENG, BOSS_ZUIJIU], 8, Gender::Male),
    // 黑无常（v256 鬼计+太平+索命+吸行；bianshen4 死亡换将=已处置关卡
    // 面；names 范|无救、shen→群目录近似）
    ("黑无常", Faction::Qun, &[BOSS_GUIJI, BOSS_TAIPING, BOSS_SUOMING, BOSS_XIXING], 9, Gender::Male),
    // 马面（v257 炼狱+鬼计共用 v256+反馈[标准 FAN_KUI 已承载]+绝策 v258；
    // shen→群目录近似）
    ("马面", Faction::Qun, &[BOSS_GUIJI, FAN_KUI, BOSS_LIANYU, BOSS_JUECE], 6, Gender::Male),
    // 董卓（v258 酒池+强征/爆敛混入已落[v255]；shen→群目录近似）
    ("董卓", Faction::Qun, &[BOSS_JIUCHI, BOSS_QIANGZHENG, BOSS_BAOLIAN], 20, Gender::Male),
    // 牛头（v255 爆敛共用+枭首；蛮甲[extra-equip 藤甲挂点分散]+niepan
    // [boss 本地死亡复活面]混入未落登记；shen→群目录近似）
    ("牛头", Faction::Qun, &[BOSS_BAOLIAN, BOSS_XIAOSHOU], 7, Gender::Male),
    // 郭嘉（v259 鬼谋+远策+奇佐；天妒=标准技共用；上游 group shen、
    // names 郭|嘉、shen→群目录近似）
    ("郭嘉", Faction::Qun, &[TIAN_DU, BOSS_GUIMOU, BOSS_YUANCE, BOSS_QIZUO], 4, Gender::Male),
    // 张春华（v260 绝情+无心+伤逝；上游 group shen→群目录近似）
    ("张春华", Faction::Qun, &[JUE_QING, BOSS_WUXIN, BOSS_SHANGSHIX], 4, Gender::Female),
    // 刘备（v261 枭雄+章武；上游 group shen→群目录近似）
    ("刘备", Faction::Qun, &[BOSS_XIAOXIONG, BOSS_ZHANGWU], 8, Gender::Male),
    // 甄姬（v262 踏水/凌波/娇孝/芳华；上游 group shen→群目录近似）
    ("甄姬", Faction::Qun, &[BOSS_TASHUI, BOSS_LINGBO, BOSS_JIAOXIA, BOSS_FANGHUA], 4, Gender::Female),
    // 周瑜（v263 火神+红莲·周瑜+仙音；上游 group shen→群目录近似）
    ("周瑜", Faction::Qun, &[BOSS_HUOSHEN, BOSS_ZYHONGLIAN, BOSS_XIANYIN], 6, Gender::Male),
    // 张角（v263 雷击·张角+天道+极点；上游 group shen→群目录近似）
    ("张角", Faction::Qun, &[BOSS_ZJLEIJI, BOSS_TIANDAO, BOSS_JIDIAN], 8, Gender::Male),
    // 五官王（v265 枳枉+公正+血池；十殿阎罗第二殿；上游 group qun→群
    // 目录直取）
    ("五官王", Faction::Qun, &[BOSS_ZHIWANG, BOSS_GONGZHENG, BOSS_XUECHI], 4, Gender::Male),
    // 宋帝王（v266 黑绳+胜负；恩怨=外部包组技未落登记随外部包批；上游
    // group qun→群目录直取）
    ("宋帝王", Faction::Qun, &[BOSS_HEISHENG, BOSS_SHENGFU], 4, Gender::Male),
    // 卞城王（v267 雷舟+雷斧+雷诛；上游 group qun→群目录直取）
    ("卞城王", Faction::Qun, &[BOSS_LEIZHOU, BOSS_LEIFU, BOSS_LEIZHU], 4, Gender::Male),
    // 泰山王（v268 服毒+苦酒+热恼；上游 group qun→群目录直取）
    ("泰山王", Faction::Qun, &[BOSS_FUDU, BOSS_KUJIU, BOSS_RENAO], 4, Gender::Male),
    // 转轮王（v269 魔道[共用]+轮回+往生+反噬；上游 group qun→群目录直取）
    ("转轮王", Faction::Qun, &[BOSS_MODAO, BOSS_LUNHUI, BOSS_WANGSHENG, BOSS_ZLFANSHI], 6, Gender::Male),
    // 孟婆（v270 拾忧+忘魂+往事；上游 group qun→群目录直取；female）
    ("孟婆", Faction::Qun, &[BOSS_SHIYOU, BOSS_WANGHUN, BOSS_WANGSHI], 3, Gender::Female),
    // 阎罗王（v271 铁面+铡刀+诛心；上游 group qun→群目录直取）
    ("阎罗王", Faction::Qun, &[BOSS_TIEMIAN, BOSS_ZHADAO, BOSS_ZHUXIN], 4, Gender::Male),
    // 平等王（v272 锁足+阿鼻+平等；上游 group qun→群目录直取）
    ("平等王", Faction::Qun, &[BOSS_SUOZU, BOSS_ABI, BOSS_PINGDENG], 4, Gender::Male),
    // 都市王（v273 热闷+炙焚+火刑；上游 group qun→群目录直取）
    ("都市王", Faction::Qun, &[BOSS_REMEN, BOSS_ZHIFEN, BOSS_HUOXING], 4, Gender::Male),
    // 地藏王（v274 不佛+无量+谛听；大愿=判定改写机 AI 态度面登记不落；
    // 上游 group qun→群目录直取）
    ("地藏王", Faction::Qun, &[BOSS_BUFO, BOSS_WULIANG, BOSS_DITING], 8, Gender::Male),
    // 楚江王（v275 帷幕+反馈[标准 FAN_KUI 复用——原生反馈语义=任意伤害
    // +手牌/装备，与界反馈一致]+冰封；上游 group qun→群目录直取）
    ("楚江王", Faction::Qun, &[BOSS_WEIMU, FAN_KUI, BOSS_BINGFENG], 4, Gender::Male),
    // 秦广王（v276 判官+拘魂+望乡；还人=关卡流程面登记不落；上游 group
    // qun→群目录直取）
    ("秦广王", Faction::Qun, &[BOSS_PANGUAN, BOSS_JUHUN, BOSS_WANGXIANG], 3, Gender::Male),
    // 庞统·涅槃凤雏（v277 天狱+栖梧；涅槃/浴火=外部包技依赖登记不落；
    // 上游 group shen→群近似登记）
    ("庞统", Faction::Qun, &[BOSS_TIANYU, BOSS_QIWU], 4, Gender::Male),
];

/// 武将目录下标 → 性别（越界返回 None）。
pub fn general_gender(g: u16) -> Option<Gender> {
    STANDARD_GENERALS.get(g as usize).map(|(_, _, _, _, gen)| *gen)
}

/// 武将目录下标 → 体力上限（越界返回 None）。
pub fn general_hp(g: u16) -> Option<u32> {
    STANDARD_GENERALS.get(g as usize).map(|(_, _, _, h, _)| *h)
}

/// 武将目录下标 → 技能集（越界返回空）。
pub fn general_skills(g: u16) -> &'static [&'static str] {
    STANDARD_GENERALS.get(g as usize).map(|(_, _, s, _, _)| *s).unwrap_or(&[])
}

/// 武将目录下标 → 势力（越界返回 None）。
/// 国战明置门控：势力判定只认「已明置」的武将槽（ADR 0756/0757）。
pub fn faction_of_general(g: u16) -> Option<crate::catalog::Faction> {
    STANDARD_GENERALS.get(g as usize).map(|(_, f, _, _, _)| *f)
}

/// 由内容机器快照重建机器实例（`MachineSnapshot::restore` 的内容侧分派）。
/// 返回 None 表示快照不属于本目录的机器类型。
pub fn restore_content_machine(s: &MachineSnapshot) -> Option<Box<dyn SkillMachine>> {
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
        MachineSnapshot::JianXiong {
            owner,
            card,
            asked,
        } => Some(Box::new(JianXiongMachine {
            owner: *owner,
            card: *card,
            asked: *asked,
        })),
        MachineSnapshot::BiYue { owner } => {
            Some(Box::new(BiYueMachine { owner: *owner }))
        }
        MachineSnapshot::BossBuchun { owner, stage, targets } => {
            Some(Box::new(BuchunMachine {
                owner: *owner,
                stage: *stage,
                targets: targets.clone(),
            }))
        }
        MachineSnapshot::BossXingxia { owner, stage, actor } => {
            Some(Box::new(XingxiaMachine {
                owner: *owner,
                stage: *stage,
                actor: *actor,
            }))
        }
        MachineSnapshot::JiZhi { owner } => {
            Some(Box::new(JiZhiMachine { owner: *owner }))
        }
        MachineSnapshot::XiaoJi { owner } => {
            Some(Box::new(XiaoJiMachine { owner: *owner }))
        }
        MachineSnapshot::QingNang { owner, stage, healed } => {
            Some(Box::new(QingNangMachine {
                owner: *owner,
                stage: match stage {
                    QingNangStageSnap::TargetAsk => QingNangStage::TargetAsk,
                    QingNangStageSnap::TargetRead => QingNangStage::TargetRead,
                    QingNangStageSnap::CardRead => QingNangStage::CardRead,
                },
                healed: *healed,
            }))
        }
        MachineSnapshot::KuRou { owner, stage } => {
            Some(Box::new(KuRouMachine {
                owner: *owner,
                stage: match stage {
                    KuRouStageSnap::Lose => KuRouStage::Lose,
                    KuRouStageSnap::Draw => KuRouStage::Draw,
                },
            }))
        }
        MachineSnapshot::ZhiHeng { owner, stage } => {
            Some(Box::new(ZhiHengMachine {
                owner: *owner,
                stage: match stage {
                    ZhiHengStageSnap::Select => ZhiHengStage::Select,
                    ZhiHengStageSnap::Draw => ZhiHengStage::Draw,
                },
            }))
        }
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
        MachineSnapshot::LuoShen { owner } => {
            Some(Box::new(LuoShenMachine { owner: *owner }))
        }
        MachineSnapshot::YingZi { owner } => {
            Some(Box::new(YingZiMachine { owner: *owner }))
        }
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
        MachineSnapshot::SanCai { owner, stage, target } => Some(Box::new(SanCaiMachine {
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
        MachineSnapshot::LuoYi { owner } => {
            Some(Box::new(LuoYiMachine { owner: *owner, asked: false }))
        }
        MachineSnapshot::GuanXing { owner, stage, top } => {
            Some(Box::new(GuanXingMachine {
                owner: *owner,
                stage: match stage {
                    GuanXingStageSnap::Arrange => GuanXingStage::Arrange,
                    GuanXingStageSnap::Place => GuanXingStage::Place,
                },
                top: top.clone(),
            }))
        }
        MachineSnapshot::LianYing { owner } => {
            Some(Box::new(LianYingMachine { owner: *owner, asked: false }))
        }
        MachineSnapshot::JieYin { owner, stage, cards } => Some(Box::new(JieYinMachine {
            owner: *owner,
            stage: match stage {
                JieYinStageSnap::CardsAsk => JieYinStage::CardsAsk,
                JieYinStageSnap::TargetAsk => JieYinStage::TargetAsk,
                JieYinStageSnap::TargetRead => JieYinStage::TargetRead,
            },
            cards: cards.clone(),
        })),
        MachineSnapshot::FanJian { owner, stage, target, chosen_suit } => Some(Box::new(FanJianMachine {
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
        MachineSnapshot::LiJian { owner, stage, card, inner } => Some(Box::new(LiJianMachine {
            owner: *owner,
            stage: match stage {
                LiJianStageSnap::DiscardAsk => LiJianStage::DiscardAsk,
                LiJianStageSnap::TargetsAsk => LiJianStage::TargetsAsk,
                LiJianStageSnap::Launch => LiJianStage::Launch,
            },
            card: *card,
            inner: match inner {
                Some(m) => match m.restore() {
                    Ok(r) => Some(r),
                    Err(_) => return None,
                },
                None => None,
            },
        })),
        MachineSnapshot::Rende { owner, stage, cards } => Some(Box::new(RendeMachine {
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
                Some(m) => match m.restore() {
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
    let (cards, skills) = standard_catalog_parts();
    let mut catalog = Catalog::build(cards, skills);
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
    let mut base_names: std::collections::BTreeSet<&str> =
        skills.iter().map(|s| s.name).collect();
    let mut host_names: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut ext_skills: Vec<crate::catalog::SkillDef> = Vec::new();
    for pack in packs.iter().filter(|p| p.enabled) {
        ext_cards.extend(pack.cards.iter().cloned());
        for decl in &pack.host_skills {
            if !base_names.insert(decl.name) {
                return Err(ExtensionError::DuplicateSkill { name: decl.name.to_string() });
            }
            if !host_names.insert(decl.name.to_string()) {
                return Err(ExtensionError::DuplicateSkill { name: decl.name.to_string() });
            }
            ext_skills.push(host_skill_def(decl.name.to_string(), decl.behavior));
        }
        for decl in &pack.vm_skills {
            let name = crate::content::names::intern(decl.name.clone());
            if !base_names.insert(name) {
                return Err(ExtensionError::DuplicateSkill { name: decl.name.clone() });
            }
            if !host_names.insert(decl.name.clone()) {
                return Err(ExtensionError::DuplicateSkill { name: decl.name.clone() });
            }
            ext_skills.push(crate::catalog::SkillDef {
                name,
                trigger: Some(decl.trigger.clone()),
                limits: Default::default(),
                behavior: Box::new(crate::rvm_bridge::VmSkillBehavior {
                    program: std::sync::Arc::new(crate::rvm_bridge::VmSkillProgram {
                        module: decl.module.clone(),
                        skill_name: decl.name.clone(),
                    }),
                    filter: decl.filter,
                }),
            });
        }
    }
    let cards = merge_extension_cards(cards, &ext_cards)?;
    let mut skills = skills;
    skills.extend(ext_skills);
    let mut catalog = Catalog::build(cards, skills);
    catalog.extension_generals = packs
        .iter()
        .filter(|p| p.enabled)
        .flat_map(|p| p.generals.iter().cloned())
        .collect();
    let lock = content_lock_id(packs);
    catalog.ruleset_version = if lock == 0 {
        STANDARD_RULESET_VERSION
    } else {
        STANDARD_RULESET_VERSION ^ (lock | (1 << 63))
    };
    Ok(catalog)
}

fn standard_catalog_parts() -> (Vec<CardDef>, Vec<SkillDef>) {
    let mut cards = Vec::new();
    // Sha x30
    for (st, rk, cnt) in [(Suit::Spade, 7u8, 1), (Suit::Spade, 8u8, 2), (Suit::Spade, 9u8, 2), (Suit::Spade, 10u8, 2), (Suit::Club, 2u8, 1), (Suit::Club, 3u8, 1), (Suit::Club, 4u8, 1), (Suit::Club, 5u8, 1), (Suit::Club, 6u8, 1), (Suit::Club, 7u8, 1), (Suit::Club, 8u8, 2), (Suit::Club, 9u8, 2), (Suit::Club, 10u8, 2), (Suit::Club, 11u8, 2), (Suit::Heart, 10u8, 2), (Suit::Heart, 11u8, 1), (Suit::Diamond, 6u8, 1), (Suit::Diamond, 7u8, 1), (Suit::Diamond, 8u8, 1), (Suit::Diamond, 9u8, 1), (Suit::Diamond, 10u8, 1), (Suit::Diamond, 13u8, 1)] {
        for _ in 0..cnt {
            cards.push(CardDef::basic("sha", CardKind::Sha).with_suit_rank(st, rk));
        }
    }
    // Shan x15
    for (st, rk, cnt) in [(Suit::Heart, 2u8, 2), (Suit::Heart, 13u8, 1), (Suit::Diamond, 2u8, 2), (Suit::Diamond, 3u8, 1), (Suit::Diamond, 4u8, 1), (Suit::Diamond, 5u8, 1), (Suit::Diamond, 6u8, 1), (Suit::Diamond, 7u8, 1), (Suit::Diamond, 8u8, 1), (Suit::Diamond, 9u8, 1), (Suit::Diamond, 10u8, 1), (Suit::Diamond, 11u8, 2)] {
        for _ in 0..cnt {
            cards.push(CardDef::basic("shan", CardKind::Shan).with_suit_rank(st, rk));
        }
    }
    // Tao x8
    for (st, rk, cnt) in [(Suit::Heart, 3u8, 1), (Suit::Heart, 4u8, 1), (Suit::Heart, 6u8, 1), (Suit::Heart, 7u8, 1), (Suit::Heart, 8u8, 1), (Suit::Heart, 9u8, 1), (Suit::Heart, 12u8, 1), (Suit::Diamond, 12u8, 1)] {
        for _ in 0..cnt {
            cards.push(CardDef::basic("tao", CardKind::Tao).with_suit_rank(st, rk));
        }
    }
    // SavageAttack x3
    for (st, rk, cnt) in [(Suit::Spade, 7u8, 1), (Suit::Spade, 13u8, 1), (Suit::Club, 7u8, 1)] {
        for _ in 0..cnt {
            cards.push(CardDef::basic("savage", CardKind::SavageAttack).with_suit_rank(st, rk));
        }
    }
    // Duel x3
    for (st, rk, cnt) in [(Suit::Spade, 1u8, 1), (Suit::Club, 1u8, 1), (Suit::Diamond, 1u8, 1)] {
        for _ in 0..cnt {
            cards.push(CardDef::basic("duel", CardKind::Duel).with_suit_rank(st, rk));
        }
    }
    // Snatch x5
    for (st, rk, cnt) in [(Suit::Spade, 3u8, 1), (Suit::Spade, 4u8, 1), (Suit::Spade, 11u8, 1), (Suit::Diamond, 3u8, 1), (Suit::Diamond, 4u8, 1)] {
        for _ in 0..cnt {
            cards.push(CardDef::basic("snatch", CardKind::Snatch).with_suit_rank(st, rk));
        }
    }
    // Dismantle x6
    for (st, rk, cnt) in [(Suit::Spade, 3u8, 1), (Suit::Spade, 4u8, 1), (Suit::Spade, 12u8, 1), (Suit::Club, 3u8, 1), (Suit::Club, 4u8, 1), (Suit::Heart, 12u8, 1)] {
        for _ in 0..cnt {
            cards.push(CardDef::basic("dismantle", CardKind::Dismantle).with_suit_rank(st, rk));
        }
    }
    // Nullify x4
    for (st, rk, cnt) in [(Suit::Spade, 11u8, 1), (Suit::Club, 12u8, 1), (Suit::Club, 13u8, 1), (Suit::Diamond, 12u8, 1)] {
        for _ in 0..cnt {
            cards.push(CardDef::basic("nullify", CardKind::Nullify).with_suit_rank(st, rk));
        }
    }
    // Le x3
    for (st, rk, cnt) in [(Suit::Spade, 6u8, 1), (Suit::Club, 6u8, 1), (Suit::Heart, 6u8, 1)] {
        for _ in 0..cnt {
            cards.push(CardDef::basic("le", CardKind::Le).with_suit_rank(st, rk));
        }
    }
    // ShanDian x2（批③-c：上游 ♠1/♥12 逐张落位；♥12 命名 shandian_safe）
    for (st, rk, name) in [(Suit::Spade, 1u8, "shandian"), (Suit::Heart, 12u8, "shandian_safe")] {
        cards.push(CardDef::basic(name, CardKind::ShanDian).with_suit_rank(st, rk));
    }
    // —— 原生扩充（G1 前置，上游无对应；花色点数为原生取值）——
    for _ in 0..4 {
        cards.push(CardDef::basic("horse_plus", CardKind::HorsePlus));
    }
    for _ in 0..2 {
        cards.push(CardDef::basic("horse_minus", CardKind::HorseMinus));
    }
    cards.push(CardDef::basic("jiu", CardKind::Jiu).with_suit_rank(Suit::Spade, 3));
    cards.push(CardDef::basic("jiu", CardKind::Jiu).with_suit_rank(Suit::Club, 9));
    cards.push(CardDef::basic("leisha", CardKind::LeiSha).with_suit_rank(Suit::Spade, 4));
    cards.push(CardDef::basic("leisha", CardKind::LeiSha).with_suit_rank(Suit::Club, 5));
    let mut weapon2 = CardDef::basic("weapon2", CardKind::Weapon);
    weapon2.range = 2;
    cards.push(weapon2);
    let mut weapon2b = CardDef::basic("weapon2b", CardKind::Weapon);
    weapon2b.range = 2;
    cards.push(weapon2b);
    let mut weapon3 = CardDef::basic("weapon3", CardKind::Weapon);
    weapon3.range = 3;
    cards.push(weapon3);
    for _ in 0..2 {
        cards.push(CardDef::basic("armor", CardKind::Armor));
    }
    for _ in 0..4 {
        cards.push(CardDef::basic("fire_sha", CardKind::FireSha));
    }
    for _ in 0..4 {
        cards.push(CardDef::basic("chain", CardKind::Chain));
    }
    cards.push(CardDef::basic("archery", CardKind::ArcheryAttack).with_suit_rank(Suit::Heart, 1));
    cards.push(
        CardDef::basic("bing", CardKind::Bing).with_suit_rank(Suit::Club, 3),
    );
    // P2/N08 缺牌种批①（2026-10-07）：无中生有 ♥7/8/9/11、桃园结义 ♥1
    //（上游 standard list 花色点数原样落位）。
    for (st, rk) in [
        (Suit::Heart, 7),
        (Suit::Heart, 8),
        (Suit::Heart, 9),
        (Suit::Heart, 11),
    ] {
        cards.push(CardDef::basic("wuzhong", CardKind::Wuzhong).with_suit_rank(st, rk));
    }
    cards.push(CardDef::basic("taoyuan", CardKind::Taoyuan).with_suit_rank(Suit::Heart, 1));
    // 批①-2（规则 v63）：五谷丰登 ♥3/♥4、借刀杀人 ♣12/♣13（上游落位）。
    for rk in [3u8, 4] {
        cards.push(CardDef::basic("wugu", CardKind::Wugu).with_suit_rank(Suit::Heart, rk));
    }
    for rk in [12u8, 13] {
        cards.push(CardDef::basic("jiedao", CardKind::Jiedao).with_suit_rank(Suit::Club, rk));
    }
    // 批③-a（规则 v64）：仁王盾 ♣2（防具；锁定黑色杀无效）。
    cards.push(CardDef::basic("renwang", CardKind::Renwang).with_suit_rank(Suit::Club, 2));
    // 批③-b 第一把（规则 v65）：青釭剑 ♠6（武器射程 2；锁定无视目标防具）
    //——命名武器牌种（特效按牌种接线，非泛型 Weapon）。
    let mut qinggang = CardDef::basic("qinggang", CardKind::Qinggang)
        .with_suit_rank(Suit::Spade, 6);
    qinggang.range = 2;
    cards.push(qinggang);
    // 批③-b 第二把（规则 v66）：麒麟弓 ♥5（武器射程 5；杀伤害时可弃目标
    // 一匹坐骑）。
    let mut qilin = CardDef::basic("qilin", CardKind::Qilin).with_suit_rank(Suit::Heart, 5);
    qilin.range = 5;
    cards.push(qilin);
    // 批③-b 第三把（规则 v67）：寒冰剑 ♠2（武器射程 2；杀伤害可防止改为
    // 弃目标两张牌）。
    let mut hanbing = CardDef::basic("hanbing", CardKind::Hanbing)
        .with_suit_rank(Suit::Spade, 2);
    hanbing.range = 2;
    cards.push(hanbing);
    // 批③-b 第四把（规则 v68）：诸葛连弩 ♣1/♦1（武器射程 1；出杀不限次）。
    for (st, rk) in [(Suit::Club, 1u8), (Suit::Diamond, 1)] {
        let mut zhuge = CardDef::basic("zhuge", CardKind::Zhuge).with_suit_rank(st, rk);
        zhuge.range = 1;
        cards.push(zhuge);
    }
    // 批③-b 第五把（规则 v69）：贯石斧 ♦5（武器射程 3；杀被闪后可弃
    // 两张牌强制伤害）。
    let mut guanshi = CardDef::basic("guanshi", CardKind::Guanshi)
        .with_suit_rank(Suit::Diamond, 5);
    guanshi.range = 3;
    cards.push(guanshi);
    // 批③-b 第六把（规则 v70）：青龙偃月刀 ♠5（武器射程 3；杀被闪后可
    // 立即对相同目标再使用一张杀）。
    let mut qinglong = CardDef::basic("qinglong", CardKind::Qinglong)
        .with_suit_rank(Suit::Spade, 5);
    qinglong.range = 3;
    cards.push(qinglong);
    // 批③-b 第七把（规则 v71）：丈八蛇矛 ♠12（武器射程 3；两张手牌当杀
    // 的配套主动技随装备授予）。
    let mut zhangba = CardDef::basic("zhangba", CardKind::Zhangba)
        .with_suit_rank(Suit::Spade, 12);
    zhangba.range = 3;
    zhangba.equip_skills.push(ZHANGBA);
    cards.push(zhangba);
    // 批③-b 第八把（规则 v72）：雌雄双股剑 ♠2（武器射程 2；对异性目标
    // 出杀：目标弃一张手牌或令使用者摸一张牌）。
    let mut cixiong = CardDef::basic("cixiong", CardKind::Cixiong)
        .with_suit_rank(Suit::Spade, 2);
    cixiong.range = 2;
    cards.push(cixiong);
    // 批③-b 第九把（规则 v73）：方天画戟 ♦5（武器射程 4；最后手牌杀
    // 可指定至多三个目标）。
    let mut fangtian = CardDef::basic("fangtian", CardKind::Fangtian)
        .with_suit_rank(Suit::Diamond, 5);
    fangtian.range = 4;
    cards.push(fangtian);
    // 官方技能（鬼道/天妒）追加在样例技能之后：技能表追加保持既有
    // SkillDefId 稳定（def id 入快照/哈希，插队会漂移样例技能编号）。
    let skills = vec![
        SkillDef {
            name: SAN_CAI,
            trigger: Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(SanCaiBehavior),
        },
        SkillDef {
            name: YI_SHANG,
            trigger: Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(YiShangBehavior),
        },
        SkillDef {
            name: SUI_DUN,
            trigger: Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits {
                once_per_turn: true,
                ..SkillLimits::default()
            },
            behavior: Box::new(SuiDunBehavior),
        },
        SkillDef {
            name: SU_QING,
            trigger: Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::First,
                priority: 100,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(SuQingBehavior),
        },
        SkillDef {
            name: SHOU_YI,
            trigger: Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(ShouYiBehavior),
        },
        SkillDef {
            name: FAN_KUI,
            trigger: Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(FanKuiBehavior {
                _p: (),
            }),
        },
        SkillDef {
            name: JIAN_XIONG,
            trigger: Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(JianXiongBehavior),
        },
        SkillDef {
            name: JI_JI,
            trigger: Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(JiJiBehavior),
        },
        SkillDef {
            name: GANG_LIE,
            trigger: Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(GangLieBehavior),
        },
        SkillDef {
            name: TU_XI,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits { once_per_turn: true, ..SkillLimits::default() },
            behavior: Box::new(TuXiBehavior),
        },
        SkillDef {
            name: LUO_YI,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits { once_per_turn: true, ..SkillLimits::default() },
            behavior: Box::new(LuoYiBehavior),
        },
        SkillDef {
            name: GUAN_XING,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits { once_per_turn: true, ..SkillLimits::default() },
            behavior: Box::new(GuanXingBehavior),
        },
        // 空城/谦逊为锁定门控：无触发机器，出牌校验侧经 skill_by_name 查询。
        SkillDef {
            name: LIAN_YING,
            trigger: Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: vec![
                    EventKind::CardResponded,
                    EventKind::CardsMoved,
                    EventKind::CardsDiscarded,
                ],
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(LianYingBehavior),
        },
        SkillDef {
            name: KONG_CHENG,
            // 权限型能力：出牌校验经 skill_by_name 查询持有者（无触发时机）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: QIAN_XUN,
            // 权限型能力：乐不思蜀目标校验经 skill_by_name 查询持有者
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: REN_DE,
            // 主动技：出牌阶段可多次发动（上游 enable: phaseUse 无次数
            // 限制）；累计给牌数入回合状态（跨过 2 张回复 1 点）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(RendeBehavior),
        },
        SkillDef {
            name: JI_JIANG,
            // 主公技主动化：激将上游入口是 chooseToUse/chooseToRespond
            // （需要使用/打出杀时发动），不绑定出牌阶段——目录声明例外
            // （once_per_turn 为原生逐回合限一次的登记约束）。
            trigger: None,
            limits: SkillLimits { once_per_turn: true, active_outside_play: true },
            behavior: Box::new(JijiangBehavior),
        },
        SkillDef {
            name: MA_SHU,
            // 权限型能力：computed_distance 距离修正经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: QING_GUO,
            // 权限型能力：杀响应选项并入黑色装备经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: QI_CAI,
            // 权限型能力：锦囊距离限制豁免经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: JIU_YUAN,
            // 权限型能力：濒死桃回复量修正经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: JI_JIU,
            // 权限型能力：濒死结算桃判定扩面经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: HU_JIA,
            // 权限型能力：出闪响应选项扩面（护驾借闪）经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: LIU_LI,
            // 权限型能力：杀转移询问（ShaResolve 内嵌流离段）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: JIE_YIN,
            // 主动技：出牌阶段限一次（activate 形态）
            trigger: None,
            limits: SkillLimits { once_per_turn: true, ..SkillLimits::default() },
            behavior: Box::new(JieYinBehavior),
        },
        SkillDef {
            name: FAN_JIAN,
            // 主动技：出牌阶段限一次（activate 形态）
            trigger: None,
            limits: SkillLimits { once_per_turn: true, ..SkillLimits::default() },
            behavior: Box::new(FanJianBehavior),
        },
        SkillDef {
            name: LI_JIAN,
            // 主动技：出牌阶段限一次（activate 形态，内嵌决斗直结算）
            trigger: None,
            limits: SkillLimits { once_per_turn: true, ..SkillLimits::default() },
            behavior: Box::new(LiJianBehavior),
        },
        SkillDef {
            name: TIE_JI,
            // 锁定式判定：ShaResolve 结算内经 skill_by_name 查询来源持有者
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: PAO_XIAO,
            // 权限型能力：出牌阶段杀限检查经 skill_by_name 查询持有者
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: QI_XI,
            // 权限型能力：出牌校验经 skill_by_name 查询持有者（无触发时机）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: WU_SHUANG,
            // 修改型能力：杀结算/决斗机器经 skill_by_name 查询来源持有者
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: JI_ZHI,
            trigger: Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(JiZhiBehavior),
        },
        SkillDef {
            name: KE_JI,
            // 条件型手牌上限修正：判定阶段（弃牌）流程经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: GUO_SE,
            // 权限型能力：出牌校验经 skill_by_name 查询持有者（无触发时机）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: WU_SHENG,
            // 权限型能力：出牌校验经 skill_by_name 查询持有者（无触发时机）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: LONG_DAN,
            // 权限型能力：出牌/出闪结算经 skill_by_name 查询持有者
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BI_YUE,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(BiYueBehavior),
        },
        SkillDef {
            name: LUO_SHEN,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(LuoShenBehavior),
        },
        SkillDef {
            name: KU_ROU,
            // 主动技：经 GameAction::ActivateSkill 发动（每回合限一次）
            trigger: None,
            limits: SkillLimits {
                once_per_turn: true,
                ..SkillLimits::default()
            },
            behavior: Box::new(KuRouBehavior),
        },
        SkillDef {
            name: ZHI_HENG,
            // 主动技：经 GameAction::ActivateSkill 发动（每回合限一次）
            trigger: None,
            limits: SkillLimits {
                once_per_turn: true,
                ..SkillLimits::default()
            },
            behavior: Box::new(ZhiHengBehavior),
        },
        SkillDef {
            name: YING_ZI,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(YingZiBehavior),
        },
        SkillDef {
            name: XIAO_JI,
            trigger: Some(TriggerMeta {
                event: EventKind::Equipped,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(XiaoJiBehavior),
        },
        SkillDef {
            name: QING_NANG,
            // 主动技：出牌阶段限一次
            trigger: None,
            limits: SkillLimits {
                once_per_turn: true,
                ..SkillLimits::default()
            },
            behavior: Box::new(QingNangBehavior),
        },
        SkillDef {
            name: GUI_CAI,
            // 权限型能力：判定流程经 skill_by_name 查询持有者（无触发时机）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: GUI_DAO,
            // 权限型能力：判定流程经 skill_by_name 查询持有者（无触发时机）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: TIAN_DU,
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: JU_SHOU,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(JushouBehavior),
        },
        SkillDef {
            name: XIAO_GUO,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(XiaoGuoBehavior),
        },
        SkillDef {
            name: KUANG_FU,
            trigger: Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits { once_per_turn: true, ..SkillLimits::default() },
            behavior: Box::new(KuangFuBehavior),
        },
        SkillDef {
            name: YAO_WU,
            trigger: Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(YaoWuBehavior),
        },
        SkillDef {
            name: SHU_SHEN,
            trigger: Some(TriggerMeta {
                event: EventKind::Recover,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(ShuShenBehavior),
        },
        SkillDef {
            name: SHEN_ZHI,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(ShenZhiBehavior),
        },
        SkillDef {
            name: XUN_XUN,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(XunXunBehavior),
        },
        SkillDef {
            name: WANG_XI,
            trigger: Some(TriggerMeta {
                event: EventKind::Damage,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(WangXiBehavior),
        },
        SkillDef {
            name: YI_CONG,
            // 权限型能力：computed_distance 动态距离修正经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: JI_JIE,
            trigger: None,
            limits: SkillLimits { once_per_turn: true, ..SkillLimits::default() },
            behavior: Box::new(JiJieBehavior),
        },
        SkillDef {
            name: JI_YUAN,
            trigger: Some(TriggerMeta {
                event: EventKind::Dying,
                timing: Timing::Normal,
                priority: 0,
                optional: true,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(JiYuanBehavior),
        },
        SkillDef {
            name: WANG_ZUN,
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(WangZunBehavior),
        },
        SkillDef {
            name: WANG_ZUN_MARK,
            // 权限型能力：弃牌阶段手牌上限 -1（妄尊授予的回合级印记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: TONG_JI,
            trigger: Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(TongJiBehavior),
        },
        SkillDef {
            name: ZHANGBA,
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(ZhangbaBehavior),
        },
        SkillDef {
            name: BOSS_XIONGSHOU,
            // 权限型能力三面：杀伤条件臂（kernel emit_sha_damage）、
            // computed_distance 距离 -1（state）、翻面取消（commit 入口
            // 直读跳过——死亡取消型同款），均经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_WUZANG,
            // 权限型能力两面：摸牌阶段额外量与手牌上限基数归零，均在
            // kernel 阶段臂经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_XIANGDE,
            // 伤害前置触发：来源有武器对持有者伤害 +1（机器一步直改
            // 载荷，YiShang 同款窗口）
            trigger: Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(XiangDeBehavior),
        },
        SkillDef {
            name: BOSS_YINZEI,
            // 权限型两面：激活门控与伤害后弃牌在 commit_damage/
            // commit_lose_hp 尾经 skill_by_name 查询（激活=SkillInstance
            // .enabled 直写，初始装配即未激活）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZHUE,
            // 权限型单面：global damageEnd 双摸在 commit_damage 尾经
            // skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_FUTAI,
            // 准备阶段触发（判定阶段开始锚点，神智同口径）：全体受伤
            // 者按座位序各回 1（机器一步，无挂起；锁定=forced 无确认）
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(FutaiBehavior),
        },
        SkillDef {
            name: BOSS_YANDU,
            // 权限型两面：激活门控（commit_damage 尾，隐贼先例同构）与
            // 他人结束阶段获得牌（kernel Phase::End 臂）均经 skill_by_name
            // 查询；装配即未激活
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_TANYU,
            // 权限型两面：跳过弃牌阶段（kernel Phase::Discard 臂直查）
            // 与结束阶段全场最多手牌失 1 体（Phase::End 臂）均经
            // skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_CANGMU,
            // 权限型单面：摸牌基数=存活角色数（kernel Draw 臂，无脏同位）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_JICAI,
            // 权限型两面：激活门控（commit_damage 尾，延妒并列）与回复
            // 后双摸（commit_recover 新臂）均经 skill_by_name 查询；装配
            // 即未激活
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_NITAI,
            // 权限型两面：回合内受伤防止与回合外火伤 +1 均在
            // commit_damage 内联经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_LUANCHANG,
            // 两面：准备阶段（判定阶段开始锚点）视为用【南蛮入侵】（触
            // 发机一步 use_virtual_card）；结束阶段视为用【万箭齐发】
            // （kernel Phase::End 臂查询式）；激活门控同隐贼/延妒/积财
            // 先例（装配即未启用）
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(LuanChangBehavior),
        },
        SkillDef {
            name: BOSS_MINWAN,
            // 权限型三面：伤害后锁定集（commit_damage 尾）、每用牌摸 1
            //（commit_card_used 尾）、PlayCard 目标校验（engine 校验段）
            // 均经 skill_by_name 查询；锁定集=TurnState.minwan_targets
            //（回合级，更替自然清零）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_CUIDUI,
            // 权限型单面：伤害后授毒（commit_damage 尾经 skill_by_name
            // 查询）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZHONGDU,
            // 中毒（运行时授技）：回合开始判定（commit_phase Judge begin
            // 段内联 draw_judge_card，铁骑式确定性判定）——非红桃伤 1、
            // 非黑桃失去此技（实例移除）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_YAOSHOU,
            // 权限型单面：computed_distance 距离 -2（state 距离链，凶兽
            // 同款）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_JIUSHOU,
            // 权限型三面：手牌上限恒 9（state hand_limit）、跳过摸牌与
            // 出牌/结束补至 9（kernel 阶段臂）均经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_DUQU,
            // 权限型单面：受击者给来源加蛇毒标记（commit_damage 尾经
            // skill_by_name 查询；PlayerState.shedu 承载）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ECHOU,
            // 权限型两面：激活门控（commit_damage 尾延妒并列）与他人桃
            // /酒获标记（commit_card_used）均经 skill_by_name 查询；装配
            // 即未激活
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_SHEDU,
            // 蛇毒标记结算（运行时触发技）：Judge begin 持有标记>0 →
            // 「弃 X 张牌或失去 X 体力」（挂起选择；取消/空选=失 X）→
            // 标记-1；X=当前标记数
            trigger: Some(TriggerMeta {
                event: EventKind::PhaseBegin,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(ShedUBehavior),
        },
        SkillDef {
            name: BOSS_BINGXIAN,
            // 权限型单面：他人回合未出杀 → 结束阶段虚拟杀（kernel
            // Phase::End 臂，乱常万箭同款 use_virtual_card）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_XUSHI,
            // 权限型两面：出牌阶段结束自翻面（commit_phase Play begin:
            // false 臂）与翻回正面全场伤 1（commit_turn_over_before 尾）
            // 均经 skill_by_name 查询；激活门控同隐贼先例（装配即未激活）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_JUYUAN,
            // 权限型两面：回合结束体力记录（kernel Phase::End 臂）与杀
            // 目标 mod（engine max_targets）均经 skill_by_name 查询；
            // juyuan_storage=PlayerState 跨回合持久字段
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZHAOHUO,
            // 两面：造成伤害视为火（DamageBefore 触发机一步直改 nature
            // ——相德同款窗口）；免疫火伤（commit_damage 入口直读——拟
            // 态回合内免伤同位）
            trigger: Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(ZhaoHuoBehavior),
        },
        SkillDef {
            name: BOSS_HONGLIANX,
            // 权限型两面：红色不计上限+弃牌阶段红牌不可弃（hand_limit
            // 与 Discard 臂查询）；回合开始获红牌/对至多 3 名各 1 火伤
            // （Judge begin 段内联，确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_YANYU,
            // 权限型两面：激活门控（commit_damage 尾第八并列臂）与判定
            // 红色火伤（kernel Judge begin 段内联，回合计数至多 3）均经
            // skill_by_name 查询；装配即未激活
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_FENGDONG,
            // 权限型两面：回合开始他人获封印（Judge begin 段内联，神智
            // 锚）与回合结束全清（Phase::End 臂）经 skill_by_name 查询；
            // PlayCard 手牌校验（engine 校验段）禁用封印者手牌
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_XUNYOU,
            // 权限型单面：他人回合开始获得座位序首名有手牌者一张手牌
            //（Judge begin 段内联，确定性近似；装备牌自动使用=直接装上）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_SIPU,
            // 权限型两面：激活门控（commit_damage 尾第九并列臂；装配即
            // 未激活）与响应/救禁（respond_shan、dying_save 选项排除）
            // 均经 skill_by_name 查询；使用面由回合模型天然承载；无懈等
            // 其余响应窗未纳入禁令（登记差异）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_SHENYI,
            // 查询型两面：翻面取消（commit_turn_over_before 臂——凶兽
            // 同位，仅正→背取消）与判定阶段延时锦囊无效化（apply_
            // judgement 逐臂条件）均经 skill_by_name 查询；洪荒之力禁
            // 用交互未落（登记差异）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_RELEIJI,
            // 查询型单面：杀/万箭闪消费位内联臂（releiji_on_shan）经
            // skill_by_name 查询；青木2（boss_qingmu2 死亡隐藏+换位重
            // 生）为关卡流程面，属 Boss 模式裸骨架缺口，登记不落内核
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_KUANGXIAO,
            // 查询型两面：杀无距离（engine sha_range_free 链追加）与目
            // 标数无限（engine kuangxiao_inf 臂，天弦同构）均经 skill_
            // by_name 查询；友方禁指面登记差异（关系网边界）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_FENTIAN,
            // 火改写面=ZhaoHuoBehavior 同构复用（数据less nature 改写，
            // 快照变体共用 BossZhaohuo——机器不携带技能身份，恢复路径按
            // 技能表重挂）；其余为查询面（engine 红牌免次数/免距离、
            // kernel 红杀跳八卦）；wuxie 禁面登记差异未挂钩
            trigger: Some(TriggerMeta {
                event: EventKind::DamageBefore,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(ZhaoHuoBehavior),
        },
        SkillDef {
            name: BOSS_LINGQU,
            // 查询型两面：受击摸 1+计数（commit_damage damageEnd 段）与
            // 大于 1 点伤害归零（commit_damage 入口臂）均经 skill_by_name
            // 查询；手牌上限 +X（hand_limit 查询+PlayerState.lingqu_count
            // 新字段）；玄麟=关卡流程面登记不落
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_SHENEN,
            // 全局光环查询型三面：engine sha/顺手距离链 boss 方豁免、
            // hand_limit 挑战者 +1（state）、Draw 臂挑战者 +1（kernel）
            // 均经 skill_by_name 查询+is_boss_side 判定；光环激活=场上
            // 存在持有者
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_XINGQIU,
            // 查询型单面：Play begin 臂（交替开关+横置敌方，direct 直写
            // linked+日志，索链甲同款）；授 boss_jiding 面登记差异未入库
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_BUCHUN,
            // 主动技两线（ActivateSkill 直调 activate——复活线/恢复线按
            // 场面分派；间隔门控读 buchun_used_round）；快照变体
            // BossBuchun（archive tag 66/机哈希 67）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(BuchunBehavior),
        },
        SkillDef {
            name: BOSS_HUIHUO,
            // 查询型两面：杀上限 +1（engine 限次臂 once_used 额度，元宝
            // 肉同款）与死亡反噬（kernel 死亡结算移除前捕获+PlayerDied
            // 后敌方 3 点火伤）均经 skill_by_name 查询
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_FURAN,
            // 查询型单面：濒死救援选项扩展（is_tao_like 臂——敌方红色
            // 牌可当桃救焰灵）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_XINGXIA,
            // 主动技（ActivateSkill 直调 activate——损焰灵线/追击线按
            // 挂起阶段推进；间隔门控读 xingxia_used_round）；快照变体
            // BossXingxia（archive tag 67/机哈希 68）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(XingxiaBehavior),
        },
        SkillDef {
            name: BOSS_JUEHONG,
            // 查询型单面：准备阶段臂（Judge begin 段——封冻同位；装备全
            // 弃撤技能+入弃堆，随机弃→首张确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_QINGZHU,
            // 查询型两面：跳弃牌（Discard 臂——贪欲同位）+杀禁用（engine
            // 校验臂——获急叮解除）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_JIAZU,
            // 查询型单面：准备阶段臂（相邻敌方坐骑全弃）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_JIDING,
            // 查询型单面：damageEnd 臂（友方受伤→对敌方来源虚拟雷杀，
            // LeiSha kind 载体；蓐收加成尾面登记差异）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZAOYI,
            // 查询型三面：死亡触发（DeathMachine 移除后捕获+授技）、
            // targetEnabled 双门控（engine 校验）、造异异变授予（运行时
            // skills.insert，中毒/蛇毒先例）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZAOYI_HP,
            // 查询型单面（运行时授予）：回合开始敌方最低体力全体失去全
            // 部体力（Judge begin 臂）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZIRUN,
            // 查询型单面：准备阶段臂（Judge begin 段——全体摸牌：有装备
            // 2/无装备 1）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_NIANRUI,
            // 查询型单面：Draw 臂固定摸 2 后补摸 2（藏目同位）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_MENGTAI,
            // 查询型多面：准备置位/摸牌清位/出牌清位/结束补摸 3/弃牌闸门
            // （TurnState.mengtai_* 双旗标承载）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_NBIANSHEN,
            // 查询型单面：Judge begin 轮换臂（assembly 重挂+hp 保留；
            // versus 替补换将先例同构；确定性互斥轮转登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_RENXING,
            // 查询型双臂：damageEnd（commit_damage 段）+recoverEnd
            // （commit_recover_skills 头）——非当前回合摸 1
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_RUIZHI,
            // 查询型单面：准备阶段臂（global 修剪：其余角色保留 1 手+1
            // 装，弃其余——保留首张确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_SHOUYI,
            // 查询型单面：sha_range_free 链+顺手距离链追加（全牌无距离，
            // 狂笑/神恩同位）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_DANSHU,
            // 查询型单面：CardsDiscarded 臂（失去手牌后非当前回合且已受
            // 伤判定红回 1）——夜叉批条目复用同名 def
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_JINGJUE,
            // 查询型单面：继承胆术（同内容共用 commit 臂按名双查）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_MODAO,
            // 查询型单面：Judge begin 臂自摸 2（藏目/年瑞同位）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_MOJIAN,
            // 查询型单面：Play begin 臂虚拟万箭（乱常 End 臂同款
            // use_virtual_card；异侧存活全目标）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_YUSHOU,
            // 查询型单面：Play begin 臂虚拟南蛮（魔剑同款 use_virtual_card）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_MOYANY,
            // 查询型单面：CardsDiscarded 臂（非当前回合判定红→2 火伤打
            // 座位序首个其他存活——确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_BAOLIAN,
            // 查询型单面：End 臂摸 2
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_QIANGZHENG,
            // 查询型单面：End 臂——每名其他有手牌角色随机一张→首张确定
            // 性近似登记，获得至手牌
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZUIJIU,
            // 查询型单面：commit_damage 臂——source sha 伤害 +1
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_XIAOSHOU,
            // 查询型单面：End 臂——座位序首个 hp≥self 其他存活受 3 伤害
            // （目标自选可放弃→恒发动确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_GUIJI,
            // 查询型单面：Judge begin 臂弃自己判定区首张（随机→首张近似）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_TAIPING,
            // 查询型单面：Draw 臂 +2（年瑞同位）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_SUOMING,
            // 查询型单面：End 臂——全部未链接异侧存活横置（链接机直写+
            // Linked 日志，[1,num] 自选→全部确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_XIXING,
            // 查询型单面：Judge begin 臂——座位序首个已链接异侧受 1 雷伤
            // +持有者回 1（目标自选→首个已链接确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_LIANYU,
            // 查询型单面：End 臂——全部其他存活各 1 火伤（无询问直伤）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_JUECE,
            // 查询型单面：CardsDiscarded 臂——自己回合内其他角色失去最后
            // 一手牌（弃后空手）→ 1 伤害（chooseBool AI→恒发动近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_JIUCHI,
            // 查询型单面：PlayCard kind 改写臂（黑桃手牌→酒，viewAs 转化
            // 武圣/奇袭同族）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_GUIMOU,
            // 查询型单面：End begin 臂——座位序首个其他未混乱存活角色
            // boss_mad 置位（其回合收束解除；随机→首个确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_YUANCE,
            // 查询型单面：Damage commit 尾臂——来源≠受伤者且双方存活 →
            // 判定（选项一恒定：黑→受伤者失 1 / 红→来源失 1；chooseControl
            // AI 态度面→恒选项一确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_QIZUO,
            // CardUsed AfterTriggers 触发：锦囊复用一步机（QizuoBehavior
            // 门=无懈/铁索排除+延时同名去重+原目标存活；载体槽守卫递归）
            trigger: Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(QizuoBehavior),
        },
        SkillDef {
            name: JUE_QING,
            // 查询型单面：commit_damage 入口直读臂（急冻前——上游
            // damageBefore 层）——来源伤害改为目标失去等量体力
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_WUXIN,
            // 查询型双面：commit_damage 入口直读臂（受伤→失 1）+
            // PlayCard 校验臂（乐/兵拒——延时禁指定他人）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_SHANGSHIX,
            // 查询型双面：四 commit 臂补牌（CardsDiscarded/Damage/LoseHp/
            // Recover——手牌<4 补至 4）+ End begin 臂（hp>1 全体各失 1）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_XIAOXIONG,
            // CardUsed AfterTriggers 触发：副本入手一步机（XiaoxiongBehavior
            // 门=他人使用+基本/锦囊白名单；End begin 臂=子技未用牌伤害）
            trigger: Some(TriggerMeta {
                event: EventKind::CardUsed,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(XiaoxiongBehavior),
        },
        SkillDef {
            name: BOSS_ZHANGWU,
            // 查询型单面：Damage commit 尾臂（damageEnd 族）——持有者弃 1
            // 张+来源弃 1 张或受 1 伤害（双方恒弃首张确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_LINGBO,
            // CardResponded+CardUsed 触发：打出/使用闪后摸 2 一步机
            trigger: Some(TriggerMeta {
                event: EventKind::CardResponded,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: vec![EventKind::CardUsed],
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(LingboBehavior),
        },
        SkillDef {
            name: BOSS_JIAOXIA,
            // 查询型单面：CardUsed commit 臂——成为红色牌目标后摸 1
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_FANGHUA,
            // 查询型单面：End begin 臂——存在翻面角色则全部各失 1
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_TASHUI,
            // CardResponded+CardUsed 触发：打出/使用黑牌后翻面一步机
            trigger: Some(TriggerMeta {
                event: EventKind::CardResponded,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: vec![EventKind::CardUsed],
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(TashuiBehavior),
        },
        SkillDef {
            name: BOSS_HUOSHEN,
            // 查询型单面：commit_damage 火伤分支直读臂（招祸同位扩展——
            // 免疫火伤+回复 1）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZYHONGLIAN,
            // 查询型单面：End begin 臂——摸 2+全部其他存活各 1 火伤
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_XIANYIN,
            // 查询型单面：CardsDiscarded 臂（非当前回合）——判定红→座位
            // 序首个其他存活失 1（chooseTarget AI→首个确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_TIANDAO,
            // 查询型单面：改判机 rewriters 扩展（hes 任意牌改判+获得原判
            // 定牌）——JudgePhase 机器内联，无独立触发
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZJLEIJI,
            // CardResponded+CardUsed 触发：打出/使用【闪】后一步机（首个
            // 其他存活判定黑→1 雷伤+摸 1）
            trigger: Some(TriggerMeta {
                event: EventKind::CardResponded,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: vec![EventKind::CardUsed],
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(ZjLeijiBehavior),
        },
        SkillDef {
            name: BOSS_JIDIAN,
            // 查询型单面：Damage commit 尾臂——距受伤者 ≤1 的首个其他存
            // 活判定黑→1 雷伤（链挡旗标不自我递归=登记差异）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZHIWANG,
            // 查询型单面：CardsMoved commit 臂——其他角色获得牌（非摸牌
            // 阶段摸牌，CardsDrawn 天然隔离）→其弃首张手牌
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_GONGZHENG,
            // 查询型单面：Judge begin 臂——判定区有牌则弃首张（鬼计同款）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_XUECHI,
            // 查询型单面：End begin 臂——座位序首个其他存活失 2（随机→
            // 首个确定性近似登记）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_HEISHENG,
            // Death 触发（死亡前置窗口=遗言面）：全体其他横置一步机
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(HeishengBehavior),
        },
        SkillDef {
            name: BOSS_SHENGFU,
            // 查询型单面：End begin 臂——首个有坐骑的其他存活弃其一
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_LEIZHOU,
            // 查询型单面：Judge begin 臂——首个其他存活 1 雷伤
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_LEIFU,
            // 查询型单面：End begin 臂——首个其他存活横置态切换
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_LEIZHU,
            // Death 触发（遗言面，黑绳同位）：座位序逐发雷伤
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(LeizhuBehavior),
        },
        SkillDef {
            name: BOSS_FUDU,
            // 查询型单面：CardUsed commit 臂——他人用桃随机其他失 1
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_KUJIU,
            // 查询型单面：Judge begin 臂（global 扫持有者）——其他角色
            // 准备阶段失 1+视为用酒
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_RENAO,
            // Death 触发（遗言面，雷诛同位）：首发其他 3 火伤
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(RenaoBehavior),
        },
        SkillDef {
            name: BOSS_LUNHUI,
            // 查询型单面：Judge begin 臂——hp≤2 与首个 hp>2 其他交换体力
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_WANGSHENG,
            // 查询型单面：Play begin 臂——恒定虚拟南蛮（全部其他存活）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZLFANSHI,
            // 查询型单面：Damage commit 尾臂——terra 旗交替+反噬 1 伤
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_SHIYOU,
            // 查询型单面：弃牌机 Done 前臂——弃牌阶段弃牌花色互异获得
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_WANGHUN,
            // Death 触发（遗言面）：清往事旗+前两个其他各禁用首个技
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(WanghunBehavior),
        },
        SkillDef {
            name: BOSS_WANGSHI,
            // 查询型单面：Judge begin global 臂（恒定禁基本牌）+
            // PlayCard 拒绝位+回合结束全清
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_TIEMIAN,
            // 查询型单面：sha 机仁王块扩展（防具槽空视为仁王盾）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZHADAO,
            // 查询型单面：sha 机青釭穿透链扩展（来源持有者即穿透）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZHUXIN,
            // Death 触发（遗言面）：最低体力其他存活受 2 伤一步机
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(ZhuxinBehavior),
        },
        SkillDef {
            name: BOSS_SUOZU,
            // 查询型单面：Prepare begin 臂——全部其他存活横置
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ABI,
            // 查询型单面：Damage commit 尾臂——来源受 1 火伤（恒定火）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_PINGDENG,
            // Death 触发（遗言面）：最高体力两波火伤（2+1）两步机
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(PingdengBehavior),
        },
        SkillDef {
            name: BOSS_REMEN,
            // 查询型单面：PlayCard 拒绝位（plain 杀）+AOE 机窗口前跳过
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_ZHIFEN,
            // 查询型单面：Prepare begin 臂——获得座位序首个其他首张手
            // 牌+其受 1 火伤（无来源）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_HUOXING,
            // Death 触发（遗言面）：全部其他存活逐座火伤机
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(HuoxingBehavior),
        },
        SkillDef {
            name: BOSS_BUFO,
            // 查询型单面×2：伤害入口减 1（终值>1）+准备阶段距离 1 火伤
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_WULIANG,
            // 查询型单面×2：结束阶段摸 2+准备阶段体力<3 回复至 3
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_DITING,
            // 查询型单面：computed_distance 双向链（from -1 / to +1）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_WEIMU,
            // 查询型单面：黑锦囊 PlayCard 拒绝位+黑色 AOE 窗口前跳过
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_BINGFENG,
            // Death 触发（遗言面）：翻面来源一步机
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(BingfengBehavior),
        },
        SkillDef {
            name: BOSS_PANGUAN,
            // 查询型单面：延时锦囊 PlayCard 拒绝位
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_JUHUN,
            // 查询型单面：End begin 臂——座位序首个其他翻面（恒定翻面）
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_WANGXIANG,
            // Death 触发（遗言面）：全场装备全弃一步机
            trigger: Some(TriggerMeta {
                event: EventKind::Death,
                timing: Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: Vec::new(),
            }),
            limits: SkillLimits::default(),
            behavior: Box::new(WangxiangBehavior),
        },
        SkillDef {
            name: BOSS_TIANYU,
            // 查询型单面：End begin 臂——自横置先解除+全场未横置横置
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
        SkillDef {
            name: BOSS_QIWU,
            // 查询型单面：CardUsed commit 臂——梅花牌使用后受伤回复 1
            trigger: None,
            limits: SkillLimits::default(),
            behavior: Box::new(NoopBehavior),
        },
    ];
    (cards, skills)
}
