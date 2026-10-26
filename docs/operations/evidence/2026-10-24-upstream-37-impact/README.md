# 2026-10-24 上游 37 提交影响核对

本目录是本轮静态审查与验证记录；当前差异状态只在[UP37 规则主账](../../../architecture/native-rules-baseline.md#upstream-2026-10-24)维护，全项目下一步只在[进度主稿](../../../architecture/native-progress.md#delivery-2026-10-24)维护。[两仓扩展台账](../../../migration/upstream-extension-reconstruction.md#upstream-2026-10-24)登记来源版本，不复制规则待办。

## 范围和方法

- 主项目：`7e28a93799a9c01794085e911a72a4faa5673883` 加已有 dirty 工作树；规则 v322 / 存档 v105。
- noname：`df2682e66af9930fe02861a25cb6f2cd2ef6ae36..1a548c95acdf8ac1ef44d74473403eda2ac86cf0`，37 提交、167 个净变化路径。`git log --reverse` + 每提交 `git show --find-renames --unified=4` 建完整清单；[commits.json](commits.json)含完整 SHA、全部路径、patch SHA256 和分类。[逐提交核对表](commit-review.md)覆盖 37/37，最终行为按最后版本合并。
- Liuli：HEAD `8485dbb29ff16f9b258a766b1a17ef75f883aade` 不变、7 个本地修改保留；没有把 noname 的新语义自动声明为 Liuli 的新基准。
- 本轮检查具体规则修正 hunks、最终角色/技能/卡牌定义、旧线 DSL/原语/已有测试与原生目录/包入口。CodeGraph 工具本轮不可用，按具体文件做源码和字面查询。大量模式异步改写已核对变更范围、导出/构建/共享接口与代表性结算路径，**没有逐场景证明全模式语义等价**。
- 不执行上游代码；使用已安装 TypeScript parser 读取对象定义。未进行自动内容同步、规则移植、资源复制、重录 golden、版本升级或提交。

## 直接结论及复核入口

1. **实际已实现行为优先**：旧线涤仇 action 10/选择续行仍要求残留标记、只清所选目标且清理晚于伤害，与本批上游“本回合已移除也可选、先清所有标记”的语义不同。既有单目标用例不能覆盖该差异。冯河已有部分效果但缺英霸标记分支。其余关键项中，多数为明确 noop，讲武/明节/自固等还存在近似；不能因为 catalog 标记 `ported` 就报成已覆盖。见[current-content.json](current-content.json)及主账 R01–R09。
2. **新增量不是 8 个新 ID**：AST 差集为 6 个新增角色、2 个已有角色从空技能/隐藏状态启用、19 个新增顶层技能和 3 个衍生牌定义。6 个新 ID / 19 个技能未进入本地 catalog，两个启用角色在本地仍为 `skills: []`。[content-delta.json](content-delta.json)记录定义位置/摘要，[提取命令](content-delta.cjs)固定比较前后 ref；其中原脚本先输出完整定义，本证据裁去重复源文，仅保留 SHA256/行号/技能元数据。
3. **原生与旧线分开**：本轮关键角色/技能未找到原生同名注册（[查询命令与结果](native-identifiers.json)），并检查 `standard/generals.rs` 和正式包入口；`bundledNativePackages.ts:93` 仍拒绝 VM 模块。缺同名符号不是所有行为都不存在的证明，既有基础能力可复用，但不能将旧线 DSL/Host 测试算成原生可玩。
4. **扩展目录不变，依赖不能跳过**：[upstream-scope.json](upstream-scope.json)证明 noname 的 `apps/core/extension`、`packages/extension` 与标准角色目录树对象 ID 不变；8/854 与 29/413 的目录/文件计数不变。Boss 扩展 `arenaReady` 实际加载 boss/versus 模式技能，因此模式改写仍需做间接回归。37 提交不等于 37 来源验收完成。[四个已覆盖 Boss 技能的前后差异样本](boss-sample.diff)中，凶兽/无脏/隐贼/助恶主要是显式参数、Async 与等价查询写法；该静态样本不能替代整套模式的轨迹回归。对决三人选将调用现在 await `chooseCharacterDouble().forResult()`，共享函数补返回事件是必要配套。
5. **工程清理有独立证据**：[format-comparison.json](format-comparison.json)核对 `ad1065b0` 的 35 份和 `a15a1e4f` 的 5 份 JS/TS。全部解析诊断为 0；忽略注释、位置及冗余括号后 34 份结构相同。其余 6 份人工核对为 `gameEvent.ts`/sandbox 空语句删除、三个 help 模板的 HTML 规范化与 config 模板排版，无需移植为原生规则。此方法不声称任意 JS 的运行等价；两个脚本及比较结果均保留。`card/standard.js` AST 相同但字节不同，标准卡规则未在该提交新增。

## 本轮验证

- **只读源同步检查未通过，属于已登记的真实版本差异**：`node scripts/sync-original-content.mjs --verify-only`，exit 1，首个差异 `legacy-source/bingshi/skill.js: size differs`。[原始输出](sync-verify.log)、[执行记录](sync-verify.json)。它在首个差异停止，不能把这一条当作完整漂移清单；完整更新范围见 Git 清单。未为消除该失败覆盖现有内容。
- 第一次完整 preflight（18:05–18:10）执行 20 项，18 通过、2 失败、0 跳过：文档行号检查和包含该检查的脚本单测都被历史 `card/standard.js` 第 5153 行越界阻断；180 脚本用例中 179 通过、1 失败。已把 10 月 7 日证据的工作区行号改为更新前固定提交链接，牌表/数量/历史结论保持不变；其 blob 与 9 月 23 日的 `5c02a745` 相同。没有关闭门禁或放宽断言。[第一次执行摘要](preflight-full-01.json)、[完整输出](preflight-full-01-raw.log)。修复后在 **18:12–18:17 +08:00** 重新运行整个 preflight：**20/20 通过，0 失败、0 跳过；脚本单测 180/180，行数门禁通过**。[第二次执行摘要](preflight-full-02.json)、[完整输出](preflight-full-02-raw.log)。第一次失败仍保留，不用单项补跑冒充全量通过。本轮不重跑产品 Rust/WASM/WS 测试，因为产品代码未修改；也没有将历史测试计数记为本次执行。

文档增量链接、既有锚点、37 提交映射与新内容数量检查见 [documentation-checks.json](documentation-checks.json)；[inventory-checks.json](inventory-checks.json)核对实际 Git 提交/patch 摘要，当前源码摘要见 [source-manifest.json](source-manifest.json)。原历史规则正文和进度轮次区保持不变，仅修正卡牌证据的可漂移引用。

## 证据边界与后续验收

规则主账每项都列具体反例：含原生内容的改动应运行 engine-native 全量测试；带额外阶段/随机/临时技能/私有状态的内容应比较 state/event/RNG 和保存恢复轨迹；新包需证明正式 Worker/真实 WS 原内容锁路径。模式异步改写、AI 强度/隐私、许劭交互、图片/语音及跨平台均未完成动态验收。旧线单测通过也不会消除这些待办。

起始备份在工作区外 `/tmp/noname-next-upstream-37-impact-20261024/`：HEAD、index/worktree patch、status、47,078 路径摘要、3 份主文档原件与随后修正引用前备份的 1 份历史文档；完整上游 patch 同样保留在该目录。之前 Git 同步的可恢复备份仍在 `.upstream-backups/20261024/`。[最终保护记录](preservation.json)核对 47,078 个既有路径，仅 3 份主文档与 1 份历史证据的行号定位变化，其余 47,074 个路径、3,651 个依赖路径及最新产品回归的 1,067 个源码摘要不变；Liuli 7 份修改的字节和两仓 status 保留。HEAD/index 不变，未暂存/提交。
