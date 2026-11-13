# 队列1 Markdown 链接修正

2026-11-14T00:47:42.953332+08:00；HEAD `bd58ef06d59940baed8d7f93e61ce571b152fb81` + tracked dirty；规则/NGSA/schema/ABI/WS：333/107/5/759/8。

首次全量检查有636处坏链，原始[日志](../21-links-first.log)与[命令记录](../21-links-first.json)保留原哈希。

本次仅修10份U01审查文档的636个href：`:line`改为`#Lline`；其中draft复制后多出一层，318个目标另外修正相对层级。已逐条确认现有目标和原行号界限，没有改链接标签、正文、日期或表格语义。原Markdown以`.original.txt`保存在本目录before/下，不参与Markdown门禁；可用[精确替换与前后哈希](correction.json)重放核对。

| 文档 | href修正 | 修改前SHA256 | 修改后SHA256 |
| --- | ---: | --- | --- |
| [independent/u01/characters.md](../independent/u01/characters.md) | 62 | `1ec89736247dabac0c0e42021239567fad099d967840a1779131f9818048a775` | `7188ff96fa9f57c70a81ad59b1665af23be12acb0bd4ba9bb2b05ca26b03a34d` |
| [independent/u01/draft-before-strict-classification/characters.md](../independent/u01/draft-before-strict-classification/characters.md) | 62 | `136539df827b7c0c11e870f4792cfc15cf10c456a3ef0687d3c41a4d22fefe27` | `47e64c24d73627d6d01c316ddecd6b1c27aba3a17d187dd5ed92b6d7cb7a32ce` |
| [independent/u01/draft-before-strict-classification/skills-a.md](../independent/u01/draft-before-strict-classification/skills-a.md) | 60 | `4ca031cabf8348f3c56be3d92da64a0dfeb63720bfcfdf58efa8bbd7de9f57c0` | `c4d37d451256fc34cbe4a665fbed573e278a8ec59767155ff2afa5146b423789` |
| [independent/u01/draft-before-strict-classification/skills-b.md](../independent/u01/draft-before-strict-classification/skills-b.md) | 64 | `feac1b1d394d06fcbc0fca74c44db5afc8f92465a5b62de96019954ace390bd0` | `9767e0641bc7c84a2ca17fc4aacac55fb14d96cc0c287aa014540453f62a103c` |
| [independent/u01/draft-before-strict-classification/skills-c.md](../independent/u01/draft-before-strict-classification/skills-c.md) | 64 | `2e9dc746fec358b7c2e2484e940b34c47bac41401b997bcafc6f0397f1432b2e` | `8fd508c40df799a0138821c10b783820ff2ce0b6dbed438d86f2938b71741f45` |
| [independent/u01/draft-before-strict-classification/skills-d.md](../independent/u01/draft-before-strict-classification/skills-d.md) | 68 | `fd13f1f4e29177e89b4dffa53651cf9c99d9f3c883bd56d1cb64ac4fd539f321` | `1af516e1fd54c50cf8f257c5bd1c4033aba4ab9381f06cda38b6f5c33d4ecfb9` |
| [independent/u01/skills-a.md](../independent/u01/skills-a.md) | 60 | `71bee0d091e1d79d5b024a010082aec4d142e8219189848b01167bab5f9aec77` | `3a5846a401fc8b466a7861fe3d232ab6cacf2931378296a97bf2e207f030e914` |
| [independent/u01/skills-b.md](../independent/u01/skills-b.md) | 64 | `e51384871537a5353bc1e615a5f1b550d819394c8ca3905e7f9b68a08eb2875f` | `612ffc1043fd08de8cfbd1909646f409f69c58f89c3bda06078ed90f15f886fb` |
| [independent/u01/skills-c.md](../independent/u01/skills-c.md) | 64 | `06edbc279ab87ea9029b6f0c7ddc8cd163b89e0d6672e6abe3566a1d1ad5970c` | `53a3298c3f2e1b3e12e94b1edee05e03dcfb47e72bb62a002a36b38d79522a6f` |
| [independent/u01/skills-d.md](../independent/u01/skills-d.md) | 68 | `f74ca2477c9dc1f33f2a4fd06e348b0b86c54d3cb64c6f5d0a3483bea3cd206b` | `9c04f3ac50d36877221b457e42805d181b2d388c227c7da87d0abf08804d0bcd` |

共确认31个既有文件；636处锚点行号保持且均未越界。全量本地链接与独立file:line检查会另存唯一名原始输出，本记录不预记通过。没有运行完整preflight、Cargo或浏览器，没有提交/暂存/推送。

## 独立复查结果

2026-11-14T00:51:14.836257+08:00：全量本地链接检查2101份Markdown、坏链0、exit0；[原始输出](local-links-all-20261113T164826124731Z.log)与[命令记录](local-links-all-20261113T164826124731Z.json)保留。

[636处href重放与锚点核对](href-integrity.json)通过：31个确切现有目标、原行号均保留且界内，标签/正文/日期/表格语义不变。

严格file:line检查10584处、越界0、exit0；另报48类未解析的缩写/缺失路径，原样保留在[日志](doc-line-refs-strict-20261113T164826124725Z.log)及[命令记录](doc-line-refs-strict-20261113T164826124725Z.json)。该脚本不扫描#L，不能把它代替上述636处锚点的独立核对。没有把未解析路径写成全部存在。
