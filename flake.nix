{
  description = "Tasklynx — a local-first task manager for developers";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs =
    {
      self,
      nixpkgs,
      flake-utils,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        pkgs = import nixpkgs { inherit system; };
        version = pkgs.lib.removeSuffix "\n" (builtins.readFile ./.version);
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "tasklynx";
          inherit version;
          src = ./.;
          cargoLock.lockFile = ./Cargo.lock;

          meta = with pkgs.lib; {
            description = "A developer-first local task manager with Git-style workflows";
            homepage = "https://github.com/peekstore/tasklynx";
            documentation = "https://peekstore.github.io/tasklynx/";
            license = licenses.mit;
            mainProgram = "tasklynx";
            platforms = platforms.unix ++ platforms.windows;
          };
        };

        packages.tasklynx = self.packages.${system}.default;

        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            cargo
            rustc
            clippy
            rustfmt
          ];
        };
      }
    );
}
