# NC02：本地同版本包副本冲突

- 实际执行：2026-10-27 23:03:37 +0800 → 2026-10-27 23:06:17 +0800（开工与结束各运行 `date '+%F %T %z'`）；源码基准 `095363a4dfdf8936f0e42d35227eeac672399f7d`（[开始](start-time.txt)、[结束](end-time.txt)、[源码](source-head.txt)）。工作树原有八处文档改动保留。

## 反例与修复

本地安装表中同一个命名空间、包 ID 和版本若已经保留了与当前目录不同的副本，旧 `installExtension` 只更新时间、`upgradeExtension` 仍保留旧副本并改写目录指纹；后者可能使此前能开局的内容锁无法再通过指纹检查。资源内容变化而逻辑指纹不变时，同版本冲突也会被忽略。

从 `git archive HEAD` 导出到[独立目录](red-path.txt)，只叠加本批新增测试，使用现有 Node 依赖；真实欢乐卡牌包分别构造逻辑指纹变化与卡牌资产变化，分别调用安装和升级。四个反例在原源码均未报错：**10 passed / 4 failed / 0 skipped，exit 1**（[日志](red-clean-head-final.log)）；最初单一测试亦复现未拒绝（[日志](red-clean-head.log)）。

安装和升级先比较同版本已有副本与来源的完整 JSON，遇到内容差异就拒绝且不写 localStorage；比较时排序对象键并忽略 JSON 格式差异，数组顺序仍参与比较。相同副本继续幂等并保留原始存储字节；无副本的旧元数据记录仍可修复。此检查并不验证包的真实性或加密摘要，也不开放 VM 本地安装入口。

链接检查首次发现本证据提前引用了尚未生成的 `end-time.txt`（[日志](local-links-before-end-file.log)），创建该文件后通过。干净 `git archive HEAD` 对照补齐父目录的 `AGENTS.md` 与上游 `archive` 引用后为 **1831 文件、0 坏链**（[日志](local-links-clean-head.log)）；两次未补齐外部路径的对照分别报 5、45 条坏链（[首次](local-links-clean-head-without-parent.log)、[第二次](local-links-clean-head-without-archive.log)），不归因于其他改动。

## 验证

| 命令 | 完整结果 |
|---|---|
| `node --import ./scripts/ts-register.mjs --test scripts/native-match-content.test.mjs` | 初版定向 **11/0/0**，exit 0（[日志](targeted.log)）；最终四个反例拆成独立测试，并随下方前端全量通过。 |
| `pnpm --dir apps/web-client test` | 最终 **85 passed / 0 failed / 0 skipped**，exit 0；包括逻辑指纹、资源内容两个变化分别在安装/升级拒绝且存储不变，以及格式/键序等价对照（[日志](frontend-final.log)）。 |
| `pnpm --dir apps/web-client typecheck` | exit 0（[日志](frontend-typecheck.log)）。 |
| `cargo test --offline -p engine-native-wasm -p content-native-loader --no-fail-fast` | exit 0；wasm **54/0/0**、loader **32/0/0**（通过/失败/忽略；[日志](rust-full.log)、[计数](rust-counts.txt)）。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py` | **1/0**；源文件行数检查通过（[单测](source-unit.log)、[行数](source-lines.log)）。 |
| `python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py` | 本地链接 0 坏链（[日志](local-links.log)）；ADR exit 0（[日志](adr.log)）。 |

NC02 仍为部分完成：VM 本地安装、跨包依赖与多类型场景待验；本批测试是生产 TypeScript 模块级测试，不宣称浏览器 UI、移动设备或整包验收。本批没有变更规则、存档格式或 ABI。
