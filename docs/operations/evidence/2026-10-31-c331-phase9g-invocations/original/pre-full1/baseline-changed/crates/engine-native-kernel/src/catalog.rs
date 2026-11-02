//! 只读规则目录：卡牌、技能定义与分发表。
//!
//! 目录构建后经 `Arc` 全局共享，**不进入对局状态**——状态只保存实例级
//! 数据并以 `SkillDefId`/`CardId` 引用目录。工作副本回滚只克隆状态，
//! 不克隆目录（这是与旧 `GameState` 内嵌 `ContentRegistry` 的关键差别）。

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::event::{EventKind, Timing};
use crate::ids::{CardId, SkillDefId};
mod card_kind;

/// 装备栏（每名玩家每栏至多一张，重复装备替换旧牌入弃牌堆）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipSlot {
    Weapon,
    Armor,
    /// +1 马（防御马：他人计算与你的距离 +1）。
    HorsePlus,
    /// -1 马（进攻马：你计算与他人的距离 -1）。
    HorseMinus,
    /// 宝物栏（G1：五行盘等 equip5 家族）。
    Treasure,
}

/// 国战势力（N11 M2）：魏 / 蜀 / 吴 / 群。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Faction {
    Wei,
    Shu,
    Wu,
    Qun,
}

/// 牌类别。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardKind {
    /// 杀。
    Sha,
    /// 闪。
    Shan,
    /// 桃。
    Tao,
    HorsePlus,
    HorseMinus,
    /// 武器（射程取 `CardDef::range`）。
    Weapon,
    /// 防具（占栏；保护效果未实现，见覆盖清单）。
    Armor,
    /// 火杀（火属性伤害，触发铁索传导）。
    FireSha,
    /// 铁索连环（横置/重置 1-2 名玩家）。
    Chain,
    /// 南蛮入侵（全体其他玩家需出杀，否则受 1 点伤害）。
    SavageAttack,
    /// 万箭齐发（全体其他玩家需出闪，否则受 1 点伤害）。
    ArcheryAttack,
    /// 决斗（目标先出杀，交替出杀，先不出者受 1 点伤害；无距离限制）。
    Duel,
    /// 无懈可击（抵消锦囊；对无懈的无懈反转抵消状态）。
    Nullify,
    /// 乐不思蜀（判定非红桃 → 跳过出牌阶段）。
    Le,
    /// 兵粮寸断（判定非梅花 → 跳过摸牌阶段）。
    Bing,
    /// 闪电（判定黑桃 2-9 → 3 点雷伤；否则传下家）。
    ShanDian,
    /// 过河拆桥（弃置一名其他角色一张牌）。
    Dismantle,
    /// 顺手牵羊（获得距离 1 的一名其他角色一张牌）。
    Snatch,
    /// 酒（G1 cardpile 前置）：出牌阶段对自身使用（每回合限一次）——
    /// 本回合使用的下一张杀伤害 +1；濒死时对自身使用回复 1 点体力。
    Jiu,
    /// 雷杀（G1 cardpile 前置）：按杀结算（共享出牌阶段次数限制），
    /// 伤害为雷属性。
    LeiSha,
    /// 手里剑（G1 欢乐卡牌前置，基础牌）：目标弃 1 张装备或受 1 点伤害。
    ShouLiJian,
    /// 五行盘（G1 wuxing，宝物栏）：占栏装备；其技能经宿主行为提供。
    WuxingPan,
    /// 望梅止渴（G1 玩点论杀，锦囊）：目标若为最小体力且已受伤回复
    /// 1 点；若为最小手牌数摸两张（已回复则摸一张）。
    WangmeiZhike,
    /// 三尖（G1 玩点论杀，武器）：武器技经宿主行为提供。
    Sanjian,
    /// 水攻（G1 运筹帷幄，锦囊）：所有有装备的目标各弃一张装备
    ///（登记差异：上游为目标自选，原生取首张确定性弃置）。
    Shuigong,
    /// 釜底抽薪（G1 运筹帷幄，锦囊）：与目标拼点，赢则获得两张拼点牌。
    Fudichouxin,
    /// 趁火打劫（G1 运筹帷幄，锦囊）：获得目标一张手牌（登记差异：
    /// 上游为目标任选可见牌，原生取首张确定性获得）。
    Chenhuodajie,
    /// 索链甲（G1 运筹帷幄，防具）：装备时横置（计入铁索）；
    /// 离场时重置。装备技（伤害修改）暂缓。
    Suolianjia,
    /// 洪水（G1 运筹帷幄，延时锦囊）：自身判定，梅花 2–9 → 3 点雷伤
    ///（登记差异：上游伤害属性未标注，原生按雷属性处理）。
    HongShui,
    /// 偷梁换柱（G1 运筹帷幄，锦囊）：与目标各选一张手牌交换，
    /// 同色则摸 1（登记差异：目标临时技暂缓）。
    Toulianghuanzhu,
    /// 灵魂之火（G1 炉石，锦囊）：目标受 1 点火焰伤害，使用者弃一张手牌。
    Linghunzhihuo,
    /// 照明弹（G1 炉石，锦囊）：弃置目标一张手牌，目标与使用者各摸 1
    ///（登记差异：上游为目标自选 hej 牌，原生取首张确定性弃置）。
    Zhaomingdan,
    /// 急耗卡（G1 炉石，锦囊）：摸一张牌（临时技暂缓）。
    Jihuocard,
    /// 诱敌深入（G1 玩点论杀，锦囊）：成为杀的目标时可打出——杀对你
    /// 无效，来源受 1 点反噬伤害（登记差异：上游为强制来源连用杀，
    /// 原生简化为单次反噬）。
    Youdishenru,
    /// 隔岸观火（G1 运筹帷幄，锦囊）：双目标拼点，赢方获得输方一张
    /// 手牌，平局则目标一弃牌交由使用者（登记差异：确定性首张）。
    Geanguanhuo,
    /// 草船借箭（G1 运筹帷幄，锦囊）：目标选分支——交出手牌中所有杀
    /// 并视为对使用者出杀（简化为 1 点伤害），或展示手牌并被弃一张。
    Caochuanjiejian,
    /// 舌战群儒（G1 运筹帷幄，锦囊）：与所有有手牌的其他角色拼点
    ///（登记差异：上游为多轮响应 Bool 链+随机牌，原生为确定性首轮
    /// 拼点——各拼点牌取首张，点数大者摸 1）。
    Shezhanqunru,
    /// 舒筋散（G1 杀海拾遗·古剑，锦囊）：目标弃置任意张手牌
    ///（至少 1，可取消）并摸等量的牌。
    Shujinsan,
    /// 面里金针（G1 杀海拾遗·古剑，机关）：目标摸一张牌然后失去
    /// 1 点体力（规则 v77：目标失去 1 点体力，对齐上游 loseHp）。
    Mianlijinzhen,
    /// 飞镖（G1 杀海拾遗·古剑，机关，毒属性）：目标弃一张黑色手牌
    /// 或失去 1 点体力（规则 v77：取消分支失去 1 点体力）。
    Feibiao,
    /// 金莲烛（G1 杀海拾遗·古剑，锦囊）：不能主动使用；当任意角色
    /// 即将受到伤害时，持有者可打出之——防止此伤害，伤害来源摸一张
    ///（登记差异：上游经 chooseToUse 选牌使用，原生为持有者确认后
    /// 确定性弃置首张）。
    Jinlianzhu,
    /// 摊黄冻（G1 杀海拾遗·古剑，食物）：目标获得 3 回合状态——
    /// 出牌阶段结束时摸一张牌（倒计时在效果触发点扣减）。
    Tanhuadong,
    /// 有更（G1 杀海拾遗·古剑，食物）：目标获得 3 回合状态——阶段
    /// 开始时若已受伤且体力值为全场最少或之一，回复 1 点体力
    ///（登记差异：原生无准备阶段，挂判定阶段开始）。
    Yougeng,
    /// 青团（G1 杀海拾遗·古剑，食物）：3 回合——回合内使用首张杀时
    /// 摸一张牌（每回合限一次）。
    Qingtuan,
    /// 元宝肉（G1 杀海拾遗·古剑，食物）：3 回合——出牌阶段可以额外
    /// 使用一张杀（每回合限一次）。
    Yuanbaorou,
    /// 咕咚羹（G1 杀海拾遗·古剑，食物）：3 回合——下次受到杀造成的
    /// 伤害时，该伤害 -1（一次性，触发即移除状态）。
    Gudonggeng,
    /// 麻婆豆腐（G1 杀海拾遗·古剑，食物）：3 回合——结束阶段弃置一
    /// 名其他角色一张牌（登记差异：上游随机敌随机牌，原生为座位序
    /// 首名有牌者首张手牌/装备）。
    Mapodoufu,
    /// 春饼（G1 杀海拾遗·古剑，食物）：5 回合——手牌上限 +1。
    Chunbing,
    /// 鲤鱼汤（G1 杀海拾遗·古剑，食物）：2 回合——阶段开始时若体力值
    /// 为全场最少或之一，获得 1 点护甲（护甲原语）。
    Liyutang,
    /// 麻茶（G1 杀海拾遗·古剑，食物）：3 回合——不能成为其他角色
    /// 黑色牌的目标（登记差异：仅出牌使用路径过滤）。
    Molicha,
    /// 菱角（G1 杀海拾遗·古剑，食物）：4 回合——摸牌阶段额外摸一张牌
    /// 然后弃置一张牌（登记差异：额外摸牌在固定摸牌后即时结算）。
    Xiajiao,
    /// 蜜链藕（G1 杀海拾遗·古剑，食物）：4 回合——红桃手牌可当桃
    /// 使用（含濒死响应；登记差异：与武圣等转化并存时按既有优先级）。
    Mizhilianou,
    /// 卤玉羹（G1 杀海拾遗·古剑，食物）：3 回合——出牌阶段限一次，
    /// 弃置一张基本牌并获得牌堆顶牌（登记差异：上游为发现牌随机
    /// 五选一，原生为确定性获得牌堆顶）。
    Luyugeng,
    /// 冰魄通（G1 杀海拾遗·古剑，机关，1-3 目标）：使用者与每目标各
    /// 展示手牌首张，同色则使用者弃展示牌、目标受 1 点伤害（登记
    /// 差异：上游为双方自选展示+失去体力，原生为确定性首张+伤害）。
    Bingpotong,
    /// 乾坤镖（G1 杀海拾遗·古剑，机关）：目标及其距离 1 的邻座各弃置
    /// 一张牌（登记差异：随机牌改确定性首张）。
    Qiankunbiao,
    /// 龙须钩（G1 杀海拾遗·古剑，机关）：获得目标一张装备
    ///（登记差异：目标自选改为首张确定性）。
    Longxugou,
    /// 神火飞鸦（G1 杀海拾遗·古剑，机关，火）：目标及其距离 1 的邻座
    /// 需出闪，否则受 1 点火焰伤害。
    Shenhuofeiya,
    /// 紫阳丹（G1 杀海拾遗·古剑，基本）：目标获得 3 点护甲；每回合
    /// 结束阶段 -1（登记差异：上游为每阶段开始衰减）。
    Ziyangdan,
    /// 玉女元神（G1 杀海拾遗·古剑，基本）：目标获得一次性状态——
    /// 下次进入濒死时回复 1 点体力并移除状态。
    Yunvyuanshen,
    /// 石会粉（G1 杀海拾遗·古剑，锦囊）：目标需出闪，否则其本回合
    /// 无法对其他角色使用卡牌（登记差异：上游主要于他人出牌阶段
    /// 响应使用，原生为主动使用；禁用至其回合结束）。
    Shihuifen,
    /// 六风散（G1 杀海拾遗·古剑，锦囊）：目标从牌堆获得两张闪
    ///（登记差异：上游生成虚拟闪，原生为获得实体闪，牌堆无闪则无事）。
    Liufengsan,
    /// 砂糖（G1 杀海拾遗·古剑，锦囊，火）：目标受 1 点火焰伤害，
    /// 然后获得 1 点护甲。
    Shatang,
    /// 黑龙鳞片（G1 杀海拾遗·古剑，锦囊）：获得 1 点护甲（登记差异：
    /// 上游附带「防御距离 +1」临时技——距离修改原语暂缓）。
    Heilonglinpian,
    /// 硫头阁（G1 杀海拾遗·古剑，机关）：对自己使用摸一张牌；对其他
    /// 角色使用其弃置首张手牌（登记差异：上游为增益/减益随机效果池，
    /// 原生简化为确定性效果）。
    Liutouge,
    /// 木头面具（G1 杀海拾遗·古剑，装备）：出牌阶段限一次，任意
    /// 手牌当【杀】使用（view-as 主动装备技）。
    Mutoumianju,
    /// 帝女玄霜（G1 杀海拾遗·古剑，基本，濒死可救）：回复 1 点体力，
    /// 然后可弃置任意张手牌摸等量的牌（濒死特殊牌结算路径）。
    Dinvxuanshuang,
    /// 玉衡（G1 杀海拾遗·古剑，装备，宝物）：出牌阶段限一次——一名
    /// 有黑桃手牌的其他角色将其首张黑桃手牌交给你（登记差异：上游
    /// 随机敌随机牌+三段升级机制，原生确定性目标+升级暂缓）。
    Gjyuheng,
    /// 盾牌格挡（G1 杀海拾遗·炉石，锦囊）：获得 1 点护甲、摸一张牌、
    /// 弃置一张手牌（登记差异：确定性首张）。
    Dunpaigedang,
    /// 治疗波（G1 杀海拾遗·炉石，锦囊）：目标需已受伤——判定红色
    /// 回复 1 点体力，否则获得 1 点护甲。
    Zhiliaobo,
    /// 元素毁灭（G1 杀海拾遗·炉石，锦囊，雷）：所有其他存活角色各可
    /// 弃置 1–2 张牌，每少弃一张受 1 点雷电伤害。
    Yuansuhuimie,
    /// 星界沟通（G1 杀海拾遗·炉石，锦囊）：体力上限 +1、回复 1 点
    /// 体力、弃置全部手牌。
    Xingjiegoutong,
    /// 传送门（G1 杀海拾遗·炉石，锦囊）：获得牌堆顶一张牌
    ///（登记差异：上游获得后可立即使用并连锁，原生仅获得）。
    Chuansongmen,
    /// 坦蛇至人（G1 杀海拾遗·炉石，锦囊）：链式弃牌——从使用者起
    /// 按座位序传递，每名有手牌的角色弃置首张手牌，共进行「存活角色
    /// 数取奇」次（登记差异：上游随机敌随机牌，原生确定性座位链）。
    Tanshezhiren,
    /// 世界树（G1 杀海拾遗·炉石，锦囊）：从牌堆搜索两张不同槽位的
    /// 装备，目标依次装备（同槽旧装备替换进弃牌堆）。
    Shijieshu,
    /// 燕子药水（G1 杀海拾遗·gwent，法术）：目标若为手牌数最少者
    /// 摸两张，否则摸一张。
    Yanziyaoshui,
    /// 奥祖之雷（G1 杀海拾遗·gwent，法术，雷）：目标受 1 点雷电伤害
    /// 然后摸一张牌（若存活）。
    Aozuzhilei,
    /// 瘟疫（G1 杀海拾遗·gwent，法术）：目标有手牌弃置首张
    ///（登记差异：随机改首张），无手牌受 1 点伤害
    ///（失去体力为规则 v77 原语）。
    Wenyi,
    /// 置换军（G1 杀海拾遗·gwent，法术）：目标反复失去 1 点体力上限
    /// 直至其未受伤或手牌数不少于体力上限。
    Zhihuanjun,
    /// 暴雪药水（G1 杀海拾遗·gwent，法术）：目标弃置两张手牌
    ///（登记差异：确定性前两张）然后摸一张牌。
    Baoxueyaoshui,
    /// 倾盆大雨（G1 杀海拾遗·gwent，法术，天气）：手牌上限 -1，
    /// 直到弃牌阶段结束。
    Qinpendayu,
    /// 碧日浓雾（G1 杀海拾遗·gwent，法术，天气）：不能使用【杀】，
    /// 直到出牌阶段结束。
    Birinongwu,
    /// 刺骨寒霜（G1 杀海拾遗·gwent，法术，天气）：下个摸牌阶段摸牌
    /// 数 -1（登记差异：原生为摸牌后弃置首张，等价净 -1）。
    Ciguhanshuang,
    /// 昆恩法印（G1 杀海拾遗·gwent，法术）：防止所有普通伤害
    ///（登记差异：上游按回合计数，原生按防止次数计数，初始为存活
    /// 数与 5 的较小值）。
    Kunenfayin,
    /// 山崩术（G1 杀海拾遗·gwent，法术）：弃置其他角色的两件装备
    ///（登记差异：随机改座位序前二）。
    Shanbengshu,
    /// 破晓（G1 杀海拾遗·gwent，法术）：二选一——解除所有天气并弃置
    /// 全部判定区牌，或获得牌堆中首张天气/暴雪法术（登记差异：随机
    /// 改牌堆序首张；上游分支一为自选任意名角色）。
    Poxiao,
    /// 祖墨手铠（G1 杀海拾遗·gwent，法术）：目标的技能失效，直到其
    /// 下一个阶段结束（登记差异：原生无锁定技区分，全部技能失效；
    /// 失效至其任意阶段结束）。
    Zumoshoukao,
    /// 猴子（G1 欢乐卡牌，equip5 宝物）：他人使用【桃】时可弃此牌，
    /// 阻止桃的结算并将其收为手牌（E03 使用前取消窗口原语）。
    Monkey,
    /// 写轮眼（G1 欢乐卡牌，equip5 宝物）：回合开始可窃取他人一个技能
    /// 直到回合结束（E03 临时技能窃取原语）。
    Xuelunyang,
    /// 漩涡面具（G1 欢乐卡牌，equip5 宝物）：装备者武将牌不能被翻面
    /// （E03 翻面前置取消锁定技）。
    Mianju,
    /// 酒尾（G1 欢乐卡牌，equip5 宝物）：结束阶段可选——受伤回复 1，
    /// 否则摸 1（E03 载荷验收补齐的第 7 张实体牌）。
    Jiuwei,
    /// 无中生有（P2/N08 缺牌种批①，上游标准牌堆 ♥7/8/9/11）：使用后摸 2。
    Wuzhong,
    /// 桃园结义（P2/N08 缺牌种批①，上游标准牌堆 ♥1）：所有其他角色
    /// 回复 1 点体力。
    Taoyuan,
    /// 五谷丰登（P2/N08 缺牌种批①-2，上游标准牌堆 ♥3/♥4）：亮出存活
    /// 角色数的牌，从使用者起按座位序逐人选择获得一张。
    Wugu,
    /// 借刀杀人（P2/N08 缺牌种批①-2，上游标准牌堆 ♣12/♣13）：对装备
    /// 区有武器的角色使用，令其对另一名角色出杀或交出武器。
    Jiedao,
    /// 仁王盾（P2/N08 批③-a，上游标准牌堆 ♣2，equip2 防具）：锁定——
    /// 黑色【杀】对你无效（与泛型 Armor 的八卦阵判定语义不同牌种）。
    Renwang,
    /// 青釭剑（P2/N08 批③-b 第一把，上游标准牌堆 ♠6，equip1 武器射程 2）：
    /// 锁定——无视目标防具（八卦阵判定与仁王盾免疫均不生效）。
    Qinggang,
    /// 轩辕剑（swd 批 146）：独特武器——攻击范围 +2（上游 `distance
    /// attackFrom:-2`）、装备时 +1 护甲（体力 ≤2 且无「轩辕」则弃置）、
    /// 造成伤害 +1 并转为雷属性、伤害后若无「轩辕」则失去 1 体力。
    Xuanyuanjian,
    /// 麒麟弓（P2/N08 批③-b 第二把，上游标准牌堆 ♥5，equip1 武器射程 5）：
    /// 杀造成伤害时可弃目标装备区一张坐骑牌。
    Qilin,
    /// 寒冰剑（P2/N08 批③-b 第三把，上游标准牌堆 ♠2，equip1 武器射程 2）：
    /// 杀造成伤害时可防止此伤害，改为弃置目标两张牌。
    Hanbing,
    /// 诸葛连弩（P2/N08 批③-b 第四把，上游标准牌堆 ♣1/♦1，equip1 武器
    /// 射程 1）：锁定——出牌阶段使用【杀】无次数限制。
    Zhuge,
    /// 贯石斧（P2/N08 批③-b 第五把，上游标准牌堆 ♦5，equip1 武器射程 3）：
    /// 杀被闪后可弃两张牌强制造成伤害。
    Guanshi,
    /// 青龙偃月刀（P2/N08 批③-b 第六把，上游标准牌堆 ♠5，equip1 武器
    /// 射程 3）：杀被闪后可立即对相同目标再使用一张【杀】。
    Qinglong,
    /// 丈八蛇矛（P2/N08 批③-b 第七把，上游标准牌堆 ♠12，equip1 武器
    /// 射程 3）：两张手牌可当【杀】使用（配套主动技能）。
    Zhangba,
    /// 雌雄双股剑（P2/N08 批③-b 第八把，上游标准牌堆 ♠2，equip1 武器
    /// 射程 2）：杀指定异性目标后，目标弃一张手牌或令使用者摸一张牌。
    Cixiong,
    /// 方天画戟（P2/N08 批③-b 第九把，上游标准牌堆 ♦5，equip1 武器
    /// 射程 4）：该杀为最后手牌时可指定至多三个目标。
    Fangtian,
    /// 护符（U05 swd 首切片，规则 v85；上游 swd ♠1/♣1，基本牌）：
    /// 仅濒死且目标为自身的桃形救援牌（上游 savable dying===player）。
    Hufu,
    /// 流星火雨（U05 swd 首批第二卡，规则 v86；上游 swd ♦3/♥6/♥9，
    /// 锦囊火）：目标弃两张 he 牌或受 1 点火伤。
    Liuxinghuoyu,
    /// 草药（U05 swd 第三批，规则 v87；上游 swd ♣3/♣4/♣5，基本牌，
    /// global 距离 1）：为距离 1 以内的受伤目标回复 1 点体力。
    Caoyao,
    /// 赤龙牙（U05 swd 第四批，规则 v88；上游 swd ♦3，equip1 武器
    /// 射程 2）：锁定技——你造成的火属性伤害 +1（damageBegin num++）。
    Chilongya,
    /// 风雪刃（U05 swd 第十一批，规则 v95；上游 swd equip1 武器射程 2）：
    /// 【杀】命中后可选择——防止伤害，目标翻面并摸一张牌。
    Fengxueren,
    /// 天仙酒（U05 swd 第五批，规则 v89；上游 swd ♥9/♦2，基本牌）：
    /// 酒的 swd 变体——濒死自救/下张杀 +1，每回合限一次（登记差异：
    /// 与酒共享每回合计数）。
    Tianxianjiu,
    /// 吸血鬼指环（U05 swd 第六批，规则 v90；上游 swd ♠5，equip5 宝物）：
    /// 锁定技——你的【杀】造成伤害后，若你已受伤，回复等量体力。
    Xixueguizhihuan,
    /// 归有界（U05 swd 第七批，规则 v91；上游 swd ♠3/♣9，延时锦囊，
    /// 他人）：判定为黑色则失去 1 点体力并随机弃一张牌。
    Guiyoujie,
    /// 潜偷面具（U05 swd 第三批，规则 v92；上游 swd ♠6，equip5 宝物）：
    /// 主动技——出牌阶段限一次，弃一张手牌，判定非梅花则获得手牌较
    /// 多目标一张随机手牌。
    Shentoumianju,
    /// 星君眼（U05 swd 第九批，规则 v93；上游 swd equip5 宝物）：锁定技
    /// ——你的【杀】造成伤害 +1（damageBegin num++）。
    Xingjunyan,
    /// 青龙令珠（U05 swd 第九批，规则 v93；上游 swd equip5 宝物）：你对
    /// 其他角色造成属性伤害后，获得其一张牌（damageEnd，he 任选由
    /// RNG 口径代抽）。
    Qinglonglingzhu,
    /// 贴房绅视（U05 swd 第十批，规则 v94；上游 swd 锦囊，他人）：本回合
    /// 无视距离指定该目标（【杀】等距离校验豁免）并摸一张牌。
    Zhufangshenshi,
    /// 御夫录（U05 swd 第十二批，规则 v96；上游 swd equip5 宝物）：主动技
    /// ——出牌阶段限一次，将一张武器牌交给其他角色，目标受 1 点伤害
    ///（上游 touzhi：discard false + prepare give + damage + gain）。
    Yufulu,
    /// 羊皮卷（U05 swd 第十三批，规则 v97；上游 swd trick toself）：出牌
    /// 阶段对自己使用——选择一种类别，发现该类别一张牌入手（上游
    /// createCard 凭空造牌，原生从牌堆发现=登记差异）。
    Yangpijuan,
    /// 带火风轮（U05 swd 第十四批，规则 v98；上游 swd equip4）：
    /// 进攻距离 +2（globalFrom -2）、防御距离 -1（globalTo -1）；
    /// 纯距离修饰无技能（cardnature fire 仅主题标记，不入结算）。
    Daihuofenglun,
    /// 霞云彩绫（U05 swd 第十四批，规则 v98；上游 swd equip3）：
    /// 进攻距离 -1（globalFrom +1）、防御距离 +2（globalTo +2）。
    Xiayuncailing,
    /// 蓝格怪衣（U05 swd 第十五批，规则 v99；上游 swd equip5 宝物）：
    /// 主动判定技「龙帆」——出牌阶段限一次，判定红桃回 1 体力/方片
    /// 摸 1/梅花随机敌人随机弃一张/黑桃无事（translate.js:308 龙帆）。
    Langeguaiyi,
    /// 龙颅山峰（U05 杀海拾遗 mtg 首切片，规则 v100；上游 mtg land 地）：
    /// 使用时换地 + 二选一获得杀/闪（上游 createCard 凭空造牌，原生从
    /// 牌堆取=登记差异）；地图效果全局：出杀次数上限 +1。
    Longlushanfeng,
    /// 海岛（U05 mtg 第二批，规则 v101；上游 mtg land 地）：换地后满血
    /// +1 护甲；受伤则弃一张回 1 或取消得 1 护甲（temp 抑制转换）；
    /// 地图效果全局锁定：获护甲时若已受伤改为回等量体力。
    Haidao,
    /// 水没墓穴（U05 mtg 第三批，规则 v102；上游 mtg land 地）：换地后
    /// 可弃一张摸二（可取消）；地图效果锁定：手牌上限 -1 + 弃牌阶段
    /// 弃过牌则阶段结束摸一。
    Shuimomuxue,
    /// 废墟（U05 mtg 第四批，规则 v103；上游 mtg land 地）：换地后从
    /// 弃牌堆发现一张牌（亮 ≤3 三选一）；地图效果：回合开始（准备
    /// 阶段）随机将弃牌堆一张置于牌堆顶。
    Feixu,
    /// 依夏兰（U05 mtg 第五批，规则 v104；上游 mtg land 地）：换地后选
    /// 类型随机获得基本/锦囊牌一张；地图效果：出牌阶段限一次将一张
    /// 基本牌重铸为锦囊牌（地附主动技，换地授予/撤销）。
    Yixialan,
    /// 沙漠绿洲（U05 mtg 第六批，规则 v105；上游 mtg land 地）：换地后
    /// 从牌堆基本牌发现一张；地图效果：基本牌不计入手牌上限。
    Shamolvzhou,
    /// 林中尖塔（U05 mtg 第七批，规则 v106；上游 mtg land 地）：换地后
    /// 从牌堆武器发现并直接装备一张；地图效果=地附主动技：装备区内有
    /// 武器时出牌阶段限一次将一张基本牌当杀使用（丈八蛇矛 declared-
    /// Sha 同构）。
    Linzhongjianta,
    /// 冰河要塞（U05 mtg 第八批，规则 v107；上游 mtg land 地）：换地后
    /// 摸二弃二（强制）；地图效果锁定全局：使用杀时若有牌须弃置一张
    ///（上游 chooseToDiscard 可取消，原生 RNG 代抽=登记差异）。
    Bingheyaosai,
    /// 幽林（U05 mtg 第九批，规则 v108；上游 mtg land 地）：换地后随机
    /// 使用一张普通锦囊（登记差异：池收窄为决斗/无中生有——内核结算
    /// 已确证者；收益判定为 AI 不迁移）；地图效果=地附主动技：出牌阶
    /// 段限一次弃一张锦囊牌并从牌堆发现一张牌。
    Youlin,
    /// 邻地溪流（U05 mtg 第十批，规则 v109；上游 mtg land 地）：换地后
    /// 牌堆♥首张入手，可弃任意张♥（可取消）——每弃一张随机敌方判定
    /// 区置入一张其未持有的延时锦囊（牌堆实体乐/兵/闪电）。
    Lindixiliu,
    /// 地载阵（U05 杀海拾遗 zhenfa 首切片，规则 v110；上游 zhenfa 牌
    /// mode:["guozhan"]）：所有非大势力角色各摸一张牌（大势力=已明置
    /// 势力存活人数最大者，effective_faction 推导；队列/围攻无依赖）。
    Dizaizhen,
    /// 机关牌（U05 swd 卡子包，规则 v189 首切片指路小虎）。
    Jiguan,
    /// 灵剑（U05 swd 机关术批，规则 v198）：swd 自造改造牌类型
    /// （hslingjian，7 张；机关术合成与机关妖术补偿的池）。
    Hslingjian,
    /// 雪霁冰雹（swd 卡牌面补切批②）：basic——目标摸牌阶段摸牌数 +1，
    /// 持续其 2 个摸牌阶段（PlayerState.xuejibingbao 计数）。
    Xuejibingbao,
    /// 神秘果（swd 卡牌面补切批②）：basic——你使用基本牌/普通锦囊结算
    /// 后可弃此牌令该牌再结算一次（每阶段限一次；窗口=host 触发技）。
    Shenmiguo,
    /// 封印之丹（swd 卡牌面补切批②）：basic——对自身使用，随机使用两
    /// 张普通锦囊牌（登记差异：原生确定性窄池{五谷/决斗}+座位序目标）。
    Fengyinzhidan,
    /// 炼妖壶（swd 卡牌面补切批③）：equip5 宝物——收纳/炼化两 equipSkill
    /// +壶境再装备/手牌上限+2 依赖。
    Lianyaohu,
    /// 乾坤袋（swd 卡牌面补切批④）：equip5 宝物——手牌上限 +1（纯 mod）。
    Qiankundai,
    /// 神农鼎（swd 卡牌面补切批④）：equip5 宝物——出牌限一次弃 2 张手
    /// 牌回复 1 点体力（受伤时）。
    Shennongding,
    /// 女娲石（swd 卡牌面补切批④）：equip5 宝物——其他角色濒死且其体力
    /// ≤0、自己体力 >1 时：目标回复 1、自己失去 1（global 触发）。
    Nvwashi,
    /// 昆仑镜（swd 卡牌面补切批⑤）：equip5 宝物——出牌限一次亮牌堆顶
    /// 3 张选 1 获得、以 1 张手牌替换（登记差异：上游替换牌入特殊区）。
    Kunlunjingc,
    /// 东皇钟（swd 卡牌面补切批⑤）：equip5 宝物——自己阶段结束弃 1 张
    /// 红牌，置一张延时锦囊实体入目标判定区（实体发现替代 createCard）。
    Donghuangzhong,
    /// 空洞印（swd 卡牌面补切批⑥）：equip5 宝物——死亡取消型：hp=1、
    /// 摸 1、弃全部宝物（death_cancelled 基建消费）。
    Kongdongyin,
    /// 惊雷山（swd 卡牌面补切批⑦）：trick AOE（雷版南蛮——全体其他需
    /// 出杀，不出则雷 1 伤；逆座位序）。
    Jingleishan,
    /// 炽羽袭（swd 卡牌面补切批⑦）：trick AOE（火版万箭——全体其他需
    /// 出闪，不出则火 1 伤；逆座位序）。
    Chiyuxi,
    /// 鬼蛇旗（swd 卡牌面补切批⑦）：trick 单目标 +1 护甲。
    Guisheqi,
    /// 仙露回（swd 卡牌面补切批⑦）：trick AOE——全体受伤角色（含自己）
    /// +1 护甲（逆座位序）。
    Xianluhui,
    /// 龟灵之桃（swd 卡牌面补切批⑧）：equip5 宝物——逆攻标记积攒（受伤
    /// +1 上限 4）与弃标记打伤/摸牌消费。
    Guilingzhitao,
    /// 四非珍面（swd 卡牌面补切批⑨）：equip5 宝物——出牌限一次令其他有
    /// 手牌角色判定，非红桃则其用首张手牌指向座位序首个其他存活者（登记
    /// 差异上游随机牌/随机目标）。
    Sifeizhenmian,
    /// 书锤（swd 卡牌面补切批⑪）：equip5 宝物——多杀循环机暂缓，kind 先
    /// 注册卡牌数据入包。
    Shuchui,
    /// 散灯金环（swd 卡牌面补切批⑪）：equip5 宝物——杀被闪后判定改写暂
    /// 缓，kind 先注册。
    Sadengjinhuan,
    /// 昊天塔（swd 卡牌面补切批⑪）：equip5 宝物——判定改写暂缓，kind 先
    /// 注册。
    Haotianta,
    /// 玉如意（swd 卡牌面补切批⑪）：equip5 宝物——摸牌改写暂缓，kind 先
    /// 注册。
    Yuruyi,
    /// 玉蝉·乾（swd 卡牌面补切批⑫）：器族手持 autoViewAs 杀。
    Yuchanqian,
    /// 玉蝉·坤（swd 卡牌面补切批⑫）：器族手持 autoViewAs 草药。
    Yuchankun,
    /// 玉蝉·震（swd 卡牌面补切批⑫）：器族手持 autoViewAs 酒。
    Yuchanzhen,
    /// 玉蝉·巽（swd 卡牌面补切批⑫）：器族手持 autoViewAs 桃。
    Yuchanxun,
    /// 玉蝉·坎（swd 卡牌面补切批⑫）：器族手持 autoViewAs 神秘果（原生
    /// 神秘果无直用面，同拒）。
    Yuchankan,
    /// 玉蝉·离（swd 卡牌面补切批⑫）：器族手持 autoViewAs 天仙酒。
    Yuchanli,
    /// 玉蝉·艮（swd 卡牌面补切批⑫）：器族手持 autoViewAs 封印之丹。
    Yuchangen,
    /// 玉蝉·兑（swd 卡牌面补切批⑫）：器族手持 autoViewAs 雪霁冰雹。
    Yuchandui,
    /// 神草结（swd 卡牌面补切批⑬）：trick 手持交互——锦囊伤害前，持有
    /// 者为受伤者则伤-1、为来源则伤+1（各限持牌数）。
    Shencaojie,
    /// 青龙之圭（swd 卡牌面补切批⑩）：器族手持——准备阶段摸 2 弃 1。
    Qinglongzhigui,
    /// 朱雀之璋（swd 卡牌面补切批⑩）：器族手持——受他人伤害后来源火 1 伤。
    Zhuquezhizhang,
    /// 白兽之琥（swd 卡牌面补切批⑩）：器族手持——弃牌阶段末弃他人一张牌。
    Baishouzhihu,
    /// 玄武之璜（swd 卡牌面补切批⑩）：器族手持——造成伤害后回复等量体力。
    Xuanwuzhihuang,
    /// 黄麟之琮（swd 卡牌面补切批⑩）：器族手持——准备阶段无护甲则+1。
    Huanglinzhicong,
    /// 天覆阵（U05 zhenfa，规则 v200/T19a）：大国 AOE 弃 1。
    Tianfuzhen,
    /// 七星阵（U05 zhenfa，规则 v200/T19a）：围攻者弃 1+视为对其用杀。
    Qixingzhen,
    /// 鸟翔阵（U05 zhenfa，规则 v200/T19a）：逐合法目标响应闪或受 1 伤。
    Niaoxiangzhen,
    /// 长蛇阵（U05 zhenfa，规则 v201/T19b）：队列摸 1 或换座成队列。
    Changshezhen,
    /// 破阵诀（U05 zhenfa，规则 v201/T19b）：随机座位重排。
    Pozhenjue,
}

/// 花色（判定用）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Suit {
    Spade,
    Heart,
    Club,
    Diamond,
}

impl Suit {
    pub fn is_heart(self) -> bool {
        matches!(self, Suit::Heart)
    }

    pub fn is_club(self) -> bool {
        matches!(self, Suit::Club)
    }

    pub fn is_spade(self) -> bool {
        matches!(self, Suit::Spade)
    }
}

/// 牌定义。`range` 仅武器使用：无武器时基础攻击范围为 1；
/// `suit`/`rank` 为判定花色与点数（目录期固定分配）。
#[derive(Debug, Clone)]
pub struct CardDef {
    pub name: &'static str,
    pub kind: CardKind,
    pub range: u32,
    pub suit: Suit,
    pub rank: u8,
    /// 装备技能（G1 生命周期：装备时授予、离场撤销；目录技能名）。
    pub equip_skills: Vec<&'static str>,
    /// 装备时横置（索链甲：计入铁索）；离场时重置。
    pub link_on_equip: bool,
}

impl CardDef {
    pub fn basic(name: &'static str, kind: CardKind) -> Self {
        Self {
            name,
            kind,
            range: 0,
            suit: Suit::Spade,
            rank: 1,
            equip_skills: Vec::new(),
            link_on_equip: false,
        }
    }

    pub fn weapon(name: &'static str, range: u32) -> Self {
        Self {
            name,
            kind: CardKind::Weapon,
            range,
            suit: Suit::Spade,
            rank: 1,
            equip_skills: Vec::new(),
            link_on_equip: false,
        }
    }

    /// 声明装备技能（G1 生命周期：装备时授予、离场撤销）。
    pub fn with_equip_skills(mut self, skills: &'static [&'static str]) -> Self {
        self.equip_skills = skills.to_vec();
        self
    }

    /// 装备时横置（索链甲语义）；离场时重置。
    pub fn with_link_on_equip(mut self) -> Self {
        self.link_on_equip = true;
        self
    }

    pub fn with_suit_rank(mut self, suit: Suit, rank: u8) -> Self {
        self.suit = suit;
        self.rank = rank;
        self
    }
}

/// 技能触发元数据（目录静态部分）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerMeta {
    pub event: EventKind,
    pub timing: Timing,
    /// 数值大者先结。
    pub priority: i32,
    /// Fixed source metadata; false for silent or unnamed triggers.
    pub order_selectable: bool,
    /// 可选触发：需目标玩家确认，确认前不出现在强制队列。
    pub optional: bool,
    /// 额外分发的事件类别（一个技能响应多种事件，如连营）。
    pub extra_events: Vec<EventKind>,
}

/// 技能固有约束（目录静态部分）。
#[derive(Debug, Clone, Copy, Default)]
pub struct SkillLimits {
    /// 每回合限一次。
    pub once_per_turn: bool,
    /// 主动技例外：不绑定出牌阶段（如激将的上游入口是
    /// `chooseToUse`/`chooseToRespond`——需要使用或打出杀时即可发动）。
    /// 默认 false = 主动技限出牌阶段（上游 `enable: "phaseUse"`）。
    pub active_outside_play: bool,
}

/// 技能定义：元数据 + 原生行为实现。
pub struct SkillDef {
    pub name: &'static str,
    pub trigger: Option<TriggerMeta>,
    pub limits: SkillLimits,
    pub behavior: Box<dyn SkillBehavior>,
}

impl std::fmt::Debug for SkillDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SkillDef")
            .field("name", &self.name)
            .field("trigger", &self.trigger)
            .field("limits", &self.limits)
            .finish_non_exhaustive()
    }
}

/// 原生技能行为接口。触发过滤在 collect 与运行前各做一次，
/// 运行前重查使「结算中途被禁用」的技能不再生效。
pub trait SkillBehavior: Send + Sync {
    fn native_damage_effect(
        &self,
        _kind: crate::event::EventKind,
    ) -> Option<crate::event::NativeDamageEffect> {
        None
    }

    /// 该技能所有者是否在给定事件上触发（只读查询）。
    fn filter(
        &self,
        ctx: &QueryCtx<'_>,
        owner: crate::ids::PlayerId,
        event: &crate::event::EventData,
    ) -> bool;

    /// 触发确认后启动的续执行机器。
    fn start(
        &self,
        owner: crate::ids::PlayerId,
        event: &crate::event::EventData,
    ) -> Box<dyn SkillMachine>;

    /// Start with the same authoritative query context used to select handlers.
    fn start_in_context(
        &self,
        _ctx: &QueryCtx<'_>,
        owner: crate::ids::PlayerId,
        event: &crate::event::EventData,
    ) -> Box<dyn SkillMachine> {
        self.start(owner, event)
    }

    /// Trigger dispatch uses the effective kind of its existing frame window.
    fn filter_in_window(
        &self,
        ctx: &QueryCtx<'_>,
        owner: crate::ids::PlayerId,
        event: &crate::event::EventData,
        _kind: crate::event::EventKind,
    ) -> bool {
        self.filter(ctx, owner, event)
    }
    fn start_in_window(
        &self,
        ctx: &QueryCtx<'_>,
        owner: crate::ids::PlayerId,
        event: &crate::event::EventData,
        _kind: crate::event::EventKind,
    ) -> Box<dyn SkillMachine> {
        self.start_in_context(ctx, owner, event)
    }
    fn vm_trigger_window_is_authorized(
        &self,
        bytes: &[u8],
        _kind: crate::event::EventKind,
    ) -> bool {
        self.vm_module_is_authorized(bytes)
    }

    /// Catalog module roles distinguish trigger continuations from active Intrinsic work.
    fn vm_module_requires_trigger_frame(&self, _bytes: &[u8]) -> bool {
        false
    }

    /// 主动技（N11）：经 GameAction::ActivateSkill 发动时由引擎调用；
    /// 默认无主动形态（返回 None）。
    fn activate(
        &self,
        _ctx: &QueryCtx<'_>,
        _owner: crate::ids::PlayerId,
    ) -> Option<Box<dyn SkillMachine>> {
        None
    }

    fn package_skill_id(&self) -> Option<u32> {
        None
    }

    fn hand_limit_base(
        &self,
        _core: &crate::state::GameStateCore,
        _owner: crate::ids::PlayerId,
    ) -> Option<usize> {
        None
    }

    fn vm_module_is_authorized(&self, _bytes: &[u8]) -> bool {
        false
    }

    fn vm_trigger_module_is_authorized(
        &self,
        _bytes: &[u8],
        _event: &crate::event::EventData,
    ) -> bool {
        false
    }

    fn vm_trigger_still_matches(
        &self,
        _bytes: &[u8],
        _ctx: &QueryCtx<'_>,
        _owner: crate::ids::PlayerId,
        _event: &crate::event::EventData,
        _kind: crate::event::EventKind,
    ) -> bool {
        false
    }

    /// PA12 批⑤-2（§2 K）：VM 桥技能探针——PhaseBegin 前置窗只前移 VM 技能
    ///（宿主技保持原执行点，行为与 golden 不变）。
    fn is_vm_skill(&self) -> bool {
        false
    }

    /// G1 规则宿主：宿主原语行为标识（非宿主行为返回 None）。
    /// 供恢复期校验快照中的扩展机器定义键与行为标签。
    fn as_host_behavior(&self) -> Option<crate::vocabulary::host::HostBehavior> {
        None
    }

    /// G1 目标过滤原语：延时锦囊（乐/兵）不能指定该技能持有者为目标。
    fn blocks_delayed_trick_target(&self) -> bool {
        false
    }

    /// 出牌封锁原语（swd 批 134 冰封）：持有该行为的启用实例时，玩家
    /// 不能使用/打出/响应/求桃任何牌（上游 `bingfeng2` mod
    /// cardEnabled/cardUsable/cardRespondable/cardSavable 全 false 同口径）。
    /// 出牌封锁的**状态相关**变体（L27 止息：标记/锦囊记录达条件才封锁）。
    /// 默认委托静态值；状态相关技能覆盖之。
    fn blocks_all_cards_when(
        &self,
        _catalog: &Catalog,
        _core: &crate::state::GameStateCore,
        _p: crate::ids::PlayerId,
    ) -> bool {
        self.blocks_all_cards()
    }

    fn blocks_all_cards(&self) -> bool {
        false
    }

    /// 本技能的具名 filter / 查询需要本回合弃置与摸牌史。
    /// 标准宿主技为 false，标准局面不写 VmAux，v295 摘要保持不变。
    fn needs_vm_card_history(&self) -> bool {
        false
    }
}

/// 只读查询上下文：机器/过滤器访问对局状态与目录的唯一只读入口。
pub struct QueryCtx<'a> {
    pub state: &'a crate::state::GameStateCore,
    pub catalog: &'a Catalog,
}

/// 技能续执行机器：类型化局部状态 + 单步推进。
///
/// 每个技能自带一个具体类型（如 `SanCaiMachine { stage: SanCaiStage, .. }`），
/// 暂停/恢复就是把该类型留在挂起槽里，恢复后继续 `step`——
/// 取代旧实现 `ContinuationKind + Vec<u32>` 的无类型载荷。
/// `fork` 用于工作副本回滚：暂停态随状态一起克隆。
/// `snapshot` 导出全部局部状态的规范快照：进 canonical hash（不同机器
/// 局部状态可区分），也是暂停点保存/恢复的唯一数据来源。
/// 线程安全（T13 工作池）：机器为纯数据状态——Send 既有，补 Sync 使
/// 引擎可跨线程共享（多座并行规划；实现体均为普通结构体）。
pub trait SkillMachine: Send + Sync {
    /// Window identity used solely for dynamic subscription injection into ancestor queues.
    fn trigger_order(&self) -> Option<&crate::resolution::TriggerOrder> {
        None
    }
    fn trigger_order_mut(&mut self) -> Option<&mut crate::resolution::TriggerOrder> {
        None
    }
    fn step(&mut self, ctx: &mut crate::machine::Ctx<'_>) -> crate::machine::Flow;
    fn fork(&self) -> Box<dyn SkillMachine>;
    fn snapshot(&self) -> crate::snap::MachineSnapshot;
}

/// 内容卡牌的固有结算：`None` = 不是内容卡牌（内核继续分派）；`Some(m)` = 已处理。
pub type CardIntrinsicHook = fn(
    &mut crate::machine::Ctx<'_>,
    &crate::event::CardUseData,
    CardKind,
) -> Option<Option<Box<dyn SkillMachine>>>;

/// 宿主机器挂起询问的派生（行为/阶段、owner/target、targets、行为专用 card、cards）。
pub type HostAskHook = fn(
    u16,
    u8,
    crate::ids::PlayerId,
    Option<crate::ids::PlayerId>,
    &[crate::ids::PlayerId],
    Option<u32>,
    &[crate::ids::CardId],
    &crate::state::GameStateCore,
    &Catalog,
) -> Option<crate::snap::ExpectedAsk>;

/// 内容机器的快照重建：`None` = 不是内容机器（内核继续分派）。
pub type RestoreMachineHook =
    fn(
        &crate::snap::MachineSnapshot,
        &Catalog,
    ) -> Option<Result<Box<dyn SkillMachine>, crate::snap::SnapshotError>>;

/// 内容层回调：内核在帧固有流程、询问派生与机器恢复处经此调用内容层，
/// 不直接引用 `crate::content`（拆 crate 第 2 步）。内容目录构建时装入
/// 内容实现；`Catalog::build` 默认 [`ContentHooks::NONE`]（不认识任何内容机器）。
#[derive(Clone, Copy)]
pub struct ContentHooks {
    pub card_intrinsic: CardIntrinsicHook,
    pub vm_active: fn(
        &rule_vm::BytecodeModule,
        &str,
        crate::ids::PlayerId,
        &crate::state::GameStateCore,
    ) -> bool,
    pub vm_validate:
        fn(&rule_api::EffectRequest, &[u32], &crate::state::GameStateCore, usize) -> bool,
    pub vm_commit: fn(
        &mut crate::machine::Ctx<'_>,
        &str,
        &rule_api::EffectRequest,
        &mut Vec<u32>,
    ) -> Option<crate::machine::Flow>,
    pub vm_ask: fn(
        &rule_api::EffectRequest,
        &[u32],
        &crate::state::GameStateCore,
        &Catalog,
    ) -> Option<crate::snap::ExpectedAsk>,
    pub host_ask: HostAskHook,
    pub restore_machine: RestoreMachineHook,
    pub borrow_options: fn(
        &crate::machine::Ctx<'_>,
        crate::ids::PlayerId,
        bool,
    ) -> Vec<crate::choice::ChoiceOption>,
    pub borrowed_owner: fn(
        &crate::machine::Ctx<'_>,
        crate::ids::PlayerId,
        crate::ids::CardId,
    ) -> Option<crate::ids::PlayerId>,
}

impl ContentHooks {
    pub const NONE: ContentHooks = ContentHooks {
        card_intrinsic: |_, _, _| None,
        vm_commit: |_, _, _, _| None,
        vm_validate: |_, state, _, _| state.is_empty(),
        vm_active: |_, _, _, _| true,
        vm_ask: |_, _, _, _| None,
        host_ask: |_, _, _, _, _, _, _, _, _| None,
        restore_machine: |_, _| None,
        borrow_options: |_, _, _| Vec::new(),
        borrowed_owner: |_, _, _| None,
    };
}

/// 目录：卡牌表、技能表与事件分发索引。
pub struct Catalog {
    pub cards: Vec<CardDef>,
    pub skills: Vec<SkillDef>,
    /// 扩展武将（G1 角色包管线）：接在基础武将表之后的目录条目。
    pub extension_generals: Vec<crate::vocabulary::generals::ExtensionGeneral>,
    /// 从启用扩展初始化声明派生的全局授技；恢复时随目录重建，不入存档。
    pub extension_global_skills: Vec<SkillDefId>,
    /// 事件类别 -> 触发定义列表（静态，运行时按实例过滤）。
    dispatch: BTreeMap<EventKind, Vec<SkillDefId>>,
    /// 名称 -> 技能定义号（构建时生成，首个匹配，与线性扫描语义一致；
    /// PA01：computed_distance 等热路径按名查技，曾随目录规模线性劣化）。
    by_name: std::collections::HashMap<&'static str, SkillDefId>,
    /// 规则版本：目录语义变化时递增；随状态入摘要、随快照做恢复绑定。
    /// 由内容模块在构建后赋值（`Catalog::build` 初始化为 0）。
    pub ruleset_version: u64,
    /// 目录里有 VM 技能需要本回合弃置/摸牌史。不入对局摘要。
    pub vm_card_history: bool,
    /// 内容层回调（内容目录构建时装入）。不入对局摘要。
    pub hooks: ContentHooks,
}

impl Catalog {
    /// 五谷剩余归获查询（玩点论杀批 15——良策）：场上存活角色持有带
    /// 五谷剩余归获语义的宿主技能（良策·余获）则返回其座位——五谷结算
    /// 后剩余牌归其所有。内核通用查询——不点名扩展技能。
    pub fn wugu_remain_beneficiary(
        &self,
        core: &crate::state::GameStateCore,
    ) -> Option<crate::ids::PlayerId> {
        core.skills
            .iter()
            .find(|((p, def), inst)| {
                inst.enabled
                    && self.skills[*def as usize]
                        .behavior
                        .as_host_behavior()
                        .is_some_and(|h| h.wugu_remain_gain())
                    && core.player(*p).alive
            })
            .map(|((p, _), _)| *p)
    }

    /// AOE 目标排除查询（目标排除原语首例，玩点论杀批 13）：目标持有的
    /// 任一宿主技能声明 AOE 排除语义（坚壁）则 AOE（南蛮/万箭）对其无效。
    /// 内核通用查询——不点名扩展技能。
    pub fn aoe_excludes_target(
        &self,
        core: &crate::state::GameStateCore,
        target: crate::ids::PlayerId,
    ) -> bool {
        core.skills
            .range((target, u16::MIN)..)
            .take_while(|((p, _), _)| *p == target)
            .any(|((_, def), inst)| {
                inst.enabled
                    && self.skills[*def as usize]
                        .behavior
                        .as_host_behavior()
                        .is_some_and(|h| h.aoe_exclude())
            })
    }

    /// 由技能表构建分发索引；重复注册同一事件按定义号排序保证确定。
    pub fn build(cards: Vec<CardDef>, mut skills: Vec<SkillDef>) -> Catalog {
        let mut dispatch: BTreeMap<EventKind, Vec<SkillDefId>> = BTreeMap::new();
        for (id, def) in skills.iter().enumerate() {
            if let Some(t) = &def.trigger {
                dispatch.entry(t.event).or_default().push(id as SkillDefId);
                for k in &t.extra_events {
                    dispatch.entry(*k).or_default().push(id as SkillDefId);
                }
            }
        }
        for list in dispatch.values_mut() {
            list.sort_unstable();
            list.dedup();
        }
        // 技能表按定义号排列，保证目录本身确定。
        skills.shrink_to_fit();
        let vm_card_history = skills.iter().any(|s| s.behavior.needs_vm_card_history());
        let mut by_name: std::collections::HashMap<&'static str, SkillDefId> =
            std::collections::HashMap::with_capacity(skills.len());
        for (id, def) in skills.iter().enumerate() {
            by_name.entry(def.name).or_insert(id as SkillDefId);
        }
        Catalog {
            cards,
            skills,
            extension_generals: Vec::new(),
            extension_global_skills: Vec::new(),
            dispatch,
            by_name,
            ruleset_version: 0,
            vm_card_history,
            hooks: ContentHooks::NONE,
        }
    }

    /// 武将总数 = 基础表 + 扩展表（G1 角色包管线）。
    pub fn general_count(&self) -> usize {
        crate::vocabulary::generals::STANDARD_GENERALS.len()
            + crate::vocabulary::generals::BOSS_GENERALS.len()
            + self.extension_generals.len()
    }

    /// 武将下标的元数据（基础表优先，扩展表接续；越界 None）。
    /// Typed identity of the immutable boss_mushengoumang source declaration.
    pub fn mushengoumang_general_id(&self) -> u16 {
        let source = crate::vocabulary::generals::BOSS_MUSHENGOUMANG;
        assert_eq!(
            crate::vocabulary::generals::BOSS_GENERALS[source.0],
            source.1
        );
        u16::try_from(
            crate::vocabulary::generals::STANDARD_GENERALS.len()
                + self.extension_generals.len()
                + source.0,
        )
        .expect("catalog general bound")
    }

    pub fn general_meta(
        &self,
        g: u16,
    ) -> Option<(
        &'static str,
        Faction,
        Vec<&'static str>,
        u32,
        crate::vocabulary::generals::Gender,
    )> {
        let base = crate::vocabulary::generals::STANDARD_GENERALS;
        if (g as usize) < base.len() {
            let (name, faction, skills, hp, gender) = base[g as usize];
            return Some((name, faction, skills.to_vec(), hp, gender));
        }
        // 扩展武将段：约定接在基础表之后（扩展将 def=base+i，g1_d3 等
        // 消费面 ABI）
        let ext = self.extension_generals.get(g as usize - base.len());
        if let Some(ext) = ext {
            return Some((
                ext.name,
                ext.faction,
                ext.skills.to_vec(),
                ext.max_hp,
                ext.gender,
            ));
        }
        // Boss 内置武将段（v208）：动态尾段（在全部启用扩展之后——扩展
        // 将 def 约定不受扰动；boss 将 def=base+ext_n+i 随启用集浮动）
        let boss = crate::vocabulary::generals::BOSS_GENERALS;
        let boss_start = base.len() + self.extension_generals.len();
        if (g as usize) >= boss_start && (g as usize) < boss_start + boss.len() {
            let (name, faction, skills, hp, gender) = boss[g as usize - boss_start];
            return Some((name, faction, skills.to_vec(), hp, gender));
        }
        None
    }

    pub fn skill_name(&self, id: SkillDefId) -> &'static str {
        self.skills[id as usize].name
    }

    /// 按名字查技能定义号（构建内容/测试用；O(1) 查表，首个匹配）。
    pub fn skill_by_name(&self, name: &str) -> Option<SkillDefId> {
        self.by_name.get(name).copied()
    }

    /// 按名字查牌编号（测试布阵用）。
    pub fn card_by_name(&self, name: &str) -> Option<CardId> {
        self.cards
            .iter()
            .position(|c| c.name == name)
            .map(|i| i as CardId)
    }

    pub fn card_kind(&self, card: CardId) -> CardKind {
        self.cards[card as usize].kind
    }

    /// 牌的**实际**装备栏（swd 批 143 改造）：`declared_slots` 覆盖优先，
    /// 否则取牌种固有栏位。装备技能仍按牌种授予（牌种未变）。
    pub fn card_slot_of(
        &self,
        core: &crate::state::GameStateCore,
        card: CardId,
    ) -> Option<EquipSlot> {
        core.declared_slots
            .get(&card)
            .copied()
            .or_else(|| self.card_kind(card).equip_slot())
    }

    /// 某事件类别的候选触发定义（静态顺序，确定）。
    pub fn candidates(&self, kind: EventKind) -> &[SkillDefId] {
        self.dispatch.get(&kind).map(Vec::as_slice).unwrap_or(&[])
    }

    /// G1 目标过滤原语：`p` 是否持有「延时锦囊目标免疫」类技能
    ///（墨守：乐/兵不能指定其为目标）。
    /// 出牌封锁查询（swd 批 134 冰封）：玩家任一启用实例的行为声明封锁。
    pub fn blocks_all_cards(
        &self,
        core: &crate::state::GameStateCore,
        p: crate::ids::PlayerId,
    ) -> bool {
        core.skills.iter().any(|((o, def), inst)| {
            *o == p
                && inst.enabled
                && self.skills[*def as usize]
                    .behavior
                    .blocks_all_cards_when(self, core, p)
        })
    }

    pub fn blocks_delayed_trick(
        &self,
        core: &crate::state::GameStateCore,
        p: crate::ids::PlayerId,
    ) -> bool {
        core.skills
            .keys()
            .any(|(o, _)| *o == p)
            .then(|| {
                core.skills.iter().any(|((o, def), inst)| {
                    *o == p
                        && inst.enabled
                        && self.skills[*def as usize]
                            .behavior
                            .blocks_delayed_trick_target()
                })
            })
            .unwrap_or(false)
    }
}

/// `Arc` 化目录别名：引擎全程共享一份。
pub type SharedCatalog = Arc<Catalog>;
