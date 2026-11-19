{
  description = "inventory-mcp";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    crane.url = "github:ipetkov/crane";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, crane, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
        craneLib = crane.mkLib pkgs;

        commonArgs = {
          src = craneLib.cleanCargoSource ./.;
          strictDeps = true;
          nativeBuildInputs = with pkgs; [ pkg-config ];
          buildInputs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isDarwin [ pkgs.libiconv ];
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        bin = craneLib.buildPackage (commonArgs // {
          inherit cargoArtifacts;
          outputs = [ "out" "man" ];
          postInstall = ''
            find target -path '*/build/inventory-mcp-*/out/*.1' \
              -exec install -Dm644 -t "$man/share/man/man1" {} +
          '';
        });
      in
      {
        packages.default = bin;

        checks = {
          inherit bin;
          wt-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });
        };

        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
          packages = with pkgs; [ rust-analyzer ];
        };
      }
    );
}
