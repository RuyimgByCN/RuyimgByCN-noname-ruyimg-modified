# L24 皮肤切换 切片 1：参数重应用 + 低血变身策略（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 来源：**L24 皮肤切换**（`extension/皮肤切换/extension.js`，624 KB；含 10 MB 资源脚本）
- 本批：装载器 `content-native-loader::skin_switch`（参数重应用退避 + 低血变身策略 + 配置）
- **规则/存档版本不变**（v321 / 存档 v105；无引擎改动）
- 台账：L24 由「待重构；未验收」→ **部分重构（切片 1/N）**

## G0 清点（本批结论）

| 面 | 上游内容 | 处置 |
|---|---|---|
| 参数重应用 | `_pfqh_reapply_saved_params`（2.7 KB）：开局重应用已保存动皮参数，依赖未就绪时 0/500/1000/2000 ms 退避重试；`_pfqh_reapply_on_init`（1.3 KB）：角色初始化路径，过滤 `player.dynamic && player.dynamic.primary` | `reapply_decision(deps_ready, attempt)`（Apply / RetryAfter(500/1000/2000) / GiveUp）+ `reapply_on_init_allowed(has_dynamic, is_primary)` + `REAPPLY_BACKOFF_MS` |
| 低血变身 | `_pfqh_check_hp_change`（9 KB）：回复且未开 `recover` → 跳过；主/副将各自记录历史最低体力（初值 hp+1000）；仅**新低**触发并更新 | `LowHpState`（主/副将独立记录）+ `lowhp_decision(state, is_primary, hp, delta, recover_allowed)` |
| 其它触发策略 | `_pfqh_check_damage`（5.7 KB）、`_pfqh_check_damage_times`（7 KB）、`_pfqh_check_jisha`（5.3 KB）、`_pfqh_check_changeGroup`（6.2 KB）、`_pfqh_check_phaseBegin`（4 KB） | deferred（后续切片；同类策略可复用本批建模方式） |
| 数据 | `_pfqh_check_phaseEnd`（**520 KB**：动皮参数/资源表） | **不迁移**（数据面，登记差异） |
| 渲染/互调 | Spine/Worker、出框/攻击/背景动画、十周年 UI 与千幻聆音互调（`updateDecadeDynamicSkin`） | **不迁移**（渲染面，登记差异） |
| 配置 | `dynamicSkin`/`useDynamic`/`attackEffect`/`isAttackFlipX`/`hideHuanFu`/`czgEnable`/`cugDynamicBg`/`showTopArc`/`savedPositions` | `SkinSwitchConfig`（含 `savedPositions` 键值校验） |

## 验证

```
cargo test --offline -p content-native-loader --test skin_switch_loader
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（装载器） | **3 passed / 0 failed**（[loader-tests.log](loader-tests.log)）：配置解析（`savedPositions` 值非法报错、缺字段报错）；重应用退避（就绪→Apply；未就绪→500/1000/2000→GiveUp；初始化过滤三态）；低血变身（首次受伤即新低、同值不触发、更低触发、回复未开 recover 不触发且不改记录、回复开 recover 走新低判定、主副将记录互不影响） |
| engine-native 全量 | **1065 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **85/0/0**（[deps-tests.log](deps-tests.log)；较上批 +3） |
| 14 golden | 2/2 **零位移**——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |

## 登记差异

- 重应用依赖（`window.skinSwitch.updateDecadeDynamicSkin` + `window.decadeUI`）以布尔注入建模；上游的 `setTimeout` 实际延迟不建模（只保留退避序列）。
- 520 KB 动皮参数/资源表不迁移；动皮触发策略只迁移「低血」一类，其余（受伤次数/击杀/换组/阶段）deferred。
- Spine/Worker 渲染、出框/攻击/背景动画、与十周年 UI/千幻聆音的互调未迁移。
