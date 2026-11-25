# 2026-11-25 Batch S3：`Wave 2`（7 源）八维仓内 7/8 ✅ 正式签署 ＋ `Wave 3`（9 源）八维预验证据

- **前批精确 CI 核验**：Batch S2 提交 `8caa8da447f393387d6107907a01e8947d66376a` 的精确提交 CI Run [`45588526123`](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/45588526123) Attempt 1 已分派 8/8 通过（仅 macOS 2 作业因托管机未分派被 `cancelled`），Attempt 2 `--failed` 重试后全部 **10/10 作业 `success`**（`logs-attempt2.zip` `sha256: 39562f5352e22a9243f1cb7ac9a525931f755b357b9b1359aeff8c5b1127e7a2`，Attempt 1/2 均已如实写入 [ci-runs.md](../ci-runs.md)）。
- **版本与门禁不变量**：`333/107/5/759/8`、`u01-dddanzhi-pending-v107.ngsa` 与 14 条 golden 哈希零改动；`inventory_status.json` 保持 `4,419 / 5 / 0`。

---

## 一、`Wave 2`（`L06`/`L01`/`L02`/`L03`/`L04`/`L05`/`L07`，7 源）正式上调签署

因 S2 精确提交 CI 已 `10/10` 通过，在 [37 来源八项签署表](../2026-10-19-source-signoff-32/README.md) 与 [两仓扩展台账](../../../migration/upstream-extension-reconstruction.md) 正式将 **`L06`/`L01`/`L02`/`L03`/`L04`/`L05`/`L07`** 上调为 **仓内 7/8 维度 `✅` 完成**，第 ⑥ 维（目标平台）如实维持 **部分（四端已验 ✅；缺 Android/iOS 物理真机与三端签名公证）**。全仓整包八项全满签署保持 **`0/37`**（累计仓内 7/8 就绪 **`13/37`**）。

---

## 二、`Wave 3`（`L13`..`L21`，9 源）性能对照与八维预验

### 2.1 `nc12_bench -- source_matrix` 同设备性能实测（摘自 `source_matrix.tsv` `sha256: 607baebb...`）

| Tier / 来源 | 包目录 | bridge (ns) | build p50 (ns) | cmd p50 (ns) | save/restore p50 (ns) | 结果与故障 |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| `none` (基线) | `-` | 0 | 13,083 | 24,791 | 292 / 12,625 | `OK` (`0/0/0/0`) |
| `src:L13` (手杀MVP) | `shousha-mvp-upstream-liuli` | 201,625 | 24,125 | 19,500 | 292 / 22,708 | `OK` (`0/0/0/0`) |
| `src:L14` (手杀ui) | `shousha-ui-upstream-liuli` | 188,125 | 24,166 | 19,250 | 292 / 22,625 | `OK` (`0/0/0/0`) |
| `src:L15` (手杀全扩) | `shousha-full-upstream-liuli` | 456,917 | 23,583 | 20,541 | 292 / 23,542 | `OK` (`0/0/0/0`) |
| `src:L16` (拖拽读取) | `drag-read-upstream-liuli` | 120,166 | 24,208 | 19,666 | 292 / 23,084 | `OK` (`0/0/0/0`) |
| `src:L17` (斗转星移) | `douzhuan-xingyi-upstream-liuli` | 172,667 | 25,167 | 19,417 | 292 / 22,500 | `OK` (`0/0/0/0`) |
| `src:L18` (更新保护) | `update-guard-upstream-liuli` | 187,375 | 24,000 | 20,042 | 292 / 22,875 | `OK` (`0/0/0/0`) |
| `src:L19` (末日乱斗) | `doomsday-brawl-upstream-liuli` | 190,750 | 24,708 | 19,834 | 333 / 24,000 | `OK` (`0/0/0/0`) |
| `src:L20` (标记补充) | `mark-supplement-upstream-liuli` | 147,208 | 27,083 | 19,041 | 292 / 24,125 | `OK` (`0/0/0/0`) |
| `src:L21` (武皇争霸) | `martial-upstream-liuli` | 130,583 | 24,000 | 19,792 | 292 / 23,125 | `OK` (`0/0/0/0`) |

### 2.2 `up37_eight_dim_signoff::wave3` 与独立差异复核（`2a8e7b2e`）

- **测试覆盖**：[wave3.rs](../../../../apps/game-server/tests/up37_eight_dim_signoff/wave3.rs) 覆盖 `L13..L21` 全部 9 源正式包全生命周期、依赖门禁、正反配置解析（`parse_mvp_config`、`parse_shousha_ui_config`、`parse_shousha_config`、`parse_drag_config`、`parse_douzhuan_config`、`parse_update_guard_config`、`parse_zuan_brawl`、`load_mark_supplement_config`、`parse_martial_config`）、两组多包正反序同局组合及升级/卸载后旧 `ContentLock` 存档恢复。
- **独立复核员 `2a8e7b2e-0048-4fdf-913e-d6171f8cdfed` 全部尚存差异/平台边界**（全 9 源共 `365 / 0 / 0` 单项等价）：
  - **`L13` 手杀MVP（10/10）**：无规则差；设计边界不替代内核 `game.over`、音频或 `get.attitude`（消费调用方参数）。
  - **`L14` 手杀ui（141/141）**：无规则差；纯 UI 包严格隔离（0 VM 技、0 牌）。
  - **`L15` 手杀全扩（14/14）**：无规则差；不播放音频、不渲染 DOM 或调用 `clsPrefixName`。
  - **`L16` 拖拽读取（3/3）**：无规则差；净室安全边界——`loadUpdateContent`/`generateZip` 仅经 `validateSafeExportFilename` 干跑清单，禁止真实磁盘写、网络与 `eval`。
  - **`L17` 斗转星移（11/11）**：无规则差；不弹菜单/播音/调用 `game.reload` 或 `eval`。
  - **`L18` 更新保护（13/13）**：无规则差；不请求网络、写盘、`eval`、弹菜单或睡眠。
  - **`L19` 末日乱斗（69/69）**：无规则差（裁决 40 `ChoiceSpec` 已收口；`_relifeskill` 已与 `L25` 隔离）；乱斗 UI 登记与 `human_seats` 参数化独立于便携规则模块。
  - **`L20` 标记补充（95/95）**：无规则差；上游块注释废案 `zuoxing` 与 `_shiqihujia_` 保持惰性未绑定。
  - **`L21` 武皇争霸（9/9）**：无规则差；禁用 `Math.random`/录像/`mightyFoe.js`/`connect` 与注释废案 `askJoin`，`double_hp` 按 `MartialHpRule` 接入 `GameMode::Kof`。
- **上调节奏**：`Wave 3` 本批提交并取得精确提交 CI `10/10` 后于 `Batch S4` 正式在签署表上调。
