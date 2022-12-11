pub fn is_aligned(value: usize, alignment: usize) -> bool {
    (value & (alignment - 1)) != 0
}

pub fn align_up(value: usize, alignment: usize) -> usize {
    let a = alignment - 1;
    (value + a) & !a
}
