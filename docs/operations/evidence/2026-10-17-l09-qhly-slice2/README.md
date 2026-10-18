# L09 千幻聆音 切片 2：皮肤/语音**索引面**（真名 / 国战回退 / 封禁 / 列表过滤）（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 来源：**L09 千幻聆音**（`extension/千幻聆音/extension.js`）
- 本批：装载器索引面 API（`real_name`/`resolve_gz_fallback`/`erase_ext`/`is_skin_banned`/`filter_skin_list`）
- **规则/存档版本不变**（v321 / 存档 v105；无引擎改动）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 真名解析 | `qhly_getRealName`：`lib.qhly_skinShare[name].name` 存在则取真名 | `real_name(name, share)`（空别名不生效——上游 `&& .name` 真值判断同口径） |
| 国战回退 | `qhly_getSkin`/`qhly_getSkinList` 的 `gz_` 兼容：未启用国战 / 非国战模式 / 无国战皮肤 → 去前缀后若为已知角色则回退 | `resolve_gz_fallback(name, guozhan_enabled, is_guozhan_mode, has_guozhan_skin, is_known_character)` |
| 去扩展名 | `qhly_earse_ext`：最后一个 `.` 之前 | `erase_ext(path)` |
| 封禁皮肤 | `qhly_skinIsBanned`：名单含 `名字-皮肤`；空皮肤 = `[original]`；未配置名单 → 不封禁 | `is_skin_banned(name, skin, ban_list)` |
| 列表过滤 | `qhly_getSkinList` 过滤链：`hideSkin` 剔除 + `locked==false` 时剔除锁定皮肤 | `filter_skin_list(list, hidden, skin_lock, locked)` |

## 验证

```
cargo test --offline -p content-native-loader --test qhly_loader
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（装载器） | **7 passed / 0 failed**（[loader-tests.log](loader-tests.log)；切片 1 的 4 项 + 本批 3 项）：真名解析（别名/空别名/无别名）与 `gz_` 回退四态；去扩展名四例；封禁（`[original]` 默认、名单未配置不封禁）；列表过滤两态（默认只剔隐藏 / 非锁定态再剔锁定） |
| engine-native 全量 | **1064 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **82/0/0**（[deps-tests.log](deps-tests.log)；较上批 +3） |
| 14 golden | 2/2 **零位移**——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |

## 登记差异

- **数据不迁移**：482 KB BGM 库、96 KB 皮肤/立绘索引、53 KB UI 按钮逻辑仍不迁移；本批只迁移**索引解析与过滤策略**（数据由宿主注入）。
- `qhly_getSkinList` 的完整链路（皮肤包查找 `qhly_foundPackage`、`loadInfoJs` 异步加载、锁定判定来源）以**注入式**接口建模（`hidden`/`skin_lock` 回调），未内置具体数据源。
- 详情页插件系统、主题、皮肤共享的**写侧**（`qhly_setCurrentSkin` 的资源路径重定向）未迁移。
