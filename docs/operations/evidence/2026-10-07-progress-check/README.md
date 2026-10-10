# 2026-10-07 进度核对

核验位置：`noname-next` 当前工作区，规则 v60、NGSA 格式 v2。当前状态与后续待做见[进度主稿](../../../architecture/native-progress.md)。本轮只核对进度、更新文档与证据，未修改产品源码或包数据。上一轮：[2026-10-06 进度复核](../2026-10-06-native-progress-audit/README.md)。

## 处理进度

| 核对项 | 结果 |
|---|---|
| 9 月 24 日审查修复之后的改动 | 按修改时间检查 `noname-next`（不含构建目录与上游归档）：只有 10-06 复核写入的 9 份文档及其证据目录；产品源码、包数据、Cargo 文件均无改动。10-07 当天在本证据写入前没有任何文件改动 |
| 源码摘要 | 以 [10-06 登记的 106 个文件](../2026-10-06-native-progress-audit/source-hashes.txt)为基线（engine-native 51、content-runtime 20、content-native-loader 3、转换包 30、根 Cargo 文件 2），测试前后 `shasum -a 256 -c` 均 106/106 一致（[测试前](hash-check-before.log)、[测试后](hash-check-after.log)） |
| P0 存档解码加固 | 未开工：`archive.rs` 摘要未变，10-06 探针复现的未知牌种 panic 与多余载荷被接受仍然存在；本次未重跑探针 |
| P1 其余六条桥接迁移 | 未开工：`g1_huanle`、`g1_d3`、`g1_hearth`、`g1_cardpile`、`g1_shahai_gwent`、`g1_shahai_yunchou` 与各包数据摘要未变；炉石 `cards.json` 仍用 `leishajian`×2、`shenenshu`×2 |
| 其余待办与 O1–O5 | 无新的实现或测量；`engine-core/Cargo.toml` 的 `tempfile` 仍在普通依赖（O1 未改） |
| 上游来源 | 两仓归档目录在 10-06 清点之后没有文件变化，未重跑清点脚本 |

## 实跑结果

在 `noname-next` 根目录执行，构建目录为沙箱独立的 `CARGO_TARGET_DIR`（未使用共享 `target/`，未执行 cargo clean）：

| 命令 | 结果 | 完整输出 |
|---|---|---|
| `cargo test --offline -p engine-native --no-fail-fast` | 345 通过 / 0 失败 / 0 忽略（20 个测试目标）；含 `replay_scenarios_match_golden`（14 个场景）、`scenarios_are_registered_in_coverage_manifest`、`nullify_rejected_when_played_actively`、`card_kind_keys_bijective` | [engine-native-tests.log](engine-native-tests.log) |
| `cargo test --offline -p content-native-loader --no-fail-fast` | 1 / 0 / 0 | [content-native-loader-tests.log](content-native-loader-tests.log) |
| `cargo test --offline -p content-runtime --no-fail-fast` | 31 / 0 / 0 | [content-runtime-tests.log](content-runtime-tests.log) |

工具链见 [environment.txt](environment.txt)：本次 shell 的 PATH 先命中 rustup 的 rustc/cargo 1.95.0，10-06 复核用的是 Homebrew 的 1.99.0。两次功能结果相同。[timings.txt](timings.txt) 的耗时只是执行记录；工具链与构建缓存都不同，不作性能对比。

## 本次静态核查

结构核查用 rg 与源码读取（核查时本项目尚未初始化 CodeGraph 索引）。核查后同日执行 `codegraph init` 与 `codegraph index`，索引位于 `noname-next/.codegraph/`（2,882 个文件）。工作区的 MCP 服务以上一级目录为根，查询时需把 `projectPath` 设为 `noname-next`。CodeGraph 不把作为参数传递的函数（如 `.map(num_to_card_kind)`）计入调用者，这类引用仍需用 rg 补查。

1. 进度主稿 P0 的描述与源码一致：`num_to_card_kind` 的 4 处调用为 `use_as`（`dec_opt_u32(r)?.map(num_to_card_kind)`）、两处 `need`（按 u32 读）和 `declared_kinds`（按 u8 读后扩成 u32）；`ArchiveError::BadTag` 只能携带 u8；`decode_archive` 在 `dec_snapshot` 之后不检查剩余字节。
2. 新发现的潜在错位：`enc_winner` 用 `*f as u8` 写势力，`dec_winner` 用显式表 `dec_faction` 读，对应的 `enc_faction` 从未被调用（编译器报 dead_code）。`Faction` 声明顺序为魏、蜀、吴、群，与解码表 0–3 一致，当前没有错误；以后增删或重排势力，编码会静默错位。已并入主稿 P0。
3. 编译警告：engine-native lib 16 条，包括未用的导入和变量、上面的 `enc_faction`，以及从未构造的 `JiJiStage::Retarget`（快照恢复把它映射回 `Target`，行为不受影响）。测试与示例另有大量未用变量和对引用调用 `drop` 的警告；`extension.rs` 单元测试里 4 条"未使用的 Result"后面紧跟状态断言，不算漏检。已登记为主稿 P3。
4. 其余六条桥接迁移的数据前提：huanle、gwent、yunchou、hearth 四个卡牌包的 `cards.json`（由 `gen_cardpile_packages` 生成）都还没有 ADR 0778 要求的 `pack` 段；欢乐的技能写在顶层 `host_skills[].behavior`，值是去掉 `host:` 前缀的行为键，由 `g1_huanle.rs` 的手写映射表转换，迁移时应改为 `skills[].mapping`；d3、cardpile、五行分别是武将、牌堆和初始化数据，需要各自的适配。已写入主稿 P1。
5. 主稿待办漏项：演进提案 A0 第 3 点（`SkillDefId` 只追加）与第 4 点剩余的存档编码表往返，此前都没有挂靠主稿任务。现分别挂到 P1 统一装载与 P0。

## 文档更新

- [进度主稿](../../../architecture/native-progress.md)：页首与快照按本次核对更新；待做表注明截至本日没有实现增量，P0 补编码表往返与势力编码，P1 统一装载补迁移前提与技能编号守护，新增 P3 编译警告；证据链接更新。
- [演进提案](../../../architecture/architecture-evolution-plan.md)：复核记录加本日一行；A0 落地状态与挂靠表写明第 3、4 点剩余部分挂到哪项主稿任务。
- 台账、覆盖清单与规则基准经抽查与源码一致，本次未改。
- 本地链接与锚点检查见 [doc-checks.txt](doc-checks.txt)，改动文件摘要见 [source-hashes.txt](source-hashes.txt)。
