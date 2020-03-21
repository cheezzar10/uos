use crate::pio;

#[link(name = "uos")]
extern {
	pub fn register_handler(vec_num: usize, handler: extern fn());

	pub fn register_handler_with_err_code(vec_num: usize, handler: extern fn(err_code: usize));

	pub fn load_idt();

	pub fn intr_enable();
}

const MASTER_ICW1_IOPORT_NUM: u32 = 0x20;
const SLAVE_ICW1_IOPORT_NUM: u32 = 0xa0;

const MASTER_ICW2_IOPORT_NUM: u32 = MASTER_ICW1_IOPORT_NUM + 1;
const SLAVE_ICW2_IOPORT_NUM: u32 = SLAVE_ICW1_IOPORT_NUM + 1;

pub unsafe fn init_pic() {
	// ICW1 edge triggered mode
	pio::out_byte(0x11, MASTER_ICW1_IOPORT_NUM);
	pio::out_byte(0x11, SLAVE_ICW1_IOPORT_NUM);

	// ICW2 assigning interrupt numbers to master and slave controller 
	// ( vectors 32-39 to master and 40-47 to slave )
	pio::out_byte(32, MASTER_ICW2_IOPORT_NUM);
	pio::out_byte(40, SLAVE_ICW2_IOPORT_NUM);

	// ICW3 connecting slave controller to master IRQ2
	pio::out_byte(0x4, MASTER_ICW2_IOPORT_NUM);
	pio::out_byte(0x2, SLAVE_ICW2_IOPORT_NUM);

	// ICW4 x86 processing mode, normal EOI
	pio::out_byte(0x1, MASTER_ICW2_IOPORT_NUM);
	pio::out_byte(0x1, SLAVE_ICW2_IOPORT_NUM );

	// OCW1 unmasking all interrupts
	pio::out_byte(0, MASTER_ICW2_IOPORT_NUM);
	pio::out_byte(0, SLAVE_ICW2_IOPORT_NUM);

	load_idt();

	intr_enable();
}

pub unsafe fn eoi() {
	pio::out_byte(0x20, MASTER_ICW1_IOPORT_NUM);
	pio::out_byte(0x20, SLAVE_ICW1_IOPORT_NUM);
}
