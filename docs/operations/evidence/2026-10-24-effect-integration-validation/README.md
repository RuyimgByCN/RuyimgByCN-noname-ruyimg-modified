# EffectRequest 拆分后的完整 preflight 与产品补验（2026-10-24）

源码基准 `7e28a93799a9c01794085e911a72a4faa5673883` 加已有未提交工作树；规则 **v322**、存档 **v105**。这是[功能拆分](../2026-10-24-effect-split/README.md)之后的补验，没有修改产品源码、测试期望值或门禁。当前结论集中在[进度主稿](../../../architecture/native-progress.md#effect-integration-2026-10-24)。

## 执行与结果

| 验证 | 命令 / 路径 | 结果与证据 |
|---|---|---|
| 完整 preflight | 仓库根目录 `python3 -u -B scripts/preflight.py` | **exit 0，20 passed / 0 failed / 0 expected failure / 0 skipped**；[原始输出](preflight-full.log)、[起讫与逐检查摘要](preflight-full.json) |
| 无生成包目录的完整构建 | 仓库根目录 `pnpm --dir apps/web-client build:desktop` | **exit 0**，依次原生 WASM、旧线 WASM、`vue-tsc --noEmit` 与 Vite 生产构建；[完整输出](build-desktop-clean.log)、[命令/起讫](build.json) |
| 前端完整回归 | web-client 内 `/usr/local/bin/node --import ./scripts/ts-register.mjs --test` | **61 passed / 0 failed / 0 skipped**；[输出](frontend-tests.log)、[起讫](web-tests.json) |
| 真实 WASM / Worker | web-client 内 `/usr/local/bin/node --import ./scripts/ts-register.mjs scripts/native-worker-smoke.mjs` | **44 checks / 0 failed**；[输出](native-worker-smoke.log)、[命令及实际加载包的摘要](worker.json) |
| 扩展存储冒烟 | web-client 内 `/usr/local/bin/node scripts/extension-store-smoke.mjs` | **9 passed / 0 failed**；[输出](extension-store-smoke.log)。这 9 项已包含于前端 61 项，不相加为 70 |
| 新生产 dist 浏览器 | 新 preview `http://127.0.0.1:5481`，执行既有浏览器脚本的精确临时副本 | **7 checks / 0 failed，pageerror 0**；[输出](browser-production-smoke.log)、[完整元数据](browser.json)、[截图](browser-formal-match.png) |

preflight 于 **13:35:37–13:42:09（UTC+8）**完整执行 20 项，包含此前快速检查跳过的台账↔枚举一致及脚本全量单测。preflight 只透传各检查的通过摘要，不透传子命令逐测试输出；另用 unittest discovery 确认当前为 **180** 项，标为发现数，没有冒称存在本次逐项测试日志。此前拆分首轮的 180 项原始输出仍在[原证据](../2026-10-24-effect-split/python-tests-final.log)。

构建前已备份并移除 `src/worker/native-pkg`、`src/wasm-pkg`、`public/pkg` 三个生成包目录，**保留 Cargo 缓存**，不称为无缓存构建。使用 Node **20.17.0**、pnpm **10.34.3**、rustup Cargo/rustc **1.95.0**、wasm-pack **0.15.0**，显式优先 rustup 与 Node 20 的 PATH；Cargo 离线执行。pnpm 主版本与 CI 配置的 9 不同，本轮没有运行远端 CI。见[环境记录](environment.json)。

三个新生成包目录合计 **18 个文件**，与开始时备份的对应文件逐一 SHA-256 一致，包括两条 WASM 二进制及绑定文件；这不是只复用旧生成目录。完整构建输出包含重新编译 `rule-api` 及原生消费者，生成目录从空重建。见[重建前后对照](artifact-comparison.json)与[新产物清单](validated-artifacts.json)。新 dist 在测试前后未变，其中两条 WASM 文件的摘要与本次新产物清单相符；浏览器已验证原生 Worker 流程，没有追踪或断言旧线 WASM 也实际被加载。

## 正式扩展开局覆盖

Worker 脚本加载本次新原生 WASM，验证实际 cards/characters/pile 进入正式消息入口、扩展武将候选与授技、原局保存、缺包/错版本/同 manifest 换资产拒绝、同内容锁恢复、身份局主公体力加成与全局授技保留。Node 中执行正式 Worker 的 `onmessage`，以 `self` 桩提供宿主；生产浏览器验证进一步覆盖真实浏览器 Worker 加载。44 项是检查点，不是 44 个来源或 44 个整包验收。

Chrome **156.0.8078.48** 的新生产页面依次验证：当前客户端可发现的 **15 包**保存实际副本 → 正式原型开局并选择 `ddd刘冲` → 存档含冻结载荷 → 卸载后按原内容锁恢复并再次保存 → 身份局载入扩展 → 缺载荷时明确报错且解除冻结 → 无未捕获页面异常。截图只记录其中一个流程时点，不单独代替这些断言。pageerror 0 仅指页面未捕获异常，不代表全面网络或 console 审查。

浏览器原脚本为[前次交付脚本](../2026-10-24-native-extension-delivery/browser-smoke.mjs)，精确复制到 `/tmp/noname-next-effect-validation-20261024/browser/` 后执行，避免覆写前次截图。原文件与副本 SHA-256 一致；本次 preview 和浏览器均已关闭。复现命令和 Chrome/Node 版本、45 个实际 dist 文件摘要保存在 `browser.json`。

## 保留的首次失败

前端首跑与清空生成目录后的 WASM 构建并行启动，过早导入尚未产出的 `engine_native_wasm.js`，结果 **37 passed / 3 failed**（3 个测试文件因模块缺失未能加载，不是执行完全部 61 项）。见[首次前端日志](frontend-tests-initial.log)与[首次命令/退出码](web-tests-initial.json)。原生生成包部署后，对同一源码完整重跑 61 项通过；真实 WASM/Worker 44 项随后使用新包通过。没有用降低断言或改期望值消除失败。

## 修改保护

开始时保存 binary diff、index diff、HEAD 和 **11166** 个既有 dirty/untracked 路径的内容/模式/符号链接属性，并完整复制原三个生成包目录与原 dirty dist 到 `/tmp/noname-next-effect-validation-20261024/original-generated/`。新构建及浏览器验证结束后，先存新产物到 `validated-generated/`，再逐目录校验并恢复原文件。更新进度前 **11166 个路径全部与开始时一致**，见[恢复记录](restoration.json)。

最终只有进度主稿添加本次明确增量，其他 **11165** 个既有路径原样保留；新增本证据目录不覆盖旧证据。**230 个源码/配置/包数据文件摘要**与实际受验版本一致，其中前次拆分清单的 35 项全部一致，见[源码清单](source-manifest.json)与[最终保护核对](preservation.json)。HEAD/index 保持不变，未暂存、未提交、未推送。

## 边界

本轮补跑完整 preflight、前端/Worker/Chrome 与真实 WASM 生产构建；未重复前轮 2165 项 Cargo 回归、6 项旧线 golden 或忽略测试。没有运行 GitHub Actions、三桌面安装包、Tauri 运行期、移动真机、Firefox/Safari、真实跨机/TLS/WS 房间或完整旧线测试。相同 WASM 文件及绿色回归不证明全部上游规则等价、37 来源整包验收或性能收益。
