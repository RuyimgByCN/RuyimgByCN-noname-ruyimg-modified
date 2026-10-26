# 服务端房间固定原局包副本（2026-10-24）

核验基准 `7e28a93799a9c01794085e911a72a4faa5673883` 加当前未提交工作树；规则 **v322**、原生存档 **v105**，本轮没有修改内核、WASM 或归档格式版本。当前状态以[进度主稿](../../../architecture/native-progress.md)为准。此前服务端要求原包保持激活的限制，已在本轮改为按房间内的原始副本恢复。

## 实现与兼容

- 新房间在发布 actor 前，固定精确安装身份、原始 `package.npack.json` 字节及全部声明资产的 bytes；JSON 包含原有 VM 模块，资产不限于卡牌。`NativeRoomState.frozen_packages` 随每次保存与恢复保留；标准无扩展房间明确保存空副本集合。
- 恢复独立核验原始 JSON 内容 hash、解析后的 logic/assets hash、namespace/id/完整版本、引擎兼容性、全部资产字节和包引用顺序。不依赖当前 active 集合或原安装目录，停用、升级、同版本重新发布或删除原安装副本后，旧局仍使用原内容。新局继续按当前激活内容装载。
- 恢复仍执行当前内容 hash/包身份撤销政策；副本被改、缺失、绑定错误或撤销时拒绝，不用当前 active 包替换。原保存文件原样保留，错误房间不发布。
- 字段缺省为 `None`，旧序列化/checksum 保持相容。旧档带完整精确安装身份且原版本仍可验证时，可补齐副本并先原子保存迁移结果，再发布房间；迁移写入失败不发布。只有短引用、原包缺失且从未保存副本的旧档仍拒绝，不猜包恢复。
- 副本校验与加载复用同一桥接路径，保留原有卡牌、武将、牌堆及 VM 装载行为；捕获资产时检查规范化路径包含关系，拒绝越界符号链接。没有新增依赖。

## 已完成的验证

| 验证 | 命令 / 范围 | 结果与证据 |
|---|---|---|
| 服务端定向 | `cargo +stable test --offline -p game-server --test native_frozen_packages --test native_product_path --test nc02_install_identity --test nc09_native_actor --test nc09_native_room_session` | **45 passed / 0 failed / 0 ignored**；[日志](backend-targeted-final.log) |
| 真 TCP/WS 重启 | `cargo +stable test --offline -p game-server --bin game-server native_ws_e2e -- --nocapture` | **2 passed / 0 failed / 16 filtered**；[最终日志](ws-frozen-green-final.log)，含下面的正例及 8 类负例 |
| 七 crate 完整回归 | `cargo +stable test --offline -p game-server -p engine-native -p engine-native-wasm -p content-native-loader -p content-runtime -p engine-protocol -p rule-api --no-fail-fast` | **1782 passed / 0 failed / 71 ignored，exit 0**；[原始输出](cargo-regressions.log)、[起讫](cargo-regressions.json)、[逐 target 汇总](cargo-counts.json) |
| 实际服务程序构建 | `cargo +stable build --offline -p game-server` | **exit 0**；[日志](server-build.log)、[起讫](server-build.json) |
| 前端完整回归 | web-client 内 Node 20 `pnpm test` | **70 passed / 0 failed / 0 skipped**；[原始输出](frontend-test.log)、[起讫](frontend-test.json) |
| 前端型别 | web-client 内 `pnpm typecheck` | **exit 0**；[日志](frontend-typecheck.log)、[起讫](frontend-typecheck.json) |
| 原生 WASM / Worker | web-client 内 `/usr/local/bin/node --import ./scripts/ts-register.mjs scripts/native-worker-smoke.mjs` | **44 checks / 0 failed**；[日志](native-worker-smoke.log)、[起讫](native-worker-smoke.json) |
| Chrome 真实程序重启 | [浏览器脚本](browser-frozen-room-smoke.mjs) + 当前新构建服务程序 + 前轮已验证生产 dist 的外部副本 | **8 组通过 / 0 失败，pageerror 0**；[原始输出](browser-frozen-room.log)、[起讫与 dist 路径](browser-frozen-room.json)、[报告](browser-report.json)、[恢复续局截图](browser-ws-restored.png) |
| 行数门禁与单测 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py` | **单测 1/1、8567 文件 ≤2000 行，exit 0**；[单测](source-lines-unit.log)、[门禁](source-lines.log)、[起讫](source-line-checks.json) |
| 完整 preflight | `python3 -u -B scripts/preflight.py` 完整模式 | **20/20 通过，0 失败 / 0 跳过 / 0 豁免，exit 0**；[摘要](preflight-full-01.log)、[全部子命令原始输出](preflight-full-01-raw.log)、[逐项结果及起讫](preflight-full-01.json) |

完整 Cargo 回归于 **14:54:58–14:58:13 +08:00** 执行：`engine-native` **1547/0/71**、`game-server` **131/0/0**、`engine-native-wasm` **36/0/0**、`content-native-loader` **32/0/0**、`content-runtime` **33/0/0**、`engine-protocol` **1/0/0**、`rule-api` **2/0/0**（passed/failed/ignored）。计数包含既有 `#[path]` 重复执行，不代表独立场景数；71 个忽略守卫没有被当作通过。日志保留既有警告，不声称零警告。

新增 WS 生命周期测试先在尚未接线的产品代码运行，恢复房间数 **0、预期 1** 而失败（[先红日志](ws-frozen-red.log)）；接线后定向和完整回归均通过。不是更改黄金哈希或期望值来消除失败。工具链为 rustup Cargo/rustc **1.95.0**、Node **20.17.0**、pnpm **10.34.3**；完整 preflight 使用默认 Python **3.10.10**，前缀构建 PATH 中另有 Python 3.12.5，见[环境记录](environment.json)。

## 真实 WS 与程序重启覆盖

Rust binary tests 使用真正的 loopback `TcpListener`、生产 `bootstrap/build_router` 与 `tokio_tungstenite`，正式安装 `upstream-noname/shahai-swd` 和 `upstream-noname/wudian`。双玩家与旁观者推进到私人弃牌选择后落盘，销毁旧 Tokio runtime，然后依次以全新 runtime/bootstrap 验证：

1. 停用两包、清空导入源并移除原安装目录，按房间副本恢复。
2. 激活同版本 `1.0.0` 的不同资产内容，旧局维持原引用、完整个人投影/状态摘要/选择/合法动作；新局使用重新发布的内容。
3. 激活 `1.1.0` 升级内容，旧局仍按原锁恢复；新局与新版 reference engine 摘要一致，并与旧内容不同。
4. 重复入房与落盘后再重启，原令牌、座位和副本继续保持；经真实 `ResolveChoice` 推进至 revision **4**，玩家/旁观者同步且新快照落盘。

8 类负例在修改内层副本后重算外层 room checksum，确保测到的是内容校验，而非仅外层 checksum 拦截。前五类还保留有效 active 原包并可创建新局，验证错误副本不会回退到 active。所有失败均保留原文件、不发布错误房间。详见当前 `native_ws_e2e` 测试和上述原始输出。

Chrome **14:58:39–14:58:44 +08:00** 使用三个独立 context，沿生产大厅按钮创建密码房间、加入、旁观、发送真实 WS 命令及页面 reload。停在私人选择屏障时对实际 `game-server` 子进程执行 **SIGTERM**，清空两个包的导入源与 active 集合并删除 installed packages，重新启动程序；三端自动以原角色/座位/玩家令牌回房，完整投影/choice/动作保持。点击渲染的选项后 revision **3→4**，三端摘要一致，磁盘快照改变且副本与引用逐项保持。8 项是断言组，不是来源验收数。独立临时数据目录、初始及重启日志见[报告](browser-report.json)、[初始服务日志](browser-server-initial.log)、[恢复服务日志](browser-server-restored.log)。

```bash
PATH=/Users/ruyimg/.cargo/bin:/usr/local/bin:$PATH cargo +stable build --offline -p game-server
NONAME_WEB_DIST=/absolute/path/to/validated/production/dist \
PLAYWRIGHT_MODULE=/absolute/path/to/playwright/index.mjs \
/usr/local/bin/node docs/operations/evidence/2026-10-24-native-room-frozen-packages/browser-frozen-room-smoke.mjs
```

浏览器脚本默认仅监听 `127.0.0.1:5483` 和 `:5484`，可通过 `NONAME_WEB_PORT` / `NONAME_WS_PORT` 覆盖。它复制两个正式包到独立临时目录，关闭自建浏览器/静态服务/游戏程序，不操作用户真实存档。当前前端源码及三个 WASM 生成包未改，本轮没有重新构建 WASM 或覆盖原 dirty dist；Worker 使用已验证的既有产物。

## 完整门禁与修改保护

完整 preflight 于 **14:58:06–15:04:31 +08:00** 运行未修改的 `preflight.main()` 完整模式，只在进程内透明记录 `subprocess.run` 的原 stdout/stderr，不改变 argv、返回值或检查项目。**20 项全部执行并通过，0 失败、0 跳过、0 豁免，exit 0**；脚本单测实际 **180 tests / OK**，全仓行数 **8567 文件 ≤2000 行**。原始输出保留现有 ResourceWarning，不把警告当成失败或宣称零警告。本次完整命令首跑通过；历史轮次失败仍按各自日期保留。

本页和进度主稿补写后，再运行全量本地链接、严格文档行号、旧值检查、最近轮次索引、行数门禁及 `git diff --check`，见[最终文档检查](final-doc-gates.log)。

开始备份 HEAD、index/worktree binary diff，并记录 **46858 个既有路径**摘要，初始 dirty/untracked status **11269 条**；备份位于 `/tmp/noname-next-room-frozen-packages-20261024/`。仅本轮 **8 个既有服务端/测试/进度文件**作增量修改，其余 **46850 个路径**内容/模式一致；新增范围为冻结包模块、定向测试及本证据目录，意外变更 **0**。本轮服务端实际增量见[差异](server-incremental.diff)，避免把既有混合改动算作本轮新增。

本轮基准清单不含 ignored 生成包，dirty dist 也只收录 Git 可见部分；因此另与前轮已恢复的完整实体备份 `/tmp/noname-next-ws-delivery-20261024/generated-backup/` 核对三个生成目录 **18 文件**及原 dirty dist **46 文件**，全部内容/模式一致、无增删。本轮没有运行会重写它们的构建命令。受验源码、测试、配置与两正式包的 **1054 文件** SHA-256、bytes、物理行数见[来源摘要](source-manifest.json)，最终再次核对摘要未变；完整保护记录见[修改保护](preservation.json)。**HEAD/index 不变，未暂存、未提交、未推送**。

## 边界与后续

- 新增字段针对服务端房间保存胶囊；没有为独立 HTTP `NativeMatchArchive` 增加冻结载荷。无法找回“从未保存副本且原版本已丢失”的旧档内容。
- hash 校验用于内容完整性，不是保存文件数字签名。撤销仍优先于旧局可恢复性。已有原子替换不等于 fsync/断电耐久保证；运行期保存失败仍沿用警告处理。
- 本轮验证本机真实 WS 和 Chrome，没有验证跨机/TLS/代理、Firefox/Safari、真机、桌面安装包或远端 Actions。没有声称完整表现、联机选将/明将、长程整局、性能提升、37 来源 G0–G5 或上游规则等价完成；截图中现有牌名/资源及武将占位仍属于表现待办。
