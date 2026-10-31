# C331 phase7：Main5fc与既有C6私有基础整合（2026-10-31）

核验基线为Main `5fc0f77403d90fa4555156bf01722450cbe7bf3b` 与冻结C6。裁决29授权的私有C331/NGSA106基础已完成源码与结果复核；Main仍330/105/758/WS6。本目录不表示C331整体验收或Main合入。

148文件整合保留C6、Main来源字段及legacy兼容，解决UI选择顺序组合，提取 `resolution::ready_park_index` 让来源读取与实际dispatch使用同一消费者。原始105字节被新106读者严格拒绝，旧读者仍可读原件；未迁移或重写旧头。检查点之后只改六个测试文件，未再改生产代码。

- [实现报告](implementation.raw.txt)、[检查点源码复审](checkpoint-source-review.raw.txt)、[最终测试增量复审](final-test-delta-review.raw.txt)保留原结论，均为限定范围No issues。
- [父独立结果](parent-independent-results.json)核148源码/tar、六测试增量、96项证据及原始结果绑定；[父解析器](parent-audit.py)和[执行记录](parent-audit.raw.txt)可追溯。
- 最终原生三包1761/0/75，nc08 77/0/50，前端181/type，独立目标 `RUSTFLAGS=-Dwarnings cargo check --offline -p engine-native --all-targets`、适配器check、legacy入场2、行数与分层0通过；命令/SHA/失败保留在[完整绑定](results-bound.json)。范围重叠不累加。
- [最终148清单](final-freeze7/source-manifest.json) SHA `0d7a18e9fd5225fd09f7902fb32172609ef10a8f783df97587eba37b716544cd`；对应tar SHA `d227cfd4ad67246359f3264e5e086e84073bd6de93bd1424e04e46ec109b0ec3`。

C8 schema5数据基础在另一个私有阶段；完整Damage/phase/provider/expiry/death/out/reveal/rescue/cleanup及来源262/290余28等仍未关闭。类型检查使用旧330生成模块只证明imports，不作C331实际WASM验收。当前状态与后续工作见[进度主稿](../../../architecture/native-progress.md)。
