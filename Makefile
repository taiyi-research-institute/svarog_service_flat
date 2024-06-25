# Makefile of the whole "Svarog" project
.PHONY: all clean

all: proto build

# Generate rust from proto files
grpc: proto
proto:
	@echo > svarog_grpc/src/lib.rs
	cargo run --bin protoc_rust -- \
		-p $(shell pwd) \
		-r $(shell pwd)/svarog_grpc/src

build: kill_tmux
	cargo fmt
	cargo build --release
	mkdir -p out
	cp target/release/svarog_sesman            out/svarog_sesman
	cp target/release/svarog_peer              out/svarog_peer
	cp target/release/test_keygen_sign         out/test_keygen_sign
	cp target/release/test_mkeygen_sign        out/test_mkeygen_sign
	cp target/release/test_reshare             out/test_reshare

clean:
	cargo clean

kill_tmux:
	@tmux kill-session -t svarog || true

test_keygen_sign: build
	@tmux new-session -s svarog \
		-n man -d ";" new-window \
		-n peer -d ";" new-window \
		-n test -d ";"
	@sleep 1
	@tmux send-keys -t svarog:man  "cd $(shell pwd)/out && ./svarog_sesman" C-m
	@tmux send-keys -t svarog:peer "cd $(shell pwd)/out && ./svarog_peer" C-m
	@sleep 1
	@tmux send-keys -t svarog:test "cd $(shell pwd)/out && ./test_keygen_sign" C-m

test_mkeygen_sign: build
	@tmux new-session -s svarog \
		-n man -d ";" new-window \
		-n peer -d ";" new-window \
		-n test -d ";"
	@sleep 1
	@tmux send-keys -t svarog:man  "cd $(shell pwd)/out && ./svarog_sesman" C-m
	@tmux send-keys -t svarog:peer "cd $(shell pwd)/out && ./svarog_peer" C-m
	@sleep 1
	@tmux send-keys -t svarog:test "cd $(shell pwd)/out && ./test_mkeygen_sign" C-m

test_reshare: build
	@tmux new-session -s svarog \
		-n man -d ";" new-window \
		-n peer -d ";" new-window \
		-n test -d ";"
	@sleep 1
	@tmux send-keys -t svarog:man  "cd $(shell pwd)/out && ./svarog_sesman" C-m
	@tmux send-keys -t svarog:peer "cd $(shell pwd)/out && ./svarog_peer" C-m
	@sleep 1
	@tmux send-keys -t svarog:test "cd $(shell pwd)/out && ./test_reshare" C-m