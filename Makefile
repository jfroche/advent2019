build:
	cargo build

run: build
	target/debug/day$(DAY) < input/day$(DAY).txt

test:
	cargo test

lint:
	pre-commit run --all

fmt:
	find . -name '*.rs' -exec rustfmt {} \;

fetch-input:
	http -d -o input/day$(DAY).txt https://adventofcode.com/2019/day/$(DAY)/input Cookie:session=***REMOVED***
