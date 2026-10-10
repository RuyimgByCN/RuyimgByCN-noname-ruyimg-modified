# 阶段 21：可访问性（§30）

> 对应架构计划 v2.0 §30（可访问性）。可访问性信息来自 Observer View，
> 不进入 GameState。

## 目标

首个完整版本的可访问性支持：

1. 键盘完整操作
2. 屏幕阅读器语义（ARIA）
3. 减少动态效果
4. 高对比度
5. 字号缩放
6. 色觉友好提示
7. 不只依赖颜色表达状态
8. 动画跳过后仍可理解规则结果

## 新增 crate

| crate | 路径 | 职责 |
|---|---|---|
| `accessibility-schema` | `crates/accessibility-schema` | AccessibilityProfile + AriaDescriptor + KeyboardAction + StateSignal |

### 类型

```rust
pub struct AccessibilityProfile {
    pub reduce_motion: bool,        // §30 减少动态效果
    pub high_contrast: bool,        // §30 高对比度
    pub font_scale: f32,            // §30 字号缩放
    pub color_blind_mode: ColorBlindMode, // §30 色觉友好
    pub skip_animations: bool,      // §30 动画跳过
    pub screen_reader: ScreenReaderLevel, // §30 屏幕阅读器
}

pub enum StateSignal {
    Alive,          // ✓ 绿色 + 图标
    Dying,          // ⚠ 红色 + "濒死" 文字
    Dead,           // ✗ 灰色 + "阵亡" 文字
    CardPlayable,   // ▸ 蓝色 + 图标
    CardUnplayable, // ✕ 灰色 + "不可用" 文字
    CurrentTurn,    // ► 高亮 + 指示器
    PendingChoice,  // ? 脉冲 + "请选择" 文字
}
```

### §30 不变量验证

```rust
/// 每个状态信号必须有非颜色指示器（图标）。
pub fn validate_state_not_color_only() -> Result<(), Vec<StateSignal>>;
```

关键状态（Dying / Dead / CardUnplayable / PendingChoice）必须有文字标签，
不只依赖颜色。

### 默认键盘操作（§30: 键盘完整操作）

| 键 | 动作 |
|---|---|
| Space | 出牌 |
| Enter | 确认选择 |
| Escape | 取消选择 |
| ← → | 上/下一张牌 |
| Tab | 下一个玩家 |
| E | 结束阶段 |
| S | 跳过动画 |
| 1/2/3 | 聚焦手牌/棋盘/日志 |

## Web 客户端 CSS（§30）

`global.css` 新增：

| CSS 属性 | §30 能力 |
|---|---|
| `[data-contrast="high"]` | 高对比度模式（纯黑背景 + 白色边框 + 亮黄焦点） |
| `[data-font-scale="small|normal|large|xlarge"]` | 字号缩放（85%~150%） |
| `[data-color-blind="protanopia|deuteranopia|tritanopia|achromatopsia"]` | 色觉友好色板偏移 |
| `[data-skip-animations="true"]` | 跳过所有动画 + 过渡 |
| `.sr-only` | 屏幕阅读器可见、视觉隐藏 |
| `.state-icon` | 非颜色状态图标（✓⚠✗▸✕►?） |
| `@media (prefers-reduced-motion: reduce)` | 遵循系统减少动态偏好 |

`SettingsView.vue` 更新为完整的可访问性控制面板，所有设置：
- 即时生效（通过 `data-*` 属性应用到 `<html>`）
- 持久化到 `localStorage`
- 带有 ARIA 标签和描述
- 包含状态信号图例（图标 + 文字 + 颜色）

## 测试

```bash
cargo test -p accessibility-schema   # 7 tests
```

| 测试 | 断言 |
|---|---|
| `profile_defaults_are_safe` | 默认配置安全（font_scale=1.0） |
| `settings_round_trip` | AccessibilitySettings JSON 往返 |
| `state_signals_have_icons` | 所有 StateSignal 有非颜色图标 |
| `dying_and_dead_have_text_labels` | 关键状态有文字标签 |
| `default_keyboard_actions_cover_core_actions` | 核心操作有键盘绑定 |
| `color_blind_modes_are_distinct` | 色觉模式互不相同 |
| `profile_with_high_contrast_and_large_font` | 完整 profile 序列化 |

## 设计约束

- 可访问性信息来自 Observer View，**不进入 GameState**（§30 末尾）
- 规则核心**不依赖**此 crate（客户端专用）
- 动画跳过后规则结果仍通过文字日志 + 图标 + Observer View 完整传达
