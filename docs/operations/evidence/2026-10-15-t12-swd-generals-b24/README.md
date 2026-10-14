# T12 swd 武将面批 24：双将整将（断月/屠阵=水镜、落梅/星点=兰茵）（2026-10-15）

## 承载

四技 HostBehavior（tag 280/281/282/283，键数冻结表 279→**283**）：

- **断月 duanyue**（水镜）：出牌阶段限一次主动技——三段机
  SelectCard（手/装备区装备牌）→ TargetChoice（其他存活）→ Act（弃牌+
  `damage_data_with` 1 伤）。取消任一段=无效果（牌留区到末段）；
  usable:1 走 once_per_turn 臂。
- **屠阵 tuzhen**（水镜）：自己造成伤害结算后（可选确认门）弃置受伤者
  手牌中的**全部非基本牌**（确定性整组；上游 randomDiscard 无——本身即
  整组弃置无差异）。第 9 触点提取臂（target=受伤者）首版漏挂由回归抓漏
  （批 19 破星同病第三次）。
- **落梅 luomei**（兰茵）：使用或打出梅花牌→频率技摸 1（CardUsed +
  extra_events CardResponded 双事件，飞虚先例 v149）。
- **星点 xingdian**（兰茵）：出牌阶段限一次主动技——弃 1 张手牌，确定性
  弃两名其他存活角色各手/装备区**首张**。**登记差异**：上游随机 2 名敌人
  （isEnemyOf 阵营面）各随机弃 1 → 原生座位序前 2 名有牌者各弃首张
  （yinbo 先例同口径）。

武将名核验（translate.js）：水镜 swd_shuijing（mojian/duanyue/tuzhen
3/3）、兰茵 swd_lanyin（xingdian/yulin/luomei 3/3）——cards.json 无新增
武将，仅两将 mapping 补全；npack 双哈希重算。

## 过程勘误（接线/测试侧五处）

1. **屠阵提取臂漏挂**（第 9 触点三犯+1）：target None 静默 Done——回归
   「非基本牌被弃」断言抓漏。
2. **星点 Act 臂未消费选牌应答**（批 22 同病）：单答直入 Act 时须自取
   `ctx.answer()`。
3. 落梅测试段 p1 已被打至 1 血——杀伤再中即濒死，死亡噪声淹没摸牌断言
   （两段「净持平/净-1」都可能假绿/假红）——前置回满血隔离变量。
4. 断月取消段会消耗 once_used（取消在机内，发动即记账）——重排为
   取消→快照重置 once_used→正例→限一次反例。
5. Gate 1b 拦未用变量 zg0（改 `_zg0`）。

## 测试（swd_generals_b24_duanyue_tuzhen_luomei_xingdian）

| 技 | 正例 | 反例 |
|---|---|---|
| 断月 | 弃 zhuge+对 p1 造成 1 伤 | 取消无效果牌留区；本回合限一次 |
| 屠阵 | 杀伤后确认→非基本（duel）全弃、基本（桃）留 | 受伤者无非基本→无窗；来源非持有者→不触发 |
| 落梅 | 用梅花杀摸 1（净持平+p1 存活佐证） | 红杀不摸（净 -1） |
| 星点 | 弃 1 后 p1/p2 各弃首张、p3 无牌不受影响 | — |

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 g1_shahai_swd | **35/35** |
| engine-native 全量 | **754/0/0**（golden 不变）——[日志](engine-native-tests.log) |
| Gate 1b | 首跑拦未用变量（已修）→ 复跑 exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- swd 武将面：**21/79 名、21/241 技转正；整将十三→十五**（水镜 3/3、
  兰茵 3/3 新晋）。
- 仍部分转正：拓跋玉儿 1/3（poyue/poyue2 需 ShaResolve 内联直挂+存档触
  点、niepan 待查来源归属）、屈娴 1/2（huanxia=viewAs+group 三子技）、
  壶中仙 1/3（协律可承接、壶境待勘察）、芝茵 1/3（朱羽伤害阶梯+雪煌
  limited）、黄雷 1/3（极略牌堆顶 viewAs+改造 niche）。
- 批 25 候选：协律（壶中仙）+ 屈娴/芝茵余技勘察。
