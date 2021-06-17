#include <avr/io.h>
#include <avr/interrupt.h>
#include <stdlib.h>
#include <avr/boot.h>
#include <util/crc16.h>
#include "bootloader.h"

#define true 1
#define false 0

#define PINVALUE (ACSR & _BV(ACO))

// bootloader commands
#define PROGCOMMAND     2
#define RUNCOMMAND      3

#define METASIZE 5
#define PAGESIZE SPM_PAGESIZE
#define FRAMESIZE (PAGESIZE + METASIZE)

union {
	uint8_t bytes[FRAMESIZE];
	struct {
		uint8_t command;
		uint16_t pageIndex;
		uint8_t page[PAGESIZE];
		uint16_t checksum;
	} data;
} frame;

void (*app_start)(void) = 0x0000;

uint8_t receiveFrame();
void boot_program_page (uint32_t page, uint8_t *buf);

void runBootloader()
{
	// enable pullup on adc3
	PORTC |= _BV(ADC3D);
	// switch off ADC so we can use the mux with the comparator
	ADCSRA &= ~_BV(ADEN);
	// enable mux input to comparator
	ADCSRB |= _BV(ACME);
	// Switch mux to use A3 as the input
	ADMUX = 0b11;
	DIDR1 = _BV(AIN0D) | _BV(AIN1D);

	TCCR1A = 0;
	TCCR1B= _BV(CS11);

	onStartRecieve();

	while (1) {

		if (!receiveFrame()) {
			onBadFrame();
			while (1);
		}

		onGoodFrame();

		if (frame.data.command == RUNCOMMAND) {
			runProgram();
		}

		uint16_t address = SPM_PAGESIZE * frame.data.pageIndex;
		if (address < BOOTLOADER_ADDRESS) {
			boot_program_page(address, (uint8_t*) &(frame.data.page));
		}
	}
}

static inline uint8_t wait_for_edge(uint8_t pinState)
{
	while (pinState == PINVALUE);
	return PINVALUE;
}

uint8_t receiveFrame()
{
	uint16_t stepTime;
  uint16_t totalTime = 0;
  uint16_t delayTime;

  uint8_t pinState = PINVALUE;
	uint8_t newPinState;

  uint8_t byteIndex=0;

	uint16_t checksum = 0;

  //*** synchronisation and bit rate estimation **************************

	pinState = wait_for_edge(pinState);

  TIMER = 0;
  for(uint8_t i = 0; i < 16; i++) {

		pinState = wait_for_edge(pinState);
		stepTime = TIMER;
		TIMER = 0;

    if(i >= 8) {
			totalTime += stepTime;
		} // time accumulator for mean period calculation only the last 8 times are used
  }

  delayTime = totalTime * 3 / 4 / 8;

  while (TIMER < delayTime);

  //****************** wait for start bit ***************************
  while(pinState == PINVALUE)
  {
		pinState = wait_for_edge(pinState);
		TIMER=0;

    while(TIMER < delayTime);

		TIMER = 0;
  }
  pinState = PINVALUE;

  //****************************************************************
  //receive data bits
  for (uint16_t n = 0; n < (FRAMESIZE*8); n++) {

		pinState = wait_for_edge(pinState);

		TIMER = 0;

		// delay 3/4 bit
		while (TIMER < delayTime);

		newPinState = PINVALUE;

		frame.bytes[byteIndex] <<= 1;

		if (pinState != newPinState) {
			frame.bytes[byteIndex] |= 1;
		}

		pinState = newPinState;

		if (n & 0x7) {

			if (byteIndex < FRAMESIZE - 2) {
					checksum = _crc_xmodem_update(checksum, frame.bytes[byteIndex]);
			}

			byteIndex++;
		}
	}

	showCrc(checksum);
	showCrc(frame.data.checksum);

  return (checksum == frame.data.checksum);
}

void boot_program_page (uint32_t page, uint8_t *buf)
{
	uint16_t w;
  cli();

  boot_page_erase (page);
  boot_spm_busy_wait();

	for (uint16_t i = 0; i < SPM_PAGESIZE; i += 2) {
		w = *buf++;
		w |= (*buf++) << 8;

		boot_page_fill(page + i, w);
		boot_spm_busy_wait();
	}

  boot_page_write(page);
  boot_spm_busy_wait();
  boot_rww_enable();
}

void runProgram(void)
{

	beforeRunProgram();

	// reintialize registers to default
	DDRB=0;
	DDRC=0;
	DDRD=0;
	cli();

	TCCR1B=0; // turn off timer

	// restore interrupts
	MCUCR &= ~_BV(IVSEL);

	// start user programm
	// asm volatile(
	// "clr r30	\n\t"
	// "clr r31	\n\t"	// z Register mit Adresse laden
	// "ijmp		\n\t"	// z Register mit Adresse laden
	// );

	app_start();
}
