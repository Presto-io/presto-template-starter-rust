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
	@echo "Testing manifest..."
	@./$(BINARY) --manifest | python3 -m json.tool > /dev/null
	@echo "Testing example round-trip..."
	@./$(BINARY) --example | ./$(BINARY) > /dev/null
	@echo "Testing version..."
	@./$(BINARY) --version > /dev/null
	@# 校验 category 字段
	@./$(BINARY) --manifest | python3 -c "\
import json, sys, re; \
m = json.load(sys.stdin); \
cat = m.get('category', ''); \
(not cat) and (print('ERROR: category is empty', file=sys.stderr) or sys.exit(1)); \
(len(cat) > 20) and (print(f'ERROR: category too long ({len(cat)} > 20)', file=sys.stderr) or sys.exit(1)); \
(not re.match(r'^[\u4e00-\u9fff\w\s-]+$$', cat)) and (print(f'ERROR: category contains invalid characters: {cat}', file=sys.stderr) or sys.exit(1)); \
print(f'  category: {cat} ✓')"
	@echo "All tests passed."

# 清理
clean:
	cargo clean
	rm -f $(BINARY)

.PHONY: build preview test clean
