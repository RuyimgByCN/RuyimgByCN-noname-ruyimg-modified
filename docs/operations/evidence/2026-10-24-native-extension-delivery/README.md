# 正式扩展开局、原生 WASM 与 CI 接线（2026-10-24）

核验基准：`7e28a93799a9c01794085e911a72a4faa5673883` 加本轮工作树；规则 v322、存档格式 v105。本轮不提交。工具链和产物指纹见[环境记录](environment.json)，修改后的源码/配置指纹见[源码清单](source-manifest.json)。当前结论与后续优先级以[进度主稿](../../../architecture/native-progress.md#delivery-2026-10-24)为准；本目录保留命令、原始输出和产品流程证据。

## 实现范围

- 安装复制实际 manifest 与 cards/characters/pile；开局只读按版本保存的本地副本。旧的纯元数据安装明确提示升级/重新安装；同版本安装不覆盖副本，禁止虚构升级版本。
- `BattleView` 将启用包传入正式 Worker；原型局和身份局均支持扩展。Rust 复用现有卡牌、武将、牌堆 loader，不把元数据展示当作入局证据。
- 存档携带原局副本，停用/卸载后仍按原锁恢复；撤销独立留档，卸载不能绕过。缺包、错版本、逻辑指纹不符，以及同 manifest 换资产均拒绝；Rust 内容锁包含完整载荷的 BLAKE3 摘要。旧的无扩展标准存档保持兼容，旧扩展存档没有实际内容副本时明确拒绝。
- 正式选将保留身份主公体力加成（Infinity 哨兵除外）、全局初始化及装备仍提供的同名技能；替换武将时移除仅来自旧将的技能。全局授技清单从启用包派生，恢复时重建，不新增存档字段。
- `build:desktop` 顺序为原生 WASM → 旧线 WASM → typecheck/生产构建，失败短路；修复 Windows PATH/pnpm 调用和 Tauri hook 工作目录。旧线 WASM 仍被客户端导入，因此保留构建。
- 新增独立 `native-web` CI：重建两条 WASM → 原生内核/窄壳/loader 回归 → 真实 Worker → 前端回归/构建；不依赖已知红的行数门禁 job。桌面 CI 补装依赖、wasm-pack，修正安装包上传路径。GitHub Actions 尚未实际运行。

## 本地执行与原始输出

| 验证 | 命令 | 结果 / 日志 |
|---|---|---|
| 无生成包目录的完整构建 | 移出 `src/worker/native-pkg`、`src/wasm-pkg`、`public/pkg` 后 `pnpm --dir apps/web-client build:desktop`，保留 Cargo 缓存 | exit 0；[完整日志](build-clean.log) |
| 原生相关完整回归 | `/Users/ruyimg/.cargo/bin/cargo +stable test --offline -p engine-native -p engine-native-wasm -p content-native-loader --no-fail-fast` | exit 0：原生 1547/0/71 ignored，窄壳 36/0/0，loader 32/0/0；[分项统计](test-totals.json)、[完整日志](native-full.log) |
| CI 同主版本前端测试 | web-client 内 `/usr/local/bin/node --import ./scripts/ts-register.mjs --test`（Node 20.17.0） | 61 passed / 0 failed；[日志](web-tests-final-node20.log) |
| CI 同主版本真实 WASM / Worker | web-client 内 `/usr/local/bin/node --import ./scripts/ts-register.mjs scripts/native-worker-smoke.mjs` | 44 passed / 0 failed；[日志](native-worker-smoke-final-node20.log) |
| 生产页面浏览器流程 | 见下方重跑方式 | 7 项检查通过，pageerror 0；[日志](browser-production-smoke.log)、[截图](browser-formal-match.png) |
| 扩展存储冒烟 | `pnpm --dir apps/web-client smoke:extensions`（直接运行生产模块用例，不复制实现） | 9 passed / 0 failed；[日志](extension-smoke.log) |
| CI 配置静态核验 | Ruby Psych 解析 YAML，并检查独立 job、命令顺序、用户原门禁、Tauri cwd、产物路径 | 通过；[日志](ci-config.log) |
| 本地链接 | `python3 -B scripts/check_local_links.py` + 本证据页单独检查 | 7 份主稿和本证据页均无坏链；[日志](local-links.log) |
| 行数脚本单测 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | 1 passed；[日志](source-line-tests.log) |
| 行数门禁 | `python3 -B scripts/check-source-lines.py` | exit 1：仅既有 `crates/rule-api/src/effect.rs` 10778 行；[日志](source-line-gate.log) |

前端 61 项包含构建链的 8 项回归与安装副本/内容锁的 9 项回归，这些计数不相加为 78。Worker 冒烟包含直接窄壳、适配层和真实 Worker 消息入口，44 项不表示 44 个独立来源。浏览器使用本机 Chrome headless、1440×1000 视口，访问本轮新建 Vite preview 的生产 dist；不是 Tauri 或移动真机证据。

浏览器检查：管理页安装实际包副本 → 正式本地开局/选择 `ddd刘冲` → 存档包含完整内容 → 卸载后恢复并再次保存原局锁 → 身份局载入同批扩展 → 旧安装缺载荷时明确报错/解冻 → 无未捕获页面异常。可用既有 Playwright 安装重跑（`PLAYWRIGHT_MODULE` 可填其绝对模块路径，浏览器默认 Chrome）：

```bash
pnpm --dir apps/web-client exec vite preview --host 127.0.0.1 --port 5474 --strictPort
NONAME_WEB_ORIGIN=http://127.0.0.1:5474 PLAYWRIGHT_MODULE=/path/to/playwright/index.mjs \
  node docs/operations/evidence/2026-10-24-native-extension-delivery/browser-smoke.mjs
```

本次已验证的 dist 另存 `/tmp/noname-next-native-delivery-20261024/validated-dist`，随后恢复用户原有 dirty dist。重跑 preview 前需先运行 `build:desktop`；复现后也应保留自己原有的生成文件。

## 全量首轮中断与修复

首轮在 `nc10_pool_full_degrades_without_losing_plans` 等待超过 60 秒，累计约 2.5 分钟后终止该测试进程；Cargo `--no-fail-fast` 继续执行其余目标，最终 exit 101。原始输出见[首轮日志](native-full-initial-hang.log)。这不是全量通过，也不以单项补跑覆盖该失败。

根因是既有测试把「首个任务提交」当成「队列仍满」：worker 若已取走首个任务，第二次提交可能成功，断言 panic 后又在池析构等待尚未释放的 Barrier。只修改测试：用 started/release channels 握手，先阻塞唯一 worker、再填满唯一队列槽；明确验证第三次提交返回 PoolFull，并在仍满载时要求 `plan_all` 串行补齐且 `degraded=true`。发送端析构可解除 worker 等待，断言失败也能清理。没有修改工作池生产代码。

修复后的该项[连续 10 次通过](nc10-pool-repeat.log)，随后完整重跑相同三个 crate 的所有目标，最终 exit 0（内核 1547/0/71、窄壳 36/0/0、loader 32/0/0）。它与此前 `HlssGaiming` 等历史偶发记录不是同一问题，不宣称已定位全部历史失败。

## 未提交修改保护

开始前以二进制 diff 和 SHA-256 清单备份既有 dirty/untracked 文件；同时复制 dist 与生成包目录到 `/tmp/noname-next-native-delivery-20261024/`。构建验证后恢复原 dist。复核 11050 个既有 dirty/untracked 文件，只有本轮明确接线的 `.github/workflows/ci.yml` 内容变化，其中用户原有四行行数门禁完整保留。详见[保留核对](preservation.json)。本轮未暂存、未提交、未推送。

## 验收边界

仅覆盖当前客户端可发现的转换包，未扩大到磁盘全部转换包或全部来源。含 VM 模块的安装包明确拒绝，第三方 SDK/rule-vm 完整装载仍待办；未改变独立扩展的完整需求。此切片不代表任何来源八项全部验收，不证明上游规则等价。

未运行 GitHub Actions、三桌面平台安装包、Tauri 运行期、移动真机、Firefox/Safari、真实跨机/TLS/WS 房间。全仓 CI 仍有既有行数门禁失败；不能称整个 CI 已绿。原生全量中的 ignored 仍属未验收守卫，历史偶发失败也不能因本次单次绿而宣称已定位。
