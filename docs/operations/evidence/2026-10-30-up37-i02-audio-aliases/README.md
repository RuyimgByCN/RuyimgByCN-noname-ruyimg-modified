# UP37-I02：声明语音别名与跨包形态（2026-10-30）

主树基线 `da7e4c019461a6c467a9dd2de849f3b1a7494b86`，规则330／archive105／ABI758／WS6、资源schema2六字段保持。本批九个维护文件来自已复审的声明语音候选；[接入清单](parent-main-source-admission.json)逐文件绑定SHA。这里只验收这一资源切片，整个I02、皮肤、动态声明、自动对局语音和37来源仍未完成。

## 实现与已修反例

同步器沿用TypeScript AST静态读取，不执行上游JS；支持既有audio、audioname、audioname2、tempname和真实characterSubstitute声明。角色关联按完整目录唯一global owner解析，禁止把共享技能池任意分给角色。标准刘备原15份错误关联缩为阵亡、仁德两条、激将两条共5份；skill自引用与阵亡语境分开。循环、歧义、未知owner在同步写入前拒绝。

首次23302别名候选漏掉8个实际owner跨包形态；P1和空fixture造成的P2经真实正向输入修正并[独立复审关闭](isolated-final-review.raw.txt)。79条替身声明、571项可用映射、91条跨包alias／56份payload补齐，原23302条顺序和内容保留。修正总补丁SHA `5a8a2d547ea6893009b2a8e9227e57349e7516f5957687af7c1391700fc84ef5`；[隔离交接](isolated-cross-pack-handoff.raw.txt)保留当时结果，不替代下面主树执行。

## 当前主树功能验证

[主树汇总](main-acceptance/summary.json)、[全部命令](main-acceptance/command-index.json)、[107份证据SHA索引](main-acceptance/artifact-sha256.json)及[完整交接](main-functional-handoff.raw.txt)记录：

- Node资源入口11/11、完整前端167/167、typecheck、resources-only同步、资源verify及完整只读verify均通过；未重新生成catalog、npack或Rust/WASM。
- 原12359条direct对象及非assets元数据精确保留；新增23393条alias，合计35752行。实际生成23393个audio-declared文件，逐项路径、长度、SHA及source/backing/target一致；8824个独立payload按inode去重额外核一次字节，原资源不反复重读。
- 79条替身声明逐条来源/行号/directives、唯一global owner、8跨包形态、571映射及91行alias均有正向核验。2582个目录角色的helper选择均属于本角色；当前可静态展开25880项映射，23393有backing、2487缺backing、234项动态表达式如实登记，未称完整上游声明等价。
- 真实Vue覆盖标准刘备5份、N01胆守、周瑜shadow、简雍tempname、刘禅／君刘备隔离及4 pot owners/8 forms。15角色121份完整WebAudio解码，14次原生controls点击均有playing、currentTime/playedRange推进且无error；214次真实HTTP核对源、清单、实际target及页面字节；九项错误归属／恢复场景通过。CDP不是OS-HID或物理听感。
- 外部Vite生产构建只写私有outDir，保留legacy WASM `2b1cc44e511484c804f10640429b32075fa525227f5d07468bdf5f9dc1fab355`、native WASM `2292d7ab1c04b05724e49eb750962bb78af119dac80011524b18574742ed189b`。使用保留329 ServeDir二进制只证明未变静态路由，不冒330房间／规则验收。
- 九源码首末聚合SHA `82fca1f0e76e48df0f557f095bacb1d8c6ce5280eb7beef12431418fd32de978` 相同；17195保护文件、461上游AST输入、CI/A源码、冻结包、两WASM、tracked dist、guard、HEAD/index不变。行数门禁、自测、strict docrefs及本地链接检查通过；六类missing-file提示和Node ExperimentalWarning保留。

15次功能wrapper调用14次exit0；唯一exit1发生在happy全部断言成功后的自有Chrome profile清理ENOTEMPTY。原driver、日志和成功browser结果保留，仅修有限清理重试，续跑剩余九错误与214GET，没有重复happy或改产品期望。entry/after/finalize三个取证wrapper另行通过；两个snapshot未独立计量queue wall，索引保留null。

## 验收边界与最终门禁

原Chrome失败 `audio/skill/tsumugi_mugyu1..5.mp3` 本轮按既有失败SHA过滤，未重测、未记成功，也没有导入派生件；121份样本均先核对旧成功码流，再执行本轮真实解码／播放。五条no-elst兼容接入另在私有树审查，不能混成此九文件验收。

最终四项门禁均exit0：行数自测2/2、8992维护文件、strict docrefs1065引用/越界0、完整preflight21/21（锁内327.392秒，脚本单测192/192、零跳过）；本地链接1910文件/坏链0。[原始命令与窄身份](final-gates/summary.json)及[完整交接](final-gates-handoff.raw.txt)保留98次ResourceWarning、六类路径提示和原cleanup失败。仅Cargo offline metadata，没有编译；source9、CI4、批准覆盖后的A53、两WASM镜像、37 tracked dist及guard/index均保持。

[主树独立复核](main-combined-independent-review.raw.txt)确认107份功能与69份最终门禁索引、实际Vue/HTTP及旧成功样本来源，未发现新增P0/P1/P2，结论OK with notes。父[独立派生结果](independent-review/browser-and-mapping-results.json)与[最终身份核对](independent-review/final-gates-independent-results.json)保全；首次派生器漏形态阵亡前缀／索引外层结构的错误日志亦保留，均未改变产品断言或旧结果。本批按裁决18收口，后续发布按裁决25单独登记；不将已有da7e十绿CI扩张至本增量。
