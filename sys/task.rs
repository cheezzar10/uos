use core::mem;
use core::ptr;
use core::usize;

#[link(name = "uos")]
extern {
	pub fn get_sp() -> *const u32;

	fn get_eflags() -> u32;

	fn get_cs() -> u32;

	fn syscall();
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

pub fn init(task_idx: usize) {
	unsafe {
		CUR_TASK_IDX = task_idx;

		// thread_save_cpu_state(&TASKS[CUR_TASK_IDX].cpu_state);
		let cur_task = &mut TASKS[CUR_TASK_IDX];

		// assuming that this function will be called for init only
		cur_task.tid = task_idx;
	}
}

pub fn create(f: fn()) {
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

			new_task_state.eip = task_wrapper as u32;
			// TODO add stack location func, 4k for each thread, growing down
			new_task_state.esp = 0x1effc;

			// placing task body function at the top of  the stack
			*(new_task_state.esp as *mut u32) = f as u32;
			// reserving space for bogus return value (task_wrapper function should never return)
			new_task_state.esp -= 4 * (mem::size_of::<u32>() as u32);

			console_println!("saved on stack task f: {:x}", *(new_task_state.esp as *mut u32).wrapping_add(2));

			// using current thread code segment and flags
			new_task_state.cs = get_cs();
			new_task_state.eflags = get_eflags();

			console_println!("new task tid: {}, state = eip: {:x}, eflags: {:x}, cs: {:x}", t.tid, new_task_state.eip, new_task_state.eflags, new_task_state.cs);
		} else {
			console_println!("task descriptors limit exceeded");
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

