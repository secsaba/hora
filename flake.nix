{
  description = "hora development environment";

  inputs = {
    # 26.05 is the last release that evaluates on x86_64-darwin (the dev host); see docs/research/rust-stack-and-wasm.md.
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-26.05-darwin";
    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };

  outputs =
    { nixpkgs, rust-overlay, ... }:
    let
      forAllSystems = nixpkgs.lib.genAttrs [
        "x86_64-darwin"
        "aarch64-darwin"
        "x86_64-linux"
        "aarch64-linux"
      ];
    in
    {
      devShells = forAllSystems (
        system:
        let
          pkgs = import nixpkgs {
            inherit system;
            overlays = [ rust-overlay.overlays.default ];
          };
          # Same fork tag as the vendored C source (ADR 0001); a changed file fails its hash.
          ephe = pkgs.linkFarm "swisseph-ephe" (
            map
              (f: {
                name = "${f.name}.se1";
                path = pkgs.fetchurl {
                  url = "https://raw.githubusercontent.com/secsaba/swisseph/v2.10.3bfinal-hora.1/ephe/${f.name}.se1";
                  inherit (f) hash;
                };
              })
              [
                { name = "sepl_12"; hash = "sha256-jcys4lV2AaIj1finz2TeTmu7ioK5sTC5WlTUmt++VG0="; }
                { name = "sepl_18"; hash = "sha256-yhOTzqs6RPvIlYh894nGiBmuahy8myIiWHLb5MzZmmY="; }
                { name = "semo_12"; hash = "sha256-HGX9u4VPNQ0942r8PkrBJqU/Ngrk9ty1e+7XxpDn62E="; }
                { name = "semo_18"; hash = "sha256-HKB71nwkN013ImGAwgpPmZbLoBNpeJSBBRjn61gspPc="; }
              ]
          );
        in
        {
          # mkShell's stdenv puts a C compiler on PATH as `cc` (clang on macOS, gcc on Linux).
          default = pkgs.mkShell {
            packages = [ (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml) ];
            # Not SE_EPHE_PATH: the server refuses to start with it set (ADR 0001).
            HORA_EPHE_PATH = ephe;
          };
        }
      );
    };
}
