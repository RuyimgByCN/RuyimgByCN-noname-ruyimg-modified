# U05 swd 第七批：归有界（规则 v91，2026-10-08）

swd 子包第七批转换：归有界（♠3/♣9 两张，延时锦囊，他人）——判定为
黑色则失去 1 点体力并随机弃一张牌。前置：
[swd 第六批](../2026-10-08-u05-swd-batch6/README.md)。

## 上游语义核验（源码为准，card/swd/card.js）

`guiyoujie`：`type: "delay"`、`filterTarget: judge 可用 && player !==
target`（只能对他人）、judge 黑色返回 -3（失败）；effect——`loseHp()`
+ `randomDiscard()`。

## 改动（分开陈述）

- **代码已实现**：`CardKind::Guiyoujie`（编码 110：archive enc/dec、a0
  variant/ALL 同步）+ PlaceDelayed 派发（无懈窗口包装，Le/Bing 同款）+
  判定结算臂——黑色：`LoseHp` 1 + he（手牌+装备）RNG 均匀抽取随机弃
  一张（装备走 revoke）；红色无事。PlayCard 门控=恰 1 其他存活目标+
  其判定区无同名延时。
- **首跑缺陷**：PlayCard 穷尽臂误吞 Guiyoujie（装备臂 `| Guiyoujie`
  误加）→ IllegalTargets，测试捕获后拆出独立验证臂。
- **转换包**：cards.json 增 ♠3/♣9 两张 + 哈希重算（现 14 张）。

## 验收口径

- **回归已加入**：`swd_guiyoujie_delayed_black_loss`——置黑桃判定牌 →
  黑判失去 1 点体力（6→5）+ 随机弃牌执行；存档往返。
- **全量通过**：engine-native **406 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**，DEPS_EXIT=0。
- **golden**：版本 90→91 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（标准场景无归有界，纯版本位移）。
- **剩余**：swd 其余 9+ 卡种逐批转换（潜偷面具/羊皮卷/贴房绅视/
  吸血系外的装备/青龙令珠/星君眼/伏符录/风雪刃/兰阁怪衣/岱火风轮）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
