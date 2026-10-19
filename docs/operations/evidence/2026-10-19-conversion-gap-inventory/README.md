# 转换缺口清单（确认版，2026-10-19）

- 工作包：**NC08 / 扩展台账**；规则版本 v322（工作树）/ 存档格式 v105
- 方法：上游同名 `card.js` 中「紧邻出现 `type:`」的卡键 → 依次核对 **①包内卡名（前缀归一）** →
  **②原生核心 `crates/engine-native/src/**`** → **③其它包 `packages/extensions/*/assets/*.json`**
  → 三者**均无**才判「未转换」
- 检索文本：原生 src + 全部包 JSON 合并（约 3.6 MB，小写匹配）

| 包 | 上游卡键 | 包内卡名 | **确认缺口** |
|---|---:|---:|---:|
| `shahai-gujian-upstream-noname` | 33 | 30 | **2** |
| `shahai-gwent-upstream-noname` | 43 | 12 | **30** |
| `shahai-hearth-upstream-noname` | 13 | 12 | **1** |
| `shahai-swd-upstream-noname` | 79 | 90 | **17** |

**合计确认缺口：50 个**

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

## shahai-swd-upstream-noname（17）

```
baihupifeng cangchizhibi fuxiqin guangshatianyi guiyanfadao hslingjian_shengxiuhaojiao hslingjian_shijianhuisu hslingjian_xingtigaizao hslingjian_xuanfengzhiren hslingjian_yinmilichang hslingjian_zhongxinghujia huanpodan lingjiandai pangufu qinglianxindeng qipoguyu xiangyuye
```

## 说明与边界

1. **本清单是「未转换」判定**（包内、原生核心、其它包三者均无），**不含**「已实现但不在该包包内」
   （如 gwent 的 `aozuzhilei`/`ansha` 已在**原生核心**实现）；
2. **gujian 的 2 项**（`gjyuheng_plus`/`gjyuheng_pro`）为**已知暂缓**（基准记「升级机制暂缓」）；
3. **未定位包不覆盖**：8 个包的 `upstream_path` 下无 `card.js`（≠ 未转换）；
4. 前缀归一表：`swd_`/`gw_`/`gj_`/`ycn_`/`wd_`/`hl_`/`xz_`/`ow_`/`hearth_`；
5. 本清单只判「**是否转换**」，**不判玩法面等价**（后者须逐卡样本）。
