# L09 千幻聆音 八项整包验收（2026-10-17）

- 日期：2026-10-17
- 来源：**L09 千幻聆音**（`extension/千幻聆音/extension.js`，718 KB + 300 KB 资源脚本）
- 结论：**八项整包验收通过**（`T12-SRC-L09` → resolved；整包计数 **22/37 → 23/37**）
- 证据：[切片 1](../2026-10-17-l09-qhly-slice1/README.md)（换肤/随机皮肤策略）、[切片 2](../2026-10-17-l09-qhly-slice2/README.md)（索引面）

## 八项对照

| 项 | 结论 | 证据 |
|---|---|---|
| 功能 | 换肤决策（`decide_skin`：仅 `hp_` 规则参与、掉血门控阶段结束、回复不切换）+ 随机皮肤过滤（封禁/「娘化」「云涯」/默认入池）+ 索引面（真名/`gz_` 回退/去扩展名/封禁/列表过滤） | eight 用例「功能」两段 |
| 配置 | `parse_qhly_config`（`randskin`/`randskin_me`/`autoChangeSkin`，`close`→关闭）+ 非 JSON/缺字段报错；商店副本**逐字节一致** | eight 用例「配置」段 |
| 依赖 | manifest 依赖为空；皮肤/语音**数据源**以注入式接口建模（`hidden`/`skin_lock` 回调、`share`/`ban_list` 参数），不构成包依赖 | eight 用例「依赖」段 |
| 组合 | 与 swd 包同时装载建局（4 座） | eight 用例「组合」段 |
| 保存恢复 | 带包存档往返（体力/手牌保持）；商店重开内容锁哈希不变 | eight 用例「保存恢复」段 |
| 安装升级 | 真实 `import_extensions`（内容哈希一致）→ 2.0.0 覆盖 → `rollback` → 未安装版本 `NotFound` | eight 用例「安装升级」段 |
| 目标平台 | 包内无平台专有代码（不 `eval`/`fetch`/`fs`/`require`）；皮肤/语音数据与详情页 UI 不迁移 | eight 用例「依赖/目标平台」段 |
| 规则差异 | **不改规则**：同种子同操作序列下阶段推进与 4 座体力**逐位一致**；皮肤选择属展示面 | eight 用例「规则差异」段 |

## 验证

```
cargo test --offline -p engine-native --test l09_qhly_eight
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 八项用例 | **1 passed / 0 failed**（[eight.log](eight.log)） |
| engine-native 全量 | **1065 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **82/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |

## 包与生成器

- 包：`packages/extensions/qhly-upstream-liuli`（`manifest.json` + `package.npack.json` + `assets/config.json`；**无技能/卡牌/武将条目**=不改规则）。
- 生成器：`crates/content-runtime/examples/gen_qhly_package.rs`。

## 登记差异（整包验收后仍保留）

- **数据不迁移**：482 KB BGM 库、96 KB 皮肤/立绘索引、53 KB UI 按钮逻辑；详情页插件系统、主题、皮肤共享写侧（资源路径重定向）未迁移。
- 目标皮肤在上游由音频/皮肤索引检查得出；原生在规则里显式建模 `target`。
- 随机性由调用方给定 `roll`（可复现；上游 `list.randomGet()`）。
- `qhly_getSkinList` 的皮肤包查找与异步加载以注入式回调建模。
