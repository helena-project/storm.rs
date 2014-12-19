use hal::gpio;
use timer;
use hal::usart;
use hal::pm;

static LED : gpio::Pin = gpio::Pin { bus : gpio::Port::PORT2, pin: 10 };

const CLOCK : &'static str = "
Tick tock... tick tock...
        ____
     _.'_____`._
   .'.-'  12 `-.`.
  /,' 11      1 `.\\
 // 10      /   2 \\\\
;;         /       ::
|| 9  ----O      3 ||
::                 ;;
 \\\\ 8           4 //
  \\\\`. 7       5 ,'/
   '.`-.__6__.-'.'
    ((-._____.-))
    _))       ((_
   '--'       '--'

Welcome to Tock OS. Nothing but blinking lights for now...
";

fn set_led() {
    LED.set();
    timer::set_alarm(1 << 16, task::Task{f:set_led});
}

fn clear_led() {
    LED.clear();
    timer::set_alarm(1 << 16, clear_led);
}

pub fn init() {
    use hal::gpio::*;
    use hal::gpio::Port::*;
    use hal::gpio::PeripheralFunction::*;

    LED.make_output();
    LED.set();

    pm::enable_pba_clock(11);

    let uart = usart::USART::UART3;
    Pin {bus : PORT1, pin : 9}.set_peripheral_function(A);
    Pin {bus : PORT1, pin : 10}.set_peripheral_function(A);

    uart.init_uart();
    uart.set_baud_rate(115200);
    uart.enable_tx();

    uart.print(CLOCK);

    LED.clear();

    timer::setup();

    timer::set_alarm(1 << 15, set_led);
    timer::set_alarm(1 << 16, clear_led);
}

