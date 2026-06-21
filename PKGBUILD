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

sha256sums=('029764f080bd220176a309a96488747e138f9519e4bab12e195896f141213c60')

package() {
    install -Dm755 "$srcdir/usr/bin/gitcha" "$pkgdir/usr/bin/gitcha"
    install -Dm644 "$srcdir/usr/share/applications/gitcha.desktop" "$pkgdir/usr/share/applications/gitcha.desktop"
    cp -r "$srcdir/usr/share/icons" "$pkgdir/usr/share/"
    cp -r "$srcdir/usr/lib/gitcha" "$pkgdir/usr/lib/"
}

post_install() {
    gtk-update-icon-cache -q -t -f usr/share/icons/hicolor || true
    update-desktop-database -q usr/share/applications || true
}

post_remove() {
    gtk-update-icon-cache -q -t -f usr/share/icons/hicolor || true
    update-desktop-database -q usr/share/applications || true
}
