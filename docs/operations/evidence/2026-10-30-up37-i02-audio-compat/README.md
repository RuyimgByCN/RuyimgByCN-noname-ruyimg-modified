# UP37-I02：五条no-elst音频兼容接入（2026-10-30）

对象为`a1f7250815eee803d931301ab46207719cb9a15b`加已审compat14增量，规则330／archive105／ABI758／WS6／资源schema2六字段保持。catalog schema4是既有角色目录格式，与私有C331内容包schema4无关。[接入身份](parent-main-source-admission.json)逐项绑定14文件及总补丁`2605af4b6a8fff48a76be560ed9d7f7cfd3b3a5e24218fb23dc7c53c90bf5264`。

## 实现及隔离复审

五条原`tsumugi_mugyu*.mp3`实际为M4A。派生件保留有效AAC包，去掉被误当媒体样本的配置包，并以no-elst容器避免已证明的WK PCM前缀变化；不裁PCM、不重编码。五份checked-in m4a合计263579字节，内部provenance绑定原SHA、派生SHA／size、固定recipe和工具身份，CI消费已核字节，不安装或执行ffmpeg。原9652份音频不替换、不改字节。

[隔离实现](isolated-implementation.raw.txt)之后，库存遍历会跳过未绑定symlink的P2经[真实反例修正](isolated-p2-fix.raw.txt)及[同reviewer复审关闭](isolated-p2-review.raw.txt)。本地专用Dirent递归拒绝任意symlink／非regular输入，保留共享遍历行为；来源、哈希、输入集合、逃逸校验均在写入前完成，不自动更新期望。

## 主树功能结果

[完整交接](main-functional-handoff.raw.txt)、[汇总](main-acceptance/summary.json)、[17次命令](main-acceptance/commands.json)及[64份SHA索引](main-acceptance/artifact-sha256.json)记录本轮实际执行，所有命令exit0：

- Node资源三入口17/17、完整前端169/169、typecheck通过；resources-only同步／verify及完整只读verify通过，没有Rust／WASM／Swift／ffmpeg编译或catalog／npack重生成。
- 原12359条direct前缀对象及23383条其他alias内容／顺序精确保留；只改10条alias的m4a basename及compat来源／payload绑定，新增5条backing，最终35757行／23393别名／raw音频9652。只对新5输入、5backing、10alias额外核body；未改对象复用原资格及dev／inode／size／mtime，异常0。正常维护sync／verify自身检查原样保留。
- 生成audio／portraits／audio-declared／audio-compat树经已审入口rm／relink，父明确纠正最初“不得触写生成域”的过严约束，[授权记录](main-acceptance/authorization.json)保留；不声称生成目录metadata不变。75项维护保护输入、A源码、旧锁、目录／包、共享遍历、CollectionVue、global-owner resolver、模块、dist、guard、HEAD／index保持。
- 真实生产Vue选择紬文德斯，DOM恰五条本角色m4a，五个原生controls经CDP、rate1、muted播放至ended并完整WebAudio解码。真实Rust404注入只影响指定path，其他loadeddata不误清该错误，切角色不污染，恢复同一路径后错误清除；八份场景快照见[Vue原始结果](main-acceptance/chrome-vue-result.json)。普通资源未mock，未改Vue业务。
- 17个直接HTTP身份由Main实际文件核长度／SHA；CDP逻辑7700请求全部转自有Rust64033，实际路由逐条保留。retained329 ServeDir只证明静态路由，不冒330联机资格。现有两份WASM先核完整SHA再复制私有输入，Vite产物在own/tmp，未写默认dist或共享依赖。
- retained WK helper运行当前Main五derived，完整解码／HTML ended及双声道Float32 SHA与旧已审派生件证明一致。本轮未重解原五份raw；它们仍保留原Chrome失败。Chrome比WK多2112帧／44ms前导，不作跨decoder位级一致或物理听感结论。

[父独立核对](parent-independent/functional-independent-results.json)复核64份证据SHA、14源SHA、完整清单差异、17条HTTP身份、五条实际解码／ended及新payload字节。初始父派生器误把basename改名当缺失path、并误用不存在category字段，错误说明及原脚本保留；纠正为精确mp3→m4a配对及既有六字段后通过，未改产品断言。

原Vue解码记录`decoded[*].bytes`为0：driver在`decodeAudioData`已分离输入ArrayBuffer后读取byteLength。解码前已经对完整status／SHA／size断言；真实长度由独立HTTP记录和当前清单另行逐项核定。原值不改写，也不因记录缺陷重跑已完成媒体采集。

## 验收边界及后续

功能及最终四门禁均通过。完整默认preflight21/21、exit0，锁内323.962755秒／wall324.007595秒；行数自测2/2、8992维护文件、strict1065引用／越界0，六类提示保留。[最终命令与日志](final-gates/commands.json)、[首末身份](final-gates/source-comparison.json)、[完整交接](final-gates-handoff.raw.txt)及[独立组合复审](main-combined-independent-review.raw.txt)均已落地，复审结论OK with notes、无新P0/P1/P2。父另核23份门禁证据SHA、四条真实exit0和完整21条状态，见[结果](parent-independent/final-gates-independent-results.json)；首次父解析器预期默认console含字面21/21的错误亦明确保留，实际默认接口输出21条逐项成功及总成功，没有改门禁或补跑。

旧64份功能证据、14源码、75保护项、两WASM及18模块、37 dist、guard、HEAD／空index在门禁首末一致。preflight只保留其接口实际输出，不声称取得内部被抑制的子项日志。临时source-only补丁的SHA与total patch不同是两种不同范围的补丁；验收身份依据始终为14份文件SHA与聚合。

本批按裁决18完成本地切片验收，提交／限定发布另存实际结果；整体I02及Goal未完成。基线a1f真实CI十绿另见[登记](../2026-10-30-up37-i02-audio-aliases/publication-ci/README.md)，不覆盖此14文件增量。

原两个Node ExperimentalWarning、strict六类missing-file提示与全部stderr保留；本轮没有新12k codec、性能、物理HID或新平台CI。来源／皮肤／动态表达式／自动对局语音、I02整项和37来源／整个Goal仍未完成，WASM默认不变。
