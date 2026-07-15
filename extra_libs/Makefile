# in dependency load order
LIBS ?= borrow hash hashmap

PREFIX = $(shell opam var prefix 2>/dev/null)
DUNE_INSTALL = dune install --prefix $(PREFIX)

.PHONY: all generate build install reinstall clean $(LIBS)

all: install

# specs + dune files
generate:
	@set -e; for d in $(LIBS); do \
		echo "==> [generate] $$d"; \
		( cd $$d && cargo refinedrust ); \
	done

# build the generated Rocq theories.
build: generate
	@set -e; for d in $(LIBS); do \
		echo "==> [build] $$d/target/verify"; \
		( cd $$d/target/verify && dune build --display short ); \
	done

# install each theory into the switch, in dependency order
install: generate
	@set -e; for d in $(LIBS); do \
		echo "==> [install] $$d/target/verify"; \
		( cd $$d/target/verify && dune build @install --display short && $(DUNE_INSTALL) ); \
	done

# force a clean reinstall
reinstall: clean install

clean:
	@for d in $(LIBS); do \
		echo "==> [clean] $$d"; \
		( cd $$d && cargo clean 2>/dev/null; rm -rf _build ) || true; \
	done