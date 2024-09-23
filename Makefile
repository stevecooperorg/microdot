DEFAULT_DOT=$(HOME)/microdot_graph.dot
DEFAULT_JSON=$(HOME)/microdot_graph.json
DEFAULT_SVG=$(HOME)/microdot_graph.svg

.PHONY: all
all: build test docker-build

.PHONY: setup
setup:
	cargo install --git https://github.com/stevecooperorg/cargo-list-files

build: target/debug/microdot

clean:
	cargo clean

fmt:
	cargo +nightly fmt

run: target/debug/microdot
	target/debug/microdot --file "$(DEFAULT_JSON)"

.PHONY: target/debug/microdot
target/debug/microdot:
	cargo --version
	cargo build

check:
	cargo clippy -- -Dwarnings

watch:
	cargo watch --ignore "examples/*.log" --ignore "examples/*.svg" --ignore "examples/*.dot" --ignore "examples/*.json"  --why -x "test" -x "clippy -- -Dwarnings" -x "build"

.PHONY: increment-docker-semver-tag
increment-docker-semver-tag:
	cd manage_semver && cargo run -- --semver-file-path ../.env > ../.env.bak && mv ../.env.bak ../.env

build: target/debug/microdot

test:
	cargo test

dot:
	dot "$(DEFAULT_DOT)" -Tsvg -o "$(DEFAULT_SVG)"

commit:
	./bin/auto-commit

docker-build: build
	export $$(cat .env | xargs) && docker buildx bake

docker-push: increment-docker-semver-tag docker-build
	export $$(cat .env | xargs) && docker push stevecooperorg/microdot:latest
	export $$(cat .env | xargs) && docker push stevecooperorg/microdot:$$CURRENT_DOCKER_SEMVER_TAG
	export $$(cat .env | xargs) && docker push stevecooperorg/live-server:latest
	export $$(cat .env | xargs) && docker push stevecooperorg/live-server:$$CURRENT_DOCKER_SEMVER_TAG

FILE=story.json
docker-run: docker-build
	mkdir -p "$$HOME/microdot"
	docker-compose up

docker-run-public: docker-build
	mkdir -p "$$HOME/microdot"
	docker-compose  --profile public up

docker-exec:
	docker exec -it microdot-microdot-1 bash

safe-commit: fmt check test commit