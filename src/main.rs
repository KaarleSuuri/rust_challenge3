#![no_std]
#![no_main]

use panic_halt as _;

use longan_nano::hal::{pac, eclic::*, prelude::*, time::*, timer::*};
use riscv_rt::entry;
use embedded_graphics::pixelcolor::*;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{Rectangle, PrimitiveStyle};
use longan_nano::{lcd, lcd_pins};

static mut TIMER: Option<Timer<longan_nano::hal::pac::TIMER1>> = None;
#[allow(non_snake_case)]
static mut COLOR : u8 = 10;

#[allow(non_snake_case)]
#[no_mangle]
fn TIMER1() {
    unsafe {
        riscv::interrupt::disable();

        COLOR = COLOR + 1;

        TIMER.as_mut().unwrap().clear_update_interrupt_flag();

        riscv::interrupt::enable();
    }
    
}

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    // Configure clocks
    let mut rcu = dp
        .RCU
        .configure()
        .ext_hf_clock(8.mhz())
        .sysclk(108.mhz())
        .freeze();

    let mut afio = dp.AFIO.constrain(&mut rcu);
    let gpioa = dp.GPIOA.split(&mut rcu);
    let gpiob = dp.GPIOB.split(&mut rcu);
    let lcd_pins = lcd_pins!(gpioa, gpiob);
    let mut lcd = lcd::configure(dp.SPI0, lcd_pins, &mut afio, &mut rcu);
    let (width, height) = (lcd.size().width as i32, lcd.size().height as i32);

    Rectangle::new(Point::new(0, 0), Size::new(width as u32, height as u32))
        .into_styled(PrimitiveStyle::with_fill(Rgb565::RED))
        .draw(&mut lcd)
        .unwrap();

    let timer_regs = dp.TIMER1;
    longan_nano::hal::pac::ECLIC::reset();
    longan_nano::hal::pac::ECLIC::set_threshold_level(Level::L1);
    longan_nano::hal::pac::ECLIC::set_level_priority_bits(LevelPriorityBits::L0P4);
    longan_nano::hal::pac::ECLIC::setup(pac::Interrupt::TIMER1, TriggerType::Level, Level::L1, Priority::P1);
    unsafe {
        let mut timer_tmp = Timer::timer1(timer_regs, Hertz(3), &mut rcu);
        timer_tmp.listen(Event::Update);
        TIMER = Some(timer_tmp);
        longan_nano::hal::pac::ECLIC::unmask(pac::Interrupt::TIMER1);
        riscv::interrupt::enable();
    }

    loop{
        let new_color;
        unsafe {new_color = Rgb565::new(COLOR, COLOR*2, 0);}
        Rectangle::new(Point::new(0, 0), Size::new(width as u32, height as u32))
        .into_styled(PrimitiveStyle::with_fill(new_color))
        .draw(&mut lcd)
        .unwrap();
        //Hint that the thread should go to sleep
        unsafe{riscv::asm::wfi();}
    };
}