# NC06／NC07：技能实体牌区的存档所有权校验

- 实际执行：2026-10-28 09:08:26 +0800 → 2026-10-28 09:18:24 +0800（[开始](start-time.txt)、[结束](end-time.txt)）；源码基准 [`6b8ee6696558da27231aed80868b909c580446bb`](source-head.txt)。原生规则 v323、存档格式 v105、全局原生 ABI 753 不变。

## 反例、修复及边界

真实选将刘馥并发动筑城，把牌从牌堆移入筑城标记后，原所有权校验漏算该牌区，将合法存档报为 `card_missing_from_all_zones`；反而将同一张牌同时置于筑城与牌堆的坏档被接受。干净 HEAD 导出与独立 target 上，四类牌区的反例 **1 通过／3 失败／0 忽略**，且首次筑城反例 **0／2／0**（[四牌区红测](red-four-piles.log)、[初次红测](red-clean.log)、[导出目录](red-path.txt)）。真实技能触发后恢复同样复现：陷阱 **0／1／0**、炼妖壶 **9／1／0**（名称过滤同时命中其他测试）、护天 **0／1／0**（[陷阱](red-trap.log)、[壶仓](red-jar.log)、[护天](red-hutian.log)）。

共享校验现在把筑城、炼妖壶仓、陷阱和护天寄存作为实体牌区，检查每张牌有效、只属于一个牌区且不缺牌。天覆和机关怨的字段引用已有离场牌，义山字段记录历史失牌，这些不作为新的实体牌区重复计数。四个真实技能均以既有内容包发动并在存档恢复后继续运行；筑城还经原生 WASM 包装层的包锁和坏档拒绝验证。改动只纠正恢复时的所有权判定，不改变存档字节结构、规则结算或状态摘要：原本错误拒绝的合法 v105 档可恢复，原本错误接受的重复牌／非法牌坏档拒绝；其他版本门禁不变。新增回归还覆盖缺牌和非所有权引用。

首次改动后的定向套件为 **118 通过／1 失败／0 忽略**（[日志](targeted-first.log)）。在干净 HEAD 导出、叠加本批校验与原始测试改动并使用独立 target 后，陷阱用例仍 **0／1／0**（[复现](trap-repro-clean.log)、[target](trap-repro-target.txt)）：该用例将同一实体牌同时作为陷阱与手牌，之后又从弃牌堆重复置入陷阱。修正测试布阵为选取不同的杀并在移动时先从弃牌堆移除；护天纯序列化用例同样改为从牌堆真实移出两张牌。最终三个定向测试二进制 **119／0／0**（[日志](targeted-final.log)）。

独立发现尚未修复的 NC07 状态摘要问题：炼妖壶仓牌序 `[8, 80]` 与 `[80, 8]` 在当前版本生成相同摘要，但实际炼化选项顺序不同。干净导出探针 **1／0／0**（[源码](lianyaohu-hash-probe.rs)、[输出](lianyaohu-hash-probe.log)）；纳入摘要会改变规则基准与旧档门禁，已向用户单独请示 v323→v324，答复前不实施。其他牌区／状态字段的完整登记、产品 UI 和跨平台恢复仍待验收，NC06、NC07 保持部分。

## 验证

| 命令／范围 | 完整结果 |
|---|---|
| `cargo test --offline -p engine-native --test nc06_zhucheng_restore --test swd_hutian_deposit --test g1_shahai_swd --no-fail-fast` | **119／0／0**，exit 0（[日志](targeted-final.log)）。 |
| `CARGO_INCREMENTAL=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_TARGET_DIR=<独立 target> cargo test --offline -p engine-native -p engine-native-wasm -p content-native-loader -p game-server --no-fail-fast` | engine-native **1596／0／73**，engine-native-wasm **62／0／0**，content-native-loader **32／0／0**，game-server **360／0／0**；总计 **2050／0／73**，379 个测试目标，exit 0（[完整日志](full-tests.log)、[计数](test-counts.txt)、[计数脚本](count-tests.py)、[测试产物映射](test-artifacts.jsonl)、[target](green-target.txt)）。现有 content-compiler 的 ABI 755 两条警告仍存在。 |
| `CARGO_NET_OFFLINE=true pnpm --dir <HEAD 导出>/apps/web-client build:native`；`node wasm-zhucheng-probe.mjs <HEAD 导出>` | 使用独立导出和 target，仅叠加本批校验源码；Web 目标构建 exit 0，Node 加载实际 WASM 的筑城存档恢复和重放 **1／0**，缺包拒绝，exit 0（[构建](wasm-build.log)、[探针](wasm-zhucheng-probe.mjs)、[输出](wasm-probe.log)、[导出路径](wasm-build-path.txt)）。此探针不经过浏览器 UI 或 Worker。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py` | **1／0**，**8638** 个受维护文件不超过 2000 行，均 exit 0（[自检](check-unit.log)、[行数](check-source.log)）。 |
| `python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py`；`git diff --check` | **1844** 个文件、0 坏链；ADR 索引通过；diff 无空白错误，新测试 rustfmt 检查通过，均 exit 0（[格式](rustfmt-check.log)、[链接](check-links.log)、[ADR](check-adr.log)、[diff](check-diff.log)）。 |

该工作开始前已有的八份用户文档改动保留；进度与台账仅暂存本批追加行。
