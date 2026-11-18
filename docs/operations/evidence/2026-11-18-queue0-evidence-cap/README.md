# 2026-11-18 队列第 0 项：证据存放改造与体量/类型门禁

- 依据基准：基线提交 `21266d00c0583b692d3c7c7090678c487f211843`；当前规则版本保持 `规则 333／NGSA 107／content schema 5／ABI 759／WS 8`；清单计数保持 `已复核等价 625／简化近似 3,799／缺失 0`；整包八项签署保持 `0/37`。
- 边界声明：本批只改证据存放规则、门禁脚本、发布守卫脚本与说明文档，不删除历史已入库或基线既有未跟踪文件，不改写 Git 历史或强推，不修改业务规则与产品代码。

## 改造内容

1. **忽略原始证据目录**：[.gitignore](../../../../.gitignore) 末尾追加 `/docs/operations/evidence-raw/`，本地建立 `docs/operations/evidence-raw/` 存放原始日志、`.ngsa` 存档、浏览器 CDP 会话、CI 下载包与原尺寸截图（不入库）。
2. **链接与行号脚本跳过原始目录**：修改 [check_local_links.py](../../../../scripts/check_local_links.py) 与 [check_doc_line_refs.py](../../../../scripts/check_doc_line_refs.py)（及对应单测），遍历 `docs/**/*.md` 时跳过 `docs/operations/evidence-raw/`；同时在 `check_local_links.py` 禁止入库文档以 Markdown 相对链接指向 `evidence-raw/`（约定入库 Markdown 改用反引号路径 + sha256）。
3. **证据体量与类型门禁**：新增 [check_evidence_budget.py](../../../../scripts/check_evidence_budget.py)、[test_check_evidence_budget.py](../../../../scripts/test_check_evidence_budget.py) 及基线既有未跟踪快照白名单 `scripts/evidence_legacy_untracked.json`（覆盖基线前 `2,985` 个未跟踪遗留路径与 `sha256`，仅放行未改动原件，拦截 2026-11-18 起新增或改写文件），并注册进 [preflight.py](../../../../scripts/preflight.py) 第 22 项「证据体量与类型门禁」：
   - 基线：`21266d00c0583b692d3c7c7090678c487f211843`，历史已入库文件不触发报错；
   - 范围：逐一检查 `baseline..HEAD` 提交 `--diff-filter=AM`、暂存区 `--cached --diff-filter=AM`、磁盘未提交改动与新增未跟踪文件；
   - 硬限制：单文件 `<= 200 KB`（`204,800 B`）；单批（单个提交／暂存区／未提交增量／单个新证据子目录）合计 `<= 2 MB`（`2,097,152 B`）；禁止新增或修改 `.rs/.py/.js/.mjs/.cjs/.ts/.tsx/.vue/.sh/.wasm/.ngsa/.archive/.zip/.tar/.gz/.jsonl`。
4. **裁决 25 发布与 CI 登记**：新增 [publish_decision25.py](../../../../scripts/publish_decision25.py) 与 [test_publish_decision25.py](../../../../scripts/test_publish_decision25.py)（校验 `HEAD == sha`、已跟踪树干净、`pre-push` 钩子与 `git config` sha256 未改、非 force 快进推送 `https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified.git` 的 `refs/heads/native-convergence`，原始推送/CI 日志写入 `docs/operations/evidence-raw/ci/<sha>/`），并新建单表台账 [CI 运行台账](../ci-runs.md)。

## 本地门禁实测与发布状态

- `python3 -B -m unittest discover -s scripts`：207 项通过校验（206 通过 / 0 失败 / 1 既有跳过：`test_report_guard_coverage.py` 因 2026-11-03 台账归零跳过）。
- `python3 -B scripts/check-source-lines.py`：9,368 个受维护文件全部符合 `<= 2,000` 行及 Rust 非测试有效代码 `<= 1,000` 行上限。
- `python3 -B scripts/preflight.py`：22 项门禁全部通过（含新增第 20 项「证据体量与类型门禁」）。
- 提交与发布：本批提交后由 `scripts/publish_decision25.py` 非 force 推送 `refs/heads/native-convergence`，原始推送与 exact-head CI 日志存档至 `docs/operations/evidence-raw/ci/<sha>/`，CI 终态另记入 [CI 运行台账](../ci-runs.md)。
- 汇总入口：[进度主稿](../../../architecture/native-progress.md)、[剩余工作台账](../../../architecture/native-remaining-ledger.md) 与 [CI 运行台账](../ci-runs.md)。
