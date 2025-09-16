#!/bin/bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

usage() {
    echo "Usage: $0 <ubuntu_version>"
    echo ""
    echo "Examples:"
    echo "  $0 20.04    # Test on Ubuntu 20.04"
    echo "  $0 22.04    # Test on Ubuntu 22.04"
    echo "  $0 24.04    # Test on Ubuntu 24.04"
    echo ""
    echo "This script will:"
    echo "  1. Build a .deb package if not already present"
    echo "  2. Create a Docker container with the specified Ubuntu version"
    echo "  3. Install the .deb package in the container"
    echo "  4. Test the clifx shine command"
    exit 1
}

check_prerequisites() {
    local required_commands=("docker" "cargo")
    
    for cmd in "${required_commands[@]}"; do
        if ! command -v "$cmd" &> /dev/null; then
            log_error "Required command '$cmd' not found"
            exit 1
        fi
    done
    
    log_success "Prerequisites check passed"
}

get_arch() {
    local arch=$(uname -m)
    case $arch in
        x86_64)
            echo "amd64"
            ;;
        aarch64|arm64)
            echo "arm64"
            ;;
        *)
            log_error "Unsupported architecture: $arch"
            exit 1
            ;;
    esac
}

get_version() {
    cd "$PROJECT_ROOT"
    grep '^version = ' Cargo.toml | head -1 | sed 's/version = "\(.*\)"/\1/'
}

build_deb_if_needed() {
    local version=$(get_version)
    local arch=$(get_arch)
    local target="x86_64-unknown-linux-musl"
    
    # Find existing deb file with version pattern
    local existing_deb=$(find "$PROJECT_ROOT/target/*/debian" -name "clifx_${version}*_${arch}.deb" 2>/dev/null | head -1)
    
    if [[ -n "$existing_deb" && -f "$existing_deb" ]]; then
        log_info "Debian package already exists: $existing_deb" >&2
        echo "$existing_deb"
        return 0
    fi
    
    log_info "Building Debian package..." >&2
    cd "$PROJECT_ROOT"
    
    # Install cargo-deb if not present
    if ! cargo deb --version &>/dev/null; then
        log_info "Installing cargo-deb..." >&2
        cargo install cargo-deb
    fi
    

    # Build the deb package
    #

    RUSTFLAGS="-C target-feature=-crt-static" cargo deb --no-strip --target $target   >&2
    
    # Find the newly created deb file
    local deb_path=$(find "$PROJECT_ROOT/target/${target}/debian" -name "clifx_${version}*_${arch}.deb" 2>/dev/null | head -1)
    
    if [[ -z "$deb_path" || ! -f "$deb_path" ]]; then
        log_error "Failed to build Debian package" >&2
        exit 1
    fi
    
    log_success "Built Debian package: $deb_path" >&2
    echo "$deb_path" | head -1
}

create_dockerfile() {
    local ubuntu_version="$1"
    local dockerfile_content="FROM ubuntu:$ubuntu_version

# Install dependencies
RUN apt-get update && apt-get install -y \\
    ca-certificates \\
    && rm -rf /var/lib/apt/lists/*

# Copy the deb file
COPY *.deb /tmp/

# Install the deb package
RUN dpkg -i /tmp/*.deb || apt-get install -f -y

# Create a test script
RUN echo '#!/bin/bash' > /test.sh && \\
    echo 'echo \"Testing clifx shine command...\"' >> /test.sh && \\
    echo 'echo \"Hello, World!\" | clifx shine --color 255,0,0 --speed 50 --duration 1000' >> /test.sh && \\
    echo 'echo' >> /test.sh && \\
    echo 'echo \"All tests completed successfully!\"' >> /test.sh && \\
    chmod +x /test.sh

CMD [\"/test.sh\"]"
    
    echo "$dockerfile_content"
}

test_in_docker() {
    local ubuntu_version="$1"
    local deb_path="$2"
    local temp_dir=$(mktemp -d)
    
    # Cleanup on exit
    trap "rm -rf '$temp_dir'" EXIT
    
    log_info "Creating Docker test environment for Ubuntu $ubuntu_version"
    
    # Copy deb to temp directory
    cp "$deb_path" "$temp_dir/"
    
    # Create Dockerfile
    create_dockerfile "$ubuntu_version" > "$temp_dir/Dockerfile"
    
    # Build Docker image
    local image_name="clifx-test-ubuntu-$ubuntu_version"
    log_info "Building Docker image: $image_name"
    
    cd "$temp_dir"
    if ! docker build -t "$image_name" .; then
        log_error "Failed to build Docker image"
        exit 1
    fi
    
    # Run the test container
    log_info "Running tests in Docker container..."
    echo "----------------------------------------"
    
    if docker run --rm "$image_name"; then
        echo "----------------------------------------"
        log_success "All tests passed on Ubuntu $ubuntu_version!"
    else
        echo "----------------------------------------"
        log_error "Tests failed on Ubuntu $ubuntu_version"
        exit 1
    fi
    
    # Clean up Docker image
    log_info "Cleaning up Docker image..."
    docker rmi "$image_name" >/dev/null 2>&1 || true
}

main() {
    if [[ $# -ne 1 ]]; then
        usage
    fi
    
    local ubuntu_version="$1"
    
    # Validate Ubuntu version format
    if ! [[ "$ubuntu_version" =~ ^[0-9]+\.[0-9]+$ ]]; then
        log_error "Invalid Ubuntu version format. Expected: XX.YY (e.g., 20.04, 22.04)"
        exit 1
    fi
    
    log_info "Starting Ubuntu $ubuntu_version integration test"
    
    check_prerequisites
    
    # Build deb if needed
    local deb_path=$(build_deb_if_needed)
    
    # Test in Docker
    test_in_docker "$ubuntu_version" "$deb_path"
    
    log_success "Integration test completed successfully!"
}

main "$@"
