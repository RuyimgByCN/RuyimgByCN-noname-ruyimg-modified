# 首轮CI失败修正：本地验收与第二次真实CI

核验日期2026-10-30，基线`38ab1678df9b4b7b991d3b3122b1d937fd2648a1`（A330已本地验收）。本批规则330／archive105／ABI758／WS6保持。最初真实[run41053688083](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/41053688083)属于`9dbc655`规则329，结果5成功/5失败，原始记录和日志[继续保留](../2026-10-30-native-ci-preparation/README.md)。下面本地结果不回填那次失败。

## 修正范围与独立审查

[父接入清单](parent-source-admission.json)记录四组已复审补丁、13个维护路径，`native-ci.md`同时保留两组说明；未混入语音别名或NC10候选。

- [TCP/名册复审](tcp-roster-review.raw.txt)：单测使用真实受限TCP窗口证明慢观众积压、丢广播并断开；健康端继续、真实重连恢复投影/成员。恢复生产64队列/5秒超时，不以writer假延迟造绿。名册测试由恒真断言改为真实协议解码及成员核验。
- [profile复审](profile-review.raw.txt)：perf两条原命令加`--release`，与phase17旧线测量口径一致。工作量、预算、profile定义与所有矩阵/目标保持；单改profile的隔离151µs启动仍超100µs，原失败保留。
- [旧线目录缓存复审](legacy-cache-review.raw.txt)：仅逐字段等于规范空ContentRegistry时复用经原路径构建的不可变标准目录及摘要，Arc写入隔离，动态/非规范空目录仍走原合并。旧线仅修现有启动成本，不新增规则；隔离329七对同条件前后测量与当前330性能分别归属。
- [安装器复审](installer-review.raw.txt)：Windows保留中文名并使用已支持的WiX zh-CN/936；Linux技术包名noname-next，中文窗口及desktop名称保留，DEB/RPM/AppImage目标不删。配置/schema核验不能替代实际Windows/Linux构建安装。

原始候选补丁、红绿/诊断、源/二进制身份和独立review入口/出口证据在[复制索引](parent-isolated-copy.json)。CRLF原始文本按原字节gzip保存，以免Git行尾转换；必要可执行产物仍私有保留，不把日志身份冒作执行身份。

## 当前主树实际结果

[16条真实命令与统计](main-acceptance/commands.json)、[汇总](main-acceptance/summary.json)、[完整交接](main-functional-handoff.raw.txt)记录本轮全部exit0。workspace6894/0/77、frontend163/163、typecheck、workspace all-targets、正式Worker82/82、sourcecaps自测2/2/8989文件、layering0通过；warning与ignored保留，782条warning header含重复/汇总，不能称782个独立问题或无warning。native3严格输入仍是A330已验收版本，本轮未重复或扩大原CI门槛。

真实TCP send/recv=1024/1024，301命令，慢观众仅178结果后断开，健康端301结果，实际重连投影/成员核验通过；这是macOS本地实测；后续Linux同一测试已在下述新CI执行通过。当前main release预算结果为启动51/100µs、距离0/300µs、恢复46/200µs、投影20/100µs，全PASS；0µs是当前计时分辨率下的记录，不表示零成本。未修改负载/阈值，也不宣称原生内核收益。

当前legacy生产WASM按维护脚本重建，SHA `2b1cc44e511484c804f10640429b32075fa525227f5d07468bdf5f9dc1fab355`；native330保持 `2292d7ab1c04b05724e49eb750962bb78af119dac80011524b18574742ed189b`。旧12生成输入原件保留，public镜像一致；conformance、旧线crosshash/manifest/skillVM和普通40步/包20步恢复轨迹通过。仓外Vite包含两份实际WASM，未写默认dist或共享node_modules，未将构建上调成新UI平台运行通过。

13维护SHA、A剩余50项、prepare3、资源catalog/manifest、冻结输入、guard、HEAD/index和tracked dist首末均不变。stage3按显式lease保留当前CI复验用途，inc0/jobs2及全局串行锁生效，每编译核盘；本轮未退休仍需使用的缓存。

最终[四项门禁](final-gates/index.json)全部exit0，完整[preflight21/21](final-gates/04-full-preflight.log)（锁内314.934秒）、strict行号1065引用/越界0、8989文件行数与自测2/2通过，六类missing-file提示保留。最终[身份与命令汇总](final-gates/summary.json)无源码/保护输入漂移；[独立组合复审](main-combined-independent-review.raw.txt)无新P1/P2、OK with notes，其要求的完整preflight条件随后已满足。原复审时pending记录按时间保留。本批已按裁决18入库`da7e4c019461a6c467a9dd2de849f3b1a7494b86`，按裁决25发布；后续真实CI结果见下，实际安装／签名／公证／真机运行仍未验收。用户同时批准的C331 schema4增量仅登记设计授权，不表示331已在本树实施。WASM默认保留，整个Goal未完成。

## 第二次发布与真实CI（2026-10-30，独立登记通过）

一次正常、非force、限定命令钩子的HTTPS push将同一`native-convergence`由`9dbc655`推进到已验收`da7e4c01`，包含A330父提交`38ab1678`；远端main保持`47347c6608d651660ae9459e59eb529b3cef277f`。原local-only钩子及持久配置不变。[实际执行结果](publication/increment-execution/result.json)记录exit0；原首次发布拒绝和run41053688083的5成功/5失败继续保留。

新push事件[run41082599800](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/41082599800)精确绑定`da7e4c01`、attempt1，10任务全部success：native-web、gates-ubuntu、两OS perf、三OS build及三OS desktop。native-web Rust1815/0/75、Worker82、前端163；gates Rust6894/0/77、conformance19、crosshash与完整原资源verify通过。三平台build各838个library tests通过，计数按各job保留、不相加作唯一测试数；Win/Mac仅Linux依赖安装step跳过，没有job跳过。watch实际exit0，API结论不是逐step数字退出码。

| release指标（µs） | 原预算 | Ubuntu | macOS |
| --- | ---: | ---: | ---: |
| start-hash-p50 | 100 | 72 | 74 |
| distance-query-p95 | 300 | 0 | 1 |
| snapshot-restore | 200 | 60 | 141 |
| observer-projection | 100 | 29 | 28 |

两份hosted采样均PASS，不构成同设备前后收益。三份安装器已上传：Windows12992007422／26273163B，Ubuntu12991606067／266337379B，macOS12990234641／32802348B；metadata和上传日志绑定run/head/digest，未下载或安装二进制。

[独立登记](second-publication-ci/independent-ci-registration.raw.txt)交叉核对原API、十份job日志、发布记录及父纯读解读。[原始复制清单](second-publication-ci/parent-copy-manifest.json)保全44文件；Windows4051/3821个CRLF以gzip原字节保留，原始manifest的43项bytes/SHA全部匹配，[独立结果](second-publication-ci/parent-independent/independent-results.json)保留具体计数。[完整发布交接](second-publication-ci/publication-handoff.raw.txt)记录981objects／354deltas／Git显示17.41MiB（非精确wire bytes），锁内70.089秒与总153.303秒；差额包括排队及wrapper开销，未独立测量queue。

新语音别名或后续源码改动在发布lease释放后发生，不属于此CI快照。CI十绿和安装器上传不证明实际安装、签名／公证、原生默认切换、真机或37来源验收。
