# T12 swd deferred 消解批 101：改天道（2026-10-17）

- 日期：2026-10-17
- 范围：一技 deferred 消解（tag 440）——**135/184 技条目已映射**（deferred 48→47）
- 上游回读：`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js`（gtiandao 逐段核验）

## 语义与接线

| 技（将） | 上游语义 | 接线 |
|---|---|---|
| 改天道 gtiandao（孤月 `swd_guyue`） | `trigger: {global: "judge"}`：全局判定——可打任意 he 牌改判并**获得原判定牌**（同 Boss 张角「天道」语义） | `SwdGTiandao`（tag 440）=查询型能力（`TriggerMeta=None`/filter=false/效果步 Done，与鬼道/鬼才/天道同构）；判定机三处扩展：①改判者入队（`has(g_tiandao)`）②可改判性门（`has_any_hes`，同天道）③选项面（he 全量，同天道）。`swd_guyue` 的 gtiandao 映射为 `host:g_tiandao` |

## 验证

- **定向回归** `swd_deferred_gtiandao_rewrites_judge_with_any_he`：1 号持改天道、判定区闪电、手牌红桃桃 → 判定时挂 `gui_dao_rewrite`；选项含红桃手牌（hes 全量证据）；应答后改判牌离手、净持平、`JudgementRewritten{player:1}` 落账。2 passed / 0 failed（[targeted.log](targeted.log)，含 b100 兰芷回归）。
- **全量**：engine-native **883 passed / 0 failed / 0 ignored**（golden 零扰动——标准场景无 gtiandao）。
- **Gate 1b**：`RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` exit 0。
- **键表守护** `host_behavior_keys_bijective` 更新为 437（tag 440）。
- 包哈希：`packages/extensions/shahai-swd-upstream-noname` 经 `rehash_npack` 重算。

## 结论

- **135/184 映射，deferred 48→47**。`T12-SWD-DEFERRED-SKILLS` 仍开放（余 47 项）。
