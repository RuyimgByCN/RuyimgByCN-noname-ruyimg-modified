# P2/N08 逐技能语义对照批四：随机类逐项对照（规则 v78，2026-10-08）

主稿 P2/N08「逐技能语义对照」第四批：随机类登记差异逐项对照上游源码，
**六项真消除 + 两处登记更正 + 七项保留登记**。前置：
[逐技对照批三](../2026-10-08-n08-skill-parity-3/README.md)。

## 勘察结论（改变处置方式）

引擎已有种子 RNG（`crates/engine-native/src/rng.rs` `RuleRng::gen_range`
Lemire 无偏采样；state+calls 入存档，开局洗牌既有消费方）——上游
`Math.random()` 类均匀随机可映射为原生种子均匀随机：**语义等价
（同为均匀分布）、确定性可复放（种子决定）**，因此随机类以「消除」
而非「登记近似」收尾。

## 逐项处置（上游源码为准，杀海拾遗包）

**消除六项**（`ctx.core.rng.gen_range` 均匀抽取）：

| 项 | 上游语义 |
|---|---|
| 瘟疫（有手牌分支） | `target.randomDiscard("h")` → 随机手牌 |
| 乾坤镖 | 每受害者 `he.randomGet()` → 手牌+装备合集随机一张（he 空者跳过=上游 changeTarget 同口径） |
| 麻婆豆腐 | `list.randomGet()` 随机敌 + `he.randomGet()` → 随机有牌其他角色随机一张（敌我阵营面按原型惯例，同疾先例） |
| 山崩术 | 敌装备合集 `randomGets(2)` 恰二才弃 → 合集均匀无序对（i+j 越过法），恰二门控保留 |
| 破晓分支二 | 随机一张铜卡法术（破晓除外）→ 牌堆铜法术集合均匀抽取，空则摸 1 不变 |
| 玉衡 | 持黑桃敌 `list.randomGet()`+`randomGet()` → 随机有黑桃手牌其他角色随机黑桃（升级机制仍登记暂缓） |

**登记更正两处**（原文误归随机类，源码实为自选类）：暴雪药水
`chooseToDiscard("h",2,true)`=目标**自选**两张；冰魄通=双方**自选**
展示一张手牌比对。

**保留登记七项**：弹射之刃（敌-友交替阵营链未实现）、六骰格（随机
效果池未实现面）、舌战群儒（多轮问答链捆绑）、草船借箭/金莲烛/
卤玉羹（随机捆绑自选/发现机制）、龙须钩（目标自选装备）。

## 验收口径

- **定向通过**：`g1_shahai_gujian` 13/0、`g1_shahai_gwent` 6/0。
- **回归适配**：破晓测试改集合成员断言（随机后不再唯一可指）；
  乾坤镖补 he 空守卫（`gen_range(0)` 反例由 `jiguan_batch_four_kinds`
  捕获后修复）。
- **全量通过**：engine-native **382 passed / 0 failed / 1 ignored**，
  单线程 `cargo test --offline -p engine-native -- --test-threads=1`；
  依赖包 **62 passed / 0 failed**，DEPS_EXIT=0。明细见
  [test-logs.txt](test-logs.txt)。
- **golden**：版本 77→78 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（无场景触及六卡，纯版本位移）。
- **上游等价边界**：随机分布等价（同为均匀），非逐次结果等价（种子
  不同则具体抽取不同——随机流口径见覆盖清单）；保留登记七项不构成
  G1 整包等价。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（6 文件，相对 noname-next 根）。
