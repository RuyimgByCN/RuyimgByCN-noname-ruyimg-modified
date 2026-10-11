# T20：gen_cardpile_packages 拆分为 cardpile 专用生成器（2026-10-11）

## 背景与选择

T15① 实证该示例整跑会连带重写 huanle/d3/gujian/gwent/hearth/yunchou/wudian/wuxing 八个手工维护的转换包，且生成器数据落后于手工包数据（重跑曾覆盖丢失 huanle 酒尾，16 测试失败，见 [2026-10-11-t15-abi-reconcile](../2026-10-11-t15-abi-reconcile/README.md)）。

修复按任务表两个选项中的「拆分」路线落地：**更新生成器至与八包手工数据一致**意味着把约 790 行手工包数据（cards/character/config JSON + npack 构建函数）反向抄回生成器，之后每次手工改包都要双写，漂移面不减反增；八包均无字节守卫消费方（守卫只覆盖 cardpile 双包），抄回无可验收收益。故裁剪为 cardpile 双包专用生成器，八包转为纯手工维护。

## 改动

- `crates/content-runtime/examples/gen_cardpile_packages.rs`：1140 行 → 181 行。删除 huanle/wuxing/d3/wudian/yunchou/gujian/gwent/hearth 八个生成段及其数据常量与 build 函数；保留 `ENTRIES`/`TOTAL`/`pile_json`/`build_package` 与 cardpile 双包写入循环（逐字节不变，输出与删除前一致）。文件头横幅由「禁止运行」改为「范围收紧」说明：仅 cardpile 双包可重生成，其余八包禁止由本示例生成。

## 验证

| 项 | 命令 | 结果 |
|---|---|---|
| 生成范围收紧 | `cargo run --offline -p content-runtime --example gen_cardpile_packages` | 仅打印 cardpile-upstream-noname / cardpile-upstream-liuli 两个写入点；`git status` 显示八包目录零改动 |
| 双包字节稳定 | 同上 + `git status --porcelain` | 双包产物零 diff（重生成=恒等） |
| 字节守卫 | `cargo test --offline -p engine-native --test g1_cardpile` | 5 passed / 0 failed（含 converted_packages_match_generator 双包断言） |
| content-runtime 全量 | `cargo test --offline -p content-runtime` | 11+5+7+1+1+5+1+1+1 = 33 passed / 0 failed |
| 八包回归 | `cargo test --offline -p engine-native --test g1_huanle --test g1_d3 --test g1_hearth --test g1_shahai_gujian --test g1_shahai_gwent --test g1_shahai_yunchou --test g1_wudian --test g1_wuxing` | 4/8/19+1i/14/7/13/117/4 全部 0 failed |

## 与任务表验收的对应

- 生成器与手工数据一致 → 以「仅 cardpile 双包可重跑且字节恒等」形态满足（cardpile 双包本就由生成器权威生成，守卫双包 2/2 通过）。
- 各包防漂移守卫 → cardpile 双包已有 `converted_packages_match_generator`（g1_cardpile.rs，守卫测试自带独立 `build_package` 副本，不受示例裁剪影响）。八包为手工维护、无生成器可对照，字节守卫无从建立；其防漂移由各自 g1_* 行为回归测试承担（上方八面全绿即当前防漂移证据）。为手工包补生成器+守卫属新增能力，不构成本任务遗留缺口，如需要应另立任务。
