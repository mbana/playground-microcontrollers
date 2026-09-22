//! CDC-ACM serial port example using polling in a busy loop.
//!
//! This example should be built in release mode.
//!
//! The following wiring is assumed:
//! - DP => GPIO20
//! - DM => GPIO19

#![no_std]
#![no_main]

esp_bootloader_esp_idf::esp_app_desc!();

use embassy_executor;
use embassy_usb_driver::host::{UsbHostAllocator, UsbHostController};
use esp_backtrace as _;
use esp_hal::peripherals::{self, GPIO18};
use esp_hal::usb::otg::embassy_usb_host::*;
use esp_hal::{
    gpio::{Io, Level, Output, OutputConfig},
    usb::otg::{Usb, embassy_usb_host},
};
use esp_println::{print, println};
use esp_rtos::{embassy, main};
use log::LevelFilter;
use static_cell::StaticCell;
// use embassy_futures::select::select;
use core::cell::RefCell;

static EXECUTOR: StaticCell<esp_rtos::embassy::Executor> = StaticCell::new();

// use esp_hal::{
//     clock::CpuClock,
//     gpio::{Io, Level, Output, OutputConfig},
//     // main,
//     time::{Duration, Instant},
// };

// // You need a panic handler. Usually, you would use esp_backtrace, panic-probe, or
// // something similar, but you can also bring your own like this:
// #[panic_handler]
// fn panic(_: &core::panic::PanicInfo) -> ! {
//     esp_hal::system::software_reset()
// }

#[embassy_executor::task]
async fn task_device_events(mut driver: Driver<'static>) -> () {
    println!("waiting for device events ...");

    // let device_event = driver.wait_for_device_event().await;
    // println!("device_event={:?}", device_event);
}

#[embassy_executor::task]
async fn task_toggle_led(pin: GPIO18<'static>) -> () {
    println!("toggling LED ...");

    // Set GPIO0 as an output, and set its state high initially.
    let mut led = Output::new(pin, Level::High, OutputConfig::default());

    // let delay = esp_hal::delay::Delay::new();
    // delay.delay_millis(5000);
    // led.toggle();
}

#[main]
// #[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    esp_println::logger::init_logger(LevelFilter::Error);

    println!("spawner.executor_id={:?}", spawner.executor_id());

    println!("initializing USB driver ...");

    let peripherals = esp_hal::init(esp_hal::Config::default());
    let usb = Usb::new_fs(peripherals.USB_FS, peripherals.GPIO20, peripherals.GPIO19);
    let driver = Driver::new(usb);

    let executor: &'static mut esp_rtos::embassy::Executor = EXECUTOR.init(esp_rtos::embassy::Executor::default());
    executor.run(|spawner: embassy_executor::Spawner| -> () {
        println!("spawner.executor_id={:?}", spawner.executor_id());

        let device_events = task_device_events(driver).expect("task_device_events to be ok");
        spawner.spawn(device_events);

        let toggle_led = task_toggle_led(peripherals.GPIO18).expect("task_toggle_led to be ok");
        spawner.spawn(toggle_led);
    });
}
