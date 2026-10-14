# T12 玩点论杀批 14：跨回合存储面——复仇双技（西真夕红整将转正，2026-10-14）

## 任务来源

[T12 玩点论杀武将面](../../migration/upstream-extension-reconstruction.md)批 13 续批：
「跨回合存储面」立项落地（面级工程——ShaResolveMachine 注入+字段批）。

## 承载

| 技能 | 上游 | 原生承载 |
|---|---|---|
| 复仇 fuchou（西真夕红，target shaBefore direct） | 成为杀目标时弃 1 张手牌取消该杀；所选牌归来源；标记来源 | **ShaResolveMachine 内联选择阶段**（响应窗前：目标持复仇且有牌 → SelectCards 自选可取消）；取消结算=杀回来源手+所选牌归来源+`fuchou_marked.push(来源)`；`Flow::Done` 跳过全部后续（多目标队列一并取消） |
| 复仇·清算 fuchou2（forced group phaseAfter） | 回合结束后对标记来源逐个摸 1+强制用杀 | `WudianFuchou2`（tag 260）：自己 End 阶段结束触发（登记差异：上游 global phaseAfter，原生自己回合结束）；逐标记：存活→摸 1→**虚拟杀强制直发**（厉焰·链杀同款；登记差异：上游 chooseToUse 选杀实体且可取消）；死亡标记移除 |

**跨回合存储原语**：`PlayerState.fuchou_marked: Vec<PlayerId>`（存档 enc/dec +
canonical 哈希 push——ciqiu_dying 同构；死亡标记在清算时消耗）。

## 字段批（规则 v278→v279、存档 v66→v67）

- `fuchou_marked` 入哈希/存档；`ShaResolveMachine.asking_fuchou/fuchou_card`
  两字段入机器快照（四触点：struct/snapshot()/snap 变体/archive enc/dec）。
- **字段批探针**：版本回退 278 复跑 14 场景三值互异（278 无字段/278+字段/279+
  字段）——位移=版本+新字段双源、无行为面扰动；复仇分支不在标准场景可达
  路径；14 golden 重录+补录注（replay_scenarios.rs）。
- KNOWN_PROMPTS 注册 `wudian_fuchou_cancel`（固定提示词表）。

## 测试设计

复仇取消全链（杀→选择挂起→选牌→不受伤+杀回来源手+所选牌归来源+标记）；
清算（标记来源摸 1+虚拟杀+标记消耗）。**反例实证待补**：取消应答分支的
桩化反例以清算消耗断言间接覆盖（标记非空=取消发生）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | g1_wudian **121/121**（含复仇取消全链/清算链） |
| engine-native 全量 | **737/0/0**（重录后）——[日志](engine-native-tests.log) |
| 装载/包管理/服务端 | **84/0** |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- 玩点论杀武将面：**9/9 名、18/22 条目转正、4 暂缓**（七整将：+西真夕红；
  刘孚 2/3、田豫 1/2、杨仪 1/2、于吉/曹操部分）。
- 剩余暂缓（4+坚壁全域面）：良策（五谷 Remained 子事件）、仇渡（deferred 卡）、
  夺启/佑赞（存量）、奸雄·变体/归汉/巧思/悲歌（各自缺口）。
