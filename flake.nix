{
  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    naersk = {
      url = "github:nix-community/naersk";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs =
    {
      self,
      fenix,
      flake-utils,
      naersk,
      nixpkgs,
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        # TODO: Update this on new releases
        version = "0.3.0";

        pkgs = nixpkgs.legacyPackages.${system};
        raspberryPiZeroTarget = "arm-unknown-linux-gnueabihf";
        defaultToolchain = fenix.packages.${system}.stable.toolchain;

        mkHyperionRsBinary =
          {
            toolchain,
            additionalAttrs ? { },
          }:
          (naersk.lib.${system}.override {
            cargo = toolchain;
            rustc = toolchain;
          }).buildPackage
            (
              {
                src = ./.;

                nativeBuildInputs = with pkgs; [
                  protobuf
                  patchelf
                ];

                # Python version used during the build
                PYO3_PYTHON = "${pkgs.python3}/bin/python";
                PYO3_USE_ABI3_FORWARD_COMPATIBILITY = "1";
                HYPERION_RS_GIT_VERSION = version;
              }
              // additionalAttrs
            );

        nativeHyperionRsBinary = mkHyperionRsBinary {
          toolchain = defaultToolchain;

          additionalAttrs.postInstall = ''
            patchelf --set-interpreter /lib64/ld-linux-x86-64.so.2 $out/bin/hyperiond
          '';
        };

        rasperryPiZeroHyperionRsBinary =
          let
            target = raspberryPiZeroTarget;
            toolchain =
              with fenix.packages.${system};
              combine [
                stable.cargo
                stable.rustc
                targets.${target}.stable.rust-std
              ];
            cc = (
              let
                inherit (pkgs.pkgsCross.raspberryPi.stdenv) cc;
              in
              "${cc}/bin/${cc.targetPrefix}cc"
            );
          in
          mkHyperionRsBinary {
            inherit toolchain;

            additionalAttrs = {
              CARGO_BUILD_TARGET = target;
              CARGO_TARGET_ARM_UNKNOWN_LINUX_GNUEABIHF_LINKER = cc;
              CC_arm_unknown_linux_gnueabihf = cc;
              # Python version for the target system
              PYO3_CROSS_LIB_DIR = "${pkgs.pkgsCross.raspberryPi.python313}/lib";

              postInstall = ''
                patchelf --set-interpreter /lib/ld-linux-armhf.so.3 $out/bin/hyperiond
              '';
            };
          };

        mkHyperionRsRoot =
          { binary, system }:
          pkgs.stdenvNoCC.mkDerivation {
            inherit version;
            name = "hyperion-rs";

            src = ./.;

            buildPhase = ''
              mkdir -p $out/bin $out/share/hyperion

              cp ${binary}/bin/hyperiond $out/bin/hyperiond-rs
              cp -rv ext/hyperion.ng/assets/webconfig $out/share/hyperion
              cp -rv ext/hyperion.ng/effects $out/share/hyperion
            '';

            passthru = {
              inherit system;
            };
          };

        mkHyperionRsArchive =
          { root }:
          pkgs.stdenvNoCC.mkDerivation {
            inherit version;
            name = "hyperion-rs-archive";

            src = root;

            nativeBuildInputs = with pkgs; [
              gnutar
              xz
            ];

            buildPhase = ''
              mkdir -p $out
              tar --transform 's,^./,hyperion.rs/,' -cJvf $out/hyperion.rs-${root.system}.tar.xz .
            '';
          };
      in
      {
        packages = rec {
          default = mkHyperionRsRoot {
            binary = nativeHyperionRsBinary;
            system = "x86_64-unknown-linux-gnu";
          };
          defaultArchive = mkHyperionRsArchive { root = default; };

          raspberryPiZero = mkHyperionRsRoot {
            binary = rasperryPiZeroHyperionRsBinary;
            system = raspberryPiZeroTarget;
          };
          raspberryPiZeroArchive = mkHyperionRsArchive { root = raspberryPiZero; };

          allArchives = pkgs.stdenvNoCC.mkDerivation {
            inherit version;
            name = "hyperion-rs-archives";

            src = ./.;

            buildPhase = ''
              mkdir -p $out
              cp ${defaultArchive}/* $out
              cp ${raspberryPiZeroArchive}/* $out
            '';
          };
        };

        devShells.default = pkgs.mkShell {
          nativeBuildInputs = [
            defaultToolchain
            pkgs.protobuf
          ];

          PYO3_PYTHON = "${pkgs.python3}/bin/python";
          PYO3_USE_ABI3_FORWARD_COMPATIBILITY = "1";
          LD_LIBRARY_PATH = "${pkgs.python3}/lib";
        };
      }
    );
}
