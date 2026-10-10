# 旧服包确定性移植

| 武将 | 体力 | 技能 | 状态 |
|---|---:|---|---|
| `weiyan`（旧魏延） | 4/4 | `shenhua:kuanggu`（狂骨） | 完整单技能武将；距离、受伤状态与等量回复有 Golden |
| `re_yujin`（毅重于禁） | 4/4 | `yijiang:yizhong`（毅重） | 完整单技能武将；跨包依赖已移植 |
| `ol_huaxiong`（旧华雄） | 6/6 | `refresh:new_reyaowu`（耀武） | 红杀来源强制摸一或回复一；非红杀受伤者摸一 |
| `old_xusheng`（旧徐盛） | 4/4 | `yijiang:pojun`（破军） | 【杀】伤害后按受伤者当前体力（至多 5）摸牌并翻面 |
| `huangzhong`（旧黄忠） | 4/4 | `shenhua:liegong`（烈弓） | 完整单技能武将；逐目标条件、可选发动和不可响应均有 Golden |
| `pangde`（旧庞德） | 4/4 | `standard:mashu`（马术）、`shenhua:mengjin`（猛进） | 完整双技能武将；足量闪避时机、目标区域弃牌与万箭隔离均有 Golden |
| `madai`（旧马岱） | 4/4 | `standard:mashu`（马术）、`yijiang:oldqianxi`（旧潜袭） | 完整双技能武将；近距离逻辑杀、判定分支、防伤与减上限均有 Golden |
| `masu`（旧马谡） | 3/3 | `yijiang:xinzhan`（心战）、`yijiang:huilei`（挥泪） | 完整双技能武将；私看牌堆顶、红桃子集获得、原序留顶与死亡惩罚均有 Golden |
| `old_gaoshun`（旧高顺） | 4/4 | `yijiang:xianzhen`（陷阵）、`yijiang:jinjiu`（禁酒） | 完整双技能武将；拼点胜负、目标绑定强化、失败禁【杀】与回合结束清理均有 Golden |
| `xuhuang`（旧徐晃） | 4/4 | `shenhua:gzduanliang`（断粮） | 完整单技能武将；黑色基本/装备转兵粮与距离 2 均复用现有规则并有 Golden |
| `old_zhuzhi`（旧朱治） | 4/4 | `yijiang:anguo`（安国） | 完整单技能武将；目标装备选择、收回手牌、攻击范围缩减判定与限次均有 Golden |
| `old_chendao`（旧陈到） | 4/4 | `shenhua:drlt_wanglie`（往烈） | 完整单技能武将；首牌无距离、当前牌不可响应、阶段内封牌与清理均复用 SHENHUA 实现 |
| `old_zhangxingcai`（旧张星彩） | 3/3 | `old:oldshenxian`（旧甚贤）、`sp:qiangwu`（枪舞） | 完整双技能武将；判定点数、严格大小边界、距离与次数策略均有 Golden |
| `old_guanzhang`（旧关兴张苞） | 4/4 | `yijiang:old_fuhun`（父魂） | 完整单技能武将；替换摸牌、公开亮牌、异色临时武圣/咆哮及回合末清理均有 Golden |
| `old_zhugezhan`（旧诸葛瞻） | 3/3 | `old:old_zuilun`（罪论）、`old:old_fuyin`（父荫） | 完整双技能武将；手牌/装备各取一次、目标补摸与阶段状态均有 Golden |
| `old_handang`（旧韩当） | 4/4 | `old:oldgongji`（弓骑）、`old:oldjiefan`（解烦） | 完整双技能武将；实体/弓骑转化杀、普通闪响应、防伤与虚拟桃救治复用现有卡牌管线 |
| `two_yj_hanbing`（牢寒冰剑少女） | 4/4 | `old:chegu`（彻骨）、`old:jianrou`（剑柔） | 完整双技能武将；取消原牌目标、两种弃牌形态与同色/同类别回合递增均有 Golden |
| `old_caocao`（旧神曹操） | 3/3 | `old:junkguixin`（归心）、`standard:feiying`（飞影） | 完整双技能武将；可获得未上场武将的主公技，或持久修改其他角色势力 |
| `yuji`（旧于吉） | 3/3 | `old:old_guhuo`（蛊惑） | 完整单技能武将；基本/普通锦囊声明、逐座质疑、验真奖惩与用牌/响应恢复均走确定性管线 |
| `x_yao_yuanshu`（牢爻袁术） | 4/4 | `old:yao_yaoyi`（爻疑）、`old:yao_chenwei`（谶纬） | 完整双技能武将；背置集合、动态转化、每回合牌名限次及交替翻牌/取牌均有 Golden |
| `junk_sunquan`（牢神孙权） | 4/4 | `extra:dili`（帝力）、`extra:yuheng`（驭衡） | 复用 EXTRA 已移植技能 |
| `new_simayi`（牢神司马懿） | 4/4 | `extra:jilin`（戢鳞）、`extra:yingyou`（英猷）、`extra:yingtian`（应天） | 复用 EXTRA 已移植技能 |
| `old_huanghao`（旧黄皓） | 3/3 | `old:oldqinqing`（寝情）、`old:oldhuisheng`（贿生） | 复用 Old 已移植技能 |
| `old_lingtong`（旧凌统） | 4/4 | `old:oldxuanfeng`（旋风） | 复用 Old 已移植技能 |
| `old_dingfeng`（旧丁奉） | 4/4 | `sp:fenxun`（奋迅）、`sp:duanbing`（短兵） | 复用 SP 已移植技能；任意牌成本与距离 1 追加目标均有 Golden |
| `old_zhuran`（旧朱然） | 4/4 | `yijiang:olddanshou`（胆守） | 造成伤害后可摸牌并终止当前结算及阶段，不计为跳过阶段 |
| `old_caozhen`（旧曹真） | 4/4 | `yijiang:sidi`（司敌） | 【闪】积累公开牌，移去一张令其他角色本阶段少用一张【杀】 |

原仓库把武将放在 `old`，技能定义分别来自 `standard`、`shenhua`、`yijiang`、`refresh` 与
`sp`。运行时需同时加载对应包。

`old_fuyin`（父荫）：无防具时，不能成为手牌数不小于自己的其他角色使用【杀】、【决斗】或
【火攻】的目标。

`oldgongji`（弓骑）：手牌或装备区中的装备牌可当无距离限制的【杀】使用或打出。

`oldshenxian`（旧甚贤）：继承新版甚贤的“回合外、其他角色因弃置失去基本牌”检测并可摸
1 张牌，但旧 JS 不添加 `shenxian2`，因此同一角色回合内可重复发动。

`oldfaen`（旧法恩）：一名角色翻面后，或由未横置变为横置后，可令该角色摸一张牌；解除
横置不会触发。

`oldre_liubiao`（RE 刘表）：复用 YIJIANG「自守」「宗室」闭合，不复制技能实现。

`old_liubiao`（旧刘表）：`oldzishou` 在受伤时按已损失体力增加本次摸牌数，并通过
ABI 100 的通用 `skip_next_play_phase` 安排跳过随后出牌阶段；「宗室」继续复用
YIJIANG 实现。

`old_chendao`（旧陈到）：技能定义保留在 SHENHUA，`old` 只声明武将和精确包依赖；
`drlt_wanglie` 由旧 JS 语义重写为 ABI 115 声明，运行时不加载旧 JavaScript。

`old_guanzhang`（旧关兴张苞）：技能定义保留在 YIJIANG，`old` 只声明武将并复用已有精确
依赖；`old_fuhun` 由旧 JS 语义重写为 ABI 225 声明，运行时不加载旧 JavaScript。

`oldanxu`（安恤）：旧 JS 重写为 ABI 262 声明；两名其他角色必须手牌数不等，由少者
获得多者一张手牌，所得牌不为黑桃时技能拥有者摸一张。ABI 262 同时为效果标签增加
兼容旧 npack 的 u16 扩展编码。

`old_jijun`（集军）：旧 JS 重写为 ABI 263 声明；直接复用主动技选牌输入和通用技能
扩展牌堆，不引入技能专用选择状态。

`old_fangtong`（方统）：旧 JS 重写为 ABI 264 声明；结束阶段检查 `old_jijun` 的“方”
牌堆，达到 36 张时按身份映射到现有主忠、反贼或内奸阵营胜利。

`oldqinqing`（寝情）：旧 JS 重写为 ABI 265 声明；目标必须攻击范围覆盖主公，依次
结算弃牌、目标摸牌和与主公的手牌数比较。

`oldmingjian`（明鉴）：旧 JS 重写为 ABI 266 声明；交出全部手牌后立即结束当前回合，
目标无视翻面状态获得一个仅包含出牌阶段的插入阶段，结束后从原回合拥有者继续正常轮转。

`oldxuanfeng`（旋风）：旧 JS 重写为 ABI 267 声明；失去装备区牌后选择一名合法目标，
复用普通虚拟【杀】响应/伤害管线，距离为 1 时也可改为直接造成 1 点伤害。

`jianrou`（剑柔）：旧 JS 重写为 ABI 268 声明；每轮一次弃两张手牌/装备牌防止当前
绑定伤害，两牌同色或同类别时摸一张并清除本轮发动记录。

`oldhuisheng`（贿生）：旧 JS 重写为 ABI 269 声明；每名伤害来源限一次，拥有者展示
任意张手牌/装备牌，来源获得其中一张并防止绑定伤害，或弃置等量牌。

`oldjiefan`（解烦）：旧 JS 重写为 ABI 270 声明；在其他角色的濒死窗口选择实体或
`oldgongji` 转化【杀】，该次【杀】仍走普通目标与【闪】响应链，绑定伤害改为虚拟【桃】救治。

`chegu`（彻骨）：旧 JS 重写为 ABI 271 声明；发动后取消本次非装备牌的全部目标，
再按“一人多牌/多人一牌”顺序弃置；弃牌同色或同类别时仅记录本回合递增值。

`junkguixin`（归心）：旧 JS 重写为 ABI 272 声明；主公技候选来自已加载但未上场的武将，
势力变更写入权威状态并参与状态哈希和所有势力查询。

`old_guhuo`（蛊惑）：旧 JS 重写为 ABI 273 声明；声明继续复用 source-free view-as
的合法目标与响应入口，新增通用的隐藏来源、逐座质疑、验真和原动作恢复 continuation。

`yao_yaoyi`（爻疑）与 `yao_chenwei`（谶纬）：旧 JS 重写为 ABI 274/275 声明；
背置手牌进入权威状态、状态哈希和观察者投影，离手统一清理，谶纬复用同一集合交替翻牌/取牌。

`old_guanyinping`（旧关银屏）：三个技能均由 SP 包提供；`xueji_old` 使用动态已损失体力
目标上限与红色手牌/装备成本，`oldhuxiao` 复用一次逻辑【杀】结算记录返还次数，
`oldwuji` 复用回合累计伤害在三点时觉醒，OLD 包只登记完整武将闭包。

`old_zhuhuan`（旧朱桓）：通过 YIJIANG 依赖授予完整 `youdi`。

`old_wuyi`（旧吴懿）：通过 YIJIANG 依赖授予旧版 `benxi`；不复用四选二的 OL/界版奔袭。

`panfeng`（旧潘凤）：通过 SP 依赖授予完整 `kuangfu`；装备移动复用标准装备栏替换规则。

`old_caochun`（旧曹纯）：通过 SP 依赖授予完整 `shanjia`；装备使用累计、七张上限及弃装备
后的无距离虚拟【杀】均有 Golden。

`old_zhaoyun`（新杀赵云）：复用 STANDARD `longdan`，并通过 REFRESH 依赖授予完整
`new_yajiao`；公开牌、任意角色获得与类别不同时的强制弃牌均有 Golden。

`old_chenqun`（旧陈群）：通过 YIJIANG 依赖授予由旧 JS 重写的 `dingpin`，并复用 OLD
`oldfaen`；定品按本回合用牌/弃牌类别过滤，红判定仅翻面，黑判定才锁定目标。

`old_zhuran`（旧朱然）：通过 YIJIANG 依赖授予 ABI 286 重写的 `olddanshou`。发动后摸一张牌，
清空共享处理区并终止当前结算，再结束当前阶段；阶段正常推进且不增加跳过阶段计数。

`old_caozhen`（旧曹真）：通过 YIJIANG 依赖授予 ABI 287 重写的 `sidi`。使用【闪】后将牌堆顶
牌置为公开技能牌；其他角色出牌阶段开始时可移去一张并获得带同名标记的临时减【杀】次数技能，
阶段结束时同时清除技能与标记。

`old_zhugezhan`（旧诸葛瞻）：`old_zuilun` 由旧 JS 语义重写为 ABI 226 声明，每个出牌阶段
分别从其他角色的手牌区和装备区获得至多一张牌，移动触发完成后目标摸一张；与既有
`old_fuyin` 闭合，旧 JavaScript 和临时子技能不进入 npack 或运行时。

重新生成仅武将包：

```bash
cargo run -q -p content-cli -- pack \
  content/original/ported/old/*.skill \
  --characters content/original/ported/old/characters.json \
  --metadata content/original/ported/old/metadata.json \
  --out content/original/ported/old/old.npack.json \
  --package-id old
```
