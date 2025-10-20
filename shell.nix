{ pkgs ? import <nixpkgs> { } }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust toolchain
    rustc
    cargo
    rustfmt
    rust-analyzer
    clippy

    # GTK4 and dependencies
    gtk4
    gtk4-layer-shell
    pkg-config

    # Additional libraries that GTK4 might need
    glib
    cairo
    pango
    gdk-pixbuf
    graphene

    # udev for device monitoring
    udev
  ];

  # Environment variables for pkg-config to find GTK4 and udev
  PKG_CONFIG_PATH = "${pkgs.gtk4}/lib/pkgconfig:${pkgs.glib}/lib/pkgconfig:${pkgs.udev}/lib/pkgconfig";

  # Set up library path
  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath [
    pkgs.gtk4
    pkgs.glib
    pkgs.cairo
    pkgs.pango
    pkgs.gdk-pixbuf
    pkgs.udev
  ];

  shellHook = ''
    echo "GTK4 Rust development environment"
    echo "GTK4 version: $(pkg-config --modversion gtk4)"
    echo "Cargo version: $(cargo --version)"
    echo "Clippy version: $(cargo clippy --version)"
  '';
}
