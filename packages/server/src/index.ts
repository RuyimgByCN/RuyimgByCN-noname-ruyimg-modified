import { WebSocket, WebSocketServer } from "ws";
import { register as doRegister, login as doLogin, verifyToken } from "./auth.ts";
import * as roomApi from "./room.ts";
import { stmts } from "./db.ts";
import fs from "node:fs";
import https from "node:https";

interface Client extends WebSocket {
	wsid: string;
	nickname: string;
	avatar: string;
	clientIp: string;
	onlineKey?: string;
	uid?: number;
	ready?: boolean;
	isObserver?: boolean;
	status?: string;
	owner?: Client;
	room?: Room;
	servermode?: boolean;
	beat?: boolean;
	keyCheck?: NodeJS.Timeout;
	heartbeat?: NodeJS.Timeout;
}

interface Room {
	key: string;
	owner?: Client;
	config?: any;
	servermode?: boolean;
}

interface EventItem {
	id: string;
	creator: string;
	nickname: string;
	avatar: string;
	utc: number;
	day: number;
	hour: number;
	content: string;
	members: string[];
}

//
// ========= 全局状态 ==========
//

const clients = new Map<string, Client>();
const rooms = new Map<string, Room>();
const events: EventItem[] = [];

const bannedKeys = new Set<string>();
const bannedIps = new Set<string>();
const bannedKeyWords: string[] = [];

const util = {
	nickname(str: any): string {
		return typeof str === "string" ? str.slice(0, 12) : "无名玩家";
	},

	isBanned(str: string): boolean {
		return bannedKeyWords.some(k => str.includes(k));
	},

	sendl(client: Client, ...args: any[]) {
		try {
			client.send(JSON.stringify(args));
		} catch {
			client.close();
		}
	},

	newId(): string {
		return Math.floor(1e9 + Math.random() * 9e9).toString();
	},

	buildRoomList(): any[] {
		const roomList: any[] = [];
		const clientCount = new Map<string, number>();

		// init counter
		rooms.forEach((room, key) => clientCount.set(key, 0));

		// count clients per room
		clients.forEach(c => {
			if (c.room && !c.servermode) {
				const key = c.room.key;
				clientCount.set(key, (clientCount.get(key) || 0) + 1);
			}
		});

		// build output list
		rooms.forEach((room, key) => {
			const count = clientCount.get(key) || 0;
			if (room.servermode) {
				roomList.push("server");
			} else if (room.owner && room.config) {
				if (count === 0) {
					util.sendl(room.owner, "reloadroom");
				}
				roomList.push([
					room.owner.nickname,
					room.owner.avatar,
					room.config,
					count,
					room.key,
				]);
			}
		});

		return roomList;
	},

	buildClientList(): any[] {
		const out: any[] = [];
		clients.forEach(c => {
			out.push([c.nickname, c.avatar, !c.room, c.status, c.wsid, c.onlineKey, c.uid ?? null]);
		});
		return out;
	},

	updateRooms() {
		const roomList = util.buildRoomList();
		const clientList = util.buildClientList();
		clients.forEach(c => {
			if (!c.room) util.sendl(c, "updaterooms", roomList, clientList);
		});
	},

	updateClients() {
		const list = util.buildClientList();
		clients.forEach(c => {
			if (!c.room) util.sendl(c, "updateclients", list);
		});
	},

	updateRoomListPublic() {
		const list = roomApi.buildRoomListPublic();
		clients.forEach(c => {
			if (!c.room) util.sendl(c, "roomListUpdate", list);
		});
	},

	checkEvents() {
		const now = Date.now();
		for (let i = 0; i < events.length; i++) {
			if (events[i].utc <= now) {
				events.splice(i--, 1);
			}
		}
		return events;
	},

	updateEvents() {
		util.checkEvents();
		clients.forEach(c => {
			if (!c.room) util.sendl(c, "updateevents", events);
		});
	},
};

const handlers = {
	create(
		client: Client,
		key: string,
		nickname: string,
		avatar: string,
		config: any,
		mode: string
	) {
		if (client.onlineKey !== key) return;

		client.nickname = util.nickname(nickname);
		client.avatar = avatar;

		const room: Room = { key, owner: client };
		rooms.set(key, room);

		client.room = room;
		delete client.status;

		util.sendl(client, "createroom", key);
		util.updateRooms();
	},

	enter(client: Client, key: string, nickname: string, avatar: string) {
		const room = rooms.get(key);
		if (!room) return util.sendl(client, "enterroomfailed");

		client.nickname = util.nickname(nickname);
		client.avatar = avatar;
		client.room = room;
		delete client.status;

		if (!room.owner) return util.sendl(client, "enterroomfailed");

		if (
			!room.config ||
			(room.config.gameStarted && (!room.config.observe || !room.config.observeReady))
		) {
			return util.sendl(client, "enterroomfailed");
		}

		client.owner = room.owner;
		util.sendl(room.owner, "onconnection", client.wsid);
		util.updateRooms();
	},

	changeAvatar(client: Client, nickname: string, avatar: string) {
		client.nickname = util.nickname(nickname);
		client.avatar = avatar;
		util.updateClients();
	},

	key(client: Client, id: any) {
		if (!id || typeof id !== "object") {
			util.sendl(client, "denied", "key");
			return client.close();
		}
		if (bannedKeys.has(id[0])) {
			bannedIps.add(client.clientIp);
			return client.close();
		}
		client.onlineKey = id[0];
		clearTimeout(client.keyCheck);
	},

	events(client: Client, cfg: any, id: string, type: string) {
		if (bannedKeys.has(id) || typeof id !== "string" || client.onlineKey !== id) {
			bannedIps.add(client.clientIp);
			client.close();
			return;
		}

		let changed = false;
		const now = Date.now();

		if (typeof cfg === "string") {
			// join / leave existing event
			for (let ev of events) {
				if (ev.id === cfg) {
					if (type === "join" && !ev.members.includes(id)) {
						ev.members.push(id);
						changed = true;
					}
					if (type === "leave") {
						const idx = ev.members.indexOf(id);
						if (idx !== -1) {
							ev.members.splice(idx, 1);
							if (ev.members.length === 0) {
								const index = events.indexOf(ev);
								events.splice(index, 1);
							}
							changed = true;
						}
					}
				}
			}
		} else if (
			cfg &&
			typeof cfg === "object" &&
			"utc" in cfg &&
			"day" in cfg &&
			"hour" in cfg &&
			"content" in cfg
		) {
			if (events.length >= 20) util.sendl(client, "eventsdenied", "total");
			else if (cfg.utc <= now) util.sendl(client, "eventsdenied", "time");
			else if (util.isBanned(cfg.content)) util.sendl(client, "eventsdenied", "ban");
			else {
				const item: EventItem = {
					...cfg,
					nickname: util.nickname(cfg.nickname),
					avatar: cfg.avatar || "caocao",
					creator: id,
					id: util.newId(),
					members: [id],
				};
				events.unshift(item);
				changed = true;
			}
		}

		if (changed) util.updateEvents();
	},

	config(client: Client, config: any) {
		const room = client.room;
		if (!room || room.owner !== client) return;

		if (room.servermode) {
			room.servermode = false;
		}
		room.config = config;
		util.updateRooms();
	},

	status(client: Client, str: any) {
		if (typeof str === "string") client.status = str;
		else delete client.status;
		util.updateClients();
	},

	send(client: Client, id: string, message: string) {
		const target = clients.get(id);
		if (target && target.owner === client) {
			try {
				target.send(message);
			} catch {
				target.close();
			}
		}
	},

	close(client: Client, id: string) {
		const target = clients.get(id);
		if (target && target.owner === client) target.close();
	},
	register(client: Client, username: string, password: string) {
		const r = doRegister(username, password);
		if (r.ok) util.sendl(client, "registerResult", true, r.uid);
		else util.sendl(client, "registerResult", false, r.error);
	},
	login(client: Client, username: string, password: string) {
		const r = doLogin(username, password);
		if (r.ok) {
			client.uid = r.uid;
			client.nickname = username;
			util.sendl(client, "loginResult", true, r.token, r.uid);
		} else {
			util.sendl(client, "loginResult", false, r.error);
		}
	},
	auth(client: Client, token: string) {
		const r = verifyToken(token);
		if (r.ok) {
			client.uid = r.uid;
			clearTimeout(client.keyCheck);
			if (!client.nickname) {
				const row = stmts.getUserById.get(r.uid) as { username: string } | undefined;
				if (row) client.nickname = row.username;
			}
			util.sendl(client, "authResult", true, r.uid);
		} else {
			util.sendl(client, "authResult", false, r.error);
		}
	},
	createRoom(client: Client, name: string, mode: string, maxPlayers: number, password: string | undefined, config: any) {
		const r = roomApi.createRoom(client as any, { name, mode, maxPlayers, password: password || undefined, config });
		if (r.ok) {
			util.sendl(client, "createRoomResult", true, r.roomId, r.key, roomApi.buildRoomInfo(client.room as any));
			util.updateRoomListPublic();
		} else util.sendl(client, "createRoomResult", false, r.error);
	},
	joinRoom(client: Client, roomId: string, password?: string) {
		const r = roomApi.joinRoom(client as any, roomId, password);
		if (r.ok) {
			util.sendl(client, "joinRoomResult", true, roomApi.buildRoomInfo(r.room));
			util.updateRoomListPublic();
		} else util.sendl(client, "joinRoomResult", false, r.error);
	},
	observe(client: Client, roomId: string, password?: string) {
		const r = roomApi.observe(client as any, roomId, password);
		if (r.ok) {
			util.sendl(client, "observeResult", true, roomApi.buildRoomInfo(r.room));
			util.updateRoomListPublic();
		} else util.sendl(client, "observeResult", false, r.error);
	},
	leaveRoom(client: Client) {
		const r = roomApi.leaveRoom(client as any);
		util.sendl(client, "leaveRoomResult", r.ok, r.ok ? undefined : (r as any).error);
		if (r.ok) util.updateRoomListPublic();
	},
	ready(client: Client, isReady: boolean) {
		const r = roomApi.setReady(client as any, isReady);
		if (!r.ok) util.sendl(client, "readyResult", false, r.error);
	},
	startGame(client: Client) {
		const r = roomApi.startGame(client as any);
		if (r.ok) {
			// 为所有 slave 设置 relay（slave.owner = ownerClient），
			// 游戏消息之后通过 send/onmessage/onclose 在 owner↔slave 间中继
			for (const slave of r.slaves) {
				const slaveClient = clients.get((slave as any).wsid) as Client | undefined;
				if (slaveClient) slaveClient.owner = client;
			}
			util.sendl(client, "startGameResult", true);
			util.updateRoomListPublic();
		} else util.sendl(client, "startGameResult", false, r.error);
	},
	// owner 调用 waitForPlayer/createServer() 后通知服务器准备好了，
	// 服务器此时才向 owner 发 onconnection，触发 lib.init.connection
	ownerReady(client: Client) {
		clients.forEach(c => {
			if (c.owner === client) {
				util.sendl(client, "onconnection", c.wsid);
			}
		});
	},
	kick(client: Client, targetWsid: string) {
		const r = roomApi.kick(client as any, targetWsid);
		if (!r.ok) util.sendl(client, "kickResult", false, r.error);
		else util.updateRoomListPublic();
	},
	transferOwner(client: Client, targetWsid: string) {
		const r = roomApi.transferOwner(client as any, targetWsid);
		if (!r.ok) util.sendl(client, "transferOwnerResult", false, r.error);
	},
	setPassword(client: Client, password?: string) {
		const r = roomApi.setPassword(client as any, password);
		if (!r.ok) util.sendl(client, "setPasswordResult", false, r.error);
	},
	setConfig(client: Client, config: any) {
		const r = roomApi.setConfig(client as any, config);
		if (!r.ok) util.sendl(client, "setConfigResult", false, r.error);
	},

	// ===== M7 战绩/好友 =====

	// owner 在游戏结束时上报结果，格式：results = [{wsid, role, outcome}]
	// outcome: 'win' | 'lose' | 'draw'
	gameResult(client: Client, mode: string, results: Array<{ wsid: string; role: string; outcome: string }>) {
		if (!client.uid) return;
		if (!Array.isArray(results)) return;
		const now = Date.now();
		for (const r of results) {
			if (!r || typeof r.wsid !== "string" || typeof r.role !== "string") continue;
			const target = clients.get(r.wsid);
			const uid = target?.uid;
			if (!uid) continue;
			const outcome = r.outcome === "win" ? [1, 0, 0] : r.outcome === "draw" ? [0, 0, 1] : [0, 1, 0];
			stmts.upsertWinrate.run(uid, mode || "identity", r.role || "unknown", ...outcome);
		}
	},

	getStats(client: Client, targetUid?: number) {
		const uid = targetUid || client.uid;
		if (!uid) return util.sendl(client, "statsResult", false, "notLoggedIn");
		const user = stmts.getUserById.get(uid) as any;
		if (!user) return util.sendl(client, "statsResult", false, "userNotFound");
		const stats = stmts.getWinrateByUid.all(uid) as any[];
		util.sendl(client, "statsResult", true, {
			uid,
			username: user.username,
			avatar: user.avatar,
			stats,
		});
	},

	getFriendList(client: Client) {
		if (!client.uid) return util.sendl(client, "friendListResult", false, "notLoggedIn");
		const rows = stmts.getFriendList.all(client.uid) as any[];
		// 标记在线状态
		const onlineUids = new Set<number>();
		clients.forEach(c => { if (c.uid) onlineUids.add(c.uid); });
		const friends = rows.map(r => ({
			uid: r.id,
			username: r.username,
			avatar: r.avatar,
			status: r.status,       // 'pending' | 'accepted'
			online: onlineUids.has(r.id),
		}));
		util.sendl(client, "friendListResult", true, friends);
	},

	addFriend(client: Client, targetUid: number) {
		if (!client.uid) return util.sendl(client, "addFriendResult", false, "notLoggedIn");
		if (targetUid === client.uid) return util.sendl(client, "addFriendResult", false, "cannotAddSelf");
		const target = stmts.getUserById.get(targetUid) as any;
		if (!target) return util.sendl(client, "addFriendResult", false, "userNotFound");
		// 检查反向是否已有好友申请（对方已先申请我 → 直接接受）
		const reverse = stmts.getFriendStatus.get(targetUid, client.uid) as any;
		if (reverse) {
			if (reverse.status === "accepted") {
				// 双方都已接受，重复操作
				return util.sendl(client, "addFriendResult", false, "alreadyFriends");
			}
			// 对方已申请我 → 直接双向接受
			stmts.acceptFriendRequest.run(targetUid, client.uid);
			stmts.insertFriendAccepted.run(client.uid, targetUid, Date.now());
			util.sendl(client, "addFriendResult", true, "accepted");
			// 通知对方申请被接受
			clients.forEach(c => {
				if (c.uid === targetUid) util.sendl(c, "friendAccepted", client.uid);
			});
			return;
		}
		stmts.insertFriendRequest.run(client.uid, targetUid, Date.now());
		util.sendl(client, "addFriendResult", true, "pending");
		// 通知目标用户（如在线）
		clients.forEach(c => {
			if (c.uid === targetUid) util.sendl(c, "friendRequest", client.uid, (stmts.getUserById.get(client.uid) as any)?.username);
		});
	},

	acceptFriend(client: Client, fromUid: number) {
		if (!client.uid) return util.sendl(client, "acceptFriendResult", false, "notLoggedIn");
		const req = stmts.getFriendStatus.get(fromUid, client.uid) as any;
		if (!req || req.status !== "pending") return util.sendl(client, "acceptFriendResult", false, "noRequest");
		stmts.acceptFriendRequest.run(fromUid, client.uid);
		stmts.insertFriendAccepted.run(client.uid, fromUid, Date.now());
		util.sendl(client, "acceptFriendResult", true);
		clients.forEach(c => {
			if (c.uid === fromUid) util.sendl(c, "friendAccepted", client.uid);
		});
	},

	removeFriend(client: Client, targetUid: number) {
		if (!client.uid) return util.sendl(client, "removeFriendResult", false, "notLoggedIn");
		stmts.removeFriend.run(client.uid, targetUid, targetUid, client.uid);
		util.sendl(client, "removeFriendResult", true);
	},
};

// ===== 启动配置（环境变量）=====
// PORT: 监听端口（平台注入或自定义，默认 8082）
// HOST: 监听地址（默认 0.0.0.0 对外开放；0.0.0.0 适合反向代理场景）
// TLS_CERT / TLS_KEY: 启用 wss://，提供证书路径即开启（不提供则裸 ws://，建议配合反向代理）
// ALLOW_ORIGIN: 允许的页面来源（逗号分隔，如 https://game.example.com）。不设则不校验
const PORT = parseInt(process.env.PORT || "8082", 10);
const HOST = process.env.HOST || "0.0.0.0";
const TLS_CERT = process.env.TLS_CERT || "";
const TLS_KEY = process.env.TLS_KEY || "";
const ALLOW_ORIGINS = process.env.ALLOW_ORIGIN
	? process.env.ALLOW_ORIGIN.split(",").map(s => s.trim()).filter(Boolean)
	: [];

// verifyClient: Origin 校验（防跨域滥用）
function verifyClient(info: { origin: string; req: any }, cb: (ok: boolean) => void) {
	if (ALLOW_ORIGINS.length === 0) return cb(true); // 未配置则放行
	const origin = info.origin || "";
	if (!origin || ALLOW_ORIGINS.includes(origin)) return cb(true);
	cb(false);
}

let wss: WebSocketServer;
if (TLS_CERT && TLS_KEY) {
	// 服务器直接做 TLS 终结（wss://）
	const server = https.createServer({
		cert: fs.readFileSync(TLS_CERT),
		key: fs.readFileSync(TLS_KEY),
	});
	wss = new WebSocketServer({ server, verifyClient });
	server.listen(PORT, HOST, () => {
		console.log(`Server listening on wss://${HOST}:${PORT} (TLS)`);
	});
} else {
	// 裸 ws://（生产建议前置 nginx/Caddy 做 TLS 反代）
	wss = new WebSocketServer({ port: PORT, host: HOST, verifyClient });
	console.log(`Server listening on ws://${HOST}:${PORT}`);
}

wss.on("connection", (ws, req) => {
	const client = ws as Client;
	const ip = req.socket.remoteAddress ?? "";

	// ban check
	if (bannedIps.has(ip)) {
		util.sendl(client, "denied", "banned");
		return setTimeout(() => ws.close(), 500);
	}

	client.wsid = util.newId();
	client.clientIp = ip;
	clients.set(client.wsid, client);

	client.keyCheck = setTimeout(() => {
		util.sendl(client, "denied", "key");
		setTimeout(() => client.close(), 500);
	}, 2000);

	util.sendl(
		client,
		"roomlist",
		util.buildRoomList(),
		util.checkEvents(),
		util.buildClientList(),
		client.wsid
	);

	// heartbeat
	client.heartbeat = setInterval(() => {
		if (client.beat) {
			client.close();
			clearInterval(client.heartbeat);
			return;
		}
		client.beat = true;
		try {
			client.send("heartbeat");
		} catch {
			client.close();
		}
	}, 60000);

	//
	// message handler
	//
	client.on("message", msg => {
		const raw = msg.toString();
		if (raw === "heartbeat") {
			client.beat = false;
			return;
		}

		// forward from slave to owner (strip "server" prefix so owner's lib.message.server can dispatch)
		if (client.owner) {
			let relayRaw = raw;
			try {
				const arr = JSON.parse(raw);
				if (Array.isArray(arr) && arr[0] === "server") {
					arr.shift();
					relayRaw = JSON.stringify(arr);
				}
			} catch {}
			util.sendl(client.owner, "onmessage", client.wsid, relayRaw);
			return;
		}

		let arr: any[];
		try {
			arr = JSON.parse(raw);
			if (!Array.isArray(arr)) throw new Error();
		} catch {
			util.sendl(client, "denied", "banned");
			return;
		}

		if (arr.shift() !== "server") return;

		const type = arr.shift();
		const handler = (handlers as any)[type];
		if (!handler) return;

		handler(client, ...arr);
	});

	//
	// disconnect handler
	//
	client.on("close", () => {
		// 新协议房间清理（房主转让 / 空房删除）
		roomApi.handleDisconnect(client as any);

		// remove rooms owned by this client
		rooms.forEach((room, key) => {
			if (room.owner === client) {
				// notify all clients in this room
				clients.forEach(c => {
					if (c.room === room && c !== client) {
						util.sendl(c, "selfclose");
					}
				});
				rooms.delete(key);
			}
		});

		// notify owner if client was slave
		if (client.owner) util.sendl(client.owner, "onclose", client.wsid);

		clients.delete(client.wsid);

		util.updateRoomListPublic();
		if (client.room) util.updateRooms();
		else util.updateClients();
	});
});

export { wss };
