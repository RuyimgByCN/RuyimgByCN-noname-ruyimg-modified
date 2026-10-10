# U05 xianjian 第二十二切片（规则 v163，2026-10-09）

小蛮（pal_xiaoman，shu 3 女）灵泉（lingquan）+神雾（shenwu）双技闭环；
anwugu 与阿奴同技 mapping 复用（零新代码）。**小蛮觉醒链全通**（v162
水云计数依赖兑现）。前置：
[xianjian 第二十一切片](../2026-10-09-xianjian-zixuan-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js lingquan/shenwu）

- `lingquan`：player phaseEnd forced juexingji——filter `game.roundNumber
  >= 3 && player.getHistory("useCard").length > player.hp`；content
  draw(3) + addSkill("shuiyun") + createTrigger 立即触发一次 shuiyun。
- `shenwu`：global phaseEnd forced juexingji——filter storage.
  shuiyun_count >= 3；content gainMaxHp() + recover() + addSkill
  ("huimeng")。

## 改动（分开陈述）

- **状态三字段（PlayerState）**：`total_used: u32`（全局用牌计数——
  commit_card_used 未取消使用递增=getHistory("useCard") 同口径）、
  `lingquan_awakened: bool`、`shenwu_awakened: bool`（juexingji 一次
  性双职责：觉醒门控+持有标记）。存档 v28→**v29**。
- **宿主行为**：`XianjianLingquan`（tag 179，PhaseEnd forced：Act 觉醒
  三连——置 flag+摸 3+**runtime 授水云实例**（skills.insert，上游
  addSkill 同口径；shuiyun def 经紫萱 mapping 已在目录）+内联水云置牌
  询问（类型去重可放弃=上游 createTrigger 立即触发同效）→SelectCard
  收应答置离场区）+ `XianjianShenwu`（tag 180，global PhaseEnd forced：
  置 flag+体力上限 +1+回复 1+runtime 授回梦实例（huimeng def 经赵灵儿
  mapping 已在目录））——六表+ALL 178→180。
- **runtime 授技模式**（首例落地，石会粉禁牌同源）：跨将授技不经
  cards.json——目标技 def 经任意持有者 mapping 进目录后，觉醒时
  `skills.insert((owner, def), SkillInstance)` 即刻持有。
- **cards.json**：小蛮行（shu 3 女；anwugu 复用 host:xj_anwugu）+
  rehash 两步；**版本** 162→163；a0 追加 179/180。

## 验收口径

- **回归已加入**：g1_wudian 2 例——`xiaoman_lingquan_awaken_grants_
  shuiyun`（r1 觉醒门槛未到（round<3）/r2 不到/r3 觉醒：置 flag+摸
  3+内置置牌询问→离场区+runtime 水云实例断言）+
  `xiaoman_shenwu_awaken_grants_huimeng`（计数注入 3（save/restore，
  真实来源=水云·救）→任意阶段末觉醒：上限+1/回复 1/回梦实例/一次
  性）。
- **全量通过**：engine-native **492 passed / 0 failed / 1 ignored**
  （v162 基线 490 +2）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 162→163 + 三字段入 canonical_hash——14 场景全
  mismatch，重录后 replay_scenarios 2/2。
- **导航坑（登记）**：Judge 起**收束=5 次 NP**（第 4 次停在 End——v160
  备注的「5 次」再次确认；写 4 次则后续 submit 全 NotYourTurn 静默失
  败、round 永不推进）。

## 剩余

- xianjian 其余 14/36 将；猎空/雪见（离场区已通）、chengxin（救援通道
  已通）可接入；云天河 zhuyue mod（杀白名单已做）、changnian 死亡拦
  截、响应链、createCard、Recover 前置窗口等缺口族待排；长期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（14 文件，相对 noname-next
根）。
