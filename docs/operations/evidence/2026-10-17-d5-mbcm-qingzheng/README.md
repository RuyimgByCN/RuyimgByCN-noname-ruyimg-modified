# D5 龙血玄黄后继批 119：清正（`mbcmqingzheng`，道心值 25 阈值解锁）（2026-10-17）

- 日期：2026-10-17
- 范围：道心值 25 阈值解锁的手杀包技能 **清正**（liuli `character/mobile.js:17117`）转换；**无状态字段变化**（不递增规则/存档版本，14 golden 零位移）
- 上游回读：`archive/upstream/noname-liuli/noname/character/mobile.js`（`mbcmqingzheng` 17117–17250）

## 上游规则面与原生接线

| 步 | 上游 | 原生 |
|---|---|---|
| 触发 | `phaseUseBegin` + `direct:true`，`filter: countCards('h') > 0` | `MbcmQingzheng`（tag 474）：`PhaseBegin(Play)` 可选触发 + 手牌非空过滤 |
| ① 选花色 | `chooseButton` 花色（该花色手牌 >0 且**全部可弃置**） | `PickTag` 花色选择（0 黑桃/1 红桃/2 梅花/3 方块，与反间同口径）；**登记差异**：原生手牌默认可弃置，不另做 `cardDiscardable` 过滤 |
| ② 选目标 | `chooseTarget` 其他有手牌者（可取消） | `SelectPlayers` 其他手牌非空存活者，min1/max1，`allow_cancel` |
| ③ 选目标花色 | `chooseControl(list)`：目标手牌中出现过的花色（观看其手牌） | `PickTag`：目标手牌花色集合；**登记差异**：观看手牌为宿主 UI 面，引擎只提供花色集合 |
| ④ 双向弃牌 | `player.discard(cards)`；`target.discard(cards2,'notBySelf').set('discarder',player)` | 逐张 `discard_from_hand`（自己→目标）；**登记差异**：`CardsDiscarded` 事件无 `discarder` 字段（弃置归属未建模） |
| ⑤ 惩罚 | `if (event.cards2.length < cards.length) target.damage()` | 目标弃牌数 < 自己 → 1 点**无来源**普通伤害（上游 `damage()` 未给来源；登记差异同口径） |

## 包与目录

- `packages/extensions/longxue-upstream-liuli` 包级 `skills` 段：`mbcmqingzheng` 由 `mapping: null`（外部依赖）改为 `mapping: "host:mbcmqingzheng"`；其余三技（`mbcmjiushi`/`mbcmfangzhu`/`mbjuejin`）保持外部依赖登记。npack 已 rehash。
- 授予路径不变：道心值 ≥25 时 `mbqianlong_refresh` 按目录存在性授予 → 清正现可被真正解锁（[批 117 证据](../2026-10-17-d5-longxue-mbqianlong/README.md)）。
- 新增测试工具 `debug_place_suit(player, suit)`（花色类技能布阵）。

## 验证

```
cargo test --offline -p engine-native --test d5_longxue_mbqianlong
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（龙血玄黄族） | **7 passed / 0 failed**（[targeted.log](targeted.log)）：原有 5 测（开局面/累加/阈值/卫统/包装载）+ **清正**两测：少弃惩罚（0 号弃 2 黑桃 → 1 号弃 1 梅花 → 受 1 点伤害）与对照（1 号弃 2 张 ≥ 1 张 → 无伤害） |
| engine-native 全量 | **920 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | `replay_scenarios` 2/2 **零位移**（无字段/版本变化，未重录） |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | `RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets` **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 471（tag 474 追加） |

## 剩余

- **酒诗 `mbcmjiushi`**：需 ①「未翻面时翻面并视为使用酒」（翻面授予原语 + 虚拟酒使用，含濒死窗）②「翻面状态下受伤后可选翻回正面」③`turnOverAfter`（翻面后）事件——原生目录无「翻面后」触发种类（触发规划器对该帧 after 窗口静默），需新增事件种类；④`get.cardPile2`（牌堆/弃牌堆检索锦囊）近似。
- **放逐 `mbcmfangzhu`**：需 `skillBlocker`（技能禁用/使用限制：仅可用手牌锦囊直到回合结束）原语。
- **绝进 `mbjuejin`**：需「移出游戏」区（`game.log(cards,'被移出了游戏')`）与标记 `mbcmxscw`。
- **D5**：`_CMeffects` 纯显示面（登记差异）；上述三技与 `mb_epic_caomao` 归外部手杀包来源批。
