#!/usr/bin/env sh
set -x

# export SSH_STAMP_PUBKEY="$(cat ~/.ssh/id_ed25519.pub)"
# ssh -i ~/.ssh/id_ed25519 -o SendEnv=SSH_STAMP_PUBKEY root@192.168.4.1
# export SSH_STAMP_WIFI_AP_SSID="OpenWrt"
# export SSH_STAMP_WIFI_AP_PSK="1234567890"
# ssh -o SendEnv=SSH_STAMP_WIFI_AP_SSID -o SendEnv=SSH_STAMP_WIFI_AP_PSK root@192.168.4.1
export SSH_STAMP_UART_BAUD="115200"
export SSH_STAMP_UART_DATA_BITS="8"
export SSH_STAMP_UART_PARITY="none"
export SSH_STAMP_UART_STOP_BITS="1"
ssh -o SendEnv='SSH_STAMP_UART_*' root@192.168.4.1
