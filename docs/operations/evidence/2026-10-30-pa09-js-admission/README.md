# PA09：桌面JS在途请求上限（2026-10-30）

本切片在已发布`777bf33969e5ecc04b63971adab26be32f12fd28`上接入三个已审文件，规则330／archive105／ABI758／WS6保持，WASM默认不变。[接入身份](parent-main-source-admission.json)绑定源SHA及总补丁`064f0576d3f752a563ef5b9bcf672617494ffea189ac14ab514370d16b9edf41`。主树定向20/20、完整前端175/175、typecheck、外部Vite production bundle、行数自测2/2及8992文件门禁已通过；父已独立核37份功能证据、11次命令记录、三源与八个继承接缝及完整产物身份，见[Main功能交接](main-functional-handoff.raw.txt)及[父独立结果](parent-main-audit/independent-results.json)。最终四门禁及组合复审已通过，见下文；不能把私有173项结果或继承的WK实跑写成新Main GUI。

## 问题与最小修复

原DesktopEngineTransport每个postMessage都在Promise尾链追加闭包；Rust侧八个排队槽不限制JS持续未决请求。本次只在该adapter加入固定总量8，包含opening／active／queued。第九个请求在追加Promise和发出IPC前同步抛明确“未提交”，EngineClient既有catch立即删除pending并记perfFailed；不重试、不切换后端。每个接受请求在finally释放槽，listener抛错后尾链恢复；terminate清零并保留资源close／旧队列不提交行为。

[隔离实现](isolated-implementation.raw.txt)记录三个文件净生产源码增加12行、六项新增行为测试。[源码复审](isolated-source-review.raw.txt)无新增P1/P2、OK with notes：固定8比Rust queue8＋active更保守，属于JS入队约束，不新增配置或协议。旧source的真实红16/4与候选绿20/0、完整私有前端173/0、typecheck、行数门禁及初始setup失败均保留在[隔离证据](isolated-js/freeze-manifest.json)。

## 实际WKWebView验证

[完整实跑交接](actual-webview-handoff.raw.txt)、[独立实测证据复审](actual-webview-independent-review.raw.txt)与[原始报告](actual-webview/evidence/actual-webview-report.json)对应2026-10-30T19:16:41–43Z左右的一次隐藏WK运行，不是mockInvoke或Node模拟。真实生产main.ts挂载Vue大厅，使用生产EngineClient／候选transport，经真实Tauri NativeEngineHost和ResourceTable：

- 128次同事件轮status接受恰8（IDs2–9），120次拒绝（IDs10–129），拒绝时tail不变且无对应IPC；128份采样pending最大8，microtask边界120已拒、8个持续未决。
- burst八条真实IPC成功，随后24次顺序status持续成功、pending最终0。四个resource各open／close一次并各有退休拒绝；49 raw invoke包含4 open／41 request／4 close，其中37条request来自生产transport、4条为退休探针。
- opening未回调时dispose，init＋7排队均不发request；active回调未完成时再排7项后dispose，只有active发IPC。trace415为dispose，422是旧成功raw回包，旧transport无client delivery；新连接独立成功。
- burst的120个明确未提交与dispose原有的未知结果拒绝分开，未修改其口径。没有mock、writer人工延迟或第二队列；这只证明同事件轮JS admission和异步桥接生命周期，不证明OS压力、Rust队列饱和、吞吐或延迟收益。

旧retained App嵌入旧JS，不能靠运行时TAURI_CONFIG替换；本轮按限定授权只编译一次a1f native桌面release，独有target约1.02GiB、锁内185.628秒。编译期FrontendDist::Url指向自有19439，launch删除TAURI_CONFIG；native Rust源码3001条与a1f Git blob精确对应，a1f→777没有Rust／Cargo变化。只为观察原invoke而加的八行依赖observer返回原Promise，参数／结果不变。生产bundle、client／transport源、编译配置、实际served五项字节、两个签名前后executable身份均绑定，不把未运行的Worker/WASM32写成验证通过；App仅ad-hoc签名。

父[独立解析脚本](parent-actual-audit/audit-frozen-evidence.py)与[结果](parent-actual-audit/independent-results.json)核全部49调用／98条invoke事件、三个NDJSON精确过滤、源码／bundle及41项原证据指纹、3001基线blob、资源账目和当前清理。独立review完整读取脚本／结果并核fixture、observer、真实bundle；没有只采信33个passed字段。原parser把98 trace当49 calls、把active-connect init混进dispose的两次范围错误保留，没有改运行记录或重跑。

自有PID62278／62280退出，19439无监听，三个identifier专属目录不存在；清理原件仍有存档SHA，存档本体留私有目录，本公开证据不复制WebKit状态。首次Vite realpath失败、18条host native dead_code warnings和首次Main index不变断言失败保留。Main移动属于父19:18:54Z授权compat提交777，实跑私有a1f身份另证，不回退Main或冒称Main首末未变。

## 最终Main门禁与组合验收

[最终交接](final-gates-handoff.raw.txt)、[四条命令](final-gates/commands.json)和[独立组合review](main-combined-independent-review.raw.txt)已收口：完整preflight21项全部成功，锁内323.802秒／wall324.106秒；行数自测2/2、8992文件行数与Rust有效行检查、strict1065引用／越界0全部exit0，六类原提示保留。门禁窗口2026-10-30T20:25:03Z–20:30:47Z，21,726份捕获输入及18项首末不变量通过；caller仅按集合比较、文件SHA精确，source4／99保护／18模块／37dist／index及空staged保持。

父[fresh只读parser](parent-main-audit/audit-final-gates.py)／[结果](parent-main-audit/final-gates-independent-results.json)重核18份索引证据及index SHA、四exactargv／原日志SHA／wrapper实际exit0、全部21条成功摘要与18项capture比较，当前三源及attribute再次hash匹配准入。没有冗余重扫当前全部21,726路径；原preflight抑制内部子项日志，只声明接口实际输出。三改动路径的主动LSP结果为[clean3／diagnostics0](parent-main-audit/lsp-paths-result.json)，不外推Rust覆盖。

独立组合结论No issues、无新增P0/P1/P2、OK with notes，仅接受PA09 JS admission和精确路径属性修正，见[父组合接纳](parent-combination-acceptance.json)。按裁决18本地提交、裁决25普通发布为`4dd0b286b995fe414128b6c52eb83a4d0d85f6cd`，push0／精确guard／main保持见[发布receipt](publication/result.json)；真实run41125396035十job成功、Windows prepare／installer／upload均成功并独立登记，见[新CI证据](publication-ci/README.md)。原777的9/1不回填；新结果不关闭PA09六条件、实体安装／签名／真机或整个Goal。门禁编排首版runs.run参数形式在启动前被校验拒绝，改为工具规定的key／params形式后才启动，未运行失败实验或更换执行通道。

## 历史测量保全与边界

[历史bundle身份](historical-measurement-bundle/bundle-identity.json)绑定[159份已审受控PA09／全量直接codec证据的压缩包](historical-measurement-bundle/historical-pa09-and-codec-evidence.tar.gz)，SHA `0a2ce017f07d1774b0d13ea4ad5380026af892b019647cba59ff5c44c4713891`、10,850,227字节。包内160项含copy-manifest，逐项解包SHA核对通过，原始记录不归一；包含43产品文本源和5fixture，不含App／native executable／WASM构建依赖或设备清单。

其中旧受控WASM→native→WASM各280样本，click-to-paint p50/p95为17/19、18/21、17/19ms；不宣原生更快或非劣，合成CGEvent不是物理HID。旧raw invoke128为76 fulfilled／52 backend not submitted、`jsBounded=false`，因为绕过adapter，与本次client8/120是不同路径，不回填旧数。全量直接资源Chrome画像2707＋音频9647成功／5失败，WK画像2707＋音频9652成功；五派生兼容件的后续验收另见[资源切片](../2026-10-30-up37-i02-audio-compat/README.md)。

[证据复制映射](evidence-copy-manifest.json)保留新120文件来源／字节／SHA，明确排除WASM payload、App／target和WebKit状态存档；原actual-freeze仍保留完整原索引，未假称所有原产品都在本目录内。当前该JS切片的私有源码／实际WK证据审查通过，主树JS功能、最终四门禁及独立组合复审通过，按裁决18接受此本地切片。Main功能窗口的99保护／18生成模块／37dist／HEAD和正式entry→exit index保持，首次探索status早于index捕获，不能声称更早字节不变。首次锁等待timeout、bundle hook过早hash及caller枚举顺序错误保留；父重复遇到groups列表顺序差异，按同一精确文件SHA与路径集合核对后通过，另留[原parser错误](parent-main-audit/initial-parser-error.raw.txt)。PA09线程池收益／同负载不劣／完整产品矩阵、真机和30分钟功耗温度等门槛仍未闭合，不改变默认引擎。
