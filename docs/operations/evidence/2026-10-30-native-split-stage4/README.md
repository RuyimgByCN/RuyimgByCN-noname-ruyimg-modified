# F7 第 4 步：三个真实 crate（2026-10-30）

本批基准为干净 HEAD `db4e37ca608877111d9d6f5e12290e6ec912da41`，本地日期 2026-10-30（UTC+08:00）。[起点证明](clean-start.json)先写到仓外，再创建本证据目录。规则 v328、存档105、原生 ABI753保持不变；实施者交付时没有提交或改变 HEAD，主线程另行审查、验收和入库。当前进度及剩余台账的状态正文留给主线程，本批只修源码定位。

## 实现边界

- 214 个 Rust 文件经 `git mv` 迁移，[逐文件映射](move-manifest.json)、[搬移后 hash](moved-source-conservation.json)、[源码链接映射](source-reference-map.json)。kernel 采用分层脚本实际的15个命名模块入口（`vocabulary` 与三个 `rvm_*` 均在内），content 保留 `src/content/mod.rs`；facade 保留 engine/view/presentation/ai/ai_pool、原模块与根类型重导出。
- 唯一定义随模块移动；HostBehavior 的 inherent impl 全在 vocabulary，HostSkillBehavior 与内容机器留 content。三根都保留 `deny(unreachable_patterns)`，没有复制执行代码或让下层开发依赖 facade。
- 仅开放准备清单已证实的函数、四类机器构造函数、三个 ask helper、12个恢复字段；编译没有要求额外开放。facade 的 rule-api/rule-vm 直接依赖移至现有集成测试开发依赖。workspace、锁文件与 CI 的 Gate 1b/原生回归覆盖三包。
- 15个 Python 消费者、源码读取测试、维护守卫与 include_str 路径迁移；[实际输入/输出守恒](scanner-source-conservation.json)。所有消费者输出经过路径映射相同，没有移除输入；全 crates 卡面扫描新增两个 crate 根文件。初始取证 harness 的失败与修复基线分开保留，见[失败清单](failure-inventory.json)。
- 分层脚本 BASELINE0 扫实际 kernel 根、根文件及全部必扫模块，缺失或空入口拒绝；外部 content/facade 路径（raw/group/alias）拒绝，并用离线 Cargo metadata 校验正常依赖方向。[红灯](layering-red.log)为2失败/1错误；[绿灯](layering-green.log)7项通过；[主树门禁](layering-main.log)0/0。
- [65份 Markdown 源路径修正清单](doc-source-path-files.json)保留历史语境，未改旧原始日志/补丁/txt/json。仓外 NONAME_NEXT_AGENT_SPEC.md 的旧源码链接在主线程后续接线范围，本批映射可直接使用。

## 核验与结果

[命令、退出码、时间与执行目录](commands.jsonl)保存各项运行；[失败清单](failure-inventory.json)包含取证 harness、预期红灯、工具超时与过宽文件名检查的首次失败。没有用补跑覆盖失败。

- [三包 all-target check](check-final.log)通过；[全量三包测试](native-tests.log) **1644 passed / 0 failed / 73 ignored**。其中 facade 单元10、content单元4、kernel单元3；facade测试合计1637通过，content4、kernel3。忽略项仍是既有登记项。
- [测试名多重集合](test-name-conservation.json) **1717→1717**，逐名和重复次数守恒；[前清单](test-list-before.log)、[后清单](test-list-after.log)。
- [依赖测试](dependent-tests.log) wasm/loader/game-server **465/0/0**；[Gate 1b](gate1b.log)独立 `RUSTFLAGS=-D warnings` 三包 all-targets 通过。依赖全测仍输出旧 engine-ai/engine-core 的既有 warning，不把它们算作 Gate 1b 的零警告范围。
- [2000总行/1000有效 Rust行门禁](source-lines-final.log)、[行数脚本自检](source-lines-unit.log)、[有效行自检](effective-lines-unit.log)通过；[文档行号门禁](doc-line-refs.log)通过。[完整 preflight](preflight-initial.log) **21/21** 通过。
- [golden/版本保存证明](golden-preservation.json)核对 replay_scenarios 源码与 HEAD 逐字相同、非 Markdown golden/version 文件集合 hash 不变，实际常量328/105/753。宽泛文件名集合中两份 Markdown 仅修源码路径，首次不加区分的断言失败另有登记。

## 构建测量

[设备、工具链与环境](environment.json)：Apple M3 Max，macOS27.2，rustc/cargo1.95.0，8 jobs，`CARGO_INCREMENTAL=1`，默认test/dev profile与 `debug=line-tables-only`。拆前与拆后使用隔离源码及空专用 target，拆前只测 engine-native，拆后测全部三包以覆盖相同测试面。两轮都直接冷跑 `cargo test --offline … --no-run`，没有先build库预热。

| 同口径测量（秒） | 拆前 | 拆后 |
|---|---:|---:|
| 冷测试构建 real | 28.68 | 25.24 |
| 内容函数体编辑后温构建 real | 4.48 | 2.79 |

[序列化重测数值](performance-serialized-summary.json)、[拆前冷](before-cold-serialized-1.log)、[拆后冷](after-cold-serialized-1.log)、[拆前温](before-warm-serialized-1.log)、[拆后温](after-warm-serialized-1.log)。四条命令均经 `/tmp/noname-run-serialized.py` 共用文件锁，`locked_seconds` 排除锁等待；[进程监测](performance-process-monitor.json)先等其他 Cargo/Clippy 结束，并在四个测量窗口每250ms采样，没有观察到其他 Cargo/rustc/Clippy。最初未加锁/未监测的[探索数值](performance.json)及原始日志保留，不能作为受控结论。编辑均在 standard_catalog 函数体将 cards 改为可变并执行 `shrink_to_fit()`，属于真实函数体修改；[拆前恢复](before-edit.json)、[拆后恢复](after-edit.json)均逐字恢复。拆后温构建仅重新编译 content/facade，kernel未重新编译。每个条件仍只有单次重测，不宣称稳定百分比收益。

[磁盘记录](disk-final.json)：起点52GiB可用、拆后冷构建时42GiB，完整依赖/Gate验证后约33GiB；target-before最终约2.3GiB，target-after复用于主树检查后约7.0GiB。初始检查 cache 是本轮私有目录，在拆后冷构建前[登记后删除](initial-check-cache-reclaimed.json)。收到主线程串行测量指示后，仅将这两个既有私有 target [登记并回收](performance-cache-reclaims.jsonl)，再做空 target 重测；重测后各约2.3/2.2GiB，最新磁盘余量见序列化摘要。没有清理共享 target，也没有第三个整套构建 cache。

按主线程 live steer 临时配置 `.pi-lens.json` 的三个禁止自动修改字段，安装版 loader 的[实际解析结果](temporary-tool-config-resolution.json)均为false；源码指纹复核未发现自动格式化/修复，临时配置已在交付前删除，未改全局配置。严格门禁保持开启。`git mv` 的[暂存重命名记录](git-mv-staged-status.log)保存后用 `git restore --staged -- .` 清空 index，[交付状态](index-status.json)无暂存文件，工作树改动保留由主线程审查并暂存。

以上记录截至实施者交付；下列为主线程追加，原始失败与审查意见均保留。

## 主线程审查与整合验收

- [独立审查原文](independent-review.txt)指出两项：extern-crate 外部别名漏检、维护守卫总数阈值被移除。前者先以真实声明/别名/raw/Unicode夹具红灯复现，再直接识别被禁止的外部 crate 声明；后者恢复 `files.len() > 10`，保留三根存在且非空的新增检查。[声明红灯](review-extern-crate-red-final.log)、[Unicode 红灯](review-extern-unicode-red.log)。
- 主线程额外发现新顶层 kernel 模块仍可能漏扫，先取得[0而非2的红灯](review-new-modules-red.log)，再改为校验必需入口后递归扫描整个 kernel src；新顶层/嵌套模块无需另登记也会计入。旧排除夹具同步放到真实 content/facade 根，不豁免 kernel 内伪装的同名目录；第一次夹具失败[另存](review-new-modules-green-first.log)。[最终套件9/0](review-layering-final-green.log)，[实际0/BASELINE0](review-final-layering-main.log)。另补 Python 可变长度依赖链的类型标注，修正主动类型诊断。
- [独立搬移核对](parent-move-verification.json)从 `db4e37ca` 逐 blob 重算214个旧文件 hash，并与新源文件比较，203逐字相同，11仅可见性/导出改动；[实际差异](parent-moved-source-diffs.patch)可逐项审查。仓外 AGENTS 与 Agent Spec 的三包测试命令、职责导航及4个源码链接也已同步。
- [主线程命令及源码指纹](parent-validation.json)：维护守卫定向通过；workspace **6729/0/74**、conformance ALL PASS、当前源码原生WASM重建、前端 **127/0** 与typecheck、真实Worker **81/0**、native/WASM标准及huanle轨迹、旧线 `cross-target-hash.sh` 全部exit0。每条完整日志以 `parent-` 开头保存；首轮/阶段失败没有被覆盖。
- [生产构建与真实Chrome结果](parent-browser-results.json)：正式扩展页两种损坏包拒绝且不写存储、合法包导入及未签名标记，共4项；[导入截图](browser-import.png)。新增[真实产品驱动](browser-native-product.mjs)仅旁路记录真实Worker回包，没有替换引擎或响应；经界面开局、存档、推进、读档后hash/revision逐字一致，原型319步与身份280步都得到权威 `finished=true` 和真实outcome，全程校验他人手牌/牌堆不泄露、console异常0。[终局记录](native-product-report.json)、[截图](native-product.png)。此功能测试显式跳过动画，不是PA08帧延迟、功耗或真机数据。
- 原型样本同时暴露既有NC03编号缺口：本地座位0..3，胜者 `SingleWin.player=4` 来自1基协议映射，客户端尚未规范化。该 helper 未被F7改变；已列回主稿NC03，不把上述终局存在性检查冒充完整胜者UI验收。NC09、PA09、37来源和上游等价仍分别待验。
- 最后一次审查后完整preflight **21/21** 通过，见[最终记录](final-integration-checks.json)。严格Gate1b首轮被工具自动修改的无关 hash 代码新增括号 warning 阻断（[原失败](parent-final-gate1b.log)）；先保存[自动差异](automatic-unrelated-late-hash.diff)，再精确恢复该文件到原HEAD blob，临时禁止自动修改后[补跑三包Gate1b通过](parent-final-gate1b-corrected.json)。没有借此改规则或hash编码，没有把首轮标为通过。
- 检查后另检测到无关 facade 自动建议改动，保存[完整差异](automatic-unrelated-late-facade.diff)，逐路径确认不属于F7清单后恢复原HEAD字节；生产构建覆盖的已跟踪dist也先在仓外保存后恢复原产物，避免机械拆分批夹入发布产物。[保护记录](post-verification-artifact-preservation.json)。源码恢复没有删掉F7改动；临时工具配置移除后再提交，后续agent继续显式验证，不依赖自动修复。
- 真实设备与GitHub Actions不在本批本地结果内，整体收敛仍未完成。
