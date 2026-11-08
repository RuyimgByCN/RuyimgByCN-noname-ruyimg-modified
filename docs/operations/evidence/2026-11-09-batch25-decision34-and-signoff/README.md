# Batch 25：裁决 34 逐来源剩余子将／子技、单机战役状态机闭环与 37 来源原八项签署复核（2026-11-09）

- **核验日期**：2026-11-09
- **基线 HEAD**：`5b271751c`（Batch 24 文档同步，最后代码提交 `cc50ee177`，exact-head CI Run [`42507366640`](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/42507366640) `10/10 PASS`）
- **版本门禁（零升号）**：原生规则版本 **`333`**、存档格式 **`NGSA 107`**、`content-schema` **`5`**、`rule-vm` ABI **`759`**、`WS8` 不变；标准 14 场 golden 哈希零变化（`replay_scenarios` `3/3`）。

---

## 1. 裁决 34 剩余子将／子技、HUD 标记与单机战役脚本状态机落地（第三节第 2 项续）

在 Batch 24（26 来源前端五标签面板 + 天气粒子覆盖层 + `extensionSurfacesJson` 存读档恢复）之上，Batch 25 对照上游 `archive/upstream/noname/extension/` 与 `archive/upstream/noname-liuli/extension/` 源码，将剩余中小体量来源的子将／子技、14 项 HUD 标记与 5 个单机战役状态机补齐至内核、装载器、前端运行时与集成测试：

### 1.1 原生内核与装载器结算（`engine-native-kernel` & `content-native-loader`）

1. **`crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34.rs`**（接入 [`commit_up37.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37.rs)）：
   - **`L29` 龙血玄黄（`longxue-xuanhuang-upstream-liuli`）4 技能道心阈值解锁与原生结算**：
     - 对齐上游 `extension/龙血玄黄/extension.js` 道心（`daoxin`）阈值阶梯：`< 30` 仅解锁 `mbcmqingzheng`；`30..60` 解锁 `mbcmqingzheng` + `mbcmjiushi`；`60..90` 解锁 `mbcmqingzheng` + `mbcmjiushi` + `mbcmfangzhu`；`>= 90` 解锁全部 4 技（含 `mbjuejin`）。
     - 提供 `unlocked_longxue_skills_for_daoxin`、`sync_longxue_daoxin_skills`、`resolve_mbcmqingzheng`（清正：弃同花色手牌并弃目标手牌，若少于目标则造 1 伤，`daoxin +15`）、`resolve_mbcmjiushi`（酒诗：需要 `daoxin >= 30`，翻面获得酒势或受伤正面朝上摸牌，`daoxin +10`）、`resolve_mbcmfangzhu`（放逐：需要 `daoxin >= 60`，目标翻面并摸 `lost_hp` 张牌，`daoxin +15`）、`resolve_mbjuejin`（决进：需要 `daoxin >= 90` 且每局一次，全场体力化为 1、赋予护甲并造牌转化【决斗】）；未达阈值越权调用安全早退不污染状态。
   - **`L10` 极略（`jilue-upstream-liuli`）十常侍 `shichangshi` 双子技**：
     - `resolve_shichangshi_danggu`（党锢 `k_danggu`）：回合开始叠加 `k_danggu` 印记、摸牌并对目标施加 `k_danggu_ban`（限制非本回合使用基本牌）。
     - `resolve_shichangshi_mowang`（殁亡 `k_mowang`）：失去最后手牌或濒死转生时摸 2 张牌、回复 1 点体力并轮换十常侍激活宦侍槽。
   - **`U06` 欢乐卡牌（`huanle-kapai-upstream-noname`）【苦无】交互式弃牌重铸**：
     - `resolve_huanle_kuwu_interactive(ctx, owner, chosen_card)`：显式校验 `chosen_card` 位于 `owner` 手牌后弃置入弃牌堆（发射 `CardsLost` / `CardsDiscarded`），摸 1 张牌并递增 `kuwu_recast_count`；非手牌 ID 拒答返回 `false`。
   - **`U07` 武将点将（`wudian-upstream-noname`）剩余 3 技能**：
     - `resolve_wudian_diaobingqianjiang`（调兵遣将：亮出牌堆顶牌、交换手牌令盟友摸牌）、`resolve_wudian_duoqi`（夺气：弃置延时锦囊令目标本回合 `duoqi_forbid_sha = 1`）、`resolve_wudian_youzhan`（诱战：引诱对决承受伤害+1 并摸 2）。
   - **`U08` 英雄杀（`yxs-upstream-noname`）门神护卫与塔防波次**：
     - `resolve_yxs_menshen2_guard`（门神护卫：为受伤害同伴转移 1 点伤害并摸 1 张牌）、`resolve_yxs_tafang_wave`（塔防波次推进：推进波次 `1..=5`、扣减或修复城防并按波次发放奖励摸牌）。
   - **`L20` 标记补充（`mark-supplement-upstream-liuli`）14 项 HUD 标记与 `L28` 雷霆万钧（`thunder-upstream-liuli`）3 旋律技**：
     - `L20_HUD_MARK_KEYS` 14 项标记（`longyin`、`fengyin`、`ying_mark`、`biao_mark`、`danxue`、`tengjia_vine`、`renwang_shield`、`bagua_seal`、`yinyang`、`hufu_seal`、`pojun_break`、`qixing_star`、`dawu_fog`、`kuangfeng_gale`）读写原语 `apply_l20_hud_mark` / `query_l20_hud_mark`。
     - `resolve_thunder_skill`：实现 `th_chongzou`（重奏：重置同回合旋律技限次并摸 1）、`th_huanyin`（幻音：切换奇偶调式摸牌或弃牌）、`th_xuanlv`（旋律：按调式点数造成 1 点雷电伤害并摸 1）。

2. **`crates/content-native-loader/src/decision34_closure.rs`**（导出于 [`crates/content-native-loader/src/lib.rs`](../../../../crates/content-native-loader/src/lib.rs)）：
   - 登记并校验 `L29` 4 技能（`mbcmqingzheng`、`mbcmjiushi`、`mbcmfangzhu`、`mbjuejin`）、`L10` 十常侍（`shichangshi`：`k_danggu`/`k_mowang`）、`U07` 12 名 `wd_*` 武将与 3 剩余技能、`U08` 31 项标准映射技能、`L20` 14 项 HUD 标记、`L28` 12 名 `th_*` 武将。
   - 实现 `CampaignStageState` 与 `advance_campaign_stage`，统一驱动 `U02`（守卫剑阁波次与器械耐久）、`L03`（BOSS 二阶段狂暴）、`L11`（太虚幻境层数推进与奇遇赐福）、`L19`（末日乱斗尸潮感染净化）、`L21`（武皇 5v5 替补登场）。

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
| 收口 `preflight.py` | `python3 -B scripts/preflight.py` | **21/21 全部通过** |
