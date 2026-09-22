//! USB HID host example using embassy-usb-host.
//!
//! This example should be built in release mode.
//!
//! Connect a mouse or keyboard to the USB port, and it will log raw HID input reports to the
//! console.
//!
//! The following wiring is assumed:
//! - DP => GPIO20 (GPIO27 on ESP32-P4)
//! - DM => GPIO19 (GPIO26 on ESP32-P4)

//% CHIP_FILTER: usb_otg_driver_supported

#![no_std]
#![no_main]
#![feature(ascii_char)]

// use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::NoopRawMutex, signal::Signal};
use embassy_usb_host::{
    BusRoute, BusState,
    class::cdc_acm::LineCoding,
    class::cdc_acm::*,
    class::{cdc_acm},
};
use esp_backtrace as _;
use esp_hal::{
    timer::timg::TimerGroup,
    usb::otg::{Usb, embassy_usb_host::Driver},
};
use log::*;
use esp_backtrace as _;
use static_cell::StaticCell;

const MAX_BUFFER_SIZE: usize = 512;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(spawner: embassy_executor::Spawner) {
    esp_println::println!("Init!");

    esp_println::logger::init_logger_from_env();
    // esp_println::logger::init_logger(LevelFilter::Trace);
    let peripherals = esp_hal::init(esp_hal::Config::default());

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    let (dp, dm) = (peripherals.GPIO20, peripherals.GPIO19);
    let usb = Usb::new_fs(peripherals.USB_FS, dp, dm);
    static BUS_STATE: BusState = BusState::new();
    let driver = Driver::new(usb);
    let (mut bus_ctrl, bus) = embassy_usb_host::bus(driver, &BUS_STATE);
    info!("USB host initialized, waiting for device...");

    loop {
        // Wait for a device to connect
        let speed = bus_ctrl.wait_for_connection().await;
        info!("Device connected at speed {:?}", speed);

        // Enumerate the device
        let mut config_buf = [0u8; 256];
        let result = bus
            .enumerate(BusRoute::Direct(speed), &mut config_buf)
            .await;

        let (enum_info, _config_len) = match result {
            Ok(r) => r,
            Err(e) => {
                error!("Enumeration failed: {:?}", e);
                continue;
            }
        };

        info!(
            "Enumerated: VID={:04x} PID={:04x} addr={}",
            enum_info.device_desc.vendor_id,
            enum_info.device_desc.product_id,
            enum_info.device_address
        );

        let result = cdc_acm::CdcAcmHost::new(&bus, config_buf.as_slice(), &enum_info);
        let mut cdc_acm_host = match result {
            Ok(r) => r,
            Err(e) => {
                error!("cdc_acm::CdcAcmHost::new failed: {:?}", e);
                continue;
            }
        };
        let line_coding = &LineCoding::default();
        let result = cdc_acm_host.set_line_coding(line_coding).await;
        if result.is_err() {
            error!("cdc_acm_host.set_line_coding failed: {:?}", result);
            continue;
        };
        info!("CdcAcmHost initialized, line_coding={:?}", line_coding);

        static SIGNAL: StaticCell<Signal<NoopRawMutex, heapless::String<MAX_BUFFER_SIZE>>> =
            StaticCell::new();
        let signal = &*SIGNAL.init(Signal::new());

        spawner.spawn(reader_task(cdc_acm_host, &signal).unwrap());
        // TODO: Fix writer_task to work with the new CdcAcmHost API.
        // spawner.spawn(writer_task(cdc_acm_host, &signal).unwrap());

        signal.wait().await;

        info!("Device disconnected, waiting for next...");
    }
}

#[embassy_executor::task]
async fn writer_task(
    mut _tx: CdcAcmHost<'static, embassy_usb_host::BusHandle<'static, embassy_usb_synopsys_otg::host::OtgHostAllocator<'static>>>,
    _signal: &'static Signal<NoopRawMutex, heapless::String<MAX_BUFFER_SIZE>>,
) {
    // use core::fmt::Write;
    // embedded_io_async::Write::write_all(
    //     &mut tx,
    //     b"Hello async USB Serial JTAG. Type something.\r\n",
    // )
    // .await
    // .unwrap();
    // loop {
    //     let message = signal.wait().await;
    //     signal.reset();
    //     write!(&mut tx, "-- received '{}' --\r\n", message).unwrap();
    //     embedded_io_async::Write::flush(&mut tx).await.unwrap();
    // }
}

#[embassy_executor::task]
async fn reader_task(
    mut rx: CdcAcmHost<'static, embassy_usb_host::BusHandle<'static, embassy_usb_synopsys_otg::host::OtgHostAllocator<'static>>>,
    _signal: &'static Signal<NoopRawMutex, heapless::String<MAX_BUFFER_SIZE>>,
) {
    // let mut rbuf = [0u8; MAX_BUFFER_SIZE];
    // loop {
    //     let r = embedded_io_async::Read::read(&mut rx, &mut rbuf).await;
    //     match r {
    //         Ok(len) => {
    //             let mut string_buffer: heapless::Vec<_, MAX_BUFFER_SIZE> = heapless::Vec::new();
    //             string_buffer.extend_from_slice(&rbuf[..len]).unwrap();
    //             signal.signal(heapless::String::from_utf8(string_buffer).unwrap());
    //         }
    //         #[allow(unreachable_patterns)]
    //         Err(e) => esp_println::println!("RX Error: {:?}", e),
    //     }
    // }

    info!("reader_task started, waiting for data...");

    loop {
        let mut read_buff = [0u8; 256];
        let read_result = rx.read(read_buff.as_mut_slice()).await;
        if let Err(e) = read_result {
            error!("cdc_acm_host.read failed: {:?}", e);
            continue;
        } else {
            info!("cdc_acm_host.read read_buff={:?}", read_buff.as_ascii());
        }
    }
}
