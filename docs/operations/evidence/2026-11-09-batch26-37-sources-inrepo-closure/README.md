# Batch 26：裁决 34 最后 9 个来源（`U01, U05, L08, L14, L19, L20, L24, L25, L27`）及 `U07` 余技全量原生闭环与 `37/37` 来源仓内 7 维 100% 闭合证据（2026-11-09）

- **核验日期**：2026-11-09
- **基线 HEAD**：`64c8f53311b291b7f6a25c299550789128253b39`（Batch 25）
- **版本门禁（零升号，14 场 golden 哈希零变化）**：
  - `STANDARD_RULESET_VERSION = 333`
  - `ARCHIVE_FORMAT_VERSION = 107`
  - `CONTENT_PACKAGE_SCHEMA_VERSION = 5`
  - `RULE_VM_ABI_VERSION = 759`
  - `NETWORK_PROTOCOL_VERSION = 8`

## 1. 本批目标与完成摘要

针对 Batch 25 后[两仓扩展台账](../../../migration/upstream-extension-reconstruction.md)与[37 来源八项签署核对表](../2026-10-19-source-signoff-32/README.md)中仍处于「部分」的最后 **9 个来源**（`U01 3D精选, U05 杀海拾遗, L08 十周年UI, L14 手杀ui, L19 末日乱斗, L20 标记补充, L24 皮肤切换, L25 祖安武将, L27 选将美化`）及 `U07 玩点论杀` 的 `jianbi`/`choudu` 余技，按用户[裁决 34](../2026-10-28-user-decisions/README.md)完成内核权威结算器、装载器扩展包构造（100% 源自上游真实 `lib.skill` / `character` / `card` 标识符并按事件类型挂载 `EventKind::Damage`/`CardUsed`/`PhaseEnd`/`Death`/`Dying`/`TurnBegin`/`PhaseBegin`）、前端原生渲染组件（含 `<canvas data-testid="spine-skeletal-canvas">` 骨骼/粒子帧绘制、3D 精选 31 将镜头立绘展台、手杀 92 项钩子 HUD、皮肤切换 520KB 参数变身面板、祖安 122 技单机 DIY 编辑器、选将美化 37 技 `_chooseCharacterTwo` 双将选将台）、存读档恢复与真实无头 Chrome 60.0 FPS 实测，使**全部 `37/37` 来源在仓内 7 个维度（①功能、②配置、③依赖、④组合、⑤保存恢复、⑦性能、⑧规则差异）达到 100% `✅` 闭环**（第⑥维目标平台为 `四端 ✅ / 待移动真机`，整包八项全满签署因物理 Android/iOS 真机未接入如实保持 `0/37`）。

## 2. 逐来源实现落点与规则对照

| 来源 | 新增/更新落点 | 核心规则、负例守卫与保存恢复 |
| --- | --- | --- |
| **U01 3D精选** | [`decision34_pack9.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs) (`DDD_SKILL_DISPATCH_ID`)、[`decision34_pack9_data.rs`](../../../../crates/content-native-loader/src/decision34_pack9_data.rs) (`U01_DDD_GENERALS` 31 将、`U01_DDD_SKILLS` 64 技)、[`ExtensionSurfacePanel.vue`](../../../../apps/web-client/src/components/battle/ExtensionSurfacePanel.vue) (`[data-testid="u01-3d-select-gallery"]`) | 全部 31 名 `ddd_*` 武将与 64 项顶层技能注册入 `Catalog`；`dddxiaxing`（侠行距离基数与横置联动）、`dddyeshen`（夜深判定与暗置）、`dddqiaoduan`（巧断用牌回复/摸牌双分支）、`dddqianlong`（潜龙受伤蓄势与解锁）、`dddqice`（奇策手牌转化虚拟锦囊及空手牌 `ddd_qice_blocked_empty = 1` 早退守卫）；前端 3D 镜头角度（`front`/`side`/`closeup`）与深度视差切换（`resolve_u01_3d_gallery`）；`encode_archive`/`decode_archive` 往返一致 |
| **U05 杀海拾遗** | [`decision34_pack9.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs) (`SWD_DUIJUE_ID`、`SWD_TANLIN_ID`、`YUNCHOU_SKILL_ID`、`MTG_LAND_CARD_ID`、`ZHENFA_POZHENJUE_ID`、`OW_SKILL_DISPATCH_ID`、`XIANJIAN_SKILL_DISPATCH_ID`)、[`decision34_pack9_data.rs`](../../../../crates/content-native-loader/src/decision34_pack9_data.rs) (`U05_SWD_REMAINING_SKILLS` 49 项、`U05_YUNCHOU_REMAINING_SKILLS` 3 项、`U05_MTG_LAND_CARDS` 2 张、`U05_ZHENFA_CARDS` 12 项、`U05_OW_SKILLS` 40 项、`U05_XIANJIAN_GENERALS` 36 将、`U05_XIANJIAN_SKILLS` 50 项) | ① `shahai-swd`：49 项余技（`duijue` 真实检查 `target.hp <= 1` 守卫 `duijue_blocked_hp = 1` + 限定技 `duijue_blocked_limited = 1` 守卫 + 2 点直伤、`tanlin` 双方非空手牌守卫 `tanlin_blocked_empty = 1` + 拼点输受伤 `tanlin_lost_damage = 1` / 赢发现回血、`funiao`/`ningxian`/`aojian`/`mohua2`/`busi`/`yinguo`/`pozhen` 等）；② `shahai-yunchou`：`ycndiyjianxiong`（`EventKind::Damage` 获牌+摸 1）、`ycnxinfu_qiaosi`（巧思六面骰确定性结算）、`xinfu_jingxie`（精械重铸防具/宝物回血摸牌）；③ `shahai-mtg`：`mtg_cangbaohaiwan`（藏宝海湾）与 `mtg_duzhao`（毒沼）及每回合限一地牌/横置守卫 `mtg_land_blocked = 1`；④ `shahai-zhenfa`：`changshezhen`/`tianfuzhen`/`dizaizhen`/`fengyangzhen`/`yunhaizhen`/`qixingzhen`/`shepanzhen`/`longfeizhen`/`huyizhen`/`niaoxiangzhen`/`yulinzhen`/`pozhenjue`（含 `pozhenjue_recast == 1` 重铸分支与破阵换位 `pozhenjue_seats_swapped = 2`）；⑤ `shaai-ow`：40 项技能（`lichang`/`zhanlong`/`huoyu`/`zihui`/`shanguang`/`qinru`/`shanhui`/`zhongdun` 及终极充能 `<100` `ow_ult_blocked = 1` 早退守卫）；⑥ `shaai-xianjian`：全部 36 名 `pal_*` 武将（`pal_lixiaoyao`..`pal_jushifang`）与 50 项技能（`leiyu`/`qixia`/`tianjian`/`xshuangren`/`tianxian`/`longxi`/`xuanning` 等） |
| **U07 玩点论杀** | [`decision34_pack9.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs) (`WUDIAN_JIANBI_ID`、`WUDIAN_CHOUDU_ID`)、[`decision34_pack9_data.rs`](../../../../crates/content-native-loader/src/decision34_pack9_data.rs) (`U07_WUDIAN_REMAINING_SKILLS`) | 补齐 `jianbi`（`EventKind::CardUsed` 坚壁清野多目标减指定与免弃，单目标 `jianbi_target_count < 2` 触发 `jianbi_blocked_single_target = 1` 早退守卫）、`choudu`（筹度强制要求 `he` 区有牌否则 `choudu_blocked_no_card = 1` 早退、每回合限一次 `choudu_blocked_usable = 1` 守卫、弃 1 牌置 `diaobingqianjiang_resolved = 1` 视为调兵遣将结算）及 `fuchou`/`fuchou2`/`liangce`/`liduan` 闭环 |
| **L08 十周年UI** | [`decision34_pack9.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs) (`L08_DECADE_SKILL_ID`)、[`decision34_pack9_data.rs`](../../../../crates/content-native-loader/src/decision34_pack9_data.rs) (`L08_DECADE_SKILLS` 9 项)、[`ExtensionSurfacePanel.vue`](../../../../apps/web-client/src/components/battle/ExtensionSurfacePanel.vue) (`<canvas data-testid="spine-skeletal-canvas">`) | 上游 `extension/十周年UI/extension.js` 全部 9 项真实 `lib.skill` 钩子（`_usecard`、`_changeEquipExt`、`_ui_auto_cheat`、`_kill_framebg`、`_rarity`、`_tiesuo_ying`、`_decadeUI_usecardBegin`、`_discard`、`_decadeUI_dieKillEffect`）完整剥离入原生结算；前端 `<canvas>` 骨骼/粒子渲染器支持 `slash_burst`/`damage_shake`/`equip_flash`/`heal_bloom`/`victory_banner` 5 种动作重绘与帧计数 |
| **L14 手杀ui** | [`decision34_pack9.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs) (`L14_SHOUSHA_SKILL_ID`)、[`decision34_pack9_data.rs`](../../../../crates/content-native-loader/src/decision34_pack9_data.rs) (`L14_SHOUSHA_SKILLS` 48 项)、[`ExtensionSurfacePanel.vue`](../../../../apps/web-client/src/components/battle/ExtensionSurfacePanel.vue) (`[data-testid="l14-shousha-hud"]`) | 上游 `extension/手杀ui/extension.js` 全部 48 项真实 `lib.skill` 钩子（`_dieNoTuoguan`、`_menuCheat`、`_jindutiaoO`..`_jindutiao_close`、`_chupaiA`..`_chupaiNX`、`_skilltip_closeB`、`_jd_ddz_tgr`..`_jd_hhjs`、`_jindutiao`、`_jindutiaopl`、`_wmkzSayChange`、`_shuohuaSayWordphase`）注册入 `Catalog` 并与前端手杀 HUD（阶段横幅、连击播报、整理手牌、记牌器）双向同步 |
| **L19 末日乱斗** | [`decision34_pack9.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs) (`L19_DOOMSDAY_SKILL_ID`)、[`decision34_pack9_data.rs`](../../../../crates/content-native-loader/src/decision34_pack9_data.rs) (`L19_DOOMSDAY_CHARACTERS` 24 将、`L19_DOOMSDAY_SKILLS` 66 项) | 上游 `extension/末日乱斗/extension.js` 全部 66 项真实 `lib.skill` 钩子（`_jisuangailv`、`_tuizhi`/`_tuizhi2`、`_jiangshi`..`_jiangshiTx3`、`shishi`、`baozou`、`ganran`/`ganran2`、`xunmeng`、`zaibian`、`_huzhu`、`chanceIdentity`..`chanceIdentity0`、`_gainSkill`、`_wujinmoshi*`）与 24 名乱斗专将（含 `jiangshifz`、`jiangshinj`）原生结算 |
| **L20 标记补充** | [`decision34_pack9.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs) (`L20_SKILL_DISPATCH_ID`)、[`decision34_pack9_data.rs`](../../../../crates/content-native-loader/src/decision34_pack9_data.rs) (`L20_MARK_SUPPLEMENT_CARDS` 7 项、`L20_MARK_SUPPLEMENT_SKILLS` 75 项) | 上游 `extension/标记补充/extension.js` 全部 75 项真实 `lib.skill` 钩子（含 `_xigua_feiyanbahu_`..`_guozhan_diebiaoji_`、`zhengsu` 擂进/变阵/鸣止三分支奖励与违例 `l20_zhengsu_reward_blocked = 1` 守卫、`zhenyi_spade` 真仪黑桃判伤、`xinhongyan` 新红颜花色锁定、`diaohulishan` 调虎离山离场、`xianqu_mark`/`zhulianbihe_mark`/`yinyang_mark`/`yexinjia_mark` 等）及 7 项扩展卡牌注册与结算 |
| **L24 皮肤切换** | [`decision34_pack9.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs) (`L24_SKIN_SKILL_ID`)、[`decision34_pack9_data.rs`](../../../../crates/content-native-loader/src/decision34_pack9_data.rs) (`L24_SKIN_SWITCH_SKILLS` 23 项)、[`ExtensionSurfacePanel.vue`](../../../../apps/web-client/src/components/battle/ExtensionSurfacePanel.vue) (`[data-testid="l24-skin-transform-panel"]`) | 上游 `extension/皮肤切换/` 全部 23 项真实 `lib.skill` 钩子（`_tsx`、`_gj`、`_ts`、`_playAudioToQueue`、`_checkDcdChuChang`、`_checkDcdShan`、`_fix_yh`、`_fix_phase_yh`、`_pfqh_check_roundCount`、`_check_die_yh`、`_pfqh_reapply_saved_params`、`_pfqh_reapply_on_init`、`_pfqh_check_hp_change`、`_pfqh_check_damage_times`、`_pfqh_check_damage`、`_pfqh_check_jisha`、`_pfqh_check_changeGroup`、`_qhlyChangeGroupSkin`、`_pfqh_check_phaseBegin`、`_pfqh_check_phaseEnd`、`_qhly_autoc`、`_qhly_addButton`、`_skin_switch_hook`）与 520KB 皮肤切换动效参数（`scale`/`offset_x`/`offset_y`/`action`）变身结算，与共享 `<canvas data-testid="spine-skeletal-canvas">` 联动重绘 |
| **L25 祖安武将** | [`decision34_pack9.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs) (`L25_ZUAN_SKILL_ID`)、[`decision34_pack9_data.rs`](../../../../crates/content-native-loader/src/decision34_pack9_data.rs) (`L25_ZUAN_CHARACTERS` 65 将、`L25_ZUAN_SKILLS` 122 项)、[`ExtensionSurfacePanel.vue`](../../../../apps/web-client/src/components/battle/ExtensionSurfacePanel.vue) (`[data-testid="l25-zuan-diy-editor"]`) | 上游 `extension/祖安武将/extension.js` 全部 65 名武将（含 60 名 `ssr_*` 武将）与 122 项真实 `lib.skill` 钩子（`_player_xxgg_zhishixian`..`_zuan_damageSendEgg`，含 `g_du` 毒牌触发、`_diycharacterskill` 单机 DIY 授技、`_emojiRebate1/2` 等）注册入 `Catalog`；联机/录像模式严格置 `l25_zuan_online_blocked = 1` 拒绝 DIY 篡改；前端实装结构化单机 DIY 武将编辑器（`validate_and_apply_l25_diy` 拒绝 `eval`/`Function`/`import()`） |
| **L27 选将美化** | [`decision34_pack9.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_pack9.rs) (`L27_SELECT_SKILL_ID`)、[`decision34_pack9_data.rs`](../../../../crates/content-native-loader/src/decision34_pack9_data.rs) (`L27_SELECT_BEAUTIFY_SKILLS` 37 项)、[`ExtensionSurfacePanel.vue`](../../../../apps/web-client/src/components/battle/ExtensionSurfacePanel.vue) (`[data-testid="l27-dual-general-picker"]`) | 上游 `extension/选将美化/` 全部 37 项活跃可读技能（`baonvezhi`..`xingchong`，含 `qimou` 奇谋限定技失 X 体力摸 X 牌且二次发动 `l27_qimou_blocked_limited = 1` 拒绝、`baonvezhi` `EventKind::Damage` 暴虐值、`rehuashen`、`taoluan`、`twfengji` 等）及 `_chooseCharacterTwo` 双将选将流程（`resolve_l27_choose_character_two` 同名/越界守卫 `l27_choose_two_invalid = 1`）与前端双将选将台 |

## 3. 验证命令与实跑结果

### 3.1 定向与三 crate 原生测试
```bash
cargo test --offline -p engine-native --test misc decision34_pack9_closure::
cargo test --offline -p engine-native --test misc decision34_rules_and_campaigns::
cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native
```
- `decision34_pack9_closure`：**5 passed; 0 failed; 0 ignored**
- `decision34_rules_and_campaigns`：**7 passed; 0 failed; 0 ignored**
- 三 crate 全量：**2,219 passed; 0 failed; 6 ignored**（规则探针 `0 ignored`）

### 3.2 行数门禁、分层棘轮、独立 `CARGO_TARGET_DIR` `-D warnings` 与 `wasm32` 检查
```bash
python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'
python3 -B scripts/check-source-lines.py
CARGO_TARGET_DIR=/tmp/noname-b26-strict-check RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets && rm -rf /tmp/noname-b26-strict-check
cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm
```
- `test_check_source_lines.py`：**Ran 2 tests in 0.011s — OK**
- `check-source-lines.py`：**Source line limit passed: 9199 files <= 2000 lines; Rust non-test files <= 1000 effective lines**
- 独立 `CARGO_TARGET_DIR` `-D warnings` 与 `wasm32-unknown-unknown`：**0 warnings, exit 0**（临时目录 `/tmp/noname-b26-strict-check` 已按裁决 23 清理）

### 3.3 前端单测、TypeScript 类型检查、WASM Worker Smoke 与真实无头 Chrome 渲染及帧率实测
```bash
pnpm --dir apps/web-client test
pnpm --dir apps/web-client typecheck
pnpm --dir apps/web-client run build:native
pnpm --dir apps/web-client run smoke:native
node apps/web-client/scripts/decision34-browser-surfaces.mjs docs/operations/evidence/2026-11-09-batch26-37-sources-inrepo-closure
```
- `pnpm test`：**469 passed; 0 failed**
- `pnpm typecheck`：**0 errors**
- `pnpm smoke:native`：**83/83 PASS (`{"total":83,"failed":0}`)**
- 真实无头 Chrome (`decision34-browser-surfaces.mjs`)：
  - 报告文件：[`decision34-browser-surfaces-report.json`](decision34-browser-surfaces-report.json)
  - 截图文件：[`decision34-browser-surfaces.png`](decision34-browser-surfaces.png)
  - 面板尺寸：`1160 × 729`（`u01Rect: 343 × 82`、`spineRect: 222 × 54`、`l14Rect: 343 × 80`）
  - 交互与恢复验证：`u01Camera: "ddd_liuchong:closeup"`、`spineAction: "action:damage_shake"`、`l14Streak: "x2"`、`zuanSaved: "saved:zuan_hero_1"`、`zuanRefused: "error:eval_refused"`、`l27Confirmed: "confirmed:2"`、`restoredCamera: "ddd_liuchong:closeup"`、`restoredStreak: "x2"`、`afterRestartCamera: "ddd_liuchong:front"`、`afterReloadCamera: "ddd_liuchong:closeup"`、`consoleErrors: []`
  - 60 帧 `requestAnimationFrame` 帧率：**`p50 = 16.66 ms, p95 = 16.67 ms, p99 = 16.67 ms, max = 16.67 ms, approx_fps = 60.0 FPS`**

### 3.4 全工作区测试、跨目标确定性、`preflight.py` 与裁决 25 发布 / exact-head CI 实跑

| 门禁项 | 命令 / 证据 | 结果 |
| --- | --- | --- |
| 全工作区 Rust 测试 | `cargo test --offline --workspace --no-fail-fast` | **7,611 passed; 0 failed; 8 ignored** |
| 原生↔旧线 Conformance | `cargo run --offline -p conformance` | **19/19 ALL PASS** |
| 桌面客户端单测 | `cargo test --offline -p desktop-client` | **20 passed; 0 failed; 1 ignored** |
| 跨目标确定性（Native vs WASM） | `python3 -B scripts/check-native-wasm-cross-target.py` & `bash scripts/cross-target-hash.sh` | **ALL CROSS-TARGET CHECKS PASSED** (`native_wasm_match: true`, `6a59d996...` / `5e41ae8e...`) |
| 收口 21 项门禁 | `python3 -B scripts/preflight.py` | **21/21 全部通过 ✅** |
| 裁决 25 发布与 exact-head CI | 单命令作用域 `pre-push` 守卫（原钩子 `sha256: e349af1ec5ccc1742df7e7d91d387ed2ef086f7c864d8cf7372d0ef66dd49ca7` 零改动）推送代码提交 `5a866c19f4682d0af6aee5e55cde4a6dede8bb52 -> refs/heads/native-convergence`（CI Run [`42536292387`](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/42536292387)，[`ci-run-42536292387-5a866c19f.json`](ci-run-42536292387-5a866c19f.json)） | **首次运行（Attempt 1）10/10 全部通过 `conclusion: success`**（`build-ubuntu-latest` `128901219497`、`perf-macos-latest` `128901219556`、`desktop-windows-latest` `128901219558`、`native-web` `128901219562`、`desktop-macos-latest` `128901219570`、`desktop-ubuntu-latest` `128901219608`、`build-macos-latest` `128901219639`、`perf-ubuntu-latest` `128901219666`、`gates-ubuntu` `128901219669`、`build-windows-latest` `128901219676` 全绿） |
