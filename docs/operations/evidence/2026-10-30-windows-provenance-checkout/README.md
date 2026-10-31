# Windows 固定 provenance 的 Git 检出字节（2026-10-30）

基准提交 `777bf33969e5ecc04b63971adab26be32f12fd28`，330／105／758／WS6不变。已独立复审后只在根 `.gitattributes` 追加一条精确路径 `content/original-audio-compat/provenance.json -text` 和why注释；原raw.log规则保留，provenance、校验器、recipe／tool、媒体及source14均不改变。候选314B、SHA `9dbbbde6fb0c137b5a47b3d3738b3ca9399dec80a03f9acc1399ffc1d3619388`，补丁见[candidate.patch](candidate.patch)。本次主树接入已通过最终四门禁及独立组合复审No issues／OK with notes，按裁决18本地接受；完整preflight21/21和源身份核验见[PA09组合证据](../2026-10-30-pa09-js-admission/README.md)。本次主树接入已正常发布为`4dd0b286`，真实run41125396035的Windows prepare／installer／upload均成功并独立登记，见[新CI证据](../2026-10-30-pa09-js-admission/publication-ci/README.md)。

原[run41114612841的Windows失败](../2026-10-30-up37-i02-audio-compat/publication-ci/README.md)保留9成功／1失败。独立fixture真实使用777原blob、私有index和Git `checkout-index`：`core.autocrlf=true/core.eol=crlf` 将2713B／81LF检出为2794B／81CRLF，JSON对象相等且十份输入hash全部匹配，但原validator在固定字节门禁报同一错误。仅替换attributes后，同配置检出恢复原SHA `1bd98a3e5d7ab22f6749b132d041f409e95a0dbf2f7ed9a603ec256422cc5313`，原validator接受；LF配置前后亦接受。

受控CRLF SHA为 `b912b07db79b7588a75fc0cb0970f98353288cb5d05e22f6c79f4268e461c821`；原runner实际SHA未被日志观察。此实验证明检出转换足以产生该错误；后续新run的Windows prepare／installer／upload已实际通过，旧777 CI仍保持失败。本地Git2.56.0／Node26.4.0，与原Windows Git2.56.0.windows.4／Node20.20.2分开记。

[实现原交接](implementation.raw.txt)、[独立review](independent-review.raw.txt)、[selfcheck](selfcheck.py)、[96命令索引](commands.json)和[完整summary](summary.json)保留实际argv／exit／输入SHA／四种检出。原资源测试17/17；首次fixture漏带原asset-manifest导致16/17，原失败保留，补原blob后完整通过，没有改产品或测试期望。

父[fresh parser](parent-independent-audit.py)／[结果](parent-independent-results.json)核472项原私有manifest现SHA／bytes／symlink target、七原blob和一候选blob的Git OID、source14及七脚本、四次十callback、原门禁pin、17/17及16/17、Main HEAD/index/staged。另[publication核对](parent-publication-invariants.json)确认旧三份证据原SHA、source14与accepted push相等、原run9/1及无覆盖祖先attributes。两个parser未运行producer／测试／build／network。

[原始证据压缩包](git-checkout-raw-evidence.tar.gz)保留358项raw命令输出与初次driver失败；[包身份](selected-bundle-identity.json)含逐项bytes／SHA，9,033,994B、SHA `84deb05fc71d98adbec8b0d4fca1b2deeb9c5abe63e29fc2414503c58c1c2027`，父解包逐字节核对。完整472项[原私有清单](original-full-private-manifest.json)保留来源身份，但本包不复制私有Git元数据／checkout／完整fixture树，不声称472项全已公开。红检出随后被真实LF检出覆盖；父fresh重构CRLF只用于核记录，不冒称红文件仍现存。父首次parser误以首次输出也是TAP的错误另留[记录](parent-initial-parser-error.raw.txt)。

原独立结论No issues／OK with notes仅此精确属性修正；新CI资格另由上述独立登记支持。未做ffmpeg、媒体decode或新实体安装／签名／设备验收，不新增维护测试框架、配置选项或运行时归一化。
