# PA09单座池release负结果（2026-10-31）

[原交接](report.raw.txt)明确否定本负载稳定收益。两个优化release build与四个独立ABBA进程均成功；生产代码保持先前已审九文件原型，serial仍精确4dd。仅两私有树同一measurement测试文件增加同样的逐请求行与断言，无新getter、协议、选项、AI admission policy或Main修改。

[97份原文bundle](retained-raw.tar.gz)／[逐项manifest](manifest.json)保留92原index文件＋index／report／父parser／结果／输出日志；解压每项SHA已验证。[父独立parser](parent-independent-audit.py)／[结果](parent-independent-results.json)fresh核92原SHA、两实际release binary身份、test overlay、六真实命令，再逐行解析四run而非采信producer汇总。没有执行producer或新跑实验。

每run1687顺序请求=init1＋start6＋autoStep1680；一局warmup及五局测量、每局280步，全部请求坐标／ID／hash一致，测量1400步、完整trajectory803da246…一致。candidate每次625accepted jobs／135池提交步，serial0；计数是提交，不是执行或重叠证明，旧two-worker/index证明单独保留。

整体roundtrip p95（微秒）A0/B1为279.875/268.417，A3/B2为269.500/262.250；实际池135步对子集的p95为347.458/302.292与346.041/306.542。3／4／7-job各组owner p50/p95也均更慢。先核完整轨迹再配对，不把subset冒充整体，不从这些数据拟造有利阈值。首次pool创建可在0accepted步骤发生，冷首步、start替换及retirement原样在样本与附加生命周期行中；不推导硬抢占。

一次seed42、四座identity、64nodes／50ms、两进程每variant仅为有界pilot。release默认lto/codegen设置不改；没有真实Tauri ResourceTable／GUI／IPC／HID／绘制／平台／功耗、native-versus-WASM或wholePA09资格。原型继续私有，不合入Main、不切默认；后续优化须有新问题与有界测量依据，本轮未追加框架或继续盲跑。按裁决23已[退休两个自有target](target-retirement/report.raw.txt)，DU分配口径5,200,220,160B，两个测量binary已移到私有保留处并fresh核SHA／mode／mtime；旧日志原路径保持，后续使用[relocation映射](target-retirement/binary-relocation-manifest.json)，不能再声称旧target路径存在。父[有界复核](target-retirement/parent-independent-results.json)确认仅两个allowlist缺失、原before／after清单摘要相同；原进程全清单与可执行文件不发布。
