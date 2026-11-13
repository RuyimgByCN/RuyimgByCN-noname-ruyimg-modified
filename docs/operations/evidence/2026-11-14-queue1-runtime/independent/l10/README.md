# L10 独立复核（2026-11-14）

本批退回，不新增「已复核等价」。原始 26 个唯一 case 已逐案核对，8 案存在已用牌区或 HP 的运行反例；声明取消、目标退出、类型取消另有具体差别。党锢三场真正进入原始 `k_danggu` step 0 后被 `dui` 异常阻断，未执行伙伴 A／B／取消。正式登记后的独立续行又复现常侍技能自动结算差异。

复核人为 `/root/repair_doc_refs`。采样与本轮复跑源码 HEAD 为 `605201f3f49e10cea61c31d52a61bbb66c85599d`，工作区 dirty；产品 dirty 为其他任务拥有的酒诗恢复定义及对应测试。规则／NGSA／schema／ABI／WS 为 333／107／5／759／8。本轮只新增本目录文件，未改产品、清单、公共文档、Git index 或 refs。当前逐文件输入哈希及 dirty 记录见 [初始输入](inputs-initial.json)、[正式复跑输入](formal-input-freeze.json)、[第二版输入](formal-input-freeze-02.json)。

主要结果：

- [逐案表](case-table.md) 与 [逐案结构化判断](case-assessments.json)：26 案分别记录实际入口、选择、取消方法、结算／挂起落点及窄范围判断。
- [具体发现与边界](findings.json)：12 项具名发现、5 项取样／载入／UI 边界；[源码逐项核对](source-review.md) 包括党锢 UI 失败后的逐行替代范围。
- [原始字段重算](case-recalculation.json)、[重算摘要](summary-recalculation.json)、[实际候选集合](candidate-recalculation.json)。不能用相同 HP、候选显示或目标分支观察标志代替规则等价。
- [CDP 精确索引](cdp-provenance.json)：595 对请求／回复按 `at` 配对；最终 26 案取每案最后一段，起始 setup 为原始 JSONL 第 145 行。初态、所有阶段快照、实际点击与终态均精确匹配原始回复。[三场原始 UI 异常](k-ui-blockers-raw.json) 单列完整 CDP 回复和实际触发堆栈。
- [正式登记与消费复跑](formal-path-review.md)：bridge-only、正式单包登记及选伙伴后的正常阶段推进分列；真实授技与错误自动消费分别说明。

原始输入来自 [native-round2/native-results.json](../../L10/native-round2/native-results.json)、[browser-round4/results.json](../../L10/browser-round4/results.json)、[CDP requests](../../L10/session-l10-1794593358863-requests.jsonl)／[replies](../../L10/session-l10-1794593358863-replies.jsonl)、实际 probe／runner 及上游／原生源码。没有读取作者 README、结论、比较表或比较器。对其冻结 manifest 的完整性检查只计算字节哈希，不解释这些说明文件。

初态比较只覆盖四座 HP／max HP／朝向、实体 h／e／j、初始主技能名和保留的 7／8 张牌堆之 ID／name／suit／rank／nature。每案有 21 个未映射的原生初始弃牌实体未复制；另闪电 77／78 的导出性质为原生 null、上游默认 thunder，原可用初始弃牌也不能称完全相同。首次重算因该差别断言失败，保留 [首版脚本](recompute-first-source.py) 与 [首次失败](recompute-first.stderr.log)，随后按明确投影重算，完整初态差别仍保留。各自初始弃牌在本轮均保持原位／原牌面；主终态比较排除这部分基线，未将性质表示差别断言为闪电规则错误。

浏览器原始 `upstream_completed` 只表示目标分支观察；14 案根事件 `done`，12 案未到根事件结束，其中包含三场 UI 阻塞和继续停在正常出牌菜单的案子。原生 `g_cancel_target`、`r_cancel_type`、`k_cancel` 终态仍待答。`k_after_initial_no_trigger`／`k_no_skill` 的两端阶段终点不同，不把正常摸牌差别记成党锢规则反例。

实际验证与结果：

| 执行 | 结果与原始记录 |
| --- | --- |
| 保留原生 binary 独立重放 26 案 | exit 0；237 个原始产物逐字节一致；[命令／二进制指纹](native-replay01-command.json)、[stdout](native-replay01.stdout.log)、[文件核对](native-replay01-file-check.json) |
| `python3 -B docs/operations/evidence/2026-11-14-queue1-runtime/independent/l10/recompute.py` | 首次 exit 1，初始弃牌性质差别未被静默归一；第二次 exit 0；[第二次 stdout](recompute-second.stdout.log)、[归档逐项重算](archive-recalculation.json) |
| 102 份原始待答 NGSA 与独立重放 | 51 direct＋51 restored；全部恢复时 state hash 相同、完整快照仅 auth_generation 加 1，26 案命令／采集终态相同，最终 Debug 仅 auth_generation 不同。102 次恢复后 NGSA 字节均变化，不能写“恢复字节相同”；7 个无待答案的 direct/restored 最终档字节相同 |
| 现有正式登记函数复跑原 26 案 | exit 0；4 host＋10 VM，目录真实授技；另查 102 份待答恢复；[正式原档](formal-run01/native-results.json)、[重算](formal-summary-recalculation.json) |
| 正式 A／B 选伙伴后各推进两个正常阶段 | exit 0；两个额外 continuation 的 direct/restored 命令／阶段／终态相同，另查 4 份待答恢复；实际自动火伤／弃牌／无杀鸱咽反例见 [原档](formal-consumption02/formal-followup-results.json) |
| `python3 -B docs/operations/evidence/2026-11-14-queue1-runtime/independent/l10/review-formal.py` | exit 0；474 份产品 Rust 源文件与复跑前哈希一致；[原始 stdout](formal-recalculation-first.stdout.log) |
| 本目录链接、行号、输入与冻结原档核验 | [校验结果](validation-first.json)、[原始输出](validation-first.log)、[校验脚本](validate.py) |

两个外部 probe 构建使用共享 target 正常排队，未清理共享 target，未创建私有 target。首编译继承原 probe 的 1 条未用导入 warning，原档保留；第二编译 0 warning。未另起独立 Chrome 重跑，未运行产品测试、工作区门禁或 preflight；全局门禁与发布由主 agent 负责。没有使用 stub，未读／复制／加载官方或上游打包 Spine 运行库；原始本地 HTTP 为 2,565 条、Spine URL 请求为 0。单独调用原始 `EpicFX.initSCSAnim` 的 `dcdAnim` 异常属于附加依赖诊断，不冒充原始党锢自动执行链。

本目录最终文件哈希见 `frozen-file-manifest.json`。原 L10、已冻结 U07 及其历史失败记录均保持原样；本批没有新增等价签署，尚未提交／发布。
