# 本会话（第 400–508 轮）**交接索引**（2026-10-19）

> 目的：让接手者**不必重读全部进度主稿**就能知道「新增了什么、推翻了什么、还差什么」。
> 完整流水见[进度主稿](../../architecture/native-progress.md)；台账见[上游扩展台账](../../migration/upstream-extension-reconstruction.md)。
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

## 6 建议的下一步（按价值/可行性）

1. **把升级/保存恢复从 13/32 推到 32/32** —— 对 19 个无资产包做 **manifest 级**升级/恢复（与现有矩阵同构）；
2. **逐条读 181 个块**（每轮 4 块）→ 产出**确定的待迁移技能清单**；
3. **把「规则差异」从 5/32 扩**：对 22 个有资产包逐一写差异族（需逐条比对上游，工作量大）；
4. **目标平台**：补齐 Android/Windows 的 C 工具链后即可复跑（**环境问题**，非代码问题）。
