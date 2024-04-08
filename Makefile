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

build: kill_tmux proto
	cargo build --release
	mkdir -p out
	cp target/release/svarog_sesman out/svarog_sesman
	cp target/release/svarog_peer out/svarog_peer
	cp target/release/toy_client out/toy_client

clean:
	cargo clean

kill_tmux:
	@tmux kill-session -t svarog || true

demo_gg18_keygen: build
	@tmux new-session -s svarog \
		-n man -d ";" new-window \
		-n peer -d ";" new-window \
		-n ses -d ";" new-window \
		-n pF  -d ";" new-window \
		-n pCl -d ";" new-window \
		-n pBr -d ";" new-window \
		-n _pI -d ";" new-window \
		-n pHe -d ";" new-window \
		-n pNe -d ";" new-window \
		-n pAr -d ";"
	@sleep 1
	@tmux send-keys -t svarog:man  "cd $(shell pwd)/out && ./svarog_sesman" C-m
	@tmux send-keys -t svarog:peer "cd $(shell pwd)/out && ./svarog_peer" C-m
	@sleep 2
	@tmux send-keys -t svarog:ses "cd $(shell pwd)/out && ./toy_client -a gg18 -m keygen --new_session" C-m
	@sleep 1
	@tmux send-keys -t svarog:pF  "cd $(shell pwd)/out && ./toy_client -m keygen -n fluorine" C-m
	@tmux send-keys -t svarog:pCl "cd $(shell pwd)/out && ./toy_client -m keygen -n chlorine" C-m
	@tmux send-keys -t svarog:pBr "cd $(shell pwd)/out && ./toy_client -m keygen -n bromine" C-m
	@tmux send-keys -t svarog:_pI "cd $(shell pwd)/out && ./toy_client -m keygen -n iodine" C-m
	@tmux send-keys -t svarog:pHe "cd $(shell pwd)/out && ./toy_client -m keygen -n helium" C-m
	@tmux send-keys -t svarog:pNe "cd $(shell pwd)/out && ./toy_client -m keygen -n neon" C-m
	@tmux send-keys -t svarog:pAr "cd $(shell pwd)/out && ./toy_client -m keygen -n argon" C-m

demo_frost_keygen: build
	@tmux new-session -s svarog \
		-n man -d ";" new-window \
		-n peer -d ";" new-window \
		-n ses -d ";" new-window \
		-n pF  -d ";" new-window \
		-n pCl -d ";" new-window \
		-n pBr -d ";" new-window \
		-n _pI -d ";" new-window \
		-n pHe -d ";" new-window \
		-n pNe -d ";" new-window \
		-n pAr -d ";"
	@sleep 1
	@tmux send-keys -t svarog:man  "cd $(shell pwd)/out && ./svarog_sesman" C-m
	@tmux send-keys -t svarog:peer "cd $(shell pwd)/out && ./svarog_peer" C-m
	@sleep 2
	@tmux send-keys -t svarog:ses "cd $(shell pwd)/out && ./toy_client -a frost -m keygen --new_session" C-m
	@sleep 1
	@tmux send-keys -t svarog:pF  "cd $(shell pwd)/out && ./toy_client -m keygen -n fluorine" C-m
	@tmux send-keys -t svarog:pCl "cd $(shell pwd)/out && ./toy_client -m keygen -n chlorine" C-m
	@tmux send-keys -t svarog:pBr "cd $(shell pwd)/out && ./toy_client -m keygen -n bromine" C-m
	@tmux send-keys -t svarog:_pI "cd $(shell pwd)/out && ./toy_client -m keygen -n iodine" C-m
	@tmux send-keys -t svarog:pHe "cd $(shell pwd)/out && ./toy_client -m keygen -n helium" C-m
	@tmux send-keys -t svarog:pNe "cd $(shell pwd)/out && ./toy_client -m keygen -n neon" C-m
	@tmux send-keys -t svarog:pAr "cd $(shell pwd)/out && ./toy_client -m keygen -n argon" C-m

demo_gg18_sign: build
	@tmux new-session -s svarog \
		-n man -d ";" new-window \
		-n peer -d ";" new-window \
		-n ses -d ";" new-window \
		-n pF  -d ";" new-window \
		-n pCl -d ";" new-window \
		-n pBr -d ";" new-window \
		-n pHe -d ";" new-window \
		-n pNe -d ";" new-window \
		-n pAr -d ";"
	@sleep 1
	@tmux send-keys -t svarog:man  "cd $(shell pwd)/out && ./svarog_sesman" C-m
	@tmux send-keys -t svarog:peer "cd $(shell pwd)/out && ./svarog_peer" C-m
	@sleep 2
	@tmux send-keys -t svarog:ses "cd $(shell pwd)/out && ./toy_client -a gg18 -m sign --new_session" C-m
	@sleep 1
	@tmux send-keys -t svarog:pF  "cd $(shell pwd)/out && ./toy_client -m sign -n fluorine" C-m
	@tmux send-keys -t svarog:pCl "cd $(shell pwd)/out && ./toy_client -m sign -n chlorine" C-m
	@tmux send-keys -t svarog:pBr "cd $(shell pwd)/out && ./toy_client -m sign -n bromine" C-m
	@tmux send-keys -t svarog:pHe "cd $(shell pwd)/out && ./toy_client -m sign -n helium" C-m
	@tmux send-keys -t svarog:pNe "cd $(shell pwd)/out && ./toy_client -m sign -n neon" C-m
	@tmux send-keys -t svarog:pAr "cd $(shell pwd)/out && ./toy_client -m sign -n argon" C-m

demo_frost_sign: build
	@tmux new-session -s svarog \
		-n man -d ";" new-window \
		-n peer -d ";" new-window \
		-n ses -d ";" new-window \
		-n pF  -d ";" new-window \
		-n pCl -d ";" new-window \
		-n pBr -d ";" new-window \
		-n pHe -d ";" new-window \
		-n pNe -d ";" new-window \
		-n pAr -d ";"
	@sleep 1
	@tmux send-keys -t svarog:man  "cd $(shell pwd)/out && ./svarog_sesman" C-m
	@tmux send-keys -t svarog:peer "cd $(shell pwd)/out && ./svarog_peer" C-m
	@sleep 2
	@tmux send-keys -t svarog:ses "cd $(shell pwd)/out && ./toy_client -a frost -m sign --new_session" C-m
	@sleep 1
	@tmux send-keys -t svarog:pF  "cd $(shell pwd)/out && ./toy_client -m sign -n fluorine" C-m
	@tmux send-keys -t svarog:pCl "cd $(shell pwd)/out && ./toy_client -m sign -n chlorine" C-m
	@tmux send-keys -t svarog:pBr "cd $(shell pwd)/out && ./toy_client -m sign -n bromine" C-m
	@tmux send-keys -t svarog:pHe "cd $(shell pwd)/out && ./toy_client -m sign -n helium" C-m
	@tmux send-keys -t svarog:pNe "cd $(shell pwd)/out && ./toy_client -m sign -n neon" C-m
	@tmux send-keys -t svarog:pAr "cd $(shell pwd)/out && ./toy_client -m sign -n argon" C-m