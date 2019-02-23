use sha2::{Sha512, Digest};
use std::mem::transmute;

pub const HASH_LOOP_COUNT: usize = 512;
pub const HASH_LENGTH: usize = 64;
pub const SEED_LENGTH: usize = 44;


#[inline]
fn slice_replace(src: &mut [u8], by: &[u8]){
    debug_assert_eq!(src.len(), by.len());
    for (a, &b) in src.iter_mut().zip(by.iter()) {
        *a = b;
    }
}

pub fn generator(address: &str, nonce: u32) ->[u8;HASH_LOOP_COUNT*HASH_LENGTH] {
    let mut source = [0u8; HASH_LOOP_COUNT * HASH_LENGTH + SEED_LENGTH];
    let total_length = SEED_LENGTH + HASH_LOOP_COUNT * HASH_LENGTH;
    debug_assert_eq!(address.as_bytes().len() + 4, SEED_LENGTH);

    // seed ..-[address 40bytes]-[nonce 4bytes]
    let bytes: [u8; 4] = unsafe { transmute(nonce.to_le()) };
    slice_replace(&mut source[(total_length-4)..], &bytes);
    slice_replace(&mut source[(total_length-SEED_LENGTH)..(total_length-4)], address.as_bytes());
    //println!("source={:?}", &source[(total_length-SEED_LENGTH)..]);

    // seed [hash(HASH_LENGTH)]-...-[hash0]-[address 40bytes]-[nonce 4bytes]
    // [hashN] = SHA512([hash(N-1)]-...-[hash0]-[address 40bytes]-[nonce 4bytes])
    let start_index = total_length - SEED_LENGTH;
    let mut final_hash = [0u8; HASH_LENGTH];
    for index in 0..(HASH_LOOP_COUNT+1) {
        let pos = start_index - index * HASH_LENGTH;
        let hash = Sha512::digest(&source[pos..]);
        if pos == 0 {
            slice_replace(&mut final_hash, &hash);
        } else {
            slice_replace(&mut source[(pos-HASH_LENGTH)..pos], &hash);
        }
    }
    //println!("final={:?}\nsource={:?}", final_hash, &source[..]);

    // all hash_ints XOR with final_int
    // from: [hash(HASH_LENGTH)]-...-[hash0]-[address 40bytes]-[nonce 4bytes]
    // to  : [hash'0]- ... - [hash'(HASH_LENGTH)]
    let mut output = [0u8; HASH_LOOP_COUNT * HASH_LENGTH];
    for (index, item) in output.iter_mut().enumerate() {
        let inner_pos = index % HASH_LENGTH;  // 0~31
        let outer_pos = index / HASH_LENGTH;
        let x = &final_hash[inner_pos];
        let y = &source[(HASH_LOOP_COUNT-outer_pos-1)*HASH_LENGTH+inner_pos];
        *item = x ^ y;
        //println!("{} {:?}=={:?}^{:?}", index, item, x, y);
    }
    //println!("output={:?}", &output[..]);
    output
}
