use crate::base::*;
use std::fmt::Debug;

// MP4 impl helpers
pub(crate) trait Mp4BoxTrait: Debug {
    const TYPE: u32; // Should be 4 bytes/ASCII chars

    fn parse_full(input: &[u8]) -> Self;
    fn parse(input: &[u8], header: &Option<(u8, u32)>) -> Self;
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
        read($input, &mut $state, 1).unwrap()[0]
    };
    { @read $input:ident $state:ident $header:ident; u16 } => {
        {
            let slice = read($input, &mut $state, 2).unwrap();
            u16::from_be_bytes([slice[0], slice[1]])
        }
    };
    { @read $input:ident $state:ident $header:ident; u32 } => {
        {
            let slice = read($input, &mut $state, 4).unwrap();
            u32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]])
        }
    };
    { @read $input:ident $state:ident $header:ident; u64 } => {
        {
            let slice = read($input, &mut $state, 8).unwrap();
            u64::from_be_bytes([slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7]])
        }
    };
    { @read $input:ident $state:ident $header:ident; i8 } => {
        read($input, $state, 1).unwrap()[0] as i8
    };
    { @read $input:ident $state:ident $header:ident; i16 } => {
        {
            let slice = read($input, &mut $state, 2).unwrap();
            i16::from_be_bytes([slice[0], slice[1]])
        }
    };
    { @read $input:ident $state:ident $header:ident; i32 } => {
        {
            let slice = read($input, &mut $state, 4).unwrap();
            i32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]])
        }
    };
    { @read $input:ident $state:ident $header:ident; i64 } => {
        {
            let slice = read($input, &mut $state, 8).unwrap();
            i64::from_be_bytes([slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7]])
        }
    };
    { @read $input:ident $state:ident $header:ident; f32 } => {
        {
            let slice = read($input, &mut $state, 4).unwrap();
            f32::from_be_bytes([slice[0], slice[1], slice[2], slice[3]])
        }
    };
    { @read $input:ident $state:ident $header:ident; f64 } => {
        {
            let slice = read($input, &mut $state, 8).unwrap();
            f64::from_be_bytes([slice[0], slice[1], slice[2], slice[3], slice[4], slice[5], slice[6], slice[7]])
        }
    };
    { @read $input:ident $state:ident $header:ident; [u8; $n:expr] } => { // Might also work with sizes defined by variables
        {
            let mut array = [0; $n];
            array.copy_from_slice(read($input, &mut $state, $n).unwrap());
            array
        }
    };
    { @read $input:ident $state:ident $header:ident; [u8] } => {
        read($input, &mut $state, $input.len() - $state.offset).unwrap()
    };
    { @read $input:ident $state:ident $header:ident; Mp4Box } => {
        parse_box($input, &mut $state).unwrap()
    };

    // Generic catch-all for metastructs
    { @read $input:ident $state:ident $header:ident; $($type:tt)* } => {
        //panic!("Unsupported type: {}", stringify!($type))
        <$($type)*>::parse($input, &$header)
    };

    // Condition expansion
    {
        @cond
        $input:ident $state:ident $header:ident;
        [Option<[$($type:tt)*]>],
        $cond:expr,
    } => {
        if $cond {
            Some(mp4box_gen! { @cond $input $state $header; [$($type)*], true, })
        } else {
            None
        }
    };
    {
        @cond
        $input:ident $state:ident $header:ident;
        [Vec<[$($type:tt)*], Option<$length:ident>>],
        $cond:expr,
    } => {
        if $cond {
            let length = $length.unwrap() as usize;
            let mut vec = Vec::with_capacity(length);
            for _ in 0..length {
                vec.push(mp4box_gen! { @cond $input $state $header; [$($type)*], true, });
            }
            vec
        } else {
            vec![]
        }
    };
    {
        @cond
        $input:ident $state:ident $header:ident;
        [Vec<[$($type:tt)*], $length:ident>],
        $cond:expr,
    } => {
        if $cond {
            let mut vec = Vec::with_capacity($length as usize);
            for _ in 0..$length {
                vec.push(mp4box_gen! { @cond $input $state $header; [$($type)*], true, });
            }
            vec
        } else {
            vec![]
        }
    };
    { // Final layer
        @cond
        $input:ident $state:ident $header:ident;
        [$($type:tt)*],
        $cond:expr,
    } => {
        if $cond {
            mp4box_gen! { @read $input $state $header; $($type)* }
        } else {
            panic!("Condition not met");
        }
    };

    // Struct construction
    // Full Type
    {
        @expand $version:ident $flags:ident;
        $name:ident Full {}; // No fields remaining
        [
            $([
                $field:ident, [$($ftype:tt)*]&[$($ctype:tt)*]; $cond:expr,
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

                fn parse_full(input: &[u8]) -> Self {
                    let mut state = ParserState { offset: 0 };
                    let (version, flags) = read_fullbox_header(input, &mut state);

                    Self::parse(&input[state.offset..], &Some(( version, flags )))
                }

                fn parse(input: &[u8], header: &Option<(u8, u32)>) -> Self {
                    let mut state = ParserState { offset: 0 };
                    let uwh = header.as_ref().unwrap();
                    let ($version, $flags) = uwh;

                    // Split out into fields so they can reference each other
                    $(
                        let $field = mp4box_gen! {
                            @cond input state header;
                            [$($ctype)*],
                            $cond,
                        };
                    )+

                    Self {
                        $(
                            $field,
                        )+
                    }
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
                $field:ident, [$($ftype:tt)*]&[$($ctype:tt)*]; $cond:expr,
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
            impl Mp4BoxTrait for [<Box $name>] {
                const TYPE: u32 = u32::from_ne_bytes(bstringify::bstringify!([<$name:lower>])[..4]);

                fn parse_full(input: &[u8]) -> Self {
                    Self::parse(input, &None)
                }

                fn parse(input: &[u8], _: &Option<(u8, u32)>) -> Self {
                    let mut state = ParserState { offset: 0 };

                    // Split out into fields so they can reference each other
                    $(
                        let $field = mp4box_gen! {
                            @cond input state header; // Header is dummy object, not used in this context
                            [$($ctype)*],
                            $cond,
                        };
                    )+

                    Self {
                        $(
                            $field,
                        )+
                    }
                }
            }
        }
    };

    // Expansion TT
    // Struct
    { // Condition
        @expand $version:ident $flags:ident;
        $name:ident $($stype:ident)? {
            $field:ident: [$type:ident] {
                $(
                    $ifield:ident: $iftype:tt $({$($ifsdef:tt)+})? $([if $ifcond:expr])?
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
                        $ifield: $iftype $({$($ifsdef)+})? $([if $ifcond])?,
                    )+
                }; []
            }
            mp4box_gen! {
                @expand $version $flags;
                $name $($stype)? {
                    $($rest)* // Remaining fields
                }; [
                    $($prev)* // Already expanded fields
                    [$field, [Vec<[<Box $name:camel $field:camel Type>]>]&[Vec<[[<Box $name:camel $field:camel Type>]],Option<$type>>]; $cond,],
                ]
            }
        }
    };
    { // No condition
        @expand $version:ident $flags:ident;
        $name:ident $($stype:ident)? {
            $field:ident: [$type:ident] {
                $(
                    $ifield:ident: $iftype:tt $({$($ifsdef:tt)+})? $([if $ifcond:expr])?
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
                        $ifield: $iftype $({$($ifsdef)+})? $([if $ifcond])?,
                    )+
                }; []
            }
            mp4box_gen! {
                @expand $version $flags;
                $name $($stype)? {
                    $($rest)* // Remaining fields
                }; [
                    $($prev)* // Already expanded fields
                    [$field, [Vec<[<Box $name:camel $field:camel Type>]>]&[Vec<[[<Box $name:camel $field:camel Type>]],$type>]; true,],
                ]
            }
        }
    };

    // Non-struct
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
                [$field, [$type]&[$type]; true,],
            ]
        }
    };

    // Container parsing
    {
        @expand $version:ident $flags:ident;
        $name:ident Container $type:tt
    } => {
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

                fn parse_full(input: &[u8]) -> Self {
                    Self::parse(input, &None)
                }

                fn parse(input: &[u8], _: &Option<(u8, u32)>) -> Self {
                    let mut state = ParserState { offset: 0 };

                    let mut data = vec![];
                    while !is_empty(input, &state) {
                        data.push(mp4box_gen!{ @read input state header; $type });
                    }

                    Self { data }
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

    {
        $version:ident $flags:ident;
        $($sname:ident $(: $stype:ident $(= $svtype:tt)?)? $({
            $(
                $field:ident: $ftype:tt $({$($fsdef:tt)+})? $([if $fcond:expr])?
            ),+ $(,)?
        })?),* $(,)? // Trailing comma may be omitted
    } => {
        $(mp4box_gen! {
            @expand $version $flags;
            $sname $($stype $($svtype)?)? $({
                $(
                    $field: $ftype $({$($fsdef)+})? $([if $fcond])?,
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

            pub(crate) fn parse_box(input: &[u8], state: &mut ParserState) -> Option<Mp4Box> {
                assert!(!is_empty(input, state));
                let data = read_header(input, state);

                match data.0 {
                    $([<Box $sname>]::TYPE => {
                        Some(Mp4Box::$sname(Box::new([<Box $sname>]::parse_full(data.1))))
                    })*
                    _ => panic!("Unknown box type: {}", String::from_utf8_lossy(&u32::to_ne_bytes(data.0))),
                }
            }
            pub(crate) fn is_box_type(box_: &Mp4Box, type_: u32) -> bool {
                match box_ {
                    $(Mp4Box::$sname(_) => [<Box $sname>]::TYPE == type_),*
                }
            }
        }
    };
}

// crate visibility
pub(crate) use mp4box_gen;
