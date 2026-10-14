# T12 运筹武将面批 5：全量入册 + 机制缺口定案（2026-10-14）

## 任务来源

[T12 运筹武将面](../../migration/upstream-extension-reconstruction.md)批 4 续批：
剩余三将（蔡文姬/陆抗/马钧）勘察与全量入册。

## 勘察定案（三机制缺口 + 两条引用缺口）

| 项 | 上游 | 勘察结论 |
|---|---|---|
| 悲歌 beige（蔡文姬） | **不在 yunchou skill.js**——character.js 引用内置标准技 | 「内置技能引用」缺口（与护驾同源）：mapping 机制无 `builtin:*` 引用形态 |
| 谦逊 qianxun（陆抗） | 同上内置引用 | 同上 |
| 精械 xinfu_jingxie（马钧） | 外部（xinfu 包族） | 「外部包技依赖」缺口（同涅槃/浴天狱先例） |
| 归汉 ycnguihan（蔡文姬） | limited 濒死主动技（回 1 摸 2 双方） | 「濒死主动技面」缺口：原生濒死通道为 Dying 求救询问（桃/酒自救），chooseToUse(dying) 技能使用面未建模 |
| 厉焰 ycnuweiyan（陆抗） | viewAs 非基本牌→水攻 + luweiyan2 链杀（useCardAfter 询问） | 「PlayCard 技能驱动 viewAs 转化」可循国色先例（内核硬编码技能名分支）+ 链杀询问面——**可立项非缺口**，排批 6 |
| 隙焰 jieyan（周瑜·DIY） | 红牌 AOE 全体火伤（multiline 主动技） | 「出牌主动技面」缺口：原生无 enable phaseUse 命令通道 |
| 野升 yiesheng（甄姬·DIY） | 弃任意张黑手牌摸等量（selectCard 1..∞） | 同上主动技面 |
| 巧思 xinfu_qiaosi（马钧） | 掷骰 + 亮牌获得 + 多步给牌/弃置分支 | 「骰子原语 + 多步主动技」缺口 |
| 蝉缘 diychanyuan（于吉） | 死亡时来源 loseMaxHp | 「max_hp 减面 + MaxHpChanged 事件」缺口（批 4 已登记） |

## 交付

- character.json 追加蔡文姬/陆抗/马钧三将（**全部映射 null**——目录技能列表
  空、元数据完整入册）：**十名武将全量入册**。
- 测试 g1_shahai_yunchou +1：全量名册元数据断言（10 将名/血/序、bridge 语义
  口径——目录技能列表=mapped 9 条、暂缓 11 条只计数）。

## 语义勘正

- 上游技能条目实为 **20**（skill.js 17 定义 + 悲歌/谦逊/精械 3 个内置/外部
  引用）——此前"17 技"统计漏计引用条目；转正 9 + 暂缓 11。
- bridge 语义确认：mapping null 不进目录技能列表（unmapped 只计数）——
  全暂缓武将目录为空技能列表（元数据面完整、规则面待后续批）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | g1_shahai_yunchou **18/18**（本批无新行为面，反例实证不适用——元数据断言直注） |
| engine-native 全量 | **732/0/0**（golden 不变）——[日志](engine-native-tests.log) |
| 装载/包管理/服务端 | **84/0** |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- 运筹武将面：**10/10 名元数据入册、9/20 技转正、11 技暂缓**（三整将）。
- 批 6 候选：厉焰（国色先例 viewAs 转化——可立项）；机制缺口三类（出牌主动
  技面/内置技能引用/濒死主动技面）按需立项。
