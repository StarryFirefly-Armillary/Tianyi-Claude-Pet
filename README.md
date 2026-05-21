# Tianyi Claude Pet

洛天依 Claude Code 桌宠 —— 基于 [aemeath_withclaude](https://github.com/77wilNd/aemeath_withclaude) 修改。

## 预览

桌面上的像素洛天依会随 Claude Code 的工作状态做出不同动作和气泡提示。

## 功能

- **透明无边框窗口**，始终置顶，可拖拽
- **13 种 CSS 动效**：待机（浮动/摇摆/扭头）、思考、工作中、读写文件、执行命令、搜索、分析、庆祝、失败、挥手
- **气泡提示**：随 Claude Code 状态变化显示对应文字
- **缩放**：托盘菜单支持 1/3x / 0.5x / 0.75x / 1x 四档
- **权限常驻气泡**：等待用户指示时气泡不会自动消失
- **HTTP + MCP 双协议**：与 Claude Code 通过 hooks 和 MCP 通信

## 技术栈

- **前端**：原生 HTML/CSS/JS（无框架），CSS `@keyframes` 动画 + `zoom` 像素缩放
- **后端**：Rust (Tauri 2) + axum (HTTP :9527) + JSON-RPC (MCP :9528)
- **窗口**：WebView2，透明无边框

## 快速开始

### 构建

```bash
# 安装依赖
npm install

# 编译
cargo build --release --manifest-path src-tauri/Cargo.toml
```

产物：`src-tauri/target/release/tianyi-pet.exe`

### 配置 Claude Code

将 `docs/hooks.json` 和 `docs/mcp.json` 的内容合并到 Claude Code 的对应配置中，即可实现状态联动。

## 项目结构

```
├── src/                    # 前端
│   ├── index.html
│   ├── pet.css             # @keyframes 动画 + 样式
│   ├── app.js              # 主逻辑
│   ├── bubble.js           # 气泡队列
│   └── luotianyi.png       # 洛天依像素图
├── src-tauri/              # Rust 后端
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs         # 入口
│       ├── http.rs         # HTTP API (hooks)
│       ├── mcp.rs          # MCP JSON-RPC
│       ├── state.rs        # 状态机
│       └── tray.rs         # 系统托盘
├── docs/
│   ├── hooks.json          # Claude Code hooks 配置
│   └── mcp.json            # MCP 服务配置
└── .claude/
    └── settings.json       # 项目级 Claude Code 设置
```

## 致谢

- 原版项目 [aemeath_withclaude](https://github.com/77wilNd/aemeath_withclaude) by [77wilNd](https://github.com/77wilNd)
- 角色形象：洛天依 (Vsinger)

## License

MIT
