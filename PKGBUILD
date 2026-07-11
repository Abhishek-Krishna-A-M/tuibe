# Maintainer: Abhishek Krishna A M <abhishekkrishna2k6@gmail.com>

pkgname=tuibe
pkgver=0.1.0
pkgrel=1
pkgdesc="High-performance TUI YouTube Music client"
arch=('x86_64')
url="https://github.com/Abhishek-Krishna-A-M/tuibe"
license=('MIT')
depends=(
  'python'
  'python-ytmusicapi'
  'cava'
  'yt-dlp'
  'libmpv'
  'pipewire'
)
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::$url/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
  cd "$srcdir/$pkgname-$pkgver"
  cargo build --release --frozen
}

check() {
  cd "$srcdir/$pkgname-$pkgver"
  cargo test --frozen
}

package() {
  cd "$srcdir/$pkgname-$pkgver"
  install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
  install -Dm644 "README.md" "$pkgdir/usr/share/doc/$pkgname/README.md"
  install -Dm644 "scripts/ytmusic_helper.py" "$pkgdir/usr/share/$pkgname/scripts/ytmusic_helper.py"
}
