#![no_std]
#![no_main]

#[macro_use]
extern crate uos;

use core::panic::PanicInfo;
use core::usize;
use core::ptr;

use uos::console;

#[link(name = "uos")]
extern {
	fn get_sp() -> *const u32;

	fn get_eflags() -> u32;

	fn get_cs() -> u32;

	fn register_interrupt_handler(vec_num: usize, handler: extern fn());

	fn register_interrupt_handler_with_err_code(vec_num: usize, handler: extern fn(err_code: usize));

	fn write_byte_to_port(byte: u32, port_num: u32);

	fn read_byte_from_port(port_num: u32) -> u32;

	fn load_idt();

	fn interrupts_enable();

	fn syscall();
}

const DIVIDE_ERROR_INTR_VEC_NUM: usize = 0;
const GENERAL_PROTECTION_ERR_VEC_NUM: usize = 13;

const TIMER_INTR_VEC_NUM: usize = 32;
const KBD_INTR_VEC_NUM: usize = 33;

const MASTER_ICW1_IOPORT_NUM: u32 = 0x20;
const SLAVE_ICW1_IOPORT_NUM: u32 = 0xa0;

const MASTER_ICW2_IOPORT_NUM: u32 = MASTER_ICW1_IOPORT_NUM + 1;
const SLAVE_ICW2_IOPORT_NUM: u32 = SLAVE_ICW1_IOPORT_NUM + 1;

const CMOS_RAM_CMD_PORT_NUM: u32 = 0x70;
const CMOS_RAM_DATA_PORT_NUM: u32 = 0x71;

const KBD_DATA_IOPORT_NUM: u32 = 0x60;
const KEY_RELEASED_BIT_MASK: u32 = 0x80;
const KEY_SCAN_CODE_MASK: u32 = !KEY_RELEASED_BIT_MASK;

#[no_mangle]
pub unsafe extern fn _start() {
	init();

	loop {}
}

struct Task {
	tid: usize,
	cpu_state: TaskCpuState
}

#[repr(C)]
struct TaskCpuState {
	// TODO add ds and pdbr
	edi: u32,
	esi: u32,
	ebp: u32,
	esp: u32,
	ebx: u32,
	edx: u32,
	ecx: u32,
	eax: u32,
	eip: u32,
	cs: u32,
	eflags: u32
}

static NULL_TASK_CPU_STATE: TaskCpuState = TaskCpuState { edi: 0, esi: 0, ebp: 0, esp: 0, ebx: 0, edx: 0, ecx: 0, eax: 0, eip: 0, cs: 0, eflags: 0 };

static mut TASKS: [Task; 2] = [
	Task { tid: usize::MAX, cpu_state: TaskCpuState { ..NULL_TASK_CPU_STATE } },
	Task { tid: usize::MAX, cpu_state: TaskCpuState { ..NULL_TASK_CPU_STATE } }
];

static mut CUR_TASK_IDX: usize = 0;

unsafe fn init() {
	console::clear();

	console_println!("stack ptr: {:p}", get_sp());

	// registering mandatory interrupt handlers
	register_interrupt_handler(DIVIDE_ERROR_INTR_VEC_NUM, divide_error);
	register_interrupt_handler_with_err_code(GENERAL_PROTECTION_ERR_VEC_NUM, general_protection_error);

	// registering HW interrupt handlers
	register_interrupt_handler(TIMER_INTR_VEC_NUM, timer_intr_handler);
	register_interrupt_handler(KBD_INTR_VEC_NUM, kbd_intr_handler);

	// programmable interrupt controller initialization
	init_pic();

	init_ata_hdd();

	// performing first task initialization
	thread_init(0);

	thread_create(idle_thread);

	console_println!("task tid: {} - yielding", get_current_task_id());
	thread_yield();

	console_println!("task tid: {} - resumed", get_current_task_id());
}

unsafe fn init_pic() {
	// ICW1 edge triggered mode
	write_byte_to_port(0x11, MASTER_ICW1_IOPORT_NUM);
	write_byte_to_port(0x11, SLAVE_ICW1_IOPORT_NUM);

	// ICW2 assigning interrupt numbers to master and slave controller 
	// ( vectors 32-39 to master and 40-47 to slave )
	write_byte_to_port(32, MASTER_ICW2_IOPORT_NUM);
	write_byte_to_port(40, SLAVE_ICW2_IOPORT_NUM);

	// ICW3 connecting slave controller to master IRQ2
	write_byte_to_port(0x4, MASTER_ICW2_IOPORT_NUM);
	write_byte_to_port(0x2, SLAVE_ICW2_IOPORT_NUM);

	// ICW4 x86 processing mode, normal EOI
	write_byte_to_port(0x1, MASTER_ICW2_IOPORT_NUM);
	write_byte_to_port(0x1, SLAVE_ICW2_IOPORT_NUM );

	// OCW1 unmasking all interrupts
	write_byte_to_port(0, MASTER_ICW2_IOPORT_NUM);
	write_byte_to_port(0, SLAVE_ICW2_IOPORT_NUM);

	load_idt();

	interrupts_enable();
}

unsafe fn init_ata_hdd() {
	// checking disk type
	write_byte_to_port(0x12, CMOS_RAM_CMD_PORT_NUM);

	let hdd_info = read_byte_from_port(CMOS_RAM_DATA_PORT_NUM);
	if hdd_info & 0xf0 == 0xf0 {
		// getting information about first hard disk from specific register
		write_byte_to_port(0x19, CMOS_RAM_CMD_PORT_NUM);
		let hda_info = read_byte_from_port(CMOS_RAM_DATA_PORT_NUM);

		console_println!("hda info: {:x}", hda_info);
	}
}

unsafe fn end_of_intr_handling() {
	write_byte_to_port(0x20, MASTER_ICW1_IOPORT_NUM);
	write_byte_to_port(0x20, SLAVE_ICW1_IOPORT_NUM);
}

extern fn divide_error() {
	console_println!("divide error");
}

extern fn general_protection_error(err_code: usize) {
	console_println!("general protection error: {:x}", err_code);

	loop {}
}

extern fn timer_intr_handler() {
	unsafe {
		end_of_intr_handling();
	}
}

extern fn kbd_intr_handler() {
	unsafe {
		let key_scan_code = read_byte_from_port(KBD_DATA_IOPORT_NUM);

		// deciding was it key press or key release
		if (key_scan_code & KEY_RELEASED_BIT_MASK) == 0 {
			// bit 7 is clear - key pressed
			console_println!("key pressed: {:x}", key_scan_code);
		} else {
			// bit 7 is set - key released
			console_println!("key released: {:x}", key_scan_code & KEY_SCAN_CODE_MASK);
		}

		end_of_intr_handling();
	}
}

// TODO move to dedicated threading library
fn thread_create(f: fn()) {
	console_println!("creating new task using task function: {:p}", f);

	unsafe {
		let mut new_task: Option<(usize, &mut Task)> = None;
		for (i, t) in TASKS.iter_mut().enumerate() {
			// searching for unused Task structure
			if t.tid == usize::MAX {
				new_task = Some((i, &mut TASKS[i]));

				break;
			}
		}

		if let Some((i, t)) = new_task {
			let new_task_state = &mut t.cpu_state;
			t.tid = i;

			new_task_state.eip = f as u32;
			// TODO add stack location func, 4k for each thread, growing down
			new_task_state.esp = 0x1effc;

			// using current thread code segment and flags
			new_task_state.cs = get_cs();
			new_task_state.eflags = get_eflags();

			console_println!("new task tid: {}, state = eip: {:x}, eflags: {:x}, cs: {:x}", t.tid, new_task_state.eip, new_task_state.eflags, new_task_state.cs);
		} else {
			console_println!("task descriptors limit exceeded");
		}
	}
}

fn thread_yield() {
	unsafe {
		syscall();
	}
}

fn thread_init(task_idx: usize) {
	unsafe {
		CUR_TASK_IDX = task_idx;

		// thread_save_cpu_state(&TASKS[CUR_TASK_IDX].cpu_state);
		let cur_task = &mut TASKS[CUR_TASK_IDX];

		// assuming that this function will be called for init only
		cur_task.tid = task_idx;
	}
}

fn get_current_task_id() -> usize {
	unsafe {
		TASKS[CUR_TASK_IDX].tid
	}
}

fn idle_thread() {
	console_println!("task tid: {} - running", get_current_task_id());

	for _ in 0..500000 {
	}

	console_println!("task tid: {} - yielding", get_current_task_id());

	thread_yield();

	console_println!("idle: exiting");
}

#[no_mangle]
pub unsafe extern fn switch_task_and_get_new_stack_ptr() -> *const u8 {
	let cur_task_id = TASKS[CUR_TASK_IDX].tid;
	console_println!("current task tid: {}, cpu state ptr: {:p}", cur_task_id, &TASKS[CUR_TASK_IDX].cpu_state);

	for (i, t) in TASKS.iter_mut().enumerate() {
		if t.tid != cur_task_id && t.tid != usize::MAX {
			CUR_TASK_IDX = i;

			console_println!("switched to task tid: {}, cpu state ptr: {:p}", t.tid, &t.cpu_state);
			break;
		}
	}

	(TASKS[CUR_TASK_IDX].cpu_state.esp - 32) as *const u8
}

#[no_mangle]
pub unsafe extern fn print_task_state() {
	let rt_state = &TASKS[CUR_TASK_IDX].cpu_state;
	console_println!("resumed task cpu state = eip: {:x}, cs: {:x}, esp: {:x}, eflags: {:x}", rt_state.eip, rt_state.cs, rt_state.esp, rt_state.eflags);

	let stack_ptr = get_sp();
	console_println!("stack ptr: {:p}", stack_ptr);

	loop {}
}

#[no_mangle]
pub unsafe extern fn save_current_task_state(task_cpu_state_ptr: *const u8) {
	ptr::copy_nonoverlapping(task_cpu_state_ptr as *const TaskCpuState, &mut TASKS[CUR_TASK_IDX].cpu_state, 1);
}

#[no_mangle]
pub unsafe extern fn restore_current_task_state(task_cpu_state_ptr: *mut u8) {
	ptr::copy_nonoverlapping(&TASKS[CUR_TASK_IDX].cpu_state, task_cpu_state_ptr as *mut TaskCpuState, 1);
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
	// making some sign that we reached this place
	loop {}
}

// the following functions make linker happy
#[no_mangle]
extern fn rust_eh_personality() {}

#[no_mangle]
extern fn _Unwind_Resume() {}
