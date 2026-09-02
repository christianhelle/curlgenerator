# Default target
all: build

# Build both the Rust CLI and the legacy .NET solution
build:
	cargo build --workspace
	dotnet build --configuration Debug src/dotnet/CurlGenerator.sln

# Optimized build
release:
	cargo build --release --workspace
	dotnet build --configuration Release src/dotnet/CurlGenerator.sln

# Run every test suite
test:
	cargo test --workspace
	dotnet test --configuration Debug src/dotnet/CurlGenerator.sln

# Formatting and lints
lint:
	cargo fmt --all --check
	cargo clippy --workspace --all-targets -- -D warnings

# Packaging
publish:
	cargo publish --dry-run --allow-dirty
	dotnet pack --no-build src/dotnet/CurlGenerator.Core/CurlGenerator.Core.csproj
	dotnet pack --no-build src/dotnet/CurlGenerator/CurlGenerator.csproj

# Clean target
clean:
	cargo clean
	dotnet clean src/dotnet/CurlGenerator.sln

.PHONY: all build release test lint publish clean
