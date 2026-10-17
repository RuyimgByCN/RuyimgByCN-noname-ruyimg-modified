# T12 L29 龙血玄黄：八项整包验收（2026-10-17）

- 日期：2026-10-17
- 来源：`archive/upstream/noname-liuli/noname/extension/龙血玄黄/`（`extension.js` 587KB 混淆入口；LX.1 可读化 + LX.2 动态采样 + LX.3 清单定案 2026-10-08）
- 真实包：`packages/extensions/longxue-upstream-liuli`（`upstream-liuli/longxue@1.0.0`；资产 `assets/cards.json` + `assets/config.json`）
- 八项测试：`crates/engine-native/tests/l29_longxue_eight.rs::l29_eight_dimension_from_fresh_install`
- **整包计数：19/37 → 20/37**

## 交付

| 面 | 内容 |
|---|---|
| 规则面（内核） | 潜龙 `mbqianlong`（tag 469）：开局面 +20（另有魏势力持卫统者 +60）、获得牌 +5、受伤 +10×伤害、造成伤害 +15×伤害，上限 99；阈值 25/50/75/99 解锁清正/酒诗/放逐/绝进（目录存在才授予）；卫统 `mbweitong`（tag 470）登记面；清正 `mbcmqingzheng`（tag 474）已转换（批 119） |
| 装载器模块 | `crates/content-native-loader/src/longxue.rs`——`parse_longxue_config`（`config.shengjian` 圣剑显示开关 + 来源/版本/作者字段校验）、`longxue_unlock_report`（阈值解锁报告，外部依赖标注 `(external)`）、`LONGXUE_THRESHOLDS` |
| 安装包 | `upstream-liuli/longxue@1.0.0`：宿主武将 `mb_caomao`（男/魏/3/主公）+ 包级技能段（清正已映射；酒诗/放逐/绝进 `mapping:null` + 理由）；资产仅 JSON |
| 显示面 | **登记差异**：`_CMeffects`（cmarea/cmjian DOM + Spine）、`dieAnimation`（SS_CM_zhenwang）、进度条 `updateProgress` 均不迁移 |

## 八项

| 项 | 正例 | 反例 |
|---|---|---|
| 功能 | 开局面 +20（非当前回合座位，隔离摸牌）、获得牌 +5 → 25 阈值**授予清正**、受伤 2 点 +20、造成 3 点伤害 +45（45→90） | 三个外部技能不入目录（`mbcmjiushi`/`mbcmfangzhu`/`mbjuejin`，不虚构空操作）；未持潜龙者道心值恒 0 |
| 配置 | 真实 `config.json` 解析：`source=noname-liuli`、`package_name=龙血玄黄`、`package_version=1.0`、`shengjian=true`（上游 `config.shengjian.init:true`）、路径含「龙血玄黄」 | 坏 JSON → `Json`；缺 `source` → `Missing`；空 `source` → `BadField`；缺 `settings` → `Missing`；`shengjian` 非布尔 → `Missing` |
| 依赖 | `manifest.dependencies` 为空；同目录其余包照常安装 | 声明 `missing-base` 的副本安装失败（`import_extensions` 返回 Err） |
| 组合 | 同一次 `import_extensions` 仍安装 `upstream-liuli/coin@1.0.0` 等；本包只登记 1 名宿主武将、包名 `longxue-upstream-liuli` | 缺依赖副本不污染已装包（商店 2 独立根） |
| 保存恢复 | 道心值 90 + 清正实例存档往返保持；恢复后规则版本校验通过 | 篡改 `ruleset_version=1` 的存档被拒 |
| 安装升级 | 重装幂等：`InstalledMeta.content_hash` 不变、版本仍 `1.0.0`、包数据字节不变 | 失败安装（缺依赖）不改动已装版本 |
| 目标平台 | 商店副本 `assets/cards.json` 可读、含 `mb_caomao` 与 `host:mbqianlong`；资产清单无图片/音频变体 | 包数据无 HTML/显示面载荷（`<img`/`innerHTML` 均无） |
| 规则差异 | 阈值报告：24 → 未解锁；25 → 清正；99 → 清正 + 三项 `(external)` | `_CMeffects`/`playSpine` 不在包数据、不在内核目录（显示面不迁移） |

## 命令

```
cargo test --offline -p engine-native --test l29_longxue_eight
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native-wasm --lib
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（八项） | **1 passed / 0 failed**（[targeted.log](targeted.log)） |
| engine-native 全量 | **921 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| wasm | **20/0**（[wasm-tests.log](wasm-tests.log)） |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| 14 golden | `replay_scenarios` 2/2 零位移（无字段/版本变化） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |

## 规则差异（本包登记）

1. **开局面时点**：上游 `phaseNumber==0`（准备阶段前）→ 原生游戏首个阶段帧（准备阶段开始无帧，时点后移一个阶段；`phase_number == 1` 精确限定一次）。
2. **开局摸牌不计 gain**：上游 `gameDraw` 不触发 `gainEnd`；原生以「首个阶段帧之前不计」近似。
3. **显示面**：`_CMeffects`/死亡动画/进度条不迁移（无 DOM/Spine 宿主）。
4. **外部依赖**：酒诗（需「翻面后」事件与翻面授予原语）、放逐（需技能禁用原语）、绝进（需「移出游戏」区）三技未转换；`mb_caomao`/`mb_epic_caomao` 属 liuli 手杀包（非本来源）。
5. **态度/势力**：卫统 +60 的「另一魏势力持主公技」以「魏势力 + 持本技」近似（原生未建模主公技资格）。

## 影响

- 台账 L29 行改为八项整包；`T12-WHOLEPACK-0-37`/`COV-EXT-37` 计数 19/37 → **20/37**；`T9-SOURCE-BEHAVIOR-SAMPLES` 的 L29 功能面基线随之更新（规则面已迁）。
- `D5-LX-MIGRATION` 保持 open（余三技与显示面登记）。
