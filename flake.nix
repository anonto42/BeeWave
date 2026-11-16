{
  description = "WalPad: Rust ESP32 firmware + Web Bluetooth PWA";

  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs = { self, nixpkgs }:
    let
      systems = [ "x86_64-linux" "aarch64-linux" "x86_64-darwin" "aarch64-darwin" ];
      forAll = f: nixpkgs.lib.genAttrs systems (s: f nixpkgs.legacyPackages.${s});
    in {
      devShells = forAll (pkgs: {
        default = pkgs.mkShell {
          packages = with pkgs; [
            # orchestration
            gnumake git direnv
            # host rust (core/protocol tests, no_std checks). The Xtensa toolchain comes from `make setup` (espup).
            rustup
            # esp tooling
            espup espflash ldproxy
            # ESP-IDF (embuild) build deps
            cmake ninja python3 pkg-config openssl libusb1 gcc
            # web
            nodejs_22 pnpm
          ];
          shellHook = ''
            export ESP_EXPORT="''${ESP_EXPORT:-$HOME/export-esp.sh}"
            echo "walpad dev shell: run 'make help'"
          '';
        };
      });

      formatter = forAll (pkgs: pkgs.nixpkgs-fmt);
    };
}
