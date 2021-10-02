
.PHONY: all
all:
	echo "nothing yet"

watch:
	cd src; cargo watch -x run 2> /dev/null

fmt:
	cd src; cargo +nightly fmt

run: src/target/debug/microdot
	src/target/debug/microdot

.PHONY: src/target/debug/microdot
src/target/debug/microdot:
	cd src; cargo build

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