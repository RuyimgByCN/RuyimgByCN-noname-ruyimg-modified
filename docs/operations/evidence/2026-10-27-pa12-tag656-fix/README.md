# PA12 / B6：VM tag 656 冲突与版本兼容

- 实际执行：2026-10-27 23:46:19 +0800 → 2026-10-28 00:11:45 +0800（[开工](start-time.txt)、[结束](end-time.txt)）；源码基准 `ac83af136731715ef91d955543a10a01fa880f0a`（[记录](source-head.txt)）。用户先后明确授权 656/772、ABI 758 与 v322→v323；全局原生 ABI 753、存档格式 v105 不变。

## 根因与修复

[原始反例](../2026-10-27-pa12-tag656-blocker/README.md)：`sp@0.1.0` 的英逝与 `onlyOL@0.1.4` 的平寇均为扩展效果 656，VM 先匹配平寇，英逝实际分派错误。现保留英逝 656，将平寇改为 772、最低 ABI 758；编译器、真实包及所有受影响断言同步。字节码指令边界通过 VM 现有验证器解码，避免把常量内部字节误判为标签。[真实模块分派回归](../../../../crates/rule-vm/tests/effect_tag_656_772.rs)分别检查英逝与平寇的效果请求；这不是完整对局或上游等价证明。

原包 [onlyOL 0.1.4](onlyOL-0.1.4.npack.json) 保留；修正版 `onlyOL 0.1.5 / ABI 758` 由真实 `.skill` 源重编译。两个包对比：36 个模块中仅 `ol_pingkou_phase_end`、包版本、ABI 和逻辑哈希改变；资产元数据和资产哈希保持一致。首次重编缺少 `--assets`，因武将引用未知资产失败；追加 `--assets` 虽可生成包，但编译器计算的一个图片压缩大小估值从 171293 变为 100，文件本身仍是 171293 字节；最终从原包保留资产元数据后重编，命令和输出见[构建日志](package-build-preserved-assets.log)。这不是图片被替换。按指令边界扫描 74 个运行包、4351 个模块，并另查保留的 0.1.2/0.1.4：只有 SP 英逝 656、修正版平寇 772、两个旧版平寇 656（[脚本](package_effect_inventory.py)、[结果](package-effect-inventory.log)）。

## 包与存档边界

- 新代码拒绝 ABI 754–757 且含 656 的包，以及 ABI 低于 758 却含 772 的包；新的 SP ABI 745 包仍能装载，新的 SP 快照可恢复。内容存储、旧线直接装载与原生 VM 桥共享检查；原生桥另外检查宿主 ABI，所以原生宿主 753 明确拒绝 onlyOL 0.1.5，不能通过过滤模块绕过整包 ABI 门。
- 旧线冻结快照现在记录每个包装载时的 ABI。修复前的真实四人对局快照没有该字段：包含 656 时无法可靠区分 SP 和 onlyOL，统一拒绝恢复，包括旧 SP；不含这些标签的旧快照未因缺少该字段而一律拒绝。真实 0.1.2、0.1.4 和 SP 快照见[回归](../../../../crates/engine-core/tests/pa12_tag656_compat.rs)；另对带 754–757 字段的冻结快照逐项检查。这里防止原样旧档误解释，不是快照元数据的密码学认证：旧线状态摘要本来不覆盖全部内容注册表字段。
- 原生 v322 旧档格式仍能解码，但 v323 引擎因规则版本不符拒绝恢复；v323/v105 新档可保存恢复（[测试](../../../../crates/engine-native/tests/pa12_tag656_version.rs)）。没有实现旧档迁移。已发布的历史旧引擎二进制可能绕过宿主 ABI 检查直接加载新包；应使用修复后的宿主／包管理器，不能声称能远程改变历史二进制的行为。
- 原生 14 条标准 replay golden 没有 656 技能；仅把修复源码叠到干净 HEAD 且保持 v322 时，原 golden 全通过（[日志](golden-v322-probe.log)）；v323 重录的 14 个摘要见[日志](golden-v323.log)，版本本身进入状态摘要。英逝语义修复另由真实 VM 模块分派回归验证，不能把标准 replay 当作该技能整局验收。

## 测试与故障定位

首次影响面全量运行以 exit 101 结束：`content-runtime::atomic_install` 5 个失败由夹具中的无效 `0x02` 指令触发；`original_merged_jsrg` 与 `original_merged_qiaoshui` 各 1 个失败由 still-757 断言触发（[完整日志](full-rust.log)、[逐包计数](full-rust-counts.txt)）。原始 `git archive HEAD`、独立 target 的 atomic 测试 7/7；只叠本批验证器并强制刷新文件时间后为 2/7（[原 HEAD](atomic-clean-head.log)、[叠加重现](atomic-with-b6-validator-forced.log)）。两条 ABI 断言在原 HEAD 各通过 1/1，仅换入新包各失败 0/1（[jsrg HEAD](abi-assert-head-original_merged_jsrg.log)、[jsrg 新包](abi-assert-new-package-original_merged_jsrg.log)、[qiaoshui HEAD](abi-assert-head-original_merged_qiaoshui.log)、[qiaoshui 新包](abi-assert-new-package-original_merged_qiaoshui.log)）。无效夹具已改为合法 `HALT`，ABI 断言已同步。曾有一次叠加文件保留旧修改时间、Cargo 复用旧产物的无效初试，记录于 [日志](atomic-with-b6-validator.log)，不作通过依据。

普通工作区全目标检查使用只包含本批改动的[干净验证提交](check-clean-head.txt)，源码文件逐项对照工作树；以检出外独立 target 执行。第一次验证提交运行期间为控制磁盘占用主动中断（[日志](check-interrupted.log)，exit 143）；此前一版完整检查通过后又补了冻结快照测试，最终版本重新检查。验证提交以 [bundle](validation-final-candidate.bundle) 保留，可在含源码基准的仓库验证，不包含用户原有未提交文档。

额外严格门禁 `RUSTFLAGS='-D warnings' cargo check --offline -p engine-native --all-targets` 仍失败：本批修复了此前的 656 不可达分支后，暴露 content-compiler 的 ABI 755 未使用导入和常量（[当前](strict-check.log)）。原始干净 HEAD 在仅屏蔽 656 的 `unreachable-patterns` 后复现相同的两条 755 错误（[原 HEAD](strict-baseline.log)，exit 101）；不等于严格门禁已通过。

## 验证结果

以下计数顺序均为通过/失败/忽略；完整命令、输出和退出码保存在对应日志及 `.exit` 文件。最终全量结果见[重跑计数](retest-affected-full-counts.txt)。

- `cargo test --offline -p engine-native -p engine-core -p game-server -p content-compiler -p rule-vm -p content-schema -p content-runtime -p engine-native-wasm -p content-native-loader --no-fail-fast`：首次 exit 101，结果及失败见上；未失败的目标原生 1588/0/72、server 360/0/0、rule-vm 17/0/0、wasm 58/0/0、loader 32/0/0、schema 0/0/0。
- `cargo test --offline -p engine-core -p content-runtime -p content-compiler --no-fail-fast`：修复后 exit 0，依次 3843/0/1、34/0/0、541/0/0（[完整日志](retest-affected-full.log)、[计数](retest-affected-full-counts.txt)）。新增的明确 ABI 快照断言另作[定向复测](legacy-compat-final.log)，3/0/0。
- `cargo check --offline --workspace --all-targets`：最终验证提交的独立干净 HEAD，exit 0，见[日志](check-clean-head.log)及[源码对照](check-source-match-final.txt)。
- `CARGO_NET_OFFLINE=true pnpm --dir <独立导出>/apps/web-client build:native`；`pnpm --dir <独立导出>/apps/web-client smoke:native`：实际 WASM 构建 exit 0，Worker 冒烟 55/0（[构建](wasm-build.log)、[冒烟](worker-smoke.log)）。冒烟未单独通过 Worker 传入 onlyOL；旧包/新包在 WASM 入口的拒绝由 Rust [定向测试](wasm-compat-target.log)覆盖。该测试为隔离 ABI 门清除包的依赖声明并重算逻辑哈希，模块不变。
- `pnpm --dir apps/web-client test` 与 `typecheck`：87/0/0，类型检查 exit 0（[测试](frontend-test.log)、[类型](frontend-typecheck.log)）；`node scripts/sync-original-content.mjs --verify-only` exit 0（[日志](sync-verify.log)）。
- `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py`：1/0/0、8626 个文件通过（[单测](source-unit.log)、[行数](source-lines.log)）。
- `python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py`：1838 个文件、坏链 0；ADR 未登记冲突、过时登记、标题错号和索引坏链均为 0（[链接](local-links.log)、[ADR](adr.log)）。
- ABI 批次结束后确认无 Cargo/rustc 占用，执行 `cargo clean` 与 `cargo clean --target-dir /private/tmp/noname-next-cargo-target`；两个目标目录均不存在（[记录](cargo-clean.log)）。

静态迁移盘点为滤镜命中 2652、推迟 1169、近似可直接原生 1101/7614（[输出](inventory.log)、[JSON](inventory.json)）；这不是整包 ABI 就绪率。N01 的原生平寇、胆守与完整对局、选择窗/主动技桥及 ABI 753→758 宿主升号仍未完成，升号须另行获得用户授权。
