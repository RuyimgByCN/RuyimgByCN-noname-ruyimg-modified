# NC08 跨目标门禁：原生引擎与 WASM 包装层同源（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第一切片：**跨目标运行与恢复门禁 + 门禁可失败证明**
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**跨目标门禁已建立（4 项）+ 受控变异证明门禁有分辨力**

## 1 门禁口径（不依赖浏览器）

新增 `crates/engine-native-wasm/tests/nc08_cross_target.rs`：直接调用 WASM crate 的
内部实现路径（新增 `#[doc(hidden)] pub mod test_support` 暴露
`engine_from_packs`/`engine_from_archive`/`hex_encode`——与 wasm-bindgen 导出**同一实现**），
与原生 `engine-native` 逐项对照。真正的 WASM 目标运行由 `wasm-pack` 产物冒烟
（`pnpm --dir apps/web-client smoke:native`）覆盖，两层边界分别记录。

| 验收项 | 判据 | 结果 |
|---|---|---|
| 同一输入归档一致 | 同 seed + 同包集合 → 两侧归档字节**逐字节相同** | 通过 |
| 推进后仍一致 | 12 步 NextPhase 后修订号与归档仍相同 | 通过 |
| 同包恢复一致 | `engine_from_archive` 恢复后**状态摘要**与原件一致（归档字节含恢复代次等宿主字段，一致性以摘要为准） | 通过 |
| **门禁可失败** | ① 标准目录恢复扩展局 → 拒绝；② 包版本不符 → 拒绝；③ 换 seed → 归档不同；④ 篡改归档尾字节 → 拒绝 | 通过 |

## 2 受控变异（反「只检查文件存在/JSON 可读」）

第 4 项即 NC08 要求的**受控变异证明**：四类反例各自必须失败或不一致，
否则门禁无分辨力。测试同时断言「不同 seed 归档不得相同」——防止门禁对输入不敏感。

## 3 验证结果

```
cargo test --offline -p engine-native-wasm --test nc08_cross_target
cargo test --offline -p engine-native-wasm --no-fail-fast
cargo test --offline -p engine-native --no-fail-fast
python3 -B scripts/check-source-lines.py
```

| 检查 | 结果 | 证据 |
|---|---|---|
| 跨目标门禁定向 | **4 passed / 0 failed** | [targeted.log](targeted.log) |
| `engine-native-wasm` 全量 | **29 passed / 0 failed**（25 → +4） | [wasm-tests.log](wasm-tests.log) |
| `engine-native` 全量（含 examples） | **1140 passed / 0 failed / 2 ignored** | [engine-native-tests.log](engine-native-tests.log) |
| 行数门禁 | 8242 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 4 未完成 / 待办（NC08）

- [ ] **真 WASM 目标**：本门禁在宿主（非 wasm32）下走同一实现路径；wasm32 目标运行由
      产物冒烟覆盖，仍缺「wasm32 下的存档往返」自动化（需 wasm-pack/wasmtime 工具链）
- [ ] **旧线对照**（`engine-cli`/`engine-wasm` hash 检查）按旧线解释：本轮未做
- [ ] **重复测试属性/过期守护**复核：`engine-native-wasm` 的 20 项 lib 测试无同名重复；
      其余 crate 的固定目录数守护需逐项复核（NC03 已修前端 `extension_catalog_15`）
- [ ] **在测试中重新实现生产逻辑的假闭环**逐项排查（本门禁已改为调用生产实现路径）
- [ ] 上游行为对照（固定上游版本/局面/随机输入逐命令比较）属 NC08 主体，未开始
- [ ] 各层报告边界（单元/内核集成/生产适配器/真实客户端/服务端/native-WASM/上游/异常输入）
      的清单化登记
