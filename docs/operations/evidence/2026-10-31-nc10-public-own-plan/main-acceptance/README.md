# Main Own修复功能及最终验收（2026-10-31）

对象为Main5fc上的精确六文件组合，补丁SHA `c2ebaa41542c63f169b0c55367817a6d22cad22231401ed6af94859017016110`。本地限定验收完成；提交/发布另登记，非全部NC10或Goal完成。330/105/758/WS6、API1/package format1与桌面transport默认WASM保持。

[原报告](verification.raw.txt)、[综合审查](combined-review.raw.txt)、[父714产物/26命令审计](parent/noname-parent-nc10-main-final-audit.json)记录原生1719/0/75、workspace6922/0/77、前端179/type、独立目标 `-Dwarnings` Gate1b、Worker82、legacy conformance19、标准/正式包40+20轨迹、7组binding/adapter完整存档及12个真实UI/dedicated-Worker场景。完整Plan相等证明来自native Rust；WASM只暴露auto-step摘要，不混称完整Plan实测。

原Shan私有驱动误以为可空取消而exit101，原失败及代码保留。实际询问min=max1/allow_cancel=false，空答案正式InvalidChoice并保持完整状态；Pass与Card分别成功，direct Card和auto完整存档相等。真实遗计Own取消是独立正控。恢复只允许已知auth_generation+1，不归一未知状态或把相同错误算隐私正证据。

当前Main发布WASM SHA `30e97f8aff0cc683e5d8bb3e2b122e023dba08260c45a3fbc0ebee630268c0bd`，实际浏览器请求、bundle及Worker身份见[产品绑定](original/product-binding.json)。18模块仅此WASM字节改变，默认dist37、source33、protected99、旧legacy输入/产品保持，未强行加入生成物。

父发现原探针旁路Cargo check查询crates.io并形成21个不同于Main的外部依赖版本，原7组native结果只能称跨依赖对照。[原旁路记录](sidecheck-provenance/record.json)保留真实flycheck日志；精确启动者/argv/环境/exit未知，不宣整会话离线，也不据此解释index变化。另以原Rust探针、Main锁复制及[30外部条目逐项核对](lock-aligned/lock-verification.json)完成严格 `--offline --locked` [补证](lock-aligned/report.raw.txt)：7组新native档与原实际WASM及旧native逐字节一致，回复/revision/hash/events也相同，Main源码和锁未改。执行二进制f9c404a4…私有保留，[父独立新产物/锁审计](parent/noname-parent-nc10-main-lock-audit.json)及[独立补充复审](lock-aligned/review-addendum.raw.txt)通过；其余已通过检查未无因重跑。

最终完整preflight21项一次通过（锁内378.023秒），冻结窗口11:42:19.226134–11:49:59.117970 UTC，44,306 tracked及14,920维护/Markdown输入、源码/产品/HEAD/tree/hooks/status/index等11字段首末相等。11:26至冻结前原始index变化原因未知，初始无原副本/逻辑项；冻结raw与逻辑项相等且全44306项stage0等于HEAD，不推称整会话raw不变。父11:51独立逻辑审计补证，不归因、不恢复index。

验证者于12:21:30明确[解除文档冻结](doc-freeze-release.raw.txt)。本目录及随后状态更新是**冻结后归档**，不声称在原21项快照内；另跑对应文档检查。[复制映射](archive-copy-map.json)逐项绑定原件、归档SHA及无损gzip替代名。编译缓存、完整bundle、旧生成物与运行二进制按原路径/指纹私有留存；原artifact清单没有改写。父解析器三次格式/路径/尾换行假设错误另保留，不作产品失败。

未新跑桌面GUI/WK、实体平台、听感/功耗，不证明no-choice/Other/VM闭合、一般强度或上游规则等价。现有Jiu-other规则缺口另列基准，未用本修复掩盖。
