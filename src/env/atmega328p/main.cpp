#include <avr/io.h>
#include "spi.h"
#include "bootloader.h"

static uint8_t buff[4];

int main()
{
  DDRD = 0;
  PORTD |= _BV(PORTD0);

  spi_init();
  buff[0] = 255;
  buff[1] = 255;
  buff[2] = 255;
  buff[3] = 255;

  spi_transmit(buff, 4);

  if (PIND & _BV(PORTD0)) {
  	runProgram();
  }

  runBootloader();
}

void onStartRecieve() {}

void onBadFrame()
{
  buff[0] = 0;
  buff[1] = 0;
  buff[2] = 0xf;
  buff[3] = 255;
  spi_transmit(buff, 4);
}

void onGoodFrame()
{
  static uint8_t b = 0b11;

  spi_transmit(&b, 1);
  b <<= 1;
  if (!b) {
    b = 0b11;
  }
}

void showCrc(uint16_t crc) {}

void beforeRunProgram()
{
  buff[0] = 0;
  buff[1] = 0;
  buff[2] = 0;
  buff[3] = 0;
	spi_transmit(buff, 4);
}
