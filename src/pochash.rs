use blake2b_simd::blake2b;
use std::mem::transmute;
use std::cmp::min;

pub const HASH_LOOP_COUNT: usize = 8192;
pub const HASH_LENGTH: usize = 64;
pub const SEED_LENGTH: usize = 21 + 4;  // ver + addr + nonce


#[inline]
fn slice_replace(src: &mut [u8], by: &[u8]){
    debug_assert_eq!(src.len(), by.len());
    for (a, &b) in src.iter_mut().zip(by.iter()) {
        *a = b;
    }
}

pub fn generator(ver_identifier: &[u8], nonce: u32, output: &mut Box<[u8;HASH_LOOP_COUNT*HASH_LENGTH]>) {
    let mut source = box [0u8; HASH_LOOP_COUNT * HASH_LENGTH + SEED_LENGTH];
    let total_length = SEED_LENGTH + HASH_LOOP_COUNT * HASH_LENGTH;
    debug_assert_eq!(ver_identifier.len() + 4, SEED_LENGTH);

    // seed ..-[ver_identifier 21bytes]-[nonce 4bytes]
    let bytes: [u8; 4] = unsafe { transmute(nonce.to_le()) };
    slice_replace(&mut source[(total_length-4)..], &bytes);
    slice_replace(&mut source[(total_length-SEED_LENGTH)..(total_length-4)], ver_identifier);
    //println!("source={:?}", &source[(total_length-SEED_LENGTH)..]);

    // seed [hash(HASH_LENGTH)]-...-[hash0]-[ver_identifier 21bytes]-[nonce 4bytes]
    // [hashN] = blake2bp([hash(N-1)]-...-[hash0]-[ver_identifier 21bytes]-[nonce 4bytes])
    let start_index = total_length - SEED_LENGTH;
    let mut final_hash = [0u8; HASH_LENGTH];
    for index in 0..(HASH_LOOP_COUNT) {
        let start = start_index - index * HASH_LENGTH;
        let end = min(start + 1024, total_length);
        let hash = blake2b(&source[start..end]);
        let hash = hash.as_bytes();
        slice_replace(&mut source[(start-HASH_LENGTH)..start], &hash);
    }
    {  // generate final hash
        let hash = blake2b(&source[..]);
        let hash = hash.as_bytes();
        slice_replace(&mut final_hash, &hash);
    }
    //println!("final={:?}\nsource={:?}", final_hash, &source[..]);

    // all hash_ints XOR with final_int
    // from: [hash(HASH_LENGTH)]-...-[hash0]-[ver_identifier 21bytes]-[nonce 4bytes]
    // to  : [hash'0]- ... - [hash'(HASH_LENGTH)]
    for (index, item) in output.iter_mut().enumerate() {
        let inner_pos = index % HASH_LENGTH;  // 0~31
        let outer_pos = index / HASH_LENGTH;
        let x = &final_hash[inner_pos];
        let y = &source[(HASH_LOOP_COUNT-outer_pos-1)*HASH_LENGTH+inner_pos];
        *item = x ^ y;
        //println!("{} {:?}=={:?}^{:?}", index, item, x, y);
    }
    //println!("output={:?}", &output[..]);
}
