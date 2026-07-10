import { test } from "node:test";
import assert from "node:assert/strict";
import { db, stmts } from "../src/db.ts";

test("users/sessions 表存在且可读写", () => {
	const now = Date.now();
	const info = stmts.insertUser.run("__test__", "hash", "salt", now) as {
		lastInsertRowid: number | bigint;
	};
	const uid = Number(info.lastInsertRowid);

	const row = stmts.getUserByName.get("__test__") as { username: string };
	assert.equal(row.username, "__test__");

	stmts.insertSession.run("tok-1", uid, now, now + 1000);
	const sess = stmts.getSession.get("tok-1") as { uid: number };
	assert.equal(sess.uid, uid);

	stmts.deleteSession.run("tok-1");
	db.prepare("DELETE FROM users WHERE id = ?").run(uid);
});
