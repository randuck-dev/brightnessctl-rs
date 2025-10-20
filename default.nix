{ pkgs ? import <nixpkgs> { } }:

pkgs.rustPlatform.buildRustPackage rec {
  pname = "brightnessctl-rs";
  version = "0.1.0";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = [ pkgs.pkg-config ];

  meta = with pkgs.lib; {
    description = "Backlight brightness control CLI";
    homepage = "https://github.com/randuck-dev/brightnessctl-rs";
    license = licenses.mit;
    maintainers = [ ];
  };
}

