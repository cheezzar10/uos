#[link(name = "uos")]
extern {
	pub fn out_byte(byte: u32, port_num: u32);

	pub fn in_byte(port_num: u32) -> u32;
}
