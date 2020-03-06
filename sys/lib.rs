#![no_std]

pub mod console;

#[macro_export]
macro_rules! console_println {
	( $f:expr ) => ( { crate::console::print(format_args!(concat!($f, "\n"))); } );
	( $f:expr, $( $a:expr ), * ) => ( { crate::console::print(format_args!(concat!($f, "\n"), $( $a ), *)); } )
}

pub mod task;
