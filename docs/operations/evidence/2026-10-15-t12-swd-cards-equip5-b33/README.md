# T12 swd 卡牌面补切批③：炼妖壶+壶境——壶中仙整将收口（2026-10-15）

## 承载

**炼妖壶 lianyaohu**（新 kind tag 143，equip5 宝物栏，club13 入包）+ 两
equipSkill + **壶境 hujing**（tag 291，壶中仙第 3 技，3/3 整将收口）：

- **收纳 shouna**（tag 289，出牌限一次）：三段机——弃 1 手牌 → 选其他有
  手牌角色 → 其**首张**手牌移入壶仓（`PlayerState.lianyaohu_store`，特
  殊区等价跨回合持久）。登记差异：上游随机取 1 张 → 原生首张。
- **炼化 lianhua**（tag 290，出牌限一次）：壶仓 ≥2 → 弃 2 张（min2 选牌
  窗）→ 牌堆序扫描首张**类别均不同**的牌入手（上游 ui.cardPile 顺序扫
  描忠实承接）。
- **壶境 hujing**：锁定 PhaseBegin(Judge 入场)——弃牌堆有炼妖壶则自动回
  收装备（同栏宝物替换+grant_equip_skills）；装备时手牌上限 +2（kernel
  手牌上限钩，getEquip("lianyaohu") 同口径）。
- 字段批：`lianyaohu_store` 仓字段——存档 v70→**v71**；规则 v283→
  **v284**；golden 14 场重录（**283+字段探针与旧 golden 14/14 逐字节一
  致**——位移唯一来源=版本字段，行为面零扰动）。
- 登记差异：壶仓归属 PlayerState（上游挂壶实体 storage，壶转移后仓不随
  移）；onLose 清 stat 由 once_per_turn 回合重置近似。

## 发牌敏感回归一处（根因新发现）

凝霜测试崩于「使用者未横置」：log 排帧显示雷杀伤害沿新横置的 p1 **铁索
传导**并自动解除横置——`place_sha_color` 的 is_sha_like 谓词在包加卡后
抓到了雷杀实体。修正=该段布阵谓词收紧为普通 Sha（链敏感段禁用属性杀）。
另：Act 直接变更立见而 emit 排队帧未续行的假象，以 resume 冲刷防御（保
留为健壮性）。

## 测试（swd_cards_equip5_b33_lianyaohu_hujing）

| 面 | 断言 |
|---|---|
| 收纳正例 | 弃成本+目标首张手牌入壶仓（p1 剩 1） |
| 收纳反例 | 本回合限一次 |
| 炼化正例 | 壶仓 2 弃尽+牌堆首张类别均不同牌入手 |
| 壶境正例 | Judge 入场自动回收装备 + 弃牌窗 min=手-(hp+2) 证明上限+2 |

测试侧勘误四处：extension_root 拼写、炼化 min2 选牌一次提交、壶仓断言、
上限断言改弃牌窗行为式（state.hand_limit 直调不含 kernel 组合层钩）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 g1_shahai_swd | **42/42** |
| engine-native 全量 | **761/0/0**（golden v284+存档 v71 往返）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- 包 54 张（余 38 行：锦囊 8/器 13/equip5 余 12/equip2 3/equip1 3/神秘
  果 2）。
- **壶中仙 3/3 整将**——部分转正将仅剩拓跋玉儿 2/3（niepan 外部包技依
  赖登记）。
- 批④ 候选：equip5 余 12 kind（多为单技能宝物，逐 kind 小批）或锦囊族。
