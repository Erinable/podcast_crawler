#!/bin/bash
echo 'export PATH="/usr/local/opt/libpq/bin:$PATH"' >> ~/.zshrc
echo 'export LDFLAGS="-L/usr/local/opt/libpq/lib"' >> ~/.zshrc
echo 'export CPPFLAGS="-I/usr/local/opt/libpq/include"' >> ~/.zshrc
echo 'export PKG_CONFIG_PATH="/usr/local/opt/libpq/lib/pkgconfig"' >> ~/.zshrc
