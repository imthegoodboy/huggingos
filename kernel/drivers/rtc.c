#include "drivers.h"
#include "../interrupts.h"
#include "../lib/lib.h"

// RTC I/O ports
#define CMOS_ADDRESS 0x70
#define CMOS_DATA    0x71

// RTC register indices
#define RTC_SECONDS  0x00
#define RTC_MINUTES  0x02
#define RTC_HOURS    0x04
#define RTC_DAY      0x07
#define RTC_MONTH    0x08
#define RTC_YEAR     0x09
#define RTC_STATUS_A 0x0A
#define RTC_STATUS_B 0x0B

// RTC Status Register B flags
#define RTC_24HOUR   0x02
#define RTC_BINARY   0x04

static rtc_time_t rtc_time = {0};

static uint8_t rtc_read_register(uint8_t reg)
{
    outb(CMOS_ADDRESS, reg);
    return inb(CMOS_DATA);
}

static uint8_t bcd_to_bin(uint8_t bcd)
{
    return ((bcd >> 4) * 10) + (bcd & 0x0F);
}

static bool rtc_is_updating(void)
{
    outb(CMOS_ADDRESS, RTC_STATUS_A);
    return (inb(CMOS_DATA) & 0x80) != 0;
}

void rtc_init(void)
{
    // Wait for RTC to not be updating
    while (rtc_is_updating());
    
    // Read status register B
    uint8_t status_b = rtc_read_register(RTC_STATUS_B);
    bool use_24h = (status_b & RTC_24HOUR) != 0;
    bool use_binary = (status_b & RTC_BINARY) != 0;
    
    // Read RTC registers
    uint8_t second = rtc_read_register(RTC_SECONDS);
    uint8_t minute = rtc_read_register(RTC_MINUTES);
    uint8_t hour = rtc_read_register(RTC_HOURS);
    uint8_t day = rtc_read_register(RTC_DAY);
    uint8_t month = rtc_read_register(RTC_MONTH);
    uint8_t year = rtc_read_register(RTC_YEAR);
    
    // Convert BCD to binary if needed
    if (!use_binary) {
        second = bcd_to_bin(second);
        minute = bcd_to_bin(minute);
        day = bcd_to_bin(day);
        month = bcd_to_bin(month);
        year = bcd_to_bin(year);
        
        if (!use_24h) {
            hour = bcd_to_bin(hour);
            // Handle 12-hour format
            if (hour & 0x80) {
                // PM
                hour = ((hour & 0x7F) + 12) % 24;
            } else {
                hour = hour & 0x7F;
                if (hour == 12) hour = 0;
            }
        } else {
            hour = bcd_to_bin(hour);
        }
    } else {
        if (!use_24h) {
            // Handle 12-hour format in binary
            if (hour & 0x80) {
                hour = ((hour & 0x7F) + 12) % 24;
            } else {
                if (hour == 12) hour = 0;
            }
        }
    }
    
    rtc_time.second = second;
    rtc_time.minute = minute;
    rtc_time.hour = hour;
    rtc_time.day = day;
    rtc_time.month = month;
    rtc_time.year = year;
    rtc_time.initialized = true;
}

void rtc_get_time(rtc_time_t* time)
{
    if (!rtc_time.initialized) {
        rtc_init();
    }
    
    // Wait for RTC to not be updating
    while (rtc_is_updating());
    
    // Read current time
    uint8_t status_b = rtc_read_register(RTC_STATUS_B);
    bool use_24h = (status_b & RTC_24HOUR) != 0;
    bool use_binary = (status_b & RTC_BINARY) != 0;
    
    uint8_t second = rtc_read_register(RTC_SECONDS);
    uint8_t minute = rtc_read_register(RTC_MINUTES);
    uint8_t hour = rtc_read_register(RTC_HOURS);
    uint8_t day = rtc_read_register(RTC_DAY);
    uint8_t month = rtc_read_register(RTC_MONTH);
    uint8_t year = rtc_read_register(RTC_YEAR);
    
    // Convert BCD to binary if needed
    if (!use_binary) {
        second = bcd_to_bin(second);
        minute = bcd_to_bin(minute);
        day = bcd_to_bin(day);
        month = bcd_to_bin(month);
        year = bcd_to_bin(year);
        
        if (!use_24h) {
            hour = bcd_to_bin(hour);
            if (hour & 0x80) {
                hour = ((hour & 0x7F) + 12) % 24;
            } else {
                hour = hour & 0x7F;
                if (hour == 12) hour = 0;
            }
        } else {
            hour = bcd_to_bin(hour);
        }
    } else {
        if (!use_24h) {
            if (hour & 0x80) {
                hour = ((hour & 0x7F) + 12) % 24;
            } else {
                if (hour == 12) hour = 0;
            }
        }
    }
    
    time->second = second;
    time->minute = minute;
    time->hour = hour;
    time->day = day;
    time->month = month;
    time->year = year;
    time->initialized = true;
}

// Get full year (assumes year 2000+)
uint16_t rtc_get_full_year(void)
{
    rtc_time_t time;
    rtc_get_time(&time);
    // RTC stores years as 0-99, we assume 2000-2099
    return 2000 + time.year;
}

uint8_t rtc_get_month(void)
{
    rtc_time_t time;
    rtc_get_time(&time);
    return time.month;
}

uint8_t rtc_get_day(void)
{
    rtc_time_t time;
    rtc_get_time(&time);
    return time.day;
}

uint8_t rtc_get_hour(void)
{
    rtc_time_t time;
    rtc_get_time(&time);
    return time.hour;
}

uint8_t rtc_get_minute(void)
{
    rtc_time_t time;
    rtc_get_time(&time);
    return time.minute;
}

uint8_t rtc_get_second(void)
{
    rtc_time_t time;
    rtc_get_time(&time);
    return time.second;
}
