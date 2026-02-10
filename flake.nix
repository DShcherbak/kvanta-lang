{

  description = "Rust with WebAssembly";

  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "nixpkgs/nixos-23.11";
    nixpkgs-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, fenix, nixpkgs, nixpkgs-unstable, flake-utils }:

    flake-utils.lib.eachDefaultSystem
      (system:
        let
          pkgs = nixpkgs.legacyPackages.${system};
          pkgsUnstable = nixpkgs-unstable.legacyPackages.${system};
          f = with fenix.packages.${system}; combine [
            stable.toolchain
            targets.wasm32-unknown-unknown.stable.rust-std
          ];
        in
          {
            devShells.default =
              pkgs.mkShell {
                name = "rust-wasm-final-attempt";
		RUST_BACKTRACE=1;

                packages = with pkgs; [
                  f
                  llvmPackages.bintools
                  nodePackages.typescript-language-server # <-- change here
                  pkgsUnstable.nodejs_22
                  vscode-langservers-extracted # <-- change here
                  wasm-pack
		              python3
                  cargo
                  llvmPackages.lldb
                ];

                CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_LINKER = "lld"; 
              };
          }
      );
}
