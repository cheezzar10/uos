#![no_std]

#[macro_export]
macro_rules! console_println {
	( $f:expr ) => ( { crate::console::print(format_args!(concat!($f, "\n"))); } );
	( $f:expr, $( $a:expr ), * ) => ( { crate::console::print(format_args!(concat!($f, "\n"), $( $a ), *)); } )
}

pub mod console;

pub mod task;

pub mod alloc;

pub mod lock;

pub mod vec;
