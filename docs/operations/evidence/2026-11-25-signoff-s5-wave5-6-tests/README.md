# 2026-11-25 Batch S5：`Wave 5`（6 含 Spine 源）＋`Wave 6`（3 冻结边界源）八维测试与 `37/46` 完备性证据

- **前批精确 CI 核验与不上调纪律**：Batch S4 提交 `d130c3f1292ed3a34b166c62de3b080665c768cf` 的 CI Run [`45616460633`](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/45616460633) Attempts 1–5 如实记入 [ci-runs.md](../ci-runs.md)（Attempt 1 `8/10`、Attempt 2 `9/10`、Attempts 3–5 仅 `desktop-macos-latest` 因 GitHub 托管 macOS arm64 容量排队 `runner_id=0, steps=[]` 取消、`job_failure_count=0`）。严格遵守「门禁未全绿不上调」，**本批不提前上调 Wave 4 签署**（签署表保持 **`22/37`** 仓内 7/8、`0/37` 整包 8/8），待本批 CI 取得精确提交 **`10/10 success`** 后，再于 Batch S6 一次正式签署 Wave 4（`28/37`）＋ Wave 5（`6` 源 6/8 ✅）＋ Wave 6（`3` 源 5/8 ✅）。
- **版本与门禁不变量**：`333/107/5/759/8`、`u01-dddanzhi-pending-v107.ngsa` 与 14 条 golden 哈希零改动；`inventory_status.json` 保持 `4,419 / 5 / 0`。

---

## 一、`nc12_bench -- source_matrix` 同设备性能实测（`Wave 5` ＋ `Wave 6`）

摘自 `source_matrix.tsv`（`sha256: 607baebb729aa7794de18dfe8169a3262afd52400bb5bbdc8a98e97dfec2c8da`）：

| Tier / 来源 | 包目录 | bridge (ns) | build p50 (ns) | cmd p50 (ns) | save/restore p50 (ns) | 结果与故障 |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| `none` (基线) | `-` | 0 | 13,083 | 24,791 | 292 / 12,625 | `OK` (`0/0/0/0`) |
| `src:L08` (十周年UI) | `decade-ui-upstream-liuli` | 174,083 | 24,000 | 19,916 | 292 / 22,750 | `OK` (`0/0/0/0`) |
| `src:L09` (千幻聆音) | `qhly-upstream-liuli` | 123,458 | 24,000 | 20,583 | 292 / 23,708 | `OK` (`0/0/0/0`) |
| `src:L10` (史诗卡牌) | `epic-cards-upstream-liuli` | 195,750 | 28,542 | 19,251 | 292 / 26,875 | `OK` (`0/0/0/0`) |
| `src:L11` (太虚幻境) | `taixu-upstream-liuli` | 174,208 | 24,167 | 20,126 | 292 / 22,333 | `OK` (`0/0/0/0`) |
| `src:L12` (如真似幻) | `rzsh-upstream-liuli` | 172,959 | 24,208 | 20,916 | 291 / 22,459 | `OK` (`0/0/0/0`) |
| `src:L24` (皮肤切换) | `skin-switch-upstream-liuli` | 182,375 | 24,041 | 19,500 | 292 / 22,750 | `OK` (`0/0/0/0`) |
| `src:U01` (3D精选) | `d3-upstream-noname` | 132,458 | 24,667 | 19,459 | 292 / 23,167 | `OK` (`0/0/0/0`) |
| `src:L28` (雷霆万钧) | `thunder-upstream-liuli` | 189,791 | 23,833 | 20,417 | 292 / 23,333 | `OK` (`0/0/0/0`) |
| `src:U05` (杀海拾遗) | 9 正式包聚合 | 2,407,750 | 225,958 | 40,375 | 542 / 218,291 | `OK` (`0/0/0/0`) |

---

## 二、`up37_eight_dim_signoff::wave5_6` 测试与独立差异复核（`15a3bb8a`）

- **测试覆盖**：[wave5_6.rs](../../../../apps/game-server/tests/up37_eight_dim_signoff/wave5_6.rs) 覆盖：
  - `Wave 5`（`L08, L09, L10, L11, L12, L24`）6 源全生命周期、依赖门禁、正反配置解析（`parse_decade_ui_config`、`parse_qhly_config`、`parse_epic_config`、`parse_taixu_config`、`parse_rzsh_config`、`parse_skin_switch_config`）、两组多包正反序同局组合（`L08+L09+L10`、`L11+L12+L24`）及物理卸载后旧 `ContentLock` 无损恢复；
  - `Wave 6`（`U01, L28, U05` 含 9 子包）全生命周期、依赖门禁、正反配置/资产解析（`bridge_generals`、`parse_thunder_config`、`parse_shahai_main_config`、9 包 `bridge_pack` + 云筹 `bridge_generals`）、三组多包组合（`U01+L28`、`L08+U01`、`U05` 9 包同局）及全生命周期无损恢复；
  - [map.tsv](../2026-10-19-package-source-map/map.tsv) 刚性断言：`wave1..wave6` 精确覆盖全部 **`37` 来源 / `46` 正式包**，无遗漏或重复。
- **独立复核员 `15a3bb8a-fcc3-4964-99bc-f4de2d1821e1` 核实全 9 源边界**：
  - **`Wave 5`（6 个 Spine 4.0 100% 等价源，`1,513 / 0 / 0`；仓内预期 `6/8 ✅`，第 ⑥、`⑧` 维均 `部分`）**：
    - `L08` 十周年UI（`153 / 0 / 0`）、`L09` 千幻聆音（`266 / 0 / 0`）、`L10` 史诗卡牌（`30 / 0 / 0`）、`L11` 太虚幻境（`936 / 0 / 0`）、`L12` 如真似幻（`4 / 0 / 0`）、`L24` 皮肤切换（`124 / 0 / 0`）：均在 Web 端净室静态替代外部 Spine 4.0 商业运行库，零规则差已入源码与清单；第 ⑥ 维缺移动真机与三端签名，第 ⑧ 维尚待用户按裁决 42 在本机 Chrome 实测 Spine 4.0 画面对照后方可转绿。
  - **`Wave 6`（3 个 Decision 40 冻结边界源；仓内预期 `5/8 ✅`，第 ①、`⑥`、`⑧` 维均 `部分`）**：
    - `U01` 3D精选（`94 / 1 / 0`，**98.95%**）：已查明 `U01` 不含 Spine 商业库（采用纯 CSS 3D `perspective(600px) rotateY(...)`），唯一剩余边界为 `dddanzhi`（`Prepare` vs `phaseBegin` 冻结于 `NGSA v107`，对应 `u01-dddanzhi-pending-v107.ngsa`）。
    - `L28` 雷霆万钧（`55 / 1 / 0`，**98.21%**）：`th_zhengjing` 依 Decision 40 以阶段抽象收口；另有低风险冻结约束与 Spine 4.0 本机对照（第 ①/⑥/⑧ 维均 `部分`）。
    - `U05` 杀海拾遗 9 包（`1,618 / 3 / 0`，**99.81%**）：严格仅余 3 项冻结边界——`old_longfan`（200ms DOM 连点小游戏由固定选数 `ChoiceSpec` 替代）及 `shixu` / `hs_nozdormu`（真钟 `get.time()` 冻结于确定性 WAL + `WS 8`；Batch S1 已补清 `is_chanyuan_sealed` 非时钟缺陷）。
