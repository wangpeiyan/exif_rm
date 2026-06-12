FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive

# System dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    ca-certificates \
    cmake \
    curl \
    git \
    openjdk-21-jdk \
    python3 \
    unzip \
    && update-ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Android SDK
ENV ANDROID_HOME=/opt/android-sdk
ENV ANDROID_NDK_HOME=${ANDROID_HOME}/ndk/30.0.14904198
ENV PATH="${ANDROID_HOME}/cmdline-tools/latest/bin:${ANDROID_HOME}/platform-tools:${PATH}"

RUN mkdir -p ${ANDROID_HOME}/cmdline-tools && \
    curl -fsSL https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip -o /tmp/cmdline-tools.zip && \
    unzip -q /tmp/cmdline-tools.zip -d ${ANDROID_HOME}/cmdline-tools && \
    mv ${ANDROID_HOME}/cmdline-tools/cmdline-tools ${ANDROID_HOME}/cmdline-tools/latest && \
    rm /tmp/cmdline-tools.zip

RUN yes | sdkmanager --licenses > /dev/null 2>&1 && \
    sdkmanager --install \
    "platform-tools" \
    "build-tools;36.0.0" \
    "platforms;android-36" \
    "ndk;30.0.14904198"

# Rust toolchain
ENV PATH="/root/.cargo/bin:${PATH}"
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    rustup target add aarch64-linux-android armv7-linux-androideabi && \
    cargo install cargo-ndk --version 4.1.2

# Project
COPY . /workspace/exif_rm
WORKDIR /workspace/exif_rm

CMD ["bash", "-c", "cargo build --release && scripts/build-android.sh && cd android && ./gradlew :library:assembleRelease && cp android/library/build/outputs/aar/library-release.aar /output/library-release.aar"]
