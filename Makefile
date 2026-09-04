_default:
	@echo "make build|install|test"

build:
	cargo build --release

install: build
	sudo cp target/release/tile_traffic /usr/local/bin/tile-traffic

depgraph:
	cargo depgraph --all-deps --dedup-transitive-deps | dot -Tpng > graph.png
	xdg-open graph.png

test: build
	./target/release/tile_traffic \
		--lon -104.99 \
		--lat 39.74 \
		--zoom 9 \
		--bursts 40 \
		--header x-cache \
	  --zxy https://tiles.perrygeo.com/latest/{z}/{x}/{y}
