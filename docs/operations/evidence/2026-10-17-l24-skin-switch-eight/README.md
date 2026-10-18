# L24 皮肤切换 八项整包验收（2026-10-17）

- 日期：2026-10-17
- 来源：**L24 皮肤切换**（`extension/皮肤切换/extension.js`，624 KB + 10 MB 资源）
- 结论：**八项整包验收通过**（`T12-SRC-L24` → resolved；整包计数 **23/37 → 24/37**）
- 证据：[切片 1](../2026-10-17-l24-skin-switch-slice1/README.md)（参数重应用 + 低血变身）、[切片 2](../2026-10-17-l24-skin-switch-slice2/README.md)（其余触发策略）

## 八项对照

| 项 | 结论 | 证据 |
|---|---|---|
| 功能 | 参数重应用（0/500/1000/2000 ms 退避 → 放弃；初始化过滤）+ 低血「仅新低」（主/副将独立）+ 受伤每头像位仅一次 + 换组势力归类 + 变身目标解析 + 五类 `TransformKind` | eight 用例「功能」三段 |
| 配置 | `parse_skin_switch_config`（8 项开关 + `savedPositions` 键值校验）+ 非 JSON/非法值报错；商店副本**逐字节一致** | eight 用例「配置」段 |
| 依赖 | manifest 依赖为空；动皮数据表/皮肤存在性以**注入式接口**提供（`special`/`has_skin`），不构成包依赖 | eight 用例「依赖」段 |
| 组合 | 与 swd 包同时装载建局 | eight 用例「组合」段 |
| 保存恢复 | 带包存档往返（体力/手牌保持）；商店重开内容锁哈希不变 | eight 用例「保存恢复」段 |
| 安装升级 | 真实 `import_extensions`（内容哈希一致）→ 2.0.0 覆盖 → `rollback` → 未安装版本 `NotFound` | eight 用例「安装升级」段 |
| 目标平台 | 包内无平台专有代码（不 `eval`/`fetch`/`fs`/`require`）；Spine/Worker 渲染与十周年/千幻互调不迁移 | 配置断言 + 登记差异 |
| 规则差异 | **不改规则**：同种子同操作序列下阶段推进与 4 座体力**逐位一致**；动皮只影响展示 | eight 用例「规则差异」段 |

## 验证

```
cargo test --offline -p engine-native --test l24_skin_switch_eight
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 八项用例 | **1 passed / 0 failed**（[eight.log](eight.log)） |
| engine-native 全量 | **1066 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **88/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |

## 包与生成器

- 包：`packages/extensions/skin-switch-upstream-liuli`（`manifest.json` + `package.npack.json` + `assets/config.json`；**无技能/卡牌/武将条目**=不改规则）。
- 生成器：`crates/content-runtime/examples/gen_skin_switch_package.rs`。

## 登记差异（整包验收后仍保留）

- **播放不迁移**：Spine/Worker 骨骼切换、音频、出框/攻击/背景动画、与十周年 UI/千幻聆音的互调（`updateDecadeDynamicSkin`/`qhly_changeDynamicSkin`）。
- **数据不迁移**：520 KB 动皮参数/资源表（`_pfqh_check_phaseEnd`）。
- 上游 `setTimeout` 实际延迟以**退避序列**建模（不建模定时器本身）。
- 动皮参数（位置/缩放/翻转）由宿主保存与注入（原生不落盘展示参数）。
