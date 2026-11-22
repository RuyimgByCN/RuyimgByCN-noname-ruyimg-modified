# Batch Spine-Tests（裁决 35 Spine 4.0 净室全特性覆盖与独立洁净审计）

- **基线 HEAD**：`aca0cc0a5ed2ac30fb3f50647e396d4cabe2dc76`（Batch L11-1 已发布并补录 [CI 运行台账](../ci-runs.md)，CI Run `44986813858` Attempt 1 `10/10 success`）
- **合入来源**：`swarm/conv-spine-tests` `a34f19a95..6913a167e2100e14f43eb1cb766924f14c009bf6`（`902ac9030`、`6913a167e`）
- **版本与内容锁**：规则 `333` / NGSA `107` / content schema `5` / ABI `759` / WS `8`（零变更）；14 项标准 golden 哈希与 `u01-dddanzhi-pending-v107.ngsa` 不变
- **清单计数**：零清单项变更（全仓保持 **`1,720 已复核等价 / 2,704 简化近似 / 0 缺失 / 0 上游未定义`**；整包八项签署仍 **`0/37`**）

---

## 1. 独立洁净室来源声明（`#143` `verifier-spine-cleanroom` `APPROVE`）

- **未受污染审计边界**：独立审计员 `verifier-spine-cleanroom` 在审计全程从未打开、读取、检索或引用任何官方 Esoteric Software Spine 运行库源码、`pixi-spine`、`spine-core`、官方示例资产（如 `spineboy`）或上游随附脚本（`archive/upstream/**/spine.js`、`spine-lib/*`、`spine-pixi-v8.js`），审查严格限定于洁净实现代码与裁决 35/37/38 规范。
- **零外部 Spine 依赖**：`apps/web-client/package.json` 仅含 `@tauri-apps/api`、`vue`、`vue-router` 及构建/类型检查开发依赖；无 `@esotericsoftware/*`、`pixi-spine` 或 `spine-core`。
- **纯 Web API 自主实现**：`apps/web-client/src/engine/spine40/{types.ts (391L), formatCodec.ts (689L), binarySkelCodec.ts (1423L), poseSolver.ts (1209L), webglRenderer.ts (900L), handcraftedFixtures.ts (352L), index.ts (7L)}` 与 `apps/web-client/src/components/battle/surfaces/Spine40SkinImportBox.vue (340L)` 零 `node:*` 导入、零 `Buffer`、零拷贝官方运行时特征，且零随包商用 `.skel`/`.atlas`/动皮资源。
- **纯手工合成测试夹具**：`handcraftedFixtures.ts`（`hero_cleanroom40.png` 合成四区域图集；`c1ea4005e4000001` 9 骨骼与 `daiji`/`gongji`/`jineng`/`shouji`/`zhenwang`/`breath_add`）与 `apps/web-client/scripts/spine40-cleanroom.test.mjs`（1,346 行，24 项测试）仅使用手写 JSON、自编码二进制 `.skel` 与合成 `.atlas`。

---

## 2. 裁决 35 十二项特性 1-对-1 审计矩阵（`24/24` 通过）

| # | 裁决 35 特性要求 | 洁净实现（`apps/web-client/src/engine/spine40/` 与 `surfaces/`） | 对应验证测试（`spine40-cleanroom.test.mjs`） | 审计结论 |
|---|---|---|---|---|
| (a) | 4.0.x 版本门禁（拒绝 3.5/3.7/3.8/4.1/4.2→`static-portrait-fallback`） | `formatCodec.ts:35-39,172-175`、`binarySkelCodec.ts:45-48,806-812`、`webglRenderer.ts:862-899` | Tests (1/7), (7/7), (8/9), (22) | **PASS** |
| (b) | JSON 与标准二进制 `.skel` 往返（zig-zag 有符号变长整数、8B hash、`nonessential=false`、`LinkedMesh` type 3、`events`、`path.lengths`） | `formatCodec.ts:166-554`、`binarySkelCodec.ts:50-1423`、`handcraftedFixtures.ts:183-190` | Tests (2/7), (8/9), (10/11), (22), (24) | **PASS**（仅合成往返，实第三方 `.skel` 边界见用户指南） |
| (c) | 全 5 种骨骼 `TransformMode`（`normal`/`onlyTranslation`/`noRotationOrReflection`/`noScale`/`noScaleOrReflection`）含旋转与负缩放镜像 | `poseSolver.ts:287-373`（含 `#140` 修复：`noScale` 下 `R * diag(1, reflect) * L`） | Tests (3/7), (12), (24) | **PASS** |
| (d) | 插槽与运行时正/负 `drawOrder` 时间轴（单/多偏移且首关键帧前不误触） | `poseSolver.ts:1077-1114`（含 `#140` 修复：`activeOrder=null` 与定槽填充） | Tests (10/11), (13), (18), (24) | **PASS** |
| (e) | 默认皮肤与命名皮肤覆盖（`gold_armor` 覆盖与 `default` 回退） | `formatCodec.ts:215-271`、`poseSolver.ts:267-285` | Tests (5/7), (10/11), (13) | **PASS** |
| (f) | 全 6 种附件（`region`/无权 `mesh`/加权 `mesh`/凸与凹耳切 `clipping`/`path`/`boundingbox` 命中测试） | `poseSolver.ts:1103-1186`、`webglRenderer.ts:90-297,352-467` | Tests (5/7), (8/9), (11/11), (14), (23) | **PASS** |
| (g) | 1 骨/2 骨 IK（含 `stretch`、`mix`、`bendPositive` 与超距钳制）、`TransformConstraint` 与 `PathConstraint` | `poseSolver.ts:382-644`（含 `#140` 修复：`parentWorldRot` 与 `normalizeDeg`） | Tests (4/7), (15), (16), (17), (24) | **PASS** |
| (h) | 全 12 种时间轴通道（`rotate`/`translate`/`scale`/`shear`/`rgba`/`attachment`/`deform`/`ik`/`transform`/`path`/`drawOrder`/`event`）及三种插值 | `poseSolver.ts:650-1120` | Tests (6/7), (18), (24) | **PASS** |
| (i) | 多轨道动画混合（`replace`/`add`/`alpha`） | `poseSolver.ts:842-955,1032-1074` | Tests (6/7), (19) | **PASS** |
| (j) | 多页 `.atlas` 解析（4.0 紧凑与旧版格式）及 `rotate: 90` UV 重映射 | `formatCodec.ts:560-689`、`webglRenderer.ts:413-457` | Tests (7/7), (20) | **PASS** |
| (k) | WebGL 批次几何、GLSL 着色器、RGBA 纹理、PMA/非 PMA 四混合模式（`normal`/`additive`/`multiply`/`screen`）与 Canvas2D 回退 | `webglRenderer.ts:40-812`（含 `#140` 修复：`multiply` `srcAlpha=GL_ONE`） | Tests (7/7), (9/9), (21), (24) | **PASS** |
| (l) | `dynamicSkin.js`/`saveSkinParams` 本地导入面板、确定性 `static-portrait-fallback` 与用户引导文档 | `webglRenderer.ts:814-900`、`Spine40SkinImportBox.vue:1-340` | Tests (7/7), (8/9), (22) ＋ Chrome CDP | **PASS** |

---

## 3. Round 1（`#140`）6 项数学/编解码/混合缺陷闭环（`902ac9030` + `6913a167e`）

1. `noScale` 镜像矩阵乘法顺序：改为先对父级 Y 轴列向量乘 `reflect`（`R * diag(1, reflect) * L`），消除带子级局部旋转时的镜像角度偏差。
2. `drawOrder` 时间轴两处修复：`sampleTime < anim.drawOrder[0].time` 保持默认槽序不再提前触发；多偏移关键帧改为目标槽先落位再顺填未移动槽，避免顺序 `splice` 相互覆盖。
3. `applyPathConstraint` 扣除父骨骼世界旋转角 `parentWorldRot`，保证任意旋转父骨骼下的世界切线角度精确。
4. `applyTransformConstraint` 对世界 `targetShearY` 加 `normalizeDeg(...)` 折叠，避免 `target.rotation = 120` 时出现 `-360°`/`-180°` 溢出。
5. `multiply` 混合模式调整为 `srcAlpha = GL_ONE, dstAlpha = GL_ONE_MINUS_SRC_ALPHA`，消除不透明底图混合后 alpha 被清零缺陷。
6. 二进制 `.skel` `PathAttachment.lengths` 无损往返对齐 `vertexCount = knotCount * 3`，并补齐 12 通道 / 超距 IK / 全部 6 项回归测试断言（Test 24）。

---

## 4. 玩家本地验证入口与视觉签署边界（裁决 35 §5 / 裁决 37）

- **用户本地并排对照指南**：见 [Spine 4.0 玩家动皮本地导入与截图对照验证指引](../2026-11-11-tabletop-and-spine40/spine40-user-guide.md)（说明自备 `.json`/`.atlas`/`.png` 本地上传、WebGL/Canvas2D 实时预览、以及 `.skel` 二进制仅经过自研编码器合成流验证的提示）。
- **诚实签署边界**：涉及动态皮肤的 7 个来源（`L08`、`L09`、`L10`、`L11`、`L12`、`L24`、`L28`）在**八项验收的“视觉/动皮呈现对照”维度保持“部分”**，待用户在本地用自有 Spine 4.0 资源完成并排截图对照后方可终签。

---

## 5. `main` 全量未过滤门禁实测记录

1. **前端与 Chrome CDP 验收**：
   - `pnpm --dir apps/web-client run typecheck`：exit `0`
   - `node --import ./apps/web-client/scripts/ts-register.mjs --test apps/web-client/scripts/spine40-cleanroom.test.mjs`：**`24` passed / `0` failed**
   - `pnpm --dir apps/web-client test`：**`540` passed / `0` failed**
   - `CARGO_INCREMENTAL=0 pnpm --dir apps/web-client run build:native` / `smoke:native`（`{"total":83,"failed":0}`、`sources=37/37 packages=46/46`） / `build`（含 `git checkout -- apps/web-client/dist/`）：exit `0`
   - 无头 Chrome `node apps/web-client/scripts/decision34-browser-surfaces.mjs /tmp/d34-spine-cdp`（已 `rm -rf`）：`passed: true`、`consoleErrors: []`、`approx_fps: 60`（`p50=16.66ms, p95=16.67ms`）、`ready-spine40 — SPINE_40_LOADED:4.0.64` 与 `static-portrait-fallback — NO_DYNAMIC_SKIN_ASSET_IMPORTED` 验证通过。
2. **Rust 编译与全量单元测试**：
   - `CARGO_TARGET_DIR=/private/tmp/noname-target-spine CARGO_INCREMENTAL=0 RUSTFLAGS="-D warnings"` native `--all-targets`（5 crate）与 `wasm32-unknown-unknown`（`engine-native-wasm`, `engine-native`）：exit `0`（`trap` 已清理）
   - `4-crate` offline：`30` 组、**`2,704` passed / `0` failed / `6` ignored**；`conformance`：**`19/19` (`ALL PASS`)** & **`4/0/0`**；`desktop-client`：**`20/0/1`**；`check-native-wasm-cross-target.py` & `cross-target-hash.sh`：`ALL CROSS-TARGET CHECKS PASSED`；`cargo test --offline --workspace --no-fail-fast`：`178` 组、**`8,052` passed / `0` failed / `8` ignored**。
3. **脚本与全仓门禁**：`test_check_source_lines.py`（`2 OK`）、`check-source-lines.py`（9,496 文件 `<= 2,000` 行）、`check-native-layering.py`（`0`）、`check_doc_line_refs.py --strict`（`10,903` 引用、越界 `0`）与 `preflight.py` 全过。
