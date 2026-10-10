# ADR 0407：ABI 532 兵势孙綝（逆固/戮连）

本批发布兵势孙綝（`pot_sunchen`，2792324428）及「逆固」
（`potnigu`，3220807381）、「戮连」（`potlulian`，2734921313）。旧 JavaScript
仅作为语义合同，不进入 npack 或运行时。

## 决策

- 主动选牌增加通用 `distinct_suits` 约束；逆固复用既有弃牌、座次选择和赠牌管线，
  以技能实例状态保存本回合尚未消费的伤害加值次数。
- 戮连在用牌开始时记录“实体载体来自手牌”和完整目标，结算后按当前手牌类别、
  体力及装备数量精确判定横置、摸牌和成势火焰伤害；选择屏障后重新校验目标。
- 字节码标签保持唯一；并发发现的桓阶标签碰撞已改至 473，孙綝使用 474–475。

## 验证

- Golden：逆固异花色成本、逐座次交牌/拒绝、下一次伤害 +1；戮连横置、摸牌、
  排除最低体力目标并造成火焰伤害，共 2/2 通过。
- 兵势全量 20 个测试二进制、65 项测试通过；content-compiler 473/473、
  engine-api 4/4、rule-vm 10+5、conformance 8/8 通过。
- `bingshi.npack.json` 为 ABI 532、21 名武将、49 项技能，logic hash
  `6a50afd5591e70d3a53700fd0aae621b0559fc1441f9c896f5ce8fe67a50ec23`，SHA-256
  `5460436c2e41af6f97dbcc3238335645feed5e3bd9f1443701329bdc11ae2a25`；已同步
  `extensions/bingshi/`，无 JavaScript 文件。
- 完整 `extensions/` 冷导入激活 26 个包，ContentLock
  `efec52d802273f33e6253fd9428aa5f80a363cbe81862b2224a3b6a705a3d86b`。
- 编译遵循架构计划 §37.10，统一使用工作区外 target，门禁结束后清理。
