# Copyright (c) 2026 Nicola Fiorillo
# SPDX-License-Identifier: PolyForm-Noncommercial-1.0.0

# Phonestra: rimandi ai comandi di sempre (cargo e gli script del repository).
#
#   make            cargo build (phonestra e phonestra-prova)
#   make test       cargo test, che comprende il controllo dei manuali
#   make clippy     cargo clippy --all-targets (nessun avviso ammesso)
#   make docs       rigenera i due manuali da docs/sources/
#   make docs-check solo il controllo dei manuali
#   make dist       l'AppImage, nel contenitore phonestra-appimage (serve podman)
#   make helper     ricompila android/phonestra-helper.jar (servono javac e strumenti/r8.jar)
#   make clean      cargo clean

# cargo installato con rustup sta in ~/.cargo/bin.
export PATH := $(HOME)/.cargo/bin:$(PATH)

CARGO   ?= cargo
PYTHON  ?= python3
PODMAN  ?= podman

.PHONY: all build test clippy docs docs-check dist helper clean

all: build

build:
	$(CARGO) build

test:
	$(CARGO) test

clippy:
	$(CARGO) clippy --all-targets

docs:
	$(PYTHON) docs/sources/build.py

docs-check:
	$(PYTHON) docs/sources/build.py --controlla

# Sempre i due passi: raccogliere senza ricompilare impacchetta un eseguibile vecchio.
dist:
	$(PODMAN) build -t phonestra-appimage packaging
	$(PODMAN) run --rm -v .:/phonestra:Z phonestra-appimage sh -c \
	  'CARGO_TARGET_DIR=target/appimage cargo build --release && packaging/collect.sh'

helper:
	sh android/helper/build.sh

clean:
	$(CARGO) clean
