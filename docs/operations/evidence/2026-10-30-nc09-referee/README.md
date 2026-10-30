# 裁判授权投影、选择权限与五窗口恢复整合（2026-10-30）

源码基准 `100489f5` 加本批工作树；规则328/存档105/原生ABI753/WS5不变。原补丁基于 `f6b7bb57`，独立审查和必要恢复修复后接入主树；当前原生合并回归及全工作区/当前产物的本轮功能验收已通过，边界如下。本批不包含裁决21已授权但尚在另一工作树实施的双将/精确选择元数据wire新增字段。

## 授权与实现

普通Player、Spectator和受信Referee共用观察者投影。Referee可见实际物理牌区、当前完整有序牌堆、身份、两个原生将槽及原生完整选择；普通观察者仍按既有隐藏信息契约过滤。服务端仅为共享视图授权的牌ID查牌面，裁判不经原始状态旁路补数据。裁判来自成员及恢复令牌，不从客户端role/observer/seat声明授予，且没有legal_actions、submit或replay权限。

choice_for改为可信role及checked座位转换，API ID0/257/MAX不再绕回0号座位，Spectator即便附带合法seat也无私有选择。全部actor调用方同步。重连后actor成员按实际session connection id同步，避免旁观者记录挤掉Player、旧Referee连接继续收私有fanout。已存在的三种wire kind精确映射，其余Custom继续保留；choice token/id/revision/续行含义未变。

物理所有权使用同一共享枚举及park机器collector。手牌、装备、判定、deck/discard/table、off_court、四类真实技能牌区、VM piles和真实machine-held卡各计一次；历史、选项、引用不成为额外实体。复审发现原collector缺XunXun/JiJie/GaiMing/HaoTianTa，补齐实际持牌阶段；GaiMing仅认可judge_card与首候选的确切alias，不全局去重掩盖非法重复。

五个真实挂起窗口的ExpectedAsk、既有prompt白名单及缺失factory已补齐：XunXun选四存二、JiJie底牌交人、GaiMing选判定、HaoTianTa双候选、LiaoYuan同花色弃牌。LiaoYuan的Host.card是suit标量，Host.cards是弃牌历史；只读HostAskHook传入既有scalar以派生原选择，不将它们误当物理持牌。没有新增状态字段、codec布局、游戏选择或规则语义。

## 审查与真实证据

[首次独立审查](independent-review.raw.txt)阻止遗漏四个窗口的物理牌；[修复复审](review-fix-independent-review.raw.txt)确认该P1及恢复接线修复。原始red、OpenChoiceMismatch、UnknownMachine和最终green均保留在isolated/与review-fix/，没有用新期望清除失败。

隔离最终源原生三包1665/0/73、server383/0、WASM72/0，严格三包Gate及preflight21/21通过。五窗实际WASM32 **75状态比较/270座位隐私检查/35命令/40次字节重启/0失败**；普通同档投影40比较/32隐私检查通过。五份修复前后挂起archive字节SHA完全相同，后续原生比较全core/resolution/events/RNG/choice，而非只看终局hash。详见[最终命令索引](review-fix/final-combined-commands.json)、[测试计数](review-fix/final-test-counts.json)、[字节比较](review-fix/final-archive-byte-comparison.json)。这些计数只属于隔离源。

主树与PA09共享session接线后六包完整 **2143/0/74，exit0**，见[主树命令结果](../2026-10-30-pa09-desktop-native/validation-command-results.json)。完整workspace首次ENOSPC失败保留，恢复后独立全量 **6782/0/75**、实际Worker82/0、前端146/0、三包严格检查及完整preflight21/21通过。主树重录的五窗实际WASM32仍为75状态比较/270座位检查/35命令/40字节重启/0失败，普通同档40比较/32隐私检查/0失败；没有修改旧期望或规则。WKWebView280步身份局IPC/Worker一致、VM生命周期和12次文档重载亦通过，详见[主树完整命令](../2026-10-30-pa09-desktop-native/recovery-command-results.json)和[验证交接](../2026-10-30-pa09-desktop-native/main-acceptance-handoff.raw.txt)。误用未注册测试target的首次命令失败单列，随后已注册misc定向18/0通过。

父应用总26路径patch时保留其字节指纹；WASM lib中的新ChoiceView字段匹配改在PA09抽出的session.rs，保持旧JSON形状。没有照抄或覆盖隔离J2的329字段/回调/提示表，未来J2整合须按hunk保留两侧变更。原始总/增量及逐文件hash见[最终清单](review-fix/final-source-manifest.json)。

## 未关闭范围

复审另报既有GaiMing取消物理重复；父现已用真实Boss挂起窗口独立复现：取消前124张、后125张，牌65重复，恢复明确报`card_in_multiple_zones`。见[原始red](gaiming-cancel-red.log)、[实际复现源码](gaiming-cancel-repro.rs)及[编译/执行/清理指纹](gaiming-cancel-result.json)。取消将首判定占位牌退回deck后仍最终discard同牌；现有绿色trace选择候选而未覆盖取消。这是独立现存规则路径问题，尚未修改规则源码/版本；须修共享根因，不能用collector去重或放松恢复门禁掩盖，也不能据本批声称全部取消分支已验收。证据编译只读复用已验收rlib，临时可执行文件已在运行后删除。

现有wire仍不能表达第二将槽完整信息及prompt/两种精确native kind；裁决21已批准下一批完整接线和兼容测试，本批只完成旧协议可表达部分。超时330/WS6、选序331、NC10剩余Host/opaque信息边界、平台真机/CI和全部37来源继续按各自合同推进。本页不声明NC09/NC03整体完成、所有上游等价或平台验收通过。
