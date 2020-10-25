export LDFLAGS="-L/usr/local/opt/opencv@3/lib"
export CPPFLAGS="-I/usr/local/opt/opencv@3/include"
export PKG_CONFIG_PATH="/usr/local/opt/opencv@3/lib/pkgconfig"

echo cargo "$*"
cargo "$*"
