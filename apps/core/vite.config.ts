import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

const port = {
	client: 8080,
	server: 8089,
};

export default defineConfig({
	appType: "mpa",
	root: ".",
	base: "./",
	resolve: {
		alias: {
			"@": "/noname",
			noname: "/noname.js",
		},
	},
	plugins: [
		vue(),
		{
			// vite 返回 text/html 不带 charset，且 @vite/client 被注入到 <meta charset> 之前，
			// 中文 locale 浏览器会锁定 GBK 编码，导致 UTF-8 classic 脚本里的中文乱码（SyntaxError）。
			// 修复：① HTTP 层声明 charset=utf-8 绕过编码嗅探；
			//       ② 剥离 HTML 请求的条件头，避免 304 命中浏览器缓存的旧（无 charset）响应头。
			name: "force-html-utf8",
			configureServer(server) {
				server.middlewares.use((req, res, next) => {
					if (req.url && /\.html?(?:$|\?)/.test(req.url)) {
						delete req.headers["if-none-match"];
						delete req.headers["if-modified-since"];
					}
					const end = res.end.bind(res);
					(res as { end: (...args: unknown[]) => unknown }).end = (...args: unknown[]) => {
						const ct = res.getHeader("content-type");
						if (typeof ct === "string" && ct.includes("text/html") && !ct.includes("charset")) {
							res.setHeader("content-type", "text/html; charset=utf-8");
						}
						return end(...args);
					};
					next();
				});
			},
		},
	],
	server: {
		host: "127.0.0.1",
		port: port.client,
		fs: {
			allow: ["../.."],
		},
		proxy: {
			"/checkFile": "http://127.0.0.1:" + port.server,
			"/checkDir": "http://127.0.0.1:" + port.server,
			"/readFile": "http://127.0.0.1:" + port.server,
			"/readFileAsText": "http://127.0.0.1:" + port.server,
			"/writeFile": "http://127.0.0.1:" + port.server,
			"/removeFile": "http://127.0.0.1:" + port.server,
			"/getFileList": "http://127.0.0.1:" + port.server,
			"/createDir": "http://127.0.0.1:" + port.server,
			"/removeDir": "http://127.0.0.1:" + port.server,
		},
	},
});
