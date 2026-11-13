# 2026-11-14 上游实际触发与原生指定投影对照

源码HEAD `bd58ef06d59940baed8d7f93e61ce571b152fb81` + tracked dirty；规则333／NGSA107／schema5／ABI759／WS8。实际命令和时刻见 [原生最终运行](native-comparison-final.json)、[投影重算](runtime-projection-reviewed.json)；实际输出 [原生13分支](native-comparison-final-results.json)、[逐分支投影及24次上游记录](runtime-comparison-reviewed.json)。

只比较所有座位的体力、手牌/装备牌面多重集、五行属性；U01另比聚贤颜色禁用及玄论待生效标志。13分支、24次上游实跑投影相同；27次是本轮原生选择挂起的新档在回答前后恢复哈希相等，不是27份历史旧档。整技能等价新增签署0。

| 来源/技能 | 实际分支 | 上游次数 | 原生挂起恢复次数 | 指定投影 |
| --- | --- | ---: | ---: | --- |
| U01 | `yuanzhen-sha` | 2 | 1 | 相同 |
| U01 | `yuanzhen-tao` | 2 | 2 | 相同 |
| U01 | `yuanzhen-cancel` | 2 | 2 | 相同 |
| U01 | `juxian-top-black` | 3 | 4 | 相同 |
| U01 | `juxian-top-red` | 3 | 4 | 相同 |
| U01 | `juxian-top-equip` | 3 | 4 | 相同 |
| U01 | `xuanlun-confirm` | 2 | 1 | 相同 |
| U01 | `xuanlun-cancel` | 2 | 1 | 相同 |
| L06 | `shengke-sha` | 1 | 1 | 相同 |
| L06 | `shengke-tao` | 1 | 1 | 相同 |
| L06 | `wuxingpan-sha-0` | 1 | 2 | 相同 |
| L06 | `wuxingpan-sha-4` | 1 | 2 | 相同 |
| L06 | `wuxingpan-tao-0` | 1 | 2 | 相同 |

## 输入及选择边界

- 原始 [case-results.jsonl](browser/case-results.jsonl) 共31条成功记录；前7条探索fixture保留但未纳入同牌面对照。exact8、ordered8、blind-repeat3与L06五条合24条全部保留、全部比较。早先错误配对及重复键断言失败见 runtime-projection-compare-first、runtime-projection-mapped-first 原始日志，没有删除失败。
- exact与ordered的三条聚贤场景牌面相同，但exact未固定原始手牌DOM顺序；ordered额外固定并断言。对照范围是牌面多重集，不声称完整初始状态、暗槽下标或手牌顺序一致。
- 上游gainPlayerCard先randomSort暗按钮。两端都只在盲选实际结算后按收到的牌面归到红/黑分支；没有为获得指定颜色预先读暗按钮，没有丢弃重复结果。规则RNG洗牌、私有映射保存及恢复不重洗由原生4项回归另测，不声明JS/native随机流逐比特一致。
- 远振、玄论通过真实Duel useCard触发；生克通过真实Sha useCard触发。聚贤和五行盘用原始useSkill子事件，入口模式/选支付牌UI未完整经过；五行盘先实际useCard装备并finishCards。
- 未比完整事件顺序、牌堆/弃牌堆、随机流、所有触发条件、组合、额外阶段及取消。L06五行盘第二选择可取消而上游强制、聚贤J01–J05等仍是差异；不能因本表上调状态。

## 启动与来源

沿用 `apps/web-client/scripts/tabletop-comparison-cdp.mjs` 的headless Chrome、独立profile、localhost/CDP方式。新版为原始noname 1.11.7的临时Vite副本，琉璃为原始noname-liuli 1.10.1.0.1本地静态服务。所有浏览器请求、返回、异常见 [browser目录](browser/launch.mjs) 同目录的session日志，启动失败保留。

来源标签勘正：`active-session-modern.json` 的upstream是launch脚本内部未使用的琉璃静态server根；实际访问URL为5173，实际TS模块与临时原始noname源码副本见 [modern-setup.json](browser/modern-setup.json)。这个旧metadata原文保留，不能据它把U01结果标成琉璃版。官方/上游打包的Spine运行库未读、未复制、未加载。

## 尚未执行的来源

U01三个交互技能、L06仅有的两个技能已执行以上限定分支；L22五个技能为自动钩子，没有独立玩家技能选择，不能拿配置凑数。其余U06、U07、L10、L29、L28、L27、L19、L20、U08、L25共10来源的30候选仍待运行；L22自身自动规则也未作上游运行验收。完整候选和实际事件入口见 [interactive-candidates.json](independent/other-sources/interactive-candidates.json)。未把本轮主动暂留写成“上游确实跑不起来”，没有以逐行审查豁免未跑项。
