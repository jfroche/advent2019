RUST_LOG?=INFO
export RUST_LOG

help:  ## Display this help
	@awk 'BEGIN {FS = ":.*##"; printf "\nUsage:\n  make \033[36m<target>\033[0m\n\nTargets:\n"} /^[a-zA-Z_-]+:.*?##/ { printf "  \033[36m%-10s\033[0m %s\n", $$1, $$2 }' $(MAKEFILE_LIST)

build:  ## Build all the days
	cargo build

run: build  ## Run a specific day using variable: make run DAY=3
	target/debug/day$(DAY) < input/day$(DAY).txt

%: input/%.txt src/bin/%.rs  ## Run a specific day by name: make day3
	@cargo build -q --bin $*
	@target/debug/$* < input/$*.txt

day5 day6 day7 day10:
	cargo build -q --bin $@
	target/debug/$@ input/$@.txt

day8:
	cargo build -q --bin $@
	target/debug/$@ --width 25 --height 6 input/$@.txt

.PRECIOUS: input/%.txt

input/%.txt:
	http -o input/$*.txt https://adventofcode.com/2019/day/$(subst day,,$*)/input Cookie:session=$(ADVENT_COOKIE_SESSION)

test:  ## Run all the tests
	cargo test

lint:  ## Lint all the code
	pre-commit run --all

fmt:  ## Format all rust code
	find . -name '*.rs' -exec rustfmt --edition 2018 {} \;
