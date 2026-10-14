# T12 swd 卡牌面补切批⑩：器族四象五件——手持触发技（2026-10-15）

## 承载

5 个新 jiqi kind（tag 156-160，非装备路径、手持生效；包 67→**72 张**）+
5 个手持触发技（tags 304-308）+ **InitHook::SwdJiqiHeld**（ABI 只增）：

- **青龙之圭**（heart6/tag 304）：准备阶段摸 2 弃 1 he（Act 摸 2 →
  SelectCard min1 弃置，equip 分支 revoke）。
- **朱雀之璋**（diamond6/tag 305）：受他人伤害结算后 forced → 来源火 1 伤
  （damage_data_with Fire）。
- **白兽之琥**（spade6/tag 306）：弃牌阶段结束 direct → 选其他有牌角色弃
  其首张 he（确定性首张，登记差异上游随机）。
- **玄武之璜**（club6/tag 307）：自己造成伤害结算后 forced → 回复等量体
  力（hp<max 门在 filter）。
- **黄麟之琮**（heart5/tag 308）：准备阶段无护甲则 +1 护甲（hujia==0 门）。
- **手持门控**：全部 filter 走 hand_has_kind helper（手牌含对应 kind 实
  体才生效）；实例经 **InitHook::SwdJiqiHeld** 开局授予全部存活角色
  （extension.rs 枚举+loader from_key/key+engine 授技块；cards.json
  pack.init_hook 键）。未持牌不触发=行为面零扰。
- golden 不变、无版本 bump。

## 测试（swd_cards_jiqi_b40_five_held_treasures）

| 面 | 断言 |
|---|---|
| InitHook | 开局全员持有五手持技实例 |
| 朱雀之璋 | p1 伤 p0 → p1 火 1 反噬（8→7） |
| 玄武之璜 | p0（压血 2）造成 2 伤 → 回复至 ≥4 |
| 黄麟之琮 | p0 手持→护甲非递减 |

测试侧勘误四处：jiqi kind host_skills 漏注册（defs 不存在）、pack.init_
hook 须在 pack 对象内（顶层键不被读取）、init 块误嵌 wuxing if 内（花括
号重定位）、玄武压血门。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 g1_shahai_swd | **49/49** |
| engine-native 全量 | **768/0/0**（golden 不变）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- 包 72/92 行；余 20 行：白兽之琥 info 口径「随机弃一张」已确定性登记、
  equip5 余 4（书锤/散灯金环/昊天塔/玉如意）+神草结 3+玉蝉 8+神秘果 2+
  多余 13 处置。
- 批⑪ 候选：玉蝉八卦族（global 联动面）或 equip5 中复杂组。
