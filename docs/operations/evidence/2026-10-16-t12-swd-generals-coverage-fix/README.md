# T12 swd 武将面数据覆盖修复：轩辕剑仙去重 + 陈辅补入

- 日期：2026-10-16
- 范围：**cards.json 数据完整性修复——武将面 79/79 名全覆盖达成**（不含代码行为变更）
- 背景：批 81–91（先前提交）后复核发现两处数据缺损：
  1. `swd_xuanyuanjianxian` 条目重复两次（批 75 与后续批各加一次，双 entry 掩盖缺口）
  2. `swd_chenfu`（陈辅：玄囹 xuanzhou+冰封 bingfeng，qun hp4 男）整将缺失

## 修复内容

1. 移除重复的 `swd_xuanyuanjianxian` 条目（两 entry 完全同构，去重无损）。
2. 补入 `swd_chenfu` 条目（两技均 deferred：xuanzhou 锦囊存储面/bingfeng limited 翻面面）。
3. npack 双哈希重算（rehash_cards_asset + rehash_npack）。
4. 首版补入脚本误将陈辅块落入顶层 `skills` 节（非 generals 数组）——json 结构校验发现后移正（skills 节保持 3 项扩展技能定义不变）。

## 勘误（入记忆）

- **多段 json 插入锚点**：cards.json 存在顶层 `cards`/`generals`/`skills` 三数组——插入将条目的锚必须绑定 generals 数组内末条目的闭合+`,\n  ],`（generals 结束符），宽松 `\n    }` 锚会命中 skills 节内部；插入后必须 json.load 复核 count+成员性双断言。

## 验证

- 上游键交叉核对：upstream character.js 79 键 vs pack 79 将（中文命名条目为寒洛/犀衍/子大暑/张诰/轩辕剑童的手写标签漂移——技能映射与上游一致，仅显示名非上游正式名，登记不改）。
- **全量**：engine-native 808 passed / 0 failed；swd 套件 84/84。
- **Gate 1b**：exit 0。

## 结论

- **swd 武将面数据覆盖 79/79 名**（184 技条目：116 已映射 + 68 项 deferred 复杂面挂账）。
- 四项用户决策维持待用户裁决清单登记（T15③ 排期/十殿阎罗族级 D/龙血玄黄立项/O5-O4 产品）。
