# `global` 卡窗口映射表（12 张，2026-10-19）

- 工作包：**NC08**；规则版本 v322（工作树）/ 存档格式 v105
- 方法（样本 150 确立）：①取卡的 `global: "g_xxx"`；②在 **`skill.js`** 找 `g_xxx` 的 **`trigger`** ⇒ **即窗口事件**

## 映射表（**逐卡逐字**）

| 卡 | `global` 子技 | **trigger（窗口）** | 与我方既有样本的一致性 |
|---|---|---|---|
| `baishouzhihu`（白首之虎） | `g_baishouzhihu` | `{ player: "discardEnd" }`（**弃牌阶段结束**） | — |
| `cangchizhibi`（苍螭之璧） | `g_cangchizhibi` | `{ player: "phaseBegin" }`（阶段开始） | —（**转换缺口**） |
| `chenhuodajie`（趁火打劫） | `g_chenhuodajie` | `{ global: "damageEnd" }`（**伤害结束后**） | ✅ 与样本 145（主动使用）不冲突 |
| `huanglinzhicong`（黄麟之璁） | `g_huanglinzhicong` | `{ player: "phaseBegin" }` | — |
| **`jinlianzhu`（金莲珠）** | `g_jinlianzhu` | **`{ global: "damageBefore" }`（伤害前）** | ✅ **与样本 87 一致**（打断伤害） |
| `qinglongzhigui`（青龙之圭） | `g_qinglongzhigui` | `{ player: "phaseBegin" }` | — |
| **`shencaojie`（神草结）** | `g_shencaojie` | **`{ source/player: "damageBegin" }`** | ✅ **与样本 150 一致** |
| `shenmiguo`（神秘果） | `g_shenmiguo` | **`{ player: "useCardAfter" }`（用牌后）** | ✅ **与源码注释「用牌后窗口消耗」一致** |
| **`shihuifen`（石灰粉）** | `g_shihuifen` | **`{ global: "phaseUseBegin" }`（出牌阶段开始）** | ✅ **与样本 90 一致** |
| `xuanwuzhihuang`（玄武之璜） | `g_xuanwuzhihuang` | `{ source: "damageEnd" }` | — |
| `zhufangshenshi`（朱fang神石） | `g_zhufangshenshi` | `{ player: "useCardAfter" }` | — |
| `zhuquezhizhang`（朱雀之璋） | `g_zhuquezhizhang` | `{ player: "damageEnd" }` | — |

## 价值

1. **可复用参考**：后续定位任一 `global` 卡窗口时，**直接查表**并构造对应事件；
2. **交叉验证**：**3 张**（金莲珠/神草结/石灰粉）与既有样本**逐条一致**，**1 张**（神秘果）与源码注释一致；
3. **修复依据**：对第 38 项（石灰粉限制）等，窗口事件（`phaseUseBegin`）已明确。

## 未完成（**待办**）

- [ ] 逐卡**构造事件**验证窗口**可达性**与**效果**（当前仅金莲珠/神草结/石灰粉三张有行为面样本）；
- [ ] `{ global: … }` 形态（趁火打劫/金莲珠/石灰粉）与 `{ player/source: … }` 形态的**受理差异**。
