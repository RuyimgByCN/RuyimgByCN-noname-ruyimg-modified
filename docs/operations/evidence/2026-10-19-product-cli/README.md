# 产品入口验证：**`engine-cli`**（2026-10-19）

- 工作包：**NC09/NC12 产品面**；规则版本 v322（工作树）/ 存档格式 v105
- 工具链：Apple M3 Max / macOS 27.0.1 / rustc 1.95.0（debug 构建，`cargo run --offline -q -p engine-cli`）
- 原始输出：[run-stage1.txt](run-stage1.txt)、[run-stage2.txt](run-stage2.txt)

## 1 验证的命令链

| # | 命令 | 结果 |
|---|---|---|
| 1 | `engine-cli hash 4 42` | `8fe8316a05b9617c3aec072d0212af65f2c62d5d906e3b4eb0bf9159788159ff` |
| 2 | `engine-cli match 4 42 200` | `steps=64 revisions=64 capped=false`，终局 `hash=b351f487…` |
| 3 | `engine-cli replay-export 4 42` | `replay_bytes=1738`、`export_manifest_hash=1a54d2ed…`、**`imported_state_hash=8fe8316a…`（= 第 1 步！）**、`imported_manifest_hash` **一致**、**`replay_roundtrip=ok`** |
| 4 | `engine-cli full-demo 4 42 200` | **「忠臣/主公胜利！所有反贼和内奸已阵亡」**、`✓ 对局自然结束，用时 64 步 / 64 次状态变更` |
| 5 | `engine-cli load-hash <cards.json> 4 42` | **无输出**（参数/输出面待查，**未验证**）|

## 2 判读（**如实**）

1. **`replay_roundtrip=ok`** 且 `imported_state_hash` 与**原始 `hash` 命令逐字相同** ⇒
   **导出→导入的初始状态一致** ✅（**进程内**往返）；
2. `match` 与 `full-demo` 在**同一 seed** 下都是 **64 步**，且 `full-demo` 给出**终局与胜方** ⇒
   **CLI 能跑完整局** ✅；
3. **`max_steps=200` 而实际 64 步**、`capped=false` ⇒ **自然结束**（非截断）✅；
4. `load-hash` **无输出** ⇒ **本节不宣称该项通过**（可能参数不符或该子命令输出到 stderr）。

## 3 边界（**如实**）

1. **debug 构建**（非 release）⇒ **性能数字不可用**";
2. **`replay-export` 未落盘**（只打印 `replay_bytes`）⇒ **跨进程 `replay-import` 未验证**（需先解决文件输出）；
3. `load-hash` 未验证；
4. 未覆盖：真机、浏览器、TLS、跨机。
