idf_component_register(SRCS "usb_cdc.c" "led.c" "generic_button_led.c" "main.c"
                    REQUIRES "usb"
                    INCLUDE_DIRS "."
                    )
