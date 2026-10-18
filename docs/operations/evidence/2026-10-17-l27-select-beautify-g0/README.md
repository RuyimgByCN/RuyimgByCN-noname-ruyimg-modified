# L27 选将美化 G0 清点：混淆面与可读技能面（2026-10-17）

- 日期：2026-10-17
- 来源：**L27 选将美化**（`extension/选将美化/extension.js`，908 KB）
- 本批：**G0 清点**（未实施）——结论：UI 主面**混淆**（结构性），另含 **42 个可读技能定义**
- 原始计数见 [survey.json](survey.json)（脚本实测）

## 实测结论

| 面 | 实测 | 影响 |
|---|---|---|
| 主面（选将 UI） | **混淆**：`_0x…` 标识符出现 **17 844** 次（唯一 **2 178** 个），字符串表 `var _0x2f90=[…]`，含 37 处 `\x..` 转义字符串 | 选将 UI 的**忠实重构不可行**（无稳定语义锚点；仅能按可观测行为登记接口） |
| 规则面（技能） | **42 个可读技能定义**（`lib.skill.X = {…}`），另有若干被注释掉的（如 `jishe2`） | 技能面**可移植**（内容批次，多轮）；与原生目录**零重叠**（42/43 不在原生/包内） |
| 配置面 | 未发现 `extension_选将美化_*` 配置键 | 无配置迁移面 |
| 覆盖流程 | `chooseCharacter`（11 处）/`chooseCharacterTwo`（3 处）/`_status.characterlist`（12 处）/`selectButton`（4 处） | 选将流程覆盖点已定位；语义需从混淆代码**行为观测**推定（登记差异） |

## 技能面（42 个，可移植；按体量分级）

- **小（< 3 KB）**：`jishe`(816)/`baonvezhi`(1314)/`xiefang`(1649)/`new_zhixi`(2137)/`clandianzhan`(2232)/`new_meibu`(2568)/`sbzhaxiang`(2903)/`shouxi`(2933)…
- **大（> 30 KB）**：`mbwuling`(73 021)/`spyanjiao`(62 902)/`sbqingzheng`(36 818)…

## 后续处置（写入台账）

1. L27 状态由「待重构；未验收」改为「**G0 完成；UI 混淆（结构性）；技能面待批次实施**」。
2. 技能面按体量分批移植（先小技能批次，再大技能）；UI 面仅登记可观测接口（不逐行重构混淆代码）。
3. 选将流程覆盖点（`chooseCharacterTwo` 等）与原生选将原语（`RevealGeneral`/`BossStagePending::SelectGeneral`/`is_boss_general_banned`）的**对齐**列为后续切片。

## 复现

```
python3 - <<'PY'   # 统计混淆标识符与技能定义（见 survey.json）
import re, json
s=open('archive/upstream/noname-liuli/noname/extension/选将美化/extension.js',encoding='utf-8').read()
print(len(re.findall(r'\b(_0x[0-9a-fA-F]+)\b', s)), len(set(re.findall(r'\b(_0x[0-9a-fA-F]+)\b', s))))
print(len(set(re.findall(r'lib\.skill\.([A-Za-z_0-9$]+)\s*=\s*\{', s))))
PY
```
