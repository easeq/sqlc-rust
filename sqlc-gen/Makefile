build: build.rs src/codegen.proto src/main.rs
	cargo build --release --target wasm32-wasip1

EXAMPLES = "authors" "batch" "booktest" "jets" "ondeck" 

prepare-list:
	xargs -I_EX printf "%s\n" "_EX" | xargs -n1 echo 

get-examples:
	echo $(EXAMPLES) | make -s prepare-list

generate-for-example:
	@echo "generating for $$example" && \
	cd examples/$$example && \
	file_with_ext=$$(ls sqlc.*) && \
	file_ext=$${file_with_ext##*.} && \
	ls sqlc.* | yq -iP ".plugins[0].wasm.sha256=\"$$sha_256\", .plugins[0].wasm.url=\"file://./../../../target/wasm32-wasip1/release/sqlc-gen.wasm\"" $$file_with_ext -o $$file_ext  && \
	RUST_LOG=debug sqlc generate && \
	cd -

generate:
	sha=$$(make -s build && sha256sum ../target/wasm32-wasip1/release/sqlc-gen.wasm | cut -d' ' -f1) && \
	echo "SHA256: $$sha" && \
	make -s get-examples | xargs -I_EX sh -c "make -s generate-for-example example=_EX sha_256=$$sha"

install-deps: 
	rustup target add wasm32-wasip1
	go install github.com/mikefarah/yq/v4@latest
