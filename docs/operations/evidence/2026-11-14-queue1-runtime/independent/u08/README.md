# U08 三交互技独立复核（2026-11-14）

本批退回：慧眼存在实际触发差异；新增“已复核等价”签署0。医心10案、鬼斧7案、慧眼14案，共31唯一case，逐案见 [对照表](case-table.md) 与 [结构化判断](case-assessments.json)。产品、测试、inventory、公共文档、原U08及L10目录均未修改，也未操作Git index/refs。

31案的八座HP/maxHP/face_down、手/装备/判定实体集合及牌面、主技能和实际7/8张牌堆顺序匹配；上游虚拟装备/判定及旧技能/预测/觉醒残留已逐案排除。28案的指定终态投影相同、3案不同；加上已用次数标记后27案相同。这里的“相同”只指列出的字段，不签完整初态、完整事件/RNG轨迹或技能规则等价。

确认的差异与限制：

- `b_under_control`：上游目标实际由慧眼主控制，过滤后不预测；原生缺该条件并额外摸实体106。控制关系未包含在共同初态投影中。
- `b_cancel_predict` / `b_cancel_then_retry`：上游真实DOM取消返回confirm/bool而无control，实际消耗本回合bolehuiyan4；原生空答不消耗，第二次预测多摸106。只有HP相同的取消单案仍有用次状态差异。
- `b_cancel_discard`：双方均已触发强制弃牌并仍待答；上游无取消按钮，未提交隐藏取消；原生空答被拒。此时杀9在上游ordering而原生discard，不写完成结算或全牌区相同。
- 每案21张无映射的初始discard实体未复制；77/78闪电的native导出nature=null、上游固有thunder。原始差别保留，不靠归一化签全初态。手牌排列亦不同，两杀案例实际出牌ID次序相反；共同比较的手牌为集合。
- 医心选满两张费用牌后的取消在上游实际点击“结束回合”，原生为空目标答，阶段端点不同。鬼斧目标均仅一件装备，多装备选牌/挂起未覆盖。其余未执行分支见 [源码复核](source-review.md) 和 [findings.json](findings.json)。

独立核验执行了：

1. 冻结输入核hash：[输入验证](input-validation-first.json) 对原raw manifest384文件/77,845,519B、完整manifest404文件/79,684,128B、29份源副本以及11个点名文件/保留二进制均匹配。只哈希作者报告字节，未读取其README、结论或比较器。原始完整manifest的SHA256为7934ad115c63fd0ace23d3b3800d1e2703e621554e9db6391035f09a1185205c。
2. 原始CDP重算：[来源记录](cdp-provenance-second.json) 匹配1072对请求/回包，最终31案的赋值从行588开始、最后快照行1072。每个保存stage/最终快照、fixture赋值、真实控件点击均对到原始回包。共70组赋值含旧失败/探索轮；没有把旧轮当最终案或按预期去重。30案root.done=true，强制弃牌1案仍pending。旧SyntaxError及所有采集轮保持原样。
3. 原生保留二进制重跑：[验证记录](native-replay01-validation.json) 的238个输出文件与原native-round2逐字节一致。二进制SHA256为6a08977ed231a8c71229dc50c650ee8b4fd02973d96389dc38d60239c4d3dffd；实际运行输出在 [native-replay01](native-replay01/native-results.json)。
4. 独立NGSA probe：[76份pending原档及后续命令](archive-replay02/pending-suffix-results.json) 全部解码/重编码字节匹配；恢复后的完整Snapshot仅auth_generation增加；询问规格一致。对每份pending单独重放后续全部命令，共134次命令的请求、实际回复、增量日志、状态摘要及最终完整Snapshot（只归一auth_generation）均匹配原记录。38个直接、38个逐答恢复原档均覆盖；31对直接/恢复final完整Debug亦只差auth_generation。[补跑原始日志](archive-replay02.raw.log) 通过76/76，失败0；不是原始NGSA字节相同。[首次编译失败](archive-replay01.raw.log)与[失败源码](archive-probe-first-source.rs)保留，原因是自有probe的json!表达式括号，不是产品失败。

独立重算首轮把每座编号输出中的seat字段误写成布尔比较值；[首轮源码](recompute-first-source.py)/first输出保留，second输出改为seat编号与seat_matches两个字段。31案投影/差异计数和CDP匹配结果未变。后续引用统一使用second结果。

版本和环境：实际采集在605201f3f49e10cea61c31d52a61bbb66c85599d及当时dirty产品快照执行，收口与本独立运行HEAD为5c73d1d9158145e33f98f2018013914667d13964。冻结源副本29份与当前字节相同，本次重跑前后489个相关Rust源码文件未变。产品受跟踪路径clean，工作区含并行未提交证据。规则333 / NGSA107 / content schema5 / ABI759 / WS8；当前rustc/cargo均1.99.1。正式hero-kill装载109host/0VM、无占位实体，实际锁与绑定规则摘要见 [注册输出](archive-replay02/formal-registration.json)。

原上游Chrome为157.0.8103.52、CDP 50185、control 50210、独立profile；使用5173现代临时ESM源码服务，FS51215归主agent。原始工具和路径见 [browser-environment](../../U08/browser-environment.json)、[源载入指纹](../../U08/source-load-verification.json)、[原始转录索引](../../U08/raw-transcript-index.json)。本轮只读转录，没有控制他人Chrome或重新启动浏览器；网络日志38,215请求中无Spine URL，未读/加载/复制禁止Spine运行库。手建fixture和root调用的原版技能不等于自然整局或原生产品UI验收。

未运行完整preflight、产品三crate/全工作区测试、其他平台或界面验收。没有私有Cargo target；独立probe使用共享target正常编译且未清理。命令、二进制/源指纹、磁盘、文档链接与冻结复核见 [commands](commands.md)、[收口记录](freeze-context.json)、[文档验证](validation-final.json)、[冻结文件清单](frozen-file-manifest.json)。

补记（2026-11-14）：慧眼主技能的来源/目标过滤不能推广到临时辅助；上游辅助也未独立绑定同一杀。精确限定见 [主技能/辅助事件范围补记](source-scope-detail.md)，原先表与发现保留。
