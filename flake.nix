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
          # css drv is built with this src
          (lib.fileset.maybeMissing ./style)
          # gets byte-included into binary
          (lib.fileset.maybeMissing ./assets/favicon.svg)
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
        # has to be built with package src bc it needs the rust source for classes
        inherit src;

        buildPhase = ''
          ${pkgs.tailwindcss_4}/bin/tailwindcss \
            --input style/main.css \
            --output $out \
            --minify
        '';
      };
      
      server-binary = craneLib.buildPackage (server-args // {
        pname = server-args.pname + "-binary";
        cargoArtifacts = craneLib.buildDepsOnly server-args;
      });

      server = pkgs.stdenv.mkDerivation {
        inherit (server-args) pname version;
        src = server-binary;
        
        nativeBuildInputs = (with pkgs; [
          makeWrapper
        ]);

        buildPhase = "";
        installPhase = ''
          mkdir $out

          # copy everything from the drv src, which is the binary
          cp -r * $out
          # copy css from the css drv
          cp ${css} $out/bin/styles.css
          # copy assets and content from the *original src*
          cp -r ${./assets} $out/bin/assets
          cp -r ${./content} $out/bin/content

          # wrap with default env vars
          wrapProgram $out/bin/${server-args.pname} \
            --set-default STATIC_ASSET_DIR $out/bin/assets \
            --set-default POSTS_DIR $out/bin/content/posts \
            --set-default TIDBITS_DIR $out/bin/content/tidbits \
            --set-default STYLESHEET_PATH $out/bin/styles.css \
        '';
      };

      server-container = pkgs.dockerTools.buildLayeredImage {
        name = server-args.pname;
        tag = "latest";
        contents = [ server ];
        config = {
          Entrypoint = [ server-args.pname ];
          WorkingDir = "${server}/bin";
        };
      };

      tailwind-command = {
        help = "runs tailwind in watch mode";
        name = "tailwind-watch";
        command = ''
          ${pkgs.tailwindcss_4}/bin/tailwindcss \
            --input $PRJ_ROOT/style/main.css \
            --output $STYLESHEET_PATH \
            --watch
        '';
      };

      devShellPkgs = with pkgs; [
        (toolchain_fn pkgs) gcc tailwindcss_4 bacon flyctl
      ];
      linuxDevShell = pkgs.devshell.mkShell {
        packages = devShellPkgs;
        motd = "\n  Welcome to the {2}$(basename $PRJ_ROOT){reset} shell.\n";
        commands = [ tailwind-command ];
      };
      darwinDevShell = pkgs.mkShell {
        nativeBuildInputs = devShellPkgs ++ [ pkgs.libiconv ];
      };
    in {
      devShells.default = if pkgs.stdenv.isDarwin
        then darwinDevShell
        else linuxDevShell;
      packages = {
        inherit server server-binary server-container;
        default = server;
      };
    });
}
