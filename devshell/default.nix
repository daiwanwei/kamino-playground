{ rustToolchain
, cargoArgs
, unitTestArgs
, pkgs
, lib
, stdenv
, darwin
, libiconv
, ...
}:

let
  cargo-ext = pkgs.callPackage ./cargo-ext.nix { inherit cargoArgs unitTestArgs; };
in
pkgs.mkShell {
  name = "dev-shell";

  buildInputs = lib.optionals stdenv.isDarwin [
    darwin.apple_sdk.frameworks.Security
    darwin.apple_sdk.frameworks.SystemConfiguration
    libiconv
  ];

  nativeBuildInputs = with pkgs; [
    cargo-ext.cargo-build-all
    cargo-ext.cargo-clippy-all
    cargo-ext.cargo-doc-all
    cargo-ext.cargo-nextest-all
    cargo-ext.cargo-test-all
    cargo-nextest
    rustToolchain

    tokei

    protobuf

    jq

    hclfmt
    nixpkgs-fmt
    nodePackages.prettier
    shfmt
    taplo
    treefmt
    # clang-tools contains clang-format
    clang-tools

    shellcheck
  ];

  shellHook = ''
    export NIX_PATH="nixpkgs=${pkgs.path}"
  '';
}
