# 2026-09-23 第 16 轮：当前开发进度与本地上游对齐核验

本轮只更新文档和独立探针，没有修改引擎或工程测试。[范围摘要](scope-audit.json)显示原生关键源码 SHA-256 与[第 15 轮](../2026-09-23-native-review-15/README.md)完全相同；两仓扩展逐文件清单也与已登记基线相同。noname HEAD 为 `5c02a745eb33b61b2e61e1167e1b71ca163911f6`（工作区干净），noname-liuli HEAD 为 `8485dbb29ff16f9b258a766b1a17ef75f883aade`（50 个本地修改）。本次对照的是这两个本地工作区，不是任意最新线上版本。

| 验证 | 结果及边界 |
|---|---|
| [原生全量测试](native-tests.log) | `cargo test --offline -p engine-native`：**238 passed / 0 failed**，分组 3+6+59+1+160+2+7；doc-tests 0。测试全绿不代表与上游规则等价。 |
| [覆盖清单与 golden 守卫](doc-guard.log) | `cargo test --offline -p engine-native --test replay_scenarios`：**2 passed / 0 failed**；覆盖清单登记与 14 个 golden 场景保持有效，这 2 项已包含在上面的 238 项中。 |
| [原有反例重跑](probes.log) | 第 15 轮探针重新链接当前库并通过：无懈/护驾/离间恢复、尾端缺牌、AI 信息集/候选/模拟序号、扩展输入边界等开放项仍可复现。源码见[原探针](../2026-09-23-native-review-15/probes.rs)。 |
| [新增上游规则探针](noname-upstream-gap-probe.rs) / [输出](upstream-gap-probe.log) | 实跑确认激将/护驾询问泄露他人暗手牌有无响应牌、激将首人拒绝后终止、仁德分次给牌被拒、仁德可在判定阶段发动。探针中的刘备和蜀将使用当前武将表槽位；牌库运行时数量为 80。 |
| [旧扩展发现器探针](noname-extension-discover-probe.rs) / [输出](extension-discovery.log) | [content-runtime 构建](content-runtime-build.log)通过。实际调用 `discover_extensions` 扫描两仓扩展根目录，分别返回 **0、0**；37 个来源目录没有现有导入器识别的 `manifest.json` 或 `package.npack.json`。这仅说明**不能直接导入**，不否定转换重构方案。 |

## 新确认的上游差距

- 标准包范围：上游 `apps/core/character/standard/character.js` 有 33 条武将，原生 `STANDARD_GENERALS` 仅 25 个非样例 + 3 个样例；缺的 8 个 ID 已列在[规则基准](../../../architecture/native-rules-baseline.md#本地上游对齐的新增核查)。上游 `apps/core/card/standard.js` 的 `list` 有 108 张物理牌，原生 `standard_catalog()` 是 80 张测试牌，五谷、桃园、无中、借刀等牌种未建模。`apps/core/mode` 有 12 个顶层入口，原生 `GameMode` 仅 Prototype、Identity、Kingdoms；入口数不是规则完成率。
- 主动技时机：上游 `standard/skill.js:961` 限仁德为 `phaseUse`；原生 `engine.rs` 的 `ActivateSkill` 只查当前回合玩家。探针在 Judge 阶段成功进入 `rende_cards`，需要共享阶段门控和逐技能例外。
- 激将：上游 `standard/skill.js:1087-1178` 有主公资格、使用/响应杀入口并逐名询问蜀将；原生 `standard.rs` 预筛蜀将暗杀、刘备选一人，首人拒绝即结束。探针在刘备视图相同的两局观察到不同公开候选，也证实第二名持杀蜀将未被问及。
- 护驾：上游 `standard/skill.js:275-315` 按资格逐名询问魏将；原生 `kernel.rs` 预筛暗闪，`engine.rs` 向其他座位暴露询问对象。探针显示攻击者可从是否出现询问推断第三人的暗闪。
- 仁德累计：上游 `standard/skill.js:1013-1023` 按同阶段此前给牌累计，跨过两张时回复；原生 `standard.rs,748-750,3086-3091` 限每回合一次、按单次给牌数判断。探针中先给一张后第二次发动被 `SkillAlreadyUsed` 拒绝。
- 扩展 G1 样本：两版 `cardpile/extension.js` 都包含 `jiu`、`leisha`；原生 `catalog.rs:35-70` 没有酒/雷杀种类，`standard.rs` 也无对应牌，不能只迁牌堆比例。两个上游旧目录与当前 `content-runtime/src/extension.rs:61-82` 的包发现格式不一致，需先转换/打包真实来源，再验证开局、结算与恢复。

当前状态和按优先级排列的后续项见[进度主稿](../../../architecture/native-progress.md)，逐包计划见[两仓扩展台账](../../../migration/upstream-extension-reconstruction.md)。上述探针是独立复核，不计入 238 项工程测试；后续实现修复时应把相应反例改为正确行为的工程回归。
