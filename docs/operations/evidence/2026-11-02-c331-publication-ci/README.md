# C331 线按裁决 25 发布与 exact-head CI 登记（2026-11-02/03）

## 发布（正常、快进、单次）

- 发布提交 `96236c96`（tree 含 `f5e560db` 合并树＋4 笔文档提交），自远端 `native-convergence` 旧值 `9f4b7423` **快进**推送（[裁决 25](../2026-10-28-user-decisions/README.md)）；远端 `main`＝`47347c66` 不变。
- 驱动脚本（本目录上级私有副本同款逻辑）：[publication-driver 描述见 publication.log]。守卫：命令级 `core.hooksPath` 指向一次性 guard hook（只放行 `96236c96:refs/heads/native-convergence` 自 `9f4b7423` 的单条快进；8 项负控＋原 local-only hook 仍拒绝全部通过，见 `push-stage/result.json` 的 `selftests`）；仓库自身 `.git/hooks/pre-push` 原样（前后 SHA 校验）；推送前后 index／未跟踪快照逐字节相同（`before_equals_after: true`）。
- 增量审计：`9f4..9623` 恰 502 路径、8 提交；无二进制（.wasm/.exe 等）、无 node_modules／target／profile／dist 路径、`.pi-lens.json` 不在树内；新对象 493 个／未压缩 18.2MB、最大 blob 983KB。
- 原始材料：[push-stage/](push-stage/)（result.json、remote-before/after、push.stdout/stderr.gz、hooks、前后快照）、[publication.log](publication.log)。

## exact-head CI（如实登记：**9/10，perf-ubuntu 失败**）

[run 41467336967](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/41467336967)，push 触发、attempt 1、精确绑定 `96236c96`：

| job | 结果 |
| --- | --- |
| build ×3（mac/win/ubuntu） | success |
| gates-ubuntu（四道门禁＋行数＋分层） | success |
| native-web（WASM 重建／Worker／前端） | success |
| desktop ×3（安装器产出＋上传） | success |
| perf-macos | success（start-hash-p50 **74**/100µs） |
| **perf-ubuntu** | **failure**：`REGRESSION start-hash-p50 111 / 100 us (111%)` → `result=FAIL` |

对失败 job 按规程补跑（首跑与补跑分开记录，不以补跑抹去失败）：**attempt 2 仍 failure，start-hash-p50 142/100µs**（[attempt2-logs.zip](ci/attempt2-logs.zip)）。

### 同机前后对照（AGENTS.md 第 5 条）

| 环境 | baseline `9f4b7423` | `96236c96` 代码面 | 结论 |
| --- | --- | --- | --- |
| 本机 arm64 release（benchmark-runner start-hash，3 次） | 50.6／49.9／50.0 µs | 49.0／50.3／48.5 µs | 无回归 |
| 本机 x86_64（Rosetta，3 次） | 90.0／85.2／84.0 µs | 85.2／85.8／85.7 µs | 无回归 |
| CI macOS（同 job） | 70 µs | 74 µs | +6%，预算内 |
| CI Ubuntu（同 job） | 48 µs（9f4；历史 48–78） | **111／142 µs** | 两 attempt 均超预算 |

原始数据：[perf-probe/](perf-probe/)。CI 两次 attempt 同 runner image（ubuntu-26.04）同 rustc（1.99.1），排除工具链漂移；benchmark-runner 与预算自 `2b9f8266` 未改。本机双架构均无代码回归，CI Ubuntu 独有 2–3× 劣化——Linux x86_64 特异或宿主噪声二选一未定，Linux 容器复现实验进行中（docker 镜像拉取受限）。**该失败按原样登记，不据此宣称发布 CI 绿；后续修复／结论另批登记。**

## 边界

- 本批只发布到 `96236c96`；其后的 `14ee4b63`（stash 恢复）与 `cf38bd0d`（C9e 收口）尚未推送，待下一轮验收批次发布。
- 安装器 artifact 仅 API 元数据（[ci/artifacts.json](ci/artifacts.json)），无下载／实体安装／签名验证。
