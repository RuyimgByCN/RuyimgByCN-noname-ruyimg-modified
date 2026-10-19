# 转换缺口清单（确认版 v2，2026-10-19）

- 工作包：**NC08 / 扩展台账**；规则版本 v322（工作树）/ 存档格式 v105
- **v2 变更**：①上游定位改为**多基址解析**（`archive/upstream/noname/`、`noname-liuli/`、
  `apps/core/extension/` 前缀补全）②目录型路径改为 **walk 全部 `.js`**（不再只找 `card.js`）
  ⇒ **可筛选包由 4 增至 10**，并**新发现 `shahai-zhenfa` 的 5 项缺口**
- 判定顺序：**包内卡名（前缀归一）→ 原生核心 → 其它包** → 三者均无才判「未转换」

| 包 | 上游卡键 | 包内卡名 | **确认缺口** |
|---|---:|---:|---:|
| `huanle-upstream-noname` | 6 | 6 | **0** |
| `shahai-gujian-upstream-noname` | 33 | 30 | **2** |
| `shahai-gwent-upstream-noname` | 43 | 12 | **30** |
| `shahai-hearth-upstream-noname` | 13 | 12 | **1** |
| `shahai-mtg-upstream-noname` | 13 | 10 | **0** |
| `shahai-swd-upstream-noname` | 80 | 90 | **18** |
| `shahai-yunchou-upstream-noname` | 10 | 11 | **0** |
| `shahai-zhenfa-upstream-noname` | 11 | 6 | **5** |
| `wudian-upstream-noname` | 6 | 7 | **0** |
| `wuxing-upstream-liuli` | 0 | 1 | **0** |

**合计确认缺口：56 个（10 包可筛选）**

## shahai-gujian-upstream-noname（2）

```
gjyuheng_plus gjyuheng_pro
```

## shahai-gwent-upstream-noname（30）

```
gw_aerdeyin gw_baishuang gw_baobaoshu gw_butianshu gw_chongci gw_dieyi gw_dieyi_equip1 gw_dieyi_equip2 gw_dieyi_equip3 gw_dieyi_equip4 gw_dieyi_equip5 gw_dieyi_judge gw_fuyuan gw_ganhan gw_guaiwuchaoxue gw_huangjiashenpan gw_hudiewu gw_leizhoushu gw_niuquzhijing gw_nuhaifengbao gw_shizizhaohuan gw_tongdi gw_tunshi gw_xinsheng gw_yigeniyin gw_youer gw_zhongmozhizhan gw_zhuoshao gw_zirankuizeng gw_zuihouyuanwang
```

## shahai-hearth-upstream-noname（1）

```
shihuawuqi
```

## shahai-swd-upstream-noname（18）

```
baihupifeng cangchizhibi fuxiqin g_yuchan_equip guangshatianyi guiyanfadao hslingjian_shengxiuhaojiao hslingjian_shijianhuisu hslingjian_xingtigaizao hslingjian_xuanfengzhiren hslingjian_yinmilichang hslingjian_zhongxinghujia huanpodan lingjiandai pangufu qinglianxindeng qipoguyu xiangyuye
```

## shahai-zhenfa-upstream-noname（5）

```
fengyangzhen huyizhen longfeizhen shepanzhen yunchuizhen
```

## 精度更正：swd 的 6 项 `hslingjian_*` 为**误报**（2026-10-19 续核）

`hslingjian_*` 为**子件命名**（前缀未在归一表中）⇒ 去掉 `hslingjian_` 前缀后**均有 12–18 处命中**
（原生核心/包内已实现）⇒ **6 项为误报**，**swd 实际缺口由 18 降为 11**（另 1 项 `g_yuchan_equip`
为全局子技噪声）⇒ **全清单合计由 56 修正为 49**。

| 键 | 去 `hslingjian_` 后 | 判定 |
|---|---|---|
| `xuanfengzhiren`（旋风之刃） | 12 命中 | 已实现（**误报**） |
| `zhongxinghujia`（重型护甲） | 13 命中 | 已实现（**误报**） |
| `xingtigaizao`（型体改造） | 18 命中 | 已实现（**误报**） |
| `shijianhuisu`（时间回溯） | 13 命中 | 已实现（**误报**；样本 64 已验其行为面） |
| `shengxiuhaojiao`（生锈号角） | 13 命中 | 已实现（**误报**） |
| `yinmilichang`（隐秘力场） | 13 命中 | 已实现（**误报**） |

**归一表补**：`hslingjian_`（**第 6 条精度更正**：前缀表须覆盖**子件命名**）。

## gwent 缺口（30 项，已逐项确认**非误报**）

**核实**：去 `gw_` 前缀后 `dieyi`/`hudiewu`/`yigeniyin`/`zhuoshao` **均 0 命中** ⇒ 与 swd 的
`hslingjian_*` 不同，**gwent 的缺口全部为真**。

| 组 | 数量 | 明细 |
|---|---:|---|
| **蝶翼系** | 7 | `gw_dieyi`（蝶翼，equip）+ `gw_dieyi_equip1..5`（器/衣/攻/防/宝）+ `gw_dieyi_judge`（判，delay） |
| **法术** | 23 | 蝴蝶舞 / 伊格尼印 / 雷咒术 / 阿尔德印 / 新生 / 纽曲之镜 / 终末之战 / 干旱 / 皇家审判 / 吞噬 / 冲刺 / 诱饵 / 通敌 / 复原 / 灼烧 / 卜天术 / 十字召唤 / 最后愿望 / 自然馈赠 / 怒海风暴 / 白霜 / 雹暴术 / 怪物巢穴（均 `spell`） |

**注**：`gw_dieyi_equip1..5` 与 `gw_dieyi_judge` 为**蝶翼的子件**（同 `hslingjian_*` 的形态），
但**基础件 `dieyi` 亦 0 命中** ⇒ 整族**确未转换**（**非**「子件命名误报」）。

## swd 两张 `basic` 的译名（未取到）

`xiangyuye`、`huanpodan`（均 `type: "basic"`、`enable: true`）在 swd 的 `translate.js` 中
**无对应键** ⇒ **中文名待补**（不影响「未转换」判定）。

## swd 其余缺口（11 项，含中文名与类型）

| 键 | 中文名 | 类型 |
|---|---|---|
| `cangchizhibi` | 苍螭之璧 | `jiqi` |
| `lingjiandai` | 零件袋 | `jiguan` |
| `pangufu` | 盘古斧 | `equip` |
| `fuxiqin` | 伏羲琴 | `equip` |
| `qinglianxindeng` | 青莲心灯 | `equip` |
| `baihupifeng` | 白狐披风 | `equip` |
| `guangshatianyi` | 光纱天衣 | `equip` |
| `qipoguyu` | 奇魄古玉 | `equip` |
| `guiyanfadao` | 鬼眼法刀 | `equip` |
| `xiangyuye` | （未取到译名） | `basic` |
| `huanpodan` | （未取到译名） | `basic` |

## 已逐名确认的缺口（2026-10-19 续核）

| 包 | 键 | 中文名 | 类型 | 双向核对（原生 src + 全部包） | 判定 |
|---|---|---|---|---|---|
| zhenfa | `fengyangzhen` | **风扬阵** | `zhenfa` | **0 命中** | **确认未转换** |
| zhenfa | `huyizhen` | **虎翼阵** | `zhenfa` | **0 命中** | **确认未转换** |
| zhenfa | `longfeizhen` | **龙飞阵** | `zhenfa` | **0 命中** | **确认未转换** |
| zhenfa | `shepanzhen` | **蛇蟠阵** | `zhenfa` | **0 命中** | **确认未转换** |
| zhenfa | `yunchuizhen` | **云垂阵** | `zhenfa` | **0 命中** | **确认未转换** |
| hearth | `shihuawuqi` | **石化武器** | `basic` | **0 命中** | **确认未转换** |

**注**：`shahai-zhenfa` 包内既有 `swd_*` 前缀卡（如 `swd_changshezhen1`）也有 `zf_*` 前缀卡
（`zf_dizaizhen`）⇒ 该包为**复合来源**；上表 5 项为其中**尚未转换**的阵法卡。

## 边界

1. 只判「**是否转换**」，不判玩法面等价；
2. 「包内没有」≠「未实现」（已在**原生核心/其它包**实现的不计入，如 gwent `aozuzhilei`/`ansha`）；
3. `wuxing`（liuli）**卡键 0**：其卡定义在 `extension.js` 内、格式不同 ⇒ **本筛选不覆盖**；
4. `gujian` 的 2 项为**已知暂缓**（升级机制）；
5. `swd` 的 `g_yuchan_equip` 为**全局子技**（非卡）⇒ **噪声项**，需人工剔除。
