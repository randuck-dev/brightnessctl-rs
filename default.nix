{ lib
, rustPlatform
, pkg-config
}:

rustPlatform.buildRustPackage rec {
  pname = "brightnessctl-rs";
  version = "0.1.0";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  nativeBuildInputs = [ pkg-config ];

  meta = with lib; {
    description = "Backlight brightness control CLI";
    homepage = "https://github.com/randuck-dev/brightnessctl-rs";
    license = licenses.mit;
    maintainers = [ ];
  };
}

