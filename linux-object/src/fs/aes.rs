extern crate alloc;

use alloc::vec::Vec;
use linux_object::fs::matrix::{S, S2, COL, COL2, RCON};

fn mul_field(mut a: u8, mut b: u8) -> u8 {
    let mut result: u8 = 0; //伽罗瓦域内乘法计算的结果
    while a != 0 { //如果a最低位是1就异或b，相当于加上b * 1
        if a & 0x01 != 0 {
            result ^= b;
        } //a右移一位，相当于除以2
        a = a >> 1;
        if b & 0x80 != 0 { //如果b最高位为1
            b = b << 1; //注：这里会丢失最高位，但是不用担心
            b ^= 0x1B; //计算伽罗瓦域内除法b = b / (x^8(刚好丢失最高位) + x^4 + x^3 + x^1 + 1)
        }
        else {
            b = b << 1;
        }
    }
    result
}

fn add_key(key: &[u8], value: & mut[u8]) {
    value.iter_mut().zip(key.iter()).for_each(|(v, k)| {*v ^= k});
}


fn shift_row(value: & mut[u8], encrypt: bool) {
    let mut tmp: u8;

    tmp = value[2];
    value[2] = value[10];
    value[10] = tmp;
    tmp = value[6];
    value[6] = value[14];
    value[14] = tmp;

    if encrypt {
        tmp = value[1];
        value[1] = value[5];
        value[5] = value[9];
        value[9] = value[13];
        value[13] = tmp;

        tmp = value[3];
        value[3] = value[15];
        value[15] = value[11];
        value[11] = value[7];
        value[7] = tmp;
    } else {
        tmp = value[1];
        value[1] = value[13];
        value[13] = value[9];
        value[9] = value[5];
        value[5] = tmp;

        tmp = value[3];
        value[3] = value[7];
        value[7] = value[11];
        value[11] = value[15];
        value[15] = tmp;
    }
}

fn sub_byte(value: & mut[u8], encrypt: bool) {
    let matrix: &[u8] = if encrypt {&S} else {&S2};
    value.iter_mut().for_each(|item| {*item = matrix[*item as usize]});
}

fn mix_column(value: & mut[u8], encrypt: bool) {
    let mut init : [u8; 16] = [0; 16];
    let matrix: &[u8] = if encrypt {&COL} else {&COL2};
    init.copy_from_slice(value);
    value.fill(0);
    for i in 0..4 {
        for j in 0..4 {
            for k in 0..4 {
                value[i + 4 * j] ^= mul_field(matrix[4 * i + k], init[k + 4 * j]); // Transpose matrix
            }
        }
    }
}

fn key_expansion(key: &[u8], expand_key: &mut [u8]) {
    let mut tmp: [u8; 4] = [0; 4];
    let mut rcon: [u8; 4] = [0; 4];
    expand_key[0..16].copy_from_slice(key);
    for i in 4..44 {
        for j in 0..4 {
            tmp[j] = expand_key[4 * (i - 1) + j];
        }
        if i % 4 == 0 {
            let _tmp = tmp[0];
            tmp[0] = tmp[1];
            tmp[1] = tmp[2];
            tmp[2] = tmp[3];
            tmp[3] = _tmp;
            rcon[0] = RCON[i / 4 - 1];
            sub_byte(&mut tmp, true);
            add_key(&mut rcon, &mut tmp);
        }
        for j in 0..4 {
            expand_key[4 * i + j] = expand_key[4 * (i - 4) + j] ^ tmp[j];
        }
    }
}

fn encrypt(key: &[u8], value: & mut[u8]) {
    add_key(&key[0..16], value);
    for i in 0..10 {
        sub_byte(value, true);
        shift_row(value, true);
        if i < 9 {
            mix_column(value, true);
        }
        add_key(&key[16 * (i + 1) .. 16 * (i + 2)], value);
    }
}

fn decrypt(key: &[u8], value: & mut[u8]) {
    for i in 0..10 {
        add_key(&key[16 * (10 - i) .. 16 * (11 - i)], value);
        if i > 0 {
            mix_column(value, false);
        }
        shift_row(value, false);
        sub_byte(value, false);
    }
    add_key(&key[0..16], value);
}


pub fn translate(key: &[u8], src: &Vec<u8>, _encrypt: bool) -> Vec<u8> {
    assert_eq!(key.len(), 16);
    assert_eq!(src.len() % 16, 0);
    let mut expand_key: [u8; 176] = [0; 176];
    key_expansion(key, &mut expand_key);
    let mut dst: Vec<u8> = Vec::new();
    dst.clone_from(src);
    if _encrypt {
        dst.chunks_mut(16).for_each(|chunk| {
            encrypt(&expand_key, chunk);
        });
    } else {
        dst.chunks_mut(16).for_each(|chunk| {
            decrypt(&expand_key, chunk);
        });
    }
    dst
}

