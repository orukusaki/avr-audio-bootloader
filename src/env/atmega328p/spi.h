#include <avr/io.h>

void spi_init();
void spi_transmit_byte(uint8_t data);
void spi_transmit(uint8_t *data, uint8_t len);
