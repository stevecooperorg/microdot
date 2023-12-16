DEFAULT_DOT=$(HOME)/microdot_graph.dot
DEFAULT_JSON=$(HOME)/microdot_graph.dot
DEFAULT_SVG=$(HOME)/microdot_graph.svg

.PHONY: all
all: build

.PHONY: setup
setup:
	cargo install --git https://github.com/stevecooperorg/cargo-list-files

build: target/debug/microdot

fmt:
	cargo +nightly fmt

run: target/debug/microdot
	target/debug/microdot

.PHONY:
target/debug/microdot:
	cargo --version
	cargo build

check:
	cargo clippy -- -Dwarnings

watch:
	cargo watch -x "test" -x "clippy -- -Dwarnings"

build: target/debug/microdot

test:
	cargo test

dot:
	#dot graph.dot -Tpng -o graph.png
	dot "$(DEFAULT_DOT)" -Tsvg -o "$(DEFAULT_SVG)"

commit:
	./bin/auto-commit

docker-build:
	docker build . --tag microdot:latest

FILE=/examples/story.json

docker-run: docker-build
	mkdir -p "$$HOME/microdot"
	docker run --rm \
		--mount type=bind,source="$$HOME/microdot",target=/microdot \
		-it microdot:latest microdot \
		--file "/microdot/${FILE}"

safe-commit: fmt check test commit