# 2026-10-11 T4 非 D1 部分：上游 42 提交复核与 inventory 基线口径

任务来源：[进度主稿 T4](../../../architecture/native-progress.md)；范围声明：史诗卡牌 E01 复核部分仍待 D1（用户决定），本批完成其余全部。开工 HEAD `a27b8959`（领取锁）。上游范围 `5c02a745..df2682e6`（noname，10-08 快进的 42 个提交）。

## 逐面复核结论

| 变化面 | 上游内容（实测 diff） | 对原生的影响 |
|---|---|---|
| `apps/core/extension/` | **0 文件变化**（42 提交全程） | 37 来源实例零直接影响 |
| `character/standard/skill.js` | 仅 1 行：洛神判定回调签名 `async event =>` → `async (event, trigger, player) =>`，函数体未动 | 原生洛神为原生实现（非 JS），语义不受影响，**无需补登差异** |
| `mode/guozhan`（33 文件） | 新内容包（九变篇 `jiubian.js`、紫凤衔佩 `zifeng.js`、群狼环鼎——additive）；Step Content 结构重写（rest.js/yingbian.js/normal.js，perf 性质）；`10cc4fec` 分道扬镳、擅统对君主规则兼容修复 | 原生国战原型（v80–84）未建模分道扬镳/擅统（engine+覆盖清单 0 命中），不受影响；新内容属未来上游同步范围 |
| `card/{guozhan,huodong,jiubian}.js` | guozhan 1 行、huodong +114、jiubian +47——全部新增卡 additive | 原生未转换这些模式卡，不受影响 |
| `library/element/{content.ts,player.js,character.js}` + `library/skill.js` | 装备栏废除/恢复的 `all` 选项扩展（`event.all` 槽位展开 + 恢复时纳入无 cardSymbol 实体装备）+ JSDoc 类型注解 | 原生**无装备栏废除建模**（engine 0 命中），不受影响 |
| `mode/{chess,single,tafang}.js` | 战旗/单挑/塔防 Step 重写（perf） | 原生无这些模式实现，不受影响 |
| 其他 | v1.11.6 发布、UI 组件化、安卓适配、武将包内容更新（`3f5b7987` 等） | 非结算语义；新武将属上游 core 内容，原生按同步批处理 |

## 结论

1. **无需补登规则差异**：42 提交中没有触及原生已实现语义面的变化。
2. **基线口径决定：保留 9-23 `inventory.json` 基线，不重录**。理由已写入台账复核段：基线锚定的是**来源级功能面核验状态**，上游 core 快进与来源实例变化是两个维度（本次即为例证——core 42 提交大改而 extension 0 变化）；重录会抹掉对照锚点。后续上游同步（v1.11.7 准备中，`62dab765`）时按当次同步范围重录。
3. D1（史诗卡牌 3 文件 148 字节）仍未决——该部分维持待用户决定。

## 验证方法

命令（上游仓只读）：
- `git -C archive/upstream/noname log --oneline 5c02a745..df2682e6`（42 提交清单）；
- `git diff 5c02a745..df2682e6 --stat` 全量 + 分面（extension/0、standard/1 行、library、mode、card）；
- 洛神/分道扬镳/擅统/装备栏废除逐点与原生源码、覆盖清单交叉检索（0 命中记录见上表）。

原生侧无需重跑测试（本批零代码改动）。
