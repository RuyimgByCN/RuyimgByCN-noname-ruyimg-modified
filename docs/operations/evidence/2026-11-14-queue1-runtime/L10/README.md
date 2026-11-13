# L10 史诗卡牌：三个交互候选的运行证据

2026-11-14，本批执行者 `/root/fix_dddanzhi`；独立复核另交 `/root/repair_doc_refs`。基线 HEAD `605201f3f49e10cea61c31d52a61bbb66c85599d`，tracked dirty 包含主 agent 的酒诗恢复修复及并发文档/构建产物，详见 [preflight](preflight.json) 和 [末尾环境](environment-final.json)。规则333 / NGSA107 / schema5 / ABI759 / WS8。没有修改产品、共享文档、清单、Git index/refs；未提交、未发布、未签新等价。

最终26个 fresh 上游页中，23场观察到所需分支，3场党锢真实进入原始技能后因 UI 依赖异常阻塞。逐座初态共104项检查通过；每页先等真正onfree/cardsAsync完成且牌堆连续600ms稳定，再建夹具，所有末态新增无ID牌为0。原生26场各执行direct/restored，共52次场景、164个命令请求/回包，102次pending NGSA恢复全部成功；两条轨迹的每命令请求/回包/log_delta/state相同，26份最终完整快照仅auth_generation不同。52份最终NGSA与102份pending NGSA均保留；最终字节只有7组完全相同，其余19组因为恢复增加auth_generation而不同，不能写成字节全部相同。

[逐场26行表](case-table.md)与[机器可读投影](comparison.json)区分：9场限定投影相同、12场实际差异、2场仅不触发门控、3场上游实际阻塞。限定投影相同包含蛊惑不质疑的杀/桃、无牌/无技能，以及让节唯一余锦囊、移动A/B和两个条件反例；不代表技能或整来源等价。

具体差异见 [findings.json](findings.json)。蛊惑缺实际质疑与真假牌分支、质疑后弃牌/失血/缠怨、响应入口；原生提前消耗次数，目标窗也不能取消。让节按相反牌堆端检索，基本与装备得到不同实体，类型窗不能取消；上游真实中文字“取消”因源码只检查cancel2而仍摸一张，原生则真取消，保留这个上游异常行为。党锢三场均在原始step0的dui调用报错，另有原始initSCSAnim的dcdAnim错误；[逐行替代核对](k-danggu-line-comparison.md)列出未运行的随机/伙伴UI/授技范围，并记录触发门控和冲突表方向的静态差异，未将静态差异称实际浏览器复现。

初态逐座对齐HP/maxHP、翻面、主技能及手牌/装备/判定实体ID、牌名、花色、点数、属性；7/8张完整保留牌堆也按实体与顺序对齐。每场有21张原生初始弃牌无法映射到该上游牌定义，完整牌面已保留在各场skippedInitialDiscard，故不宣称完整初态相同。比较手牌/弃牌按集合，未比较DOM排序/弃牌顺序、跨引擎完整事件树、全历史、phase状态与RNG游标；两场党锢反例的上游后续正常摸牌也不与原生截点比较。原生内部保存恢复验证则使用完整结构快照，而非只按牌名或人数比较。

原始入口与数据索引：

- [命令及首错修正链](commands.md)：保留原生首跑guohe名不匹配、Python首跑SyntaxError、首party夹具缺playerMap登记，以及修正后真实dui依赖错误；此前日期记录未覆盖。
- 原生 [probe源码](native-probe/src/main.rs)、[第二轮原始输出](native-second.raw.txt)、[完整逐场记录](native-round2/native-results.json)、[direct/restored核对](native-round2/continuation-comparison.json)。同目录有完整命令jsonl、包输入/锁、全部NGSA和最终snapshot Debug文本。
- 上游最终 [browser-round4/results.json](browser-round4/results.json) 与逐场upstream-*.json；冻结的 [runner](browser-round4-runner.py)、[fixture/event入口](browser-round4-setup.js)、[真正扩展加载](load-source.js)。原始代码通过本地服务读取，质疑/取消/移动均实际点击上游控件，规则步骤没有替换。
- CDP原始 [请求](session-l10-1794593358863-requests.jsonl)、[回包](session-l10-1794593358863-replies.jsonl)、[异常/浏览器日志](session-l10-1794593358863-events.jsonl)、[HTTP日志](session-l10-1794593358863-http.jsonl)；[启动元数据](active-session-l10.json)。服务在读任何文件前屏蔽spine路径，整个会话0个spine路径请求，未读/复制/加载官方或上游打包Spine运行库。

[清理记录](cleanup.json)与[直接socket复查](cleanup-direct-check.json)：仅清理自有Chrome/profile，两个PID已退出、三端口直接TCP拒绝连接。使用共享target但未清；没有自建private target。二进制及hash保留在environment-final.json记录的 `/private/tmp/noname-l10-runtime-preserved-tvv_75wj/native-probe-round2`，待主 agent 通知再退；旧L29 probe同样保留。

本批仅是队列1的L10三技能运行补课。没有运行完整Cargo/前端/真机门禁，没有验收其他L10条目、十名伙伴完整能力或产品UI，也没有提升清单状态。原始运行数据和本目录在交接时冻结。
