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
| 5 | `engine-cli load-hash`（无包路径） | `hash=86daeaff…` ✅（**正确用法**；详见第 5 节）|
| 5b | `engine-cli load-hash <cards.json>` | **`exit=2`** + stderr 结构化错误 ⇒ **用法错（非缺陷）**，见第 5 节 |
| 6 | `replay-export 4 42 <file>` → **另进程** `replay-import <file>` | **见第 4 节**：落盘 1738 B、`chain=ok`、`hash` 与第 1 行**逐字相同** ✅ |

## 2 判读（**如实**）

1. **`replay_roundtrip=ok`** 且 `imported_state_hash` 与**原始 `hash` 命令逐字相同** ⇒
   **导出→导入的初始状态一致** ✅（**进程内**往返）；
2. `match` 与 `full-demo` 在**同一 seed** 下都是 **64 步**，且 `full-demo` 给出**终局与胜方** ⇒
   **CLI 能跑完整局** ✅；
3. **`max_steps=200` 而实际 64 步**、`capped=false` ⇒ **自然结束**（非截断）✅；
4. `load-hash` **无输出** ⇒ **本节不宣称该项通过**（可能参数不符或该子命令输出到 stderr）。

## 3 边界（**如实**）

1. **debug 构建**（非 release）⇒ **性能数字不可用**";
2. ~~`replay-export` 未落盘~~ ⇒ **已更正**：CLI 支持输出路径参数，**跨进程已实测通过**（见第 4 节）；
3. `load-hash` 未验证；
4. 未覆盖：真机、浏览器、TLS、跨机。

---

## 4 ✅ 第二轮：**跨进程**重放往返（消除第 3 节的边界 2）

**关键发现**：`replay-export` **本来就支持可选输出路径**（`replay-export [players] [seed] [out.json]`，
`main.rs:53-57`）——上一轮我**没传路径**就说「未落盘」❌ **是我的用法不对，不是功能缺失**。

### 4.1 实测（两个独立进程）

```
== replay-export 4 42 <file>
replay_bytes=1738
export_manifest_hash=1a54d2ed4e4ff107e1d5348c620c00776f9f29fd13c03dd95bfee17217cbdefd
wrote=/…/docs/operations/evidence/2026-10-19-product-cli/replay-4-42.json
imported_state_hash=8fe8316a05b9617c3aec072d0212af65f2c62d5d906e3b4eb0bf9159788159ff
imported_manifest_hash=1a54d2ed4e4ff107e1d5348c620c00776f9f29fd13c03dd95bfee17217cbdefd
replay_roundtrip=ok

== 文件：1738 bytes

== 另进程 replay-import <file>
format:   replay_log
records:  1
commands: 1
manifest: 1a54d2ed4e4ff107e1d5348c620c00776f9f29fd13c03dd95bfee17217cbdefd
chain=ok
revision: 1
hash:     8fe8316a05b9617c3aec072d0212af65f2c62d5d906e3b4eb0bf9159788159ff
```

| 判据 | 结果 |
|---|---|
| 导出**落盘** | ✅ **1738 字节**（[replay-4-42.json](replay-4-42.json)）|
| **另一进程**导入 | ✅ `format=replay_log`、`records=1`、`commands=1` |
| **重放链完整** | ✅ **`chain=ok`** |
| **state_hash 与 `hash 4 42` 逐字相同** | ✅ `8fe8316a…`（**三条命令互相印证**）|
| manifest 一致 | ✅ `1a54d2ed…` |

⇒ **跨进程**（导出进程 → 导入进程）重放往返**成立** ✅。

### 4.2 更正（**如实**）

第 1 节第 3 行与第 3 节边界 2 写的「**`replay-export` 未落盘 ⇒ 跨进程未验**」——**错误** ❌：
CLI **本就支持**输出路径参数；**是本轮我漏传参数**。现已实测通过。

### 4.3 剩余边界

1. ~~`load-hash` 无输出~~ ⇒ **已查清**（第 5 节）：**正确用法可用**；`cards.json` 不是 `ContentPackage` ⇒ `exit=2`；
2. **debug 构建** ⇒ 性能数字不可用；
3. 未覆盖：真机、浏览器、TLS、跨机。

---

## 5 ✅ 第三轮：`load-hash` 查清（**又一例「用法错非缺陷」**）

### 5.1 读实现

`run_load_hash`（`main.rs:336-356`）：

* **有包路径**：`content_schema::ContentPackage::from_json_bytes(&data)` —— **要 `ContentPackage` JSON**，
  不是 `cards.json`（内容资产）；失败走 `eprintln!` + **`exit(2)`**，**stdout 不输出**；
* **无包路径**：直接 `Engine::new()` 并打印 `hash=…`。

### 5.2 实测

```
== load-hash（无包路径）
hash=86daeaffe7f71fb6facda6e6ebe358a87076f5d9e2e7be196deb111b698c31a7

== load-hash cards.json
exit=2        （stdout 为空；错误走 stderr，是**结构化拒绝**而非崩溃）
```

| 判据 | 结果 |
|---|---|
| `load-hash`（**正确用法**，无包路径） | ✅ 打印 `hash=…` |
| `load-hash <cards.json>` | ✅ **`exit=2` + stderr 结构化错误**（**用法错，非缺陷**）|
| 上一轮「无输出」的结论 | ❌ **错误**——我把 `stderr` 丢到 `/dev/null`、又传了**非 ContentPackage** 文件 |

### 5.3 ⚠ **两个 hash 不是同一个量**（**不要混用**）

| 命令 | 输出 |
|---|---|
| `hash 4 42` | `8fe8316a…` |
| `load-hash`（无路径） | `86daeaff…` |

⇒ 两者**口径不同**（`load-hash` 在 `Engine::new()` 默认态上取 hash；`hash` 命令按 `players/seed` 建局）。
**本证据不据此判断谁对谁错**，只登记「**不可混用**」。
