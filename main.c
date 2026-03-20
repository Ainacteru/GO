#include "include/led.h"
#include <atmel_start.h>


int main(void)
{
	/* Initializes MCU, drivers and middleware */
	atmel_start_init();
	delay_ms(2000);

	for(int i =0; i<20;i++) {
		blink();
	}
}
