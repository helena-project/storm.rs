TockOS Driver Infrastructure
=============================

This document provides a detailed overview of the proposed TockOS driver
infrastructure.

Overview
--------

The TockOS driver infrastructure aims to be a transparent, magic-free mechanism
for connecting device drivers to devices. A device driver offers a (usually
high-level) interface to a specific device connected to the platform. For
example, a SPI-based flash storage device driver could offer a key-value store
interface for flash-based storage connected via an on-board SPI controller.

Device drivers do not communicate directly with the platform's device control
hardware. Such an approach would be fragile and unsafe as it would require the
device driver to tie its implementation to a specific architecture and program
for the low-level details of the control hardware. Further, two device drivers
utilizing the same control hardware would be required to implement similar
logic. As a result, device drivers in TockOS communicate with the hardware
through controller interfaces implemented by platform drivers.

A platform driver offers an interface to platform control hardware. These
interfaces are fixed and platform-independent. A device driver targeting a
controller interface can be used on any architecture with platform drivers
supplying the necessary interface. Platform drivers and device drivers operate
independently; device drivers need not have knowledge of a platform driver's
implementation and vice versa.

Device Drivers and Platform Drivers
-----------------------------------

Device drivers are instantiated declaratively via a device tree. Device trees in
TockOS are similar to device trees in Linux, except that they are verified at
compile time to ensure that device drivers exist, that the drivers' resource
requirements are satisfied exclusively, that the driver is instantiated with
all necessary parameters of the correct type, and that no race conditions can
occur when one driver uses another driver.

A device tree is written inside of a device_tree! macro. The following is an
example of a device tree that instantiates five devices: (1, 2) GPIO based LEDs,
(3) a bit-banged temperature sensor, (4) a UART based serial console, and (5) a
SPI flash device.

    device_tree!(
      red_led: led::GPIOLed(GPIO@5);
      green_led: led::GPIOLed(GPIO@4);

      temperature: bit_bang::Temperature(GPIO@0, GPIO@1, GPIO@2);

      console: uart::Console(UART(USART@3)) {
        baud_rate = 115200;
        data_bits = 8;
        parity = 0;
        stop_bits = 1;
      }

      flash: spi::MTDFlashM25P80(SPIMaster(USART@0)) {
        # 128mbit/16MB n25q128 SPI flash storage
        max_tx_lines = 2;
        max_rx_lines = 2;
        max_frequency = 50e6;

        # 16 * 2^20 == 16MB == 128mbit
        size = 0x1000000;
        status_indicator = &green_led;
      }
    );

Let's walk through each of these drivers as each makes use of a unique feature
of the device tree syntax and semantics.

The first device declared is the `red_led` device. The name `red_led` is
arbitrary; a handle to the device with that name will be available to the
user after device initialization. The device is declared with type
`led::GPIOLed`. This type is a path to the driver's type rooted at the OS
driver path, which is `::drivers`. The `led::GPIOLed` device driver requires
a `GPIO` resource to function. This resource is declared in parenthesis as
`GPIO@5`, indicating that the platform `GPIO` at location `5` will be allocated
to this driver.

At compile time, the path and resource availability are verified. The OS will
not compile if the path is incorrect or the resource requested is unavailable or
previously requested and allocated.

The `green_led` device is declared similarly, albeit with the `GPIO` at location
`4` as its resource.

The `temperature` device has a similar declaration. It is of type
`bit_bang::Temperature`. This example illustrates a driver requiring multiple
resources to function. Here, it requires three `GPIO` resources, and the `GPIO`s
at locations `0`, `1`, and `2` are requested.

The console device describes a device named `console` of type `uart::Console`.
The `Console` driver requires a `UART` resource to function. Here we illustrate
a scenario where a dedicated `UART` resource is not available, but a polymorphic
resource, a `USART`, is. Polymorphic resources can be requested to act as given
variant. This declaration requests that the `USART` controller at location
`3` be configured as a `UART`. The device driver need not be modified to reflect
this; the `USART` will expose the same interface a dedicated `UART` would.

The `console` device driver requires properties to be instantiated. These
properties are declared inside of a block. Here, four properties, `baud_rate`,
`data_bits`, `parity`, and `stop_bits` with values `115200`, `8`, `0`, and `1`,
respectively, are declared. These properties are used by the driver for
device configuration. It is a compile time error to supply properties of
incorrect type, and to omit or inject extra properties.

Finally, a `flash` device is declared similarly to the console device. The
`status_indicator` property is of particular interest here as it references the
previously declared `green_led` device. References to other devices must be
unique if the device being referenced modifies any state. If the device is
read-only, then multiple references can be created.

### Compiled Value

The device tree declared above is transformed at compile time into the following
Rust code:

    let red_led = drivers::led::GPIOLed(gpio_4);
    red_led.initialize();

    let green_led = drivers::led::GPIOLed(gpio_4);
    green_len.initialize();

    let temperature = drivers::bit_bang::Temperature(gpio_0, gpio_1, gpio_2);
    temperature.initialize();

    let console = drivers::uart::Console(usart_3.as_UART());
    console.initialize(drivers::uart::Console::InitParams {
      baud_rate: 115200,
      data_bits: 8,
      parity: 0,
      stop_bits: 1
    });

    let flash = drivers::spi::MTDFlashM25P80(usart_0.as_SPIMaster);
    flash.initialize(drivers::spi::MTDFlashM25P80::InitParams {
      max_tx_lines: 2,
      max_rx_lines: 2,
      max_frequency: 50e6,
      size: 0x1000000,
      status_indicator: Some(&green_led);
    });

The following is an implementation aside:

  Note: We could do something like:

    flash.initialize(drivers::spi::MTDFlashM25P80::InitParams {
      max_tx_lines: 2,
      max_rx_lines: 2,
      max_frequency: 50e6,
      size: 0x1000000,
      .. Default::default()
    });

  So that the device tree implementors don't have to specify every parameter. In
  this case, when there is no LED, we can simply omit that parameter. I'm not
  sure this is the best way to go, though.

### Device Tree Syntax

The following grammar defines the device tree syntax:

    device_tree :=
      device+

    device :=
      ID':' PATH'('resource_requirement[',' resource_requirement]*) [block | ';']

    block :=
      '{' property* '}'

    property :=
      ID '=' VALUE;

    resource_requirement := [
      basic_resource_requirement
      | variant_resource_requitement
    ]

    basic_resource_requirement :=
      ID'@'VALUE

    variant_resource_requitement :=
      ID'('basic_resource_requirement')'

    ID, PATH := { as defined by Rust }


Platform Drivers
----------------

Here's an example:

    platform_tree!(s4mlxx,
      cpu@0 = CortexM4 {
        clock = 12e6;
        mpu_present = true;
        nvic = &nvic;

        pll = PLL {
          m = 50;
          n = 3;
          divisor = 4;
        };
      };

      gpio@[0..26] = GPIO;

      usart@0 = SPI_USART {
        base_address = 0x40024000;

        _variants = [
          UART, RS485, Modem, LINMaster, LINSlave, SPIMaster, SPISlave
        ];
      };

      usart@[1..3] = USART {
        base_address = 0x40024000;

        _variants = [
          UART, RS485, Modem, LINMaster, LINSlave
        ];
      };
    );

...which becomes...

    fn main() {
      use platform::s4mlxx;
      s4mlxx::platform_init_start();

      let cpu_0 = s4mlxx::platform::CortexM4 {
        clock: 12e6,
        mpu_present: true,
        nvic: &nvic,
        pll: s4mlxx::platform::CortexM4::PLL {
          m: 50,
          n: 3,
          divisor: 4
        }
      }

      let gpio_n = s4mlxx::platform::GPIO {
        location: n,
      };

      let usart_0 = s4mlxx::platform::SPI_USART {
        location: 0,
        base_address: 0x40024000
      };

      let usart_n = s4mlxx::platform::USART {
        location: n,
        base_address: 0x40024000
      };

      /* device init inlined here */

      s4mlxx::platform_init_done();
    }

