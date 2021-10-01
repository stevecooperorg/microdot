
.PHONY: all
all:
	echo "nothing yet"

watch:
	cd src; cargo watch -x run 2> /dev/null

run: src/target/debug/microdot
	if [ ! -f md.out ]; then mkfifo md.out; fi
	src/target/debug/microdot 2> md.out

.PHONY: src/target/debug/microdot
src/target/debug/microdot:
	cd src; cargo build

build:
	cd src
	cargo build

test:
	cd src
	cargo watch -x test