#!/bin/bash

set -e

action=$1
devname=$2
device="/dev/${devname}"

# Change this to the user running the photobooth application
user="photobooth"

mount_point=$(/bin/mount | /bin/grep -e "^${device}" | awk '{ print $3 }')

case "${action}" in
  "mount")
    if [[ -n "${mount_point}" ]]; then
      echo "[I] ${device} already mounted on ${mount_point}"
      exit 0
    fi

    uid=$(id -u "${user}")
    gid=$(id -g "${user}")

    label=$(/sbin/blkid -o value -s LABEL "${device}")
    if [[ -z "${label}" ]]; then
      mount_point="/mnt/${devname}"
    else
      mount_point="/mnt/${label}"
    fi

    echo "[I] Mounting ${device} to ${mount_point}"

    /bin/mkdir -p "${mount_point}"

    fstype=$(/sbin/blkid -o value -s TYPE "${device}")
    if [[ "${fstype}" == "vfat" || "${fstype}" == "exfat" || "${fstype}" == "ntfs" ]]; then
      /bin/mount -o nosuid,nodev,nofail,uid=${uid},gid=${gid},umask=0022 "${device}" "${mount_point}"
    else
      /bin/mount -o nosuid,nodev,nofail "${device}" "${mount_point}"

      /bin/chown "${uid}:${gid}" "${mount_point}"
    fi
  ;;
  "umount")
    if [[ -z "${mount_point}" ]]; then
      echo "[I] ${device} is not mounted"
      exit 0
    fi

    /bin/umount -l "${device}"
    /bin/rm -r "${mount_point}"
  ;;
esac
