# C331包排序元数据：schema兼容补充审批稿

核验日期：2026-10-30。依据源码为`9dbc655`加已验收A330（本地提交`38ab1678`）；当前规则330／存档105／ABI758／WS6。本稿是可审批设计，尚未实施或占用schema4，也不表示运行验收通过。

[裁决22](../2026-10-28-user-decisions/README.md)与[ADR0788](../../../adr/0788-native-player-trigger-order.md)已经批准331的技能选序、typed Skill、PickTag/Answer、小型可恢复机器及必要排序元数据；新包schema仍须另行裁决。A330本地验收前置已满足。本次只请求下面的包格式兼容增量，不重复请求已有C实施授权。

## 需要新增的合同

| 数据或宿主 | 批准后承诺 |
| --- | --- |
| schema3默认包 | 保留原字节、包版本、逻辑摘要和旧锁；新宿主继续读取。排序默认normal／priority 0／order_selectable true。 |
| 新非默认排序包 | 仅这类产物使用schema4，新不可变版本发行；三个字段为timing∈normal/first/last、effective i32 priority、bool order_selectable。 |
| 新原生宿主 | 明确支持schema3默认和schema4非默认；拒非默认schema3、全默认schema4、未知schema及执行入口的schema0绕过。所有执行入口自行校验完整包与组合，不能只依赖上层恰好校验过。 |
| 新旧线宿主 | 保留schema3默认包执行；明确拒schema4及非默认schema3，不扩展旧调度器规则。 |
| 旧原生宿主 | 固定的旧native public bridge通过现有host3/package4兼容门拒绝合规schema4。下面的旧程序局限必须保留。 |
| 其他版本轴 | package format1、engine_api 1、VM/宿主ABI758、NGSA105、WS6保持；不新增VM指令、查询、效果、命令或Worker信封字段。 |

旧程序无法由新schema追溯修复：旧host0可跳过schema门；恶意作者将新数据伪装成schema3时，旧serde会忽略未知排序字段。保留新摘要的降级包通常会因摘要不符被旧正式读者拒绝，但若攻击者按旧算法重新生成摘要并作为新包导入，旧正式读者也可能接受并按旧语义执行。已有可信冻结锁仅保护其绑定的内容。新宿主必须拒这种降级；本补充不承诺所有旧执行者都必拒伪装包。

## 已核对的源码依据

旧[PackageTriggerEntry与ContentPackage](../../../../crates/content-schema/src/package.rs)反序列化会忽略未知字段；旧[public bridge](../../../../crates/content-native-loader/src/lib.rs)检查ABI、模块及兼容门，但不直接校验整包逻辑摘要，生成decl时硬编码Normal/0。[兼容层](../../../../crates/content-runtime/src/compat.rs)对非零schema要求精确相等，旧tooling host为3，因此合规4能在产生decl前被拒。正式WASM、server冻结读者额外校验包hash，不能据此假定public bridge已有同等校验。

这是静态可达性证明，尚未运行旧binary复现。14个必要源码指纹和带行号正文已保留；[原稿](original-proposal.raw.txt)、[独立复审](independent-review.raw.txt)及[核验指纹](sourceproof.json)分别归档。复审要求收窄降级保证、明确hash条目位置，本稿已采纳两项。

## 实施边界

- [engine-api版本定义](../../../../crates/engine-api/src/version.rs)报告最高读取能力4，与默认包输出3分开；compiler、manifest构造、merge/rehash按实际元数据选择3或4，禁止简单改常量导致旧包失效。
- schema验证、runtime compat、public trigger/active bridge、正式安装/锁定读取、WASM/Worker、server冻结恢复与legacy装载同步接线。验证先于幂等早退；失败不得提交安装状态或替换原局包。
- 旧schema3默认逻辑hash输入保持字节中性。schema4的非默认条目在原trigger条目的原位追加域分隔`trigger-order/v1`规范段，绑定原数组位置及trigger/skill/event/module/entry_pc，固定timing tag／i32小端priority／bool与边界编码；保留声明顺序。不得仅用可能重复的trigger ID定位，也不能移动元组而不改变摘要。显式默认JSON与省略默认JSON可以逻辑hash相同，不能声称content hash也相同。
- 同一logical skill多个handler必须保留稳定身份、声明顺序及一致排序元组。暂不可表达的不同元组或身份冲突须明确拒装、保留原包并继续阻断对应来源验收，不能取首项、丢handler或拆假技能名。
- forceDie/out/indexed/hidden/filterStop、直接ExtensionPack语义身份及未覆盖来源仍按C22待办保留。本补充不授权额外权威状态、任意indexedData或新VM作者ABI；拒装不等于移出37来源范围。

## 实施后必须提供的证据

1. 固定真实旧源码／binary：默认真实DSL包成功；同一合法模块的合规schema4经schema门拒绝，不能靠坏模块造绿。旧public bridge与旧正式安装、WASM、server分别验证实际拒绝顺序。
2. 新native读取3默认／4非默认成功，legacy只执行3默认。非默认3、全默认4、未知schema、非法枚举／整数／bool、host0执行绕过拒绝。
3. 包字段／条目位置／filter绑定／顺序／冻结JSON／锁篡改，schema4降级3且不重算、新算法重算或旧算法重算摘要均分别登记；新宿主拒非默认3，旧程序已知可接受情形按真实结果保留。
4. 原位非默认hash段、默认hash中性、merge保留元数据、逐handler一致性、验证先于幂等早退均有能红的负控。
5. 真实DSL→元数据→正式发行／安装／武将授技→真实事件及跨native/WASM/Worker/server、pending/OptionalGate/嵌套效果、跨进程冻结恢复完整对象比较。旧字节、旧包、旧锁不原地改写。
6. C331原已批准的规则／存档／选序／超时WAL矩阵、原生三包、受影响包、workspace、conformance、严格门禁和完整preflight另行真实通过。本稿不计作测试存在或实现完成。

审批范围仅为上述schema4发行、3/4读取与拒绝、hash/不可变包策略及实际拒旧矩阵。其余升号或新状态/VM能力不在本请求内。
