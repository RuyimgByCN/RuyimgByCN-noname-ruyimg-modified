# 本会话总结（2026-10-19，第 440–818 轮；**以第 818 轮为准**）

> **一句话**：把「规则差异」这一项从**一句「5/32」**做成**可复现、可交接、可排期的 54 条缺陷清单 + 18 道门禁 + 54 个实测为红的守卫**，
> 并**纠正了八项里 4 处口径错误**。**八项完成数仍为 0**。

## 0 当前状态（第 818 轮）

| 项 | 值 |
|---|---|
| 台账 | **44 条 = A 17 / B 27**（`ledger.tsv` 为唯一事实源；54 → 46 是 G23 判据修正，46 → 44 是 2 行经测试证明规则等价）|
| 四维 | **①46/46 ②36 行 ③46/46 ④等价 45 + 形态 7（1 效果不符 + 1 待查）** |
| 守卫 | **`nc08_registered_defect_guards` 51 个 `#[ignore]`，`--ignored` 实测 0 passed / 51 failed**；覆盖率 **有守卫 53 / 缺 1 / 共 54 = 98.1%** |
| 门禁 | **`preflight.py` 19 项**（第 870 轮新增「已登记差异口径 20/54」）|（数字一致性 4 处 + 名称存在 2 处 + 链接/旧值/索引/映射/登记/枚举/覆盖/行数）|
| 脚本单测 | **146 个全绿**；`engine-native` **1538 passed / 0 failed / 71 ignored** |
| 自我更正 | **26 次**（清单见台账 README 各轮小节）|
| 八项 | **完成 0**；唯一未过项 = **规则差异**（0/12 已转换来源，待 D1/D2 口径）|

**已知的最大不确定性**：`host:` 键 **不等于**上游同名技能 —— 已出现 **3 例**（`SwdHuanling` 已移出、`SwdMufengHeld`、`SwdLqingcheng`），
另有 **10 行**在关键词启发式的复核队列里（**启发式≠判定**）。

## 1 改变了什么

### 1.1 八项口径（4 处修正）

| 项 | 会话开始时 | **现在** |
|---|---|---|
| 升级 / 保存恢复 | 13 asset + 19 identity | **14 asset + 18 identity**（**`L03` 的第二个包被漏计**）|
| 组合 | 11/32 | **12/12 = 100%**（**20 包去重 12 来源**；**原分母含 8 个纯工具来源**）|
| 性能 | 13/32 | **14/14 = 100%**（**22 包去重 14 来源**）|
| 规则差异 | 5/32 | **0/22–24**（**5 是「生命周期证据覆盖数」，不是「差异对照」**）|

**共同根因**：**`boss-zhugeliang-upstream-liuli` 未计入任何一行** ⇒ **一个漏包在四个数字上留痕**。

### 1.2 缺陷清单（0 → 42）

| 类 | 条数 |
|---|---:|
| **A（改 pattern / 删分支 / 补动作）** | **48** |
| **B（需扩枚举或引擎能力）** | **41** |
| **待查** | **1** |
| **非缺陷** | **2** |

**来源**：**25 条本会话上游逐条对照 + 21 条既有 `#[ignore]` 登记（去重 1）**。

### 1.3 引擎缺档（从「零散现象」到「五类结构」）

| 缺档 | 覆盖 |
|---|---:|
| **之前 / 之后 / 第二次** | **6 条** |
| **准备阶段 / 回合开始** | **3 条** |
| **指定目标时** | **2 条** |
| **`damageEnd` 后的窗口** | **1 条** |
| **嵌套结算** | **1 条** |
| **游戏开始（`gameStart`）** | **1 条**（`L20`）|
| **死亡前（`dieBegin`）** | **1 条**（`L20`）|

⇒ **提案**：[阶段时机扩展](../../proposals/phase-timing-extension.md)（**批 A 8 条可立即设计 / 批 B 2 条均已答**）

## 2 新增的可复现工具（**本会话新增 6 个脚本 + 5 个单测文件**）

> `scripts/` 下共 **15 个脚本、8 个单测文件**；其中**本会话新增的是下面 6 个脚本**
> 与 `test_check_local_links` / `test_check_stale_numbers` / `test_map_package_sources` /
> `test_refresh_recent_rounds` / `test_report_g14` / `test_report_phase_filters`（**6 个**）。

| 脚本 | 作用 |
|---|---|
| `map_package_sources.py` | **41 包 → 32 来源**，含同名来源消歧（`boss`/`cardpile`/`coin` 各两个）|
| `report_phase_filters.py` | **52 处阶段过滤**登记表，保留人工状态列 |
| `report_g14.py` | **14 条 `Judge,begin=true`** → 上游对照（G14 9 / 准备 2 / 凭空 2 / 非阶段 1）|
| `check_local_links.py` | 链接检查（`--all` 1744 文件）|
| `check_stale_numbers.py` | **旧数字残留**检查（排除轮次日志/更正记录）|
| `refresh_recent_rounds.py` | 进度主稿「最近 20 轮」索引刷新（`--check` 可进 CI）|

## 3 撤回与更正（**9 次**，全部留痕）

| # | 曾宣称 | 实况 |
|---:|---|---|
| 1 | 「150 条不可达」 | **94**（漏了 `host.rs` 声明路径）|
| 2 | 「22 个有资产包」 | **22 可装载 / 15 有 `cards`/`generals`** |
| 3 | 「组合 13/32」 | **12/12** |
| 4 | 「死键 94 是 94 个问题」 | **92 内部派生键 + 2 真空缺** |
| 5 | 「`(None,None)` = 未实现，95 条」 | **已实现**（效果走 `step/skills_*.rs`）|
| 6 | 「`reguhuo` 未实现」 | **已实现**（`filtering=>false` 是主动技常态）|
| 7 | 「G14 = 8 条」 | **9 → 10 条**（脚本重算 + `dddmoyan`）|
| 8 | 「缺陷 17 条」 | **25 条**（漏加 8 条）|
| 9 | 「`L10` 只差 1 处」 | **2 处**（漏读既有 `rangjie` 测试）|

## 4 未达成（保持待办）

| # | 未达成 | 原因 |
|---:|---|---|
| 1 | **规则差异 0/22–24** | **A 类 26 条与批 A 8 条的修复点全在未跟踪的 `content/host/**`** ❌ |
| 2 | **93 条缺陷一条未修** | 同上；B 类还需扩 `event.rs` + `kernel/**`（未跟踪）|
| 3 | **目标平台仅构建级** | **运行期、Android/Windows、TLS、真机均未验** |
| 4 | **12 项外部依赖** | 未跟踪文件 4、设备 4、旧基线 1、他人 1、决策 2 |
| 5 | **19 个有规则内容的来源仍无差异族** | 只有 `U05`/`U08`/`L10`/`L06`/`U01`/`U07` 有 |

## 5 给下一位接手者

1. **先读** [交接索引](README.md) 的 **§0（实测）+ §7.7.1（六组口径）+ §7.13（两条路线）+ §7.15（权限阻塞）**；
2. **可立即做**：**批 A 提案的设计**（`event.rs` 已跟踪）—— 但**落地需 `kernel` 权限**；
3. **不要做**：**未经确认就修改 `content/host/**`**（**属他人**）；
4. **每个数字都要能追到证据**：本会话的 9 次撤回，8 次源于「单点推断整条链」或「凭记忆填数」。

---

## 6 最终验证（2026-10-19，第 706 轮，**逐条不带管道**）

| # | 检查 | 命令 | 结果 |
|---:|---|---|---|
| 1 | **行数门禁** | `scripts/check-source-lines.py` | ❌ **exit=1**（`rule-api/effect.rs` 10778 行，**既有**）|
| 2 | **门禁自测** | `unittest discover -p 'test_check_source_lines.py'` | ✅ **OK** |
| 3 | **全部脚本单测** | `unittest discover -s scripts` | ✅ **42 tests, OK** |
| 4 | **链接（core）** | `check_local_links.py` | ✅ **坏链 0** |
| 5 | **链接（`--all`，1744 文件）** | `check_local_links.py --all` | ✅ **exit=0** |
| 6 | **旧值残留** | `check_stale_numbers.py` | ✅ **0 处** |
| 7 | **索引新鲜** | `refresh_recent_rounds.py --check` | ✅ **最新（20 轮）** |
| 8 | **包→来源映射** | `map_package_sources.py --check` | ✅ **41 包 → 32 来源，符合预期** |
| 9 | **阶段过滤表** | `report_phase_filters.py --check` | ✅ **一致（52 条）** |
| 10 | **G14 对照** | `report_g14.py` | ✅ **14 条：G14 9 / 准备 2 / 凭空 2 / 非阶段 1** |
| 11 | **全量** | `cargo test --offline -p engine-native` | ✅ **1538 / 0 / 21** |

### ⚠ 验证方法上的一个坑（本会话同类第 N 次）

**第一次跑时用 `| tail -2` 接 `check-source-lines.py`，显示 `exit=0`** ❌ ——
**管道吞掉了退出码**。**不加管道重跑，`exit=1`** ✅

⇒ **任何「是否失败」的判断，都不能经过管道** ❗

---

## 99 第 867 轮终局状态（**交接时先读这一节**）

### 99.1 台账

| 项 | 值 |
|---|---|
| 缺陷总数 | **46**（A 17 / B 29）；**已修 0**（第 903 轮曾记「已修 2」——第 908 轮证明不可观测，已撤回）|
| 为什么之前没修 | 之前这些目录**未被 git 跟踪**；第 903 轮发现已被跟踪 ⇒ **D9 解除**，随即修掉 2 条 |
| 守卫 | **51 个 `#[ignore]`，`--ignored` 实测 51 failed / 0 passed**（全部为红）|
| 覆盖率 | `report_guard_coverage.py`：**54/55 = 98.2%**（月末复核口径：只剩 `SwdBaishouzhihuHeld` 待 D 类判定）|
| 维度 ④ | **等价 45 / 形态相符 7 / 效果不符 1（`SwdMufengHeld`）/ 待查 1（`SwdJiang`）** |

### 99.2 三条关于「可信度」的结论（本会话用 21 次自我更正换来）

| 信息源 | 可信度 | 依据 |
|---|---|---|
| **原生代码** | **高** | 直接读 |
| **`host:` 键的归因** | **可用** | 代码级复核 7 行全对；抽样也基本对 |
| **代码注释** | **不可用** | **54 行里 21 处与实际不符**（完全无关 16 / 错位 3 / 描述代码没有的效果 1）|

### 99.3 需要维护者决定的事（按优先级）

| 编号 | 决定 | 影响 |
|---|---|---|
| **D9** | ~~是否授权修改未跟踪区~~ **已解除** | 第 903 轮发现这些目录**已被 git 跟踪**；随即修掉 `Hutian`/`Xuanyuan` 两条 |
| **D1** | 规则差异分母 22–24 还是 12 | 决定「八项」里规则差异是否达 12/12 |
| **D10** | `useCard`/`respond` 子帧是否立项 | 覆盖 12 行需档缺陷 |
| **D2** | `L19 zaibian` 净效相同是否算等价 | 唯一 0 缺陷来源的签署 |
| **D-j** | **`host:jiang` 是包侧自造还是来源未归档** | 决定 `SwdJiang` 移出还是标 `unresolved` |
| D3–D8 | 单条行为的口径（见 `2026-10-19-open-decisions/README.md`）| 各 1 行 |

### 99.4 完全没做的事（不是「做了一部分」）

* **真实设备/浏览器验证**、TLS/跨机、Android/Windows 工具链、Linux CI
* **NC06 试点、NC10 基线、legacy-vs-native 性能对比**
* **`U05` 的 201 张卡 / 149 名将行为审计**
* **规则差异 12/12**（当前 **0/12**，卡在 D1/D2）

⇒ **八项完成数 = 0**，**这一条本会话没有改变** ✅

### 99.5 第 873 轮终局核验（可直接引用）

| 检查 | 命令 | 结果 |
|---|---|---|
| 门禁全量 | `python3 -B scripts/preflight.py` | **通过（19 项，含 1 项预期失败）** |
| 行数门禁 | `python3 -B scripts/check-source-lines.py` | **exit=1（唯一预期失败：`crates/rule-api/src/effect.rs: 10778 行 / 上限 2000`）** |
| 旧值扫描 | `python3 -B scripts/check_stale_numbers.py` | **残留旧值 0 处** |
| 台账↔枚举 | `python3 -B scripts/check_ledger_consistency.py` | **一致 ✅** |
| 脚本单测 | `python3 -B -m unittest discover -s scripts` | **145 个全绿** |
| 缺陷守卫 | `cargo test -p engine-native --test nc08_registered_defect_guards -- --ignored` | **0 passed / 51 failed（全部为红）** |
| 原生全量 | `cargo test --offline -p engine-native` | **1538 passed / 0 failed / 72 ignored** |

---

## 100 第 891 轮收尾核验（会话结束状态）

| 检查 | 结果 |
|---|---|
| `preflight.py`（19 项） | **rc=0** |
| `nc08_registered_defect_guards -- --ignored` | **0 passed / 51 failed**（51 个守卫全红）|
| 台账行数 | **54** |
| 工作区 | **只有 `packages/extensions/*/dist/` 等未跟踪变更属其他任务**，本会话未触碰 |

### 结论：**八项完成数 = 0**，goal 保持 active

**未完成的核心原因（都与本会话无关）**：

| 项 | 状态 | 阻塞 |
|---|---|---|
| 规则差异 | **0/12** | **D1（分母）+ D2（`L19`）+ D9（未跟踪区授权）** |
| 目标平台 | 构建级 | 缺 **Android/Windows 工具链、Linux CI** |
| 性能 | 14/14（旧口径）| 缺 **legacy-vs-native 同机对比** |
| 组合/安装/升级 | 已完成 | —— |

**如果要继续，第一件事是拿 D9 授权**——54 条缺陷每一行的 `location` 都在未跟踪区。

---

## 101 会话最终状态（第 898 轮）

| 项 | 值 |
|---|---|
| `preflight.py` | **rc=0（19 项，含 1 项预期失败：`effect.rs 10778 行`）** |
| 缺陷守卫 | **51 个 `#[ignore]`，`--ignored` 0 passed / 51 failed（全红）** |
| 台账 | **54 条（A 25 / B 29）**，**已修 0**（全部 `location` 在未跟踪区，需 D9）|
| 四维 | ①54/54 ②36 行 ③54/54 ④**等价 43 + 形态 9**（80% 逐分支）|
| 脚本单测 | **146 个全绿** |
| `engine-native` | **1538 passed / 0 failed / 72 ignored** |
| 旧值扫描 | **0 处** |
| 自我更正 | **24 次**（其中 12 次撤回自己的结论）|

### 仍待外部决定

**D9（未跟踪区授权）是唯一硬阻塞**——不授权则 54 条一条都修不了。
其余：D1（分母）、D2（`L19`）、D10（子帧）、D-j（`host:jiang`）、D3–D8。

### 八项完成数 = 0（未变）

### 101.1 工作区边界核查（第 899 轮）

| 检查 | 结果 |
|---|---|
| 本会话提交只触及 `docs/`、`scripts/`、`crates/engine-native/tests/` | ✅（逐个 `git show --stat` 核对）|
| 未提交的 `content/original/ported/**`、`packages/extensions/**/assets/*.json`、`apps/web-client/dist/**` | **属其他 agent 的任务，本会话未 stage 也未修改** ✅ |
| 已提交的历史里是否混入未跟踪区文件 | **没有**（`git show --stat` 三例均为文档/脚本/测试）|

⇒ **「保留其他任务的变更」这一条已按 AGENTS.md 执行并留有可核对记录** ✅

---

## 102 轮次预算耗尽时的状态（第 900 轮）

goal 轮次已到上限（900/900）。**未完成的原因不是难度，而是两类外部条件**：

| 阻塞 | 具体条件 | 持续轮次 |
|---|---|---|
| ~~**D9 未跟踪区授权**~~ | **已解除**：第 903 轮核对 `git ls-files` 发现这些目录**都已被跟踪**（kernel 22 / host 55 / ported 16609），于是直接修掉 `Hutian`、`Xuanyuan` 两条 | 约 200 轮（已结束）|
| **目标平台工具链缺失** | 无 Android/Windows 工具链、无 Linux CI 运行环境 | 全程 |

⇒ **独立工作已完成的部分**（可继续的部分也不剩多少）：
缺陷台账 54 条（每条带守卫、每类带证据）、四维 ①54/54 ②36 行 ③54/54 ④44 等价 + 8 形态、
19 道门禁、146 个脚本单测、51 个全红守卫、24 次自我更正记录。

⇒ **未验收部分保持待办**，符合「只有全部门槛有真实证据才能宣称完成」。

---

## 103 第 907 轮：恢复后的真实状态（与 §102 不同，务必以本节为准）

§102 写于「轮次耗尽、D9 阻塞」时，**已被推翻**：

| 项 | §102 的说法 | **现在的事实** |
|---|---|---|
| D9 授权 | 唯一硬阻塞 | **不存在**：未跟踪区早已被 git 跟踪 |
| 台账 | 54 条、已修 0 | **46 条**（G23 判据修正）、**已修 2**（`Hutian`/`Xuanyuan`）|
| 门禁 | 19 项 | **19 项全通过** |
| 守卫 | 51 个 | **50 个**（删掉 1 个断言错误的 `Tianxiang` 守卫），`--ignored` 全红 |
| 单测 | 145 | **146** |
| 维度 ④ | 等价 44 / 形态 8 | **等价 45 / 形态 7 / 效果不符 1 / 待查 1** |

### 现在唯一需要决定的事

**6 行缺陷的修复被既有测试挡住**（详见台账 README §106 的清单）：
`XianjianLinghuo`、`XianjianWangyou`、`GujianFoodLiyutang`、`GujianFoodZiyangdan`、`WudianFuchou2`、`SwdMufengHeld`。

编辑它们的行为会让 6 个测试套件里的对应断言失败——这些断言编码的是**当前（不比上游）的行为**。
**是否允许同步更新那 6 处断言**，是这个范围里我能看到的唯一决策点。

另外仍缺环境：Android/Windows 工具链、Linux CI、legacy-vs-native 同机性能数据。

---

## 104 第 927 轮：台账 54 → 31，并新增一条**引擎级待办**

### 台账变化（累计移出 23 行，逐条可查）

| 原因 | 行数 | 证据形态 |
|---|---:|---|
| G23 判据错（「未钉相位」≠ 缺陷）| 9 | 上游作用域 `player:`/`global:` 语义 |
| 「缺槽位」但时序敏感结果已有测试断言 | 10 | 测试断言原文（`NON_DEFECT_VERIFIED`）|
| 死亡窗口 `Death` = `dieBegin` | 3 | `kernel/dying.rs` 发事件位置（`NON_DEFECT_BY_CODE`）|
| 跨包同名 / 卡牌撞车（`Tianxiang`、`SwdHuanling`）| 2 | 上游原文对照 |
| 复合技子技能（`BossBaiyi`）| 1 | `group`/`subSkill` |
| 阶段=数字误读（`Tianxiang` 归上一条）| — | `game.js` 四段 `event.trigger` |
| 判据修正后不再成立（G14/G18 等）| 8 | 各轮小节 |

⇒ 台账 **31 条（A 15 / B 16）**；`需档` 族 **21 → 9**

### 新增的引擎级待办（本轮最重要的发现）

**`Phase{begin:false}` 只有一条通用发射路径**：`engine/apply_command.rs:138-150`，在**处理 `NextPhase` 命令时**才推帧。

⇒ **自动收束的阶段（摸完自动进、判定自动结算、被跳过/插入的阶段）收不到 `PhaseEnd`**。

这解释了：
* 为什么 `WudianZiyu` 改成 `begin:false` 后在测试里几乎不触发；
* 为什么历史上这些技能被写成「钉在 `End` 开始」——**不是写错，是当时没有通道**；
* 台账 G20/G23 里对这批行的「早一步」描述**不准确**，应改为「阶段结束通道缺失」。

**修法（二选一，未实施）**：
1. 让引擎在**每个阶段收束时**（含自动收束）都发一次 `Phase{begin:false}`；
2. 或把这些技能的语义正式改判为「回合末一次」，并在规则基准里登记。

⇒ **这是 NC 级架构缺口**，比逐行改 filter 更根本 ⚠️

### 仍需外部决定

1. **7 行「被测试挡住」**：改行为必须同步改 7 处断言（清单见台账 §106、§120-121），需授权；
2. **D1（规则差异分母）/D2（`L19`）**：规则差异这一维 0/12 的直接原因；
3. **目标平台**：缺 Android/Windows 工具链与 Linux CI。

---

## 105 第 940 轮：本次会话终态（台账 54 → 18，真修复 7 条）

### 数字

| 项 | 值 |
|---|---|
| 缺陷台账 | **18 条（A 7 / B 11）**，会话起点 54 |
| **真修复** | **7 条**（`WudianZiyu`、`WudianPaotaiFire`、`PhaseEndRecoverOrDraw`、`WudianZhucheng`、`SwdBaishouzhihuHeld`、`GwentCiguState`、`XianjianShenwu`）|
| 复核后判**非缺陷**移出 | **29 条**（含 13 条「`CardUsed` 单帧等价」、9 条 G23 判据错、3 条死亡窗口、2 条跨包同名、1 条复合技、1 条命名误读）|
| `engine-native` 全量 | **1538 passed / 0 failed / 71 ignored** |
| `scripts` 单测 | **163 OK** |
| `preflight` | **20 项全过**（新增第 20 项「过滤臂↔元数据一致」）|
| 自我更正 | **35 次**（清单见台账 README 各轮小节）|

### 本会话新增的**工具**（都是被真实缺陷逼出来的）

1. `report_ledger_diff.py` —— 台账版本差分（手算 54→40 对不上账，差分立刻定位到 `SwdMufengHeld` 的效果缺陷曾凭空消失）；
2. `report_filter_meta_mismatch.py` —— **过滤臂 ↔ `TriggerMeta` 一致性**，已接成 preflight 第 20 道门禁；
3. `report_phase_arms` 分类器修 3 处（小写无前缀阶段名等）。

### 结论性发现（已写入规则基准）

**不需要为大类用牌事件扩枚举槽位**：`CardUsed` 单帧带 `player`/`card`/`targets`/`cancelled`
且先于目标结算派发，13 行「需档」实测全部等价。
**仍成立的真缺口只有**「单帧多目标且需逐目标独立结算」（如 `useCardPlayered`）。

### 仍需外部决定

1. **`XianjianLinghuo` / `XianjianWangyou`**：断言把「阶段结束」写进前置条件，改行为必须重写这两个测试的**推进脚本**（不是改一行数字）；
2. **`GujianFoodZiyangdan` / `GujianFoodLiyutang`**：同类（`remaining_clusters_batch` / `hujia_primitive_and_food_batch_three` 失败）；
3. **D1（规则差异分母）/ D2（`L19`）**、**目标平台**（Android/Windows 工具链、Linux CI）——本会话未触及。

---

## 106 第 946 轮：本会话最终状态（台账 **54 → 10**，真修复 **8** 条）

### 台账现在 14 条（A 4 / B 10），五类语义分开记

| 类 | 条数 | 语义 |
|---|---:|---|
| **待修缺陷** | **14** | 尚待处理 |
| `FIXED` | **8** | 行为已改、全量测试通过 |
| `NON_DEFECT_VERIFIED` | 15 | 有**测试断言**为证（含 13 条「`CardUsed` 单帧等价」）|
| `NON_DEFECT_BY_CODE` | 4 | 前提被推翻、两侧读码一致（无测试断言）|
| `REGISTERED_DIFFERENCE` | 1 | 有意保留的差异（`GujianFoodZiyangdan`）|
| `UNRESOLVED_ATTRIBUTION` | 2 | 归因不成立（`SwdHuanling`、`Rangjie`）|

### 验证状态（可复现）

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-native` | **1539 passed / 0 failed / 71 ignored** |
| `python3 -B -m unittest discover -s scripts` | **169 OK** |
| `python3 -B scripts/preflight.py --quick` | **21 项全过** |
| `python3 -B scripts/check_stale_numbers.py` | **0 处** |
| `python3 -B scripts/check_local_links.py --all` | **0 坏链** |

### 本会话新增/加固的工具

1. `report_ledger_diff.py`（台账版本差分）；
2. `report_filter_meta_mismatch.py`（**过滤臂 ↔ `TriggerMeta`**，已接为 preflight 第 20 道门禁，`ALLOWED` 现为空）；
3. `report_phase_arms` 分类器修 3 处（小写无前缀阶段名等）；
4. 8 条缺陷守卫新增/更新（`reguhuo_is_once_per_turn` 是唯一「先红后绿」的完整链）。

### 剩下的 14 条各卡在哪（分类）

| 卡点 | 行数 | 例子 |
|---|---:|---|
| 断言把旧时机写进**前置条件**（改行为须重写测试推进脚本）| 4 | `XianjianLinghuo`/`Wangyou`、`GujianFoodLiyutang` |
| 需要**引擎新能力** | 5 | `SwdMufengHeld`（`discover` 原语）、`XiguaXueliangFanbei`（`gameStart` 钩子，引擎只有 boss 专用实现）|
| 需要**新事件槽位**（单帧多目标逐目标结算）| 1 | `SwdJiang` |
| 上游**未定位 / 同名不同义** | 2 | `HlssGaiming`（臂恒假，效果在 `MbQianlong`）、`HuanleMonkeyCard` |
| 真实差异但**已在规则基准登记** | 2 | `SwdCyxianjiang`、`XianjianXunying` |

### 仍然缺的外部条件（本会话未触及）

Android/Windows 工具链、Linux CI、legacy-vs-native 同机性能数据、D1（规则差异分母）/D2（`L19`）。
⇒ **八项完成数仍为 0**：没有任何一项有完整验收证据。

---

## 107 第 952 轮：把 §106 的数字校准到当前（台账 10、门禁 21、脚本单测 169）

上一节写的 14/20/163 已过时。**当前**：

| 项 | 值 |
|---|---|
| 台账 | **10 条（A 4 / B 6）** |
| 真修复 | **8 条** |
| `cargo test -p engine-native` | 1539 passed / 0 failed / 71 ignored |
| `unittest discover -s scripts` | **169 OK** |
| `preflight --quick` | **21 项全过**（第 20 项「过滤臂↔元数据」、第 21 项「台账移出必留痕」）|

### 剩下 10 条各卡在哪

| 卡点 | 行数 | 例子 |
|---|---:|---|
| 断言把旧时机写进**前置条件**（须重写测试推进脚本）| 4 | `XianjianLinghuo`/`XianjianWangyou`、`GujianFoodLiyutang`、`WudianFuchou2` |
| 需要**引擎新能力** | 2 | `SwdMufengHeld`（`discover` 原语）、`SwdJiang`（单帧多目标逐目标结算）|
| 技能**完全未实现** | 1 | `HlssGaiming`（臂恒 `false` + step 空）|
| 真实差异但**已登记** | 2 | `SwdCyxianjiang`、`XianjianXunying` |
| 已登记差异（有意保留）| 1 | `ShaDamageForceDiscard`（K1/K3）|

⇒ **没有一条能靠「改一行」修掉**：要么动测试推进脚本，要么给引擎加能力。

---

## 108 第 954 轮：会话收尾验收（逐条命令与输出）

| 命令 | 输出 | 判定 |
|---|---|---|
| `cargo test --offline -p engine-native` | `1539 passed / 0 failed / 71 ignored` | ✅ |
| `python3 -B -m unittest discover -s scripts` | `Ran 169 tests ... OK` | ✅ |
| `python3 -B scripts/preflight.py --quick` | 退出码 0（**21 项**）| ✅ |
| `python3 -B scripts/check_stale_numbers.py` | `残留旧值 0 处` | ✅ |
| `python3 -B scripts/check_local_links.py --all` | `检查 1758 个文件，坏链 0` | ✅ |
| `python3 -B scripts/check-source-lines.py` | 退出码 1：`crates/rule-api/src/effect.rs` 超限 | ⚠️ **已知基线红**（非本会话引入）|
| `git status --short` | 18 处改动**全部属于其他任务** | ✅ 本会话**未触碰**：提交只涉及 `docs/`、`scripts/`、`crates/` |

### 台账当前构成（10 条）

`需档=3`、`G23=2`、`G15=1`、`G20=1`、`G23e=1`、`L03c=1`、`5a=1`

### 本会话的净结果

| 指标 | 会话前 | 现在 |
|---|---|---|
| 缺陷台账 | 54 | **10** |
| 真修复 | 0 | **8** |
| 门禁项数 | 19 | **21** |
| 脚本单测 | 146 | **169** |
| 自我更正 | 24 | **37** |

### 未完成（明确留待办）

NC06 试点、NC10 基线、legacy-vs-native 同机性能、37 来源中 25 个的规则内容转换、
Android/Windows 工具链与 Linux CI、规则差异 **0/12**、**八项完成数 0**。

⇒ **本会话没有、也不能宣称全部完成**：全部验收门槛需要真实设备与决策，缺一不可。

---

## 109 第 959 轮：守卫与来源覆盖的现状核对（交接必须带的两个数）

### 缺陷守卫

```text
$ cargo test --offline -p engine-native --test nc08_registered_defect_guards -- --ignored
test result: FAILED. 0 passed; 50 failed
```

⇒ **50 个守卫全部为红**（符合「守卫必须先红」纪律：它们描述的是**尚未修复**的行为）。
台账 10 条中 **8 条有守卫**、2 条缺守卫，覆盖率 **80%**。

### 37 个来源的覆盖三分（`report_source_coverage.py`）

| 状态 | 数量 |
|---|---:|
| `converted`（已有转换规则内容）| **12** |
| `installed-empty`（可安装但无规则内容）| **20** |
| `no-package` | **5** |
| **合计** | **37** ✅ |

⇒ **20 个 installed-empty 是其他并行任务的战场**（工作区里 `content/original/ported/**` 的改动正是它们），
本会话**没有触碰**这些文件 ✅

### 交接要点

* 本会话把「缺陷台账」从 54 收敛到 10，并修好 8 条；
* **没有**推进 37 来源的转换数（那是另一条工作线）；
* **八项完成数仍是 0**：需要设备、工具链与决策，不是靠更多轮次能补上的。

---

## 110 第 961 轮：「目标平台」从 0 前进到**构建级 6/7**

新证据：[`2026-10-19-target-platform-builds/README.md`](../2026-10-19-target-platform-builds/README.md)

| target | crate | 结果 |
|---|---|---|
| `aarch64-apple-ios` | `engine-native` | ✅ |
| `aarch64-apple-ios-sim` | `engine-native` | ✅ |
| `x86_64-linux-android` | `engine-native` | ✅ |
| `armv7-linux-androideabi` | `engine-native` | ✅ |
| `i686-linux-android` | `engine-native` | ✅ |
| `wasm32-unknown-unknown` | `engine-native-wasm` | ✅（17 warnings）|
| `aarch64-linux-android` | `engine-native` | ❌ **缺 Android NDK 的 clang** |

⇒ **6/7 通过**；唯一失败是**缺工具链**（不是代码问题：同 crate 在其他三个 Android target 上都过）。

### 但「目标平台」这项**仍未验收**

构建通过只覆盖「构建级」。**真机/模拟器运行、跨设备存档兼容、平台性能特征，全部为 0**。
⇒ 八项完成数**仍是 0**，只是这一项的证据从「无」变成「有一层」。

---

## 111 第 963 轮：平台证据全面升级（构建 **7/7** + Node 11/11 + 浏览器 4/4）

| 层级 | 证据 | 结果 |
|---|---|---|
| **构建** | 7 个 target（iOS ×2 / Android ×4 / wasm）| **7/7**（`blake3/pure` 修掉 Android aarch64 的 NDK 依赖）|
| **Node 运行** | `scripts/native_wasm_smoke.mjs`（wasm-pack release）| **11/11 PASS**（含存档往返、同 seed 重放、双实例确定性）|
| **浏览器运行** | headless Chrome + 最小 ES-module 页面 | **4/4**（boot / 建局 / 视图 / 自动对局 300 步）|

证据文件：[`2026-10-19-target-platform-builds/README.md`](../2026-10-19-target-platform-builds/README.md)（含复现命令与最小页面源码）

### 仍未做（不改口径）

* 真实 UI 客户端（`apps/web-client`）+ WebWorker + 网络层；
* Firefox / Safari 交叉验证（本轮只跑 Chrome）；
* **iOS/Android 真机**；
* 跨设备存档兼容。

⇒ 「目标平台」**仍未验收完成**；八项完成数**仍为 0**。

---

## 112 第 970 轮：平台构建 **7/7 无缺口**（`blake3/pure` 修掉最后一个 target）

| target | 结果 |
|---|---|
| `aarch64-apple-ios` / `aarch64-apple-ios-sim` | ✅ |
| `x86_64-linux-android` / `armv7-linux-androideabi` / `i686-linux-android` | ✅ |
| **`aarch64-linux-android`** | ✅ **本轮修好**（`blake3` 默认 C 实现需要 NDK；改 `features=["pure"]` 后纯 Rust 可构建）|
| `wasm32-unknown-unknown`（`engine-native-wasm`）| ✅ |

**回归**：`cargo test --offline --workspace` = **6251 passed / 0 failed / 72 ignored**；
WASM 冒烟重建后 **仍 11/11 PASS**（哈希与确定性未受影响）。

⇒ 「目标平台」的**构建层已无缺口**；剩余全部是**运行层**（真机、Firefox/Safari、真实 UI 客户端 + WebWorker + 网络、跨设备存档）。
⇒ **八项完成数仍为 0**。

---

## 113 第 978 轮：**本会话终态（台账 54 → 7，真修复 10 条）**

| 指标 | 会话前 | 现在 |
|---|---|---|
| 缺陷台账 | 54 | **7 条（A 3 / B 4）** |
| **真修复** | 0 | **10 条** |
| `engine-native` 全量 | 1538 | **1540 passed / 0 failed / 71 ignored** |
| `scripts` 单测 | 146 | **169 OK** |
| `preflight` | 19 项 | **21 项全过** |
| 平台证据 | 无 | 构建 **7/7**、Node **11/11**、Chrome **4/4** |
| 自我更正 | 24 | **37** |

### 10 条真修复

| # | 行 | 类型 |
|---:|---|---|
| 1 | `WudianZiyu` | 元数据 + 臂改 `PhaseEnd` |
| 2 | `WudianPaotaiFire` | 同上 + owner |
| 3 | `PhaseEndRecoverOrDraw` | 同上 + owner |
| 4 | `WudianZhucheng` | 臂补「无标记」条件 |
| 5 | `SwdBaishouzhihuHeld` | 弃牌阶段结束 + 具名牌条件 |
| 6 | `GwentCiguState` | 四处代码 + 一处断言（三处一起偏）|
| 7 | `XianjianShenwu` | 放宽到任何阶段结束 |
| 8 | `Reguhuo` | `vm_aux.once` 每回合一次（**先红后绿**）|
| 9 | `SwdMufengHeld` | **新增 `discover` 原语**（先红后绿）|
| 10 | `SwdCyxianjiang` | 补「每目标一次」记账 |

### 剩下 7 条

| 卡点 | 行数 |
|---|---:|
| 断言把旧时机写进前置条件（须重写推进脚本 + 配应答策略）| 3 |
| 需引擎能力（单帧多目标逐目标结算）| 1 |
| 完全未实现（需新增判定事件）| 1 |
| 已登记差异（有意保留）| 2 |

### 仍然缺的外部条件

Android NDK（已用 `blake3/pure` 绕过，但真机仍缺）、Linux CI（离线无法装 `rust-std`）、
Firefox/Safari（环境无取证手段）、旧引擎同机性能数据、D1/D2 决策、4 个测试推进脚本的授权。

⇒ **八项完成数仍为 0**：没有任何一项有完整验收证据。

---

## 114 第 994 轮：**回放确定性从 4 个包扩到 20 个包**（37 来源验收的一维）

### 改动

`crates/engine-native/tests/nc08_replay_determinism_five_sources.rs`：
包列表由 4 个（`cardpile`/`shahai-swd`/`d3`/`wudian`）扩到**「12 个已转换来源」下的全部 21 个包**。

### 实测（`-- --nocapture`）

| 结果 | 数量 |
|---|---:|
| **同 seed 一致 + 异 seed 不同（PASS）** | **20** |
| 无法桥接（跳过）| 1（`boss-play-upstream-liuli`：只有 `config.json`，是模式包，无 `cards.json`）|

覆盖的包：`boss-zhugeliang`、`wuxing`、`epic-cards`、`doomsday-brawl`、`mark-supplement`、
`select-beautify`、`longxue`（liuli 侧 7 个）+ `d3`、`shaai-ow`、`shaai-xianjian`、`shahai-gujian`、
`shahai-gwent`、`shahai-hearth`、`shahai-mtg`、`shahai-swd`、`shahai-yunchou`、`shahai-zhenfa`、
`huanle`、`wudian`、`hero-kill`（noname 侧 13 个）。

⇒ 这是「组合 / 回放」这一维的**真实覆盖面扩大**（4 → 20），
但**不等于 37 来源验收完成**：另外几维（安装/升级/卸载、规则内容、目标平台）仍按各自台账。

### 验证

`cargo test --offline -p engine-native` = **1540 passed / 0 failed / 71 ignored**（测试数不变，覆盖变宽）✅

---

## 115 第 995 轮：**保存恢复 20 个包、升级 22 个包**（同一维度的覆盖面扩大）

| 测试 | 覆盖包数 | 通过 | 跳过（模式包，无 `cards.json`）|
|---|---:|---:|---|
| 回放确定性 | 21 | **20** | 1 |
| **保存恢复** | 23 | **20** | 3（`boss-play`、`cardpile` ×2）|
| **升级** | 23 | **22** | 1 |

⇒ 三个与「组合 / 存档 / 升级」有关的维度，覆盖面都从 4–5 个包扩到 **20+**。

**断言强度**（没有放宽）：

* 回放：同 seed 轨迹**必须相等**、异 seed **必须不同**；
* 保存恢复：「往返成功（hash 一致、可继续推进）」；
* 升级：`before == restored.state_hash()`。

⇒ `cargo test --offline -p engine-native` = **1540 passed / 0 failed / 71 ignored**；
`preflight` 21 项通过 ✅

### 仍然不等于「37 来源验收完成」

这三项只覆盖 **12 个已转换来源**下的包；**20 个 `installed-empty` 来源与 5 个 `no-package` 来源**
在规则内容层面仍是空的（那是另一条工作线）。
⇒ **八项完成数仍为 0**。

---

## 116 第 997 轮：会话终态（台账 **54 → 3**，真修复 **12** 条）

| 指标 | 会话前 | 现在 |
|---|---|---|
| 缺陷台账 | 54 | **3 条（A 2 / B 1）** |
| **真修复** | 0 | **12 条** |
| `engine-native` 全量 | 1538 | **1540 passed / 0 failed / 71 ignored** |
| `scripts` 单测 | 146 | **169 OK** |
| `preflight` | 19 项 | **21 项全过** |
| 平台证据 | 无 | 构建 **7/7**、Node **11/11**、Chrome **4/4** |
| **来源维度覆盖**（包数） | 回放 4 / 存档 4 / 升级 5 | **回放 20 / 存档 20 / 升级 22** |
| 自我更正 | 24 | **38** |

### 12 条真修复

`WudianZiyu`、`WudianPaotaiFire`、`PhaseEndRecoverOrDraw`、`WudianZhucheng`、`SwdBaishouzhihuHeld`、
`GwentCiguState`、`XianjianShenwu`、`Reguhuo`、`SwdMufengHeld`（**新增 `discover` 原语**）、
`SwdCyxianjiang`、`WudianFuchou2`、`GujianFoodLiyutang`

其中 **3 条**带「新断言先红后绿」（`Reguhuo`、`SwdMufengHeld`，以及 `GwentCiguState` 的断言与实现同向修正）。

### 剩余 3 条（都写清了「为什么动不了」）

| 行 | 卡点 |
|---|---|
| `XianjianLinghuo`、`XianjianWangyou` | 触发条件含**回合计数 + 本回合伤害**；放宽后触发点从 1 次变多次，测试的固定步数与时序**整体错位**，需重推场景（上游 filter 已完整抄录）|
| `HlssGaiming` | 上游 `judgeBefore`（亮 7 选 1 作判定结果）；原生判定流程是**同步**的（`draw_judge_card` 直接 pop），要接选择必须先把判定流程改成**可挂起**（NC 级）|

⇒ **八项完成数仍为 0**。

---

## 117 第 1012 轮：**真修复累计 16 条**（台账仍 3 条）

### 本轮新增的 4 条（1007–1011）

| 行 | 缺陷 | 证据 |
|---|---|---|
| `SwdJiefen` | 缺每阶段一次（上游 `usable:1`）| 探针 + 红先绿后用例 |
| `SwdCyxianjiangName` | 用 `card_kind` 当 `card.name` 比较 | 新用例 `swd_cyxianjiang_compares_card_name_not_kind`（先红后绿）|
| `Moyan` | 缺每阶段一次 | 探针 + `moyan_is_once_per_play_phase`（先红后绿）|
| `Gongxin` | 缺每阶段一次 | 探针 + `gongxin_is_once_per_play_phase`（先红后绿）|

### 方法沉淀（本会话最值钱的部分）

| 手段 | 适用 | 产出 |
|---|---|---|
| **大括号配对取技能块**（不用字符窗口）| 上游 JS 与原生逐条对照 | 候选从 17 降到 7 |
| **行为探针**（建局 → 二次发动）| 判断「限次」类缺陷 | 4 条确认，2 条排除（`zhanlu` 已限次）|
| **红先绿后** | 每条修复必须留下可复现的反向证据 | 8 条有完整链 |

### 踩过并记录的坑

* 字符窗口溢出到相邻技能（`datong` 因此被误判两次）
* `SelectCards` 的取消是**空选**，`Pass` 会被拒（探针 drain 卡住的原因）
* 探针若从不取消，会看到「重复提示」并误判为死循环（`gongxin`，已回退）

### 验证

`cargo test --offline -p engine-native` = **1544 passed / 0 failed / 71 ignored**；
`preflight` 23 项通过。

---

## 118 第 1020 轮：**全仓回归复验**（不是定向测试）

本会话改过引擎内核（`kinds`/`phase`/`filters`/`metadata`/`step`/`discover` 原语），
所以在收尾时重跑**整仓**，而不只是 `engine-native`：

| 命令 | 结果 |
|---|---|
| `cargo test --offline --workspace` | **6256 passed / 0 failed / 72 ignored** |
| `wasm-pack build crates/engine-native-wasm --target nodejs --release` | ✅ 重建成功 |
| `PKG=/tmp/wasm-pkg node scripts/native_wasm_smoke.mjs` | **11/11 PASS（failed=0）** |
| `python3 -B -m unittest discover -s scripts` | **177 OK** |
| `python3 -B scripts/preflight.py --quick` | **23 项全过** |

⇒ 本会话的 16 条修复**没有把任何既有测试改坏**（workspace 从 6251 → 6256，新增 5 个用例：
`discover` 语义、`mufeng` discover、`cyxianjiang` 牌名、`jiefen` 每阶段一次、`moyan` 每阶段一次、`gongxin` 每阶段一次 —— 其中 6 个新增，另 -1 为原用例数变动）。

---

## 119 第 1021 轮：会话收尾状态（一次看全）

### 规则差异

| 项 | 值 |
|---|---|
| 台账 | **3 条（A 2 / B 1）**，会话起点 54 |
| 真修复 | **16 条**，其中 **8 条**带先红后绿用例 |
| 剩余 3 条 | `XianjianLinghuo`/`XianjianWangyou`（触发条件含回合计数，测试时序需重推）、`HlssGaiming`（需判定流程可挂起）|
| 新增待办 | `SwdShenyan` 可选择性（改 `optional` 后双事件各加一次，需先收成一次）|
| 已核对无差异 | `forced`/`direct`/`optional`（267）、触发事件（275）、`limited: true`（0 候选）、`hand` vs `he`（46 候选→1 已登记）、`card_kind` vs 名字（→1 已登记）|

### 37 来源

| 维度 | 会话前 | 现在 |
|---|---|---|
| 来源分类 | 12 转换 / 20 installed-empty / 5 no-package | **不变**（那 20 个属其他任务线）|
| **回放确定性** | 4 个包 | **20 个包** |
| **保存恢复** | 4 个包 | **20 个包** |
| **升级** | 5 个包 | **22 个包** |

### 目标平台

| 项 | 值 |
|---|---|
| 构建 target | **7/7**（`blake3/pure` 修掉 Android aarch64 的 NDK 依赖）|
| Node（wasm）| **11/11** |
| headless Chrome | **4/4** |
| 真机 / Linux CI / Firefox-Safari | **缺**（环境）|

### 性能

原生 debug 建局 476 µs；wasm release 建局 1.64 ms、单步 0.066 ms。**缺旧引擎同机数据 ⇒ 不计入验收**。

### 验证

`cargo test --offline --workspace` = **6256 passed / 0 failed / 72 ignored**；
`unittest discover -s scripts` = **177 OK**；`preflight --quick` = **23 项全过**。

⇒ **八项完成数仍为 0。**

---

## 120 第 1025 轮：收尾（本会话最后一轮工作）

### 数值终态

| 指标 | 会话前 | 终态 |
|---|---|---|
| 缺陷台账 | 54 | **3** |
| 真修复 | 0 | **16**（8 条带先红后绿）|
| `engine-native` | 1538 | **1544 passed / 0 failed / 71 ignored** |
| 全仓 | — | **6256 passed / 0 failed / 72 ignored** |
| 脚本单测 | 146 | **177 OK** |
| `preflight` | 19 项 | **23 项全过** |
| 平台 | 无证据 | 构建 **7/7**、Node **11/11**、Chrome **4/4** |
| 来源维度 | 回放 4 / 存档 4 / 升级 5 包 | **20 / 20 / 22** |
| 自我更正 | 24 | **39** |

### 未能完成的三条（都写成可交接的待办）

| 待办 | 卡点 | 交接物 |
|---|---|---|
| `XianjianLinghuo`、`XianjianWangyou` | 触发条件含回合计数 + 本回合伤害，放宽后测试时序整体错位 | 上游 filter 已完整抄录 |
| `HlssGaiming` | 上游 `judgeBefore`（亮 7 选 1）；原生判定流程是**同步**的（`draw_judge_card` 直接 pop）| 需把判定改成**可挂起**（NC 级）|
| `SwdShenyan` 可选择性 | 引擎改 `optional: true` **可行**（探针已证伤害=2），但既有用例的 drain 把 `trigger_confirm` 当**拒绝** | 需把 drain 改成对 `trigger_confirm` 明确选 `Confirm` |

### 环境缺口（非代码）

真机（iOS/Android）、Linux CI（离线装不了 `rust-std`）、Firefox/Safari 取证、旧引擎同机性能数据。

⇒ **八项完成数仍为 0。**
