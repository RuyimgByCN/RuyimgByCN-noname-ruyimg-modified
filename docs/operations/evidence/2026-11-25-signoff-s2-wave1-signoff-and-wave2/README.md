# 2026-11-25 Batch S2：`Wave 1`（6 源）八维仓内 7/8 ✅ 正式签署 ＋ `Wave 2`（7 源）八维预验证据

- **前批精确 CI 核验**：Batch S1 提交 `6dbf6e0f0ad15258b6b577f6d76a68ac611a613c` 的精确提交 CI Run [`45577611687`](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/45577611687) Attempt 1 全部 **10/10 作业 `success`**（`logs-first.zip` `sha256: 3254896c0a3661313f24e21b56c788d49c3afc11766942e50795e0ac826efe94`，已登记入 [ci-runs.md](../ci-runs.md)）。
- **版本与门禁不变量**：`333/107/5/759/8`、`u01-dddanzhi-pending-v107.ngsa` 与 14 条 golden 哈希零改动；`inventory_status.json` 保持 `4,419 / 5 / 0`。

---

## 一、`Wave 1`（`U06`/`U07`/`U08`/`U02`/`U03`/`U04`，6 源）正式上调签署

因 S1 精确提交 CI 已 `10/10` 通过，在 [37 来源八项签署表](../2026-10-19-source-signoff-32/README.md) 与 [两仓扩展台账](../../../migration/upstream-extension-reconstruction.md) 正式将 **`U06`/`U07`/`U08`/`U02`/`U03`/`U04`** 上调为 **仓内 7/8 维度 `✅` 完成**（①功能 100% 等价 + 全生命周期、②配置、③依赖、④组合、⑤保存恢复、⑦性能、⑧规则差异全部 `✅`），第 ⑥ 维（目标平台）如实维持 **部分（四端已验 ✅；缺 Android/iOS 物理真机与三端签名公证）**。全仓整包八项全满签署保持 **`0/37`**（仓内 7/8 就绪 **`6/37`**）。

---

## 二、`Wave 2`（`L06`/`L01`/`L02`/`L03`/`L04`/`L05`/`L07`，7 源）性能对照与八维预验

### 2.1 `nc12_bench -- source_matrix` 同设备性能实测（第 ⑦ 维，摘自 `source_matrix.tsv` `sha256: 607baebb...`）

| Tier / 来源 | 包目录 | bridge (ns) | build p50 (ns) | cmd p50 (ns) | save/restore p50 (ns) | 结果与故障 |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| `none` (基线) | `-` | 0 | 13,083 | 24,791 | 292 / 12,625 | `OK` (`0/0/0/0`) |
| `src:L06` (wuxing) | `wuxing-upstream-liuli` | 138,959 | 26,833 | 24,043 | 417 / 25,000 | `OK` (`0/0/0/0`) |
| `src:L01` (AI优化) | `ai-tuning-upstream-liuli` | 239,542 | 24,041 | 19,708 | 292 / 22,542 | `OK` (`0/0/0/0`) |
| `src:L02` (SJ Settings) | `sj-settings-upstream-liuli` | 493,375 | 25,625 | 21,959 | 333 / 22,875 | `OK` (`0/0/0/0`) |
| `src:L03` (boss 组合) | `boss-play-upstream-liuli+boss-zhugeliang-upstream-liuli` | 314,000 | 26,042 | 19,834 | 292 / 24,625 | `OK` (`0/0/0/0`) |
| `src:L04` (cardpile) | `cardpile-upstream-liuli` | 298,333 | 41,792 | 23,542 | 375 / 40,541 | `OK` (`0/0/0/0`) |
| `src:L05` (coin) | `coin-upstream-liuli` | 127,208 | 24,416 | 20,875 | 292 / 23,125 | `OK` (`0/0/0/0`) |
| `src:L07` (全能搜索) | `omnisearch-upstream-liuli` | 152,667 | 24,291 | 19,250 | 292 / 22,292 | `OK` (`0/0/0/0`) |

### 2.2 `up37_eight_dim_signoff::wave2` 与独立差异复核（`6c2971da`）

- **测试覆盖**：[wave2.rs](../../../../apps/game-server/tests/up37_eight_dim_signoff/wave2.rs) 针对 7 源 8 个正式包运行全生命周期（验签/拒无签/幂等/启停重启/升级回滚/物理卸载清空）、缺失依赖拒装与被依赖阻断、`parse_wuxing_config`/`parse_ai_tuning_config`/`parse_sj_config`/`parse_boss_play_config`/`bridge_pile`+`parse_pile_factors`/`parse_coin_config`/`parse_config` 正反例、同局多包正反序组合守恒及升级/卸载后旧 `ContentLock` 存档无损恢复。
- **独立复核员 `6c2971da-c04c-405e-80b9-8d0d55b6e5e3` 全部尚存差异/平台边界**：
  - **`L06` wuxing（3/3 等价）**：依冻结 `NGSA 107`，`taoyuan`/`wugu` 保持单批事件结构且 `wuxingpan_skill` 第二步固定执行 `SelectNature`；使用确定性 RNG 与弃牌（联机/录像/help 排除）。
  - **`L01` AI优化（133/133 等价）**：原生 `engine-ai` 为独立策略引擎，装载器仅注入调参配置与 `l01_first_ks_done` 启动钩子（不执行文件 I/O、网络或 `eval`）。
  - **`L02` SJ Settings（5/5 等价）**：安全隔离——不拉远端目录（`Offline`/`NetworkRefused`）、不 `eval` `update.js`、不打开 PDF、`eruda`/`reload`/`loop` 均阻断（`eruda_started=false`、`restarted=false`、`looped=false`）。
  - **`L03` boss（8/8 等价）**：8 名剑阁 Boss 按 `L03` 独立口径分派（如 `retieji` 仅 `chooseToDiscard`+直击，区别于 `U02`）；双正式包 `boss-play-upstream-liuli` + `boss-zhugeliang-upstream-liuli` 全部纳入。
  - **`L04` cardpile（21/21 等价）**：点数按 `RuleRng::from_seed` 确定性注入替代 `Math.random()`；缺倍率牌不追加（`0` 张）。
  - **`L05` coin（4/4 等价）**：Rust 数值/下注/结算引擎不触碰画布/音频/弹窗，`unit_random` 由参数/RNG 确定性注入；4 项粒子表面由前端 `visualEffectsBatch7Runtime.ts` 承担。
  - **`L07` 全能搜索（13/13 等价）**：安全/隐私边界——仅检索公开目录；暗牌实例查询返回 `SearchError::HiddenCard`、`debug_execute` 恒返 `SearchError::EvalRefused`。
- **上调节奏**：`Wave 2` 本批提交并取得精确提交 CI `10/10` 后于 `Batch S3` 正式在签署表上调。
