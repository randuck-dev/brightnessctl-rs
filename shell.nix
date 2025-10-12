{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    rustc
    cargo
    rustfmt
    rust-analyzer
    clippy
  ];
  
  # For projects with native dependencies
  nativeBuildInputs = with pkgs; [
    pkg-config
  ];
}
