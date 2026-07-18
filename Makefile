CARGO ?= cargo
INSTALL ?= install
PREFIX ?= /usr/local
SYSCONFDIR ?= /etc
SYSTEMD_UNIT_DIR ?= $(SYSCONFDIR)/systemd/system

.PHONY: all build test check install

all: build

build:
	$(CARGO) build --release --locked

test:
	$(CARGO) test --locked

check:
	$(CARGO) fmt --all -- --check
	$(CARGO) clippy --all-targets --locked -- -D warnings
	$(CARGO) test --locked

install:
	$(INSTALL) -d "$(DESTDIR)$(PREFIX)/bin"
	$(INSTALL) -m 0755 target/release/salyut-site \
		"$(DESTDIR)$(PREFIX)/bin/salyut-site"
	$(INSTALL) -d "$(DESTDIR)$(SYSTEMD_UNIT_DIR)"
	$(INSTALL) -m 0644 etc/systemd/system/salyut-site.service \
		"$(DESTDIR)$(SYSTEMD_UNIT_DIR)/salyut-site.service"
