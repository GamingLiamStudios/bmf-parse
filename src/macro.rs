use crate::base::*;
use std::fmt::Debug;

// MP4 impl helpers
pub(crate) trait Mp4BoxTrait: Debug {
    const TYPE: u32; // Should be 4 bytes/ASCII chars

    fn parse_full(input: &[u8], state: &mut ParserState) -> Self;
    fn parse(input: &[u8], state: &mut ParserState, header: &Option<(u8, u32)>) -> Self;

    fn write_full(&self, output: &mut Vec<u8>);
    fn write(&self, output: &mut Vec<u8>);
}

pub(crate) fn read_header<'a>(input: &'a [u8], state: &mut ParserState) -> (u32, &'a [u8]) {
    let size = read(input, state, 4).unwrap();
    let size = u32::from_be_bytes([size[0], size[1], size[2], size[3]]);

    let type_ = read(input, state, 4).unwrap();
    let type_ = u32::from_ne_bytes([type_[0], type_[1], type_[2], type_[3]]);

    (type_, read(input, state, size as usize - 8).unwrap())
}

pub(crate) fn read_fullbox_header(input: &[u8], state: &mut ParserState) -> (u8, u32) {
    let version = read(input, state, 1).unwrap()[0];
    let flags = read(input, state, 3).unwrap();
    let flags = u32::from_be_bytes([0, flags[0], flags[1], flags[2]]);

    (version, flags)
}

macro_rules! mp4box_gen {
    // Read types
    { @read $input:ident $state:ident $header:ident; u8 } => {
        read($input, $state, 1).unwrap()[0]
    };
    { @read $input:ident $state:ident $header:ident; u16 } => {
        {
            let slice = read($input, $state, 2).unwrap();
            u16::from_be_bytes([slice[0], slice[1]])
        }
    };
    { @read $input:ident $state:ident $header:ident; u32 } => {
        {
            let slice = read($input, $state, 4).unwrap();
            u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]])
        }
    };
    { @read $input:ident $state:ident $header:ident; u64 } => {
        {
            let slice = read($input, $state, 8).unwrap();
            u64::from_be_bytes([slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7]])
        }
    };
    { @read $input:ident $state:ident $header:ident; i8 } => {
        read($input, $state, 1).unwrap()[0] as i8
    };
    { @read $input:ident $state:ident $header:ident; i16 } => {
        {
            let slice = read($input,$state, 2).unwrap();
            i16::from_be_bytes([slice[0], slice[1]])
        }
    };
    { @read $input:ident $state:ident $header:ident; i32 } => {
        {
            let slice = read($input, $state, 4).unwrap();
            i32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]])
        }
    };
    { @read $input:ident $state:ident $header:ident; i64 } => {
        {
            let slice = read($input, $state, 8).unwrap();
            i64::from_be_bytes([slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7]])
        }
    };
    { @read $input:ident $state:ident $header:ident; f32 } => {
        {
            let slice = read($input, $state, 4).unwrap();
            f32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]])
        }
    };
    { @read $input:ident $state:ident $header:ident; f64 } => {
        {
            let slice = read($input, $state, 8).unwrap();
            f64::from_be_bytes([slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7]])
        }
    };
    { @read $input:ident $state:ident $header:ident; [$type:tt; $n:expr] } => { // Might also work with sizes defined by variables
        {
            let mut array = [0; $n];
            for i in 0..$n {
                array[i] = mp4box_gen! { @read $input $state $header; $type };
            }
            array
        }
    };
    { @read $input:ident $state:ident $header:ident; Mp4Box } => {
        parse_box($input, $state).unwrap()
    };
    { @read $input:ident $state:ident $header:ident; String } => {
        {
            // Read CString in utf8
            let mut string = String::new();

            let mut bytes = Vec::new();
            while let Some(byte) = read($input, $state, 1) {
                if byte[0] == 0 {
                    break;
                }

                // Read utf8 character
                match byte[0] {
                    0..=0x7F => {
                        // 1 byte
                        bytes.push(byte[0]);
                    },
                    0xC0..=0xDF => {
                        // 2 bytes
                        bytes.push(byte[0]);
                        bytes.push(read($input, $state, 1).unwrap()[0]);
                    },
                    0xE0..=0xEF => {
                        // 3 bytes
                        bytes.push(byte[0]);
                        bytes.push(read($input, $state, 1).unwrap()[0]);
                        bytes.push(read($input, $state, 1).unwrap()[0]);
                    },
                    0xF0..=0xF7 => {
                        // 4 bytes
                        bytes.push(byte[0]);
                        bytes.push(read($input, $state, 1).unwrap()[0]);
                        bytes.push(read($input, $state, 1).unwrap()[0]);
                        bytes.push(read($input, $state, 1).unwrap()[0]);
                    },
                    _ => panic!("Invalid utf8 character"),
                }
            }

            // Convert to string
            string.push_str(std::str::from_utf8(&bytes).unwrap());

            string
        }
    };


    // Generic catch-all for metastructs
    { @read $input:ident $state:ident $header:ident; $($type:tt)* } => {
        //panic!("Unsupported type: {}", stringify!($type))
        <$($type)*>::parse($input, $state, &$header)
    };

    // Write Types
    { @write $output:ident $($item:ident).+; &u8 } => {
        $output.push(*$($item).+);
    };
    { @write $output:ident $($item:ident).+; u8 } => {
        $output.push($($item).+);
    };
    { @write $output:ident $($item:ident).+; &u16 } => {
        $output.extend_from_slice(&u16::to_be_bytes(*$($item).+));
    };
    { @write $output:ident $($item:ident).+; u16 } => {
        $output.extend_from_slice(&u16::to_be_bytes($($item).+));
    };
    { @write $output:ident $($item:ident).+; &u32 } => {
        $output.extend_from_slice(&u32::to_be_bytes(*$($item).+));
    };
    { @write $output:ident $($item:ident).+; u32 } => {
        $output.extend_from_slice(&u32::to_be_bytes($($item).+));
    };
    { @write $output:ident $($item:ident).+; &u64 } => {
        $output.extend_from_slice(&u64::to_be_bytes(*$($item).+));
    };
    { @write $output:ident $($item:ident).+; u64 } => {
        $output.extend_from_slice(&u64::to_be_bytes($($item).+));
    };
    { @write $output:ident $($item:ident).+; i8 } => {
        $output.push($($item).+ as u8);
    };
    { @write $output:ident $($item:ident).+; i16 } => {
        $output.extend_from_slice(&i16::to_be_bytes($($item).+));
    };
    { @write $output:ident $($item:ident).+; &i32 } => {
        $output.extend_from_slice(&i32::to_be_bytes(*$($item).+));
    };
    { @write $output:ident $($item:ident).+; i32 } => {
        $output.extend_from_slice(&i32::to_be_bytes($($item).+));
    };
    { @write $output:ident $($item:ident).+; i64 } => {
        $output.extend_from_slice(&i64::to_be_bytes($($item).+));
    };
    { @write $output:ident $($item:ident).+; f32 } => {
        $output.extend_from_slice(&f32::to_be_bytes($($item).+));
    };
    { @write $output:ident $($item:ident).+; f64 } => {
        $output.extend_from_slice(&f64::to_be_bytes($($item).+));
    };
    { @write $output:ident $($item:ident).+; [$type:tt; $n:expr] } => { // Might also work with sizes defined by variables
        for entry in $($item).+ {
            mp4box_gen! { @write $output entry; $type };
        }
    };
    { @write $output:ident $($item:ident).+; &[$type:tt; $n:expr] } => { // Might also work with sizes defined by variables
        for entry in $($item).+ {
            let entry = *entry;
            mp4box_gen! { @write $output entry; $type };
        }
    };
    { @write $output:ident $($item:ident).+; String } => {
        $output.extend_from_slice($($item).+.as_bytes());
    };

    // Generic catch-all for metastructs
    { @write $output:ident $($item:ident).+; $($type:tt)* } => {
        $($item).+.write($output);
    };

    // Condition expansion
    // Final layer Read
    { // vec w/ no length
        @cond read
        $input:ident $state:ident $header:ident;
        [Vec<$type:tt, Remain>],
    } => {
        {
            let length = ($input.len() - $state.offset) / std::mem::size_of::<$type>();
            let mut vec = Vec::with_capacity(length);
            for _ in 0..length {
                vec.push(mp4box_gen! { @read $input $state $header; $type });
            }
            vec
        }
    };
    { // vec w/ non-option length
        @cond read
        $input:ident $state:ident $header:ident;
        [Vec<$type:tt, $length:ident>],
    } => {
        {
            let length = $length as usize;
            let mut vec = Vec::with_capacity(length);
            for _ in 0..length {
                vec.push(mp4box_gen! { @read $input $state $header; $type });
            }
            vec
        }
    };
    { // vec w/ option length
        @cond read
        $input:ident $state:ident $header:ident;
        [Vec<$type:tt, Option<[$($length:tt)*]>>],
    } => {
        {
            let length = ($($length)*) as usize;
            let mut vec = Vec::with_capacity(length);
            for _ in 0..length {
                vec.push(mp4box_gen! { @read $input $state $header; $type });
            }
            vec
        }
    };
    { // Simple type
        @cond read $($opt:ident)*;
        [$type:tt],
    } => {
        mp4box_gen! { @read $($opt)*; $type }
    };

    // Iterate
    { // Either
        @cond read $($opt:ident)*;
        [Either<$type:tt, [$($btype:tt)*]>],
        $cond:expr,
        $($rest:tt)*
    } => {
        if $cond {
            Either::A(mp4box_gen! { @cond read $($opt)*; [$type], })
        } else {
            Either::B(mp4box_gen! { @cond read $($opt)*; [$($btype)*], $($rest)* })
        }
    };
    { // Option
        @cond read $($opt:ident)*;
        [Option<[$($type:tt)*]>],
        $cond:expr,
        $($rest:tt)*
    } => {
        if $cond {
            Some(mp4box_gen! { @cond read $($opt)*; [$($type)*], $($rest)* })
        } else {
            None
        }
    };

    // Write
    { // borrow vec w/ non-option length
        @cond write
        $output:ident $($item:ident).+;
        [&Vec<$type:tt, $length:ident>],
    } => {
        for entry in $($item).+ {
            mp4box_gen! { @write $output entry; $type };
        }
    };
    { // borrow vec w/ option length
        @cond write
        $output:ident $($item:ident).+;
        [&Vec<$type:tt, Option<[$($length:tt)*]>>],
    } => {
        for entry in $($item).+ {
            mp4box_gen! { @write $output entry; $type };
        }
    };
    { // vec w/ non-option length
        @cond write
        $output:ident $($item:ident).+;
        [Vec<$type:tt, $length:ident>],
    } => {
        for entry in &$($item).+ {
            mp4box_gen! { @write $output entry; &$type };
        }
    };
    { // vec w/ option length
        @cond write
        $output:ident $($item:ident).+;
        [Vec<$type:tt, Option<[$($length:tt)*]>>],
    } => {
        for entry in $($item).+ {
            mp4box_gen! { @write $output entry; $type };
        }
    };
    { // Simple type
        @cond write
        $output:ident $($item:ident).+;
        [$type:tt],
    } => {
        mp4box_gen! { @write $output $($item).+; $type }
    };
    { // Simple type
        @cond write
        $output:ident $($item:ident).+;
        [&$type:tt],
    } => {
        mp4box_gen! { @write $output $($item).+; &$type }
    };

    // Iterate
    { // borrow Either
        @cond write
        $output:ident $($item:ident).+;
        [&Either<$type:tt, [$($btype:tt)*]>],
        $cond:expr,
        $($rest:tt)*
    } => {
        match $($item).+ {
            Either::A(item) => mp4box_gen! { @cond write $output item; [&$type], },
            Either::B(item) => mp4box_gen! { @cond write $output item; [&$($btype)*], $($rest)* },
        }
    };
    { // Either
        @cond write
        $output:ident $($item:ident).+;
        [Either<$type:tt, [$($btype:tt)*]>],
        $cond:expr,
        $($rest:tt)*
    } => {
        match $($item).+ {
            Either::A(item) => mp4box_gen! { @cond write $output item; [$type], },
            Either::B(item) => mp4box_gen! { @cond write $output item; [$($btype)*], $($rest)* },
        }
    };
    { // borrow Option
        @cond write
        $output:ident $($item:ident).+;
        [&Option<[$($type:tt)*]>],
        $cond:expr,
        $($rest:tt)*
    } => {
        if let Some(item) = $($item).+ {
            mp4box_gen! { @cond write $output item; [&$($type)*], $($rest)* }
        }
    };
    { // Option
        @cond write
        $output:ident $($item:ident).+;
        [Option<[$($type:tt)*]>],
        $cond:expr,
        $($rest:tt)*
    } => {
        if let Some(item) = &$($item).+ {
            mp4box_gen! { @cond write $output item; [&$($type)*], $($rest)* }
        }
    };

    // Struct construction
    // Full Type
    {
        @expand $version:ident $flags:ident;
        $name:ident Full {}; // No fields remaining
        [
            $([
                $field:ident, [$($ftype:tt)*]&[$($ctype:tt)*]; $($cond:expr,)*
            ],)+ // Expanded fields
        ]
    } => {
        use $crate::base::*;
        use $crate::r#macro::*;

        paste::paste! {
            #[derive(Debug)]
            pub struct [<Box $name>] {
                header: Option<(u8, u32)>,
                $(
                    pub $field: $($ftype)*,
                )+
            }
            impl [<Box $name>] {
                const IDSTR: &[u8] = bstringify::bstringify!([<$name:lower>]);
            }
            impl Mp4BoxTrait for [<Box $name>] {
                const TYPE: u32 = u32::from_ne_bytes([Self::IDSTR[0], Self::IDSTR[1], Self::IDSTR[2], Self::IDSTR[3]]);

                fn parse_full(input: &[u8], state: &mut ParserState) -> Self {
                    let (version, flags) = read_fullbox_header(input, state);
                    let header = Some((version, flags));

                    let mut instance = Self::parse(input, state, &header);
                    instance.header = header;
                    instance
                }

                fn parse(input: &[u8], state: &mut ParserState, header: &Option<(u8, u32)>) -> Self {
                    let uwh = header.unwrap();
                    let ($version, $flags) = uwh;

                    // Split out into fields so they can reference each other
                    $(
                        let $field = mp4box_gen! {
                            @cond read input state header;
                            [$($ctype)*],
                            $($cond,)*
                        };
                    )+

                    Self {
                        header: None,
                        $(
                            $field,
                        )+
                    }
                }

                fn write_full(&self, output: &mut Vec<u8>) {
                    let mut data = Vec::new();
                    self.write(&mut data);

                    // Write header
                    let size = data.len() as u32 + 12;
                    output.extend_from_slice(&u32::to_be_bytes(size)); // Size
                    output.extend_from_slice(&u32::to_ne_bytes(Self::TYPE)); // Type

                    // Version
                    let (version, flags) = self.header.unwrap();
                    output.push(version); // Version (1 byte)
                    output.extend_from_slice(&u32::to_be_bytes(flags)[1..]); // Flags (3 bytes)

                    output.extend(data);
                }

                fn write(&self, output: &mut Vec<u8>) {
                    $(
                        mp4box_gen! {
                            @cond write output self.$field;
                            [$($ctype)*],
                            $($cond,)*
                        };
                    )+
                }
            }
        }
    };

    // Base Type
    {
        @expand $version:ident $flags:ident;
        $name:ident {}; // No fields remaining
        [
            $([
                $field:ident, [$($ftype:tt)*]&[$($ctype:tt)*]; $($cond:expr,)*
            ],)+ // Expanded fields
        ]
    } => {
        use $crate::base::*;
        use $crate::r#macro::*;

        paste::paste! {
            #[derive(Debug)]
            pub struct [<Box $name>] {
                $(
                    pub $field: $($ftype)*,
                )+
            }
            impl [<Box $name>] {
                const IDSTR: &[u8] = bstringify::bstringify!([<$name:lower>]);
            }
            impl Mp4BoxTrait for [<Box $name>] {
                const TYPE: u32 = u32::from_ne_bytes([Self::IDSTR[0], Self::IDSTR[1], Self::IDSTR[2], Self::IDSTR[3]]);

                fn parse_full(input: &[u8], state: &mut ParserState) -> Self {
                    Self::parse(input, state, &None)
                }

                fn parse(input: &[u8], state: &mut ParserState, _: &Option<(u8, u32)>) -> Self {
                    // Split out into fields so they can reference each other
                    $(
                        let $field = mp4box_gen! {
                            @cond read input state header;
                            [$($ctype)*],
                            $($cond,)*
                        };
                    )+

                    Self {
                        $(
                            $field,
                        )+
                    }
                }

                fn write_full(&self, output: &mut Vec<u8>) {
                    let mut data = Vec::new();
                    self.write(&mut data);

                    // Write header
                    let size = data.len() as u32 + 8;
                    output.extend_from_slice(&u32::to_be_bytes(size)); // Size
                    output.extend_from_slice(&u32::to_ne_bytes(Self::TYPE)); // Type
                    output.extend(data);
                }

                fn write(&self, output: &mut Vec<u8>) {
                    $(
                        mp4box_gen! {
                            @cond write output self.$field;
                            [$($ctype)*],
                            $($cond,)*
                        };
                    )+
                }
            }
        }
    };

    // Conditional expansion
    { // Complete
        @cond_expand $version:ident $flags:ident;
        $name:ident $($stype:ident)? {
            [],
            $($rest:tt)* // Remaining fields
        }; [$($prev:tt)*], // Already expanded fields
        [$field:ident, [$($ftype:tt)*]&[$($ctype:tt)*]; $($done:tt)*] // Current
    } => {
        mp4box_gen! {
            @expand $version $flags;
            $name $($stype)? {
                $($rest)* // Remaining fields
            }; [
                $($prev)* // Already expanded fields
                [$field, [$($ftype)*]&[$($ctype)*]; $($done)*],
            ]
        }
    };
    { // Complete with condition
        @cond_expand $version:ident $flags:ident;
        $name:ident $($stype:ident)? {
            [] [if $cond:expr],
            $($rest:tt)* // Remaining fields
        }; [$($prev:tt)*], // Already expanded fields
        [$field:ident, [$($ftype:tt)*]&[$($ctype:tt)*]; $($done:tt)*] // Current
    } => {
        mp4box_gen! {
            @expand $version $flags;
            $name $($stype)? {
                $($rest)* // Remaining fields
            }; [
                $($prev)* // Already expanded fields
                [$field, [Option<$($ftype)*>]&[Option<[$($ctype)*]>]; $cond, $($done)*],
            ]
        }
    };
    { // Iterate
        @cond_expand $version:ident $flags:ident;
        $name:ident $($stype:ident)? {
            [$type:tt, $($rtype:tt,)*] [if $cond:expr] $([if $rcond:expr])*,
            $($rest:tt)* // Remaining fields
        }; [$($prev:tt)*], // Already expanded fields
        [$field:ident, [$($ftype:tt)*]&[$($ctype:tt)*]; $($done:tt)*] // Current
    } => {
        mp4box_gen! {
            @cond_expand $version $flags;
            $name $($stype)? {
                [$($rtype,)*] $([if $rcond])*,
                $($rest)* // Remaining fields
            }; [$($prev)*], // Already expanded fields
            [$field, [Either<$type,$($ftype)*>]&[Either<$type,[$($ctype)*]>]; $cond, $($done)*]
        }
    };

    // Expansion TT
    // Struct
    { // Condition
        @expand $version:ident $flags:ident;
        $name:ident $($stype:ident)? {
            $field:ident: [$($type:tt)*] {
                $(
                    $ifield:ident: $iftype:tt $({$($ifsdef:tt)+})? $([if $ifcond:expr])*
                ),+ $(,)?
            } [if $cond:expr],
            $($rest:tt)* // Remaining fields
        }; [$($prev:tt)*] // Already expanded fields
    } => {
        paste::paste! {
            mp4box_gen! {
                @expand $version $flags;
                [<$name:camel $field:camel Type>] $($stype)? {
                    $(
                        $ifield: $iftype $({$($ifsdef)+})? $([if $ifcond])*,
                    )+
                }; []
            }
            mp4box_gen! {
                @expand $version $flags;
                $name $($stype)? {
                    $($rest)* // Remaining fields
                }; [
                    $($prev)* // Already expanded fields
                    [$field, [Option<Vec<[<Box $name:camel $field:camel Type>]>>]&[Option<[Vec<[<Box $name:camel $field:camel Type>], Option<[$($type)*]>>]>]; $cond,],
                ]
            }
        }
    };
    { // No condition
        @expand $version:ident $flags:ident;
        $name:ident $($stype:ident)? {
            $field:ident: [$type:ident] {
                $(
                    $ifield:ident: $iftype:tt $({$($ifsdef:tt)+})? $([if $ifcond:expr])*
                ),+ $(,)?
            },
            $($rest:tt)* // Remaining fields
        }; [$($prev:tt)*] // Already expanded fields
    } => {
        paste::paste! {
            mp4box_gen! {
                @expand $version $flags;
                [<$name:camel $field:camel Type>] $($stype)? {
                    $(
                        $ifield: $iftype $({$($ifsdef)+})? $([if $ifcond])*,
                    )+
                }; []
            }
            mp4box_gen! {
                @expand $version $flags;
                $name $($stype)? {
                    $($rest)* // Remaining fields
                }; [
                    $($prev)* // Already expanded fields
                    [$field, [Vec<[<Box $name:camel $field:camel Type>]>]&[Vec<[<Box $name:camel $field:camel Type>], $type>]; ],
                ]
            }
        }
    };

    // Non-struct
    { // Multi-Condition
        @expand $version:ident $flags:ident;
        $name:ident $($stype:ident)? {
            $field:ident: [$ftype:tt, $($type:tt),+ $(,)?] $([if $cond:expr])+,
            $($rest:tt)* // Remaining fields
        }; [$($prev:tt)*] // Already expanded fields
    } => {
        mp4box_gen! {
            @cond_expand $version $flags;
            $name $($stype)? {
                [$($type,)*] $([if $cond])+,
                $($rest)* // Remaining fields
            }; [
                $($prev)* // Already expanded fields
            ], [$field, [$ftype]&[$ftype]; ]
        }
    };
    { // Condition
        @expand $version:ident $flags:ident;
        $name:ident $($stype:ident)? {
            $field:ident: $type:tt [if $cond:expr],
            $($rest:tt)* // Remaining fields
        }; [$($prev:tt)*] // Already expanded fields
    } => {
        mp4box_gen! {
            @expand $version $flags;
            $name $($stype)? {
                $($rest)* // Remaining fields
            }; [
                $($prev)* // Already expanded fields
                [$field, [Option<$type>]&[Option<[$type]>]; $cond,],
            ]
        }
    };
    { // No condition
        @expand $version:ident $flags:ident;
        $name:ident $($stype:ident)? {
            $field:ident: Vec<$type:tt>,
            $($rest:tt)* // Remaining fields
        }; [$($prev:tt)*] // Already expanded fields
    } => {
        mp4box_gen! {
            @expand $version $flags;
            $name $($stype)? {
                $($rest)* // Remaining fields
            }; [
                $($prev)* // Already expanded fields
                [$field, [Vec<$type>]&[Vec<$type, Remain>];],
            ]
        }
    };
    { // No condition
        @expand $version:ident $flags:ident;
        $name:ident $($stype:ident)? {
            $field:ident: $type:tt,
            $($rest:tt)* // Remaining fields
        }; [$($prev:tt)*] // Already expanded fields
    } => {
        mp4box_gen! {
            @expand $version $flags;
            $name $($stype)? {
                $($rest)* // Remaining fields
            }; [
                $($prev)* // Already expanded fields
                [$field, [$type]&[$type];],
            ]
        }
    };

    // Container parsing
    {
        @expand $version:ident $flags:ident;
        $name:ident Container $type:tt
    } => {
        use $crate::base::*;
        use $crate::r#macro::*;

        paste::paste! {
            pub struct [<Box $name>] {
                pub data: Vec<$type>,
            }
            impl std::fmt::Debug for [<Box $name>] {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}: {:?}", stringify!($name), self.data)
                }
            }
            impl Mp4BoxTrait for [<Box $name>] {
                const TYPE: u32 = u32::from_ne_bytes(*bstringify::bstringify!([<$name:lower>]));

                fn parse_full(input: &[u8], state: &mut ParserState) -> Self {
                    Self::parse(input, state, &None)
                }

                fn parse(input: &[u8], state: &mut ParserState, _header: &Option<(u8, u32)>) -> Self {
                    let mut data = vec![];
                    while !is_empty(input, &state) {
                        data.push(mp4box_gen!{ @read input state _header; $type });
                    }

                    Self { data }
                }

                fn write_full(&self, output: &mut Vec<u8>) {
                    let mut data = Vec::new();
                    self.write(&mut data);

                    // Write header
                    let size = data.len() as u32 + 8;
                    output.extend_from_slice(&u32::to_be_bytes(size));
                    output.extend_from_slice(&u32::to_ne_bytes(Self::TYPE));
                    output.extend(data);
                }

                fn write(&self, output: &mut Vec<u8>) {
                    for item in &self.data {
                        mp4box_gen! { @write output item; &$type }
                    }
                }
            }
        }
    };
    {
        @expand $version:ident $flags:ident;
        $name:ident Container
    } => {
        mp4box_gen!{@expand $version $flags; $name Container Mp4Box}
    };

    // Skip box
    {
        @expand $version:ident $flags:ident;
        $name:ident Skip
    } => {
        use $crate::base::*;
        use $crate::r#macro::*;

        paste::paste! {
            pub struct [<Box $name>] {
                pub data: Vec<u8>,
            }
            impl std::fmt::Debug for [<Box $name>] {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    write!(f, "{}: {:?}", stringify!($name), self.data)
                }
            }
            impl Mp4BoxTrait for [<Box $name>] {
                const TYPE: u32 = u32::from_ne_bytes(*bstringify::bstringify!([<$name:lower>]));

                fn parse_full(input: &[u8], state: &mut ParserState) -> Self {
                    Self::parse(input, state, &None)
                }

                fn parse(input: &[u8], state: &mut ParserState, _: &Option<(u8, u32)>) -> Self {
                    // Read all data in box into data
                    let data = read(input, state, input.len() - state.offset).unwrap().to_vec();

                    Self { data }
                }

                fn write_full(&self, output: &mut Vec<u8>) {
                    let mut data = Vec::new();
                    self.write(&mut data);

                    // Write header
                    let size = data.len() as u32 + 8;
                    output.extend_from_slice(&u32::to_be_bytes(size));
                    output.extend_from_slice(&u32::to_ne_bytes(Self::TYPE));
                    output.extend(data);
                }

                fn write(&self, output: &mut Vec<u8>) {
                    output.extend(&self.data);
                }
            }
        }
    };

    {
        $version:ident $flags:ident;
        $($sname:ident $(: $stype:ident $(= $svtype:tt)?)? $({
            $(
                $field:ident: $ftype:tt$(<$iftype:tt>)? $({$($fsdef:tt)+})? $([if $fcond:expr])*
            ),+ $(,)?
        })?),* $(,)? // Trailing comma may be omitted
    } => {
        $(mp4box_gen! {
            @expand $version $flags;
            $sname $($stype $($svtype)?)? $({
                $(
                    $field: $ftype$(<$iftype>)? $({$($fsdef)+})? $([if $fcond])*,
                )+
            }; [])?
        })*

        paste::paste! {
            pub enum Mp4Box {
                $( $sname(Box<[<Box $sname>]>), )*
            }
            impl std::fmt::Debug for Mp4Box {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        $( Mp4Box::$sname(box_) => write!(f, "{:?}", box_), )*
                    }
                }
            }

            impl Mp4Box {
                pub(crate) fn write(&self, output: &mut Vec<u8>) {
                    match self {
                        $( Mp4Box::$sname(box_) => box_.write_full(output), )*
                    }
                }
            }

            pub(crate) fn parse_box(input: &[u8], state: &mut ParserState) -> Option<Mp4Box> {
                assert!(!is_empty(input, state));
                let data = read_header(input, state);

                match data.0 {
                    $([<Box $sname>]::TYPE => {
                        Some(Mp4Box::$sname(Box::new([<Box $sname>]::parse_full(data.1, &mut ParserState { offset: 0 }))))
                    })*
                    _ => panic!("Unknown box type: {}", String::from_utf8_lossy(&u32::to_ne_bytes(data.0))),
                }
            }
            pub(crate) fn is_box_type(box_: &Mp4Box, type_: u32) -> bool {
                match box_ {
                    $(Mp4Box::$sname(_) => [<Box $sname>]::TYPE == type_),*
                }
            }
            pub(crate) fn get_box_type(box_: &Mp4Box) -> String {
                let int = match box_ {
                    $(Mp4Box::$sname(_) => [<Box $sname>]::TYPE),*
                };
                String::from_utf8(u32::to_ne_bytes(int).to_vec()).unwrap()
            }
        }
    };
}

// crate visibility
pub(crate) use mp4box_gen;
