
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    rust-overlay = {
      url = "https://flakehub.com/f/oxalica/rust-overlay/0.1.tar.gz";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane.url = "https://flakehub.com/f/ipetkov/crane/0.17.tar.gz";
  };

  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };

        toolchain = pkgs.rust-bin.selectLatestNightlyWith (toolchain: toolchain.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
          targets = [ "wasm32-unknown-unknown" ];
        });

        inherit (pkgs) lib;
        craneLib = (crane.mkLib pkgs).overrideToolchain toolchain;

        src = ./.;

        # Common arguments can be set here
        common_args = {
          inherit src;

          pname = "server";
					version = "0.1.0";

          nativeBuildInputs = [
            # Add additional build inputs here
            pkgs.cargo-leptos
            pkgs.cargo-generate
            pkgs.binaryen
            pkgs.dart-sass
            pkgs.tailwindcss
            pkgs.clang
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
          ];

          buildInputs = [
            pkgs.pkg-config
            pkgs.openssl
          ];
        };

        blog-deps = craneLib.buildDepsOnly (common_args // {
          # if work is duplicated by the `server-site` package, update these
          # commands from the logs of `cargo leptos build --release -vvv`
          buildPhaseCargoCommand = ''
            # build the server dependencies
            cargo build --package=server --no-default-features --release
            # build the frontend dependencies
            cargo build --package=frontend --lib --target-dir=/build/source/target/front --target=wasm32-unknown-unknown --no-default-features --profile=wasm-release
          '';
        });

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        blog = craneLib.buildPackage (common_args // {
          buildPhaseCargoCommand = "cargo leptos build --release -vvv";
          installPhaseCommand = ''
            mkdir -p $out/bin
            cp target/release/server $out/bin/blog
            cp -r target/site $out/bin/
            cp target/release/hash.txt $out/bin/
            cp -r content $out/bin/
          '';
          # Prevent cargo test and nextest from duplicating tests
          doCheck = false;
          cargoArtifacts = blog-deps;
        });

      in {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit blog;

          blog-clippy = craneLib.cargoClippy (common_args // {
            cargoArtifacts = blog-deps;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });
        };

        packages.blog = blog;
        packages.default = blog;

        apps.default = flake-utils.lib.mkApp {
          drv = blog;
        };

        # only create container if the system is x86_64-linux
        packages.container = pkgs.dockerTools.buildLayeredImage {
          name = "blog";
          tag = "latest";

          contents = [ blog pkgs.cacert ];
          config = {
            Cmd = [ "blog" ];
            WorkingDir = "${blog}/bin";
            Env = [
              "LEPTOS_OUTPUT_NAME=blog"
              "LEPTOS_SITE_ROOT=site"
              "LEPTOS_SITE_PKG_DIR=pkg"
              "LEPTOS_SITE_ADDR=0.0.0.0:3000"
              "LEPTOS_RELOAD_PORT=3001"
              "LEPTOS_ENV=PROD"
              "LEPTOS_HASH_FILES=true"
            ];
          };
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.checks;

          # Extra inputs can be added here
          nativeBuildInputs =
            common_args.buildInputs ++
            common_args.nativeBuildInputs ++
            (with pkgs; [
              toolchain
              dive # docker images
              cargo-leptos
              flyctl
              skopeo # docker registries
              bacon # cargo check w/ hot reload
              marksman # markdown lsp
            ]);
        };
      });
}
