SHELL := /bin/bash
RAGEL := ragel 

.PHONY: build
build: internal/parser/machine.go

internal/parser/machine.go: internal/parser/machine.go.rl
	$(RAGEL) -Z -G2 -e -o $@ $<
	@sed -i '/^\/\/line/d' $@
	$(MAKE) file=$@ snake2camel

.PHONY: snake2camel
snake2camel:
	@awk -i inplace '{ \
	while ( match($$0, /(.*)([a-z]+[0-9]*)_([a-zA-Z0-9])(.*)/, cap) ) \
	$$0 = cap[1] cap[2] toupper(cap[3]) cap[4]; \
	print \
	}' $(file)

.PHONY: clean
clean: internal/parser/machine.go
	@rm -f $?
