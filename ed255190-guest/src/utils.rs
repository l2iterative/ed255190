#[cfg(target_os = "zkvm")]
extern "C" {
    fn sys_bigint(
        result: *mut [u32; 8],
        op: u32,
        x: *const [u32; 8],
        y: *const [u32; 8],
        modulus: *const [u32; 8],
    );
}

#[cfg(not(target_os = "zkvm"))]
pub fn mul_mod(a: &[u32; 8], b: &[u32; 8], n: &[u32; 8]) -> [u32; 8] {
    let a = num_bigint::BigUint::from_bytes_le(bytemuck::cast_slice::<_, u8>(a));
    let b = num_bigint::BigUint::from_bytes_le(bytemuck::cast_slice::<_, u8>(b));
    let n = num_bigint::BigUint::from_bytes_le(bytemuck::cast_slice::<_, u8>(n));

    let res_digits = (a * b % n).to_u32_digits();

    let mut res = [0u32; 8];
    for (i, digit) in res_digits.iter().enumerate() {
        res[i] = *digit;
    }
    res
}

#[cfg(target_os = "zkvm")]
#[inline(always)]
pub fn mul_mod(a: &[u32; 8], b: &[u32; 8], n: &[u32; 8]) -> [u32; 8] {
    use std::mem::MaybeUninit;
    let mut res = MaybeUninit::<[u32; 8]>::uninit();

    unsafe {
        sys_bigint(
            res.as_mut_ptr(),
            0u32,
            a as *const [u32; 8],
            b as *const [u32; 8],
            n as *const [u32; 8],
        );
    }

    return unsafe { res.assume_init() };
}
