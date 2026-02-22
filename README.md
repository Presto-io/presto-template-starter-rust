# Presto Rust Template Starter

GitHub Template Repository — 用于创建 Presto 模板的 Rust 语言起始项目。

Presto 模板是一个将 Markdown 转换为 Typst 排版源码的命令行工具。本仓库提供了最小骨架，你只需关注转换逻辑的开发。

## 快速开始

### 1. 放入参考文件

将你要模仿的排版参考文件放入 `reference/` 目录（PDF、DOCX、TYP、图片等）。

### 2. 用 AI 工具开发转换逻辑

打开项目，让 AI 工具分析参考文件并生成转换代码：

```
AI 会自动阅读 CONVENTIONS.md，了解开发流程和 Typst 语法。
```

### 3. 构建并预览

```bash
make preview
```

在 Presto 中刷新模板列表，查看排版效果。

## 常用命令

| 命令 | 说明 |
|------|------|
| `make build` | 编译当前平台二进制 |
| `make preview` | 编译并安装到 Presto 预览 |
| `make test` | 运行基础测试 |
| `make clean` | 清理编译产物 |

## 发布流程

确保所有改动已 commit，然后打 tag 推送：

```bash
git tag v1.0.0
git push origin main --tags
```

GitHub Actions 会自动：
1. 编译 6 平台二进制（darwin/linux/windows × amd64/arm64）
2. 生成 SHA256SUMS
3. 创建 GitHub Release 并上传所有产物

发布后，Presto template-registry 会自动发现并收录你的模板（需要仓库有 `presto-template` topic）。

## 项目结构

```
├── src/main.rs          # 核心转换逻辑
├── Cargo.toml           # Rust 包配置和依赖
├── manifest.json        # 模板元数据（嵌入二进制）
├── example.md           # 示例输入（嵌入二进制）
├── reference/           # 参考文件（不会被编译）
├── Makefile             # 构建脚本
├── CONVENTIONS.md       # 完整开发指南
└── .github/workflows/   # CI 自动发布
```

## 文档

详细的开发指南、Typst 语法参考、发布流程等，请参阅 [CONVENTIONS.md](CONVENTIONS.md)。
