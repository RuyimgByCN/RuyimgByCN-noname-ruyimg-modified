# 阶段 26：设备档位基准（§38.1）

> **历史阶段记录**（T16④ 补注，2026-10-11）：本文为旧线阶段存档，「当前/已完成/待做」类陈述仅反映撰写时点；现行状态以[进度主稿](../architecture/native-progress.md)、[覆盖清单](../native-coverage-manifest.md)与[规则基准](../architecture/native-rules-baseline.md)为准。

> 对应架构计划 v2.0 §38.1（目标设备档位）+ §38.4（临时性能预算）。
> 性能目标按目标设备分层，不使用单一开发机结论。

## 新增

`apps/benchmark-runner/src/device_tiers.rs` — 6 个设备档位 + 每档预算门。

## §38.1 六个设备档位

| 档位 | label | 代表硬件 |
|---|---|---|
| 低端 Android WebView | `android-low-webview` | Android 8+, 2GB RAM |
| 主流 Android | `android-mainstream` | Android 12+, 6GB RAM |
| iPhone / iPad | `iphone-ipad` | A12+, Safari |
| 普通桌面浏览器 | `desktop-browser` | Chrome on desktop |
| 桌面 Native | `desktop-native` | Tauri/Rust native |
| 服务器基准机 | `server-baseline` | 专用服务器 |

## 每档性能预算（§38.4）

> **注（2026-08-21）**：下表为阶段 26 时点的初始预算。`desktop-native` 的
> StartGame 预算已由阶段 17 的真内容重测**重基线**（见
> `phase17-test-perf-infrastructure.md` 第二笔基线），以 phase17 为准。

| 指标 | 低端Android | 主流Android | iPhone/iPad | 桌面浏览器 | 桌面Native | 服务器 |
|---|---|---|---|---|---|---|
| StartGame P50 | 2000μs | 1000μs | 800μs | 500μs | 300μs | 100μs |
| Command P95 | 8000μs | 4000μs | 3000μs | 2000μs | 1000μs | 500μs |
| 距离查询 P95 | 10000μs | 5000μs | 3000μs | 2000μs | 1000μs | 300μs |
| 快照恢复 | 5000μs | 2000μs | 1500μs | 1000μs | 500μs | 200μs |
| 观察者投影 | 3000μs | 1500μs | 1000μs | 800μs | 400μs | 100μs |
| 帧预算 | 16ms | 16ms | 16ms | 8ms | 8ms | 8ms |
| 内存预算 | 128MB | 256MB | 256MB | 512MB | 1024MB | 2048MB |

预算随档位提升递减（更快设备 = 更严格门）。

## CLI

```bash
# 列出所有档位
benchmark-runner tiers

# 按档位运行基准 + 预算检查
benchmark-runner tier-report desktop-native
benchmark-runner tier-report android-low-webview
benchmark-runner tier-report server-baseline
```

输出示例：
```text
=== 桌面 Native (Tauri/Rust) ===
tier=desktop-native
  ✓ distance-query-p95       1 / 1000 us (0%)
  ✓ snapshot-restore         347 / 500 us (69%)
  ✓ observer-projection      166 / 400 us (41%)
  REGRESSION start-hash-p50  391 / 300 us (130%)
result=FAIL
```

超预算的场景标记为 REGRESSION，CI 可按 exit code 阻止发布。

## 测试

```bash
cargo test -p benchmark-runner    # 8 tests
```

| 测试 | 断言 |
|---|---|
| `all_tiers_defined` | 恰好 6 个档位，label 唯一 |
| `tier_budgets_decrease_for_faster_tiers` | 服务器预算 < 低端 Android |
| `from_label_round_trips` | label ↔ enum 往返 |
| `budget_check_passes_within_limit` | 预算内 ✓ |
| `budget_check_fails_over_limit` | 超预算 REGRESSION |
| `tier_report_all_passed` | 报告渲染 |
| `frame_budget_within_plan_guidelines` | 桌面/服务器 ≤ 8ms |
| `low_end_android_has_lowest_memory_budget` | 低端 Android 内存最低 |
