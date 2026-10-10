# ADR 0776：E01–E06 扩展运行时基础与验收

日期：2026-09-23（第七十七批）
状态：**局部原型已落地，路线图 E01–E06 均未完成验收**（2026-09-23 第 15 轮独立复核）。
[本轮实跑](../operations/evidence/2026-09-23-native-review-15/README.md)：`ExtensionRegistry` 只在内存管理静态名称；`merge_extension_cards` 只把示例卡追加到临时 `Vec<CardDef>`。正式 `Engine::new`/`restore` 仍构造标准目录；没有磁盘包导入、启停后重建目录、扩展技能执行/选择/保存恢复，也没有两仓 37 来源的真实扩展验收。测试名 `e04/e05/e06` 与路线图 E04（模式/AI/UI）、E05（联机/存档）、E06（全量迁移）并非同一验收。
上游：[0775](0775-native-p2-n08-n05n06-closeout.md)

## 交付

1. **E01 扩展包类型**：`content::extension` 模块——`ExtensionPack`
   （元数据 + 额外武将 + 额外卡牌 + 启用标志）、`ExtensionDef`（名称/
   版本/描述）、`ExtensionGeneral`（武将条目）、`merge_extension_cards`
   （列表追加 API，尚未进入引擎目录）。
2. **E02 内存状态**：`ExtensionRegistry`——内存安装/启用/禁用/卸载流程，
   BTreeSet 保证确定性。示例扩展 `example_extension()` 提供 1 张
   「赤闪电」+ 2 名扩展武将。
3. **E03 列表原型**：`merge_extension_cards` 将扩展卡牌追加到测试中的基础卡牌列表；扩展武将只有条目和计数函数，尚未追加到运行引擎的武将表。
4. **名称冲突片段**：基础卡牌名撞名时 `merge_extension_cards` 触发 panic；另有 `check_general_conflicts` 检查与基础武将名冲突，但正式建局不调用。扩展内部重复卡牌名仍通过，尚无包间冲突或原子安装错误返回。
5. **元数据检查片段**：`e05` 重查临时列表中「赤闪电」的名称/类别/花色；`e06` 检查示例武将引用的基础技能名和体力范围。示例卡仍按标准 `ShanDian` 种类建模，注释中的「火焰 2 点」没有对应规则实现。

## 验收

- `e01_extension_registry_lifecycle`：注册表全流程正反回归。
- `e03_extension_cards_in_catalog_and_game`：名字虽含 game，实际仅检查临时列表追加后的数量与卡牌字段，没有在对局中发牌、出牌或结算。
- `e04_extension_conflict_detection`：基础卡牌同名会 panic；`e05_extension_card_in_game` 只检查临时列表属性；`e06_extension_general_metadata` 只检查元数据，没有进入真实对局。
- `content::extension::tests::extension_registry_lifecycle` 为第 6 项相关模块测试。全量 **238 passed / 0 failed**（相关 6 项）；通过不等于 E01–E06 验收完成。
