# T12 运筹武将面批 17：归汉——濒死主动技面（蔡文姬部分转正，2026-10-14）

## 任务来源与承载

[T12 运筹武将面](../../architecture/native-progress.md)批 15 续批：三类可立项
缺口中「归汉濒死主动技面」立项落地。

- **limited 承载**：`PlayerState.guihan_used: bool`（暴雪 baoxue_used v127
  先例同构；存档 enc/dec + canonical 哈希 push——**字段批**：规则 v279→
  **v280**、存档 v67→**v68**）。
- **DyingSaveMachine 内联**（求救循环前一次性确认）：`guihan_asked` 快照字段
  （快照变体/archive enc/dec/engine 哈希模式四触点）+ 确认分支——目标持归汉
  且未用 → Confirm 门 → 效果：自回 1（救回由循环顶 hp>0 判定自然发出
  DyingSaved）摸 2 + 座位序首男性其他存活回 1 摸 2 + limited 置位。
- **登记差异**：上游 chooseToUse 玩家主动+自选男性目标，原生锁定确认+确定性
  目标（座位序首男性，满血回 1 饱和 +0）。
- character.json：蔡文姬归汉 mapping 转正（`host:guihan`，声明名 ycnguihan）。

## 测试要点（回归驱动）

- 归汉自回 1 **不足以从 -1 救回**（需 hp>0）——布阵 hp2-2=-1 → 归汉 +1=1 救回
  ✓；男性目标座位 0 满血回 1 饱和 +0（摸 2 不饱和可断言）。
- limited：第二次濒死不再触发（死亡）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | g1_shahai_yunchou **24/24**（救回链/limited 一次性/男性目标饱和回） |
| engine-native 全量 | **738/0/0**（重录后；探针三值互异 [probe-v279-field.log](probe-v279-field.log)） |
| 装载/包管理/服务端 | **84/0** |
| Gate 1b | exit 0（拦 dec/verify 模式未用绑定三处，已修）——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- 运筹武将面：**10/10 名、17/20 条目转正、3 暂缓**（蔡文姬 1/2——悲歌本体
  实现后 builtin 扩白名单即整将）。
- 剩余暂缓（3+坚壁全域面）：奸雄·变体（身份改写面）、悲歌（本体）、巧思
  （骰子+多步）。
- 濒死确认注入模式泛化：后续濒死主动技（归汉类）循 DyingSaveMachine
  确认位直接承载。

## 补遗（工作区核验发现 enc/dec 不对称并修复）

批 17 初版的 DyingSave 快照改动存在 **archive enc/dec 不对称**（enc 臂
`guihan_asked: _` 忽略写入、dec 读三字段）——各批定向与全量在无 DyingSave
挂起存档往返路径下未暴露；全工作区核验由 a0_tag_registry 往返守护与
native_kernel dying_save 恢复测试抓出（UnexpectedEof/BadTag choice_kind）。
已修（enc 写入对齐 dec）。同时同步 lib 冻结键数（258→262，批 12–17 的
Liduan/Jianbi/Fuchou/Fuchou2/LiangceRemain/Guihan 六行为）。全工作区
**5325/0/0**（exit 0）。**流程强化**：涉及快照结构的批次清单增加「archive
enc/dec 对称 + 全工作区核验」双项。
