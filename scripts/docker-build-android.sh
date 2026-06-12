#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="${PROJECT_DIR}/output"
IMAGE_NAME="exif-rm-android"

mkdir -p "${OUTPUT_DIR}"

echo "Building Docker image..."
docker build -t "${IMAGE_NAME}" "${PROJECT_DIR}"

echo "Running AAR build in container..."
# --user avoids root-owned output files locally; skip on CI where it breaks PATH access
USER_FLAG=""
if [ -z "${CI:-}" ]; then
    USER_FLAG="-u $(id -u):$(id -g)"
fi

docker run --rm ${USER_FLAG} -v "${OUTPUT_DIR}:/output" "${IMAGE_NAME}"

echo "Done! AAR written to ${OUTPUT_DIR}/library-release.aar"
