#!/usr/bin/env bash

source "$(dirname "$(which "$0")")/common.sh"

build_mountpoint

FSTAB_CONTENT="
${MOUNTPOINT_PATH}#${S3_BUCKET_NAME} /mnt/mountpoint fuse rw,allow-delete,allow-other,_netdev,nosuid,nodev
"

spawn_mounts "$FSTAB_CONTENT"

ls -l /mnt/mountpoint/

echo "data" | sudo tee /mnt/mountpoint/data

if ! grep -q 'data' /mnt/mountpoint/data
then
  echo "Data file does not contain correct data"
  exit 1
fi
sudo rm /mnt/mountpoint/data
