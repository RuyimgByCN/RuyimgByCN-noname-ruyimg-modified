# U05 ow 纵切批七：堡垒/侦察切换（规则 v124，2026-10-08）

ow 纵切批七：**堡垒/侦察切换**（bshaowei，堡垒双技共用 mapping 通道）
——出牌阶段确认切换模式（三态翻转：无→哨卫→侦察→哨卫），
`PlayerState.baolei_mode: u8` 三态入状态（存档 v6 内追加字段）。**模式
效果钩登记待做**（哨卫四合一/侦察用杀摸回——需 ShaResolve 深层多处
修改）。前置：[ow 纵切批六](../2026-10-08-ow-paotai-batch/README.md)。

## 上游语义核验（源码为准，…/character/ow/skill.js bshaowei/zhencha）

- `bshaowei`：phaseEnd 确认切换至哨卫（storage 翻转）；哨卫效果=杀无
  视距离/防具、无次数限制、不可闪避（subSkill sha：mod targetInRange/
  cardUsable + trigger shaBegin directHit）+ 持有者不能闪避杀。
- `zhencha`：phaseEnd 确认切换至侦察；侦察效果=subSkill sha：用杀后
  chooseDrawRecover（摸 1 或回 1）。

## 改动（分开陈述）

- **代码已实现**：`PlayerState.baolei_mode: u8`（0=无/1=哨卫/2=侦察，
  跨回合持久；存档 v6 内追加）+ `WudianBaoleiSwitch`（tag 117）单步切
  换机（三态翻转：2→1 回哨卫、其余 +1——上游双技各 phaseEnd 确认的
  互斥切换合并为确认式翻转）。mapping 双技同键（bshaowei/zhencha 均
  映射 switch 行为）。
- **登记暂缓（模式效果钩）**：哨卫四合一（杀无视距离/防具、无次数、
  不可闪避+持有者禁闪——需 ShaResolve/闪避门控深层多处）与侦察用杀
  摸回（需 CardUsed Sha 触发机）——留待后续批；本切片仅状态通道。
- **转换包**：cards.json 增 堡垒（generals 双技同键）+ bshaowei skills
  条目，三哈希重算。

## 验收口径

- **回归已加入**：`baolei_switch_mode_cycle`——三态翻转 0→1→2→1 →
  存档往返保持 → 跨回合持久。
- **全量通过**：engine-native **437 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 123→124 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（堡垒不在标准场景，纯版本位移）。
- **剩余**：ow 其余 19 武将（含堡垒模式效果钩）、xianjian 36 武将逐武
  将纵切；U07 duoqi/youzhan 暂缓；zhenfa 五种阵法。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（15 文件，相对 noname-next 根）。
