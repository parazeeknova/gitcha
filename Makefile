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
	mkdir -p .makepkg
	cp PKGBUILD .makepkg/PKGBUILD
	cd .makepkg && makepkg -sf --skippgpcheck
	mv .makepkg/*.pkg.tar.zst . 2>/dev/null || true
	rm -rf .makepkg

pkg-install: pkg
	sudo pacman -U --noconfirm gitcha-bin-*.pkg.tar.zst

pkg-git:
	mkdir -p .makepkg
	cp PKGBUILD.git .makepkg/PKGBUILD
	cd .makepkg && makepkg -sf --skippgpcheck
	mv .makepkg/*.pkg.tar.zst . 2>/dev/null || true
	rm -rf .makepkg

pkg-git-install: pkg-git
	sudo pacman -U --noconfirm gitcha-git-*.pkg.tar.zst

pkg-clean:
	rm -rf .makepkg gitcha-bin-*.pkg.tar.zst gitcha-git-*.pkg.tar.zst
