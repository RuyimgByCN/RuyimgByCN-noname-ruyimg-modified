# I02 Main typed-wire phase 1：限定验收

核验日期2026-10-31。Main基线d07b35f94a42b282222dda4ab7bbc5c39ab8fddb，规则330／NGSA105／ABI758／WS6不变；30路径补丁SHA256 `1b154aa7573fff3fc5b2fdfb54e5dbd743206c4c2b91f1e384050f38b5cdae94`。父精确接入后保留Own六源和主锁；源码与完整证据见[原报告](verification.raw.txt)、[归档映射](archive-copy-map.json)。

验收限于可缺省DTO／S→C metadata、严格consumer入口、八项有界pending及旧响应省略兼容。九生产constructor仍为None；不产生真实presentation事件，不证明提交范围、历史可见性、qualified资源factory、音频owner或完整I02／SDK／37来源。

实际命令及原日志在original目录：API-protocol12/0/0，native1719/0/75，workspace6929/0/77，frontend441/type、独立目标`RUSTFLAGS=-Dwarnings cargo check --offline -p engine-native --all-targets`、Worker82、conformance19、crosshash通过；唯一完整preflight21通过。忽略项照实保留，计数互有重叠。父[独立脚本](parent/noname-parent-i02-main-final-audit.py)重新hash1054产物、核37命令绑定和源码36、解析实际测试计数，不重跑已完成producer。

当前原生WASM为`7c2a9f266c88e73e720a7b4e4073d28100eb2b6b3a18f087f721a57d881460da`，实际重建legacy为`846351a9b844989c04d868dfb57b829abe5320ef452cf3da49494dd611e4cab5`；engine-api变更会影响旧线编译输入，不能把固定旧fixture等同当前源码产品。18模块仅三WASM改变，默认dist37不变；四个精确授权Tauri schema的字节和mtime保持，未手改生成文件或源权限。

原生与当前WASM七组完整回复／事件／存档相同，标准40与安装包20轨迹通过；真实生产UI／Worker12用例通过。严格入口另用当前Main JS构成私有entry，实际加载于Chromium/SAB和hidden Tauri/WK IPC：前者162注入case／80共享坏输入／163省略正控，后者161／80／162并有811实际callback；注入边界只证consumer，不能宣Rust producer或原发版桌面UI完成。真实服务器Accepted省略响应与断线后同键重放证明无第二执行。见[加载身份](original/actual-main-js-load-binding.json)和[WS结果](original/server-wire-results.json)。

门禁窗口约13:39:20–13:46:26，gate-entry／exit／14:03:45 seal的实际12比较类全同；原报告“13”保留并[追加勘误](index-command-supplement/report.raw.txt)。初始raw index至gate-entry仅33项ctime秒／纳秒变化，父独立解析全部47913项与SHA1 footer，确认路径／OID／mode／stage／flags／其余stat／padding／扩展均相同；写入来源未知。原whole-session raw相等断言失败保留，不恢复index或重跑测试。

[综合复审](combined-review-with-P2.raw.txt)的P2是两份所谓原始parser命令误捕获后续extractor；旧封存文件原样保留，追加真实执行命令／tool结果且唯一匹配定义，见[tool-only证据](index-command-supplement/tool-only/summary.json)及[独立闭合复审](index-command-supplement/review-closure.raw.txt)。完整assistant/session记录仅私下保留，不作发布材料。原父核验JSON的pending supplement状态属于当时，后续补充及闭合复审完成该项。

14:03:47明确解除Main冻结；14:07:43父另接入[checkout局部longpaths修复](../2026-10-31-nc10-public-own-plan/publication-ci/README.md)。它不是原preflight输入。修正验收提交34e于15:19:09正常发布、推送租约随后明确释放；父另fresh核3433文件／105988114B、552排除项、guard8控制、raw／逻辑index／flags及独立远端引用。见[发布检查点](publication-checkpoint/README.md)及[最终独立登记](publication-ci/README.md)。新CI41214274772 attempt1最终10/10、130steps128success／2skip／0failure／0null；两Windows checkout及完整build／desktop installer/upload实际成功。父fresh119项SHA／CRC、十完整日志和六live API，独立登记复审OK with notes；只证明本次CI，安装器仍仅API元数据，无实体安装／签名／功耗验收。原d07 CI的8成功／2checkout失败不改写，冻结后归档和文档另查链接／行数，不冒称属于冻结快照或34e远端CI输入。

编译产品和浏览器缓存保持私下原件，不force-add；old-generated18／frontend-dist-before-legacy别名、测试profile／缓存及四份生成schema备份由[发布排除清单](publication-exclusions.json)逐项绑定，保留本地、不入修正提交；原copy map只记录物理复制。初次本地bdaf误含491个profile／缓存文件，发布者在任何push／guard／网络请求之前发现并停下，父另排除四份生成备份；错误提交保留在本地保护引用，修正仅重做这枚未发布叶提交，父d07及完整已发布历史保持。见[撤回与修正记录](prepublication-correction/correction-scope.json)及[停止原报告](prepublication-correction/withdrawal-report.raw.txt)。原3913文件审计只证当时blob／mode相符，不再作为修正发布范围证明。大JSON／raw／日志可无损gzip，逐文件原SHA、发布SHA和路径由映射记录。所有源码复审和本轮验收只关闭上述phase1边界；全目标仍未完成，当前待办以[进度主稿](../../../architecture/native-progress.md)为准。
