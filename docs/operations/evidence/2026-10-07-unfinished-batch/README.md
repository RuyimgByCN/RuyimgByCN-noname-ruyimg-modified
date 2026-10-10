# 2026-10-07 未完成任务批次核验（P0 解码加固 + P1 六桥迁移 + A0 第 3 点 + P3 警告）

本批次完成主稿待做表中可闭环的四项：P0 存档解码加固（10-06 探针复现的两个
缺陷关闭）、P1/C0 其余六条桥接迁移（E02 装载器收尾）、演进提案 A0 第 3 点
（SkillDefId 只追加声明表）、P3 编译警告（engine-native lib 零警告）。
上一轮：[2026-10-07 进度核对](../2026-10-07-progress-check/README.md)。

## 检查与结果

| 检查 | 结果 |
|---|---|
| P0 未知牌种编号 | `num_to_card_kind` → `card_kind_from_num(n) -> Result<CardKind, ArchiveError>`，新增 `ArchiveError::UnknownCardKind(u32)`（保留完整编号）；4 处调用（`use_as`、两处 `need`、`declared_kinds`）全部传播，不再 panic。回归：declared_kinds 编号字节改 255（双快照差分定位）→ `Err(UnknownCardKind(255))` |
| P0 载荷读尽 | `decode_archive` 在 `dec_snapshot` 后检查 `r.pos == payload.len()`，新增 `TrailingBytes { extra }`。回归：payload 尾部夹带 4 字节（重算长度与校验和）→ `Err(TrailingBytes { extra: 4 })`，不再被静默接受 |
| P0 编码表往返 | `CardKind::ALL` 86 个牌种经 declared_kinds 路径编码→解码往返相等（`card_kind_encoding_roundtrip_all_variants`）；`enc_winner` 的势力分支改用显式 `enc_faction`（与 `dec_faction` 同源），`Winner::Faction(Wu)` 往返 + 非法势力值 5 → `BadTag("faction", 5)`，原 dead_code 警告消除 |
| P1 六桥迁移 | huanle/hearth/gwent/yunchou 四包 `cards.json` 补 `pack` 段（欢乐改 `skills[].mapping` 加 `host:` 前缀、range 仅武器、苦无 `equip_skills`；索链甲 `link_on_equip`；炉石键规范化 `leishajian→leisha`、`shenenshu→jihuocard`）；d3 `character.json` 补 pack 段；wuxing 新增自描述 `cards.json`（wuxingpan + 生克/盘属性技能 + `init_hook`）。六份测试 `bridge()` 全部改调 `content_native_loader::bridge_pack`/`bridge_generals`：huanle 6/6、hearth 7/7、gwent 6/6、yunchou 12/12、d3 3/3、wuxing 3/3——测试名、数量与结果不变 |
| 装载器扩展 | `cards` 缺省为空（纯武将包合法）；`generals` 段合并进 `bridge_pack`（宿主技能次序：伴随技能 → `skills` mapping → 武将命中声明，去重异行为报 `DuplicateSkill`）；`link_on_equip` 字段解析 |
| A0 第 3 点 | `standard_skill_ids_append_only`：standard 目录 46 技能名序声明表（v60 实序锁定），中间插入/重排/删除立即失败，只允许尾部追加 |
| P3 编译警告 | `cargo fix` 清 12 条 + 人工处理 4 条（两处无效 `drop(&引用)` 删除、`JiJiStage::Retarget` 保留变体加 `#[allow(dead_code)]` 注明 v2 字节兼容、`enc_faction` 随 P0 接线）——**engine-native lib 零警告** |
| 全量验收 | engine-native **347/0**（345 + 新增 2 项回归）、content-native-loader 1/0、content-runtime 31/0，合计 **380 通过 / 0 失败**（[test-logs.txt](test-logs.txt)） |
| 兼容与版本 | 规则 v60、格式 v2 不递增：14 个 golden 场景通过（合法 v2 编码字节不变，加固只影响非法输入路径）；未改动的古剑/玩点论杀/cardpile 包数据与 10-06 基线逐字节一致 |

## 结论边界

- 已实现并验收：P0 两缺陷关闭 + 编码表往返、六桥迁移（E02 装载器不再依赖
  测试逐包代码）、A0 第 3 点声明表、P3 lib 零警告。
- 装载器新增非法输入负例随测试运行；14 golden 与全部 G1 行为/恢复断言保留。
- 未做（仍为主稿待做）：P1/E01、G0 功能清单（37 来源）、E03 开放规则宿主、
  C3 冲突/依赖/回滚场景、P2 规则对齐与整包（0/37）、N10/N12 AI 与产品、
  O1–O5；名称类型 `Arc<str>` 深化；`deny(warnings)`/CI `-D warnings` 评估
  （P3 尾项，lib 已零警告可接）。
- 14 场景由一个 golden 测试核对；全绿不等于完整规则或整包验收。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（shasum -a 256，30 文件，相对
noname-next 根）。
