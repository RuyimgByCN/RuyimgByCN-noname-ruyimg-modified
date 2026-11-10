# 裁决 34 第四轮及对抗复审加固证据：剩余 7 大体量来源全量逐项重做、死旗标/触发器/异步摸牌修复、Spine 4.0 标准二进制/凹凸耳切裁剪/贴图导入闭环与 37/37 来源仓内 4,435 项清零

- **核验日期**：2026-11-10
- **基线版本**：代码基线 `1f0d16d6c` / 初版第四轮 `3d6b4eb31`（`refs/heads/native-convergence`），`main` 基线 `834c660a9`
- **冻结协议与版本号（零升号）**：规则 `v333` ／ 存档 `NGSA v107` ／ content schema `v5` ／ `RULE_VM_ABI_VERSION 759` ／ WS `v8`；14 场 golden 哈希零变化
- **对应任务条款**：
  - 《原生架构收敛执行方案》第三节第 3 项（逐来源按上游 `file:line` 重做并接入 `packages/extensions/*` 正式包与 Web/WASM、Worker、Server、Desktop 四端产品路径；真实浏览器全页截图对照与帧率记录）
  - 第三节第 3a 项（[裁决 35](../2026-10-28-user-decisions/README.md)：净室自写 Spine 4.0 播放器标准二进制 `.skel` 流式编解码、凹凸多边形耳切+Sutherland-Hodgman 裁剪、浏览器 WebGL GLSL 着色器纹理批量管线、按 `dynamicSkin.js`／`saveSkinParams` 的玩家动皮与 `.png` 贴图导入组件及无资源/非 4.0 静态立绘回退，并经未参与编写的独立复审 Agent `340a14cf-873a-40d9-9c39-f7ab2ac11104` 净室复核）
  - 第三节第 6 项（37 来源整包八项签署逐来源复核；整包八项签署因第⑥维移动端真机与 7 个 Spine 来源本机 4.0 动皮截图对照未就绪如实维持 **`0/37`**）

---

## 一、裁决 35（Spine 4.0 净室播放器）第四轮与独立复审加固明细

严格遵守净室边界（全程未读取或复制 Esoteric 官方运行库源码、官方示例资产或上游打包的 `十周年UI/spine.js`、`皮肤切换/spine-lib/*`、`皮肤切换/v/spine-pixi-v8.js`，未引入 `pixi-spine` 等依赖包）：

1. **标准 Spine 4.0 二进制 `.skel` 顺序流编解码器**（[`apps/web-client/src/engine/spine40/binarySkelCodec.ts`](../../../../apps/web-client/src/engine/spine40/binarySkelCodec.ts#L1-L1360)）：
   - 按 Spine 4.0 Binary Export Format 公开规范实现 8 字节头部 `hash` 保真、LEB128 `varint`（含 `optimizePositive = false` zig-zag 有符号编码，修复 `drawOrder` 负偏移如 `-2` 损坏问题）、`nonessential = true/false` 双分支、UTF-8 字符串表引用、骨骼树、插槽、IK／变换／路径三类约束、默认与命名皮肤（含 `Region`/`BoundingBox`/`Mesh`/`LinkedMesh`/`Path`/`Point`/`Clipping` 7 类附件）、事件（含音频浮点字段）与动画时间轴（含 Bezier 曲线手柄流）的双向二进制编码（`encodeStandardSpine40Skel` / `encodeSpine40StandardBinary`）与解码（`parseStandardSpine40Skel` / `decodeSpine40StandardBinary`）；[`formatCodec.ts`](../../../../apps/web-client/src/engine/spine40/formatCodec.ts#L506-L516) 直接委托标准二进制编解码并移除旧 `"SP40"` + `JSON.stringify` 示意包装捷径。
2. **耳切法凹凸多边形裁剪与浏览器 WebGL2/WebGL1 GLSL 着色器管线**（[`apps/web-client/src/engine/spine40/webglRenderer.ts`](../../../../apps/web-client/src/engine/spine40/webglRenderer.ts#L1-L889)）：
   - 实现任意简单凹凸 `ClippingAttachmentData` 多边形耳切三角剖分（`triangulateSimplePolygon`）与逐三角形 Sutherland-Hodgman 裁剪（`clipBatchTrianglesToPolygon`），修复凹形裁剪框半平面交集过裁剪缺陷；
   - 实现完整浏览器 WebGL2/WebGL1 GLSL 顶点/片元着色器编译链接、`a_position`/`a_uv`/`a_color` 交错 VBO 上传、图集 RGBA `WebGLTexture` 上传绑定（`uploadRgbaTextureToWebGL`）与预乘透明 4 混合模式（`normal`/`additive`/`multiply`/`screen`）状态机（`renderSpineBatchesToWebGL`）。
3. **玩家本地 Spine 4.0 动皮与 `.png` 贴图导入面板及严格静态立绘回退**（[`apps/web-client/src/components/battle/surfaces/Spine40SkinImportBox.vue`](../../../../apps/web-client/src/components/battle/surfaces/Spine40SkinImportBox.vue#L1-L337)、[`webglRenderer.ts`](../../../../apps/web-client/src/engine/spine40/webglRenderer.ts#L829-L889)）：
   - 对齐上游 `dynamicSkin.js`／`saveSkinParams` 参数（`generalId, skinName, x, y, scale, angle, speed, action, background, clipSlots, hideSlots, flipX, version`），支持玩家从本地文件（`.json`/`.skel` + `.atlas` + `.png` `[data-testid="input-spine-png-file"]`）导入自有 Spine 4.0 动皮并实时驱动 `<canvas data-testid="spine-webgl-canvas">`；未导入资产或骨骼版本非 `4.0.x`（如 `3.8`、`4.1`）时显式回退静态立绘占位框（`status: "static-portrait-fallback"`，不再暗中绘制内置校核骨骼）。
   - 定向单测：`node --import ./apps/web-client/scripts/ts-register.mjs --test apps/web-client/scripts/spine40-cleanroom.test.mjs` -> **`11 passed; 0 failed`**（详见 [`2026-11-09-spine40-cleanroom/README.md`](../2026-11-09-spine40-cleanroom/README.md)）。

---

## 二、对抗复审检出并根因修复的 6 类缺陷

1. **死旗标与 43 项单旗标空桩修复**：`3d6b4eb31` 中 `u05_viewas:*`、`u05_mod:*`、`u05_buff:*`、`l11_txhj_zhuge_sha_plus3`、`l11_fangtian_extra_targets`、`l11_xingtian_*`、`l11_xuwang_hand_minus1` 等旗标无内核读取点，`decision34_l25_ssr.rs` 中 43 个 `ssr_*` 分支仅写 `l25_skill:ssr_*` 旗标。已新增 [`d34_flags.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_flags.rs#L1-L674) 接入 `rvm_bridge/modifiers/eval.rs` 与 `Catalog::find_player_vm_view_as`，并新增 [`decision34_l25_ssr_mods.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l25_ssr_mods.rs#L1-L341) 逐项替换为上游真实效果与修饰。
2. **误挂 `extra_events: vec![EventKind::PhaseBegin]` 与单回合 6 次自毁修复**：`decision34_pack9.rs` 此前把所有 `Death`/`Dying`/`Damage`/`CardUsed`/`PhaseEnd` 技能额外挂上 `PhaseBegin` 且每阶段重复触发。已移除该错误 `extra_events` 并在 [`commit_up37.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37.rs) 增加 `guard_phase_auto_trigger` 单阶段守卫。
3. **异步 `start_draw` 早于同步弃牌/夺牌时序修复**：`txhj_yiyi`、`yiyi`、`ycnxinfu_qiaosi`、`ssr_newsance_weicheng`、`txhj_yuanjiao` 此前调用异步 `ctx.start_draw` 后立即同步弃牌导致空手牌者弃 `0` 张白赚摸牌，`txhj_shuiyan` 误弃手牌，`txhj_wangmeizhike` 与 `U05` 判定/拼点未真实弹牌。已新增 `draw_cards_sync` 同步摸牌、真实牌堆弹牌落弃牌堆与仅弃装备区修正。
4. **跨包预置实体顺序敏感重名修复**：`attach_formal_package_upstream_rules`（[`decision34_pack9.rs`](../../../../crates/content-native-loader/src/decision34_pack9.rs#L420-L465)）补齐 `pack.generals`/`cards`/`host_skills`/`vm_skills` 对 `prior_packs` 去重，`[shahai-zhenfa, wuxing]` 正逆序装载均零冲突。
5. **前端残留虚构名、实时战局信封断连与截图截断修复**：清除 `extensionSurfaceRuntime.ts` 与 `ExtensionSurfacePanel.vue` 中残留的 `颍川前哨`/`玄木灵鹿`/`苍雷幼麟`/`黄天当立`/`破军神威`/`spshidi`/`zhefu`，修复 `ingestBattleEnvelopeIntoSurfaces` 实时派发五套 UI 钩子（含 `_pfqh_check_damage`/`_pfqh_check_jisha`），并改为 `captureBeyondViewport: true` 全页无遮挡截图。
6. **冻结 `ABI 759` 单窗口确定性结算边界透明登记**：在 [`native-rules-baseline.md`](../../../architecture/native-rules-baseline.md) 明确登记零升号约束下多步交互 `choose*` 子询问在 `decision34_*` 通道内按上游优先级确定性结算的协议边界，不冒充多步子询问。

---

## 三、剩余 7 个大体量来源按上游 `file:line` 逐项重做与正式包产品路径闭环

| 来源编号与名称 | 正式包目录（`packages/extensions/*`） | 上游 `file:line` 依据 | 原生内核／装载器／前端落点 | 条目统计（忠实/示意/缺失） | 定向验证测试 |
| --- | --- | --- | --- | --- | --- |
| **`U05 杀海拾遗`** | `shahai-{gujian,gwent,hearth,mtg,swd,yunchou,zhenfa}-upstream-noname`、`shaai-{ow,xianjian}-upstream-noname`（9 包） | `archive/upstream/noname/apps/core/extension/杀海拾遗/{main/config.js:1-38, character/{gujian,gwent,hearth,mtg,swd,ow,xianjian}.js, card/{gujian,gwent,hearth,mtg,swd,yunchou,zhenfa}.js}` | [`shahai_config.rs`](../../../../crates/content-native-loader/src/shahai_config.rs#L1-L236)、[`shahai_generals_data.rs`](../../../../crates/content-native-loader/src/shahai_generals_data.rs#L1-L374)、[`shahai_cards_skills_data.rs`](../../../../crates/content-native-loader/src/shahai_cards_skills_data.rs#L1-L535)、[`shahai_attach.rs`](../../../../crates/content-native-loader/src/shahai_attach.rs#L1-L461)；内核 [`d34_flags.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_flags.rs#L1-L674)、[`decision34_u05_ops.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_u05_ops.rs#L1-L372)、[`decision34_u05_gujian_gwent.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_u05_gujian_gwent.rs#L1-L499)、[`decision34_u05_hearth.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_u05_hearth.rs#L1-L535)、[`decision34_u05_swd_cards.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_u05_swd_cards.rs#L1-L617)、[`decision34_u05_swd_chars.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_u05_swd_chars.rs#L1-L538)、[`decision34_u05_ow_xianjian.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_u05_ow_xianjian.rs#L1-L566) | **1,621 / 0 / 0**（332 将、1,007 技、275 卡、7 配置） | [`decision34_round4_u05_shahai.rs`](../../../../crates/engine-native/tests/decision34_round4_u05_shahai.rs#L1-L540)（`g1_shahai` **181/181 PASS**） |
| **`L08 十周年UI`** | `decade-ui-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/十周年UI/{extension.js:1-12723, content.js:1-487, skill.js:1-3141, component.js:1-117, layout.css}` | [`decadeMarkSkillsData.ts`](../../../../apps/web-client/src/engine/surfaces/decadeMarkSkillsData.ts#L1-L302)、[`decadeUiSurface.ts`](../../../../apps/web-client/src/engine/surfaces/decadeUiSurface.ts#L1-L323)、[`UiSkinSuitePanel.vue`](../../../../apps/web-client/src/components/battle/surfaces/UiSkinSuitePanel.vue#L1-L411)（保持纯 UI 包 0 VM 技能/0 卡牌隔离） | **406 / 0 / 0**（44 配置、319 钩子、43 表面） | `pnpm --dir apps/web-client test`（**489/489 PASS**）+ Headless Chrome `60.0 FPS` |
| **`L09 千幻聆音`** | `qhly-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/千幻聆音/{extension.js:1-15969, skinShare.js:1-1726}` | [`qhlySurface.ts`](../../../../apps/web-client/src/engine/surfaces/qhlySurface.ts#L1-L391)、[`UiSkinSuitePanel.vue`](../../../../apps/web-client/src/components/battle/surfaces/UiSkinSuitePanel.vue#L1-L411) | **102 / 0 / 0**（82 配置、8 钩子、12 表面） | `pnpm --dir apps/web-client test`（**489/489 PASS**）+ Headless Chrome `60.0 FPS` |
| **`L11 太虚幻境`** | `taixu-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/太虚幻境/{extension.js:1-6460, extension_character.js:1-1493, extension_skill.js:1-4513, extension_card.js:1-2482}` | [`taixu_generals_data.rs`](../../../../crates/content-native-loader/src/taixu_generals_data.rs#L1-L402)、[`taixu_skills_data.rs`](../../../../crates/content-native-loader/src/taixu_skills_data.rs#L1-L195)、[`taixu_campaign_data.rs`](../../../../crates/content-native-loader/src/taixu_campaign_data.rs#L1-L378)；内核 [`d34_flags.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_flags.rs#L1-L674)、[`decision34_l11_buffs.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l11_buffs.rs#L1-L707)、[`decision34_l11_skills.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l11_skills.rs#L1-L708)；前端 [`taixuSurface.ts`](../../../../apps/web-client/src/engine/surfaces/taixuSurface.ts#L1-L214) | **942 / 0 / 0**（320 将、380 技、32 卡、206 赛季/事件/侍灵节点、4 表面） | [`decision34_round4_l11_taixu.rs`](../../../../crates/engine-native/tests/decision34_round4_l11_taixu.rs#L1-L413)（**2/2 PASS**）+ Headless Chrome 战役表面全页截图 |
| **`L14 手杀ui`** | `shousha-ui-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/手杀ui/{extension.js:1-7480, lbtn/main1.js:1-1119, skill/main1.js:1-1104, character/main1.js:1-444}` | [`shoushaUiSurface.ts`](../../../../apps/web-client/src/engine/surfaces/shoushaUiSurface.ts#L1-L337)、[`UiSkinSuitePanel.vue`](../../../../apps/web-client/src/components/battle/surfaces/UiSkinSuitePanel.vue#L1-L411)（保持纯 UI 包 0 VM 技能隔离） | **141 / 0 / 0**（48 配置、53 钩子、40 表面） | `pnpm --dir apps/web-client test`（**489/489 PASS**）+ Headless Chrome `60.0 FPS` |
| **`L24 皮肤切换`** | `skin-switch-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/皮肤切换/{extension.js:1-11955, skinParams.js}` | [`skinSwitchSurface.ts`](../../../../apps/web-client/src/engine/surfaces/skinSwitchSurface.ts#L1-L342)、[`Spine40SkinImportBox.vue`](../../../../apps/web-client/src/components/battle/surfaces/Spine40SkinImportBox.vue#L1-L337) | **46 / 0 / 0**（14 配置、22 钩子、10 表面） | `pnpm --dir apps/web-client test`（**489/489 PASS**）+ Headless Chrome `60.0 FPS` |
| **`L25 祖安武将`** | `zuan-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:1-51473` | [`zuan_data.rs`](../../../../crates/content-native-loader/src/zuan_data.rs#L1-L376)；内核 [`d34_flags.rs`](../../../../crates/engine-native-kernel/src/rvm_bridge/modifiers/d34_flags.rs#L1-L674)、[`decision34_l25_ssr.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l25_ssr.rs#L1-L692)、[`decision34_l25_ssr_mods.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l25_ssr_mods.rs#L1-L341)、[`decision34_l25_rules.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l25_rules.rs#L1-L547)；前端 [`zuanSurface.ts`](../../../../apps/web-client/src/engine/surfaces/zuanSurface.ts#L1-L179) | **262 / 0 / 0**（60 将、84 技、8 卡、70 配置、40 钩子） | [`decision34_round4_l25_zuan.rs`](../../../../crates/engine-native/tests/decision34_round4_l25_zuan.rs#L1-L405)（**2/2 PASS**） |

---

## 四、真实无头 Chrome WebGL / DOM 全页截图与 60 帧性能实测

执行命令：

```bash
node apps/web-client/scripts/decision34-browser-surfaces.mjs docs/operations/evidence/2026-11-10-decision34-round4-37-closure
```

实测结果（原始输出见 [`decision34-browser-surfaces-report.json`](decision34-browser-surfaces-report.json)，全页截图见 [`decision34-ui-skin-suites.png`](decision34-ui-skin-suites.png)、[`decision34-taixu-campaign-surface.png`](decision34-taixu-campaign-surface.png) 与 [`decision34-browser-surfaces.png`](decision34-browser-surfaces.png)）：

- **`passed`**: `true`（`consoleErrors: []`，`spinePngInputPresent: true`）
- **面板与子画布尺寸**：主面板 `1160×1306`、UI/动皮套装区 `1121×502`、Spine 4.0 导入 WebGL 画布 `122×54`、共享 Spine WebGL 画布 `162×54`
- **Spine 4.0 净室导入与静态回退实证**：
  - 未导入时：`static-portrait-fallback — NO_DYNAMIC_SKIN_ASSET_IMPORTED`（严格展示静态立绘占位框）
  - 导入 4.0 骨骼后：`ready-spine40 — SPINE_40_LOADED:4.0.64`，`l24Meta: "Spine 4.0 净室 · 动作:gongji · 偏移(24,-8) · 缩放 108% · 资产状态:ready-spine40 · WebGL批次:2/8△"`
- **5 大 UI/战役表面交互与存档恢复实证**：
  - `L08 十周年UI`：`decadeLayout.css · standard`，技能标记栏 `"成略[阴] 暴戾[2] 🍶酒态"`
  - `L09 千幻聆音`：`newui_lolbig.css · 共享映射：sb_caopi → re_caopi (台词 rexingshang) · BGM：qhly_bgm/re_caopi.mp3`
  - `L14 手杀ui`：`出牌阶段 (控件#2) 提示：出牌阶段，请选择一张牌 · 自动牌序：[102,103,101] · 露头(ZLLT)：关`；连破横幅 `"手杀播报:手杀播报·天下无双 · 倒计时 70%"`
  - `L24 皮肤切换`：`弹窗:cang-zhen-ge · 变身:jisha_form 出框：GongJi(flipX=true) · 语音队列：sha_fire`
  - `L11 太虚幻境`：`extension_MobileStyle.css (rem) 赛季：ChongYingChuLin · 侍灵：jiuwei (frame3) · 金币：150 (巨贾冻结:是)`；战役地图节点 `#2 (幻境守护童子（战斗）)`、赐福 2 项（`丰裕生财、焱火`）、灵宠 `九尾`
  - `L25 祖安武将`：`已应用离线 DIY: ssr_sunquan (5血)`，联机拦截 `ONLINE_MULTIPLAYER_DIY_FORBIDDEN`
  - 同锁存读档往返（`beforeSaveDaoxin: "50"` → 改写 `"80"` → 读档恢复 `"50"`）与重开局重置（`"20"`）全部一致
- **60 帧 `requestAnimationFrame` 帧率指标**：
  - `n = 60, p50_ms = 16.66, p95_ms = 16.67, p99_ms = 16.67, max_ms = 16.67, approx_fps = 60`

---

## 五、第五节全量门禁实跑记录

1. **源码行数、文档行号与分层棘轮**：
   - `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` -> `Ran 2 tests OK`
   - `python3 -B scripts/check-source-lines.py` -> 通过（`9,244` 个受维护文件全部 `<= 2,000` 行，Rust 非测试源码全部 `<= 1,000` 有效行）
   - `python3 -B scripts/check_doc_line_refs.py --strict` -> 通过（`0` 越界）
   - `python3 -B scripts/check-native-layering.py` -> `engine-native layering passed: 0 upward references <= 0`
2. **Rust 定向、原生三 crate、独立 `-D warnings`、全工作区与跨目标一致性**：
   - `cargo test --offline -p engine-native --test g1_shahai --test l_rounds --test misc -- decision34_ u05_` -> **`22 passed; 0 failed; 0 ignored`**
   - `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native` -> **`2,228 passed; 0 failed; 6 ignored`**（剩余 6 个 `#[ignore]` 均为手工诊断探针）
   - 独立 `CARGO_TARGET_DIR` 下 `CARGO_INCREMENTAL=0 RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader -p desktop-client --all-targets` 与 `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm` -> **exit 0（0 warnings）**，临时目录已即时清理（裁决 23，磁盘余量 `92 GiB`）
   - `cargo test --offline --workspace --no-fail-fast` -> **`7,621 passed; 0 failed; 8 ignored`**（145 非空套件全绿）
   - `cargo run --offline -p conformance` -> `19/19 ALL PASS`
   - `cargo test --offline -p desktop-client` -> `20 passed; 0 failed; 1 ignored`（保持 WASM 默认与 `?desktopEngine=native` 双路径测试通过，裁决 36）
   - `python3 -B scripts/check-native-wasm-cross-target.py` & `bash scripts/cross-target-hash.sh` -> `ALL CROSS-TARGET CHECKS PASSED`
3. **前端单测、Typecheck、Native WASM 构建、Worker Smoke 与 `preflight.py`**：
   - `node --import ./apps/web-client/scripts/ts-register.mjs --test apps/web-client/scripts/spine40-cleanroom.test.mjs` -> `11 passed; 0 failed`
   - `pnpm --dir apps/web-client test` -> `489 passed; 0 failed`
   - `pnpm --dir apps/web-client typecheck` & `pnpm --dir apps/web-client build` -> `exit 0`
   - `pnpm --dir apps/web-client run build:native` & `pnpm --dir apps/web-client run smoke:native` -> `83/83 PASS (failed: 0)`
   - `python3 -B scripts/preflight.py` -> **`21/21 全部通过 ✅`**
4. **按裁决 25 发布与 exact-head CI 实跑登记**：
   - 初版第四轮代码提交 `3d6b4eb31` 上 exact-head CI Run [`42718816733`](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/42718816733)（[`ci-run-42718816733-3d6b4eb31.json`](ci-run-42718816733-3d6b4eb31.json)）首跑 `10/10 PASS`；
   - 本轮对抗复审修复代码提交推送 `refs/heads/native-convergence` 后的最新 exact-head CI Run ID 见 [`native-progress.md`](../../../architecture/native-progress.md) 顶部最新登记。

---

## 六、剩余外部条件与用户本机核对步骤

1. **裁决 35 用户本机 Spine 4.0 动皮截图对照步骤（产品不附带外部版权动皮/配音资产）**：
   - 在浏览器打开 `apps/web-client` 战局扩展表面面板，展开「玩家动皮导入 (`saveSkinParams`)」；
   - 选择本机自有的 Spine `4.0.x` 武将动皮资产（`.json` 或标准 `.skel` 二进制骨骼 + `.atlas` 图集 + `.png` 贴图文件），填入对应 `action`（如 `DaiJi` / `GongJi`）与 `x, y, scale` 参数；
   - 与旧版无名杀同武将动皮同动作帧进行同场景视觉对照；若导入 `3.5`/`3.7`/`3.8`/`4.1`/`4.2` 骨骼，确认界面按规则基准显示静态立绘回退占位框。
2. **PA08 / PA11 / NC12 移动端真机与三平台签名公证凭据**：
   - 待接入开启 USB 调试的中低端 Android 真机与解锁信任的物理 iPhone（裁决 19），执行整局与 `>= 30 min` 功耗／温度／降频实测，据此判定 PA11 并完成 37 来源第⑥维移动端目标平台签署；
   - 待配置 Apple Developer ID 签名证书 + 公证凭据与 Windows Authenticode 证书，执行三平台安装包真实签名安装验证；第六阶段（旧线安全退出）在上述门槛齐备并获用户确认前保持未开始。
