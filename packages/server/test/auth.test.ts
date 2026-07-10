import { test, before, after } from "node:test";
import assert from "node:assert/strict";
import { register, login, verifyToken } from "../src/auth.ts";
import { db } from "../src/db.ts";

// 用户名上限 16 字符，用短计数器保证唯一且不超长
let _n = 0;
function uniq() {
	return "u" + ++_n;
}

// 测试前清理上次残留（noname.db 持久化，跨运行需清理 u% 测试用户）
before(() => {
	db.exec("DELETE FROM sessions; DELETE FROM users WHERE username LIKE 'u%';");
});

test("注册→登录→verifyToken 全流程", () => {
	const name = uniq();
	const r = register(name, "secret123");
	assert.equal(r.ok, true);

	const dup = register(name, "secret123");
	assert.equal(dup.ok, false);

	const bad = login(name, "wrong");
	assert.equal(bad.ok, false);

	const lg = login(name, "secret123");
	assert.equal(lg.ok, true);
	if (lg.ok) {
		const v = verifyToken(lg.token);
		assert.equal(v.ok, true);
		if (v.ok) assert.equal(v.uid, (r as { uid: number }).uid);
	}
});

test("校验规则", () => {
	assert.equal(register("a", "secret123").ok, false); // 用户名太短
	assert.equal(register(uniq(), "123").ok, false); // 密码太短
	assert.equal(verifyToken("nonexistent-token").ok, false);
});

// 测试后清理（避免污染 noname.db）
after(() => {
	db.exec("DELETE FROM sessions; DELETE FROM users WHERE username LIKE 'u%';");
});
