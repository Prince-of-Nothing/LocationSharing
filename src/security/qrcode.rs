//! QR code generation (pure Rust, no external crates).
//!
//! Produces an SVG string for a given text payload (e.g. an `otpauth://`
//! URI). Used by the MFA enrolment flow so authenticator apps can scan a
//! code instead of typing the secret. Byte mode, ECC level L, versions 1-10.
#![allow(clippy::all)]

// GF(2^8) exponent/log tables for Reed-Solomon encoding.
const GF_EXP: [u8; 512] = {
    let mut t = [0u8; 512];
    let mut x: u16 = 1;
    let mut i = 0usize;
    while i < 255 {
        t[i] = x as u8;
        x <<= 1;
        if x & 0x100 != 0 {
            x ^= 0x11d;
        }
        i += 1;
    }
    let mut j = 255usize;
    while j < 512 {
        t[j] = t[j - 255];
        j += 1;
    }
    t
};

const GF_LOG: [u8; 256] = {
    let mut t = [0u8; 256];
    let mut i = 0usize;
    while i < 255 {
        t[GF_EXP[i] as usize] = i as u8;
        i += 1;
    }
    t
};

fn gf_mul(a: u8, b: u8) -> u8 {
    if a == 0 || b == 0 {
        0
    } else {
        let ai = GF_LOG[usize::from(a)] as usize;
        let bi = GF_LOG[usize::from(b)] as usize;
        GF_EXP[(ai + bi) % 255]
    }
}

fn rs_gen_poly(nsym: usize) -> Vec<u8> {
    let mut g = vec![1u8];
    for _ in 0..nsym {
        let mut ng = vec![0u8; g.len() + 1];
        for ((j, v), &gf_val) in g.iter().enumerate().zip(GF_EXP.iter()) {
            ng[j] ^= *v;
            ng[j + 1] ^= gf_mul(*v, gf_val);
        }
        g = ng;
    }
    g
}

fn rs_encode(data: &[u8], nsym: usize) -> Vec<u8> {
    let gen = rs_gen_poly(nsym);
    let mut msg = vec![0u8; data.len() + nsym];
    msg[..data.len()].copy_from_slice(data);
    for i in 0..data.len() {
        let coef = msg[i];
        if coef != 0 {
            for j in 1..gen.len() {
                msg[i + j] ^= gf_mul(gen[j], coef);
            }
        }
    }
    msg[data.len()..].to_vec()
}

// Capacity (bytes) for byte mode, ECC L, versions 1..=10.
const CAPACITY_L: [usize; 11] = [0, 17, 32, 53, 78, 106, 134, 154, 192, 230, 271];

fn pick_version(len: usize) -> Option<usize> {
    CAPACITY_L
        .iter()
        .enumerate()
        .skip(1)
        .find_map(|(v, &cap)| if len <= cap { Some(v) } else { None })
}

// Alignment pattern centres per version.
fn alignment_centers(v: usize) -> Vec<usize> {
    match v {
        2 => vec![6, 18],
        3 => vec![6, 22],
        4 => vec![6, 26],
        5 => vec![6, 30],
        6 => vec![6, 34],
        7 => vec![6, 22, 38],
        8 => vec![6, 24, 42],
        9 => vec![6, 26, 46],
        10 => vec![6, 28, 50],
        _ => Vec::new(),
    }
}

fn size_for(v: usize) -> usize {
    v * 4 + 17
}

/// Generate a QR code matrix (true = dark) for `data`. Returns None if too long.
pub fn qr_matrix(data: &str) -> Option<Vec<Vec<bool>>> {
    let v = pick_version(data.len())?;
    let n = size_for(v);
    let mut matrix = vec![vec![false; n]; n];
    let mut reserved = vec![vec![false; n]; n];

    // Finder patterns (3 corners).
    for (r0, c0) in [(0usize, 0usize), (0usize, n - 7), (n - 7, 0usize)] {
        for r in 0..7usize {
            for c in 0..7usize {
                let dark = r == 0
                    || r == 6
                    || c == 0
                    || c == 6
                    || (2..=4).contains(&r) && (2..=4).contains(&c);
                matrix[r0 + r][c0 + c] = dark;
                reserved[r0 + r][c0 + c] = true;
            }
        }
    }
    // Separators.
    for (r0, c0) in [(0usize, 0usize), (0usize, n - 7), (n - 7, 0usize)] {
        for r in 0..8usize {
            for c in 0..8usize {
                let rr = r0 + r;
                let cc = c0 + c;
                if rr < n && cc < n {
                    if r == 7 || c == 7 {
                        reserved[rr][cc] = true;
                        matrix[rr][cc] = false;
                    }
                }
            }
        }
    }
    // Timing patterns.
    for i in 8..n - 8 {
        matrix[6][i] = i % 2 == 0;
        reserved[6][i] = true;
        matrix[i][6] = i % 2 == 0;
        reserved[i][6] = true;
    }
    // Alignment patterns.
    if v >= 2 {
        let centers = alignment_centers(v);
        for &r in &centers {
            for &c in &centers {
                if reserved[r][c] {
                    continue;
                }
                for dr in 0..5usize {
                    for dc in 0..5usize {
                        let dark = dr == 0 || dr == 4 || dc == 0 || dc == 4 || (dr == 2 && dc == 2);
                        matrix[r - 2 + dr][c - 2 + dc] = dark;
                        reserved[r - 2 + dr][c - 2 + dc] = true;
                    }
                }
            }
        }
    }
    // Dark module.
    matrix[n - 8][8] = true;
    reserved[n - 8][8] = true;

    // Encode data (byte mode).
    let mut bits: Vec<u8> = Vec::new();
    // mode indicator 0100
    bits.extend([0, 1, 0, 0]);
    // char count (8 bits for v1-9, 16 for v10+).
    let count_bits = if v <= 9 { 8 } else { 16 };
    for i in (0..count_bits).rev() {
        bits.push(((data.len() >> i) & 1) as u8);
    }
    for byte in data.bytes() {
        for i in (0..8).rev() {
            bits.push(((byte >> i) & 1) as u8);
        }
    }
    // terminator
    bits.extend([0, 0, 0, 0]);
    // pad to byte boundary
    while !bits.len().is_multiple_of(8) {
        bits.push(0);
    }
    // convert to bytes
    let mut bytes_out: Vec<u8> = Vec::new();
    for chunk in bits.chunks(8) {
        let mut b = 0u8;
        for &bit in chunk {
            b = (b << 1) | bit;
        }
        bytes_out.push(b);
    }
    // pad to capacity with alternating pad bytes.
    let nsym = ecc_symbols(v);
    let mut pad_byte = 0xECu8;
    while bytes_out.len() < CAPACITY_L[v] {
        bytes_out.push(pad_byte);
        pad_byte = if pad_byte == 0xEC { 0x11 } else { 0xEC };
    }
    // Reed-Solomon per block (single block for v1-10 L).
    let ecc = rs_encode(&bytes_out, nsym);
    bytes_out.extend(ecc);
    let codewords = bytes_out;

    // Place data bits into matrix in zigzag order.
    let mut bit_idx = 0;
    let mut positions: Vec<(usize, usize)> = Vec::new();
    let mut upward = true;
    let mut c = n - 1;
    while c > 0 {
        if c == 6 {
            c -= 1;
        }
        let rows: Box<dyn Iterator<Item = usize>> = if upward {
            Box::new(0..n)
        } else {
            Box::new((0..n).rev())
        };
        for r in rows {
            for cc in [c, c - 1] {
                if !reserved[r][cc] {
                    positions.push((r, cc));
                }
            }
        }
        upward = !upward;
        c = if c >= 2 { c - 2 } else { 0 };
        if c == 0 {
            break;
        }
    }
    // Place bits.
    for (r, c) in positions {
        if bit_idx < codewords.len() * 8 {
            let byte = codewords[bit_idx / 8];
            let bit = (byte >> (7 - (bit_idx & 7))) & 1;
            matrix[r][c] = bit == 1;
            bit_idx += 1;
        }
    }

    // Format info (ECC L = 01, mask 0).
    let fmt_bits: [u8; 15] = [1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0];
    // Place around top-left finder.
    let mut idx = 0;
    for i in 0..6 {
        matrix[8][i] = fmt_bits[idx] == 1;
        idx += 1;
    }
    matrix[8][7] = fmt_bits[idx] == 1;
    idx += 1;
    matrix[8][8] = fmt_bits[idx] == 1;
    idx += 1;
    matrix[7][8] = fmt_bits[idx] == 1;
    idx += 1;
    for i in (0..6).rev() {
        matrix[i][8] = fmt_bits[idx] == 1;
        idx += 1;
    }
    // Top-right.
    let mut idx2 = 0;
    for i in (n - 8..n).rev() {
        matrix[8][i] = fmt_bits[idx2] == 1;
        idx2 += 1;
    }
    // Left column bottom.
    let mut idx3 = 0;
    for i in (n - 7..n).rev() {
        matrix[i][8] = fmt_bits[7 + idx3] == 1;
        idx3 += 1;
    }

    // Apply mask 0: (r + c) % 2 == 0 -> invert.
    for r in 0..n {
        for c in 0..n {
            if !reserved[r][c] && (r + c) % 2 == 0 {
                matrix[r][c] = !matrix[r][c];
            }
        }
    }

    Some(matrix)
}

fn ecc_symbols(v: usize) -> usize {
    // ECC level L nsym per version.
    match v {
        1 => 7,
        2 => 10,
        3 => 15,
        4 => 20,
        5 => 26,
        6 => 18,
        7 => 20,
        8 => 24,
        9 => 28,
        10 => 36,
        _ => 7,
    }
}

/// Render a QR matrix as an SVG string.
pub fn qr_svg(data: &str) -> Option<String> {
    let matrix = qr_matrix(data)?;
    let n = matrix.len();
    let scale = 8;
    let margin = 4;
    let size = (n + margin * 2) * scale;
    let mut svg = String::new();
    svg.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{size}" height="{size}" viewBox="0 0 {size} {size}">"#
    ));
    svg.push_str(r##"<rect width="100%" height="100%" fill="#fff"/>"##);
    for (r, row) in matrix.iter().enumerate() {
        for (c, &cell) in row.iter().enumerate() {
            if cell {
                let x = (c + margin) * scale;
                let y = (r + margin) * scale;
                svg.push_str(&format!(
                    r##"<rect x="{x}" y="{y}" width="{scale}" height="{scale}" fill="#000"/>"##
                ));
            }
        }
    }
    svg.push_str("</svg>");
    Some(svg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn qr_matrix_generates_for_short_text() {
        let m = qr_matrix("hello").unwrap();
        assert!(!m.is_empty());
        assert_eq!(m.len(), m[0].len());
    }

    #[test]
    fn qr_matrix_too_long_returns_none() {
        let long = "a".repeat(1000);
        assert!(qr_matrix(&long).is_none());
    }

    #[test]
    fn qr_svg_contains_svg_tag() {
        let s = qr_svg("otpauth://totp/x").unwrap();
        assert!(s.starts_with("<svg"));
        assert!(s.ends_with("</svg>"));
        assert!(s.contains("<rect"));
    }

    #[test]
    fn qr_matrix_is_square() {
        for data in ["a", "hello world", "https://example.com"] {
            let m = qr_matrix(data).unwrap();
            assert_eq!(m.len(), m[0].len());
        }
    }
}
