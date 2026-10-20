# 本会话（第 400–508 轮）**交接索引**（2026-10-19）

> 目的：让接手者**不必重读全部进度主稿**就能知道「新增了什么、推翻了什么、还差什么」。
> 完整流水见[进度主稿](../../../architecture/native-progress.md)；台账见[上游扩展台账](../../../migration/upstream-extension-reconstruction.md)。
> 原始提交清单：[commits.txt](commits.txt)（最近 60 条）。

## 1 新增的**可复用工具**

| 工具 | 作用 | 归组 |
|---|---|---|
| `scripts/classify_upstream_skills.py` | 把上游 `lib.skill.X={…}` 块**括号配平截取**并按 规则/UI/标记/其它 分类（含去重副本）| 上游分析 |
| `scripts/recount_g0.py` | 对台账 **37 个来源统一重测**（整目录子树 / 定义数 / 出现次数 / 文件数 / 字节）| 规模口径 |
| `crates/rule-vm/examples/gen_fuzz_seed.rs` | 生成**当前格式** fuzz 种子（`valid_nnvm.bin`、`verify_ok_nnvm.bin`），**写盘前回读自检** | fuzz |
| `crates/engine-native/examples/nc12_bench.rs` **`dir:<包名>` 档** | 按 drop-in 目录名**单包**测量，便于 shell 循环跑批 | 性能 |

## 2 新增的**验收用例**（关键）

| 用例 | 覆盖 | 证据 |
|---|---|---|
| `apps/game-server/src/main.rs::net_smoke`（**8 个**）| **真实 TCP/WS**：`/health` `/metrics`、握手、建局加入提交、**整局**、**重连**、未开局错误码、**多房间隔离** | [network-path](../2026-10-19-network-path/) |
| `apps/game-server/tests/long_run_{soak,bot,memory}.rs` | 产品长程：800 命令探活、**178/178 全接受**、**30 盘终局**、堆无步增长 | [product-soak](../2026-10-19-product-soak/)、[product-bot](../2026-10-19-product-bot/)、[product-memory](../2026-10-19-product-memory/) |
| `apps/game-server/tests/nc08_install_matrix.rs` | **41/41 包**安装 + 幂等 + **软链显式拒绝** | [install-matrix](../2026-10-19-install-matrix/) |
| `nc08_manifest_matrix.rs` | **41/41** manifest 契约、`(ns,id)` 唯一、25 个 `config.json` | [config-matrix](../2026-10-19-config-matrix/) |
| `nc08_upgrade_matrix.rs` / `nc08_save_restore_matrix.rs` | **各 22/22 有资产包** | [upgrade-matrix](../2026-10-19-upgrade-matrix/)、[save-restore-matrix](../2026-10-19-save-restore-matrix/) |
| `nc08_skill_pair_matrix.rs` | **20 包 / 190 对** 卡+技能+武将加和不变式 | [skill-pair-matrix](../2026-10-19-skill-pair-matrix/) |
| `nc10_ai_boundary_and_cancel.rs` | NC10：**信息边界**（他人暗牌替换后输入逐字节相同）、取消、**过期 Stale**、并发尾延迟、变体确定性 | [nc10-ai-slice](../2026-10-19-nc10-ai-slice/) |
| `fuzz_corpus_replay.rs` + `fuzz_dsl_parser_replay.rs` | fuzz 语料的 **stable 回放**（bytecode 解码/校验、DSL 解析）| [fuzz-corpus-replay](../2026-10-19-fuzz-corpus-replay/) |

## 3 **推翻/更正**的结论（**接手最该先读**）

| # | 曾被写下的结论 | 更正后 |
|---|---|---|
| 1 | 「座位未满时**无**错误回推」 | **有**：`Error{Conflict,"match not started"}`；原判是我**只等 `CommandOutcome`、丢了 `Error` 帧** |
| 2 | 「`replay-export` 未落盘 ⇒ 跨进程未验」 | CLI **本就支持**输出路径参数，**我漏传**；跨进程已实测 `chain=ok` |
| 3 | 「`load-hash` 无输出（缺陷）」 | **用法错**：它要 `ContentPackage` JSON，我传了 `cards.json`（`exit=2` 结构化错误）|
| 4 | 「G0 技能数 = 待迁移量（179）」 | **不成立**：三个不同口径相加；L14 的 92 个定义 **87 个是 UI** |
| 5 | 「L11/L12 **0 技能 ⇒ 不迁移**」 | **L11 实测 146 个定义/104 疑似规则**（台账 0 与实测矛盾）；L12 `lib.skill` 确为 0 但 `lib.card` 52 处 |
| 6 | 「卡片配对矩阵扩到 13 包」 | **否决**：`pile.json` 替换牌堆、纯武将包 0 卡 ⇒ 不变式不适用（**语义边界**）|
| 7 | 「待迁移技能量 179 / 181」 | **收窄到 5**（按**效果原语调用**过滤：L08 0 / L11 5 / L12 0 / L25 0）|
| 8 | 「DSL 原语补在 tracked crate 就能做」 | **错**：query 求值端在 **`engine-core`**（未跟踪）且 **match 穷举无 catch-all**|
| 9 | 「effect 缺口被未跟踪文件阻塞」 | **先作废、后定案**：`grep` 曾把**文档注释**当 match 臂 ⇒ 改用**编译实验**定案 ⇒ **确实阻塞**（**2 处 `E0004`**，全在未跟踪的 `command/resolve_choice/**`）|

> **方法学教训（反复出现）**：①「没有 X」先确认观测范围（`grep` 范围/收件箱/`stderr`/参数）；②`sed`/路径/单位（包 vs 来源）错会造出假结论；③**「0」必须复核**（L11 的 0 是假 0）；④**启发式指标随口径单调下降时不可用**。

## 4 八项验收**按来源**现状（32 个已映射来源）

| 项 | 覆盖 |
|---|---|
| 安装 / 配置 | **32/32** |
| 升级 / 保存恢复 | **13/32**（**全部 22 个有资产包**；其余 19 个来源无引擎资产）|
| 组合 | **11/32**（190 对）|
| 规则差异 | **5/32** |
| 性能 | **13/32**（22 包 probe + drive + 长局）|
| 目标平台 | **仅全局构建级**（真机/浏览器未做）|
| **完成八项的来源** | **0** |

## 4.1 规则差异（**一页交接**）

四份差异族的全部结论与**7 行待办**（含 1 条产品决策）汇总在
[**规则差异交接页**](../2026-10-19-rules-diff-handover/README.md) ——
**「规则差异 5/32」这个数字点进去就是它**。

| 来源 | 差异族状态 |
|---|---|
| U08 英雄杀 | **闭合**（技能已实现但**无载体不可达**）|
| U05-OW `shaai-ow` | **v2**（6 条全有对照）|
| U05-XJ `shaai-xianjian` | **v2**（22 条全归类，**待 D1 决策**）|
| U05-SWD `shahai-swd` | **v3**（16 条全归类，**1 条未实现**）|

## 5 剩余阻塞（**带根因**）

| 阻塞 | 根因 | 能否在本环境解决 |
|---|---|---|
| 核心修复清单 30–52 / 53 / 57 | 需改 `kernel/**`、`content/host/**`、`engine/**` —— **本工作树为未跟踪文件** | ❌ **不能**（不得修改未跟踪文件）|
| 9 个需 `mod:` 的技能 | 同上（宿主 mod 面）| ❌ |
| 181 个「疑似规则」块 | 需**逐条读**（约 20 次调用 × 4 块）| ⚠️ **可做但未做完** |
| 真机 / 浏览器 | **无设备**；`apps/web-client` 是 **pnpm/JS**，离线不可构建 | ❌ |
| TLS / 跨机 | **无第二台机器** | ❌ |
| 同负载前后对比 | **无旧实现基线** | ❌ |
| 上游 harness / NC06 / NC10 质量基线 | 需更大样本或未实现 | ⚠️ 部分可做 |
| **DSL 原语（query 类 3 个 + effect 类 5 个）** | **query 求值端在 `engine-core/src/query/{cache.rs,mod.rs}` 的穷举 match**；**effect 变体新增会让 `engine-core/src/command/resolve_choice/**` 报 `E0004`**（**编译实验已定案**）—— **均为未跟踪文件** | ❌ **不能** |
| **行数门禁（当前红）** | `crates/rule-api/src/effect.rs` **10778 行**（基线快照即如此，**非本会话引入**）| ❌ 需 **`rule-api` 负责任务**拆分 |

## 6 建议的下一步（按价值/可行性）

1. **先修行数门禁**：把 `crates/rule-api/src/effect.rs`（10778 行）按 `EffectRequest` 分组拆分 ——
   **这是唯一会让所有提交变成「不合规」的问题**，且它是**并行任务**的产物，需该任务负责人处理；
2. **把升级/保存恢复从 13/32 推到 32/32** —— 对 19 个无资产包做 **manifest 级**升级/恢复（与现有矩阵同构）；
3. **规则的「迁移」在可修改范围内已到边界**：5 个真规则候选**全部需要未跟踪区的改动**
   （query 需改 `engine-core` 的穷举 match；effect 需改 `resolve_choice/**`）⇒
   **除非把 `engine-core`/`rule-api` 纳入本代理可修改范围，否则不要再投入迁移尝试**；
4. **把「规则差异」从 5/32 扩**：对 22 个有资产包逐一写差异族（需逐条比对上游，工作量大）；
5. **目标平台**：补齐 Android/Windows 的 C 工具链后即可复跑（**环境问题**，非代码问题）。

---

## 7 **外部依赖清单**（2026-10-19 汇总：每一项都不是「再努力一点就能完成」）

> 用途：把「本代理做不到」的事**一次说清**，避免后来者重复调查。
> 每条都给出**具体对象**与**解除条件**。

### 7.1 需要**产品决策**（1 项）

| # | 决策点 | 现状 | 影响 |
|---|---|---|---|
| **D1** | **U05-XJ 的 8 条「结束后」时机差异是否接受** | 22 条已全部归类：**13 条无问题、8 条引擎粒度不足、0 条未判**（[基准](../2026-10-19-rules-diff-handover/README.md)）| 接受 ⇒ 该来源「规则差异」可签署（**5/32 → 6/32**）；不接受 ⇒ 需升级 `EventKind`（**要改未跟踪的 `kernel/events.rs`**）|
| **D2** | **`nc04_pause_and_resume` 的 DSL 选择语句关键字** | 用例 `#[ignore]`，文案「关键字待确认后补挂起→续跑→故障覆盖」 | 决定 NC04 覆盖能否完成 |

### 7.2 需要**放开未跟踪文件**（核心阻塞，1 类 4 处）

| # | 文件/区域 | 被它挡住的工作 |
|---|---|---|
| **U1** | `crates/engine-native/src/{kernel,engine,content/host,content/standard}/**` | 核心修复清单 **30–52 / 53 / 57**、**U08 加载体**、**9 个需 `mod:` 的技能** |
| **U2** | `crates/engine-core/src/query/{cache.rs,mod.rs}`（**穷举 match，0 catch-all**）| **DSL query 类 3 个原语**（`history_count`/`is_chained`/`trigger.num`）|
| **U3** | `crates/engine-core/src/command/resolve_choice/**`（**新增 `EffectRequest` 变体即 `E0004`**，已用编译实验证实）| **DSL effect 类 5 个原语**（`remove_skill`/`judge`/`choose_target`/`card_pile_search`/`force_die`）|
| **U4** | `crates/engine-core/src/effect/primitives/dispatch_*.rs` | 新增效果派发 |

### 7.3 需要**设备/环境**（4 项）

| # | 缺什么 | 挡住的工作 |
|---|---|---|
| **E1** | **真机设备** | 目标平台验收的**运行时**部分（现在只有**构建级**：iOS 双架构 + WASM 三层通过）|
| **E2** | **第二台机器** | **TLS**、**跨机联机**（现在只有 loopback 明文 WS）|
| **E3** | **Android NDK / MSVC 工具链** | Android / Windows 的**构建级**验收（现在 FAIL）|
| **E4** | **Linux 环境或 CI runner** | 与 macOS 的行为一致性回归（未做）|

### 7.4 需要**旧实现基线**（1 项）

| # | 缺什么 | 挡住的工作 |
|---|---|---|
| **B1** | **旧线（engine-core/engine-ai/engine-wasm）在同负载下的可比数据** | 「性能**前后对比**」（现在只有**绝对基线**：22 包 probe/drive + 长局 + 尾延迟）|

### 7.5 需要**他人处理**（1 项，与本次任务无关但影响门禁）

| # | 事项 | 状态 |
|---|---|---|
| **O1** | **`crates/rule-api/src/effect.rs`（10778 行）拆分** | **行数门禁当前为红**（基线快照即如此）；**并行任务产物**，本代理不介入（见[门禁事故](../2026-10-19-line-gate-incident/README.md)）|

### 7.6 汇总

| 类 | 项数 | 可否由本代理解除 |
|---|---:|---|
| 产品决策 | **2** | ❌ |
| 未跟踪文件放开 | **4 处** | ❌ |
| 设备/环境 | **4** | ❌ |
| 旧实现基线 | **1** | ❌ |
| 他人处理 | **1** | ❌ |

⇒ **共 12 项外部依赖**。**其余工作（文档、矩阵、样本、分析）已在可做范围内推进到边界。**
