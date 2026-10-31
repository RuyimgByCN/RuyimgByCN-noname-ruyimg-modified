# NC10公开Own纯规划修复与限定验收（2026-10-31）

基线Main `5fc0f77403d90fa4555156bf01722450cbe7bf3b`，330/105/758/WS6。最终六文件已精确接入Main并完成限定验收，按裁决18入库 `d07b35f94a42b282222dda4ab7bbc5c39ab8fddb`，随后按25正常发布至native-convergence；远端main保持。[本次exact-head CI失败已独立登记](publication-ci/README.md)，两个Windows checkout失败的局部longpaths修正另行审查并接入，真实新Windows结果待后续CI，不能挪用旧5fc十绿。总体Goal授权本次现有API修复，沿用 `plan_action -> Plan`；裁决30属于I02，不是本修复的授权来源。

Own分支在时钟、克隆、遮蔽、候选枚举及模拟前返回，和NativeSession共享公开选项选择器。真实P/Q完整公开输入及事件相同，完整Plan由nodes4/11差异修为相等，两个建议实际提交均Pending；后续续行仍不同，不构成执行证书。v2只在allow_cancel为真且选项数不合界时清成空答案，否则保留确定性建议；不隐藏搜索或重试正式提交。

- [初始实现与五文件复审](implementation/implementation.raw.txt)、[v2取消边界复审](implementation/ae2d0aea-source-review.raw.txt)及原30命令保留。初次缺fixtures的76失败、随后1718/1/75的seed8211旧484断言得到119，分别保留，不冒写全绿。
- [最终六文件及语义回归处置](final-regression-disposition/README.md)保留原484成功和119红；测试检查900操作内终局/P3、完整规划不变性、正式成功、零模拟/探针故障与完整恢复，未改成断言119。最终私有1719/0/75及十条定向/门禁通过，父核同源后接Main。
- [self-play质量原证据](quality-outcomes/summary.json)记录各200种子/局、135,200次Accepted/Pending、400终局恢复；[固定旧参考对手](fixed-reference-pilot/README.md)记录32种子×4挑战席×两臂、84,344次成功返回和256终局恢复。胜32/128对31/128是相关种子的描述性结果，不能宣统计非劣或普遍强度。合计219,544成功返回/656终局恢复，原逐行流无损gzip保留，测量二进制私有留存。原校准Own数3→4、session数7→6另存勘误，不改原报告。
- [Main功能、产品、最终门禁与独立复审](main-acceptance/README.md)包含当前Main原生1719/0/75、workspace6922/0/77、前端179/type、独立Gate1b、当前发布WASM/实际Worker与UI、完整存档和40+20轨迹。父核714原产物及26命令；原探针21个依赖版本漂移另补Main-lock30外部条目的严格离线探针，7组完整存档与当前WASM逐字节相等，父审计及补充复审通过。原网络旁路和未知index变化均如实保留。

seed8211第61步暴露当前正式规则缺口：seat1的Jiu85可救P0，Card与Pass均被330正式接受；上游要求self或真实jiuOther资格，天仙酒还须区分仅self语义。这里未通过AI禁酒掩盖，也未改规则或restore，见[输入及来源审计](dying-wine-source-audit/parent-source-audit.json)与[规则基准](../../../architecture/native-rules-baseline.md)。

完整NC10、no-choice/Other/descendant VM/private effects、原19 NoCert研究、一般质量/上游等价、全部37来源与实体平台验收仍未完成。当前状态统一见[进度主稿](../../../architecture/native-progress.md)。
