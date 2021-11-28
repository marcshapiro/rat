help:
	@echo Targets include: build run test wc bak cov

build:
	@cargo build

rbuild:
	@cargo build --release

# run default function
run:
	RUST_BACKTRACE=1 cargo run -- -time -dec40 -std "root(2, 2, 1e-30)"

# run with version
runv:
	RUST_BACKTRACE=1 cargo run -- -v

# run with no params
runx:
	RUST_BACKTRACE=1 cargo run

rrun:
	RUST_BACKTRACE=1 cargo run --release -- -time -dec40 -std "root(2, 2, 1e-30)"

wc:
	@./wc.py

test:
	@RUST_BACKTRACE=1 cargo test -- --quiet

# verbose test
testv:
	@RUST_BACKTRACE=1 cargo test

# rat's automated testing framework
rtest: rbuild
	@./rtest.py -pfx "u auto" -file-first -binary ./target/release/rat `find auto -name *.ratu`
	@./rtest.py -pfx "u sys" -file-first -binary "./target/release/rat -sys" `find sys -name *.ratu`
	@./rtest.py -pfx "u std" -file-first -binary "./target/release/rat -std" `find std -name *.ratu`
	@./rtest.py -pfx "u usr" -file-first -binary "./target/release/rat -usr" `find usr -name *.ratu`
	@./rtest.py -pfx "u my" -file-first -binary "./target/release/rat -my" `find my -name *.ratu`
	@./rtest.py -pfx "i auto" -binary ./target/release/rat `find auto -name *.rati`
	@./rtest.py -pfx "i sys" -binary "./target/release/rat -sys" `find sys -name *.rati`
	@./rtest.py -pfx "i std" -binary "./target/release/rat -std" `find std -name *.rati`
	@./rtest.py -pfx "i usr" -binary "./target/release/rat -usr" `find usr -name *.rati`
	@./rtest.py -pfx "i my" -binary "./target/release/rat -my" `find my -name *.rati`

# coverage
cov:
	cargo tarpaulin -v -o Html --output-dir target/tarpaulin/ 1> /dev/null 2> /dev/null
	chromium target/tarpaulin/tarpaulin-report.html 2> /dev/null

doc:
	cargo doc
	rustdoc -o ./target/doc/rat README.md
	@chromium ./target/doc/rat/index.html 2> /dev/null


git1:
	git status

git2:
	git add -n .

git3:
	git add .

git4:
	git status

git5:
	git commit


update:
	cargo update
	cargo outdated

### docs

cli:
	@chromium https://rust-cli.github.io/book/index.html 2> /dev/null

lint1:
	grep -r panic src/

lint2:
	grep -r " -> Option" src/

lint3:
	cargo clippy

lint3b:
	cargo clippy 2> clippy.err
	less clippy.err

todo:
	grep -riGHn "todo\|fixme" src/
