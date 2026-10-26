// 冒烟与 node:test 共用生产存储/安装副本/内容锁用例，不复制实现。
import "./ts-register.mjs";
await import("./native-match-content.test.mjs");
