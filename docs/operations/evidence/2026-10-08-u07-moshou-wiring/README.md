# U07 玩点论杀：莫守装配接线（2026-10-08）

台账 U07 明确的小步转换：「把莫守实际装配到费诗」。莫守宿主行为
（`host:moshou_immunity`，延迟锦囊目标转移免疫）此前已有实现与授予
驱动测试，但费诗在 generals 段的声明仍 `mapping:null`——装配/明置
授予路径读不到该技能。

## 改动

- `cards.json`：费诗.generals.skills.moshou `mapping: null →
  host:moshou_immunity`（skills 段已有同映射，两处对齐）。
- npack 重算：cards.json `source_hash`（blake3 of bytes）+
  manifest `asset_hash`（`rehash_npack`）。
- 测试：`feishi_moshou_declaration_wired_to_host_behavior`——桥接后
  目录元数据费诗 skills=`["moshou"]`（装配/明置授予路径据此自动
  挂接）；未映射声明计数 10→9（既有断言同步）。

## 验收口径

- **回归已加入**：上述测试（目录元数据断言；宿主行为本体由既有
  `moshou_blocks_delayed_trick_target` 授予驱动覆盖）。
- **全量通过**：engine-native **397 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**，DEPS_EXIT=0（数据接线，规则 v84 不变，无 golden
  变化）。
- **剩余**：其余 9 条技能声明转换（调兵遣将×2 完整语义、诱敌深入
  完整化、思继外的 8 将技能）——G3/大 批次。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（6 文件，含转换包两文件）。
