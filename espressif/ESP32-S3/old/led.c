#include "led.h"
#include "driver/gpio.h"
#include "esp_log.h"
#include "freertos/FreeRTOS.h"
#include "freertos/task.h"
#include "led_strip.h"
#include "sdkconfig.h"
#include <stddef.h>
#include <stdio.h>

static const char *TAG = "led.c";

uint8_t s_led_state = 0;

/* Use project configuration menu (idf.py menuconfig) to choose the GPIO to
   blink, or you can edit the following line and set a number here.
*/
#ifdef CONFIG_BLINK_LED_STRIP

static led_strip_handle_t led_strip;

#define LED_STRIP_LED_COUNT 1

void blink_led(void) {
  if (!s_led_state) {
    ESP_LOGI(TAG, "LED ON!");

    /* Set the LED pixel using RGB from 0 (0%) to 255 (100%) for each color */
    led_strip_set_pixel(
        led_strip, 0, 0, 0,
        100); // Blue = Blue Hex/RGB color code = #0000FF = 0*65536+0*256+255 =
              // (0,0,255), RED=0, GREEN=0, BLUE=255
    /* Refresh the strip to send data */
    // led_strip_refresh(led_strip);
    ESP_ERROR_CHECK(led_strip_refresh(led_strip));

    // /* Set the LED pixel using RGB from 0 (0%) to 255 (100%) for each color
    // */ for (int i = 0; i < LED_STRIP_LED_COUNT; i++) {
    //     ESP_ERROR_CHECK(led_strip_set_pixel(led_strip, i, 5, 5, 5));
    // }
    // /* Refresh the strip to send data */
    // ESP_ERROR_CHECK(led_strip_refresh(led_strip));
  } else {
    /* Set all LED off to clear all pixels */
    ESP_ERROR_CHECK(led_strip_clear(led_strip));
    ESP_LOGI(TAG, "LED OFF!");
  }

  s_led_state = !s_led_state;
  vTaskDelay(pdMS_TO_TICKS(500));
}

void configure_led(void) {
  ESP_LOGI(TAG, "Example configured to blink addressable LED!");
  /* LED strip initialization with the GPIO and pixels number*/
  led_strip_config_t strip_config = {
      .strip_gpio_num = CONFIG_BLINK_GPIO,
      .max_leds = 1, // at least one LED on board.
      .color_component_format = LED_STRIP_COLOR_COMPONENT_FMT_GRB,
  };
#if CONFIG_BLINK_LED_STRIP_BACKEND_RMT
  led_strip_rmt_config_t rmt_config = {
      .resolution_hz = 10 * 1000 * 1000, // 10MHz
      .flags.with_dma = false,
  };
  ESP_ERROR_CHECK(
      led_strip_new_rmt_device(&strip_config, &rmt_config, &led_strip));
#elif CONFIG_BLINK_LED_STRIP_BACKEND_SPI
  led_strip_spi_config_t spi_config = {
      .spi_bus = SPI2_HOST,
      .flags.with_dma = true,
  };
  ESP_ERROR_CHECK(
      led_strip_new_spi_device(&strip_config, &spi_config, &led_strip));
#else
#error "unsupported LED strip backend"
#endif
  /* Set all LED off to clear all pixels */
  led_strip_clear(led_strip);
}

#elif CONFIG_BLINK_LED_GPIO

void blink_led(void) {
  /* Set the GPIO level according to the state (LOW or HIGH)*/
  gpio_set_level(BLINK_GPIO, s_led_state);

  /* Toggle the LED state */
  s_led_state = !s_led_state;
}

void configure_led(void) {
  ESP_LOGI(TAG, "Example configured to blink GPIO LED!");
  gpio_reset_pin(BLINK_GPIO);
  /* Set the GPIO as a push/pull output */
  gpio_set_direction(BLINK_GPIO, GPIO_MODE_OUTPUT);
}

#else
#error "unsupported LED type"
#endif