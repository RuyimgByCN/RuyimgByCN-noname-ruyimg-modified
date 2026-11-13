# 第 0 项阶段过滤登记表增量独立复核（2026-11-13）

- 复核人：`queue0_independent_review`；核验时间：2026-11-13 22:00 +08:00。
- HEAD：`3a6e398485db6ef0729abe3013c3c4b7705a5d6d` + tracked dirty；规则 333／NGSA 107／content schema 5／ABI 759／WS8。
- 输入为实际原生 filter／key 源码、两个 Python 脚本的完整内容与 diff、新旧 registry；未读主 agent 的本次说明文档。CodeGraph 工具未提供，按项目规则退回指定文件读取。只在独立证据目录写入，未修改生产、测试、登记表或 inventory，未占用 Cargo target，也未重新运行暗织 20 项。

本增量可接受：当前登记表确有 37 个扫描结果，新增 5 个阶段分支均能定位到既有 HEAD 源码；旧 32 行和历史文件保留，新增项全部为「未核」，没有新增等价认证。9 项轻量自测及登记表 `--check` 独立通过。本结论仅核对登记表与当前扫描器的一致性，不签署这些技能的上游规则等价，也不把扫描结果称为所有原生阶段技能的完整覆盖。

## 五个新增分支

以下均在 [filtering/common.rs](../../../../../crates/engine-native-content/src/content/host/filtering/common.rs)，该文件与 HEAD 字节完全相同。对应 key 已在内核 [keys.rs](../../../../../crates/engine-native-kernel/src/vocabulary/host/keys.rs) 登记。原始行文、git blame 与完整提交号见 [phase-registry-source-audit.txt](phase-registry-source-audit.txt)、[独立逐行比较](phase-registry-independent-comparison.json)。

| 行为／key | 新登记阶段 | 实际源码条件 | 既有来源提交 |
| --- | --- | --- | --- |
| BossChuanyun／boss_chuanyun | End／true（960 行） | 本人结束阶段开始，且有其他存活角色体力高于自己。 | `c47831ddbb38cb2ff000a5bab400abfa73fcb188` |
| BossTianyun／boss_tianyun | End／true（977 行） | 本人结束阶段开始，且有其他存活角色。 | `c47831ddbb38cb2ff000a5bab400abfa73fcb188` |
| DddXiaxing／dddxiaxing | Prepare／true（1003 行） | 本轮未使用、手牌或装备非空，且当前进入准备阶段的角色存活；同臂另有濒死分支，后者不是本表新增阶段行。 | `c47831ddbb38cb2ff000a5bab400abfa73fcb188` |
| KMowang／k_mowang | Prepare／true（1089 行） | 本人准备阶段开始且 k_mowang_resting>0。 | `0d5795f3bde907973a61d2f7594220b742afd174` |
| KMowang／k_mowang | End／false（1104 行） | 本人结束阶段结束；同臂另有濒死分支，后者不是阶段行。 | `0d5795f3bde907973a61d2f7594220b742afd174` |

因此新增 Prepare 两行、End 三行；其中 begin=true 四行、false 一行。最终分布为 Prepare 4／Judge 1／Draw 7／Play 8／Discard 3／End 14，共 37 行，begin=true 为 21。没有凭脚本自测期望值推断上述来源，已单独检查新增 tuple 所在实际匹配分支与 key。

## 历史与状态保留

[2026-10-19 历史 registry](../../2026-10-19-phase-audit/registry.txt) 为 32 个数据行，整个文件与 `git show HEAD:<path>` 字节相等，SHA256 `5bdc93d018484779bdd533ca9ef451860008315ee2942be50632e219da501292`。当前 [native-phase-filter-registry.tsv](../../../../architecture/native-phase-filter-registry.tsv) 为 37 行；剔除上表新增五行后，与历史 32 行按原顺序逐行相同，key、Phase、begin、状态、依据均未改，没有旧行删除。

| 状态 | 历史 32 行 | 当前 37 行 |
| --- | ---: | ---: |
| 已核 | 29 | 29 |
| 不适用 | 2 | 2 |
| 未核 | 1 | 6 |

五个新增行均为「未核」，依据列为空，已核数量没有增加。原有一行 SwdBaishouzhihuHeld 的空 key／未核也原样保留，本轮没有替它认证。当前表的逐字采集见 [phase-registry-current.tsv](phase-registry-current.tsv)。

## 脚本与实际运行

[report_phase_filters.py](../../../../../scripts/report_phase_filters.py) 的实际 diff 仅调整说明与 REG 目的路径，从旧日期证据表改到当前架构 TSV；扫描范围、段切片、正则与状态保留逻辑没有变化。其按 behavior 保存状态的既有方式在本轮两个 KMowang 行均未核的输入中不发生冲突。默认写入不再改历史文件；本 agent 只运行只读 `--check`。

[test_report_phase_filters.py](../../../../../scripts/test_report_phase_filters.py) 只同步旧快照数量与注释：32→37、17→21、Prepare 2→4、End 11→14、含表头 33→38；保留格式／状态列、Xiefang Draw 分支及段切片守卫。实际 diff 见 [phase-registry-reviewed.diff](phase-registry-reviewed.diff)。复核发现注释称「夏行」，源码实际为「侠行」，仅文字错字，已通知主 agent，不影响断言或登记；未由本 agent改动。

最终补记：主 agent 已将该注释更正为「侠行」。最初只读输出仍是错字，首次比较证据保存时已采到更正后的版本；最终脚本与首次比较证据中的脚本哈希直接相等，未重复运行测试。当前 report 脚本、新旧 registry 哈希均保持不变；最终核对见 [phase-registry-final-inputs.json](phase-registry-final-inputs.json)、[最终脚本 diff](phase-registry-reviewed-final.diff)。最初尝试反向替换错字重建旧哈希的采集检查不成立，已改以实际保存的哈希直接比较，不把该探针失败当作代码或自测失败。

| 仓库根目录实际命令 | 独立结果 | 证据 |
| --- | --- | --- |
| `python3 -B -m unittest discover -s scripts -p test_report_phase_filters.py -v` | 9 passed／0 failed／0 skipped，exit 0。实际是 unittest discover，包含 main 块后定义的 ArmSlicingTest。 | [完整原始输出](phase-registry-tests.raw.txt)、[命令／时刻](phase-registry-tests-command.json) |
| `python3 -B scripts/report_phase_filters.py --check` | `registry 一致（37 条）`，exit 0；执行前后新旧 registry 字节均未变。 | [完整原始输出](phase-registry-check.raw.txt) |

扫描器只看 `host/filtering*.rs` 和 `host/filtering/*.rs`、按 HostBehavior 段切片并匹配显式 Phase／begin 字面量。这是既有范围，没有因为这次表同步扩大为所有按名技能／helper／其他事件窗口的等价审计。未运行上游浏览器、未新增规则变更、未改版本、未更改任何来源签署或 inventory 状态；完整 preflight 首跑与补跑由主 agent 另记。
