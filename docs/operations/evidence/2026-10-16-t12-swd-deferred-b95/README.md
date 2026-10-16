# T12 swd deferred 消解批 95：狩猎/不死

- 日期：2026-10-16
- 范围：两技 deferred 消解（tag 400-401）——**124/184 技条目已映射**（deferred 60→58）；存档 v77→v78（Sha 快照尾部追加）
- 上游回读：`archive/qita/sanguosha/noname/extension/杀海拾遗/character/swd/{skill.js,translate.js}`（shoulie/busi 逐段核验）

## 两技语义与接线

| 技（将） | 上游语义 | 接线 |
|---|---|---|
| 狩猎 shoulie（子达书） | 杀指定目标后可弃一张手牌令此杀不可闪避 | SwdShoulie(tag 401)=marker：Sha 机器 !asked 门内联询问（shoulie_discard 窗）→应答消费弃牌置 `shoulie_direct` 旗→与破月同点跳过防具判定+闪窗口（整杀有效=机器实例生命周期） |
| 不死 busi（康那里士） | 濒死时判定：黑桃照常死亡；非黑桃体力回至 1 并翻面（unique 一次） | SwdBusi(tag 400)=Dying forced priority 7：内联判定→非黑桃 recover 至 1+TurnOverBefore 帧（经 commit 含凶兽守卫/急冻解除）→移除实例 |

## 字段批

- ShaResolveMachine + `shoulie_direct: bool`（快照/archive enc/dec 尾部追加 v78/a0 夹具；**不入 canonical hash——golden 零扰动**）。
- 哨兵复用：`hujia_holder=Some(source)` 标记狩猎待应答（消费点在 asked 门外首个到达点；消费后 `asked=false + Yield` 重入门——防应答被闪窗误读，青龙再杀同款）。

## 勘误（入记忆）

- **asked 门内询问的应答消费须防串读**：在 `!asked` 门内开询问并 return 后，恢复时直接落入「已恢复」段把应答误读为闪应答（狩猎首版实证：弃牌被当作闪→杀被「闪避」）——消费后必须 `asked=false; Yield` 重入门。
- hujia_holder 哨兵复用与真护驾窗口的窄冲突（source 同时持 shoulie 且为目标护驾者）登记可接受。

## 验证

- **定向回归** `swd_deferred_b95_shoulie_busi`：狩猎弃牌→杀直击（p1 有闪不可避+闪未消耗）；放弃→杀照常被闪避反例；不死非黑桃判定→存活+hp 回 1+翻面+实例移除。
- **全量**：engine-native 812 passed / 0 failed。
- **Gate 1b**：exit 0（engine push_machine pattern 未用字段 `shoulie_direct: _` 拦截后修）。

## 结论

- **124/184 映射，deferred 60→58**。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
