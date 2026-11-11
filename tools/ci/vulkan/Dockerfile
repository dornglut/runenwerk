ARG UBUNTU_BASE=ubuntu:24.04
FROM ${UBUNTU_BASE}
ARG UBUNTU_BASE

ARG APT_SNAPSHOT=20261108T000000Z
ARG RUST_VERSION=1.99.1

LABEL org.opencontainers.image.source="https://github.com/dornglut/runenwerk" \
      org.opencontainers.image.title="Runenwerk Vulkan CI environment" \
      org.opencontainers.image.description="Versioned software Vulkan and X11 test environment" \
      org.opencontainers.image.base.name="${UBUNTU_BASE}"

ENV DEBIAN_FRONTEND=noninteractive \
    CARGO_HOME=/opt/rust/cargo \
    RUSTUP_HOME=/opt/rust/rustup \
    PATH=/opt/rust/cargo/bin:$PATH \
    VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json \
    VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json \
    WGPU_BACKEND=vulkan \
    LIBGL_ALWAYS_SOFTWARE=1 \
    XDG_RUNTIME_DIR=/tmp/runenwerk-vulkan-xdg

# One explicit historical package snapshot for every dependency.
# Failure blocks the image build, not a normal CI consumer.
RUN set -eux; \
    printf 'APT::Snapshot "%s";\n' "$APT_SNAPSHOT" > /etc/apt/apt.conf.d/50-runenwerk-snapshot; \
    apt-get update --error-on=any; \
    apt-get install --yes --no-install-recommends \
      ca-certificates curl git openssh-client \
      build-essential clang cmake ninja-build pkg-config \
      python3 tar gzip xz-utils unzip \
      libssl-dev libudev-dev libasound2-dev \
      libvulkan1 libvulkan-dev mesa-vulkan-drivers vulkan-tools \
      xvfb xauth xdotool x11-utils \
      libx11-dev libx11-xcb-dev libxcb1-dev libxcb-randr0-dev libxcb-xfixes0-dev \
      libxkbcommon-dev libxkbcommon-x11-0 libxrandr-dev libxi-dev libxinerama-dev \
      libxcursor-dev libwayland-dev libegl1-mesa-dev libgl1-mesa-dev libdrm-dev; \
    rm -rf /var/lib/apt/lists/*; \
    mkdir -p "$CARGO_HOME" "$RUSTUP_HOME" "$XDG_RUNTIME_DIR"; \
    chmod 700 "$XDG_RUNTIME_DIR"

# Existing proofs use cargo +stable. Fail if stable moved since reviewed version.
RUN set -eux; \
    curl --fail --location --retry 3 --proto '=https' --tlsv1.2 https://sh.rustup.rs \
      --output /tmp/runenwerk-rustup.sh; \
    sh /tmp/runenwerk-rustup.sh -y --profile minimal --default-toolchain stable; \
    rm /tmp/runenwerk-rustup.sh; \
    rustc +stable --version | grep -E "^rustc $RUST_VERSION "; \
    cargo +stable --version; \
    test -f "$VK_DRIVER_FILES"

COPY smoke-image.sh /usr/local/bin/runenwerk-image-smoke
RUN chmod 755 /usr/local/bin/runenwerk-image-smoke && \
    /usr/local/bin/runenwerk-image-smoke

WORKDIR /workspace
