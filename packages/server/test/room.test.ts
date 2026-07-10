import { test } from "node:test";
import assert from "node:assert/strict";
import * as room from "../src/room.ts";

// mock client：收集 send 的消息，模拟 close
function mkClient(uid?: number, wsid?: string) {
	const received: any[] = [];
	const c: any = {
		wsid: wsid ?? "ws_" + Math.random().toString(36).slice(2, 8),
		nickname: uid ? "p" + uid : "guest",
		avatar: "caocao",
		uid,
		send: (d: string) => received.push(JSON.parse(d)),
		close: () => { c._closed = true; },
		_received: received,
		_closed: false,
	};
	return c;
}

test.beforeEach(() => {
	room._resetForTests();
});

test("createRoom: 房主创建房间", () => {
	const c = mkClient(1);
	const r = room.createRoom(c, { name: "测试房", mode: "identity", maxPlayers: 8 });
	assert.equal(r.ok, true);
	if (r.ok) {
		assert.equal(r.roomId.length, 6);
		assert.ok(r.key);
		assert.equal(c.room?.id, r.roomId);
		assert.equal(c.room?.ownerUid, 1);
		assert.equal(c.room?.players.length, 1);
		assert.ok(c.room?.readySet.has(1));
		assert.equal(c.room?.state, "waiting");
	}
});

test("createRoom: 未登录拒绝", () => {
	const c = mkClient(); // 无 uid
	const r = room.createRoom(c, { name: "x", mode: "identity", maxPlayers: 8 });
	assert.equal(r.ok, false);
	if (!r.ok) assert.equal(r.error, "notLoggedIn");
});

test("createRoom: 密码房 bcrypt 哈希存储", () => {
	const c = mkClient(2);
	const r = room.createRoom(c, { name: "密码房", mode: "identity", maxPlayers: 8, password: "secret" });
	assert.equal(r.ok, true);
	assert.ok(c.room?.passwordHash);
	assert.doesNotMatch(c.room!.passwordHash!, /secret/);
});

test("buildRoomInfo: 不含 passwordHash，房主标记就绪", () => {
	const c = mkClient(3);
	room.createRoom(c, { name: "r", mode: "identity", maxPlayers: 8, password: "pw" });
	const info = room.buildRoomInfo(c.room!);
	assert.equal(info.hasPassword, true);
	assert.equal((info as any).passwordHash, undefined);
	assert.equal(info.players[0].isOwner, true);
	assert.equal(info.players[0].ready, true);
});

test("buildRoomListPublic: 列表项含 mode/人数/state", () => {
	const c = mkClient(4);
	room.createRoom(c, { name: "r", mode: "identity", maxPlayers: 8 });
	const list = room.buildRoomListPublic();
	assert.equal(list.length, 1);
	// [id, name, mode, playerCount, maxPlayers, state, hasPassword, ownerUid, key]
	const item = list[0];
	assert.equal(item[2], "identity");
	assert.equal(item[3], 1);
	assert.equal(item[4], 8);
	assert.equal(item[5], "waiting");
});

test("joinRoom: 正常加入并广播 roomInfo", () => {
	const owner = mkClient(10);
	room.createRoom(owner, { name: "r", mode: "identity", maxPlayers: 8 });
	const guest = mkClient(11);
	guest._received.length = 0;
	const r = room.joinRoom(guest, owner.room!.id);
	assert.equal(r.ok, true);
	assert.equal(owner.room!.players.length, 2);
	assert.ok(guest._received.some(m => m[0] === "roomInfo"));
});

test("joinRoom: 人满拒绝 full", () => {
	const owner = mkClient(20);
	room.createRoom(owner, { name: "r", mode: "identity", maxPlayers: 2 });
	room.joinRoom(mkClient(21), owner.room!.id);
	const r = room.joinRoom(mkClient(22), owner.room!.id);
	assert.equal(r.ok, false);
	if (!r.ok) assert.equal(r.error, "full");
});

test("joinRoom: 密码错拒绝 wrongPassword", () => {
	const owner = mkClient(30);
	room.createRoom(owner, { name: "r", mode: "identity", maxPlayers: 8, password: "pw" });
	const r = room.joinRoom(mkClient(31), owner.room!.id, "WRONG");
	assert.equal(r.ok, false);
	if (!r.ok) assert.equal(r.error, "wrongPassword");
});

test("joinRoom: 房间不存在 roomNotFound", () => {
	const r = room.joinRoom(mkClient(40), "NOPE00");
	assert.equal(r.ok, false);
	if (!r.ok) assert.equal(r.error, "roomNotFound");
});

test("observe: 观战加入 observers", () => {
	const owner = mkClient(50);
	room.createRoom(owner, { name: "r", mode: "identity", maxPlayers: 8 });
	const obs = mkClient(51);
	const r = room.observe(obs, owner.room!.id);
	assert.equal(r.ok, true);
	assert.equal(owner.room!.observers.length, 1);
	assert.equal(obs.isObserver, true);
});

test("leaveRoom: 离开后从 players 移除", () => {
	const owner = mkClient(60);
	room.createRoom(owner, { name: "r", mode: "identity", maxPlayers: 8 });
	const g = mkClient(61);
	room.joinRoom(g, owner.room!.id);
	const r = room.leaveRoom(g);
	assert.equal(r.ok, true);
	assert.equal(owner.room!.players.length, 1);
	assert.equal(g.room, undefined);
});

test("setReady: 玩家准备/取消更新 readySet", () => {
	const owner = mkClient(70);
	room.createRoom(owner, { name: "r", mode: "identity", maxPlayers: 8 });
	const g = mkClient(71);
	room.joinRoom(g, owner.room!.id);
	room.setReady(g, true);
	assert.ok(owner.room!.readySet.has(71));
	room.setReady(g, false);
	assert.ok(!owner.room!.readySet.has(71));
});

test("setReady: 观战者拒绝 notPlayer", () => {
	const owner = mkClient(72);
	room.createRoom(owner, { name: "r", mode: "identity", maxPlayers: 8 });
	const obs = mkClient(73);
	room.observe(obs, owner.room!.id);
	const r = room.setReady(obs, true);
	assert.equal(r.ok, false);
	if (!r.ok) assert.equal(r.error, "notPlayer");
});

test("startGame: 未全员就绪拒绝 notAllReady", () => {
	const owner = mkClient(80);
	room.createRoom(owner, { name: "r", mode: "identity", maxPlayers: 8 });
	const g = mkClient(81);
	room.joinRoom(g, owner.room!.id);
	const r = room.startGame(owner);
	assert.equal(r.ok, false);
	if (!r.ok) assert.equal(r.error, "notAllReady");
});

test("startGame: 全员就绪成功 + 广播 gameStarted", () => {
	const owner = mkClient(90);
	room.createRoom(owner, { name: "r", mode: "identity", maxPlayers: 8, config: { mode: "identity" } });
	const g = mkClient(91);
	room.joinRoom(g, owner.room!.id);
	room.setReady(g, true);
	owner._received.length = 0;
	g._received.length = 0;
	const r = room.startGame(owner);
	assert.equal(r.ok, true);
	assert.equal(owner.room!.state, "playing");
	assert.ok(owner._received.some(m => m[0] === "gameStarted"));
	assert.ok(g._received.some(m => m[0] === "gameStarted"));
});

test("startGame: 非房主拒绝 notOwner", () => {
	const owner = mkClient(100);
	room.createRoom(owner, { name: "r", mode: "identity", maxPlayers: 8 });
	const g = mkClient(101);
	room.joinRoom(g, owner.room!.id);
	const r = room.startGame(g);
	assert.equal(r.ok, false);
	if (!r.ok) assert.equal(r.error, "notOwner");
});

test("kick: 房主踢人，target 收 kicked 并关闭", () => {
	const owner = mkClient(110);
	room.createRoom(owner, { name: "r", mode: "identity", maxPlayers: 8 });
	const g = mkClient(111);
	room.joinRoom(g, owner.room!.id);
	const r = room.kick(owner, g.wsid);
	assert.equal(r.ok, true);
	assert.equal(owner.room!.players.length, 1);
	assert.ok(g._received.some(m => m[0] === "kicked"));
	assert.equal(g._closed, true);
	assert.equal(g.room, undefined);
});

test("kick: 非房主拒绝 notOwner", () => {
	const owner = mkClient(120);
	room.createRoom(owner, { name: "r", mode: "identity", maxPlayers: 8 });
	const g = mkClient(121);
	room.joinRoom(g, owner.room!.id);
	const r = room.kick(g, owner.wsid);
	assert.equal(r.ok, false);
	if (!r.ok) assert.equal(r.error, "notOwner");
});

test("kick: target 不在房 targetNotInRoom", () => {
	const owner = mkClient(122);
	room.createRoom(owner, { name: "r", mode: "identity", maxPlayers: 8 });
	const r = room.kick(owner, "nobody");
	assert.equal(r.ok, false);
	if (!r.ok) assert.equal(r.error, "targetNotInRoom");
});

test("transferOwner: 转让房主 + 广播 ownerChanged", () => {
	const owner = mkClient(130);
	room.createRoom(owner, { name: "r", mode: "identity", maxPlayers: 8 });
	const g = mkClient(131);
	room.joinRoom(g, owner.room!.id);
	const r = room.transferOwner(owner, g.wsid);
	assert.equal(r.ok, true);
	assert.equal(owner.room!.ownerUid, 131);
	assert.ok(g._received.some(m => m[0] === "ownerChanged"));
});

test("transferOwner: target 是观战者拒绝 targetNotPlayer", () => {
	const owner = mkClient(132);
	room.createRoom(owner, { name: "r", mode: "identity", maxPlayers: 8 });
	const obs = mkClient(133);
	room.observe(obs, owner.room!.id);
	const r = room.transferOwner(owner, obs.wsid);
	assert.equal(r.ok, false);
	if (!r.ok) assert.equal(r.error, "targetNotPlayer");
});

test("setPassword / setConfig: 仅房主", () => {
	const owner = mkClient(140);
	room.createRoom(owner, { name: "r", mode: "identity", maxPlayers: 8 });
	assert.equal(room.setPassword(owner, "newpw").ok, true);
	assert.ok(owner.room!.passwordHash);
	assert.equal(room.setConfig(owner, { foo: 1 }).ok, true);
	assert.deepEqual(owner.room!.config, { foo: 1 });
	const g = mkClient(141);
	room.joinRoom(g, owner.room!.id);
	assert.equal(room.setConfig(g, { x: 2 }).ok, false);
});

test("handleDisconnect: 房主掉线自动转让给最早加入者", () => {
	const owner = mkClient(150);
	room.createRoom(owner, { name: "r", mode: "identity", maxPlayers: 8 });
	const g = mkClient(151);
	room.joinRoom(g, owner.room!.id);
	const theRoom = owner.room!;
	g._received.length = 0;
	room.handleDisconnect(owner);
	assert.equal(owner.room, undefined);
	assert.equal(theRoom.ownerUid, 151);
	assert.equal(theRoom.players.length, 1);
	assert.ok(g._received.some(m => m[0] === "ownerChanged"));
});

test("handleDisconnect: 最后一人掉线删房", () => {
	const owner = mkClient(160);
	room.createRoom(owner, { name: "r", mode: "identity", maxPlayers: 8 });
	const roomId = owner.room!.id;
	room.handleDisconnect(owner);
	const r = room.joinRoom(mkClient(161), roomId);
	assert.equal(r.ok, false);
	if (!r.ok) assert.equal(r.error, "roomNotFound");
});

test("handleDisconnect: 普通玩家掉线移除", () => {
	const owner = mkClient(170);
	room.createRoom(owner, { name: "r", mode: "identity", maxPlayers: 8 });
	const g = mkClient(171);
	room.joinRoom(g, owner.room!.id);
	room.handleDisconnect(g);
	assert.equal(owner.room!.players.length, 1);
	assert.equal(g.room, undefined);
});

test("handleDisconnect: 观战者掉线移出 observers", () => {
	const owner = mkClient(180);
	room.createRoom(owner, { name: "r", mode: "identity", maxPlayers: 8 });
	const obs = mkClient(181);
	room.observe(obs, owner.room!.id);
	room.handleDisconnect(obs);
	assert.equal(owner.room!.observers.length, 0);
});

test("handleDisconnect: 非新协议房间 no-op", () => {
	const c = mkClient(190);
	c.room = { key: "legacy", owner: c } as any; // 模拟旧协议 Room
	room.handleDisconnect(c);
	assert.ok(c.room); // 未被清理
});
