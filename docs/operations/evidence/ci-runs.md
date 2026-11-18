# 原生收敛 CI 运行台账

- 授权范围：裁决 25（仅允许非 force 推送到 `https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified.git` 的 `refs/heads/native-convergence`）。
- 登记规范：每次发布后在本表按每次一行追加登记；若首跑（Attempt 1）失败，保留首跑失败行并另起一行记录后续处理，不再为每次发布新建 `docs/operations/evidence/` 子目录。
- 原始日志规范：推送记录、CI API JSON、`logs-first.zip` 及解包日志统一存放在被 `.gitignore` 忽略的 `docs/operations/evidence-raw/ci/<sha>/`，本表仅写反引号路径与 sha256（不使用 Markdown 相对链接）。
- 2026-11-18 之前历史 CI 证据保留于 `docs/operations/evidence/2026-11-14-queue1-runtime/publication/` 等既有目录，不迁移不改写。

| 日期 | 提交 SHA | Run ID | 结果 | 原始日志与 sha256 |
| --- | --- | --- | --- | --- |
