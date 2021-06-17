#include <Arduino.h>
#include "bootloader.h"

#define LEDPORT (1<<PB5);

void ledToggle() {
  PORTB^=LEDPORT;
}

void setup()
{
	Serial.begin(9600);
}

void loop()
{
	Serial.println("start loop");
	DDRB|=LEDPORT;

  runBootloader();
}

void onStartRecieve()
{
  Serial.println("receiving..");
}

void onBadFrame()
{
  Serial.println(":(");

  while (1) {
    if ((TIMER & 0xfff) == 0xfff) {
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
