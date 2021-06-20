#include "bootloader.h"
#include <avr/boot.h>
#include <avr/interrupt.h>
#include <stdlib.h>
#include <util/crc16.h>

// bootloader commands
constexpr uint8_t PROGCOMMAND = 2;
constexpr uint8_t RUNCOMMAND = 3;

constexpr uint8_t METASIZE = 5;
constexpr uint8_t FRAMESIZE = SPM_PAGESIZE + METASIZE;

union {
	uint8_t bytes[FRAMESIZE];
	struct {
		uint8_t command;
		uint16_t pageIndex;
		uint8_t page[SPM_PAGESIZE];
		uint16_t checksum;
	} data;
} frame;

static volatile inline uint8_t pinValue() {
	return ACSR & _BV(ACO);
}

static inline uint8_t wait_for_edge(uint8_t pinState)
{
	while (pinValue() == pinState) {}

	return pinValue();
}

static void wait_for_time(uint16_t delayTime)
{
	TIMER = 0;
	while (TIMER < delayTime) {}
}

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
	TCCR1B = _BV(CS11);

	onStartRecieve();

	while (receiveFrame()) {

		onGoodFrame();

		if (frame.data.command == RUNCOMMAND) {
			runProgram();
		}

		uint16_t address = SPM_PAGESIZE * frame.data.pageIndex;
		if (address < BOOTLOADER_ADDRESS) {
			boot_program_page(address, (uint8_t*) &(frame.data.page));
		}
	}

	onBadFrame();
	while (1) {}
}

uint8_t receiveFrame()
{
  uint16_t totalTime = 0;
  uint16_t delayTime;
  uint8_t pinState = pinValue();
  uint8_t byteIndex = 0;
	uint16_t checksum = 0;

  //synchronisation and bit rate estimation
	pinState = wait_for_edge(pinState);

  for (uint8_t i = 0; i < 16; i++) {

		TIMER = 0;
		pinState = wait_for_edge(pinState);

    if(i >= 8) {
			totalTime += TIMER;
		}
  }

  delayTime = totalTime * 3 / 4 / 8;

	// Wait for start (1) bit
  do {
  	pinState = wait_for_edge(pinState);
		wait_for_time(delayTime);
  } while (pinState == pinValue());
  pinState = pinValue();

  //receive data bits
  for (uint16_t n = 0; n < (FRAMESIZE * 8); n++) {

		pinState = wait_for_edge(pinState);
		wait_for_time(delayTime);

		uint8_t newPinState = pinValue();

		frame.bytes[byteIndex] <<= 1;
		frame.bytes[byteIndex] |= (pinState != newPinState);

		pinState = newPinState;

		if ((n & 0x7) == 0x7) {  // every 8th bit

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

void boot_program_page(uint32_t page, uint8_t *buf)
{
  cli();

  boot_page_erase(page);
  boot_spm_busy_wait();

	for (uint16_t i = 0; i < SPM_PAGESIZE; i += 2) {
		uint16_t w = *buf++;
		w |= (*buf++) << 8;

		boot_page_fill(page + i, w);
		boot_spm_busy_wait();
	}

  boot_page_write(page);
  boot_spm_busy_wait();
  boot_rww_enable();
}

void runProgram()
{
	beforeRunProgram();

	DDRB=0;
	DDRC=0;
	DDRD=0;
	cli();

	TCCR1B=0; // turn off timer

	// restore interrupts
	MCUCR &= ~_BV(IVSEL);

	//Jump to address 0
	((void (*)()) 0)();
}
