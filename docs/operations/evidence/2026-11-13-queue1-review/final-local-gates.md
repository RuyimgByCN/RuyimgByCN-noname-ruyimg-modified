# 2026-11-14 队列1部分增量最终本地门禁

核验时间 2026-11-14T00:57:58.387710+08:00；源码HEAD `bd58ef06d59940baed8d7f93e61ce571b152fb81` + tracked dirty；规则333／NGSA107／schema5／ABI759／WS8。以下命令均为实际执行且exit0；CWD和完整环境、起止时间见每行JSON。三crate和workspace无过滤参数。尚未提交／发布本批，精确代码CI另增记录。

| 实际命令 | 结果与边界 | 完整证据 |
| --- | --- | --- |
| `python3 -B -m unittest discover -s scripts -p test_check_source_lines.py` | 2通过／0失败 | [命令](01-line-unit-final.json)／[原始输出](01-line-unit-final.log) |
| `python3 -B scripts/check-source-lines.py` | 9364文件；物理≤2000／Rust非测试有效≤1000 | [命令](02-line-limit-final.json)／[原始输出](02-line-limit-final.log) |
| `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native` | 2446通过／0失败／6忽略，18段，过滤0 | [命令](03-native-three-final.json)／[原始输出](03-native-three-final.log) |
| `cargo check -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets` | 独立CARGO_TARGET_DIR，RUSTFLAGS=-D warnings | [命令](04-strict-all-targets-final.json)／[原始输出](04-strict-all-targets-final.log) |
| `cargo test --offline --workspace --no-fail-fast` | 7842通过／0失败／8忽略，178段，过滤0 | [命令](05-workspace-final.json)／[原始输出](05-workspace-final.log) |
| `cargo run -p conformance` | ALL PASS；旧线conformance范围 | [命令](06-conformance-final.json)／[原始输出](06-conformance-final.log) |
| `cargo check --target wasm32-unknown-unknown -p engine-native-wasm` | wasm32检查通过 | [命令](07-wasm-target-final.json)／[原始输出](07-wasm-target-final.log) |
| `bash scripts/cross-target-hash.sh` | ALL CROSS-TARGET CHECKS PASSED | [命令](08-cross-target-final.json)／[原始输出](08-cross-target-final.log) |
| `cargo test -p desktop-client` | 20通过／0失败／1忽略，4段 | [命令](09-desktop-final.json)／[原始输出](09-desktop-final.log) |
| `pnpm test` | 489通过／0失败／0跳过 | [命令](10-frontend-test-final.json)／[原始输出](10-frontend-test-final.log) |
| `pnpm typecheck` | exit0 | [命令](11-frontend-typecheck-final.json)／[原始输出](11-frontend-typecheck-final.log) |
| `pnpm build:native` | 原生WASM产物重建成功 | [命令](12-build-native-final.json)／[原始输出](12-build-native-final.log) |
| `pnpm smoke:native` | 83检查／0失败 | [命令](13-smoke-native-final.json)／[原始输出](13-smoke-native-final.log) |
| `python3 -B scripts/check-native-wasm-cross-target.py` | 标准及欢乐包轨迹/恢复一致；脚本内Rust定向3通过／140过滤，不算全量crate | [命令](14-native-wasm-cross-target-final.json)／[原始输出](14-native-wasm-cross-target-final.log) |
| `pnpm build` | 生产构建成功；仅5个已核实生成文件从HEAD还原，原built副本哈希保留 | [命令](15-frontend-build-final.json)／[原始输出](15-frontend-build-final.log) |
| `python3 -B scripts/preflight.py` | 21项通过／0失败；第二次完整运行 | [命令](22-preflight-after-link-correction.json)／[原始输出](22-preflight-after-link-correction.log) |

## 首次失败与补跑

- 第一次wasm32检查exit101：Homebrew rustc1.99.1缺wasm std。未改源码；命令级PATH切到现有rustup1.95.0后原命令通过。原失败[命令](07-wasm-target-first.json)／[输出](07-wasm-target-first.log)保留；[工具链观察](toolchain-observation.json)区分本机原生1.99.1、WASM1.95.0，CI工具链须以新run实际日志为准。
- 首次完整preflight exit1：20项通过、全量本地链接一项失败636处；[原命令](20-preflight-first.json)／[原输出](20-preflight-first.log)、[精确链接修正](link-correction-20261114-01/correction.md)保留。只修10份证据href，不改语义、状态、原日期或检查器。第二次完整preflight21项通过；首次结果不覆盖。
- RNG修复前的`*-first`全套和定向首红均保留；本表只采用RNG修复后的`*-final`结果。独立首审的未洗牌退回见[初版结论](independent/u01/implementation-history/initial-unshuffled/j06-rejection-conclusion.json)，最终限定源码复审见[补审](independent/u01/j06-followup-review.md)。

## 未执行与保护范围

各门禁计数重叠，不累加。6/8/1忽略名单见[原始逐项清单](final-ignored-test-scope.json)；其中悬空引用探针及人工性能测量未执行，原登记缺陷未据此关闭。全量链接坏链0；严格行号检查10584处越界0，另有48类未解析路径，不能据此声称所有引用均已核实；本批636个锚点由独立[精确目标验证](link-correction-20261114-01/verification.json)补验。

14标准golden和一份2149字节旧NGSA107的[核验范围](golden-and-old-archive-final.json)不代表全部历史档；5技能13分支的[上游指定投影](runtime-comparison.md)不代表全轨迹/整技能等价。其余来源、UI、Spine、0/37签署、设备/签名与第六阶段见[剩余范围](remaining-scope.md)。

[提交前冻结核验](precommit-frozen-protection.json)：32259输入无漂移、672原未跟踪路径仍在、8个旧工作树登记相同；[生成输出处理](build-output-restoration.json)与[私有缓存退休](owned-cache-and-browser-cleanup.json)／[最后build副本退休](final-private-build-cleanup.json)不触碰共享target或旧工作树。


## 2026-11-14T01:00:26.252972+08:00 补充暂存差异检查

完整 `git diff --cached --check` 为exit2，48个文件全部在本轮原始输出、源码/差异快照或带日期审查表中（空行/行末空格）；[首次输出](25-staged-diff-check-first.log)和[逐文件范围](staged-whitespace-scope.json)保留，不把全暂存检查写成通过。保留原证据字节，不改历史审查表来消掉提示。6产品文件、19公共主稿及inventory的明确范围检查exit0，见[实际命令](26-staged-owned-check.json)。第五节必跑命令仍全部exit0；本补充结果不作为整仓空白清洁声明。


## 2026-11-14T01:33:42+08:00 发布与精确代码CI补充

代码 `1fa8ac6cc3a70446436d97ff49aa19ea4626e22d` 已按裁决25正常发布；[精确SHA CI](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/43468568765) attempt1全部10作业success／0失败／0取消，未由agent补跑。记录时HEAD为该代码加文档/证据dirty；规则333／NGSA107／schema5／ABI759／WS8。 见[逐作业计数/工具链/内部尝试/原始日志](publication/code-exact-ci-first/README.md)。本地门禁表仍为提交前的原日期记录；额外全暂存空白检查exit2及保留原证据的范围不改写为通过。
