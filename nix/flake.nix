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
        devShells.default = let
          # https://eu90h.com/wgpu-winit-and-nixos.html
          libPath = with pkgs;
            lib.makeLibraryPath [
              libGL
              libxkbcommon
              wayland
            ];
        in
          pkgs.mkShell {
            nativeBuildInputs = with pkgs; [
              pkg-config
            ];
            buildInputs = with pkgs; [
              libxkbcommon.dev
              alsa-lib.dev
              udev.dev
            ];

            LD_LIBRARY_PATH = libPath;

            packages = with pkgs; [
              rust-analyzer
              rust-bin.stable."1.84.0".default

              hexyl
            ];
          };
      }
    );
}
