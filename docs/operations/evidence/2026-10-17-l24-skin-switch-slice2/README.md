# L24 皮肤切换 切片 2：其余动皮触发策略（受伤 / 击杀 / 换组 / 阶段）（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 来源：**L24 皮肤切换**（`extension/皮肤切换/extension.js`）
- 本批：装载器新增 `TransformKind` / `TransformFlags` / `damage_transform_allowed` / `group_nature` / `group_changed` / `resolve_transform`
- **规则/存档版本不变**（v321 / 存档 v105；无引擎改动）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 受伤变身 | `_pfqh_check_damage`（5.7 KB）：`player['damagePrimaryTransform'/'damageDeputyTransform']` 已置位则**取消变身**（每头像位仅一次） | `TransformFlags` + `damage_transform_allowed(flags, is_primary)`（首次允许并置位；二次返回 false） |
| 换组变身 | `_pfqh_check_changeGroup`（6.2 KB）：`groupMap`（wei/shu/wu/qun 前缀）→ 归类；当前势力归类与特化配置不同则触发 | `group_nature(group)`（前缀归类，未知 → `None`）+ `group_changed(special_group, current_group)`（同归类不算换组；未知归类不触发） |
| 变身目标 | `_pfqh_check_jisha`（5.3 KB）/`_pfqh_check_phaseBegin`（4 KB）：`effs.transform` 须在 `special` 表中；`trans.name` 形如 `key/skinName`，两者须在动态皮肤表中存在 | `resolve_transform(transform_key, special, has_skin)` → `Option<(骨骼, 皮肤)>`（空段/无分隔符/皮肤缺失 → `None`） |
| 触发类别 | `skinSwitch.dynamic.getSpecial(player, 类别)`：`lowhp`/`damage`/`jisha`/`changeGroup`/`phaseBegin` | `TransformKind` 枚举（五类，含切片 1 的 `LowHp`） |

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
| 定向（装载器） | **6 passed / 0 failed**（[loader-tests.log](loader-tests.log)；切片 1 的 3 项 + 本批 3 项）：受伤变身每头像位仅一次（主副独立）；势力归类（前缀/未知）与换组判定（同归类不算、未知不触发）；变身目标解析（正常/空皮肤/无分隔符/键缺失/皮肤表缺失/无配置六态） |
| engine-native 全量 | **1065 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **88/0/0**（[deps-tests.log](deps-tests.log)；较上批 +3） |
| 14 golden | 2/2 **零位移**——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |

## 登记差异

- 五类触发策略的**决策**已迁移；**播放**（Spine/Worker 骨骼切换、音频、出框/攻击/背景动画）不迁移。
- 520 KB 动皮参数/资源表（`_pfqh_check_phaseEnd`）仍不迁移；`special` 表与皮肤存在性以注入式接口提供。
- 与十周年 UI / 千幻聆音的互调（`updateDecadeDynamicSkin`、`qhly_changeDynamicSkin`）不迁移。
