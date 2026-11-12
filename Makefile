ARTIFACT ?= oriha-cli
RELEASE_GROUP ?= oriha
RELEASE_ARGS ?= --apply
override RELEASE_ARGS := --group $(RELEASE_GROUP) $(RELEASE_ARGS)
ACCEPTANCE_ARGS ?= --gate cli-standalone
include standards/make/common.mk
CRAFTGUILD_LAUNCH = sh tools/standards.sh

.PHONY: all build-cli build-expo build-font-registry prepare-libraries acceptance-font-registry
all build-cli: build
build-expo:
	node packages/expo-oriha/scripts/build-package.mjs
build-font-registry:
	$(CRAFTGUILD_LAUNCH) build --artifact font-registry
prepare: prepare-libraries
prepare-libraries:
	$(CRAFTGUILD_LAUNCH) prepare --artifact native-bridge
	$(CRAFTGUILD_LAUNCH) prepare --artifact wasm
	$(CRAFTGUILD_LAUNCH) prepare --artifact windows-bridge
acceptance-font-registry:
	$(CRAFTGUILD_LAUNCH) acceptance --gate font-registry-runtime
