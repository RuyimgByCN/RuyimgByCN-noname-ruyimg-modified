# T6③ 正式服务端接入：安装包 → 装载 → 原生建局（2026-10-13）

## 目标与范围

关闭[主稿](../../../architecture/native-progress.md) T6③「正式客户端/服务端接入『安装包 → 装载 → 原生建局』，和 T13 衔接：产品路径有实测证据」。

本批改动全部落在 `apps/game-server`（新模块 `src/native.rs` + 两条 HTTP 路由 + 集成测试）；`crates/engine-native`、`crates/content-native-loader`、`crates/content-runtime` 零改动，故无规则版本递增、无 14 golden 重录、无 engine-native Gate 1b（按 AGENTS.md 影响面口径）。

## 产品路径语义

1. **安装包**：启动时 `NativeContent::open(native_packages_root, native_store_root)` + `import_extensions`（content-runtime 包存储，幂等）。配置项 `native_packages_root`（默认 `packages/extensions`）与 `native_store_root`（默认 `data/native-content`），env `NONAME_NATIVE_PACKAGES_ROOT` / `NONAME_NATIVE_STORE_ROOT`；与旧线 drop-in（`extensions/` 的 npack，engine-core）分根互不干扰。
2. **装载**：按包 id 读安装后 `assets/cards.json` → `content_native_loader::bridge_pack`（C0 装载器，符号表上限门 ADR 0779 在装载器内）。装载产物含暂缓牌清单（`deferred`），计入 summary `deferred_cards`。
3. **原生建局**：`Engine::new_with_extensions`（E03 冲突校验合并目录，内容锁版本绑定扩展集合）。
4. **有界 AI 驱动（T13 衔接）**：`BoundedPlanner`（64 节点 + 50ms 时限）逐座规划/主通道应用；开放询问座位优先应答，否则回合座位行动；连续 64 步无进展记 `stalled` 诚实退出。

HTTP 面：

- `GET /native/packages` — 已安装原生包清单（`NativeContent::installed`）。
- `POST /native/games` — body `{packages:[包id], seed, seats(2..=8), max_steps}`，返回 `NativeGameSummary`（实际装载包/暂缓牌数/目录基线对照/步数/终局与胜利方/stalled/elapsed_ms）。CPU 密集段经 `tokio::task::spawn_blocking`。

## 实测记录

集成测试 `apps/game-server/tests/native_product_path.rs`（3/3 通过，debug 构建）：

| 测试 | 结果 |
|---|---|
| `t63_import_load_build_and_drive_with_extension` | 安装→列出→装载 7 牌（与 G1 口径一致）→建局目录 124+7→AI 驱动 195 步**终局** Player(1) 胜，34ms |
| `t63_standard_only_build_and_drive` | 纯标准局驱动 128 步**终局**，23ms |
| `t63_load_unknown_package_errors_cleanly` | 未安装包装载可处理报错（`包未安装: …`） |

真进程 HTTP 端到端（`./target/debug/game-server`，`GAME_SERVER_ADDR=127.0.0.1:7772`，根同上，2026-10-13）：

```
GET  /native/packages → 15 包（huanle-cards/wuxing/d3-select/shaai-ow/cardpile×2…）
POST /native/games {"packages":["huanle-cards"],"seed":7301,"seats":3,"max_steps":2000}
→ {"packages":["huanle-cards"],"deferred_cards":0,"seed":7301,"seats":3,
   "catalog_cards":131,"base_catalog_cards":124,"steps":195,
   "finished":true,"winner":"Player(1)","stalled":false,"elapsed_ms":41}
```

同一 seed(7301)/同座位数下集成测试与 HTTP 路径结果一致（195 步、Player(1)），路径确定性可复现。

## 命令与结果

```
cargo test --offline -p game-server --test native_product_path   # 3/0/0
cargo test --offline -p game-server                              # 46/0/0
  （main 单测 9 + fault_injection 11 + m8_rolling_upgrade 4 + multi_room_stress 5
    + native_product_path 3 + room_flow 4 + tournament_wal_signing 10）
cargo check --offline -p game-server --all-targets               # 本批源零新增警告
```

Gate 口径说明：`RUSTFLAGS="-D warnings"` 全量检查会在**上游既有** `content-compiler` 警告（unused `value`、dead `patch_jumps_if`/`patch_loop`，非本批引入）上失败；本批 Gate 按 AGENTS.md 影响面收窄为「game-server 源零新增警告」（`tests/../src/native.rs` 无警告；`persist.rs` 既有警告不在本批范围）。

## 已知边界与登记

- 装载面当前为**卡牌包**（`bridge_pack`）；武将装载（`bridge_generals`）未进产品端点——G1 纵向口径的卡牌包先行，武将包接入选批另行登记。
- `packages/extensions` 存在两个不同来源的 `cardpile` 包 id（upstream-noname / upstream-liuli），列表端点按存储原样展示；包 id 唯一性属 C0/安装层既有语义，不在本批处理。
- AI 驱动为启发式（伤害差评分），`stalled`/步数上限均为诚实口径输出；策略质量归 T13。
- 客户端（web/desktop）仍未接原生对局 UI——T6③ 验收以「服务端产品路径 + 实测证据」为准；客户端接入随 T13 产品子项推进。
