# `global` 卡主动使用面普查（2026-10-19，第 384 轮）

- 工作包：**NC08**；规则版本 v322（工作树）/ 存档格式 v105
- 目的：第 383 轮交接**优先级 1**——`global`（响应窗口）卡的**基线**

## 上游清单（杀海拾遗 12 张带 `global`）

`chenhuodajie` / `jinlianzhu` / `shihuifen` / `shencaojie` / `zhuquezhizhang` / `xuanwuzhihuang` /
`huanglinzhicong` / `cangchizhibi` / `qinglongzhigui` / `baishouzhihu` / `shenmiguo` / `zhufangshenshi`

## 结果（**主动使用，三形态 `[]`/`[0]`/`[1]`**）

| 卡 | 在目录 | 主动使用 | 与既有事实的一致性 |
|---|---|---|---|
| `chenhuodajie`（趁火打劫） | true | `[1]→Accepted` | ✅ 与样本 145（其他角色目标）一致 |
| `jinlianzhu`（金莲珠） | true | **均被拒** | ✅ 与样本 87（**打断牌**）一致 |
| `shihuifen`（石灰粉） | true | `[1]→Pending` | ✅ 与样本 90（当前回合角色自己用）一致 |
| `shencaojie`（神草结） | true | `[]→Accepted` | — |
| `zhuquezhizhang`/`xuanwuzhihuang`/`huanglinzhicong`/`qinglongzhigui`/`baishouzhihu`（**器族五件**） | true | `[]→Accepted` | ✅ 与「器族五件：**手持牌**（非装备路径）」注释不矛盾（作为**装备**受理） |
| **`cangchizhibi`（苍螭之璧）** | **false** | — | ✅ **转换缺口**（与样本 146 一致） |
| `shenmiguo`（神秘果） | true | **均被拒** | ✅ 与源码注释「**不可直接使用**（窗口机登记结构性暂缓）」一致 |
| `zhufangshenshi`（朱fang神石） | true | `[1]→Accepted` | — |

⇒ **12 张中 11 张在目录**（`cangchizhibi` 属缺口）；**受理结果与既知事实逐条一致**（金莲珠/石灰粉/神秘果/苍螭之璧）。

## 未完成（**待办**）

- [ ] **各卡的具体响应窗口**（上游 `global` 子技的触发事件）与**窗口内可达性**——须**逐卡定位**；
- [ ] `shencaojie`（神草结）的计数修正面；`zhufangshenshi` 的效果面。
