# ADR 0611：上游音频挂接（Phase 3）

> 状态：已落地（2026-08-05）
> 领取：C 段（omp），2026-08-05
> 前置：ADR 0610（上游立绘）

## 背景

上游 `apps/core/audio/` 9,257 个音频文件（609M）：skill 7,357（按技能名+序号）、
die 1,621（按武将名）、card 159（按卡名+性别子目录）、voice 46（按编号）、
effect 60、background 14。

## 决策

1. 映射键：
   - `audio/skill/<skillname>N.mp3` → 按技能名（去尾数字）归属到含该技能的包
     （npack skill_index.name 与 DSL 技能名一致）。
   - `audio/die/<char>.mp3` → 按武将 name_key 归属到包。
   - `audio/card/{female,male}/<card>.mp3` → 按 card_index.name 归属。
2. 目标布局：`assets/audio/skill|die|card/{female,male}/<file>`，保持上游路径语义。
3. `audio/voice/`（按角色列表序编号）、`audio/effect/`、`audio/background/`
   （全局 UI 音效，无归属包）不迁移，文档化登记（引擎消费层未来若引入
   voice-key 绑定再处理）。
4. 基础牌语音（shandian/juedou/zhangba 等 156 个）无归属包——standard 的
   119 张基础卡在引擎域（DSL 不声明卡名，card_index 为空），引擎卡模型
   未来消费时才绑定，登记不迁移。

## 落地清单

- skill 语音：6,343 份拷贝到 29 个包（1,022 未命中 = 上游未迁移技能，如 guhuo）。
- die 语音：1,486 份拷贝（136 未命中 = 未迁移武将）。
- card 语音：2 份命中（其余 156 基础牌留档）。
- 全部 31 个资产包重打包：logic_hash 全包 SAME-HASH 核验通过。

## 验证

- 31 包重打包 + 部署；32 包空 store 冷导入通过
  （content_lock=ee263cfbd792594da24861576b30236f1232f8e7541b2aee810d347a46c94f1f）。
- `real_extensions_lock` 门禁 1/1 通过。

## 后续

- 运行时语音查找 API（voice key → asset id）未实现，属引擎消费层，不在本 ADR。
