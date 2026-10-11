# 2026-10-11 T9：龙血玄黄战斗内触发轨迹勘察 + boss 专属技能包首切片评估

任务来源：[进度主稿 T9](../../../architecture/native-progress.md)（verifier 指派勘察）。

## 一、龙血玄黄战斗内触发轨迹——阻塞点登记（确需旧线运行环境）

LX.2 已完成**注册期**动态采样（VM+记录型 Proxy 桩，content×2），采样到 `mbqianlong` 注册与 `_CMeffects` 门控——注册期行为已完整。残余「战斗内触发轨迹」需要把扩展放进**带对局的旧线完整运行环境**（技能真实触发）。具体阻塞点三条：

1. **无头运行未验证**：战斗循环依赖 noname core 的真实触发系统/游戏循环（浏览器 UI 链）；LX.2 的 VM 桩只 stub 了 lib/game 写路径，不覆盖对局驱动。node 无头跑通 noname core 是独立未验证工程（先例为零）。
2. **依赖与只读约束冲突**：`archive/upstream/noname` 无 node_modules（pnpm 工程需网络安装+写入 archive 目录）——archive 只读约束直接阻塞。
3. **前置立项未决**：按 LX.3 定案口径「如迁移规则面再补」——触发轨迹的采集前提是 mbqianlong/_CMeffects 的 G1 迁移立项（D 面决策），立项前采集无消费点。

结论：确需旧线运行环境且三条阻塞中两条非本仓可解（①工程量专项、②约束、③立项）——登记后不强行推进，维持「迁移前置残余」。

## 二、boss 专属技能包——首切片评估

- **数据源定位**：上游 `apps/core/mode/boss.js:858–1660` `characterPack.mode_boss`——**约 40 名 boss/随从武将、78 处 skills 引用**（boss_hundun hp25 五技、boss_qiongqi hp20(max25) 五技、boss_zhuyin hp4 单技随从等；含 `_switch` 双技与 isHiddenBoss 随从）；另有关卡 `bossinfo`（chongzheng/loopFirst/gameDraw/minion/control）与 `addBossFellow` 关卡脚本族。
- **首切片候选**：最小武将 `boss_zhuyin`（hp4、单技 `boss_xiongshou` 凶兽=全 boss 共有技）+其 `bossinfo`。
- **消费侧缺口（关键依赖）**：原生 `GameMode::Boss` 是裸骨架（重整复活回环+addFellow 已建模，但**无 boss 选将/关卡系统**——测试经 debug 面装配）。boss 内容包的可消费点（boss 将表+关卡配置+按关卡初始化）不存在——首切片不能直接落「转换+回归」，需先补消费面。
- **建议切片序**：①boss 选将/关卡面（`new_boss` 增 boss 武将 def+关卡配置参数：chongzheng 已有、gameDraw 开局摸牌数[同斗地主 v205 先例]、loopFirst 回合起点）→ ②boss_zhuyin+boss_xiongshou 转换落包 → ③按 T12 逐包节奏推进。每片均「领取即加锁」可独立验收；①约为 v206 替补批量级（中批）。

## 结论

- LX 残余：阻塞登记（本目录），维持迁移前置。
- boss 内容包：首切片=消费面（boss 关卡骨架），后续批可拆；本轮完成勘察登记，实施待下轮指派。
