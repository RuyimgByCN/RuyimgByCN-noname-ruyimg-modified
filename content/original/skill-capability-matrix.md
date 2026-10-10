# M0 技能能力画像与原语矩阵

> Heuristic source inventory only; each row still requires semantic Golden tests before migration is complete.

共 **6108** 个源码技能画像：

- `bingshi`：86
- `clan`：89
- `collab`：155
- `diy`：137
- `extra`：62
- `huicui`：254
- `jsrg`：180
- `key`：240
- `mobile`：423
- `newjiang`：101
- `offline`：3
- `old`：27
- `onlyOL`：185
- `refresh`：309
- `sb`：120
- `shenhua`：174
- `shiji`：166
- `sixiang`：174
- `sp`：718
- `sp2`：277
- `standard`：69
- `sxrm`：51
- `tw`：391
- `xianding`：521
- `yijiang`：274
- `yingbian`：91

## 覆盖等级

| 等级 | 数量 |
|---|---:|
| A | 1 |
| B | 3104 |
| C | 2592 |
| D | 411 |

## 缺口聚合

| 缺口 | 技能数 | 当前处理 |
|---|---:|---|
| `choice_cost_policy` | 3366 | 已有 Choice/optional；逐技能确认成本语义 |
| `effect_or_intent_review` | 2101 | 已有通用 Effect/Intent；缺口按 M0 工艺补原语 |
| `forbidden_host_rewrite` | 2834 | 禁止透传 UI/状态全局，改写为投影或通用 Host |
| `semantic_manual_review` | 696 | 逐技能 Golden 人工确认 |
| `subskill_state` | 2330 | 需技能状态/组合技能语义 |
| `trigger_point_review` | 2134 | 与 rule-semantics 对齐后新增通用 Trigger Point |

完整逐技能行见 `skill-capability-matrix.json`。该 JSON 保存触发点、主动/视为/锁定/可选画像、已映射 effect、所需 surface 与缺口，可由 `legacy-analyze matrix` 重建。
