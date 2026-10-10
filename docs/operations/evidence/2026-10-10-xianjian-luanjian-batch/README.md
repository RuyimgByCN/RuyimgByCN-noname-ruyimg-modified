# U05 xianjian 第四十五批切片（规则 v187，2026-10-10）

长卿（pal_changqing）**整将入包**（35→**36 将全员入包**）+乱剑系
（luanjian+luanjian2）闭环——天符挂附三缺口暂缓登记=长卿半闭环。
虚拟杀原语第五次消费+载体牌锚定增伤首例。前置：
[械磐批](../2026-10-10-xianjian-xiepan-batch/README.md)。

## 未入包 1 将选型依据

- **乱剑**：双杀 viewAs=虚拟杀（天剑弃素材同款）+多目标（机器直发）
  +50% 增伤（DamageBefore 载体锚定）——全部基建复用。**本批选乱剑**。
- **天符**：闪置武将牌（挂附实体区）+目标造成伤害后雷伤窗（伤害后
  内联）+死亡清理（dieBegin）——三缺口，登记暂缓。

## 上游语义核验（源码为准，…/character/xianjian/skill.js luanjian/luanjian2）

- `luanjian`：enable phaseUse——selectCard=2（两张杀）viewAs sha；
  selectTarget [1,∞]；filterTarget=targetEnabled（距离不在其中=无视
  距离）。
- `luanjian2`：source damageBegin forced——event.parent.skill===
  "luanjian"（来源为乱剑杀）&& Math.random()<0.5 → trigger.num++。

## 改动（分开陈述）

- **宿主行为 1**：`XianjianLuanjian`（tag 211，phaseUse once_per_turn
  主动=天剑命令层结构）——Act 发问两张杀（min=max=2，可取消=放弃）
  → 弃置+发问多目标（SelectPlayers [1,∞]，必选不退素材=登记差异）
  → 预读牌堆底载体存 `TurnState.luanjian_card` → use_virtual_card
  虚拟杀（机器直发不经 PlayCard 校验=无视距离+任意多目标同效果；
  **登记差异**：目标侧防具检查不适用）。
- **宿主行为 2**：`XianjianLuanjian2`（tag 212，DamageBefore forced）
  ——filter（source=自己+非防止+载体锚定=普通杀/其他虚拟技载体不
  命中）→ step：RNG<50% → DamageBefore 帧直写 amount+1（张目
  prevented 同款机制；每目标独立 roll）。
- **状态**：`TurnState.luanjian_card: Option<u32>`（入 canonical_hash
  +存档 tag 式 Option 编码）。存档 v37→**v38**；Judge begin 清。
- **cards.json**：长卿整将条目（wei/4 血/male；乱剑+乱剑2 host[乱
  剑2 reason_ref=天剑授技 def 源同款]、天符 null+理由登记）+
  rehash 两步；**版本** 186→187；a0 追加 211+212。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`changqing_luanjian_dual_sha_
  viewas_multi_target`（发动→选两杀→多目标 [1,2]→seed 10062 混合
  roll：目标 1 hp 6→5[无增伤]/目标 2 6→4[增伤 2 点]=增伤臂真实执
  行+逐目标独立；两杀入弃堆）。
- **全量通过**：engine-native **520 passed / 0 failed / 1 ignored**
  （v186 基线 519 +1）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 186→187 + luanjian_card 入 canonical_hash——14
  场景全 mismatch，重录后 replay_scenarios 2/2。

## 剩余

- **xianjian 36 将全员入包**。长卿半闭环（天符=挂附实体+伤害后窗+
  死亡清理三缺口）。机关牌族（御甲/偃师/dujiang）需 createCard 实
  体造牌基建先行；长期项不变（zhenfa 五阵法、整包验收 0/37 等）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（14 文件，相对 noname-next
根）。
