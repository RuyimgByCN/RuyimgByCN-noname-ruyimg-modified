# 777 发布后的真实 CI（2026-10-30）

已发布提交 `777bf33969e5ecc04b63971adab26be32f12fd28`，tree `7f2d41e257dfc44072bd6a5ef68dad4cb6c70496`。[run41114612841](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/41114612841)、attempt1 最终为 **9 success / 1 failure**，2026-10-30T19:44:31Z 更新。此结果不覆盖 a1f 的十绿，也不能按成功发布推定 CI 通过。

失败为 Windows desktop job124265455086，step7 `prepare-original-content`：Node20.20.2 的原 `validateAudioCompat` 在 `original-audio-resources.mjs:59:11` 抛出 `audio compat fixed provenance/recipe/tool binding differs`，调用方 `sync-original-content.mjs:277:24`，process exit1。Windows build job 成功，但安装器及上传跳过，无 Windows artifact。原 runner provenance 实际 SHA 未记录；另项受控检出及修复见[精确路径属性证据](../../2026-10-30-windows-provenance-checkout/README.md)，不能回填本次旧结果。

[producer原交接](producer.raw.txt)、[独立登记](independent-review.raw.txt)、[父只读parser](parent-independent-parser.py)与[结果](parent-independent-results.json)保留不同资格。独立登记 OK with notes 仅接受证据；CI 验收仍失败。唯一 default watch exit1 来自实际失败，没有 502／fallback／重跑。

完整87份primary（含[原清单](copy-manifest.json)）逐字节复制；十份raw日志保持gzip原字节，解压共4,634,356B，Windows build／desktop CRLF为3822／898。[复制映射](archive-copy-map.json)另列producer／review／父parser，不将private原始解压日志冒充已复制。

130条step为121 success／1 failure／8 skipped／0 null。workspace6894/0/77、native1815/0/75、三OS库矩阵各838/0/0、Worker82及前端169/0/0范围重叠。3480 warning loglines与ignored／skipped保留，不相加为独立测试数。性能预算Ubuntu64/0/55/25、macOS70/0/136/28μs通过，不作同设备收益。仅macOS与Ubuntu安装器artifact元数据，未下载／安装／签名／真机验收。

普通push exit0，远端native-convergence为777、main仍47347c6608d651660ae9459e59eb529b3cef277f；原hook与persistent设置不变。主树写租约19:22:05Z释放后父接入PA09三源文件，其dirty不属于777发布身份漂移。原始发布receipt另见[publication](../publication/result.json)。
