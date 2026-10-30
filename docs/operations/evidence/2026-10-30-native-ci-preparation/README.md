# 原生CI入口准备与外部发布边界（2026-10-30）

**首次发布事实（2026-10-30）**：本地入口验收提交`9dbc65530feb695688d3160dc38b6cf64ad39ecd`已依裁决25推送到现有公开仓库的`native-convergence`，远端main仍为`47347c66`。实际[Actions run41053688083](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/41053688083)已完成，conclusion=failure，10个job为5成功/5失败：native-web、三OS build、macOS desktop成功；gates慢TCP断开、两perf预算、Windows MSI1252码页、Linux RPM中文技术名失败。仅macOS安装器artifact已上传，32,791,200字节；不据此登记其他平台产品完成。最终原始[run](first-publication/later-run.json)、[10job](first-publication/later-jobs.json)和[artifact](first-publication/later-artifacts.json)均保留，失败原日志见同目录。第一次无条件旧local-only钩子拒绝保留；用户新授权后的命令级精确仓库/SHA/新分支检查通过，原钩子及持久配置不变。实际Git进度写入921.26MiB pack，精确网络字节未测；后续API的HTTP2 EOF通过单次HTTP1只读查询恢复，不当CI失败。见[first-publication原始证据](first-publication/decision25-scoped-execution/result.json)与[真实run响应](first-publication/later-http1-runs.json)。

**准备阶段记录**：四文件配置及输入修复独立复审通过，接入`977cdf1f`后的本地主树，完整preflight新一次21/21、exit0、310.023s。该准备检查当时尚未push或实跑Actions；下面旧记录按当时语境保留，不替代后续平台结果。 本批不改变329/105/758/WS5及WASM默认，不拿本地准备当平台验收。

## 问题与修正

fresh checkout没有web TypeScript依赖、`../archive/upstream/noname`及ignored资源，原完整Gate4无法通过；完整原生集成还读取12份Liuli文本，Gate3亦需匹配的WASM构建工具。四文件补丁保留5类job/10次矩阵和全部旧门禁：

- 原生checkout放在workspace/noname-next，与固定提交的上游checkout形成采用的兄弟目录拓扑，修正所有cwd/cache/artifact路径。
- core采用`1a548c95acdf8ac1ef44d74473403eda2ac86cf0`，在CI新checkout将已有origin绑定到采用的`.git` URL，不改本地canonical仓或provenance期望。
- gates/native-web固定取得Liuli提交`8485dbb29ff16f9b258a766b1a17ef75f883aade`的12份实际读取文本。仅在新CI副本应用真实已采用的龙血文本差异，pre/post SHA双校验；七份canonical修改不动，也不将其它旧HEAD文本冒称全语义等价。
- pnpm9 frozen安装后只恢复ignored资源；不生成catalog/ported/frozen，tracked内容必须保持clean。Gate4仍完整verify；Gate3安装现有wasm-pack0.15.0以避免fallback0.2.100与lockfile0.2.126不配。

原三P1和发布说明P2、各自定向red/green及所有初次失败见[初审](independent-initial-review.raw.txt)、[纠正交接](correction-handoff.raw.txt)及[最终复审](independent-final-review.raw.txt)。原误表“push清单排除tracked dist/evidence”已明确撤回，历史交接原文保留。

## 本地验证与权限

隔离真实小Git/sparse fixture验证origin元数据、12文本依赖、龙血差异与错误preimage拒绝；真实pnpm9 frozen安装及完整资源verify通过，17031保护文件字节一致。进程/文件系统桩只验证WASM调用路由，不当作安装或编译通过。主树另跑pipeline2项、resource verify、tracked content guard均exit0，见[主树入口检查](parent-current-main-bootstrap-checks.json)。完整preflight首次20/21失败的根因是外部README相对链接；副本保持原字节改存raw.txt后，新一次完整21/21通过，见[最终结果](parent-recovered-local-validation.json)、[原失败](parent-final-local-validation.json)及[原字节路由](parent-raw-readme-routing.json)。首次worker900秒超时包括577秒锁排队，命令实际结果不据超时改成通过；见[超时保留](parent-validator-timeout.json)。本机Node26、actionlint未安装，远程checkout、Node20及三平台打包仍待真实Actions。

core/main index二进制stat缓存观察曾变化，logical index仍等于HEAD；首次保护断言失败与后续观察保留，不擅改或恢复他人的Git元数据。主树6个未请求自动style改动已完整备份并精确恢复到验收字节，见[保存记录](parent-unexpected-auto-edits/preserved-before-restoration.json)和[精确恢复](parent-unexpected-auto-edits/restoration-exact-accepted-head.json)，没有合入这些改动。

外部目标是现有公开仓库`RuyimgByCN/RuyimgByCN-noname-ruyimg-modified`的新`native-convergence`分支。准备时远端仅main、无原生源码，已完成具体树/历史审计再取得用户[裁决25明确授权](parent-user-publication-decision25.json)：本次及后续同仓同分支的本地验收增量可持续push及运行CI；原来裁决18仅本地的边界不再阻止此目标。仍不force、不新增remote、不改default main、不合并、不创建PR/release/comment、不改持久设置/secrets。授权不改变独立验收、保留失败及未授权新协议/版本须另报的要求。

普通push包含完整tracked tree及远端缺失的必要可达历史，不能靠路径清单过滤。`977cdf1f`实际41706路径，含37个dist和9037个原始evidence；target及ignored资源不被跟踪。`255bbfbd`可达对象解压约3.35GB、无超过100MiB blob，这不是网络传输量，也不是未来最终提交的清单。最终必须重新审计，不能承诺尚未执行的上传大小或Actions成功。
