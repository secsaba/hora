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
                # Star positions for star-anchored ayanamsas such as True Chitrapaksha.
                { name = "sefstars.txt"; hash = "sha256-GLDcr75bckB3PauiwDijJfWz/EFj9h4Kf06Sq9T1F8Y="; }
              ]
          );
          # GeoNames regenerates its dumps daily, so the flake pins a snapshot uploaded as a release asset (CC BY 4.0).
          geonames =
            name: hash:
            pkgs.fetchurl {
              url = "https://github.com/secsaba/hora/releases/download/geonames-2026-10-27/${name}";
              inherit hash;
            };
          # cities500 lacks most Hungarian villages (GeoNames has no population for them), so every Hungarian
          # populated place joins it, less city sections (PPLX) and abandoned or destroyed places (PPLQ, PPLW).
          # Columns: geonameid, name, asciiname, alternatenames, latitude, longitude, country, population, zone.
          places = pkgs.runCommand "geonames-places.tsv" { nativeBuildInputs = [ pkgs.unzip ]; } ''
            unzip -p ${geonames "cities500.zip" "sha256-TBYwZYJYH6WZ0kbvED0k1R7XzdYVYD5rvzk7zWs+/0I="} cities500.txt > cities.txt
            unzip -p ${geonames "HU.zip" "sha256-Dzh2+DHhU2wY0ZOkxvq7VIyKstdmPdOBx575lat0b10="} HU.txt > hu.txt
            awk -F'\t' -v OFS='\t' '
              (FNR == NR || ($7 == "P" && $8 != "PPLX" && $8 != "PPLQ" && $8 != "PPLW")) && !seen[$1]++ {
                print $1, $2, $3, $4, $5, $6, $9, $15, $18
              }' cities.txt hu.txt > $out
          '';
          toolchain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
          hora-web-bin = (pkgs.makeRustPlatform { cargo = toolchain; rustc = toolchain; }).buildRustPackage {
            pname = "hora-web";
            version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).workspace.package.version;
            # Tracked files only, so the gitignored book/ never enters the store.
            src = self;
            cargoLock.lockFile = ./Cargo.lock;
            cargoBuildFlags = [ "-p" "hora-web" ];
            # The tests run here too, with the dev shell's data files, so a red main never ships.
            HORA_EPHE_PATH = ephe;
            HORA_PLACES = places;
            TZDIR = "${pkgs.tzdata}/share/zoneinfo";
            # Panic locations name rust-src files, which would pull the whole toolchain (gigabytes) into the image.
            nativeBuildInputs = [ pkgs.removeReferencesTo ];
            postFixup = "remove-references-to -t ${toolchain} $out/bin/hora-web";
          };
        in
        {
          inherit ephe places hora-web-bin;
        }
        # dockerTools builds Linux images only; production is x86_64 Linux.
        // pkgs.lib.optionalAttrs pkgs.stdenv.isLinux {
          hora-web = pkgs.dockerTools.buildLayeredImage {
            name = "ghcr.io/secsaba/hora";
            tag = hora-web-bin.version;
            config = {
              # nobody: hora only reads its data files.
              User = "65534:65534";
              Cmd = [ "${hora-web-bin}/bin/hora-web" ];
              # As the dev shell sets them, but listening beyond the container's loopback (ADR 0007).
              Env = [
                "HORA_EPHE_PATH=${ephe}"
                "HORA_PLACES=${places}"
                "TZDIR=${pkgs.tzdata}/share/zoneinfo"
                "HORA_BIND=0.0.0.0:3000"
              ];
              ExposedPorts."3000/tcp" = { };
              Labels."org.opencontainers.image.source" = "https://github.com/secsaba/hora";
            };
          };
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
              pkgs.poppler-utils
            ];
            # Not SE_EPHE_PATH: the server refuses to start with it set (ADR 0001).
            HORA_EPHE_PATH = self.packages.${system}.ephe;
            HORA_PLACES = self.packages.${system}.places;
            # Plain tzdb, no backzone (ADR 0005); jiff reads it here instead of the host's copy.
            TZDIR = "${pkgs.tzdata}/share/zoneinfo";
            # cc passes LLVM's x86_64-apple-macosx, which the wrapper string-compares to its x86_64-apple-darwin.
            NIX_CC_WRAPPER_SUPPRESS_TARGET_WARNING = 1;
            shellHook = "unset SE_EPHE_PATH";
          };
        }
      );
    };
}
