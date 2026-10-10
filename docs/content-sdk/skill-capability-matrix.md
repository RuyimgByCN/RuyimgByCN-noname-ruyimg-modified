# M0 技能能力画像与原语矩阵

> 这是 `legacy-analyzer` 测试钉住的 **M0 样本矩阵**（standard + shenhua，243 行），
> **不是**当前全仓进度。活清单在
> [`content/original/skill-capability-matrix.md`](../../content/original/skill-capability-matrix.md)
> （5988 源码画像）与 [`content/original/catalog.json`](../../content/original/catalog.json)
> （6068 顶层技能 / 5982 条关联画像）。
>
> Heuristic source inventory only; each row still requires semantic Golden tests before migration is complete.

共 **243** 个源码技能画像：

- `shenhua`：174
- `standard`：69

## 覆盖等级

| 等级 | 数量 |
|---|---:|
| A | 1 |
| B | 135 |
| C | 101 |
| D | 6 |

## 缺口聚合

| 缺口 | 技能数 | 当前处理 |
|---|---:|---|
| `choice_cost_policy` | 116 | 已有 Choice/optional；逐技能确认成本语义 |
| `effect_or_intent_review` | 83 | 已有通用 Effect/Intent；缺口按 M0 工艺补原语 |
| `forbidden_host_rewrite` | 82 | 禁止透传 UI/状态全局，改写为投影或通用 Host |
| `semantic_manual_review` | 48 | 逐技能 Golden 人工确认 |
| `subskill_state` | 39 | 需技能状态/组合技能语义 |
| `trigger_point_review` | 57 | 与 rule-semantics 对齐后新增通用 Trigger Point |

完整逐技能行见 `skill-capability-matrix.json`。该 JSON 保存触发点、主动/视为/锁定/可选画像、已映射 effect、所需 surface 与缺口，可由 `legacy-analyze matrix` 重建。
