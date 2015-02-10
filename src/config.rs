use core::prelude::*;
use core::intrinsics;
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
    Option<drivers::uart::Console<usart::USART>> = None;

pub fn console_driver_writec_svc(r1: usize, _: usize) -> isize {
    let mut console = unsafe {
        Console.as_mut().expect("Console is None!")
    };

    console.putc(r1 as u8);
    0
}

pub fn console_driver_readc_sub(callback: usize, _: usize) -> isize {
    let mut console = unsafe {
        Console.as_mut().expect("Console is None!")
    };

    // !! SO very unsafe! See the note at the bottom of this document.
    let callback_fn: fn(u8) = unsafe { intrinsics::transmute(callback) };
    console.read_subscribe(callback_fn);
    0
}

pub static mut LED:
    Option<drivers::gpio::LED<gpio::GPIOPin>> = None;

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

    syscall::SUBSCRIBE_DRIVERS[1] = console_driver_readc_sub;
    syscall::NUM_SUBSCRIBE_DRIVERS += 1;

    LED = Some(init_led());
    syscall::CMD_DRIVERS[1] = led_driver_toggle_svc;
    syscall::NUM_CMD_DRIVERS += 1;
}

fn init_led() -> drivers::gpio::LED<gpio::GPIOPin> {
    use platform::sam4l::gpio;

    let pin_10 = gpio::GPIOPin::new(gpio::Params {
        location: gpio::Location::GPIOPin10,
        port: gpio::GPIOPort::GPIO2,
        function: None
    });

    drivers::gpio::LED::new(pin_10,
        drivers::gpio::LEDParams {
            start_status: drivers::gpio::LEDStatus::On
        }
    )
}

fn init_console() -> drivers::uart::Console<usart::USART> {
    let uart_3 = usart::USART::new(usart::Params {
        location: usart::Location::USART3
    });

    let _ = gpio::GPIOPin::new(gpio::Params {
        location: gpio::Location::GPIOPin9,
        port: gpio::GPIOPort::GPIO1,
        function: Some(gpio::PeripheralFunction::A)
    });

    let _ = gpio::GPIOPin::new(gpio::Params {
        location: gpio::Location::GPIOPin10,
        port: gpio::GPIOPort::GPIO1,
        function: Some(gpio::PeripheralFunction::A)
    });

    let console: drivers::uart::Console<usart::USART> =
        drivers::uart::Console::new(uart_3,
            drivers::uart::ConsoleParams {
                baud_rate: 115200,
                data_bits: 8,
                parity: drivers::uart::Parity::None
            }
        );

    console
}

#[no_mangle]
#[allow(non_snake_case)]
pub extern fn USART3_Handler() {
    let mut console = unsafe {
        Console.as_mut().expect("Console is None!")
    };

    // This is totally unsafe right now. The UART interrupt handler stored a
    // pointer to user space and calls it in kernel space. This is definitely
    // not what should be hapenning! We could use 'task.post', but then we can't
    // pass parameters to user space. We'll need a better mechanism to invoke
    // user functions.
    console.uart_interrupt();
}
