
{
  description = "Build the Leptos Website for !";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";

    crane = {
      url = "github:ipetkov/crane";
      inputs.nixpkgs.follows = "nixpkgs";
    };

    flake-utils.url = "github:numtide/flake-utils";
    
    cargo-leptos = {
      #url= "github:leptos-rs/cargo-leptos/v1.7";
      url = "github:benwis/cargo-leptos";
      flake = false;
    };

    rust-overlay = {
      url = "github:oxalica/rust-overlay";
      inputs = {
        nixpkgs.follows = "nixpkgs";
        flake-utils.follows = "flake-utils";
      };
    };
  };

  outputs = { self, nixpkgs, crane, flake-utils, rust-overlay, ... } @inputs:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ (import rust-overlay) ];
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
				ttfFilter = filterGenerator ".*ttf$";
				woff2Filter = filterGenerator ".*woff2$";
				webpFilter = filterGenerator ".*webp$";
				jpegFilter = filterGenerator ".*jpeg$";
				pngFilter = filterGenerator ".*png$";
				icoFilter = filterGenerator ".*ico$";
        protoOrCargo = path: type:
          (craneLib.filterCargoSources path type) || (cssFilter path type) || (ttfFilter path type) || (woff2Filter path type) || (webpFilter path type) || (jpegFilter path type) || (pngFilter path type) || (icoFilter path type);

        # Include more types of files in our bundle
        src = lib.cleanSourceWith {
          src = ./.; # The original, unfiltered source
          filter = protoOrCargo;
        };

        # Common arguments can be set here
        commonArgs = {
          inherit src;

          pname = "server";
					version = "0.1.0";

          buildInputs = [
            # Add additional build inputs here
            cargo-leptos
            pkgs.pkg-config
            pkgs.openssl
            pkgs.cargo-generate
            pkgs.binaryen
            pkgs.dart-sass
            pkgs.clang
          ] ++ lib.optionals pkgs.stdenv.isDarwin [
            # Additional darwin specific inputs can be set here
            pkgs.libiconv
          ];
        };

        # Build *just* the cargo dependencies, so we can reuse
        # all of that work (e.g. via cachix) when running in CI
        cargoArtifacts = craneLib.buildDepsOnly (commonArgs // {
          doCheck = false;
        });

        # Build the actual crate itself, reusing the dependency
        # artifacts from above.
        blog = craneLib.buildPackage (commonArgs // {
          buildPhaseCargoCommand = "cargo leptos build --release -vvv";
          installPhaseCommand = ''
            mkdir -p $out/bin
            cp target/release/server $out/bin/blog
            cp -r target/site $out/bin/
          '';
          # Prevent cargo test and nextest from duplicating tests
          doCheck = false;
          inherit cargoArtifacts;

          SQLX_OFFLINE = "true";
          # LEPTOS_BIN_TARGET_TRIPLE = "x86_64-unknown-linux-gnu"; # Adding this allows -Zbuild-std to work and shave 100kb off the WASM
          LEPTOS_BIN_PROFILE_RELEASE = "release";
          LEPTOS_LIB_PROFILE_RELEASE = "release-wasm-size";
          APP_ENVIRONMENT = "production";
        });
        
        cargo-leptos = pkgs.rustPlatform.buildRustPackage rec {
          pname = "cargo-leptos";
          version = "0.1.8.1";
          buildFeatures = ["no_downloads"]; # cargo-leptos will try to download Ruby and other things without this feature

          src = inputs.cargo-leptos; 

          cargoSha256 = "sha256-XgKr1XLGHtCZbc4ZQJuko4dsJPl+hWmsIBex62tKEJ8=";
          # cargoSha256 = "";

          nativeBuildInputs = [pkgs.pkg-config pkgs.openssl];

          buildInputs = with pkgs;
            [openssl pkg-config]
            ++ lib.optionals stdenv.isDarwin [
            darwin.Security darwin.apple_sdk.frameworks.CoreServices darwin.apple_sdk.frameworks.SystemConfiguration
          ];

          doCheck = false; # integration tests depend on changing cargo config

          meta = with lib; {
            description = "A build tool for the Leptos web framework";
            homepage = "https://github.com/leptos-rs/cargo-leptos";
            changelog = "https://github.com/leptos-rs/cargo-leptos/blob/v${version}/CHANGELOG.md";
            license = with licenses; [mit];
            maintainers = with maintainers; [benwis];
          };
	      };

        flyConfig = ./fly.toml;

        # Deploy the image to Fly with our own bash script
        flyDeploy = pkgs.writeShellScriptBin "flyDeploy" ''
          OUT_PATH=$(nix build --print-out-paths .#container)
          HASH=$(echo $OUT_PATH | grep -Po "(?<=store\/)(.*?)(?=-)")
          ${pkgs.skopeo}/bin/skopeo --insecure-policy --debug copy docker-archive:"$OUT_PATH" docker://registry.fly.io/$FLY_PROJECT_NAME:$HASH --dest-creds x:"$FLY_AUTH_TOKEN" --format v2s2
          ${pkgs.flyctl}/bin/flyctl deploy -i registry.fly.io/$FLY_PROJECT_NAME:$HASH -c ${flyConfig} --remote-only
        '';

      in {
        checks = {
          # Build the crate as part of `nix flake check` for convenience
          inherit blog;

          # Run clippy (and deny all warnings) on the crate source,
          # again, resuing the dependency artifacts from above.
          blog-clippy = craneLib.cargoClippy (commonArgs // {
            inherit cargoArtifacts;
            cargoClippyExtraArgs = "--all-targets -- --deny warnings";
          });

          blog-doc = craneLib.cargoDoc (commonArgs //{
            inherit cargoArtifacts;
          });

          # Check formatting
          blog-fmt = craneLib.cargoFmt (commonArgs //{
            inherit src;
          });
        };

        packages.blog = blog;
        packages.default = blog;

        apps.default = flake-utils.lib.mkApp {
          drv = blog;
        };

        # only create container if the system is x86_64-linux
        packages.container = pkgs.dockerTools.buildImage {
          name = "blog";
          created = "now";
          tag = "latest";

          copyToRoot = pkgs.buildEnv {
            name = "image-root";
            paths = [ pkgs.cacert ./.  ];
          };
          config = {
            Env = [ "PATH=${blog}/bin" "APP_ENVIRONMENT=production" "LEPTOS_OUTPUT_NAME=blog" "LEPTOS_SITE_ADDR=0.0.0.0:3000" "LEPTOS_SITE_ROOT=${blog}/bin/site" ];

            ExposedPorts = {
              "3000/tcp" = { };
            };

            Cmd = [ "${blog}/bin/blog" ];
          };
        };

        apps.flyDeploy = flake-utils.lib.mkApp {
          drv = flyDeploy;
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = builtins.attrValues self.checks;

          # Extra inputs can be added here
          nativeBuildInputs = with pkgs; [
            toolchain
            openssl
            dive
            wasm-pack
            pkg-config
            binaryen
            tailwindcss
            cargo-leptos
            dart-sass
            flyctl
            skopeo
          ];
          RUST_SRC_PATH = "${toolchain}/lib/rustlib/src/rust/library";
        };
      });
}
