FILE(GLOB_RECURSE srcs ${CMAKE_SOURCE_DIR}/main/*.cpp)
FILE(GLOB_RECURSE include_dirs ${CMAKE_SOURCE_DIR}/include/*.h)

idf_component_register(
  SRCS ${srcs}
  INCLUDE_DIRS ${include_dirs}
#   REQUIRES espressif/esp_bsp_devkit espressif/usb_host_ch34x_vcp espressif/usb_host_cdc_acm espressif/usb_host_vcp espressif/esp_tinyusb espressif usb_host_cp210x_vcp espressif/usb_host_ftdi_vcp espressif/led_strip espressif/esp_bsp_generic
)

# # idf_component_register(SRCS "*.c"
# #                     INCLUDE_DIRS "."
# #                     )

# Make only this component build with modern C++ and exceptions
target_compile_features(${COMPONENT_LIB} PUBLIC cxx_std_17)   # or cxx_std_14
target_compile_options(${COMPONENT_LIB} PRIVATE -fexceptions)

# TODO IDF-10802
if (CMAKE_CXX_COMPILER_VERSION VERSION_LESS_EQUAL 13.2)  # https://gcc.gnu.org/bugzilla/show_bug.cgi?id=116070
    set_target_properties(${COMPONENT_LIB} PROPERTIES
        CXX_STANDARD 14
        CXX_STANDARD_REQUIRED ON
    )
    target_compile_options(${COMPONENT_LIB} PRIVATE -fconcepts)
endif()
