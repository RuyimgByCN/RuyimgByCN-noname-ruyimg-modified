# 2026-11-14T02:25:12+08:00 本批验证边界（完整preflight仍在运行）

HEAD `605201f3f49e10cea61c31d52a61bbb66c85599d` + tracked dirty；333／107／5／759／WS8。完整代码命令与环境、输出hash和每次失败见[机器表](required-code-gates.json)。三crate2447通过／0失败／6忽略、workspace7843／0／8，均无过滤；desktop20／0／1与前者有重叠，不能相加当独立覆盖。严格5包all-targets在私有target、RUSTFLAGS=-D warnings、native rustc1.99.1通过；WASM使用本机已安装的rustup1.95.0，上一份CI实际stable1.100.0另记。

前端489／0／0、typecheck、native构建、Worker冒烟83／0、native/WASM真实轨迹比较、conformance、旧线cross-target及生产构建均exit0。cross-target比较内部定向Rust3通过、140过滤是该比较器原定场景范围，不冒充完整workspace或三crate。未改UI/Worker/渲染源码，本批未重跑原生牌桌截图路径；实际上游Chrome是规则采样，不代替原生UI验收。

6项原生忽略与desktop1项壁钟测量由完整日志记录；workspace另有旧线忽略，完整命令总8忽略，不能写成0忽略。旧engine-core/旧测试的既有warnings与构建chunk提示保留；严格无警告结论只覆盖指定5包all-targets检查，不说全仓无warning。14标准golden聚合回归与一份旧U01 NGSA107通过且输入不变，见[golden与旧档](golden-and-old-archive.json)。L29独立before/after68旧档与136续行见[独立报告](../independent/u06-l29/review.md)。

首轮新增测试因CommandOutcome无PartialEq编译失败；修正测试后真实red为OpenChoiceMismatch，再做产品修复。env/cwd放错位置导致3次runner启动失败（未启动检查程序），已分开保留；U06诊断比较器漏duel别名触发初态断言，也已保留。完整代码门禁首次实际执行均exit0；本文件不预称完整preflight或新代码CI成功。

生产构建确实改写已跟踪dist，先完整备份/记录hash再恢复37原tracked文件，并仅回收本次12个精确输出名。64,674,147字节私有build副本在逐文件校验后退休；原native-pkg保留供运行。U06自有Chrome/profile、私有probe crate/target及已完成strict target已退休；U07/L10自有缓存由各作者另记，共享target未清，现代Vite/FS服务为后续现代来源保留。
