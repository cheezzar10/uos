use core::mem;
use core::ptr;
use core::usize;

#[link(name = "uos")]
extern {
	fn get_sp() -> *const u32;

	fn get_eflags() -> u32;

	fn get_cs() -> u32;

	fn syscall();
}

const TASK_STACK_SIZE: usize = 0x1000;
const STACK_PTR_MASK: u32 = !(TASK_STACK_SIZE - 1) as u32;

struct Task {
	tid: usize,
	cpu_state: TaskCpuState
}

#[repr(C)]
struct TaskCpuState {
	// TODO add ds and pdbr (also u32 should be changed to usize)
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

pub fn init(task_idx: usize) {
	unsafe {
		CUR_TASK_IDX = task_idx;

		// thread_save_cpu_state(&TASKS[CUR_TASK_IDX].cpu_state);
		let cur_task = &mut TASKS[CUR_TASK_IDX];

		// assuming that this function will be called for init only
		cur_task.tid = task_idx;

		let cur_sp: usize = get_sp() as usize;

		// calculating task stack pointer location by rounding current stack location to stack limit boundary
		let task_sp = ((cur_sp + (TASK_STACK_SIZE - 1)) & (!TASK_STACK_SIZE + 1)) - 4;

		console_println!("task {} stack ptr: {:x}", cur_task.tid, task_sp);
		cur_task.cpu_state.esp = task_sp as u32;
	}
}

unsafe fn find_free_task() -> Option<(usize, &'static mut Task)> {
	for (i, t) in TASKS.iter_mut().enumerate() {
		if t.tid == usize::MAX {
			return Some((i, t))
		}
	}

	None
}

unsafe fn get_stack_ptr() -> u32 {
	let mut min_stack_page: u32 = (usize::MAX as u32) & STACK_PTR_MASK;

	for t in TASKS.iter() {
		if t.tid != usize::MAX {
			let stack_page = t.cpu_state.esp & STACK_PTR_MASK;
			if stack_page < min_stack_page {
				min_stack_page = stack_page;
			}
		}
	}

	min_stack_page - 4
}

pub fn create(f: fn()) {
	console_println!("creating new task using task function: {:p}", f);

	unsafe {
		let new_task = find_free_task();

		if let Some((i, t)) = new_task {
			let new_task_state = &mut t.cpu_state;

			new_task_state.eip = task_wrapper as u32;
			new_task_state.esp = get_stack_ptr();

			t.tid = i;

			// placing task body function at the top of  the stack
			*(new_task_state.esp as *mut u32) = f as u32;
			// reserving space for bogus return value (task_wrapper function should never return)
			new_task_state.esp -= 4 * (mem::size_of::<u32>() as u32);

			// using current thread code segment and flags
			new_task_state.cs = get_cs();
			new_task_state.eflags = get_eflags();

			console_println!("new task tid: {}, state: {{ eip: {:x}, esp: {:x}, eflags: {:x} }}", t.tid, new_task_state.eip, new_task_state.esp, new_task_state.eflags);
		} else {
			console_println!("task descriptors limit exceded");
		}
	}
}

pub fn suspend() {
	unsafe {
		syscall();
	}
}

pub fn curr_task_id() -> usize {
	unsafe {
		TASKS[CUR_TASK_IDX].tid
	}
}

fn task_wrapper(task_fn: fn()) {
	console_println!("task tid: {} - started", curr_task_id());

	task_fn();

	console_println!("task tid: {} - exited", curr_task_id());

	unsafe {
		// marking current task structure as unused
		let cur_task: &mut Task = &mut TASKS[CUR_TASK_IDX];
		cur_task.tid = usize::MAX;
	}

	suspend();
}

#[no_mangle]
pub unsafe extern fn switch_task_and_get_new_stack_ptr() -> *const u8 {
	let cur_task_id = curr_task_id();

	let curr_task_cpu_state = &TASKS[CUR_TASK_IDX].cpu_state;
	console_println!("current task tid: {}, state: {{ eip: {:x}, esp: {:x}, eflags: {:x} }}", cur_task_id, 
			curr_task_cpu_state.eip, curr_task_cpu_state.esp, curr_task_cpu_state.eflags);

	for (i, t) in TASKS.iter_mut().enumerate() {
		if t.tid != cur_task_id && t.tid != usize::MAX {
			CUR_TASK_IDX = i;

			let next_task_cpu_state = &t.cpu_state;
			console_println!("switched to task tid: {}, state: {{ eip: {:x}, esp: {:x}, eflags: {:x} }}", t.tid, 
					next_task_cpu_state.eip, next_task_cpu_state.esp, next_task_cpu_state.eflags);
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

