# 4dd 发布及真实 CI（2026-10-30，2026-10-31登记）

普通发布已完成，exact提交`4dd0b286b995fe414128b6c52eb83a4d0d85f6cd`、tree`184829236b7c17fe472fecbb1ba77b8129c65a6e`、parent777。[run41125396035](https://github.com/RuyimgByCN/RuyimgByCN-noname-ruyimg-modified/actions/runs/41125396035) attempt1为completed/success，最终更新2026-10-30T21:06:28Z；十个job全部成功。唯一default watch exit0，无transport failure／fallback／rerun／dispatch。

[producer交接](producer.raw.txt)、[独立复审](independent-review.raw.txt)、[父fresh只读parser](parent-independent-audit.py)及[结果](parent-independent-results.json)分开保留。父核95清单项的实际bytes／SHA256／CRC，清单自身为第96份primary；十份detail与jobs step精确对应，130条step为128 success／2 skipped／0 failure／0 null。两skip均是macOS／Windows的Linux Tauri依赖，不由job绿补step。

Windows job124302465532的step7资源准备、step10安装器、step11上传实际成功，三平台artifact均绑定本run／exact4dd。它支持本次精确路径-text属性的CI检出／打包闭环，不反推旧777 runner实际provenance SHA；[旧run41114612841的9成功／1失败](../../2026-10-30-up37-i02-audio-compat/publication-ci/README.md)保持失败。原受控CRLF red→green证据与原验证器门禁不变。

实际gates Rust6894/0/77、native1815/0/75、三OS库各838/0/0、Worker82、前端175/0/0、资源17/17、conformance19及cross-target通过；范围重叠，不相加为独立测试数。3628 warning loglines和ignored均保留。hosted legacy release预算Ubuntu57/0/53/22、macOS71/0/61/27μs相对于100/300/200/100通过，不作native收益或同设备非劣结论。

| Artifact | ID | API bytes | API SHA256 |
| --- | ---: | ---: | --- |
| macOS | 13012150411 | 32803079 | c19ff30080cd48afe7eeac672d397b202f117856ca254c8cce0b2eb8f10d693b |
| Windows | 13010748280 | 26274800 | e0130f1105d01170adc3caaf42c4b6a5a6b55d5bc29f5393e92cb012df755d3c |
| Ubuntu | 13010039279 | 266343986 | 4af11ffe90452497f890962f8c4ea3dd07dd849e60fc12587607c994c6bdffa2 |

只核API元数据，未下载安装／签名／真机。十原日志以gzip原字节保存，解压4,733,040B；Windows build／desktop CRLF3837／2207，父核解压SHA／bytes／CRC／CRLF。10份private解压原件未复制，[最终清单](copy-manifest.json)及[复制映射](archive-copy-map.json)明确范围。checkpoint live-stream索引是当时快照，不能拿它核后续变化的watch文件；当前复制按最终清单。

push exit0，exact native-convergence由777→4dd，远端main保持47347c6608d651660ae9459e59eb529b3cef277f。新命令级guard SHA20c22832833ba9d74b5f3f6436925d3e4fdc4dc9d0500f8de5b691893b750f14、八项正负自检，旧hook e349af…和persistent unset保持。351路径／155保护首末相同、3包树、18模块／37dist／index保持；没有重扫21,726路径或12k媒体。Git279objects／53deltas／22.72MiB为舍入进度；wall8.445246s、locked8.386769s，差值含启动排队，不是纯queue或精确wire bytes。

Main发布租约20:43:27.690435Z已释放；后续CI采集全own/tmp。当前裁决27文档和私有新实现不在4dd身份内。规则330／NGSA105／ABI758／WS6／WASM默认保持；本次独立结论OK with notes仅登记／归档CI，不关闭PA09六条件、I02／37来源／整体Goal或产品默认切换。
