# UP37-I02：原仓库音频清单与武将库试听（2026-10-30）

本批源码基线 `44982f45edfb1a365163ecdacc08fa972219684e`，规则328／存档105／原生ABI753／WS5；父线程329／758的B、J2不在该基线。采用的核心来源仍为 `1a548c95acdf8ac1ef44d74473403eda2ac86cf0`。这是资源切片，不是I02、UP37或两仓37来源整包验收。

## 实现与来源

原同步器只导入角色树和画像，`asset-manifest.json` 没有音频；真实目录 `sp2:star_dongyun` 的 `starbishi1.mp3` 查找反例先失败，再随音频树导入转绿。现复用schema_version2的kind/path/size/source_category/source_path/sha256字段导入核心 `audio/`，新增9652条、651011056字节；2707条画像保持原顺序与内容。所有12359条资源在源与目标分别核对长度和SHA256，共24718次，0差异。核心来源许可仍为 `preserved_from_source_requires_review`，没有外部下载或发布授权。

`--resources-only` 复用既有树同步与清单生成，只处理资源，先检查采用来源的完整provenance；不运行cargo、重写catalog、人工DSL、npack、冻结包或旧内容锁。目录与ported/packages合计16623个受保护文件逐字节不变。另修复来源目录经符号链接查找时git status错误地使用逻辑绝对路径、误报仓库外路径的问题；来源标签仍为archive路径。

CollectionView从同一清单寻找武将声明的直接技能和阵亡MP3，通过既有game-server `/original-content` 静态资源路由加载；原生audio controls提供播放、暂停和音量，preload=none。路径拒绝遍历、绝对URL、反斜杠及编码分隔符；失败显示错误，不替换成其他武将语音。资源统计读取清单，不把资源可用性当作规则可玩性。

## 已执行验证

- 新资源回归3项、现有前端合计149/149；真实catalog ID覆盖星董允、两个周瑜、赤兔、乐孟获及魔张梁的跨包依赖语音，唐棠保持source_synced且语音为空。
- 同步入口／区域排序2/2，验证受保护文件不改、目标坏字节被拒且源文件不受损、未采用来源在写入前被拒。
- `node scripts/sync-original-content.mjs --resources-only --verify-only`、完整 `--verify-only` 均exit0。原生/Rust源码未改、未编译。
- `npm run build`执行vue-tsc和vite，exit0。首次缺两个ignored生成WASM目录而失败；后临时只读查找父线程main已生成的329／758模块以满足页面导入，未宣称328源码与借用引擎的规则组合验收。临时链接在交接时移除。
- 独立私有HTTP端口和headless Chrome真实构建页面：星董允、幻周瑜、OL谋周瑜、谋周瑜共20份源音频HTTP读取／WebAudio解码成功，画像naturalWidth>0；点击星董允audio播放控件得到paused=false、currentTime>0、readyState=4、error=null。唐棠无语音，仍显示待完整移植。测试只将7700原资源请求重定向到自己私有静态文件服务器，未运行或模拟规则引擎、未改父服务。

全部初始失败、二次成功日志、截图、逐源文件SHA/MIME、保护文件摘要和纯patch在隔离任务的兄弟evidence目录交接。9652个源文件MIME分别为audio/mpeg8811、application/octet-stream742、audio/ogg16、audio/x-wav67、audio/x-m4a9、audio/x-hx-aac-adts1、video/mp4 6；导入保留原始字节，不能声称每个格式／文件都已解码。

## 剩余范围

- N01的`ol_jie_danshou`上游声明 `audio: "xindanshou"`；源音频存在但直接身份匹配未消费该映射。`dc_sb_zhouyu`的两个技能另有 `audioname: ["dc_sb_zhouyu_shadow"]`。需要沿用既有安全AST读取，把可证明的音频别名绑定到清单/包资源；不得改借别人的声音补数量。本批不声称已解决别名。
- N01界朱然／界朱桓、N04孙寒华、N07许劭在核心源没有对应画像。唐棠、魔张梁、星董允等既有4张新增画像已经在I01；本批不重复计数。汉末于毒明确使用 `img: "image/character/yudu.jpg"`，现目录已正确保留。
- 唐棠／孙寒华／许劭及魔张梁新增技能等缺对应源语音；赤兔、乐孟获及魔张梁依赖技中有真实可用音频，见清单，不上调其玩法状态。
- catalog皮肤源未配置，静态／动态皮肤和共享别名均0；现有原包未发布skin_index。Liuli七份本地修改只读保留。合法皮肤源、真实作者包/稳定依赖与PackageStore生命周期验收仍缺，不能拿schema测试样本补数。
- UP37的54条资源变化路径已逐条保留来源路径／SHA／MIME和导入状态：35条当前可用资源已在清单，1条源画像已删除；另18条可用输入尚未导入或消费（`image/card/mashiro.png`、`image/card/ol_cailian.png`、`image/olxushao/*`的15份界面图片/按钮音效、`layout/others/skill.css`）。它们依赖N04/N06衍生牌与N07复合选择界面的真实呈现/交互契约，不因本次音频树导入被标成已支持。
- 自动对局技能／阵亡播放仍缺授权事件到资源键的消费链；N02–N07规则、模式迁移、R10剩余显示／AI、I03–I05验收与37来源G0–G5保持原范围。物理听感、桌面／移动真机、跨浏览器以及上游规则等价未在本批证明。
