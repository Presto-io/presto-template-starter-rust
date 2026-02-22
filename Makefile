NAME := my-template
BINARY := presto-template-$(NAME)

# 当前平台构建
build:
	cargo build --release
	cp target/release/presto-template-starter $(BINARY)

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
	cargo clean
	rm -f $(BINARY)

.PHONY: build preview test clean
