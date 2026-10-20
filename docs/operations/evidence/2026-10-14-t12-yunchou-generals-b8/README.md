# T12 运筹武将面批 8：内置技能引用面——护驾/谦逊转正（2026-10-14）

## 任务来源

[T12 运筹武将面](../../../migration/upstream-extension-reconstruction.md)批 7 续批：
三类可立项缺口之首「内置技能引用面」——mapping `builtin:*` 形态一次性解锁
护驾/悲歌/谦逊三技中的**有实现者**。

## 勘察定案

| 技 | 原生实现 | 处置 |
|---|---|---|
| 护驾 HU_JIA | standard 常量 + kernel 主公技臂（v204 差异消除批） | **builtin:护驾 转正** |
| 谦逊 QIAN_XUN | standard 常量 + SkillDef（乐不思蜀目标校验查询面） | **builtin:谦逊 转正** |
| 悲歌 beige | **standard 零命中——本体未实现** | 不入白名单，mapping 保持 null 暂缓 |

## 交付

- **content-native-loader**：`mapping_of` 三态化（`MappedSkill`：Deferred/
  Builtin/Host）——`builtin:名` 前缀 + **白名单** `BUILTIN_SKILL_REFS=[护驾,
  谦逊]`（严格桥接哲学：loader 无目录访问，白名单而非开放查名——引用不存在的
  def 会行为面静默失效）。Builtin 只 intern 进武将技能列表、不产出 host 行为
  （def 已在标准目录）。cards.json 侧调用点同步三态（Deferred/Builtin 跳过）。
- character.json：护驾/谦逊改 `builtin:*`。
- 测试 +2：桥接三态断言（曹操=[戏才,护驾]、陆抗含谦逊、unmapped=6）、建局
  目录面（护驾 def 在标准目录、陆抗三技含谦逊）、**谦逊行为定向**（持谦逊者
  乐不思蜀指定被拒——勘正：谦逊原生校验面=乐不思蜀，上游无顺手牵羊挂账见
  覆盖清单）、**白名单拒装负例**（`builtin:悲歌` → UnknownBehaviorKey）。

## 语义勘误（回归驱动）

谦逊原生校验对象勘正：engine.rs 乐分支（「谦逊：不能成为乐不思蜀目标；上游
无【顺手牵羊】，见覆盖清单」）——测试首版误用顺手牵羊（该卡上游无实体）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | g1_shahai_yunchou **22/22**（含 builtin 白名单拒装负例=反例实证） |
| engine-native 全量 | **736/0/0**（golden 不变）——[日志](engine-native-tests.log) |
| 装载/包管理/服务端 | **84/0** |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- 运筹武将面：**10/10 名、15/20 条目转正、5 暂缓**（五整将+曹操/陆抗近整将
  ——曹操 2/3、陆抗 3/3、甄姬/周瑜·DIY 2/2、于吉 1/2）。
- 剩余暂缓（5）：奸雄·变体（身份改写）、悲歌（本体实现）、蝉缘（max_hp 减
  面）、归汉（濒死主动技面）、巧思（骰子+多步）——精械已并入外部包依赖类。
- 内置引用面泛化：后续武将包引用有实现的标准技循 `builtin:*`+白名单扩充。
