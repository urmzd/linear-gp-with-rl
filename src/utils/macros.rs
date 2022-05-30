#[macro_export]
macro_rules! executable {
    ( $fn_name: ident,  $op: tt, $val: expr) => {
        fn $fn_name<'r>(registers: &'r mut [RegisterValue], data: &[RegisterValue]) -> &'r [RegisterValue] {
            assert_eq!(registers.len(), data.len());

            for index in 0..registers.len() {
                registers[index] = registers[index] $op $val
            }

            return registers;
        }
    };

    ( $fn_name: ident, $op: tt) => {
        fn $fn_name<'r>(registers: &'r mut [RegisterValue], data: &[RegisterValue]) -> &'r [RegisterValue] {
            assert_eq!(registers.len(), data.len());

            for index in 0..registers.len() {
                registers[index] = registers[index] $op data[index]
            }

            return registers;
        }
    };

}

#[macro_export]
macro_rules! executables {
    ($const_name: ident, $($fn_tail: path),*) => {
        pub const $const_name: &'static [AnyExecutable] = &[
            $(
                AnyExecutable(stringify!($fn_tail), $fn_tail),
            )*
        ];
    };
}
