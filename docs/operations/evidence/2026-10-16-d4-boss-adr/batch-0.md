# D4 Boss 关卡流程 · 批次 0（引擎开工清单）

**日期**：2026-10-16　**待做 ID**：`T11-BOSS-STAGE-SUBSYSTEM`　**ADR**：[0783](../../../adr/0783-boss-stage-flow-subsystem.md)　**机器可读**：[batch-0.json](batch-0.json)

**性质**：文档交付；**禁止**本批改 `crates/**`（实现留给引擎线）。本清单对应 ADR §6 **B0 子系统骨架**——无内容技落包、无对局可达语义变化（默认 `kind=None`）。

**核验基线（只读，2026-10-16）**：

| 项 | 值 |
|---|---|
| `STANDARD_RULESET_VERSION` | **290**（`crates/engine-native-content/src/content/standard.rs`） |
| `ARCHIVE_FORMAT_VERSION` | **78**（`crates/engine-native-kernel/src/archive.rs`） |
| `HostBehavior` 当前最高 tag | **410**（`SwdMinjing`）；下一可用起点 **411** |
| `GameMode::Boss` | `{ boss, chongzheng, fellows }`（v84 fellow 已有）；**尚无** `BossStageState` / `change_boss` |
| `TurnState` | B0 **不触碰**（流程态挂模式层，不进回合态） |
| 上游权威路径 | `archive/upstream/noname/apps/core/mode/boss.js`（本工作区 archive 可能未检出；行号以既有勘点/证据为准） |

---

## 首批实现项（6 项 · 机制，非内容技）

| # | ID | 机制 | 上游符号 / 路径 | 建议落地 | HostBehavior tag | 触碰版本/golden |
|---|---|---|---|---|---|---|
| 1 | `B0-STAGE-STATE` | `BossStageState` 数据模型 + 默认空态挂 `GameMode::Boss`（或并列字段） | `_status.shidianyanluo_level` / `_mengpo` / `_mengpodie`（boss.js ≈2512+、3376+、3418+、4134）；ADR §3.1 | `state.rs` + `GameMode` 扩展；默认 `kind=None`、`level=0`、旗 false、禁将空、`wing_armed=0`、`pending=None` | **不挂**（内核模式态，非内容技） | 字段入快照 → 存档 **必须 +1**；哈希含字段 → 规则通常 **+1**；14 golden 探针后重录或证无位移 |
| 2 | `B0-ARCHIVE-ROUNDTRIP` | 存档编解码往返（含往旧档拒绝/迁移策略登记） | （模式态序列化面；无单一上游符号） | `archive.rs`：`ARCHIVE_FORMAT_VERSION` 78→**79**；Boss 分支读写新字段 | 不挂 | **存档 +1**；旧档策略须有单测/注释 |
| 3 | `B0-HASH-PROBE` | canonical / 摘要哈希纳入 `BossStageState`；版本回退探针 | （原生哈希管线） | `engine.rs` / `snap.rs` 摘要路径；`STANDARD_RULESET_VERSION` 290→**291**（若位移源=版本+字段） | 不挂 | **规则 +1**；golden：**先探针再重录**（禁只改期望值） |
| 4 | `B0-CHANGE-BOSS-API` | 内部 `change_boss` / `reinit_boss` 原语（装配重挂 + Boss 指针更新；**无**技能表挂名） | `changeBoss`（boss.js ≈1688）；对照 versus 替补 / `boss_nbianshen` 装配轮换先例 | `engine`/`kernel` 私有或 `pub(crate)` API；复用 `apply_general_assembly`；RNG 预留位但 B0 可不消费 | 不挂 | 仅内部 API 且默认路径不可达 → 可不单独升规则；若测夹具进默认对局面则跟 #3 |
| 5 | `B0-ADD-FELLOW-API` | 正式 `add_boss_fellow`（对齐现有 `debug_add_fellow` / fellows 队尾） | `addBossFellow` / `game.addFellow`（boss.js ≈1680） | 收敛 debug 入口与正式 API；座位扩张口径对齐 ADR 0781 环访问器 | 不挂 | 同 #4 |
| 6 | `B0-BAN-QUERY-STUB` | 禁将集合查询 + 乱入旗只读查询（空实现：恒未禁、旗恒 false） | 还人禁将字面名单（boss.js ≈3436：`luxun`/`re_luxun`/…/`kongrong`）；`boss_newhuanren*`（≈3404/3448） | `is_boss_general_banned(id) -> bool`；`mengpo_spawned/died` getter；**B0 不写入、不拦截选将** | 不挂 | 无语义拦截 → 可不升版本；写入面留给 B3 `T11-BOSS-HUANREN` |

**本批明确不做（留给后续批）**：隐藏链八技（B1）、bianshen4（B2）、还人选将链/孟婆乱入写入（B3）、青赤白翼（B4）。内容技落包时 HostBehavior 自 **tag 411** 起顺序编号（只增不改）。

---

## 定向测试夹具草案

| 建议测试名 | 断言要点 |
|---|---|
| `boss_stage_state_default_none_roundtrip` | 新局 Boss 模式 `kind=None`；encode→decode 字段全等 |
| `boss_stage_archive_format_v79_rejects_v78_without_migration` | 按选定策略：拒收或缺省填充；行为有单测锁定 |
| `boss_stage_hash_includes_stage_fields` | 仅改 `level`/`mengpo_spawned` 等 → 摘要变化；默认空态与升版前基线关系由探针说明 |
| `boss_change_boss_api_reinit_updates_pointer` | 调用 `change_boss` 后 `GameMode::Boss.boss` 与装配技能表更新；座位环不破坏 |
| `boss_add_fellow_api_appends_and_marks_boss_side` | fellow 入 `fellows` 尾；胜负侧判定含 fellow（既有 v84 口径） |
| `boss_ban_query_stub_always_false` | B0 下任意 id `is_boss_general_banned==false`；旗 getter 为 false |
| `ruleset_290_to_291_golden_probe`（若升规则） | 回退 290 复跑 golden；位移源=版本字段则重录 14 行并记证据 |

全量验收（实现批，非本文档批）：`cargo test --offline -p engine-native --no-fail-fast` + Gate 1b；证据目录建议名见 ADR：`YYYY-MM-DD-d4-boss-stage-b0-skeleton/`。

---

## 排除边界（本批与 D4 后续批均不得顺手做）

| 项 | 待做 / 裁决 | 说明 |
|---|---|---|
| **D5 / `ylyg*` / 龙血玄黄** | D5；`D5-LX-MIGRATION` | 关卡子系统不得为加载 `ylyg*` 开特例；诸葛亮整将另批 |
| **hp Infinity** | 随 D5 | B0–B4 禁止顺手改 hp 表达 |
| **四季映姬 / `boss_yingzhong`** | `T11-BOSS-SHIKIEIKI-EXCLUDED`（**永久排除**） | 不建外部将池；还人映姬候选池未就绪则登记不可达 |
| **庞统涅槃/浴火** | `T11-BOSS-PANGTONG-EXTERNAL`（wait_d5） | 非 D4 |
| **地藏「大愿」** | `T11-BOSS-DIZANG-DAYUAN`（**非 D4**） | 判定改写 AI 态度面；不依赖本子系统 |
| **地藏「无量」建局摸 3** | `T11-BOSS-DIZANG-WULIANG-GAMEDRAW`（**非 D4**） | gameDrawAfter/enterGame；勿并入 B0/B3 |
| **孟婆「往事」打出面** | v270 永久登记差异（父项已拆分关闭） | 不阻塞子系统 |
| **孟婆 `mengpodie` / 回魂** | 并入 `T11-BOSS-HUANREN`（**B3**，非 B0） | 旗字段可在 B0 建模，**写入与回魂牌堆在 B3** |
| **试炼态 `boss_qingmu` init 壳 / `boss_bianshen` 开局壳** | 不落包 | 条目以目标将代位 |
| **`cards.json` / npack** | 禁止 | 流程在内核模式态 |

交叉参考：[ambiguous-resolutions](../2026-10-16-todo-register/ambiguous-resolutions.md)、[todo-register](../2026-10-16-todo-register/README.md)、[D4–D8 决定](../2026-10-16-d4-d8-decisions/README.md)。

---

## 上游勘点快查（实现对照）

权威：`archive/upstream/noname/apps/core/mode/boss.js`（详见[本目录 README](README.md)）。

| 族 | 关键符号（约行） |
|---|---|
| API | `addBossFellow`≈1680；`changeBoss`≈1688 |
| 十殿态 | `shidianyanluo_level`≈2512+；`_mengpo`≈3418+；`_mengpodie`≈4134 |
| 还人 | `boss_newhuanren`≈3404；`boss_newhuanren2`≈3448；禁将≈3436 |
| 隐藏链 | `boss_qingmu2/3`、`boss_chiyan2/3`、`boss_xuanlin2/3`、`boss_baimang2/3`（≈6071–6742）→ **B1** |
| 翼 | `boss_baiyi`≈5325；`boss_chiyi`≈7105；`boss_qingyi`≈7323 → **B4** |
| bianshen | `boss_bianshen4`≈8779 → **B2** |

---

## 本清单验证

| 项 | 结果 |
|---|---|
| ADR 0783 / INDEX | 已有（提交 `40754758`）；本批仅补 batch-0 与 README 链 |
| `crates/**` | **未改** |
| `native-progress.md` | **未改** |
| 规则/存档/golden | 文档批不递增、不重录 |
