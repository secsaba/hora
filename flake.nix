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
    {
      self,
      nixpkgs,
      rust-overlay,
      ...
    }:
    let
      forAllSystems = nixpkgs.lib.genAttrs [
        "x86_64-darwin"
        "aarch64-darwin"
        "x86_64-linux"
        "aarch64-linux"
      ];
      pkgsFor =
        system:
        import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };
    in
    {
      # An output, not just a shell variable, so CI and release builds get the same pinned files.
      packages = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
          # The commit ADR 0001 vendors (tag v2.10.3bfinal-hora.1); bump both together. A changed file fails its hash.
          sweRev = "e041a0077076a574b8411ceee7f24b0733dc56ca";
          ephe = pkgs.linkFarm "swisseph-ephe" (
            map
              (f: {
                inherit (f) name;
                path = pkgs.fetchurl {
                  url = "https://raw.githubusercontent.com/secsaba/swisseph/${sweRev}/ephe/${f.name}";
                  inherit (f) hash;
                };
              })
              [
                # Covers 1200-2399 AD only; outside it the library silently falls back to Moshier (ADR 0001).
                { name = "sepl_12.se1"; hash = "sha256-jcys4lV2AaIj1finz2TeTmu7ioK5sTC5WlTUmt++VG0="; }
                { name = "sepl_18.se1"; hash = "sha256-yhOTzqs6RPvIlYh894nGiBmuahy8myIiWHLb5MzZmmY="; }
                { name = "semo_12.se1"; hash = "sha256-HGX9u4VPNQ0942r8PkrBJqU/Ngrk9ty1e+7XxpDn62E="; }
                { name = "semo_18.se1"; hash = "sha256-HKB71nwkN013ImGAwgpPmZbLoBNpeJSBBRjn61gspPc="; }
                # Star positions for star-anchored ayanamshas such as True Chitrapaksha.
                { name = "sefstars.txt"; hash = "sha256-GLDcr75bckB3PauiwDijJfWz/EFj9h4Kf06Sq9T1F8Y="; }
              ]
          );
        in
        {
          inherit ephe;
        }
      );

      devShells = forAllSystems (
        system:
        let
          pkgs = pkgsFor system;
        in
        {
          # No compiler package needed: mkShell's stdenv already provides `cc` (clang on macOS, gcc on Linux).
          default = pkgs.mkShell {
            packages = [
              (pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml)
              pkgs.cargo-deny
            ];
            # Not SE_EPHE_PATH: the server refuses to start with it set (ADR 0001).
            HORA_EPHE_PATH = self.packages.${system}.ephe;
            # cc passes LLVM's x86_64-apple-macosx, which the wrapper string-compares to its x86_64-apple-darwin.
            NIX_CC_WRAPPER_SUPPRESS_TARGET_WARNING = 1;
            shellHook = "unset SE_EPHE_PATH";
          };
        }
      );
    };
}
