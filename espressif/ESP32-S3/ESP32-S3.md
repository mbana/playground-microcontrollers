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

---

`cherryusb_host` is the one that is working:

```sh
[I/usbh_hub] New full-speed device on Bus 0, Hub 1, Port 1 connected
[I/usbh_core] New device found,idVendor:04d9,idProduct:b534,bcdDevice:0210
[I/usbh_core] The device has 1 bNumConfigurations
[I/usbh_core] The device has 3 interfaces
[I/usbh_core] Enumeration success, start loading class driver
[I/usbh_core] Loading cdc_acm class driver
[I/usbh_cdc_acm] Ep=02 Attr=02 Mps=64 Interval=00 Mult=00
[I/usbh_cdc_acm] Ep=83 Attr=02 Mps=64 Interval=00 Mult=00
[I/usbh_cdc_acm] Register CDC ACM Class:/dev/ttyACM0
[I/usbh_core] Loading cdc_data class driver
[I/usbh_core] Loading hid class driver
I (2[W/usbh_hid] Do not support set idle
144) cdc_acm.c[I/usbh_hid] Register HID Class:/dev/input0
: Data received
I (2164) hid.c: intf 2, SubClass 0, Protocol 0
I (2164) cdc_acm.c: 0x3fceb02c   43                                                |C|
W (2174) hid.c: no intin ep desc
I (2184) cdc_acm.c: Data received
I (2184) cdc_acm.c: 0x3fceb02d   44 43 3a 20 48 65 6c 6c  6f 2c 20 77 6f 72 6c 64  |DC: Hello, world|
I (2194) cdc_acm.c: 0x3fceb03d   21 0d 0a 2d 61 73 68 3a  20 43 44 43 3a 3a 20 6e  |!..-ash: CDC:: n|
I (2204) cdc_acm.c: 0x3fceb04d   6f 74 20 66 6f 75 6e 64  0d 0a 5e 40 72 6f 6f 74  |ot found..^@root|
I (2214) cdc_acm.c: 0x3fceb05d   40 4f 70 65 6e 57 72 74  3a 7e 23 20 0d 0a 72 6f  |@OpenWrt:~# ..ro|
I (2214) cdc_acm.c: 0x3fceb06d   6f 74 40 4f 70 65 6e 57  72 74 3a 7e 23 20        |ot@OpenWrt:~# |


```