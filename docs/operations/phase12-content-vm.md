# 阶段 12：内容包进引擎（RunVm）与跨目标

## 目标

- 编译后的 `.skill` / content package 可装入 `Engine`
- 包内 trigger **覆盖** 同 skill_def 的 bootstrap 原生 handler
- `TriggerHandler::RunVm` 真正执行 Rule VM → `EffectRequest`
- 未装包时行为与黄金用例不变

## API

```rust
let pkg = content_compiler::compile_source(src)?;
engine.load_package(&pkg)?;
// StartGame 时 ruleset.logic_hash = content.combined_logic_hash
// 角色授予 skill_def=1 时走 package RunVm（若包提供了 skill 1）
```

`ContentRegistry`：

| 方法 | 作用 |
|---|---|
| `load_package` | 校验 bytecode、登记 module、写入 package trigger（id ≥ 10000） |
| `defs_for_skill` | 有包定义则覆盖 bootstrap |
| `module(name)` | RunVm 查找 |

## CLI

```bash
# DSL → 伤害摸牌（VM）演示
cargo run -p engine-cli -- skill-vm content/standard/jianxiong.skill

# 装包后 start hash
cargo run -p content-cli -- pack content/standard/jianxiong.skill \
  --out /tmp/jx.pkg.json --package-id standard
cargo run -p engine-cli -- load-hash /tmp/jx.pkg.json 4 42

# 无包黄金（与 phase 11 一致）
cargo run -p conformance
cargo run -p engine-cli -- hash 4 42
```

## 测试

```bash
cargo test -p engine-core --test phase12_content_vm
cargo test -p conformance
# WASM 编译（不可在 host 直接跑 .wasm 测试 harness）
cargo build -p engine-core --target wasm32-unknown-unknown
```

## 跨目标

Native 与 WASM 共用同一 `engine-core` 源码与 `conformance` 向量。  
host 上 `cargo test --target wasm32-unknown-unknown` 无法直接执行 wasm 二进制；CI 应：

1. `cargo build -p engine-core --target wasm32-unknown-unknown`
2. 用 `wasmtime` / browser worker 跑 `engine-wasm` 导出的 hash API，与 native `engine-cli hash` 对比

见 `tests/cross-target/README.md`。

## 安全

- 装包只加载已 verify 的 bytecode
- 内容不获文件系统 / 网络
- live match 冻结仍由 `DesktopHost::freeze_for_match` 负责（壳层）
