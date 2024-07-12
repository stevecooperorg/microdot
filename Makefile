DEFAULT_DOT=$(HOME)/microdot_graph.dot
DEFAULT_JSON=$(HOME)/microdot_graph.json
DEFAULT_SVG=$(HOME)/microdot_graph.svg

.PHONY: all
all: build

.PHONY: setup
setup:
	cargo install --git https://github.com/stevecooperorg/cargo-list-files

build: target/debug/microdot

clean:
	cargo clean

fmt:
	cargo +nightly fmt

run: target/debug/microdot
	target/debug/microdot --file "$(DEFAULT_JSON)" --port 7777

.PHONY:
target/debug/microdot:
	cargo --version
	cargo build

check:
	cargo clippy -- -Dwarnings

watch:
	cargo watch --ignore "examples/*.log" --ignore "examples/*.svg" --ignore "examples/*.dot" --ignore "examples/*.json"  --why -x "test" -x "clippy -- -Dwarnings" -x "build"

.PHONY: increment-docker-semver-tag
increment-docker-semver-tag:
	cd manage_semver && cargo run -- --semver-file-path ../CURRENT_DOCKER_SEMVER_TAG > ../CURRENT_DOCKER_SEMVER_TAG.bak && mv ../CURRENT_DOCKER_SEMVER_TAG.bak ../CURRENT_DOCKER_SEMVER_TAG

build: target/debug/microdot

test:
	cargo test

dot:
	dot "$(DEFAULT_DOT)" -Tsvg -o "$(DEFAULT_SVG)"

commit:
	./bin/auto-commit

docker-build:
	docker build . --tag stevecooperorg/microdot:latest --tag stevecooperorg/microdot:$(shell cat CURRENT_DOCKER_SEMVER_TAG)

docker-push: increment-docker-semver-tag docker-build
	docker push stevecooperorg/microdot:latest
	docker push stevecooperorg/microdot:$(shell cat CURRENT_DOCKER_SEMVER_TAG)

FILE=story.json
docker-run: docker-build
	mkdir -p "$$HOME/microdot"
	docker run --rm \
		-p 7777:7777 \
		--mount type=bind,source="$$HOME/microdot",target=/microdot \
		--mount type=bind,source="$$HOME/.microdot_history",target=/root/.microdot_history \
		-it stevecooperorg/microdot:latest microdot \
		--file "/microdot/${FILE}" \
		--port 7777

safe-commit: fmt check test commit