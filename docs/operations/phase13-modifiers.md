# 阶段 13：包化 Modifier + 跨目标对账

## 目标

- 内容包 `modifier_index` 装入 `ContentRegistry`
- 同 `skill_def` 的 package modifier **覆盖** bootstrap（马术 / 奇策）
- 标准 DSL：`mashu.skill` / `qicao.skill`
- `scripts/cross-target-hash.sh` 一键校验 native 黄金 + wasm 编译

## 行为

| 状态 | skill_def 2 (马术) | skill_def 3 (奇策) |
|---|---|---|
| 未装包 | bootstrap distance_from -1 | bootstrap attack_range +1 |
| 装了对应包 | package def id ≥ 10000 | package def id ≥ 10000 |

Trigger 覆盖规则（阶段 12）不变：有 package trigger 则不用 bootstrap DrawSelf。

## CLI

```bash
cargo run -p engine-cli -- mod-demo
# distance_1_to_2=Int(...) attack_range_1=Int(...) ok=true

cargo run -p content-cli -- pack \
  content/standard/jianxiong.skill \
  content/standard/mashu.skill \
  content/standard/qicao.skill \
  --package-id standard --out /tmp/std.pkg.json
```

## 测试

```bash
cargo test -p engine-core --test phase13_modifiers
bash scripts/cross-target-hash.sh
```

## 跨目标脚本

`scripts/cross-target-hash.sh`：

1. `cargo run -p conformance`
2. 钉死 start hash 断言
3. `cargo build` wasm32 engine-core / engine-wasm
4. 可选 wasmtime 提示
5. `skill-vm` 冒烟
