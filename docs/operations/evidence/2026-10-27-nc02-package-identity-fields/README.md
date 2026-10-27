# NC02：浏览器与 WASM 包身份字段校验

- 实际执行：2026-10-27 23:07:23 +0800 → 2026-10-27 23:11:38 +0800（开工与结束各运行 `date '+%F %T %z'`）；源码基准 `a0ec5a5eb424bfb4d9fc1bc014e4440df35f4d42`（[开始](start-time.txt)、[结束](end-time.txt)、[源码](source-head.txt)）。原有八处文档改动保留。

## 反例与修复

服务端 `ContentPackage::validate` 已拒绝身份字段为 `.`、`..` 或含 `/`、`\\`；浏览器先前只要求非空，WASM 只额外拒绝空白。浏览器内容锁把 `namespace/package_id` 拼为键，未经约束的分隔符会使不同的字段组合共享同一个文本键；WASM 同时会接受这些服务端拒绝的身份进入对局与恢复。

`git archive HEAD` 导出到[独立目录](red-path.txt)，只叠加回归测试；Rust 使用独立 `CARGO_TARGET_DIR`，Node 使用现有依赖。对三个字段分别测试 `.`、`..`、`name/child`、`name\\child`、空白：浏览器 **14 passed / 1 failed**，15 个非法值均被接受（[日志](red-frontend.log)）；WASM 定向 **0/1、exit 101**，四种路径样式在普通/身份开局和其原始存档恢复中均被接受，空白原已拒绝（[首轮](red-rust.log)、[最终反例](red-rust-final.log)）。

浏览器与 WASM 共用的各自身份入口现拒绝以上值，服务端规则不改，WASM 原有空白拒绝保持。普通、身份开局与恢复在构造任何包之前校验；以前被错误允许的非法身份存档现明确拒绝，合法命名空间不同的同 ID 包仍允许。没有更改规则、存档格式、ABI 或合法包状态。初轮 WASM 构建期间同一导出目录被用于加强干净源码反例，不能确认初轮产物所用源码；反例完成并还原文件后重新构建，以下只采用最终构建（[初轮日志](native-build.log)）。

链接检查首次引用尚未生成的冒烟日志而报 1 条坏链（[记录](local-links-before-smoke.log)）；补齐日志路径后通过，干净 HEAD 导出并补齐父目录的 AGENTS 与上游引用亦通过（[对照](local-links-clean-head.log)）。

## 验证

| 命令 | 结果 |
|---|---|
| `cargo test --offline -p engine-native-wasm --test installed_package_payload` | **6 passed / 0 failed / 0 ignored**，包括三个身份字段、普通与身份局及合法不同命名空间；后续补强恢复错误字段断言，随全量验证（[日志](wasm-targeted.log)）。 |
| `cargo test --offline -p engine-native-wasm -p content-native-loader --no-fail-fast` | exit 0；WASM **55/0/0**，loader **32/0/0**（通过/失败/忽略；[初轮](rust-full.log)、[最终](rust-final.log)、[计数](rust-counts.txt)）。 |
| `pnpm --dir apps/web-client test`；`pnpm --dir apps/web-client typecheck` | **86/0/0**；类型检查 exit 0（[测试](frontend-test.log)、[类型](frontend-typecheck.log)）。 |
| `CARGO_NET_OFFLINE=true pnpm --dir <独立导出>/apps/web-client build:native`；`pnpm --dir <独立导出>/apps/web-client smoke:native` | 实际 WASM 构建 exit 0；Node Worker onmessage 冒烟 **53/0**，exit 0（[最终构建](native-build-final.log)、[冒烟](native-smoke.log)）；非法命名空间在普通、身份开局及恢复被拒，原对局存档不变。导出仅叠加本批源码、测试和冒烟脚本，独立 target；未覆盖工作区生成产物。 |
| `rustfmt --check --edition 2021` 两个 Rust 文件 | exit 0（[最终日志](rustfmt-final.log)）。 |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'`；`python3 -B scripts/check-source-lines.py` | **1/0**；8621 个文件通过（[单测](source-unit.log)、[行数](source-lines.log)）。 |
| `python3 -B scripts/check_local_links.py --all`；`python3 -B scripts/check-adr-index.py` | 链接 0 坏链（[日志](local-links.log)）；ADR exit 0（[日志](adr.log)）。 |

NC02 仍为部分完成：VM 本地安装、跨包依赖和多类型场景待验。本批没有做浏览器 UI 全流程或跨设备验收，也不声称所有包内容或指纹已认证。
