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

sha256sums=('SKIP')

package() {
    install -Dm755 "$srcdir/gitcha" "$pkgdir/usr/bin/gitcha"
    install -Dm644 "$srcdir/LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE" 2>/dev/null || true
}
