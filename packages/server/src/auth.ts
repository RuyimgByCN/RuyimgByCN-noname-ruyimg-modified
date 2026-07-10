import bcrypt from "bcryptjs";
import { randomUUID } from "node:crypto";
import { stmts } from "./db.ts";

const TOKEN_TTL_MS = 30 * 24 * 60 * 60 * 1000; // 30 天

export type AuthResult<T> = ({ ok: true } & T) | { ok: false; error: string };

export function register(
	username: string,
	password: string
): AuthResult<{ uid: number }> {
	if (!username || username.length < 2 || username.length > 16) {
		return { ok: false, error: "用户名长度需 2-16" };
	}
	if (!password || password.length < 6) {
		return { ok: false, error: "密码至少 6 位" };
	}
	if (stmts.getUserByName.get(username)) {
		return { ok: false, error: "用户名已存在" };
	}
	const salt = bcrypt.genSaltSync(10);
	const hash = bcrypt.hashSync(password, salt);
	const now = Date.now();
	const info = stmts.insertUser.run(username, hash, salt, now) as {
		lastInsertRowid: number | bigint;
	};
	return { ok: true, uid: Number(info.lastInsertRowid) };
}

export function login(
	username: string,
	password: string
): AuthResult<{ uid: number; token: string }> {
	const user = stmts.getUserByName.get(username) as
		| { id: number; password_hash: string }
		| undefined;
	// 用户不存在与密码错误返回同一信息（防枚举）
	if (!user || !bcrypt.compareSync(password, user.password_hash)) {
		return { ok: false, error: "用户不存在或密码错误" };
	}
	const token = randomUUID();
	const now = Date.now();
	stmts.insertSession.run(token, user.id, now, now + TOKEN_TTL_MS);
	stmts.updateLastLogin.run(now, user.id);
	return { ok: true, uid: user.id, token };
}

export function verifyToken(token: string): AuthResult<{ uid: number }> {
	if (!token) return { ok: false, error: "无 token" };
	const row = stmts.getSession.get(token) as
		| { uid: number; expires_at: number }
		| undefined;
	if (!row) return { ok: false, error: "token 无效" };
	if (Date.now() > row.expires_at) {
		stmts.deleteSession.run(token);
		return { ok: false, error: "token 已过期" };
	}
	return { ok: true, uid: row.uid };
}
