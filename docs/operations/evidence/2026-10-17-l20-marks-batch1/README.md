# L20 标记补充 批 1：配置门控模式修正（摸牌翻倍 / 血量翻倍）（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 来源：**L20 标记补充**（`extension/标记补充/extension.js`，356 KB；G0 见 [余下 G0](../2026-10-17-remaining-sources-g0/README.md)）
- 本批：转换包 `packages/extensions/mark-supplement-upstream-liuli`（生成器 `gen_mark_supplement_package.rs`）+ 2 个配置门控修正
- **规则/存档版本不变**（v321 / 存档 v105；无字段变化）
- 台账：L20 由「G0 完成」→ **部分重构（批 1/N）**

## G0 补充（本批实测，按体量/性质分类）

| 类别 | 数量（示例） | 说明 |
|---|---|---|
| 纯标记展示 | `yinyang_mark`(45 B)/`zhulianbihe_mark`(49 B)/`yexinjia_mark`(184 B)/`wengua`(125 B，组技) | 空定义或纯组技 → 展示面（不迁移） |
| **配置门控模式修正** | `_xigua_mopaifanbei_`(495 B)/`_xigua_xueliangfanbei_`(516 B) | **本批迁移**（上游以 `lib.config` 门控） |
| Spine 动画/音效 | `_losehp_`(743 B)/`_o_o_yishugaochao`(747 B)/`_o_o_miaoshouhuichun`(766 B) 等 | 展示面（不迁移） |
| 其余规则技能 | 中位 2.5 KB（如 `zhengsu` 22 KB） | 待批次 |

## 上游语义与原生接线

| 技能 | 上游 | 原生 |
|---|---|---|
| 摸牌翻倍 `_xigua_mopaifanbei_` | `trigger {player:'phaseDrawBegin2'}` 锁定 → `trigger.num += 2` | `XiguaMopaiFanbei`（tag 531）：摸牌收束臂 `+2`（并入既有修正链） |
| 血量翻倍 `_xigua_xueliangfanbei_` | `trigger {global:'gameStart'}` 锁定 → `gainMaxHp(player.hp)` + `recover(player.hp)` | `XiguaXueliangFanbei`（tag 532）：**首次阶段开始**（一次性标记门控）→ `max_hp += hp` + `recover(hp)` |
| 配置门控 | `lib.config.mpfbmoshi` / `xueliangfanbei` | **技能实例启用**承载（宿主按包配置注入；登记差异） |

## 验证

```
cargo test --offline -p engine-native --test l20_config_mods_batch1
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（批 1） | **3 passed / 0 failed**（[targeted.log](targeted.log)）：摸牌翻倍——摸牌阶段摸 **4**（2+2）；血量翻倍——上限 +3、体力 3→6 且**仅一次**；对照（未持技：正常摸 2、上限不变） |
| engine-native 全量 | **1090 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **94/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 528（tag 531/532 追加） |

## 包与生成器

- 包：`packages/extensions/mark-supplement-upstream-liuli`（namespace `upstream-liuli`）+ `assets/config.json`（`mpfbmoshi`/`xueliangfanbei`/`daojianfu` 开关）。
- 生成器：`crates/content-runtime/examples/gen_mark_supplement_package.rs`。

## 登记差异

- **配置门控**（上游 `lib.config`）以技能实例启用承载：宿主需按包配置决定是否授予实例（未做自动装配）。
- 血量翻倍的触发点以**首次阶段开始**近似 `gameStart`（一次性标记门控）。
- 纯标记展示（`*_mark`）与 Spine 动画/音效技能（`_losehp_`/`_o_o_*`）不迁移。
- 其余 73 个技能待批次（中位 2.5 KB，含 22 KB 的 `zhengsu`）。
