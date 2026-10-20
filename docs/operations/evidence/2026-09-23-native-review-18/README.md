# 2026-09-23 第 18 轮：G1 旧包转换（cardpile 双来源）与规则版本 30

本轮为 **G1 实现批次**：两仓 cardpile 扩展转换为 `discover_extensions` 可发现的真实包并实际导入，
打通**发现 → 安装 → 激活 → 内容锁 → 桥接建局 → 扩展牌规则结算 → 存档/恢复**的纵向样本。
前置：engine-native 新增【酒】/【雷杀】牌种及规则（`STANDARD_RULESET_VERSION` **29 → 30**，
14 golden 因牌库加入酒×2/雷杀×2 与新状态字段重录）。

| 验证 | 结果及边界 |
|---|---|
| [engine-native 全量测试](native-tests.log) | **277 passed / 0 failed**（第 17 轮 266 + 酒/雷杀规则 6 + G1 纵向 3 + 牌堆桥接单元 2）。全工作区 4,843 项通过。测试全绿不代表与上游规则等价。 |
| [第 17 轮探针在 v30 下重跑](probes-v30.log) | 21 项 FIXED 维持 + 1 项 OPEN（单选候选枚举，P2）——G1 改动未破坏既有验收。 |

## 转换产物（仓库内真实文件，可复现）

- `packages/extensions/cardpile-upstream-noname/`（manifest.json + package.npack.json + assets/pile.json）
- `packages/extensions/cardpile-upstream-liuli/`（同构；独立来源身份 namespace `upstream-liuli`）
- 数据源逐字节核对自上游工作区：noname `5c02a745` 的 `apps/core/extension/cardpile/extension.js`、
  noname-liuli `8485dbb2` 的 `noname/extension/cardpile/extension.js`；两仓 data 表语义一致
  （160 张目标、12 条目），各自保留独立包身份。
- 生成器：`crates/content-runtime/examples/gen_cardpile_packages.rs`（可复跑）；
  [G1 测试](../../../crates/engine-native/tests/g1_cardpile.rs) 断言仓库产物与生成器输出逐字节一致（防漂移）。

## 纵向样本验收点（g1_cardpile_vertical_install_enable_newgame_saverestore）

1. **发现**：`discover_extensions` 对 `packages/extensions/` 返回两仓 cardpile 目录——
   旧结论「37 个旧目录直接发现 0 包」由**转换产物**关闭（原始旧目录未改，仍不可直接导入）。
2. **安装+激活**：`import_extensions` 安装 2 包、0 跳过，content lock 含 2 包。
3. **内容锁**：启用扩展参与目录版本派生（集合指纹）。
4. **桥接建局**：已安装 pile.json → `PileSpec` → `extension_pack_from_pile` → 113 张扩展牌
   （酒×5/雷杀×9/火杀×5 及标准牌种）合并入目录；`Engine::new_with_extensions` 建局成功。
5. **规则结算**：扩展【酒】→ 扩展【杀】造成 2 点伤害（扩展牌走完整引擎规则通道）。
6. **存档/恢复**：同扩展集合恢复同哈希；纯标准目录与另一来源集合恢复均被
   `RuleVersionMismatch` 拒绝（双来源内容锁隔离）。

## 规则前置（v30，engine-native）

- `CardKind::Jiu`：出牌阶段对自身使用（每回合限一次，拒绝 `CardLimit`）——本回合使用的
  下一张杀伤害 +1（效果于杀派发时捕获进 `ShaResolveMachine.jiu_bonus` 并消耗）；濒死时
  对自身使用回复 1 点体力（并入濒死救援选项与恢复派生）。
- `CardKind::LeiSha`：按杀结算（共享出牌阶段次数限制），伤害为雷属性
  （`DamageNature::Thunder`，铁索传导按属性走既有通道）。
- 基础测试目录加入 酒×2/雷杀×2（目录 80→84 张）；身份局纯命令终局用例的种子→阵营
  映射随牌库变化实跑复核更新（seed 5 → 主公方胜；seed 1 → 反贼胜，保持双阵营覆盖）。

## 边界（如实登记）

- cardpile 的**倍率缩放**（上游按 `cardpile_*_playpackconfig` 与 160 总量补足其余条目）未实现——
  原生映射为比例表逐单元 1:1 实体牌（113 张），不宣称与上游牌堆逐张等价。
- 上游运行时随机点数（`Math.random`）未映射——原生点数为确定性流水号，随机性由建局洗牌的
  规则随机流承接。
- 扩展机器/规则宿主（新技能语义执行、扩展机器续行编码）仍待做——cardpile 为数据型扩展，
  不含机器载荷；G1 其余样本（欢乐卡牌、3D精选、wuxing）未开始。
- 原始旧目录（archive/upstream）本身未改动、仍不可被 discover_extensions 直接导入；转换是
  生成新包产物。
