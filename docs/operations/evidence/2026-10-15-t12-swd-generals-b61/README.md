# T12 swd 武将面批㉑（三将部分转正，2026-10-15）

本目录为**补记**（原批只有日志，README 缺失；2026-10-17 由后续会话据提交 `3f742adb` 补写，链接修正）。

## 内容

- 疾影（tag 312 赛特）：锁定 sha 无视距离——`sha_range_free` 链查询。
- 沐风（tag 313 桓远之）：结束阶段本回合用过基本牌则摸 1——`TurnState.mufeng_used_basic`。
- 望尘（tag 314 古月圣）：弃牌阶段弃置过基本牌则 End 入场令一名角色翻面——`TurnState.wangchen_discard_basic`。

## 字段批

`TurnState` 两旗入哈希；存档 v73→v74；规则 v286→v287；14 golden 重录 + 286 探针零扰动。

## 登记

`cards.json` 三将部分入册（赛特 1/3、桓远之 1/3、古月圣 1/3）。触发机时序勘误：沐风 filter 用 `!*begin`（阶段末帧）而非 begin。

## 验证

定向 **54/54**、全量 **771/0/0**、Gate 1b 绿（[engine-native-tests.log](engine-native-tests.log)、[gate1b-all-targets.log](gate1b-all-targets.log)）。**21/79 名 29/241 技，整将维持十七**。
