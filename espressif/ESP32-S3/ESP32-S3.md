# `ESP32-S3`

```sh
export PATH="$IDF_PATH/tools:$PATH"
```

```sh
pushd  ~/.espressif/v6.1/esp-idf
. ./export.sh
popd

```

```sh
. ~/.espressif/tools/activate_idf_v6.1.sh
idf.py set-target esp32s3
idf.py -p /dev/serial/by-id/ flash monitor
```

```sh
source ~/.espressif/tools/activate_idf_v6.1.sh
```

<https://circuitpython.org/board/yd_esp32_s3_n16r8/>

---

<https://components.espressif.com/components/espressif/iot_usbh_cdc/versions/3.1.0/examples/usb_cdc_basic?language=en>.
See `usb_cdc_basic`.
