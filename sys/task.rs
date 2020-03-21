use core::mem;
use core::ptr;
use core::usize;

use crate::lock;
use crate::vec;

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

type TaskQueue = (Option<Task>, vec::Vec<Task>);

static TASKS: lock::Mutex<TaskQueue> = lock::Mutex::new((None, vec::Vec::new()));

pub fn init_curr_task(ttid: usize) {
	unsafe {
		let mut cur_task = Task {
			tid: ttid,
			cpu_state: TaskCpuState {
				..NULL_TASK_CPU_STATE
			}
		};

		let cur_sp: usize = get_sp() as usize;

		// calculating task stack pointer location by rounding current stack location to stack limit boundary
		let task_sp = ((cur_sp + (TASK_STACK_SIZE - 1)) & (!TASK_STACK_SIZE + 1)) - 4;

		cur_task.cpu_state.esp = task_sp as u32;

		let mut tasks_guard = TASKS.lock();
		let (curr_task, _): &mut (Option<Task>, vec::Vec<Task>) = &mut *tasks_guard;

		*curr_task = Some(cur_task);
	}
}

fn get_max_tid() -> usize {
	let mut max_tid: usize = 0;

	let tasks_guard = TASKS.lock();
	let (_, tasks) = &*tasks_guard;
	
	for t in tasks.iter() {
		if t.tid > max_tid {
			max_tid = t.tid;
		}
	}

	max_tid
}

fn get_stack_ptr() -> u32 {
	let tasks_guard = TASKS.lock();
	let (curr_task, tasks) = &*tasks_guard;

	let mut min_stack_page: u32 = if let Some(cur_task) = curr_task {
		cur_task.cpu_state.esp
	} else {
		console_println!("new task stack ptr calculation failed - current task not set");

		loop {}
	} & STACK_PTR_MASK;

	for t in tasks.iter() {
		let stack_page = t.cpu_state.esp & STACK_PTR_MASK;
		if stack_page < min_stack_page {
			min_stack_page = stack_page;
		}
	}

	min_stack_page - 4
}

pub fn create(f: fn()) {
	unsafe {
		let mut new_task = Task {
			tid: get_max_tid() + 1,
			cpu_state: TaskCpuState {
				..NULL_TASK_CPU_STATE
			}
		};

		let new_task_state = &mut new_task.cpu_state;

		new_task_state.eip = task_wrapper as u32;
		new_task_state.esp = get_stack_ptr();

		// placing task body function at the top of  the stack
		*(new_task_state.esp as *mut u32) = f as u32;
		// reserving space for bogus return value (task_wrapper function should never return)
		new_task_state.esp -= 4 * (mem::size_of::<u32>() as u32);

		// using current thread code segment and flags
		new_task_state.cs = get_cs();
		new_task_state.eflags = get_eflags();

		let mut tasks_guard = TASKS.lock();
		let (_, tasks) = &mut *tasks_guard;

		// placing new tasks to tasks queue
		tasks.push(new_task);
	}
}

pub fn suspend() {
	unsafe {
		syscall();
	}
}

pub fn curr_task_id() -> usize {
	let tasks_guard = TASKS.lock();
	let (curr_task, _) = &*tasks_guard;

	if let Some(ct) = curr_task {
		ct.tid
	} else {
		console_println!("current task not set");

		usize::MAX
	}
}

fn task_wrapper(task_fn: fn()) {
	task_fn();

	reset_curr_task();

	suspend();

	// completed task execution should never be resumed
	console_println!("error: completed task resumed");

	loop {}
}

fn reset_curr_task() {
	let mut tasks_guard = TASKS.lock();
	let (curr_task, _) = &mut *tasks_guard;

	*curr_task = None;
}

#[no_mangle]
pub unsafe extern fn switch_task_and_get_new_stack_ptr() -> *const u8 {
	let mut tasks_guard = TASKS.lock();
	let (curr_task, tasks) = &mut *tasks_guard;

	let task_queue_head = tasks.pop();
	if let Some(next_task) = task_queue_head {

		if let Some(cur_task) = curr_task {
			// placing current task to the end of the task queue
			tasks.push(Task {
				tid: cur_task.tid,
				cpu_state: TaskCpuState {
					..cur_task.cpu_state
				}
			});
		}

		let next_task_esp = next_task.cpu_state.esp;

		*curr_task = Some(next_task);

		(next_task_esp - 32) as *const u8
	} else {
		if let Some(cur_task) = curr_task {
			let rv = (cur_task.cpu_state.esp - 32) as *const u8;

			rv
		} else {
			console_println!("failed to switch task: current task not set and task queue is empty");

			loop {}
		}
	}
}

#[no_mangle]
pub unsafe extern fn save_current_task_state(task_cpu_state_ptr: *const u8) {
	let mut tasks_guard = TASKS.lock();
	let (curr_task, _) = &mut *tasks_guard;

	if let Some(cur_task) = curr_task {
		ptr::copy_nonoverlapping(task_cpu_state_ptr as *const TaskCpuState, &mut cur_task.cpu_state, 1);
	}
}

#[no_mangle]
pub unsafe extern fn restore_current_task_state(task_cpu_state_ptr: *mut u8) {
	let tasks_guard = TASKS.lock();
	let (curr_task, _) = &*tasks_guard;

	if let Some(cur_task) = curr_task {
		ptr::copy_nonoverlapping(&cur_task.cpu_state, task_cpu_state_ptr as *mut TaskCpuState, 1);
	} else {
		console_println!("current task not set - failed to restore task state");

		loop {}
	}
}

