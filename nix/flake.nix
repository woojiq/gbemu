{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    rust-overlay,
  }:
    flake-utils.lib.eachDefaultSystem (
      system: let
        pkgs = nixpkgs.legacyPackages.${system}.extend (import rust-overlay);
      in {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          # buildInputs = with pkgs; [
          #   libxkbcommon.dev
          #   alsa-lib.dev
          #   udev.dev
          # ];

          packages = with pkgs; [
            rust-analyzer
            rust-bin.stable."1.84.0".default

            hexyl
          ];
        };
      }
    );
}
