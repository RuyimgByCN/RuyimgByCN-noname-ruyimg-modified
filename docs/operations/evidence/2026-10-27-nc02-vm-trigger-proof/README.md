# NC02：VM 武将技能归属与实际触发复核

- 实际执行：2026-10-27 22:12:12 +0800 → 2026-10-27 22:23:43 +0800（开始和结束各运行 `date`）；开工源码 HEAD `f756eaf7b0d9f1b5d68877073f57f7eb824a6586`。规则 v322、存档 v105、rule-vm ABI 均未升号；没有重录 golden。
- 开工时另有 8 个已修改文档，本批保留其内容；本批只回填 NC02/NC09 相关行。

## 反例与修复

在 `git archive HEAD` 的干净导出上用独立 `CARGO_TARGET_DIR`，旧 Worker 测试原样 **1/0**，追加“选中武将实际持有编译器输出的模块名”断言后 **0/1，exit 101**（[原测试](clean-original.log)、[反例](clean-counterexample.log)、[状态](clean-repro-status.txt)）。旧夹具声明的技能名与编译器生成的 `worker_phase_end_phase_end` 不同，而旧测试只检查手牌增加，普通摸牌阶段即可满足断言。服务端集成测试使用正确技能名，但也只检查手牌增长。2026-10-25 浏览器 E2E 的生成器同样写了未加事件后缀的技能名：原浏览器证据支持当时的安装/管理流程，不证明 VM 技能实际执行。

`bridge_generals_with_known` 对含 VM 模块的包拒绝未映射的武将技能，并在错误中列出名称；不含模块的数据包保留既有延后映射行为。服务端冻结包装载与 WASM Worker 两个调用方同步；空触发索引但有模块也不再静默丢失技能。生成器改为引用真实 `trigger_index` 模块名，并提交从编译器生成的 Worker 夹具。定向回归检查技能归属、结束阶段特有效果、同一包快照恢复后的重放和最终状态哈希；真实 Worker 冒烟检查了同样的路径和错误引用拒绝。

此变更不修改已成功映射技能的规则或状态格式；此前被静默丢弃的 VM 武将技能现在会阻止对应含模块包的加载，包括从冻结副本加载。依赖其他包才能解析的 VM 技能目前也会明确拒绝；跨包解析尚未实现，不能将拒绝误述为兼容。旧的只含数据的包保持原来的未映射计数与装载行为。

## 验证过程与边界

首次加强的恢复测试出现失败：在干净 HEAD 导出只叠加测试修改后也失败（[记录](clean-recovery-counterexample.log)）；进一步跟踪显示是测试驱动在选择回答时使用小于前一个命令的 dedup ID，导致 `DuplicateCommand`（[跟踪](clean-recovery-trace.log)、[状态](clean-recovery-status.txt)），不作为产品故障。修正测试驱动使用递增 ID 并断言命令未拒绝后重跑。夹具首次生成成功后的 shell 包装误用了 zsh 只读变量 `status`（[记录](fixture-generation-shell-error.log)），改用专用变量重跑成功（[日志](fixture-generation-02.log)）。

完整工作区检查使用临时隔离 index 构造的验证提交 `fdc1b525ee6a9479aae1a76a52e8a3223902603f`（父提交为上述 HEAD，只包含本批源码、测试和夹具），在独立检出中运行。首次检查时依赖链接/编译目标曾出现在检出内（[状态](validation-worktree-status-before.txt)、[状态](validation-worktree-status-clean.txt)），不计为干净检出结果；移到检出外之后，检查前后 `git status --porcelain` 均为空（[前](validation-status-clean-before.log)、[后](validation-status-clean-after.log)）。临时链接和 target 不计为源码；工作目录和构建缓存位置见 [路径](validation-worktree.txt)。

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p content-native-loader --test loader_acceptance` | 3 passed，0 failed，0 ignored（[日志](loader-targeted-02.log)） |
| `cargo test --offline -p engine-native-wasm --test vm_package_worker` | 3 passed，0 failed，0 ignored（[日志](wasm-targeted-02.log)） |
| `cargo test --offline -p game-server --test native_product_path vm_package_with_characters_installs_loads_and_runs -- --exact` | 1 passed，0 failed（[日志](server-targeted-02.log)） |
| `cargo run --offline -p game-server --example emit_vm_fixture -- <临时目录>` | exit 0；生成含真实模块名和资产哈希的 npack / character 夹具（[日志](fixture-generation-02.log)） |
| `cargo test --offline -p engine-native -p engine-native-wasm -p content-native-loader -p game-server --no-fail-fast` | exit 0：engine-native **1584/0/71**、engine-native-wasm **52/0/0**、content-native-loader **32/0/0**、game-server **358/0/0**（通过/失败/忽略；[计数](rust-full-counts.txt)、[完整输出](rust-full.log)） |
| 干净验证提交 `cargo check --offline --workspace --all-targets`，`CARGO_TARGET_DIR` 在检出外 | exit 0（[日志](workspace-check-clean.log)、[状态](workspace-check-clean.exit)） |
| `pnpm --dir apps/web-client test`；`pnpm --dir apps/web-client typecheck` | 前端 80 passed、0 failed、0 skipped；类型检查 exit 0（[测试](frontend-test.log)、[类型](frontend-typecheck.log)） |
| 干净验证提交 `CARGO_NET_OFFLINE=true pnpm --dir apps/web-client build:native`；`pnpm --dir apps/web-client smoke:native` | 新构建的实际 WASM Worker **51/0**，包含技能归属、回合末生效、恢复重放和错名拒绝（[构建](native-build.log)、[冒烟](native-worker-smoke.log)） |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py` | 单测 **1/0**；**8621** 个受维护文件均不超过 2000 行（[单测](source-line-tests.log)、[行数](source-lines.log)） |
| `python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py` | **1826** 文件、0 坏链；ADR 编号检查 exit 0（[链接](local-links.log)、[ADR](adr-index.log)） |

NC02 因历史端到端夹具无效而回退为部分：本批未重跑修正夹具的真实浏览器+HTTP 同链生命周期，也未覆盖第三方跨包依赖或主动技/修正器/view-as。NC09 客户端本地内置包入口仍拒绝 VM 包；单纯移除拒绝会跳过 ABI、依赖、资产及未桥接能力验证，本批没有开放该入口。
