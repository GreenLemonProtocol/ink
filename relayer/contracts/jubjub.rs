use zkp_u256::{InvMod, One, Zero, U256};

pub(crate) static JUBJUB_A: u64 = 168700;
pub(crate) static JUBJUB_D: u64 = 168696;

pub fn scalar_mult(x: U256, y: U256, mut value: U256) -> Option<(U256, U256)> {
    let scalar_field = U256::from_decimal_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();
    let mut p = point_to_etec(x, y, scalar_field.clone());

    let mut a = [U256::zero(), U256::one(), U256::zero(), U256::one()];
    while !value.is_zero() {
        if !(&value & U256::one()).is_zero() {
            a = etec_add(&a, &p, &scalar_field, &JUBJUB_A.into(), &JUBJUB_D.into());
        }
        p = etec_double(&p, &scalar_field, &JUBJUB_A.into());
        value /= U256::from(2);
    }
    etec_to_point(a, scalar_field)
}

pub fn scalar_add(p: (U256, U256), q: (U256, U256)) -> Option<(U256, U256)> {
    let scalar_field = U256::from_decimal_str(
        "21888242871839275222246405745257275088548364400416034343698204186575808495617",
    )
    .unwrap();
    let a = point_to_etec(p.0, p.1, scalar_field.clone());
    let b = point_to_etec(q.0, q.1, scalar_field.clone());
    let ret = etec_add(&a, &b, &scalar_field, &JUBJUB_A.into(), &JUBJUB_D.into());
    etec_to_point(ret, scalar_field)
}

// Project (x,y) point to extended edwards coordinates.
pub fn point_to_etec(x: U256, y: U256, q: U256) -> [U256; 4] {
    let mut output = [
        U256::default(),
        U256::default(),
        U256::default(),
        U256::default(),
    ];
    output[0] = x.clone();
    output[1] = y.clone();
    output[2] = x.mulmod(&y, &q);
    output[3] = U256::one();
    output
}

// Extended twisted edwards coordinates to extended affine coordinates
pub fn etec_to_point(point: [U256; 4], q: U256) -> Option<(U256, U256)> {
    point[3]
        .inv_mod(&q)
        .map(|inv_z| (point[0].mulmod(&inv_z, &q), point[1].mulmod(&inv_z, &q)))
}

// x3 = (x1y2 + y1x2) * (z1z2 - dt1t2)
// y3 = (y1y2 - ax1x2) * (z1z2 + dt1t2)
// t3 = (y1y2 - ax1x2) * (x1y2 + y1x2)
// z3 = (z1z2 - dt1t2) * (z1z2 + dt1t2)
#[allow(clippy::many_single_char_names)]
pub fn etec_add(
    p1: &[U256; 4],
    p2: &[U256; 4],
    local_q: &U256,
    local_a: &U256,
    local_d: &U256,
) -> [U256; 4] {
    // a = x1 * x2
    let a = p1[0].mulmod(&p2[0], local_q);
    // b = y1 * y2
    let b = p1[1].mulmod(&p2[1], local_q);
    // c = d * t1 * t2
    let c = local_d.mulmod(&p1[2], local_q).mulmod(&p2[2], local_q);
    // d = z1 * z2
    let d = p1[3].mulmod(&p2[3], local_q);
    // e = (x1 + y1) * (x2 + y2) - A - B
    let e = ((&p1[0] + &p1[1]) % local_q).mulmod(&((&p2[0] + &p2[1]) % local_q), local_q)
        + (((local_q - &a) + (local_q - &b)) % local_q) % local_q;
    // f = d - c
    let f = &d + (local_q - &c) % local_q;
    // g = d + c
    let g = &d + c % local_q;
    // h = b - a * A
    let h = &b + (local_q - local_a.mulmod(&a, local_q)) % local_q;

    // x3 = e * f
    // y3 = g * h
    // t3 = e * h
    // z3 = f * g
    let input = [(&e, &f), (&g, &h), (&e, &h), (&f, &g)];
    let mut p3 = [
        U256::default(),
        U256::default(),
        U256::default(),
        U256::default(),
    ];
    input
        .iter()
        .enumerate()
        .for_each(|(i, (l, r))| p3[i] = l.mulmod(r, local_q));
    p3
}

// local_a := 0x292FC
// local_q := 0x30644E72E131A029B85045B68181585D2833E84879B9709143E1F593F0000001
#[allow(clippy::many_single_char_names)]
pub fn etec_double(p1: &[U256; 4], local_q: &U256, local_a: &U256) -> [U256; 4] {
    // a = x * x
    let a = p1[0].mulmod(&p1[0], local_q);
    // b = y * y
    let b = p1[1].mulmod(&p1[1], local_q);

    // c = z * z * 2
    let c = p1[3]
        .mulmod(&p1[3], local_q)
        .mulmod(&U256::from(2), local_q);

    // d = jubjub_a * a
    let d = local_a.mulmod(&a, local_q);
    // e = x + y
    let mut e = &p1[0] + &p1[1] % local_q;
    // e = e^2 - a - b
    e = e.mulmod(&e, local_q) + (local_q - &a) % local_q + (local_q - &b) % local_q;

    // g = d + b
    let g = &d + &b % local_q;

    // f = g - c
    let f = &g + (local_q - &c) % local_q;

    // h = d - b;
    let h = &d + (local_q - &b) % local_q;

    // x3 = e * f
    // y3 = g * h
    // t3 = e * h
    // z3 = f * g
    let mut p2 = [
        U256::default(),
        U256::default(),
        U256::default(),
        U256::default(),
    ];
    let input = [(&e, &f), (&g, &h), (&e, &h), (&f, &g)];
    input
        .iter()
        .enumerate()
        .for_each(|(i, (l, r))| p2[i] = l.mulmod(r, local_q));
    p2
}
