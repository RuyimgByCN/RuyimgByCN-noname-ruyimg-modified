# NC10：Own询问来源字段与兼容验收（2026-10-31）

[裁决27](user-decision27.json)批准的source_skill已实现，33文件候选已接入Main并完成本地限定接纳，尚待本批入库发布。基准HEAD仍4dd0b286，规则330／NGSA105／ABI758／WS6保持；桌面执行transport仍默认WASM（规则backend默认native与之不同）。最终Main门禁、父fresh核验及组合复审已通过，见[Main V3限定接纳](main-final/README.md)；这不构成整NC10或Goal验收。

## 合同与代码边界

[v1](proposal-v1.raw.txt)＋[规范v2](proposal-v2.raw.txt)及原[初审](review-v1.raw.txt)／[复审](review-v2.raw.txt)原样保留。普通已知native OptionalGate的Own可空来源须catalog全局唯一、exact def、owner既有可见技能资格、全部top-level asks中唯一完整producer且等于实际原顺序first-depth consumer；仅直接OptionalGate inner链沿真正内层来源。Other generic，未知／VM／歧义／Host同prompt均None；可信Referee及AI复用owner投影，不因此授应答权限。

API／TS／NativeSession／Worker／IPC／WS6／network／Vue文本展示转发保留全部旧deadline、options/token、revision／continuation等字段。wire缺失／null兼容，present空名及错类型拒绝。legacy typed capture／两typed restore复制并只清新字段，直接serde／bytes也剥离，不改输入、失败原子性或其余原接受／错误。source不是root／Confirm-prefix／Cancel-tail／VM续行证书，不修改正式调度／restore／fuel／算术门禁。

## 冻结、实跑与独立核验

[V2交接](implementation-v2/report.raw.txt)封存31改动／43,867文件。[恢复交接](implementation-v2/recovery.raw.txt)确认05:16:23实体检查点已存在，约18分41秒封存；父当时未收到完整路径，不能把未收到通知写成文件不存在。原一小时TIMEOUT是编排失败，记录保留；恢复只读封存，没有重跑或偷换退出码。

[源码复审](implementation-v2/source-review.raw.txt)No issues。私有V2 workspace6916/0/77、native三层1715/0/75、前端179、Worker82、conformance19、current WASM／轨迹、实际WS6 owner／Referee／Other／重连及Web WK／Tauri native各四场景记录在[原始证据包清单](implementation-v2/raw-bundle-manifest.json)。missing／null／恶意名称是实际restore后注入的UI兼容／转义场景，不冒称自然来源；旧WS6 DTO／TS reader是冻结reader fixture，不是旧客户端binary实连。95产品身份保留，包内不复制执行文件／WASM／完整3GB源码树，base Git＋[overlay](implementation-v2/source-overlay.tar.gz)及[manifest](implementation-v2/source-manifest.json)可重构源码。

父[fresh V2 parser](parent-audit/audit-v2.py)／[结果](parent-audit/v2-independent-results.json)核43,867份实际SHA、43,836未改动Git blob OID、31工作／冻结一致、58日志／命令结果及全部95现存产品SHA；无产品执行。这里不是采信manifest字段代替当前字节。

## legacy错误P2的独立修复

[原legacy复审](implementation-v2/legacy-review-with-P2.raw.txt)发现scalar expected文案漂移，后续真实差分又确认短序列13→14诊断变化。[V3实现](legacy-p2-v3/report.raw.txt)仅一生产reader＋三个测试相关文件：私有serde remote恢复旧13字段／名称，source按skip在严格String之前忽略，删除手写map／seq适配，不替换错误字符串、不复制公共enum。

原445输入实际18差异（7scalar／11短序列），同一矩阵修后0差异；比较完整Deserialize错误及行列号。成功分支只证明接受一致，不能从矩阵推导所有payload／序列化字节相等。完整值剥离、typed输入不变与失败原子性由原T/J/W测试提供相应证据。[复审闭合](legacy-p2-v3/independent-recheck.raw.txt)No issues／OK with notes。受影响core3861/0/1，新legacy WASM实跑120步／33实际choices无source；typed-poisoned恢复的JS API不存在，该部分仅原生目标sink覆盖。

父[增量parser](parent-audit/audit-incremental.py)／[结果](parent-audit/incremental-independent-results.json)核四文件、同一445原始输入及旧／bytes／直接serde结果、全部日志／产品SHA和oracle反向import后与4dd原blob逐字相等。V3完整43,869文件由已核V2＋四文件overlay组成；未将V2的6916计成V3全量。

## 私有资源材料化失败与纠正

[操作交接](resource-materialization/report.raw.txt)保留原exit1：复制缓存后，2707画像／9652音频内容相同但inode不同，expected hardlinked被计为copied，verifyCatalog汇总失败。仅用既有resources-only修私有ignored资源链接后，新resources-only verify及原full verify均exit0；正式manifest／catalog／六compat输入及其余source14路径的字节未改。source14是八脚本／CI／资源输入＋六compat输入，不能误称compat目录有14文件。

复制指纹、前后全tuple、旧失败／新通过日志在raw包内，父增量parser已核19原证据SHA及实际链接计数。原失败不回填；这是材料化修正，不是规则源码Fixed或全门禁通过。

## Main与运行时边界

[Main V2功能交接](main-functional-v2/report.raw.txt)实际原生1715/0/75、前端179／type／外部构建通过，1131输入及31／99保护／18模块／37dist／6脏文档首末一致；首探索status早于index捕获，不能外推更早index不变。V3已按[父接入身份](parent-audit/main-admission-v3.json)组成33文件；最终功能及21项门禁、父fresh核验与组合接纳已通过；[本批Main V3](main-final/README.md)与前述私有结果不相加。

新的P/Q实际byte restore均接受：完整初始PolicyInput／pre-events／source=遗计相同，Confirm后首Pending分别jiji_target/owner1与jijie_give_target/owner0。原[源码候选](confirm-prefix-source-countercandidate.raw.txt)仍保留其未执行时的历史资格，后来实际反例／红绿在原始包与已登记nc_rounds测试中。此例只限制来源字段的证书含义，不放任何门禁。

原19 NoCert／旧329 runtime、完整质量／上游／平台验收仍未关闭；本批没有并入旧私有runtime，不用两个Err代替隐私正向通过。Android授权设备仍0，最新Apple CLI18s超时当前unknown，历史unavailable与当前unknown分开见[readiness](physical-readiness-after-user27.json)。整体Goal仍未完成。
