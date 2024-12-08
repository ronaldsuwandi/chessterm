#[macro_export]
macro_rules! precompute_moves {
    ($func:ident) => {{
        let mut moves = [0u64; 64];
        let mut i = 0;
        while i < 64 {
            moves[i] = $func(i as u8);
            i += 1;
        }
        moves
    }};
}


#[macro_export]
macro_rules! precompute_rays {
    ($size: expr, $func: ident) => {{
        let mut rays = [[0u64; $size]; 64];
        let mut i = 0;
        while i < 64 {
            rays[i] = $func(i as u8);
            i += 1;
        }
        rays
    }};
}
