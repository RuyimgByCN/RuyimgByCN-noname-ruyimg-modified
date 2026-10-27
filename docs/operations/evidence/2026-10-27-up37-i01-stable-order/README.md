# UP37-I01 内容同步排序：跨区域复现与修复

- 执行时间：2026-10-27 21:57:42 → 22:04:57 +0800（开始与结束各运行 `date '+%F %T %z'`）；源码基准 `33cf3371d4a398c58eb78961473b18e75a68d7a2`（开工时 `git rev-parse HEAD`）；规则 v322、存档 v105 未改。
- 范围：修复同步脚本的排序切片；不是 I01 全量完成。开工时另有 8 个已修改的文档文件，保留其在途改动。

## 反例与修复

开工前真实工作树在英文区域 `--verify-only` 报 `asset manifest entries mismatch`、中文区域通过（[英文](before-en.log)、[中文](before-zh.log)）。使用 `git archive HEAD` 导出干净源码，独立 `CARGO_TARGET_DIR`、只读借用原上游与画像数据，英文区域复现同一错误（[干净 HEAD](clean-head-en.log)）。首次临时导出经软链访问上游时触发 Git 的仓库外路径错误（[记录](clean-head-symlink-setup-error.log)），改在工作区同级导出后才复现上述排序错误。

`Intl.Collator("und")` 在本机 Node 20/26 会回退到进程默认区域，因此不满足本切片要求；[实测](collator-locales.log)显示中英文排序反转。文件遍历采用区域无关的 UTF-16 码元顺序，目录显示索引使用显式 `en-US` 排序及码元平局判定，以保持现有角色/技能列表顺序。目录显示索引仍受 Node 所带 ICU 版本影响，不能声称任意 ICU 版本间完全相同。

已核对原有 **2,582** 角色、**6,209** 技能的目录顺序不变（[核对](catalog-order.log)）。画像清单按实际文件遍历重排：**2,707** 条，**188** 个位置变化；逐条 path、size、source 和 SHA-256 及清单其余元数据完全相同（[记录](manifest-reorder.log)）。未更改规则、存档或状态摘要；未重录 golden。可选皮肤的完整真实来源未接入本次集成验证。

## 验证

| 命令 | 结果 |
|---|---|
| `node --test scripts/test_original_content_order.mjs`；`/usr/local/bin/node --test scripts/test_original_content_order.mjs` | Node 26.12.1 / 20.17.0 各 **1/0/0**；子进程覆盖 en-US、zh-CN、tr-TR、嵌套路径和 Unicode 平局（[26](node26-order-test.log)、[20](node20-order-test.log)） |
| Node 26 `--verify-only`，`LANG`/`LC_ALL` 为 `en_US.UTF-8`、`zh_CN.UTF-8`、`tr_TR.UTF-8` | 各 exit 0；366 源码文件、2,707 画像（[en](after-node26-en.log)、[zh](after-node26-zh.log)、[tr](after-node26-tr.log)） |
| Node 20 `--verify-only`，`LANG`/`LC_ALL` 为 `en_US.UTF-8`、`zh_CN.UTF-8` | 各 exit 0；数量同上（[en](after-node20-en.log)、[zh](after-node20-zh.log)） |
| `pnpm --dir apps/web-client test`；`pnpm --dir apps/web-client typecheck` | **80/0/0**；typecheck exit 0（[测试](frontend-test.log)、[类型](frontend-typecheck.log)） |
| `cargo test --offline -p engine-native-wasm -p content-native-loader -p game-server --no-fail-fast` | exit 0：wasm **52/0/0**、loader **32/0/0**、game-server **358/0/0**；编译有非致命警告（[完整日志](product-crates.log)） |
| `python3 -B -m unittest discover -s scripts -p 'test_check_source_lines.py'` | **1/0**（[日志](source-line-tests.log)） |
| `python3 -B scripts/check-source-lines.py` | **8621** 文件通过（[日志](source-lines.log)） |
| `python3 -B scripts/check_local_links.py --all` | **1,825** 文件、0 坏链（[日志](local-links.log)） |
| `python3 -B scripts/check-adr-index.py` | exit 0；未登记冲突、过时登记、登记内容不符和索引坏链均 0（[日志](adr-index.log)） |

## 遗留

I01 的分包差异审计和旧局内容锁反例仍待完成；可选皮肤完整来源的生成／验证链未在本轮跑通。本轮没有改变原生内核或旧线规则，未将脚本排序测试记作上游等价或完整产品验收。
