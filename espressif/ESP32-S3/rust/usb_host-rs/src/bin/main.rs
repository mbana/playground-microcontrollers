#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use bt_hci::controller::ExternalController;
use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_radio::ble::controller::BleConnector;
use panic_rtt_target as _;
use trouble_host::prelude::*;

extern crate alloc;

const CONNECTIONS_MAX: usize = 1;
const L2CAP_CHANNELS_MAX: usize = 1;

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

use embassy_sync::channel::Sender;
use esp_hal_smartled::LedAdapterError;
use thiserror::Error;

/// Alias for the actor's inbox
pub type ActorInbox<M> = Sender<'static, NoopRawMutex, M, 10>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Failed to write to LED: {0:?}")]
    LedWrite(LedAdapterError),
    #[error("Failed to send message to LED actor")]
    LedActorSend,
    #[error("Failed to send message to IMU actor")]
    ImuActorSend,
    #[error("Read time {0}ms must be less than max read time {1}ms")]
    InvalidReadTime(u64, u64),
    #[error("Read period {0}ms must be greater than read time {1}ms")]
    InvalidReadPeriod(u64, u64),
    #[error("Failed to read from Ambient Sensor")]
    AmbientI2cRead,
}

use actor_private::*;
use esp_hal::rmt;
use esp_hal_smartled::SmartLedsAdapterAsync;
use smart_leds::{RGB8, SmartLedsWriteAsync, brightness, colors::BLACK, gamma};
use {
    core::future::pending,
    embassy_executor::SpawnError,
    embassy_futures::select::{Either, select},
};

pub type Led = SmartLedsAdapterAsync<rmt::Channel<esp_hal::Async, 0>, 25>;

/// Set the colour and brightness of the specified LED.
pub async fn write(led: &mut Led, colour: RGB8, level: u8) -> Result<(), AppError> {
    led.write(brightness(gamma([colour].into_iter()), level))
        .await
        .map_err(AppError::LedWrite)
}

/// The actor's repeat mode.
#[derive(Clone, Copy)]
pub enum Repeat {
    /// Run the sequence once
    Once,
    /// Run the sequence a fixed number of times
    N(u8),
    /// Run the sequence forever
    Forever,
}

pub struct LedActor(ActorInbox<Message>);

impl LedActor {
    /// Turn on the LED
    pub fn on(&self) -> Result<(), AppError> {
        self.0
            .try_send(Message::On)
            .map_err(|_| AppError::LedActorSend)
    }
    /// Turn off the LED
    pub fn off(&self) -> Result<(), AppError> {
        self.0
            .try_send(Message::Off)
            .map_err(|_| AppError::LedActorSend)
    }
    /// Set the colour of the LED
    pub fn set_colour(&self, colour: RGB8) -> Result<(), AppError> {
        self.0
            .try_send(Message::SetColour(colour))
            .map_err(|_| AppError::LedActorSend)
    }
    /// Set the brightness of the LED
    pub fn set_brightness(&self, level: u8) -> Result<(), AppError> {
        self.0
            .try_send(Message::SetBrightness(level))
            .map_err(|_| AppError::LedActorSend)
    }
    /// Set the LED to a sequence of colours
    pub fn set_sequence(
        &self,
        sequence: &'static [RGB8],
        step_duration: Duration,
        repeat: Repeat,
    ) -> Result<(), AppError> {
        self.0
            .try_send(Message::SetSequence((sequence, step_duration, repeat)))
            .map_err(|_| AppError::LedActorSend)
    }
}

/// Create a new actor with a spawner and a configuration.
/// This pattern could be made into a macro to simplify the actor creation.
pub fn spawn_actor(spawner: Spawner, led: Led) -> Result<LedActor, SpawnError> {
    static CONTEXT: ActorContext<Actor, NoopRawMutex, 10> = ActorContext::new();
    let inbox = CONTEXT.address();
    spawner.spawn(actor_task(&CONTEXT, Actor::new(spawner, led, inbox)))?;
    Ok(LedActor(inbox))
}

mod actor_private {

    use ector::{DynamicAddress, Inbox};
    use log::error;

    use super::*;

    /// The actor's message type, communicating the finite states of the actor.
    pub(super) enum Message {
        /// Set the colour of the LED
        SetColour(RGB8),
        /// Set the brightness of the LED
        SetBrightness(u8),
        /// Turn the LED off
        Off,
        /// Turn the LED on
        On,
        /// Set the LED to a sequence of colours
        SetSequence((&'static [RGB8], Duration, Repeat)),
    }

    /// A scheduler to run a sequence of actions.
    struct Scheduler {
        /// The timer to schedule the next action
        timer: Timer,
        /// The period between actions
        period: Duration,
        /// The current sequence of colours
        sequence: &'static [RGB8],
        /// The current index in the sequence
        index: usize,
        /// The current repeat mode
        repeat: Repeat,
    }

    /// The actor's private data, not to be shared with other actors.
    /// This is where the actor's state is stored.
    pub(super) struct Actor {
        /// A timer to schedule the next message
        scheduler: Option<Scheduler>,
        /// The LED to control
        led: Led,
        /// The current colour of the LED
        colour: RGB8,
        /// The current brightness of the LED
        /// This is a percentage from 0 to 100
        brightness: u8,
    }

    impl ector::Actor for Actor {
        type Message = Message;

        /// Actor pattern for either handling new incoming messages or running a scheduled action.
        async fn on_mount<M>(&mut self, _: DynamicAddress<Message>, mut inbox: M) -> !
        where
            M: Inbox<Self::Message>,
        {
            info!("LED Task started!");
            loop {
                let deadline = async {
                    match self.scheduler.as_mut() {
                        Some(Scheduler { timer, .. }) => timer.await,
                        None => pending().await,
                    }
                };
                if let Err(err) = match select(inbox.next(), deadline).await {
                    Either::First(action) => self.act(action).await,
                    Either::Second(_) => self.next().await,
                } {
                    error!("Error in LED actor: {:?}", err);
                };
            }
        }
    }

    impl Actor {
        /// Create a new actor with a spawner and a configuration.
        pub(super) fn new(_: Spawner, led: Led, _: ActorInbox<Message>) -> Self {
            // Opportunity to do any setup before mounting the actor
            // this could include spawning child actors or setting up resources
            // we have access to our own inbox here to send down to child actors.
            Self {
                led,
                scheduler: None,
                colour: RGB8 { r: 0, g: 0, b: 0 },
                brightness: 50,
            }
        }
        /// The message handler
        async fn act(&mut self, msg: Message) -> Result<(), AppError> {
            self.scheduler = None; // cancel any scheduled actions
            match msg {
                Message::SetColour(colour) => {
                    self.colour = colour;
                    write(&mut self.led, colour, self.brightness).await
                }
                Message::SetBrightness(level) => {
                    self.brightness = level;
                    write(&mut self.led, self.colour, level).await
                }
                Message::Off => write(&mut self.led, BLACK, 0).await,
                Message::On => write(&mut self.led, self.colour, self.brightness).await,
                Message::SetSequence((sequence, period, repeat)) => {
                    self.scheduler = Some(Scheduler {
                        timer: Timer::after(period),
                        period,
                        sequence,
                        index: 0,
                        repeat,
                    });
                    Ok(())
                }
            }
        }
        /// Run the next scheduled action.
        async fn next(&mut self) -> Result<(), AppError> {
            let Some(scheduler) = self.scheduler.as_mut() else {
                return Ok(()); // no scheduled action
            };
            scheduler.timer = Timer::after(scheduler.period);
            // run the next action in the sequence.
            match scheduler.sequence.get(scheduler.index) {
                Some(&colour) => {
                    write(&mut self.led, colour, self.brightness).await?;
                    scheduler.index += 1;
                }
                None => {
                    // if we've reached the end of the sequence, handle the repeat mode.
                    match scheduler.repeat {
                        Repeat::Once => self.scheduler = None,
                        Repeat::N(0) => self.scheduler = None,
                        Repeat::N(n) => scheduler.repeat = Repeat::N(n - 1),
                        Repeat::Forever => scheduler.index = 0,
                    }
                }
            };
            Ok(())
        }
    }

    #[embassy_executor::task]
    /// The actor's task, to be spawned by the actor's context.
    pub(super) async fn actor_task(
        context: &'static ActorContext<Actor, NoopRawMutex, 10>,
        actor: Actor,
    ) {
        context.mount(actor).await;
    }
}

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[esp_rtos::main]
async fn main(spawner: Spawner) -> ! {
    // generator version: 1.4.0
    // generator parameters: -o esp32s3 -o unstable-hal -o alloc -o esp32s3-wroom-1-octal-psram -o embassy -o wifi -o ble-trouble -o stack-smashing-protection -o probe-rs -o defmt -o panic-rtt-target -o embedded-test -o cargo-embed -o vscode -o esp

    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // The following pins are used to bootstrap the chip. They are available
    // for use, but check the datasheet of the module for more information on them.
    // - GPIO0
    // - GPIO3
    // - GPIO45
    // - GPIO46
    // These GPIO pins are in use by some feature of the module and should not be used.
    let _gpio27 = peripherals.GPIO27;
    let _gpio28 = peripherals.GPIO28;
    let _gpio29 = peripherals.GPIO29;
    let _gpio30 = peripherals.GPIO30;
    let _gpio31 = peripherals.GPIO31;
    let _gpio32 = peripherals.GPIO32;
    let _gpio33 = peripherals.GPIO33;
    let _gpio34 = peripherals.GPIO34;
    let _gpio35 = peripherals.GPIO35;
    let _gpio36 = peripherals.GPIO36;
    let _gpio37 = peripherals.GPIO37;

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 73744);
    // COEX needs more RAM - so we've added some more
    esp_alloc::heap_allocator!(size: 64 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_rtos::start(timg0.timer0, peripherals.FROM_CPU_INTR0);

    info!("Embassy initialized!");

    let _wifi_controller =
        esp_radio::wifi::WifiController::new(peripherals.WIFI, Default::default())
            .expect("Failed to initialize Wi-Fi controller");
    let _wifi_interface = esp_radio::wifi::Interface::station();
    // find more examples https://github.com/embassy-rs/trouble/tree/main/examples/esp32
    let transport = BleConnector::new(peripherals.BT, Default::default()).unwrap();
    let ble_controller = ExternalController::<_, 1>::new(transport);
    let mut resources: HostResources<_, DefaultPacketPool, CONNECTIONS_MAX, L2CAP_CHANNELS_MAX> =
        HostResources::new();
    let _stack = trouble_host::new(ble_controller, &mut resources).build();

    // TODO: Spawn some tasks
    let _ = spawner;

    loop {
        info!("Hello world!");
        Timer::after(Duration::from_secs(1)).await;
    }

    // for inspiration have a look at the examples at https://github.com/esp-rs/esp-hal/tree/esp-hal-v1.2.2/examples
}
