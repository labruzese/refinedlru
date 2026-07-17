LIB_NAME = vlru
VERIFICATION_DIR = ./verification

PREFIX = $(shell opam var prefix 2>/dev/null)
DUNE_INSTALL = dune install --prefix $(PREFIX)

BLUE   := $(shell tput setaf 4 2>/dev/null)
GREEN  := $(shell tput setaf 2 2>/dev/null)
YELLOW := $(shell tput setaf 3 2>/dev/null)
BOLD   := $(shell tput bold 2>/dev/null)
RESET  := $(shell tput sgr0 2>/dev/null)

define log
	echo "$(BOLD)$(BLUE)==>$(RESET) $(BOLD)$(1)$(RESET) $(2)"
endef

.PHONY: all generate build rebuild clean nuke

all: build

generate:
	@$(call log,generate,$(LIB_NAME))
	cargo refinedrust

build: generate
	@$(call log,build,$(VERIFICATION_DIR)/$(LIB_NAME))
	cd $(VERIFICATION_DIR)/$(LIB_NAME) && dune build --display short

rebuild: clean build
	@$(call log,rebuild,done)

clean:
	@$(call log,clean,$(VERIFICATION_DIR))
	cargo clean
	rm -rf $(VERIFICATION_DIR)/$(LIB_NAME)/generated
	rm -f $(VERIFICATION_DIR)/$(LIB_NAME)/interface.rrlib
	rm -f $(VERIFICATION_DIR)/$(LIB_NAME)/proofs/dune
	cd $(VERIFICATION_DIR) && dune clean

nuke: clean
	@$(call log,nuke,$(VERIFICATION_DIR)/$(LIB_NAME))
	rm -rf $(VERIFICATION_DIR)/$(LIB_NAME)
	rm -f $(VERIFICATION_DIR)/dune-project
	@echo "$(YELLOW)warning:$(RESET) removed manual proofs in $(LIB_NAME)"