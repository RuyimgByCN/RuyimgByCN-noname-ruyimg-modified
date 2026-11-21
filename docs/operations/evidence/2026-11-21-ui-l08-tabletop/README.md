# Batch UI-1: L08 十周年UI 真实牌桌布局验证证据

- 验证日期：2026-11-21
- 工作分支：`swarm/conv-ui`（基线 `af6ef4899`）
- 范围：L08 `十周年UI` 真实牌桌布局重构与复核整改（2–8 人环绕席位、角标身份印记、全量 175 种 `CardKind` `def` 牌名/类别映射、全幅画布雪花粒子散点与不透明禁用牌面、桌面/平板 `768px–1440px` 与移动端 `390x844` 零重叠零裁切响应式、以及真实对局阶段 CDP 对比）。未触及 Rust crate、`inventory_status.json`、`native-progress.md` 或任何闭源 Spine 资源。

## 上游参考边界（cleanroom / non-spine）

- 仅参考上游 `archive/upstream/noname-liuli/noname/extension/十周年UI/` 的布局与样式规则：`decadeLayout.css`、`layout.css`、`layout_new.css`、`player_new.css`、`component.css`、`equip.css`、`equip_new.css`、`dialog_new.css`、`card.css`、`icon.css`、`extension.js`（未读取上游 `spine.js`、`皮肤切换/spine-lib/*` 或 `皮肤切换/v/spine-pixi-v8.js`）。
- 未引入任何商业二进制皮肤资产（`.skel` / `.atlas` / `.png` / 语音文件）。

## 源码与测试变更

| 文件 | 行数 | 职责 |
| --- | ---: | --- |
| `apps/web-client/src/engine/surfaces/cardCatalogLabels.ts` | 205 | 对齐 `catalog.rs` 全部 175 个 `CardKind` 判别式（`0..=174`）提供牌名、类别与角标 |
| `apps/web-client/src/engine/surfaces/decadeTabletopLayout.ts` | 237 | `computeRelativeSeatOrder`、`computeDecadeSeatAnchor`（768px+ 防碰撞坐标）、`identityBadgeInfo`、`cardKindLabel`、`cardCategoryOf`、`cardCategoryLabel` |
| `apps/web-client/src/engine/extensionSurfaceRuntime.ts` | 1535 | `renderCoinCanvasParticles` 全画布黄金比准随机雪花散点（保留 12 粒子口径） |
| `apps/web-client/src/components/battle/GeneralPortrait.vue` | 251 | 竖向立绘、按 `self` 条件偏移 `.reveal-pill`、移动端名称/缺图占位于上下分区隔离 |
| `apps/web-client/src/components/battle/SeatCard.vue` | 825 | 竖向武将框、自席按 `player_new.css:1079` 隐藏冗余手牌角标、`.meta` 等距不换行、`def` 判定区图标 |
| `apps/web-client/src/components/battle/HandStrip.vue` | 426 | 使用 `cardCategoryOf(c.kind, c.def)` 与 `cardCategoryLabel`、不透明 `.card:disabled`、`.strip` `overflow-y: hidden` 与移动端紧凑排布 |
| `apps/web-client/src/components/battle/BattleBoard.vue` | 1114 | `.arena-stage`（`min-height: 780px`）、2–8 人环绕席位与中央提示/操作区自然高度防碰撞、移动端双列均分 |
| `apps/web-client/src/components/battle/BattleShoushaHud.vue` | 326 | 移动端紧凑顶栏断点样式 |
| `apps/web-client/src/views/BattleView.vue` | 1452 | 移除牌桌外旧响应面板，将阶段/选牌/确认/取消/结束阶段接线至 `<BattleBoard>` |
| `apps/web-client/src/styles/global.css` | 293 | 移动端 `< 768px` 取消 `.topbar` 吸顶遮挡并收紧外边距 |
| `apps/web-client/scripts/tabletop-comparison-cdp.mjs` | 834 | 上游自动跳过手气卡进入真实出牌阶段；原生端推进至真实可用手牌点击并移除伪造动画类 |
| `apps/web-client/scripts/decade-tabletop-layout.test.mjs` | 113 | 2–8 人席位顺序、768px+ 环绕间距、身份徽章与全 175 种 `def` 映射单测 |
| `apps/web-client/scripts/tabletop-responsive-cdp.mjs` | 226 | 取消强行改写高度，在 `1440x1080`(2–8p)、`768x1024`(5p/8p)、`1024x768`(8p)、`390x844`(5p/8p) 校验零碰撞/零裁切/零溢出 |

## 同屏对比缩略图与原始截图（SHA-256）

- 同屏对比缩略图：`docs/operations/evidence/2026-11-21-ui-l08-tabletop/tabletop-comparison-thumb.jpg`（`69,782` bytes，2880×1080 无畸变 1:1 并排对比压缩图）
- 原始截图（仅存放在 `.gitignore` 忽略目录 `docs/operations/evidence-raw/ui-batch1/`）：
  - `docs/operations/evidence-raw/ui-batch1/tabletop-upstream.png`（`98,949` bytes）— SHA-256: `3104b558e98b85928145af37382b9674aa92141c32aca729651ebd0b5756d220`
  - `docs/operations/evidence-raw/ui-batch1/tabletop-native.png`（`628,213` bytes）— SHA-256: `319422d2ac8b83e9bf78e7be5abfbfefd11de6c50302116df3a6f524a8935c8d`
  - `docs/operations/evidence-raw/ui-batch1/tabletop-comparison.png`（`1,453,967` bytes）— SHA-256: `2cab9e86d8b1459a03854e4529b2489de796e57db2d2a4b7e9ba846270ca29fe`
  - `docs/operations/evidence-raw/ui-batch1/tabletop-8p-desktop.png`（`543,932` bytes）— SHA-256: `d82597c76e7b49108d6caed12e804d4c2557582159f9b8538d58f9c676e28401`
  - `docs/operations/evidence-raw/ui-batch1/tabletop-5p-mobile.png`（`190,977` bytes）— SHA-256: `2d502b166919f83816947c1ae401f25aa97744794f4b26dffdddb64fd9451177`
  - `docs/operations/evidence-raw/ui-batch1/decision34/decision34-battle-table-hud.png` — SHA-256: `e4eb66ee534680ca5b99d73c1b711516c230019ef323766088987e4df3bf2d57`
  - `docs/operations/evidence-raw/ui-batch1/decision34/decision34-browser-surfaces.png` — SHA-256: `2fb612777fa5c9a1dcdd8309c11b5b7d0313779f57b21b57baec2276c90c849f`
  - `docs/operations/evidence-raw/ui-batch1/decision34/decision34-legacy-liuli-reference.png` — SHA-256: `8e8c53ffe5957b47f21e436b743676cbbf39ed135f72c06faf48f5f90be6ba1b`
  - `docs/operations/evidence-raw/ui-batch1/decision34/decision34-modal-taixu-campaign.png` — SHA-256: `b2d6e4d8277a0650f61ada6f3b9b52facfab5107dca14f5b33c77fff8353af7a`
  - `docs/operations/evidence-raw/ui-batch1/decision34/decision34-taixu-campaign-surface.png` — SHA-256: `77aa1ed4b78bd74d3ed3b683bd6091602d0d1a387243576d46a5498e9af7b2d4`
  - `docs/operations/evidence-raw/ui-batch1/decision34/decision34-ui-skin-suites.png` — SHA-256: `a4aecec5c7d806dd210010cbb7ec88acd8b3aa16e1300df74c0dc328ef3d1d9e`

## 验证命令与实测结果

注：在独立工作树验证期间曾临时软链 `noname-next/apps/web-client/{node_modules,src/worker/native-pkg,src/wasm-pkg}` 以复用预编译 WASM 产物与依赖（未在工作树重复运行 `build:native` 全量编译），测试结束后已删除临时软链并还原 `apps/web-client/dist/`。

1. `pnpm --dir apps/web-client run typecheck`（`vue-tsc --noEmit`）— exit 0。
2. `pnpm --dir apps/web-client test` — `493` pass / `0` fail / `0` skipped（`node --import ./scripts/ts-register.mjs --test`）。
3. `pnpm --dir apps/web-client run smoke:native` — `{"total":83,"failed":0}`。
4. `TABLETOP_EVIDENCE_DIR=docs/operations/evidence-raw/ui-batch1 TABLETOP_THUMBNAIL_PATH=docs/operations/evidence/2026-11-21-ui-l08-tabletop/tabletop-comparison-thumb.jpg node apps/web-client/scripts/tabletop-comparison-cdp.mjs` — exit 0；120 帧采样 `avg_fps: 60`（`p50_ms: 16.66`, `p95_ms: 16.67`, `p99_ms: 16.67`, `max_ms: 16.67`），`seat_count: 5`，`hand_card_count: 7`，`has_card_selection: true`，`battleboard_fully_visible: true`。
5. `node apps/web-client/scripts/tabletop-responsive-cdp.mjs` — 无高度覆写，全部矩阵（`2p`–`8p` @ `1440x1080`、`5p`/`8p` @ `768x1024`、`8p` @ `1024x768`、`5p`/`8p` @ `390x844`）均 `overlaps: []`、`overflowX: false`、`topbarClipped: false`、`hudTop >= 0`，其中 5p 移动端 `handBottom: 770 <= 844`（`ALL 2-8 PLAYER DESKTOP, TABLET & MOBILE LAYOUT CHECKS PASSED!`）。
6. `node apps/web-client/scripts/decision34-browser-surfaces.mjs docs/operations/evidence-raw/ui-batch1/decision34` — `passed: true`，`approx_fps: 60`，`consoleErrors: []`。
7. `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` — 2 tests OK。
8. `python3 -B scripts/check-source-lines.py` — `Source line limit passed: 9376 files <= 2000 lines; Rust non-test files <= 1000 effective lines`（以提交后复跑为准）。
9. `python3 -B scripts/check_evidence_budget.py` — 通过（单文件 `<= 200 KB`，批次 `<= 2 MB`，无禁用资产扩展名）。
