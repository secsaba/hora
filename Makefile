# The image for this machine's architecture, built from the committed HEAD (uncommitted edits are left out).
# Production's x86_64 image comes from .github/workflows/deploy.yml; this one checks the image starts.
# The hora-nix volume keeps the container's Nix store between runs.
.PHONY: image
image:
	mkdir -p target
	docker run --rm -v "$(CURDIR)":/src:ro -v "$(CURDIR)/target":/out -v hora-nix:/nix nixos/nix:2.31.2 sh -c '\
	  git config --global --add safe.directory "*" && \
	  nix --extra-experimental-features "nix-command flakes" build "git+file:///src#hora-web" -L --out-link /tmp/result && \
	  cp -L /tmp/result /out/hora-image.tar.gz'
	docker load < target/hora-image.tar.gz
