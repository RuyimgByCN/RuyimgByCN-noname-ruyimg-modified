# 2026-11-25 Batch S4：`Wave 3`（9 源）八维仓内 7/8 ✅ 正式签署 ＋ `Wave 4`（6 源）八维预验证据

- **前批精确 CI 核验**：Batch S3 提交 `77a5f26829b0d1be3744ddd43642bae9c2eaac22` 的精确提交 CI Run [`45603333616`](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/45603333616) Attempt 1 全部 **10/10 作业 `success`**（`logs-first.zip` `sha256: 88295cc4d4ee80f516aea79ea4ac89f3dbb597383f35caba637501ee4f9af4ba`，已记入 [ci-runs.md](../ci-runs.md)）。
- **版本与门禁不变量**：`333/107/5/759/8`、`u01-dddanzhi-pending-v107.ngsa` 与 14 条 golden 哈希零改动；`inventory_status.json` 保持 `4,419 / 5 / 0`。

---

## 一、`Wave 3`（`L13..L21`，9 源）正式上调签署

因 S3 精确提交 CI 已 `10/10` 通过，在 [37 来源八项签署表](../2026-10-19-source-signoff-32/README.md) 与 [两仓扩展台账](../../../migration/upstream-extension-reconstruction.md) 正式将 **`L13`/`L14`/`L15`/`L16`/`L17`/`L18`/`L19`/`L20`/`L21`** 上调为 **仓内 7/8 维度 `✅` 完成**，第 ⑥ 维（目标平台）如实维持 **部分（四端已验 ✅；缺 Android/iOS 物理真机与三端签名公证）**。全仓整包八项全满签署保持 **`0/37`**（累计仓内 7/8 就绪 **`22/37`**）。

---

## 二、`Wave 4`（`L22`/`L23`/`L25`/`L26`/`L27`/`L29`，6 源）性能对照与八维预验

### 2.1 `nc12_bench -- source_matrix` 同设备性能实测（摘自 `source_matrix.tsv` `sha256: 607baebb...`）

| Tier / 来源 | 包目录 | bridge (ns) | build p50 (ns) | cmd p50 (ns) | save/restore p50 (ns) | 结果与故障 |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| `none` (基线) | `-` | 0 | 13,083 | 24,791 | 292 / 12,625 | `OK` (`0/0/0/0`) |
| `src:L22` (段位制裁) | `elo-sanction-upstream-liuli` | 123,083 | 24,167 | 19,542 | 292 / 23,792 | `OK` (`0/0/0/0`) |
| `src:L23` (电脑适配) | `desktop-adapt-upstream-liuli` | 175,875 | 24,750 | 19,375 | 292 / 22,750 | `OK` (`0/0/0/0`) |
| `src:L25` (祖安武将) | `zuan-upstream-liuli` | 123,209 | 24,250 | 19,584 | 292 / 23,958 | `OK` (`0/0/0/0`) |
| `src:L26` (笨蛋插件) | `fool-plugin-upstream-liuli` | 175,583 | 24,125 | 19,459 | 292 / 22,667 | `OK` (`0/0/0/0`) |
| `src:L27` (选将美化) | `select-beautify-upstream-liuli` | 186,459 | 25,750 | 19,875 | 292 / 24,041 | `OK` (`0/0/0/0`) |
| `src:L29` (龙血玄黄) | `longxue-upstream-liuli` | 311,042 | 25,917 | 19,208 | 292 / 24,833 | `OK` (`0/0/0/0`) |

### 2.2 `up37_eight_dim_signoff::wave4` 与独立差异复核（`c323960a`）

- **测试覆盖**：[wave4.rs](../../../../apps/game-server/tests/up37_eight_dim_signoff/wave4.rs) 覆盖 `L22, L23, L25, L26, L27, L29` 全生命周期、依赖门禁、正反配置解析（`parse_elo_config`、`parse_desktop_config`、`parse_zuan_config`、`parse_fool_config`、`parse_select_beautify_config`、`parse_longxue_config` 与 `bridge_pack`）、两组多包正反序同局组合及升级/卸载后旧 `ContentLock` 存档无损恢复。
- **独立复核员 `c323960a-a1e4-4c5e-826d-24c93c1b075b` 全部尚存差异/平台边界**（全 6 源共 `336 / 0 / 0` 单项等价）：
  - **`L22` 段位制裁（8/8）**：不筛选将池（`restrict_pool` 原样返回）、不睡眠/发台词/读 `sessionStorage`，`EloCard.score` 由参数供给。
  - **`L23` 电脑适配（1/1）**：禁窗口缩放与 DOM/CSS 注入（`invoke_emoji` 返 `DomRefused`），`export_download` 仅返安全文件名与字节。
  - **`L25` 祖安武将（257/257）**：安全拦截联机/非离线 `eval` DIY 编辑（`UnsafeEditorOutsideOfflineDebug` / `ONLINE_MULTIPLAYER_DIY_FORBIDDEN`）；`ssr_jinfan` 去实体化保静态锁；`_relifeskill` 与 `L19` 隔离；11 项上游禁用/注释钩子保持惰性。
  - **`L26` 笨蛋插件（7/7）**：Rust 规则不含画布/音频/睡眠/随机采样（参数供偏移）；两项上游死分支（`choose_card`/`custom_choose_file`）不执行。
  - **`L27` 选将美化（54/54）**：上游主文件混淆（`_0x` 打包），纯混淆 `select_ui` 主面板不逐字迁入；12 技 `ChoiceSpec` 抽象（`sbtongye`..`mbwuling`）、`jishe`/`jishe2` 注释分支实现、`huamu` 死分支 `l27_huamu_disabled_upstream=1` 均已登记。
  - **`L29` 龙血玄黄（9/9）**：上游 `extension.js` `jsjiami` 混淆基于 D5 轨迹与可读还原件验收；`shengjian`/`_CMeffects`/`main1` 在 Web 前端净室实现且零规则状态写入（不含商用 Spine 资产）。
- **上调节奏**：`Wave 4` 本批提交并取得精确提交 CI `10/10` 后于 `Batch S5` 正式在签署表上调。
