# 第 0 项最终本地门禁

核验：2026-11-13T22:21:49+08:00；源码 HEAD `3a6e398485db6ef0729abe3013c3c4b7705a5d6d` + tracked dirty；规则333／NGSA107／schema5／ABI759／WS8。本表仅表示本地验证，尚未提交、发布或取得本批CI。

| 原始证据 | 实际命令 | 最终结果 |
| --- | --- | --- |
| [02-line-check-tests](02-line-check-tests.log) | `python3 -B -m unittest discover -s scripts -p test_check_source_lines.py` | exit 0 |
| [03b-source-lines-final](03b-source-lines-final.log) | `python3 -B scripts/check-source-lines.py` | exit 0 |
| [04d-native-three-complete-final](04d-native-three-complete-final.log) | `cargo test --offline -p engine-native-kernel -p engine-native-content -p engine-native` | 2442通过／0失败／6忽略 |
| [05c-strict-all-targets-final](05c-strict-all-targets-final.log) | `cargo check -p engine-native-kernel -p engine-native-content -p engine-native -p engine-native-wasm -p content-native-loader --all-targets` | exit 0 |
| [06b-wasm-check-rustup](06b-wasm-check-rustup.log) | `cargo check --target wasm32-unknown-unknown -p engine-native-wasm` | exit 0 |
| [07-workspace-complete](07-workspace-complete.log) | `cargo test --offline --workspace --no-fail-fast` | 7838通过／0失败／8忽略 |
| [08-conformance](08-conformance.log) | `cargo run -p conformance` | exit 0 |
| [10d-frontend-tests](10d-frontend-tests.log) | `pnpm test` | exit 0 |
| [11-frontend-typecheck](11-frontend-typecheck.log) | `pnpm typecheck` | exit 0 |
| [12-build-native](12-build-native.log) | `pnpm build:native` | exit 0 |
| [13-smoke-native](13-smoke-native.log) | `pnpm smoke:native` | 83通过／0失败（装载冒烟不代表37来源整包签署） |
| [14-native-wasm-cross-target](14-native-wasm-cross-target.log) | `python3 -B scripts/check-native-wasm-cross-target.py` | 3通过／0失败／0忽略 |
| [15-cross-target-hash](15-cross-target-hash.log) | `bash scripts/cross-target-hash.sh` | exit 0 |
| [16-desktop-tests](16-desktop-tests.log) | `cargo test -p desktop-client` | 20通过／0失败／1忽略 |
| [17-frontend-build](17-frontend-build.log) | `pnpm build` | exit 0 |
| [22b-preflight-frozen-v4](22b-preflight-frozen-v4.log) | `python3 -B scripts/preflight.py` | 21项全部通过；无quick或过滤 |

每项同名JSON保存实际argv、cwd、env、版本起点、dirty、时刻及exit；[机器汇总](gate-summary.json)。严格all-targets使用独立CARGO_TARGET_DIR和RUSTFLAGS=-D warnings，原生／新增测试rustc1.99.1；WASM检查和实际构建用已装目标的rustup1.95.0，仅命令级PATH覆盖，见[工具链边界](wasm-toolchain-before.json)。旧执行线conformance与cross-target单独读取，不作为原生全产品等价证明。

## 首次失败与补跑

- 前端旧提示词文件、三crate旧Adv21缺陷诊断、WASM缺目标、preflight链接／阶段表／脚本断言失败均保留。preflight首跑18为3项红，19中间诊断仍有2个脚本失败；21运行期间工具变化，只作中间证据。最终冻结采用22b，不能把前面的运行改写为全绿。
- phase当前37行只新增5个“未核”分支，旧日期32行原件保留；once注释反例首轮退回与修后6项通过、Damage酒诗旧实现88→89的因果均由[独立增量复核](independent-review/once-event-incremental-review.md)确认，没有提高等价项。

## 兼容与范围

- [14场golden及一份旧档](golden-and-old-archive.json)：golden源码／期望与起点相同，无重录；旧NGSA107为2149字节，decode／encode、原内容锁、完整快照与续行通过，不泛称全部历史档。
- 原生WASM跨端覆盖标准40步及20+20恢复、正式欢乐包20步及10+10恢复；hash、revision和event一致，见[原始输出](14-native-wasm-cross-target.log)。
- [独立逐分支表](independent-review/final-report.md)含暗织20项独立真实事件回归、U01安装64/0名称核对及3136提示词完整集合。上游浏览器实际触发0；[各来源计数](source-counts.md)为849／3575／0，没有上调。
- 暗织Prepare及暗织／Fz选序、桃园未含自身、实体Duel0仍为登记差异；后续源重审、真实上游触发、UI并排、Spine逐特性／净室／用户验证、真机／签名及第六阶段仍待，见[剩余范围](remaining-scope.md)。

2026-11-13T22:22:24+08:00 汇总补充：前端原始Node spec输出为489通过／0失败／0跳过；首版机器汇总只识别TAP前缀，故该字段为空。补齐spec格式解析的[汇总v2](gate-summary-v2.json)保留首版，不改变任何执行结果。
