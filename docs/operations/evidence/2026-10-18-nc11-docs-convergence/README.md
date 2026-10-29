# NC11 验收数据与文档收敛（2026-10-18）

本目录是 **NC11（验收数据与文档收敛）** 的原始命令、输出与结论依据。方案见
[原生架构收敛执行方案](../../../architecture/native-convergence-plan.md#nc11)，
当前状态仍以[进度主稿](../../../architecture/native-progress.md#native-convergence)为准。

- 核验日期：2026-10-18；本切片开工时 HEAD `fea360ff`、最终核验时 HEAD `3012d960`（并行提交持续推进，
  读数前先 `git rev-parse HEAD`；本次实测源码版本见 [`source-versions.log`](source-versions.log)）。
- 工作区：**dirty**（他人 249→261 项未提交改动；本切片只改文档/脚本/证据，未碰产品源码）。
- 规则/存档（实测）：工作树 `STANDARD_RULESET_VERSION = 322`、`ARCHIVE_FORMAT_VERSION = 105`；
  已提交 HEAD 为 321 / 105（`crates/engine-native-content/src/content/standard.rs:126`、
  `crates/engine-native-kernel/src/archive.rs:113`）。
- 本切片**不改产品源码**；两条行数门禁命令通过（见 [`source-lines-gate.log`](source-lines-gate.log)）。

## 1 主稿一致性核对：发现并修正的不一致

以**源码与实测**为准（版本常量、`todo-register.json` 机器字段、`packages/extensions/*/package.npack.json`、
行数门禁、ADR 目录、链接检查）。逐项如下，均保留历史日期语境，只把「当前结论」改为与证据相符：

| # | 不一致 | 修正 | 依据 |
|---|---|---|---|
| 1 | 规则版本：进度主稿「当前版本」写 **v321**；进度 NC06/NC07/NC08/NC10 行与规则基准 NC01 行分别写 v322 工作树、且把 v322 归因给 NC01 | 统一为「已提交 HEAD v321 / 当前工作树 v322 / 存档 v105」，并写明 v322 归因是 **NC07 座位环入摘要**，NC01 不升版本 | `crates/engine-native-content/src/content/standard.rs:126`（工作树 322）、`git show HEAD:...standard.rs` 第 127 行（321）、`crates/engine-native/tests/replay_scenarios.rs:1061`、[`source-versions.log`](source-versions.log) |
| 2 | 存档格式：覆盖清单写 **`ARCHIVE_FORMAT_VERSION = 86`**、进度「完成状态」写「NGSA 存档（现 v60）」、进度 P0 行写「现为 v41」 | 三处统一为 **v105**，并标注原数字的时点 | `crates/engine-native-kernel/src/archive.rs:113`；[`source-versions.log`](source-versions.log) |
| 3 | 规则基准头部写「现源码为 **v291 / 存档 v79**，v291 为最新」 | 改为 2026-10-18 实测值（HEAD 321 / 工作树 322 / 存档 105），v322 条目补入「近期版本」块 | 同上 |
| 4 | 转换包数量：台账写「packages/extensions 现为 **15 个**转换包目录」；进度/清单多处沿用「15 个转换包」 | 统一标注为 **实测 41 个目录 / 41 份 `package.npack.json`**（其中 40 个对应 37 来源，1 个 `boss-zhugeliang-upstream-liuli` 属 T11 Boss 内容）；15 的句子保留为 2026-10-10 时点 | [`package-manifests.log`](package-manifests.log)（`ls -d ... | wc -l` = 41、`package.npack.json` = 41） |
| 5 | 整包数量：台账同一页出现 **18/37、19/37**，覆盖清单写 **20/37**，进度完成状态写 **18/37** 而同页批次行一路推到 **24/37** | 全部改为引用 **登记表机器字段**：37 条「整包」条目 resolved **24** / open **13**（`meta.wholepack_count = 24`），并保留各处原数字为「2026-10-17 批次快照」 | [`register-fields.log`](register-fields.log)：`整包条目: 37 {'open': 13, 'resolved': 24}` |
| 6 | 台账行尾「总账 18/37 / 19/37」逐行互相矛盾；L13 行写 19/37、L23 行写 18/37、E06 行写 18/37 | 在 §3 表头加「整包计数勘正（2026-10-18）」块，注明行尾字样为当时记录、读数以机器字段为准 | 同上 |
| 7 | 待做登记：进度写「登记表 resolved 21→39、open 68→50、剩余 50 项」；登记表 README 写「resolved 54 / open 35、整包 20/37」；而 `todo-register.json` 机器字段为 61/30，`items` 91 条（`meta.item_count` 89 亦过期，已由 2026-10-18 复核提交 `9cc8560f` 修正为 91） | 进度行改为引用机器字段（**resolved 61 / open 30 / `items` 91**，`meta.item_count = 89` 已过期），并注明目录 README 与 JSON 不一致、以 JSON 为准 | [`register-fields.log`](register-fields.log) |
| 8 | 台账 L29 行只有 2 个单元格（缺「已完成的核查 / 待处理与验收 / 批次」三列），与表头 5 列不符，机器解析失败 | 只补列结构（原内容留在「原生重构状态」列；缺列写明「原行未单列」），不新增结论；解析已验证 37/37 行均为 5 列 | `gen_registry.py` 行解析（[`registry-gen.log`](registry-gen.log)） |
| 9 | 台账 L01/L09/L24/L27/U07 行未写出转换包目录，证据关系断链 | 按在盘包补链接（`ai-tuning-upstream-liuli`、`qhly-upstream-liuli`、`skin-switch-upstream-liuli`、`select-beautify-upstream-liuli`、`wudian-upstream-noname`） | [`package-manifests.log`](package-manifests.log)（目录存在）；登记表「生产接口」列 |
| 10 | 「行数门禁 8215 文件」等历史计数被当作当前值 | 当前结论改为「门禁通过（本次实测 8254 文件，计数随并行提交增长）」，历史计数保留在日期表中 | [`source-lines-gate.log`](source-lines-gate.log) |

**未做（避免编造）**：没有给出「真实整包验收数」这一新数字。24 只是登记表/台账的**标签计数**；
方案阶段 5 要求功能、配置、依赖、组合、保存恢复、目标平台、规则差异逐项通过，因此当前结论写成
**整包验收未完成、逐项复核后再上调**（19/24 个标签行在同一行内仍登记未迁移面）。

## 2 证据模型升级（逐来源）

`docs/operations/evidence/2026-10-17-t9-source-samples/gen_registry.py` 由「台账状态 / 证据目录 /
测试入口」三列升级为方案要求的七段关系：

```
来源功能·配置入口 → 上游证据 → 生产接口 → 测试案例 → 源码/规则/平台版本 → 结果 → 未覆盖项
```

- **脚本生成、计数实测**：全部计数由脚本解析台账行、登记表 JSON、`packages/extensions/*/package.npack.json`、
  测试目录与证据目录得出，README 由脚本重写（不手写）。
- **与台账绑定、不建第二套状态表**：来源行、标签、证据目录、包目录、测试名均解析自
  `docs/migration/upstream-extension-reconstruction.md`；整包状态取自 `todo-register.json`
  中 `tags` 含「整包」条目的 `status` 机器字段。
- **禁止字符串/目录/测试存在即判完成**：脚本不输出完成结论，只报告引用是否存在与是否登记未覆盖项，
  并强制产出**证据缺失清单**。台账行内引用的证据目录/包目录/测试文件本次**缺失 0**。

实测计数（[`registry-gen.log`](registry-gen.log)，按 `--fail-on-missing` 退出 0）：

| 口径 | 值 |
|---|---|
| 来源实例 | 37（U01–U08、L01–L29） |
| 台账标签 | 八项整包 24 / 部分重构 9 / G0 完成 4 / 待重构 0 |
| 登记表整包条目 | resolved 24 / open 13（共 37） |
| 行内引用 | 证据目录、转换包、测试文件缺失 0 |
| 证据缺失清单 | 缺生产接口 **5**（L08/L11/L12/L14/L25）、缺测试案例 **4**（L08/L11/L14/L25）、缺平台结果 **6**（U05/U07/L08/L11/L14/L25） |
| 整包标签存疑 | **19/24**（行内仍登记未迁移功能/表现/平台面） |

登记表：`docs/operations/evidence/2026-10-17-t9-source-samples/README.md`（含逐来源关系表、
三类缺失清单与「标签 ≠ 验收」判据说明）。

**平台结果判据是关键词判据**（wasm/浏览器/冒烟/真实客户端/平台/设备等），命中只说明有平台线索，
不等于平台验收通过；完整平台档位取证仍按方案第 4/5 阶段逐目标补。

## 3 ADR 重复编号与标题错号

- 检查脚本：`scripts/check-adr-index.py`（单文件 200 行级；只读，不重命名/删除历史 ADR）。
- 单测：`scripts/test_check_adr_index.py`，命令
  `python3 -B -m unittest discover -s scripts -p 'test_check_adr_index.py'` → **Ran 9 tests, OK**。
- 实测（[`adr-index-check.log`](adr-index-check.log)）：ADR 文件 732（编号文件 731、唯一编号 702）、
  **重复编号 27 组 / 56 个文件**、标题错号 0、索引坏链 0，**exit 0**。与 2026-10-18 审查快照
  `doc-links-audit.json` 的「27 组重复编号」一致（该文件仅作历史快照，执行时重算）。
- 索引侧：`docs/adr/INDEX.md` 的「编号冲突消歧表」升级为机器校验的**唯一映射依据**，补充
  「唯一标识 / 阻止新增冲突 / 标题错号勘正」三节；新增未登记冲突、登记过期、分组不符、索引坏链、
  标题错号任一出现即 exit 1（可在 CI 直接运行）。
- 标题错号修正 2 处（属笔误，正文与决策未改）：
  - `docs/adr/0446-clan-batch1.md`：标题 `0444` → `0446`（0444 已被 `0444-shiji-sp-cuiyan-yajun-zundi.md` 占用）；
  - `docs/adr/0447-extra-finalize-stats-fix.md`：标题 `0445` → `0447`（0445 已被两个文件占用，且
    `ABI-COORDINATION.md` 的 extra 收尾行引用的是 0447）。
- **未做**：未批量删除/重命名历史 ADR；**未接入 CI**——`.github/workflows/ci.yml` 当前含其他 agent
  未提交改动，按「只提交自己的改动」不改动该文件。接线方式：在现有 `scripts` 检查步骤后追加
  `python3 -B scripts/check-adr-index.py` 与
  `python3 -B -m unittest discover -s scripts -p 'test_check_adr_index.py'`。

## 4 本地链接与锚点

- 工具：本目录 [`check_links.py`](check_links.py)（只读；校验相对路径存在性与锚点存在性，
  锚点支持显式 `<a id="...">` 与 GitHub 风格标题 slug，含中文）。
- 范围与结果（[`links-check.log`](links-check.log)）：四份主稿 + ADR 索引 + 生成登记表，
  **0 坏链/坏锚点，exit 0**；`#native-convergence` 等主锚点已随检查通过。
- 检查器自证有分辨力：探针中「错锚点」「缺文件」被判失败（`exit 1`），正确锚点通过。
- 检查器自身的两处口径修正（不影响文档结论）：① 标题 slug 不折叠连续空格产生的 `--`
  （原实现把 `native-progress-history.md#p2--n08n11-...` 误报为坏锚点，该链接实际有效）；
  ② 目标文件在 `--root` 之外时路径显示不再抛异常。

## 5 行数门禁（必跑两条）

[`source-lines-gate.log`](source-lines-gate.log)：

```
$ python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'
Ran 1 test in 0.004s
OK
$ python3 -B scripts/check-source-lines.py
Source line limit passed: 8254 files <= 2000 lines
exit=0
```

新增文件行数：`scripts/check-adr-index.py` 约 200 行、`scripts/test_check_adr_index.py` 约 110 行、
本目录 `check_links.py` 约 170 行、`gen_registry.py` 约 500 行，均在 2000 行内。

## 6 未完成项 / 边界

1. **整包验收本身未完成**：24/37 只是登记表标签；19 个标签行内仍登记未迁移的功能/表现/平台面，
   13 个来源仍 open。逐来源复核（含平台/表现证据）后才能重述整包结论。
2. **平台结果缺口 6 个来源**（U05、U07、L08、L11、L14、L25）：无平台/真实客户端关键词证据；
   关键词判据不排除「有实机证据但没写进证据目录」的情况，需按来源补证。
3. **生产接口缺口 5 个来源**（L08/L11/L12/L14/L25）：确实还没有转换包（G0 或切片阶段）。
4. **CI 接线未做**：ADR 检查脚本已可跑（exit 0/1，任意 cwd 均可，默认路径按脚本位置解析），
   但未写入 `.github/workflows/ci.yml`（该文件含他人未提交改动）。待接线的补丁：

   ```yaml
         - name: Source files stay within 2,000 lines
           run: |
             python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'
             python3 scripts/check-source-lines.py
         - name: ADR index (duplicate numbers / heading mismatches)   # NC11 新增
           run: |
             python3 -B -m unittest discover -s scripts -p 'test_check_adr_index.py'
             python3 -B scripts/check-adr-index.py
   ```
5. **登记表 README 与 JSON 不一致未改**：`docs/operations/evidence/2026-10-16-todo-register/README.md`
   仍写 resolved 54 / open 35 / 整包 20/37，与 `todo-register.json`（61/30、整包 24/13）矛盾；
   本切片只在主稿与登记表注明「以 JSON 为准」，未改该历史证据正文。
6. ~~**登记表 `meta.item_count = 89` 与实际 91 条不一致**~~ **已修（2026-10-18 复核，提交 `9cc8560f`）**：`meta.item_count` 89→91，与 `items` 91 条一致；本切片当时未改该证据文件。
7. **27 组重复编号未消解为唯一编号**：按 NC11 只做标注与防新增，不重命名历史 ADR。
8. 本切片未跑 cargo/前端/WASM 门禁（不改产品源码）；相关状态以各 NC 证据与进度主稿为准。

## 7 文件清单

| 文件 | 内容 |
|---|---|
| `README.md` | 本文件：结论、依据、未完成项 |
| `check_links.py` | 主稿本地链接/锚点检查器 |
| `source-versions.log` | 版本常量（工作树/HEAD）、HEAD、dirty 数、v322 归因源码行 |
| `package-manifests.log` | 41 个转换包目录与 `package.npack.json` 清单（namespace/package_id/version） |
| `register-fields.log` | `todo-register.json` 机器字段与逐条整包状态 |
| `adr-index-check.log` | ADR 编号检查（27 组重复 / 标题错号 0，exit 0） |
| `adr-index-unittest.log` | ADR 检查脚本单测（Ran 9 tests, OK） |
| `registry-gen.log` | 登记表重建（含 `--fail-on-missing`，引用缺失 0） |
| `links-check.log` | 链接/锚点检查与检查器探针 |
| `source-lines-gate.log` | 行数门禁两条命令 |
| `SHA256SUMS` | 上述文件校验和 |
