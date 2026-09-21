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

use esp_println::{print, println};
use esp_backtrace as _;
use esp_hal::usb::otg::{Usb, embassy_usb_host};
use esp_hal::{
    main,
    usb::otg::embassy_usb_host::*,
};
use embassy_usb_driver::host::{self, UsbHostAllocator, UsbHostController};
use embassy_usb_synopsys_otg::{
    // otg_v1::Otg,
};

#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    let mut peripherals = esp_hal::init(esp_hal::Config::default());
    let mut usb = Usb::new_fs(peripherals.USB_FS, peripherals.GPIO20, peripherals.GPIO19);
    let mut driver = Driver::new(usb);
    
    // let mut host_controller = UsbHostController::allocator(&driver);
    
    // let mut controller = UsbHostController::allocator(&driver);
    // controller.wait_for_device_event().await();

    loop {
        println!("looping forever ...");
        let delay = esp_hal::time::Duration::from_millis(1000);
        while esp_hal::time::Instant::now().elapsed() < delay {
            print!(".");
        }
        println!()
    }

    // loop {
    //     if !usb_dev.poll(&mut [&mut serial]) {
    //         continue;
    //     }

    //     let mut buf = [0u8; 64];

    //     match serial.read(&mut buf) {
    //         Ok(count) if count > 0 => {
    //             // Echo back in upper case
    //             for c in buf[0..count].iter_mut() {
    //                 if 0x61 <= *c && *c <= 0x7a {
    //                     *c &= !0x20;
    //                 }
    //             }

    //             let mut write_offset = 0;
    //             while write_offset < count {
    //                 match serial.write(&buf[write_offset..count]) {
    //                     Ok(len) if len > 0 => {
    //                         write_offset += len;
    //                     }
    //                     _ => {}
    //                 }
    //             }
    //         }
    //         _ => {}
    //     }
    // }
}
