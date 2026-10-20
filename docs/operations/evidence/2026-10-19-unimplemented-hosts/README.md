# ❌ 本目录结论已撤回（2026-10-19）

`list.tsv` 统计的是 **`factory.rs` 里为 `(None, None)` 的行为**，
当时据此推测「未实现」，**已被第 677 轮实测推翻**：
`Bingfeng2`/`Cyqiaoxie`/`Gaizao`/`Gongxin`/`Niepan` 等**都有 `step`/`metadata`/`tags` 实现**。

⇒ **`(None, None)` 只表示「没有 `activatable` 与 `factory` 层 step」，与「未实现」无关。**

**保留本目录的理由**：`list.tsv` 仍是一份「无 `activatable` 入口的行为清单」，
**对「哪些技能无法主动发动」这个问题有参考价值**，但**不能读作「未实现」**。

**撤回与依据**：[交接页 §4.8](../2026-10-19-rules-diff-handover/README.md)
