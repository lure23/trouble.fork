#![no_std]
#![no_main]

use bt_hci::controller::ExternalController;
use embassy_executor::Spawner;
use esp_hal::{clock::CpuClock, efuse::Efuse, timer::timg::TimerGroup};
use esp_wifi::ble::controller::BleConnector;
use trouble_example_apps::ble_bas_peripheral;
use trouble_host::Address;
use {esp_alloc as _, esp_backtrace as _};

const L2CAP_MTU: usize = 251;

#[esp_hal_embassy::main]
async fn main(_s: Spawner) {
    esp_println::logger::init_logger_from_env();
    let peripherals = esp_hal::init({
        let mut config = esp_hal::Config::default();
        config.cpu_clock = CpuClock::max();
        config
    });
    esp_alloc::heap_allocator!(72 * 1024);
    let timg0 = TimerGroup::new(peripherals.TIMG0);

    let init = esp_wifi::init(
        timg0.timer0,
        esp_hal::rng::Rng::new(peripherals.RNG),
        peripherals.RADIO_CLK,
    )
    .unwrap();

    #[cfg(not(feature = "esp32"))]
    {
        let systimer = esp_hal::timer::systimer::SystemTimer::new(peripherals.SYSTIMER);
        esp_hal_embassy::init(systimer.alarm0);
    }
    #[cfg(feature = "esp32")]
    {
        esp_hal_embassy::init(timg0.timer1);
    }

    let address = if !cfg!(feature="test-fixed-address") {
        Address::random(Efuse::mac_address())
    } else {
        Address::random([0x41, 0x5A, 0xE3, 0x1E, 0x83, 0xE7])
    };

    let bluetooth = peripherals.BT;
    let connector = BleConnector::new(&init, bluetooth);
    let controller: ExternalController<_, 20> = ExternalController::new(connector);

    if cfg!(any(feature="esp32c2", feature = "esp32c6", feature="esp32h2")) {
        ble_bas_peripheral::run::<_,255>(controller, address)
            .await;
    } else {
        ble_bas_peripheral::run::<_,L2CAP_MTU>(controller, address)
            .await;
    }
}
