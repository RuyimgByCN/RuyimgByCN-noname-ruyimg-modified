# 2026-11-14T03:29:13.975149+08:00 三来源纯文档批本地核验

源码HEAD `5c73d1d9158145e33f98f2018013914667d13964` + 文档/证据dirty；规则333／NGSA107／schema5／ABI759／WS8。产品/测试字节沿用已发布且精确CI首次10/10成功的酒诗修复。本次无新代码、版本、ADR或协议变化；不重跑已通过且输入未变的产品全门禁。

完整执行 `python3 -B scripts/preflight.py`，没有--quick或其它过滤，21项通过／0失败；实际命令、开始/结束时间、退出码和完整preflight输出见[元数据](restore-fix/24-preflight-documentation-batch.json)／[日志](restore-fix/24-preflight-documentation-batch.log)。其中包含脚本单测、全部本地链接、文档引用、来源/台账/分层与物理/有效行数门禁；21是检查项数量，不伪装成单元测试数量。独立agent此前瞬时缺少尚未落档清单的首次链接失败保留在[辅助最终核验](independent/u08-auxiliary-adjudication.json)，不因本次绿色覆盖。

L10／U08／L28采集与各自独立目录均已逐文件冻结。U08两辅助补充JSON仅追加verification栏，移除此栏可逐字节重建早期hash；[最终冻结差别核验](u08-auxiliary-final-freeze-check.json)保留前后记录。[外部条件公开副本来源](external-prerequisites-publication-redaction-01.json)保留本机未改的原始字节，只隐去公开设备名称/标识，状态和命令不变。最后追加状态段和明确stage审计后，另跑全部本地链接检查与自有文档diff检查；结果另记，不预称通过。

当前[625／3799／0](source-counts-runtime-04.md)、新增等价0，队列1仍部分、八项0/37。文档尚未提交/发布；实际stage/提交/推送与精确文档SHA CI另留原始证据。
