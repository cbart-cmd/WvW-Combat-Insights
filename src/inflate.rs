//! Descompresor DEFLATE (RFC 1951) propio, para no depender de crates externos.
//! Sigue el esquema de `puff.c` de zlib: decodificacion canonica de Huffman bit a bit.

const MAX_BITS: usize = 15;

struct Bits<'a> {
    data: &'a [u8],
    pos: usize,
    bit_buf: u32,
    bit_cnt: u32,
}

impl<'a> Bits<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            pos: 0,
            bit_buf: 0,
            bit_cnt: 0,
        }
    }

    /// Lee `need` bits (como maximo 16), del menos al mas significativo.
    fn bits(&mut self, need: u32) -> Result<u32, String> {
        let mut val = self.bit_buf;
        while self.bit_cnt < need {
            let byte = *self.data.get(self.pos).ok_or("deflate: datos truncados")?;
            self.pos += 1;
            val |= (byte as u32) << self.bit_cnt;
            self.bit_cnt += 8;
        }
        self.bit_buf = val >> need;
        self.bit_cnt -= need;
        Ok(val & ((1u32 << need) - 1))
    }

    fn align(&mut self) {
        self.bit_buf = 0;
        self.bit_cnt = 0;
    }
}

struct Huffman {
    count: [u16; MAX_BITS + 1],
    symbol: Vec<u16>,
}

impl Huffman {
    /// Construye la tabla a partir de las longitudes de codigo. Admite codigos incompletos
    /// (validos en deflate, p. ej. un unico codigo de distancia).
    fn new(lengths: &[u8]) -> Result<Self, String> {
        let mut count = [0u16; MAX_BITS + 1];
        for &len in lengths {
            count[len as usize] += 1;
        }
        let mut left: i32 = 1;
        for len in 1..=MAX_BITS {
            left <<= 1;
            left -= count[len] as i32;
            if left < 0 {
                return Err("deflate: codigo Huffman sobresuscrito".into());
            }
        }
        let mut offs = [0u16; MAX_BITS + 1];
        for len in 1..MAX_BITS {
            offs[len + 1] = offs[len] + count[len];
        }
        let mut symbol = vec![0u16; lengths.len()];
        for (sym, &len) in lengths.iter().enumerate() {
            if len != 0 {
                symbol[offs[len as usize] as usize] = sym as u16;
                offs[len as usize] += 1;
            }
        }
        Ok(Self { count, symbol })
    }

    fn decode(&self, bits: &mut Bits) -> Result<u16, String> {
        let (mut code, mut first, mut index) = (0i32, 0i32, 0i32);
        for len in 1..=MAX_BITS {
            code |= bits.bits(1)? as i32;
            let count = self.count[len] as i32;
            if code - count < first {
                return Ok(self.symbol[(index + (code - first)) as usize]);
            }
            index += count;
            first += count;
            first <<= 1;
            code <<= 1;
        }
        Err("deflate: codigo Huffman invalido".into())
    }
}

const LEN_BASE: [u16; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131,
    163, 195, 227, 258,
];
const LEN_EXTRA: [u8; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DIST_BASE: [u16; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
    2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [u8; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13,
    13,
];

fn codes(
    bits: &mut Bits,
    out: &mut Vec<u8>,
    lit: &Huffman,
    dist: &Huffman,
    limit: usize,
) -> Result<(), String> {
    loop {
        let sym = lit.decode(bits)?;
        match sym {
            0..=255 => out.push(sym as u8),
            256 => return Ok(()),
            _ => {
                let i = (sym - 257) as usize;
                if i >= 29 {
                    return Err("deflate: longitud invalida".into());
                }
                let len = LEN_BASE[i] as usize + bits.bits(LEN_EXTRA[i] as u32)? as usize;
                let d = dist.decode(bits)? as usize;
                if d >= 30 {
                    return Err("deflate: distancia invalida".into());
                }
                let distance = DIST_BASE[d] as usize + bits.bits(DIST_EXTRA[d] as u32)? as usize;
                if distance > out.len() {
                    return Err("deflate: distancia fuera de rango".into());
                }
                let start = out.len() - distance;
                if distance >= len {
                    out.extend_from_within(start..start + len);
                } else {
                    // copia solapada: cada byte puede depender del recien escrito
                    for k in 0..len {
                        let byte = out[start + k];
                        out.push(byte);
                    }
                }
                if out.len() > limit {
                    return Err("deflate: datos descomprimidos demasiado grandes".into());
                }
            }
        }
    }
}

fn fixed_tables() -> (Huffman, Huffman) {
    let mut lengths = [0u8; 288];
    lengths[..144].fill(8);
    lengths[144..256].fill(9);
    lengths[256..280].fill(7);
    lengths[280..].fill(8);
    let lit = Huffman::new(&lengths).expect("tabla fija valida");
    let dist = Huffman::new(&[5u8; 30]).expect("tabla fija valida");
    (lit, dist)
}

fn dynamic_tables(bits: &mut Bits) -> Result<(Huffman, Huffman), String> {
    const ORDER: [usize; 19] = [
        16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
    ];
    let nlen = bits.bits(5)? as usize + 257;
    let ndist = bits.bits(5)? as usize + 1;
    let ncode = bits.bits(4)? as usize + 4;
    if nlen > 286 || ndist > 30 {
        return Err("deflate: cabecera dinamica invalida".into());
    }
    let mut lengths = [0u8; 320];
    for &idx in ORDER.iter().take(ncode) {
        lengths[idx] = bits.bits(3)? as u8;
    }
    let lencode = Huffman::new(&lengths[..19])?;
    lengths[..19].fill(0);
    let mut index = 0;
    while index < nlen + ndist {
        let sym = lencode.decode(bits)?;
        if sym < 16 {
            lengths[index] = sym as u8;
            index += 1;
            continue;
        }
        let (value, repeat) = match sym {
            16 => {
                if index == 0 {
                    return Err("deflate: repeticion sin longitud previa".into());
                }
                (lengths[index - 1], 3 + bits.bits(2)? as usize)
            }
            17 => (0, 3 + bits.bits(3)? as usize),
            _ => (0, 11 + bits.bits(7)? as usize),
        };
        if index + repeat > nlen + ndist {
            return Err("deflate: demasiadas longitudes".into());
        }
        lengths[index..index + repeat].fill(value);
        index += repeat;
    }
    if lengths[256] == 0 {
        return Err("deflate: falta el codigo de fin de bloque".into());
    }
    let lit = Huffman::new(&lengths[..nlen])?;
    let dist = Huffman::new(&lengths[nlen..nlen + ndist])?;
    Ok((lit, dist))
}

/// Descomprime un flujo DEFLATE crudo. `limit` corta descompresiones anormalmente grandes.
pub fn inflate(data: &[u8], size_hint: usize, limit: usize) -> Result<Vec<u8>, String> {
    let mut bits = Bits::new(data);
    let mut out = Vec::with_capacity(size_hint.min(limit));
    loop {
        let last = bits.bits(1)? == 1;
        match bits.bits(2)? {
            0 => {
                bits.align();
                let header = bits
                    .data
                    .get(bits.pos..bits.pos + 4)
                    .ok_or("deflate: bloque almacenado truncado")?;
                let len = u16::from_le_bytes([header[0], header[1]]);
                let nlen = u16::from_le_bytes([header[2], header[3]]);
                if len != !nlen {
                    return Err("deflate: longitud de bloque almacenado invalida".into());
                }
                bits.pos += 4;
                let chunk = bits
                    .data
                    .get(bits.pos..bits.pos + len as usize)
                    .ok_or("deflate: bloque almacenado truncado")?;
                out.extend_from_slice(chunk);
                bits.pos += len as usize;
                if out.len() > limit {
                    return Err("deflate: datos descomprimidos demasiado grandes".into());
                }
            }
            1 => {
                let (lit, dist) = fixed_tables();
                codes(&mut bits, &mut out, &lit, &dist, limit)?;
            }
            2 => {
                let (lit, dist) = dynamic_tables(&mut bits)?;
                codes(&mut bits, &mut out, &lit, &dist, limit)?;
            }
            _ => return Err("deflate: tipo de bloque invalido".into()),
        }
        if last {
            return Ok(out);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::inflate;

    #[test]
    fn stored_block() {
        // bloque final almacenado con "hola"
        let data = [0x01, 0x04, 0x00, 0xFB, 0xFF, b'h', b'o', b'l', b'a'];
        assert_eq!(inflate(&data, 0, 1024).unwrap(), b"hola");
    }

    #[test]
    fn fixed_huffman_with_overlapping_copy() {
        // "hello hello hello" comprimido por zlib (deflate crudo, nivel 9)
        let data = [0xCB, 0x48, 0xCD, 0xC9, 0xC9, 0x57, 0xC8, 0x40, 0x90, 0x00];
        assert_eq!(inflate(&data, 0, 1024).unwrap(), b"hello hello hello");
    }

    #[test]
    fn limit_is_enforced() {
        let data = [0xCB, 0x48, 0xCD, 0xC9, 0xC9, 0x57, 0xC8, 0x40, 0x90, 0x00];
        assert!(inflate(&data, 0, 4).is_err());
    }
}
