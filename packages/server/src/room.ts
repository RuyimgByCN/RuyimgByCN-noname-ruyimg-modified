import bcrypt from "bcryptjs";

// 最小客户端接口（index.ts 的 Client extends WebSocket 天然满足，避免循环依赖）
export interface RoomClient {
	wsid: string;
	nickname: string;
	avatar: string;
	uid?: number;
	ready?: boolean;
	isObserver?: boolean;
	room?: Room;
	send: (data: string) => void;
	close?: () => void;
}

export interface Room {
	id: string;
	key: string;
	name: string;
	ownerUid: number;
	passwordHash?: string;
	mode: string;
	config: any;
	maxPlayers: number;
	players: RoomClient[];
	observers: RoomClient[];
	state: "waiting" | "playing" | "finished";
	readySet: Set<number>;
	createdAt: number;
}

export type Result<T> = ({ ok: true } & T) | { ok: false; error: string };

const rooms = new Map<string, Room>();      // key -> Room
const roomsById = new Map<string, Room>();  // id -> Room

const ID_ALPHABET = "ABCDEFGHJKMNPQRSTUVWXYZ23456789"; // 去除易混淆的 I/O/0/1

function genRoomId(): string {
	let id: string;
	do {
		id = "";
		for (let i = 0; i < 6; i++) id += ID_ALPHABET[Math.floor(Math.random() * ID_ALPHABET.length)];
	} while (roomsById.has(id));
	return id;
}

function genKey(): string {
	return Math.random().toString(36).slice(2, 12);
}

function sendl(client: RoomClient, ...args: any[]) {
	try { client.send(JSON.stringify(args)); } catch { /* ignore */ }
}

function broadcast(room: Room, ...args: any[]) {
	const data = JSON.stringify(args);
	for (const c of room.players) { try { c.send(data); } catch {} }
	for (const c of room.observers) { try { c.send(data); } catch {} }
}

function removeFromRoom(room: Room, client: RoomClient) {
	const pi = room.players.indexOf(client);
	if (pi !== -1) room.players.splice(pi, 1);
	const oi = room.observers.indexOf(client);
	if (oi !== -1) room.observers.splice(oi, 1);
	if (client.uid) room.readySet.delete(client.uid);
}

export function buildRoomInfo(room: Room) {
	return {
		id: room.id,
		name: room.name,
		mode: room.mode,
		maxPlayers: room.maxPlayers,
		state: room.state,
		hasPassword: !!room.passwordHash,
		ownerUid: room.ownerUid,
		players: room.players.map(c => ({
			wsid: c.wsid, nickname: c.nickname, avatar: c.avatar,
			uid: c.uid ?? null,
			ready: c.uid ? room.readySet.has(c.uid) : false,
			isOwner: c.uid === room.ownerUid,
		})),
		observers: room.observers.map(c => ({
			wsid: c.wsid, nickname: c.nickname, avatar: c.avatar, uid: c.uid ?? null,
		})),
	};
}

export function buildRoomListPublic(): any[] {
	const list: any[] = [];
	for (const room of rooms.values()) {
		list.push([
			room.id, room.name, room.mode,
			room.players.length, room.maxPlayers, room.state,
			!!room.passwordHash, room.ownerUid, room.key,
		]);
	}
	return list;
}

export function createRoom(
	client: RoomClient,
	opts: { name: string; mode: string; maxPlayers: number; password?: string; config?: any }
): Result<{ roomId: string; key: string }> {
	if (!client.uid) return { ok: false, error: "notLoggedIn" };
	if (client.room) return { ok: false, error: "alreadyInRoom" };
	const name = typeof opts.name === "string" ? opts.name.slice(0, 20) : "未命名房间";
	const mode = typeof opts.mode === "string" ? opts.mode : "identity";
	const maxPlayers = Math.max(2, Math.min(10, Number(opts.maxPlayers) || 8));
	const room: Room = {
		id: genRoomId(), key: genKey(), name, ownerUid: client.uid, mode,
		config: opts.config ?? {}, maxPlayers, players: [client], observers: [],
		state: "waiting", readySet: new Set([client.uid]), createdAt: Date.now(),
	};
	if (typeof opts.password === "string" && opts.password) room.passwordHash = bcrypt.hashSync(opts.password, 10);
	rooms.set(room.key, room);
	roomsById.set(room.id, room);
	client.room = room;
	return { ok: true, roomId: room.id, key: room.key };
}

function exitRoom(room: Room, client: RoomClient): { roomClosed: boolean } {
	removeFromRoom(room, client);
	if (client.uid === room.ownerUid) {
		if (room.players.length > 0) {
			const next = room.players[0];
			if (next.uid) {
				room.ownerUid = next.uid;
				room.readySet.add(next.uid);
				broadcast(room, "ownerChanged", next.uid, next.wsid);
			}
		} else {
			for (const o of room.observers) sendl(o, "roomClosed");
			rooms.delete(room.key);
			roomsById.delete(room.id);
			return { roomClosed: true };
		}
	}
	broadcast(room, "roomInfo", buildRoomInfo(room));
	return { roomClosed: false };
}

export function joinRoom(client: RoomClient, roomId: string, password?: string): Result<{ room: Room }> {
	if (!client.uid) return { ok: false, error: "notLoggedIn" };
	if (client.room) return { ok: false, error: "alreadyInRoom" };
	const room = roomsById.get(roomId);
	if (!room) return { ok: false, error: "roomNotFound" };
	if (room.state !== "waiting") return { ok: false, error: "inProgress" };
	if (room.passwordHash && (!password || !bcrypt.compareSync(password, room.passwordHash))) {
		return { ok: false, error: "wrongPassword" };
	}
	if (room.players.length >= room.maxPlayers) return { ok: false, error: "full" };
	room.players.push(client);
	client.room = room;
	broadcast(room, "roomInfo", buildRoomInfo(room));
	return { ok: true, room };
}

export function observe(client: RoomClient, roomId: string, password?: string): Result<{ room: Room }> {
	if (!client.uid) return { ok: false, error: "notLoggedIn" };
	if (client.room) return { ok: false, error: "alreadyInRoom" };
	const room = roomsById.get(roomId);
	if (!room) return { ok: false, error: "roomNotFound" };
	if (room.passwordHash && (!password || !bcrypt.compareSync(password, room.passwordHash))) {
		return { ok: false, error: "wrongPassword" };
	}
	room.observers.push(client);
	client.room = room;
	client.isObserver = true;
	broadcast(room, "roomInfo", buildRoomInfo(room));
	return { ok: true, room };
}

export function leaveRoom(client: RoomClient): Result<{}> {
	const room = client.room;
	if (!room || !rooms.has(room.key)) return { ok: false, error: "roomNotFound" };
	exitRoom(room, client);
	client.room = undefined;
	client.isObserver = undefined;
	client.ready = undefined;
	return { ok: true };
}

export function setReady(client: RoomClient, isReady: boolean): Result<{}> {
	const room = client.room;
	if (!room || !rooms.has(room.key)) return { ok: false, error: "roomNotFound" };
	if (!client.uid) return { ok: false, error: "notLoggedIn" };
	if (client.uid === room.ownerUid) return { ok: true }; // 房主恒就绪
	if (!room.players.includes(client)) return { ok: false, error: "notPlayer" };
	if (isReady) room.readySet.add(client.uid);
	else room.readySet.delete(client.uid);
	client.ready = isReady;
	broadcast(room, "roomInfo", buildRoomInfo(room));
	return { ok: true };
}

export function startGame(client: RoomClient): Result<{ config: any; slaves: RoomClient[] }> {
	const room = client.room;
	if (!room || !rooms.has(room.key)) return { ok: false, error: "roomNotFound" };
	if (!client.uid || client.uid !== room.ownerUid) return { ok: false, error: "notOwner" };
	if (room.state !== "waiting") return { ok: false, error: "alreadyPlaying" };
	for (const p of room.players) {
		if (p.uid && !room.readySet.has(p.uid)) return { ok: false, error: "notAllReady" };
	}
	room.state = "playing";
	// 每个玩家收到自己是否是房主的标记，让客户端区分 host/slave 模式
	for (const p of room.players) {
		sendl(p, "gameStarted", room.config, p.uid === room.ownerUid);
	}
	for (const obs of room.observers) {
		sendl(obs, "gameStarted", room.config, false);
	}
	const slaves = room.players.filter(p => p.uid !== room.ownerUid);
	return { ok: true, config: room.config, slaves };
}

export function kick(client: RoomClient, targetWsid: string): Result<{}> {
	const room = client.room;
	if (!room || !rooms.has(room.key)) return { ok: false, error: "roomNotFound" };
	if (!client.uid || client.uid !== room.ownerUid) return { ok: false, error: "notOwner" };
	const target = [...room.players, ...room.observers].find(c => c.wsid === targetWsid);
	if (!target) return { ok: false, error: "targetNotInRoom" };
	removeFromRoom(room, target);
	target.room = undefined;
	target.isObserver = undefined;
	sendl(target, "kicked");
	target.close?.();
	broadcast(room, "roomInfo", buildRoomInfo(room));
	return { ok: true };
}

export function transferOwner(client: RoomClient, targetWsid: string): Result<{}> {
	const room = client.room;
	if (!room || !rooms.has(room.key)) return { ok: false, error: "roomNotFound" };
	if (!client.uid || client.uid !== room.ownerUid) return { ok: false, error: "notOwner" };
	const target = room.players.find(c => c.wsid === targetWsid);
	if (!target || !target.uid) return { ok: false, error: "targetNotPlayer" };
	room.ownerUid = target.uid;
	room.readySet.add(target.uid);
	broadcast(room, "ownerChanged", target.uid, target.wsid);
	broadcast(room, "roomInfo", buildRoomInfo(room));
	return { ok: true };
}

export function setPassword(client: RoomClient, password?: string): Result<{}> {
	const room = client.room;
	if (!room || !rooms.has(room.key)) return { ok: false, error: "roomNotFound" };
	if (!client.uid || client.uid !== room.ownerUid) return { ok: false, error: "notOwner" };
	room.passwordHash = (typeof password === "string" && password) ? bcrypt.hashSync(password, 10) : undefined;
	broadcast(room, "roomInfo", buildRoomInfo(room));
	return { ok: true };
}

export function setConfig(client: RoomClient, config: any): Result<{}> {
	const room = client.room;
	if (!room || !rooms.has(room.key)) return { ok: false, error: "roomNotFound" };
	if (!client.uid || client.uid !== room.ownerUid) return { ok: false, error: "notOwner" };
	room.config = config;
	broadcast(room, "roomInfo", buildRoomInfo(room));
	return { ok: true };
}

export function handleDisconnect(client: RoomClient) {
	const room = client.room as Room | undefined;
	// 非新协议房间（不在 rooms Map）→ no-op，交给 index.ts 旧逻辑
	if (!room || !rooms.has(room.key)) return;
	exitRoom(room, client);
	client.room = undefined;
}

export function _resetForTests() {
	rooms.clear();
	roomsById.clear();
}
