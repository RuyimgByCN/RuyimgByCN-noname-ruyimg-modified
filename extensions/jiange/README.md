# 剑阁（jiange）fork 扩展迁移

- 来源：`archive/Noname-ruyimg-Modified/apps/core/character/jiange`（源自
  Noname-Liuli-Modified M4，GNU GPL v3）。
- 8 名武将全部入角色索引：浴火士元（庞统）/工神月英（黄月英）/天侯孔明（诸葛亮）/
  烈帝玄德（刘备）/绝尘妙才（夏侯渊）/佳人子丹（曹真）/巧魁儁乂（张郃）/
  断狱仲达（司马懿）；跨包依赖 shenhua（新神速/八阵复用既有定义）。

## 已发布（真实运行时）

- `jizhen`（激阵，刘备）：结束阶段可选至多两名已受伤角色各摸一张牌。
- `sfanshi`（反噬，司马懿）：锁定技，结束阶段失去 1 点体力。

## 元数据登记（无运行时行为）

- 天狱（多目标横置）、智囊（翻顶赠牌）、精妙（他人无懈过滤器）、变天（天气状态机）、
  灵锋（摸牌替换+亮牌联动）、绝尘（非杀目标弃牌）、持盈（多段伤害防止）、惊帆
  （用牌计数距离修正）、惑敌（未指定他人过滤器）、绝汲（目标摸牌削减）、玄雷
  （全场判定区雷伤）、控魂（多目标雷伤+等量回复）、奇雾（上游引用遗留占位）——
  依赖未移植机制，登记元数据 + 文档化近似，详见 `jiange_meta.skill`。

```bash
cargo run -q -p content-cli -- pack \
  content/original/ported/jiange/jiange_impl.skill \
  content/original/ported/jiange/jiange_meta.skill \
  --characters content/original/ported/jiange/characters.json \
  --metadata content/original/ported/jiange/metadata.json \
  --out content/original/ported/jiange/jiange.npack.json \
  --package-id jiange
```
