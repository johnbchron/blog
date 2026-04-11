{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    crane.url = "github:ipetkov/crane";
    devshell.url = "github:numtide/devshell";
  };

  outputs = { nixpkgs, rust-overlay, devshell, flake-utils, crane, ... }: 
    flake-utils.lib.eachDefaultSystem (system: let
      pkgs = import nixpkgs {
        inherit system;
        overlays = [
          (import rust-overlay)
          devshell.overlays.default
        ];
      };
      lib = pkgs.lib;

      toolchain_fn = p: p.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
        extensions = [ "rust-src" "rust-analyzer" ];
      });
      minimal_toolchain_fn = p: p.rust-bin.selectLatestNightlyWith (toolchain: toolchain.minimal);

      craneLib = (crane.mkLib pkgs).overrideToolchain minimal_toolchain_fn;

      unfilteredRoot = ./.;
      src = lib.fileset.toSource {
        root = unfilteredRoot;
        fileset = lib.fileset.unions [
          (craneLib.fileset.commonCargoSources unfilteredRoot)
          (lib.fileset.maybeMissing ./public)
          (lib.fileset.maybeMissing ./style)
        ];
      };

      server-args = {
        inherit src;
        inherit (craneLib.crateNameFromCargoToml { inherit src; }) version;
        pname = "blog";

        strictDeps = true;
        doCheck = false;
      };

      # transform the css with tailwind
      css = pkgs.stdenv.mkDerivation {
        pname = "grid-css";
        version = "0.1.0";
        inherit src;

        buildPhase = ''
          ${pkgs.tailwindcss_4}/bin/tailwindcss \
            --input style/main.css \
            --output $out \
            --minify
        '';
      };
      
      server = craneLib.buildPackage (server-args // {
        cargoArtifacts = craneLib.buildDepsOnly server-args;

        nativeBuildInputs = (server-args.nativeBuildInputs or [ ]) ++ (with pkgs; [
          makeWrapper
        ]);

        doNotPostBuildInstallCargoBinaries = true;
        installPhaseCommand = ''
          mkdir -p $out/bin
          cp target/release/${server-args.pname} $out/bin/${server-args.pname}
          cp ${css} $out/bin/styles.css
          cp -r public $out/bin/public

          wrapProgram $out/bin/${server-args.pname} \
            --set-default STATIC_ASSET_DIR $out/bin/public \
            --set-default STYLESHEET_PATH $out/bin/styles.css \
        '';
      });

      server-container = pkgs.dockerTools.buildLayeredImage {
        name = server-args.pname;
        tag = "latest";
        contents = [ server ];
        config = {
          Entrypoint = [ server-args.pname ];
          WorkingDir = "${server}/bin";
        };
      };
    in {
      devShells.default = pkgs.devshell.mkShell {
        packages = [
          (toolchain_fn pkgs)
          pkgs.gcc
          pkgs.tailwindcss_4
          pkgs.bacon
        ];
        motd = "\n  Welcome to the {2}${server-args.pname}{reset} shell.\n";
      };
      packages = {
        inherit server server-container;
        default = server;
      };
    });
}
