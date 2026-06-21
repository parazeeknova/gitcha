.PHONY: test check check-types fmt install-hooks bump-version update release-tag clean-local dev run-release lint build install uninstall pkg pkg-install pkg-git pkg-git-install pkg-clean

dev:
	cargo run

run-release:
	cargo run --release

clean-local:
	rm -rf ~/.local/share/gitcha

check:
	./scripts/check.sh

test:
	./scripts/test.sh

check-types:
	./scripts/check-types.sh

fmt:
	cargo fmt --all

lint:
	cargo clippy --all-targets --all-features -- -D warnings && cargo fmt --all --check

install-hooks:
	./scripts/install-hooks.sh

bump-version:
	./scripts/bump-version.rs

update:
	cargo update

release-tag:
	./scripts/tag-release.sh

build:
	cargo build --release
	strip target/release/gitcha

install: build
	install -d $(DESTDIR)/usr/local/bin
	install -m 755 target/release/gitcha $(DESTDIR)/usr/local/bin/gitcha

uninstall:
	rm -f $(DESTDIR)/usr/local/bin/gitcha

pkg:
	cp PKGBUILD ./PKGBUILD.build
	makepkg -sf --skippgpcheck --config PKGBUILD.build
	rm -f PKGBUILD.build

pkg-install: pkg
	sudo pacman -U --noconfirm gitcha-bin-*.pkg.tar.zst

pkg-git:
	cp PKGBUILD.git ./PKGBUILD.build
	makepkg -sf --skippgpcheck --config PKGBUILD.build
	rm -f PKGBUILD.build

pkg-git-install: pkg-git
	sudo pacman -U --noconfirm gitcha-git-*.pkg.tar.zst

pkg-clean:
	rm -rf pkg src gitcha-bin-*.pkg.tar.zst gitcha-git-*.pkg.tar.zst
