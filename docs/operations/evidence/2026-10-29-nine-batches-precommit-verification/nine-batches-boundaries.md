## Review

**结论：九批的逻辑归属与实施顺序可以确认；当前工作树不能按整文件直接拆成九个安全提交。** 版本、golden、版本断言及后续源码拆分已经交叠，需要先恢复相应中间态，再逐批核验。此结论不代表产品验收通过。

- **Correct**：已读取父目录 AGENTS.md、进度顶部状态扫描、剩余台账、裁决登记及九批 evidence README，并对照当前 diff、未跟踪清单、夹具、golden 清单、测试登记与迁移后的源码位置。
- **Fixed**：无。本次只读，未修改文件、index 或提交，未运行构建。
- **基线**：主线程采集的 [environment.json](environment.json#L2) 记录实际 HEAD 为 `95f51867a3648321da1cc92149b60e2bbfaae748`，暂存路径为空；原始工作树指纹为 `0306191d6838efb12f3b76986e805a42cb15e38bfa9b31663dc11055677e7e78`。本审查引用该采集结果，未自行执行 Git 命令。

### 九批归属与入库顺序

下表证据目录均位于 `docs/operations/evidence/`。测试结果仅作为对应历史批次的记录，不作为当前工作树通过的证明。

| 顺序 | 批次 / evidence 目录 | 应归属的主要改动 | 版本与逐批入库风险 |
|---|---|---|---|
| ① | B8 / `2026-10-28-b8-gate1b` | 编译器 ABI 755 死常量及导入删除；28 个 native 测试文件 lint 清理；12 个证据文件 | 保持 v325。可先拆，但编译器文件头已与⑨合成同一 hunk，部分测试还混有⑨路径调整。 |
| ② | NC04 / `2026-10-29-nc04-cards-given-v326` | `give_hand_cards`、仁德/蝶舞调用、事件说明、3 个回归、两份 v325 夹具、golden/版本断言；25 个证据文件 | **独占 v326**。必须恢复 v326 断言及 golden；蝶舞改动已被⑨移入新文件。 |
| ③ | GT-014 / `2026-10-29-gt014-import-deep-validation` | WASM 校验函数/导出、Worker 请求、客户端导入事务、未签名 UI、前端/Worker/Rust 回归；19 个证据文件 | 不升号，处于 v326。产品文件边界最清楚，按所定顺序接在②之后。 |
| ④ | NC07 / `2026-10-29-nc07-archive-hash-fields` | 10 字段入 hash、字段反例、结构守卫、v326 夹具、golden/版本断言；23 个证据文件 | **独占 v327**。新增守卫现已依赖⑨才产生的文件，不能原样提前入库。 |
| ⑤ | J1 / `2026-10-29-adr0786-j1` | 提交期挂起、快照/归档/hash/恢复/询问、prompt 注册、4 个回归、ADR 状态、ABI 盘点；22 个证据文件 | **独占 v328**。主体实现与 hash 编码已被⑨移出，恢复路径又与⑧交叠。 |
| ⑥ | 统计 / `2026-10-29-nc08-core-pack-statistics` | 扫描脚本、命中 JSON、filter 片段、统计/文档检查；11 个证据文件 | 不改语义/版本。目录自身可独立归属；共享主稿中只取本批统计登记。 |
| ⑦ | O6 / `2026-10-29-o6-test-grouping` | workspace debug 配置、两 crate Cargo 配置及22个分组/守卫文件、3处子进程测试名、跨目标脚本；9 个证据文件 | 保持 v328。分组文件已登记④⑤的新测试，故应置于它们之后。 |
| ⑧ | F7 步骤1–2 / `2026-10-29-native-split-stage1-2` | 分层棘轮及单测、CI/preflight、Catalog hooks、固有流程/宿主询问/恢复迁移、架构方案；4 个证据文件 | 保持 v328/v105。新增的宿主询问文件已再被⑨拆分，需恢复本批单体形态。 |
| ⑨ | 有效行拆分 / `2026-10-29-rust-effective-1000` | 八个 crate 的源码拆分、行数门禁及单测、coverage 消费者、源码扫描测试路径、文档引用与收口记录；1 个 evidence README | 最后入库。必须同时带齐新模块、旧文件路由/导出、扫描测试及脚本消费者；本批没有保存可重放的完整补丁或历史源码快照。 |

版本顺序与[裁决登记第10–13项及版本顺序说明](../2026-10-28-user-decisions/README.md#L37)一致：

**B8 → NC04 v326 → GT-014 → NC07 v327 → J1 v328 → 统计 → O6 → F7步骤1–2 → 有效行拆分。**

### 文件清单是否闭合

[完整状态清单](git-status-before.txt#L1)中的原始 **757 个脏路径**，按路径与变更职责可以归类：

| 类别 | 数量 | 归属 |
|---|---:|---|
| 已跟踪文件差异 | 370 | CI/Cargo/前端10；crate文件309；文档46；脚本5。包含369个修改、1个删除。 |
| 新增 crate 文件 | 257 | GT-014测试；NC07/J1测试；O6分组；F7新模块；⑨拆分模块。 |
| 九批 evidence 文件 | 126 | 分别为12、25、19、23、22、11、9、4、1个。 |
| 新增脚本 | 4 | ⑧分层棘轮及单测；⑨有效行计数器及单测。 |
| **合计** | **757** | 未发现必须另解释为第十批产品功能的路径。 |

状态文本还包含核验过程自身生成的 `git-status-before.bin`；本轮 `nine-batches-precommit-verification/` 下继续产生的日志和清单应归本次核验，不混入历史九批。

需保留的伴随归属：

- 38份历史 evidence README 的修改可归于⑧/⑨源码引用迁移；同一 README 可能同时含两批引用调整。
- `native-progress.md`、`native-remaining-ledger.md`、`native-rules-baseline.md` 是多批共享文件。
- `native-convergence-agent-prompt.md` 是九批之后的交接收口文档；`self-check-guide.md` 混合⑦测试写法、⑧第21项门禁、⑨有效行要求。不能把这些文件的全部内容算作较早某一批。
- 父目录 `AGENTS.md` 的有效行规则已存在，但它位于本 checkout 外，不在上述757项内；本仓提交不能自动包含该父目录文件。

**遗失检查结果**：两份 v325 夹具、一份 v326 夹具、三份版本 golden 清单、O6分组登记均存在。唯一删除的 `snap/host_asks.rs` 有对应迁入文件及其⑨拆分文件，未见该文件整段遗失的证据。此处确认的是路径和承接关系，不是对所有迁移代码逐字守恒的认证。

### 同文件 / 同 hunk 的关键交叠

| 位置 | 已验证的交叠及最小处理 |
|---|---|
| [standard.rs:136](../../../../crates/engine-native-content/src/content/standard.rs#L136) | 同一 hunk 累积三版说明并把 `325`直接改成`328`；同文件还有⑧恢复签名和 hooks。②、④、⑤须分别得到326、327、328，不能原样选择当前版本 hunk。 |
| [replay_scenarios.rs:1068](../../../../crates/engine-native/tests/replay_scenarios.rs#L1068) | 14项期望已是v328，旁边累计三批归因注释。v326/v327/v328清单均在，能帮助重建各版期望，但当前 diff 没有中间两版替换。 |
| [nc04_cards_given_event.rs:351](../../../../crates/engine-native/tests/nc04_cards_given_event.rs#L351) | ②新增测试内的目录版本已由④⑤改成328；`nc04_vm_resume`、`nc07_lianyaohu_hash`、`pa12_tag656_version`同样连续改写。②提交须用326，④提交须用327。 |
| [effect_set_draw_phase_from_bottom.rs:1](../../../../crates/content-compiler/src/codegen/effect_set_draw_phase_from_bottom.rs#L1) | 文件头同一 hunk 同时删除755导入（①）和757导入（⑨，因相应结算器迁出）。①只应删除755；若提前取完整文件头，原位置仍存在的757使用会失去导入。 |
| [xianjian_card_use.rs:80](../../../../crates/engine-native-content/src/content/host/step/xianjian_card_use.rs#L80) | ②蝶舞批量给牌现位于⑨新增文件。②应在原`xianjian_cards.rs`位置实施；不能为了带入②而提前带入整个⑨模块。仁德及`machine.rs`的②边界较清楚。 |
| [hash.rs:229](../../../../crates/engine-native/src/engine/hash.rs#L229) / [hash/machine.rs:273](../../../../crates/engine-native/src/engine/hash/machine.rs#L273) | ④字段摘要仍在原文件；⑤提交挂起摘要已随⑨迁到新文件。⑤须先在旧结构加入编码，⑨再搬移。 |
| [nc07_archive_hash_guard.rs:55](../../../../crates/engine-native/tests/nc07_archive_hash_guard.rs#L55) | ④新增守卫现`include_str!`⑨的`engine/hash/machine.rs`。若④原样入库而⑨未入，缺少该文件；④中间态须保持拆分前读取方式。 |
| [rvm_bridge.rs:29](../../../../crates/engine-native-kernel/src/rvm_bridge.rs#L29) / [rvm_bridge/machine.rs:408](../../../../crates/engine-native-kernel/src/rvm_bridge/machine.rs#L408) | ⑤新逻辑与⑨机器实现搬移重合。当前未跟踪`machine.rs`包含⑤主体及既有实现，不能全部归⑤或全部延迟而遗漏⑤。`rvm_ops`的codec迁移同类。 |
| [snap/restore.rs:8](../../../../crates/engine-native-kernel/src/snap/restore.rs#L8) / [snap/asks.rs:634](../../../../crates/engine-native-kernel/src/snap/asks.rs#L634) | ⑤提交挂起恢复/询问、⑧Catalog回调/恢复参数、⑨shape帮助项拆出同时存在。须按⑤→⑧→⑨恢复各自形态。 |
| [content/host/asks.rs:9](../../../../crates/engine-native-content/src/content/host/asks.rs#L9) | ⑧迁入的新文件现导入⑨新增的`asks_mtg`和`asks_wudian`。⑧原样取当前文件、却不取⑨兄弟模块，会形成缺失依赖。 |
| [nc08_upstream_mashu.rs:30](../../../../crates/engine-native/tests/nc08_upstream_mashu.rs#L30) | 同文件同时有①`mut`清理和⑨`state/core_ops.rs`扫描路径调整。类似测试不能按整个`tests/`目录归①。 |
| [.github/workflows/ci.yml:44](../../../../.github/workflows/ci.yml#L44) | 一个相邻 hunk 含⑧分层门禁和⑨步骤名调整；`self-check-guide.md`另混⑦⑧⑨。按职责拆取。 |

### Findings 与暂存判断

**Finding — P1，逐批入库边界尚未恢复。** 上表中存在真实的跨批缺失依赖，以及同一版本字面量被后批覆盖的情况。当前只有最终状态，不能通过“每批列一组文件并整文件暂存”保证九个提交分别完整。最小处理是按上述顺序恢复版本、golden、断言、源码位置及扫描消费者的中间态；每个候选提交再验证自身差异和必要检查。历史全绿及最终工作树编译通过，都不能替代这一步。

**Finding — P2，⑨文档行号迁移留下新的越界引用。** 例如：

- [rule-distance-unmodeled/README.md:14](../2026-10-17-rule-distance-unmodeled/README.md#L14) 将引用改为`state/core_ops.rs` 的旧行号 `90/1192`，但当前目标文件总共929行。
- [blocked-fixes/README.md:126](../2026-10-18-blocked-fixes/README.md#L126) 改为`kernel/sha/step.rs` 的旧范围 `693–1243`，但当前目标文件总共1066行；同类范围也出现在规则基准及其他历史证据。

这是本次 diff 将范围起点迁移、却保留旧终点造成的问题。最小修正是核对并同步范围终点，或改为准确的符号/单行引用；不能以“引用起点未越界”认定整个范围有效。

**Merge verdict：BLOCK——针对“直接按当前文件切成九个独立提交”的方案。** 九批归属和顺序可用；③产品文件、⑥证据目录、⑦配置/分组等边界相对清楚。共享文件仍需上述重建，之后才能逐批判定可入库。本审查不对全部产品代码作合并通过结论。

无法确认的事项：

- 九批证据目录未提供逐批完整补丁或对应dirty源码快照。早期`source-head.txt`只绑定共同HEAD；本轮指纹只绑定最终扫描状态，无法认证九个历史中间态。
- ⑨ evidence目录当前只有README，无法从该目录复核其汇总的原始测试输出。主线程本轮编译应另行记录、另行判断。
- 本次没有执行逐批暂存演练、构建、测试或当前WASM重建；没有认证整包、平台或上游规则等价。已登记B9保持未修，不在此次边界审查中归为新增缺陷。

未检查全部搬移代码的逐字/行为守恒，也未运行构建；主要风险是漏带未跟踪模块或把后批版本、扫描路径提前带入，导致中间提交不可用。