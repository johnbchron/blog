
# run server and watch changes -- surreal must be running
watch:
	cargo leptos watch
# run server in release mode -- surreal must be running
serve:
	cargo leptos serve --release
# build and run server container -- surreal must be running
container:
	nix build "./#container"
	docker load -i result
	@ # if you're wondering why I added `--init`, it's because I built this
	@ # for fly.io which uses firecracker, so I left `tini` out of the image
	@ # also `--network host` to use surrealdb
	docker run --rm --init --network host site-server
# run server in release mode with chrome tracing -- surreal must be running
trace:
	cargo leptos serve --bin-features chrome-tracing
# run nix checks
check:
	nix flake check -L
# run surrealdb
surreal:
	surreal start file:/tmp/omthub_surreal_data --log=info --auth
# nuke surreal data in /tmp/surreal_data
wipe-surreal:
	rm -rf /tmp/omthub_surreal_data
# run surrealdb migrations -- surreal must be running
apply-surreal:
	surrealdb-migrations apply
