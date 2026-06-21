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

sha256sums=('0019dfc4b32d63c1392aa264aed2253c1e0c2fb09216f8e2cc269bbfb8bb49b5')

package() {
    install -Dm755 "$srcdir/gitcha" "$pkgdir/usr/bin/gitcha"
    install -Dm644 "$srcdir/LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE" 2>/dev/null || true
}
