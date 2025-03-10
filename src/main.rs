use embedded_graphics::image::Image;
use embedded_hal::delay::DelayNs;
use esp_idf_hal::delay::Ets;
use esp_idf_hal::gpio::*;
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::spi::*;
use esp_idf_hal::units::FromValueType;

use embedded_graphics::mono_font::iso_8859_1::FONT_6X10;
use embedded_graphics::mono_font::MonoTextStyleBuilder;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::text::{Baseline, Text};
use mipidsi::interface::SpiInterface;
use mipidsi::{models, Builder};

use esp_backtrace as _;
use tinybmp::Bmp;

const IMG_BYTES: &[u8] = include_bytes!("./img.bmp");

fn main() -> ! {
    println!("start!");
    let peripherals = Peripherals::take().unwrap();
    let gpios = peripherals.pins;

    let mut rst = PinDriver::output(gpios.gpio34).unwrap();
    let dc = PinDriver::output(gpios.gpio33).unwrap();
    let mut backlight = PinDriver::output(gpios.gpio16).unwrap();
    let sclk = gpios.gpio17;
    let sda = gpios.gpio21;
    let cs = gpios.gpio15;

    let mut delay = Ets;

    let spi = peripherals.spi2;

    println!("hard reset start");
    rst.set_low().unwrap();
    delay.delay_ms(100);
    // thread::sleep(Duration::from_millis(100));
    rst.set_high().unwrap();
    delay.delay_ms(2000);
    // thread::sleep(Duration::from_millis(2000));

    // configuring the spi interface
    let config = config::Config::new().baudrate(10.MHz().into());

    let driver =
        SpiDriver::new::<SPI2>(spi, sclk, sda, None::<Gpio15>, &SpiDriverConfig::new()).unwrap();

    let device = SpiDeviceDriver::new(driver, Some(cs), &config).unwrap();

    // display interface abstraction from SPI and DC
    let mut buffer = [0u8; 8];
    let di = SpiInterface::new(device, dc, &mut buffer);

    let mut display = Builder::new(models::GC9107, di)
        .reset_pin(rst)
        // .display_size(128, 128)
        .init(&mut delay)
        .unwrap();

    println!("setup!");

    // turn on the backlight
    backlight.set_high().unwrap();

    let text_style_a = MonoTextStyleBuilder::new()
        .font(&FONT_6X10)
        .text_color(Rgb565::WHITE)
        .build();
    let text_a = Text::with_baseline(
        "Hello World.",
        Point::new(0, 0),
        text_style_a,
        Baseline::Top,
    );

    let bmp = Bmp::from_slice(IMG_BYTES).unwrap();
    let img = Image::new(&bmp, Point::new(0, 0));
    println!("setup2!");

    // draw image on black background
    display.clear(Rgb565::GREEN).unwrap();

    let r = text_a.draw(&mut display);
    println!("text r: {:?}", r);

    let r = img.draw(&mut display);
    println!("img r: {:?}", r);

    println!("Image printed!");

    loop {
        println!("Hello, world!");
        delay.delay_ms(1000);
    }
}
