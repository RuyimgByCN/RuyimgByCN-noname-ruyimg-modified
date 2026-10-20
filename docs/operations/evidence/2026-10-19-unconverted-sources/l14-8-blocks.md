# L14（手杀ui）8 个「疑似规则」逐条判定：**全部是 UI**（2026-10-19）

分类脚本给出 L14 有 **8 个「疑似规则」**（含 `content:` + `player.`/`game.` 等文本特征）。
本轮**逐条读块体**判定 —— 结论：**0 个规则技能**。

## 逐条证据

| 技能 | 大小 | 文件 | 关键 UI 特征 |
|---|---:|---|---|
| `_jd_ddxyA` | 697 B | `extension.js` | **`game.as_showImage(.../ddxy.jpg)`** |
| `_jd_ddxyB` | 689 B | `extension.js` | **`as_showImage`** |
| `_jd_ddxyC` | 1386 B | `extension.js` | **`as_showImage`** |
| `_jd_dfsk` | 863 B | `extension.js` | **`as_showImage(.../dfsk.jpg)`** |
| `_wmkzSayChange` | 609 B | `extension.js` | **重写 `player.say` + `game.addChatWord`** |
| `_jd_ddxyA` | 694 B | `fonts/extension.js` | **`as_showImage`**（同源副本）|
| `_jd_ddxyC` | 1383 B | `fonts/extension.js` | **`as_showImage`**（同源副本）|
| `_wmkzSayChange` | 553 B | `fonts/extension.js` | **`player.say` + `addChatWord`**（同源副本）|

## 结论

1. **6 个是「显示图片」**（`game.as_showImage`）、**2 个是「聊天记录」**（重写 `player.say` 并把发言写入聊天框）；
2. **没有一条改动牌局状态**（无伤害/摸牌/判定/技能效果）⇒ **L14 的「规则技能」= 0**；
3. L14 与 **L11（0 技能）/L12（`lib.skill` 0）** 同类 ⇒ **「UI 面不迁」（ADR 0782）⇒ 不属引擎迁移范围**；
4. 分类脚本把它们的「`content:` + `game.`/`player.`」判成**疑似规则** —— **这正是文本特征分类的上限**，
   必须**逐条人工复核**才能定论（本轮即示范）。
