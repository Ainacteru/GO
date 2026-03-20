#include <atmel_start.h>
#include "include/led.h"

int main(void)
{
	/* Initializes MCU, drivers and middleware */
	atmel_start_init();

	/* Replace with your application code */
	for(int i = 0; i<10;) {
		blink();
	}
}
