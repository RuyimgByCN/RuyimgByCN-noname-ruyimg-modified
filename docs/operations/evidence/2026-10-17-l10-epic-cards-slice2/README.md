# L10 史诗卡牌 切片 2：让节（`rangjie`）——受伤后获得指定类型牌 / 移动场上牌（v321/存档 v105 不变，2026-10-17）

- 日期：2026-10-17
- 来源：**L10 史诗卡牌**（`extension/史诗卡牌/extension.js`）
- 本批：技能 `rangjie`（让节）落地；包内 `deferred` 相应收缩（余十常侍面 + 展示面）
- **规则/存档版本不变**（v321 / 存档 v105；无字段变化）

## 上游语义与原生接线

| 面 | 上游 | 原生 |
|---|---|---|
| 触发 | `trigger {player:'damageEnd'}`，`direct:true`（可选）；按**伤害点数**逐次（`event.count = trigger.num`） | `Rangjie`（tag 516）：`Damage` 触发（自己为承受者、未防止）+ 可选确认门；机器内以 `self.cards` 队列承载剩余次数 |
| 选择 | `chooseControl(['获得牌','移动牌'（`canMoveCard` 时）,'取消'])` | `PickTag`（0=获得牌 / 1=移动牌（存在可见区牌时）/ 2=取消） |
| 获得牌 | `player.draw()` + `chooseButton(['basic','trick','equip'])` → `get.cardPile2(type)` 取一张 | 摸 1 + `PickTag`（基本/锦囊/装备）→ **牌堆实体发现**首张该类型牌入手（登记差异：上游为搜索面） |
| 移动牌 | `player.moveCard(true)`（把场上可见区一张牌移到一名角色处） | `SelectCards`（全场装备区+判定区牌）→ `SelectPlayers`（存活角色）→ 移牌：装备且目标栏位空 → 入目标装备区（授予装备技）；否则入目标手牌（登记差异：非 `moveCard` 全语义） |
| 重复 | `if(event.count>0) goto(1)` | 队列非空则回 `Act` 再开选择窗 ✓ |

包数据：`epic-cards-upstream-liuli` 的 `skills` 增加 `rangjie → host:rangjie`（生成器同步更新）；npack 已重生成。

## 计数

- 包内 deferred：`rangjie` 移出 → 余 `shichangshi`（十常侍面）与展示面。
- L10 台账：仍「部分重构（切片 2/N）」；八项验收未完成。

## 验证

```
cargo test --offline -p engine-native --test epic_rangjie_after_damage
cargo test --offline -p engine-native --no-fail-fast
cargo test --offline -p content-runtime -p engine-ai -p content-native-loader --no-fail-fast
cargo test --offline -p engine-native --test replay_scenarios
node apps/web-client/scripts/extension-store-smoke.mjs
RUSTFLAGS="-D warnings" cargo check -p engine-native --all-targets   # 独立 CARGO_TARGET_DIR
```

| 项 | 结果 |
|---|---|
| 定向（让节） | **3 passed / 0 failed**（[targeted.log](targeted.log)）：受伤 1 点 → 确认 → 获得牌 → 选「基本」→ 摸 1 + 得 1 张基本牌；移动牌分支（2 号武器 → 3 号空栏位装备区，含装备技迁移）；对照（放弃无所得、未持技不开窗） |
| engine-native 全量 | **1059 passed / 0 failed / 1 ignored**（[engine-native-tests.log](engine-native-tests.log)） |
| 装载三包 | **66/0/0**（[deps-tests.log](deps-tests.log)） |
| 14 golden | 2/2 **零位移**（无版本变化）——[replay-golden-v321.log](replay-golden-v321.log) |
| 客户端扩展冒烟 | **8/8**（[client-store-smoke.log](client-store-smoke.log)） |
| Gate 1b | **exit 0**（[gate1b-all-targets.log](gate1b-all-targets.log)） |
| 键表守护 | `HostBehavior` 512（tag 516 追加） |

## 登记差异

- 「获得牌」的牌堆取牌为**首张匹配**（上游 `get.cardPile2` 搜索面/随机性未建模）。
- 「移动牌」为简化移牌（可见区 → 目标：装备且栏位空则入装备区、否则入手牌）；上游 `moveCard` 的完整区域/替换交互未建模。
- 上游 `ai`（`maixie`/`effect` 等估值）未迁移。
