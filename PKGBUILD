# Maintainer: parazeeknova <harsh@itssingularity.com>
pkgname=gitcha-bin
pkgver=0.1.70
pkgrel=1
pkgdesc='one app to gitcha all — a native git client built in Rust'
arch=('x86_64')
url='https://github.com/parazeeknova/gitcha'
license=('MIT')
depends=('gtk3' 'libxkbcommon-x11')
provides=('gitcha')
conflicts=('gitcha')

source=("$url/releases/download/v$pkgver/gitcha_${pkgver}_x86_64.tar.gz")

sha256sums=('69ce006d22b8c7cd794bb4214e6b020deb152ff18dc95950a0b88209f0a313ec')

package() {
    install -Dm755 "$srcdir/usr/bin/gitcha" "$pkgdir/usr/bin/gitcha"
    install -Dm644 "$srcdir/usr/share/applications/gitcha.desktop" "$pkgdir/usr/share/applications/gitcha.desktop" 2>/dev/null || true
    cp -r "$srcdir/usr/share/icons" "$pkgdir/usr/share/" 2>/dev/null || true
    cp -r "$srcdir/usr/lib/gitcha" "$pkgdir/usr/lib/" 2>/dev/null || true
}
