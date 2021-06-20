#include <Arduino.h>
#include "bootloader.h"

#define LEDPORT (1<<PB5);

const uint16_t baud = 9600;

void ledToggle() {
  PORTB^=LEDPORT;
}

void setup()
{
	Serial.begin(baud);
}

void loop()
{
	Serial.println("start");
	DDRB|=LEDPORT;

  runBootloader();
}

void onStartRecieve()
{
  Serial.println("receiving..");
}

void onBadFrame()
{
  const uint16_t timerTop = 0xfff;
  
  Serial.println(":(");

  while (1) {
    if ((TIMER & timerTop) == timerTop) {
        ledToggle();
    }
  }
}

void onGoodFrame()
{
    ledToggle();
    Serial.println(":)");
}

void showCrc(uint16_t crc)
{
    Serial.print("crc:");
    Serial.println(crc, HEX);
}

void beforeRunProgram()
{
  Serial.println("Running");
}
