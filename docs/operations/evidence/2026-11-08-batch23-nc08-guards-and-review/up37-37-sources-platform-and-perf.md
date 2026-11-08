# 37 来源（46 扩展包）目标平台产品路径（Web/WASM + Server + Desktop）与同机同负载前后性能证据（2026-11-08）

- **核验日期**：2026-11-08
- **设备与工具链**：Apple Silicon macOS (aarch64-apple-darwin)，`rustc 1.93.0` (`release` profile，同进程连续测量)
- **规则 / 存档 / ABI / WS / Content Schema**：`333 / 107 / 759 / 8 / 5`
- **覆盖范围**：`docs/operations/evidence/2026-10-19-package-source-map/map.tsv` 全部 **37 个上游扩展来源（`L01–L29`、`U01–U08`）** 及其映射的全部 **46 个扩展包目录**。

## 1. 目标平台实际产品路径验证（Web/WASM + Server + Desktop）

| 目标平台 | 实际产品入口与验证链路 | 执行命令 | 结果 |
| --- | --- | --- | --- |
| **Web / WASM (`engine-native-wasm`)** | `validate_pack_json` + `validate_frozen_package_json` + `prepare_locked_content_json` → `NativeSession::new_with_packs_json` & `new_with_locked_content_json` → `auto_step` → `save_hex` / `save_locked_json` → `restore_hex_with_packs_json` / `restore_hex_with_locked_content_json` + 空锁拒装校验 | `cargo test --offline -p engine-native-wasm --test installed_package_payload` | **PASS**（37/37 来源、46/46 包通过；`8 passed; 0 failed`） |
| **Web Worker (`engine.native.worker.ts` + WASM)** | 真实 Worker 线程消息往返：`validatePackage` → `validateFrozenPackage` → `prepareLockedContent` → `startLocked` → `autoStep` → `saveLocked` → `restoreLocked`（校核 `revision` 推进与 `hash_hex` 一致） | `pnpm --dir apps/web-client smoke:native` | **PASS**（`worker_all_37_sources_46_packages_locked_matrix` `sources=37/37 packages=46/46`；总计 `83/83 PASS`） |
| **Server (`apps/game-server`)** | `NativeContent::open` → `import()`（46 包入库 `PackageStore`）→ `load_scoped(Some(ns), pid)` → `Engine::new_with_extensions` → `BoundedPlanner(64)` 8 步推进 → `encode_archive` / `decode_archive` / `Engine::restore_with_extensions` + 无锁 `Engine::restore` 拒绝 | `cargo test --offline -p game-server --test nc08_install_matrix` | **PASS**（37/37 来源、46/46 包通过；`3 passed; 0 failed`） |
| **Desktop (`apps/desktop-client`)** | `DesktopHost::import_extension_directory` + `HostSession` IPC：`validatePackage` → `validateFrozenPackage` → `prepareLockedContent` → `startLocked` → `autoStep` → `saveLocked` → `restoreLocked` + 空锁恢复拒绝 | `cargo test --offline -p desktop-client` | **PASS**（37/37 来源、46/46 包通过；`20 passed; 0 failed; 1 ignored`） |
| **Mobile（Android / iOS 物理真机）** | 物理移动设备真机安装与对局验证 | `adb devices -l` / `xcrun devicectl list devices` | **未完成（环境阻塞）**：当前主机无物理 Android 设备、iPhone 处于 `unavailable`，保持待真机接入后验收 |

## 2. 同设备、同负载前后性能数据（`none` 基线 vs 37 来源逐一启用）

- **执行命令**：`cargo run --offline --release -p engine-native --example nc12_bench -- source_matrix`
- **统一负载参数**：`seats=4, seed=42007, budget=64, lookahead=1, variant=0, build_samples=20, drive_steps=80, archive_samples=20`
- **原始 TSV 证据**：[`up37-37-sources-perf.tsv`](up37-37-sources-perf.tsv)
- **启用前基线（`none`，无扩展）**：`cards=124`, `skills=232`, `generals=0`, `build_p50=12583 ns` (`p95=37625 ns`), `cmd_p50=10125 ns` (`p95=105416 ns`), `plan_p50=6875 ns`, `submit_p50=4083 ns`, `save_p50=333 ns`, `encode_p50=2458 ns`, `decode_p50=3500 ns`, `restore_p50=13291 ns` (`p95=18667 ns`), `archive_bytes=1934`。

### 2.1 37 个来源启用前后对照表（`none` vs `src:<来源>`）

| 来源 | 名称 | 包目录 | 目录规模 (`cards/skills/generals`) | `bridge_ns` | `build_p50` (`ns`, vs `none`) | `cmd_p50` / `cmd_p95` (`ns`) | `save` / `encode` / `decode` / `restore_p50` (`ns`) | `archive_bytes` | 故障/失配 (`fault/panic/fail`) |
| --- | --- | --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| `none` | 标准目录基线（启用前） | `-` | `124/232/0` | `0` | `12583` (`+0`) | `10125` / `105416` | `333` / `2458` / `3500` / `13291` | `1934` | `0/0/0` (`OK`) |
| `L01` | AI优化 | `ai-tuning-upstream-liuli` | `124/232/0` | `293584` | `22833` (`+10250`) | `9167` / `58291` | `333` / `2334` / `3333` / `22958` | `1934` | `0/0/0` (`OK`) |
| `L02` | SJ Settings | `sj-settings-upstream-liuli` | `124/232/0` | `42209` | `22417` (`+9834`) | `10292` / `57041` | `292` / `2208` / `3084` / `21125` | `1934` | `0/0/0` (`OK`) |
| `L03` | boss | `boss-play-upstream-liuli+boss-zhugeliang-upstream-liuli` | `124/238/1` | `83208` | `23708` (`+11125`) | `8375` / `54542` | `292` / `2334` / `3292` / `23875` | `1934` | `0/0/0` (`OK`) |
| `L04` | cardpile | `cardpile-upstream-liuli` | `237/232/0` | `75542` | `37083` (`+24500`) | `13833` / `67958` | `416` / `3333` / `4375` / `41709` | `2555` | `0/0/0` (`OK`) |
| `L05` | coin | `coin-upstream-liuli` | `124/232/0` | `53625` | `23167` (`+10584`) | `9417` / `60959` | `333` / `2375` / `3375` / `22459` | `1934` | `0/0/0` (`OK`) |
| `L06` | wuxing | `wuxing-upstream-liuli` | `125/234/0` | `59250` | `24875` (`+12292`) | `11625` / `84167` | `416` / `2667` / `4000` / `23291` | `2134` | `0/0/0` (`OK`) |
| `L07` | 全能搜索 | `omnisearch-upstream-liuli` | `124/232/0` | `46084` | `22791` (`+10208`) | `8625` / `52834` | `292` / `2334` / `3209` / `21750` | `1934` | `0/0/0` (`OK`) |
| `L08` | 十周年UI | `decade-ui-upstream-liuli` | `124/232/0` | `65583` | `23083` (`+10500`) | `10625` / `55500` | `292` / `2375` / `3375` / `23250` | `1934` | `0/0/0` (`OK`) |
| `L09` | 千幻聆音 | `qhly-upstream-liuli` | `124/232/0` | `36666` | `21917` (`+9334`) | `8251` / `49750` | `292` / `2333` / `3292` / `22833` | `1934` | `0/0/0` (`OK`) |
| `L10` | 史诗卡牌 | `epic-cards-upstream-liuli` | `124/234/0` | `52084` | `22417` (`+9834`) | `8041` / `48500` | `292` / `2333` / `3291` / `22500` | `1934` | `0/0/0` (`OK`) |
| `L11` | 太虚幻境 | `taixu-upstream-liuli` | `124/232/0` | `37584` | `21750` (`+9167`) | `8042` / `48167` | `292` / `2333` / `3291` / `22417` | `1934` | `0/0/0` (`OK`) |
| `L12` | 如真似幻 | `rzsh-upstream-liuli` | `124/232/0` | `71709` | `23625` (`+11042`) | `9833` / `48125` | `292` / `2375` / `3292` / `22583` | `1934` | `0/0/0` (`OK`) |
| `L13` | 手杀MVP | `shousha-mvp-upstream-liuli` | `124/232/0` | `66542` | `21959` (`+9376`) | `8126` / `47500` | `292` / `2334` / `3292` / `22459` | `1934` | `0/0/0` (`OK`) |
| `L14` | 手杀ui | `shousha-ui-upstream-liuli` | `124/232/0` | `28583` | `22000` (`+9417`) | `8792` / `48625` | `291` / `2167` / `3083` / `21167` | `1934` | `0/0/0` (`OK`) |
| `L15` | 手杀全扩 | `shousha-full-upstream-liuli` | `124/232/0` | `300000` | `21666` (`+9083`) | `10000` / `50168` | `292` / `2333` / `3292` / `21916` | `1934` | `0/0/0` (`OK`) |
| `L16` | 拖拽读取 | `drag-read-upstream-liuli` | `124/232/0` | `51167` | `21792` (`+9209`) | `8125` / `50583` | `292` / `2333` / `3333` / `22875` | `1934` | `0/0/0` (`OK`) |
| `L17` | 斗转星移 | `douzhuan-xingyi-upstream-liuli` | `124/232/0` | `43250` | `21875` (`+9292`) | `9375` / `53334` | `292` / `2292` / `3292` / `22125` | `1934` | `0/0/0` (`OK`) |
| `L18` | 更新保护 | `update-guard-upstream-liuli` | `124/232/0` | `37250` | `21500` (`+8917`) | `8084` / `49791` | `292` / `2292` / `3292` / `22458` | `1934` | `0/0/0` (`OK`) |
| `L19` | 末日乱斗 | `doomsday-brawl-upstream-liuli` | `124/235/0` | `42833` | `22667` (`+10084`) | `8000` / `46625` | `292` / `2334` / `3292` / `23250` | `1934` | `0/0/0` (`OK`) |
| `L20` | 标记补充 | `mark-supplement-upstream-liuli` | `124/238/0` | `40334` | `23333` (`+10750`) | `8750` / `48500` | `292` / `2375` / `3292` / `23416` | `1934` | `0/0/0` (`OK`) |
| `L21` | 武皇争霸 | `martial-upstream-liuli` | `124/232/0` | `59542` | `21709` (`+9126`) | `10125` / `55501` | `292` / `2416` / `3333` / `22042` | `1934` | `0/0/0` (`OK`) |
| `L22` | 段位制裁 | `elo-sanction-upstream-liuli` | `124/232/0` | `32667` | `21458` (`+8875`) | `8208` / `47375` | `291` / `2333` / `3291` / `22000` | `1934` | `0/0/0` (`OK`) |
| `L23` | 电脑适配 | `desktop-adapt-upstream-liuli` | `124/232/0` | `32541` | `21709` (`+9126`) | `8083` / `46667` | `292` / `2333` / `3292` / `22041` | `1934` | `0/0/0` (`OK`) |
| `L24` | 皮肤切换 | `skin-switch-upstream-liuli` | `124/232/0` | `33875` | `21542` (`+8959`) | `8666` / `48625` | `292` / `2333` / `3292` / `22291` | `1934` | `0/0/0` (`OK`) |
| `L25` | 祖安武将 | `zuan-upstream-liuli` | `124/232/0` | `38417` | `21583` (`+9000`) | `8000` / `49250` | `292` / `2292` / `3292` / `22084` | `1934` | `0/0/0` (`OK`) |
| `L26` | 笨蛋插件 | `fool-plugin-upstream-liuli` | `124/232/0` | `28750` | `21334` (`+8751`) | `7958` / `49626` | `292` / `2333` / `3292` / `22250` | `1934` | `0/0/0` (`OK`) |
| `L27` | 选将美化 | `select-beautify-upstream-liuli` | `124/239/0` | `53167` | `22875` (`+10292`) | `8208` / `46791` | `291` / `2333` / `3291` / `23708` | `1934` | `0/0/0` (`OK`) |
| `L28` | 雷霆万钧 | `thunder-upstream-liuli` | `124/232/0` | `45334` | `21500` (`+8917`) | `8041` / `46876` | `292` / `2334` / `3292` / `22333` | `1934` | `0/0/0` (`OK`) |
| `L29` | 龙血玄黄 | `longxue-upstream-liuli` | `124/235/1` | `69917` | `20875` (`+8292`) | `8542` / `46792` | `292` / `2375` / `3333` / `23041` | `1934` | `0/0/0` (`OK`) |
| `U01` | 3D精选 | `d3-upstream-noname` | `124/234/1` | `48209` | `22000` (`+9417`) | `8083` / `47542` | `291` / `2375` / `3292` / `22750` | `1934` | `0/0/0` (`OK`) |
| `U02` | boss | `boss-play-upstream-noname` | `124/232/0` | `43208` | `21667` (`+9084`) | `7751` / `47583` | `291` / `2209` / `3083` / `21250` | `1934` | `0/0/0` (`OK`) |
| `U03` | cardpile | `cardpile-upstream-noname` | `237/232/0` | `72667` | `38250` (`+25667`) | `10250` / `59166` | `375` / `3250` / `4375` / `41458` | `2555` | `0/0/0` (`OK`) |
| `U04` | coin | `coin-upstream-noname` | `124/232/0` | `47583` | `21208` (`+8625`) | `9001` / `48292` | `292` / `2416` / `3417` / `23125` | `1934` | `0/0/0` (`OK`) |
| `U05` | 杀海拾遗 | `shaai-ow-upstream-noname+shaai-xianjian-upstream-noname+shahai-gujian-upstream-noname+shahai-gwent-upstream-noname+shahai-hearth-upstream-noname+shahai-mtg-upstream-noname+shahai-swd-upstream-noname+shahai-yunchou-upstream-noname+shahai-zhenfa-upstream-noname` | `333/612/149` | `914667` | `194084` (`+181501`) | `25000` / `109209` | `500` / `3959` / `5959` / `204500` | `3234` | `0/0/0` (`OK`) |
| `U06` | 欢乐卡牌 | `huanle-upstream-noname` | `131/237/0` | `73083` | `23875` (`+11292`) | `7875` / `45875` | `292` / `2333` / `3250` / `25625` | `1962` | `0/0/0` (`OK`) |
| `U07` | 玩点论杀 | `wudian-upstream-noname` | `138/251/9` | `75209` | `29833` (`+17250`) | `8541` / `47833` | `292` / `2375` / `3334` / `30583` | `1990` | `0/0/0` (`OK`) |
| `U08` | 英雄杀 | `hero-kill-upstream-noname` | `124/237/0` | `48041` | `22083` (`+9500`) | `8584` / `45833` | `292` / `2333` / `3292` / `24042` | `1934` | `0/0/0` (`OK`) |

### 2.2 多包来源（`L03` × 2、`U05` × 9）单包拆分性能对照表

| 子包标识 | 所属来源:包目录 | 目录规模 (`cards/skills/generals`) | `bridge_ns` | `build_p50` (`ns`, vs `none`) | `cmd_p50` / `cmd_p95` (`ns`) | `save` / `encode` / `decode` / `restore_p50` (`ns`) | `archive_bytes` | 故障/失配 |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| `pkg:boss-play-upstream-liuli` | `L03:boss-play-upstream-liuli` | `124/232/0` | `42000` | `22208` (`+9625`) | `8458` / `46500` | `292` / `2208` / `3125` / `22166` | `1934` | `0/0/0` (`OK`) |
| `pkg:boss-zhugeliang-upstream-liuli` | `L03:boss-zhugeliang-upstream-liuli` | `124/238/1` | `41292` | `24042` (`+11459`) | `8209` / `47833` | `292` / `2292` / `3292` / `24333` | `1934` | `0/0/0` (`OK`) |
| `pkg:shaai-ow-upstream-noname` | `U05:shaai-ow-upstream-noname` | `124/269/24` | `133834` | `33458` (`+20875`) | `8000` / `47375` | `292` / `2333` / `3292` / `34000` | `1934` | `0/0/0` (`OK`) |
| `pkg:shaai-xianjian-upstream-noname` | `U05:shaai-xianjian-upstream-noname` | `124/307/36` | `172125` | `46709` (`+34126`) | `7833` / `47333` | `292` / `2375` / `3292` / `47375` | `1934` | `0/0/0` (`OK`) |
| `pkg:shahai-gujian-upstream-noname` | `U05:shahai-gujian-upstream-noname` | `164/250/0` | `83334` | `32708` (`+20125`) | `10749` / `47375` | `333` / `2708` / `3584` / `33166` | `2121` | `0/0/0` (`OK`) |
| `pkg:shahai-gwent-upstream-noname` | `U05:shahai-gwent-upstream-noname` | `145/237/0` | `47125` | `26542` (`+13959`) | `8750` / `47959` | `291` / `2417` / `3333` / `27333` | `2018` | `0/0/0` (`OK`) |
| `pkg:shahai-hearth-upstream-noname` | `U05:shahai-hearth-upstream-noname` | `141/232/0` | `49250` | `24375` (`+11792`) | `8417` / `46416` | `292` / `2500` / `3417` / `23334` | `2151` | `0/0/0` (`OK`) |
| `pkg:shahai-mtg-upstream-noname` | `U05:shahai-mtg-upstream-noname` | `134/245/0` | `86750` | `26583` (`+14000`) | `8459` / `47626` | `291` / `2375` / `3250` / `26833` | `1974` | `0/0/0` (`OK`) |
| `pkg:shahai-swd-upstream-noname` | `U05:shahai-swd-upstream-noname` | `214/447/79` | `386917` | `109417` (`+96834`) | `21959` / `92501` | `417` / `3125` / `4667` / `108208` | `2545` | `0/0/0` (`OK`) |
| `pkg:shahai-yunchou-upstream-noname` | `U05:shahai-yunchou-upstream-noname` | `147/249/10` | `124042` | `29584` (`+17001`) | `10458` / `52458` | `292` / `2500` / `3500` / `31417` | `2058` | `0/0/0` (`OK`) |
| `pkg:shahai-zhenfa-upstream-noname` | `U05:shahai-zhenfa-upstream-noname` | `132/232/0` | `43000` | `24500` (`+11917`) | `8791` / `47459` | `291` / `2334` / `3291` / `23875` | `1966` | `0/0/0` (`OK`) |
