# T13 worker 切换第三片：view 深字段补全（2026-10-14）

## 交付（Rust 内核→窄壳→适配层三层贯通）

- **view.rs**：`PlayerCellView` 加 `turned_over: bool`（公开翻面状态）与
  `skills: Vec<&'static str>`（已授技能名，仅 enabled 实例，排序稳定）——
  均为公开信息，不泄暗置归属。
- **engine.rs** `build_view`：turned_over=face_down；skills 从
  `core.skills` 按 `(owner, def)` 有序 range 收集该座位 enabled 技能名
  （catalog def 名，符号稳定）。
- **窄壳** view_json 加 `turnedOver`/`skills` 投影。
- **client.ts**：`NativeView` 接口与 observer 适配映射同步——
  `turned_over: p.turnedOver`、`skills: p.skills.map(name => ({name} as
  SkillView))`（适配首层形状；SkillView 深字段 id/def/use_count 等窄壳不
  提供，置默认值——登记差异）。
- **适配层**：`nativeObserverToSnapshot` 同步 skills/turned_over（同源逻辑）。
- **回归**：t13_info_isolation 新增 `deep_fields_turned_over_and_skills`
  （默认未翻面/已授技能在本人与**他人视角**均公开——技能授予是公开信息）。
- **wasm 冒烟**：DEEP 断言 turned=[false×4]/skills 计数投影（默认无授
  0——窄壳 rebuild 后验证字段贯通）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 | t13_info_isolation **10/10**（+1 深字段） |
| wasm 冒烟 | **8/8**（DEEP 深字段投影贯通验证） |
| engine-native 全量 | **744/0/0**（view 为只读投影，不进哈希/存档） |
| Gate 1b | exit 0 |
| vue-tsc | 0 error |

## 登记边界

- skills 投影含**扩展包技能名**（catalog def 名——npack 声明名）；适配层
  SkillView 深字段（id/def/use_count/active/view_as/locked）置默认值——
  完整 SkillView 映射需窄壳暴露 SkillInstance 细节，随 UI 批。
- identity（身份局）不投影——隐藏信息，正确不泄。
- 真实浏览器回归归后续批（worker 生命周期与 UI 联动）。
