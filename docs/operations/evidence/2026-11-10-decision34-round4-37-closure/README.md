# 裁决 34 第四轮与裁决 35 加固证据：剩余 7 大体量来源全量逐项重做、Spine 4.0 标准二进制/裁剪/导入闭环与 37/37 来源仓内 4,435 项清零

- **核验日期**：2026-11-10
- **基线版本**：代码基线 `1f0d16d6c`（`refs/heads/native-convergence`），`main` 文档基线 `ccfe70598`
- **冻结协议与版本号（零升号）**：规则 `v333` ／ 存档 `NGSA v107` ／ content schema `v5` ／ `RULE_VM_ABI_VERSION 759` ／ WS `v8`；14 场 golden 哈希零变化
- **对应任务条款**：
  - 《原生架构收敛执行方案》第三节第 3 项（逐来源按上游 `file:line` 重做并接入 `packages/extensions/*` 正式包与 Web/WASM、Worker、Server、Desktop 四端产品路径；真实浏览器截图对照与帧率记录）
  - 第三节第 3a 项（[裁决 35](../2026-10-28-user-decisions/README.md)：净室自写 Spine 4.0 播放器标准二进制 `.skel` 流式编解码、Sutherland-Hodgman 裁剪多边形、浏览器 WebGL GLSL 着色器纹理批量管线、按 `dynamicSkin.js`／`saveSkinParams` 的玩家动皮导入组件与无资源/非 4.0 静态立绘回退）
  - 第三节第 6 项（37 来源整包八项签署逐来源复核；整包八项签署因第⑥维移动端真机与 7 个 Spine 来源本机 4.0 动皮截图对照未就绪如实维持 **`0/37`**）

---

## 一、裁决 35（Spine 4.0 净室播放器）第四轮加固明细

严格遵守净室边界（全程未读取或复制 Esoteric 官方运行库源码、官方示例资产或上游打包的 `十周年UI/spine.js`、`皮肤切换/spine-lib/*`、`皮肤切换/v/spine-pixi-v8.js`，未引入 `pixi-spine` 等依赖包）：

1. **标准 Spine 4.0 二进制 `.skel` 顺序流编解码器**（[`apps/web-client/src/engine/spine40/binarySkelCodec.ts`](../../../../apps/web-client/src/engine/spine40/binarySkelCodec.ts#L1-L1213)）：
   - 按 Spine 4.0 Binary Export Format 公开规范实现 LEB128 `varint`（含 `optimizePositive`）、UTF-8 字符串表引用、骨骼树、插槽、IK／变换／路径三类约束、默认与命名皮肤（含 `Region`/`BoundingBox`/`Mesh`/`LinkedMesh`/`Path`/`Point`/`Clipping` 7 类附件）、事件与动画时间轴（含 Bezier 曲线手柄流）的双向二进制编码（`encodeSpine40StandardBinary`）与解码（`decodeSpine40StandardBinary`），保留 `SP40` 测试容器兼容分支（[`formatCodec.ts`](../../../../apps/web-client/src/engine/spine40/formatCodec.ts#L698-L739)）。
2. **Sutherland-Hodgman 凸包裁剪与浏览器 WebGL2/WebGL1 GLSL 着色器管线**（[`apps/web-client/src/engine/spine40/webglRenderer.ts`](../../../../apps/web-client/src/engine/spine40/webglRenderer.ts#L1-L736)）：
   - 实现 `ClippingAttachmentData` 激活至 `endSlot` 期间对后续 `Region`/`Mesh` 三角形批次的精确半平面多边形裁剪与扇形重三角化（`clipBatchTrianglesToPolygon`）；
   - 实现完整浏览器 WebGL2/WebGL1 GLSL 顶点/片元着色器编译链接、`a_position`/`a_uv`/`a_color` 交错 VBO 上传、图集 `WebGLTexture` 绑定与预乘透明 4 混合模式（`normal`/`additive`/`multiply`/`screen`）状态机（`renderSpineBatchesToWebGL`）。
3. **玩家本地 Spine 4.0 动皮导入面板与静态立绘回退**（[`apps/web-client/src/components/battle/surfaces/Spine40SkinImportBox.vue`](../../../../apps/web-client/src/components/battle/surfaces/Spine40SkinImportBox.vue#L1-L260)、[`webglRenderer.ts`](../../../../apps/web-client/src/engine/spine40/webglRenderer.ts#L676-L736)）：
   - 对齐上游 `dynamicSkin.js`／`saveSkinParams` 参数（`generalId, skinName, x, y, scale, angle, speed, action, background, clipSlots, hideSlots, flipX, version`），支持玩家从本地文件（`.json`/`.skel`/`.atlas`/`.png`）导入自有 Spine 4.0 动皮并实时驱动 `<canvas data-testid="spine40-import-webgl-canvas">`；未导入资产或骨骼版本非 `4.0.x`（如 `3.8`、`4.1`）时显式回退静态立绘（`status: "static-portrait-fallback"`）。
   - 定向单测：`node --test apps/web-client/scripts/spine40-cleanroom.test.mjs` -> **`9 passed; 0 failed`**（详见 [`2026-11-09-spine40-cleanroom/README.md`](../2026-11-09-spine40-cleanroom/README.md)）。

---

## 二、剩余 7 个大体量来源按上游 `file:line` 逐项重做与正式包产品路径闭环

| 来源编号与名称 | 正式包目录（`packages/extensions/*`） | 上游 `file:line` 依据 | 原生内核／装载器／前端落点 | 条目统计（忠实/示意/缺失） | 定向验证测试 |
| --- | --- | --- | --- | --- | --- |
| **`U05 杀海拾遗`** | `shahai-{gujian,gwent,hearth,mtg,swd,yunchou,zhenfa}-upstream-noname`、`shaai-{ow,xianjian}-upstream-noname`（9 包） | `archive/upstream/noname/apps/core/extension/杀海拾遗/{main/config.js:1-38, character/{gujian,gwent,hearth,mtg,swd,ow,xianjian}.js, card/{gujian,gwent,hearth,mtg,swd,yunchou,zhenfa}.js}` | [`shahai_config.rs`](../../../../crates/content-native-loader/src/shahai_config.rs#L1-L236)、[`shahai_generals_data.rs`](../../../../crates/content-native-loader/src/shahai_generals_data.rs#L1-L374)、[`shahai_cards_skills_data.rs`](../../../../crates/content-native-loader/src/shahai_cards_skills_data.rs#L1-L535)、[`shahai_attach.rs`](../../../../crates/content-native-loader/src/shahai_attach.rs#L1-L461)；内核结算器 [`decision34_u05_ops.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_u05_ops.rs#L1-L367)、[`decision34_u05_gujian_gwent.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_u05_gujian_gwent.rs#L1-L499)、[`decision34_u05_hearth.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_u05_hearth.rs#L1-L535)、[`decision34_u05_swd_cards.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_u05_swd_cards.rs#L1-L617)、[`decision34_u05_swd_chars.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_u05_swd_chars.rs#L1-L509)、[`decision34_u05_ow_xianjian.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_u05_ow_xianjian.rs#L1-L566) | **1,621 / 0 / 0**（332 将、1,007 技、275 卡、7 配置） | [`decision34_round4_u05_shahai.rs`](../../../../crates/engine-native/tests/decision34_round4_u05_shahai.rs#L1-L437)（`g1_shahai` **180/180 PASS**） |
| **`L08 十周年UI`** | `decade-ui-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/十周年UI/{extension.js:1-12723, content.js:1-487, skill.js:1-3141, component.js:1-117, layout.css}` | [`decadeMarkSkillsData.ts`](../../../../apps/web-client/src/engine/surfaces/decadeMarkSkillsData.ts#L1-L302)（310 项技能角标钩子）、[`decadeUiSurface.ts`](../../../../apps/web-client/src/engine/surfaces/decadeUiSurface.ts#L1-L323)（9 项交互钩子与 43 项十周年布局表面）、[`UiSkinSuitePanel.vue`](../../../../apps/web-client/src/components/battle/surfaces/UiSkinSuitePanel.vue#L1-L411)（保持纯 UI 包 0 VM 技能/0 卡牌隔离） | **406 / 0 / 0**（44 配置、319 钩子、43 表面） | `pnpm --dir apps/web-client test`（**486/486 PASS**）+ Headless Chrome `60.0 FPS` |
| **`L09 千幻聆音`** | `qhly-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/千幻聆音/{extension.js:1-15969, skinShare.js:1-1726}` | [`qhlySurface.ts`](../../../../apps/web-client/src/engine/surfaces/qhlySurface.ts#L1-L391)（8 项换肤/角色 BGM/击杀特效/胜利台词钩子、`skinShare.js` 武将立绘台词映射与 12 套千幻主题窗口）、[`UiSkinSuitePanel.vue`](../../../../apps/web-client/src/components/battle/surfaces/UiSkinSuitePanel.vue#L1-L411) | **102 / 0 / 0**（82 配置、8 钩子、12 表面） | `pnpm --dir apps/web-client test`（**486/486 PASS**）+ Headless Chrome `60.0 FPS` |
| **`L11 太虚幻境`** | `taixu-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/太虚幻境/{extension.js:1-6460, extension_character.js:1-1493, extension_skill.js:1-4513, extension_card.js:1-2482}` | [`taixu_generals_data.rs`](../../../../crates/content-native-loader/src/taixu_generals_data.rs#L1-L402)、[`taixu_skills_data.rs`](../../../../crates/content-native-loader/src/taixu_skills_data.rs#L1-L195)、[`taixu_campaign_data.rs`](../../../../crates/content-native-loader/src/taixu_campaign_data.rs#L1-L378)；内核 [`decision34_l11_buffs.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l11_buffs.rs#L1-L707)、[`decision34_l11_skills.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l11_skills.rs#L1-L702)；前端 [`taixuSurface.ts`](../../../../apps/web-client/src/engine/surfaces/taixuSurface.ts#L1-L214) | **942 / 0 / 0**（320 将、380 技、32 卡、206 赛季/事件/侍灵节点、4 表面） | [`decision34_round4_l11_taixu.rs`](../../../../crates/engine-native/tests/decision34_round4_l11_taixu.rs#L1-L298)（**1/1 PASS**）+ Headless Chrome 战役表面截图 |
| **`L14 手杀ui`** | `shousha-ui-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/手杀ui/{extension.js:1-7480, lbtn/main1.js:1-1119, skill/main1.js:1-1104, character/main1.js:1-444}` | [`shoushaUiSurface.ts`](../../../../apps/web-client/src/engine/surfaces/shoushaUiSurface.ts#L1-L337)（53 项手杀阶段提示/木牛流马自动理牌/露头/语音钩子与 40 项 `lbtn`/`skill`/`character` 界面模块）、[`UiSkinSuitePanel.vue`](../../../../apps/web-client/src/components/battle/surfaces/UiSkinSuitePanel.vue#L1-L411)（保持纯 UI 包 0 VM 技能隔离） | **141 / 0 / 0**（48 配置、53 钩子、40 表面） | `pnpm --dir apps/web-client test`（**486/486 PASS**）+ Headless Chrome `60.0 FPS` |
| **`L24 皮肤切换`** | `skin-switch-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/皮肤切换/{extension.js:1-11955, skinParams.js}` | [`skinSwitchSurface.ts`](../../../../apps/web-client/src/engine/surfaces/skinSwitchSurface.ts#L1-L340)（22 项出框攻击 flipX/变身/语音队列钩子与 10 项藏珍阁/特效调参面板）、[`Spine40SkinImportBox.vue`](../../../../apps/web-client/src/components/battle/surfaces/Spine40SkinImportBox.vue#L1-L260) | **46 / 0 / 0**（14 配置、22 钩子、10 表面） | `pnpm --dir apps/web-client test`（**486/486 PASS**）+ Headless Chrome `60.0 FPS` |
| **`L25 祖安武将`** | `zuan-upstream-liuli` | `archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:1-51473` | [`zuan_data.rs`](../../../../crates/content-native-loader/src/zuan_data.rs#L1-L376)（60 名真实 `ssr_*` 武将、8 张专属牌、84 项技能入 `zuan-upstream-liuli` 正式包）；内核 [`decision34_l25_ssr.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l25_ssr.rs#L1-L841)、[`decision34_l25_rules.rs`](../../../../crates/engine-native-kernel/src/rvm_ops/commit_up37/decision34_l25_rules.rs#L1-L544)；前端 [`zuanSurface.ts`](../../../../apps/web-client/src/engine/surfaces/zuanSurface.ts#L1-L179)（40 项元气/台词/DIY 界面钩子与联机拒绝门禁） | **262 / 0 / 0**（60 将、84 技、8 卡、70 配置、40 钩子） | [`decision34_round4_l25_zuan.rs`](../../../../crates/engine-native/tests/decision34_round4_l25_zuan.rs#L1-L263)（**1/1 PASS**） |

### G0 37 来源逐符号核对汇总（`scripts/legacy-harness/decision34_inventory_builder.py`）

- 分类器严守无 `sid in (...)` 整源强行标绿，`else:` 默认分支保持 `"缺失"`，全部按正式包与装载器/内核/前端实际注册符号集逐项核验：
  - **上游总条目**：**4,435 项**（武将 **834**、技能 **1,914**、卡牌 **380**、模式/战役节点 **215**、配置项 **436**、界面钩子 **507**、渲染面 **149**）
  - **`忠实实现`**：**4,435 项（100%）**
  - **`示意或自造`**：**0 项**
  - **`缺失`**：**0 项**
  - **仓内 7 维（①功能、②配置、③依赖、④组合、⑤保存恢复、⑦性能、⑧规则差异）闭合来源**：**37/37 来源**（其中涉及 Spine 动皮的 7 个来源 `L08, L09, L10, L11, L12, L24, L28` 已完成净室 Spine 4.0 播放器、无资源/非 4.0 静态立绘回退及无头浏览器 WebGL 批次实测，仍待用户本机使用自有 Spine 4.0 动皮与旧版无名杀同场景截图对照）
  - **整包八项全满签署**：**`0/37`**（待第⑥维移动端 Android/iOS 物理真机与上述用户本机动皮截图对照）

---

## 三、真实无头 Chrome WebGL / DOM 表面截图与 60 帧性能实测

执行命令：

```bash
node apps/web-client/scripts/decision34-browser-surfaces.mjs docs/operations/evidence/2026-11-10-decision34-round4-37-closure
```

实测结果（原始输出见 [`decision34-browser-surfaces-report.json`](decision34-browser-surfaces-report.json)，截图见 [`decision34-browser-surfaces.png`](decision34-browser-surfaces.png) 与 [`decision34-taixu-campaign-surface.png`](decision34-taixu-campaign-surface.png)）：

- **`passed`**: `true`（`consoleErrors: []`）
- **面板与子画布尺寸**：主面板 `1160×1306`、UI/动皮套装区 `1121×502`、Spine 4.0 导入 WebGL 画布 `122×54`、共享 Spine WebGL 画布 `162×54`
- **Spine 4.0 净室导入与静态回退实证**：
  - 未导入时：`static-portrait-fallback — NO_DYNAMIC_SKIN_ASSET_IMPORTED`
  - 导入 4.0 骨骼后：`ready-spine40 — SPINE_40_LOADED:4.0.64`，`l24Meta: "Spine 4.0 净室 · 动作:gongji · 偏移(24,-8) · 缩放 108% · 资产状态:ready-spine40 · WebGL批次:2/8△"`
- **5 大 UI/战役表面交互与存档恢复实证**：
  - `L08 十周年UI`：`decadeLayout.css · standard`，技能标记栏 `"成略[阴] 暴戾[2] 🍶酒态"`
  - `L09 千幻聆音`：`newui_lolbig.css · 共享映射：sb_caopi → re_caopi (台词 rexingshang) · BGM：qhly_bgm/re_caopi.mp3`
  - `L14 手杀ui`：`出牌阶段 (控件#2) 提示：出牌阶段，请选择一张牌 · 自动牌序：[102,103,101] · 露头(ZLLT)：关`
  - `L24 皮肤切换`：`弹窗:cang-zhen-ge · 变身:jisha_form 出框：GongJi(flipX=true) · 语音队列：sha_fire`
  - `L11 太虚幻境`：`extension_MobileStyle.css (rem) 赛季：ChongYingChuLin · 侍灵：jiuwei (frame3) · 金币：150 (巨贾冻结:是)`；战役地图节点 `#2 (颍川前哨（战斗）)`、赐福 2 项、灵宠 `苍雷幼麟`
  - `L25 祖安武将`：`已应用离线 DIY: ssr_sunquan (5血)`，联机拦截 `ONLINE_MULTIPLAYER_DIY_FORBIDDEN`
  - 同锁存读档往返（`beforeSaveDaoxin: "50"` → 改写 `"80"` → 读档恢复 `"50"`）与重开局重置（`"20"`）全部一致
- **60 帧 `requestAnimationFrame` 帧率指标**：
  - `n = 60, p50_ms = 16.66, p95_ms = 16.67, p99_ms = 16.67, max_ms = 16.67, approx_fps = 60`

---

## 四、第五节全量门禁实跑记录

1. **源码行数、文档行号与分层棘轮**：
   - `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` -> `Ran 2 tests OK`
   - `python3 -B scripts/check-source-lines.py` -> 通过（`9,242` 个受维护文件全部 `<= 2,000` 行，Rust 非测试源码全部 `<= 1,000` 有效行）
   - `python3 -B scripts/check_doc_line_refs.py --strict` -> 通过（`6,344` 处引用，`0` 越界）
   - `python3 -B scripts/check-native-layering.py` -> `engine-native layering passed: 0 upward references <= 0`
2. **Rust 定向、原生三 crate、独立 `-D warnings`、全工作区与跨目标一致性**：
   - `cargo test --offline -p engine-native --test g1_shahai` -> **`180 passed; 0 failed; 0 ignored`**（含单子包独立装载与跨子包借技装配守卫 `u05_single_subpackage_isolation_and_cross_subpack_general_assembly`）
   - `cargo test --offline -p engine-native --test misc decision34_` -> **`15 passed; 0 failed; 0 ignored`**（含 `decision34_round4_l25_zuan` 1/1、`decision34_round4_l11_taixu` 1/1、`decision34_pack9_closure` 6/6、`decision34_rules_and_campaigns` 7/7）
   - `cargo test --offline -p game-server --test nc02_nc05_extension_lifecycle_matrix --test nc08_install_matrix` -> **`47 passed; 0 failed; 0 ignored`**（`44/44` + `3/3`，全 37 来源 46 扩展包服务端 `load_scoped` 驱动与存档恢复全绿；首跑曾检出 `U05` 跨子包借技重名与 `play/wuxing` 归包碰撞，已在 `shahai_attach.rs` 与 `apps/game-server/src/native.rs` 根因修复）
   - `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native` -> **`2,225 passed; 0 failed; 6 ignored`**（剩余 6 个 `#[ignore]` 均为手工诊断探针）
   - 独立 `CARGO_TARGET_DIR` 下 `CARGO_INCREMENTAL=0 RUSTFLAGS="-D warnings" cargo check --offline -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader -p desktop-client --all-targets` 与 `cargo check --offline --target wasm32-unknown-unknown -p engine-native-wasm` -> **exit 0（0 warnings）**（注：`game-server` 因仍依赖旧线 `engine-core` 存量 107 警告按 `.github/workflows/ci.yml` Gate 1b 口径单独 `cargo check --offline -p game-server --all-targets` 通过），临时目录已即时清理（裁决 23，磁盘余量 `95 GiB`）
   - `cargo test --offline --workspace --no-fail-fast` -> **`7,618 passed; 0 failed; 8 ignored`**（177 套件全绿）
   - `cargo run --offline -p conformance` -> `19/19 ALL PASS`
   - `cargo test --offline -p desktop-client` -> `20 passed; 0 failed; 1 ignored`（保持 WASM 默认与 `?desktopEngine=native` 双路径测试通过，裁决 36）
   - `python3 -B scripts/check-native-wasm-cross-target.py` & `bash scripts/cross-target-hash.sh` -> `ALL CROSS-TARGET CHECKS PASSED`
3. **前端单测、Typecheck、Native WASM 构建、Worker Smoke 与 `preflight.py`**：
   - `node --test apps/web-client/scripts/spine40-cleanroom.test.mjs` -> `9 passed; 0 failed`
   - `pnpm --dir apps/web-client test` -> `486 passed; 0 failed`
   - `pnpm --dir apps/web-client typecheck` -> `exit 0`
   - `pnpm --dir apps/web-client build:native` -> `exit 0`
   - `pnpm --dir apps/web-client smoke:native` -> `83/83 PASS (failed: 0)`
   - `python3 -B scripts/preflight.py` -> **`21/21 全部通过 ✅`**
4. **按裁决 25 发布与 exact-head CI 实跑登记**：
   - 第四轮代码提交 `3d6b4eb31d62a26cfed05430d0485b6e64b7dd61`（`3d6b4eb31`）经单命令作用域 `pre-push` 守卫（原钩子 `.git/hooks/pre-push` `sha256: e349af1ec5ccc1742df7e7d91d387ed2ef086f7c864d8cf7372d0ef66dd49ca7` 零改动、远端 `main` 零改动、未新增 `remote`）推送至 `refs/heads/native-convergence`；
   - Exact-head CI Run [`42718816733`](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/42718816733)（原始记录见 [`ci-run-42718816733-3d6b4eb31.json`](ci-run-42718816733-3d6b4eb31.json)）**首次运行（Attempt 1，零补跑）10/10 全部通过 `conclusion: success`**（`native-web` `129507216752`、`perf-ubuntu-latest` `129507217116`、`gates-ubuntu` `129507217129`、`desktop-ubuntu-latest` `129507217132`、`perf-macos-latest` `129507217173`、`build-ubuntu-latest` `129507217178`、`desktop-windows-latest` `129507217307`、`build-macos-latest` `129507217320`、`build-windows-latest` `129507217349`、`desktop-macos-latest` `129507217412` 全绿）。

---

## 五、剩余外部条件与用户本机核对步骤

1. **裁决 35 用户本机 Spine 4.0 动皮截图对照步骤（产品不附带外部版权动皮/配音资产）**：
   - 在浏览器打开 `apps/web-client` 战局扩展表面面板，展开「Spine 4.0 净室动皮导入（`dynamicSkin.js` / `saveSkinParams`）」；
   - 选择本机自有的 Spine `4.0.x` 武将动皮资产（`.json` 或标准 `.skel` 二进制骨骼 + `.atlas` 图集 + `.png` 纹理解析图），填入对应 `action`（如 `DaiJi` / `GongJi`）与 `x, y, scale` 参数点击导入；
   - 与旧版无名杀同武将动皮同动作帧进行同场景视觉对照；若导入 `3.5`/`3.7`/`3.8`/`4.1`/`4.2` 骨骼，确认界面按规则基准显示静态立绘回退提示。
2. **PA08 / PA11 / NC12 移动端真机与三平台签名公证凭据**：
   - 待接入开启 USB 调试的中低端 Android 真机与解锁信任的物理 iPhone（裁决 19），执行整局与 `>= 30 min` 功耗／温度／降频实测，据此判定 PA11 并完成 37 来源第⑥维移动端目标平台签署；
   - 待配置 Apple Developer ID 签名证书 + 公证凭据与 Windows Authenticode 证书，执行三平台安装包真实签名安装验证；
   - 本子代理运行于沙箱环境（`adb` / `xcrun` / `security find-identity` 被系统沙箱策略拦截，沿用同日 02:24 主环境探测状态：Android 空、物理 iPhone `unavailable`、`codesigning` `0 valid identities found`）；第六阶段（旧线安全退出）在上述门槛齐备并获用户确认前保持未开始。
