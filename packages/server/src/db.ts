import { DatabaseSync } from "node:sqlite";
import fs from "node:fs";
import path from "node:path";

// 数据库文件落 packages/server/data/noname.db
const DB_PATH = path.join(import.meta.dirname, "..", "data", "noname.db");
fs.mkdirSync(path.dirname(DB_PATH), { recursive: true });

export const db = new DatabaseSync(DB_PATH);

db.exec(`
CREATE TABLE IF NOT EXISTS users (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  username TEXT UNIQUE NOT NULL,
  password_hash TEXT NOT NULL,
  salt TEXT NOT NULL,
  avatar TEXT NOT NULL DEFAULT '',
  created_at INTEGER NOT NULL,
  last_login INTEGER
);
CREATE TABLE IF NOT EXISTS sessions (
  token TEXT PRIMARY KEY,
  uid INTEGER NOT NULL,
  created_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL,
  FOREIGN KEY (uid) REFERENCES users(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_sessions_uid ON sessions(uid);
CREATE TABLE IF NOT EXISTS winrate (
  uid INTEGER NOT NULL,
  mode TEXT NOT NULL,
  role TEXT NOT NULL,
  win INTEGER NOT NULL DEFAULT 0,
  lose INTEGER NOT NULL DEFAULT 0,
  draw INTEGER NOT NULL DEFAULT 0,
  UNIQUE(uid, mode, role)
);
CREATE TABLE IF NOT EXISTS friends (
  uid INTEGER NOT NULL,
  friend_uid INTEGER NOT NULL,
  status TEXT NOT NULL DEFAULT 'pending',
  created_at INTEGER NOT NULL,
  UNIQUE(uid, friend_uid)
);
CREATE INDEX IF NOT EXISTS idx_friends_uid ON friends(uid);
CREATE INDEX IF NOT EXISTS idx_friends_friend_uid ON friends(friend_uid);
`);

// 预编译语句（node:sqlite 的 StatementSync：.get / .run / .all）
export const stmts = {
	getUserByName: db.prepare("SELECT * FROM users WHERE username = ?"),
	getUserById: db.prepare("SELECT * FROM users WHERE id = ?"),
	insertUser: db.prepare(
		"INSERT INTO users (username, password_hash, salt, created_at) VALUES (?, ?, ?, ?)"
	),
	updateLastLogin: db.prepare("UPDATE users SET last_login = ? WHERE id = ?"),
	insertSession: db.prepare(
		"INSERT INTO sessions (token, uid, created_at, expires_at) VALUES (?, ?, ?, ?)"
	),
	getSession: db.prepare("SELECT * FROM sessions WHERE token = ?"),
	deleteSession: db.prepare("DELETE FROM sessions WHERE token = ?"),

	// 战绩
	upsertWinrate: db.prepare(`
		INSERT INTO winrate (uid, mode, role, win, lose, draw) VALUES (?, ?, ?, ?, ?, ?)
		ON CONFLICT(uid, mode, role) DO UPDATE SET
		  win = win + excluded.win,
		  lose = lose + excluded.lose,
		  draw = draw + excluded.draw
	`),
	getWinrateByUid: db.prepare("SELECT mode, role, win, lose, draw FROM winrate WHERE uid = ?"),

	// 好友
	getFriendStatus: db.prepare(
		"SELECT status FROM friends WHERE uid = ? AND friend_uid = ?"
	),
	insertFriendRequest: db.prepare(
		"INSERT OR IGNORE INTO friends (uid, friend_uid, status, created_at) VALUES (?, ?, 'pending', ?)"
	),
	acceptFriendRequest: db.prepare(`
		UPDATE friends SET status = 'accepted' WHERE uid = ? AND friend_uid = ?
	`),
	// 接受后需同时在反方向插入一条 accepted 记录
	insertFriendAccepted: db.prepare(
		"INSERT OR REPLACE INTO friends (uid, friend_uid, status, created_at) VALUES (?, ?, 'accepted', ?)"
	),
	removeFriend: db.prepare(
		"DELETE FROM friends WHERE (uid = ? AND friend_uid = ?) OR (uid = ? AND friend_uid = ?)"
	),
	getFriendList: db.prepare(`
		SELECT u.id, u.username, u.avatar, f.status
		FROM friends f
		JOIN users u ON u.id = f.friend_uid
		WHERE f.uid = ?
		ORDER BY f.status DESC, u.username ASC
	`),
};
