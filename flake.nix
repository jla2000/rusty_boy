{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, rust-overlay }:
    let
      system = "x86_64-linux";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ rust-overlay.overlays.default ];
      };
      rust = pkgs.rust-bin.stable."latest".default.override {
        extensions = [ "rust-src" "rust-analyzer" ];
      };
      rustPlatform = pkgs.makeRustPlatform {
        cargo = rust;
        rustc = rust;
      };
      graphicsPackages = with pkgs; [
        libGL
        vulkan-loader
        libxkbcommon
        wayland
        xorg.libX11
        xorg.libXcursor
        xorg.libXi
        xorg.libXrandr
        xorg.libXinerama
      ];
      libPath = pkgs.lib.makeLibraryPath graphicsPackages;
    in
    {
      packages.${system}.default = rustPlatform.buildRustPackage rec {
        pname = "rusty_boy";
        version = "0.1.0";
        src = self;
        cargoLock = {
          lockFile = ./Cargo.lock;
        };
        postFixup = ''
          patchelf --add-rpath ${libPath} $out/bin/rusty_boy
        '';
      };
      devShells.${system}.default = pkgs.mkShell {
        packages = [ rust pkgs.cmake ];
        LD_LIBRARY_PATH = libPath;
        buildInputs = graphicsPackages;
        RUST_LOG = "info";
      };
    };
}
