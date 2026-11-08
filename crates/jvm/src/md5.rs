// the per round shift amounts and sine derived constants of RFC 1321
const SHIFTS: [u32; 64] = [
    7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 7, 12, 17, 22, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20, 5, 9, 14, 20,
    4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 4, 11, 16, 23, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15, 21, 6, 10, 15,
    21,
];

fn sineTable() -> [u32; 64]
{
    let mut table: [u32; 64] = [0; 64];
    for (i, entry) in table.iter_mut().enumerate()
    {
        *entry = ((i as f64 + 1.0).sin().abs() * 4294967296.0) as u32;
    }
    table
}

// MessageDigest.getInstance("MD5").digest
pub fn digest(message: &[u8]) -> [u8; 16]
{
    let table: [u32; 64] = sineTable();
    let mut state: [u32; 4] = [0x67452301, 0xEFCDAB89, 0x98BADCFE, 0x10325476];
    let mut data: Vec<u8> = message.to_vec();
    data.push(0x80);
    while data.len() % 64 != 56
    {
        data.push(0);
    }
    data.extend_from_slice(&((message.len() as u64).wrapping_mul(8)).to_le_bytes());
    for chunk in data.chunks(64)
    {
        let words: Vec<u32> =
            chunk.chunks(4).map(|word| u32::from_le_bytes([word[0], word[1], word[2], word[3]])).collect();
        let [mut a, mut b, mut c, mut d]: [u32; 4] = state;
        for i in 0..64
        {
            let (f, g): (u32, usize) = match i / 16
            {
                0 => ((b & c) | (!b & d), i),
                1 => ((d & b) | (!d & c), (5 * i + 1) % 16),
                2 => (b ^ c ^ d, (3 * i + 5) % 16),
                _ => (c ^ (b | !d), (7 * i) % 16),
            };
            let rotated: u32 = a.wrapping_add(f).wrapping_add(table[i]).wrapping_add(words[g]).rotate_left(SHIFTS[i]);
            a = d;
            d = c;
            c = b;
            b = b.wrapping_add(rotated);
        }
        state =
            [state[0].wrapping_add(a), state[1].wrapping_add(b), state[2].wrapping_add(c), state[3].wrapping_add(d)];
    }
    let mut digest: [u8; 16] = [0; 16];
    for (i, word) in state.iter().enumerate()
    {
        digest[i * 4..i * 4 + 4].copy_from_slice(&word.to_le_bytes());
    }
    digest
}
