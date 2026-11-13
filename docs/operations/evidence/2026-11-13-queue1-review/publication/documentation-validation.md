# 2026-11-14T01:37:30+08:00 纯文档伴随提交前核验

HEAD `1fa8ac6cc3a70446436d97ff49aa19ea4626e22d` + 文档/证据dirty；规则333／NGSA107／schema5／ABI759／WS8。代码首轮精确CI的10个作业已通过，见[终态及全部原始日志](code-exact-ci-first/README.md)。

本次只新增日期记录、发布元数据与CI原始证据；[提交前保护](pre-documentation-commit-protection.json)核对32259个冻结产品输入无变化、672个原有未跟踪路径仍在、8个旧工作树登记未动。既有文档仅新增行，旧日期记录及表格保留。

| 实跑检查 | 结果 | 原始命令与输出 |
| --- | --- | --- |
| 默认主稿链接 | 7份，坏链0 | [元数据](../29-documentation-links.json)／[日志](../29-documentation-links.log) |
| 全量文档链接 | 2103份，坏链0 | [元数据](../33-documentation-links-all.json)／[日志](../33-documentation-links-all.log) |
| 行号引用 | 10584处，越界0；48类路径未解析仍保留 | [元数据](../30-documentation-line-refs.json)／[日志](../30-documentation-line-refs.log) |
| 行数门禁单测 | 2通过／0失败 | [元数据](../31-documentation-source-unit.json)／[日志](../31-documentation-source-unit.log) |
| 源码物理行／Rust有效行 | 9364文件通过 | [元数据](../32-documentation-source-lines.json)／[日志](../32-documentation-source-lines.log) |
| 本次已跟踪文档空白 | exit0 | [元数据](../34-documentation-owned-diff-check.json)／[日志](../34-documentation-owned-diff-check.log) |

[机器摘要](documentation-validation.json)。不重复执行源码未变的完整Rust与产品门禁；结果仍以此前对应源码的[本地门禁表](../final-local-gates.md)和[精确代码CI](code-exact-ci-first/README.md)为准。没有把原始日志空白、48类未解析路径、安装器构建或绿色CI解释成整源规则/平台验收。记录时文档伴随提交尚未生成，实际SHA、推送与文档CI另留本地证据。


## 2026-11-14T01:38:57+08:00 暂存日志字节口径更正

Git依现有配置将26份Windows日志文本视图的CRLF归一化为LF；此前暂存审计中 `raw_ci_logs_preserved_byte_for_byte=true` 不适用于全部Git文本blob，撤回该解释。原始ZIP的SHA256 `83295d5edb78f2e7c3c1b4040a4fbf59af6d48b50ad5d1815b206a55efe8ef48` 与暂存blob一致，148个ZIP条目与本地下载原始文件逐字节一致；文本视图只有换行归一化，无其他字节变化。见[逐文件指纹与更正](raw-log-normalization-correction.json)。未修改持久Git配置或产品输入；此前记录保留。
