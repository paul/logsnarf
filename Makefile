
STACK_NAME ?= logsnarf-test-sam
ARCH := aarch64-unknown-linux-gnu
ARCH_SPLIT = $(subst -, ,$(ARCH))

.PHONY: build deploy tests

all: build test deploy
ci: build test

build:
	cargo lambda build --release --target $(ARCH)

deploy:
	if [ -f samconfig.toml ]; \
		then sam deploy --stack-name $(STACK_NAME); \
		else sam deploy -g --stack-name $(STACK_NAME); \
	fi

test:
	cargo test --lib --bins
