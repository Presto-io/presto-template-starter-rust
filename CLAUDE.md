# Presto Rust Template Starter

> 组织级规则见 `../Presto-homepage/docs/ai-guide.md`
> 完整开发规范见 `../Presto-homepage/docs/CONVENTIONS.md`

GitHub Template Repository，开发者从此仓库创建新仓库后，用 AI 工具开发自己的模板。

## 核心概念

⚠️ Presto 模板是**独立二进制**，不是文件模板。

- 编译后的二进制通过 stdin/stdout 协议工作
- manifest.json 和 example.md **编译时嵌入**（`include_str!()`），不是运行时读取
- 二进制禁止网络访问、禁止文件写入、禁止额外 CLI flag
- 只允许四种调用：管道转换、`--manifest`、`--example`、`--version`

## 技术栈

- Rust + pulldown-cmark（Markdown AST）+ serde_yaml（frontmatter）
- 禁止使用网络相关 crate（reqwest、hyper 等）
- 禁止使用 `std::process::Command`
- 不要引入非必要的第三方依赖

## 开发命令

- `make build` — 编译二进制
- `make preview` — 安装到 Presto 并预览
- `make test` — 安全测试 + 功能测试

## 开发流程

1. 检查 `reference/` 目录中的参考文件
2. 分析排版特征，与用户确认
3. 实现 src/main.rs 转换逻辑
4. 编写 manifest.json 和 example.md
5. `make preview` 验证效果
