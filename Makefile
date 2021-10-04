
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
	dot graph.dot -Tsvg -o graph.svg

watchdot:
	nodemon --exec "make dot"