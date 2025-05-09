if [ -z "${!FSTAB_CONTENT}" ]; then
    echo "Need to set environment var $VARIABLE" && exit 1;
else

cargo build --bin mount-s3 --release --features=fstab

export SYSTEMD_FSTAB=$(mktemp)
export SYSTEMD_PROC_CMDLINE=""

MOUNTPOINT_PATH=$(pwd)/target/release/mount-s3
OUTPUT_DIR=./out
GENERATOR_BIN="/usr/lib/systemd/system-generators/systemd-fstab-generator"
SYSTEMD_MOUNT_UNIT=/etc/systemd/system/mnt-mountpoint.mount

echo "Mountpoint path: $MOUNTPOINT_PATH"

cat > "$SYSTEMD_FSTAB" <<- EOM
${MOUNTPOINT_PATH}#${S3_BUCKET_NAME} /mnt/mountpoint fuse rw,allow-delete,allow-other,_netdev,nosuid,nodev
EOM

exit 0

echo "fstab file:"
cat "$SYSTEMD_FSTAB"

rm -r $OUTPUT_DIR || true
mkdir -p "$OUTPUT_DIR"/{normal,early,late}

$GENERATOR_BIN "$OUTPUT_DIR/normal" "$OUTPUT_DIR/early" "$OUTPUT_DIR/late"

MOUNT_UNIT="$OUTPUT_DIR/normal/mnt-mountpoint.mount"
cat "$MOUNT_UNIT"

sudo cp $MOUNT_UNIT $SYSTEMD_MOUNT_UNIT
sudo systemctl daemon-reload
sudo systemctl start mnt-mountpoint.mount


echo "\nStatus of systemd unit:"
sudo systemctl status mnt-mountpoint.mount | cat

function cleanup {
  sudo umount /mnt/mountpoint
  sudo systemctl stop mnt-mountpoint.mount
  rm "$SYSTEMD_FSTAB"
  sudo rm "$SYSTEMD_MOUNT_UNIT"
}

trap cleanup EXIT

echo -e "Mounted!\n\n"

ls -l /mnt/mountpoint/

echo "data" | sudo tee /mnt/mountpoint/data

if ! grep -q 'data' /mnt/mountpoint/data
then
  echo "Data file does not contain correct data"
  exit 1
fi
sudo rm /mnt/mountpoint/data
