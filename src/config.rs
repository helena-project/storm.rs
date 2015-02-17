use core::mem;
use core::prelude::*;
use core::intrinsics;
use platform::sam4l::{usart, ast, gpio};
use platform::sam4l;
use hil::timer::AlarmHandler;
use drivers;
use process;
use syscall;

pub static mut VirtualTimer:
    Option<drivers::timer::VirtualTimer<ast::Ast>> = None;

pub fn virtual_timer_driver_callback() {
    let mut vt = unsafe {
        VirtualTimer.as_mut().expect("VirtualTimer is None!")
    };

    vt.fire_alarm(|&: process_ptr, addr, r0, r1, r2| {
        let process : &mut process::Process = unsafe { mem::transmute(process_ptr) };
        process.callbacks.enqueue(
            process::Callback{
                pc: addr, r0: r0, r1: r1, r2: r2
            });
    });
}

pub fn virtual_timer_driver_svc(process_ptr: *mut (), r1: usize, r2: usize) -> isize {
    let mut vt = unsafe {
        VirtualTimer.as_mut().expect("VirtualTimer is None!")
    };

    vt.set_user_alarm(process_ptr, r1 as u32, r2)
}

pub static mut Console:
    Option<drivers::uart::Console<usart::USART>> = None;

pub fn console_driver_writec_svc(_: *mut (), r1: usize, _: usize) -> isize {
    let mut console = unsafe {
        Console.as_mut().expect("Console is None!")
    };

    console.putc(r1 as u8);
    0
}

pub fn console_driver_readc_sub(_: *mut (), callback: usize, _: usize) -> isize {
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

pub fn led_driver_toggle_svc(_: *mut (), _: usize, _: usize) -> isize {
    let mut led = unsafe {
        LED.as_mut().expect("LED is None!")
    };

    led.toggle();
    0
}

pub static mut TMP006:
    Option<drivers::i2c::tmp006::TMP006<sam4l::i2c::I2CDevice>> = None;


// bradjc: this should be temporary until we have a better app<->device driver
//         interface
pub fn tmp006_driver_read_svc(_: usize, _: usize) -> isize {
    let mut tmp006 = unsafe {
        TMP006.as_mut().expect("TMP006 is None!")
    };

    // return
    tmp006.read_sync() as isize
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

    TMP006 = Some(init_tmp006());
    syscall::CMD_DRIVERS[2] = tmp006_driver_read_svc;
    syscall::NUM_CMD_DRIVERS += 1;

    // In the near future, all config will be handled by a config_tree
    // similar to the one below.
    // TODO(SergioBenitez): Sublocations?
    // IE: gpiopin@1.[0..32], or gpiopin@[1..3][0..32];
    // TODO(SergioBenitez): Two Macro Split through structs? File Inlining?
    // #![allow(unused_variables)] // Can't do this per block YET
    // config_tree!(
    //     platform {sam4l,
    //         gpiopin@[41..43]: gpio::GPIOPin {
    //             port: GPIOPort::GPIO1,
    //             function: ::Some(PeripheralFunction::A)
    //         }

    //         gpiopin@[64..96]: gpio::GPIOPin {
    //             port: GPIOPort::GPIO2,
    //             function: ::None
    //         }

    //         uart@[0..4]: usart::USART;
    //     }

    //     devices {
    //         first_led: gpio::LED(GPIOPin@74) {
    //             start_status: LEDStatus::On
    //         }

    //         console: uart::Console(UART@3) {
    //             baud_rate: 115200,
    //             data_bits: 8,
    //             parity: Parity::None
    //         }
    //     }
    // );
}

fn init_led() -> drivers::gpio::LED<gpio::GPIOPin> {
    use platform::sam4l::gpio;

    let pin_10 = gpio::GPIOPin::new(gpio::GPIOPinParams {
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
    let uart_3 = usart::USART::new(usart::USARTParams {
        location: usart::Location::USART3
    });

    let _ = gpio::GPIOPin::new(gpio::GPIOPinParams {
        location: gpio::Location::GPIOPin9,
        port: gpio::GPIOPort::GPIO1,
        function: Some(gpio::PeripheralFunction::A)
    });

    let _ = gpio::GPIOPin::new(gpio::GPIOPinParams {
        location: gpio::Location::GPIOPin10,
        port: gpio::GPIOPort::GPIO1,
        function: Some(gpio::PeripheralFunction::A)
    });

    drivers::uart::Console::new(uart_3,
        drivers::uart::ConsoleParams {
            baud_rate: 115200,
            data_bits: 8,
            parity: drivers::uart::Parity::None
        }
    )
}

fn init_tmp006() -> drivers::i2c::tmp006::TMP006<sam4l::i2c::I2CDevice> {

    // Create the I2C device with the correct parameters for firestorm
    let i2c_device = sam4l::i2c::I2CDevice::new(sam4l::i2c::I2CParams {
        location:  sam4l::i2c::I2CLocation::I2CPeripheral02,
        bus_speed: sam4l::i2c::I2CSpeed::Fast400k
    });

    // Configure the I2C pins to be in TWIM2 mode
    let _ = gpio::GPIOPin::new(sam4l::gpio::GPIOPinParams {
        location: sam4l::gpio::Location::GPIOPin21,
        port: sam4l::gpio::GPIOPort::GPIO0,
        function: Some(sam4l::gpio::PeripheralFunction::E)
    });

    let _ = gpio::GPIOPin::new(sam4l::gpio::GPIOPinParams {
        location: sam4l::gpio::Location::GPIOPin22,
        port: sam4l::gpio::GPIOPort::GPIO0,
        function: Some(sam4l::gpio::PeripheralFunction::E)
    });

    // return
    drivers::i2c::tmp006::TMP006::new(i2c_device, drivers::i2c::tmp006::TMP006Params {
        addr: 0x40
    })
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
