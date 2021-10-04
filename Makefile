DEFAULT_DOT=$(HOME)/microdot_graph.dot
DEFAULT_JSON=$(HOME)/microdot_graph.dot
DEFAULT_SVG=$(HOME)/microdot_graph.svg

.PHONY: all
all:
	echo "nothing yet"

fmt:
	cd src; cargo +nightly fmt

run: src/target/debug/microdot
	src/target/debug/microdot

.PHONY: src/target/debug/microdot
src/target/debug/microdot:
	cd src; cargo build

watch:
	nodemon --exec "make run"

build:
	cd src
	cargo build

test:
	cd src
	cargo watch -x test

dot:
	#dot graph.dot -Tpng -o graph.png
	dot "$(DEFAULT_DOT)" -Tsvg -o "$(DEFAULT_SVG)"

watchdot:
	nodemon --exec "make dot" --watch "$(DEFAULT_DOT)"
