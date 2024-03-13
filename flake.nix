
{
  inputs = {
    nixpkgs.url = "https://flakehub.com/f/NixOS/nixpkgs/0.2311.556557.tar.gz";
    rust-overlay.url = "https://flakehub.com/f/oxalica/rust-overlay/0.1.1271.tar.gz";
    crane.url = "https://flakehub.com/f/ipetkov/crane/0.16.1.tar.gz";
    cargo-leptos-src = { url = "github:leptos-rs/cargo-leptos"; flake = false; };
  };

  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, cargo-leptos-src }:
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

        # Only keeps markdown files
				filterGenerator = pattern: path: _type: builtins.match pattern path != null;
				cssFilter = filterGenerator ".*css$";
        jsFilter = filterGenerator ".*js$";
				ttfFilter = filterGenerator ".*ttf$";
				woff2Filter = filterGenerator ".*woff2$";
				webpFilter = filterGenerator ".*webp$";
				jpegFilter = filterGenerator ".*jpeg$";
				pngFilter = filterGenerator ".*png$";
				icoFilter = filterGenerator ".*ico$";
        protoOrCargo = path: type:
          (craneLib.filterCargoSources path type) || (cssFilter path type) || (jsFilter path type) || (ttfFilter path type) || (woff2Filter path type) || (webpFilter path type) || (jpegFilter path type) || (pngFilter path type) || (icoFilter path type);

        # Include more types of files in our bundle
        src = lib.cleanSourceWith {
          src = ./.; # The original, unfiltered source
          filter = protoOrCargo;
        };

        cargo-leptos = (import ./nix/cargo-leptos.nix) {
          inherit pkgs craneLib;
          cargo-leptos = cargo-leptos-src;
        };

        # Common arguments can be set here
        common_args = {
          inherit src;

          pname = "server";
					version = "0.1.0";

          nativeBuildInputs = [
            # Add additional build inputs here
            cargo-leptos
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
              "LEPTOS_SITE_ROOT=blog"
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
