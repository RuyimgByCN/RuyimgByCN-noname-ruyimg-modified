# 裁决 35 证据：净室 Spine 4.0 播放器、功能清单测试、静态回退与本机截图对照步骤

- **核验日期**：2026-11-09
- **适用分支**：`main` / `native-convergence`
- **对应任务条款**：《原生架构收敛执行方案》第三节第 3a 项（裁决 35：「不引入官方 Spine 运行库，净室自写 4.0 播放器。只依据 Spine 官网公开的 JSON／二进制格式文档；实现者不得阅读或复制官方运行库源码，也不得阅读上游打包的 `十周年UI/spine.js`、`皮肤切换/spine-lib/*`、`皮肤切换/v/spine-pixi-v8.js`；不引入 `pixi-spine` 等依赖 `spine-core` 的包；不复制官方示例资源……产品不附带任何动皮资源，按上游 `dynamicSkin.js`／`saveSkinParams` 的目录与参数格式提供玩家导入入口；无资源或非 4.0 资源回退静态立绘，并在规则基准登记差异。」）

---

## 一、净室合规与零污染声明

1. **未读取任何禁读文件**：实现与清点全程在代码中硬编码拦截了 `spine.js`、`spine-lib/*`、`spine-pixi-v8.js`，未打开或复制任何官方 Spine 运行库源码及示例资源。
2. **零外部 Spine 依赖**：`apps/web-client/package.json` 未引入 `@esotericsoftware/spine-*`、`pixi-spine` 或任何第三方骨骼库；二进制编解码仅使用 Web 标准 `Uint8Array`、`DataView`、`TextEncoder`、`TextDecoder`（无 Node `Buffer` 依赖）。
3. **旧示意画法已删除**：`ExtensionSurfacePanel.vue` 中原先以 2D Canvas 圆加两条线冒充骨骼动画的 `drawSpineSkeletalCanvas` 已彻底删除。

---

## 二、净室 Spine 4.0 功能清单与手写骨骼测试映射

| 序号 | 功能清单项 | 原生实现落点 | 手写 JSON / 自写编码器二进制测试（`apps/web-client/scripts/spine40-cleanroom.test.mjs`） | 状态 |
| --- | --- | --- | --- | --- |
| 1 | 4.0 版本门禁（拒绝 `3.5`/`3.7`/`3.8`/`4.1`/`4.2`） | [`isSpine40Version`](../../../../apps/web-client/src/engine/spine40/formatCodec.ts#L33-L37) | `Decision 35 (1/7): Spine 4.0 version gate accepts 4.0.x and rejects 3.5/3.7/3.8/4.1/4.2` | 已通过 |
| 2 | JSON 与自写二进制 `.skel` 编解码器 | [`parseSpine40Json`](../../../../apps/web-client/src/engine/spine40/formatCodec.ts#L164-L464)、[`encodeSpine40Binary`](../../../../apps/web-client/src/engine/spine40/formatCodec.ts#L523-L553)、[`parseSpine40Binary`](../../../../apps/web-client/src/engine/spine40/formatCodec.ts#L558-L616) | `Decision 35 (2/7): JSON parser and self-written binary .skel encoder/decoder round-trip identically` | 已通过 |
| 3 | 骨骼层级与 5 种世界变换模式（`normal`/`onlyTranslation`/`noRotationOrReflection`/`noScale`/`noScaleOrReflection`） | [`computeBoneWorldTransform`](../../../../apps/web-client/src/engine/spine40/poseSolver.ts#L272-L358) | `Decision 35 (3/7): Bone hierarchy and all 5 TransformMode world transforms solve accurately` | 已通过 |
| 4 | 插槽（Slots）与绘制顺序偏移（DrawOrder） | [`createSkeletonInstance`](../../../../apps/web-client/src/engine/spine40/poseSolver.ts#L123-L209)、[`CleanRoomAnimationState.apply`](../../../../apps/web-client/src/engine/spine40/poseSolver.ts#L735-L953) | `Decision 35 (6/7)` & `Decision 35 (7/7)` | 已通过 |
| 5 | 皮肤（`default` 与自定义皮肤附件覆盖） | [`resolveAttachment`](../../../../apps/web-client/src/engine/spine40/poseSolver.ts#L252-L270) | `Decision 35 (5/7): Skins, all 6 attachment types ...` | 已通过 |
| 6 | 6 类附件（`region`、无权重 `mesh`、多骨骼加权 `mesh`、`clipping`、`path`、`boundingbox` 碰撞检测） | [`computeAttachmentWorldVertices`](../../../../apps/web-client/src/engine/spine40/poseSolver.ts#L956-L1024)、[`containsPointInBoundingBox`](../../../../apps/web-client/src/engine/spine40/poseSolver.ts#L1026-L1039) | `Decision 35 (5/7): Skins, all 6 attachment types (region, mesh, weighted mesh, clipping, path, boundingbox), and hit testing` | 已通过 |
| 7 | IK 约束（单骨骼与双骨骼解析解）、变换约束（Transform）、路径约束（Path） | [`applyIkConstraint`](../../../../apps/web-client/src/engine/spine40/poseSolver.ts#L367-L426)、[`applyTransformConstraint`](../../../../apps/web-client/src/engine/spine40/poseSolver.ts#L428-L465)、[`applyPathConstraint`](../../../../apps/web-client/src/engine/spine40/poseSolver.ts#L504-L545) | `Decision 35 (4/7): 2-bone IK, Transform, and Path constraints solve target positions and orientations` | 已通过 |
| 8 | 时间轴与曲线（`stepped`、`linear`、三次贝塞尔曲线 `[cx1, cy1, cx2, cy2]` 牛顿-二分求根及 `transform`/`path`/`deform` 时间轴） | [`evaluateCubicBezier`](../../../../apps/web-client/src/engine/spine40/poseSolver.ts#L573-L608)、[`CleanRoomAnimationState.apply`](../../../../apps/web-client/src/engine/spine40/poseSolver.ts#L819-L927) | `Decision 35 (6/7): Stepped, linear, cubic Bezier, transform/path/deform timelines, and multi-track mixing (replace + add)` | 已通过 |
| 9 | 多轨道动画混合（Track `alpha` 混合与 `replace` / `add` 叠加） | [`CleanRoomAnimationState`](../../../../apps/web-client/src/engine/spine40/poseSolver.ts#L705-L954) | `Decision 35 (6/7): Stepped, linear, cubic Bezier, transform/path/deform timelines, and multi-track mixing (replace + add)` | 已通过 |
| 10 | `.atlas` 图集解析（支持 `bounds`/`offsets`、`xy`/`size`/`orig`/`offset`、`rotate: 90`、`pma`） | [`parseTextureAtlas`](../../../../apps/web-client/src/engine/spine40/formatCodec.ts#L622-L738) | `Decision 35 (7/7): Atlas parser, WebGL batch geometry, premultiplied alpha blend modes, and dynamicSkin.js fallback` | 已通过 |
| 11 | WebGL 批量几何构建、WebGL 批次提交（`renderSpineBatchesToWebGL`）、裁剪多边形边界与预乘透明混合模式（`normal`/`additive`/`multiply`/`screen`） | [`resolvePremultipliedBlendFunc`](../../../../apps/web-client/src/engine/spine40/webglRenderer.ts#L40-L74)、[`buildBatchGeometry`](../../../../apps/web-client/src/engine/spine40/webglRenderer.ts#L102-L260)、[`renderSpineBatchesToWebGL`](../../../../apps/web-client/src/engine/spine40/webglRenderer.ts#L352-L386) | `Decision 35 (7/7): Atlas parser, WebGL batch geometry, premultiplied alpha blend modes, and dynamicSkin.js fallback` | 已通过 |
| 12 | `dynamicSkin.js` / `saveSkinParams` 参数导入与缺资源／非 4.0 静态立绘回退 | [`resolveDynamicSkinImport`](../../../../apps/web-client/src/engine/spine40/webglRenderer.ts#L414-L474) | `Decision 35 (7/7)` | 已通过 |

---

## 三、独立复审 Agent 净室合规与实现复核记录（按裁决 35）

1. **未照搬官方运行库源码复核**：
   - 由独立复审 Agent（`ef28937b-7c05-41e0-bea6-ad9c9c4620ad`）在不读取任何官方 Spine runtime 或上游 `十周年UI/spine.js`、`皮肤切换/spine-lib/*`、`皮肤切换/v/spine-pixi-v8.js` 的前提下，逐文件审查了 `apps/web-client/src/engine/spine40/{types.ts,formatCodec.ts,poseSolver.ts,webglRenderer.ts,handcraftedFixtures.ts,index.ts}` 及 `apps/web-client/scripts/spine40-cleanroom.test.mjs`。
   - 确认全部类型定义、JSON／`SP40` 自写二进制编解码器、2×3 仿射矩阵世界变换、双骨骼余弦定理 IK 求解、折线路径采样、二分法三次贝塞尔求值、多轨道混合及 WebGL 批次合并均为简洁自包含 TypeScript 实现，无官方运行库类名／私有字段／内部状态机照搬痕迹，且 `apps/web-client/package.json` 零外部 Spine 依赖。
2. **独立复审补齐项**：
   - 复审发现初版 `CleanRoomAnimationState.apply` 虽在 `formatCodec.ts` 解析了 `anim.transform`、`anim.path` 与 `anim.deform`，但未在 `apply()` 中插值应用到约束状态与 `slot.deform`，且缺少直接驱动 WebGL 上下文提交批次的 `renderSpineBatchesToWebGL`；已在本次复审中补齐 `transform`／`path`／`deform` 时间轴应用与 `renderSpineBatchesToWebGL` 并在 `spine40-cleanroom.test.mjs` 增加回归断言。

---

## 四、用户本机自有动皮与旧版无名杀截图对照步骤（待用户本机执行）

由于仓库与产品按裁决 35 不附带任何商业动皮资源（`.skel` / `.json` / `.atlas` / `.png`），动皮同场景截图对照需由用户在本机使用自有 Spine 4.0 动皮资源与旧版无名杀对照执行：

1. **准备自有 Spine 4.0 动皮资源**：
   - 将自有 Spine 4.0 动皮文件（`<skin>.json` 或 `<skin>.skel`、`<skin>.atlas`、`<skin>.png`）放入本机测试目录（不提交入库），确认头字段 `"spine": "4.0.x"`。
2. **在原生前端导入并渲染**：
   - 启动 `pnpm --dir apps/web-client dev`，在扩展渲染面传入 `dynamicSkin.js` / `saveSkinParams` 同构参数（`{ generalId, skinName, x, y, scale, angle, speed, action: "DaiJi" | "GongJi", version: "4.0" }`）与骨骼资产；
   - 记录 `DaiJi`（`t = 0.0s`、`t = 0.5s`）与 `GongJi`（`t = 0.3s`）三帧截图及帧率。
3. **与旧版无名杀同参数对照**：
   - 在旧版无名杀十周年UI／皮肤切换中加载同一动皮与相同 `x, y, scale, angle, action` 参数，截取同时间点画面对照骨骼姿态、网格变形、裁剪边界与混合模式。
