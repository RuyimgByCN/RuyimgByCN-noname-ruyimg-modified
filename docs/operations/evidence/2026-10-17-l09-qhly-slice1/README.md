# L09 千幻聆音 切片 1：换肤/随机皮肤策略（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 来源：**L09 千幻聆音**（`extension/千幻聆音/extension.js`，718 KB；另 300 KB 资源脚本）
- 本批：装载器 `content-native-loader::qhly`（换肤决策 + 随机皮肤策略 + 配置）
- **规则/存档版本不变**（v321 / 存档 v105；无引擎改动）
- 台账：L09 由「待重构；未验收」→ **部分重构（切片 1/N）**

## G0 清点（本批结论）

| 面 | 上游内容 | 处置 |
|---|---|---|
| 换肤策略 | `_qhlyChangeSkin`（2.1 KB）：`changeHp` + `phaseEnd` 触发；`qhly_skinChange[角色][皮肤]` 规则（`source` 以 `hp_` 开头才参与；`phase` 仅阶段结束；`_qhly_gonnaChange` 门控） | `SkinChangeRule`（`source`/`phase_only`/`target`）+ `decide_skin(rule, trigger, state)`（**逐分支对齐**：回复不切换、掉血置位、阶段结束在「已掉血换过」时跳过并清标志） |
| 随机皮肤 | `_qhly_randskin`（5.5 KB）：候选含默认（`false`）；剔除被封禁皮肤与名称含「娘化」「云涯」者；`qhly_randskin_me` 时不作用于人类玩家 | `pick_random_skin(candidates, banned, exclude_terms, roll)`（同过滤口径 + 默认入池 + 回绕取模） |
| 自动换肤 | `_qhly_autoc`（678 B）：`qhly_autoChangeSkin` 开关（`close`/空 = 关） | `QhlyConfig.auto_change_skin: Option<String>` |
| 皮肤/语音**数据** | `_qhly_bgm`（482 KB 音频库）、`_qhlyCheckSkin`（96 KB 皮肤/立绘索引）、`_qhly_addButton`（53 KB UI） | **不迁移**（数据/渲染面，登记差异） |
| 其余 | `_qhcreateYH`（413 B UI）、`_qhlyChageKillingSkin`（586 B 死亡皮肤） | deferred（渲染面） |
| 插件面 | 详情页插件接口、主题、皮肤共享 | deferred（后续切片） |

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
| 定向（装载器） | **4 passed / 0 failed**（[loader-tests.log](loader-tests.log)）：配置解析（`close`→关闭、非 JSON/缺字段报错）；仅 `hp_` 规则参与；**掉血门控阶段结束**（掉血换→同回合阶段结束跳过并清标志→下一阶段结束切换；回复不切换）；随机皮肤过滤（封禁/「娘化」「云涯」剔除、默认入池、回绕） |
| engine-native 全量 | **1064 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **79/0/0**（[deps-tests.log](deps-tests.log)；较上批 +4） |
| 14 golden | 2/2 **零位移**——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |

## 登记差异

- 目标皮肤在上游由音频/皮肤索引检查（`qhly_checkPlayerImageAudio`）得出；原生在规则里**显式建模** `target`（索引数据不迁移）。
- 皮肤/语音数据（482 KB BGM 库 + 96 KB 皮肤索引）与详情页 UI、主题、插件系统未迁移。
- 随机性由调用方给定 `roll`（可复现；上游为 `list.randomGet()`）。
