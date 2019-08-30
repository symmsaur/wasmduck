.PHONY: build-docker build
build-docker:
	docker build -f docker/Dockerfile docker/ -t wasm
serve:
	docker run --rm -it -v $(PWD):$(PWD) -w $(PWD) --network=host wasm cargo wasm
rundocker:
	docker run --rm -it -v $(PWD):$(PWD) -w $(PWD) --network=host wasm /bin/bash
