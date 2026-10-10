# U07 技能批三：筑城+筑城守卫（刘馥）（规则 v113，2026-10-08）

玩点论杀武将技能声明批三：**筑城**（zhucheng）+ **筑城守卫**
（zhucheng2，上游 group 子技能显式化）双映射——刘馥双技全量接线。未
映射声明 7→6（zhucheng2 显式化 +1 声明，映射 +2）。夺启维持暂缓
（phaseUse 中止未建模，且依赖筑城标记）。前置：
[U07 技能批二](../2026-10-08-u07-skill-batch2/README.md)。

## 上游语义核验（源码为准，archive/qita/sanguosha/noname/extension/玩点论杀/character/skill.js:160/213）

- `zhucheng`：trigger player phaseEnd、filter 无标记、content 开关——
  无标记：`get.cards(max(1, maxHp-hp))` 牌堆顶存标记（不入手）；有标
  记：收回入手+清空。check 为 AI→玩家确认（OptionalGate）。
- `zhucheng2`（group）：trigger target shaBefore、filter 标记非空 →
  使用者 chooseToDiscard(弃 len(标记) 张，可取消)——弃则杀照常、取消
  则 `trigger.cancel()`（杀无效）。

## 改动（分开陈述）

- **代码已实现**：`WudianZhucheng`（tag 99）+ `WudianZhuchengGuard`
  （tag 100）。**状态**：`PlayerState.zhucheng: Vec<CardId>`（跨回合
  持久标记；**存档格式 v4→v5**，enc/dec/哈希全套）。**开关**：原生无
  begin:false 帧——`commit_phase` PhaseEnd 段直推 OptionalGate（拥有者
  自己每阶段结束确认，上游 player:"phaseEnd" optional 同口径）；
  Act 收支两分支。**守卫**：TriggerMeta CardUsed/Normal + filter
  （use_as/实际=Sha && targets 含 owner && 标记非空）+ 三段机（使用者
  选一张=弃 n 张放行【首张自选+余随机代抽——上游一次性选 n 张，登记
  差异】/Pass=杀无效：帧 `cancelled=true` 写回 + 杀归还手牌 +
  **intrinsic_machine CardUsed 臂 cancelled 跳过结算**——CardUsed:
  Commit→AfterTriggers→Intrinsic 时序中 AfterTriggers 机器可写回帧载
  荷）。mapping 双条目（zhucheng/zhucheng2 显式化）。
- **登记差异**：守卫弃牌上游一次性选 n 张——原生首张自选+余随机代抽
  （代抽池排除正在结算的杀）；杀无效无专用日志（Death/CardPlayed 缺席
  可观察）；夺启暂缓。
- **转换包**：cards.json 刘馥 skills 三项（zhucheng/zhucheng2 映射 +
  duoqi 暂缓），npack 三哈希重算。

## 验收口径

- **回归已加入**：`liufu_zhucheng_toggle_and_sha_guard`——①结束阶段
  确认存 2 张（maxHp-hp）②他人杀→Pass=杀无效+杀回手+不掉血 ③再杀→
  弃 2 放行（掉血 2→1、杀结算离手）④开关收回（标记清空、标记牌入手）。
- **全量通过**：engine-native **430 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 112→113 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（筑城不在标准场景，纯版本位移；存档格式
  v4→v5 由版本门禁一并覆盖）。
- **剩余**：U07 未映射 6 条（夺启/绝道/割据/抗yin/车阵/诱战）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（19 文件，相对 noname-next 根）。
