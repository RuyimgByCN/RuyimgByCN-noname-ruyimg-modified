# C331 phase8：schema5数据基础与未闭合入场边界（2026-10-31）

本目录冻结私有C7上的35文件增量／完整166文件组合，基线Main5fc。声明、编译、哈希、SDK与native入场基础已有实现和测试；**原冻结的复审发现两P1与一P2，原结论保持BLOCK**。后续[7文件实际入场修复](admission-repair/README.md)经真实WASM／SDK红绿、父核验与复审关闭三项问题，仅限定接纳私有C8基础；C9真实Damage阶段已另行推进，整C331与Main合入仍BLOCK。

[实现报告](implementation.raw.txt)记录准确能力与未实现项；[原源码复审](source-review-v1.raw.txt)指出：modules缺失时asset-only fallback跳过声明守卫；实际导入链规范化JSON后丢失重复字段；active executor的SDK互斥校验遗漏。修复已在同一私有writer的独立admission-repair阶段完成限定接纳，原冻结、失败、报告与artifact-index保持原样。

- [最终35清单](final-freeze8-v2/source-manifest.json) SHA `852ead19b0098ca13a329040dd95aa7efb728b28bb60ef6fa44266980513c494`；[完整166清单](full-source/source-manifest.json) SHA `b34ae2357add02177ac30c9cf4c4c9d1a204e3014c39bc367132549869f904d1`。前者原补丁格式错误已修为新artifact，未更改源码，失败副本保留。
- [父独立结果](parent-independent-results.json)核166当前源、35增量与pre-full2相等、全部原结果计数、版本／授权和暂存区；[父artifact核验](parent-artifact-index-fresh.json)另核130证据及报告SHA。父不重跑producer。
- [完整命令绑定](results-bound.json)及原logs记录：四内容包609/0/0，最终原生三包1773/0/75，前端183/type，独立target `RUSTFLAGS=-Dwarnings cargo check --offline -p engine-native --all-targets`、行数自测2／source9019／分层0通过。范围重叠，不累加，不以绿测覆盖复审缺口。
- 原3个红用例、坏fixture编译、前端原诊断回归、首次原生setup未执行、后续test_registry单失败、补丁reverse-check失败均保留。原helper与锁缺失的环境中断，经父确认无活动重任务／旧锁FD后，从已存原脚本逐字恢复并实测互斥与退出码，见[恢复记录](coordination-recovery/restoration.json)。原消失原因未知。

schema3缺省与schema4非默认tuple旧bytes/hash正控保持；schema5字段presence/hash数据链并不产生执行权限。provider/lifetime/death/out/indexed/named windows/reveal/rescue/cleanup等实际消费者仍未实现；未知／未支持机制必须受控拒绝。onremove String仅私有中间表示，未完成VM cleanup或SDK全部能力对等。

私有331/NGSA106/ABI758/WS6，Main仍330/105/758/WS6；格式1/API1不变。来源排序262/290、剩28、16译名及全部37范围不变。[C29合同](../2026-10-31-c331-runtime-contract/README.md)和[进度主稿](../../../architecture/native-progress.md)保留后续顺序；本目录不证明全C331、当前C WASM／UI／WAL或上游等价。
