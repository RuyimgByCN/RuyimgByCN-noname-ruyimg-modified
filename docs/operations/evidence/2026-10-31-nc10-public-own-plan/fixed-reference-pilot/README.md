# NC10固定参考对手pilot（2026-10-31）

[原报告](verification.raw.txt)保留原字节；[父独立解析](noname-parent-nc10-fixed-reference-audit.json)已核全部84,344提交、完整证据清单／二进制与临时源码绑定、256终局档SHA、24校准行、冻结源码恢复及空暂存区，PASS。父仅读原证据，没有重跑producer或再次执行恢复。

同一临时测量二进制中，所有非挑战席始终走可逐字还原Main5fc的旧规划路径；候选挑战席走v2公开Own分支。held-out种子46000..46031，每种子四挑战席、两臂：256局全终局，正式提交均Accepted/Pending，原Rust执行实际完成256终局NGSA解码恢复。挑战者胜场基线32/128、候选31/128；配对候选独胜5／基线独胜6／两者胜26／两者不胜91。

四个baseline挑战席标签重复同一seed轨迹，128对不独立；样本仅32种子／四人Prototype+swd无指定武将，候选步数较长。**不能声明统计非劣、全面强度或规则等价**。测量代码与冻结v2不是整文件相同；所有实际actor输入只有Own/None，未出现Other。限定域上的行为适用性已由[质量处置复审](../quality-disposition-review.raw.txt)核对：Own路径相同，None根的root sim不生成新询问，差异fallback在本样本不可达。这仍不是整文件身份或时限等价证明。

**原报告勘误**：报告写“3 Own choices”，而原 `calibration-actual.json` 及父重计为**4**；`own>=3`是断言下限。实际24行完整input／Plan／返回／hash校准与原baseline相等。原报告、摘要及SHA不改，见[父结果中的具体勘误](noname-parent-nc10-fixed-reference-audit.json)。

临时参考开关与测试均已恢复，不进入产品；原484断言及119失败保留。Jiu-other正式规则差异仍独立待修，不用AI特判遮蔽。原始522,400,812字节逐提交JSON已无损gzip保存并核解压SHA；测量二进制仍在原私有Q路径保留，见[复制清单](parent-copy-manifest.json)。当前验收处置与最终回归见[上级证据](../README.md)及[进度主稿](../../../../architecture/native-progress.md)。
