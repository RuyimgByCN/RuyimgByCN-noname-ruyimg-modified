# I02 phase1入口修复限定验收（2026-10-31）

基线Main5fc，裁决30实施范围内的私有typed-wire基础；30文件组合、11文件修复尚未合Main，九个Rust生产constructor仍presentation=None。原29文件审查的两P1/两P2及后加孤立surrogate反例已修，历史BLOCK及原封存不改。

[完整修复报告](implementation.raw.txt)、[独立复审No issues](independent-review.raw.txt)、[父新核验](parent-audit.json)对应[最终30源清单](original/unicode-acceptance-checkpoint/manifest.json)；全补丁SHA `1b154aa7573fff3fc5b2fdfb54e5dbd743206c4c2b91f1e384050f38b5cdae94`，修复差异 `8d807c2fbad722f55b06d6fca435f78dcfac6a6eff83f78b740b953e57941708`。

- compound-invalid IPC先验证presentation，坏元数据把原成功信封交给既有uncertain路径，避免Source27另一错误先转普通失败并清receipt；合法/缺省元数据保持原Source27行为。
- network固定8在key/counter/map/storage/send前判断，恢复placeholder同界；同key重放不分配新key，第九项明确未提交，无新队列或收据台账。
- Rust/TS显式空白/控制集合一致，含0085/FEFF；TS拒孤立UTF16，保留合法astral。Rust直接注入原始坏JSON转义，避免先经Value导致空测。
- RequestSnapshot真实旧/新读者先成功解码，再验证新增C→S字段拒绝；不再用不存在的SnapshotRequest得到Error==Error。

最终Unicode源码完整前端441/441、Rust API/protocol12/0/0、typecheck通过；80坏输入=原47+27空白/控制+6孤立surrogate。此前422/330不是最终源码证明，原排队超时与真正执行的红分开。受影响callers check在Unicode补充前通过；补充没有改Rust生产体。父核最终source SHA/patch并解析原日志和exit，其余证据新建父索引，不冒称与不存在的worker全artifact索引逐项比较。

原5fc匹配fixture nativeWASM225f…只作为冻结旧输入，不声称等于当前Main30e97…；无新WASM/GUI/网络/语音运行。3GiB是free disk，旧误用RSS/AS限制的尝试保留为执行偏差，最终无该限制。仅接纳私有结构/入口基础；发生时公开权限、真实producer/资源成员、committed范围、C29/R05/VM死亡与全37/作者生命周期均仍待实施。
