# NC02：Worker 完整 VM 包摘要核对

- 实际执行：2026-10-28 00:15:53 +0800 → 2026-10-28 00:23:53 +0800（[结束](end-time.txt)；开工与结束各运行 `date '+%F %T %z'`）；源码基准 `df8b1587cfe1e25dc27449eac303ec453b66c1b0`（[开始](start-time.txt)、[源码](source-head.txt)）。八份先前已有的文档改动保留，本批只暂存自己的回填。
- 规则 v323、存档格式 v105、宿主 ABI 753 不变。已被错误接受的摘要不匹配包及其旧档现在明确拒绝；合法包的规则、归档结构和标准 golden 不变。

## 反例与修复

服务端内容运行时调用 `ContentPackage::validate`，WASM 的完整 npack 路径此前只校验 ABI 和桥接能力。以 `git archive HEAD` 的[独立导出](red-export-path.txt)叠加[回归测试](../../../../crates/engine-native-wasm/tests/vm_package_integrity.rs)，并使用[独立 target](cargo-target-path.txt)：修改 VM 字节码、触发事件、声明逻辑摘要或资源摘要的四种包均被普通局和身份局接受，各自产生并恢复了旧档；恢复合法档时则在后续目录指纹处失败，而非在包摘要处拒绝。反例 **0/1、exit 101**（[日志](red-clean-head.log)、[旧档](old-saves.json)）。字节码反例由同源 DSL 分别编译“摸 1”与“摸 2”，仅替换代码而不改声明摘要。

WASM 共用的 `installed_pack` 在完整 npack 桥接前调用已有 `ContentPackage::validate`。同一测试对普通局、身份局、合法档替换输入及上述四份实际旧档均检查摘要错误；合法包开局、恢复摘要一致。既有“缺触发声明”测试先重算包摘要，仍专门验证未映射技能不会静默消失。真实 Worker 中，篡改触发事件的包在开局及恢复均拒绝，失败恢复前后现局存档字节相同；有效包的武将技能仍实际触发、存档后重放一致。

## 尚未覆盖的真实缺口

包摘要自校验不是签名：可以重算摘要的来源仍未被认证。`ContentPackage::compute_logic_hash` 没有直接覆盖模块 `entry_pc`、`local_count`、`caps_mask` 等字段，当前也未复核模块自身指纹。[独立探针](uncovered-module-entry.rs)在本批修复后的 WASM 源码上仅把模块入口改到 `HALT`，包摘要不变且 `validate` 接受：同一个技能在结束阶段使体力分别为 **4 与 3**（[日志](uncovered-module-entry-final.log)，**1/0**）。首次探针使用了不存在的 `pending_choice`/`Choose`，编译 exit 101（[日志](uncovered-module-entry.log)）；在干净 HEAD 仅叠加该测试同样复现 E0599（[日志](uncovered-module-entry-clean-compile.log)），改用既有 `open_choice`/`Answer` 后才得到上述结果。

现有模块指纹算法覆盖入口、local 数、能力、名称与代码；[清点](module-inventory.py)对 Git 跟踪的 **144 个 npack JSON、13,771 个模块**逐一重算，0 个不匹配（[输出](module-inventory.log)；这包含重复与历史夹具，不代表第三方包）。后续须在共享校验边界补齐模块指纹验证并核对旧包兼容；`max_stack` 仍不在该指纹内。包的 `asset_hash` 只校验包内资源元数据，未核对额外内嵌 `cards`/`characters`/`pile` 的实际字节；旧的精简资产载荷也没有完整 npack 可供重算。VM 本地安装入口、签名、资产绑定和跨包真实规则引用仍未完成。

## 验证

| 命令 | 完整结果 |
|---|---|
| `NC02_VM_HASH_OLD_SAVES_INPUT=<证据目录>/old-saves.json cargo test --offline -p engine-native-wasm --test vm_package_integrity --test vm_package_worker -- --nocapture` | **5 passed / 0 failed / 0 ignored**，含四份真实旧档（[日志](targeted.log)）。 |
| `cargo test --offline -p <crate> --no-fail-fast`，依次对 engine-native、engine-native-wasm、content-native-loader、content-runtime、engine-ai 执行 | 五项均 exit 0；分别 **1588/0/72、59/0/0、32/0/0、34/0/0、29/0/0**（通过/失败/忽略；[计数](rust-full-counts.txt)、[状态](rust-full-status.txt)、[内核](engine-native-full.log)、[WASM](engine-native-wasm-full.log)、[加载器](content-native-loader-full.log)、[内容](content-runtime-full.log)、[AI](engine-ai-full.log)）。均在独立 `CARGO_TARGET_DIR`、`CARGO_INCREMENTAL=0`、test/dev debug info 关闭时执行。 |
| `CARGO_NET_OFFLINE=true pnpm --dir <独立导出>/apps/web-client build:native`；`pnpm --dir <独立导出>/apps/web-client smoke:native` | 实际 web WASM 构建 exit 0；Worker **57/0**（[构建](wasm-build.log)、[冒烟](worker-smoke.log)）。独立导出基于上述 HEAD，叠加四个本批代码/测试文件并逐字核对（[路径](build-export-path.txt)、[对照](build-source-match.log)）；依赖通过指向工作区现有 node_modules 的链接读取，产物只写入导出目录。 |
| `pnpm --dir apps/web-client test`；`pnpm --dir apps/web-client typecheck` | **87 passed / 0 failed / 0 skipped**；typecheck exit 0（[测试](frontend-test.log)、[类型](frontend-typecheck.log)）。 |
| `rustfmt --check --edition 2021 crates/engine-native-wasm/tests/vm_package_integrity.rs` | exit 0（[日志](rustfmt.log)）。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py` | **1/0**；8627 个受维护文件均在 2000 行以内（[单测](source-unit.log)、[行数](source-lines.log)）。 |
| `python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py` | **1839 个文件，坏链 0**；ADR 检查通过、未登记或过时冲突 0（[链接](local-links.log)、[ADR](adr-index.log)）。 |

首次 WASM 导出准备把 zsh 特殊变量 `path` 当循环变量，遮蔽 PATH，复制前退出；未执行构建（[记录](build-prepare-failure.txt)）。另一次 `cargo fmt -- <文件>` 意外格式化了 579 个已有源码文件；已对比开工状态、将该次差异保存到 `/tmp/noname-nc02-accidental-rustfmt.patch` 并用精确反向补丁撤销，原有八份文档与本批新测试未受影响。
