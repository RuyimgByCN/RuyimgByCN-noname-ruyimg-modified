# T12 swd deferred 消解批 102：热血 + 流离内置引用（2026-10-17）

- 日期：2026-10-17
- 范围：热血（2 条目：墨耶/夜鸦熙）+ 流离（1 条目：墨耶）——**138/184 技条目已映射**（deferred 47→44）
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js`（rexue 逐段核验）；`liuli` 不在 swd 重定义，解析到标准包 `流离`。

## 语义与接线

| 技（将） | 上游语义 | 接线 |
|---|---|---|
| 热血 rexue（墨耶 `swd_moye` / 夜鸦熙 `swd_yaxi`） | `trigger: {global: "phaseBegin"}`：任意角色阶段开始时，若其在你杀范围内且你有杀，`chooseToUse` 对其使用一张杀；用牌后摸 1 | `SwdRexue`（tag 441）=PhaseBegin 全局触发（filter：阶段玩家≠自己 ∧ 自己在杀范围内 ∧ 手牌有杀）；`machine()` 提取阶段玩家为目标；效果步 `use_virtual_card(Sha, [actor])` + `draw_cards(1)`。**登记差异**：虚拟杀（不消耗实体杀）+ 恒发动近似（上游可放弃） |
| 流离 liuli（墨耶） | 标准包「流离」（swd 未重定义，引用标准实现） | 内置引用：`builtin:流离`（`BUILTIN_SKILL_REFS` 10→11；原生已有流离实现） |

## 验证

- **定向回归** `swd_deferred_rexue_and_liuli`：墨耶技能表含内置引用「流离」；0 号持热血+手牌杀，1 号阶段开始 → 1 号受 1（虚拟杀命中 4→3）、0 号摸 1（杀+1）。3 passed / 0 failed（[targeted.log](targeted.log)，含 b100/b101 回归）。
- **全量**：engine-native **884 passed / 0 failed / 0 ignored**（golden 零扰动）。
- **Gate 1b**：`RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` exit 0。
- **键表守护** `host_behavior_keys_bijective` 更新为 438（tag 441）。
- 包哈希：`packages/extensions/shahai-swd-upstream-noname` 经 `rehash_npack` 重算。

## 结论

- **138/184 映射，deferred 47→44**。`T12-SWD-DEFERRED-SKILLS` 仍开放（余 44 项）。
- 内置引用白名单新增「流离」（原生已有实现的标准技）。
