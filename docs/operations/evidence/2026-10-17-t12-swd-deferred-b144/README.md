# swd deferred 消解批 144：涅槃（`niepan`）跨来源引用面（v318/存档 v102 不变，2026-10-17）

- 日期：2026-10-17
- 范围：swd 来源 deferred 技能 **涅槃**（`swd拓跋玉儿` 的 `niepan`）——**包内引用映射**，不新增引擎行为
- **规则/存档版本不变**（v318 / 存档 v102；无代码改动，仅包数据 mapping）
- 依据：涅槃即基础包庞统限定技，原生已于**批 118**（T11 Boss 庞统族）以宿主行为 `HostBehavior::Niepan`（tag 471、键 `host:niepan`、目录技能名「涅槃」）落地；本批把 swd 包内该技能由 `mapping:null`（deferred，理由「外部包技依赖」）改为 `mapping:"host:niepan"`，npack 已 rehash。

## 计数

- 武将技能条目：191 总数，**deferred 9 → 8**（映射 182 → 183）。
- 台账口径（登记表）：**174/184 → 175/184 映射，deferred 8 → 7**。

## 验证

```
cargo test --offline -p engine-native --test swd_niepan_ref
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（涅槃引用面） | **2 passed / 0 failed**（[targeted.log](targeted.log)）：swd 包装载下「涅槃」可授予；濒死流程完成不 panic；`bridge_generals` 的映射键含 `niepan`（不再计入未映射） |
| engine-native 全量 | **1021 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | `replay_scenarios` 2/2 **零位移**（无代码/版本变化，未重录）——[replay-golden-v318.log](replay-golden-v318.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |

## 说明

- 本条为**跨来源技能复用**（同一技能在 swd 包内引用基础包实现）——不再作为 deferred 计数，且**不新增**引擎行为；涅槃本体语义与差异仍以批 118 证据为准。
- swd 包剩余 deferred（7）：`mohua2`（需先落地 魔焰/灭道/疾风/修罗 四技）、`liaoyuan`、`shengong`、`gongxin`、`xuanyuan`、`kunlunjing`、`duijue`。
