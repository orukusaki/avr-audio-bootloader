#include <avr/boot.h>

#define TIMER TCNT1

// Defined in bootloader.cpp
void runBootloader();
void runProgram();

// Defined in <env>/main.cpp
void onStartRecieve();
void onBadFrame();
void onGoodFrame();
void showCrc(uint16_t crc);
void beforeRunProgram();
