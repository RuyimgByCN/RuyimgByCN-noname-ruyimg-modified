# 原生线玩法规则基准（项目声明与版本约定）

状态：活文档，2026-10-10 复核（源码当时为 `STANDARD_RULESET_VERSION = 192`、`ARCHIVE_FORMAT_VERSION = 41`；**2026-10-18 NC11 勘正：已提交 HEAD 为 v321 / 存档 v105，当前 dirty 工作树为 v322 / 存档 v105**——`crates/engine-native/src/content/standard.rs:126`、`crates/engine-native/src/archive.rs:113`；本行原写「现源码为 v291 / 存档 v79」为 2026-10-16 时点）。v1–v60 见下表，v61 起逐版本条目按批次写在其后各节，**v322 为最新（工作树）**。本页只记录规则与兼容取舍，当前测试结果统一见[进度主稿](native-progress.md#当前核验快照)。存档未知牌种与载荷读尽已于 10-07 加固（负例随测试运行），10-06 的[诊断探针](../operations/evidence/2026-10-06-native-progress-audit/archive-probes.log)只描述当时源码。截断事故两次：snap.rs 于第 43 轮重建（见第 43 轮证据），host.rs 于 2026-10-10 v192 批重建（见 [v192 恢复证据](../operations/evidence/2026-10-10-replay-golden-v192-recovery/README.md)）。
本文记录内部预期与未决项；本地上游/旧线参考、实现符合声明、官方版本等价分别举证。
当前结果与反例以 [进度主稿](native-progress.md) 为准。

## 版本约定与历史

规则语义、目录或状态编码变化须递增规则版本，登记原因后更新 golden；不能仅重录哈希消除失败。**例外（2026-10-17）**：内核调度缺陷修复（`resolution::run` 挂起机器恢复深度改为开窗时刻栈深——修「应答被深度失配的别台机器消费」panic）未改规则语义、未动版本，仅 `nullify_chain_on_savage` 一场位移且快照差只有 `resolution.budget`（调度步数 −4）；重录依据写在 `replay_scenarios.rs` 表头，语义等价核验见[证据](../operations/evidence/2026-10-17-t13-ai-policy-v2/README.md)。
版本 28 和第 48 轮的版本 59 均有 14 个 golden 通过记录；v60 差异核因与重录依据已记录于下表“回放修复”行，最新结果见主稿。golden 通过也不能证明未进入场景的技能/模式正确。

| 版本 | 主要变化 / 决策 |
|---|---|
| 1–3 | 初始内核、逐目标无懈/身份局、使用时窗口与段预算，ADR 0704–0709 |
| 4–5 | 判定稳定分组、鬼道/天妒目录与范围/旧牌归属，ADR 0712–0714 |
| 6 | 真实装备资格/恢复、判定事件全序、finalized/judge_held 编码，ADR 0720 |
| 7–8 | 濒死多轮救援、八卦阵判定，ADR 0726/0729 |
| 9–14 | 鬼才/反馈、伤害牌来源、奸雄/遗计、阶段技能、转化、无双/集智，ADR 0736–0745 |
| 15–19 | 主动技、制衡/英姿/克己、国色声明牌种、奇袭/咆哮，ADR 0746–0752 |
| 20–23 | 双将/势力/明置、刚烈/突袭/裸衣、观星/连营、仁德/激将/马术/铁骑，ADR 0754–0760 |
| 24 | 顺手牵羊、CardsDiscarded 事件、激将内嵌杀结算，ADR 0763 |
| 25 | 令牌代次、校验、倾国/奇才/救援，ADR 0768；ADR 0769 明置装配沿用此版本 |
| 26 | 六技能机器、提示词映射，ADR 0770；ADR 0771–0773 沿用此版本 |
| 27 | Sha 流离/护驾五字段进入快照、字节与哈希；提示词注册表，ADR 0774 |
| 28 | 武将性别字段及离间/结姻筛选，ADR 0775；14 golden 通过 |
| 29 | 护驾/激将逐人询问重构（Sha/Jijiang 机器快照字段变化）、仁德阶段累计（`TurnState.rende_given` 入状态）、主动技阶段门控（`SkillLimits.active_outside_play` 例外）、存档 v2（OpenChoice 显式编码 ChoiceKind；v1 被门禁拒绝）、E03 扩展内容锁（启用扩展集合指纹参与目录版本派生）——ADR 0776；14 golden 因状态编码与语义变化重录 |
| 30 | G1 cardpile 前置：新增 `CardKind::Jiu`（每回合限一次、下一杀 +1 伤害、濒死回血；`TurnState.jiu_used`/`jiu_active` 入状态）与 `CardKind::LeiSha`（雷属性杀、共享杀次数）；设计意图为基础目录加酒×2/雷杀×2（80→84 张），当前构造循环实际张数须复核——ADR 0776 G1；当轮 14 golden 因牌库与状态变化重录 |
| 31 | G1 规则宿主原型：`MachineSnapshot::Host`（扩展机器载荷：def_key/行为标签/续行阶段，入快照/哈希/字节，恢复期绑定校验）、`CardKind::ShouLiJian`、宿主技能目录注册（`ExtensionPack.host_skills` → SkillDef）——ADR 0776 G1；14 golden 因目录与快照变化重录 |
| 32 | G1 wuxing：五行属性入状态（`PlayerState.wunature` / `core.card_natures`，`InitHook::WuxingNatures` 经引擎 RNG 初始化）、`ChoiceKind::PickTag`/`ChoiceOption::Tag` 选择原语、`CardKind::WuxingPan` + `EquipSlot::Treasure`——ADR 0776 G1；14 golden 因状态变化重录 |
| 33 | G1 3D精选：角色包管线（`Catalog.extension_generals` + `general_meta` 统一查询；`RestoreBounds.general_count` 槽位界）、装备技能生命周期（`CardDef.equip_skills` 装备授予/离场撤销 6 路径 + `EquipSkillMismatch` 恢复校验）、精彀宿主行为——ADR 0776 G1；14 golden 因目录与语义变化重录 |
| 34 | G1 玩点论杀：`CardKind::WangmeiZhike` 锦囊（无懈窗口包装）+ `WangmeiZhikeCard` 宿主原语（最小体力/最小手牌条件增益）；卡牌派发机器 def_key 绑定按牌种在场判定——ADR 0776 G1；14 golden 因目录变化重录 |
| 35 | G1 玩点论杀武将批：6 名武将元数据入包（角色包管线）；思继宿主原语（`CardsDiscarded` 可选触发、弃杀数×2 摸牌；`HostMachine.cards` 关联牌组入快照）——ADR 0776 G1；14 golden 因机器快照变化重录 |
| 36 | G1 玩点论杀原语增补：三尖武器技（`SanjianWeaponDamage` 弃牌+目标两段选择）与墨守目标免疫（`blocks_delayed_trick_target` 原语钩子，乐/兵命令边界过滤）；`CardKind::Sanjian`——ADR 0776 G1；14 golden 因目录变化重录 |
| 37 | G1 运筹帷幄原语增补：水攻（`ShuigongCard` 多目标强制弃装备；`HostMachine.targets` 多目标入快照）与釜底抽薪（`FudichouxinCompare` 拼点两段选择，平局目标胜）；`CardKind::Shuigong`/`Fudichouxin`——ADR 0776 G1；14 golden 因目录与快照变化重录 |
| 38 | G1 运筹帷幄切片扩展：趁火打劫（`ChenhuodajieCard` 获得目标手牌，无懈窗口包装）与索链甲（`CardKind::Suolianjia` + `CardDef.link_on_equip` 横置/重置钩子）——ADR 0776 G1；14 golden 因目录变化重录 |
| 39 | G1 运筹帷幄切片扩展：洪水（`CardKind::HongShui` 自-target 延时锦囊，判定梅花 2–9 → 3 点雷伤）与偷梁换柱（`TouliangSwap` 三段选择换牌，同色摸 1；目标临时技暂缓）——ADR 0776 G1；14 golden 因目录变化重录 |
| 40 | G1 杀海拾遗·炉石子包切片：灵魂之火/照明弹/急耗卡（`CardKind` 三种 + 宿主行为 tag 15–17，def_key 按牌种在场绑定）——ADR 0776 G1；14 golden 因目录变化重录 |
| 41 | G1 玩点论杀余量原语：诱敌深入响应转化（`CardKind::Youdishenru`，杀响应选项并入，打出后杀无效 + 来源反噬 1 点；登记差异：无连用循环）——ADR 0776 G1；14 golden 因目录变化重录 |
| 42 | G1 运筹帷幄余量原语：隔岸观火（`GeanguanhuoCompare` 双目标拼点链，赢方获得输方手牌，平局目标一弃牌）；`CardKind::Geanguanhuo`——ADR 0776 G1；14 golden 因目录变化重录 |
| 43 | G1 运筹帷幄余量原语：草船借箭（`CaochuanCard` 分支选择 PickTag——交出所有手牌杀+反噬杀 / 展示+被弃一张；登记差异：简化为直接伤害+确定性首张）；`CardKind::Caochuanjiejian`——ADR 0776 G1；14 golden 因目录变化重录 |
| 44 | G1 运筹帷幄收官：舌战群儒（`ShezhanqunruCompare` 全体确定性首轮拼点——使用者与所有有手牌的其他角色各比手牌首张，大者摸 1；登记差异：上游为多轮 Bool 问答链）；`CardKind::Shezhanqunru`；kernel 侧 AOE 目标展开（按座位序存活且有手牌）——ADR 0776 G1；14 golden 因目录变化重录 |
| 45 | G1 杀海拾遗·古剑首切片：舒筋散（`GujianShujinsanCard` 目标弃任意张手牌摸等量，可取消；登记差异：原生仅手牌）/面里金针（`GujianMianliCard` 摸 1 后受 1 点伤害，登记差异：失去体力简化为伤害）/飞镖（`GujianFeibiaoCard` 弃黑色手牌或受伤，可取消=空选择）；`CardKind::Shujinsan`/`Mianlijinzhen`/`Feibiao`——ADR 0776 G1；14 golden 因目录变化重录 |
| 46 | G1 古剑续切片：金莲烛（`JinlianzhuCancel` damageBefore 事件取消——监听技授予全体角色、持烛者逐个确认、弃烛防止伤害、来源摸 1；登记差异：chooseToUse 简化为确认后确定性弃置）；`CardKind::Jinlianzhu`（响应牌拒主动使用）——ADR 0776 G1；14 golden 因目录变化重录 |
| 47 | G1 古剑食物计时状态：`SkillInstance.rounds` 倒计时字段（入哈希/存档；效果触发点扣减归零移除）+ 授予机/监听技框架；摊黄冻（出牌阶段结束摸 1）/有更（已受伤且体力最少回 1，登记差异：挂判定阶段开始）；`CardKind::Tanhuadong`/`Yougeng`——ADR 0776 G1；14 golden 因状态编码与目录变化重录 |
| 48 | G1 古剑食物效果面：青团（首杀摸 1，once_per_turn）/元宝肉（额外一张杀——杀次数校验钩子）/咕咚羹（受杀伤害 -1，一次性移除）/麻婆豆腐（结束阶段确定性弃牌，登记差异：随机改确定性）/春饼（手牌上限 +1——弃牌上限计算钩子）；`CardKind::Qingtuan`/`Yuanbaorou`/`Gudonggeng`/`Mapodoufu`/`Chunbing`——ADR 0776 G1；14 golden 因目录变化重录 |
| 49 | G1 护甲原语 + 食物第三批：`PlayerState.hujia`（伤害 1:1 先行抵扣，入哈希/存档；HujiaAbsorbed 日志）+ 鲤鱼汤（体力最少得护甲）/麻茶（黑色牌目标限制——出牌校验钩子，登记差异：不含响应路径）/菱角（摸牌阶段额外摸 1 后弃 1——选择机）；`CardKind::Liyutang`/`Molicha`/`Xiajiao`——ADR 0776 G1；14 golden 因状态编码与目录变化重录 |
| 50 | G1 古剑食物簇收官：蜜链藕（红桃当桃——状态驱动 view-as：出牌转化链+通用尾部 use_as+濒死 is_tao_like 并入；登记差异：转化并存按既有优先级）/卤玉羹（出牌阶段限一次弃基本牌获得牌堆顶——主动型宿主监听技 activate；登记差异：发现牌简化为牌堆顶）；`CardKind::Mizhilianou`/`Luyugeng`——ADR 0776 G1；14 golden 因目录变化重录 |
| 51 | G1 古剑机关簇：天女散花（逐目标首张同色比对——**同色：使用者弃展示牌 + 目标失去 1 点体力**；**登记差异：确定性首张**。〔2026-10-19 更正：原记「失去体力简化为伤害」**不准确**——样本 102 以护甲载体实测：目标有护甲 2 仍掉血且护甲不变 ⇒ 原生为 **`loseHp` 语义**，非 damage〕）/乾坤镖（目标及邻座各弃首张；登记差异：随机改确定性）/龙须钩（获得目标首张装备）/神火飞鸦（目标及邻座需闪否则 1 火伤——targets 响应队列逐目标循环+挂起派生）；`CardKind::Bingpotong`/`Qiankunbiao`/`Longxugou`/`Shenhuofeiya`——ADR 0776 G1；14 golden 因目录变化重录 |
| 52 | G1 古剑余簇：紫阳丹（+3 护甲可叠加，结束阶段 -1；登记差异：上游每阶段衰减）/玉女元参（Dying 触发一次性回复标记）/石会粉（需闪否则禁用对其他角色用牌——出牌校验钩子+expires_end_of_turn；登记差异：响应使用改主动使用）/流风散（获得两张实体闪；登记差异：虚拟牌改实体牌）；`CardKind::Ziyangdan`/`Yunvyuanshen`/`Shihuifen`/`Liufengsan`——ADR 0776 G1；14 golden 因目录变化重录 |
| 53 | G1 古剑最后一批：沙棠（1 火伤+1 护甲——emit/Yield 两段式时序）/黑龙鳞片（+1 护甲；登记差异：防御距离临时技暂缓）/硫头阁（自己摸 1/他人弃首张；登记差异：随机效果池简化为确定性）/木头面具（view-as 装备技——equip_skills+once_used，EquipSlot::Armor 注册）；`CardKind::Shatang`/`Heilonglinpian`/`Liutouge`/`Mutoumianju`——ADR 0776 G1；14 golden 因目录变化重录 |
| 54 | G1 古剑收官双卡：帝女玄霜（濒死特殊牌——is_tao_like 并入按桃救回 + 主动使用回复+rider 弃任意摸等量；登记差异：濒死救援弃摸骑乘暂缓）/玉衡（主动型装备技——确定性取其他角色首张黑桃；登记差异：升级机制+随机敌暂缓）；`CardKind::Dinvxuanshuang`/`Gjyuheng`——ADR 0776 G1；14 golden 因目录变化重录 |
| 55 | G1 炉石余量：盾牌格挡（+1 护甲摸 1 弃 1，登记差异：确定性首张）/治疗波（判定红回 1 否则+1 护甲）/元素毁灭（逐目标弃 1–2 每少弃一张受 1 雷伤——逐目标弃减伤循环）/星界沟通（上限+1 回 1 弃全部）/传送门（获得牌堆顶；登记差异：立即使用链暂缓）/坦蛇至人（座位序链式弃牌；登记差异：随机改确定性）；`CardKind` 67–72——ADR 0776 G1；14 golden 因目录变化重录 |
| 56 | G1 炉石收官：世界树（`HearthShijieshuCard` 装备搜索+穿戴机——牌堆自顶扫描两张不同槽位装备，同槽替换+grant_equip_skills 授予；登记差异：按槽位去重近似上游 subtype、机器直穿不经 Equipped 帧）；`CardKind::Shijieshu` 73——ADR 0776 G1；14 golden 因目录变化重录 |
| 57 | G1 gwent 首切片：燕子药水（最小手牌数摸 2 否则 1）/奥祖之雷（1 雷伤后摸 1）/瘟疫（弃首张或受伤；登记差异：随机改首张、失去体力简化为伤害）/置换军（反复失去体力上限）/暴雪药水（弃前两张摸 1；登记差异：确定性前两张）；`CardKind::Yanziyaoshui`/`Aozuzhilei`/`Wenyi`/`Zhihuanjun`/`Baoxueyaoshui`——ADR 0776 G1；14 golden 因目录变化重录 |
| 58 | G1 gwent 天气状态群：倾盆大雨（手牌上限 -1 至弃牌阶段结束，手牌上限钩子读取）/碧日浓雾（不能使用杀至出牌阶段结束，杀次数校验前置钩子）/刺骨寒霜（摸牌阶段结束弃首张；登记差异：等价净 -1）——授予机目标+邻座（含使用者）跳过已持有；`CardKind::Qinpendayu`/`Birinongwu`/`Ciguhanshuang`——ADR 0776 G1；14 golden 因目录变化重录 |
| 59 | G1 gwent 收官：昆恩法印（防止普通伤害——DamageBefore+prevented，按防止次数计数；登记差异：上游按回合计数）/山崩术（弃其他角色前两件装备；登记差异：随机改座位序前二）/破晓（PickTag 二选一——清除天气+判定区牌或获得牌堆首张天气法术；登记差异：随机改牌堆序）；`CardKind::Kunenfayin`/`Shanbengshu`/`Poxiao`——ADR 0776 G1；14 golden 因目录变化重录 |
| 60（初次引入） | gwent 祖墨手铠/封印原语使卡牌切片达 21/21；当时定向测试 6/6、`replay_scenarios` 有 1 项失败（14 个 golden 场景哈希不符）。这是修复前快照；后续处理与依据见下一行 |
| 60（回放修复，2026-09-24） | 14 golden 重录并全量通过。差异核因：①规则版本 59→60 字段入 canonical_hash（`engine.rs`），必变全部哈希；v60 行为增量仅经 `CardKind::Zumoshoukao`（枚举末位追加）的 gwent 分支，标准场景目录无此牌、命令/事件/状态轨迹语义不变（14 场景脚本中途断言全部通过）；②`standard_catalog_parts` 拆牌循环体误含顺手牵羊/酒/雷杀入目录语句（实际产出顺 8、酒 4、雷杀 4，共 88 张），修正为[声明表](#标准目录构成声明表)的 80 张。证据见[回放与目录核验](../operations/evidence/2026-09-24-replay-golden-v60/README.md) |

> **v61–v294 的逐版本变化**按批次登记于[进度主稿](native-progress.md) T8/T11/T12 行与各批证据目录（如 [v294 D4 Boss B3](../operations/evidence/2026-10-16-d4-boss-b3-sixiang3/README.md)）；本表保留至 v60 的历史粒度。
>
> **近期版本（2026-10-17）**：**v295**（D4 Boss B4 后援/还人/青赤白翼，tag 427–437，存档 v79）；**v296**（swd 魂身 `hyunshen_count`，存档 v80→81）；**v297**（swd 待行 `daixing_count`，存档 v82）；**v298**（swd 聚悉 `juxi_count`，存档 v83）；**v299**（swd 破舟 `pozhou_count`，存档 v84）；**v300**（swd 陷阱 `sxianjing_traps`，存档 v85）；**v301**（swd 孤行 `guxing_armed`/`guxing_count`，存档 v86）；**v302**（swd 锦鳞 `jinlin_used`，存档 v86→87）；**v303**（T11 Boss 诸葛亮族 hlss 星系统 `hlss_stars`/`hlss_count`/`hlss_damage`，存档 v87→88）；**v304**（同族批 116 起风/改命：`hlss_fengqi_used` + 选择项 `ChoiceOption::Kind` + 判定机改命候选位，存档 v88→89）；**v305**（D5 龙血玄黄潜龙道心值 `mbqianlong_marks`，存档 v89→90）；**v306**（T11 Boss 庞统族涅槃 `niepan_used` + 浴火关卡循环型 `loop_type`，存档 v90→91）；**v307**（swd 灵武：`cards_used_turn`/`extra_phases`/`extra_phase_lingwu`，存档 v91→92）；**v308**（swd 神焰：`shenyan_used`/`shenyan_pending`，存档 v92→93）；**v309**（swd 入梦：`skip_discard_phase`，存档 v93→94）；**v310**（swd 义山：`yishan_lost`，存档 v94→95）；**v311**（swd 守印：`shouyin_used`，存档 v95→96）；**v312**（swd 魅惑 + `CardsLost` 失去牌事件原语；无字段变化、存档仍 v96）；**v313**（swd 颂词 `songci_targets`，存档 v96→97）；**v314**（swd 护天四字段，存档 v97→98）；**v315**（swd 极略 `jilve_used` + 通用目标规则表 `legal_targets`，存档 v98→99）；**v316**（swd 脉轮三字段，存档 v99→100）；**v317**（swd 天书两字段，存档 v100→101）；**v318**（swd 改造 `declared_slots` 装备栏改写表，存档 v101→102）；**v319**（swd 燎原 `liaoyuan_count`，存档 v102→103）；**v320**（swd 轩辕剑族：新牌种 `CardKind::Xuanyuanjian` + `TurnState.xuanyuanjian_marked`，存档 v103→104）；**v321**（swd 昆仑镜快照两字段，存档 v104→105）；**v322（2026-10-18 工作树，NC07 同源审计）**：`seat_ring`（换位结果）纳入规范摘要 + 恢复期新增座位环长度/取值/排列校验——摘要覆盖变化 ⇒ 14 场 golden 全部重录（回退探针见 [NC07 证据](../operations/evidence/2026-10-18-nc07-same-source/README.md)，重录依据见 `crates/engine-native/tests/replay_scenarios.rs:1061`）。**归因说明（2026-10-18 NC11 勘正）**：v322 属 NC07 座位环入摘要，**不是** NC01 座位距离/顺序语义修正（NC01 未改版本，14 golden 哈希在修正前后逐字一致，见同表 NC01 行）。无版本批：**批 113 灵矢**（tag 460 四条锁定臂内联，golden 零扰动）与 **批 114 之外**的装载器来源包（如 L13 手杀MVP）不改规则版本。每批 14 golden 重录 + 版本回退探针，证据见 [b100](../operations/evidence/2026-10-17-t12-swd-deferred-b100/README.md)…[b114](../operations/evidence/2026-10-17-t12-swd-deferred-b114/README.md)、[hlss 星系统](../operations/evidence/2026-10-17-t11-boss-zhugeliang-hlss/README.md)、[起风/改命+Boss 装配](../operations/evidence/2026-10-17-t11-boss-zhugeliang-four/README.md)、[潜龙道心值](../operations/evidence/2026-10-17-d5-longxue-mbqianlong/README.md)、[涅槃/浴火/神威](../operations/evidence/2026-10-17-t11-boss-pangtong-niepan/README.md)、[灵武+额外阶段](../operations/evidence/2026-10-17-t12-swd-deferred-b120/README.md)、[神焰](../operations/evidence/2026-10-17-t12-swd-deferred-b121/README.md)、[入梦](../operations/evidence/2026-10-17-t12-swd-deferred-b122/README.md)、[义山](../operations/evidence/2026-10-17-t12-swd-deferred-b123/README.md)、[守印](../operations/evidence/2026-10-17-t12-swd-deferred-b124/README.md)、[魅惑+失去牌事件](../operations/evidence/2026-10-17-t12-swd-deferred-b125/README.md)、[颂词](../operations/evidence/2026-10-17-t12-swd-deferred-b129/README.md)、[护天](../operations/evidence/2026-10-17-t12-swd-deferred-b131/README.md)、[极略](../operations/evidence/2026-10-17-t12-swd-deferred-b136/README.md)、[脉轮](../operations/evidence/2026-10-17-t12-swd-deferred-b137/README.md)、[天书](../operations/evidence/2026-10-17-t12-swd-deferred-b142/README.md)、[改造](../operations/evidence/2026-10-17-t12-swd-deferred-b143/README.md)、[燎原](../operations/evidence/2026-10-17-t12-swd-deferred-b145/README.md)、[轩辕剑族](../operations/evidence/2026-10-17-t12-swd-deferred-b146/README.md)、[昆仑镜](../operations/evidence/2026-10-17-t12-swd-deferred-b149/README.md)。

NGSA 已有字节格式版本及规则版本字段，内存 restore 会检查当前规则版本；v2 起开放选择显式编码
ChoiceKind（机器种类为权威，不再由提示词推导），未知提示词仍被注册表拒绝，v1 字节被版本门禁拒绝。
头部版本/长度、提示词无限增长池、流离内存标记原反例已修。决斗/AOE/流离/无懈/遗计两阶段/天妒/
弃牌上限/护驾逐人/激将逐人的字节与内存恢复均已通过；
牌归属完整核算（全集 ∪ 机器暂离）拒绝尾端/中段/全量缺失。
ADR 0769 的明置实际行为、ADR 0771 的恢复映射变化沿用旧版本：后续需核对语义变更与版本递增的一致性。
格式布局变更、规则语义变更与宿主日志扩展需分别说明，旧数据不因格式号相同就自动兼容。

**N11 机器编码 tag 表（2026-09-24 修正，格式版本保持 v2）：**`MachineSnapshot` 的
XiaoJi（枭姬摸牌机）原与 Dismantle 共用 tag 29，解码 XiaoJi 分支不可达——枭姬挂起
无法恢复；修复为 XiaoJi=43（表尾追加），由 [A0 检查](../../crates/engine-native/tests/a0_tag_registry.rs)
守护（42 变体 tag 唯一 + 编码→解码往返；`HostBehavior` 74 tag 唯一；`CardKind`
86 变体 ↔ 0..=85 编码双射经静态核查确认）。兼容取舍：tag 29 的解码语义保持
Dismantle，含 Dismantle 挂起的旧 v2 字节完全兼容，故不递增格式版本；XiaoJi 挂起的
旧字节因字段错位从未可正确解码，属本来损坏的数据，不在兼容范围。未分配 tag（如
23/24/255）解码拒绝并报 BadTag。

**NGSA 格式 v3（2026-10-07，E03 使用前取消）：**`EventData::CardUsed` 载荷尾部
追加 `cancelled: u8`（猴子偷桃的确认机器写回，提交阶段读到真值跳过结算）。
取舍：该字段不入 canonical_hash（取消的最终状态——牌位置与体力——已可观测，
路径标记无需入摘要），14 个 golden 不变；存档布局变更递增格式版本，旧 v2 字节
被版本门禁拒绝（存档仅存在于测试与内部工具，无生产迁移数据）。同批 `CardKind`
尾部追加 `Monkey`（编码 86、键 `monkey`、宝物栏），牌种伴随技能表登记
欢乐猴子目录技能（装备技随真实穿戴/离场授予撤销）。

**规则 v61 + 格式 v4（2026-10-07，E03 翻面原语）：**`PlayerState` 尾部追加
`face_down`（武将牌朝向，E03 欢乐【漩涡面具】原语）——状态编码追加使
14 golden 重录（face_down 初始 false，行为无变化，重录依据充分）；存档
PlayerState 编码尾部追加同字段（格式 v3→v4，旧 v3 字节版本门禁拒绝）。
翻面语义：`TurnOverBefore` 前置取消窗口（漩涡面具锁定技强制取消）→
翻转 → `TurnOvered`；`debug_flip` 为测试驱动的翻面来源（真实翻面来源
如据守类武将技另批）。同批 `CardKind::Mianju`（编码 88、键 `mianju`）
伴随技能表登记欢乐面具目录技能。

**翻面帧派发语义（2026-10-07 帧生命周期缺陷修复固化）：**机器
`emit` 的 `TurnOverBefore` 子事件与直接压帧（`debug_flip`）一致从
`BeforeTriggers` 起步——取消窗口先于 Commit 完整派发，取消写回后
Commit 跳过翻转；翻面帧 after 窗口静默（目录无「翻面后」触发种类，
取消技不重复入队）。曹仁「据守」（原生新增，上游标准包无此条目）的
翻面走同一子事件通道：面具取消翻面时摸牌仍执行（登记差异）。

**猴子偷桃语义（E03，与上游差异登记）：**上游在 `useCardToBegin` 帧后取消
（事件有痕迹）；原生在出牌提交拦截——确认门挂起（`huanle_monkey_confirm`，
逐持有者按座位序，确认者弃猴子、桃收为其手牌；全部放弃则按普通桃结算）。
确认的使用不发 CardUsed 帧（连营等监听 CardUsed 的技能不触发——上游取消
语义下触发亦不发生，一致）。多持有者顺序与放弃/确认组合有测试。

**欢乐包独立规则载荷验收（2026-10-07，逐张对照上游 `card/list.js`+
`card/card.js`+`card/skill.js`，commit 5c02a745）：**上游 7 张实体牌全部
在册（`huanle_pack_payload_matches_upstream_list` 守护名称序/花色/点数/
牌种/装备栏）：苦无 ♠6 武器、手里剑 ♦4/♠4 基础牌、猴子 ♦5 宝物、
写轮眼 ♥2 宝物、面具 ♣3 **防具**（equip2；本批修正，曾误登记宝物栏）、
酒尾 ♥9 宝物（本批补齐：`CardKind::Jiuwei` 编码 89 表尾追加，包声明
映射既有 `host:phase_end_recover_or_draw`——结束阶段可选受伤回 1 否则
摸 1，与上游 frequent 语义一致，真实装备驱动 + 确认/放弃/恢复测试）。
本批修正：猴子 ♦5（曾 ♥8）、面具 ♣3（曾 ♠1）。苦无登记差异：上游
`damageSource` 过滤 `card.name=="sha"` + `notLink()` + 目标 `he` 区有牌
+ 目标自选弃牌 + `_notrigger` 排除；原生为 Damage 帧来源过滤 +
`is_sha_like()` + 不查连环（铁索传导伤害仍触发）+ 不查 `_notrigger`
+ 目标仅持装备时也不弃（无「有牌」门槛）+ 确定性弃目标首张手牌
（仅手牌、无牌静默）。**2026-10-18 更正**：旧文记「`is_sha_like()` 含
火/雷杀，超集」有误——杀族牌 `name` 恒为 `"sha"`、火/雷只体现在
`nature`（`card/extra.js:1481` `["heart",4,"sha","fire"]`、
`library/element/card.js:315` `this.name = card[2]`、`card/standard.js:99-102`
`sha: { nature: ["thunder","fire","kami","ice"] }`），故上游 `name==="sha"`
对火/雷杀同为真，`is_sha_like()`（Sha/FireSha/LeiSha）与之**同口径**，
不是超集。NC08 样本 5 逐条验收与行为固定见
[证据](../operations/evidence/2026-10-18-nc08-upstream-kuwu/README.md)。

五行生克登记差异（NC08 样本 6，2026-10-18，[证据](../operations/evidence/2026-10-18-nc08-upstream-shengke/README.md)）：
上游 `_shengke` 是 `_` 前缀全局技（`game/index.js:8859-8861` 自动
`addGlobalSkill`），被克 `chooseToDiscard(true,'he')`（角色自选）、被生摸 1；
牌属性默认概率取 `lib.config.wuxing_num_playpackconfig`（未配置时 `||0`
→ **0 张**，与扩展 help 文案「三分之一」不一致）。原生以每座位技能实例
授予、被克确定性弃首张手牌（仅手牌；仅持装备时不弃）、开局钩子按包数据
`nature_fraction` **1/3** 直接分配属性牌，触发事件面为 `CardUsed` 帧
（顺序与上游 `useCardToBegin` 对齐：帧先于效果机器入栈）；属性随机流走
引擎 RNG，不与上游 `Math.random` 逐局等价。**缺陷登记（未修，2026-10-18
探针实测）**：五行盘主动技 `wuxingpan_skill` 上游 `usable:1`（每阶段一次），
原生 `host_skill_def` 未登记 `once_per_turn`，同一出牌阶段可再次发动
（`content/host.rs` 的 limits 分支缺 `HostBehavior::ChangeCardNature`）。
写轮眼登记差异：原生无 charlotte 区分，全部技能实例可窃（上游滤除
charlotte 技）。

2026-09-24 审查更正：旧 XiaoJi 字节写的是 tag 29，现表下仍按 Dismantle 解码（不会报
`BadTag("machine", 29)`），错位后才在后续字段报错。实测三种布局：单挂起、两个 XiaoJi
为 `UnexpectedEof`，XiaoJi 后接 Dismantle 为 `BadTag("slot", 3)`。解码不校验载荷是否
读尽，错位字节恰好可解析时会被接受，"显式报错"没有机制保证（[审查修复证据](../operations/evidence/2026-09-24-review-fixes/README.md)）。
同批发现：存档牌种编号解码遇未知值直接 panic（伪造载荷重算 blake3 即可触发），
与载荷读尽校验一起登记为主稿待做。2026-10-06 [探针复核](../operations/evidence/2026-10-06-native-progress-audit/archive-probes.log)确认：未知编号 255 仍 panic，载荷尾部插入 4 字节并正确更新长度/校验和后仍被接受；合法存档对照正常往返。该探针的“通过”是缺陷复现，不是解码器验收。

## 标准目录构成声明表

`standard_catalog()` 是自造测试牌堆（非上游 `card/standard.js` 的 108 张），构成以
`crates/engine-native/tests/standard_catalog_inventory.rs` 的显式声明表为准（A0 检查，
测试守护）。**规则 v74 起共 124 张**，分四段：

| 段 | 构成 | 花色点数口径 |
|---|---|---|
| 对齐段 79 张 | 杀 30/闪 15/桃 8/南蛮 3/决斗 3/顺牵 5/过拆 6/无懈 4/乐 3/闪电 2（♠1 + ♥12 `shandian_safe`） | **v74 起上游 standard list 逐张落位**（闪/桃全红） |
| 原生扩充段 25 张 | ±马 6（无花色）/酒 2（♠3 ♣9）/雷杀 2（♠4 ♣5）/泛型武器 3（射程 2/2/3）/防具八卦 2/火杀 4/铁索 4/万箭 1（♥1 同上游）/兵粮 1（♣3） | 原生取值，登记花色差异（酒/雷杀入目录为 G1 cardpile 前置，v30 起） |
| 批①新牌种 9 张 | 无中生有 4（♥7/8/9/11）/桃园 1（♥1）/五谷 2（♥3/♥4）/借刀 2（♣12/♣13） | v62/v63 上游落位 |
| 命名装备 11 张 | 仁王盾 ♣2/青釭 ♠6/麒麟 ♥5/寒冰 ♠2/连弩 ♣1 ♦1/贯石斧 ♦5/青龙 ♠5/丈八 ♠12/雌雄 ♠2/方天 ♦5 | v64–v73 上游落位（射程面并行） |

声明表变更 = 牌堆语义变化：须递增 `STANDARD_RULESET_VERSION` 并重录
14 golden（v62–v74 均按此执行）。

## 上游标准牌表对照

上游 `card/standard.js` `list` 块实测 108 张物理牌、32 牌种（逐张清单见
[牌表对照证据](../operations/evidence/2026-10-07-n08-cardlist-parity/README.md)）。
原生 `standard_catalog()` 80 张声明测试牌（21 牌种功能）与之逐张对照：

| 对照面 | 上游 | 原生现状 | 差距 |
|---|---|---|---|
| 基本牌 | 杀 30 / 闪 15 / 桃 8 | 杀 12（+火杀 4+雷杀 2）/ 闪 12 / 桃 12 | 数量差；酒/火杀/雷杀为原生扩充（上游军争） |
| 锦囊 | 过拆 6、顺牵 5、无中 4、无懈 4、南蛮 3、决斗 3、乐 3、五谷 2、借刀 2、闪电 2、桃园 1、万箭 1 | 过拆 2、顺牵 4、无懈 4、决斗 4、闪电 2、万箭 1、南蛮 1、乐 1、兵粮 1（原生扩充） | **缺牌种 4：无中生有/五谷丰登/桃园结义/借刀杀人**；其余为数量差 |
| 马 | 六匹命名马（绝影/的卢 -1，爪黄飞电/赤兔/大宛/紫骍 +1） | 泛型 HorsePlus/HorseMinus 各 4 | 命名化 vs 泛型建模（6 张数量差） |
| 武器 | 9 把命名武器（诸葛连弩 2/雌雄/青釭/青龙/丈八/贯石斧/方天/麒麟/寒冰） | 泛型 Weapon 3（射程建模） | 命名特效未建模 |
| 防具 | 八卦阵 2、仁王盾 1 | 泛型 Armor 1（八卦阵判定语义已实现） | 仁王盾缺、数量差 |

**缺牌种补齐分批**：①技能牌种 4（无中/五谷/桃园/借刀）；②装备命名化
取舍评估（17 张：维持泛型+数量对齐 或 逐把命名特效）；③数量对齐
（声明表变更 = 递增 STANDARD_RULESET_VERSION + 重录 golden）。
**三批全部完成**：批① v62 无中/桃园、v63 五谷/借刀；批② 取舍定案（马/八卦
维持泛型、仁王盾+武器 9 把逐把特效）；批③-a v64 数量对齐+仁王盾（89→114 张）、
③-b①~⑨ v65–v73 九把武器（114→124 张）、③-c v74 对齐段花色点数逐张落位
（张数不变；马/八卦/原生扩充段花色为原生取值并登记，见[声明表](#标准目录构成声明表)）。

### 方天画戟立项评估（批③-b⑨ 单独立项，2026-10-07）

上游口径（`fangtian_skill`，useCard1 强制触发 + selectTarget mod）：
使用【杀】时若**全部手牌均被选为该杀的卡牌**（= 杀为最后手牌；国战
模式除外），该杀可选目标上限 +2（= 至多三个目标）；content 为空——
多目标由杀卡自身结算逐目标处理。

原生差距与工程范围（ShaResolveMachine 为单目标设计）：
1. PlayCard Sha 臂：`targets.len() != 1` 门控放宽为 1..=3，条件=来源
   装备方天且该杀为其最后手牌（手牌数==1 时使用）；目标逐个查距离/
   空城/存活。
2. ShaResolveMachine 目标队列化：现有单 target 字段改为目标队列，
   逐目标完整走 仁王盾→铁骑→八卦→流离→护驾→出闪（含无双双闪）
   →贯石斧/麒麟弓/寒冰→伤害；逐目标状态入快照（现 armor_judged/
   dodges_used/tieji_red/liuli/hujia 均为单目标语义，需按目标索引）。
3. 交互登记：无双=每个目标各需双闪；护驾逐人询问每目标各一轮；流离
   可把目标转给其他角色后再继续队列；青龙再杀在队列耗尽后才追问。
4. 快照/编码：Sha 快照结构改动大，规格随实现批一次递增（批③-b⑨ 一
   次版本递增 + golden 重录）。
5. 体量估计：最大单把（约为前七把总和）；建议独立批次一次成型，不
   拆半成品入牌堆（沿用「未实现不落位」定案）。

**定案**：方天画戟批③-b⑨ 单独立项，前置=批③-a/③-b①~⑧ 已合入
（均已完成）；实现顺序上置于全部逐把武器之后作为收官批。

### 装备命名化取舍（批②，2026-10-07 评估定案）

上游 17 张装备的特效清单与原生建模对照（射程由 `distance.attackFrom`
换算：青釭 2/雌雄 2/寒冰 2/青龙 3/丈八 3/贯石斧 3/方天 4/麒麟 5/连弩 1）：

| 牌 | 上游特效 | 原生现状 | 决策 |
|---|---|---|---|
| 马 6（绝影/的卢 −1；爪黄飞电/赤兔/大宛/紫骍 +1） | 距离 ±1 | 泛型 Horse±（语义完全等价） | **维持泛型**，批③按 ±各 3 张落位；命名差异登记为展示层，不立项 |
| 八卦阵 2 | 判定红色当闪 | 泛型 Armor=八卦阵语义已实现 | **维持泛型**，批③按 2 张落位 |
| 仁王盾 1 | 锁定：黑色杀对你无效 | 无 | 随批③实现（sha 目标防具检查，小改动） |
| 青釭剑 | 锁定：无视目标防具 | 仅射程 | 逐把特效批（先落射程+防具忽略门控） |
| 麒麟弓 | 杀伤害时可弃目标一匹马 | 仅射程 5 | 逐把特效批 |
| 寒冰剑 | 造成伤害时改为弃目标两张牌 | 仅射程 2 | 逐把特效批 |
| 诸葛连弩 | 出杀不限次 | 仅射程 1 | 逐把特效批（sha_limit 豁免） |
| 贯石斧 | 杀被闪后可弃两张牌强制伤害 | 仅射程 3 | 逐把特效批 |
| 青龙偃月刀 | 杀被闪后可再出一张杀 | 仅射程 3 | 逐把特效批 |
| 丈八蛇矛 | 两张手牌可当杀 | 仅射程 3 | 逐把特效批（view_as 转化） |
| 雌雄双股剑 | 对异性目标出杀：目标弃手牌或令你摸牌 | 仅射程 2 | 逐把特效批（性别分支） |
| 方天画戟 | 最后手牌杀可指定至多三个目标 | 仅射程 4 | **批③-b⑨ 单独立项（2026-10-07 评估定案，见下方「方天画戟立项评估」）** |

**定案**：马与八卦阵维持泛型+数量对齐（零语义债）；仁王盾随批③实现；
武器 9 把**逐把特效分批实现、每把落地才落位声明表**（未实现前不入牌堆
——避免错误语义进牌堆），建议由简到繁：青釭 → 麒麟 → 寒冰 → 连弩 →
贯石斧 → 青龙 → 丈八 → 雌雄 → 方天。批③相应拆分：③-a 无语义债部分
（马 6/八卦 2 + 基本牌与锦囊数量对齐，一次版本递增+重录）；③-b 武器
逐把落地各 +1~2 张。

**规则 v62（2026-10-07，缺牌种批①-1）**：无中生有（♥7/8/9/11，使用后
摸 2）与桃园结义（♥1，所有其他存活角色回复 1 点体力，满血跳过）落位
声明表（80→85 张，inventory 同步），锦囊均经无懈窗口（Dismantle 同款
包装）；14 golden 重录（牌堆构成变化、行为语义无变化，重录依据=确认
语义后更新流程）；种子敏感级联三处实跑复核修复（洛神判定首张注入、
传导测试无懈窗口排空、身份局终局阵营快照更新）。

**规则 v63（2026-10-07，缺牌种批①-2）**：五谷丰登（♥3/♥4，亮存活角色
数的牌、从使用者起按座位序逐人择一；机器快照含牌池与剩余择牌队列）与
借刀杀人（♣12/♣13，恰两名目标[武器持有者,另一角色]：持有者装备区有
武器且后者在其攻击范围内；持有者选出杀或交械——**出杀登记差异为直接
伤害结算无出闪窗口**，上游 chooseToUse 完整通道未建模；交械含双方
武器技能撤销与旧武器替换）。声明表 85→89 张；14 golden 重录；种子
敏感级联四处实跑复核（身份局阵营、倾国测试八卦阵判定置黑、auth_archive
鬼道黑杀注入与推进循环带应答）。**缺牌种清零**；补齐完成前仍非上游
全量对齐（数量差与装备命名化见上表）。

**规则 v64（2026-10-07，批③-a 数量对齐 + 仁王盾）**：非武器段数量对齐
上游口径——杀 30/闪 15/桃 8/过拆 6/顺牵 5/南蛮 3/决斗 3/乐 3/八卦阵
（Armor）2/-马 2/+马 4（花色点数沿用原生循环，逐张花色对齐另批）；
**仁王盾**新牌种（编码 94，equip2 防具）：锁定黑色【杀】无效（先于八卦
判定，红杀照常）。声明表 89→**114 张**（=上游 108 非武器段 + 原生扩充
13 + 测试武器 3）；14 golden 重录；种子敏感级联八处实跑复核修复（马牌
差分注入化、武器按名精确取、身份局阵营、倾国八卦置黑、auth_archive 鬼
道注入、急援重构自救场景、裸衣决斗排空、cardpile 按牌种挑扩展牌）。
批③-b 武器逐把落地后再增 9 张。

**规则 v65（2026-10-07，批③-b 第一把）**：青釭剑（♠6，命名武器牌种
编码 95，射程 2）：锁定——来源装备青釭时无视目标防具（八卦阵判定与
仁王盾免疫双检查点穿透，ShaResolve 实现）。声明表 114→**115 张**；
14 golden 重录；identity 阵营快照更新（v65 双种子均主公方）。剩余八把
（麒麟→寒冰→连弩→贯石斧→青龙→丈八→雌雄→方天）逐把落地。

**规则 v66（2026-10-07，批③-b 第二把）**：麒麟弓（♥5，命名武器牌种
编码 96，射程 5——`named_weapon_range` 命名武器射程面接入 attack_range，
与泛型 Weapon 的 CardDef.range 并行）：杀造成伤害时来源可弃目标装备区
一张坐骑（可选可放弃；ShaResolve 两条伤害路径结算前挂询问，空应答=放弃）。
声明表 115→**116 张**；14 golden 重录。剩余七把（寒冰→连弩→贯石斧→
青龙→丈八→雌雄→方天）逐把落地。

**规则 v67（2026-10-07，批③-b 第三把）**：寒冰剑（♠2，命名武器牌种
编码 97，射程 2，named_weapon_range 登记为 2）：杀造成伤害时来源可防止
此伤害，改为弃置目标两张牌（依次手牌→装备，不足则全弃；**登记差异：
上游为目标牌自选次序，原生自动依次——已于 v75 消除，见下**；可选可放弃，min=0 空应答=放弃，
放弃则照常伤害）。声明表 116→**117 张**；14 golden 重录；国色测试推进
循环排空级联修复（v67 牌堆位移途中出现弃牌询问）。剩余六把（连弩→
贯石斧→青龙→丈八→雌雄→方天）逐把落地。

**规则 v68（2026-10-07，批③-b 第四把）**：诸葛连弩（♣1/♦1 ×2，命名
武器牌种编码 98，射程 1）：装备时出牌阶段使用【杀】无次数限制
（ShaLimit 检查的连弩装备豁免，与咆哮同口径）。声明表 117→**119 张**；
14 golden 重录。剩余五把（贯石斧→青龙→丈八→雌雄→方天）逐把落地。

**规则 v69（2026-10-07，批③-b 第五把）**：贯石斧（♦5，命名武器牌种
编码 99，射程 3）：杀被目标出闪闪避后，来源可弃置两张自己的牌（手牌+
装备，不含贯石斧本体）强制造成 1 点伤害（恰两张才发动，放弃无伤；
**登记差异：强伤不带裸衣/酒加成，固定 1 点**）。声明表 119→**120 张**；
14 golden 重录。剩余四把（青龙→丈八→雌雄→方天）逐把落地。

**规则 v70（2026-10-07，批③-b 第六把）**：青龙偃月刀（♠5，命名武器
牌种编码 100，射程 3）：杀被目标出闪闪避后（无双需求已满），来源可
立即对相同目标再使用一张手牌中的杀（可选可放弃，min=0 空应答=放弃；
新杀弃置后以全新状态重启杀结算，防具/仁王盾/闪/贯石斧/麒麟弓全流程
重走；每次被闪都可再追问，手牌杀尽自然终止）。声明表 120→**121 张**；
14 golden 重录。剩余三把（丈八→雌雄→方天）逐把落地。

**规则 v71（2026-10-07，批③-b 第七把）**：丈八蛇矛（♠12，命名武器
牌种编码 101，射程 3）：装备授予配套主动技「丈八蛇矛」（出牌阶段
发动）：选两张手牌 → 选射程内其他角色 → 次张弃为素材、首张作
declared-Sha 载荷进入杀结算（防具/贯石斧/麒麟弓/青龙全流程重走）。
声明表 121→**122 张**；14 golden 重录。剩余两把（雌雄→方天）逐把
落地。

**规则 v72（2026-10-07，批③-b 第八把）**：雌雄双股剑（♠2，命名武器
牌种编码 102，射程 2）：杀指定异性目标后（性别=主将槽武将性别、无槽
默认男，effective_gender 辅助），目标选择：弃一张手牌 或 令使用者摸
一张（可选不可放弃；**登记差异：原生弃目标手牌首张，上游目标自选——
已于 v75 消除，见下**；目标无手牌仅可令摸）。声明表 122→**123 张**；
14 golden 重录。**批③-b 八把全部落地。**

**规则 v73（2026-10-07，批③-b 第九把）**：方天画戟（♦5，命名武器
牌种编码 103，射程 4）：该杀为来源最后手牌时可指定 1..=3 个目标
（逐目标独立合法性：距离/存活/空城/去重）。ShaResolveMachine 目标
队列化：extra_targets 队列逐目标推进，每目标完整走 仁王盾→铁骑→八卦
→流离→护驾→无双双闪→贯石斧/麒麟弓/寒冰→伤害（逐目标状态入快照）；
青龙再杀询问优先于队列剩余目标（放弃后继续队列）。声明表 123→
**124 张**；14 golden 重录。**批③-b 九把全部落地。**

**规则 v74（2026-10-07，批③-c 存量花色点数对齐）**：对齐段 79 张（杀 30/
闪 15/桃 8/南蛮 3/决斗 3/顺牵 5/过拆 6/无懈 4/乐 3/闪电 2）由原生循环
花色改为上游 standard list **逐张落位**（花色+点数+数量三对齐；闪电 ♠1
与 ♥12 同名，后者命名 `shandian_safe` 防目录重名）。**上游口径下闪 15 张
与桃 8 张全红**（♥/♦），黑闪/黑桃自此不存在——测试布阵须按此取牌。
原生扩充段 25 张（±马 6/酒 2/雷杀 2/泛型武器 3/防具八卦 2/火杀 4/铁索 4/
万箭 1/兵粮 1）保留原生取值，登记为花色差异（万箭 ♥1 与上游同值）；
批①新牌种 9 张与批③命名装备 11 张此前已按上游落位。张数不变（124 张），
花色点数分布变化 + 版本 73→74 入 canonical_hash——14 golden 全部重录
（场景脚本中途断言全过，仅终态哈希变）。级联修复：
`ganning_qixi_black_card_as_dismantle` 取黑闪循环改为黑雷杀 ♠4。

**规则 v75（2026-10-08，逐技能语义对照批一：杀结算目标侧选择类）**：
消除两处既有登记差异，对齐上游 `card/standard.js`：
1. **雌雄双股剑**（v72 差异消除）：目标弃牌由「手牌首张」改为目标
   **自选**一张手牌（SelectCards 询问目标，可取消）——上游
   `chooseToDiscard({prompt: 弃置一张手牌，或令X摸一张牌})` 可取消语义
   = 空应答（取消）令使用者摸 1；无手牌直接令来源摸 1（上游
   `!hasCards("h")` 分支同）。
2. **寒冰剑**（v67 差异消除）：来源确认防止伤害后，弃牌由「自动依次
   手牌→装备」改为**目标自选**——`hanbing_pending`（剩余次数 2）逐次
   开目标 SelectCards 询问（手牌+装备，min=1 强制、不可取消），次数
   用尽或目标 he 面无牌可弃即止；上游为 `discardPlayerCard("he",
   forced)`×2（第二问前复查 hasDiscardableCards）。
ShaResolveMachine 新增 `hanbing_pending: u8`（快照/存档/哈希三处接线，Sha 变体尾部追加）；版本 74→75 入 canonical_hash——14 golden 全部重录
（无场景装备雌雄/寒冰，语义轨迹不变，纯版本位移）。回归更新：
`cixiong_opposite_gender_discard_or_draw`（自选指定牌/取消令摸/同性
不触发三路径）、`hanbing_prevents_damage_discards_two_cards`（目标先
弃装备再弃手牌的选序断言/放弃路径）。

**规则 v76（2026-10-08，逐技对照批二：流离弃牌自选+范围）**：对齐上游
`character/standard/skill.js` `liuli`（chooseCardTarget 一体询问）：
1. **弃牌自选**（选择类差异消除）：被转移者从「手牌首张」改为自选
   弃置一张牌——**手牌+装备**（上游 position "he"）；SelectCards
   询问目标（`liuli_discard_choose`，可取消=不发动）。
2. **转移范围**（同批消除「不限距离」）：候选=在**被转移者攻击范围内**
   （computed_distance ≤ attack_range，±马/武器射程照常折算）、非此杀
   使用者、非自己、且为该杀合法目标（空城不可）——上游 inRange +
   targetEnabled 同口径；第二段强制选一（`liuli_transfer`，min=1）。
3. 两段式实现（先选牌后选目标，语义等价上游一体询问）：ShaResolve
   `liuli_pending: bool` → `liuli_stage: u8`（1=弃牌挂起/2=目标挂起）
   + `liuli_card`（暂存待弃牌）；转移生效后弃置暂存牌（上游 content
   顺序——牌在被转移者区域）；取消路径=不发动照常出闪。快照/存档/
   哈希/active_ask 派生（两段各自派生）同步；青龙再杀重启与方天队列
   推进均复位。版本 75→76——14 golden 重录（无场景触及大乔，纯版本
   位移）。回归更新：`n08_liuli_transfers_sha_target`（两段流断言：
   选项=手牌+装备、候选=范围内合法目标、取消语义）；
   `p2_new_choice_points_per_submission_conservation` 适配两段应答。

**规则 v77（2026-10-08，逐技对照批三：失去体力原语）**：类级修复——
新增「失去体力」原语并消除 G1 四处「失去体力简化为伤害」登记差异：
1. **原语**：`EventData::LoseHp { player, amount }` + `EventKind::LoseHp`
   + `LogEvent::HpLost`（表现层 `HpLost` 文本/事件投影）。commit 语义
   （`commit_lose_hp`）：直接扣减体力——**不经护甲抵扣、无伤害前置
   窗口、无铁索传导、Damage 触发技（遗计/反馈类）不响应**（LoseHp 是
   独立事件种，天然不进伤害监听）；体力 ≤0 与伤害同口径进入濒死
   （source=None）——对齐上游 loseHp。接线：帧数据 enc/dec（tag 16
   表尾追加）、事件种 enc 18/dec 18（**顺修缺口**：dec_event_kind 此前
   缺 TurnOverBefore=16/TurnOvered=17 两臂）、canonical_hash（tag 22）、
   快照边界检查、presentation 双投影。近期存档兼容由规则版本门禁覆盖
   （v76→77 拒绝旧规则存档）。
2. **发射点替换（四处）**：面里金针（目标摸 1 后失去 1 点体力）、
   飞镖（取消分支）、瘟疫（无手牌分支）、天女散花（同色分支）——均由
   Damage 改 LoseHp。
3. **保留登记**：草船借箭/舌战群儒等「失去体力」与其他大差异（自选
   展示/多轮问答链）捆绑的条目不在本批（单点替换会半对齐）；随机类
   登记（瘟疫弃首张/乾坤镖/麻婆豆腐等「随机改确定性」）保留——引擎
   无熵源，属确定性近似策略，见覆盖清单行为差异表。
版本 76→77 入 canonical_hash——14 golden 重录（无场景触及四卡，纯
版本位移）。回归更新：`g1_shahai_gujian` 面里金针/飞镖断言改失去体力
口径并加「无 DamageDealt 日志/有 HpLost 日志」守卫、
`g1_shahai_gwent` 瘟疫同口径。

**「随机改确定性」族的定位（2026-10-19 补注，样本 98/101/108 等）**：本会话样本把
`乾坤镖`（随机改确定性）、`六骰格`（随机效果池简化）、`鲈鱼羹`（发现牌随机五选一 → 牌堆顶）等
归入「**随机→确定性**」族——该族**与下文 v78 的既有登记同源**：
上游 `Math.random()` 类**均匀随机**可由**种子 RNG**（`RuleRng::gen_range`，state+calls 入存档）
**语义等价消除**（**可复放**）⇒ **修法方向**是**改用种子 RNG**，而**不是**保留确定性取首张。
（即：这些条目的**正确收敛方式**已由 v78 给出。）

**规则 v78（2026-10-08，逐技对照批四：随机类逐项对照）**：勘察确认
引擎有种子 RNG（`RuleRng::gen_range` Lemire 无偏采样，state+calls 入
存档、洗牌既有消费方）——上游 `Math.random()` 类均匀随机可**语义等价
消除**（原生种子均匀=可复放），不必登记为近似。逐项对照
`杀海拾遗` 上游源码后的处置：

| 项 | 上游语义（源码位置） | 处置 |
|---|---|---|
| 瘟疫（无手牌分支已 v77 失去体力） | `target.randomDiscard("h")` | **v78 消除**：随机手牌 |
| 乾坤镖 | 每受害者 `he.randomGet()` | **v78 消除**：手牌+装备合集随机一张（he 空者跳过=changeTarget 同口径） |
| 麻婆豆腐 | `list.randomGet()` 随机敌 + `he.randomGet()` | **v78 消除**：随机有牌其他角色（敌我阵营面按原型惯例）随机一张 |
| 山崩术 | 敌装备合集 `randomGets(2)` 恰二才弃 | **v78 消除**：合集均匀无序对（i+j 越过法），恰二门控保留 |
| 破晓分支二 | 随机一张铜卡法术（破晓除外） | **v78 消除**：牌堆铜法术集合均匀抽取，空则摸 1 不变 |
| 玉衡 | 持黑桃敌 `list.randomGet()` + `randomGet()` | **v78 消除**：随机有黑桃手牌其他角色随机黑桃（升级机制仍登记暂缓） |
| 弹射之刃 | 敌-友-敌交替阵营链随机 | **保留登记**：阵营链结构未实现（原型无阵营面） |
| 六骰格/硫头阁 | 随机正/负面效果池 | **保留登记**：效果池未实现面 |
| 舌战群儒 | 多轮 Bool 问答链+随机牌 | **保留登记**（结构差异捆绑） |
| 草船借箭/金莲烛/卤玉羹 | 随机牌捆绑自选/发现机制 | **保留登记**（结构差异捆绑） |
| 暴雪药水 | `chooseToDiscard("h",2,true)` **目标自选两张** | **更正归类**（自选类）→ **v79 消除**：目标 SelectCards 强制两张（不足全选）后摸 1 |
| 天女散花比对牌 | 双方**自选**展示一张手牌 | **更正归类**（自选类）→ **v79 消除**：双段展示（ComparePlayerPick/TargetPick 逐目标队列），同色弃展示牌+目标失去 1 体力（v77 原语），异色无事；任一方无手牌该目标跳过 |
| 龙须钩 | 使用者自选目标装备区一张 | **v79 消除**：使用者 SelectCards 选目标装备（装备区明牌）；原登记「目标自选」系措辞不准，上游 `choosePlayerCard` 为使用者选 |

随机抽取实现口径：`ctx.core.rng.gen_range(候选数)` 均匀抽取；rng 流
入存档可复放，golden/恢复轨迹口径不变。版本 77→78 入 canonical_hash
——14 golden 重录（无场景触及六卡，纯版本位移）。回归适配：
破晓测试改集合成员断言（随机后不再唯一可指）；乾坤镖补 he 空守卫。
定向 `g1_shahai_gujian` 13/0、`g1_shahai_gwent` 6/0。

**规则 v79（2026-10-08，逐技对照批五：自选类遗留收尾）**：三卡消除，
逐技对照主线标准/杀海范围内登记清账：
1. **暴雪药水**：目标自选弃两张手牌（SelectCards 强制 min=max=min(2,手牌数)，
   `gw_baoxue_discard`；不足则全选，无手牌跳过弃牌仍摸 1=上游 step1
   draw 恒执行）——上游 `chooseToDiscard("h",2,true)`。
2. **天女散花**：双段自选展示——使用者选展示牌（暂存 `cards` 入快照）→
   目标选展示牌 → 同色使用者弃展示牌+目标失去 1 体力（v77 原语）、
   异色无事（展示不移动牌）；任一方无手牌该目标直接跳过（上游 step0
   finish 条件）；复用拼点双段 stage（ComparePlayerPick/CompareTargetPick）
   逐目标队列推进。
3. **龙须钩**：使用者自选目标装备区一张获得（上游 `choosePlayerCard
   ("e", target)`——装备区明牌由使用者选，原登记「目标自选」措辞更正）。
派生面：三询问（暴雪/天女散花两段/龙须钩）入 active_ask 派生表；创建
stage 按 behavior 特例（暴雪=SelectCard、天女散花=ComparePlayerPick、
龙须钩=SelectCard）。版本 78→79 入 canonical_hash——14 golden 重录
（无场景触及三卡，纯版本位移）。回归适配：天女散花测试改双段应答
（并覆盖「使用者无手牌后续目标跳过」分支、展示牌保留断言）、龙须钩
加选牌询问断言、暴雪测试验证自选语义（选 s2/s3 保留 s1）。

**规则 v80（2026-10-08，P2/N08 模式入口批一：对决 versus 2v2 基线）**：
上游 `mode/versus.js`（9k 行 UI/流程脚本）的引擎可接核心=队伍表+胜负+
奖惩，2v2 基线接入：
1. `GameMode::Versus { teams: Vec<Vec<PlayerId>> }`（两队互斥、并集=
   全部座位；存档 tag 3、canonical_hash 模式段 tag 4、快照结构检查=
   队数 2/成员互斥/全覆盖）+ `Winner::Versus(usize)`（存档 tag 5、
   哈希 tag 6——Traitor 已占 5，注意三套 tag 编号独立）+ 
   `versus_teams()/versus_team_of()` 辅助 + `Engine::new_versus` 构造器。
2. DeathMachine 奖惩（先于胜负）：杀死敌方摸三张；杀死友方弃掉自己
   全部手牌与装备（同主公杀忠臣路径）。胜负：任一队伍全灭 → 对方
   队伍胜（`Winner::Versus(对方下标)`）。
3. **登记未建模**：替补武将上场、4v4 主公制/号位置换、间断/四国
   变体（UI/流程面）。
版本 79→80 入 canonical_hash——14 golden 重录（标准场景无 versus 局，
纯版本位移）。回归：`versus_mode_team_victory_and_rewards`（杀敌摸三/
杀友弃全部/存档往返/全灭终局）。

**规则 v81（2026-10-08，P2/N08 模式入口批二：斗地主）**：上游
`mode/doudizhu.js`（4k 行 UI/流程）引擎可接核心=身份组成+胜负，复用
versus 接线模式接入：
1. `GameMode::Doudizhu { lord: PlayerId }`（3 人局 zhu+fan+fan，地主
   1 名；存档 tag 4、canonical_hash 模式段 tag 5）+ `Winner::Doudizhu
   { lord_won: bool }`（存档 tag 6、哈希 tag 7）+ `doudizhu_lord()`
   辅助 + `Engine::new_doudizhu` 构造器；快照检查=地主座位越界、
   outcome-模式一致性。
2. DeathMachine 胜负臂：地主死亡 → 农民胜；农民全灭 → 地主胜。
   **无击杀奖惩**（上游 dieAfter 仅 dieidentity 展示+checkResult 判负）。
3. **登记未建模**：特殊开局摸牌变体（binglin 地主 5/农民 4、按 infoHp
   加成）、开黑/欢乐变体 UI 流程。
版本 80→81 入 canonical_hash——14 golden 重录（标准场景无斗地主局，
纯版本位移）。回归：`doudizhu_mode_victory_and_roundtrip`（无奖惩断言/
存档往返/地主死农民胜/农民全灭地主胜）。

### Boss 模式立项评估（P2/N08 模式入口批三，2026-10-08 评估定案）

上游 `mode/boss.js`（11.5k 行，模式入口中体量最大）实测结构：

1. **胜负本体与 doudizhu 同构**（单体对抗）：boss 持 zhu 身份
   （`boss.setIdentity("zhu")`），`checkResult` = boss 死亡 → 挑战者
   胜、挑战者全灭 → boss 胜（`game.over(!game.boss.isAlive())`）。
2. **招牌机制=重整复活回环**（bossPhaseLoop 自定义回合循环）：死者
   回合开始累加 `boss_chongzheng` 计数、每轮 hp+1（或手牌 <4 摸 1）、
   计数达 `bossinfo.chongzheng` 阈值 `player.revive` 复活；boss 侧
   `chongzheng` 标志跳过一回合。此为引擎新面（回合循环重写+复活计数
   状态入快照），非四件套可复用。
3. **addFellow 中途入队**：boss 助战（`fellow.side=true`、identity
   zhong、directgain 4）——座位表运行时扩张，存档/座位扫描面需扩。
4. **内容面**：boss 数值（maxHp 25–30）与专属技能包（boss_ 引用
   1506 处）——属来源扩展内容，按 G1 转换路径，不入模式结构批。

**定案**：不做「无重整复活的 Boss 贴皮接入」（boss 模式的定义性机制
即重整回环，缺失即为半成品——沿「未实现不落位」先例）。拆批建议：
批三-a Boss 基线=单体对抗接线（复用 Doudizhu 模式语义，映射登记）+
重整复活回环（PhaseLoop 重写+复活计数快照/存档/哈希）；批三-b=
addFellow 中途入队（座位扩张面）。立项待排期，未实现前 boss 不设
GameMode 入口。

**批三-a 执行（规则 v82，2026-10-08）**：
1. `GameMode::Boss { boss: PlayerId, chongzheng: u8 }`（存档 tag 5、
   canonical_hash 模式段 tag 6）+ `Winner::Boss { boss_won: bool }`
   （存档 tag 7、哈希 tag 8）+ `boss_mode()/new_boss` + `PlayerState
   .boss_chongzheng: u8` 计数字段（PlayerState enc/dec/哈希接线）。
2. **重整复活回环**：`commit_turn_ended` Boss 分支——死亡座位不跳过，
   轮到其回合执行重整 tick（计数 +1；hp<上限则 +1，否则手牌 <4 摸 1；
   连续死亡座位逐个 tick 后落在下一存活座位）；计数达阈值复活
   （alive 恢复、计数清零）；`chongzheng=0` 为无复活模式（上游
   bossinfo.chongzheng 未设同口径）。新日志 BossChongzhengTick/
   BossRevived（表现层映射 Recovered）。
3. 胜负臂：boss 死亡 → 挑战者胜；挑战者全灭 → boss 胜。无击杀奖惩。
4. **登记未建模**：boss 专属数值/技能包（内容面，G1）、
   bossinfo.loopFirst/loopType 变体。
版本 81→82 入 canonical_hash——14 golden 重录（标准场景无 Boss 局，
纯版本位移）。回归：`boss_mode_chongzheng_revival_and_victory`
（tick 计数/回血、阈值复活清零、双向往返、双终局）。

**批三-b 执行（规则 v84，2026-10-08）**：addFellow 中途入队——
`GameMode::Boss` 增 `fellows: Vec<PlayerId>`（存档/哈希/快照座位界
接线）；`debug_add_fellow(hp)` 队尾追加新玩家（PlayerState 直接入
players 向量，直授 4 张手牌=上游 directgain(4)，登记入 fellows）；
胜负臂挑战者判定排除 fellow（fellow 属 boss 方）。**登记差异**：
入队位置固定队尾（上游 `game.addFellow(position,…)` 可按位置插入；
原生玩家 id=下标、插入会移动既有座位号，不可接受）。
版本 83→84 入 canonical_hash——14 golden 重录（标准场景无 fellow，
纯版本位移）。回归：`boss_mode_add_fellow_seat_expansion`（入队座位
=队尾/直授 4 牌/座位表扩张/fellows 登记/存档往返/fellow 不算挑战者
→boss 胜；濒死桃强制应答守卫）。

**规则 v83（2026-10-08，P2/N08 模式入口批四：KOF 引擎层）**：示例/
测试层的 KOF 会话语义升入引擎：
1. `GameMode::Kof { pools: [Vec<u16>; 2], wins: [u32; 2], games: u32 }`
   ——双方将池（GENERAL 下标）、比分、局数随引擎状态存档（mode tag 6）
   /哈希（模式段 tag 7）；快照检查=2 人局 + outcome-模式一致性
   （Winner::Player）。
2. 单局终局自动推进会话（DeathMachine Kof 臂）：唯一存活者胜
   （Winner::Player）→ 败者队首淘汰、胜者队首轮转（移至队尾）、
   比分/局数 +1——与示例/测试层内联实现同构（KOF1 会话行的引擎态
   版本）；无存活者不推进。
3. `Engine::new_kof(seed, pools, wins, games)` 构造器（2 人局）。
4. **边界登记**：多局连续会话循环（终局→自动开局下一局）仍在示例/
   宿主层（引擎终局即停，会话推进结果随终局态可读）；单局内将领
   授予仍按装配/授予原语。
版本 82→83 入 canonical_hash——14 golden 重录（标准场景无 KOF 局，
纯版本位移）。回归：`kof_engine_mode_session_advance`（两局推进：
队首淘汰/队首轮转/比分局数、存档往返、败者池清空出线形态）。

v85–v95 条目于 2026-10-10 由进度主稿批次流水原样移入（此前本页缺这 11 个版本的条目；原文见[进度历史](native-progress-history.md#p2--n08n11-规则对齐批次流水2026-10-07--10-08)）。

**规则 v85（2026-10-08，U05 swd 首切片）**：护符 hufu（♠1/♣1，基本牌）——`CardKind::Hufu` 编码 104（archive/a0 同步）、is_tao_like 双镜像限自身（responder==dying=上游 savable 同口径，重整 tick 无涉）、转换包 `shahai-swd-upstream-noname`（2 张）真实宿主生命周期验收；他人濒死负例固化（持符者被问但选项不含护符）；14 golden 重录（目录变化）。证据见[swd 首切片](../operations/evidence/2026-10-08-u05-swd-slice/README.md)。

**规则 v86（2026-10-08，U05 swd 第二批）**：流星火雨——`CardKind::Liuxinghuoyu` 编码 105（archive/a0 同步）+ 宿主行为 `SwdLiuxinghuoyuCard`（tag 79，SelectCard 两段：目标自选弃两张 he（min=max=2 可取消）/取消或不足两张受 1 点火伤（上游 step1 directfalse||!bool → damage("fire") 同口径）；无懈窗口包装；转换包增至 5 张；弃牌落 discard 修复（首跑发现 to_discard 缺失致 card_missing_from_all_zones）。14 golden 重录（纯版本位移）；回归 `swd_liuxinghuoyu_discard2_or_fire`（弃二无伤/不足两张火伤/往返）。证据见[swd 第二批](../operations/evidence/2026-10-08-u05-swd-batch2/README.md)。

**规则 v87（2026-10-08，U05 swd 第三批）**：草药 ♣3/♣4/♣5——`CardKind::Caoyao` 编码 106（archive/a0/ALL 同步）+ PlayCard 门控（恰 1 目标、存活且 hp<maxHp、computed_distance≤1=上游 range.global 1）+ 内联回复结算（Recover 事件走 Tao 同路）；转换包增至 8 张。14 golden 重录（纯版本位移）；回归 `swd_caoyao_heals_wounded_in_range`（距离内受伤回复/满血拒绝）。证据见[swd 第三批](../operations/evidence/2026-10-08-u05-swd-batch3/README.md)。

**规则 v88（2026-10-08，U05 swd 第四批）**：赤龙牙 ♦3——`CardKind::Chilongya` 编码 107（equip1 射程 2、archive/a0/ALL 同步）+ commit_damage 火伤增幅（来源装备赤龙牙时火属性伤害 +1；非火照常、铁索传导的二次结算照常 +1=上游 damageBegin num++ 每事件语义）；转换包增至 9 张。14 golden 重录（纯版本位移）；回归 `swd_chilongya_amplifies_fire_damage`（火伤 1→2/非火照常/射程 2）。证据见[swd 第四批](../operations/evidence/2026-10-08-u05-swd-batch4/README.md)。

**规则 v89（2026-10-08，U05 swd 第五批）**：天仙酒 ♥9/♦2——`CardKind::Tianxianjiu` 编码 108（archive/a0/ALL 同步）；酒的 swd 变体：intrinsic/PlayCard/is_tao_like 三处与 Jiu 同臂（濒死自救+下张杀 +1），登记差异=与酒共享每回合计数（上游独立 usable:1）；转换包增至 11 张。14 golden 重录（纯版本位移）；回归 `swd_tianxianjiu_ameisha_and_self_rescue`（酒增益杀 2 伤/共享计数 CardLimit）。证据见[swd 第五批](../operations/evidence/2026-10-08-u05-swd-batch5/README.md)。

**规则 v90（2026-10-08，U05 swd 第六批）**：吸血鬼指环 ♠5——`CardKind::Xixueguizhihuan` 编码 109（equip5 宝物、archive/a0/ALL 同步）+ commit_damage 杀伤害后吸血（来源存活且已受伤且伤害牌为【杀】→ 回复等量体力，上游 xixue damageEnd 同口径；非杀/满血不触发）；转换包增至 12 张。14 golden 重录（纯版本位移）；回归 `swd_xixue_lifesteal_on_sha_damage`（吸血回复/满血不触发/杀计数重置脚手架）。证据见[swd 第六批](../operations/evidence/2026-10-08-u05-swd-batch6/README.md)。

**规则 v91（2026-10-08，U05 swd 第七批）**：归有界 ♠3/♣9——`CardKind::Guiyoujie` 编码 110（他人延时锦囊、archive/a0/ALL 同步）+ PlaceDelayed 派发（无懈窗口包装）+ 判定结算臂（黑色失去 1 点体力+he 随机弃一张——RNG 均匀抽取同批四口径；红色无事；PlayCard 门控=恰 1 其他存活目标+其判定区无同名延时）。14 golden 重录（纯版本位移）；回归 `swd_guiyoujie_delayed_black_loss`（黑判失体力+随机弃牌/往返）。证据见[swd 第七批](../operations/evidence/2026-10-08-u05-swd-batch7/README.md)。

**规则 v92（2026-10-08，U05 swd 第八批）**：潜偷面具 ♠6——`CardKind::Shentoumianju` 编码 111（equip5 宝物、archive/a0/ALL 同步）+ 装备主动技 `SwdShentouEquipSkill`（Act 问代价弃一张手牌→判定：梅花失败无事、非梅花获得手牌更多目标一张随机手牌；上游 shentou chooseCard cost + judge 梅花 + randomGet 同口径；快照派生 SelectCard 询问重 derivation）；装备注入同步授予技能实例（快照校验 EquipSkillMismatch 首跑捕获）。14 golden 重录（纯版本位移）；回归 `swd_shentou_active_steal_with_club_judge`（守恒断言/代价弃置）。证据见[swd 第八批](../operations/evidence/2026-10-08-u05-swd-batch8/README.md)。

**规则 v93（2026-10-08，U05 swd 第九批）**：星君眼+青龙令珠 双 equip5 宝物——`CardKind::Xingjunyan` 编码 112（杀伤 +1 锁定，damageBegin card==sha 同赤龙牙口径）+`CardKind::Qinglonglingzhu` 编码 113（属性伤害后获得目标一张 he 牌——damageEnd hasNature+alive，RNG 代抽登记差异）；两钩子均在 commit_damage（星君眼 hp 扣减前、青龙令珠 DamageDealt 日志后）；装备注入全区域清除（首跑 card_in_multiple_zones 捕获）。14 golden 重录（纯版本位移）；回归 `swd_xingjunyan_amp_and_qinglonglingzhu_steal`（星君眼 6→4/青龙令珠目标-1 使用者+1）。证据见[swd 第九批](../operations/evidence/2026-10-08-u05-swd-batch9/README.md)。

**规则 v94（2026-10-08，U05 swd 第十批）**：贴房绅视——`CardKind::Zhufangshenshi` 编码 114（他人锦囊、archive/a0/ALL 同步）+ `TurnState.zhufangshenshi_target: Option<PlayerId>`（存档 opt_u8/哈希 match/快照豁免）；sha 距离校验豁免（zfs_free 跳过 computed_distance 比较=上游 targetInRange mod 同口径）；使用后摸一张牌（上游 g_zhufangshenshi draw）；登记差异=无懈不可触发（上游 wuxie AI 权重 0 实战不戒，原生简化为直接结算）。14 golden 重录（纯版本位移）；回归 `swd_zhufangshenshi_distance_waiver`（绅视+标记目标出杀/存档往返）。证据见[swd 第十批](../operations/evidence/2026-10-08-u05-swd-batch10/README.md)。

**规则 v95（2026-10-08，U05 swd 第十一批）**：风雪刃 ♣1——`CardKind::Fengxueren` 编码 115（equip1 射程 2、archive/a0/ALL 同步）+ ShaResolve `asking_fengxueren`（shaHit 后 Confirm 询问：确认→防止伤害+目标翻面（TurnOvered 事件）+摸 1；放弃→照常伤害；目标已翻面不再询问=上游 filter !isTurnedOver 同口径）；快照/存档/哈希/derive 全接线。14 golden 重录（纯版本位移）；回归 `swd_fengxueren_flip_or_damage`（翻面+摸牌/放弃伤害/翻面目标不再询问）。证据见[swd 第十一批](../operations/evidence/2026-10-08-u05-swd-batch11/README.md)。

**规则 v96（2026-10-08，U05 swd 第十二批：御夫录）**：swd 转换批 +
潜偷限一次补执行：
1. `CardKind::Yufulu`（编码 116，equip5 宝物）+ `HostBehavior::SwdYufuluEquipSkill`
   （tag 81）三段询问机「投掷」：出牌阶段限一次取一张武器（手牌/装备
   区，`discard:false`）交给一名其他角色——其受 1 点普通伤害并获得该
   武器（上游 content `damage(); gain(cards)` 同序）。**登记差异**：无
   （语义等价；AI check() 不迁移同各批口径）。武器在目标应答后才离开
   原区域（上游 `prepare:"give"` 同口径：content 执行时才移动）。
2. **潜偷 `usable:1` 补执行**（v92 批遗漏）：`host_skill_def` 对
   潜偷/御夫录设 `once_per_turn`（上游两者均 usable 1；once_used 于
   拥有者回合开始重置为既有路径）。
3. **源码校正**（对上游 swd/translate.js:314 与 list.js:31）：卡名
   「御夫录」（此前工作记录误记「伏符录」；键名 yufulu 本就正确）、
   花色点数 ♠6（此前误记 ♠5）。
版本 95→96 入 canonical_hash——14 golden 重录（御夫录/潜偷均不在
标准场景，纯版本位移）。回归：`swd_yufulu_give_weapon_and_damage`
（无武器拒绝/两段询问×存档往返/1 伤+给牌/SkillAlreadyUsed）。
证据见 [swd 第十二批](../operations/evidence/2026-10-08-u05-swd-batch12/README.md)。

**规则 v97（2026-10-08，U05 swd 第十三批：羊皮卷）**：`CardKind::Yangpijuan`
（编码 117，♠9/♣7 trick toself）+ `HostBehavior::SwdYangpijuanCard`
（tag 82）三段询问机：选类别（0=基本/1=锦囊/2=装备，上游 chooseControl
顺序）→ 牌堆部分 Fisher-Yates 随机亮该类别 ≤3 张（种子 RNG）→ 选一张
从牌堆入手。**登记差异**：上游 `game.createCard` 凭空造牌不占牌堆实体，
原生无自由牌 id——发现牌从牌堆实体取出，亮出的 3 张择定前留在牌堆；
类别池按目录牌种枚举（基本 9 种/通用锦囊 26 种/装备=equip_slot 非空），
gwent/炉石/古剑机关/食物/欢乐等专用牌种不入池；牌堆无该类别牌则空发；
上游 AI 推荐/chooseButton ai 不迁移（玩家询问代替，各批同口径）。
版本 96→97 入 canonical_hash——14 golden 重录（羊皮卷不在标准场景，
纯版本位移）。回归：`swd_yangpijuan_discover_from_deck`（无目标门控/
类别询问×往返/发现选项⊆牌堆且类别一致/入手守恒）。
证据见 [swd 第十三批](../operations/evidence/2026-10-08-u05-swd-batch13/README.md)。

**规则 v98（2026-10-08，U05 swd 第十四批：带火风轮+霞云彩绫距离对）**：
命名距离修饰两张（无机器，纯数据卡）——带火风轮（编码 118，♥1，
equip4→+1马栏）：持有者算到他人 -2（globalFrom）、他人算到持有者 -1
（globalTo）；霞云彩绫（编码 119，♦2，equip3→-1马栏）：持有者算到他人
+1、他人算到持有者 +2。`computed_distance` 以命名牌种计数并入
plus/minus（饱和下限 1 为既有路径；与马术/义从/基础马叠加）。
**登记差异**：cardnature "fire" 仅上游主题标记不入结算；不与基础马做
同栏跨 kind 替换（批②装备命名化取舍定案沿用）。
版本 97→98 入 canonical_hash——14 golden 重录（两卡不在标准场景，纯
版本位移）。回归：`swd_daihuo_xiayun_distance_pair`（6 人局对座双向/
叠加/邻座下限饱和/装配路径）。
证据见 [swd 第十四批](../operations/evidence/2026-10-08-u05-swd-batch14/README.md)。

**规则 v99（2026-10-08，U05 swd 第十五批：蓝格怪衣「龙帆」）**：
`CardKind::Langeguaiyi`（编码 120，♦1，equip5 宝物）+
`HostBehavior::SwdLangeguaiyiEquipSkill`（tag 83）单段判定机：出牌
阶段限一次（once_per_turn，onLose 清 getStat 为上游 usable 簿记等价），
判定按花色——红桃回 1 体力（recover 饱和同钳制）/方片摸 1/梅花随机
敌人随机弃一张（候选=其他存活且手牌∪装备区非空，上游 he filter 同
口径）/黑桃无事。**登记差异**：上游 judge 花色提示函数为 AI 排序不
迁移；`randomDiscard()` 默认弃牌区归档不可核验（现有调用点均显式
"h"）——手牌优先、无手牌退随机装备；**源码校正**：技能名「龙帆」
（translate.js:308），此前工作记录误作「乱番」。
版本 98→99 入 canonical_hash——14 golden 重录（蓝格怪衣不在标准场景，
纯版本位移）。回归：`swd_langeguaiyi_longfan_suit_branches`（六子局：
红桃回血+限一次/满血饱和/方片摸 1/梅花弃 1/无候选/黑桃三不变）。
证据见 [swd 第十五批](../operations/evidence/2026-10-08-u05-swd-batch15/README.md)。

**规则 v100（2026-10-08，U05 mtg 首切片：龙颅山峰/land 地类）**：
land 类基建——`CardKind::Longlushanfeng`（编码 121，♣10）+
`GameStateCore.current_land`（换地覆盖、跨回合持续，入存档/哈希）+
`TurnState.land_used/land_sha_used`（本回合限一张地；有地时第二张杀
放行并标记）。地机器 `HostBehavior::MtgLongshanfengLand`（tag 84）：
Act 换地 → 问获得杀/闪（PickTag，上游 chooseControl 顺序）→ 牌堆取该
类牌入手。**登记差异**：上游 createCard 凭空造牌→原生牌堆实体取（取
牌堆内首张该类，确定性）；地图效果全局（addGlobalSkill 同口径）；地
非锦囊无无懈窗口；专用 LandChanged 日志暂缓（状态可观察）。**来源
勘误**：台账所记 ow/xianjian 为 `character/` 武将子目录，非卡牌子包。
版本 99→100 入 canonical_hash——14 golden 重录（mtg 不在标准场景，纯
版本位移）。回归：`mtg_longshanfeng_land_flow`（换地/获得杀/本回合限
一张/杀上限+1/跨回合持续）+ `mtg_real_host_full_lifecycle`（六步）。
证据见 [mtg 首切片](../operations/evidence/2026-10-08-u05-mtg-slice/README.md)。

**规则 v101（2026-10-08，U05 mtg 第二批：海岛）**：`CardKind::Haidao`
（编码 122，♦5 land）+ `HostBehavior::MtgHaidaoLand`（tag 85）：换地后
满血直接 +1 护甲（不受伤转换不触发）；受伤问弃一张（he=手牌∪装备区，
可取消）——弃则回 1/取消得 1 护甲。**地图效果落在 `change_hujia`
原语**：current_land=海岛时，任何玩家获护甲若已受伤改为回复等量体力
（锁定、全局，上游 addGlobalSkill 同口径；失去护甲不触发）。
**登记差异**：上游 chooseToDiscard 取消经 result.bool，原生空选择表达；
`_temp_mtg_haidao` 抑制以取消路线直写字段（路径级）实现。
版本 100→101 入 canonical_hash——14 golden 重录（海岛不在标准场景，
纯版本位移）。回归：`mtg_haidao_land_effect`（满血/弃牌回血×往返/
取消得甲/换地覆盖跨回合+杀加成消失/双包盾牌格挡全局转换）。
证据见 [mtg 第二批](../operations/evidence/2026-10-08-u05-mtg-batch2/README.md)。

**规则 v102（2026-10-08，U05 mtg 第三批：水没墓穴）**：`CardKind::
Shuimomuxue`（编码 123，♠11 land）+ `HostBehavior::MtgShuimomuxueLand`
（tag 86）：换地后 he 有牌可弃一张摸二（可取消，无牌空发）。地图效果
锁定两钩：手牌上限 -1（弃牌阶段 limit 计算处，倾盆大雨/妄尊印同款）
+ `TurnState.discard_phase_discarded` 计数（弃牌阶段开始清零、上限弃
牌机器累加、PhaseEnd 离开弃牌阶段时若 >0 摸一并清零；入存档/哈希）。
**登记差异**：计数面仅弃牌阶段上限弃牌机器（与上游 event.cards 同为
该阶段弃牌集合）。版本 101→102 入 canonical_hash——14 golden 重录
（水没墓穴不在标准场景，纯版本位移）。回归：
`mtg_shuimomuxue_land_effect`（弃一摸二×往返/取消/空发/上限-1 触发弃
牌+弃后摸一/换地覆盖海岛转换消失）。
证据见 [mtg 第三批](../operations/evidence/2026-10-08-u05-mtg-batch3/README.md)。

**规则 v103（2026-10-08，U05 mtg 第四批：废墟）**：`CardKind::Feixu`
（编码 124，♣8 land）+ `HostBehavior::MtgFeixuLand`（tag 87）：换地后
从弃牌堆随机亮 ≤3 张三选一（选定才离弃堆，空弃堆空发）。地图效果落
`commit_phase` Judge begin：current_land=废墟且弃堆非空 → RNG 随机一
张弃堆→牌堆顶（锁定、全局，上游 phaseBegin silent 同口径）。
**登记差异**：discoverCard 展示数量以原生 ≤3 为准（羊皮卷同口径）。
版本 102→103 入 canonical_hash——14 golden 重录（废墟不在标准场景，
纯版本位移）。回归：`mtg_feixu_land_effect`（发现三选一×往返/选定离
弃堆/地图效果跨回合观测/空弃堆空发）。
证据见 [mtg 第四批](../operations/evidence/2026-10-08-u05-mtg-batch4/README.md)。

**规则 v104（2026-10-08，U05 mtg 第五批：依夏兰/首例地附主动技）**：
`CardKind::Yixialan`（编码 125，♣12 land）+ `MtgYixialanLand`（tag 88）
两段机（换地 → PickTag 类型 0=基本/1=锦囊 → 牌堆随机该类一张入手，
`yangpijuan_class` 类别池复用）+ **地附主动技基建 `sync_land_skill`**：
五台地机器 Act 统一调用——撤销旧地附技、授予新地附技（当前仅依夏兰
附 `MtgYixialanRecast` tag 89 重铸技：出牌阶段限一次，基本牌放回牌堆
→获得牌堆一张锦囊/无则摸一补偿；activate 守卫读 current_land；once_
per_turn）。**登记差异**：createCard→牌堆实体取；canRecast 引擎内置
不可核验→原生手牌基本牌；重铸放回牌堆顶；补偿分支回归未覆盖（构造
无锦囊牌堆不可靠）。版本 103→104 入 canonical_hash——14 golden 重录
（依夏兰不在标准场景，纯版本位移）。回归：
`mtg_yixialan_land_and_recast`（无地无实例/类型获得×往返/重铸主路径/
限一次/换地撤销）。
证据见 [mtg 第五批](../operations/evidence/2026-10-08-u05-mtg-batch5/README.md)。

**规则 v105（2026-10-08，U05 mtg 第六批：沙漠绿洲）**：`CardKind::
Shamolvzhou`（编码 126，♥6 land）+ `MtgShamolvzhouLand`（tag 90）：换地
后牌堆基本牌随机亮 ≤3 张三选一（选定才离牌堆，无基本牌空发）。地图
效果锁定：current_land=沙漠绿洲时**基本牌不计入手牌上限**——弃牌阶段
需弃数按非基本手牌数计 + DiscardToLimit 选项排除基本牌（上游
ignoredHandcard basic 同口径）。**登记差异**：类口径以 yangpijuan_class
为准（杀/闪/桃/酒/swd 基本牌均基本类）；上游弃牌选择集是否排除基本
牌归档不可完全核验——原生排除（不计入即不要求弃）。
版本 104→105 入 canonical_hash——14 golden 重录（沙漠绿洲不在标准场
景，纯版本位移）。回归：`mtg_shamolvzhou_land_effect`（发现×往返/选
项全基本类/地图效果非基本计数+选项排除）。
证据见 [mtg 第六批](../operations/evidence/2026-10-08-u05-mtg-batch6/README.md)。

**规则 v106（2026-10-08，U05 mtg 第七批：林中尖塔）**：`CardKind::
Linzhongjianta`（编码 127，♥6 镜像组 land）+ `MtgLinzhongjiantaLand`
（tag 91）：换地后牌堆武器亮 ≤3 三选一，选定**先入手再 emit Equipped**
（commit_equip 从手牌取牌的原生适配；完整装配含同 kind 替换/装备技授
予）。**地附技第二例**：`sync_land_skill` 映射表化（依夏兰→重铸、林
中尖塔→`MtgLinzhongjiantaStrike` tag 92 塔击）——装备区有武器时出牌
阶段限一次将一张基本牌当杀使用（`declared_kinds`+`card_use_as` 丈八
TargetRead 同构，素材留手牌由杀结算消费，出闪/贯石斧/麒麟弓全流程重
走）。**登记差异**：塔击上游 chooseToUse 响应+出牌双入口——原生仅出
牌入口（同丈八登记）；"nogain" 直接装备以入手→Equipped 两步适配。
版本 105→106 入 canonical_hash——14 golden 重录（林中尖塔不在标准场
景，纯版本位移）。回归：`mtg_linzhongjianta_land_and_strike`（武器发
现装备/塔击杀结算掉血/限一次/换地撤销）。
证据见 [mtg 第七批](../operations/evidence/2026-10-08-u05-mtg-batch7/README.md)。

**规则 v107（2026-10-08，U05 mtg 第八批：冰河要塞）**：`CardKind::
Bingheyaosai`（编码 128，♦5 镜像组 land）+ `MtgBingheyaosaiLand`（tag
93）：换地后摸 2 → 弃两张（he=手牌∪装备，min2 max2 强制不可取消）。
地图效果落 commit_card_used Sha 臂：current_land=冰河要塞时，使用者
手牌∪装备非空 → RNG 代抽弃一张（手牌/装备各半随机；装备离装撤销技）。
**登记差异**：上游 chooseToDiscard 玩家自选+可取消——原生 RNG 代抽不
可取消（青龙令珠代抽先例同口径；info「需弃置」与实现可取消的出入以
实现为准）。版本 106→107 入 canonical_hash——14 golden 重录（冰河要
塞不在标准场景，纯版本位移）。回归：`mtg_bingheyaosai_land_effect`
（摸二弃二×往返/用杀代抽弃一/换地覆盖后效果消失）。
证据见 [mtg 第八批](../operations/evidence/2026-10-08-u05-mtg-batch8/README.md)。

**规则 v108（2026-10-08，U05 mtg 第九批：幽林）**：`CardKind::Youlin`
（编码 129，♣10 land）+ `MtgYoulinLand`（tag 94）：换地后随机使用一张
普通锦囊——实体牌入手后 emit CardUsed（内核各锦囊结算/无懈窗口自动复
用），随机其他存活目标。**登记差异（最小闭环口径）**：随机池收窄为决
斗/无中生有（内核结算已确证者，随覆盖逐批扩）；正收益目标判定为 AI
不迁移；上游虚拟牌 → 原生牌堆实体。**地附技第三例**：
`MtgYoulinDiscard`（tag 95，sync_land_skill 第三映射）——出牌阶段限一
次弃一张锦囊并从牌堆亮 ≤3 三选一入手。版本 107→108 入 canonical_hash
——14 golden 重录（幽林不在标准场景，纯版本位移）。回归：
`mtg_youlin_land_and_discover`（随机使用离堆/地附技×往返/限一次/换地
撤销）。证据见
[mtg 第九批](../operations/evidence/2026-10-08-u05-mtg-batch9/README.md)。

**规则 v109（2026-10-08，U05 mtg 第十批：邻地溪流+两暂缓）**：
`CardKind::Lindixiliu`（编码 130，♠11 land）+ `MtgLindixiliuLand`
（tag 96）：换地后牌堆♥首张入手，可弃任意张♥（he，可取消）——每张随
机敌方判定区置入其未持有的延时锦囊（牌堆 Le→Bing→ShanDian 顺序实体；
判定区直接 push，JudgePhase 取队；无专用日志登记）。
**暂缓定案**：藏宝海湾（上游 enable 无 mode_derivation 即禁用——原生
同禁用态）与毒沼（createCard("du") 衍生实体无毒结算）登记暂缓不入转
换包；mtg 12 张上游清单全覆盖（10 转换 + 2 暂缓）。
版本 108→109 入 canonical_hash——14 golden 重录（邻地溪流不在标准场
景，纯版本位移）。回归：`mtg_lindixiliu_land_effect`（♥入手/弃♥×往返
/置延时断言/取消路径）。证据见
[mtg 第十批](../operations/evidence/2026-10-08-u05-mtg-batch10/README.md)。

**规则 v110（2026-10-08，U05 zhenfa 首切片：地载阵）**：新转换包
shahai-zhenfa-upstream-noname + `CardKind::Dizaizhen`（编码 131，♦3/
♠3/♥3）+ `DizaizhenMachine`（无懈窗口包装，展开非大势力存活角色各摸
1，本体离结算区）+ `GameStateCore::is_major_faction`——effective_
faction 非空者按势力统计存活数、最大者并列同算为大。**登记差异**：
forceMajor 技能标特例未建模；队列（inline）/围攻/swapSeat 体系未建模
——长蛇/天覆/七星/鸟翔/破阵决 deferred；上游 mode:["guozhan"] 限定未
建模（大势力按 effective_faction 推导：Kingdoms=明置势力、其余=主将
势力）。版本 109→110 入 canonical_hash——14 golden 重录（地载阵不在
标准场景，纯版本位移）。回归：`zf_dizaizhen_major_faction_draw`（魏
魏蜀并列大势力判定/全魏负例）+ `zf_real_host_full_lifecycle`（六步）。
证据见 [zhenfa 首切片](../operations/evidence/2026-10-08-u05-zhenfa-slice/README.md)。

**规则 v111（2026-10-08，U07 技能批一：刺猬/韩龙）**：将领 mapping
通道（cards.json skills[].mapping → host 行为）+ `WudianCiqiu`（tag
97，权限型 Noop）：commit_damage 钩——来源持 ciqiu 实例 && 杀 && 目标
满血 → 伤害 +1；加后 ≥ 目标体力 → `TurnState.ciqiu_dying` push（回合
级，换回合清空）+ 来源技能实例移除（一次性）。**DyingSave 即死分
支**：dying 者标记在位 → 跳过救援询问直接 Death（上游 ciqiu_dying
dyingBegin firstDo die 同口径）。**键名口径**：mapping 通道目录技能名
=cards.json skills name（"ciqiu"）。版本 110→111 入 canonical_hash
——14 golden 重录（刺猬不在标准场景，纯版本位移）。回归：
`hanlong_ciqiu_sha_healthy_bonus_and_dying_mark`（非满血不加成/满血
+1+濒死即死/移除后不加成）。证据见
[U07 技能批一](../operations/evidence/2026-10-08-u07-skill-batch1/README.md)。

**规则 v112（2026-10-08，U07 技能批二：率言/费诗）**：`WudianShuaiyan`
（tag 98）——TriggerMeta Recover/Normal **global 触发**（plan_triggers
按 (owner, def) 实例遍历）：他人回复体力后（filter 排除回复者=你/当前
回合=你/回复者死亡），回复者二选一——Tag0 费诗摸 1 / Tag1 费诗弃其一张
（被弃者自选 min1 不可取消=上游 discardPlayerCard forced 同义）。**登记
差异**：chooseControl AI 默认项（attitude 推导）不迁移（玩家自选，各批
同口径）。费诗目录技能=["nsshuaiyan","moshou"]。版本 111→112 入
canonical_hash——14 golden 重录（率言不在标准场景，纯版本位移）。回归：
`feishi_shuaiyan_recover_trigger_two_choice`（两分支+负例×2）。证据见
[U07 技能批二](../operations/evidence/2026-10-08-u07-skill-batch2/README.md)。

**规则 v113（2026-10-08，U07 技能批三：筑城双技/刘馥）**：`Wudian-
Zhucheng`（tag 99）+ `WudianZhuchengGuard`（tag 100，上游 group 子技能
zhucheng2 显式化）。**状态**：`PlayerState.zhucheng: Vec<CardId>`（跨
回合持久标记；**存档格式 v4→v5**）。开关：commit_phase PhaseEnd 段直
推 OptionalGate（原生无 begin:false 帧；拥有者自己每阶段结束确认）—
无标记→牌堆顶存 max(1, maxHp-体力) 张；有标记→收回入手。守卫：杀指
定且标记非空 → 使用者选一张=弃 n 张放行 / Pass=杀无效（帧 cancelled
写回 + 杀归还手牌 + intrinsic cancelled 跳过结算——CardUsed:
Commit→AfterTriggers→Intrinsic 时序允许 AfterTriggers 机器写回帧载
荷）。**登记差异**：守卫弃牌上游一次性选 n 张——原生首张自选+余随机
代抽（池排除在途杀）；夺启暂缓（phaseUse 中止未建模）。未映射 7→6。
版本 112→113 入 canonical_hash——14 golden 重录（筑城不在标准场景，
纯版本位移）。回归：`liufu_zhucheng_toggle_and_sha_guard`（存牌/杀无
效回手/弃 n 放行/收回）。证据见
[U07 技能批三](../operations/evidence/2026-10-08-u07-skill-batch3/README.md)。

**规则 v114（2026-10-08，U07 技能批四：抗yin/留赞）**：`WudianKangyin`
（tag 101，once_per_turn）四段机：目标（其他存活 he 非空）→ 失去 1 体
力（commit_lose_hp **同步提交**——分支依赖 isDamaged 判定，异步子事件
读旧体力）→ 被弃者自选弃一张（he）→ 已受伤时分支：基本（yangpijuan_
class==0）=选至多 X 名各摸一（CardsAsk 多选）；非基本=授
`TurnState.kangyin2: Option<(PlayerId, u32)>`（X=maxHp-体力，回合内；
computed_distance 持有者算到他人 -X + 方天同款杀额外目标上限 +X；换回
合清空）。未受伤无分支。**登记差异**：kangyin2 上游 mark+双 mod——原
生状态项+上限扩展；无专用日志。未映射 6→5。版本 113→114 入
canonical_hash——14 golden 重录（抗yin 不在标准场景，纯版本位移）。回
归：`liuzan_kangyin_losehp_discard_and_branch`（失去体力/被弃/基本分
支多摸/限一次/kangyin2 杀双目标）。证据见
[U07 技能批四](../operations/evidence/2026-10-08-u07-skill-batch4/README.md)。

**规则 v115（2026-10-08，U07 技能批五：车阵/田豫）**：`WudianChezhen`
（tag 102，权限型 Noop）——锁定距离技：computed_distance 内查 (a,def)/
(b,def) 实例 → a 有装备 minus+1（攻侧）/ b 无装备 plus+1（守侧）。
**登记差异**：无（纯数据钩语义等价；距离下限 1 饱和为既有路径）。未映
射 5→4（夺启暂缓）。版本 114→115 入 canonical_hash——14 golden 重录
（车阵不在标准场景，纯版本位移）。回归：
`tianyu_chezhen_distance_mod`（5 人局对座：关/开对比+卸装备消散+相邻
基线不受影响）。证据见
[U07 技能批五](../operations/evidence/2026-10-08-u07-skill-batch5/README.md)。

**规则 v116（2026-10-08，U07 技能批六：绝道/刘焉）**：`WudianJuedao`
（tag 103）——activate 守卫（未横置才可发动，上游
filter isLinked()===false 同口径）+ 单步机（emit Linked 翻转，
commit_linked 原语复用）+ `computed_distance` 绝道钩：**持技者**横置时
双向 +1（上游 mod 挂在 juedao 技能上：globalFrom 查 from 的技能、
globalTo 查 to 的技能——非持技者横置不影响距离；初版误为「任意横置
者」致铁索传导距离炸裂，已修正并回归 native_kernel 传导三测）。
**登记差异**：content 无弃牌代价——上游 filterCard/check 为转换技 AI
残留未挂 content，原生按 content 实现无代价横置。未映射 4→3。版本
115→116 入 canonical_hash——14 golden 重录（绝道不在标准场景，纯版本
位移）。回归：`liuyan_juedao_link_and_distance`（横置/双向对座
2→3/已横置拒绝/他人横置不影响/双横置叠加）。证据见
[U07 技能批六](../operations/evidence/2026-10-08-u07-skill-batch6/README.md)。

**规则 v117（2026-10-08，U07 技能批七：割据/刘焉）**：`WudianGeju`
（tag 104，TriggerMeta PhaseBegin/Normal）——Judge begin 帧触发强制：
收集他人 effective_faction 种数 → 按**各攻击者自身 attack_range** 逐个
移除杀范围内他势力 → draw(剩余种数)。**登记差异**：势力判定
effective_faction（与地载阵同口径）；frequent 无确认同上游。未映射
11→2（duoqi/youzhan 暂缓）。版本 116→117 入 canonical_hash——14
golden 重录（割据不在标准场景，纯版本位移）。回归：
`liuyan_geju_out_of_range_faction_draw`（3 人局负例）+
`liuyan_geju_out_of_range_counts`（4 人局正例：范围外蜀计入/范围内魏
不计入）。证据见
[U07 技能批七](../operations/evidence/2026-10-08-u07-skill-batch7/README.md)。

**规则 v118（2026-10-08，U05 ow 纵切首切片：士兵76）**：新转换包
shaai-ow-upstream-noname（**character 管线首例**：cards 空数组纯武将
包，general_meta+mapping 通道）+ `WudianTuji`（tag 104 权限型）——
`TurnState.cards_used_play_phase: u32`（Play begin 清零、
commit_card_used 非取消 +1；computed_distance 持有者+当前回合 →
minus+=计数，上游 tuji mod globalFrom countUsed 同口径）+
`WudianMujing`（tag 105）三段机——黑牌当杀 declared-Sha（塔击同构）。
**登记差异**：mujing 上游 chooseToRespond/chooseToUse 双入口——原生仅
出牌入口（丈八同口径）；**lichang 暂缓**（下准备阶段子回合时点未建
模，包内 deferred reason）。版本 117→118 入 canonical_hash——14 golden
重录（ow 不在标准场景，纯版本位移）。回归：
`ow_shibing_tuji_and_mujing`（目录技能断言/目镜当杀掉血/突击攻距
-1/换回合计数清零）。证据见
[ow 纵切首切片](../operations/evidence/2026-10-08-ow-shibing-slice/README.md)。

**规则 v119（2026-10-08，U05 ow 纵切批二：光书/虚无之光）**：
`WudianGuangshu`（tag 107）三段机（弃 he 一张 → 目标（四子技实例同查
排除）→ 按弃牌花色授子技 SkillDefId 实例）+ 4 子行为（tag 108–112）：
♥光盾 Damage 触发 recover 1 / ♦光流 source Damage 触发 draw 2 / ♣光井
杀门控拒绝（PlayCard Sha 臂查实例，碧日浓雾先例）+ PhaseEnd 触发移除 /
♠光塔 commit_phase PhaseEnd 段直推 1 无源雷伤（无 PhaseEnd 帧，水没墓
穴/筑城第三例）。**登记差异**：♠上游"下个结束阶段"实为任一阶段结束即
触发（trigger phaseEnd 逐阶段）——原生同口径；mark intro UI 不迁移。
版本 118→119 入 canonical_hash——14 golden 重录（光书不在标准场景，纯
版本位移）。回归：`zhixuzhiguang_guangshu_suit_marks`（♠全链：弃黑
桃/授予断言/阶段结束雷伤）。证据见
[ow 纵切批二](../operations/evidence/2026-10-08-ow-guangshu-batch/README.md)。

**规则 v120（2026-10-08，U05 ow 纵切批三：自愈/禅雅塔）**：
`WudianZiyu`（tag 112，TriggerMeta PhaseBegin/Normal optional——End 阶
段开始帧）+ `TurnState.phase_number: u32`（每阶段开始 +1，入存档/哈
希）+ filter（phase_number>0 && %4==0）+ OptionalGate 确认 + 两段机
（Tag0 摸 1 / Tag1 回 1；满血仅摸牌；可取消=放弃）。**登记差异**：上
游 phaseEnd 逐阶段 global——原生锁定 End 阶段开始；phaseNumber 从 0
计（>0 防开局触发，微差登记）。版本 119→120 入 canonical_hash——14
golden 重录（自愈不在标准场景，纯版本位移）。回归：
`chanyata_ziyu_every_four_phases`（12 回合内 %4 至少触发一次/回 1）。
证据见
[ow 纵切批三](../operations/evidence/2026-10-08-ow-ziyu-batch/README.md)。

**规则 v121（2026-10-08，U05 ow 纵切批四：守护/天使）**：
`WudianShouhu`（tag 113，once_per_turn 不适用——上游无 usable 限制）
三段机：弃手牌【杀】→ 选其他受伤角色 → recover 1（上游 target.recover
同口径）+ **禁杀门控**：PlayCard Sha 臂查持有者实例拒绝（碧日浓雾/光
井同款先例，上游 mod cardEnabled sha=false 同口径）。**版本号勘误**：
v120 批遗漏提交版本号（常量实为 119）——本批 119→121 合并跳变重录。
回归：`tianshi_shouhu_discard_heal_and_sha_ban`（弃杀救人/禁杀拒绝/无
杀不可发动）。证据见
[ow 纵切批四](../operations/evidence/2026-10-08-ow-shouhu-batch/README.md)。

**规则 v122（2026-10-08，U05 ow 纵切批五：毒瘟/黑百合）**：
`WudianDuwen`（tag 114，权限型 Noop）——commit_damage 钩：来源持 duwen
实例 && 来源手牌数==受伤害角色手牌数 → 伤害 +1（上游 duwen damageBegin
num++ 同口径；铁索传导为独立事件照常判定）。**登记差异**：无。未映射
2 不变（duoqi/youzhan 暂缓）。版本 121→122 入 canonical_hash——14
golden 重录（毒瘟不在标准场景，纯版本位移）。回归：
`heibaihe_duwen_equal_hand_damage_bonus`（手牌相等+1/不等无加成）。证
据见
[ow 纵切批五](../operations/evidence/2026-10-08-ow-duwen-batch/README.md)。

**规则 v123（2026-10-08，U05 ow 纵切批六：炮台/托比昂）**：
`PlayerState.paotai_lv: u8`（跨回合持久；**存档格式 v5→v6**）+
`WudianPaotai`（tag 115）主动技两段机（弃杀→lv+1 ≤3，activate 守卫
lv<3）+ `WudianPaotaiFire`（tag 116，TriggerMeta PhaseBegin/Normal——
End 阶段开始帧）触发臂：概率 30%×层（RuleRng）→ 随机存活敌人 1 点火
焰伤害 + **commit_damage 炮台降级钩**（受伤 1 点降 1 级，下限 0；上游
paotai3 同口径）。**登记差异**：上游 paotai2 入队前掷骰——原生结算时
掷骰（等价分布）；mark intro UI 不迁移；maoding 暂缓（零件牌池不在原
生目录）。版本 122→123 入 canonical_hash——14 golden 重录（炮台不在
标准场景，纯版本位移）。回归：
`tuobiang_paotai_stack_fire_and_downgrade`（三层布置/满级拒绝/100% 必
中/受伤降级/跨回合持久）。证据见
[ow 纵切批六](../operations/evidence/2026-10-08-ow-paotai-batch/README.md)。

**规则 v124（2026-10-08，U05 ow 纵切批七：堡垒/侦察切换）**：
`PlayerState.baolei_mode: u8`（0=无/1=哨卫/2=侦察，跨回合持久；存档
v6 内追加）+ `WudianBaoleiSwitch`（tag 117）单步确认切换机（三态翻转：
2→1 回哨卫、其余 +1——上游双技各 phaseEnd 确认的互斥切换合并为确认式
翻转）。**登记暂缓（模式效果钩）**：哨卫四合一（杀无视距离/防具、无
次数、不可闪避+持有者禁闪——ShaResolve/闪避门控深层多处）与侦察用杀
摸回（CardUsed Sha 触发机）——留待后续批。mapping 双技同键
（bshaowei/zhencha）。版本 123→124 入 canonical_hash——14 golden 重录
（堡垒不在标准场景，纯版本位移）。回归：
`baolei_switch_mode_cycle`（三态翻转/存档往返/跨回合持久）。证据见
[ow 纵切批七](../operations/evidence/2026-10-08-ow-baolei-batch/README.md)。

**规则 v125（2026-10-08，U05 ow 纵切批八：哨卫模式效果钩）**：消
v124 待做登记——PlayCard Sha 门控：哨卫模式（来源持 bshaowei &&
baolei_mode==1）①杀次数豁免（与 paoxiao/has_zhuge 并列）②距离豁免
（baolei_range_free 与 zfs_free 并列）；ShaResolve ③哨卫源侧直中（仁
王盾后复用 emit_sha_damage，跳过闪询问/防具/雌雄等）④目标侧闪选项过
滤（目标持技+哨卫模式 → shans 空）。**登记差异**：directHit 同时无视
防具（八卦/仁王盾不判定）——上游 directHit 同口径。版本 124→125 入
canonical_hash——14 golden 重录（哨卫不在标准场景，纯版本位移）。回
归：`baolei_mode_effects_on_sha`（源侧直中/目标侧非哨卫正常）。证据见
[ow 纵切批八](../operations/evidence/2026-10-08-ow-mode-effects/README.md)。

**规则 v126（2026-10-08，U05 ow 纵切批九：卢西奥三技）**：扩音
（kuoyin→host:wudian_kuoyin，tag 120）——出牌阶段弃 he 区恰 1 张→乐动
目标数 +2 / 恰 2 张→乐动摸/回量 +1（各限一次、双满 activate 拒绝；
selectCard 张数按存量恰 1/恰 2/1–2 同上游 selectCard）；呼唤
（huhuan→host:wudian_huhuan，tag 121）——弃 2 张本回合乐动改回复模式
（限一次）；乐动（yuedong→host:wudian_yuedong，tag 119）——出牌阶段
结束（PhaseEnd+Play/begin:false 同 GwentBirinState 通道）选至多 1+数
增量名目标各摸 1+量增量张（呼唤后改各回复等量，目标须未满体力；可整
体放弃=空选择；空选项直跳过）。`TurnState.yuedong_num/eff/recover` 三
字段换回合清零=上游 kuoyin2 phaseBegin 静默重置等价（存量仅自己出牌
阶段可积攒/消费）；存档 v6→**v7**。**登记差异**：逐目标顺序摸（上游
asyncDraw 同时摸）；上游 storage 跨回合残留不可观察。版本 125→126 入
canonical_hash——14 golden 重录（卢西奥不在标准场景，纯版本/零值字段
位移）。回归：`luxiao_kuoyin_num_extends_yuedong_draw` 等 4 例。证据见
[ow 纵切批九](../operations/evidence/2026-10-08-ow-luxiao-batch/README.md)。

**规则 v127（2026-10-08，U05 ow 纵切批十：美双技+翻面跳回合基建）**：
暴雪（baoxue→host:wudian_baoxue，tag 123）——出牌阶段 limited（每局一
次）选至多「黑色 he 牌数」名其他角色：弃全部黑色 he 牌、逐目标其 he
区随机弃 1+翻至背面、自己翻至背面（空选择=放弃不消耗；turnOver(true)
幂等同口径）；急冻（jidong→host:wudian_jidong，tag 122）——任意角色
阶段结束时 hp=1 且未翻面：翻面+回复 2+冰封（`PlayerState.jidong_
active`：受所有伤害无效=commit_damage 入口 DamagePrevented、不能被其
他角色的牌指定=PlayCard 目标拒绝——AOE 自动指向不受影响同上游
targetEnabled mod），解除点=翻回正面（commit_turn_over_before，上游
turnOverAfter 到期同口径）。**翻面跳回合基建**：Judge begin 且
face_down 的回合=发 TurnOverBefore（前置取消窗口）后直达 TurnEnded，
整回合无阶段帧、回合槽照常消耗；回合中翻面不跳剩余阶段（上游
_turnover phaseBefore 仅回合开始判定）。bingqiang 冰墙登记暂缓（座位
相邻+距离修饰子技族）。**登记差异**：暴雪/急冻翻面内联无窗口（风雪刃
先例）；showHandcards 不建模。`PlayerState.baoxue_used/jidong_active`
入存档 v7→**v8**。版本 126→127 入 canonical_hash——14 golden 重录
（标准场景无翻面面，纯版本/零值字段位移；据守测试随跳回合新基线更新
断言）。回归：`mei_baoxue_limited_discard_and_flips` 等 3 例。证据见
[ow 纵切批十](../operations/evidence/2026-10-08-ow-mei-batch/README.md)。

**规则 v128（2026-10-08，U05 ow 纵切批十一：路霸双技+v126 修复）**：
链钩（liangou→host:wudian_liangou，tag 124，usable:1 走命令层
once_used=发动即消耗）——弃 1 张 he+指定一名其他角色+判定（非红桃成
功，龙帆判定形态）→挂钩：`TurnState.liangou_hook` 置位——本回合内①
路霸算到被钩者距离恒 1（computed_distance 开头短路；上游 liangou2
globalTo -Infinity 经距离下限钳制同口径）②被钩者本回合首次受伤 +1
（commit_damage 增幅段，liangou_bonus_used 消耗位；来源不限=上游无
source 条件）。换回合随 TurnState 清零解除。吸阳（xiyang→
host:wudian_xiyang，tag 125）——本人回合结束（受伤且未翻面）确认门
（optional:true=上游无 forced 有 check 同口径）→翻面+回复 2（内联翻
面同急冻口径）。**v126 遗留缺陷修复**：TurnState 换回合为原地改字段
而非重建——乐动三角（v126）从未清零、扩音存量跨回合残留；commit_turn_
ended 两分支补显式复位（yuedong 三角+liangou 两字段）。**批十补记登
记差异：急冻触发为直发（optional:false），上游无 forced 会弹确认**。
`TurnState.liangou_hook/liangou_bonus_used` 入存档 v8→**v9**。版本
127→128 入 canonical_hash——14 golden 重录（标准场景不触及，纯版本/
零值字段位移+复位语义）。回归：`luba_liangou_hook_distance_and_damage`
等 3 例+`luxiao_yuedong_storage_resets_next_turn` 缺陷回归。证据见
[ow 纵切批十一](../operations/evidence/2026-10-08-ow-luba-batch/README.md)。

**规则 v129（2026-10-09，U05 ow 纵切批十二：源氏飞刃）**：飞刃
（feiren→host:wudian_feiren，tag 126，锁定被动无机器）三钩——①你的
黑桃杀伤害 +1（commit_damage 增幅段；非铁索传导=**DamageData.from_
chain** 新字段（传导 emit 置 true，随存档 v9 事件编解码首发）；花色按
实体牌，火/雷杀为黑桃同样命中）②你的杀无视距离（PlayCard 首目标与
方天多目标循环均豁免，上游 mod targetInRange 同口径）③梅花杀额外目
标上限 +1（max_targets 加项；实体花色、转化声明牌种不影响=上游
selectTarget range[1]++ 同口径）。**登记暂缓**：lianpo（源定义不在
ow/skill.js=共享技能文件，feiying 同款）、zhanlong（limited+createCard
凭空造牌 3 杀+zhanlong2 临时技链）。存档 v9（随批十一首发）。版本
128→129 入 canonical_hash——14 golden 重录（标准场景 from_chain 恒
false，纯版本/零值字段位移）。回归：`yuanshi_feiren_bonus_range_and_
club_extra_target`、`yuanshi_feiren_chain_damage_not_bonused`。证据见
[ow 纵切批十二](../operations/evidence/2026-10-09-ow-yuanshi-batch/README.md)。

**规则 v130（2026-10-09，U05 ow 纵切批十三：法老之鹰飞弹）**：飞弹
（feidan→host:wudian_feidan，tag 127，Damage after 窗口触发
optional:false=上游 direct 同口径）——你的杀对一名角色造成伤害后（未
防止有量、受伤者存活），可弃 1 张 he（可取消=不弃不弹）：受伤者距离
1 内的其他存活角色各受 1 点无牌伤害（来源=持有者、按座位序、弹射目
标自动全收含自己；弹射 card=None 不再触发飞弹）。杀无视距离（PlayCard
豁免与飞刃通用化）。**登记口径**：受伤者死亡不触发（保守口径，登记）。
**登记暂缓**：huoyu（limited+虚拟 useCard——chiyuxi 凭空用牌×2，
youzhan 同款原语缺口）、feiying（共享技能文件）。无新状态字段——存档
v9 不变。版本 129→130 入 canonical_hash——14 golden 重录（纯版本位
移）。回归：`falaozhiying_feidan_splash_cancel_and_range`。证据见
[ow 纵切批十三](../operations/evidence/2026-10-09-ow-falaozhiying-batch/README.md)。

**规则 v131（2026-10-09，U05 ow 纵切批十四：安娜支援）**：支援
（zhiyuan→host:wudian_zhiyuan，tag 128，EventKind::DamageBefore 前置窗
口触发、optional:true 确认门=上游 check 询问式同口径）——当你即将对
一名角色造成伤害时（来源=持有者、未防止、量>0）可确认：防止此伤害
（current_damage_mut().prevented 直写=碎盾先例），受伤者回复等量体力
（满血饱和同上游 recover）。**DamageBefore 前置窗口首个 ow 消费者**。
**登记暂缓**：mianzhen（响应询问+子技）、aqianghua（子技链）。无新状
态字段——存档 v9 不变。版本 130→131 入 canonical_hash——14 golden
重录（纯版本位移）。回归：`ana_zhiyuan_cancel_damage_and_equal_recover`
（确认=防止+等量回复、取消=照常伤害）。证据见
[ow 纵切批十四](../operations/evidence/2026-10-09-ow-ana-batch/README.md)。

**规则 v132（2026-10-09，U05 ow 纵切批十五：麦克雷神枪）**：神枪
（shenqiang→host:wudian_shenqiang，tag 129，Damage after 窗口触发
optional:false=上游 forced 同口径）——你的杀对一名角色造成伤害后（自
己回合）本回合杀使用计数回退（`turn.sha_used_play_phase=false`），可
再出一张杀；**无次数上限**——每次杀伤害各回退一次（链式解锁）。杀判
定按实体牌口径（转化声明杀不触发，与飞刃/飞弹一致）；land 加成杀的
额外标记不回退（登记）。**登记暂缓**：shanguang（shanguang2 全卡牌
禁用子技）、tiandan（createCard 凭空造杀+主动跳阶段）。无新状态字段
——存档 v9 不变。版本 131→132 入 canonical_hash——14 golden 重录
（纯版本位移）。回归：`maikelei_shenqiang_sha_limit_refund`（链式回
退三连断言）。证据见
[ow 纵切批十五](../operations/evidence/2026-10-09-ow-maikelei-batch/README.md)。

**规则 v133（2026-10-09，U05 ow 纵切批十六：狂鼠拾火）**：拾火
（shihuo→host:wudian_shihuo，tag 130，Damage after 窗口 global 触发
optional:false=上游 forced 同口径）——任何人受到火属性伤害后你摸一张
牌（filter：DamageNature::Fire && 未防止 && 有量——prevented/零量不
触发=上游 damageEnd 事件口径登记；传导的独立火伤事件照常，每次各摸
1）。**登记暂缓**：liudan（杀 50% 概率随机追加合法目标）、shoujia
（给牌+挂牌子技）。无新状态字段——存档 v9 不变。版本 132→133 入
canonical_hash——14 golden 重录（纯版本位移）。回归：
`kuangshu_shihuo_fire_damage_draw`（火伤摸 1/普通伤害不触发）。证据见
[ow 纵切批十六](../operations/evidence/2026-10-09-ow-kuangshu-batch/README.md)。

**规则 v134（2026-10-09，U05 ow 纵切批十七：莱因哈特猛击）**：猛击
（mengji→host:wudian_mengji，tag 131，EventKind::DamageBefore 前置窗
口触发 optional:false=上游 forced 同口径）——你无护甲时你的杀伤害 +1
（非铁索传导=from_chain 字段；前置窗口直写帧量=伊籍改目标同机制）。
上游 filter 的 storage.zhongdun 置位以**持技等价**（登记）。**登记暂
缓**：zhongdun（init2 开局 +8 甲——无 GameStart 事件种类=开局事件基
建缺口；主动转甲技）。测试基建：`Engine::debug_set_hujia`（debug_set_hp
先例）。无新状态字段——存档 v9 不变。版本 133→134 入 canonical_hash
——14 golden 重录（纯版本位移）。回归：
`laiyinhate_mengji_no_armor_bonus`（无甲 +1/有甲不加成）。证据见
[ow 纵切批十七](../operations/evidence/2026-10-09-ow-laiyinhate-batch/README.md)。

**规则 v135（2026-10-09，U05 ow 纵切批十八：半藏音波）**：音波
（yinbo→host:wudian_yinbo，tag 132，usable:1 走命令层 once_used）——
出牌阶段弃 1 张黑桃（he 区必选）：随机 ≤3 名有牌的其他角色各随机弃 1
张（swap_remove 抽取式取 ≤3+he 池随机弃=龙帆/暴雪同款）。**登记差异**：
原型无阵营——上游 enemies=其他存活玩家（多阵营引入时重审）。**登记暂
缓**：bfengshi（shaBegin 随机直击+countUsed 攻距+组技）。无新状态字段
——存档 v9 不变。版本 134→135 入 canonical_hash——14 golden 重录
（纯版本位移）。回归：`banzang_yinbo_random_discard_three`。证据见
[ow 纵切批十八](../operations/evidence/2026-10-09-ow-banzang-batch/README.md)。

**规则 v136（2026-10-09，U05 ow 纵切批十九：奥丽莎强固）**：强固
（qianggu→host:wudian_qianggu，tag 133，usable:1）——出牌阶段弃 2 张
he（必选）→+2 护甲+强固置位（`PlayerState.qianggu_active`）；强固守
卫（qianggu2→host:wudian_qianggu_guard，tag 134，CardUsed after 窗口
触发=**筑城守卫同款 cancelled 写回**）——强固生效中他人对持有者使用
杀：使用者弃一张基本牌（Sha/Shan/Tao/Jiu=上游 basic 同口径）否则杀
无效（Intrinsic 按 cancelled 跳过）。解除=自己回合判定阶段开始
（commit_phase Judge begin 清位；翻面跳回合路径同清）。**登记暂缓**：
woliu（组距离/杀限定子技+清除组）；D.Va 全系（机甲独立血条+changeHp
拦截链，立项核验未入包）。存档 v9→**v10**（qianggu_active 追加）。版
本 135→136 入 canonical_hash——14 golden 重录。回归：
`orisa_qianggu_armor_and_guard_nullify`。证据见
[ow 纵切批十九](../operations/evidence/2026-10-09-ow-orisa-batch/README.md)。

**规则 v137（2026-10-09，U05 ow 纵切批二十：温斯顿跳跃）**：跳跃
（feitiao→host:wudian_feitiao，tag 135，usable 无限制）——出牌阶段弃
1 张 he（必选）并指定一名其他角色（必选）：其 he 池随机弃 1，本回合
自己算到其距离恒 1（`TurnState.feitiao_target`；computed_distance 开
头短路=链钩同款；上游 feitiao2 -Infinity 钳距同口径）。换回合随
TurnState 复位（两分支显式）。**登记差异**：chooseCardTarget 复合拆两
段。**登记暂缓**：dianji（jingleishan 牌种缺失）。存档 v10→**v11**。
版本 136→137 入 canonical_hash——14 golden 重录。回归：
`wensidun_feitiao_range_and_random_discard`。证据见
[ow 纵切批二十](../operations/evidence/2026-10-09-ow-wensidun-batch/README.md)。

**规则 v138（2026-10-09，U05 ow 纵切批二十一：查莉娅屏障+应用）**：
屏障（pingzhang→host:wudian_pingzhang，tag 136，DamageBefore global 窗
口 direct=取消在弃牌询问）——自己受伤弃 1 张红桃 / 他人受伤弃 1 张黑
桃（各每回合一次，`PlayerState.pingzhang_self_used/other_used` 两标志
=上游空壳 pingzhang2/3 等价）→伤害 -1（前置窗口直写 saturating_sub；
上游 trigger.num--）。应用（owliyong→host:wudian_owliyong，tag 137，
PhaseBegin+Draw/begin:true 过滤 forced）——摸牌阶段额外摸
`pingzhang_count` 张；**标志转计数**=自己回合判定阶段开始（count=两标
志和并清；上游 pingzhang_count phaseBegin silent 同口径），count 保持
至下回合判定阶段（owliyong 只读不清=上游 storage 同口径）。无暂缓——
查莉娅全技能闭环。存档 v11→**v12**（三字段追加）。版本 137→138 入
canonical_hash——14 golden 重录。回归：
`zhaliya_pingzhang_barrier_and_owliyong_draw`。证据见
[ow 纵切批二十一](../operations/evidence/2026-10-09-ow-zhaliya-batch/README.md)。

**规则 v139（2026-10-09，U05 ow 纵切批二十二：黑影隐匿）**：隐匿
（yinshen→host:wudian_yinshen，tag 138，CardsDiscarded 帧触发 forced=
思继先例同位）——弃置含基本牌（Sha/Shan/Tao/Jiu）且自己手牌无基本牌
→潜行（`PlayerState.yinshen_active`；PlayCard 目标门控与急冻冰封并列
一处条件；解除=自己回合判定阶段开始，kernel Judge begin 清位块）。
上游 tempHide→qianxing 至 phaseBeginStart 同口径。**登记差异**：上游
loseEnd 含所有失牌路径（original=="h" 区分），原生按弃置路径
CardsDiscarded 覆盖。**登记暂缓**：qinru/maichong（跨回合列表）。存档
v12→**v13**。版本 138→139 入 canonical_hash——14 golden 重录。回归：
`heiying_yinshen_untargetable_until_own_judge`。证据见
[ow 纵切批二十二](../operations/evidence/2026-10-09-ow-heiying-batch/README.md)。

**规则 v140（2026-10-09，U05 ow 纵切批二十三：死神霰弹）**：霰弹
（xiandan→host:wudian_xiandan，tag 139 目录标记技；ShaResolve 内联钩）
——出杀结算开始（防具/闪询问前）来源可弃 1 张 he（可取消）：红=直击
（emit_sha_damage 哨卫同款路径）；黑=本杀伤害 +1（emit 伤害式加项=
xiandan2 damageBegin num++ 折算，随本杀一次性）；取消=照常（asked 复
位=青龙再杀 reboot 同款）。ShaResolveMachine 三字段入快照/哈希。**登
记差异**：xiandan2 触发形态折算为本杀 emit 加项（同一伤害事件观察等
价）。**登记暂缓**：yihun（虚拟 useCard）、shouge（createCard 治疗波
跨包牌种）。存档 v13→**v14**（Sha 快照三 bool）。版本 139→**140** 入
canonical_hash——14 golden 重录（终态哈希不含挂起机器，位移来自版本
——首轮漏 bump 由 golden 零位移暴露后补）。回归：
`sishen_xiandan_red_direct_black_bonus`。证据见
[ow 纵切批二十三](../operations/evidence/2026-10-09-ow-sishen-batch/README.md)。

**规则 v141（2026-10-09，U05 ow 纵切批二十四：D.Va 机甲）**：机甲
（jijia→host:wudian_jijia，tag 140 目录标记技）——初始 4 点机甲血
（`PlayerState.jijia_hp`）：受伤时 1:1 抵扣（commit_damage 护甲块后拦
截，体力不损、HujiaAbsorbed 日志复用；归零后照常扣体力=上游 changeHp
拦截体力回补观察等价）；机甲血>0 时手牌上限 +4（弃牌阶段 limit 钩）。
**登记差异**：上限 +4 常数简化（上游随剩余机甲血递减）；初始点数经测
试助手/授予方设定（生产授予待开局事件基建）。**登记暂缓**：
tuijin/zihui/chongzhuang。存档 v14→**v15**。版本 140→**141** 入
canonical_hash——14 golden 重录。回归：
`dva_jijia_absorbs_damage_and_hand_limit`。证据见
[ow 纵切批二十四](../operations/evidence/2026-10-09-ow-dva-batch/README.md)。

**规则 v142（2026-10-09，U05 ow 收官 + xianjian 首切片）**：① ow 来
源纵切收官——猎空双技登记 deferred（区域离场=牌守恒纪律结构性依赖），
ow 包 24/24 武将全部入包（23 闭环+1 结构性暂缓；**口径勘误：上游 24
武将非 25**，与来源清点一致）。② xianjian 首切片——新转换包
shaai-xianjian-upstream-noname（character 管线第二例）：灵雪
（lingxue→host:xj_lingxue，tag 141，Recover 窗口触发 optional:false=
上游 forced 同口径）——你回复体力后 +1 护甲（change_hujia 原语；多次
回复各 +1）。**登记暂缓**：feizhua/leiyu。无新状态字段——存档 v15 不
变。版本 141→**142** 入 canonical_hash——14 golden 重录。回归：
`xingxuan_lingxue_recover_grants_hujia`。证据见
[ow 收官 + xianjian 首切片](../operations/evidence/2026-10-09-ow-closeout-xianjian-slice/README.md)。

**规则 v143（2026-10-09，U05 xianjian 第二切片：沈欺霜冰心）**：冰心
（binxin→host:xj_binxin，tag 142，PhaseEnd global 窗口触发
optional:true=上游 check 询问式同口径）——任何体力为 1 的角色阶段结束
时（含自己）确认后其 +1 护甲（事件者经 machine() 分流带入）。**登记暂
缓**：qixia（花色集集齐 4 色 insertPhase 额外回合——ADR0646 同源缺
口）、jianzhen（shaAfter 逐人 chooseToUse 询问链）。无新状态字段——
存档 v15 不变。版本 142→**143** 入 canonical_hash——14 golden 重录。
回归：`shenqishuang_binxin_grants_hujia_at_low_hp`。证据见
[ow xianjian 第二切片](../operations/evidence/2026-10-09-ow-shenqishuang-batch/README.md)。

**规则 v144（2026-10-09，U05 xianjian 第三切片：赵灵儿回梦+蛇蜕）**：
回梦（huimeng→host:xj_huimeng，tag 143，Recover after 直发=灵雪同位；
**登记差异**：frequent 询问式原生直发——无选择面观察等价）——你回复体
力后摸两张牌。蛇蜕（tianshe→host:xj_tianshe，tag 144，DamageBefore 前
置窗口 forced）——属性伤害对你无效（prevented 直写=碎盾先例；传导事
件同免疫=上游 hasNature 同口径）。蛇蜕·息（tianshe2→host:xj_tianshe2，
tag 145，Damage after forced）——你造成属性伤害后回复 1（filter 收窄
已受伤）。无暂缓——赵灵儿全技能闭环。无新状态字段——存档 v15 不变。
版本 143→**144** 入 canonical_hash——14 golden 重录。回归：
`zhaoliner_huimeng_tianshe_family`。证据见
[xianjian 第三切片](../operations/evidence/2026-10-09-ow-zhaoliner-batch/README.md)。

**规则 v145（2026-10-09，U05 xianjian 第四切片：瑕返灵）**：返灵
（fanling→host:xj_fanling，tag 146，**EventKind::LoseHp after 窗口触
发**=首个 LoseHp 消费者；forced）——他人失去体力后（有量）已受伤时回
复 1；每回合限一次（`TurnState.fanling_used`——链钩 usable:1 同款
TurnState bool）。**登记暂缓**：xjzongyu（虚拟 useCard 飞镖）。测试基
建：`Engine::debug_place_judge_id`（按 ID 摆判——跨包无名牌）。存档
v15→**v16**。版本 144→**145** 入 canonical_hash——14 golden 重录。
回归：`xia_fanling_recover_on_others_hp_loss`（归有界失体力→回复 1/
每回合一次）。证据见
[xianjian 第四切片](../operations/evidence/2026-10-09-ow-xia-batch/README.md)。

**规则 v146（2026-10-09，U05 xianjian 第五切片：温慧龙翔）**：龙翔
（longxiang→host:xj_longxiang，tag 147 目录标记技；ShaResolve 内联钩
——霰弹钩后）——出杀结算开始时目标手牌多于自己→目标 he 池随机弃差
值张（swap_remove 抽取；**杀在结算区不计入来源手牌**=上游 countCards
同口径）。无询问零机器字段（每杀至多一次由 !asked 分支天然保证）。
**登记差异**：上游 check=AI 姿态（内容强制）同口径。**登记暂缓**：huxi
（无实体牌虚拟用牌+respondSha）。无新状态字段——存档 v16 不变。版本
145→**146** 入 canonical_hash——14 golden 重录。回归：
`wenhui_longxiang_random_discard_diff`。证据见
[xianjian 第五切片](../operations/evidence/2026-10-09-ow-wenhui-batch/README.md)。

**规则 v147（2026-10-09，U05 xianjian 第六切片：林月如归元）**：归元
（guiyuan→host:xj_guiyuan，tag 148，usable:1 走命令层 once_used）——
出牌阶段已受伤时弃一张杀（he 区必选）→回复 1 点体力（满血饱和）+摸一
张牌。**登记差异**：无。**登记暂缓**：qijian（虚拟用牌——phaseDiscard-
End 弃牌→虚拟 useCard 杀）。无新状态字段——存档 v16 不变。版本
146→**147** 入 canonical_hash——14 golden 重录。回归：
`linyueru_guiyuan_discard_sha_recover_draw`。证据见
[xianjian 第六切片](../operations/evidence/2026-10-09-ow-linyueru-batch/README.md)。

**规则 v148（2026-10-09，U05 xianjian 第七切片：韩菱纱探囊）**：探囊
（tannang→host:xj_tannang，tag 149 目录标记技；usable:1 走命令层 once_
used）——持技者梅花手牌在 PlayCard view-as 链路由为顺手牵羊（奇袭同
级分支）+ **Snatch 距离豁免**（实体牌非顺牵 && 持技→无距离限制=上游
mod targetInRange 同口径；实体顺牵不受影响）。**登记差异**：上游
chooseToUse 杀/顺牵双形态，原生无拆分入口——梅花杀路由为顺牵（杀形
态不可选）。**登记暂缓**：tuoqiao（石会粉跨包牌种）。无新状态字段——
存档 v16 不变。版本 147→**148** 入 canonical_hash——14 golden 重录。
回归：`hanlingsha_tannang_snatch_unlimited_range`。证据见
[xianjian 第七切片](../operations/evidence/2026-10-09-ow-hanlingsha-batch/README.md)。

**规则 v149（2026-10-09，U05 xianjian 第八切片：王蓬絮废墟）**：废墟
（feixu→host:xj_feixu，tag 150，CardUsed 主事件+
extra_events=[CardResponded] 双事件同技；optional:true=上游 check 询问
式同口径）——任意角色使用或打出闪后（filter 牌为闪）确认后该角色摸一
张牌。**登记差异**：无（useCard/respond 双事件一一对应）。**登记暂
缓**：duxinshu（按钮选择+双向换牌）。无新状态字段——存档 v16 不变。
版本 148→**149** 入 canonical_hash——14 golden 重录。回归：
`wangpengxu_feixu_shan_draw`。证据见
[xianjian 第八切片](../operations/evidence/2026-10-09-ow-wangpengxu-batch/README.md)。

**规则 v150（2026-10-09，U05 xianjian 第九切片：雷元戈风雅颂）**：三
计数器技全闭环。风（feng→host:xj_feng，tag 151，CardsDrawn after
forced）——累计摸 2 次牌后下一次摸牌后**额外摸 1** 并清零（**登记差
异**：上游 drawBegin 前置改数量，原生 after 补摸——净效果同为每三次
摸牌多摸一张）。雅（ya→host:xj_ya，tag 152，DamageBefore forced）——
累计受伤 2 次后本次伤害 -1（saturating_sub；num>0 才减=上游同口径）。
颂（song→host:xj_song，tag 153，DamageBefore forced source scope）——
累计造成伤害 2 次后本次伤害 +1。三计数器=PlayerState 三 u8（存档
v16→**v17**）。无暂缓——雷元戈全技能闭环。版本 149→**150** 入
canonical_hash——14 golden 重录。回归：
`leiyuange_feng_ya_song_counters`。证据见
[xianjian 第九切片](../operations/evidence/2026-10-09-ow-leiyuange-batch/README.md)。

**规则 v151（2026-10-09，U05 xianjian 第十切片：李逍遥御风）**：御风
（xjyufeng→host:xj_xjyufeng，tag 154，CardsDiscarded 帧触发 forced，
每回合限两次=TurnState.xjyufeng_used u8）——弃置含任意牌且手牌<2 时
摸（2-手牌数）张（上游 loseEnd usable:2 同口径；弃置路径覆盖登记差异
=隐匿同款）。**登记暂缓**：tianjian（viewAs 万箭 AOE+伤害归属双缺
口）。存档 v17→**v18**。版本 150→**151** 入 canonical_hash——14 gol-
den 重录。回归：`lixiaoyao_xjyufeng_draw_to_two_twice_per_turn`。证据
见[xianjian 第十切片](../operations/evidence/2026-10-09-ow-lixiaoyao-batch/README.md)。

**规则 v152（2026-10-09，U05 xianjian 第十一切片：玄霄玄炎+凝冰）**：
玄炎·燃（xuanyan2→host:xj_xuanyan2，tag 155，DamageBefore forced source
scope）——你的火属性伤害 +1（非铁索传导=from_chain，飞刃同款）。玄炎·
烬（xuanyan3→host:xj_xuanyan3，tag 156，Damage after forced source）—
—你造成火属性伤害后失去 1 点体力（LoseHp 独立事件=返灵同位）。凝冰
（ningbin→host:xj_ningbin，tag 157，Damage after forced target scope）
——你受到雷属性伤害后回复 1（满血饱和）。玄炎 group 壳（xuanyan→host:
xj_xuanyan，tag 158 目录标记技=tianshe 系先例，xuanyan2/3 各自 mapping
独立实例）。**登记差异**：无。**登记暂缓**：xfenxin（changeHp 事件缺口
——jijia 用 commit_damage 拦截替代口径不同源）。无新状态字段——存档
v18 不变。版本 151→**152** 入 canonical_hash——14 golden 重录。回归：
`xuanxiao_xuanyan_ningbin_fire_thunder`。证据见
[xianjian 第十一切片](../operations/evidence/2026-10-09-ow-xuanxiao-batch/README.md)。

**规则 v153（2026-10-09，U05 xianjian 第十二切片：龙葵蝶舞+灵愈）**：
蝶舞（diewu→host:xj_diewu，tag 159，usable 无限制）——出牌阶段将一张
杀**直接转移**给一名其他角色（give_hand_card=discard:false prepare
give 同口径）；本回合首次转移后摸 1（`TurnState.diewu_draw_used`=上游
diewu2 临时技等价）。灵愈（lingyu→host:xj_lingyu，tag 160，PhaseEnd
owner 触发）——令一名其他已受伤角色回复 1（可放弃=空目标）。**登记差
异**：diewu2 上游 addTempSkill 原生 TurnState bool 等价。**登记暂缓**：
fenxing（双形态变身结构性依赖）。存档 v18→**v19**。版本 152→**153** 入
canonical_hash——14 golden 重录。回归：
`longkui_diewu_give_and_lingyu_recover`。证据见
[xianjian 第十二切片](../operations/evidence/2026-10-09-ow-longkui-batch/README.md)。

**规则 v154（2026-10-09，U05 xianjian 第十三切片：慕长兰掠影）**：掠影
（lueying→host:xj_lueying，tag 161 目录标记技；ShaResolve 内联钩，每回
合限一次=TurnState.lueying_used）——出杀结算开始：获得目标 he 随机一
张；另有其他有牌者→目标选人（询问 observer=目标）→来源随机弃受害者
he 一张。**登记差异**：无（三步一一对应）。**登记暂缓**：feixia（虚拟
用牌）。Sha 快照两字段（asking_lueying/lueying_victim）入存档 v19→
**v20**。版本 153→**154** 入 canonical_hash——14 golden 重录。回归：
`muchanglan_lueying_gain_and_victim_discard`。证据见
[xianjian 第十三切片](../operations/evidence/2026-10-09-ow-muchanglan-batch/README.md)。

**规则 v155（2026-10-09，U05 xianjian 第十四切片：阿奴蛊舞）**：蛊舞
（anwugu→host:xj_anwugu，tag 162，Damage after 确认门=上游 check 同口
径）——你伤害其他未中蛊的存活角色后，其获得蛊舞（`PlayerState.anwugu_
count=3`）。蛊舞·缚（anwugu2→host:xj_anwugu2，tag 163 三钩无机器）——
蛊舞生效中：①出牌受限（PlayCard WrongPhase 后 anwugu_count>0 && 卡片
计数≥3 → CardLimit，**登记差异**：countUsed 以出牌阶段计数近似——上
游跨阶段计 countUsed）②手牌上限 -1（弃牌阶段 limit 钩）③自己每阶段
结束 -1（归 0 解除=上游 anwugu2 phaseAfter 递减同口径）。对同一目标唯
一（filter anwugu_count==0=上游 !hasSkill 同口径）。**登记暂缓**：lingdi
（距离递增+花色去重存储）。存档 v20→**v21**（anwugu_count 追加）。版
本 154→**155** 入 canonical_hash——14 golden 重录。回归：
`anu_anwugu_afflict_and_limits`。证据见
[xianjian 第十四切片](../operations/evidence/2026-10-09-ow-anu-batch/README.md)。

**规则 v156（2026-10-09，U05 xianjian 第十五切片：唐雨柔甜香去）**：甜
香去（txianqu→host:xj_txianqu，tag 164，DamageBefore 前置窗口确认门
optional=true=上游 check 同口径，每回合限一次=TurnState.txianqu_used）
——你即将造成伤害（未防止有量）且在自己出牌阶段内时确认：防止此伤害
（prevented 直写=安娜支援同款）+摸两张牌+自己回复 1 点体力（满血饱
和）。**登记不转**：qiongguang（纯表现技 noop）。无新状态字段语义变化
——存档 v21→**v22**（txianqu_used 追加）。版本 155→**156** 入 canon-
ical_hash——14 golden 重录。回归：
`tangyurou_txianqu_prevent_and_replenish`。证据见
[xianjian 第十五切片](../operations/evidence/2026-10-09-ow-tangyurou-batch/README.md)。

**规则 v157（2026-10-09，U05 xianjian 第十六切片：姜城焚逝+龙火）**：焚
逝（fenshi→host:xj_fenshi，tag 165，DyingSaved 触发强制=上游 dyingAfter
juexingji 同口径）——本次濒死救回后 +2 护甲+摸两张+获得龙火，觉醒一次。
**新事件种 `EventKind::DyingSaved`**（九点接线：EventKind/EventData/
kind()、snap 载荷校验、commit 空臂（catch-all）、intrinsic None、
event_kind tag 19、EventData 编解码 tag 17、canonical_hash tag 23）——
固有救援机（Intrinsic 段）在体力>0 且脱离濒死的出口发射，触发在
AfterTriggers 窗口消费，时序与上游 dyingAfter（救援机之后）一致；此前
若挂 Dying 入口则濒死者摸到的牌会参与自救（功能性偏离），故立新事件。
龙火（longhuo→host:xj_longhuo，tag 166，PhaseEnd 确认门 optional=true）
——对自己阶段结束对所有存活角色按座位序（座位号升序=上游 lib.sort.seat）
各 1 点火伤（含自己），逐目标 `Flow::Yield` 后重查存活——铁索传导致死
者不再向后结算（上游 redo isIn 同口径）。**登记差异**：衍生技以常驻实
例+`PlayerState.jiangcheng_longhuo` 门控建模（上游 addSkill("longhuo")
动态获得——cards.json longhuo 列入技能表但 flag 未置位不触发，语义等
价）。**登记暂缓**：yanzhan 系（红牌转火杀 usable:1 viewAs+目标
yanzhan3 响应花色限制 mod+yanzhan2→4 杀限次 +1 mod）。存档 v22→**v23**
（jiangcheng_longhuo 追加至 PlayerState 段）。版本 156→**157** 入
canonical_hash——14 golden 重录。回归：
`jiangcheng_fenshi_awaken_once_on_rescue`、
`jiangcheng_longhuo_fires_at_own_phase_end`。证据见
[xianjian 第十六切片](../operations/evidence/2026-10-09-xianjian-jiangcheng-batch/README.md)。

**规则 v158（2026-10-09，U05 xianjian 第十七切片：景天洒金+聚宝）**：
洒金（sajin→host:xj_sajin，tag 167，出牌阶段限一次主动=usable:1 命令
层 once_per_turn）——弃置一张手牌并指定任意名**已受伤**角色（上游
filterTarget hp<maxHp 同口径）各判定，判定颜色与所弃牌相同者回复 1 点
体力；逐目标 `Flow::Yield` 后重查存活（姜城龙火同款）。判定为无改判窗
直读（蓝格怪衣 v99 同口径，**登记差异**）。聚宝（jtjubao→
host:xj_jtjubao，tag 168，global CardsDiscarded 强制=上游
discardAfter frequent 自动无确认）——其他角色于自己回合外弃非基本牌
入弃堆时（本回合首次=`TurnState.jtjubao_used`，上游 jtjubao2 临时技
等价、换回合复位）随机获得其中一张（此刻离弃堆，羊皮卷 v97 同款）。
**登记差异**：触发面=CardsDiscarded 两发射点（弃牌阶段+选择弃置机器
路径）——技能代价弃置路径（to_discard 直写不发事件）不触发，上游
discardAfter 含全部入弃堆弃置。存档 v23→**v24**（jtjubao_used 追加至
TurnState 段）。版本 157→**158** 入 canonical_hash——14 golden 重录。
回归：`jingtian_sajin_judge_color_recover_and_once`、
`jingtian_jtjubao_gains_nonbasic_and_gates`。证据见
[xianjian 第十七切片](../operations/evidence/2026-10-09-xianjian-jingtian-batch/README.md)。

**规则 v159（2026-10-09，U05 xianjian 第十八切片：龙幽月行+敏杀）**：
月行（yuexing→host:xj_yuexing，tag 169，出牌阶段限一次主动）——指定一
名其他角色，双方距离基数交换至自己出牌阶段结束：
`PlayerState.yuexing_pair` 双向互指；`computed_distance` 增臂——a 持对
偶时出距基数改用 seat_distance_alive(pair,b)（义从/链钩/跳跃同区）；
commit_phase `!begin && Play` 双向解除（上游 yuexing2 临时技
phaseUseAfter 同口径）。**登记差异**：上游 mod 为 pure 距离数值代换后
其余修饰照算，原生=仅替换座位基数、马匹等修饰照常叠加。敏杀
（minsha→host:xj_minsha，tag 170，出牌阶段限一次主动）——弃两张 he
牌→指定体力>1 的其他角色受 1 雷伤→距其 1 内其他角色（除自己）按座位
序各随机弃一张（逐目标 Flow::Yield 重查存活=上游 redo isIn 同口径；
随机弃手牌优先退随机装备=蓝格怪衣 v99 同款**登记差异**）。存档 v24→
**v25**（yuexing_pair 追加至 PlayerState 段）。版本 158→**159** 入
canonical_hash——14 golden 重录。回归：
`longyou_yuexing_distance_swap_and_clear`、
`longyou_minsha_thunder_and_neighbor_discard`。证据见
[xianjian 第十八切片](../operations/evidence/2026-10-09-xianjian-longyou-batch/README.md)。

**规则 v160（2026-10-09，U05 xianjian 第十九切片：明绣灵火+归烬）**：
灵火（linghuo→host:xj_linghuo，tag 171，global PhaseEnd 确认门=上游
check 询问式）——本回合造成过伤害的其他角色阶段结束对其 1 火伤，每 2
轮冷却（`PlayerState.linghuo_ready_round`=触发后 round+2，**登记差异**：
以轮数冷却近似上游 round 标签）。回合伤害统计
`PlayerState.turn_damage`（commit_damage 实扣后按来源累加，护甲/机甲吸
收后口径）；**清零点=各玩家自己 Judge begin**——Phase{begin:false} 帧
的 Commit（收束+下一回合开始）先于其 AfterTriggers 执行，收束清零会使
阶段末触发读到已清零统计（机制登记）。归烬（guijin→host:xj_guijin，
tag 172，出牌阶段主动每 3 轮冷却）——亮牌堆顶 4 张逐张交付任意角色
（`PlayerState.skip_next_draw` 获得者跳下次摸牌阶段，Draw begin 消费；
可取消弃置剩余；**登记差异**：逐张交付=上游 chooseCardButton 可批
量）。**登记暂缓**：chengxin（濒死技能型救援通道缺口——
DyingSaveMachine 仅询问桃类牌）。存档 v25→**v26**（turn_damage/
linghuo_ready_round/guijin_ready_round/skip_next_draw 追加至
PlayerState 段）。版本 159→**160** 入 canonical_hash——14 golden 重
录。回归：`mingxiu_linghuo_round_cooldown_and_fire`、
`mingxiu_guijin_deal_and_skip_draw`。证据见
[xianjian 第十九切片](../operations/evidence/2026-10-09-xianjian-mingxiu-batch/README.md)。

**规则 v161（2026-10-09，U05 xianjian 第二十切片：云天河逐月+贯日）**：
逐月（zhuyue→host:xj_zhuyue，tag 173，出牌阶段限一次主动）——弃一张非
基本牌，指定至多两名有牌其他角色各随机弃一张（手牌优先退随机装备=蓝
格怪衣同款**登记差异**）；本出牌阶段你的杀须以逐月目标为目标且距离豁
免——`TurnState.zhuyue_targets` 白名单（PlayCard 校验块白名单拒绝+距
离豁免项，贴房绅视/飞刃同区；commit_phase Play 尾钩清=上游 zhuyue2
phaseUseEnd 同口径）。贯日（guanri→host:xj_guanri，tag 174，limited
一次性主动=`PlayerState.guanri_used`）——弃两张红手牌→失去 1 点体力
→指定体力≥自己的其他角色受 2 火伤，存活则弃其装备区所有牌（
revoke_equip_skills 逐张）。**登记暂缓**：longxi（chooseToRespond/
chooseToUse 响应询问链缺口）。**新增缺口登记**：Recover 前置窗口（罗
招嫣 tanhua——Recover 帧无取消语义）；离场区（武将牌上置牌区）缺口族
合并指认：猎空/雪见双刃/紫萱水云/慕容紫英 xuanning。存档 v26→**v27**
（zhuyue_targets Vec 追加至 TurnState 段、guanri_used 追加至
PlayerState 段）。版本 160→**161** 入 canonical_hash——14 golden 重
录。回归：`yuntianhe_zhuyue_whitelist_and_clear`、
`yuntianhe_guanri_once_fire_and_equip_strip`。证据见
[xianjian 第二十切片](../operations/evidence/2026-10-09-xianjian-yuntianhe-batch/README.md)。

**规则 v162（2026-10-09，U05 xianjian 第二十一切片：离场区基建+紫萱
水云系）**：**离场区基建**——`PlayerState.off_court: Vec<u32>` 武将牌
上置牌区（实体牌守恒：离开全部正常区、随所有者存放；snap validate_core
check_cards 纳入；machine `hand_to_off_court`/`off_court_discard` 助
手；`debug_off_court_place` 布阵）。水云（shuiyun→host:xj_shuiyun，
tag 175，player phaseEnd direct 机内取消）——从手中选一张与离场区类型
（基本/锦囊/装备=yangpijuan_class 三分类）不重复的牌置入离场区（≤3）。
水云·救（shuiyun5→host:xj_shuiyun5，tag 176，global Dying 确认门）——
**濒死救援通道首例**（chengxin 同族后续接入）：确认后必选离场区一张牌
弃置，濒死者回复 1，`shuiyun_count`++（小蛮 shenwu 链依赖；**登记差
异**：触发时点=Dying 帧确认（救援机之前），上游=求桃询问内选项——功
能等价机制登记）。忘忧（wangyou→tag 177+wangyou2→tag 178 global
Damage after silent）——受伤者获回合戳 `wangyou3_round`；他人阶段末可
弃一张手牌令标记者们各摸 1（**登记差异**：戳保留至标记者自己下回合
开始——上游 addTempSkill 获标回合结束，原生 End 帧 Commit 先于
AfterTriggers、收束清零会先于触发抹除）。**登记暂缓**：changnian（死
亡拦截钩+跨将运行时授技复合缺口）。**勘误**：慕容紫英移出离场区缺口
族（xuanning 为计数标记且上游无获得路径）。存档 v27→**v28**（
off_court/shuiyun_count/wangyou3_round 追加至 PlayerState 段）。版本
161→**162** 入 canonical_hash——14 golden 重录。回归：
`zixuan_shuiyun_off_court_and_type_dedup`、
`zixuan_shuiyun5_dying_rescue_channel`、
`zixuan_wangyou_roundstamp_and_draw`。证据见
[xianjian 第二十一切片](../operations/evidence/2026-10-09-xianjian-zixuan-batch/README.md)。

**规则 v163（2026-10-09，U05 xianjian 第二十二切片：小蛮灵泉+神雾）**：
灵泉（lingquan→host:xj_lingquan，tag 179，phaseEnd forced
juexingji）——轮次≥3（TurnState.round）且全局用牌数>体力
（`PlayerState.total_used`，commit_card_used 未取消使用递增=
getHistory("useCard") 同口径）时觉醒：摸 3+**runtime 授水云实例**
（skills.insert——跨将授技首例，石会粉禁牌同源；上游 addSkill 同口
径）+内联水云置牌询问（类型去重可放弃=上游 createTrigger 立即触发同
效）。神雾（shenwu→host:xj_shenwu，tag 180，global phaseEnd forced
juexingji）——水云计数≥3 时觉醒：体力上限+1、回复 1、runtime 授回梦
实例。anwugu 与阿奴同技 mapping 复用。存档 v28→**v29**（total_used/
lingquan_awakened/shenwu_awakened 追加至 PlayerState 段）。版本
162→**163** 入 canonical_hash——14 golden 重录。回归：
`xiaoman_lingquan_awaken_grants_shuiyun`、
`xiaoman_shenwu_awaken_grants_huimeng`。证据见
[xianjian 第二十二切片](../operations/evidence/2026-10-09-xianjian-xiaoman-batch/README.md)。

**规则 v164（2026-10-09，U05 xianjian 第二十三批：明绣诚信补完）**：
诚信（chengxin→host:xj_chengxin，tag 181，global Dying 确认门=濒死救
援通道第二例，round:4 冷却=`chengxin_ready_round`=触发后 round+4）——
濒死者回复至 1（recover 量=1-当前体力，负体力一次拉回）+免伤
（`chengxin_active`：commit_damage 急冻同区直钩，受所有伤害无效=上游
chengxin2 damageBefore cancel 同口径；清钩=本人 End 阶段结束，**登记差
异**：上游存活至最近阶段结束）。v160 暂缓消账，明绣升全闭环。存档
v29→**v30**（chengxin_active/chengxin_ready_round 追加至 PlayerState
段）。版本 163→**164** 入 canonical_hash——14 golden 重录。回归：
`mingxiu_chengxin_rescue_and_immunity`。证据见
[xianjian 第二十三批](../operations/evidence/2026-10-09-xianjian-chengxin-batch/README.md)。

**规则 v165（2026-10-09，U05 xianjian 第二十四批切片：雪见神暮+毒
刺）**：神暮（shenmu→host:xj_shenmu，tag 182，global Dying 确认门=濒
死救援通道第三例）——弃全部红手牌，濒死者回复 1 并摸等量张（上游
showHandcards 纯表现不迁移；**登记差异**：check 询问简化为红手牌有无
filter）。毒刺（duci→host:xj_duci，tag 183，CardsDiscarded direct）——
弃牌含装备 kind（yangpijuan_class=2）入弃堆时，询问距离 1 内一名其他
角色受 1 伤害（chooseTarget 取消=放弃；**登记差异**：仅弃置路径触发
（CardsDiscarded 两发射点），被获得/替换不触发=上游 loseEnd 全失去面
收窄）。**登记暂缓**：xshuangren（失去武器置武将牌+addAdditionalSkill
武器技能转移复合缺口）。无新状态字段。版本 164→**165** 入
canonical_hash——14 golden 重录。回归：
`xuejian_shenmu_red_discard_rescue`、
`xuejian_duci_equip_discard_damage`。证据见
[xianjian 第二十四批切片](../operations/evidence/2026-10-09-xianjian-xuejian-batch/README.md)。

**规则 v166（2026-10-09，U05 xianjian 第二十五批切片：柳梦璃润心）**：
润心（runxin→host:xj_runxin，tag 184，TriggerMeta **extra_events 双事
件首例**：event=CardUsed+extra_events=[CardResponded]=废墟双事件同
族）——自己使用/打出红桃牌（仅红桃，suit==Heart）后令一名已受伤角色
回复 1（chooseTarget 取消=放弃）。**登记暂缓**：tianxian（杀距离/目标
数 mod+useCardToBefore 逐目标判定取消——mod 系+逐目标窗口复合缺口）、
xjzhimeng（暗牌标记+useCardToBegin 同类型消耗——跨玩家持牌暗标记缺
口）。无新状态字段。版本 165→**166** 入 canonical_hash——14 golden
重录。回归：`liumengli_runxin_heart_use_and_respond`。证据见
[xianjian 第二十五批切片](../operations/evidence/2026-10-09-xianjian-liumengli-batch/README.md)。

**规则 v167（2026-10-09，U05 xianjian 第二十六批切片：柳梦璃织梦系）**：
织梦（xjzhimeng→host:xj_xjzhimeng，tag 185，player phaseEnd direct 机
内取消）——牌堆顶一张暗置于一名其他角色武将牌上（off_court 离场区复
用第二次消费）+runtime 授织梦·梦实例（灵泉同款）。织梦·梦
（xjzhimeng2→host:xj_xjzhimeng2，tag 186，CardUsed frequent silent 自
动）——暗牌持有者被使用同类型（yangpijuan_class 三分类）牌时摸 1（暗
牌保留=上游 storage 不清除同口径；**登记差异**：自目标牌（无中类
targets 空）以使用者为隐式目标=上游 target 事件含自目标）。柳梦璃升全
闭环（仅 tianxian 仍暂缓）。无新状态字段。版本 166→**167** 入
canonical_hash——14 golden 重录。回归：
`liumengli_xjzhimeng_darkmark_and_draw`。证据见
[xianjian 第二十六批切片](../operations/evidence/2026-10-09-xianjian-zhimeng-batch/README.md)。

**规则 v168（2026-10-09，U05 xianjian 第二十七批切片：南宫煌摄灵）**：
摄灵（sheling→host:xj_sheling，tag 187，TriggerMeta **三事件首例**：
event=CardUsed+extra_events=[CardResponded, CardsDiscarded]，frequent
silent 自动）——自己回合内、他人使用/打出/弃置的牌入弃堆时回收之
（`TurnState.sheling_used` 每回合 3 次=上游 usable:3 同口径，Judge
begin 清；羊皮卷同款离堆入手）。**登记差异**：仅三事件面（使用/打出/
弃置），其他入弃堆路径（装备替换等）不触发=上游 position:"d" 全失去
面收窄；上游 check 排除毒不迁移（毒牌种不在目录）。**登记暂缓**：
zhaoyao（global phaseDrawBegin 拼点时序偏差——原生 PhaseBegin 触发晚
于固定摸牌=摸后拼点功能性偏离）、zhangmu（响应询问链缺口）。存档
v30→**v31**（sheling_used 追加至 TurnState 段）。版本 167→**168** 入
canonical_hash——14 golden 重录。回归：
`nangonghuang_sheling_three_event_reclaim`。已通基建批量解锁优先级排
序见[台账](../migration/upstream-extension-reconstruction.md)与
[本批证据](../operations/evidence/2026-10-09-xianjian-sheling-batch/README.md)。

**规则 v169（2026-10-09，U05 xianjian 第二十八批切片：虚拟 useCard 原
语+王小虎虎煞）**：**虚拟 useCard 原语**（machine
`use_virtual_card(source, as_kind, targets)`）——以牌堆底一张实体为载
体、use_as 声明牌种发起完整使用管线（响应/伤害/弃置走 commit
effective 分派既有机制=武圣/龙胆 view-as 同路径；**登记差异**：载体
实体结算后入弃堆[上游凭空造牌无实体不入弃堆]、载体取自牌堆底[确定非
随机]）。虎煞·集（husha_count→tag 188，source Damage forced）——自己
出牌回合内伤人 +伤害量虎煞标记（`PlayerState.husha_mark` ≤3=上游
storage.husha 同口径）。虎煞（husha→tag 189，phaseEnd direct 机内取
消）——消耗标记虚拟用牌：1 枚视为杀（chooseTarget）/2 枚视为南蛮入
侵（AOE 全其他存活；use_virtual_card 首例）；3 枚元素毁灭牌种不存在
目录=**登记不转**。存档 v31→**v32**（husha_mark 追加至 PlayerState
段）。版本 168→**169** 入 canonical_hash——14 golden 重录。回归：
`wangxiaohu_husha_marks_and_virtual_cards`、
`wangxiaohu_husha_virtual_nanman`。证据见
[xianjian 第二十八批切片](../operations/evidence/2026-10-09-xianjian-husha-batch/README.md)。

**规则 v170（2026-10-09，U05 xianjian 第三十批：死亡前置窗口+紫萱长
念）**：**死亡前置窗口基建**——Death 帧起步 Intrinsic→**BeforeTriggers**
（四点改造=TurnOverBefore E03 同款：push_frame、**机器侧 Ctx::emit**
[独立 stage 映射，漏改则 DyingSaveMachine 发射的 Death 绕过窗口——本
批实测根因]、next_stage Before→After[静默]→Intrinsic→Done、
plan_triggers before_kind 特判+After 对 Death 静默）。长念
（changnian→host:xj_changnian，tag 190，Death Before 窗口 direct 机内
取消）——死亡时选一名其他角色移交 hej 全部牌（全部入手，装备 revoke
后）+runtime 授长念·思实例；DeathMachine 照常执行（遗言型不取消死
亡）。长念·思（changnian2→tag 191，phaseEnd forced，runtime 授技实
例）——结束阶段摸 1。**来源勘误**：changnian 属紫萱（v162 暂缓登记）
非柳梦璃——紫萱升全闭环。**登记暂缓**：月琦 shiying（取消型：需
PlayerState 死亡拦截标记+DeathMachine 直读跳过——窗口已通待单切）。
无新状态字段。版本 169→**170** 入 canonical_hash——14 golden 重录。
回归：`liumengli_changnian_death_bequest`。证据见
[xianjian 第三十批](../operations/evidence/2026-10-09-xianjian-changnian-batch/README.md)。

**规则 v171（2026-10-09，U05 xianjian 第二十九批切片：月琦适盈死亡取
消型）**：适盈（shiying→host:xj_shiying，tag 192，Death Before 窗口
limited forced）——**死亡取消型首例**：`PlayerState.death_cancelled`
拦截标记（DeathMachine step 开头直读：置位则清除并跳过死亡执行=上游
dieBefore trigger.cancel 同口径，存活态保持）+ limited 一次性
（`shiying_used`）——确认后双方体力上限/体力置 3、clearSkills 双方
（SkillToggled 日志；适盈实例自身也被清=上游同款）、弃 hej 全部（装备
revoke）、各摸 3。**登记不转**：tianwu（纯表现+tianwu2 空壳）。**登记
暂缓**：liguang（moveCard 移动场上装备/判定区牌基建）。存档 v32→
**v33**（death_cancelled/shiying_used 追加至 PlayerState 段）。版本
170→**171** 入 canonical_hash——14 golden 重录。回归：
`yueqi_shiying_cancel_death_and_reset`。证据见
[xianjian 第二十九批切片](../operations/evidence/2026-10-09-xianjian-shiying-batch/README.md)。

**规则 v172（2026-10-09，U05 xianjian 第三十批切片：南宫煌照妖）**：
照妖（zhaoyao→host:xj_zhaoyao，tag 193，global Draw-begin 确认门
direct）——确定性首张拼点（舌战群儒同款**登记差异**：双方 hand.first
比点，无选牌询问）→赢→摸 2+选两张手牌置牌堆顶（先选在上=后选的先压
栈）；输/平无事。**登记差异**：上游 global phaseDrawBegin 在摸牌前拼
点，原生线 PhaseBegin AfterTriggers 在固定摸牌后=摸后拼点（双方各含
新摸 2 张）。南宫煌升全闭环（仅 zhangmu 响应链暂缓）。无新状态字段。
版本 171→**172** 入 canonical_hash——14 golden 重录。回归：
`nangonghuang_zhaoyao_compare_and_top`。证据见
[xianjian 第三十批切片](../operations/evidence/2026-10-09-xianjian-zhaoyao-batch/README.md)。

**规则 v173（2026-10-09，U05 xianjian 第三十一批切片：月琦离光
moveCard 基建）**：**moveCard 基建首例**（host 步内确定性扫描）——装
备→首个空同栏目的地（revoke_equip_skills+grant_equip_skills 技能随
迁+Equipped 日志）；判定区→首个无同名牌目的地（CardsGiven 日志）；
**登记差异**：上游 moveCard(true)=AI 自动选取，原生=确定性首张可移牌
+首个合法目的地（座位序）。离光（liguang→host:xj_liguang，tag 194，
phaseEnd direct 机内取消）——弃一张手牌（可取消=放弃）并移动场上的
一张牌。月琦升全闭环（tianwu 登记不转）。无新状态字段。版本 172→
**173** 入 canonical_hash——14 golden 重录。回归：
`yueqi_liguang_discard_and_move_equip`。证据见
[xianjian 第三十一批切片](../operations/evidence/2026-10-09-xianjian-liguang-batch/README.md)。

**规则 v174（2026-10-09，U05 xianjian 第三十二批切片：实体发现基建+
夏侯瑾轩玄墨）**：**实体发现基建首例**（createCard 缺口族差异方案，
台账待建排序③）——「凭空造牌」以「牌堆+弃堆实体池随机发现同类型
（yangpijuan_class 三分类）实体」替代（RNG 洗牌；**登记差异**：上游
createCard 凭空造牌无实体来源，原生从实体池发现；实体不足给余量）。
玄墨（xuanmo→host:xj_xuanmo，tag 195，phaseUse usable:1 主动）——弃一
张基本/装备/锦囊手牌置牌堆顶，随机获得实体池同类型 2 张入手（排除置
顶牌；不足给余量）。**登记不转**：danqing（四花色收集→getBuff(false)
纯表现目标选择，无机制本体）。无新状态字段。版本 173→**174** 入
canonical_hash——14 golden 重录。回归：
`xiahoujinxuan_xuanmo_discover_same_class`。证据见
[xianjian 第三十二批切片](../operations/evidence/2026-10-09-xianjian-xuanmo-batch/README.md)。

**规则 v175（2026-10-09，U05 xianjian 第三十三批切片：苏媚射影+花
好）**：射影（sheying→host:xj_sheying，tag 196，source Damage forced
每回合 1 次=`TurnState.sheying_used`，Judge begin 清）——伤害牌入弃堆
时判定，黑色→从弃堆回收伤害牌入手（cards 分流带入=武圣/龙胆 view-as
分流同路径）。花好（huahu→host:xj_huahu，tag 197，phaseUse limited
多目标主动）——失 1 体力上限+clearSkills 自己（适盈同款 SkillToggled
日志），令任意名其他角色各回复 1+获 1 护甲+摸 1（逐目标 Yield）；
`PlayerState.huahu_used` 一次性。**登记暂缓**：dujiang（毒牌种不在目
录，实体发现亦不可用）。存档 v33→**v34**（sheying_used 追加至
TurnState 段、huahu_used 追加至 PlayerState 段）。版本 174→**175** 入
canonical_hash——14 golden 重录。回归：
`sumei_sheying_black_judge_reclaim`、
`sumei_huahu_sacrifice_multi_target`。证据见
[xianjian 第三十三批切片](../operations/evidence/2026-10-09-xianjian-sumei-batch/README.md)。

**规则 v176（2026-10-09，U05 xianjian 第三十四批切片：仙卿探息）**：
探息（xtanxi→host:xj_xtanxi，tag 198，phaseUse usable:1 主动）——弃一
张手牌，持有同名牌手牌的其他角色中随机一名，随机获得其一张手牌（无同
名牌持有者则牌照弃；**登记差异**：同名牌判定按牌种=上游 get.name 同口
径、addExpose 亮牌纯表现不迁移）。**登记暂缓**：xiaoyue（roundStart
事件缺口+自动虚拟用牌复合缺口）。无新状态字段。版本 175→**176** 入
canonical_hash——14 golden 重录。回归：
`xianqing_xtanxi_same_name_steal`、
`xianqing_xtanxi_no_holder_cancel`。证据见
[xianjian 第三十四批切片](../operations/evidence/2026-10-09-xianjian-xtanxi-batch/README.md)。

**规则 v177（2026-10-09，U05 xianjian 第三十五批切片：罗招嫣凤略）**：
凤略（fenglue→host:xj_fenglue，tag 199，Play-begin 确认门 direct 机内
取消）——出牌阶段开始选一名其他角色，逐张自动对其使用手中可指定其的
杀（距离内）与决斗（**真实牌自动使用首例**：emit CardUsed 真实牌，
commit take_hand_card 自取——机器侧不预取；逐张 Yield 后重查存活）；
**登记差异**：仅杀与决斗自动使用，其余锦囊不自动=上游 targetEnabled2
全量收窄。**登记暂缓**：tanhua（Recover 前置窗口缺口——回复改摸牌需
回复取消语义）。无新状态字段。版本 176→**177** 入 canonical_hash——
14 golden 重录。回归：`luozhaoyan_fenglue_auto_use_sha_duel`。证据见
[xianjian 第三十五批切片](../operations/evidence/2026-10-09-xianjian-fenglue-batch/README.md)。

**规则 v178（2026-10-09，U05 xianjian 第三十六批切片：姜云凡疾风+天
剑）**：疾风（liefeng→host:xj_liefeng，tag 200，CardUsed forced）——出
牌回合内第 3/4 张使用牌时 runtime 授天剑/御风实例（灵泉同款；
`cards_used_play_phase` 计数；**登记差异**：第 2 张授炎杀 yanzhan
viewAs 暂缓不授）。天剑（tianjian→host:xj_tianjian，tag 201，phaseUse
usable:1 主动，liefeng 授技实例）——弃一张手牌杀视为使用万箭齐发（
**虚拟 AOE 原语消费**：use_virtual_card ArcheryAttack 全其他存活）+
伤害目标随机弃 he 一张（**登记差异**：discard 子技直连=上游为
damageEnd 子技）。**登记暂缓**：xunying（shaAfter 响应链缺口）。无新
状态字段。版本 177→**178** 入 canonical_hash——14 golden 重录。回归：
`jiangyunfan_liefeng_grant_and_tianjian_aoe`。证据见
[xianjian 第三十六批切片](../operations/evidence/2026-10-09-xianjian-liefeng-batch/README.md)。

**规则 v179（2026-10-09，U05 xianjian 第三十七批切片：慕容紫英破云）**：
破云（poyun→host:xj_poyun，tag 202，source Damage forced direct）——玄
砚计数>0 且受伤目标 he 有牌时，令目标自选弃 he 1-2 张并计数 -1
（`PlayerState.xuanning_mark`；**登记差异**：xuanning 计数获得路径上
游缺失——原生仅支持调试/宿主注入）。**登记暂缓**：qianfang（AOE 万箭
虚拟已通但依赖 xuanning 获得路径）。无其他新状态字段。存档 v34→
**v35**（xuanning_mark 追加至 PlayerState 段）。版本 178→**179** 入
canonical_hash——14 golden 重录。回归：
`murongziying_poyun_discard_and_decrement`。证据见
[xianjian 第三十七批切片](../operations/evidence/2026-10-09-xianjian-poyun-batch/README.md)。

**规则 v180（2026-10-09，U05 xianjian 第三十八批切片：雪见双刃系）**：
双刃（xshuangren→host:xj_xshuangren，tag 203，CardsDiscarded forced）—
—弃牌含武器 kind 时，从弃堆回收该武器置离场区（off_court）并
grant_equip_skills 武器技能随迁；**登记差异**：仅弃置路径触发（被替
换/被获得不触发=上游 loseEnd 全失去面收窄）、武器技能随迁按 grant 全
量、殁清离场区全部=通用区共用语义。双刃·殁（xshuangren2→tag 204，
Death 前置窗口 forced）——死亡时弃置离场区全部牌。无新状态字段
（off_court 复用）。版本 179→**180** 入 canonical_hash——14 golden
重录。回归：`xuejian_xshuangren_offcourt_and_death_discard`。证据见
[xianjian 第三十八批切片](../operations/evidence/2026-10-09-xianjian-xshuangren-batch/README.md)。

**规则 v181（2026-10-10，U05 xianjian 第三十九批切片：南宫煌张目）**：
张目（zhangmu→host:xj_zhangmu，tag 205，DamageBefore forced 每回合 1 次
=`TurnState.zhangmu_used`，Judge begin 清）——需要打出闪响应时弃一张手
牌闪直接视为已响应（杀不命中）。**登记差异**：①响应询问改写简化为伤
害前置直防（上游 chooseToRespondBegin untrigger+responded 语义在响应询
问内，原生 DamageBefore 窗口 prevented 直写——杀不命中等价）②弃哪张
闪确定性取首张闪（上游玩家自选）。无其他新状态字段。存档 v35→**v36**
（zhangmu_used 追加至 TurnState 段）。版本 180→**181** 入
canonical_hash——14 golden 重录。回归：
`nangonghuang_zhangmu_flash_discard_prevent`。证据见
[xianjian 第三十九批切片](../operations/evidence/2026-10-10-xianjian-zhangmu-batch/README.md)。

**规则 v182（2026-10-10，U05 xianjian 第四十批切片：慕容紫英千方）**：
千方（qianfang→host:xj_qianfang，tag 206，player phaseBegin[Judge] direct
机内取消）——玄砚计数换弃牌：num=3-`xuanning_mark`（clamp≥0），num>0
时弃 num 张 he（可取消=放弃）、num=0 直接 AOE；xuanning 清零后视为对
全部其他角色使用万箭齐发（**虚拟 AOE 原语第三次消费**：use_virtual_
card ArcheryAttack）；被伤敌人每名紫英摸 1（qianfang_draw 直连；**登
记差异**：上游 damageEnd 逐目标即时，原生结算后统一摸）。**登记差
异**：xuanning 计数获得路径上游缺失（v179 已登记）。无新状态字段。版
本 181→**182** 入 canonical_hash——14 golden 重录。回归：
`murongziying_qianfang_discard_and_aoe`。证据见
[xianjian 第四十批切片](../operations/evidence/2026-10-10-xianjian-qianfang-batch/README.md)。

**规则 v183（2026-10-10，U05 xianjian 第四十一批切片：姜云凡迅影）**：
迅影（xunying→host:xj_xunying，tag 207，CardUsed AfterTriggers
[shaAfter] direct）——杀结算后 filter（自己使用杀+目标存活+距离内+
有手牌杀）成立时自动取手牌首张杀对同一目标 emit CardUsed 再杀（真实
牌 commit 自取，机器不预取）；链式再杀终止于无杀/目标亡/超距。
**登记差异**：①上游 chooseToUse 玩家自选可放弃，原生自动取首张杀直
到无杀②上游 canUse 简化为 computed_distance≤attack_range③酒状态保
留不迁移（上游 removeSkill jiu→chooseToUse 未用还原）④再杀不经
PlayCard 限次校验（上游 chooseToUse 第 4 参 -1 同口径）。无新状态字
段。版本 182→**183** 入 canonical_hash——14 golden 重录。回归：
`jiangyunfan_xunying_auto_second_sha`。证据见
[xianjian 第四十一批切片](../operations/evidence/2026-10-10-xianjian-xunying-batch/README.md)。


**规则 v184（2026-10-10，U05 xianjian 第四十二批切片：柳梦璃天弦）**：
天弦（tianxian→host:xj_tianxian，tag 208，锁定旗标行为=效果内联无机
器，铁掠同款）三件：①杀无视距离（engine sha_range_free 臂，上游 mod
targetInRange 同口径）②杀目标数上限无限（engine max_targets 臂，上游
mod selectTarget range[1]=∞ 同口径——sha 有限上限必满足其 !==-1 条
件）③逐目标判定黑取消（杀机器 !asked 臂顶=每目标窗口位，上游
useCardToBefore 同窗口；**登记差异**：铁骑同款内联确定性判定，无鬼
道/天妒改判窗）。无新状态字段。版本 183→**184** 入 canonical_hash
——14 golden 重录。回归：
`liumengli_tianxian_range_targets_and_judge_cancel`（无技距离/双目标
双拒绝→授技距离 2 双目标合法；判红目标正常结算、判黑目标取消不问
闪）。证据见
[xianjian 第四十二批切片](../operations/evidence/2026-10-10-xianjian-tianxian-batch/README.md)。


**规则 v185（2026-10-10，U05 xianjian 第四十三批切片：越今朝影锋）**：
影锋（xjyingfeng→host:xj_xjyingfeng，tag 209，CardUsed AfterTriggers
forced）——杀结算完毕后随机对一名非本杀目标的其他角色使用虚拟杀
（use_virtual_card 原语第四次消费=牌堆底载体完整管线）。防递归：
TurnState.yingfeng_chain 链挡旗标（影锋机器 emit 前置位→虚拟杀的
AfterTriggers filter 拒→Judge begin 清；存档 v36→**v37**）。**登记
差异**：①敌方=全部其他角色（prototype 无阵营）②上游 canUse 简化为
距离口径③同回合多张真实杀仅首次触发（上游 parent.name 只挡影锋自
己的杀——真杀 use_as 实测本为 Some(Sha)，转化声明挡递归不可行）。
雾魂（额外回合+全局牌复原）暂缓登记。版本 184→**185** 入
canonical_hash——14 golden 重录。回归：
`yuejinzhao_xjyingfeng_random_virtual_sha`。证据见
[xianjian 第四十三批切片](../operations/evidence/2026-10-10-xianjian-yingfeng-batch/README.md)。


**规则 v186（2026-10-10，U05 xianjian 第四十四批切片：居十方械磐）**：
械磐（xiepan→host:xj_xiepan，tag 210，CardsDiscarded forced 机内取
消）——弃牌含基本牌+手牌已无基本牌+手牌非空（失去最后一张基本牌）
时可取消询问弃一张手牌，实体发现随机装备入手（羊皮卷同款实体池=牌
堆+弃堆 RNG 发现，**第二次消费**）。**登记差异**：①弃置路径触发
（上游 loseEnd 覆盖一切失去路径=双刃同款收窄）②弃牌来源不分手牌/
装备（CardsDiscarded 无 original 标记）③获得=实体池发现不足无（替
代上游 createCard 凭空造牌）。无新状态字段。版本 185→**186** 入
canonical_hash——14 golden 重录。回归：
`jushifang_xiepan_discard_and_discover_equip`。证据见
[xianjian 第四十四批切片](../operations/evidence/2026-10-10-xianjian-xiepan-batch/README.md)。


**规则 v187（2026-10-10，U05 xianjian 第四十五批切片：长卿乱剑）**：
乱剑（luanjian→host:xj_luanjian，tag 211，phaseUse once_per_turn 主
动=天剑命令层结构）——两张杀弃置→虚拟杀任意名多目标（use_virtual_
card 原语第五次消费；机器直发不经 PlayCard 校验=上游「无视距离+可
指定任意多目标」同效果，**登记差异**：目标侧防具检查不适用、素材已
弃不退[目标必选 allow_cancel=false]）。乱剑·增伤（luanjian2→host:
xj_luanjian2，tag 212，DamageBefore forced）——载体牌锚定
（TurnState.luanjian_card=乱剑虚拟杀载体，普通杀/其他虚拟技载体不
命中）50% 伤害+1，每目标独立 roll（上游 Math.random 同 RNG 流）；存
档 v37→**v38**。**登记暂缓**：天符（挂附实体+伤害后窗+死亡清理）。
版本 186→**187** 入 canonical_hash——14 golden 重录。回归：
`changqing_luanjian_dual_sha_viewas_multi_target`（seed 10062 混合
roll：目标 1 无增伤/目标 2 增伤 2 点=臂真实执行+逐目标独立）。证据
见[xianjian 第四十五批切片](../operations/evidence/2026-10-10-xianjian-luanjian-batch/README.md)。


**规则 v188（2026-10-10，U05 xianjian 第四十六批切片：长卿天符）**：
天符（ctianfu→host:xj_ctianfu，tag 213，phaseUse once_per_turn 主动）
——选一张闪跨玩家置**目标**离场区（武将牌上承载=上游 discard:false
同口径）+同授 ctianfu2/ctianfu3 双实例（上游 group 同授）+记来源
（PlayerState.ctianfu_source/ctianfu_card；存档 v38→**v39**）。天
符·反噬（ctianfu2，tag 214，Damage 帧 AfterTriggers forced）——挂附
者造成伤害后受来源 1 雷伤（来源亡=无来源）+随机弃 he 一张+挂附闪入
弃堆+清挂附；触发即清=双向挂附递归链天然终止。天符·清挂（ctianfu3
，tag 215，Death 窗 forced）——死亡清挂附；**登记差异**：挂附闪入
弃堆（上游凭空消失）。版本 187→**188** 入 canonical_hash——14
golden 重录。回归：`changqing_ctianfu_attach_thunder_and_death_
clear`（双场景：挂附断言[闪离手入目标离场区+双实例+来源]→挂附者杀
人反噬雷伤+全清；死亡清挂）。证据见
[xianjian 第四十六批切片](../operations/evidence/2026-10-10-xianjian-ctianfu-batch/README.md)。


**规则 v189（2026-10-10，U05 swd 第十六批：机关牌种+指路小虎）**：
`CardKind::Jiguan` 编码 **132**（swd 转换包 ♣4，花色点数手补=护符同
口径）。指路小虎（SwdZhiluxiaohuCard，tag 216）——自目标使用
（PlayCard 校验 targets==[current]，上游 selectTarget -1+filterTarget
target===player 同口径）→无懈窗口包装→随机对一名其他存活角色使用
虚拟杀（use_virtual_card 第六消费）。**登记差异**：①敌方=全部其他
角色（prototype 无阵营）②上游 enable=能用杀门不校验。机关牌族解锁
路径首格（御甲/偃师需木甲人等更多机关牌+计数，续暂缓）。swd 存量
回归修复：发牌敏感注入补三区 retain（deck-composition-test-traps 同
款）+计数 25→26。版本 188→**189** 入 canonical_hash——14 golden 重
录。回归：`swd_zhiluxiaohu_random_enemy_virtual_sha`。证据见
[xianjian 第四十六批同目录](../operations/evidence/2026-10-10-xianjian-ctianfu-batch/README.md)。


**规则 v190（2026-10-10，U05 swd 第十七批：木甲人）**：
木甲人（SwdMujiarenCard，tag 217，卡牌机器=无懈窗包装）——**同
kind 异行为按目录名分流**（kernel Jiguan 臂含 mujiaren→木甲人，否
则指路小虎）。usable:1（`TurnState.mujiaren_used`，Judge begin 清，
存档 v39→**v40**；PlayCard CardLimit=酒口径）。效果：弃自己全部非
基本手牌（静默移除=上游 lose _triggered=null 同口径，不触发弃牌事
件）→随机机关牌 1+N 张入手（实体发现 Jiguan 池=牌堆+弃堆、排除本
体、不足给余量=上游 createCard 凭空造牌登记差异）。花色点数 ♥4 手
补。**登记差异**：机关牌池当前 2/4+（勾魂锣/机关桶待切片，发现池
随切片扩充）。swd 存量计数 26→27。版本 189→**190** 入
canonical_hash——14 golden 重录。回归：
`swd_mujiaren_discard_nonbasic_discover_jiguan`。证据见
[README-v190](../operations/evidence/2026-10-10-xianjian-ctianfu-batch/README-v190.md)。


**规则 v191（2026-10-10，U05 swd 第十八批：勾魂锣）**：
勾魂锣（SwdGouhunluoCard，tag 218，卡牌机器=无懈窗包装；♠9/♣7 上
游原文花色）——目标恰 1 名无挂附角色（PlayCard 校验
gouhunluo_rounds==0，上游 filterTarget !hasSkill 同口径）挂 3 阶段
倒计时+授实例（PlayerState.gouhunluo_rounds/source；存档 v40→
**v41**）。倒计时（SwdGouhunluoSub，tag 219，PhaseBegin forced
global 定向）——来源回合各阶段开始-1，归 0=目标失去 1 体力
（LoseHp）+弃全部手牌（真实弃置事件=械磐可触发）+清挂附清实例；
kind 级 def 注册=card_kind_host_skills(Jiguan) 首例。**登记差异/说
明**：①来源死亡后标记残留但 filter 不满足=永不触发（上游 dieBegin
显式清除，效果等价）②上游 intro 文案称「轮」但实现按阶段计，原生
跟随实现。机关牌池勘误：上游 6 种 13 张，已切 3 种 5 张。版本
190→**191** 入 canonical_hash——14 golden 重录。回归：
`swd_gouhunluo_countdown_lose_hp_discard`（挂附三断言+重复挂拒+循
环推进至归 0 结算三断言）。证据见
[README-v191](../operations/evidence/2026-10-10-xianjian-ctianfu-batch/README-v191.md)。

**规则 v192（2026-10-10，U05 swd 第十九批：机关桶，2026-10-10 补登）**：
机关桶（上游 `jiguantong`「机关火筒」，♦4/♣7，`SwdJiguantongCard` tag
220，与其余机关牌共用无懈窗包装）——PlayCard 校验目标须恰为当前回合角色
起座位序的全部其他存活角色（上游 selectTarget -1、target!==player）；逐目
标：有手牌者选弃一张手牌（可取消=受 1 点火伤，上游 chooseToDiscard 同口
径），无手牌者直接受 1 点火伤，本体入弃堆。**登记差异**：①结算序=座位
序（上游 `reverseOrder` 逆序）②**未实现** contentAfter「若没有人选择受
到伤害，使用者摸一张牌」（上游 translate 与 card.js `preResult` 同此语义；
原生机器无摸牌分支，测试未覆盖）。机关牌池：上游 6 种 13 张，已切 4 种 7
张（余机关术×2/机关妖术×2）。swd 转换包 30 张（含机关牌 6 张）。版本
191→**192**，存档格式不变（v41）。本批开发中 host.rs 被截断，事后从会话
记录重建；回归 `swd_jiguantong_aoe_discard_or_fire`（弃牌免伤/空手火伤/本
体入弃堆）2026-10-10 通过。14 golden 补齐 v191/v192 两批重录，原因核验
（版本回退探针、轨迹语义探针、行为面全量）见
[v192 恢复证据](../operations/evidence/2026-10-10-replay-golden-v192-recovery/README.md)；
本批无独立批次证据，实现语义以源码与
[2026-10-10 进度核对](../operations/evidence/2026-10-10-progress-check/README.md)为准。

**规则 v193（2026-10-10，T10 机关桶收尾）**：撤销 v192 条目登记的两条机关桶
差异，对齐上游 `jiguantong`：① 结算序改为**逆序**（上游 `reverseOrder`）——
kernel 机关牌工厂臂在装载机器时把 PlayCard 目标表（当前回合角色起座位序）
反转为结算序；② **补上 contentAfter**「若没有人选择受到伤害，使用者摸一张
牌」——机器 Act 完成点按日志实伤判定：本次结算中无以本牌归因的
`DamageDealt`（弃牌免伤、取消弃牌改受伤算受伤；被无懈抵消的目标无结算不
算）则使用者摸 1，随后本体入弃堆（上游 content 在 contentAfter 前逐目标
置 `preResult`，原生以实际结算日志同口径；上游 `damage()` 调用即置位与原生
实伤判定的差别仅当伤害被完全阻止时可达，当前内容无此交互，按实伤判定登
记）。回归：`swd_jiguantong_aoe_discard_or_fire` 补「有人受伤不摸」断言，
新增 `swd_jiguantong_reverse_order_and_content_after_draw`（首个弃牌询问
observer=2 证逆序 + 全员弃牌后使用者摸 1）。版本 192→**193**，存档格式不
变（v41，机器字段未增）。14 golden 全部重录：版本回退探针（193→192 复跑
golden 全绿）证明位移唯一来源是版本字段入 canonical_hash，jiguantong 分支
仅经 swd 牌种、标准场景不可达，逐场景命令与断言不变——证据见
[T10 证据](../operations/evidence/2026-10-10-t10-jiguantong-v193/README.md)。

**规则 v194（2026-10-11，T11 掠影对齐）**：撤销 v154 条目"登记差异：无"的
掠影（lueying，慕长兰）漏登记——上游 `skill.js:1910` filter 要求目标 he
有牌**且另有一名其他有牌角色**（缺第三者则整个技能不可发动，连获得也不
触发），且为**非强制**（usable:1，确认后消耗，放弃不计数、同一杀不再
问、本回合下一杀可再询问）。原生改为两相协议：确认门（observer=使用者，
Confirm/空应答）→ 获得目标 he 随机一张 → 仍有其他有牌者则目标选人
（原 victim 相不变）。机器 `ShaResolve` 尾部追加 `lueying_declined`（放
弃标记，阻止 step 重入再问）——**存档格式 41→42**（v41 字节被版本门禁
拒绝，无生产迁移数据）。版本 193→**194**；14 golden 重录：双版本回退
探针（194/42→193/41 复跑 golden 全绿）证明位移唯一来源是两个版本字段，
掠影分支仅经 xianjian 扩展牌种、标准场景不可达。回归：既有
`muchanglan_lueying_gain_and_victim_discard` 加确认门步；新增
`muchanglan_lueying_no_third_party_never_fires`（无第三者完全不发动）与
`muchanglan_lueying_decline_keeps_usable_for_next_sha`（放弃不消耗、
下一杀可再询问）——证据见
[T11 证据](../operations/evidence/2026-10-11-t11-lueying-v194/README.md)。

**规则 v195（2026-10-11，T12 机关凤切片）**：机关凤（上游 `jiguanfeng`，
♣3/♠4，`SwdJiguanfengCard` tag 221，与其余机关牌共用无懈窗包装）——
PlayCard 校验恰 1 名其他存活角色（上游 filterTarget target!==player）；
目标选择打出一张手牌闪（无闪/取消=未响应，上游 chooseToRespond+自动失败
同口径）；未响应→获得"封印"状态并受 1 点伤害（上游 addTempSkill
"fengyin"+damage 同序）；响应则无事，本体入弃堆。封印=**全部技能实例禁
用，拥有者任一阶段开始解除**（`SwdFengyinState` tag 222，PhaseBegin
forced；登记差异：上游 fengyin=非锁定技失效[library skillBlocker]，原生
无锁定技区分=全部失效，gwent 封印同口径已登记）。**机关牌池勘误**：此前
登记"6 种 13 张"漏计机关凤×2 与机关怨×2——按转换源 `swd/list.js` 权威
实为 **8 种 17 张**（机关术×2/机关妖术×2/机关怨×2/机关桶×2/机关凤×2/
勾魂锣×2/指路小虎×2/木甲人×3）；已切 5 种 11 张，余机关术×2/机关妖术
×2/机关怨×2。版本 194→**195**，存档格式不变（v42，无新机器字段）。
14 golden 重录：版本回退探针（195→194 复跑 golden 全绿）证明位移唯一来
源是版本字段，机关凤分支仅经 swd 牌种、标准场景不可达。回归：
`swd_jiguanfeng_no_shan_seal_then_unseal_at_phase_begin`（无闪→封印+1
伤+阶段开始解除全链）、`swd_jiguanfeng_shan_respond_no_damage`（打闪→
无事、闪与本体入弃堆）——证据见
[T12 证据](../operations/evidence/2026-10-11-t12-jiguanfeng-v195/README.md)。

**规则 v196（2026-10-11，T12 机关怨切片）**：机关怨（上游 `jiguanyuan`，
♣3/♦2，`SwdJiguanyuanCard` tag 223，无懈窗包装）——PlayCard 校验恰 1 名
其他存活角色且目标无待获得状态（上游 filterTarget target!==player+
!hasSkill("jiguanyuan")）、使用者除本体至少还有一张手牌（上游 enable 同
口径）；使用者选一张手牌，与本体一起置于目标武将牌上（离场区
`jiguanyuan_cards` 记录，**本体不进弃堆**），使用者摸 1；目标获得
`SwdJiguanyuanGain`（tag 224，phaseEnd forced）——其下一结束阶段将待获
得牌移入手牌并移除状态（上游 jiguanyuan 技同口径，伴生技在 swd 包内定
义）。死亡清理沿用离场区既有口径。**存档格式 42→43**
（`PlayerState` 尾部追加 `jiguanyuan_cards` len+u32 序列；v42 字节被版
本门禁拒绝，无生产迁移数据）。版本 195→**196**。14 golden 重录：版本
回退探针（196→195 复跑 golden 全绿）证明位移唯一来源是版本字段。回归：
`swd_jiguanyuan_pending_then_gain_at_phase_end`（置牌→Judge 收束获得 2
张+状态清除全链）、`swd_jiguanyuan_same_target_rejected_while_pending`
（同目标待获得期间再指定拒、换目标放行）——证据见
[T12 证据](../operations/evidence/2026-10-11-t12-jiguanyuan-v196/README.md)。

**规则 v197（2026-10-11，T12 机关妖术切片）**：机关妖术（上游
`jiguanyaoshu`，♣3/♠3，`SwdJiguanyaoshuCard` tag 225，无懈窗包装，
range.global 1 无距离约束）——PlayCard 校验恰 1 名存活角色（含使用者自
身，上游 filterTarget 未排除 self）：目标有空装备栏（上游空 subtype 槽
同口径）、无防重标记（上游 !hasSkill("jiguanyaoshu_skill")）、非 isMin
（手牌数>体力，上游 !isMin）。结算：从**牌堆+弃堆**装备实体中发现一张
「目标空栏」子类型的装备（EquipSlot 对齐上游 subtype equip1–5），置入
目标手牌后 emit Equipped 走完整装配（commit_equip 同 Kind 替换/装备技授
予），并授予防重标记 `SwdJiguanyaoState`（tag 226，运行时授入、无时限，
上游 addSkill 同为持久）。**登记差异**：①发现池=牌堆+弃堆可见实体，上
游=get.inpile('equip') 全部已定义装备牌凭空造牌；无候选则不装备（上游
必有候选）。②上游 `jiguanyaoshu_skill` 为复合技：防重面+**补偿触发面**
（非自己回合内失去装备区牌时，随机补获等量 hslingjian 灵剑）——补偿面
依赖 hslingjian 灵剑类型池（7 张 swd 自造卡，list.js 无、仅机关术/该技
生成），**暂缓登记至机关术批**，本切片仅落地防重面。机关牌池已切 7 种
15 张，**余机关术×2**（机关术与灵剑池同批）。版本 196→**197**，存档格
式不变（v43）。14 golden 重录：版本回退探针（197→196 复跑 golden 全
绿）证明位移唯一来源是版本字段。回归：
`swd_jiguanyaoshu_discovers_equip_and_marks_target`（发现装配+防重拒+
isMin 拒）、`swd_jiguanyaoshu_full_slots_rejected`（5 栏全占拒）——证据
见[T12 证据](../operations/evidence/2026-10-11-t12-jiguanyaoshu-v197/README.md)。

**规则 v198（2026-10-11，T12 机关术批：灵剑池+补偿触发；存档 v44）**：
hslingjian 灵剑类型池 7 张（`CardKind::Hslingjian`，键 `hslingjian`，编
码 133 追加）——旋风之刃（♦4，随机弃 1 张 he）、中行虎甲（♥4，防具发现
装配[登记差异：牌堆+弃堆可见实体 vs 上游装备定义池凭空造]+随机弃 1 手
牌）、星体改造（♠6，自目标摸 1+手牌上限 -1 累加[重置时点登记差异：原生
下一回合开始 vs 上游上一结束阶段]）、时间回溯（♦6，目标装备区全部牌入
手[装备技随迁撤销]）、紧急冷冻（♠8，+2 护甲+翻面+冻结[不能使用牌也不
能成为目标，翻回解除；护甲直写无日志原语]）、圣休号角（♥8，嘲讽[相邻
角色只能以其为杀目标，其下一阶段开始解除]）、隐秘离场（♣10，潜行[不能
成为牌的目标，本回合结束解除；上游 tempHide 花色随机登记差异同池]）。
灵剑实体花色点数为原生登记值（上游凭空造牌花色随机）。**机关妖术补偿触
发补齐**：防重标记授予同时授 `SwdJiguanyaoGain`（tag 234，CardsDiscarded
forced）——非自己回合弃出装备 Kind 牌时随机补获等量灵剑（牌堆+弃堆发现；
**登记差异**：①以弃牌事件近似失去装备区牌——替换/获得离去路径不触发；
②发现池同①差异，无候选不补）。**机关术×2 暂缓**（deferred 登记）：运行
时卡牌合成/改造原语缺口——上游 lingjianduanzao 名字组合升级数值改造装
备，原生无此类原语（同虚拟用牌缺口类结构性登记，T18）。**存档格式
43→44**（PlayerState 尾部追加 `jileng_active`/`chaofeng_active`/
`xingtigaizao_count`/`yinmi_active` 灵剑状态四字段；v43 字节被版本门禁
拒绝）。版本 197→**198**。14 golden 重录：双版本回退探针（198/44→
197/43 复跑 golden 全绿）证明位移唯一来源是版本字段。回归 4 支（旋风/
时间回溯、紧急冷冻全链、嘲讽/潜行/星体改造、机关妖术补偿确定性构造）
——证据见[T12 证据](../operations/evidence/2026-10-11-t12-jiguanshu-v198/README.md)。

**规则 v199（2026-10-11，T18 机关术×2 转正：CardMod 受控合成原语；存档 v45）**：
机关术（上游 `lingjianduanzao`，`swd_jiguanshu1` ♣2 / `swd_jiguanshu2` ♦2，
kind jiguan）自目标主动牌：对每张有效装备受控合成——CardMod 伴随表
（`PlayerState.card_mods: BTreeMap<u32,u32>`，装备实体→附着灵剑实体，灵
剑随机入表），目录保持只读（不运行时改定义、不 intern 新名，
[ADR 0780](../adr/0780-card-instance-mod-primitive.md)）。**登记差异**：
①上游名字组合+数值升级（getValue 封顶 10）以「附着登记」近似——名称合
成/数值升级仅入改造表，点数升级当前无原生消费点；②jiqi/yuchan 池暂缓
（机关术全部附着灵剑）；③改造不随卡牌转移（换装/获得离去即失效，表随
拥有者存档）。**存档格式 44→45**（PlayerState 尾部 len+(u32,u32) 序列；
v44 字节被版本门禁拒绝）。版本 198→**199**。14 golden 双探针后重录。回
归 `swd_jiguanshu_attaches_card_mods_and_roundtrips`（施放后 CardMod 恰
一条且附着实体为灵剑；存档字节往返一致；无装备不可用）——机关牌池
8 种 17 张至此全数转正，swd 包 45 张——证据见[T18 证据](../operations/evidence/2026-10-11-t18-jiguanshu-primitive-v199/README.md)。

**规则 v200（2026-10-11，T19a 国战派生判定原语+阵法三张）**：
state.rs 纯派生判定原语（effective_faction 推导+存活座位序，无独立状
态）：`is_friend_of`（双方明置势力已知且相同）、`alive_neighbors`（存
活左右邻，跳过死亡座位）、`inline`/`inline_with`（连续同势力友方段）、
`sieged`/`sieges`（己方已知且存活左右邻互异非己方且互为友方）。zhenfa
启用门=存活≥4（上游 guozhanZhenfaEnabled；九变 ≥3 未建模）。zhenfa 包
3→6 张，CardKind 134–136 追加：**天覆阵**（AOE 全部大势力且有牌者各强
制弃 1 张 he，目标自选；HostMachine targets 队列+SelectCard 逐目标询问）、
**七星阵**（被围攻者施放：围攻者逐个弃 1+视为对其用杀[响应闪或 1 伤]；
targets=围攻者弃牌队列、cards=用杀队列 kernel 工厂预填）、**鸟翔阵**
（AOE 全部合法目标[异施放者势力、已知、与自身邻座同势力=连续段端点]各
响应闪或 1 伤）。**登记差异**：七星阵视为用杀以响应闪或 1 伤简化（上游
真实 useCard——虚拟用牌原语缺口同族，无加成/无额外面）；邻居含死亡座位
语义不明保守取存活；构逆/野心家结盟/undist 不建模。版本 199→**200**。
14 golden 双版本回退探针后重录。回归 `g1_shahai_zhenfa` 6/6——证据见
[T19a 证据](../operations/evidence/2026-10-11-t19-zhenfa-v200/README.md)。

**规则 v201（2026-10-11，T19b swapSeat 座位环原语+长蛇阵；存档 v46；破
阵诀登记上游未上架）**：
`GameStateCore.seat_ring: Vec<PlayerId>` 显式座位环
（[ADR 0781](../adr/0781-mutable-seat-ring.md)；建局恒等 0..n）：
`swap_seat(a,b)` 交换环位置（PlayerId 身份不变，区域/装配全随身份）；
座位语义全经 `ring_pos`/`ring_next`/`ring_prev` 访问器（环长度不符退恒
等），`+1%n` 直算禁止——下游适配 `next_alive`/`seat_order_from`/
`alive_neighbors`/`inline(_with)`/Boss 重整巡席。**存档格式 45→46**
（GameStateCore 尾部 seat_ring，len+pid 恒等亦显式编码自校验；v45 字节
被版本门禁拒绝）。**长蛇阵**（♣1/♦1 各 1，CardKind 137）：队列中
（inline）→全队列（含自己）各摸 1；否则与座位距离最近的友方 swap_seat
成队列（上游 absolute 距离排序同口径），本体入弃堆。**破阵诀**（编码
138 机器臂已就绪）登记**上游未上架**（list.js 4 条全注释无活跃实体，
非原生缺口；上游上架随批转换）。**登记差异**：swapSeat 第 4 参（保留相
对方位等 UI 语义）不建模；死亡座位在环中保留、阵法判定经存活过滤。版本
200→**201**。14 golden 双版本回退探针（201/46→200/45 复跑全绿）后重录。
回归 `g1_shahai_zhenfa` 6/6（换座成队列环变化断言/入队列后全队列摸
1[施放者净 0]/环不变量每 id 恰一次）——证据见[T19b 证据](../operations/evidence/2026-10-11-t19b-swapseat-v201/README.md)。

**规则 v202（2026-10-11，T11 标准差异消除批：倾国/救援/结姻/急救）**：
四条登记差异按上游 `standard/skill.js` 消除——**倾国**（甄姬）改黑**手
牌**当闪（上游 `position: "hs"`；装备区黑牌撤销，kernel 与 snap 视图重
建双处同步，排除已入列常规闪/龙胆杀防重复选项）；**救援**（孙权主公
技）补主公资格（`is_lord`，原型局恒真）+用桃者为吴势力（effective_faction
，护驾魏同口径）+非濒死者本人，桃回复 +1；**结姻**（孙尚香）目标改「其
他受伤男性」（上游 filterTarget：male+hp<maxHp+非自己——撤销「已婚=装
备区有牌」误登记，上游无此二条件）；**急救**（华佗）限持有者回合外
（上游 `player !== _status.currentPhase`）。存档格式不变（v46）。版本
201→**202**。版本回退探针（202→201 复跑 golden 全绿）证明 14 golden 位
移唯一来源是版本字段（四技能修改分支不在标准场景可达路径），全量重
录。native_kernel 204/0（倾国黑手牌+装备反例、救援吴势力 2 血+非吴对照
1 血、急救自己回合内不开本人询问直接死亡、结姻受伤目标）——证据见
[T11 证据](../operations/evidence/2026-10-11-t11-standard-diff-v202/README.md)。

**规则 v203（2026-10-11，T11 反间对齐；存档 v47）**：
反间（周瑜）补目标选花色环节并对齐上游条件伤害——目标先 `PickTag` 选
花色（0=♥/1=♦/2=♣/3=♠，上游 chooseControl 序，强制），再从周瑜手牌任
选一张获得；获得牌花色与所选**不符**才受 1 点伤害（相符无事；撤销「无
条件伤害+以看牌选牌承载花色」登记差异）。`MachineSnapshot::FanJian` 追
加 `chosen_suit`、`FanJianStageSnap` 追加 `SuitAsk`（stage 名=消费者语
义，恢复臂三询问映射同步）。**存档格式 46→47**（v46 字节被版本门禁拒
绝）。版本 202→**203**。版本回退探针（203→202 复跑 golden 全绿）证明
14 golden 位移唯一来源是版本字段，全量重录。回归 `n08_fanjian_give_and_damage`
三段（不符受伤/相符不伤/挂起态存档往返恢复后继续结算）——证据见
[T11 反间证据](../operations/evidence/2026-10-11-t11-fanjian-v203/README.md)。

**规则 v204（2026-10-11，T11 护驾对齐）**：
护驾（曹操主公技）代出询问改持牌者**自选**一张可用闪（SelectCards：
真闪+倾国转化黑手牌；选项只含持牌者自己手牌，他人暗牌 id 不出现；空
选=拒绝本位继续问下一位——`allow_cancel=false`+`min=0` 空选择非取消语
义），撤销「Confirm 自动弃第一张」的弃置+计数自动面；选中仍经
discard+CardResponded，无双计数不变。**登记差异撤销**（选择权+转化面
对齐上游 chooseToRespond）；保留既有惯例：代出转化牌以实体牌响应（上
游 result 为虚拟闪）。存档格式不变（v47）。版本 203→**204**。版本回退
探针（204→203 复跑 golden 全绿）证明 14 golden 位移唯一来源是版本字
段，全量重录。护驾 6 测试全绿（自选/转化/空选拒绝/无双计数/字节往返/
视图等价）——证据见[T11 护驾证据](../operations/evidence/2026-10-11-t11-hujia-v204/README.md)。

**规则 v205（2026-10-11，T11 斗地主 binglin 开局变体）**：
`DoudizhuVariant { Standard, Binglin }` 建局面（state.rs）——**变体不入
模式编码**（开局手牌数已进状态与 canonical_hash，存档格式不变 v47）；
`Engine::new_doudizhu_binglin`：开局地主 5 张/农民 4 张（上游 gameDraw
`next.num` 覆盖同口径），胜负/无奖惩/存档与标准一致。**登记不建模**：
binglin 的 replacePile 换牌堆（用标准牌堆）；online 变体整体（地骑虚拟
装备+选将后 infoHp 摸牌时点——结构性时点依赖）。版本 204→**205**。版
本回退探针（205→204 复跑 golden 全绿）证明 14 golden 位移唯一来源是版
本字段，全量重录。回归 `doudizhu_binglin_opening_draw`（地主 5/农民
4/地主死农民胜/存档往返/标准回归）——证据见
[T11 斗地主证据](../operations/evidence/2026-10-11-t11-doudizhu-binglin-v205/README.md)。
（撤销 v81 条目「特殊开局摸牌变体未建模」中 binglin 面；online 面维持登记。）

**规则 v206（2026-10-11，T11 versus 替补；存档 v48）**：
`GameMode::Versus` 增每队替补武将池（`substitutes`，与队伍同队序可为
空；**存档格式 47→48**，模式段/canonical_hash/快照界校验同步）；
`new_versus_with_substitutes` 建局。DeathMachine 死亡执行前替补臂：死
亡座位所在队池非空 → 弹队首 → 清本座位技能与区域入弃堆 → 换将装配
（apply_general_assembly 授新将技）→ 新将满血复活 → 摸 4 → 死亡执行
被替代（奖惩/终局判定不发生）；池空照常死亡结算（上游 versus.js 替补
出场 revive+uninit/init+draw 4+濒死链取消同口径）。**登记差异**：替补
替代死亡时击杀奖惩不结算；出场日志不入原生日志；替补选择以确定性队首
序近似（上游我方主公操控选替补）。版本 205→**206**。版本回退探针
（206→205 复跑 golden 全绿）证明 14 golden 位移唯一来源是版本字段，全
量重录。回归 `versus_substitute_revive_on_death`（两次替补+池空死亡+
奖惩分界+存档往返）——证据见
[T11 替补证据](../operations/evidence/2026-10-11-t11-versus-substitute-v206/README.md)。

**规则 v207（2026-10-11，T11 Boss 关卡骨架）**：
Boss 关卡开局面（消费面先行——boss 专属技能包首切片前置）：引擎私有
`OpeningPlan` 建局面枚举泛化 v205 斗地主专用参数，增
`BossStage { boss_cards, player_cards, first }`——关卡 `bossinfo.gameDraw`
（boss/挑战者分列发牌数）与 `loopFirst`（回合起点座位，以显式座位承载，
越界回退 0）；`new_boss_stage` 建局。变体为建局输入不入模式编码（存档
不变 v48）。**登记差异**：loopFirst 函数形态以座位参数近似；
`bossinfo.minion/control/init` 关卡脚本与 `loopType==2` 交替回合序属内
容面（随 boss 专属技能包逐关卡推进）。版本 206→**207**。版本回退探针
（207→206 复跑 golden 全绿）证明 14 golden 位移唯一来源是版本字段，全
量重录。回归 `boss_stage_opening_and_first_actor`（关卡发牌/起点/往返/
越界回退/标准回归）——证据见
[T11 Boss 关卡证据](../operations/evidence/2026-10-11-t11-boss-stage-v207/README.md)。

**规则 v208（2026-10-11，T11 boss 内容落包首批：烛阴+凶兽）**：
`BOSS_GENERALS` 内置 Boss 武将表（上游 mode/boss.js characterPack
.mode_boss 首批）——烛阴（男/群/4 血/单技，`isHiddenBoss` 随从且
`isBossAllowed`）；目录索引三段化 base→ext→**boss**（扩展将 def=base+i
的既定 ABI 不受扰动，boss 将 def=base+ext_n+i 动态尾段随启用集浮动，
`general_meta`/`general_count`/快照界同步）。共有技**凶兽**（技能表尾
追加，权限型 Noop+三面查询，马术同款）：①来源使用【杀】对体力值小于
自己的角色伤害 +1（`emit_sha_damage` 加成链）；②你与其他角色距离 -1
（`computed_distance` minus 链）；③你不能被翻面（`commit_turn_over_
before` 入口直读跳过，死亡取消型基建同款）。`debug_set_general_def`
通用装配面；FROZEN 技能名序尾加「凶兽」（62 名）。**登记差异**：翻面
取消在 commit 入口直读（上游 priority 20 触发器窗口）；boss 将 def 随
启用扩展集浮动；烛阴 isHiddenBoss/extraModeData 随关卡内容推进。版本
207→**208**（存档不变 v48）。版本回退探针（208→207 复跑 golden 全绿）
证明 14 golden 位移唯一来源是版本字段，全量重录。回归
`boss_zhuyin_xiongshou_semantics`（三面+对照+装配+往返）——证据见
[T11 烛阴证据](../operations/evidence/2026-10-11-t11-boss-zhuyin-v208/README.md)。

**规则 v209（2026-10-11，T11 boss 内容第二批首片：混沌无脏）**：
`BOSS_GENERALS` 增**混沌**（群/25 血/`[凶兽, 无脏]`——相德/隐贼(+switch)
三技未落随下批）。**无脏**两面（权限型 Noop+查询）：①摸牌阶段基数
`max(5, floor(hp/2))`（固定摸 2 后补额外 `max(5,hp/2)-2`，kernel 阶段
臂）；②手牌上限基数 0（`hand_limit` 签名增 `catalog`——体力贡献清零，
机甲/蛊舞/春饼等其余修饰保留）。FROZEN 名单尾加「无脏」（63 名）。存
档不变 v48。版本 208→**209**。版本回退探针（209→208 复跑 golden 全
绿）证明 14 golden 位移唯一来源是版本字段，全量重录。回归
`boss_hundun_wuzang_draw_and_hand_limit`（hp25 摸 12/降档 5/上限 0/他
人不受影响）——证据见
[T11 混沌证据](../operations/evidence/2026-10-11-t11-boss-hundun-wuzang-v209/README.md)。

**规则 v210（2026-10-11，T11 boss 内容第二批续片：相德+隐贼）**：
混沌四技补全。**相德**：DamageBefore 触发技（TriggerMeta forced，
YiShang 同款窗口）——来源非己且装备区有武器时对持有者伤害 +1（机器一
步直改载荷）；新 `MachineSnapshot::BossXiangde { owner }` 变体（无挂
起一步机：archive tag 61/canonical_hash 62/恢复臂 None/a0 样例尾）。
**隐贼**：`commit_damage` 尾双臂——激活门控（持有者含未启用实例，体力
>0 且 ≤max/2 → enabled 直写+SkillToggled；装配即未启用=apply 特判，
上游 switch gameStart disable 同口径）与伤害后弃牌（激活态 ∧ 持有者无
手牌 → 来源弃手牌首张；**随机改首张登记差异**）。混沌 skills 补全四
技；FROZEN 名单尾加「相德」「隐贼」（65 名）。存档不变 v48（相德机无
挂起态）。版本 209→**210**。版本回退探针（210→209 复跑 golden 全绿）
证明 14 golden 位移唯一来源是版本字段，全量重录。回归
`boss_hundun_xiangde_yinzei`（相德+1/隐贼激活门控/弃牌面）——证据见
[T11 相德隐贼证据](../operations/evidence/2026-10-11-t11-boss-xiangde-yinzei-v210/README.md)。

**规则 v211（2026-10-11，T11 boss 内容第三批首片：穷奇逐厄）**：
`BOSS_GENERALS` 增**穷奇**（群/hp20/`[凶兽, 逐厄]`——**登记差异**：上游
maxHp25 暂以 hp=20 建模，maxHp 面随覆泰批）。**逐厄**：权限型 Noop+
`commit_damage` 尾 global 臂——任何人受到伤害后（来源存活且非穷奇），
穷奇与来源各摸一张（穷奇为来源不触发）。覆泰(+全场禁桃)/延妒(+switch)
未落随下批。FROZEN 名单尾加「逐厄」（66 名）。存档不变 v48。版本
210→**211**。版本回退探针（211→210 复跑 golden 全绿）证明 14 golden
位移唯一来源是版本字段，全量重录。回归 `boss_qiongqi_zhue_dual_draw`
（双摸/自源不触发）——证据见
[T11 穷奇证据](../operations/evidence/2026-10-11-t11-boss-qiongqi-zhue-v211/README.md)。

**规则 v212（2026-10-11，T11 boss 内容第三批续片：穷奇覆泰）**：
**覆泰**：触发技（`TriggerMeta::PhaseBegin` forced——判定阶段开始锚点，
神智同口径先例）——持有者准备阶段（判定阶段开始）全体存活受伤者按座
位序各回 1（机器一步无挂起）；新 `MachineSnapshot::BossFutai { owner }`
变体（archive tag 62/canonical_hash 63/恢复臂 None/a0 样例尾）。
**覆泰2（全场禁桃）未落**：跨界濒死救援与 PlayCard 桃校验两面下批单
独落；延妒(+switch) 同批。存档不变 v48。版本 211→**212**。版本回退探
针（212→211 复跑 golden 全绿）证明 14 golden 位移唯一来源是版本字段，
全量重录。回归 `boss_qiongqi_futai_recover_wounded`（受伤者回 1/满血
不动/往返）——证据见
[T11 覆泰证据](../operations/evidence/2026-10-11-t11-boss-qiongqi-futai-v212/README.md)。

**规则 v213（2026-10-11，T11 boss 内容第三批再续片：穷奇延妒）**：
**延妒**两面（权限型 Noop+查询）：①激活门控——持有者体力>0 且
≤max/2 时 `SkillInstance.enabled` 直写+SkillToggled（装配即未启用
=apply 特判扩隐贼+延妒；与隐贼门控并列独立判定——穷奇无隐贼，嵌套
写法会静默失效）；②其他角色结束阶段开始时若其 `turn_damage==0` 且有
手牌 → 穷奇获得其**手牌首张**（确定性首张登记差异——上游 gainPlayerCard
由穷奇选）。**覆泰2（全场禁桃）未落**（下批）。存档不变 v48。版本
212→**213**。版本回退探针（213→212 复跑 golden 全绿）证明 14 golden
位移唯一来源是版本字段，全量重录。回归
`boss_qiongqi_yandu_gain_at_end_phase`（激活门控/有伤害不触发/无伤害
获得首张）——证据见
[T11 延妒证据](../operations/evidence/2026-10-11-t11-boss-yandu-v213/README.md)。

**规则 v214（2026-10-11，T11 覆泰2 全场禁桃两面）**：
`tao_futai2_banned`（kernel pub(crate) helper）——持覆泰的存活穷奇（排
除用桃者）在场 ∧ 当前回合≠穷奇 → 桃禁。两面消费：PlayCard `CardKind::
Tao` 校验（banned → IllegalTargets；护符濒死专用不受此 mod）与濒死救援
`is_tao_like` 真桃分支（banned → 不可救、选项不入）；穷奇自己回合禁令
解除（穷奇自用桃可用）。**登记差异**：上游 `_status.event.skill` 转化
用桃上下文未建模（原生转化救援走独立谓词无 futai2 面）。存档不变
v48。版本 213→**214**。版本回退探针（214→213 复跑 golden 全绿）证明
14 golden 位移唯一来源是版本字段，全量重录。回归
`boss_futai2_tao_banned_both_sides`（用桃拒/救援面禁/穷奇回合解除）—
—证据见[T11 覆泰2 证据](../operations/evidence/2026-10-11-t11-boss-futai2-v214/README.md)。

**规则 v215（2026-10-11，T11 boss 内容第四批首片：饕餮贪欲）**：
`BOSS_GENERALS` 增**饕餮**（群/hp20/`[凶兽, 贪欲]`——藏目/集财(+switch)
未落随下批）。**贪欲**两面（权限型 Noop+查询）：①跳过弃牌阶段
（`Phase::Discard` begin 臂直查提前 return，上游 trigger.cancel 同口
径）；②结束阶段全场最多手牌（≥任何其他存活者，`tanyu_is_max_hand` 抽
函数）→ 失 1 体。FROZEN 名单尾加「贪欲」（69 名）。存档不变 v48。版本
214→**215**。版本回退探针（215→214 复跑 golden 全绿）证明 14 golden
位移唯一来源是版本字段，全量重录。回归
`boss_taotie_tanyu_skip_discard_and_max_hand_lose_hp` 三局（跳弃牌/最
多会失体[自动推进多轮非精确]/非最多状态面 is_max=false）——证据见
[T11 饕餮证据](../operations/evidence/2026-10-11-t11-boss-taotie-v215/README.md)。

**规则 v216（2026-10-11，T11 boss 内容第四批续片：饕餮藏目+积财——四技全齐）**：
**藏目**：摸牌基数=存活角色数（Draw 臂无脏同位：固定摸 2 后补额外
存活数-2）。**积财**两面：激活门控（`commit_damage` 尾延妒并列第四
臂；装配即未启用=apply 特判扩隐贼/延妒/积财）+回复结算后双摸（**新
`commit_recover_skills` 臂**：`commit_frame` 增 `EventData::Recover`
分派——机制层 Ctx.recover 内联、技能面归 kernel commit 层；回复者==
持有者摸 2/否则各摸 1）。`debug_recover` 测试面（发 Recover 事件走完
整结算）。FROZEN 名单尾加「藏目」「积财」（71 名）。`jicai_switch`
独立技不落表（门控并入 enabled，先例同构）。存档不变 v48。版本
215→**216**。版本回退探针（216→215 复跑 golden 全绿）证明 14 golden
位移唯一来源是版本字段，全量重录。回归
`boss_taotie_cangmu_jicai`（藏目 4 人局摸 4/激活门控/他人回复双摸/自
己回复摸 2）——证据见
[T11 藏目积财证据](../operations/evidence/2026-10-11-t11-boss-cangmu-jicai-v216/README.md)。

**规则 v217（2026-10-11，T11 boss 内容第五批首片：梼杌拟态）**：
`BOSS_GENERALS` 增**梼杌**（群/hp25/`[凶兽, 拟态]`——冥顽/乱常(+switch)
未落随下批）。**拟态**两面（commit_damage 内联查询）：①回合内受伤防止
（诚信免伤同位：turn.current==target → DamagePrevented return）；②回合
外火伤 +1（amount 链赤龙牙旁）。**勘误**：乱常非交替回合序——实为虚拟
南蛮/万箭 AOE 双技（loopType==2 才是关卡回合序面）。FROZEN 名单尾加
「拟态」（72 名）。存档不变 v48。版本 216→**217**。版本回退探针
（217→216 复跑 golden 全绿）证明 14 golden 位移唯一来源是版本字段，全
量重录。回归 `boss_taowu_nitai_in_shield_out_fire`（回合内免伤/回合外
火伤+1/普通不加成）——证据见
[T11 拟态证据](../operations/evidence/2026-10-11-t11-boss-taowu-nitai-v217/README.md)。

**规则 v218（2026-10-11，T11 boss 内容第五批续片：梼杌乱常——四技位
全齐）**：**乱常**三件：①激活门控（`commit_damage` 尾第五并列臂，
SkillInstance.enabled 直写；装配即未启用；触发计划层过滤 disabled——
门控即触发开关）；②准备阶段（判定阶段开始锚点）虚拟【南蛮入侵】（触
发机一步 `use_virtual_card(owner, SavageAttack, 全其他存活)`——虎煞
先例同款，载体=牌堆底实体）；③结束阶段虚拟【万箭齐发】（Phase::End
臂查询式同款 targets）。新 `MachineSnapshot::BossLuanchang` 变体
（archive tag 63/canonical_hash 64/恢复臂 None/a0 样例尾）。
`boss_luanchang_switch` 不落表（门控并入 enabled，先例同构）。**冥顽**
未落（下批：目标锁定集+playerEnabled 目标 mod）。FROZEN 名单尾加「乱
常」（73 名）。存档不变 v48。版本 217→**218**。版本回退探针（218→217
复跑 golden 全绿）证明 14 golden 位移唯一来源是版本字段，全量重录。回
归 `boss_taowu_luanchang_virtual_aoe`（门控激活/两虚拟 AOE 结算——日
志佐证载体与响应分布）——证据见
[T11 乱常证据](../operations/evidence/2026-10-11-t11-boss-luanchang-v218/README.md)。

**规则 v219（2026-10-11，T11 梼杌批末片：冥顽——梼杌四技全齐）**：
**冥顽**三面（权限型 Noop+查询）：锁定集=**`TurnState.minwan_targets`
回合级新字段**（回合更替 `reset_turn_limits` 显式清零=上游 phaseAfter
clear 同窗；**存档格式 48→49** turn 段尾 len+pids）。①伤害后入集
（commit_damage 尾：回合内自己用牌伤害他人 → 集合空先 push 自己再
push 受伤者）；②每用牌摸 1（commit_card_used：集合非空 ∧ 持技 → draw
1）；③PlayCard 目标校验（targets 含「非自己且不在集合」→ IllegalTargets
——上游 mod playerEnabled 同口径）。FROZEN 名单尾加「冥顽」（74 名）。
版本 218→**219**。版本回退探针（219→218 复跑 golden 全绿）证明 14
golden 位移唯一来源是版本字段，全量重录。回归
`boss_taowu_minwan_lockset_draw_and_target_mod`（锁定集/摸 1/目标拒/
集合目标可用/更替清零/v49 往返）——证据见
[T11 冥顽证据](../operations/evidence/2026-10-11-t11-boss-minwan-v219/README.md)。

**规则 v220（2026-10-11，T11 书精批：催毒+中毒——书精全技）**：
`BOSS_GENERALS` 增**书精**（女/hp2/`[催毒]`——势力暂 Shu 近似上游
shen 族，登记）。**催毒**：commit_damage 尾查询臂——来源持技伤害非己
目标且目标无中毒 → `skills.insert` 中毒实例（**运行时授技**，濒死救援
通道首例同模式）+SkillToggled；木神勾芒联动不在目录跳过、敌方以受伤者
≠持技者近似（均登记）。**中毒**（授技 def）：Judge begin 段内联确定性
判定（铁骑先例）——红桃无伤/黑桃保留/方片梅花伤 1+实例移除。FROZEN
名单尾加两技（76 名）。存档不变 v49。版本 219→**220**。版本回退探针
（220→219 复跑 golden 全绿）证明 14 golden 位移唯一来源是版本字段，全
量重录。回归 `boss_shujing_cuidui_poison_cycle`（授毒/方片伤 1 失技/黑
桃保留）——证据见
[T11 书精证据](../operations/evidence/2026-10-11-t11-boss-shujing-v220/README.md)。

**规则 v221（2026-10-11，T11 四神兽系首片：相柳妖兽）**：
`BOSS_GENERALS` 增**相柳**（群/hp25/`[妖兽]`——毒妒[蛇毒标记链]/
兽九[手牌补 9]/邪酬(+switch) 未落随下批，三技均涉 storage 标记面）。
**妖兽**：`computed_distance` 距离 -2（凶兽同款距离链）。FROZEN 名单尾
加「妖兽」（77 名）。**连带修正**：boss 段（动态尾段，现 7 将）使
g1_d3 两处「base+7 越界」硬编码假设过期——改 `general_count()` 动态
总尾。存档不变 v49。版本 220→**221**。版本回退探针（221→220 复跑
golden 全绿）证明 14 golden 位移唯一来源是版本字段，全量重录。回归
`boss_xiangliu_yaoshou_distance_minus_two`（距离-2 钳下限/他人不受影
响）——证据见
[T11 相柳证据](../operations/evidence/2026-10-11-t11-boss-xiangliu-yashou-v221/README.md)。

**规则 v222（2026-10-11，T11 相柳续片：兽九）**：
**兽九**三面（权限型 Noop+查询）：①手牌上限恒 9（`hand_limit` 函数分
支覆盖基数）；②跳过摸牌阶段（Draw 臂）；③出牌阶段开始/回合结束补至
9（Play/End begin 臂 `draw(9-count)`）。FROZEN 名单尾加「兽九」（78
名）。**登记**：毒妒/邪酬(+switch) 涉跨回合标记存储（PlayerState 新字
段+存档 v50+弃/失选择询问）下批一并落。存档不变 v49。版本 221→
**222**。版本回退探针（222→221 复跑 golden 全绿）证明 14 golden 位移
唯一来源是版本字段，全量重录。回归
`boss_xiangliu_jiushou_limit_nine_and_refill`（上限恒 9/出牌补 9/End
补 9/他人不受影响）——证据见
[T11 兽九证据](../operations/evidence/2026-10-11-t11-boss-jiushou-v222/README.md)。

**规则 v223（2026-10-11，T11 相柳批末片：毒妒+蛇毒链+邪酬——四技全
齐）**：蛇毒标记存储=**`PlayerState.shedu: u8` 新字段**（跨回合持久；
**存档格式 49→50**；canonical_hash 同步）。**毒妒**：相柳受击 → 来源
获 1 蛇毒（自身不获；commit_damage 尾查询臂）。**邪酬**两面：激活门控
（hp≤max/2 enable 直写，apply 特判扩至五技）+激活态他人用【桃】/【酒】
（含 use_as）→ 相柳获 1 蛇毒。**蛇毒结算机**（BOSS_SHEDU 触发技
PhaseBegin forced——授毒时与中毒实例双授）：标记>0 → 挂起「弃 X 张牌
或失去 X 体力」（SelectCards min0/maxX allow_cancel，空选=失 X）→ 标
记-1；新 `MachineSnapshot::Shedu` 变体（archive tag 64/canonical_hash
65/**恢复臂重挂询问**——真挂起机）。**登记差异**：毒妒「桃视为杀」
mod 未建模；shedu 增减无独立 LogEvent。FROZEN 名单尾加「毒妒」「邪
酬」「蛇毒」（79 名）。回归 `boss_xiangliu_duqu_shedu_echou` 四段——
证据见[T11 毒妒证据](../operations/evidence/2026-10-11-t11-boss-duqu-v223/README.md)。
**注**：本批运行时版本因断链停留 222（见 v224 条目审计），名义按 223
登记。

**规则 v224（2026-10-11，T11 朱厌批首片：兵先+版本断链修正）**：
`BOSS_GENERALS` 增**朱厌**（群/hp25/`[妖兽, 兵先]`——maxHp30 登记差异
同穷奇；巨源 selectTarget mod/虚失(+switch) 未落随下批）。**兵先**：
Phase::End 臂——他人回合结束 ∧ 其本回合未使用【杀】（**出牌阶段口径**
turn.sha_used_play_phase，登记差异：上游 countUsed 含响应杀）→ 视为对
之使用一张【杀】（use_virtual_card 乱常同款）。FROZEN 名单尾加「兵
先」（80 名）。**版本断链修正**：v220–v223 四批 bump/restore 脚本链断
链（restore 锚失效未写盘）致运行时版本停留 222——本批修正至 **224**，
replay_scenarios rationale 登记审计注记；探针口径变化（223 版+本批代
码 14 全位移佐证）同步登记。存档不变 v50。版本 222（实际）→**224**。
全量重录 14 golden。回归
`boss_zhuyan_bingxian_virtual_sha_on_no_sha_turn`（未出杀被虚拟杀/出杀
不触发）——证据见
[T11 兵先证据](../operations/evidence/2026-10-11-t11-boss-bingxian-v224/README.md)。

**规则 v230（2026-10-11，T11 应昭批首片：封冻；运行时版本断链修正
227→229）**：封印存储=`PlayerState.fengyin_active: bool` 新字段（**存
档格式 52→53**；canonical_hash 入参——**字段新增批**：全部场景哈希位
移含版本字段，探针「位移唯一来源=版本字段」口径不适用，以 229 版+当
前代码 14 全位移佐证 Boss 不可达）。**封冻**两面（Judge begin 段授予：
持封冻者回合开始全部其他存活获封印[自身不封]；Phase::End begin 臂回
合结束全清）。**PlayCard 校验**：被封印者使用手牌区牌 → IllegalTargets
（装备区牌不受限——上游 mod 仅 position=h）。**登记**：巡游[随机获得
hej 牌+装备自动使用]/死缚(+switch) 未落随下批。FROZEN 名单尾加「封
冻」（83 名）。回归 `boss_yingzhao_fengdong_hand_ban`（授予/解除/可用
恢复/v53 往返）——证据见
[T11 封冻证据](../operations/evidence/2026-10-11-t11-boss-fengdong-v230/README.md)。
（运行时版本断链修正：228 红莲批后运行时停留 228，本批一并修正。）

**规则 v231（2026-10-11，T11 应昭批续片：巡游）**：
**巡游**单面（权限型 Noop+Judge begin 臂查询）：global phaseBegin——
enabled 持有者（≠当前回合玩家）在场即触发（炎狱 holders 先例同构）→
受害者=座位序从持有者起首个有手牌的其他角色，取其手牌首张（**登记差
异**：hej 全区域/随机以座位序+首张确定性近似；判定区牌不取）；装备牌
自动使用（`kind.equip_slot()` 同栏替换：旧装备 revoke+入弃堆、新装备
grant——931 区先例）。FROZEN 名单尾加「巡游」（84 名）。存档不变 v53。
版本 229→**231**（运行时断链修正 230→231 合并录）。全量重录 14 golden
（231 版+当前代码全位移佐证）。回归
`boss_yingzhao_xunyou_steal_and_equip`（自身除外/取牌入手）——证据见
[T11 巡游证据](../operations/evidence/2026-10-11-t11-boss-xunyou-v231/README.md)。

**规则 v232（2026-10-11，T11 应昭批末片：司圃死缚使用面——应昭四技全
齐）**：**司圃**（权限型 Noop+查询）三件：激活门控（commit_damage 尾
第九并列臂，hp≤max/2 enable 直写；apply 特判扩至八技）+响应禁
（respond_shan 选项排除：杀来源=enabled 持有者、当前回合=持有者、已
用<3 → 闪选项仅 Pass）+救禁（dying_save 选项排除：濒死者=持有者同窗
→ 他人救援桃/酒不入选项、持有者自救可用）。**使用面
（cardEnabled/cardUsable）**由原生回合模型天然承载（仅回合玩家可提交
PlayCard——模型结构承载，非逐条实现）。**登记差异**：无懈等其余响应
窗未纳入禁令（上游 cardRespondable 覆盖全部响应窗）。FROZEN 名单尾加
「司圃」（85 名）。存档不变 v53。版本 231→**232**。版本回退探针
（232→231 复跑 golden 全绿）证明 14 golden 位移唯一来源是版本字段，
全量重录。回归 `boss_yingzhao_sipu_use_ban_first_two` 五段（禁闪/计数/
解除自救/他人禁救）——证据见
[T11 司圃证据](../operations/evidence/2026-10-11-t11-boss-sipu-v232/README.md)。
**同批测试注册缺陷修复**（横切历批）：native_kernel 属性堆叠致 4 fn
重复注册（最高×10）、20 个历批测试 fn 零属性从未注册，5 处失修 def 索
引（朱厌×3/书精/相柳）与招祸凶兽条件伤期望一并修正——修复后复位测试
全绿、全量 578/0/1；**历史批次计数口径不可比**（详见证据 README）。

**规则 v233（2026-10-11，T11 四象系首片：神裔＋青龙条目）**：
**神裔**（查询型 Noop）两面：①判定反转——判定阶段延时锦囊判定对持有者
恒不生效（上游 mod judge `event.type=="phase"`：bool true→false、false→
null；上游考证：`type=="phase"` 仅判定阶段延时锦囊判定打标
[content.ts:4968 `judge(card).set("type","phase")`]，技能自判定不受影
响。原生=apply_judgement 五臂[乐/兵粮/洪水/归有界/闪电]效果条件一并短
路，消费面等价；牌去向不变：锦囊照常弃置、闪电照常传下家）；②翻面取
消——仅**正→背**取消（turnOverBefore filter `!isTurnedOver`；凶兽同位
臂，背→正照常翻回）。`BOSS_GENERALS` 尾追加**青龙**（群/hp4/`[神裔]`，
def 索引 10；releiji 雷击/boss_qingmu2 青木未落随下批）。**登记差异**：
洪荒之力禁用交互（honghuangzhili）未落。FROZEN 名单尾加「神裔」（86
名；首版 SkillDef 误插司圃前被冻结表测试当场拦截——已移真表尾）。存档
不变 v53。版本 232→**233**。版本回退探针（233→232 复跑 golden 全绿）
证明 14 golden 位移唯一来源是版本字段，全量重录。回归
`boss_qinglong_shenyi_judge_void_and_never_flipped` 五段（乐无效/闪电传
下家/非持有者对照/翻面两面/往返）——证据见
[T11 神裔证据](../operations/evidence/2026-10-11-t11-boss-shenyi-v233/README.md)。

**规则 v234（2026-10-11，T11 四象系续片：雷击＋青木2 处置登记）**：
**雷击**（查询型 Noop，上游 releiji 界雷击）：打出/使用闪后——座位序下
一名其他存活角色确定性判定（铁骑内联口径，无鬼道/天妒改判窗）：黑桃→
其受 2 点雷伤；梅花→持有者回复 1、其受 1 点雷伤；其他无事。原生=杀/
万箭两闪消费位内联臂（`releiji_on_shan` 共享 helper；无双第二闪每张各
触发一次同上游）。**登记差异**：chooseTarget 自由选目标→座位序首名确
定性近似；useCard 半面由「闪无主动使用」模型承载，护驾代出等其余打出
窗未挂钩。**青木2**（boss_qingmu2 死亡隐藏+换位重生）=关卡流程面，属
Boss 模式裸骨架缺口，登记不落内核（青龙条目不含）。`BOSS_GENERALS`
青龙条目扩为 `[神裔, 雷击]`。FROZEN 名单尾加「雷击」（87 名；SkillDef
再次误插非表尾被冻结表测试拦截——移真表尾）。存档不变 v53。版本 233→
**234**。版本回退探针（234→233 复跑 golden 全绿）证明 14 golden 位移
唯一来源是版本字段，全量重录。回归
`boss_qinglong_releiji_thunder_judge` 六段（梅花/黑桃/红桃/非持有者对
照/万箭触发/往返）——证据见
[T11 雷击证据](../operations/evidence/2026-10-11-t11-boss-releiji-v234/README.md)。

**规则 v235（2026-10-11，T11 四象系末片首刀：狂笑＋白虎条目）**：
**狂笑**（查询型 Noop）两面（engine PlayCard 校验段）：【杀】无距离限制
（sha_range_free 链追加，飞刃/天弦同位）+目标数上限无限（`kuangxiao_
inf` 臂，天弦 tianxian_inf 同构→usize::MAX）。`BOSS_GENERALS` 尾追加
**白虎**（def 索引 11/`[神裔, 狂笑]`/hp4；boss_baimang2 白芒=死亡隐藏+
换位重生关卡流程面，同青木2 登记不落内核）。**登记差异**：势力 shen→
群目录近似（Faction 无神势力枚举）；友方禁指面（playerEnabled isFriend
Of）未建模——boss 侧关系网未入 PlayCard 校验；range[0]=-1 空指未建模
（出牌语义不变面）。FROZEN 名单尾加「狂笑」（88 名；SkillDef 三连坑再
犯非表尾插入——冻结表测试拦截后移真表尾）。存档不变 v53。版本 234→
**235**。版本回退探针（235→234 复跑 golden 全绿）证明 14 golden 位移
唯一来源是版本字段，全量重录。回归
`boss_baihu_kuangxiao_sha_range_and_targets` 四段（隔位杀/三目标齐中/
非持有者对照/往返）——证据见
[T11 狂笑证据](../operations/evidence/2026-10-11-t11-boss-kuangxiao-v235/README.md)。

**规则 v236（2026-10-11，T11 四象系朱雀首片：焚天＋朱雀条目）**：
**焚天**四件：①伤害视为火（ZhaoHuoBehavior 同构复用——数据less nature
改写机器/快照变体共用 BossZhaohuo，恢复按技能表重挂，登记同构复用）；
②红牌免次数（杀限+酒限豁免臂）；③红牌免距离（sha_range_free 链+顺手
牵羊臂）；④红杀无视防具（kernel 八卦阵判定步短路——原生防具=八卦阵，
directHit 承载）。`BOSS_GENERALS` 尾追加**朱雀**（def 索引 12/female/
`[神裔, 焚天]`/hp4；boss_chiyan2 赤炎未落随下批）。**登记差异**：
wuxieRespondable 红牌禁他人无懈未挂钩（红色锦囊 directHit 面同门空缺）；
红酒免次数面目录无实体红酒（仅声明转化可达，以登记代测）；势力 shen→
群目录近似。**坑位**：行为触发总线只扫 `trigger: Some(meta)` 技能——
复用带行为的 SkillDef 必须连 TriggerMeta 一起抄（首版静默失效）。FROZEN
名单尾加「焚天」（89 名）。存档不变 v53。版本 235→**236**。版本回退
探针（236→235 复跑 golden 全绿）证明 14 golden 位移唯一来源是版本字段，
全量重录。回归 `boss_zhuque_fentian_red_cards` 五段（火改写免疫反证/
红杀免次免距/黑杀仍限/红酒登记/跳八卦+往返）——证据见
[T11 焚天证据](../operations/evidence/2026-10-11-t11-boss-fentian-v236/README.md)。

**规则 v237（2026-10-11，T11 四象系玄武首片：灵渠＋玄武条目；存档 v54）**：
`PlayerState.lingqu_count: u8` 新字段（**存档 53→54**；canonical_hash 入
参——**字段新增批**：14 golden 位移=新字段+版本两源，v230 口径；实测
236+新代码与 237+新代码两组哈希互异佐证两源独立）。**灵渠**三臂：受击
摸 1+计数（commit_damage damageEnd 段，逐厄同位；0 点伤害照常）+终值
>1 伤害归零（增幅后/统计前臂——上游 damageBegin4 低优先级终值口径）+
手牌上限 +X（hand_limit 尾臂）。`BOSS_GENERALS` 尾追加**玄武**（def 索
引 13/`[神裔, 灵渠]`/hp4；boss_xuanlin2 玄麟=关卡流程面登记不落）——
**四象系四条目全部入目录**（青龙/白虎/朱雀/玄武=10–13）。FROZEN 名单
尾加「灵渠」（90 名；SkillDef 第四次非表尾插入冻结表拦截后移正）。存档
53→**54**。版本 236→**237**。回归
`boss_xuanwu_lingqu_damage_counter` 五段（归零/1 点照常/终值门槛/上限+
计数/v54 往返）——证据见
[T11 灵渠证据](../operations/evidence/2026-10-11-t11-boss-lingqu-v237/README.md)。

**隐藏链八技整族处置（2026-10-11，无版本递增）**：boss_qingmu2/baimang2/
chiyan2/xuanlin2（阶段 1→2）与 boss_qingmu3/baimang3/chiyan3/xuanlin3
（阶段 2→3）八技均为同构关卡流程面（dieBegin hide + dieAfter changeSeat/
changeBoss/补给/复活链）——属 Boss 模式裸骨架缺口既定边界，不落内核；
四象隐藏条目本就只列已转换技，无目录/行为影响。规则 237/存档 v54 不变，
golden 不动（无语义变化先例同 versus 4v4 固化批）——证据见
[隐藏链处置证据](../operations/evidence/2026-10-11-t11-boss-hidechain-disposition/README.md)。
下批候选勘察：boss_shenen 神恩（boss 侧无距离+挑战者侧上限/摸牌 +1，
三面全查询型，最小下一片）/boss_xingqiu 星囚（连环+授技，较重）/boss_
buchun 不臣（复活书精，阶段流程）。

**规则 v238（2026-10-11，T11 星宿阶段2系首片：神恩＋太昊条目）**：
**神恩**（全局光环查询型 Noop）三面：boss 方用牌无视距离（sha_range_free
链+顺手距离链 `shenen_boss_free` 臂）+挑战者侧手牌上限 +1（hand_limit
尾臂）+挑战者侧摸牌 +1（Draw 臂固定摸 2 后追加）。**阵营承载**：新增
`PlayerState::is_boss_side`（Boss 模式=boss 座位+fellows；非 Boss 模式恒
false）。光环激活=场上存在神恩持有者（上游 global 子技全员发放的域化
近似，登记）。`BOSS_GENERALS` 尾追加**太昊**（def 索引 14/hp6/`[神裔,
神恩]`；names 风|伏羲；boss_qingyi 青翼未落随下批）。**登记差异**：阶段
换将体不在裸骨架（side 域登记）；势力 shen→群目录近似。FROZEN 名单尾
加「神恩」（91 名；SkillDef 第五次非表尾插入冻结表拦截后移正）。存档不
变 v54。版本 237→**238**。版本回退探针（238→237 复跑 golden 全绿）证
明 14 golden 位移唯一来源是版本字段，全量重录。回归
`boss_taihao_shenen_aura` 五段（boss 方无距离/挑战者对照/摸牌+1/上限
+1/往返）——证据见
[T11 神恩证据](../operations/evidence/2026-10-11-t11-boss-shenen-v238/README.md)。

**规则 v239（2026-10-11，T11 星宿阶段2系续片：星囚＋蓐收条目；存档 v55）**：
`PlayerState.xingqiu_toggled: bool` 新字段（**存档 54→55**；canonical_hash
入参——字段新增批）。**星囚**单臂（kernel Phase::Play begin 段）：持有者
**本人**出牌阶段开始开关取反，true 回合把未横置的敌方全部横置（direct
直写 linked+日志，索链甲同款；只链不解；敌方判定=`is_boss_side` 异侧）。
`BOSS_GENERALS` 尾追加**蓐收**（def 索引 15/hp5/`[神裔, 星囚]`；names
嬴|该；boss_baimang3 白芒3=已处置关卡面，条目不含）。**登记差异**：对
boss_mingxingzhu 授 boss_jiding 面依赖武将未入库。FROZEN 名单尾加「星
囚」（92 名）。存档 54→**55**。版本 238→**239**（字段批探针：238+新代
码全 14 位移佐证字段源）。回归
`boss_jinshenrushou_xingqiu_link_toggle` 五段（全横置/只链不解/开关取反
/v55 往返/fellow 例外）——证据见
[T11 星囚证据](../operations/evidence/2026-10-11-t11-boss-xingqiu-v239/README.md)。
**坑位**：phase 触发技 player 域必须过滤持有者本人（首版漏滤被回归断言
抓出——横置面绿≠开关面绿）；drain_choices 空应答被 min>0 SelectCards
校验拒绝（静默挂起→清手牌规避）。

**规则 v240（2026-10-11，T11 星宿阶段2系末片：不臣＋勾芒条目；存档 v56）**：
`PlayerState.buchun_used_round: u32` 新字段（**存档 55→56**；canonical_hash
入参——字段新增批）。**不臣**（本批首个 boss 主动技——`ActivateSkill`
直调 `activate`，不经触发总线）两线：场上有死亡书精 → 失 1 体并全部复
活（重整计数清零同款+hp1+`BossRevived`+摸 2）；否则 SelectPlayers 令一
名同阵营受伤者回 2（active_ask 挂起态派生）。间隔 ≥2 轮门控
（`round-used<2 → InvalidChoice`）。快照变体 `BossBuchun`（archive tag
66/机哈希 67/a0 64；stage 显式编码）。`BOSS_GENERALS` 尾追加**勾芒**
（def 索引 16/hp5/`[神裔, 不臣]`；names 姬|重；青木3 已处置）。**登记
差异**：上游仅复同 parentNode 书精+上家优先——native 全场死亡书精座
位序（关卡分组概念裸骨架无对应）；势力 shen→群目录近似。FROZEN 名单
尾加「不臣」（93 名）。存档 55→**56**。版本 239→**240**（字段批探针：
239+新代码全 14 位移佐证字段源）。回归
`boss_mushengoumang_buchun_revive_and_recover` 五段（复活线/同轮拒/恢
复线/v56 往返/守恒）——证据见
[T11 不臣证据](../operations/evidence/2026-10-11-t11-boss-buchun-v240/README.md)。
**坑位**：主动技 Behavior 的 start 为必需方法（unreachable 占位）；书精
将无常量（BOSS_GENERALS 段按名查目录）；answer_first 不存在
（`ctx.answer().first().copied()` 惯用）。

**规则 v241（2026-10-11，T11 炎灵陪将片：回火+复燃＋焰灵条目）**：
**回火**（查询型 Noop）两面：杀上限 +1（engine 杀限臂元宝肉同款额度——
`SkillInstance.once_used` 每回合一次，`reset_turn_limits` 随本人回合开
始清零）+死亡反噬（DeathMachine 移除前捕获、`PlayerDied` 后敌方全体各
受 3 点无来源火伤，`is_boss_side` 异侧判定）。**复燃**（查询型 Noop）：
`is_tao_like` 追加臂——焰灵濒死时其敌方可用红色牌当桃救（急救块同位）。
`BOSS_GENERALS` 尾追加**焰灵**（def 索引 17/hp4/`[回火, 复燃]`；上游陪
将无神裔）。**译名勘误**：指令面「回祸」实为上游译名「回火」（10973）。
**登记差异**：boss 死亡即终局截停事件队列——反噬面以 fellow 线回归验
证（测试结构边界登记，引擎语义不受影响）。FROZEN 名单尾加「回火」「复
燃」（94/95 名）。存档不变 v56。版本 240→**241**。版本回退探针
（241→240 复跑 golden 全绿）证明 14 golden 位移唯一来源是版本字段，全
量重录。回归 `boss_yanling_huihuo_furan` 三段（杀限额度/复燃救援/反噬
fellow 线+往返）——证据见
[T11 焰灵证据](../operations/evidence/2026-10-11-t11-boss-yanling-v241/README.md)。

**规则 v242（2026-10-11，T11 星宿阶段2系星夏片：星夏＋祝融条目；存档 v57）**：
`PlayerState.xingxia_used_round: u32` 新字段（**存档 56→57**；canonical
_hash 入参——字段新增批）。**星夏**（主动技，ActivateSkill 直调——不臣
先例第二例）：对场上焰灵（BOSS_GENERALS 段按名查目录+座位装配匹配）
`Damage(2, Fire)`；追击线可放弃（SelectPlayers 异侧存活 min0/cancel）→
选中者弃一张红色牌（he 位）或受 1 点火伤；间隔 ≥2 轮门控
（`InvalidChoice`）。快照变体 `BossXingxia`（archive tag 67/机哈希 68/
a0 65；active_ask 挂起态派生）。`BOSS_GENERALS` 尾追加**祝融**（def 索
引 18/hp5/`[神裔, 星夏]`；names 姜|重黎；赤炎3 已处置）。**登记差异**：
selectTarget -1 以「全场唯一焰灵座位」承载；势力 shen→群目录近似。
**赤翼/白翼处置**（青翼同族，登记不落）：赤翼=轮次 [3,5,7] 编排+授
chiyi2 标记技；白翼=三子技全硬绑轮次（round5 敌弃 2/rounds<3 敌摸-1/
rounds<7 雷免）。FROZEN 名单尾加「星夏」（96 名）。存档 56→**57**。版
本 241→**242**（字段批探针：241+新代码全 14 位移佐证字段源）。回归
`boss_huoshenzhurong_xingxia_yanling_burn` 六段（损焰灵/弃红免伤/同轮
拒/放弃追击/弃牌拒受伤/往返）——证据见
[T11 星夏证据](../operations/evidence/2026-10-11-t11-boss-xingxia-v242/README.md)。
**坑位**：min=0 可放弃询问绝不能 drain_choices（空应答合法→静默吞成放
弃分支，本批三踩）；Gate 1b exit 须取 cargo 真实退出码（管道 tail 吞非
零——死常量 `-D warnings` 拦截后修正）。

**规则 v243（2026-10-11，T11 星宿阶段2系纯入库批：炎帝/少昊双条目）**：
两将技能面全部已覆盖（神裔 v233+神恩光环 v238；赤翼/白翼已处置）——
纯条目无新引擎代码：`BOSS_GENERALS` 尾追加**炎帝**（def 索引 19/hp6/
`[神裔, 神恩]`；names 姜|轨）与**少昊**（def 索引 20/hp6/`[神裔, 神恩]`；
names 嬴|挚）。神恩光环臂为通用查询，新携带者装配即激活。**排除记录**：
共工（带未落技绝红）、明星（庆祝/夹击/急叮三技未落）随各自内容技入库。
**探针新数据点**：纯条目不位移 14 场景哈希（242+新条目复跑全绿——条目
不入标准场景哈希，位移仅版本源）。FROZEN 名单不变（无新技能）。存档
不变 v57。版本 242→**243**。回归
`boss_yandi_shaohao_pure_entries` 五段（元数据/装配/光环通用臂/挑战者
摸牌+1/往返）——证据见
[T11 纯入库证据](../operations/evidence/2026-10-11-t11-boss-entries-v243/README.md)。

**规则 v244（2026-10-11，T11 星宿阶段2系绝红片：绝红＋共工条目）**：
**绝红**（查询型 Noop）单臂（kernel Judge begin 段，封冻同位——准备阶段
=判定阶段开始锚点先例族）：持有者准备阶段敌方全体逐个——无牌跳过；有
装备 → 装备全弃（撤技能+入弃堆）；无装备 → 随机弃一张手牌（**随机→首
张确定性近似**登记先例族）。`BOSS_GENERALS` 尾追加**共工**（def 索引
21/hp5/`[神裔, 绝红]`；names 姜|共工；玄麟3 已处置）。**登记差异**：随
机→首张确定性近似。FROZEN 名单尾加「绝红」（97 名）。存档不变 v57。版
本 243→**244**。版本回退探针（244→243 复跑 golden 全绿）证明 14 golden
位移唯一来源是版本字段，全量重录。回归
`boss_shuishengonggong_juehong_strip_enemies` 五段（装备全弃/首张近似/
无牌跳过/守恒/往返）——证据见
[T11 绝红证据](../operations/evidence/2026-10-11-t11-boss-juehong-v244/README.md)。

**规则 v245（2026-10-11，T11 明星片：庆祝/夹击/急叮三技＋明星条目）**：
**庆祝**两面：跳弃牌——**弃牌机器生成口真实闸门**（Phase{Discard,begin}
dispatch：tanyu/qingzhu → None；**勘误**：v215 贪欲臂的 return 只跳收尾
计数，机器生成未被拦——贪欲回归靠测试 drain 掩盖，本批补闸后贪欲双测
仍绿）+杀禁用（engine 校验臂 qz && !jd → 拒）。**夹击**：准备阶段臂——
环上/下家异侧存活者坐骑（HorsePlus/HorseMinus 槽）全弃。**急叮**：
damageEnd 段臂（灵渠同位）——友方受敌方来源伤害 → 对来源虚拟雷杀
（`LeiSha` kind 载体，nature=Thunder 免改 use_virtual_card 签名）。
`BOSS_GENERALS` 尾追加**明星**（def 索引 22/hp3/female/`[庆祝, 夹击]`——
**急叮不随条目装配**：星囚授予技，庆祝禁杀窗口依赖初始缺席）。**登记差
异**：急叮蓐收加成尾面未建模；势力 shen→群目录近似。FROZEN 名单尾加
「庆祝」「夹击」「急叮」（98/99/100 名）。存档不变 v57。版本 244→
**245**。版本回退探针（245→244 复跑 golden 全绿）证明 14 golden 位移唯
一来源是版本字段，全量重录。回归
`boss_mingxingzhu_qingzhu_jiazu_jiding` 五段（杀禁用/授予面+虚拟雷杀/
夹击/跳弃牌+解除/往返）——证据见
[T11 明星证据](../operations/evidence/2026-10-11-t11-boss-mingxingzhu-v245/README.md)。
**坑位**：min=0 可放弃询问不 drain（星夏老坑再犯两处）；技能 def 误当武
将 def 装配 fellow（垃圾武将致 WrongPhase 假象）；跳弃牌真实闸门在机器
生成口（见勘误）。

**规则 v246（2026-10-11，T11 星宿阶段2系造异片：造异＋颛顼条目）**：
**造异**（查询型 Noop）三面：死亡触发（DeathMachine `PlayerDied` 后——玄
冥/共工死亡且场上无存活同名将 → 造异持有者摸 4+`skills.insert` 授「造
异异变」[中毒/蛇毒先例]）+targetEnabled 双门控（engine 校验前置：玄冥
在场敌方不可为锦囊目标、共工在场敌方不可为基本牌目标，装备不受限）+
**造异异变**（`BOSS_ZAOYI_HP` 运行时授予技，FROZEN 登记）：Judge begin
臂——敌方体力最低者全体 `LoseHp(min)` 失去全部体力。
`BOSS_GENERALS` 尾追加**颛顼**（def 索引 23/hp6/`[神裔, 神恩, 造异]`；
names 姬|颛顼）。**勘误**：玄冥携带自润非造异；造异携带者=颛顼。**登记
差异**：玄冥半面随自润批激活（门控按名查表天然支持）。FROZEN 名单尾加
「造异」「造异异变」（101/102 名）。存档不变 v57。版本 245→**246**。
版本回退探针（246→245 复跑 golden 全绿）证明 14 golden 位移唯一来源是
版本字段，全量重录。回归
`boss_zhuanxu_zaoyi_grant_and_exec` 五段（共工门控/死亡授技/门控解除/
异变波/往返）——证据见
[T11 造异证据](../operations/evidence/2026-10-11-t11-boss-zaoyi-v246/README.md)。
**坑位**：`answer_cards` 会用桃应答濒死救援询问（循环推进须按 prompt 分
流显式 Pass）；造异异变永续——授技后首个判定锚即触发；random 挑战者钉
成共工 def 消除不确定性。

**规则 v247（2026-10-11，T11 星宿阶段2系自润片：自润＋玄冥条目）**：
**自润**（查询型 Noop）单臂（Judge begin 段）：准备阶段全体存活角色按座
位序各摸牌——有装备摸 2、无装备摸 1。`BOSS_GENERALS` 尾追加**玄冥**
（def 索引 24/hp5/female/`[神裔, 自润]`；names 嬴|禺强；玄麟3 已处置）
——**造异玄冥半门激活**（v246 targetEnabled 玄冥分支自此有携带者）。
**连带修正**：造异门控补「使用者须持造异」限定（上游 mod 挂 boss_zaoyi
——v246 版缺失，玄冥入库后门控面扩大才暴露；v246 造异五段复跑绿）。
FROZEN 名单尾加「自润」（103 名）。存档不变 v57。版本 246→**247**。版
本回退探针（247→246 复跑 golden 全绿）证明 14 golden 位移唯一来源是版
本字段，全量重录。回归
`boss_shuishenxuanming_zirun_draw_all` 四段（非对称摸牌/玄冥半门/非持有
者不受限/往返）——证据见
[T11 自润证据](../operations/evidence/2026-10-11-t11-boss-zirun-v247/README.md)。
**坑位**：debug_place 用卡牌 name（"duel"）非 kind key（"juedou"）——两
套命名空间；断言时点须叠加先手回合摸牌与神恩光环。

**mode_boss 收尾清点（2026-10-11，勘验登记批，无版本递增）**：全量枚举
mode/boss.js 条目——已入库 26 条目（四凶+神兽系 v208–v232、四象阶段 1
v233–v237、星宿阶段 2 v238–v247、焰灵陪将 v241）+运行时授予技四件；已
处置=隐藏链八技+青翼/赤翼/白翼+不臣；**未落路线图**=试炼态四将（本技+
intro UI 技）、年兽系（nianrui/mengtai/bianshen 变身族）、阴间系（黑白
无常/罗刹夜叉/牛头马面+bianshen3/4）、标准技混入核对（juece/yizhong/
bazhen/niepan）——bianshen 变身族建议先勘机制承载面（技能组重挂可经运
行时授技承载，changeSkin 视觉面登记）。规则 247/存档 v57 不动——证据见
[收尾清点证据](../operations/evidence/2026-10-11-t11-boss-shen2-closeout/README.md)。

**bianshen 变身机制勘验＋试炼态处置（2026-10-11，勘验登记批，无版本递增）**：变身族三形态——①开局 init 壳（试炼态四将→青龙/朱雀/白虎/玄武代位[均已入库]+捉鬼祛邪魑魅魍魉随机；intro=nobracket UI）：纯视觉壳不落；②bianshen4 死亡 changeBoss（黑白无常/牛头马面）：隐藏链同族不落；③年兽形态轮换（nbianshen：init 换将+hp/maxHp/护甲保留+形态序列轮转）：**可承载**——versus 替补换将先例同构（assembly 重挂+属性保留），随机序列登记差异。年兽系本技逐技勘验后即可逐批落包——证据见
[bianshen 勘验证据](../operations/evidence/2026-10-11-t11-boss-bianshen-survey/README.md)。

**规则 v248（2026-10-11，T11 年兽系首片：年瑞+猛态＋年兽条目；存档 v58）**：
`TurnState.mengtai_draw_pending/mengtai_use_pending: bool` 新字段（**存档
57→58**；canonical_hash 入参——字段新增批）。**年瑞**：Draw 臂固定摸 2
后补摸 2（藏目同位）。**猛态**五臂：准备置位（Judge begin）+摸牌/出牌清
位（跳过路径不清位=上游 phaseBegin 未触发同口径）+弃牌闸门（生成口
use_pending → None，庆祝族同位延伸）+结束补摸 3（旗标消费）。
`BOSS_GENERALS` 尾追加**年兽合体**（def 索引 25/hp12/`[年瑞, 猛态]`；形
态轮换 nbianshen 未落随下批——承载结论见 bianshen 勘验批）。**登记差
异**：形态轮换未落；势力 shen→群目录近似。FROZEN 名单尾加「年瑞」「猛
态」（104/105 名）。存档 57→**58**。版本 247→**248**（字段批探针：247+
新代码全 14 位移佐证字段源）。回归
`boss_nianshou_nianrui_mengtai` 四段（年瑞+2/跳摸跳用补摸 3+跳弃/守恒/
往返）——证据见
[T11 年兽证据](../operations/evidence/2026-10-11-t11-boss-nianshou-v248/README.md)。
**坑位**：TurnState 新字段构造点三处（含 lib test 内嵌 ctor——分批补齐
产生双重插入+漏第三处，E0062/E0063 编译拦截）；End 臂漏旗标消费（回归
抓出）；golden old 表 sha/judge 错位再犯（assert 第四次拦截）。

**规则 v249（2026-10-11，T11 年兽系形态片：形态轮换+忍性＋忍性年兽条目；存档 v59）**：
`PlayerState.nbianshen_toggle: bool` 新字段（**存档 58→59**；canonical_hash
入参——字段新增批）。**形态轮换**（查询型 Noop）单臂（Judge begin 最前
=上游 priority 25）：首回合置位不换将；启动后每回合轮换至另一「形态轮
换」携带将（技能实例清空+general_slots 重挂+apply_general_assembly——
versus 替补换将先例同构；hp/手牌/装备/判定区保留=上游 init 保留口径）。
**随机序列→确定性互斥轮转登记**。**忍性**双臂（damageEnd 段+recoverEnd
头）：持有者非当前回合角色时任意伤害/回复摸 1。`BOSS_GENERALS` 尾追加
**忍性年兽**（def 索引 26/hp12/`[年瑞, 猛态, 忍性, 形态轮换]`）；年兽合
体条目补「形态轮换」。**登记差异**：随机序列→确定性轮转；年兽余量形态
（ruizhi/jingjue/baonu/shouyi）随各自依赖批。**⚠️ 引擎待查**：轮换后多
回合推进触发 Suspended 风暴（中局 assembly 的 Yield 循环——resume 无进
展；单回合内全部面正常），多回合轮换回归待查后补。FROZEN 名单尾加「形
态轮换」「忍性」（106/107 名）。存档 58→**59**。版本 248→**249**（字段
批探针：248+新代码全 14 位移佐证字段源）。回归
`boss_nianshou_nbianshen_rotation_renxing` 两局五段（忍性三面/首回合置
位/次回合轮换/往返）——证据见
[T11 形态轮换证据](../operations/evidence/2026-10-11-t11-boss-nbianshen-v249/README.md)。
**坑位**：general_slots 比较 Option 化；回归循环 open_choice 前置+Suspended
分支（中局 assembly 触发时间片）；skill_name 返回 &str。

**「Suspended 风暴」勘误（2026-10-11，v249 批待查项撤销，无版本递增）**：
逐 resume 探针证实形态轮换后多回合推进全部正常（Accepted/Pending 交替有
进展、零 Suspended）——Yielded 唯一产生点=`steps >= slice`（time_slice
未设时不可达）；v249 首版「风暴」为回归草稿脚手架缺陷（循环缺分支/
open_choice 未前置/应答与推进迭代混计 12 次上限）+轮换双向交替的断言锚
定错误。多回合回归已补（round-3 互斥回切断言通过）。**引擎无缺口，N07
分片续行收敛性无问题**；方法论：引擎异常结论须逐 resume 探针为证——证
据见
[风暴勘误证据](../operations/evidence/2026-10-11-t11-boss-storm-rebuttal/README.md)。

**规则 v250（2026-10-11，T11 年兽系睿智片：睿智＋睿智年兽条目）**：
**睿智**（查询型 Noop）单臂（Judge begin 段，夹击臂后）：global 修剪——
持有者在场时其余存活角色（手牌>1 或装备>1）保留一张手牌+一张装备区内
的牌，弃置其余（装备撤技能+入弃堆）。`BOSS_GENERALS` 尾追加**睿智年兽**
（def 索引 27/hp12/`[年瑞, 猛态, 睿智, 形态轮换]`）。**登记差异**：上游
目标自选保留哪张 → 保留首张确定性近似（先例族）；修剪触发于每个判定锚
（global 面天然覆盖）。FROZEN 名单尾加「睿智」（108 名）。存档不变 v59。
版本 249→**250**。版本回退探针（250→249 复跑 golden 全绿）证明 14 golden
位移唯一来源是版本字段，全量重录（**行级程序化重录**——按名读取当前期
望配新实际值，根治 sha/judge 手抄错位四连犯）。回归
`boss_nianshou_ruizhi_prune` 四段（0 号 3 手 2 装修剪/2 号本就不超不触
发/3 号 3 手 0 装弃至 1/往返+守恒）——证据见
[T11 睿智证据](../operations/evidence/2026-10-11-t11-boss-ruizhi-v250/README.md)。
**坑位**：布阵须清各挑战者初始手牌（身份断言全错的根因）；修剪触发于每
个判定锚（布阵须在 0 号回合前）。

**规则 v251（2026-10-11，T11 年兽系守翼片：守翼＋守翼年兽条目）**：
**守翼**（查询型 Noop）单面：使用所有牌无距离限制（sha_range_free 链追
加[狂笑/神恩同位，单/多目标共用]+顺手距离链 `shouyi_free` 臂）。
`BOSS_GENERALS` 尾追加**守翼年兽**（def 索引 28/hp12/`[年瑞, 猛态, 守翼,
形态轮换]`；形态表上游含 boss_nbaonu 暴怒未落随批，条目只列已转换技）。
**登记差异**：势力 shen→群目录近似。FROZEN 名单尾加「守翼」（109 名）。
存档不变 v59。版本 250→**251**。版本回退探针（251→250 复跑 golden 全
绿）证明 14 golden 位移唯一来源是版本字段，全量重录。回归
`boss_nianshou_shouyi_range_free` 四段（杀隔位/顺牵隔位/非持有者对照/
往返）——证据见
[T11 守翼证据](../operations/evidence/2026-10-11-t11-boss-shouyi-v251/README.md)。

**规则 v252（2026-10-11，T11 年兽系精觉片：精觉（继承胆术）＋精觉年兽条目）**：
**胆术+精觉**（双 SkillDef 查询型 Noop 共用 commit 臂按名双查——ZhaoHuo
同构复用先例）：CardsDiscarded 面（dispatch 新增臂）——玩家失去手牌后非
当前回合且已受伤 → 判定红回 1（铁骑内联口径）。`BOSS_GENERALS` 尾追加
**精觉年兽**（def 索引 29/hp12/`[年瑞, 猛态, 精觉, 形态轮换]`）；胆术
def 供夜叉批复用。**登记差异**：上游 loseAsync/gain/addJudge/equip/
addToExpansion 全卡牌移动事件族 → 原生仅 CardsDiscarded 面；铁骑内联判
定无改判窗。FROZEN 名单尾加「胆术」「精觉」（110/111 名）。存档不变
v59。版本 251→**252**。版本回退探针（252→251 复跑 golden 全绿）证明
14 golden 位移唯一来源是版本字段，全量重录。回归
`boss_nianshou_jingjue_danshu_recover` 四段（判定红回 1/满血对照/非持有
者对照/往返）——证据见
[T11 精觉证据](../operations/evidence/2026-10-11-t11-boss-jingjue-v252/README.md)。
**坑位**：拆桥不造成伤害（对照断言误写伤害期望）；git checkout 误回滚整
测试文件（重加）；debug_place 卡名="dismantle"（name≠kind key）。

**规则 v253（2026-10-11，T11 夜叉批：魔道+魔剑+胆术复用＋夜叉条目）**：
**魔道**（查询型 Noop）：Judge begin 臂自摸 2（藏目/年瑞同位）。**魔剑**
（查询型 Noop）：Play begin 臂——异侧存活全目标 `use_virtual_card(Arch
eryAttack)` 虚拟万箭机器直发（乱常先例；空手敌方不询问直接受伤=AOE 跳
过语义挂账清单口径）。**胆术复用**：夜叉条目含 v252 胆术 def 零改动。
`BOSS_GENERALS` 尾追加**夜叉**（def 索引 30/hp11/`[魔道, 魔剑, 胆术]`）。
**bazhen 混入核对**：八阵=标准装备八卦阵由 Armor 判定机承载（v208–v252
装备面回归覆盖），不入条目技能表。**勘误**：魔眼属罗刹（boss_luocha）
不在夜叉表。**登记差异**：魔剑上游 canUse 过滤→native 机器直发不经
PlayCard 校验（乱常先例）；势力 shen→群目录近似。FROZEN 名单尾加「魔
道」「魔剑」（113/114 名）。存档不变 v59。版本 252→**253**。版本回退
探针（253→252 复跑 golden 全绿）证明 14 golden 位移唯一来源是版本字段，
全量重录。回归 `boss_yecha_modao_mojian_danshu` 四段（魔道/魔剑/胆术复
用/往返）——证据见
[T11 夜叉证据](../operations/evidence/2026-10-11-t11-boss-yecha-v253/README.md)。
**坑位**：万箭打空手敌方不询问直接受伤（AOE 跳过语义）；魔剑 Play begin
触发窗——advance 后清手会错过，须 advance 前布阵。

**规则 v254（2026-10-11，T11 阴间系罗刹批：御兽+魔炎＋罗刹条目）**：
**御兽**（查询型 Noop）：Play begin 臂——异侧存活全目标 `use_virtual_ca
rd(SavageAttack)` 虚拟南蛮机器直发（魔剑同款）。**魔炎**（查询型 Noop）：
commit_cards_discarded 追加段——非当前回合持有者失去牌 → 判定红 → 座位
序首个其他存活受 2 火伤。`BOSS_GENERALS` 尾追加**罗刹**（def 索引 31/
hp12/female/`[魔道, 御兽, 魔炎]`；魔道共用 v253）。**yizhong 混入核对**：
一将成名包防御技（防具栏空时黑杀无效）——非标准包未落登记（随一将批）。
**登记差异**：魔炎目标自选→座位序首其他存活确定性近似；loseEnd 全牌损
失面→CardsDiscarded 面（胆术同口径）；势力 shen→群目录近似。FROZEN 名
单尾加「御兽」「魔炎」（115/116 名）。存档不变 v59。版本 253→**254**。
版本回退探针（254→253 复跑 golden 全绿）证明 14 golden 位移唯一来源是
版本字段，全量重录。回归 `boss_luocha_yushou_moyany` 五段（虚拟南蛮/判
定红火伤/判定黑对照/非持有者对照/往返）——证据见
[T11 罗刹证据](../operations/evidence/2026-10-11-t11-boss-luocha-v254/README.md)。
**坑位**：南蛮 CardKind=SavageAttack（非 Nanman）；御兽/魔炎 def 非表尾
插入冻结表拦截后移真表尾；回归断言口径两处（南蛮伤持续/拆桥不伤害）。

**规则 v255（2026-10-11，T11 阴间系收尾片：爆敛/强征/醉酒/枭首＋白无常/牛头条目）**：
**爆敛**（查询型 Noop）：End 臂摸 2（白无常/牛头共用）。**强征**（查询型
Noop）：End 臂——每名其他有手牌角色首张手牌收归持有者（随机→首张确定
性近似登记；CardsGiven 日志）。**醉酒**（查询型 Noop）：commit_damage 增
幅尾臂（灵渠归零臂前）——source sha 族伤害 +1（终值后灵渠仍可归零=上游
damageBegin4 -11 最晚优先一致）。**枭首**（查询型 Noop）：End 臂——座位
序首个 hp≥self 其他存活受 3 伤害（可放弃→恒发动确定性近似登记）。
`BOSS_GENERALS` 尾追加**白无常**（def 索引 32/hp8/`[爆敛,强征,醉酒]`）与
**牛头**（def 索引 33/hp7/`[爆敛,枭首]`）。**登记差异**：juece 界绝策
[refresh 包]/蛮甲[extra-equip 藤甲]/niepan[boss 本地死亡复活面]未落登
记；黑无常/马面随其技批；势力 shen→群目录近似。FROZEN 名单尾加「爆敛」
「强征」「醉酒」「枭首」（117–120 名）。存档不变 v59。版本 254→**255**。
版本回退探针（255→254 复跑 golden 全绿）证明 14 golden 位移唯一来源是
版本字段，全量重录。回归
`boss_yinjian_baolian_qiangzheng_zuijiu_xiaoshou` 五段+枭首触发面扩展
段（2026-10-11 追加：压 2 号 hp≥self 后 End 首个目标受 3 断言——恒发动
近似技须兼测负例与正例）——证据见
[T11 阴间收尾证据](../operations/evidence/2026-10-11-t11-boss-yinjian-v255/README.md)。
**坑位**：醉酒臂插扣血后无效（移增幅尾——语义序与上游最晚优先一致）；
四 def 非表尾插入冻结表拦截（第 5 次）后移真表尾（锚=MOYANY 块+数组闭
合）；回归两处口径（爆敛强征同 End 合计 5/南蛮伤持续）。

**规则 v256（2026-10-11，T11 阴间系黑无常片：鬼计/太平/索命/吸行＋黑无常条目）**：
**鬼计**（查询型 Noop）：Judge begin 臂——弃己判定区首张（随机→首张确
定性近似登记）。**太平**（查询型 Noop）：Draw 臂 +2（年瑞同位）。**索
命**（查询型 Noop）：End 臂——全部未链接异侧存活横置（linked 直写+Lin
ked 日志，索链甲同款；[1,num] 自选→全部确定性近似登记）。**吸行**（查
询型 Noop）：Judge begin 臂——座位序首个已链接异侧受 1 雷伤+持有者回 1
（目标自选→首个已链接确定性近似登记）。`BOSS_GENERALS` 尾追加**黑无常**
（def 索引 33/hp9/`[鬼计, 太平, 索命, 吸行]`；names 范|无救；bianshen4
已处置）。**登记差异**：鬼计随机→首张、索命自选→全部、吸行自选→首个
已链接（确定性近似先例族）；势力 shen→群目录近似。FROZEN 名单尾加「鬼
计」「太平」「索命」「吸行」（121–124 名）。存档不变 v59。版本 255→
**256**。版本回退探针（256→255 复跑 golden 全绿）证明 14 golden 位移唯
一来源是版本字段，全量重录。回归
`boss_heiwuchang_guiji_taiping_suoming_xixing` 五段（鬼计弃判定/太平+
2/索命全横置/吸行雷伤+回复/四技往返）——证据见
[T11 黑无常证据](../operations/evidence/2026-10-11-t11-boss-heiwuchang-v256/README.md)。
**坑位**：条目插入连锁位移后续索引（黑无常插 33 顺移牛头 33→34——按名
断言绿、按 hp 断言假失败）；回归断言前须驱动至技能所属阶段（索命在 End
臂，Draw 后两相）；条目 hp=上限时回复不可见（前置掉血）。

**规则 v256（2026-10-11，T11 阴间系马面片：炼狱＋马面条目；黑无常内容并入生效）**：
**炼狱**（查询型 Noop）：End 臂——全部其他存活角色各 1 点火焰伤害（无
询问直伤）。`BOSS_GENERALS` 尾追加**马面**（def 索引 34/hp6/`[鬼计, 反馈
, 炼狱]`）。**反馈核对**：FAN_KUI 标准技原生已实现（SkillDef 在表）——
入条目技能表（区别于夜叉 bazhen 装备承载不入表）。**juece 混入勘误**：
马面表列旧绝策——refresh 包仅 rejuece 界绝策，旧绝策未勘未落登记。
**版本勘误**：黑无常批 bump 脚本未执行（提交时 255+内容），黑无常内容
随本批 v256 生效——规则号以内容生效批为准（黑无常 README 已勘误注）。
**坑位**：炼狱 SkillDef 本体漏加（assembly panic 当场拦截）——新技能四
件套（常量/SkillDef/条目/臂）核对应入批收尾清单。FROZEN 名单尾加「炼狱」
（125 名）。存档不变 v59。版本 **256**（含黑无常+马面内容）。版本回退
探针（256→255+内容复跑 golden 全绿）证明 14 golden 位移唯一来源是版本
字段。回归 `boss_mamian_lianyu_fire_all` 三段（全敌火伤/共用+反馈断言/
往返）——证据见
[T11 马面证据](../operations/evidence/2026-10-11-t11-boss-mamian-v256/README.md)。

**规则 v257（2026-10-11，T11 阴间系旧绝策片：绝策＋马面条目补全）**：
**绝策**（查询型 Noop，`BOSS_JUECE`，一将包李儒旧绝策）：commit_cards_di
scarded 追加段——自己回合内其他角色失去最后一手牌（弃后空手）→ 对其 1
点伤害（chooseBool AI→恒发动确定性近似登记）。**马面条目补全**：`[鬼计
, 反馈, 炼狱]` → `+绝策`。**登记差异**：上游 lose/equip/gain 全事件族→
CardsDiscarded 面（胆术同口径）；getIndex 逐目标→每次弃牌事件逐空手者等
价。FROZEN 名单尾加「绝策」（126 名）。存档不变 v59。版本 256→**257**。
版本回退探针（257→256 复跑 golden 全绿）证明 14 golden 位移唯一来源是
版本字段，全量重录。回归
`boss_mamian_juece_last_hand_damage` 五段（非最后手牌不触发/最后手牌触发
/自身 exempt/非持有者对照/往返）——证据见
[T11 旧绝策证据](../operations/evidence/2026-10-11-t11-boss-juece-v257/README.md)。
**坑位**：回归驱动循环禁止裸链式推进——必须锚定目标座位+阶段
（advance_to_play 或 break 条件），跨回合裸推进会以错误身份提交命令。

**炎狱计数器勘误＋两项待查勘验（2026-10-11，勘验登记批，无版本递增）**：
①炎狱计数器口径差异登记（不修正）：上游 getIndex:3=每事件判定三份（每
次他人回合开始判 3 次）；原生 v229=回合级计数器封顶 3（每回合 1 判定跨
回合累计）——总量近似时序分布不同；yanyu_count 已成 v59 存档面，修正属
语义变化待专项批。②first=0 交互待查撤销：虚失翻面臂锚=持有者自己的弃
牌阶段，与 first 无交互；旧登记实为回归驱动链误置，回归持续绿佐证。
③bianshen3 勘验处置：牛头/马面死亡 changeBoss 白/黑无常=bianshen4 同族
关卡流程面不落。规则 257/存档 v59 不动——证据见
[bianshen3/遗留勘验证据](../operations/evidence/2026-10-11-t11-bianshen3-yanjiu-survey/README.md)。

**mode_boss 主线收口评估（2026-10-11，勘验登记批，无版本递增）**：程序化
全量枚举 mode/boss.js 全部 71 条目——已入库 31 条目（主线 Boss 对抗链四
凶→神兽→四象→星宿→年兽/阴间六将 v207–v257）+运行时授予技五件；已处置=
隐藏链八技+bianshen 族+编排三翼+不臣+试炼态。**剩余四族独立子内容集**
（约 34 将/90+ 技，不属主线缺口）：十殿阎罗系（13 将/40 技全族未勘）、
魑魅魍魉系（4 将/9 技，guimei/minbao 机制承载待勘）、吕布变身系（暴怒
changeHp+recastable 复杂面）、群雄乱入系（16 将，大量混入已落技可较快入
库）+年兽散量（气象/damagecount/jiding 本体）。规则 257/存档 v59 不动——
证据见
[mode_boss 收口证据](../operations/evidence/2026-10-11-t11-modeboss-closeout/README.md)。

**规则 v258（2026-10-11，T11 群雄乱入董卓片：酒池＋董卓条目）**：
**酒池**（查询型 Noop）：PlayCard kind 改写臂（engine 转化链追加，武圣/
奇袭/探囊同族）——持有者黑桃手牌 → `kind = Jiu`（酒效果承载：下杀+1/
濒死回血/jiu_used 每回合一次）。`BOSS_GENERALS` 尾追加**董卓**（def 索
引 36/hp20/`[酒池, 强征, 爆敛]`——强征/爆敛 v255 已落零改动混入）。
**登记差异**：与奇袭上游同持择技→native 链奇袭优先（无同持条目，登记）；
势力 shen→群目录近似。FROZEN 名单尾加「酒池」（127 名）。存档不变 v59。
版本 257→**258**。版本回退探针（258→257 复跑 golden 全绿）证明 14 golden
位移唯一来源是版本字段，全量重录。回归
`boss_dongzhuo_jiuchi_viewas` 四段（黑桃当酒+jiu_used/第二坛拒/强征爆敛
混入/往返）——证据见
[T11 董卓证据](../operations/evidence/2026-10-11-t11-boss-dongzhuo-v258/README.md)。
**坑位**：转化链守卫 `if kind != Sha && kind != Le` 排除 plain Sha——酒池
测试须用非杀黑桃牌（闪电=黑桃1）；SkillDef 本体漏加第 6 次（assembly 拦
截）；条目追加位移牛头索引（36 修正）。

**规则 v290（2026-10-16，覆盖清单 N06：铁索连环无懈窗口逐目标对齐）**：
覆盖清单行为差异表原列「铁索无懈仍为整牌级」——原生把铁索连环的 1–2 个
目标包在一个**整牌级**无懈窗口里，任一无懈生效即**全部**目标不横置。
**规则来源核对（上游）**：`card/extra.js::tiesuo` 为 `type: trick`、
`complexTarget: true`、`selectTarget: [1,2]`（**非** `multitarget`），
`content: await event.target.link()` 逐目标执行；`card/standard.js::_wuxie`
`trigger: { player: ['useCardToBegin','phaseJudge'] }`——`useCardToBegin`
**逐目标**触发（`map.tempnowuxie = trigger.targets.length > 1 && !multitarget`
即铁索这类多目标非 multitarget 牌的逐目标询问）；无懈本体
`content` 对目标事件调 `triggerEvent.neutralize()`，仅国战「对势力使用」
分支才额外扩展 `excluded`。→ 上游口径：抵消以**单目标**为单位。
**本批实现**：`ChainMachine` 改为**派发层**（`idx/inner/dispatcher`），对
每个存活目标开一个独立无懈窗口（窗口持有 `NullifyWindowMachine` + 单目
标叶子），抵消只跳过该目标，其余目标照常横置；全部目标结算后铁索仍只入
弃牌堆一次（唯一弃置点不变）。存档 76→**77**（`MachineSnapshot::Chain`
载荷 `emitted` → `idx/inner/dispatcher`）。版本 289→**290**。版本回退探
针（289 + 新编码 + **回退 golden 表**复跑 14 场全绿）证明位移唯一来源是
版本字段：14 个 golden 场景在任何记录点都没有挂起的 Chain 机器（铁索场
景无人持无懈时窗口不挂起），新编码不进入这些哈希；行为面由定向测试覆盖。
回归 `chain_nullify_is_per_target_first_only` /
`chain_nullify_is_per_target_second_only` /
`chain_double_nullify_leaves_nobody_linked`（含同窗口无懈对无懈反转）/
`pause_roundtrip_chain_target_window`（窗口挂起点往返）——证据见
[2026-10-16 N06 证据](../operations/evidence/2026-10-16-n06-chain-per-target-nullify/README.md)。
**登记边界**：同一窗口内第二张无懈反转抵消状态（上游 `map.state` 经父
`_wuxie` 链翻转，语义一致，非差异）；铁索 `recastable`（重铸）面仍为既
有未建模登记，本批不动。

**规则 v291（2026-10-16，D4 Boss 关卡流程子系统 B0 / ADR 0783）**：
`GameMode::Boss` 增可序列化 `BossStageState`（kind/level/孟婆乱入与已死
旗/禁将表/翼武装位旗/pending；默认 `kind=None`，对现有局零可达语义）+
内部原语 `change_boss` / `reinit_boss_seat` / `add_boss_fellow` + 禁将/
乱入旗只读查询空实现。存档 78→**79**（旧档版本门禁拒绝、无迁移）。版
本 290→**291**。版本回退探针（291→290 复跑 golden 全绿）证明位移唯一
来源是规则版本字段：14 个标准场景均为 Prototype，不进入 Boss 哈希分支。
内容技（隐藏链/还人/翼等）自 HostBehavior tag **411** 起留给 B1+；本批
不挂 tag。证据见
[2026-10-16 D4 B0](../operations/evidence/2026-10-16-d4-boss-b0/README.md)。

**规则 v292（2026-10-16，D4 Boss B1 青木隐藏链打样 / ADR 0783）**：
HostBehavior tag **411–414**——青木 / 青木·转阶 / 青木3 / 青木3·转阶
（Death BeforeTriggers：First hide + Normal priority -10 转阶；
`death_cancelled` 拦 DeathMachine；`SixiangChainParams` 参数化复用
`reinit`/`add_fellow`/挑战者重整）。青龙/勾芒条目补挂。存档不变 **v79**。
版本 291→**292**。版本回退探针（292→291 复跑 golden 全绿）证明位移唯
一来源是规则版本字段。登记差异：无 dieAfter 窗、无八座 changeSeat、
changeBoss 座位保留清区摸 4、hide 为日志/stage 近似。证据见
[2026-10-16 D4 B1](../operations/evidence/2026-10-16-d4-boss-b1-qingmu/README.md)。
（原 2026-10-11「隐藏链八技整族处置不落」对青木四技由本批撤销；白芒/
赤炎/玄麟六技仍待 B1 余项或后续批。）

**规则 v293（2026-10-16，D4 Boss B2 白芒/赤炎/玄麟 2/2x / ADR 0783）**：
HostBehavior tag **415–420**——白芒 / 白芒·转阶 / 赤炎 / 赤炎·转阶 /
玄麟 / 玄麟·转阶（同构复用 `SixiangChainParams`：蓐收+两明星、祝融+
两焰灵、共工+一玄冥；heal_all+挑战者重整）。白虎/朱雀/玄武条目补挂。
存档不变 **v79**。版本 292→**293**。版本回退探针（293→292 复跑
golden 全绿）证明位移唯一来源是规则版本字段。登记差异同 B1。证据见
[2026-10-16 D4 B2](../operations/evidence/2026-10-16-d4-boss-b2-sixiang/README.md)。

**规则 v294（2026-10-16，D4 Boss B3 白芒/赤炎/玄麟 3/3x / ADR 0783）**：
HostBehavior tag **421–426**——白芒3 / 白芒3·转阶 / 赤炎3 / 赤炎3·转阶 /
玄麟3 / 玄麟3·转阶（侧座 reinit 复用青木3 `SixiangChainParams`：少昊+
蓐收/明星+明星 fellow、炎帝+祝融/焰灵、颛顼+共工 fellow+次座玄冥）。
蓐收/祝融/共工条目补挂。存档不变 **v79**。版本 293→**294**。版本回退
探针（294→293 复跑 golden 全绿）证明位移唯一来源是规则版本字段。登记
差异同 B1，另含侧座 maxHp-- 未建模、玄麟3 hide 同侧条件近似三人、玄冥
条目不挂玄麟3。证据见
[2026-10-16 D4 B3](../operations/evidence/2026-10-16-d4-boss-b3-sixiang3/README.md)。
（四象隐藏链 2+3 全系至此入内核；翼/还人/bianshen4 仍待 B4。）

**规则 v278（2026-10-14，T8 增量 14：出牌阶段三类计数字段批）**：
`TurnState.bte_used_play_phase`（出牌阶段已用基本/锦囊/装备牌数，
`commit_card_used` 尾按**结算牌种**[view-as 转化后]计数，Play 阶段开始
清零）入哈希，承载 DSL filter
`owner_is_target_play_phase_card_category_count_positive`（ABI 42）与
`owner_alive_play_phase_card_category_count_positive`（ABI 293）双策略
（合取=事件主玩家为持有者 ∧ 事件为 Play 结束[ABI 293 的持有者归因由
「仅当前回合玩家可用牌」化归] ∧ 计数>0；`VmTriggerCtx` 增 phase 捕获）。
`CardKind::is_basic_trick_or_equip` 三类判定：地十种+机关牌为异类不计；
扩展种未归类按三类保守近似（登记差异，同 is_basic_kind 先例）。**登记
差异**：装备牌走 Equipped 帧不经出牌提交，不计数（与
`cards_used_play_phase` 同集；上游 countUsed 装备口径未逐项对照）。
装载器映射 2 名。存档 v64→**v65**（增量 2 起 v63→v64）。版本
277→**278**。字段批探针：版本回退 277 复跑 14
场景哈希三值互异（277 无字段/277+字段/278+字段），证明位移=版本字段+
新增字段双源、无行为面扰动；全量重录。回归
`t8_bte_play_phase_filter_dual_gate_and_phase_kind` /
`t8_bte_zero_count_negative` / `t8_bte_kind_classification`——证据见
[2026-10-14 T8 增量 14](../operations/evidence/2026-10-14-t8-increment14-bte-filters/README.md)。

**规则 v277（2026-10-14，T11 群雄乱入 Boss 专项：庞统片[天狱/栖梧]＋庞
统条目）**：**天狱**（查询型单面，End-begin 臂）：结束阶段自己已横置先
解除（toggle），再将全部未横置的其他存活横置（炼狱/锁足 linked 直写同
款；臂内幂等）。**栖梧**（查询型单面，CardUsed commit 臂娇孝同族）：使
用梅花牌后若已受伤回复 1（机器直发 useCard 同覆盖）。**涅槃/浴火已落（2026-10-17 批 118）**：涅槃（濒死/出牌阶段限定技：弃全部牌+解横置+翻正+摸 3+回复至 3）、
浴火（链式授予神威 + 关卡循环型 1）、神威（摸牌/手牌上限 +min(3,其他存活)）；看破 `kanpo`（offline 转发桩→sb 包）与
`zhuyu`、`ylygniepan`（本仓快照无定义）为外部依赖登记（[证据](../operations/evidence/2026-10-17-t11-boss-pangtong-niepan/README.md)）；**诸葛亮已落（2026-10-17 批 116）**：hp Infinity 承载（`INFINITE_HP` 哨兵）+ 四技
（禳星/隐月/隐月·判/起风 + 起风·闭/改命）+ 转换包 `upstream-liuli/boss-zhugeliang@1.0.0`
（[证据](../operations/evidence/2026-10-17-t11-boss-zhugeliang-four/README.md)）。T9 上游对局复核记录
（体力 Infinity、四技装配、`fengqi`/`mbqianlong` content 命中）见
[证据](../operations/evidence/2026-10-17-t9-lx-content-hooks/README.md)；`mbqianlong` 已落（2026-10-17 批 117：
道心值系统 + 曹髦包，[证据](../operations/evidence/2026-10-17-d5-longxue-mbqianlong/README.md)）；`_CMeffects` 为纯显示面（登记差异），
四个 `mbcm*`（手杀包）与 `ylyg*` 外部依赖仍归外部来源批/庞统条目。
`BOSS_GENERALS` 尾追加**庞统·涅槃凤雏**（def 索引 55/**hp4**/`[天狱, 栖
梧]`；上游 group shen→群近似登记）。FROZEN 名单尾加「天狱」「栖梧」
（180–181 名）。存档不变 v63。版本 276→**277**。版本回退探针（277→276
复跑 golden 全绿）证明位移唯一来源是版本字段，全量重录。回归
`boss_pangtong_tianyu_end_link_all` / `boss_pangtong_qiwu_club_recover`
——证据见
[2026-10-14 庞统片](../operations/evidence/2026-10-14-t11-boss-pangtong-v277/README.md)。

**规则 v276（2026-10-14，T11 十殿阎罗秦广王片：判官/拘魂/望乡＋秦广王
条目）**：**判官**（查询型单面）：延时锦囊（乐/闪电）目标不可指定
（PlayCard 拒绝位 IllegalTargets，帷幕同位；上游 targetEnabled delay→目
标不可指定近似登记；兵粮寸断不在原生目录）。**拘魂**（查询型单面，
End-begin 臂炼狱同位）：结束阶段座位序首个其他翻面（上游
["turnOver","link"] randomGet→恒定翻面+首个确定性近似登记）。**望乡**
（Death 触发遗言机，忘魂同位直写一步机）：死亡时全部有装备的其他角色
装备全弃（绝红拆装管线同款 revoke_equip_skills+入弃堆）；
`MachineSnapshot::BossWangxiang` 新变体（archive tag 81/机哈希 81）。
**还人：登记不落**（关卡流程面换将链+孟婆乱入+禁将表，隐藏链同族随
boss 模式扩展批）。`BOSS_GENERALS` 尾追加**秦广王**（def 索引
54/**hp3**/`[判官, 拘魂, 望乡]`）。FROZEN 名单尾加「判官」「拘魂」
「望乡」（177–179 名）。存档不变 v63。版本 275→**276**。版本回退探针
（276→275 复跑 golden 全绿）证明位移唯一来源是版本字段，全量重录。回归
`boss_qinguangwang_panguan_delay_reject` /
`boss_qinguangwang_juhun_end_flip_first` /
`boss_qinguangwang_wangxiang_death_strip_equips`——证据见
[2026-10-14 秦广王片](../operations/evidence/2026-10-14-t11-boss-qinguangwang-v276/README.md)。

**规则 v275（2026-10-14，T11 十殿阎罗楚江王片：帷幕/反馈复用/冰封＋楚
江王条目）**：**帷幕**（查询型单面，双消费点，新落）：黑色锦囊对持有者
无效——显式目标锦囊（顺手/拆桥/决斗/乐）PlayCard 拒绝位
（IllegalTargets，造异/热闷同族；上游 targetEnabled→目标不可指定近似登
记）＋黑色 AOE 窗口前跳过（按牌实体花色，万箭红天然不触发）；无指向锦
囊面天然不适用=登记。**反馈**：标准 `FAN_KUI` def 复用（原生语义=任意
伤害+手牌/装备与界反馈一致）。**冰封**（Death 触发遗言机，忘魂同位直写
一步机）：死亡时 killer 存活且未翻面则翻面（face_down 写+TurnOvered）；
`MachineSnapshot::BossBingfeng` 新变体（archive tag 80/机哈希 80）。
`BOSS_GENERALS` 尾追加**楚江王**（def 索引 53/**hp4**/`[帷幕, 反馈, 冰封]`）。
FROZEN 名单尾加「帷幕」「冰封」（175–176 名）。存档不变 v63。版本
274→**275**。版本回退探针（275→274 复跑 golden 全绿）证明位移唯一来源
是版本字段，全量重录。回归 `boss_chujiangwang_weimu_black_trick_block` /
`boss_chujiangwang_fankui_reuse_gain` /
`boss_chujiangwang_bingfeng_death_flip_source`——证据见
[2026-10-14 楚江王片](../operations/evidence/2026-10-14-t11-boss-chujiangwang-v275/README.md)。
**勘正**：v269 起批次证据把楚江王误计入已完成（目录实无该条目），本批
实际落包——十殿阎罗真实进度自本批起按 11/13 计（余秦广王/Shikieiki）。

**规则 v274（2026-10-14，T11 十殿阎罗地藏王片：不佛/无量/谛听＋地藏王
条目）**：**不佛**（查询型双面）：伤害入口——受到大于 1 点的伤害改为其
中 1 点（灵渠归零检查前，灵渠读调整后终值）＋准备阶段距离 1 以内其他
存活各受 1 火伤（有来源同上游）。**无量**（查询型双面）：结束阶段摸 2
＋准备阶段体力<3 回复至 3（ctx.recover 双封顶）；建局摸 3 面
（gameDrawAfter/enterGame）登记未落随装配批。**谛听**（查询型单面）：
computed_distance 双向链——持有者算到他人 -1、他人算到持有者 +1（义从
同位）。**大愿：登记不落**（判定改写机 AI 态度面，条目只列已转换技——
宋帝王先例）。`BOSS_GENERALS` 尾追加**地藏王**（def 索引 52/**hp8**/
`[不佛, 无量, 谛听]`）。**显示名勘误**：boss_bufo=不佛（非不覆）。
FROZEN 名单尾加「不佛」「无量」「谛听」（172–174 名）。存档不变 v63。
版本 273→**274**。版本回退探针（274→273 复跑 golden 全绿）证明位移唯一
来源是版本字段，全量重录。回归 `boss_dizangwang_bufo_damage_cap` /
`boss_dizangwang_bufo_prepare_range1_fire` /
`boss_dizangwang_wuliang_prepare_recover_end_draw` /
`boss_dizangwang_diting_distance_both_ways`——证据见
[2026-10-14 地藏王片](../operations/evidence/2026-10-14-t11-boss-dizangwang-v274/README.md)。

**规则 v273（2026-10-14，T11 十殿阎罗都市王片：热闷/炙焚/火刑＋都市王
条目）**：**热闷**（查询型单面，双消费点）：防具槽空时——plain 杀目标不
可指定（PlayCard 拒绝位 IllegalTargets，造异同族；上游 useCardToBefore
cancel→目标不可指定近似登记）＋南蛮/万箭目标队列窗口前跳过
（`dushiwang_remen_blocks`，无懈窗/响应段一并跳过）；火/雷杀不覆盖。
**炙焚**（查询型单面，Prepare-begin 臂持有者域）：准备阶段座位序首个其
他的首张手牌获得并受 1 火伤（randomGet×2→确定性近似登记；无来源同上
游）。**火刑**（Death 触发遗言机）：死亡时全部其他存活按座位序各受 1 火
伤（emitted=座位索引防重入）；`MachineSnapshot::BossHuoxing` 新变体
（archive tag 79/机哈希 79）。`BOSS_GENERALS` 尾追加**都市王**（def 索引
51/**hp4**/`[热闷, 炙焚, 火刑]`）。**显示名勘误**：boss_zhifen=炙焚
（非炽焚）/boss_huoxing=火刑（非祸刑）。FROZEN 名单尾加「热闷」「炙焚」
「火刑」（169–171 名）。存档不变 v63。版本 272→**273**。版本回退探针
（273→272 复跑 golden 全绿）证明位移唯一来源是版本字段，全量重录。回归
`boss_dushiwang_remen_sha_reject_then_armor` / `boss_dushiwang_remen_aoe_skip`
/ `boss_dushiwang_zhifen_prepare_steal_and_fire` /
`boss_dushiwang_huoxing_death_fire_all`——证据见
[2026-10-14 都市王片](../operations/evidence/2026-10-14-t11-boss-dushiwang-v273/README.md)。

**规则 v272（2026-10-14，T11 十殿阎罗平等王片：锁足/阿鼻/平等＋平等王
条目）**：**锁足**（查询型单面）：Prepare-begin 臂——持有者准备阶段全部
其他存活横置（上游 content link 同口径不分阵营；linked 直写索链甲/星囚
同款+Linked 日志）。**阿鼻**（查询型单面）：Damage commit 尾臂（远策同
位）——受有来源伤害后来源受 1 火伤（上游 randomGet fire/thunder→恒定火
近似登记；新伤害无来源→天然无递归）。**平等**（Death 触发遗言机，诛心
同构两波）：死亡时最高体力其他存活（座位序首个）先受 2 火伤、重算池后
再受 1 火伤（上游两波 randomGet×lib.linked randomGet→确定性近似登记）；
`MachineSnapshot::BossPingdeng` 新变体（archive tag 78/机哈希 78）。
`BOSS_GENERALS` 尾追加**平等王**（def 索引 50/**hp4**/`[锁足, 阿鼻, 平等]`）。
FROZEN 名单尾加「锁足」「阿鼻」「平等」（166–168 名）。存档不变 v63。
版本 271→**272**。版本回退探针（272→271 复跑 golden 全绿）证明位移唯一
来源是版本字段，全量重录。回归
`boss_pingdengwang_suozu_link_all_others` /
`boss_pingdengwang_abi_source_fire_no_source_skip` /
`boss_pingdengwang_pingdeng_death_two_wave_fire`——证据见
[2026-10-14 平等王片](../operations/evidence/2026-10-14-t11-boss-pingdengwang-v272/README.md)。

**规则 v271（2026-10-14，T11 十殿阎罗阎罗王片：铁面/铡刀/诛心＋阎罗王
条目）**：**铁面**（查询型单面）：sha 机仁王块扩展——持有者防具槽空时
黑杀无效（视为装备仁王盾；装防具后不生效、八卦判定照常；在
`!qinggang_pierce` 门内被穿透天然覆盖）。**铡刀**（查询型单面）：sha 机
青釭穿透链扩展——来源持有即穿透（上游 inherit qinggang_skill 同口径）。
**诛心**（Death 触发遗言机，热恼同构 emitted 防重入）：死亡时座位序首
个最低体力的其他存活受 2 伤（上游 chooseTarget 最低体力池自选→确定性
近似登记）；`MachineSnapshot::BossZhuxin` 新变体（archive tag 77/机哈希
77）。`BOSS_GENERALS` 尾追加**阎罗王**（def 索引 49/**hp4**/`[铁面, 铡刀,
诛心]`）。FROZEN 名单尾加「铁面」「铡刀」「诛心」（163–165 名）。存档
不变 v63。版本 270→**271**。版本回退探针（271→270 复跑 golden 全绿）
证明位移唯一来源是版本字段，全量重录。回归
`boss_yanluowang_tiemian_black_sha_block` /
`boss_yanluowang_zhadao_pierce_renwang` /
`boss_yanluowang_zhuxin_death_min_hp`——证据见
[2026-10-14 阎罗王片](../operations/evidence/2026-10-14-t11-boss-yanluowang-v271/README.md)。

**规则 v270（2026-10-14，T11 十殿阎罗孟婆片：拾忧/往事/忘魂＋孟婆条目，
字段批）**：**拾忧**（查询型单面）：弃牌机 commit 前臂——其他角色弃牌阶
段弃置牌后持有者获得花色互异者（上游 chooseCardButton 可选→锁定化恒发
动+按弃牌序贪心首个未见花色近似登记；CardsGiven 日志）。**往事**（查询
型单面）：Prepare-begin 臂（苦酒同位）置 `PlayerState.wangshi_ban`＋
PlayCard 拒绝位（CardLimit）＋回合结束全清（反噬全清同位）；上游
randomGet basic/trick/equip→恒定基本牌近似（`is_basic_kind`=杀族/闪/桃/
酒族/手里剑）；上游 getEnemies 敌方面→全部其他近似；**打出面响应窗未落
=登记差异**（仅使用位校验，封冻同口径族）。**忘魂**（Death 触发遗言机，
黑绳前置窗口同位）：死亡时清全场往事封禁＋座位序前两个其他存活各禁用
首个启用技（隐贼同款 enabled 直写+SkillToggled；上游 randomGet 两敌将×
可失技→确定性近似登记）；`MachineSnapshot::BossWanghun` 新变体
（archive tag 76/机哈希 76）。`_status.shidianyanluo_mengpodie` 模式旗与
回魂牌堆注入/回收（boss_mengpohuihun）未落随 boss 模式扩展批。新字段入
canonical hash+存档玩家段，`ARCHIVE_FORMAT_VERSION` 62→**63**。
`BOSS_GENERALS` 尾追加**孟婆**（def 索引 48/**hp3**/female/`[拾忧, 忘魂,
往事]`）。FROZEN 名单尾加「拾忧」「忘魂」「往事」（160–162 名）。版本
269→**270**。字段批探针（269 提交全绿；269+字段/270+字段两态全差且
actual 互异）后全量重录。回归 `boss_mengpo_wangshi_basic_ban_turn_clear`
/ `boss_mengpo_shiyou_distinct_suit_gain` /
`boss_mengpo_wanghun_death_clear_and_disable`——证据见
[2026-10-14 孟婆片](../operations/evidence/2026-10-14-t11-boss-mengpo-v270/README.md)。

**规则 v269（2026-10-13，T11 十殿阎罗转轮王片：魔道共用/轮回/往生/反噬
＋转轮王条目，字段批）**：**魔道**（共用技复用 `BOSS_MODAO`，夜叉/罗刹
先例，零新增）。**轮回**（查询型 Noop）：Judge-begin 臂——准备阶段体力
≤2 时与体力最高且>2 的一名其他角色交换体力值（hp 直写双向 min(maxHp)
封顶同上游；**并列最高取座位序首个**——`max_by_key` 平局取末位故显式
先求最大值再取首个；无事件面直写=登记差异）。**往生**（查询型 Noop）：
Play-begin 臂——视为随机使用【南蛮】/【万箭】，`use_virtual_card` 载体
管线（乱剑/千方同口径）；randomGet→**恒定取南蛮**近似登记；目标=全部
其他存活（机器直发不经校验）。**反噬**（查询型 Noop）：Damage commit
尾臂（极点同位）——每回合首次受伤置 `PlayerState.zlfanshi_terra` 旗，
同回合再受伤→随机其他受 1 伤；旗回合结束全清（封印解除同位）。新状态
字段入 canonical hash+存档玩家段，`ARCHIVE_FORMAT_VERSION` 61→**62**。
`BOSS_GENERALS` 尾追加**转轮王**（def 索引 47/**hp6**/`[魔道, 轮回, 往生,
反噬]`）。**登记差异**：随机→首个近似族；轮回直写无事件面；往生恒定
南蛮。FROZEN 名单尾加「轮回」「往生」「反噬」（157–159 名）。版本
268→**269**。字段批探针（v268 提交全绿；268+字段/269+字段两态全差且
actual 互异）后全量重录。回归 `boss_zhuanlunwang_lunhui_swap_at_judge` /
`boss_zhuanlunwang_wangsheng_virtual_savage` /
`boss_zhuanlunwang_fanshi_damage_toggle`——证据见
[2026-10-13 转轮王片](../operations/evidence/2026-10-13-t11-boss-zhuanlunwang-v269/README.md)。

**规则 v268（2026-10-13，T11 十殿阎罗泰山王片：服毒/苦酒/热恼＋泰山王
条目）**：**服毒**（查询型 Noop）：`commit_card_used` 内联臂（邪酬同
位）——其他角色使用【桃】时随机另一名其他角色失 1 体力（kind 判定含
use_as；候选=全体减使用者减持有者；players>2 门保留；randomGet→座位序
首个确定性近似登记）。**苦酒**（查询型 Noop）：Judge begin 臂位 global
扫持有者——其他角色的准备阶段其失 1 体力并视为使用【酒】（jiu_active
置位不占 jiu_used=上游 useCard jiu 同口径；无 useCard 事件面=登记差异）。
**热恼**（Death 触发 SkillDef=`BOSS_RENAO`，雷诛同位遗言面）：你死亡时
随机一名其他角色受 3 火伤——`BossRenao { owner, emitted }` 单发机
（emit+Yield 可挂起，入重建表）。**显示名勘误**：boss_fudu=服毒/
boss_kujiu=苦酒（会话勘验存量误记复读/苦久，boss.js:11282–11287）。
`BOSS_GENERALS` 尾追加**泰山王**（def 索引 46/hp4/`[服毒, 苦酒, 热恼]`）。
**登记差异**：随机→首个近似族；苦酒无 useCard 事件面；热恼死亡被取消
仍已触发。FROZEN 名单尾加「服毒」「苦酒」「热恼」（154–156 名）。存档
编码追加 BossRenao=75（存档版本不变 v61）。版本 267→**268**。版本回退
探针（267 全绿/268 全差）证明位移唯一来源是版本字段，全量重录。回归
`boss_taishanwang_fudu_kujiu_arms` / `boss_taishanwang_renao_death_fire`
——证据见
[2026-10-13 泰山王片](../operations/evidence/2026-10-13-t11-boss-taishanwang-v268/README.md)。

**规则 v267（2026-10-13，T11 十殿阎罗卞城王片：雷舟/雷斧/雷诛＋卞城王
条目，首个可挂起 boss 机）**：**雷舟**（查询型 Noop）：准备阶段（Judge
begin 臂位）随机一名其他存活受 1 雷伤（吸行同位；randomGet→座位序首个
确定性近似登记；勘验勘误——会话存量误记 Judge begin，源码实为
phaseZhunbeiBegin）。**雷斧**（查询型 Noop）：End begin 臂——随机一名
其他存活横置态切换（上游 `link()` 非恒链同口径，emit
`EventData::Linked` 走 commit_linked 翻转写回；随机→首个近似登记）。
**雷诛**（Death 触发 SkillDef=`BOSS_LEIZHU`，黑绳同位遗言面）：你死亡
时按座位序对全部其他角色各发 1 雷伤——逐目标 emit Damage+Yield（铁索
逐发同款），**内容侧首个可挂起机**：`BossLeizhu { owner, emitted }`
快照携带进度，**入 snap.rs 重建表首例**（此前 boss 一步机单步 Done 不
入表）。`BOSS_GENERALS` 尾追加**卞城王**（def 索引 45/hp4/
`[雷舟, 雷斧, 雷诛]`）。**登记差异**：雷诛死亡前置窗口先于取消判定
（死亡被取消仍已触发）；雷舟/雷斧随机→首个近似。FROZEN 名单尾加「雷
舟」「雷斧」「雷诛」（151–153 名）。存档编码追加 BossLeizhu=74（存档
版本不变 v61）。版本 266→**267**。版本回退探针（266 复跑全绿/267 全
差）证明 14 golden 位移唯一来源是版本字段，全量重录。回归
`boss_bianchengwang_leizhou_leifu_phase_arms` /
`boss_bianchengwang_leizhu_death_thunder_sequential`（真死逐发雷伤+
濒死挂起存档往返续跑不重发）——证据见
[2026-10-13 卞城王片](../operations/evidence/2026-10-13-t11-boss-bianchengwang-v267/README.md)。

**规则 v266（2026-10-11，T11 十殿阎罗宋帝王片：黑绳/胜负＋宋帝王条目，
遗言面首例）**：**黑绳**（Death 触发 SkillDef=`BOSS_HEISHENG`，**Death
前置窗口遗言面 catalog 首例**——changan 氏 host 内联同款；Death 帧
AfterTriggers plan_triggers 早退不重排=遗言恰触发一次）：你死亡时全部
其他存活横置（索命直写同款）。**胜负**（查询型 Noop）：End begin 臂——
首个装备区有坐骑牌的其他存活弃置其中一张（随机目标/牌→首个/首张确定性
近似；equip6 宝物面随宝物栏登记）。`BOSS_GENERALS` 尾追加**宋帝王**
（def 索引 44/hp4/`[黑绳, 胜负]`——**条目只列已转换技**守翼年兽先例；
恩怨=外部包组技 renyuan 继承 xinenyuan1/2 未落随外部包批）。**登记差
异**：黑绳死亡前置窗口先于取消判定（死亡被取消仍已横置）；黑绳机死亡
者存活期触发（forceDie 死后面=时序不敏感）；胜负随机→首个/首张。FROZEN
名单尾加「黑绳」「胜负」（149–150 名）。存档不变 v61。版本 265→**266**。
版本回退探针（266→265 复跑 golden 全绿）证明 14 golden 位移唯一来源是
版本字段，全量重录。回归
`boss_songdiwang_heisheng_death_link_shengfu`（真死横置全其他/胜负弃坐
骑/往返）——证据见
[T11 宋帝王证据](../operations/evidence/2026-10-11-t11-boss-songdiwang-v266/README.md)。
**坑位**：Boss 模式全座位重整复活拦真死（黑绳遗言测试须 chongzheng=0）；
dying_save 询问 min=1 空应答被拒（显式 Pass）；debug_place 置入手牌非装
备区（装备须本人 play）。

**规则 v265（2026-10-11，T11 十殿阎罗五官王片：枳枉/公正/血池＋五官王条
目，十殿阎罗首片）**：**枳枉**（查询型 Noop）：CardsMoved commit 显式臂
（原默认臂加处理）——其他角色获得牌结算后其随机弃一张手牌（随机→首张
确定性近似；摸牌走 CardsDrawn 天然隔离=上游排除 draw 同口径；wugu 机器
直接 hand.push 不发事件表面差异登记；planetarian 彩蛋不落）。**公正**（查
询型 Noop）：Judge begin 臂——判定区有牌弃首张（鬼计同款；随机→首张）。
**血池**（查询型 Noop）：End begin 臂——座位序首个其他存活失 2（随机→
首个）。`BOSS_GENERALS` 尾追加**五官王**（def 索引 43/hp4/`[枳枉, 公正,
血池]`；上游 group **qun→群目录直取**=群雄乱入首个非 shen 条目）。**调试
API 新增**：`debug_give_hand_card`（真实 CardsMoved 事件面测试布阵——gain
类技能测试必需）。**登记差异**：枳枉随机→首张、planetarian 不落；公正/
血池随机→首张/首个。FROZEN 名单尾加「枳枉」「公正」「血池」（146–148
名）。存档不变 v61。版本 264→**265**。版本回退探针（265→264 复跑 golden
全绿）证明 14 golden 位移唯一来源是版本字段，全量重录。回归
`boss_wuguanwang_zhiwang_gongzheng_xuechi`（枳枉 CardsMoved 正例/公正判定
区首张弃/血池失 2/往返）——证据见
[T11 五官王证据](../operations/evidence/2026-10-11-t11-boss-wuguanwang-v265/README.md)。
**坑位**：公正循环按回合跨度估轮数（10 轮不足穿三回合，18 轮）；闪电置判
定区开无懈窗（pass_nullify_windows 放行）；holders.first() 拷贝 u8 不加解引。

**规则 v264（2026-10-11，T11 群雄乱入张角片：天道/雷击·张角/极点＋张角
条目，字段批）**：`TurnState.jidian_chain`（极点链挡旗标）入状态/摘要/
存档（**存档格式 60→61**）。**天道**（查询型 Noop）：**改判机 rewriters
三处扩展**（JudgePhase 机器内联）——①队列构造 filter 加天道持有者（与
鬼道/鬼才并列座位序）；②skip 步可改判条件加「天道+任意 hes 牌」；③选项
步天道持有者全部手牌+装备牌入选项。改判语义=鬼道同款：所打牌成新判定
牌、原判定牌由改判者获得（机器隐式，无独立日志）。**雷击·张角**（触发
CardResponded+extra CardUsed+一步机 ZjLeijiMachine）：使用/打出【闪】后
座位序首个其他存活**内联确定性判定**（铁骑先例，无改判窗=登记）黑→其
1 雷伤+张角摸 1。**极点**（查询型 Noop）：Damage commit 尾臂（章武后）
——张角造成伤害后距受伤者 ≤1（computed_distance 全修饰）的首个其他存
活判定黑→其 1 雷伤；**链挡旗标**（emit 前置位+嵌套臂见旗标清除跳过，挡
恰一层）。`BOSS_GENERALS` 尾追加**张角**（def 索引 42/hp8/`[雷击·张角,
天道, 极点]`）。**同名改名映射（迅影批先例第二例）**：上游 boss_leiji
（张角）与青龙 boss_releiji 显示名同为「雷击」——张角雷击以「雷击·张角」
（BOSS_ZJLEIJI）入目录；映射：原生「雷击·张角」↔boss_leiji、原生「雷
击」↔boss_releiji。**登记差异**：极点雷伤不自我递归（上游 damageAfter
真递归=链挡登记）；极点/雷击判定为内联确定性口径（无改判窗）；AI 态度
面（chooseTarget 不选己）→确定性首个可含张角自身（测试覆盖）；势力
shen→群目录近似。FROZEN 名单尾加「天道」「雷击·张角」「极点」（143–
145 名，顺序=def 表序）。存档 v60→**v61**。版本 263→**264**。字段批
探针：版本回退 263 复跑 14 场景哈希**三值互异**（263 无字段/263+字段/
264+字段）——位移=版本+字段双源、无行为面扰动，全量重录。回归
`boss_zhangjiao_tiandao_rewrite`（闪电判定天道改判+获得原判定牌/往返）+
`boss_zhangjiao_leiji_shan_thunder`（闪后判定黑雷伤+摸 1）+
`boss_zhangjiao_jidian_chain_guard`（极点判定黑雷伤/链挡/自雷伤路径）—
—证据见
[T11 张角证据](../operations/evidence/2026-10-11-t11-boss-zhangjiao-v264/README.md)。
**坑位**：TurnState 构造器在 engine.rs 不在 state.rs（接线脚本锚错文件
×2）；next_phase_settled 假设无前置挂起（循环内须直接应答当前挂起）；天
道应答牌被测试自身 clear（须用当前选项快照）；改判者获得原判定牌无独立
日志（Gained 仅天妒——以 Rewritten+手牌收支断言）；链挡 emit 后立即清除
=无挡（须嵌套臂见旗标清除跳过）。

**规则 v263（2026-10-11，T11 群雄乱入周瑜片：火神/红莲·周瑜/仙音＋周瑜
条目）**：**火神**（查询型 Noop）：commit_damage 火伤分支直读臂（招祸同
位=damageBegin 层）——火属性伤害对你无效：DamagePrevented+回复 1 点体力
（上游 cancel+recover() 无参=1 同口径；取消后增幅不结算；Damage 帧仍走
AfterTriggers=prevent 族同口径登记）。**红莲·周瑜**（查询型 Noop）：End
begin 臂——摸 2+全部其他存活各 1 点火焰伤害。**仙音**（查询型 Noop）：
CardsDiscarded 臂（非当前回合）——失去牌后判定红→座位序首个其他存活失
1（chooseTarget AI→首个确定性近似登记）。`BOSS_GENERALS` 尾追加**周瑜**
（def 索引 41/hp6/`[火神, 红莲·周瑜, 仙音]`）。**同名改名映射（迅影批
先例）**：上游 boss_honglian（周瑜）与 boss_honglianx（毕方）显示名同为
「红莲」——毕方红莲已以「红莲」落目录（FROZEN 冻结不可改名），周瑜红莲
以「红莲·周瑜」（BOSS_ZYHOLLIAN）入目录；映射登记：原生「红莲·周瑜」↔
上游 boss_honglian、原生「红莲」↔上游 boss_honglianx。**登记差异**：仙
音 loseEnd 全牌损失面→CardsDiscarded 面（胆术同口径）；chooseTarget AI
面→首个其他存活；势力 shen→群目录近似。FROZEN 名单尾加「火神」「红莲·
周瑜」「仙音」（140–142 名，顺序=def 表序）。存档不变 v60。版本 262→
**263**。版本回退探针（263→262 复跑 golden 全绿）证明 14 golden 位移唯
一来源是版本字段，全量重录。回归
`boss_zhouyu_huoshen_fire_immune_recover`（火伤免疫+回 1/普通伤害对照/
往返）+`boss_zhouyu_honglian_end_and_xianyin`（红莲 End 摸 2+全场火伤/
仙音判定黑负例+判定红正例/往返）——证据见
[T11 周瑜证据](../operations/evidence/2026-10-11-t11-boss-zhouyu-v263/README.md)。
**坑位**：FROZEN 追加顺序必须=def 表顺序（首版写成火神/仙音/红莲·周瑜
被 standard_skill_ids_append_only 名序断言拦）；仙音判定负例须预置黑桃
（自然判定序不可控恰红假失败）。

**规则 v262（2026-10-11，T11 群雄乱入甄姬片：踏水/凌波/娇孝/芳华＋甄姬
条目）**：**凌波**（触发 CardResponded+extra CardUsed+一步机
LingboMachine）：打出/使用【闪】后摸 2（frequent；viewAs 闪按声明牌
种）。**踏水**（同族触发+TashuiMachine）：使用/打出黑色牌后，座位序首
个其他存活翻面（TurnOverBefore 通道=虚失同款；chooseTarget AI 面→首
个确定性近似登记）。**娇孝**（查询型 Noop）：CardUsed commit 内联臂—
—成为红色牌目标后摸 1（实体花色；闪电自置=targets 空隐式自目标等价分
支）。**芳华**（查询型 Noop）：End begin 臂——存在翻面角色时全部翻面
者各失 1。`BOSS_GENERALS` 尾追加**甄姬**（def 索引 40/hp4/female/
`[踏水, 凌波, 娇孝, 芳华]`）。**登记差异**：娇孝对桃园/五谷/无中隐式
全员（targets 空）不触发（上游 useCardToTargeted 逐目标）；踏水目标
AI 态度面→首个；势力 shen→群目录近似。**顺带修复**：
`debug_move_named_card_to_hand` 手牌回收分支双 push（同 id 双份实体，
区域守恒炸——删函数体内 push，尾部统一保留）。FROZEN 名单尾加「凌波」
「娇孝」「芳华」「踏水」（136–139 名）。存档不变 v60。版本 261→
**262**。版本回退探针（262→261 复跑 golden 全绿）证明 14 golden 位移
唯一来源是版本字段，全量重录。回归
`boss_zhenji_lingbo_jiaoxia_draws`（闪打出摸 2/自置红闪电娇孝摸 1/黑
杀负例）+`boss_zhenji_tashui_flip_and_fanghua`（黑杀翻面/芳华翻面者失
1/往返）——证据见
[T11 甄姬证据](../operations/evidence/2026-10-11-t11-boss-zhenji-v262/README.md)。
**坑位**：debug_set_deck_top_by 实际 push 牌堆尾（判定摸牌「后推先抽」
口径）与 debug_place 头部扫描不衔接；debug_place 手牌回收分支只查不删
（重影——测试用 state_core_mut 精确搬移绕开）；place 只按 kind 匹配
（shandian/shandian_safe 同 kind——按名取须 move_named）；play 者须为
回合玩家；娇孝臂先按 targets 过滤则自置分支死路（先取全部持有者再分
支）。

**规则 v261（2026-10-11，T11 群雄乱入刘备片：枭雄/章武＋刘备条目）**：
**枭雄**（触发 CardUsed AfterTriggers+一步机 XiaoxiongMachine，奇佐同
款）：他人使用基本/锦囊牌（含延时——上游 get.type(card,"trick") 延时算
trick；装备排除）结算结束后获得该牌副本（牌堆底实体入手=上游
createCard 凭空造牌→载体实体登记差异；副本入手不 emit CardUsed 无递
归）。**子技 xiaoxiong_damage**：End begin 臂——回合角色未用牌（上游
countUsed()==0→出牌阶段用牌计数==0 登记差异：响应/装备不计）→对其 1
伤害（刘备自己回合不触发）。**章武**（查询型 Noop）：Damage commit 尾
臂（damageEnd 族）——你受到伤害后来源存活且非自己→你弃手牌首张（若
有）→来源弃手牌首张或受 1 反伤。`BOSS_GENERALS` 尾追加**刘备**（def
索引 39/hp8/`[枭雄, 章武]`）。**登记差异**：枭雄载体实体（花色点数随
载体+消耗库顶）vs 凭空造牌；子技未用牌口径=出牌阶段计数；章武 [1,∞]
自选→恒 1 张、AI 态度面→恒弃手牌首张、来源装备面不弃；势力 shen→群
目录近似。FROZEN 名单尾加「枭雄」「章武」（134–135 名）。存档不变
v60。版本 260→**261**。版本回退探针（261→260 复跑 golden 全绿）证明
14 golden 位移唯一来源是版本字段，全量重录。回归
`boss_liubei_xiaoxiong_gain_and_damage`（入手/子技正负例/自用不触发/
往返）+`boss_liubei_zhangwu_discard_or_damage`（双方弃 1/无手牌反伤/
往返）——证据见
[T11 刘备证据](../operations/evidence/2026-10-11-t11-boss-liubei-v261/README.md)。
**坑位**：first=0 建局 advance_to_play(0) 不跨回合（子技正例须显式推至
目标 End）；章武弃牌源用闪开杀响应询问（改桃）；章武弃与枭雄入手同事
件相抵（断言按相抵写）。

**规则 v260（2026-10-11，T11 群雄乱入张春华片：绝情/无心/伤逝＋张春华
条目）**：**绝情**（查询型 Noop，一将包旧绝情）：commit_damage 入口直
读臂（急冻前——上游 source damageBefore 层）——你造成的伤害改为目标失
去等量体力（emit LoseHp{target, d.amount}+return 不走伤害路径；num=未
增幅原始值与上游 damageBefore 先于增幅同口径；受击/加成类不触发=cancel
同口径；Damage 帧仍走 AfterTriggers 触发扫描与既有 prevent 族同登记）。
**无心**（查询型双面）：①伤害半入口直读臂（绝情后=priority 10 序）——
你受到的伤害改为失去 1 点体力；②延时半 PlayCard 校验臂——持有者不可
用乐/兵（上游 mod targetEnabled delay 禁他人目标；原生乐/兵路由本就只
置他人→等价持有者不可用；闪电自置不受限）。**伤逝**（查询型双面）：
①补牌半 shangshix_draw 四 commit 臂（CardsDiscarded/Damage/LoseHp/
Recover——上游 loseEnd/changeHp 双触发面展开）手牌<4 补至 4；②子技
shangshix2 End begin 臂——hp>1 时全体存活（含自己）各失去 1。
`BOSS_GENERALS` 尾追加**张春华**（def 索引 38/hp4/female/`[绝情, 无心,
伤逝]`）。**登记差异**：绝情/无心入口直读 vs 上游 cancel 后 damageEnd
不入队（prevent 族同口径）；无心+绝情自伤双命中触发序（上游 priority
序 vs 原生固定绝情先）登记；伤逝 loseEnd→CardsDiscarded 面、弃装备等
非 CardsDiscarded 失牌面不触发；AI 估伤标签不落；势力 shen→群目录近
似。FROZEN 名单尾加「绝情」「无心」「伤逝」（131–133 名）。存档不变
v60。版本 259→**260**。版本回退探针（260→259 复跑 golden 全绿）证明
14 golden 位移唯一来源是版本字段，全量重录。回归
`boss_zhangchunhua_jueqing_wuxin_damage_faces`（绝情转失去+对照/往返）+
`boss_zhangchunhua_wuxin_damage_only`（无心伤害半独立局）+
`boss_zhangchunhua_wuxin_delay_and_shangshix`（乐拒/他人照常/补至 4/
shangshix2 全体-1+自补/往返）——证据见
[T11 张春华证据](../operations/evidence/2026-10-11-t11-boss-zhangchunhua-v260/README.md)。
**坑位**：advance_to_play 跨张春华 End 时 shangshix2 先行全体-1 使多段
绝对 hp 断言偏 1（无心面拆独立局+同局改相对值）；出牌阶段限一次杀复用
被拒（分段拆局）；4 人环 0→2 杀距离不足（目标改相邻）。

**规则 v259（2026-10-11，T11 群雄乱入郭嘉片：鬼谋/远策/奇佐＋郭嘉条目，
字段批）**：`PlayerState.boss_mad`+`TurnState.qizuo_copy` 双字段入状态
/摘要/存档（**存档格式 59→60**；`MachineSnapshot::BossQizuo` 变体
archive 68/摘要 66；a0 登记样例）。**鬼谋**（查询型 Noop）：End begin
臂——座位序首个其他未混乱存活 `boss_mad` 置位（随机→首个确定性近似登
记），本人回合收束解除（翻面跳过路径同解；上游 `goMad`=addTempSkill
"mad" {player:"phaseAfter"}，mad 上游为纯标记技+AI 态度面——原生规则面
消费仅鬼谋去重过滤，AI 面登记不落）。**远策**（查询型 Noop）：Damage
commit 尾臂（damageEnd 族，扣血后）——来源≠受伤者且双方存活 hp>0 →
判定：黑→受伤者失 1 / 红→来源失 1（emit LoseHp）。**奇佐**（触发
CardUsed AfterTriggers+一步机 QizuoMachine，乱常机同款）：可复制标准
锦囊 12 种（无懈排除/铁索不复制=上游 AI check 同口径/延时锦囊判定区同
名去重=canUse 同口径/原目标全体存活）→牌堆底实体为载体按原牌种
use_virtual_card（v169 原语）+载体入 qizuo_copy 槽守卫递归（上游
event.parent name 守卫同口径；任一非载体提交即清+回合收束兜底清）。
`BOSS_GENERALS` 尾追加**郭嘉**（def 索引 37/hp4/`[天妒, 鬼谋, 远策,
奇佐]`——天妒=标准技共用随条目授入）。**登记差异**：奇佐触发时机
useCardAfter→CardUsed AfterTriggers 窗口（副本效果先于原牌效果）；副
本载体牌花色点数随载体（上游 createCard 保留）；载体实体入弃堆（v169
既有）；远策 chooseControl 态度选面+rejudge stop+judge 修正回调不落
（恒选项一确定性近似）；濒死未决 hp≤0 跳过；势力 shen→群目录近似。
FROZEN 名单尾加「鬼谋」「远策」「奇佐」（128–130 名）。存档 v59→
**v60**。版本 258→**259**。字段批探针：版本回退 258 复跑 14 场景哈希
三值互异（旧 258 无字段/258+字段/259+字段）——位移=版本字段+新增字段
双源、无行为面扰动，全量重录。回归
`boss_guojia_guimou_mad_lifecycle`（四技实例/mad 置位/收束解除/过滤负
例/往返）+`boss_guojia_yuance_judge_branches`（黑/红双分支/无郭嘉对
照）+`boss_guojia_qizuo_trick_reuse`（无中生有 2+2/链挡/载体槽保持/
杀不复制清槽/往返）——证据见
[T11 郭嘉证据](../operations/evidence/2026-10-11-t11-boss-guojia-v259/README.md)。
**坑位**：min≥1 询问空应答被拒两处复踩（discard_to_limit min=2/无懈窗
Confirm——空应答=ChoicePending 假死象，新增 next_phase_settled/
pass_nullify_windows 助手）；原生无 begin:false 帧直读误导（阶段结束走
NextPhase 推 begin:false 帧，回合收束=Play 起 3 次 NP）；远策过滤勿排
除持有者（上游允许郭嘉自身为来源/受伤者）。

**规则 v227（2026-10-11，T11 毕方批首片：招祸；断链批合并录 v225/v226/
v227）**：`BOSS_GENERALS` 增**毕方**（群/hp25/`[妖兽, 招祸]`，表序插梼
杌后——书精/相柳/朱厌 def 索引后移一位测试同步）。**招祸**两面：伤害
视为火（DamageBefore 触发机相德同款一步直改 nature=Fire）+免疫火伤
（commit_damage 入口直读，诚信免伤同位）。**登记差异**：回合中防具无
效面挂点分散未建模；红莲/炎狱(+switch) 未落随下批。新
`MachineSnapshot::BossZhaohuo` 变体（archive tag 65/canonical_hash
66）；a0 修正 Shedu 撞号（63）与样例重复。FROZEN 名单尾加「招祸」（80
名）。存档不变 v51。版本 224→**227**（断链批合并录：v225 虚失、v226
巨源+存档 v51、v227 招祸）。全量重录 14 golden（224 版+当前代码全位移
佐证）。回归 `boss_bifang_zhaohuo_fire_nature_and_immune` 三断言——证
据见[T11 招祸证据](../operations/evidence/2026-10-11-t11-boss-bifang-v227/README.md)。

**规则 v228（2026-10-11，T11 毕方批续片：红莲）**：
**红莲**两面（权限型 Noop+查询）：①红色牌不计入手牌上限——弃牌机器
双位排除（统计口径只数非红+SelectCards 选项排红，shamolvzhou 先例同
构）；②回合开始（Judge begin 段内联）顺序获牌堆前缀红牌至多 3（非红
即停）+对座位序前 3 名其他角色各 1 火伤（招祸联动恒为火）。**登记差
异**：上游随机（获 0-3/目标 3-0 名）→确定性前缀/座位序近似（随机近似
先例族）。FROZEN 名单尾加「红莲」（81 名）。存档不变 v51。版本
227→**228**。版本回退探针（228→227 复跑 golden 全绿）证明 14 golden
位移唯一来源是版本字段，全量重录。回归
`boss_bifang_honglianx_red_excluded_and_judge_gain`（三目标各-1+获红
牌）——证据见
[T11 红莲证据](../operations/evidence/2026-10-11-t11-boss-honglianx-v228/README.md)。

**规则 v229（2026-10-11，T11 毕方批末片：炎狱——毕方四技全齐）**：
`TurnState.yanyu_count: u8` 回合级计数器（**存档格式 51→52**；canonical
_hash 入参；reset_turn_limits 清零）。**炎狱**两面：激活门控（commit_
damage 尾第八并列臂，hp≤max/2 enable 直写；apply 特判扩至七技）+其他
角色回合开始判定（Judge begin 段内联，中毒判定同位）——enabled 持有
者（≠当前回合）在场且计数<3 → 确定性判定（铁骑先例）：**红色** → 当
前回合玩家受 1 点来自持有者的火伤、计数+1；黑色无伤。`boss_yanyu_
switch` 不落表（门控并入 enabled，先例同构）。**登记待查**：跨回合计
数器精确口径（reset+封顶交互）需专项核对；行为面已验证。FROZEN 名单
尾加「炎狱」（82 名）。存档 51→**52**。版本 228→**229**。版本回退探
针（229→228 复跑 golden 全绿）证明 14 golden 位移唯一来源是版本字段，
全量重录。回归 `boss_bifang_yanyu_judge_fire_max_three`（激活/判定红伤
/计数封顶）——证据见
[T11 炎狱证据](../operations/evidence/2026-10-11-t11-boss-yanyu-v229/README.md)。

## 当前声明与验证边界

| 语义 | 当前行为/项目声明 | 验证边界 |
|---|---|---|
| 触发全序 | 原生键 `(Timing, -priority, optional, seat_distance(turn.current, owner), def_id)`，确定性；上游为玩家主序（座位序自当前回合玩家起）× 桶内优先级 + 同优先级交互选择（2026-10-16 对照） | **决策保留原生确定性全序**（重放/恢复契约要求；上游交互并列不可无交互复现）；跨玩家优先级主序差异按逐技台账继续验收 |
| 无懈 | 结算型锦囊生效前，延时锦囊使用时开窗；来源起问；AOE 与铁索均**逐目标**（v290 对齐上游 `useCardToBegin`） | 已有主动无懈×真实洗牌×恢复组合与铁索逐目标窗口（含挂起点往返）；全部轮次/死亡动态边界仍按覆盖清单 |
| 距离/座距 | 座距只计存活、环形取最小且下限 1；±马按装备 `distance.globalTo/globalFrom`；采用玩法版本=本地 upstream noname v1.11.7（`get.distance`，2026-10-16 核对） | 独立参照模型全对全比对（ADR 0723）+ 装备/死座用例；其余距离类技能仍逐技能验收；`undist`/`isMin` **已登记为未建模差异（无当前内容依赖，附解除条件）**——[证据](../operations/evidence/2026-10-17-rule-distance-unmodeled/README.md) |
| 判定排序/闪电 | 非闪电按放置序在前、闪电在后（组内 FIFO，源自旧线/ADR 0713）；黑桃 2–9 命中 3 雷伤，未中传下家、同名弃置。**采用玩法版本（upstream noname v1.11.7）为 LIFO、无闪电分组**（2026-10-16 核对） | 双放置序、多张 + 快照往返（`judge_order_grouping_multi_cards_and_restore`）及转移用例已有；**已定案保留原生**（非闪电 FIFO + 闪电最后）；与上游 LIFO 差异见语义选择表 / 覆盖清单；死亡剩余机器持牌路径不由终态测试完全覆盖 |
| hlss 星系统（T11 Boss 诸葛亮族批 115，2026-10-17） | 禳星：装配 3 星，受伤害/失血累计 3 → 失一星并向其他角色分配 3 点雷伤（全场随机 1 + 随机敌 2，原生「敌」= `is_boss_side` 异侧、无敌人退化全场），星归零觉醒；隐月：受伤 +1 并记录伤害类型（雷/火/锦囊，供禳星免除：3=雷/2=火/1=锦囊）；光：星未耗尽时体力变化后回满（伤害/失体提交点、濒死判定前）；隐月·判：判定区延时锦囊对持有者不生效（神裔同臂）+ 不能翻面（[证据](../operations/evidence/2026-10-17-t11-boss-zhugeliang-hlss/README.md)） | 装配初始化/隐月 +1/光回满/免除/失星雷伤分配/觉醒/翻面取消/判定失效与对照共 5 测；起风与改命未迁移、Boss 未装配（`T11-BOSS-ZHUGE-LIANG` 仍 open）；态度按 Boss 阵营近似、隐月记录合并 trick/delay 两位、防止伤害窗口内已记录（登记差异） |
| 让节（L10 史诗卡牌切片 2，2026-10-17） | 受伤后可选（tag 516），按**伤害点数逐次**：「获得牌」（摸 1 + 牌堆取一张指定类型牌——基本/锦囊/装备）或「移动牌」（全场装备区/判定区一张牌 → 一名角色：装备且栏位空入装备区、否则入手牌）或取消（[证据](../operations/evidence/2026-10-17-l10-epic-cards-slice2/README.md)） | 3 测（获得/移动/对照）；**登记差异**：牌堆取首张匹配（非搜索面）、移动牌为简化移牌、AI 未迁移 |
| 攻心（swd 批 155，2026-10-17） | 出牌阶段限一次（tag 514）：选一名其他有手牌角色，**展示其全部手牌**，选一张**红桃**牌——**弃置**之，或将其**置于牌堆顶**（`deck.push`；非红桃应答重开窗，等价 `filterButton` 拒绝）（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b155/README.md)） | 3 测（弃置/置顶/非红桃重开 + 对照）；**登记差异**：单窗双按钮拆两窗、置顶日志面、AI 未迁移 |
| 魔化（swd 批 154，2026-10-17） | 濒死变身（tag 513，锁定）：移除 `miles_xueyi`/`aojian`/`mohua2` → 获得撒旦四技（魔焰/灭道/疾风/修罗）→ **体力上限改 4** → 回复 2 → 摸 2；仅触发一次（实例已移除）（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b154/README.md)） | 3 测（变身/对照/仅一次）；**登记差异**：武将名/头像/faction 不迁移（展示面；避免扰动种子化武将分配）、priority 未建模 |
| 修罗（swd 批 153，2026-10-17） | 魔化前置技（tag 512）：准备阶段可选——弃一张**手牌**并弃置判定区一张**同花色**延时锦囊（两窗：手牌 → 同花色判定区；判定区移除记失去牌事件并入弃堆）（[证据](../operations/evidence/2026-10-17-t12-swd-mohua2-prereq-xiuluo/README.md)） | 3 测（双弃 + 两对照）；**登记差异**：单窗双选拆两窗、AI 未迁移、定义取自离线包 |
| 灭道（swd 批 152，2026-10-17） | 魔化前置技（tag 511，组技两面）：体力未满时**摸牌阶段多摸「已损失体力」张**（内联摸牌收束臂，跳摸路径不生效）+ **弃牌阶段强制弃等量张**（手/装备，不可取消；不足则弃全部）（[证据](../operations/evidence/2026-10-17-t12-swd-mohua2-prereq-miedao/README.md)） | 3 测（多摸/强制弃/对照）；**登记差异**：摸面为内联臂、强制弃不足按弃全部、AI 未迁移 |
| 魔焰（swd 批 151，2026-10-17） | 魔化前置技（tag 510）：出牌阶段限一次——体力未满且有红牌时，弃 N 张红/方块手牌（N ≤ min(其他存活数, 体力上限-体力)）并对**恰 N 名**其他角色各造成 1 点火焰伤害（逐帧让出保持目标序）（[证据](../operations/evidence/2026-10-17-t12-swd-mohua2-prereq-moyan/README.md)） | 3 测（选牌上限/目标数联动/双目标火焰伤 + 两对照）；**登记差异**：目标数联动在机器内强制、AI check/result/effect 未迁移 |
| 身份局开局差异的**上游判定**（NC08，2026-10-18） | 依归档上游 `apps/core/mode/identity.js`：`beginner = … || game.zhu || …`（首回合主公）与 `gameDraw` 回调**返回 4**（含主公；仅 `purple` 且座位 >5 为 5）→ **原生一致**（主公座位 0 起手、每人 4 张）；**旧线偏差**（主公 6 张、开局随机伤害无上游依据）。阶段顺序 `Prepare→Judge→Draw→…` → **原生缺 Prepare 阶段**（差异登记，需按准备阶段类技能核对落点）——[证据](../operations/evidence/2026-10-18-nc08-upstream-identity-start/README.md) | 上游判定 3 项通过；Prepare 语义影响与旧线偏差处置待办 |
| 旧线↔原生线开局差异（NC08 对照，2026-10-18） | 同种子同座位、两侧仅做阶段推进时的**公开面对照**确证 4 处差异：①初始阶段/当前座位（旧 `Draw`/座位 1 vs 原生 `Judge`/座位 0）；②初始手牌（旧 `[6,4,4,4]` 主公多 2 张 vs 原生 `[4,4,4,4]`）；③一次推进后（旧 `Play`/座位 1 vs 原生 `Draw`/座位 0）；④旧线开局存在随机伤害类事件（seed 20261602 座位 2 初始 hp=3）。**对齐项**：回合数、体力/上限/存活（身份局主公 +1 上限）。差异属「开局流程/发牌与回合起点」的规则选择，**尚未逐条判定哪一侧符合采用的上游语义**（NC08 待办）；本行只登记差异与影响，不修改任一侧行为——[证据](../operations/evidence/2026-10-18-nc08-cross-line/README.md) | 对照工具 2 项通过；上游判定与事件/选择/随机流/续行面对照待做 |
| NC01 座位语义修正（2026-10-18） | `seat_distance` 的 u8 环绕取模错误已修（5 人局 `d(0,1)` 曾为 0）；**顺序**语义拆出 `seat_steps_from`（座位环顺座次序，`swapSeat` 后随之改变）并迁移触发结算序键与 3 处排序调用方；14 场 golden **哈希与修正前逐字一致**（[证据](../operations/evidence/2026-10-18-nc01-seat-semantics/README.md)） | 6 项回归（2–8 人局逐对/旋转/对称、审计反例、两概念分离、存活环、换位）；**版本升格 v322 与 3 处迁移待并行重构提交**（当前 HEAD 为 v321）**；2026-10-18 NC11 勘正：v322 由 NC07（`seat_ring` 入摘要）承担，NC01 本身不升版本**——依据 `crates/engine-native/tests/replay_scenarios.rs:1061`（「重录依据（v322，NC07 同源审计修正）」） |
| 疾风（swd 批 150，2026-10-17） | 魔化前置技（tag 509）：持技者【杀】目标数上限无限（并入既有上限计算）；多目标使用时**锁定**概率判定 `1-(1.3+n/5)/n` → 整牌取消（写回 `cancelled`，Intrinsic 前生效，日志「疾风·失误」）（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b150/README.md)） | 3 测（多目标接受/未持技拒绝、失误与命中样本并存、单目标必命中）；**登记差异**：仅【杀】族支持多目标（单目标锦囊机器为单目标结算）、千分位 RNG 近似、priority 未建模 |
| 昆仑镜（swd 批 149，2026-10-17） | 组技两面（tag 507）：**阶段收束**锁定记录全场手牌/装备/判定区快照与当时体力；**阶段开始**若体力低于快照值，机器内确认门后**回滚**——把快照牌从任意区拉回原位（装备区变更同步授予/撤销装备技）（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b149/README.md)） | 3 测覆盖快照/破坏现场/回滚 + 对照 + 存档往返；**登记差异**：增量回滚（非完整倒带）、快照不含 viewAs/栏位覆盖、priority 未建模、AI check 未迁移 |
| 神弓与「借用响应」（swd 批 148，2026-10-17） | 神弓（tag 506，被动修饰）：需【闪】/【杀】时可借用**其他角色**的防具/武器作为响应（`shengong_borrow_options` 在响应窗追加候选；应答期由 `shengong_borrowed_owner` 定位所有者并取其装备入弃堆）；覆盖闪响应窗/AOE 响应窗/决斗响应杀窗 3 处；「无闪直接命中」早退先纳入借用候选（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b148/README.md)） | 3 测覆盖借用闪避/取牌 + 两对照；**登记差异**：覆盖 3 窗（非通用响应入口）、槽位走 `card_slot_of` |
| 轩辕剑族（swd 批 146，2026-10-17） | 新牌种 `CardKind::Xuanyuanjian`（武器、射程 3）+ 装备钩（体力 ≤2 且无「轩辕」→ 弃置；否则 +1 护甲）；`xuanyuanjian2`（tag 503）：造成伤害 **+1 并转雷**（非铁索传导）；`xuanyuanjian3`（tag 504）：加伤后无「轩辕」→ 失去 1 体力；`xuanyuan`（tag 505）：结束阶段限定技——弃黑桃牌取弃牌堆【轩辕剑】装备；使用门（无「轩辕」且体力 ≤2 不可用）（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b146/README.md)） | 2 测（加伤/反噬/对照/存档往返）；**牌实体与 8 项种子敏感用例重导留待后续批次**（`T12-SWD-XUANYUANJIAN-CARD`）；**登记差异**：标记位以 `TurnState` 承载、取剑经手牌中转 |
| 燎原（swd 批 145，2026-10-17） | 自己使用【杀】时可选（tag 502）：循环弃同花色牌（首张必问、余 >1 才续问）→ 目标额外需打出等量张【闪】（与无双需求合并为 `needed`）；未能全打出 → **直接命中**且伤害 +张数（`emit_sha_damage` 统一计入并清零）（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b145/README.md)） | 4 测覆盖弃牌/直命/加伤 + 两对照 + 存档往返；**登记差异**：无双与燎原需求合并、加伤时机统一、回合字段承载 storage、AI 未迁移 |
| 涅槃跨来源引用（swd 批 144，2026-10-17） | swd 包 `swd拓跋玉儿` 的 `niepan` 由 deferred 改为 `mapping:"host:niepan"`——**复用批 118** 的宿主行为（tag 471，目录技能名「涅槃」）；不新增引擎行为、无版本变化（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b144/README.md)） | 2 测（授予可用 + 映射键登记）；14 golden 零位移 |
| 改造与装备栏改写原语（swd 批 143，2026-10-17） | 装备牌入槽前（tag 501，`Equipped` **前置窗**——新增 `EventKind::Equipped => BeforeTriggers`）：固有栏位已被占用时可选一个**空栏位**，以 `declared_slots` 覆盖实际栏位（牌种不变 → **装备技能保留**）；`Catalog::card_slot_of(core, card)` 统一栏位查询（46 处实例派生点改走）；`commit_equip` 按栏位替换（修复具名/泛型同栏不互替缺陷）（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b143/README.md)） | 4 测覆盖改栏/保留旧装/对照/存档往返；**连带**：`ai_planner` 冻结步数 218→189（胜方不变，测试内注明）；**登记差异**：以栏位覆盖替代「复制牌面+改名」、filterx 判定近似 |
| 天书（swd 批 142，2026-10-17） | 出牌阶段限定技（tag 500）：弃一张锦囊手牌 → 选一名角色 → 获取其**武将技能表**中自己未持有的一个技能（永久实例；`swdtianshu_from`/`swdtianshu_granted` 记录）；来源阵亡时移除（`DeathMachine` 首步内联臂）（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b142/README.md)） | 4 测覆盖获取/移除/对照/存档往返；**登记差异**：候选不排除限定技（原生无 `unique` 元数据）、用牌消耗在机器内收、候选以「下标 Tag + 重枚举」承载 |
| 李香龙与 `ShaDodged` 事件（swd 批 141，2026-10-17） | 新事件种 `ShaDodged { card, source, target }`（杀被闪避，闪避分支末尾发射）；李香龙（tag 499）：自己的杀被闪避后可选——对攻击范围内一名角色使用一张**虚拟杀**（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b141/README.md)） | 4 测覆盖反击 + 三对照；**golden**：`sha_dodge_then_damage` 单场重录（新增事件入日志；其余 13 场零位移证明状态语义未变）、版本不变；**登记差异**：窗口次序（武器窗口之后）、追加杀为虚拟杀、priority 未建模 |
| 移花（swd 批 140，2026-10-17） | 成为**单目标**牌的目标时可选（tag 498）：弃 2 张手牌（可取消）→ 写回帧 `player=自己 / targets=[原使用者]`——**反弹**（原使用者成为承受者；Intrinsic 前写回即生效）（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b140/README.md)） | 4 测覆盖反弹 + 三对照；**登记差异**：`addedTargets` 防重入标记未建模（可链式反弹、受手牌数限制）、事件重放以帧写回等价；无版本变化、14 golden 零位移 |
| 贞烈与「移出目标」原语（swd 批 139，2026-10-17） | 贞烈（tag 497）成为他人【杀】/锦囊目标时可选：失 1 点体力 + **写回 `CardUseData.targets` 移出自己**（原生 Intrinsic 阶段才构造效果机器，故结算窗写回即生效）+ 弃置使用者一张牌（手/装备）；新护栏：目标清空后「恰一名目标」牌种（杀族/决斗/拆顺/乐兵/借刀/铁索）不再结算（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b139/README.md)） | 4 测覆盖移出目标/失体力/弃牌 + 三对照；**登记差异**：`excluded` 的「不可再指定」面未建模、AI check 未迁移；无版本变化、14 golden 零位移 |
| 天伦（swd 批 138，2026-10-17） | 判定改判窗（tag 496，可选）：候选＝本次判定牌 + 全场判定区牌 + **判定机待结算队列**；选中牌成为判定牌、**旧判定牌弃置**（上游 `judging[0].discard()`），不走「旧牌归改判者」的鬼道/鬼才路径（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b138/README.md)） | 3 测覆盖替换/旧牌弃置 + 两对照；**登记差异**：判定区整体入队（候选含队列牌）、同持多改判技按天伦候选集单次询问、无判定牌时不发动 |
| 脉轮（swd 批 137，2026-10-17） | 任意阶段开始可选（tag 495）：8 选 1 效果——1 失 1 体力上限 +1（上限 6 时不可选）、2 反向、3 首次造成/受到伤害 -1、4 首次伤害 +1（各一次性）、5 少摸一张且上限 +1、6 多摸一张且上限 -1、7 双向距离 -1、8 双向距离 +1；重选时清除旧效果（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b137/README.md)） | 6 测覆盖四类效果/一次性/上限门控/对照/存档往返；**登记差异**：首次伤害以模式位 + 一次性位建模、AI 自动选择未迁移 |
| 极略与通用目标规则表（swd 批 136，2026-10-17） | 极略（tag 494）出牌阶段限三次：看牌堆顶 3 张，从中选一张**基本/锦囊牌**使用（非手牌用牌，牌入手后经 `CardUsed` 结算）。新原语 `GameStateCore::legal_targets(catalog, kind, player) -> (min, max, 目标)`：杀族按射程、桃按已受伤自己、酒/闪电自己、无中无目标、五谷/桃园全体、南蛮/万箭全体其他、决斗任一其他、顺手/拆桥距离内有牌者、铁索 1–2 名、乐/兵无同名牌者、借刀有武器者（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b136/README.md)） | 4 测覆盖用牌/计数/三对照 + 目标表 5 组断言；**登记差异**：目标表为 PlayCard 校验的近似镜像（忽略逐技豁免），仅机器用牌路径使用 |
| 革新印（swd 批 135，2026-10-17） | 出牌阶段限一次（tag 493）：选一种手牌花色**静默弃置**其全部手牌（不触发失去牌类事件）→ 再选另一花色，从牌堆获得**等量**张该花色牌（不足时续取弃牌堆，登记差异）（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b135/README.md)） | 3 测覆盖弃置/静默/等量获得 + 两对照；无版本变化、14 golden 零位移 |
| 冰封与出牌封锁（swd 批 134，2026-10-17） | 冰封（tag 491）出牌阶段限定技：翻面自己与至多 3 名未翻面其他角色 + 失去 1 点体力上限 + 移除玄咒 + 给双方授予 `bingfeng2`（tag 492）；新原语 `SkillBehavior::blocks_all_cards`（`Catalog::blocks_all_cards`）——封锁者**不能使用牌**（`PlayCard` 拒绝，新 `RejectReason::CardBlocked`）、**不能打出闪**（必中）、**不能求桃**（濒死窗无选项）、决斗响应杀与 AOE 响应同样封锁；翻回正面时自移除（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b134/README.md)） | 4 测覆盖翻面/失上限/授予/出牌拒绝/必中/对照/解除；**登记差异**：改判等其余牌类窗口未逐一封锁；无版本变化、14 golden 零位移 |
| 此桥邪（swd 批 133，2026-10-17） | 失去装备牌后（tag 490，频繁技）：按失去张数**逐次**从可得机关牌名中选一张使用（`jiguantong` 全部其他存活 / `gouhunluo`·`jiguanfeng`·`jiguanyuan`·`jiguanyaoshu` 恰 1 名其他 / `zhiluxiaohu`·`mujiaren`·`jiguanshu` 自目标）；取消即止；实体入手经 `CardUsed` 结算（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b133/README.md)） | 4 测覆盖使用与三对照；**登记差异**：候选限当前牌堆/弃堆可得者（上游全局牌名池）、取前 3（上游随机）、单目标前置近似；无版本变化、14 golden 零位移 |
| 魂殉志（swd 批 132，2026-10-17） | 出牌阶段限定技（tag 488）：觉醒（实例禁用）→ 视为对全部其他存活角色使用【万箭齐发】→ 授予**武圣/咆哮**（标准目录技实例）与**殉**（tag 489 旗标）；殉在出牌阶段**收束**时直接死亡（`Death{killer:None}`，内联臂）（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b132/README.md)） | 3 测覆盖万箭/授予/限定/死亡 + 两对照；**登记差异**：万箭目标按全部其他存活角色（上游 `canUse` 过滤未逐一建模）；无版本变化、14 golden 零位移 |
| 护天（swd 批 131，2026-10-17） | 结束阶段可选（tag 487）：选任意张牌（手/装备）寄存于体力上限不低于张数的其他角色处——其体力低于寄存数时回复至该数；寄存期间其**伤害与失去体力**以「体力降至寄存数」为上限（内联两臂）；寄存只跨一个阶段边界，**下一次阶段收束**或寄存方**死亡**时归还（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b131/README.md)） | 4 测覆盖寄存/补体/护体上限/归还/死亡归还/存档往返 + 目标过滤与放弃对照；**登记差异**：寄存牌以专用字段承载（上游 `ui.special`）、归还时点按原生阶段收束 commit 顺序等价建模 |
| 天香（swd 批 130，2026-10-17） | 受到伤害前（tag 485）：弃一张红桃手牌，把该伤害**转移**给一名其他角色（帧 `target` 改写 + 转移链记录）；该角色获得 `tianxiang2`（tag 486，运行时实例）：承受伤害后按本次伤害量摸等量牌并移除（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b130/README.md)） | 4 测覆盖转移/摸牌/移除 + 三对照 + 实例存档往返；**登记差异**：选牌与选目标拆两窗（上游合并窗）、天香·受按伤害量而非实际失去体力摸牌；无版本变化、14 golden 零位移 |
| 颂词（swd 批 129，2026-10-17） | 出牌阶段主动（可重复，tag 484）：选一名未记录角色（**含自己**）——其手牌多于体力则强制弃 2（手/装备），否则摸 2；记录按座位序。组技 `songci_draw`：**全部角色（含阵亡者）**均已记录时弃牌阶段末摸 1（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b129/README.md)） | 4 测覆盖两分支/记录去重/组技与对照/阶段门/存档往返；**登记差异**：强制弃置不足 2 张按弃全部处理 |
| 诛害（swd 批 128，2026-10-17） | 任意角色**结束阶段开始**时（tag 483）：若其本回合造成过伤害（`turn_damage > 0`）、自己手中有【杀】且其在自己攻击距离内，可对其使用一张【杀】（实体牌经 `CardUsed` 结算）（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b128/README.md)） | 4 测覆盖用杀命中/杀离手 + 三对照（无伤害/无杀/未持技）；**登记差异**：`targetEnabled` 以攻击距离近似；无版本变化、14 golden 零位移 |
| 林韵（swd 批 127，2026-10-17） | 出牌阶段主动（tag 482）：两张牌（手牌或装备）当【杀】使用（次张弃置、首张为载荷牌并声明 `Sha`，经 `CardUsed` 帧走杀结算）；组技 `linyun2`「目标须出闪否则直击」在原生杀结算中天然等价（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b127/README.md)） | 4 测覆盖手牌素材/装备素材/两对照；**登记差异**：机器发射的转化杀不计每回合【杀】次数（丈八/乱剑同款）、`directHit` 的防具豁免面未迁移（原生照常先判防具）；无版本变化、14 golden 零位移 |
| 集控（swd 批 126，2026-10-17） | 每回合限一次（tag 481）：自己**阶段开始**或**失去手牌致手牌为空**时，可对攻击距离内一名其他角色使用**虚拟【雷杀】**；次数在**使用时**消费（取消目标窗不消费=上游 `counttrigger--` 返还同口径）（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b126/README.md)） | 4 测覆盖两面/限一次/取消返还/未持技对照；**登记差异**：目标合法性按攻击距离近似（上游 `targetEnabled` 全套）；无版本变化、14 golden 零位移 |
| 失去牌事件与魅惑（swd 批 125，2026-10-17） | 新原语 `EventKind::CardsLost { player, cards, origins }`（来源区 Hand/Equip/Judge 对应上游 `card.original`）：弃置手牌/交出手牌/取走他人牌三处发射（**装备替换不发**，同上游 `equiping` 过滤）；`Ctx::current_frame_id()` 跳过该子帧（`top_frame_id()` 取裸栈顶）以保持既有调用点语义。魅惑（tag 480）：自己失去装备且装备区已空 → 可选选一名其他有牌角色取其一张牌（装备牌直接装备、否则入手）（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b125/README.md)） | 5 测覆盖两路径/两对照/事件挂起存档往返；全量 948/0/1（回归抓漏两处既有语义，见证据「过程勘误」）；**登记差异**：装备替换不发事件、原生无 `equiping` 标记（以不发射等价） |
| 守印（swd 批 124，2026-10-17） | 限定技（濒死救助窗，任意角色濒死）：自己未翻面时发动——自翻面（TurnOverBefore 提交帧）并按座位序将全部已受伤角色回复至满体力（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b124/README.md)） | 4 测覆盖开窗/全员回复/自翻面/限一次/存档往返 + 两对照；**登记差异**：救助窗以可选确认门承载（上游 `save:true` 并入濒死窗） |
| 义山（swd 批 123，2026-10-17） | `yishan2` 记录失去的牌（新在前，表长 ≤16；记录点=弃置/交出/装备离场三类原生路径）；受伤后（`yishan`）取回其中「不属于自己且不在特殊区」者至多 2 张——可跨区取回（弃牌堆/桌面/他人手牌·装备·判定区，装备区取回同步撤销装备技）（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b123/README.md)） | 4 测覆盖记录序/取回上限/出表/跨区/存档往返 + 两对照；**登记差异**：原生无 loseEnd 事件（记录点内联三类丢失路径）、触发为可选窗（上游 notemp 直发）、特殊区（移出游戏）未建模故不记 |
| 入梦（swd 批 122，2026-10-17） | 他人出牌阶段开始前可选：持有者弃一张非基本牌（手/装备）→ 回合角色弃一张基本牌（手牌），否则其**跳过出牌与弃牌阶段**（`skip_discard_phase` 位 + 立即收束出牌阶段）（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b122/README.md)） | 4 测覆盖两路径 + 两对照 + 存档往返；**登记差异**：原生无 phaseBefore 取消位（以「置跳位 + 发射阶段收束帧」等价实现）；无可弃基本牌即等同取消 |
| 神焰（swd 批 121，2026-10-17） | 限定技：自己造成的**火属性**伤害 +1，并记录原目标距离 ≤1 的其他角色（座位序，不含来源与原目标）；主伤害结算后逐个对其造成 1 点**无来源**火伤并清队列（上游 `shenyan`+`shenyan2` 两技 → 原生同行为两面，按状态区分）（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b121/README.md)） | 5 测覆盖加成/蔓延/限定一次/两面按状态区分/存档往返 + 两对照；**登记差异**：蔓延伤害无来源（上游 `damage('fire')` 同口径）、两面合机（上游两技能分挂前后事件） |
| 额外阶段与灵武（swd 批 120，2026-10-17） | `insertPhase` 原语：`TurnState.extra_phases`（阶段码队列）+ 阶段收束时弹出为下一阶段；灵武（tag 475）：频繁技——阶段结束时本回合已用牌数 ≥ 体力则插入一个额外出牌阶段，插入阶段收束不再插入（防递归位）；`cards_used_turn` 为回合级已用牌数（CardUsed +1、回合开始清零）（[证据](../operations/evidence/2026-10-17-t12-swd-deferred-b120/README.md)） | 5 测覆盖插入/不插入（两对照）/防递归/存档往返/换回合清零；**登记差异**：频繁技恒发动（上游不询问同口径）；原生无 PhaseEnd 帧（臂内联于阶段收束）；队列支持任意阶段码（上游默认出牌阶段） |
| 涅槃/浴火/神威（T11 Boss 庞统族批 118，2026-10-17） | 涅槃：限定技（`niepan_used`）——**濒死时**（可选触发）或**出牌阶段**发动：弃置全部牌（手/装/判）、解除横置、翻回正面、摸 3、回复至 3；浴火：涅槃后**链式**授予看破/神威/朱羽（目录存在才授予）并置关卡循环型 `loop_type=1`；神威：摸牌阶段 +min(3, 其他存活数)、手牌上限同量（[证据](../operations/evidence/2026-10-17-t11-boss-pangtong-niepan/README.md)） | 4 测覆盖出牌阶段发动/限一次/浴火链、濒死自救与放弃对照、神威摸牌与手牌上限（含对照）、存档往返；**登记差异**：上游独立 `niepanAfter` 事件 → 原生链式；`oldniepan` 同效果异名不另立；`kanpo`/`zhuyu`/`ylygniepan` 外部依赖未转换；`_status.roundStart` 轮次起点归 D4 |
| 清正（D5 龙血玄黄后继批 119，2026-10-17） | 手杀曹髦道心值 **25 阈值解锁**技能（`mbcmqingzheng`，tag 474）：出牌阶段开始可选——弃置自己一种花色的全部手牌 → 令一名其他角色弃置其一种花色的全部手牌（花色由自己选，可观看其手牌）→ 其弃牌数**少于**自己时受 1 点无来源伤害 （[证据](../operations/evidence/2026-10-17-d5-mbcm-qingzheng/README.md)） | 7 测（含少弃惩罚与足量弃牌对照）；**登记差异**：可弃置性过滤未做（原生手牌默认可弃）、观看手牌为宿主 UI 面（引擎只给花色集合）、`discarder` 归属未建模；酒诗/放逐/绝进仍为外部依赖（翻面后事件/技能禁用/移出游戏区原语缺） |
| 潜龙道心值（D5 龙血玄黄批 117，2026-10-17） | 曹髦 `mbqianlong`：开局面 +20（另有魏势力持「卫统」者 +60）、获得牌 +5、受伤 +10×伤害、造成伤害 +15×伤害，上限 99；阈值 25/50/75/99 解锁清正/酒诗/放逐/绝进（目录存在才授予：清正已转换，酒诗/放逐/绝进仍为外部依赖按上游守卫跳过）；`mbweitong` 为登记面（主公技）。包 `upstream-liuli/longxue@1.0.0`（`mb_caomao` 两技映射）（[证据](../operations/evidence/2026-10-17-d5-longxue-mbqianlong/README.md)） | 5 测覆盖开局面 20/60、累加面与上限、阈值授予、卫统登记与存档往返；**登记差异**：开局面时点后移一个阶段（原生准备阶段开始无帧）、开局摸牌不计 gain、`_CMeffects`/死亡动画纯显示面不迁移；四个 `mbcm*` 与 `mb_epic_caomao` 为手杀包外部依赖（未转换）、`ylyg*` 归庞统条目 |
| 起风/改命与 Boss 装配（T11 Boss 诸葛亮族批 116，2026-10-17） | 起风：准备阶段可选，从 9 种标准非延时锦囊牌名中选一张使用（已用牌名排除；候选以目录牌种码 `ChoiceOption::Kind` 承载）；起风·闭：来源持技时其牌不可被无懈（无懈窗开窗即关）；改命：判定前亮牌堆顶 7 张选 1 作判定结果（其余弃置、改判/天妒窗跳过），放弃则 7 张回牌堆顶照常判定。Boss 包 `upstream-liuli/boss-zhugeliang@1.0.0`（hp `Infinity` + 五技映射）可发现/导入/装配 （[证据](../operations/evidence/2026-10-17-t11-boss-zhugeliang-four/README.md)） | 8 测覆盖起风（选牌名/排除已用/放弃）与改命（改判/放弃回堆）；**登记差异**：起风候选限目录 9 种（上游全 `lib.inpile`、借刀不入）、目标合法性近似、可选窗以确认门承载、改命放弃路径牌堆归还；Boss 关卡流程接入归 D4 后续批，`ylyg*`/`mbqianlong` 归 D5/庞统条目 |
| 无限体力（D5 前置，2026-10-17） | `max_hp == state::INFINITE_HP`（= i32::MAX 哨兵）即无限：伤害/失去体力/回复/上限减都不改变体力（hp 恒等于 max_hp）、永不濒死、手牌上限基数取哨兵（等效无上限）；装载器接受 `"Infinity"` 与哨兵数值。**无规则版本变化**（既有内容不可达该哨兵，14 golden 零扰动）；上游 Boss 诸葛亮体力即 `Infinity`（[证据](../operations/evidence/2026-10-17-d5-infinite-hp/README.md)） | 无限角色受伤/回复/手牌上限/投影/存档与 AI 规划回归；四技（`hlss_xiangxing`/`yueyin`/`fengqi`/`gaiming`）与 Boss 装配未迁移（D5/T11 登记项仍 open）；视图显示哨兵数值不显示 ∞（登记差异） |
| 鬼道/天妒 | 黑手牌或黑装备改判，改判者获得旧牌；天妒限本人最终牌 | 仅装备/混合/恢复、确认/放弃和原事件反例通过，N08-c/d 关闭 |
| 判定事件 | Judged、逐次 Rewritten、Finalized 恰一次且先于 Gained | 原反例已回归；新增洛神/八卦/铁骑等判定路径需独立语义与事件覆盖 |
| 死亡清技能/跨线口径 | 原生死亡即删技能实例；旧线保留实例靠 `alive` 过滤（哈希面不同） | **决策保留原生**；跨线对照按投影后语义（可见状态/事件/终局）而非原始哈希（2026-10-16） |
| 濒死 | 整轮无人用桃才死亡；有桃且仍濒死继续新轮（原生按轮重建名单，重建起点=濒死者）。上游为 `acted` **单趟**（每人至多被问一次、救回者原位复问至 hp>0、无重建）；旧线为同一名单 + 救回者前插复问（2026-10-16 三方对照） | -2 血三桃救活与逐询问恢复已通过；轮次结构三线不同，是否改判上游与「座位序起点」族一并决策 |
| 身份 | 4–8 人配置、阵营胜负、**奖惩先于终局判定**（原生决策；上游为 `checkResult`→`over` 先于 `dieAfter2` 奖惩，2026-10-16 对照，上游 over 后仍发奖惩——2026-10-17 审计关闭） | 配比/范围拒绝、三阵营终局、奖励回归已有；**身份可见视图已投影**（本人/主公/死亡公开，其余隐藏；`PlayerCellView.role`，纯投影不入状态，2026-10-17），身份局逐命令恢复一致（`non_prototype_script_parity`）；主动 `showIdentity` 技能面未建模 |
| 技能与武将 | 已注册补充技能；34 个非样例名称（含原生新增曹仁与 2026-10-07 五批补齐的八将，见下文对齐表）+ 3 样例；武将表有体力 | 六技能已接表；性别有槽时筛选、无槽默认通过；v204 后本块登记差异全部消除 |
| 模式 | 国战明置实际授予表内技能/体力；KOF 引擎层（v83）：会话将池/比分/局数入 GameMode，单局终局自动推进（败者队首淘汰/胜者队首轮转）；对决 versus 2v2 基线（v80）；斗地主（v81）；Boss 基线+重整复活回环（v82） | KOF 空池解析已修；会话行往返/恢复续跑终局一致、单局中途快照恢复哈希轨迹一致、死亡清理已固化（`kof_session_consistency_and_midrestore`，2026-10-07）；versus 奖惩/终局/存档往返已固化（`versus_mode_team_victory_and_rewards`，2026-10-08）；doudizhu 终局/存档往返已固化（`doudizhu_mode_victory_and_roundtrip`，2026-10-08）；versus 4v4 变体已固化（teams 参数化天然支持，`versus_4v4_eight_seats_team_victory`，2026-10-11——无语义变化不递增版本）；versus 替补已建模（v206/存档 v48：死亡座位从本队池弹队首换将复活+摸 4+死亡不结算，池空照常死亡；奖惩不结算/队首序选择为登记差异）；斗地主特殊开局摸牌变体的 binglin 面已建模（v205），online 面（地骑虚拟装备+选将后 infoHp 时点）维持登记；Boss addFellow 中途入队已建模（v84 批三-b：`debug_add_fellow` 队尾扩张+直授 4 张+fellows 登记，`boss_mode_add_fellow_seat_expansion` 固化）；Boss 关卡骨架已建模（v207：gameDraw 分列发牌数+loopFirst 回合起点座位，`new_boss_stage`）；boss 内容包已落二十批（v208 烛阴+凶兽；v209/v210 混沌[四技全齐]；v211–v213 穷奇[五技位全齐]；v215/v216 饕餮[四技全齐]；v217–v219 梼杌[四技全齐]；v220 书精[全技]；v221–v223 相柳[四技全齐+蛇毒面]；v224–v226 朱厌[兵先/虚失/巨源]；v227 毕方+招祸首片——扩展将 def ABI 不变）；boss 专属技能包其余（毕方红莲/炎狱(+switch)、应昭/四象系/天帝系等约 32 将）与 loopType==2 变体未建模（内容面逐批登记）；KOF 多局连续会话循环（终局自动开局下一局）仍在示例/宿主层——引擎只承载会话状态与单局推进；首局大量拒绝后超时判负为示例策略面问题（非内核） |
| 可见性 | 原观星 ChoiceAnswered 脱敏已修；护驾/激将逐人询问不以暗手牌预筛——互换暗响应牌后攻击者视图与公开询问序列不变（v29）；规划信息集为未知池重分配，不复制真实牌堆 ID（v29） | 第 48 轮既有探针为 22 FIXED / 0 OPEN；这不代表所有询问类技能的信息隔离或策略质量已验收，继续按覆盖清单扩面 |
| 预算/时间片 | 段预算/resume/abort 与同步规划节点/时限/预置取消已有回归；真实提交与规划模拟统一用会话高水位（u64::MAX 干净拒绝）；规划信息集为未知池确定性重分配——物理牌集合守恒、空牌堆退化确定（v29 修订） | 单选候选合法回退原反例已于 v31 修复；独立工作池（`PlanPool`）**已实现**（2026-10-14）；命令配额阻塞及更广策略质量仍待验收（见 `T13-AI-STRONGER-POLICY`）；令牌原代次反例已修 |

### 判定区的消费时机（2026-10-19 查明）

**判定区（`player.judge`）在 `Phase { phase: Judge, begin: true }` 事件处被 `std::mem::take` 消费**
（`kernel/intrinsic.rs:1325`；非闪电在前、闪电在后，队首消费）。由此推出两条**实现/夹具约束**：
1. **引擎初态即「0 号判定阶段」**（该 begin 已发出）→ 在 `Engine::new` 之后才放入判定区的牌
   **永远不会被结算**；
2. 夹具要构造「判定」，须在**本轮判定阶段之后**（如 Draw 阶段）放入，由**下一轮**判定阶段消费
   （天妒/洛神等样本即按此修正）。

## 语义选择：窗口/顺序口径（2026-10-16 定案）

原生线多处沿用**旧线**口径（迁移期已固化，并已有成批回归与 14 golden 基线），而采用玩法版本
（本地 `archive/upstream/noname` v1.11.7）在同类位置使用另一套约定。2026-10-16 逐项对照后
**定案：保留原生口径，上游对应行为登记为已知差异（不追）；改判需另立 ADR**（语义变化：递增
`STANDARD_RULESET_VERSION`、重录 14 golden、改造相关测试）。

| 口径 | 项目采用（原生，源自旧线） | 上游 v1.11.7 | 对照证据 |
|---|---|---|---|
| 无懈应答名单起点 | 自**牌来源**起（`seat_order_from(source)`，ADR 0708） | `list.sortBySeat(_status.currentPhase)`（`card/standard.js:4487`） | [AOE/决斗/无懈](../operations/evidence/2026-10-16-aoe-duel-nullify-ask-semantics/README.md) |
| 无懈对无懈 | 单窗口取反 + 自出无懈者下家重建环 | 嵌套 `_wuxie` 窗口（无懈本体重走 `useCardToBegin`） | 同上 |
| 铁索传导目标顺序 | `seat_order_from(受害者)` | `sort(lib.sort.seat)`（起点 `_status.currentPhase`，`library/skill.js:2037-2039`） | [铁索传导/座位序](../operations/evidence/2026-10-16-chain-propagation-seat-order/README.md) |
| 解链时点 | 发传导帧**前**一次解除全部连环 | **逐受害者**结算时解链 + `notLink()` 事后防级联 | 同上 |
| 多目标非 multitarget 锦囊目标定序 | 命令提交序（使用者下家起） | `targets.sortBySeat(_status.currentPhase)`（`content.ts:10143`） | 同上 |
| 判定阶段顺序 | 非闪电放置序 FIFO + 闪电恒最后（ADR 0713） | 判定区数组 **LIFO**、无闪电分组（`content.ts:4931`+`player.js:15791`） | [判定顺序](../operations/evidence/2026-10-16-judge-order-upstream/README.md) |
| 濒死轮次结构 | 按轮重建名单（重建起点=濒死者） | `acted` **单趟**、救回者原位复问 | [濒死名单](../operations/evidence/2026-10-16-dying-save-roster-upstream/README.md) |
| 延时锦囊无懈时点 | 仅**使用时**开窗 | 使用时 **+ 判定时（`phaseJudge`）** 均开窗 | [延时锦囊无懈时点](../operations/evidence/2026-10-16-delayed-trick-nullify-timing/README.md) |
| 临时技能存续期 | 至**拥有者**回合结束（上界近似） | 逐站点 `addTempSkill` expire 规格（默认≈当前阶段结束） | [临时技能存续期](../operations/evidence/2026-10-16-temp-skill-expiry-audit/README.md) |
| 装备即时生效 | 装备提交后**立即**参与距离/射程/防具判定 | 装备走 `equip` 事件（`equipBegin`/`equip` 窗口，`content.ts:1017`） | [装备/触发桶定案](../operations/evidence/2026-10-17-cov-permanent-diffs/README.md) |
| 触发 `getIndex`/桶 | `firstDo`/`lastDo` 由 `Timing` 桶承载；`getIndex` 多次触发不建模 | `arrangeTrigger` 逐 `doing` 消费、`firstDo`/`lastDo` 不做优先级选择（`content.ts:4031`） | [装备/触发桶定案](../operations/evidence/2026-10-17-cov-permanent-diffs/README.md) |

**定案依据**：①翻转属语义变化且会打破既有成批回归/golden 证据链；②数处是原生/旧线为**确定性重放与
恢复契约**选定的结构（判定顺序确定性、窗口状态可编码），上游对应处依赖运行时交互或 UI 约定；
③上游对应行为未被证明与官方规则一致（判定区 LIFO 与"按放置顺序"口径相反；无懈名单起点属上游实现
约定）——故不构成改判充分依据。逐条可复核内容见各证据目录。

## 扩展与规则版本

v3.3 恢复原有扩展机制。单个 STANDARD_RULESET_VERSION 不足以标识任意第三方规则组合：
本局还须固定扩展版本、逻辑哈希、依赖、规则配置与宿主接口版本，并纳入恢复/重放兼容检查。
扩展机器状态和等待选择的续行数据必须编码、校验并参与状态摘要；更新只影响后续开局。
历史或当前原生测试通过均不证明扩展整包已验收；v60 golden 修复已完成，独立执行、生命周期和逐来源功能仍按[扩展台账](../migration/upstream-extension-reconstruction.md)验收，能力边界见[扩展契约](native-extensions.md)。2026-10-17 U03 牌堆、U06 欢乐卡牌、L06 五行、L07 全能搜索、L23 电脑适配、U04 金币、L04 牌堆、L05 金币、L22 段位制裁、L02 SJ Settings、L15 手杀全扩、L17 斗转星移、L18 更新保护、L26 笨蛋插件、L21 武皇争霸 、L16 拖拽读取 与 L13 手杀MVP 八项已从真实安装验收（[U03](../operations/evidence/2026-10-17-t12-u03-eight/README.md)、[U06](../operations/evidence/2026-10-17-t12-u06-eight/README.md)、[L06](../operations/evidence/2026-10-17-t12-l06-eight/README.md)、[L07](../operations/evidence/2026-10-17-t12-l07-eight/README.md)、[L23](../operations/evidence/2026-10-17-t12-l23-eight/README.md)、[U04](../operations/evidence/2026-10-17-t12-u04-eight/README.md)、[L04](../operations/evidence/2026-10-17-t12-l04-eight/README.md)）、[L05](../operations/evidence/2026-10-17-t12-l05-eight/README.md)、[L22](../operations/evidence/2026-10-17-t12-l22-eight/README.md)、[L02](../operations/evidence/2026-10-17-t12-l02-eight/README.md)、[L15](../operations/evidence/2026-10-17-t12-l15-eight/README.md)、[L17](../operations/evidence/2026-10-17-t12-l17-eight/README.md)、[L18](../operations/evidence/2026-10-17-t12-l18-eight/README.md)、[L26](../operations/evidence/2026-10-17-t12-l26-eight/README.md)、[L21](../operations/evidence/2026-10-17-t12-l21-eight/README.md)、[L16](../operations/evidence/2026-10-17-t12-l16-eight/README.md)），计入整包 19/37。苦无、五行的引擎 RNG、确定性弃牌、联机/录像/帮助，全能搜索的界面、语音和脚本执行，电脑适配的窗口、DOM 和抛表情，牌堆随机点数，金币的画布、音效、提示、CSS 和粒子，以及段位制裁的将池不筛、烧条台词和调用方分数，以及 SJ Settings 不请求网络、不 eval update.js、不打开 PDF、不启动 eruda、不调用 game.reload 或 game.loop，以及手杀全扩不播放奶龙音频、不画界面、不调用 clsPrefixName，以及斗转星移不打开菜单、不播放音频、不调用 game.reload、不 eval，以及更新保护不请求网络、不写盘、不 eval、不睡眠，以及笨蛋插件不画界面、不播放音频、不采样 Math.random、不执行已关闭的选择卡牌优化，以及武皇争霸不画界面、不采样 Math.random、不播放录像、不执行已注释的双将体力公式，仍为登记差异，不是上游等价。113 张比例牌是目录材料，缺配置时上游不追加。金币的 `unit_random` 由调用方给出。L05 已按自身安装计入，不由 U04 行计入。L22 已按自身安装计入。L02 已按自身安装计入。其余 18 个来源仍按台账验收。U01 刘冲末焱（`host:d3_moyan`，2026-10-17，[证据](../operations/evidence/2026-10-17-t12-u01-moyan/README.md)）已从真实安装测过。登记差异：开局已在判定阶段内，不补发 roundStart；取消与未选牌立即弃置，不留处理区；展示牌挂在宿主机器（tag 438），不进桌面；获得为手牌推入加 `CardsGiven`，不是上游 gain2。其余 30 将与 56 条技能引用未实现，不计整包。L21、L16、U02 与 L03 计入后总账为 18/37，末焱本身不计。

## 内容差异与后续验收

本地 [standard 技能说明](../../../archive/upstream/noname/apps/core/character/standard/translate.js) 与实现的差异已在逐批收敛：
流离弃牌选择与范围（v76）；倾国黑手牌、救援主公/吴势力、结姻受伤男性、急救回合外（**v202**）；反间花色选择+条件伤害（**v203**）；护驾自选代出+倾国转化（**v204**）——本块登记差异已**全部消除**（2026-10-11）。
保留既有登记惯例：护驾代出转化牌以实体牌响应（上游 result 为虚拟闪）；激将借牌直结算非完整 chooseToRespond 通道（ADR 0776）。
Boss 内容面余项已登记为差异（不实现）：地藏王「大愿」=判定改写机的 AI 态度面（`get.attitude`+`chooseControl`）、「无量」建局摸 3 面（上游 `gameDrawAfter`/`enterGame`）——缺态度模型与「建局帧」原语，见[证据](../operations/evidence/2026-10-17-t11-boss-dizang-remainders/README.md)。
swd（U05 杀海拾遗子包）暂缓技消解推进：**2026-10-17 批 100–114 消解 15 技，映射 133/184 → 150/184（deferred 49→32）**，涵盖单触发/判定/多步机/状态字段、无版本锁定四臂批与 limited 一次性授技字段批；未映射余 32 项（多步/跨包引用/缺事件型）仍为暂缓，逐来源状态见[两仓台账](../migration/upstream-extension-reconstruction.md)。
完整差异和复现见 [进度主稿](native-progress.md)。

### NC08 第⑦层逐技能对照发现的**登记差异族**（2026-10-19）

以下差异由「上游 JS 逐字引用 → 原生行为验收」逐技能对照发现（样本 1–20，17/37 来源；
逐条见[扩展台账 §3.3](../migration/upstream-extension-reconstruction.md#33-nc08-第七层上游行为逐技能对照样本台账2026-10-19)）。
**一律登记为差异，不写成对齐**：

| 差异族 | 上游口径 | 原生现状 | 出现样本 |
|---|---|---|---|
| **取牌位置** | 伤害/使用牌在**处理区**（`position === 'o'`）时可取 | `table` 优先、**弃牌堆兜底**（`xicai`）；`jianxiong` 门槛为「**已在弃牌堆**」 | xicai / jianxiong |
| **计数建模** | 技能标记数（如 `chunbing_markcount = 5`） | 建模为监听技 **`rounds`（剩余回合）** | chunbing |
| **随机取/弃牌** | `hs.randomGet()` / `gainMultiple`（真随机） | **确定性首张**（`hand.first()`；突袭为「各区首张」） | **linghunzhihuo + tuxi** |
| **门槛集合/缺失** | 阵法 `enable` 仅要求「**存在大势力**」；`dizaizhen` 带 **`mode: ["guozhan"]`**（仅国战）；`qiankunbiao` 要求目标 **`he` 有牌** | ①`tianfuzhen` **额外要求存活 ≥ 4**；②**`dizaizhen` 模式门槛未校验**（身份局仍可发动，第 49 项）；③**`qiankunbiao` `he` 门槛未校验**（第 35 项） | **4 例：tianfuzhen + dizaizhen + qiankunbiao + zhongxinghujia（口径自创：手牌>体力 vs !isMin）** |
| **技能覆盖面** | 集智 = **任意锦囊**（`get.type === 'trick'`） | 仅 **5 种**锦囊（南蛮/万箭/决斗/铁索/无懈）→ **规则损失** | jizhi |
| ~~标记/子技面未实现~~ | — | **口径更正（2026-10-19）**：`sbjizhi`/`sbjianxiong` 属 `character/sb/` 面，**非规则基准来源**；基准来源 `character/standard/skill.js` 的集智/奸雄**本就没有**标记面 → **不作为差异** | ~~jizhi / jianxiong~~ |
| **传导事件判定** | `event.notLink()`：铁索传导**不**加成 | duwen：传导为独立事件**照常判定**（已登记）；luoyi：helper **未检查** `notLink`（由调用方决定，**未判定**） | **duwen + luoyi** |
| **阶段计数起点** | `game.phaseNumber`（全局阶段计数） | `turn.phase_number`（局内计数） | ziyu（待核对） |
| **阶段锚点** | 上游有**准备阶段**；原生无（相位序列前移，起手 `Judge` vs 旧线 `Draw`） | 已登记（开局差异判定证据） | 身份局开局对照 |

| **转化入口范围**（标准面 `wusheng`） | `enable: ["chooseToUse","chooseToRespond"]`（**使用或打出**） | **「杀」方向响应转化缺失（规则损失，方向不对称）**：决斗要求打出【杀】时选项只含实体【杀】（`kernel/duel.rs:70`）；而**「闪」方向已实现**（`kernel/sha/step.rs:1207–1243` 龙胆/倾国） | wusheng |
| **主公技分支**（标准面 `wusheng`） | `get.zhu(player,"shouyue")` → **任意**牌可转化 | 原生**未建模**守约（全 crate 无 `shouyue`） | wusheng |
| **门槛历史范围**（标准面 `keji`） | `useCard` **与** `respond` 两历史都查【杀】 | 原生只看 `turn.sha_used_play_phase`（出牌阶段使用） | keji |
| **实现层级**（标准面 `keji`） | 事件级「跳过弃牌阶段」 | 上限级「手牌上限视为无限」（结果等价、事件面不同） | keji |
| **时机锚点**（`luoshen`/`guanxing`） | `phaseZhunbeiBegin`（**准备阶段开始**） | 折入 **`Phase::Judge + begin`**（原生无准备阶段；`luoshen` 注释「判定阶段」、`guanxing` 注释「判定阶段占位」） | **luoshen + guanxing（2 例）** |
| **循环询问粒度**（标准面 `luoshen`） | 每轮 `chooseBool` 询问是否继续 | **整链一次确认**（源码注释自述） | luoshen |
| **目标集合范围**（运筹 `jieyan`/`huoshan` / 古剑 `dinvxuanshuang` / 炉石 `shijieshu`） | `filterTarget: true` + `selectTarget: -1` → **全部角色**（**含使用者**；依据 `library/index.js:11681` `targetEnabled` 对布尔 `filterTarget` 直接返回，**无隐式排除使用者**） | 原生 `seats` 过滤 **`i != self.owner`** → **排除使用者** → **规则损失**（上游应伤害使用者） | **4 例：jieyan（已裁定）+ dinvxuanshuang + shijieshu（自-target）+ huoshan（只受理其他角色，与 toself 相反）** |
| **结算顺序**（swd `jiguantong`） | 上游 `reverseOrder: true` → **逆序**结算 AOE 目标 | 原生按**座位序**（`engine/play_card.rs:1526` 注释自述「结算序=座位序**登记差异**」） | **jiguantong（1 例，待扩展核对其它 AOE）** |
| **生成 vs 实体取牌**（swd `mujiaren`/`yangpijuan`、古剑 `liufengsan`、mtg `longlushanfeng`/`linzhongjianta`） | 上游 `game.createCard(...)` 按**牌定义池****凭空造牌**（候选牌**不属于**牌堆） | 原生从**牌堆/弃堆**取**实体牌**（原生注释自述「实体发现」；候选择定前**留在牌堆**；数量受 `pool.len().min(n)` 限制） | **5 例：mujiaren + yangpijuan + liufengsan（−2）+ longlushanfeng（−1）+ linzhongjianta（−1）** |
| **牌区入口**（`zhiheng`/`qixi`/`guose`/`wusheng`/`lijian`/`qingguo`/`luweiyan`） | `position: "he"`/`"hes"`/`"hs"`（含装备区、判定区） | **命令面/候选生成限定手牌**：`play_card.rs:71` 要求牌在当前回合角色手牌中；`qingguo` 的响应候选只遍历 `.hand` | **7 例** |
| **改判牌区**（标准面 `guicai`） | 非国战 `hs`（手牌 + **装备区**），**不限花色** | 手牌全量 ✅；**装备区候选受 `is_black` 限制**（鬼道口径）→ 鬼才非黑装备不能改判 | guicai |
| **自我目标**（标准面 `qingnang`） | `filterTarget` 仅排除满血（**含自己**） | `activate` 要求 `i != owner` → **不能青囊自己**（规则损失） | qingnang |
| **守约变体**（`wusheng`/`tieji`） | `get.zhu(player, "shouyue")` 时判据变化——**已逐字核对上游**：武圣 `filterCard` 守约时 **`return true`（任意牌）**、否则 `get.color(card) === "red"`；铁骑守约时判据为**非黑桃**、否则**红色** | 两处**均未建模守约变体** | **wusheng + tieji（2 例）** |
| **回合限制**（标准面 `jijiu`） | `viewAsFilter: player !== _status.currentPhase`（**回合外**） | `is_tao_like` 只判花色 → 自己回合内亦可用（**原生更宽松**） | jijiu |
| **取消摸牌的弃置面**（标准面 `tuxi`） | `trigger.changeToZero()`：**取消摸牌**（不摸不弃） | `Read` 阶段**无条件 `hand.pop()` 弃 2 张** → 若固定摸牌尚未发生即弃原有手牌（净变化 0，应为 +取牌数） | tuxi |
| **必中标记通道**（`tieji` 写入 / `wushuang` 读取） | 铁骑判定成功 → `directHit.add(target)`（**通用标记**）；无双 filter **读取** `directHit` 以决定是否加倍 | **两端均未走通用标记**：写入方存 `tieji_red`（杀机器字段）、读取方只判持技 → **同一机制面缺口**（宜一并修：写入方写 `directHit` + 读取方检查） | **tieji + wushuang（2 例，互为印证）** |
| **必中排除**（标准面 `wushuang`） | **`wushuang1`（【杀】方向）**的 filter 排除 `event.getParent()?.directHit.includes(event.target)`（**必中时无双不加倍**）；**`wushuang2`（决斗方向）filter 仅判牌名、无必中语义**（已核对） | `source_has_wushuang` **只判是否持技**，未见 `directHit` 判定（**修复范围仅限【杀】方向**） | wushuang |
| **规则量修饰方式**（`kongcheng`/`qianxun`/`paoxiao`） | `mod`：`targetEnabled`（统一拦截目标）/ `cardUsable`（把【杀】上限设为 `Infinity`） | 显式分支：`kongcheng` **3 处**校验、`qianxun` **2 处**校验、`paoxiao` **限次处 bypass** → 语义等价，但**规则量新增消费点时会漏**（**维护性风险**，非缺陷）；建议一次性收敛为「规则量修饰器」 | **kongcheng + qianxun + paoxiao（3 例）** |
| **主公技资格判定**（`hujia`/`jijiang`/`wang_zun`/`jiuyuan`） | `hasZhuSkill(...)`（**仅身份局**具主公技旗标时成立） | `is_lord` 在**非身份局返回 true** → 四处调用在**原型模式**下均可发动；**经核对为有意设计**（`state.rs:1118` 注释：「无身份体系的模式持技即可」；原型模式非上游模式 → 等价性无定义）→ **设计差异，非缺陷**；旧线 `owner_is_lord` 用显式主公身份（若将来对齐可参照） | hujia（+3 处） |
| **限次表示方式**（标准面 `kuangfu`） | `addTempSkill("stdkuangfu_used", "phaseChange")` 记每阶段一次 | 以「当前回合 + 出牌阶段」判定表达（源码自述**等价**，因触发仅在自身出牌阶段） | kuangfu |
| **事件臂覆盖面**（`xiaoji`/`lianying`/`xinfu_jiyuan`） | 上游以**全局事件 + 多臂**表达：`loseAfter`+5 全局异步面（xiaoji/lianying）、`dying`+`gainAfter`+`loseAsyncAfter`（急援） | 原生只实现**其中一臂/四类显式事件**（枭姬仅 `Equipped{replaced}`；急援仅 `Dying`）→ **3 臂 → 1 臂**类缺口（**规则损失/覆盖面差异**） | **xiaoji + lianying + xinfu_jiyuan（3 例）** |
| **失去事件覆盖面**（标准面 `lianying`） | `loseAfter` + 5 个全局异步面 | 四类显式事件（`CardUsed`/`CardResponded`/`CardsMoved`/`CardsDiscarded`）；注释自述**批量弃置未覆盖** | lianying |
| **阈值 off-by-one**（标准面 `lianying`） | 失去后**手牌为空**才触发 | filter `hand.len() > 1` 在**牌已离手后**求值 → 剩 1 张即触发（**原生更宽松**，实测出现确认门） | lianying |
| **多目标选择权**（标准面 `lijian`） | `selectTarget: 2` + `filterTarget`：玩家手动选 2 名男性（顺序决定谁先出杀） | **自动选目标**（全程仅 `lijian_discard` 询问）→ 玩家无目标/顺序选择权 | lijian |
| **取牌选择权**（`fankui`/`lijian`/运筹 `diyguhuo`/`zonghuo`/`beige`） | 上游凡「玩家选牌」处均为**自选**：`gainPlayerCard({forced:true})`（离间/反馈）、`discardPlayerCard(player,"hej",2,true)`（蛊惑·变体）、`chooseToDiscard`（纵火，**可取消**） | 原生多以**确定性取牌**代替：`.hand` 优先/`equips.first()`（反馈/离间）、**固定顺序**判定区→装备→手牌（蛊惑·变体）、**手牌首张**（纵火）→ 玩家无选牌权 | **5 例：lijian + fankui + diyguhuo + zonghuo + beige（草花分支）** |
| **失去方式覆盖面**（标准面 `xiaoji`） | `loseAfter` + `es`（替换/弃置/获得/被拆**全部**） | 仅 `Equipped{replaced}`（**替换**）→ 规则损失 | xiaoji |
| **`viewAs` 固定优先级**（`qixi`/`guose`/`wusheng`） | 多候选由**玩家选择** | 固定顺序（国色 > 武圣 > 龙胆/奇袭等，`play_card.rs` 注释） | qixi/guose/wusheng |
| **转化候选选择**（标准面 `qixi`） | 多个 `viewAs` 之间由**玩家选择** | **固定优先级**（杀类优先，`play_card.rs:156`）；黑色【杀】持奇袭时只作【杀】 | qixi |
| **摸牌数固定分支**（标准面 `yingzi`） | `filter: !event.numFixed`（摸牌数被固定时**不**加成） | filter **只判阶段**；全 crate 无 `numFixed`/`num_fixed` | yingzi |
| **摸牌数固定分支**（`luoyi`/`tuxi` 同族） | `filter: !event.numFixed`（`luoyi` 另含 `num > 0`） | filter 只判阶段（同一缺口；三技能同族） | **yingzi + luoyi + tuxi** |
| **弃牌执行时机**（口径，非差异） | — | 上限裁剪发生在**进入弃牌阶段时**（探针实测 15→4）；夹具须在**回合前**堆手牌 | keji |

**标准面「原生更宽松」汇总（2026-10-19）**：逐技能对照发现 **5 处**原生比上游宽松的差异（护驾经核对移出，见下）：

| 技能 | 更宽松之处 | 上游依据 | 修复项 |
|---|---|---|---|
| `jijiu` 急救 | 缺**回合外**限制（自己回合内亦可用红牌当桃） | `viewAsFilter: player !== _status.currentPhase` | 清单第 10 项 |
| 兵粮寸断（牌种） | **无距离限制**（对所有人） | 延时锦囊默认距离 1 | 清单第 13 项 |
| `guicai` 鬼才 | 装备区候选受 `is_black` 限制（鬼道口径）→ **手牌面**反而更宽 ✓ 装备面更窄 ✗ | 非国战 `hs` 不限花色 | 清单第 11 项 |
| `lianying` 连营 | **阈值 off-by-one**：剩 1 张手牌即触发 | 失去后手牌为空 | 清单第 14 项 |
| `fankui` 反馈 | **自伤时仍可拿手牌**（上游自伤仅装备区） | `event.source !== player ? "he" : "e"` | 清单第 18 项 |
| ~~`hujia` 护驾~~ | ~~非身份局亦可发动~~ | 经核对为**有意的设计差异**（原型模式非上游模式）→ **移出本表** | — |

**标准面规则损失汇总（2026-10-19，25 个标准面样本）**：逐技能对照发现**7 处规则损失**（另有 11 个差异族，见上表）
（均有行为/源码证据，修法与验收见[待实施修复清单](../operations/evidence/2026-10-18-blocked-fixes/README.md)）：

| 技能 | 规则损失 | 上游依据 | 修复项 |
|---|---|---|---|
| `jizhi` 集智 | 仅覆盖 **5 种**锦囊（应为**任意锦囊**） | `get.type(event.card) === "trick"` | 清单第 4 项 |
| `ganglie` 刚烈 | **缺**「来源弃置两张手牌」分支 | `source.chooseToDiscard({selectCard: 2})` | 清单第 5 项 |
| `wusheng` 武圣 / `longdan` 龙胆 | **「杀」方向响应转化缺失**（「闪」方向已实现，方向不对称） | `enable: [chooseToUse, chooseToRespond]` | 清单第 7 项 |
| `zhiheng` 制衡 | 候选**只取手牌**，缺**装备区**牌 | `position: "he"` | 清单第 8 项 |
| `qingnang` 青囊 | **不能自我治疗**（`activate` 排除 owner） | `filterTarget` 仅排除满血（含自己） | 清单第 9 项 |
| `xiaoji` 枭姬 | 只覆盖**替换**失去装备（拆/弃/获得不触发） | `loseAfter` + `es` 计数（**所有**失去方式） | 清单第 12 项 |
| `shushen` 淑慎 / `yiji` 遗计 / `xiaoji` 枭姬 | **单次结算**（回复/伤害 N 点、失去 N 张只触发 1 次） | `getIndex` → `event.num`（点数）/ `evt.es.length`（**失去装备张数**）→ **按数量计次**；标准面 `getIndex` 共 4 处，其中 3 处属本族（`xinfu_jiyuan` 返回玩家数组，**不属**） | **族（3 例）：清单第 16 项（16.1–16.4）** |

**共性判读（不夸大）**：4 处损失集中在**「候选集合/转化入口」**类——即**规则覆盖面**而非结算逻辑；
结算主路径（伤害、弃置、摸牌、判定）在 7 个样本中均与上游一致。

**中文名须取自 `translate.js`（2026-10-19，样本 88–101 批量更正）**：卡牌**中文名不可按拼音猜**
（本会话曾把 `liutouge` 猜为「六头葛」（实为**六骰格**）、`shatang` 猜为「砂糖」（实为**沙棠**）、
`yunvyuanshen` 猜为「玉女元神」（实为**玉女元参**）、`liufengsan` 猜为「六风散」（实为**流风散**）等，
共 **9 处**）⇒ 一律**读上游 `translate.js`** 取值；**历史证据目录不因改名回改**（保留日期语境）。

**按卡名 grep 会漏（2026-10-19，样本 97 黑龙鳞片）**：宿主**行为枚举名可能与卡名不同**
（`HostBehavior::JiguanHeilongCard` ↔ 卡名 `heilonglinpian`；玉女元参为 `*_LISTENER_KEY`）
⇒ **按卡名检索会漏判为「无实现」**。做法：按**行为枚举名**、**键常量**（`GJ_*_KEY`）或
`keys.rs` 的 `host:*` 映射检索；必要时**顺 `intrinsic.rs` 的 `CardKind::X =>` 分支**找到真实行为。

**技能授予的探测方式（2026-10-19，样本 99 玉女元参）**：判定「某子技是否被授予」时，
**不要凭上游键猜名**（原生可能用 `*_LISTENER_KEY` 等**不同词**命名——玉女元参即为
`gj_yunvyuanshen_listener`）。做法：**先读原生 `constants.rs` 的键常量**（或 `keys.rs` 的
`host:*` 映射），**再按该名探测**；否则会**漏判为「未授予」**（本会话已因此产生一次误判并更正）。

**AOE 卡的目标形态有两种（2026-10-19，样本 116 元素毁灭）**：同为 `selectTarget: -1`，
部分卡按「**全体显式列表**」受理（如 `vec![1,2,3]`），部分卡按「**无显式目标**」受理
（`vec![]`，引擎自算目标）⇒ **夹具须两种都试**，否则会误判为「不可发动」。

**打断窗口的形态（2026-10-19，样本 87 金莲珠）**：宿主「打断/响应」窗口可能是
**`Confirm`/`Pass`**（如 `gj_jinlianzhu_confirm`）而**不是**「选牌」→ 夹具若按「选项含该牌」筛选
会**误判为窗口不可达**。做法：**按询问名匹配**（含技能/卡名片段）并回答 `Confirm`。

**置顶牌须回读核对（2026-10-19，样本 81 烧营）**：`debug_set_deck_top(name)` **按牌种**取牌，
同种牌可能有多张**不同花色** → 按花色挑牌后传**牌名**会**静默失配**（放上去的可能是黑牌）。
做法：**置顶后回读 `deck.last()` 的花色**（必要时换名重试），或改用按 id 置顶。
误判后果：会被当成「技能效果未执行」→ 本会话因此一度错误登记为「跨技能共性缺陷」（已撤回）。

**源码断言写法（2026-10-19，3 次踩坑后固化）**：对**注释/多行代码**做子串断言时，
**不要**用长跨行短语（注释按行宽折行 → 长短语匹配失败）。做法：①用**短片段**分别断言
（如「`chooseTarget 无`」+「`过滤器`」）；②或用**全文件**检查；③定位代码时**锚定唯一子串**
（如 `qingguo && black`）再向两侧取窗口。本会话因长短语断言失败 **3 次**（仁德/耀武/机捷）。

**「注释所称差异」须与上游原文核对（2026-10-19）**：本会话已发现 **2 例**原生注释把
**不存在的差异**写作「登记差异」——`rende`（「限一次」↔ 上游无 `usable`）与 `yaowu`
（「上游由 AI/玩家确认」↔ 上游同为 `forced: true`）。→ 引用「登记差异」前**必须**回到上游
逐字核对；**注释滞后**按文档问题处理（修复清单第 25/26 项），**不当作规则差异**。

**夹具数值纪律（2026-10-19，样本 56 结姻）**：体力类前置**必须相对 `max_hp` 计算**
（如 `debug_set_hp(p, max_hp - 1)`），**不要写死数值**——装配武将可能改变 `max_hp`，
写死会导致「以为受伤、实为满血」→ 技能不可发动（结姻 `Rejected(InvalidChoice)` 即此因）。
同理：**判定牌须在摸牌阶段之后置顶**（铁骑）、**引擎初态即判定阶段**（天妒/观星）。

**夹具/助手注意（2026-10-19）**：`nc08_common::advance_to_phase` 等共享助手**每步都会 drain**
（对可取消询问取「放弃」）→ 若用例需要**观察或应答特定询问**（如 `tuxi_target`），必须改用
**不 drain** 的推进方式，否则目标询问会被默认应答掉（突袭样本初版即因此失败）。

**差异分级（2026-10-19 修订）**：差异按**影响面**分级，修复优先级依次为
**规则损失 → 原生更宽松 → 机制差异 → 维护性风险 → AI/表现面**。示例：`guanxing_fail`
（观星未移动牌时的标记）经核对**只被 AI 决策消费**（`card/standard.js` 的无懈判断），
**不改变规则判定** → 记为 **AI 面差异**，**不计入规则损失**。

**证据强度分级（2026-10-19）**：NC08 第⑦层样本按**对照依据**分两级，**不得混记**：

| 强度 | 对照依据 | 来源数 | 记账口径 |
|---|---|---:|---|
| **源码级** | 上游 JS **明文可读**，逐字引用语义 | **22** | 可计入「上游规则等价」通过数 |
| **弱对照** | 上游**混淆**（`jsjiami`）→ 只能引用**原生注释所载口径** + 包映射 | **1**（`longxue`） | **不计入**等价通过数（只证明「实现与自述口径一致」） |

**上游混淆来源全库扫描**（`grep -rl jsjiami archive/upstream/` → 6 文件）：**受影响来源恰 2 个**——
`longxue`（龙血玄黄）与 **L12 如真似幻**（后者未建样本，**预期同为弱对照**）；其余 4 个命中在核心
（疑为混淆检测器）。**扩展目录内命中为 0** → 其余来源源码可读。
`longxue` 的**静态提取升级已尝试并失败**（十六进制表无中文、目录内无明文）→ 升级须走
**D5 运行期行为序列采集**。

**非基准面技能登记（2026-10-19）**：以下技能**不在** `character/standard/skill.js`（基准面），
仅见于其它面（`sb` 等）→ **不为它们建第⑦层样本**，其行为差异另按对应面处理：

| 技能 | 出现面 | 说明 |
|---|---|---|
| `jushou` 据守 | `character/sb/` | 标准面无此技能；规则基准已注明「原生新增曹仁『据守』为第 34 将」 |
| `guidao` 鬼道 | `character/sb/` | 标准面无此技能（原生内核有 `GUI_DAO` 黑色改判实现） |

**已撤回的猜测（方法学记录，2026-10-19）**：①「该路径下弃牌上限未强制执行」——探针实测
`Play: hand=15 → Discard: hand=4`，**不成立**；②「红色锦囊不可转化」——专门构造红色【无懈可击】
复测为 Accepted，**不成立**；③「奇袭黑色载体转化未生效 / 状态依赖」——打印 card id/suit 后查明是
`debug_place(player, name)` **按名字查找命中不同花色实例**所致，原生行为一致且正确，**不成立**。
④「原生接受出牌阶段主动使用【无懈可击】」——改为**按 card id 取黑色实例**后实测
`Rejected(WrongPhase)`，与上游一致，**不成立**（同一 `debug_place` 实例问题）；
⑤「`PlayCard` 用手牌校验的 `turn.current` 而非 `actor` 可能有风险」——核对 `play_card.rs:12`
已有 `actor != core.turn.current` 守卫，二者等价，**不成立**。
前四条均因**夹具前提不成立**而误判，第五条为**读码不完整** →
**方法学**：登记差异前须先自证夹具前提；涉及**颜色/花色**的验证必须**按 card id 布阵**
（`debug_place` 不保证实例）。

**修复状态**：其中 **4 项已给出修法与验收**（[待实施修复清单](../operations/evidence/2026-10-18-blocked-fixes/README.md)），
因涉及**未跟踪的 host/standard/kernel 拆分文件**而**未实施**；其余为**待口径判定**。


### 本地上游对齐的新增核查

**采用玩法版本（2026-10-16 明确）**：本地 `archive/upstream/noname`（上游 v1.11.7，核对 HEAD `df2682e6`）的 standard 规则与标准包；Liuli 侧差异与扩展自带规则另见[两仓台账](../migration/upstream-extension-reconstruction.md)，旧线 `engine-core` 只作迁移对照、不是规则来源。距离/座距语义据此核对（[证据](../operations/evidence/2026-10-16-rule-source-distance-basis/README.md)）。

下表以当前本地 `archive/upstream/noname` 为规则和标准包参照；Liuli 的相应扩展与旧规则另见[两仓台账](../migration/upstream-extension-reconstruction.md)。[第 16 轮证据](../operations/evidence/2026-09-23-native-upstream-review-16/README.md)保留源码位置、清点口径和实际 API 探针。这里的数量只表示目录覆盖，不表示行为等价。

| 对齐面 | 本地上游行为或范围 | 原生线当前状态与下一步 |
|---|---|---|
| 标准武将与牌堆 | `standard/character.js` 有 33 个武将条目，`card/standard.js` 有 108 张物理牌 | `STANDARD_GENERALS` 为 34 个非样例将（**上游 33 条目全对齐**——2026-10-07 五批补齐乐进/潘凤/华雄/甘夫人/李典/公孙瓒/伊籍/袁术 + **原生新增曹仁**「据守」为第 34 个）+ 3 个样例；缺失来源将清零（2026-10-07 补齐八将语义——狂斧每回合限一次等价于上游每出牌阶段限一次（触发仅在自身出牌阶段）、体力比较为伤害提交后口径（同上游 damageSource content）、耀武来源强制二选（满血无回复选项，同上游 chooseDrawRecover 满血自动摸）、神智以判定阶段开始为准备阶段锚点（原生无准备阶段，先于一切阶段结算）、恂恂把固定摸的 2 张还回牌堆顶再观看顶四调整两置顶两置底后摸 2（净效果与上游一致）、忘隙双方各摸 1 顺序为座位号升序（上游按座位序自当前回合角色起算）、义从采用上游唯一 reyicong 定义（refresh 改版）：外向距离无条件 -1、体力值≤2 时他人算向自己 +1（computed_distance 状态修饰，马术同款；与 ±马叠加且共享下限 1）、机捷为出牌阶段限一次主动技（牌堆底一张交任意存活角色含自选——上游 chooseTarget 无过滤器）、急援登记差异仅濒死臂（赠牌/失去牌臂需 giver 跟踪面未建模）、妄尊主公资格沿激将 ADR 0776 惯例（原型模式视为真；手牌上限 -1 以回合级「妄尊印」实例在弃牌结算生效，到该角色回合结束）、同疾杀目标转移（你在目标攻击范围内时目标可弃一张牌使杀改指向你；改写 CardUsed 帧载荷、sha 结算 Intrinsic 读取，inRange 口径=目标对你的攻击范围）；华雄体力 6 使武将表体力区间断言放宽到 ≤6）。装配授予：国战明置与原型装配原语（`debug_assemble_general`）共用 `apply_general_assembly`（2026-10-07 抽取，授予语义不分叉）；`debug_set_general` 仅装槽（势力/性别资格测试依赖）。`standard_catalog()` 现为 **124 张**声明表（2026-10-11 更正：原记 80 张为 v60 时点计数，v74 起按[声明表](#标准目录构成声明表)核定并加 `standard_catalog_inventory` 守护）。**五谷、桃园、无中、借刀已于 2026-10-07 缺牌种批①补齐**（规则 v62/v63，落位与上游同：无中 ♥7/8/9/11、桃园 ♥1、五谷 ♥3/4、借刀 ♣12/13），牌表逐张对照见[上游标准牌表对照](#上游标准牌表对照)；需逐将/逐牌固定来源版本和语义后再验整包 |
| 主动技阶段 | 上游仁德限 `phaseUse` | **已修（v29）**：`ActivateSkill` 默认限出牌阶段；激将经目录 `active_outside_play` 声明例外（上游 `chooseToUse/chooseToRespond`）。回归：`activate_skill_rejected_outside_play_phase` |
| 激将 | 上游有主公资格、出杀使用/响应入口，并按座位逐一询问蜀将 | **已重构（v29）**：按座位序逐人询问蜀势力（拒绝继续、全拒绝回滚 once_per_turn、借杀同一结算、身份局主公资格）。与上游剩余差异：借牌直结算非完整 chooseToRespond 通道；非国战模式以主将势力定资格 |
| 护驾 | 上游向符合资格的魏将逐个询问，不以暗手牌预筛 | **已重构（v29）**：按座位序逐人询问魏势力（不以暗手牌预筛、身份局主公资格、代出闪计入无双需求、挂起态可恢复）。与上游剩余差异：代出闪为弃置+计数而非完整 respond 通道 |
| 仁德累计 | 上游出牌阶段可多次给牌，累计跨过 2 张时回复一次 | **已修（v29）**：多次发动 + `TurnState.rende_given` 阶段累计（入快照/哈希/字节），跨 2 回复一次。回归：`rende_cumulative_crosses_two_across_activations` |
| 模式入口 | 上游 `apps/core/mode` 有 12 个顶层模式入口；入口数不等于 12 套独立规则 | 原生 `GameMode` 有 7 类（2026-10-10 源码核对）：Prototype、Identity、Kingdoms 及模式入口批次接入的 Versus（v80）、Doudizhu（v81）、Boss（v82/v84）、Kof（v83）。各模式仅基线与所测路径，变体与未建模项见上方“当前声明与验证边界”表的模式行；其余上游入口尚无接入与逐模式验收 |

后续处理顺序和完成条件统一见[进度主稿](native-progress.md#后续待做按阻塞关系)，避免在本页保留另一套易过期待办。规则任务按本表固定来源与语义，逐技能/将/牌/模式比较命令、事件、随机流和中间恢复轨迹；逐来源扩展与组合验收见[两仓台账](../migration/upstream-extension-reconstruction.md)。

[覆盖清单](../native-coverage-manifest.md) 登记机制证据；[ADR 索引](../adr/INDEX.md) 保留历史决策。

## U07（玩点论杀）差异族（2026-10-19，由 10 个证据目录提炼）

> 来源：`docs/operations/evidence/*-u07-*`（deferral-closeout / moshou-wiring / skill-batch1–7 / lifecycle），
> 提取方式见[提炼证据](../operations/evidence/2026-10-19-u07-diff-extraction/README.md)。

| # | 差异 | 上游 | 原生 | 族 |
|---|---|---|---|---|
| 1 | **`damageBegin1` 时点** | 目标**受伤前体力**判定 | 原生时点不同 | 时机锚点（`host:wudian_ciqiu`） |
| 2 | **`chooseControl` 的 AI 默认项** | 由 `attitude` 推导 | **不迁移**（固定选项） | AI 面（非规则差异）（`nsshuaiyan`） |
| 3 | **守卫弃牌张数** | **一次性选 n 张** | **首张自选 + 余随机代抽** | 确定性首张（`zhucheng`）；**回放确定性已验证**：同 seed 完全可复现（`nc08_replay_determinism_five_sources`，4 包通过），差异仅「与上游 RNG 流不可比」；**源码级确认**：`wudian_cards.rs:156-178` 自带登记注释 + `Act` 阶段 `[Pass]+手牌` 的 `WaitChoice` ⇒ 「首张自选」成立（**行为面单点验证仍待办**）|
| 4 | **`discardPlayerCard(target, true)`** | **被弃者自选** | **确定性首张** | 确定性首张（`host:wudian_kangyin`） |

**另 3 条「差异：无」**（纯数据钩语义等价 / `filterCard` 无弃牌代价 / `frequent` 无确认）——
保留在证据中，不计入差异族。

## U01 / U03 / L04 差异族（2026-10-19，由各自证据目录提炼）

> 来源：`docs/operations/evidence/*-{u01,u03,l04}-*`（各 2 个目录；**U01 的 6 条全部出自 `2026-10-17-t12-u01-moyan`**，U03/L04 各出自 `2026-10-17-t12-{u03,l04}-eight`，另两目录均为 lifecycle、0 差异），提炼见
> [证据](../operations/evidence/2026-10-19-u01-u03-l04-diff/README.md)。

### U01（d3 / 末焱）6 条

| # | 差异 | 族 |
|---|---|---|
| 1 | 开局已在判定阶段且 `round == 1` 时不发阶段开始（末焱不补发 `roundStart` 第 1 轮） | **时机锚点** |
| 2 | 取消和未选中的牌**立即弃置**（上游留处理区） | 事件面收窄 |
| 3 | 展示牌**挂在宿主机器上**（不进桌面/处理区） | 事件面收窄 |
| 4 | 获得＝手牌推入 + `CardsGiven`（**不发** Moved/Drawn/Discarded，**非 `gain2`**） | 事件面收窄 |
| 5 | 点数与随机流无关 | 确定性 |
| 6 | 空牌库/空弃牌堆负例把多余牌挪到 `core.table`（仅为恢复归属） | 恢复面；**已提炼通用不变式**「每张牌唯一归属且恢复前后一致」（`nc08_card_ownership_partition`，d3 124 张 / wudian 138 张均通过） |

### U03 / L04（cardpile 两包）各 5 条

| # | 差异 |
|---|---|
| 1 | 追加牌名含 `_scale_huosha_`，`kind.key()` 为 `huosha` |
| 2 | 花色按 club/diamond/heart/spade 排序（只改生成顺序） |
| 3 | 点数不是 `Math.random`（确定性） |
| 4 | 未配置时不追加——113 张比例牌**不是上游默认追加** |
| 5 | `playpackconfig` **已逐项对照**：配置键/**未配置不补**/`dn` 公式/**上限 1000**/取整**全部一致**；**唯一差异＝补牌点数**（上游 `Math.random`，原生确定性，见 #3）⇒ 升级为「已对照，点数差异并入 #3」；见[口径与对照](../operations/evidence/2026-10-19-playpackconfig-upstream/README.md) |

> L04 与 U03 条目基本相同，但台账明确「不因此把两行合成一次验收」⇒ **分来源登记**。

## U08（英雄杀）差异族（2026-10-19，第 524 轮）

**来源**：`archive/upstream/noname/apps/core/extension/英雄杀/`
**原生包**：`packages/extensions/hero-kill-upstream-noname`（`cards: 0` / `generals: 0` / `skills: 5` / `deferred: 2`）

| # | 上游（读源码） | 原生（读包数据） | 性质 |
|---|---|---|---|
| **U08-1** | `character/` 下 **43 条武将**（`character.js`）引用大量外部技能（`nvquan`/`qiandu`/`yxsweiyi` 等） | **`generals: 0`** —— **武将全部未迁移**，包内 `deferred[0]` **自述**「43 条武将……依赖未齐」 | **缺口（量化）**：43 → 0 |
| **U08-2** | `yxs_fanji`（反击）：`damageEnd` + `direct` + `clearTime` + **`priority 12`**；filter 要求**手牌有【杀】**且事件牌为 `sha`/`juedou`；content = `addTempSkill("yxs_fanji2","shaAfter")` + **`chooseToUse`** 对来源出【杀】 | `skills[0] = { name: "yxs_fanji", mapping: "host:yxs_fanji" }` —— **宿主实现**（`content/host/**`，**本工作树未跟踪**）| **未逐条判定**（宿主实现未读）|
| **U08-3** | `yxs_menshen`（门神）：`phaseEnd` + **`priority 15`** + `group: yxs_menshen3` + `onremove`；content = **`chooseTarget`** ⇒ `target.storage.yxs_menshen2 = player` + **`target.addSkill("yxs_menshen2")`**（**跨角色施加技能**）| `host:yxs_menshen` + `host:yxs_menshen2` + `host:yxs_menshen3`（三个宿主键**都在**）| **未逐条判定**（同上）|
| **U08-4** | 上游技能依赖 `priority`/`clearTime`/`onremove`/`group` 等**元数据**参与调度 | 原生宿主行为的**元数据面未核对** | **未判定** |

**当前的可靠结论**：

1. **U08-1 是量化的缺口**：上游 43 条武将 ⇒ 原生 **0**；包内 `deferred` **自我登记**「依赖未齐」，
   与台账「部分重构」一致；
2. **U08-2/3 只能在「键存在」层面确认**（`host:yxs_fanji` 等 5 个键**都已映射**），
   **行为等价性未判定** —— 宿主实现在 `crates/engine-native/src/content/host/**`，**本工作树为未跟踪文件**，
   本代理**只读不改**，且本轮**未逐行读完**（`factory.rs`/`filtering.rs` 两处派发）；
3. **U08-4 未判定**。

### U08 **v2**：决定性事实 —— 技能**已实现但不可达**（同日）

用例 `nc08_u08_reachability`（`crates/engine-native/tests/`）：

```
DBG U08 可达性：声明技能 5 个、cards=0、generals=0
DBG U08 可达性：catalog 里含 5 个 U08 技能；全库武将引用 0 条
```

| 判据 | 结果 |
|---|---|
| 包声明 5 个技能 | ✅ |
| 包内 `cards` / `generals` | **0 / 0** |
| 5 个技能进入 catalog | ✅ **5/5** |
| **全库 41 包里授予这些技能的武将** | **0 条** |

⇒ **这 5 个技能永远不会出现在任何角色的技能表里** ⇒ **行为不可达** ⇒
**`priority` 与时机那两条差异「无法用样本判定」**（不是「不影响结果」）。

**因此 U08 的规则差异结论是**：

1. **实现存在** ✅（v1 已定位）；
2. **有 2 条待判定的实质差异** ⚠️（fanji 时机、priority）；
3. **但当前无载体 ⇒ 差异无法实测** ❗；
4. ⇒ **不能签署「规则差异通过」，也不能签署「差异已查清」**；
5. **解锁条件**：迁移 43 条武将（或给这些技能加 `cards[].skills` 赠予通道）。

### U08 差异族 **v1**：已定位实现，且**发现 2 条实质差异**（同日）

**实现位置已定位**（`crates/engine-native/src/content/host/**`）：

| 面 | 位置 |
|---|---|
| 键映射 | `keys.rs:836-840` |
| 触发元数据 | **`metadata/common.rs:170/188`（`TriggerMeta`）** |
| filter | `filtering/common.rs:101/119` |
| **步骤机** | **`step/skills_damage.rs:7`（反击）、`step/skills_targets.rs:7`（门神）** |
| ABI tag | `tags.rs:290/292` ⇒ **526（反击）/ 528（门神）** |

#### v1 差异表

| # | 上游 | 原生 `TriggerMeta` | 判定 |
|---|---|---|---|
| **U08-2a** | `yxs_fanji`：**`damageEnd`** | **`EventKind::Damage`** | ⚠️ **实质差异**：**时机不同**（伤害**结束**后 vs 伤害**发生时**）|
| **U08-2b** | `yxs_fanji`：**`priority 12`** | **`priority: 0`** | ⚠️ **实质差异**：多技能同事件时的**排序**可能不同 |
| **U08-3a** | `yxs_menshen`：`phaseEnd` | **`PhaseEnd`** | ✅ **一致** |
| **U08-3b** | `yxs_menshen`：**`priority 15`** | **`priority: 0`** | ⚠️ **实质差异**：排序 |
| **U08-3c** | `yxs_menshen`：`optional`（可选发动）| **`optional: true`** | ✅ **一致** |
| **U08-4** | `yxs_menshen3`：上游以 **`group`** 挂在门神上（任意阶段开始/死亡时清除）| **`EventKind::PhaseBegin` + `extra_events: [Death]`** | ✅ **形式一致** |
| **U08-5** | `yxs_menshen2`：上游 `addSkill` 施加的**被动标记** | **`None`**（注释：标记持有，被动无触发时机）| ✅ **形式一致** |
| **U08-6** | `yxs_fanji2`：上游 `addTempSkill(..., "shaAfter")` ⇒ **杀结算后**时机 | **`None`**（注释：红杀直伤在杀机器内联）| ⚠️ **未判定**（内联实现未读）|

#### v1 结论（**如实**）

1. **U08 的 5 个技能都有实现**（触发元数据 + filter + 步骤机 + ABI tag 齐全）✅ ——
   **推翻**了「只有键映射、没有实现」的担心；
2. **发现 2 类实质差异**：**`yxs_fanji` 的时机（`Damage` vs `damageEnd`）** 与 **两者的 `priority` 都是 0**
   （上游 12 / 15）⚠️ —— 后者在**同事件有多个技能**时可能改变结算顺序；
3. **形式一致 3 条**（`menshen` 事件、`menshen3` 的 `PhaseBegin+Death`、`menshen2` 的被动）✅；
4. **仍未判定 2 条**：`yxs_fanji2` 的内联实现、以及**行为等价**（需要样本）。

⇒ **U08 记为「差异族 v1」**：**有实质内容（2 条 ⚠️ + 3 条 ✅ + 2 条 ❓）**，
但**仍不足以签署「规则差异通过」** ❌ —— 签署需要：①`priority` 差异是否真的影响结果（**写样本**）
②`Damage` vs `damageEnd` 的语义差（**写样本**）。

### U08 复核：**一条被否定的假设**（同日）

我一度以为 `factory.rs:327-331`（`HostBehavior::YxsFanji => (None, None)` 等五行）意味着
「**这 5 个技能没有实现**」。读函数签名后**否定**该判断：

```rust
pub(super) fn machine(&self, owner: PlayerId, event: &EventData) -> HostMachine
// 返回的是 (target, card) —— 事件的**上下文**，不是 trigger/effect
```

⇒ `(None, None)` 只表示「**该事件不提供额外的 target/card 上下文**」，**不表示技能未实现** ❌。

**已确认的部分**：

| 面 | 位置 | 状态 |
|---|---|---|
| **键映射** | `content/host/keys.rs:836-840` | ✅ 5 个键都在 |
| **filter** | `content/host/filtering/common.rs:101+`（`YxsFanji` 有 `EventData::Damage(d) if d.target == owner` + `has_sha` 判定）| ✅ **有实际条件** |
| **触发/效果** | **未定位** | ❓ |
| **元数据面**（priority 等）| —— | ❓ |

⇒ **U08-2/3 仍为「未判定」** ✅（**只是范围缩小到「trigger/effect 在哪」**）。

**因此 U08 记为「差异族 v0（部分登记）」**，**不记为已签约的规则差异项**；
要完成需：①读宿主实现 ②对 `yxs_fanji`/`yxs_menshen` 各写一个**行为样本** ③判定 `priority` 一类元数据是否影响结果。

## U05-OW（杀海拾遗 / ow 子包）差异族 **v0**（2026-10-19，第 530 轮）

**来源**：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/ow/skill.js`
**原生包**：`packages/extensions/shaai-ow-upstream-noname`（**24 武将 / 11 技能，全部 `host:` 映射**）
**宿主行为**：**复用 `host:wudian_*`**（`guangshu*`/`ziyu`/`shouhu`/`duwen`/`paotai*`/`baolei_switch`）

### trigger 对照（上游 `trigger:` vs 原生 `TriggerMeta.event`）

| 技能 | 上游 trigger | 原生 `TriggerMeta` | 判定 |
|---|---|---|---|
| `guangshu` | **（无）** —— 被动 | **`EventKind::Damage`** | ⚠️ 上游无触发时机，原生有 |
| `ziyu` | **`global: phaseEnd`** | **`PhaseBegin`** | ⚠️ **时机 + 作用域都不同**（结束/全局 vs 开始/自己）|
| `shouhu` | **（无）** | **`PhaseBegin`** | ⚠️ |
| `duwen` | **`source: damageBegin`** | **`PhaseBegin`** | ⚠️ **差异最明显**（伤害开始 vs 阶段开始）|
| `paotai` | **（无）** | **`PhaseBegin`** | ⚠️ |
| `bshaowei` | **`player: phaseEnd`** | **`CardUsed`** | ⚠️ **差异明显**（阶段结束 vs 卡牌使用）|

⇒ **6/6 的 `TriggerMeta.event` 与上游 `trigger` 不一致** ❗

### 但**不能直接判「语义不同」**（重要 caveat）

`TriggerMeta` 只决定**宿主何时被唤醒**；真正的语义在 **`step` 步骤机**（`step/skills_*.rs`）里，
唤醒时机可以**故意不同**（例如用 `PhaseBegin` 作为「每回合检查一次」的**轮询入口**）。
⇒ 必须**读对应 step 机器**才能判定等价性。

### 本轮结论（**如实**）

1. **6 条 trigger 全部不一致** —— 这是**已登记的事实**，不是「差异已判定」；
2. **判定等价需要**：①读 `step` 步骤机 ②写**行为样本**（该包有 **24 个武将**，**载体存在** ⇒ **可写样本** ✅）；
3. 因此 **U05-OW 记为「差异族 v0」**，**不签署规则差异**。

### U05-OW **v1**：两条硬结论（读 `step` 机器与源码注释所得，同日）

读 `crates/engine-native/src/content/host/step/wudian_status.rs` 后，**6 条「trigger 不一致」中有 2 条得到解释**，
且**代码注释本身就是差异登记**（本仓库用「规则 v122/v124」这类版本号标注改动）：

#### `duwen`（毒吻）—— 时机**可能对齐**，实现在别处

```rust
// 毒吻（ow/黑白合，规则 v122）：锁定——造成伤害时若来源与受伤害角色手牌数相等，伤害 +1。
// 上游 duwen damageBegin；在 commit_damage（kernel.rs 167 行），宿主无机
HostBehavior::WudianDuwen => { … Flow::Done }
```

⇒ **真正的实现在 `commit_damage`（`kernel.rs:167`）**，**不是** `TriggerMeta` 也不是 step 机器；
**`PhaseBegin` 的 `TriggerMeta` 只是轮询入口**（step 立即 `Done`）—— 这**证实了 v0 的 caveat** ✅。

**判定**：上游 `source: damageBegin` ⇒ 原生 **`commit_damage`（伤害提交时）** ⇒ **时机语义接近** ✅，
但**仍需样本**验证「手牌数相等 ⇒ 伤害 +1」的条件与时机。

#### `bshaowei`（堡垒/侦察切换）—— **有意合并**，已登记

```rust
// 堡垒/侦察切换（U05 ow，规则 v124）：三态翻转——0→1（哨卫）、1→2（侦察）、2→1（回哨卫；
// 上游双技各 phaseEnd 确认切换的互斥语义合并为一个确认式翻转）。
```

⇒ **上游是「两个技能各自在 `phaseEnd` 确认切换」**，原生**合并为一个确认式三态翻转** ⇒
**这是有意为之的差异**（**注释自述**），**不是缺陷** ✅ —— 但**属「规则差异」登记项**（需在基准里保留）。

#### 其余 4 条

`guangshu`/`ziyu`/`shouhu`/`paotai` **未读 step 机器** ⇒ **仍为「未判定」** ❗。

### v1 结论

| 条 | 状态 |
|---|---|
| `duwen` | **时机可能对齐**（实现在 `commit_damage`）✅ —— **待样本** |
| `bshaowei` | **有意合并**（上游双技 ⇒ 原生单确认式翻转）✅ —— **已登记差异** |
| 其余 4 条 | **未判定** ❗ |

⇒ **U05-OW 仍不能签署规则差异**，但**从「6 条全不一致」变成「2 条有解释 + 4 条未判」**。

### U05-OW **v2**：6 条全部有原生语义描述（同日，收尾）

补读 `step/wudian_cards.rs` 与 `step/wudian_recovery.rs`，**4 条未判定项都有原生的语义注释**：

| 技能 | 原生注释（摘录） | 上游 | 判定 |
|---|---|---|---|
| `guangshu` | 「**出牌阶段**弃 he 一张并指定一名角色，按弃牌花色授对应子技（♥受伤回 1 / ♦伤害后摸 2 / ♣至下阶段结束不能用杀 / ♠下阶段结束受 1 无源雷伤；各一次性移除；已有任意子技者不可指定）」**规则 v119** | **无 `trigger`**（被动/主动式）| ⚠️ 原生以 `Act` 阶段驱动 ⇒ **属「主动技」口径差异**，**待样本** |
| `ziyu` | 「**摸一张或回复 1 点体力（可放弃；满血时无回复选项——上游 `chooseDrawRecover` 同口径）**」**规则 v120** | **`global: phaseEnd`** | ⚠️ **作用域 + 时机双差异**（上游**全局** `phaseEnd`，原生**只对 owner** 的 `PhaseBegin`）|
| `shouhu` | 「**Act 弃一张杀 → 选其他受伤角色 → 回复 1 点体力（上游 `target.recover` 同口径）**」**规则 v121** | **无 `trigger`** | ⚠️ 同上（**主动技口径**），**待样本** |
| `duwen` | 「造成伤害时若来源与受伤害角色手牌数相等，伤害 +1（**上游 `duwen` damageBegin；在 commit_damage**）」**规则 v122** | `source: damageBegin` | ✅ **时机接近** |
| `paotai` | 「**Act 弃一张杀 → 布置/升级（≤3；层数跨回合持久入 PlayerState）**」**规则 v123** | **无 `trigger`** | ⚠️ **主动技口径**，**待样本** |
| `bshaowei` | 「三态翻转……**上游双技各 `phaseEnd` 确认切换的互斥语义合并为一个确认式翻转**」**规则 v124** | **双技各 `phaseEnd`** | ✅ **有意合并** |

### v2 结论

1. **6/6 都有原生语义描述**（作者在注释里写了「上游是什么 / 原生是什么 / 规则 vNNN」）✅；
2. **2 条作者自述对齐**（`duwen` 时机接近、`bshaowei` 有意合并）✅；
3. **4 条有实质口径差异**（`guangshu`/`shouhu`/`paotai` 由「被动/触发」变「`Act` 主动技」；`ziyu` 由「全局 `phaseEnd`」变「对自己 `PhaseBegin`」）⚠️；
4. ⇒ **U05-OW 差异族 v2 完成**（**6 条全部有对照**），但**仍不签署「规则差异通过」** ——
   需要**行为样本**来证明 ②③ 两种口径在实际对局中**等价或差异可接受**；
5. **该包有 24 个武将 ⇒ 载体存在 ⇒ 样本可写** ✅（**这是它比 U08 强的唯一一点**）。

## U05-XJ（杀海拾遗 / xianjian 子包）差异族 **v0**（2026-10-19，第 533 轮）

**来源**：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/xianjian/skill.js`
**原生包**：`packages/extensions/shaai-xianjian-upstream-noname`（**36 武将 / 22 技能**，全部 `host:xj_*`）
**宿主实现**：`content/host/metadata/xianjian.rs`（**该组共 73 个 `HostBehavior::Xianjian*` 行为**）

### 22 个声明技能的 trigger 对照

| 技能 | 上游 `trigger` | 原生 `EventKind` | 判定 |
|---|---|---|---|
| `tianshe` | `damageBefore` | **`DamageBefore`** | ✅ 一致 |
| `ya` / `song` / `xuanyan2` / `txianqu` | `damageBegin` | **`DamageBefore`** | ✅ 一致 |
| `lingyu` | `phaseEnd` | **`PhaseEnd`** | ✅ 一致 |
| `feixu` | `useCard`/`respond` | **`CardUsed`** | ✅ 基本一致 |
| `guiyuan` / `tannang` / `xuanyan` / `diewu` | **（无）** 被动 | **`None`** | ✅ 一致 |
| `tianshe2` | `damageAfter` | **`Damage`** | ⚠️ 待判（`Damage` 是否含 after）|
| `xuanyan3` / `ningbin` / `anwugu` | `damageEnd` | **`Damage`** | ⚠️ 待判 |
| `lingxue` | `recoverEnd` | **`Recover`** | ⚠️ 待判 |
| `huimeng` | `recoverAfter` | **`Recover`** | ⚠️ 待判 |
| `fanling` | `loseHpAfter` | **`LoseHp`** | ⚠️ 待判 |
| `feng` | `drawBegin` | **`CardsDrawn`** | ⚠️ 待判 |
| `xjyufeng` | `loseEnd` | **`CardsDiscarded`** | ⚠️ **语义可能不同**（失去牌 vs 弃牌）|
| `anwugu2` | `phaseAfter` | **`None`** | ⚠️ **上游有触发，原生无** |

**小计**：**7 条一致** ✅ / **11 条待判** ⚠️ / **4 条被动一致** ✅。

### 关键 caveat：`EventKind` 的语义粒度**未确认**

`damageBefore` / `Damage` / `DamageEnd` 在引擎里是**不同变体**（`DamageBefore` 已出现 ✓），
所以「`damageEnd` ⇒ `Damage`」**可能只是命名映射**，也可能**真的少了一档** ❗
⇒ 必须**读 `EventKind` 定义**（或写样本）才能判定，**不能由名字直接下结论**。

### v0 结论

1. **22 条全部有对照** ✅（**比 U05-OW 的 v0 更完整**）；
2. **11 条待判**，其中 **`xjyufeng`（loseEnd vs CardsDiscarded）** 与 **`anwugu2`（phaseAfter vs None）**
   是**最可疑的两条**（前者语义不同、后者少了触发）；
3. **不签署规则差异** —— 需先读 `EventKind` 语义 + 写样本。

### U05-XJ **v1**：`EventKind` 变体表读出后的三点结论（同日）

`crates/engine-native/src/event.rs`（**tracked**）的 `EventKind` **共 22 个变体**：

```
DamageBefore Damage Recover CardsDrawn CardsMoved CardUsed CardResponded Dying Death
PhaseBegin PhaseEnd TurnEnded SkillToggled Equipped Linked CardsDiscarded TurnOverBefore
TurnOvered LoseHp DyingSaved CardsLost ShaDodged
```

#### 结论 1：**「结束 / 之后」类时机在引擎里没有独立变体**（共性限制）

`DamageBefore` / `Damage` **只有两档**；`Recover`、`LoseHp`、`CardsDrawn` **各只有一档**，**不分 before/after**。
⇒ v0 里 8 条「待判」（`damageEnd`×3、`damageAfter`、`recoverEnd`、`recoverAfter`、`loseHpAfter`、`drawBegin`）
**不是映射错误，而是引擎粒度不足** ❗ —— 属**引擎层共性限制**，需在**基准里统一登记**，而非逐包重复。

#### 结论 2：**`xjyufeng` 的映射可疑**（⚠️ 可能是真缺陷）

上游 `xjyufeng` 是 **`loseEnd`（失去牌之后）**；原生挂在 **`CardsDiscarded`（弃牌）**。
但引擎里**存在 `CardsLost` 变体** ⇒ **「失去牌」应有独立事件** ⇒ 原生**可能挂错事件** ⚠️。
**判定需要**：读 `CardsLost` 的触发时机 + 写样本（**弃装备/被拿走** vs **主动弃牌**）。

#### 结论 3：**`anwugu2` 可能少挂**

上游 `phaseAfter`（**阶段之后**）；原生 `TriggerMeta = None`（注释自述「伤害前置窗口」）。
引擎里有 **`TurnEnded`**（回合结束）但**没有「phaseAfter」** ⇒
**要么该用 `PhaseEnd`，要么它确实由别的窗口驱动** ⚠️ —— **需读 step/filter 才能定**。

### v1 小结

| 类 | 条数 | 性质 |
|---|---:|---|
| ✅ 一致 | 7 | 触发时机同名同档 |
| ✅ 被动一致 | 4 | 上游无 trigger ⇒ `None` |
| **⚠️ 引擎粒度不足**（非缺陷）| **8** | `damageEnd`/`recoverEnd`/`recoverAfter`/`loseHpAfter`/`drawBegin`/`damageAfter` 类 |
| **⚠️ 可疑映射**（待样本）| **2** | `xjyufeng`（`loseEnd`⇒`CardsDiscarded`，而 `CardsLost` 存在）、`anwugu2`（`phaseAfter`⇒`None`）|
| | **21** | （`lingxue` 归入粒度不足；**`guiyuan`/`tannang`/`xuanyan`/`diewu` 计 4 条被动**）|

⇒ **U05-XJ 仍不能签署**，但**22 条全部有了明确归类**（**7 ✅ / 4 ✅ / 8 ⚠️粒度 / 2 ⚠️可疑**）。

## 引擎层共性限制：**事件时机的表达力边界**（2026-10-19，第 536 轮）

> 这一节是**跨来源**的：任何包的「before/after 类时机差异」都应先对照本节，**不要逐包重复登记**。

### 1 `EventKind` 只有 22 个变体（`crates/engine-native/src/event.rs`）

```
DamageBefore Damage Recover CardsDrawn CardsMoved CardUsed CardResponded Dying Death
PhaseBegin PhaseEnd TurnEnded SkillToggled Equipped Linked CardsDiscarded TurnOverBefore
TurnOvered LoseHp DyingSaved CardsLost ShaDodged
```

### 2 因此**上游的「结束/之后」粒度无法一一对应**

| 上游时机 | 引擎有无对应 | 说明 |
|---|---|---|
| `damageBegin` | ✅ `DamageBefore` | |
| `damageEnd` / `damageAfter` | ⚠️ **只有 `Damage`（一档）** | **「结束后」无独立变体** |
| `recoverEnd` / `recoverAfter` | ⚠️ **只有 `Recover`** | 同上 |
| `loseHpAfter` | ⚠️ **只有 `LoseHp`** | 同上 |
| `drawBegin` / `drawEnd` | ⚠️ **只有 `CardsDrawn`** | 同上 |
| `phaseAfter` | ⚠️ **只有 `PhaseEnd` / `TurnEnded`** | 「阶段之后」无独立变体 |

⇒ **这类差异不是「映射错」，是「引擎根本没这一档」** —— 属**表达力边界**，
处理方式：**要么在基准统一登记，要么升级 `EventKind`**（后者需改未跟踪的 `kernel/events.rs`）。

### 3 「失去牌」族的**既定口径**（已由源码自述）

`crates/engine-native/src/kernel/events.rs:478-481`：

> 「……玩家失去手牌后（上游 `loseAfter`+`loseAsync` 族的 **`CardsDiscarded` 面**；
> `gain`/`addJudge`/`equip` 等**其余卡牌移动面登记差异**）」

⇒ **项目把上游「失去牌」族统一挂到 `CardsDiscarded`**，**其余移动面登记为差异**。
**因此 U05-XJ 的 `xjyufeng`（上游 `loseEnd` ⇒ 原生 `CardsDiscarded`）是「遵循既定口径」，不是挂错** ✅。
（引擎虽有 `CardsLost` 变体，但它承载的是**「失去牌的来源区」载荷**，不是 `loseEnd` 触发点。）

### U05-XJ **v2**：22 条**全部有归类，0 条未判**（同日，收尾）

最后一条 `anwugu2` 定案 —— `step/xianjian_status.rs:177-181` 源码自述：

```rust
HostBehavior::XianjianAnwugu2 => match self.stage {
    // 蛊舞·缚（无机器——三钩直读：PlayCard 出牌上限/弃牌阶段 limit/自己阶段结束递减）。
    HostStage::Act => Flow::Done,
    _ => Flow::Done,
},
```

⇒ **`TriggerMeta = None` 是故意的** ✅ —— 它不靠触发，而是**三个钩子直读**
（**出牌上限 / 弃牌阶段 limit / 自己阶段结束递减**）；上游 `phaseAfter` 的语义由**「自己阶段结束递减」**钩子承担。

#### v2 最终归类（22 条）

| 类 | 条数 | 依据 |
|---|---:|---|
| ✅ **触发一致** | 7 | 上游与原生**同名同档** |
| ✅ **被动一致** | 4 | 上游无 `trigger` ⇒ 原生 `None` |
| ⚠️ **引擎粒度不足** | 8 | [共性限制节](#引擎层共性限制事件时机的表达力边界2026-10-19第-536-轮)：引擎**无「结束后」档** |
| ✅ **既定口径** | 1 | `xjyufeng`：`loseEnd` ⇒ `CardsDiscarded`（`events.rs:478-481` 自述）|
| ✅ **三钩直读** | 1 | `anwugu2`：`None` + 出牌上限/弃牌 limit/阶段结束递减 |
| …… | **21** | 另 `lingxue` 已计入「粒度不足」的 8 条内 |

⇒ **22/22 有归类，0 条「未判定」** ✅

#### 但**仍不能直接签署**（需一次产品决策）

**7 + 4 + 1 + 1 = 13 条无问题** ✅；
**8 条「引擎粒度不足」是已知限制** ⚠️ —— **签署「规则差异通过」等价于「接受这 8 条粒度差异」**，
这属**产品决策**（**不是文档或测试能单方面决定的事**）。

⇒ **U05-XJ 差异族 v2 完成**；**签署状态 = 待决策**（**决策点：8 条「结束后」时机差异是否可接受**）。

## U05-SWD（杀海拾遗 / swd 子包）差异族 **v0**（2026-10-19，第 546 轮）

**来源**：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js`（**注意是 `character/` 不是 `card/`**）
**原生包**：`packages/extensions/shahai-swd-upstream-noname`（**90 卡牌 / 79 武将 / 16 技能**）
**宿主**：`HostBehavior::Swd*` / 复用 `Xuanyuan`/`Jifeng`/`Moyan`/`Miedao`/`Xiuluo` 等

### 13 个技能可对照（另 3 个上游未在本文件定义）

| 包技能 | 上游 `trigger` | 原生 `EventKind` | 判定 |
|---|---|---|---|
| `swd_zhenjiu2` | `phaseBegin` | `PhaseBegin` | ✅ |
| `swd_lmazui2` | `damageBegin` | `DamageBefore` | ✅ |
| `swd_huajin2` | `damageBegin` | `DamageBefore` | ✅ |
| `swd_jinlin2` | `phaseBegin` | `PhaseBegin` | ✅ |
| `swd_guiyan2` | `dying` | `Dying` | ✅ |
| `bingfeng2` | `turnOverAfter` | **`TurnOvered`** | ✅ |
| `xuanyuan` | `phaseEnd` | `PhaseEnd` | ✅ |
| `swd_huanxing_state` | `damageAfter` | `Damage` | ⚠️ **粒度**（引擎无 after 档）|
| `hxunzhi2` | **`phaseUseEnd`** | **`PhaseEnd`** | ⚠️ **阶段不同**（出牌阶段结束 vs 阶段结束）|
| `jifeng` | **`useCardToBefore`** | **`CardUsed`** | ⚠️ **缺「指定目标前」这一档** |
| `swd_xiuluo` | **`phaseZhunbeiBegin`** | **`PhaseBegin`** | ⚠️ **准备阶段 vs 阶段开始**（可能不同）|
| `moyan` | **（无 trigger）** | **`CardUsed`** | ⚠️ 上游无 ⇒ 原生有 |
| `miedao` | **（无 trigger）** | **`PhaseBegin`** | ⚠️ 同上 |

**未找到（3）**：`tianxiang2`、`xuanyuanjian2`、`xuanyuanjian3` —— 上游**不在本文件**（\(\`xuanyuanjian*\`\) 在两个文件里都出现过一次，需再定位）。

### ⚠ **v0 更正**：原生列的提取脚本有误，至少 1 行是错的（同日）

我的提取正则 `HostBehavior::X\b[^\n]*\n((?:\s{12,}[^\n]*\n){0,9})` 会**越过分支边界**，
把**下一个分支的字段**当成当前分支的 —— 逐条复核后**已确认一处错误**：

| 行为 | v0 记录 | **复核实测** | 结论 |
|---|---|---|---|
| **`Hxunzhi2`** | **`PhaseEnd`** ❌ | **`None`** ✅（注释：「魂殉志 / 殉：**无触发时机（主动面 + 内联臂）**」）| **v0 错** |
| `Jifeng` | `CardUsed` | **`CardUsed`** ✅（注释：「疾风：自己使用多目标单目标牌（结算窗；锁定）」）| 一致 |
| `Xiuluo` | `PhaseBegin` | **`PhaseBegin`** ✅ | 一致 |
| `SwdZhenjiu2` / `SwdHuanxing2` | `PhaseBegin` / `Damage` | **不在 `metadata/common.rs`** ❗（在别的 metadata 文件）| **未复核** |

⇒ **U05-SWD 的「7 一致 / 6 需判定」中，`Hxunzhi2` 那条作废**；
**其余 5 条「需判定」与 7 条「一致」的数字，在逐条复核前都应视为「未复核」** ❗。

### 正确做法（下一步）

按 `HostBehavior::X =>` 的**分支起始位置切片**（`m.start()` 到**下一个** `HostBehavior::`），
而不是用固定缩进宽度 —— 这也说明**「用正则读结构化代码」本身就是脆弱手段**。

### 小结

| 类 | 条数 |
|---|---:|
| ✅ 一致 | **7** |
| ⚠️ 需判定（粒度/阶段/缺档/上游无）| **6** |
| ❓ 未定位上游 | **3** |

⇒ **`hxunzhi2`（`phaseUseEnd`⇒`PhaseEnd`）与 `jifeng`（`useCardToBefore`⇒`CardUsed`）是最可疑的两条** ⚠️
（前者是**不同阶段**，后者**缺「指定目标前」档**）；**不签署规则差异**。
