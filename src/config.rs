use core::prelude::*;
use platform::sam4l::{usart, ast, gpio};
use hil::timer::AlarmHandler;
use drivers;
use syscall;
use task::Task::UserTask;

pub static mut VirtualTimer:
    Option<drivers::timer::VirtualTimer<ast::Ast>> = None;

pub fn virtual_timer_driver_callback() {
    let mut vt = unsafe {
        VirtualTimer.as_mut().expect("VirtualTimer is None!")
    };

    vt.fire_alarm(|&: addr| {
        UserTask(addr).post();
    });
}

pub fn virtual_timer_driver_svc(r1: usize, r2: usize) -> isize {
    let mut vt = unsafe {
        VirtualTimer.as_mut().expect("VirtualTimer is None!")
    };

    vt.set_user_alarm(r1 as u32, r2)
}

pub static mut Console:
    Option<drivers::uart::console::Console<usart::USART>> = None;

pub fn console_driver_writec_svc(r1: usize, _: usize) -> isize {
    let mut console = unsafe {
        Console.as_mut().expect("Console is None!")
    };

    console.putc(r1 as u8);
    0
}

pub fn console_driver_readc_svc(_: usize, _: usize) -> isize {
    let mut console = unsafe {
        Console.as_mut().expect("Console is None!")
    };

    console.getc() as isize
}

pub static mut LED:
    Option<drivers::gpio::led::LED<gpio::GPIOPin>> = None;

pub fn led_driver_toggle_svc(_: usize, _: usize) -> isize {
    let mut led = unsafe {
        LED.as_mut().expect("LED is None!")
    };

    led.toggle();
    0
}

pub unsafe fn config() {
    let mut ast = ast::Ast::new(virtual_timer_driver_callback);
    ast.setup();

    VirtualTimer = Some(drivers::timer::VirtualTimer::initialize(ast));
    syscall::SUBSCRIBE_DRIVERS[0] = virtual_timer_driver_svc;
    syscall::NUM_SUBSCRIBE_DRIVERS += 1;

    Console = Some(init_console());
    syscall::CMD_DRIVERS[0] = console_driver_writec_svc;
    syscall::NUM_CMD_DRIVERS += 1;

    syscall::CMD_DRIVERS[2] = console_driver_readc_svc;
    syscall::NUM_CMD_DRIVERS += 1;

    LED = Some(init_led());
    syscall::CMD_DRIVERS[1] = led_driver_toggle_svc;
    syscall::NUM_CMD_DRIVERS += 1;
}

fn init_led() -> drivers::gpio::led::LED<gpio::GPIOPin> {
    use platform::sam4l::gpio;

    let pin_10 = gpio::GPIOPin::new(gpio::Params {
        location: gpio::Location::GPIOPin10,
        port: gpio::GPIOPort::GPIO2
    });

    drivers::gpio::led::init(pin_10,
        drivers::gpio::led::InitParams {
            start_status: drivers::gpio::led::LEDStatus::On
        }
    )
}

fn init_console() -> drivers::uart::console::Console<usart::USART> {
    use platform::sam4l::pm;
    use hil::uart;

    let uart_3 = usart::USART::new(usart::Params {
        location: usart::Location::USART3
    });

    let pin_9 = gpio::GPIOPin::new(gpio::Params {
        location: gpio::Location::GPIOPin9,
        port: gpio::GPIOPort::GPIO1
    });

    let pin_10 = gpio::GPIOPin::new(gpio::Params {
        location: gpio::Location::GPIOPin10,
        port: gpio::GPIOPort::GPIO1
    });

    // USART3 clock; this should probably be in USART's init, and should likely
    // depend on the location.
    pm::enable_pba_clock(11);

    drivers::uart::console::init(uart_3, pin_9, pin_10,
        drivers::uart::console::InitParams {
            baud_rate: 115200,
            data_bits: 8,
            parity: uart::Parity::None
        }
    )
}
