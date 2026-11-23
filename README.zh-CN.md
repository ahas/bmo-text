[English](README.md) | [한국어](README.ko.md) | [日本語](README.ja.md) | **简体中文**

# bmo-text

一个基础库,按照聊天房间、控制台和归档读取的方式来读写 Markdown 与文本:把 Markdown 转成 Telegram 的 HTML 和纯文本、
检查一段标记的结构、折叠与拆分成多条消息、正文中的网址、破折号改写、按禁用词表检查文本、回答长度上限、Markdown 文档的
结构、搜索结果的结构行、交给翻译器的文本片段,以及把 HTML 页面读成行。这些读取逻辑只在 Rust crate 中写一次;TypeScript
包就是把该 crate 构建成 WebAssembly 的产物,在 Bun、Node 和浏览器中都会在 import 时同步加载。除非名称中带有 `chars`,
长度一律按 UTF-16 单位计算,空白和 trim 的规则与 JavaScript 一致,因此同一输入在两边得到同一结果。crate 的二进制以
`bmo-text mcp` 运行时就是 MCP 服务器,装有 `bmo-text` 插件的编码工具通过它调用这些纯读取功能。

| 目录 | 内容 |
|---|---|
| `docs/` | 设计、公开 API、边界、库的来源清单、基础库拆分计划,以及个人数据规则 |
| `src/` | Rust crate `bmo-text` 的源码,其清单位于根目录 |
| `tests/` | crate 的测试,以及维护仓库自身结构的仓库测试 |
| `package/` | TypeScript 包 `bmo-text`:基于 crate 的 WebAssembly 构建的各模块、位于 `wasm/` 的构建产物及其测试 |
| `cases/` | crate 与包都必须满足的用例,每个模块一个文件 |
| `plugin/` | 编码工具插件 `bmo-text`:清单、MCP 服务器配置和 `bmo-text` 技能 |

本库只依赖注册表中的包,没有 HTTP 服务器,也不需要部署。使用方通过路径或 git 引入 crate,把包作为依赖引入,并传入由它
自己决定的内容:读者看到的措辞、英文策略以及词表的路径。

个人数据不进入代码、测试、文档和提交;`docs/PERSONAL-DATA.md` 写明了这条规则,以及让本仓库遵守它的检查。

本库以 MIT 许可证发布,许可证全文见 `LICENSE`。

检查和测试在 `testbed.toml` 中声明,具体命令写在 `CLAUDE.md` 中。
