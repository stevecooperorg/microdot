.PHONY: all
all:
	echo "nothing yet"

watch:
	cd src; cargo watch -x run

run:
	cd src; cargo run

build:
	cd src; cargo build

test:
	cd src; cargo watch -x test