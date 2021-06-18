#include <avr/boot.h>

#define TIMER TCNT1

// Defined in bootloader.cpp
void runBootloader();
void runProgram();
uint8_t receiveFrame();
void boot_program_page (uint32_t page, uint8_t *buf);


// Defined in <env>/main.cpp
void onStartRecieve();
void onBadFrame();
void onGoodFrame();
void showCrc(uint16_t crc);
void beforeRunProgram();
