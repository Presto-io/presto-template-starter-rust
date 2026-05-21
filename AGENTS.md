# Presto Template Starter

请阅读并遵循 [Presto 模板开发规范](https://github.com/Presto-io/Presto-Homepage/blob/main/docs/CONVENTIONS.md)。

## 关键规则

- 模板是独立二进制，通过 stdin/stdout 协议工作
- 默认转换模式必须输出非空 Typst；转换失败或空白输出必须写 stderr 并非 0 退出
- manifest.json 和 example.md 编译时嵌入（`include_str!()`），不是运行时读取
- 禁止网络访问、文件写入、额外 CLI flag
- 只允许协议内调用：管道转换、`--manifest`、`--example`、`--version`、`--info`
- 所有对话和 commit 消息用中文
- 安全测试必须通过：`make test`
