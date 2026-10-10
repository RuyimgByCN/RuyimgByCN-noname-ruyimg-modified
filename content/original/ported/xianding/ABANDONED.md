# 限定包（xianding）移植放弃清单

截至 2026-08-03（持续迁移，100 项技能已发布），旧 JS `xianding/skill.js` 中全部技能定义
均已逐项人工评估。除已移植者外，未移植技能因以下 DSL 原语缺失暂列放弃（技能名按
主类原因分组；括号内为同机制代表技能）：

## 1. 转换技（zhuanhuanji）无原语
阴阳两态互切 + 皮肤切换：bazhan、caiyi、dcsbbizuo、dcsbhengye、dcsbjingmou、dcsbjunmou、
dcsbmengmou、dcsbmoyou、dcsbquanmou、dcsbshimou、dcsbxixing、dcsbzhengong、dcsbzhouxi、
dcshoutan、dcquanshi、dclongnu、dcsbwuwei、fengliao、dcyaoyi、dcwoheng、
dcshoutan 等。

## 2. 武将牌上置牌（expansion/技能牌堆）无原语
dcyanzuo、dczuyin、dcpijian、dcruxian、dcxingwu、dcluoyan、dcwangyuan、dclingyin、
dclingxi、dczhifou、dczhengyue、dcjingzao、dcchangqu、dchangzhu、dcjincui、dcxiaoyin、
dcfozong、dcshuren、dcyitong、dcpeiniang、dcsaran、dcyuanrong、dczhimin 等。

## 3. 观看/展示手牌无原语
dcshangyi、dcjianzheng、dcfuji、dcsushou、dcsbxixing、dcsbyizhen、dcsbkongwu、dcshizha、
dcxialei、dcdiyi（不存在）、xicha、dcqingshi 等。

## 4. 牌无效/额外结算（excluded/extra resolution）无原语
dcenyu、dcsbhoude、dcsbqiaodui、dczhifou、dclaoyan、dcweiwei、dcwuxie（换手牌 swap 无）等。

## 5. 额外回合/跳过阶段无原语
dcwumei、dcsbbizuo、dcguili、dcfengying（崔毛）、dcshiji 等。

## 6. 废除/恢复装备栏无原语
dcduorui、dcshuren、dcsaran、dcxiankuang、dcdufeng、dcguangyong（部分）、dczhiti 等。

## 7. 动态牌名/点数转化无原语
dcjuewu、dcposuo、dcfuhui（view_as 动态家族限 from none）、dcsbyizhi（五谷丰登不在
VIEW_AS_KINDS）、dcjuxi、dcczchouxi、dcshouhu（已以 active+虚拟杀移植）等。

## 8. 判定分支/判定改写无原语
dccilv、dcqingshi、dcxiaoren、dczhantao、dcrenshuang、dchuayi、dcjilie、dcxiangmian、
dcdixian、dcsbshijin、dcjieying（神刘备）等。

## 9. 跨回合/跨轮记录存储无原语
dczhimin、dcpandi、dcjieshu（圆环）、dcmianyao、dcgongtu、dcsbyouyi、
dcyinjun、dcfumou、dcshouzhi、dczhengming（不存在）、dcyuanmo 等。

## 10. 技能授予/移除（addSkills/removeSkills 动态）无原语
dushi、dcpijing、dczimu、dcsbsushen、dcsbrushi、dcshangyu、huandao、
dczhizhe、dcxiangchen、dcjiesi、dclianji、dcmoucheng、zunjian、sejianchu、dclishi 等。

## 11. 主公技（zhuSkill）/势力联动无原语
yuwei、zhushi、dcjujian、dcsbtiancheng、dcsbdingnan、dcrengou、dcsbyizhi、dctongye、
dcjingzao（部分）、dcguangyong（部分）等。

## 12. 觉醒/限定技联动（换技能）无原语
mengqing、zhanyuan、choujue、zunjian、dcmoucheng、dcgusuan、dcjincui、sejianchu、
dcqijing、dcjianghuo、dcxunbie、dcmingding、dcruxian、dcyixian、syxiongyi、dczhiyuan（不存在）等。

## 13. 拼点复杂循环无原语
dckegou、dcshenduan、dcsbjibian、dcsbmingfang、dcxiongwei、dczhidui、dcjueyan、
dcsbkuangzhan、dcshiji 等。

## 14. 特定数字/花色/点数统计无原语
dczhangcai（点数8计数）、dcchixing（弃杀数）、dcmianyao（点数）、dcdianlun（等差）、
dctianji（三类同比）、dcjieshu（圆环）、dcxialei（红牌进弃牌堆）、dcdichou（部分已做）、
dcxiongmu、dczhimin 等。

## 15. 队友/阵营识别无原语
wlcuorui、dcfudao、dcdujun（部分已做）、dcjijie、dcsushou 等。

## 16. 复杂多分支选择链无原语（逐个列举）
dclianyou、dcliexiang（主体已做）、dcwuyan、dcguangyong、dcfozong、dccansi、dcjuchui、
dcjizhan、dczhuitao、dcbaguan、dcxiaowu、dcdulu、dcsbyingjia、dcjichun、dcwanchan、
dcsbbibu、dcdyqingshi、dczhanyu、dcqingzheng、dcshuren、dcquanshou（①已做）、
dcdixian、dcyuanrong、dcjincui、dcxianlve、dcjichao、dczhangji（排序未建模）等。

## 17. 历史/计数跟踪无原语
dcsbhengye（四数值+1）、dcsbqinqiang（连续同色）、dcrengou、dcsbfengmin、refenyin、
dcliying、dcyuhui、dczhengqing、dcsbshiao、dcjijie、dctongye 等。

## 18. 特定事件/时机无原语
dcshangyu（游戏开始）、dctianjie（洗牌历史）、dczhengqing（轮结束）、wfyuyan（秘密选角）、
guili（第一回合）、dcshizha（体力变化）、dcwuhun 死亡判定（已做标记部分）、dcmiyi
（跨阶段双执行）等。

## 19. 装备技能/武器牌技能（equipSkill）不属角色技能移植范围
pilitoushiche、dagongche_skill、pyzhuren_*、dcwanglu、dcxianzhu、dcchaixie、dcpoyuan、
dchuangnu、dcguangyong（大攻车部分）等。

## 20. 主公技专用/武将牌替换
dcyuandi、dcyuhui、dcxunbie（换武将牌）、dczhimin 等。

## 21. 旧版/衍生/子技能（不独立移植）
old_*（oldjue、old_dckegou、old_dcrenshuang、old_dcshizha、old_dcdixian、oldjinjie、
olddcxiangmian、old_dcgaojian、old_dcjiduan、old_dcjuanji、old_dcsbkongwu）、jiaoying2/3、
xingzuo2、lvli2-6、dushi2、xinzhoufu2、zhuihuan2_new、onedcspzhuiji、onedcspshichou、
twodcspzhuiji、twodcspshichou、spzhuilie（SP王双已做 spzhuilie2）、dcjianying（连招辅助）、
x_dc_*、y_dc_*、pytianjiang_move、busuan_angelbeats、spsongshu_block 等。

## 22. 其他无法表达
retianxiang_daxiaoqiao（继承天香）、dcmengchi（无武将归属孤技）、diancai（引擎 bug 已修复
并移植为典财）、dczhengque（不存在）、dcjingjin（不存在）、dczhiyuan（不存在）、
dcjiaohui（不存在）、dcdanqi（不存在）、dcmanxiang（不存在）等。
