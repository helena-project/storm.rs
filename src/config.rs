use core::prelude::*;
use platform::sam4l::usart;
use platform::sam4l::ast;
use hil::timer::AlarmHandler;
use drivers;
use syscall;
use task::Task::UserTask;

pub static mut VirtualTimer :
    Option<drivers::timer::VirtualTimer<ast::Ast>> = None;

pub fn virtual_timer_driver_callback() {
    unsafe {
        let mut vt = VirtualTimer.take().unwrap();
        vt.fire_alarm(|&: addr| {
            UserTask(addr).post();
        });
        VirtualTimer = Some(vt);
    }
}

pub fn virtual_timer_driver_svc(r1 : usize, r2 : usize) -> isize {
    unsafe {
        let mut vt = VirtualTimer.take().unwrap();
        let res = vt.set_user_alarm(r1 as u32, r2);
        VirtualTimer = Some(vt);
        return res;
    }
}

pub static mut Console :
    Option<drivers::uart::console::Console<usart::USART>> = None;

pub fn console_driver_writec_svc(r1: usize, _: usize) -> isize {
    unsafe {
        let mut console = Console.take().unwrap();
        console.putc(r1 as u8);
        Console = Some(console);
        return 0;
    }
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

}

// Mock UART Usage
fn init_console() -> drivers::uart::console::Console<usart::USART> {
    use platform::sam4l::{gpio, pm};
    use hil::gpio::*;
    use hil::uart;

    let uart_3 = usart::USART::new(usart::Params {
        location: usart::Location::USART3
    });

    // Set up as USB output
    let p1 = gpio::Pin {bus : gpio::Port::PORT1, pin : 9};
    p1.set_peripheral_function(PeripheralFunction::A);
    let p2 = gpio::Pin {bus : gpio::Port::PORT1, pin : 10};
    p2.set_peripheral_function(PeripheralFunction::A);

    // USART3 clock; this should probably be in USART's init
    pm::enable_pba_clock(11);

    let console = drivers::uart::console::init(uart_3,
        drivers::uart::console::InitParams {
            baud_rate: 115200,
            data_bits: 8,
            parity: uart::Parity::NONE
        }
    );

    console
}
