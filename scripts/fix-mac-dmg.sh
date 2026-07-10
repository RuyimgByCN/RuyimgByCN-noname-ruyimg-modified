#!/bin/bash
# 修复 electron-builder 打 dmg 时 Electron Framework 二进制丢失的问题
#
# 问题:electron-builder 26.7.0 在 identity:null(未签名) + arm64 组合下,
# 打 dmg 时会丢失 Electron Framework 的 166M 主二进制(Versions/A/Electron Framework),
# 导致安装后启动崩溃 "Library not loaded: @rpath/Electron Framework.framework"。
#
# 本脚本:用 hdiutil 手动制作 dmg(绕过 electron-builder 的 dmg 打包),保留所有文件。
#
# 用法:在 Noname-ruyimg-Modified/ 根目录执行 ./scripts/fix-mac-dmg.sh
# 前提:先跑过 pnpm build:mac(生成了 output/mac-arm64 和 output/mac)

set -e

OUTPUT_DIR="output"
TMP_DMG="/tmp/noname-fix.dmg"
TMP_MNT="/tmp/noname-fix-mnt"

make_dmg() {
	local src_app="$1"
	local dest_dmg="$2"
	local arch="$3"

	echo "=== 制作 $arch dmg ==="

	# 验证源 app 的 Framework 二进制存在
	local fw_bin="$src.app/Contents/Frameworks/Electron Framework.framework/Versions/A/Electron Framework"
	if [ ! -f "$src_app/Contents/Frameworks/Electron Framework.framework/Versions/A/Electron Framework" ]; then
		echo "✗ 源 app 缺少 Electron Framework 二进制: $src_app"
		return 1
	fi
	local fw_size=$(ls -lh "$src_app/Contents/Frameworks/Electron Framework.framework/Versions/A/Electron Framework" | awk '{print $5}')
	echo "源 app Framework 二进制: $fw_size"

	# 创建可读写 dmg
	rm -f "$TMP_DMG"
	hdiutil create -size 8g -volname "noname" -fs HFS+ -type UDIF "$TMP_DMG" >/dev/null 2>&1

	# 挂载
	hdiutil attach "$TMP_DMG" -nobrowse -mountpoint "$TMP_MNT" >/dev/null 2>&1

	# 拷贝 app(保留所有属性,含符号链接和权限)
	cp -pR "$src_app" "$TMP_MNT/"

	# 验证拷贝后 Framework
	local cp_size=$(ls -lh "$TMP_MNT/noname.app/Contents/Frameworks/Electron Framework.framework/Versions/A/Electron Framework" | awk '{print $5}')
	echo "dmg 内 Framework 二进制: $cp_size"

	# 卸载
	hdiutil detach "$TMP_MNT" >/dev/null 2>&1

	# 转换为压缩 dmg
	rm -f "$dest_dmg"
	hdiutil convert "$TMP_DMG" -format UDZO -imagekey zlib-level=9 -o "$dest_dmg" >/dev/null 2>&1
	rm -f "$TMP_DMG"

	echo "✓ $arch dmg 完成: $dest_dmg ($(ls -lh "$dest_dmg" | awk '{print $5}'))"
}

# arm64
if [ -d "$OUTPUT_DIR/mac-arm64/noname.app" ]; then
	make_dmg "$OUTPUT_DIR/mac-arm64/noname.app" "$OUTPUT_DIR/noname-1.11.4-arm64.dmg" "arm64"
else
	echo "跳过 arm64(output/mac-arm64/noname.app 不存在,请先跑 pnpm build:mac)"
fi

# x64
if [ -d "$OUTPUT_DIR/mac/noname.app" ]; then
	make_dmg "$OUTPUT_DIR/mac/noname.app" "$OUTPUT_DIR/noname-1.11.4.dmg" "x64"
else
	echo "跳过 x64(output/mac/noname.app 不存在)"
fi

echo ""
echo "=== 完成 ==="
echo "现在可以安装 dmg 测试。注意:app 是 ad-hoc 签名,首次打开需在"
echo "系统设置 → 隐私与安全性 里点「仍要打开」"
