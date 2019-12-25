RUST_LOG?=INFO
export RUST_LOG

build:
	cargo build

run: build
	target/debug/day$(DAY) < input/day$(DAY).txt

%: input/%.txt src/bin/%.rs
	@cargo build -q --bin $*
	@target/debug/$* < input/$*.txt

day5:
	cargo build -q --bin $@
	target/debug/$@ input/$@.txt


.PRECIOUS: input/%.txt

input/%.txt:
	http -o input/$*.txt https://adventofcode.com/2019/day/$(subst day,,$*)/input Cookie:session=$(ADVENT_COOKIE_SESSION)

test:
	echo $(RUST_LOG)
	cargo test

lint:
	pre-commit run --all

fmt:
	find . -name '*.rs' -exec rustfmt {} \;
