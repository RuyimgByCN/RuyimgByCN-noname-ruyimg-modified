# ADR 0241: 决斗对手多杀

`duel_opponent_response_count` 是参与者侧 modifier：每层令技能拥有者的【决斗】对手在每轮
响应中多打出一张【杀】，拥有者自己的响应数不变。

ABI 167 为【决斗】续效保存 `need/have`，实体【杀】和视为【杀】统一逐张响应。STANDARD
「无双」同时声明既有 `response_count add 1` 与本 modifier，完整覆盖旧 `wushuang1/2`。
Engine semantics 升至 173；六个 Native Golden 向量按新语义重钉，并继续要求 WASM 完全一致。
