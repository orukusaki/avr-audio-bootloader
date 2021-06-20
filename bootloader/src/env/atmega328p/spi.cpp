#include "spi.h"

#define MOSI PORTB3
#define SS PORTB2
#define SCK PORTB5

void spi_init()
{
	DDRB = _BV(SCK)|_BV(SS)|_BV(MOSI);
	// spi enable. master mode, full speed
	SPCR = _BV(SPE)|_BV(MSTR);
	PORTB |= _BV(SCK) | _BV(SS);
}

void spi_transmit_byte(uint8_t data)
{
	SPDR = data;
	while(!(SPSR & (1<<SPIF))){}
}

void spi_transmit(uint8_t *data, uint8_t len)
{
		PORTB &= ~_BV(SS);
		while (len--) {
			spi_transmit_byte(*(data++));
		}

		PORTB |= _BV(SS);
}
