# 2026-11-14：12来源141样本的 inventory 应用复审

核验快照：2026-11-14T00:31:03.433735+08:00；HEAD `bd58ef06d59940baed8d7f93e61ce571b152fb81`，tracked dirty。规则333／NGSA107／content schema5／ABI759／WS8。复审人 `/root/fix_dddanzhi`。本次只新增本目录 `application-review*` 证据；不修改源码、清单、台账、旧审查或公共文档，不运行Cargo/浏览器，不提交。

限定结论：141个原始key的应用及更正记录可重放到本次清单字节。原79条确认中，保留68条具体静态差异对近似状态的支持，撤回11条误判或过度确认。修正后的审查分类是68条源码确认差异／47条待核验／26条静态未见差异；新增等价签署0。

当前样本清单状态是历史「已复核等价」65／「简化近似」76。其中67条由本reviewer的具体差异新撤回，1条原已近似而维持；另3条由其他独立审查确认同key差异而撤回。保留历史状态不表示本轮给47条待核验或26条静态未见差异签署通过。

| 来源 | 全来源分母 | 实审 | 源码确认差异 | 待核验 | 静态未见差异 | 当前历史等价/近似 |
|---|---:|---:|---:|---:|---:|---:|
| U06 | 11 | 11 | 4 | 1 | 6 | 7/4 |
| U07 | 36 | 10 | 7 | 1 | 2 | 3/7 |
| L06 | 3 | 3 | 2 | 0 | 1 | 1/2 |
| L22 | 8 | 8 | 2 | 1 | 5 | 6/2 |
| L29 | 9 | 9 | 3 | 6 | 0 | 3/6 |
| L10 | 17 | 11 | 7 | 4 | 0 | 4/7 |
| L28 | 56 | 10 | 4 | 4 | 2 | 6/4 |
| L27 | 54 | 11 | 7 | 4 | 0 | 3/8 |
| L19 | 69 | 12 | 9 | 2 | 1 | 3/9 |
| L20 | 95 | 12 | 2 | 9 | 1 | 6/6 |
| U08 | 150 | 15 | 5 | 7 | 3 | 10/5 |
| L25 | 257 | 29 | 16 | 8 | 5 | 13/16 |

逐key的原审查、首应用完整before/after、当时reviews、当前完整row、是否可由本reviewer支持撤回及其他独立同key证据，均保存于 [application-review-final.json](application-review-final.json)。输入文件SHA256和102个已引源码文件的当前SHA256也在该文件，不能用它们替代未执行的运行验证。

## 撤回原确认的11个完整key

下表只撤回原确认差异对状态下调的支持；不把这些项重新认证为等价。原始 [findings.json](findings.json) 与首应用保留。每项下一步可复现场景见JSON中的 `returned_confirmation.next_check`。

| 完整key | 更正理由 |
|---|---|
| `L10:configs:cardFold_proportion@archive/upstream/noname-liuli/noname/extension/史诗卡牌/extension.js:2215` | 上游有30%至100%比例，EpicConfig和包资产未建该字段；目前只有字段/文字搜索，未定位完整手牌布局配置入口或验证折叠宽度，撤回‘比例效果缺失’的确认程度。 |
| `L10:configs:分割线02@archive/upstream/noname-liuli/noname/extension/史诗卡牌/extension.js:2258` | 上游是line.png菜单分隔图。EpicConfig没有此项不能单独证明产品没有等效菜单装饰；没有在实际菜单观察或截图，因此撤回‘纯菜单装饰缺失’确认程度。 |
| `L19:configs:zuan_brawl@archive/upstream/noname-liuli/noname/extension/末日乱斗/extension.js:3363` | doomsday.rs已有off/standard/random/all/daily五值、默认all和完整解析。原‘枚举未实现’不成立；入口、daily跨日持久化目前仅未找到调用，没有产品反例。 |
| `L20:ui_hooks:_xigua_zhuwanzhongji_@archive/upstream/noname-liuli/noname/extension/标记补充/extension.js:2290` | 实际优先绑定ZhuWanzhongJi；死亡者为Lord/持有者为Loyalist的过滤及身份交换已经存在。原‘L20BatchA只计flag、没有继任’理由错误。mingzhong/zhuUpdate与完整死亡时序未在本次执行核验。 |
| `L25:configs:bizhi@archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:50762` | 原证据仅为ZuanConfig四键、包资产及activeWallpaper状态中未见bizhi；activeWallpaper属于对局状态，但未追完菜单壁纸入口，不能据此确认枚举/保存转换缺失。 |
| `L25:configs:hidessr@archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:51168` | 原证据仅为ZuanConfig无hidessr和SSR目录始终登记。目录存在不证明默认AI/玩家可选；未追完选将过滤或实际开关，撤回‘缺默认隐藏/将池排除’确认程度。 |
| `L25:configs:newhuihe@archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:50572` | ZuanConfig及ZuanUiRuntimeState没有旧四值枚举只是模型检查；未追完实际阶段动画入口或观察四分支，撤回‘四种配置分支缺失’确认程度。 |
| `L25:configs:perspective@archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:50757` | 仅查ZuanConfig和祖安UI状态无perspective；实际牌桌也可能由通用布局控制。没有开关前后同座位截图/尺寸，撤回‘透视参数未实现’确认程度。 |
| `L25:configs:showboss@archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:51163` | 包配置无showboss和未查到旧键不证明普通模式将池没有等效扩入入口；未追完各模式候选注册，撤回‘跨模式开关缺映射’确认程度。 |
| `L25:configs:text@archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:51404` | enable_editor/offline_debug_only的解析和拒绝条件是确定存在的原生门禁，但原表未追完text到当前产品编辑器的映射；键名不同及解析门禁不足证明全场编辑行为反例。 |
| `L25:configs:windowHigh@archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:51344` | 上游当前菜单仅列0，旧1至7选项已注释但onclick仍保留历史值分支。原生模型未列windowHigh不证明宿主窗口/旧值迁移失败；未测试桌面窗口与重启。 |

`L25:skills:_guanmodeskill@archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js:42978`另有路由更正，仍保留近似支持：`host_skill_bindings.rs:548`在L25 fallback前已绑定L19DoomsdayBatchB，实际进入 `l19_batch_b.rs:813` 的防翻面、回合摸牌及伤首名他人分支；与上游主公开局授guanfu/guifu_nei及生命/身份初始化不同。撤回原「L25通用展示桩」解释，不能继续引用未执行到的L25分支作为实际实现。

L10 cardFold/cardPhantom/shichangshi保留具体静态差异支持，是因为上游init=true，而包资产与parser默认false存在直接值反例；本次不借此宣称实际画面、技能替换消费或配置持久化已经核验。

## 其他独立同key撤回

本reviewer原待核验的 `L27:ui_hooks:sbqingzheng`、`L20:ui_hooks:_mianshang_`、`L20:ui_hooks:_shanghaipaishiyongtexiao_` 是应用清单中额外撤回的3条。首应用reviews均指向 `/root/repair_doc_refs` 的 [nonrule-audit/findings.json](../../nonrule-audit/findings.json)，分类属于脚本允许的具体差异，key完全相同；没有由本reviewer的gap自动放行。本次仅核查其应用依据，运行语义不由本reviewer重新认证。

## 应用链与元数据

HEAD清单SHA256：`4792c24d1855c8c156345ce5d013a348b849236aa779973bd51107e236605033`。

首应用完整 [inventory-application.json](../../inventory-application.json) 的after SHA256：`45629cc212b1a89ee91bdaaff7764bf522e9029a4bd339dc656d1e21aea27b96`；最初378行改动、226条状态撤回如实保留。

更正01至05后，本次输入清单SHA256：`2658808bbe361ab248e53166157e25606c47155eb31fa19947cb601eb3f35c89`。更正链全局有效撤回为215条，作用是撤回历史声明，不是215条规则已修复。该全局数字仅作字节重放与记录核对，不认证本任务141项以外的语义。

首应用每个before与HEAD原row一致；依序重放首应用及5份更正记录后，序列化字节与本次清单完全一致；4424个key及样本类别/id/upstream_ref保持不变。更正04/05仅记录difference字段变更的格式也纳入重放。共同日期note作为出处注记可以重复，不能充当逐项复核结论。

本次新变更的复核人/日期只来自独立reviewer。11条退回项恢复历史status/reviewer/date；L10 _emojiRebate1原先已近似且reviewer/date为空，未被本次伪填成等价复核。全局没有从其他状态新升为等价的项，141样本中也没有给保留的历史等价行新填复核元数据。

曾检查到L20两CSS行、两效果hook的新difference各有一组逐字重复，已通知主agent；更正04/05加入各自的样式对象、触发与动画条件后，本快照141项新增实质difference没有逐字重复。历史未变的「无」等旧句没有被本次冒充新签署。

## 命令、结果与限制

实际只读应用复审命令（没有执行会改清单的apply-reviewed-inventory.py）：

```bash
python3 -B docs/operations/evidence/2026-11-13-queue1-review/independent/other-sources/application-review.py --label final > docs/operations/evidence/2026-11-13-queue1-review/independent/other-sources/application-review-final.raw.txt 2>&1
```

结果：exit0，1136项证据/应用一致性检查通过，0失败；这不是对局测试数量。[完整原始输出](application-review-final.raw.txt)。此前 [initial原始输出](application-review-initial.raw.txt) 因审查脚本未支持更正04的字段级格式而KeyError退出；[checked01补跑](application-review-checked01.raw.txt) 支持该格式后1100项通过；更正05纳入final再跑1136项通过。原失败未删除。

本reviewer未运行Cargo、浏览器、真实对局、保存恢复、配置界面或桌面窗口测试。68条只支持具体静态差异的状态应用；47条待核验和26条静态未见差异尚不满足新等价签署要求。12来源765项中的624个样本外key未审；其余来源、U01/juxian修复、门禁、发布与CI也不在本次范围。
