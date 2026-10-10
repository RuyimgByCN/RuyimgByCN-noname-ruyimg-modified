# P2/N08 批③-b 第七把：丈八蛇矛（规则 v71，2026-10-07）

主稿 P2/N08「批③-b 武器逐把」第七把：丈八蛇矛（♠12，射程 3，两张手牌
可当【杀】使用）。上游对照 `card/standard.js`（zhangba 条目：
attackFrom −2 = 射程 3；特效 zhangba_skill）。前置：
[第六把青龙偃月刀](../2026-10-07-n08-qinglong-3b6/README.md)。

## 本批落地

- **命名武器牌种 Zhangba（编码 101）**：equip1 武器、射程 3
  （named_weapon_range 登记 3）。
- **配套主动技「丈八蛇矛」（SkillDef 61 追加）**：装备即授予
  （CardDef.equip_skills）；出牌阶段 ActivateSkill 发动：
  Stage 1 选两张手牌（SelectCards min 2）→ Stage 2 选射程内其他角色 →
  次张弃为转化素材、首张 `declared_kinds` 记为 Sha 并以
  `card_use_as(declared Sha)` 发射 CardUsed——**杀结算全流程重走**
  （八卦/仁王盾/贯石斧/麒麟弓/青龙均正常作用于该 declared-Sha）。
  **登记差异：转化次序固定为首张作杀载荷，上游由玩家交互确认。**
- HostBehavior::ZhangbaSha（编码 78 表尾追加）仅为 a0 伴随一致性登记
  （机器实现在 standard.rs ZhangbaBehavior；HostMachine 臂为
  Flow::Done 占位）。
- Sha 快照追加 `asking_hanbing` 之外的 Zhangba 变体（tag 60，独立
  MachineSnapshot::Zhangba{owner, cards, stage}）。
- 声明表 121→**122 张**；`STANDARD_RULESET_VERSION` 70→**71**；
  **14 golden 重录**。

## 测试（`zhangba_two_cards_as_sha`）

- 装备授予主动技实例 → 发动 → 选桃+闪两张 → 选射程内 1 号 →
  1 号受 1 伤、两张素材离手、declared-Sha 记录存在。

## 验收口径（分开陈述）

- **代码已实现**：丈八牌种/声明表/配套主动技（激活/选牌/选目标/
  declared-Sha 结算）+ 装备授予。
- **回归已加入**：`zhangba_two_cards_as_sha`。
- **golden 已重录**：14 场景全量（v71）。
- **全量通过**：engine-native **379/0/1**（唯一 ignore=写轮眼规模诊断
  探针）；a0 7/7（含伴随技能数 26→27、技能声明表 61 项）、inventory
  2/2、replay 2/2；content-runtime 32/0、engine-ai 29/0、loader 1/0；
  lib `-D warnings` 通过。

## 剩余

批③-b 其余两把（雌雄→方天）；模式对齐；主稿其余待做照旧。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（14 文件，相对 noname-next 根）。
