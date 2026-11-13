# 阶段过滤**审计登记表**（52 条，2026-10-19）

> 2026-11-13T22:03:30+08:00 导航补充：本目录 `registry.txt` 保留为原32行历史快照，文件逐字节未改。当前生成器改读写[现行阶段登记表](../../../architecture/native-phase-filter-registry.tsv)；新增5个既有源码分支均未核，未新增等价认证。HEAD `3a6e398485db6ef0729abe3013c3c4b7705a5d6d` + tracked dirty，版本333／107／5／759／WS8；[迁移与独立复核](../2026-11-13-queue0-gates/independent-review/phase-registry-incremental-review.md)。下文原日期的范围、数字与核对结论保留。


> **为什么需要**：核了 35 条之后发现，**「已核哪些」只存在于会话记录里**，文档里只有数字 ⇒ **下一个人无法接着核剩下的**。
> 本表把 **52 条全部列出并标注状态** ⇒ **每核一条改一行即可**。

- 数据：[registry.txt](registry.txt)（**TSV：行为 / key / Phase / begin / 状态 / 依据**）
- 生成方式：扫 `content/host/filtering*.rs`，取每个 `HostBehavior::X` 分支里的 `Phase { phase, begin }`

## 1 统计

| 状态 | 条数 |
|---|---:|
| **已核** | **36** |
| **不适用**（卡牌效果，上游无 `trigger`）| **4** |
| **未核** | **12** |
| **合计** | **52** |

## 2 状态定义

| 状态 | 含义 |
|---|---|
| **已核** | **已与上游 `trigger` 逐条比对**，结论写在[规则基准](../../../architecture/native-rules-baseline.md)对应章节 |
| **不适用** | **比对对象不存在**（`GujianFood*` 三条 + `PhaseEndRecoverOrDraw` 是**卡牌效果宿主**，上游卡牌没有 `trigger`）|
| **未核** | **还没比对** |

## 3 怎么用

1. 打开 `registry.txt`，筛 `状态 = 未核`；
2. 按 `key` 到包数据 `skills[].mapping` 反查**上游技能名**（**这一步跳过会白查**）；
3. 到 `archive/upstream/*/skill.js` 读该技能的 `trigger`；
4. 与 `Phase`/`begin` 比对，**结论写进基准**，**本表状态改 `已核`**。

## 4 边界

- **「已核」36 条**里，**部分结论是「需判定」**（**不等于「无问题」**）；
- **本表只记「有没有比对过」**，**不记结论好坏** —— 结论在基准里；
- **`未核` 12 条**：**`GwentXiqiang*` 类的自定义命名** 为主，**需经 `skills[].mapping` 反查**。
