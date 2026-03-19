/*
 * Code generated from Atmel Start.
 *
 * This file will be overwritten when reconfiguring your Atmel Start project.
 * Please copy examples or other code you want to keep to a separate file
 * to avoid losing it when reconfiguring.
 */
#ifndef ATMEL_START_PINS_H_INCLUDED
#define ATMEL_START_PINS_H_INCLUDED

#include <hal_gpio.h>

// SAMD21 has 8 pin functions

#define GPIO_PIN_FUNCTION_A 0
#define GPIO_PIN_FUNCTION_B 1
#define GPIO_PIN_FUNCTION_C 2
#define GPIO_PIN_FUNCTION_D 3
#define GPIO_PIN_FUNCTION_E 4
#define GPIO_PIN_FUNCTION_F 5
#define GPIO_PIN_FUNCTION_G 6
#define GPIO_PIN_FUNCTION_H 7

#define PA00 GPIO(GPIO_PORTA, 0)
#define PA01 GPIO(GPIO_PORTA, 1)
#define PYRO_2 GPIO(GPIO_PORTA, 2)
#define PYRO_1 GPIO(GPIO_PORTA, 4)
#define RF_NRST GPIO(GPIO_PORTA, 5)
#define RF_SW GPIO(GPIO_PORTA, 7)
#define RF_INT GPIO(GPIO_PORTA, 8)
#define RF_BUSY GPIO(GPIO_PORTA, 9)
#define RF_CS GPIO(GPIO_PORTA, 10)
#define SD_DETECT GPIO(GPIO_PORTA, 11)
#define MISO GPIO(GPIO_PORTA, 12)
#define BUZZER GPIO(GPIO_PORTA, 13)
#define SD_CS GPIO(GPIO_PORTA, 14)
#define BMP_INT GPIO(GPIO_PORTA, 15)
#define SERVO_1 GPIO(GPIO_PORTA, 16)
#define FLASH_CS GPIO(GPIO_PORTA, 17)
#define SERVO_2 GPIO(GPIO_PORTA, 18)
#define SERVO_3 GPIO(GPIO_PORTA, 19)
#define IMU_INT GPIO(GPIO_PORTA, 21)
#define SDA GPIO(GPIO_PORTA, 22)
#define SCL GPIO(GPIO_PORTA, 23)
#define PA30 GPIO(GPIO_PORTA, 30)
#define RGB_RED GPIO(GPIO_PORTB, 2)
#define RGB_BLUE GPIO(GPIO_PORTB, 8)
#define RGB_GREEN GPIO(GPIO_PORTB, 9)
#define MOSI GPIO(GPIO_PORTB, 10)
#define SCK GPIO(GPIO_PORTB, 11)

#endif // ATMEL_START_PINS_H_INCLUDED
