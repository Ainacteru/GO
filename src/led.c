#include <atmel_start.h>
#include "include/led.h"

void blink(void) {
        gpio_toggle_pin_level(RGB_GREEN);
        delay_ms(500);
        gpio_toggle_pin_level(RGB_GREEN);
        delay_ms(500);
}