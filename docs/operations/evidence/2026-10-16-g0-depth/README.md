# 2026-10-16 E01/G0 深度清单收尾

## 交付

- **37/37** 来源 G0 文档：[`docs/migration/g0/`](../../../migration/g0/README.md)（每份含功能清单、菜单/规则分支语义、目标接口、验证入口、旧行为动态样本引用）。
- 生成器：`scripts/legacy-harness/generate-g0-docs.py`（基线=10-07 功能面清点 + 台账摘要）。
- 无头旧线 harness：`scripts/legacy-harness/`（草稿可复用并续写；`node_modules` 已 ignore）。
- 龙血/Boss/ylyg 轨迹：[2026-10-16-t9-lx-battle-trace](../2026-10-16-t9-lx-battle-trace/README.md)。

## 命令

```bash
python3 scripts/legacy-harness/generate-g0-docs.py
ls docs/migration/g0 | wc -l   # 38 = 37 来源 + README
```

核验日期：2026-10-16。
