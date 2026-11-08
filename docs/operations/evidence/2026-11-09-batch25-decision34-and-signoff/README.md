# Batch 25：裁决 34 逐来源剩余子将／子技、单机战役状态机闭环与 37 来源原八项签署复核（2026-11-09）

- **核验日期**：2026-11-09
- **基线 HEAD**：`5b271751c`（Batch 24 文档同步，最后代码提交 `cc50ee177`，exact-head CI Run [`42507366640`](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/42507366640) `10/10 PASS`）
- **版本门禁（零升号）**：原生规则版本 **`333`**、存档格式 **`NGSA 107`**、`content-schema` **`5`**、`rule-vm` ABI **`759`**、`WS8` 不变；标准 14 场 golden 哈希零变化（`replay_scenarios` `3/3`）。

---

## 1. 裁决 34 剩余子将／子技、HUD 标记与单机战役脚本状态机落地（第三节第 2 项续）

在 Batch 24（26 来源前端五标签面板 + 天气粒子覆盖层 + `extensionSurfacesJson` 存读档恢复）之上，Batch 25 对照上游 `archive/upstream/noname/extension/` 与 `archive/upstream/noname-liuli/extension/` 源码，将剩余中小体量来源的子将／子技、14 项 HUD 标记与 5 个单机战役状态机补齐至内核、装载器、前端运行时与集成测试：

### 1.1 原生内核与装载器结算（`engine-native-kernel` & `content-native-loader`）

1. **`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs`**（接入 [`commit_up37.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37.rs) `try_commit_decision34`）：
   - **`L29` 龙血玄黄（`longxue-upstream-liuli`）4 技能道心阈值解锁与原生结算**：
     - 对齐上游 `extension/龙血玄黄/extension.js` 道心（`mbqianlong_marks` `0..=99`）阈值阶梯：`>= 25` 解锁 `mbcmqingzheng`（清正，由宿主 `HostBehavior` 承载）；`>= 50` 解锁 `mbcmjiushi`；`>= 75` 解锁 `mbcmfangzhu`；`>= 99` 解锁全部 4 技（含 `mbjuejin`）。
     - 提供 `is_daoxin_unlocked`、`resolve_mbcmjiushi`（酒诗：门控 `mbqianlong_marks >= 50`，正面朝上时翻面并置 `jiu_active = true` / `mbcmjiushi_jiu_used = 1`；背面朝上时翻回正面并从牌堆/弃牌堆检索 1 张锦囊牌置入手牌，`mbcmjiushi_trick_gained = 1`）、`resolve_mbcmfangzhu`（放逐：门控 `mbqianlong_marks >= 75`，受伤害时令目标翻面并摸 `lost_hp = (max_hp - hp).max(1)` 张牌）、`resolve_mbjuejin`（绝进：门控 `mbqianlong_marks >= 99` 且限定每局一次 `mbjuejin_used`，赋予 2 点护甲并将场上【闪】/【桃】/【酒】移入 `"xiang"` 扩展牌堆）；未达阈值越权调用写 `mbqianlong_locked_guard = 1` 安全早退不污染状态。
   - **`L10` 极略（`epic-cards-upstream-liuli`）十常侍 `shichangshi` 双子技**：
     - `resolve_k_danggu`（党锢 `k_danggu`）：回合/阶段开始初始化十常侍 10 位掩码 `k_danggu_roster_mask = 0x3FF`（若未耗尽），按 `k_danggu_pick_index` 激活当值宦侍槽 `k_danggu_active_eunuch` 并摸 1 张牌。
     - `resolve_k_mowang`（殁亡 `k_mowang`）：濒死（`EventKind::Dying`）时从 `k_danggu_roster_mask` 退场当前宦侍位；若仍有剩余宦侍则清除 `dying`、体力恢复至 1、轮换下一宦侍槽并摸 1 张牌；10 位全退场后置 `k_mowang_exhausted = 1` 且不再重置掩码。
   - **`U06` 欢乐卡牌（`huanle-upstream-noname`）【苦无】交互式弃牌／失体力结算**：
     - `resolve_kuwu_interactive`：校验非连环传导伤害（`kuwu_from_chain == 0`），目标可弃置 1 张 `he` 区（手牌或装备区，支持 `kuwu_prefer_equip` 优先弃装备并卸载装备技）牌入弃牌堆，或选择失去 1 点体力（`EventData::LoseHp`）。
   - **`U07` 玩点论杀（`wudian-upstream-noname`）剩余 3 技能**：
     - `resolve_diaobingqianjiang`（调兵遣将：亮出牌堆顶 `market` 牌，按座位序与存活角色手牌交换，首张剩余归使用者、其余置入弃牌堆）、`resolve_duoqi`（夺气：弃置 1 张 `he` 区牌摸 1 张牌并对目标置 `duoqi_draw_penalty = 1`）、`resolve_youzhan`（诱战：对目标造成 1 点伤害并摸 1 张牌）。
   - **`U08` 英雄杀（`hero-kill-upstream-noname`）门神护卫与塔防波次**：
     - `resolve_yxs_menshen2_guard`（门神护卫：显式核验 `yxs_menshen2` 标记或 `yxs_menshen2_granter` 旗标存在且授标者存活非己时，将 1 点伤害转移给授标者；无标无旗标安全早退不误伤 0 号位）、`resolve_yxs_tafang_wave`（塔防波次推进：推进波次 `1..=5`、维护城防 `tafang_fort_hp`、发放补给摸牌，第 5 波置 `tafang_Cleared = 1`）。
   - **`L20` 标记补充（`mark-supplement-upstream-liuli`）14 项 HUD 标记与 `L28` 雷霆万钧（`thunder-upstream-liuli`）3 旋律技**：
     - `L20_MARK_IDS` / `L20_MARK_SPECS` 14 项标记（`longmark` 龙印、`biaoji` 标记、`danlie` 胆裂、`fengyin` 封印、`fuhun` 父魂、`guimen` 鬼门、`hejie` 和解、`huxiao` 虎啸、`jianying` 渐营、`jinfan` 锦帆、`kuangbao` 狂暴、`lunhui` 轮回、`mohua` 魔化、`niepan` 涅槃）由 `resolve_l20_mark_dispatch` 写入 `vm_aux.marks` 与 `l20_mark:<key>`（其中 `niepan` 涅槃同步将体力恢复至 3）。
     - `resolve_th_chongzou`（重奏：摸 1 张牌并置 `th_chongzou_triggered = 1`）、`resolve_th_huanyin`（幻音：根据 `th_melody_mask` 花色位数摸牌或回复 1 点体力）、`resolve_th_xuanlv`（旋律：累积四花色和弦位掩码，集齐 `0x0F` 时造成 1 点雷电伤害并摸 1 张牌）。
   - **`U02`/`L03`/`L11`/`L19`/`L21` 五大单机战役阶段状态机**：
     - `resolve_boss_baonu_stage`（`U02`/`L03` BOSS 狂暴达阈值转入二阶段、`max_hp` 抬升至 `>= 6`、联动剑阁攻城器械并摸 2 张牌）、`resolve_taixu_expedition_step`（`L11` 太虚幻境层数推进、赐福加牌与灵兽协战 1 点伤害）、`resolve_doomsday_infection_step`（`L19` 末日乱斗感染满 3 层尸变或使用血清净化回复 1 点体力）、`resolve_martial_formation_step`（`L21` 武皇 5v5 替补登场回满体力摸 4 张牌并触发连环阵型加摸）。

2. **`crates/content-native-loader/src/decision34_closure.rs`**（导出于 [`crates/content-native-loader/src/lib.rs`](../../../../crates/content-native-loader/src/lib.rs)）：
   - `decision34_closure_extension_pack()` 构建原生 `ExtensionPack`，将全部 19 项 `VmSkillDecl` 绑定至真实 `EventKind`（`PhaseBegin`、`Damage`、`TurnBegin`、`Dying`、`CardUsed`）与 `VmFilterPolicy`（`OwnerIsTarget` / `OwnerIsSource`），并装配 `L10` 十常侍（`shichangshi`：`k_danggu`/`k_mowang`）与 `L28` 12 名 `th_*` 变体武将（`L28_THUNDER_GENERALS`）。
   - 提供 `longxue_full_unlock_report(marks)`（4 技能完整解锁报告）与 `verify_u08_standard_skill_bindings(catalog)`（校验 `U08_STANDARD_SKILL_REFS` 31 项跨包标准技能在 `Catalog` 中 100% 命中）。

3. **前端运行时与面板扩展（`apps/web-client`）**：
   - [`apps/web-client/src/engine/extensionSurfaceRuntime.ts`](../../../../apps/web-client/src/engine/extensionSurfaceRuntime.ts) 与 [`apps/web-client/src/components/battle/ExtensionSurfacePanel.vue`](../../../../apps/web-client/src/components/battle/ExtensionSurfacePanel.vue) 补齐 `L20_HUD_MARK_SPECS`（14 项标记徽章与 `+1` 交互）、`unlockLongxueSkills`（道心技能解锁实时展示 `mbcmqingzheng/mbcmjiushi/mbcmfangzhu/mbjuejin`）、`advanceTafangWave`（`U08` 塔防波次 `#1/5 -> #2/5` 推进）、`resolveCoinHonorTitle`（`U04/L05` 金币荣誉称号阶梯）及 `L07` 全能搜索 7 条新词条检索。

---

## 2. 37 来源（46 扩展包）原八项签署逐来源复核（第三节第 4 项）

按[两仓扩展台账 §2.3](../../../migration/upstream-extension-reconstruction.md) 原八项完成条件（①功能、②配置、③依赖、④组合、⑤保存恢复、⑥目标平台、⑦性能、⑧规则差异）逐来源复核：
- **28/37 来源仓内 7 项（①②③④⑤⑦⑧）全部达到「完成」**：`U02, U03, U04, U06, U07, U08, L01, L02, L03, L04, L05, L06, L07, L09, L10, L11, L12, L13, L15, L16, L17, L18, L21, L22, L23, L26, L28, L29` 仅余第⑥项移动端（Android/iOS）物理真机验证。
- **9/37 来源在①功能／⑧规则差异保持「部分」**：`U01`（剩余 69/93 武将）、`U05`（杀海拾遗剩余武将技，其中 `shahai-swd` 91/91 张卡已 100% 完成）、`L08`（十周年 UI Spine `.skel/.atlas` 骨骼二进制）、`L14`（手杀 UI Spine 骨骼特效）、`L19`（末日乱斗剩余子技）、`L20`（标记补充剩余专属武将技，14 项 HUD 标记已完成）、`L24`（特效测试 Spine 骨骼资产）、`L25`（祖安武将剩余恶搞武将）、`L27`（选将美化 Spine 骨骼二进制）。
- **整包八项签署计数**：因第⑥项移动端真机（第三节第 5 项外部硬件条件）未就绪，37 来源整包八项签署如实保持 **`0/37`**。逐来源明细已同步至[两仓扩展台账 §3.1/§3.2/§3.2.1](../../../migration/upstream-extension-reconstruction.md) 与 [37 来源八项签署核对表](../2026-10-19-source-signoff-32/README.md)。

---

## 3. 第五节门禁全集实跑记录

| 门禁项 | 执行命令 | 结果 |
|---|---|---|
| 行数单测 | `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | **2/2 OK**（0.011s） |
| 源码行数门禁 | `python3 -B scripts/check-source-lines.py` | **PASS**：`9195 files <= 2000 lines; Rust non-test files <= 1000 effective lines` |
| 原生三层依赖棘轮 | `python3 -B scripts/check-native-layering.py` | **PASS**：`0 upward references <= 0` |
| 裁决 34 新增集成测试 | `cargo test --offline -p engine-native --test misc decision34_rules_and_campaigns::` | **7 passed; 0 failed; 0 ignored** |
| 原生三 crate 全量测试 | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native` | **`2,214 passed; 0 failed; 6 ignored`**（较 Batch 24 `+7 passed`，规则探针 `0 ignored`） |
| 独立目录 Gate 1b `-D warnings` | `CARGO_TARGET_DIR=/tmp/noname-next-b25-warn RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets && rm -rf /tmp/noname-next-b25-warn` | **PASS（0 warnings，跑后已按裁决 23 清理 `/tmp/noname-next-b25-warn`）** |
| `wasm32-unknown-unknown` 编译检查 | `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm` | **PASS（0 errors）** |
| 全工作区测试 | `cargo test --offline --workspace --no-fail-fast` | **`7,606 passed; 0 failed; 8 ignored`**（较 Batch 24 `+7 passed`） |
| `conformance` | `cargo run --offline -p conformance` | **`passed=19, ALL PASS`** |
| `desktop-client` | `cargo test --offline -p desktop-client` | **`20 passed; 0 failed; 1 ignored`** |
| 跨目标确定性门禁 | `python3 -B scripts/check-native-wasm-cross-target.py && bash scripts/cross-target-hash.sh` | **`ALL CROSS-TARGET CHECKS PASSED`** |
| 前端单测 / 类型 / 构建 / Worker 冒烟 | `pnpm --dir apps/web-client test && pnpm --dir apps/web-client typecheck && pnpm --dir apps/web-client build:native && pnpm --dir apps/web-client smoke:native` | **`468 passed; 0 failed`**、`typecheck` **0 错**、`build:native` **PASS**、`smoke:native` **`83/83 PASS`** |
| 真实 Headless Chrome 裁决 34 渲染面与帧率 | `node apps/web-client/scripts/decision34-browser-surfaces.mjs docs/operations/evidence/2026-11-09-batch25-decision34-and-signoff` | **`passed: true`**，面板 `1160×409`，`unlockedSkills: "mbcmqingzheng/mbcmjiushi"`，`l20MarkBadges: "L20 标记补充 (14项)：龙印 ×1"`，`tafangWave: "#2/5"`，`honorTitle: "初入茅庐"`，60 帧 `requestAnimationFrame` **`p50 = 16.66 ms, p95 = 16.67 ms, p99 = 16.67 ms, max = 16.67 ms, approx_fps = 60.0 FPS`**，`consoleErrors: []`（见 [`decision34-browser-surfaces-report.json`](decision34-browser-surfaces-report.json) 与 [`decision34-browser-surfaces.png`](decision34-browser-surfaces.png)） |
| 收口 `preflight.py` | `python3 -B scripts/preflight.py` | **21/21 全部通过 ✅** |
| 裁决 25 发布与 exact-head CI | 单命令作用域 `pre-push` 守卫（原钩子 `sha256: e349af1ec5ccc1742df7e7d91d387ed2ef086f7c864d8cf7372d0ef66dd49ca7` 零改动）推送代码提交 `e079045f259eb440f135eca65bcf5d361bbc31ce -> refs/heads/native-convergence`（CI Run [`42516616541`](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/42516616541)，[`ci-run-42516616541-e079045f2.json`](ci-run-42516616541-e079045f2.json)） | **首次运行（Attempt 1）10/10 全部通过 `conclusion: success`**（`native-web` `128840919043`、`build-windows-latest` `128840919224`、`build-ubuntu-latest` `128840919252`、`build-macos-latest` `128840919269`、`desktop-ubuntu-latest` `128840919287`、`desktop-macos-latest` `128840919288`、`perf-ubuntu-latest` `128840919305`、`desktop-windows-latest` `128840919309`、`gates-ubuntu` `128840919323`、`perf-macos-latest` `128840919351` 全绿） |
