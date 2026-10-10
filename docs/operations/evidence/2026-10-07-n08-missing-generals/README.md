# P2/N08 标准缺失将子切片：乐进/潘凤/华雄（3/8，2026-10-07）

主稿 P2/N08 行「标准 8 个缺失来源将」的第一子切片：单技能三将先行
（`STANDARD_GENERALS` 追加 + 技能实现 + 装配/行为测试）。上游对照
`archive/upstream/noname/apps/core/character/standard/{character,skill}.js`
（其余 5 将 9 技的上游体已全部定位，见下表）。

## 本批落地（上游条目 → 原生实现）

| 将（势力/体力） | 技 | 机制 | 原生实现 |
|---|---|---|---|
| 乐进（魏/4） | 骁果 stdxiaoguo | 其他角色结束阶段开始时，弃一张基本手牌 → 对其造成 1 伤 | PhaseBegin(End begin) 可选门 + 手牌基本牌自选（SelectCards）+ 伤害子事件；`MachineSnapshot::XiaoGuo`（tag 45） |
| 潘凤（群/4） | 狂斧 stdkuangfu | 出牌阶段【杀】造成伤害后：目标体力小于你 → 摸 2，否则失去 1 体力 | Damage 来源触发锁定 + once_per_turn；体力比较为**伤害提交后**口径（与上游 damageSource content 一致）；失去体力走苦肉同款濒死检查；tag 46 |
| 华雄（群/6） | 耀武 yaowu | 锁定：受到红杀伤害时，伤害来源摸 1 或回 1 | DamageBefore 前置窗口锁定 + 来源 PickTag 二选（满血无回复选项，同上游 chooseDrawRecover 满血自动摸）；tag 47 |

登记差异：狂斧限次口径为每回合（等价上游每出牌阶段——触发仅在自身
出牌阶段）；骁果/耀武无语义差异（骁果上游 chooseToDiscard 自选基本牌
同效）。华雄体力 6 使 `general_faction_mapping_complete` 的体力区间
断言放宽到 ≤6（原 3–4 假设不适用于上游条目）。

## 上游 12 技全部定位（其余 5 将 9 技，后续分批）

| 将 | 技 | 上游位置 | 原生机制缺口 |
|---|---|---|---|
| 李典 old_re_lidian | 恂恂 xunxun | refresh/skill.js | 观星型（看 4 张分牌堆顶/底）机器 |
| | 忘隙 wangxi | refresh/skill.js | 伤害后双方各摸 1（可选，座位序） |
| 甘夫人 ganfuren | 淑慎 stdshushen | standard/skill.js | 回复后选他人摸 1/2 |
| | 神智 shenzhi | sp/skill.js | 准备阶段弃全部手牌，数量≥体力则回 1 |
| 公孙瓒 gongsunzan | 义从 reyicong | refresh/skill.js | 锁定动态距离修正（hp>2 距他人 −1；hp≤2 他人距你 +1） |
| 伊籍 xf_yiji | 机捷 xinfu_jijie | standard/skill.js | 出牌阶段限一次：牌堆底牌给任一角色 |
| | 急援 xinfu_jiyuan | standard/skill.js | 濒死/他人获得你的牌 → 摸 1（多事件源） |
| 袁术 re_yuanshu | 妄尊 rewangzun | standard/skill.js | 他人准备阶段锁定：血比你多摸 1；主公再摸 1+其手牌上限 −1（模式相关） |
| | 同疾 retongji | standard/skill.js | 杀目标可弃牌转移给你（目标转移机制） |

## 验收口径（分开陈述）

- **代码已实现**：3 技 + 3 将 + 快照变体/编解码/哈希臂 + 装配授予
  （复用 `apply_general_assembly`）。
- **回归已加入**：`standard_missing_generals_assembly_and_behaviors`
  （装配授予/华雄上限 6/骁果弃牌+伤害/狂斧两路径/耀武满血无回复与
  受伤可回复/伤害照常生效）。
- **a0 同步**：技能编号声明表 47–49（骁果/狂斧/耀武表尾追加）、
  快照变体样例 45–47（测试内变体序 43–45）。
- **全量通过**：engine-native **364/0/1**（唯一 ignore=写轮眼规模诊断
  探针）；golden（replay_scenarios）全绿——新将/新技能不触碰原型场景
  哈希，规则/格式版本不变（v61/v4，据守先例：目录追加不递增）；
  lib `-D warnings` 通过。
- 过程修正：狂斧/耀武测试首版踩「同回合第二张杀被 ShaLimit 先行拒绝」
  「respond_shan 强制应答」「目标被打死按存活检查跳过」三坑，均已按
  引擎既有口径修正测试（详见测试内注释）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（9 文件，相对 noname-next 根）。
