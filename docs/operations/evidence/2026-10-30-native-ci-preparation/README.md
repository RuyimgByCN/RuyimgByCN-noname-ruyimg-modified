# 原生CI入口准备与外部发布边界（2026-10-30）

**状态：配置及输入修复已独立复审通过，已接入`977cdf1f`之后的本地主树；本地完整preflight新一次21/21、exit0、310.023s，入库审计进行中。真正GitHub Actions运行0次，push0次。** 本批不改变329/105/758/WS5及WASM默认，不拿本地准备当平台验收。

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

外部目标是现有公开仓库`RuyimgByCN/RuyimgByCN-noname-ruyimg-modified`的新`native-convergence`分支。当前仅main、无原生源码；本地/API权限不等于用户发布授权。必须先本地提交并刷新真实tree/history/object清单，再单独取得明确许可；不force、不新增remote、不改default main、不合并、不创建PR/release/comment、不改设置/secrets。

普通push包含完整tracked tree及远端缺失的必要可达历史，不能靠路径清单过滤。`977cdf1f`实际41706路径，含37个dist和9037个原始evidence；target及ignored资源不被跟踪。`255bbfbd`可达对象解压约3.35GB、无超过100MiB blob，这不是网络传输量，也不是未来最终提交的清单。最终必须重新审计，不能承诺尚未执行的上传大小或Actions成功。
