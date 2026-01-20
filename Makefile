# cli-web-search Makefile
#
# Common targets for building, testing, and packaging

PACKAGE_NAME := cli-web-search
VERSION := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*= *"\(.*\)"/\1/')
CARGO := cargo
DESTDIR ?=
PREFIX ?= /usr/local
BINDIR := $(PREFIX)/bin

# Detect architecture for package naming
ARCH := $(shell dpkg --print-architecture 2>/dev/null || uname -m)

# Build output
RELEASE_BIN := target/release/$(PACKAGE_NAME)
DEBUG_BIN := target/debug/$(PACKAGE_NAME)

.PHONY: all build release debug test test-release clean distclean install uninstall deb deb-clean lint fmt check help

# Default target
all: release

## Build targets

# Build release binary (optimized)
release:
	$(CARGO) build --release

# Build debug binary
debug:
	$(CARGO) build

# Alias for release
build: release

## Testing targets

# Run tests in debug mode
test:
	$(CARGO) test

# Run tests in release mode
test-release:
	$(CARGO) test --release

## Code quality targets

# Run clippy linter
lint:
	$(CARGO) clippy -- -D warnings

# Format code
fmt:
	$(CARGO) fmt

# Check formatting without changing files
fmt-check:
	$(CARGO) fmt -- --check

# Run all checks (format, lint, test)
check: fmt-check lint test

## Clean targets

# Clean build artifacts
clean:
	$(CARGO) clean

# Clean everything including Debian build artifacts
distclean: clean deb-clean
	rm -rf debian/.debhelper debian/cli-web-search debian/debhelper-build-stamp
	rm -rf debian/files debian/*.substvars debian/*.debhelper.log
	rm -f *.deb *.buildinfo *.changes

## Installation targets

# Install to system (requires release build)
install: release
	install -d $(DESTDIR)$(BINDIR)
	install -m 755 $(RELEASE_BIN) $(DESTDIR)$(BINDIR)/$(PACKAGE_NAME)

# Uninstall from system
uninstall:
	rm -f $(DESTDIR)$(BINDIR)/$(PACKAGE_NAME)

## Debian packaging targets

# Build Debian package
deb: release
	@echo "Building Debian package..."
	@mkdir -p debian/$(PACKAGE_NAME)/usr/bin
	@cp $(RELEASE_BIN) debian/$(PACKAGE_NAME)/usr/bin/
	@mkdir -p debian/$(PACKAGE_NAME)/DEBIAN
	@echo "Package: $(PACKAGE_NAME)" > debian/$(PACKAGE_NAME)/DEBIAN/control
	@echo "Version: $(VERSION)" >> debian/$(PACKAGE_NAME)/DEBIAN/control
	@echo "Section: utils" >> debian/$(PACKAGE_NAME)/DEBIAN/control
	@echo "Priority: optional" >> debian/$(PACKAGE_NAME)/DEBIAN/control
	@echo "Architecture: $(ARCH)" >> debian/$(PACKAGE_NAME)/DEBIAN/control
	@echo "Maintainer: Scott Glover <scottgl@gmail.com>" >> debian/$(PACKAGE_NAME)/DEBIAN/control
	@echo "Description: Cross-platform CLI web search tool for AI agents" >> debian/$(PACKAGE_NAME)/DEBIAN/control
	@echo " cli-web-search provides web search capabilities to AI agents and CLI users." >> debian/$(PACKAGE_NAME)/DEBIAN/control
	@echo " Supports multiple search providers including Brave, Google, DuckDuckGo," >> debian/$(PACKAGE_NAME)/DEBIAN/control
	@echo " Tavily, Serper, Firecrawl, SerpAPI, and Bing." >> debian/$(PACKAGE_NAME)/DEBIAN/control
	@dpkg-deb --build debian/$(PACKAGE_NAME) $(PACKAGE_NAME)_$(VERSION)_$(ARCH).deb
	@echo "Created: $(PACKAGE_NAME)_$(VERSION)_$(ARCH).deb"

# Build Debian package using dpkg-buildpackage (full build)
deb-full:
	dpkg-buildpackage -us -uc -b

# Clean Debian build artifacts
deb-clean:
	rm -rf debian/$(PACKAGE_NAME)
	rm -f $(PACKAGE_NAME)_*.deb
	rm -f $(PACKAGE_NAME)_*.buildinfo
	rm -f $(PACKAGE_NAME)_*.changes

## Documentation targets

# Generate documentation
doc:
	$(CARGO) doc --no-deps

# Open documentation in browser
doc-open:
	$(CARGO) doc --no-deps --open

## Utility targets

# Show version
version:
	@echo "$(PACKAGE_NAME) version $(VERSION)"

# Show help
help:
	@echo "cli-web-search Makefile"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Build targets:"
	@echo "  all, build, release  Build optimized release binary"
	@echo "  debug                Build debug binary"
	@echo ""
	@echo "Test targets:"
	@echo "  test                 Run tests in debug mode"
	@echo "  test-release         Run tests in release mode"
	@echo ""
	@echo "Code quality:"
	@echo "  lint                 Run clippy linter"
	@echo "  fmt                  Format code with rustfmt"
	@echo "  fmt-check            Check code formatting"
	@echo "  check                Run fmt-check, lint, and test"
	@echo ""
	@echo "Clean targets:"
	@echo "  clean                Clean build artifacts"
	@echo "  distclean            Clean all artifacts including packages"
	@echo ""
	@echo "Installation:"
	@echo "  install              Install to system (PREFIX=$(PREFIX))"
	@echo "  uninstall            Remove from system"
	@echo ""
	@echo "Packaging:"
	@echo "  deb                  Build Debian package (simple method)"
	@echo "  deb-full             Build Debian package (dpkg-buildpackage)"
	@echo "  deb-clean            Clean Debian build artifacts"
	@echo ""
	@echo "Documentation:"
	@echo "  doc                  Generate documentation"
	@echo "  doc-open             Generate and open documentation"
	@echo ""
	@echo "Utility:"
	@echo "  version              Show package version"
	@echo "  help                 Show this help message"
