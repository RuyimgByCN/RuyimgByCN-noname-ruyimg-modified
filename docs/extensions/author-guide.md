# 扩展作者指南（旧线 DSL / M5 记录）

> 2026-09-22 范围修订：原生重写保留扩展机制，见 [原生扩展方案](../architecture/native-extensions.md)。
> 以下命令与 DSL 面向现有 engine-core/content-runtime，不代表 engine-native SDK 已可用；
> 新 SDK、独立包装载、模式/界面接口与旧扩展兼容仍待实现和验收。

> 更新：2026-08。对应架构计划 §49.4 M5 总验收门第 2 项：
> 「扩展作者文档：如何写 DSL、如何测、如何签名」。

## 1. 如何写 DSL（`.skill` 文件）

一个技能文件由若干 `skill` 块组成。最小示例（`content/community/hero.skill`）：

```
// 示例技：出牌阶段限一次，你摸一张牌。
skill hero_example(4200000901) {
  active play_phase uses_per_turn 1 {
    effect draw_cards(self, 1);
  }
}
```

语法要点：

- `skill <name>(<def_id>) [locked] { ... }`：技能名 + 稳定定义 ID（u32）+ 可选 `locked`。
- 构件（可组合多个）：
  - `trigger <event> <filter> [optional] { ... }`：触发技。事件见
    `crates/content-compiler/src/typecheck.rs` 的 `EVENTS`；过滤词见 `FILTERS`。
  - `active <window> [uses_per_turn N] [targets ...] [select_hand/select_he ...] { ... }`：
    主动技。窗口为 `play_phase`。
  - `view_as <kind> <window> <filter> [from <zone>] [uses_per_turn N] [cost ...] ...;`：
    转化技。可视为的基本/锦囊牌名与过滤词见 typecheck 的 `VIEW_AS_KINDS`/`VIEW_AS_FILTERS`。
  - `modifier <target>[(selector)] <add|override|...> <value|source>;`：修改技。
  - 效果语句：`effect <effect_name>(args);`；查询表达式：`hp(self)`、`hand_count(self)`、
    `mark_count(self, id)`、`skill_pile_count(self, id)` 等（见 typecheck 查询表）。
- 控制流：`if cond { ... }`、`else { ... }`、`let name = expr;`。比较符仅 `==`、`<`、`>`，
  逻辑仅 `&&`；`<=` 用 `< n+1` 表达，`||` 用并列 `if`。
- 定义 ID 必须是稳定 u32（`4200000xxx` 保留给示例/测试助手）。

## 2. 如何打包

```bash
cargo run -p content-cli -- pack content/community/hero.skill \
  --out content/community/dist/hero.npack.json \
  --package-id hero --namespace community --version 0.1.0 \
  --characters content/community/characters.json \
  --metadata content/community/metadata.json
```

- `metadata.json` 声明 `dependencies`（官方包依赖，如 `standard`）。
- `characters.json` 为角色列表（id/name_key/max_hp/skills/sex/group）。

## 3. 如何测试

- 单元/集成：在 `crates/engine-core/tests/` 添加 golden 测试（参见 `m5_community_example.rs`）：
  加载 npack → `StartGame` → `AddSkill` → 触发/激活 → 断言领域事件与状态。
- 四道门禁：`cargo test --workspace`、`cargo run -p conformance`、
  `bash scripts/cross-target-hash.sh`、`node scripts/sync-original-content.mjs --verify-only`。
- 修改 npack 后若涉及包哈希冻结测试（如 `original_sp_dingpan_buzhi`），需同步更新
  `PACKAGE_LOGIC_HASH` 与 `modules.len()`。

## 4. 如何签名与发布

- 内容锁校验：`content-cli --store <dir> install pkg.json --allow-unsigned`（开发期）。
- 生产签名流程随 M5 后续（channel 签名/撤销契约）推进；示例包全流程见
  `extension-studio` 与 `content-cli` 命令说明。
