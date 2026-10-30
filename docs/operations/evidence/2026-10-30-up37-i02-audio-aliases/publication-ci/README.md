# 声明语音别名发布后的真实 CI（2026-10-30）

验收对象为已发布提交 `a1f7250815eee803d931301ab46207719cb9a15b`，规则330／archive105／ABI758／WS6、资源schema2。真实 [run41100475767](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/41100475767)，push事件、native-convergence、attempt1，最终 `completed/success`、十个job全部成功。该运行不覆盖随后接入但尚未验收的compat14增量。

## 发布与观察

[发布及最初观察交接](publication-and-initial-transport-handoff.raw.txt)保留精确限定push、原hook不变、远端main不变的证据。三次watch均exit1／HTTP502，保持原结果；没有第四次watch、重新push、CI rerun或dispatch。后续成功由只读API与完整日志确认，不记成watch exit0。

run最终更新时间为2026-10-30T18:06:33Z；父独立API采集18:15:57Z；最终收集18:18:20–18:19:23Z，补充step读取至18:23:20Z。[最终交接](completed-ci-handoff.raw.txt)与[独立登记](independent-registration.raw.txt)区分各次观察时刻。旧queued/in_progress快照、502 stderr与退出码均保留。

| Job | 实际ID | 结果 |
| --- | ---: | --- |
| build-ubuntu-latest | 124216768563 | success |
| desktop-ubuntu-latest | 124216768677 | success |
| perf-ubuntu-latest | 124216768689 | success |
| native-web | 124216768707 | success |
| build-macos-latest | 124216768715 | success |
| gates-ubuntu | 124216768734 | success |
| desktop-macos-latest | 124216768760 | success |
| build-windows-latest | 124216768777 | success |
| desktop-windows-latest | 124216768968 | success |
| perf-macos-latest | 124216769050 | success |

## 实际验证范围

日志给出workspace6894/0/77、native回归1815/0/75、每OS库矩阵838/0/0（顺序passed/failed/ignored）、Worker82、前端167/0/0。各范围重叠，不相加为独立测试数。19项conformance、跨目标摘要、Gate4 Node2/2、完整资源verify（legacy366／画像2707／音频9652）通过；五个资源准备job均记录同步、verify及tracked content无差异检查，四个声明别名前端场景通过。真实浏览器／音频验收仍引用[主树别名验收](../README.md)。

旧线server-baseline预算为100/300/200/100微秒；Ubuntu实际48/0/45/19，macOS68/0/59/27，两份tier报告PASS。零读数受计时分辨率限制，hosted测量不能证明同设备性能收益或native引擎收益。

| 安装器 | artifact ID | API字节 | API SHA256 |
| --- | ---: | ---: | --- |
| macOS | 12998118328 | 32802940 | `add7868181f107bc5ff3c2d80e05df7e78816d8b876de54c4b3203fadf35161f` |
| Ubuntu | 12997584073 | 266340325 | `4ac4d2e21fd712f1f73aee7887bfb3c7761c9325767fda3323d6b4b7c5a3bf06` |
| Windows | 12997069742 | 26272381 | `4b3e9a8044bd475ed7a3766c9a78e17cb0846d4a0325939b90472d944fde8c4a` |

三份artifact采集时未过期且绑定该run/head；没有下载、安装、签名、公证或真机验证。

## 证据与限制

[原收集清单](raw/final-stage/copy-manifest.json)覆盖146份其他文件，共147文件含清单自身。[父独立解析器](parent-independent/audit-frozen-evidence.py)及[结果](parent-independent/independent-results.json)核过146项bytes/SHA、十个job详情与日志、Rust结果和三artifact绑定。独立review另定向核原API及关键日志，结论OK with notes，没有声称全文重新审读全部日志。

[归档映射](archive-mapping.json)逐项保留来源、原字节SHA与存储SHA。两份Windows原日志以gzip封存原始CRLF，解压后即原文件，未归一文本。十份原日志总4,721,558字节；Windows build/desktop的CRLF分别3821/2212。上游原manifest保留其原始相对路径；归档映射给出对应的gzip路径。

3628条warning日志行含重复compiler summaries，不是3628个独立问题；ignored测试与两处Linux依赖skip保留。125条合并step记录仍有18个null conclusion（native-web1／gates9／macOS desktop6／Windows desktop2），不得以job success推填。父解析器的初次jobs快照计数也不替代合并step口径。

本登记不覆盖旧da7e十绿、首次9dbc五红五绿之外的新历史，也不证明当前compat14、331、NC10、PA09六条件或全部37来源完成。WASM默认保持。
