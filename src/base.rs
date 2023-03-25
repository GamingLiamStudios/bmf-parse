pub(crate) struct ParserState {
    pub(crate) offset: usize,
}

pub(crate) fn read<'a>(input: &'a [u8], state: &mut ParserState, n: usize) -> Option<&'a [u8]> {
    if state.offset + n <= input.len() {
        let slice = &input[state.offset..state.offset + n];
        state.offset += n;
        Some(slice)
    } else {
        None
    }
}

#[inline]
pub(crate) fn is_empty(input: &[u8], state: &ParserState) -> bool {
    state.offset == input.len()
}

#[derive(Debug)]
pub enum Either<A, B> {
    A(A),
    B(B),
}
