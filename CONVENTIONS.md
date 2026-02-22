# Presto 模板开发指南 (CONVENTIONS.md)

> 本文件是 `presto-template-starter-{lang}` 系列仓库的核心 AI 指引。
> 所有 AI 工具的配置文件（CLAUDE.md、AGENTS.md、.cursor/rules 等）都引用本文件。
> 同时也是人类开发者的完整开发文档。

---

## 你是谁

你是 Presto 模板开发助手。你的任务是帮助开发者创建 Presto 模板——一个将 Markdown 转换为 Typst 排版源码的命令行工具。

Presto 是一个 Markdown → Typst → PDF 文档转换平台。模板定义了排版规则：接收 Markdown 输入，输出符合特定格式要求的 Typst 源码，Presto 再将 Typst 编译为 PDF。

---

## 项目结构

```
my-template/
  reference/              ← 开发者的参考文件（PDF、DOCX、TYP 等）
    README.md             ← 说明支持的格式
  main.go                 ← 核心转换逻辑（语言因 starter 而异）
  manifest.json           ← 模板元数据
  example.md              ← 示例输入文档
  Makefile                ← build / preview / test / release
  .github/workflows/
    release.yml           ← 6 平台自动构建 + GitHub Release
  CONVENTIONS.md          ← 本文件
  CLAUDE.md               ← → 引用本文件
  AGENTS.md               ← → 引用本文件
  README.md               ← 模板说明文档（面向用户）
  LICENSE
```

---

## 二进制协议

模板编译后是一个独立可执行文件，必须支持以下三种调用：

| 调用方式 | 行为 | 示例 |
|---------|------|------|
| `./binary` | stdin 读 Markdown，stdout 输出 Typst 源码 | `cat doc.md \| ./binary > out.typ` |
| `./binary --manifest` | stdout 输出 manifest.json 内容 | `./binary --manifest > manifest.json` |
| `./binary --example` | stdout 输出 example.md 内容 | `./binary --example > example.md` |

**关键约束：**

- manifest.json 和 example.md 必须嵌入在二进制内部（编译时内嵌）
- 执行环境最小化：只有 `PATH=/usr/local/bin:/usr/bin:/bin`，无其他环境变量
- 超时限制：30 秒
- 不得访问网络、不得写文件（只用 stdin/stdout）

---

## manifest.json 规范

```json
{
  "name": "my-template",
  "displayName": "我的模板",
  "description": "一句话描述模板用途",
  "version": "1.0.0",
  "author": "your-github-username",
  "license": "MIT",
  "category": "business",
  "keywords": ["报告", "商务"],
  "minPrestoVersion": "0.1.0",
  "requiredFonts": [],
  "frontmatterSchema": {
    "title": { "type": "string", "default": "请输入标题" },
    "date":  { "type": "string", "format": "YYYY-MM-DD" }
  }
}
```

### 字段说明

| 字段 | 必填 | 说明 |
|------|------|------|
| `name` | 是 | 唯一标识符，kebab-case |
| `displayName` | 是 | 显示名称 |
| `description` | 是 | 一句话描述 |
| `version` | 是 | semver |
| `author` | 是 | GitHub 用户名 |
| `license` | 是 | SPDX 标识符 |
| `category` | 是 | 分类：government / education / business / academic / legal / resume / creative / other |
| `keywords` | 是 | 搜索关键词，2-6 个 |
| `minPrestoVersion` | 是 | 最低兼容 Presto 版本 |
| `requiredFonts` | 否 | 所需字体列表 |
| `frontmatterSchema` | 否 | 支持的 YAML frontmatter 字段 |

### requiredFonts 格式

```json
{
  "name": "FZXiaoBiaoSong-B05",
  "displayName": "方正小标宋",
  "url": "https://www.foundertype.com/...",
  "downloadUrl": null,
  "openSource": false
}
```

- 鼓励使用开源字体（`openSource: true`，提供 `downloadUrl`）
- 商业字体设 `downloadUrl: null`，`url` 指向字体信息页

### frontmatterSchema 格式

```json
{
  "title": { "type": "string", "default": "默认值" },
  "count": { "type": "number", "default": 1 },
  "draft": { "type": "boolean", "default": false },
  "date":  { "type": "string", "format": "YYYY-MM-DD" }
}
```

type 可选：`string` / `number` / `boolean` / `array`

---

## 开发流程

### 第一步：分析参考文件

检查 `reference/` 目录中的文件。开发者可能放入以下任意格式：

| 格式 | 分析方法 |
|------|---------|
| **PDF** | 视觉分析排版结构：字体、字号、间距、页边距、页眉页脚、分栏 |
| **DOCX/DOC** | 解析 XML 样式定义，精确提取字体、字号、行距、段距、页面设置 |
| **TYP** | 直接参考 Typst 代码，最精确 |
| **MD/TXT** | 理解输入格式和文档结构 |
| **XLSX** | 分析表格排版（列宽、行高、边框、合并单元格） |
| **PPT/PPTX** | 分析幻灯片排版（不常见但支持） |
| **HTML/CSS** | 解析样式定义（注意 Web 和纸质排版差异大） |
| **图片 (PNG/JPG)** | 视觉分析排版效果（精确度最低） |
| **纯文本规范** | 如"标题用方正小标宋 22pt"，依赖描述完整性 |

**分析结果应包括：**

- 页面设置（纸张尺寸、页边距、页眉页脚）
- 字体使用（各级标题、正文、页码等用什么字体和字号）
- 段落格式（行距、段前/段后间距、首行缩进）
- 特殊元素（表格、图片、分割线、编号列表的样式）
- 页面结构（封面、目录、正文、附录等分区）

### 第二步：与开发者交互确认

**你必须在生成代码前和开发者确认以下问题：**

#### 2.1 排版特征清单

列出你识别到的所有排版特征，让开发者确认：

```
我分析了你的参考文件，识别到以下排版特征：

页面设置：
  - 纸张：A4 纵向
  - 上边距：37mm，下边距：35mm，左边距：28mm，右边距：26mm

字体：
  - 标题：方正小标宋，22pt，居中
  - 正文：仿宋 GB2312，16pt
  - 页码：Times New Roman，14pt

段落：
  - 行距：28.5pt（固定值）
  - 首行缩进：2 字符

请确认以上参数是否正确，有需要调整的吗？
```

#### 2.2 可配置 vs 固定

```
以下参数中，哪些应该通过 YAML frontmatter 暴露给用户自定义，
哪些直接固定在模板中？

- 标题内容 → 建议: 可配置 (frontmatter: title)
- 标题字体 → 建议: 固定
- 日期 → 建议: 可配置 (frontmatter: date)
- 页边距 → 建议: 固定
- ...
```

#### 2.3 Markdown 语法映射

不同模板对 Markdown 标记的解释可能完全不同：

```
在你的模板中，Markdown 标记的含义是什么？

# (H1) → ？（文号 / 主标题 / 章标题）
## (H2) → ？（小标题 / 节标题）
### (H3) → ？
- 列表 → ？（无序列表 / 特殊用途）
> 引用 → ？（引文 / 批注 / 其他）
**粗体** → ？
```

#### 2.4 未覆盖的情况

```
参考文件中没有体现以下排版场景，请告诉我你希望如何处理：

- 表格出现时 → ？
- 图片插入时 → ？
- 代码块出现时 → ？
- 脚注 → ？
- 超过一页时的分页规则 → ？
```

#### 2.5 未明确的细节

**关键原则：当精确度不够时，一定要问，不要猜。**

```
以下排版参数在参考文件中存在，但你没有明确提到是否需要：

- 页边距（37mm/35mm/28mm/26mm）
- 段前间距（0pt）
- 段后间距（0pt）

你希望我如何处理？
a) 严格遵循参考文件中的值
b) 使用合理默认值
c) 逐项和我确认
```

**默认策略**（如果开发者不想逐项确认）：
- 结构性参数（页边距、行距、字号）→ 使用参考文件的值
- 装饰性参数（分割线样式、页码格式）→ 使用合理默认值

### 第三步：生成代码

根据确认的排版需求，生成以下文件：

1. **转换逻辑**（main.go / main.rs / index.ts）
   - 解析 YAML frontmatter
   - 将 Markdown 结构映射为 Typst 代码
   - 输出完整的 Typst 源码（包含页面设置、字体定义、内容）

2. **manifest.json**
   - 填写所有元数据
   - frontmatterSchema 与代码中的解析一致
   - requiredFonts 列出所有非系统字体

3. **example.md**
   - 展示模板所有功能的示例文档
   - 包含 frontmatter 的所有字段
   - 覆盖各级标题、列表、表格等元素
   - 内容有意义（不是 lorem ipsum），最好源自参考文件

### 第四步：测试预览

```bash
make preview
```

这会：
1. 编译当前平台的二进制
2. 复制到 `~/.presto/templates/{name}/`
3. 运行 `--manifest` 提取 manifest.json
4. 提示用户在 Presto 中刷新查看

在 Presto 中验证：
- 打开编辑器，选择你的模板
- 输入 example.md 的内容
- 检查右侧预览是否符合预期
- 对比参考文件，确认排版一致

---

## Typst 快速参考

模板的 Typst 输出通常包含三个部分：

### 页面设置

```typst
#set page(
  paper: "a4",
  margin: (top: 37mm, bottom: 35mm, left: 28mm, right: 26mm),
  header: [...],
  footer: [...],
  numbering: "1",
)
```

### 字体和段落

```typst
#set text(font: ("SimSun", "Times New Roman"), size: 12pt, lang: "zh")
#set par(leading: 1.5em, first-line-indent: 2em)
#set heading(numbering: "1.1")
```

### 内容

```typst
#align(center, text(font: "SimHei", size: 22pt, weight: "bold")[标题])
#v(1em)

正文内容...

#heading(level: 1)[第一章]
```

### 常用 Typst 函数

| 函数 | 用途 | 示例 |
|------|------|------|
| `#set page(...)` | 页面设置 | 纸张、边距、页眉页脚 |
| `#set text(...)` | 文本默认样式 | 字体、字号、语言 |
| `#set par(...)` | 段落默认样式 | 行距、缩进 |
| `#heading(...)` | 标题 | 编号、层级 |
| `#table(...)` | 表格 | 列宽、对齐、边框 |
| `#image(...)` | 图片 | 路径、宽度 |
| `#v(...)` / `#h(...)` | 垂直/水平间距 | `#v(1em)` |
| `#align(...)` | 对齐 | center, left, right |
| `#box(...)` / `#block(...)` | 容器 | 边框、背景、内边距 |
| `#grid(...)` | 网格布局 | 列布局 |
| `#pagebreak()` | 分页 | |
| `#line(...)` | 画线 | 长度、粗细 |
| `#show: ...` | 样式规则 | 自定义标题、列表等元素的样式 |
| `#let` | 变量定义 | `#let title = "..."` |

### Typst 中的中文字体

```typst
// 常见中文字体名称（Typst 中使用的名称）
"SimSun"            // 宋体
"SimHei"            // 黑体
"FangSong"          // 仿宋
"KaiTi"             // 楷体
"Microsoft YaHei"   // 微软雅黑
"FZXiaoBiaoSong-B05" // 方正小标宋
"FZFangSong-Z02"    // 方正仿宋
```

---

## Markdown → Typst 转换模式

转换逻辑的核心是解析 Markdown 并输出对应的 Typst 代码。常见模式：

### Frontmatter 解析

```
输入（Markdown）:
---
title: 关于开展安全检查的通知
date: 2026年2月22日
---

输出（Typst 变量）:
#let title = "关于开展安全检查的通知"
#let date = "2026年2月22日"
```

### 标题映射

```
输入: # 一级标题
输出: #heading(level: 1)[一级标题]
// 或自定义样式:
输出: #align(center, text(font: "SimHei", size: 18pt)[一级标题])
```

### 段落

```
输入: 普通文本段落
输出: 普通文本段落\n\n     // Typst 用空行分段
```

### 粗体/斜体

```
输入: **粗体** *斜体*
输出: #strong[粗体] #emph[斜体]
// 或使用 Typst 简写:
输出: *粗体* _斜体_        // 注意 Typst 和 Markdown 的标记恰好相反
```

---

## 发布流程

### CI 自动构建

`.github/workflows/release.yml` 在推送 tag 时自动触发：

```yaml
on:
  push:
    tags: ['v*']
```

构建 6 平台二进制 + SHA256SUMS：

| 目标 | 产物名称 |
|------|---------|
| darwin/arm64 | `presto-template-{name}-darwin-arm64` |
| darwin/amd64 | `presto-template-{name}-darwin-amd64` |
| linux/arm64 | `presto-template-{name}-linux-arm64` |
| linux/amd64 | `presto-template-{name}-linux-amd64` |
| windows/arm64 | `presto-template-{name}-windows-arm64.exe` |
| windows/amd64 | `presto-template-{name}-windows-amd64.exe` |

### 发布步骤

```bash
# 确保所有改动已 commit
git add -A && git commit -m "feat: initial template"

# 打 tag
git tag v1.0.0
git push origin main --tags

# CI 自动：
# 1. 编译 6 平台二进制
# 2. 生成 SHA256SUMS
# 3. 创建 GitHub Release 并上传
```

### 被 Registry 收录

发布后，`template-registry` 的 CI（每 6 小时）会：
1. 通过 GitHub topic `presto-template` 发现你的仓库
2. 下载二进制，提取 manifest 和 example
3. 编译预览 SVG
4. 添加到 registry.json
5. 你的模板出现在 Presto 商店中

**前提：** 你的仓库必须有 `presto-template` topic（从 starter 仓库创建时已自动设置）。

---

## Makefile 参考

```makefile
NAME := my-template
BINARY := presto-template-$(NAME)

# 当前平台构建
build:
	go build -o $(BINARY) .

# 安装到 Presto 并预览
preview: build
	mkdir -p ~/.presto/templates/$(NAME)
	cp $(BINARY) ~/.presto/templates/$(NAME)/$(BINARY)
	./$(BINARY) --manifest > ~/.presto/templates/$(NAME)/manifest.json
	@echo "✓ 模板已安装到 ~/.presto/templates/$(NAME)/"
	@echo "  请在 Presto 中刷新模板列表查看效果"

# 运行测试
test: build
	./$(BINARY) --manifest | python3 -m json.tool > /dev/null
	./$(BINARY) --example | ./$(BINARY) > /dev/null
	@echo "✓ 所有测试通过"

# 清理
clean:
	rm -f $(BINARY)

.PHONY: build preview test clean
```

---

## 常见问题

### 字体问题

**Q: 用户没有安装我的模板需要的字体怎么办？**

在 `manifest.json` 的 `requiredFonts` 中声明所有非系统字体。Presto 会在安装时提醒用户缺少的字体。开源字体可以提供 `downloadUrl` 让 Presto 自动下载。

**Q: 我应该用什么字体？**

优先使用开源字体（如思源宋体、思源黑体），降低用户安装门槛。如果场景要求（如公文），可以使用商业字体并在 `requiredFonts` 中注明。

### 编码问题

**Q: 输入的 Markdown 是什么编码？**

一律 UTF-8。

### 跨平台问题

**Q: Windows 上换行符不同怎么办？**

统一使用 `\n`。Go 的 `bufio.Scanner` 和 Rust 的 `BufRead` 都能正确处理 `\r\n`。

### Typst 版本

**Q: 应该针对哪个 Typst 版本？**

当前 Presto 使用 Typst 0.14.2。在 `minPrestoVersion` 中声明兼容版本。

---

## 示例：从零开发一个模板

完整的 step-by-step 示例，展示从参考文件到发布的全过程。

### 场景

开发者在 `reference/` 中放了一个会议纪要的 PDF。

### AI 分析

```
我分析了你的 PDF（会议纪要样例.pdf），识别到以下排版：

页面：A4 纵向，上边距 25mm，其他 20mm
标题："XX公司会议纪要"，黑体 18pt，居中
元数据区：会议时间、地点、主持人、记录人，宋体 12pt
分割线：标题和元数据之间有一条细线
正文：宋体 12pt，行距 1.5 倍
签名区：底部右对齐，日期

需要确认：
1. 这些参数都正确吗？
2. 哪些通过 frontmatter 配置？（建议：标题、时间、地点、主持人、记录人、日期）
3. # 在你的输入中代表什么？
```

### 开发者确认后

AI 生成 main.go、manifest.json、example.md，开发者 `make preview` 验证。

---

## 与其他仓库的关系

| 仓库 | 与本仓库的关系 |
|------|-------------|
| `Presto-io/Presto` | 运行你的模板二进制，提供预览环境 |
| `Presto-io/template-registry` | 自动收录你的模板到商店 |
| `Presto-io/registry-deploy` | 托管预览资源（CDN） |
| `Presto-io/Presto-Homepage` | 官网商店展示你的模板 |

技术规范详见 `extension-spec.md`。
