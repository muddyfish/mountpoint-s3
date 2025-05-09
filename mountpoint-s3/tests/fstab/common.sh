function build_mountpoint() {
  cargo build --bin mount-s3 --release --features=fstab
  MOUNTPOINT_ROOT=$(dirname "$(which "$0")")/../../..
  MOUNTPOINT_PATH=$MOUNTPOINT_ROOT/target/release/mount-s3
  echo "Mountpoint path: $MOUNTPOINT_PATH"
}

function spawn_mounts() {
  FSTAB_CONTENT=$1
  export SYSTEMD_FSTAB=$(mktemp)
  export SYSTEMD_PROC_CMDLINE=""

  OUTPUT_DIR="$MOUNTPOINT_ROOT/out"
  GENERATOR_BIN="/usr/lib/systemd/system-generators/systemd-fstab-generator"
  SYSTEMD_MOUNT_DIR=/etc/systemd/system/
  UNIT_SOURCE_DIR=$OUTPUT_DIR/normal

  echo "$FSTAB_CONTENT" > "$SYSTEMD_FSTAB"

  rm -r "$OUTPUT_DIR" || true
  mkdir -p "$OUTPUT_DIR"/{normal,early,late}

  $GENERATOR_BIN "$UNIT_SOURCE_DIR" "$OUTPUT_DIR/early" "$OUTPUT_DIR/late"

  sudo cp -r "$UNIT_SOURCE_DIR" "$SYSTEMD_MOUNT_DIR"
  sudo systemctl daemon-reload

  for unit in "$UNIT_SOURCE_DIR"/*; do
    unit=$(basename $unit)
    sudo systemctl start "$unit"

    echo -e "\nStatus of systemd unit $unit:"
    sudo systemctl status "$unit" | cat
  done

  trap cleanup EXIT
}

function cleanup {
  rm "$SYSTEMD_FSTAB"
  for unit in "$UNIT_SOURCE_DIR"/*; do
    sudo systemctl stop "$(basename "$unit")"
    sudo rm "$unit"
  done
}
