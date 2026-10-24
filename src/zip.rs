//! Lectura del primer archivo de un ZIP (los `.zevtc` de arcdps son un ZIP con un unico `.evtc`).

use crate::inflate::inflate;

/// Tope de tamano descomprimido: un log de WvW enorme ronda las decenas de MB.
const MAX_UNCOMPRESSED: usize = 400 * 1024 * 1024;

fn u16_at(data: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(data.get(at..at + 2)?.try_into().ok()?))
}

fn u32_at(data: &[u8], at: usize) -> Option<u32> {
    Some(u32::from_le_bytes(data.get(at..at + 4)?.try_into().ok()?))
}

/// Contenido del primer archivo del ZIP. Si `data` no es un ZIP se devuelve tal cual (`.evtc` sin comprimir).
pub fn first_entry(data: &[u8]) -> Result<Vec<u8>, String> {
    if u32_at(data, 0) != Some(0x0403_4b50) {
        return Ok(data.to_vec());
    }
    // el directorio central tiene los tamanos fiables aunque la cabecera local use descriptor de datos
    let (method, compressed, uncompressed, local) =
        central_entry(data).ok_or("zip: directorio central no encontrado")?;
    if u32_at(data, local) != Some(0x0403_4b50) {
        return Err("zip: cabecera local invalida".into());
    }
    let name_len = u16_at(data, local + 26).ok_or("zip: truncado")? as usize;
    let extra_len = u16_at(data, local + 28).ok_or("zip: truncado")? as usize;
    let start = local + 30 + name_len + extra_len;
    let body = data
        .get(start..start + compressed)
        .ok_or("zip: datos truncados")?;
    match method {
        0 => Ok(body.to_vec()),
        8 => inflate(body, uncompressed, MAX_UNCOMPRESSED),
        other => Err(format!("zip: metodo de compresion {other} no soportado")),
    }
}

/// (metodo, tamano comprimido, tamano descomprimido, desplazamiento de la cabecera local) de la primera entrada.
fn central_entry(data: &[u8]) -> Option<(u16, usize, usize, usize)> {
    let min = data.len().saturating_sub(22 + 0xFFFF);
    let eocd = (min..=data.len().checked_sub(22)?)
        .rev()
        .find(|&at| u32_at(data, at) == Some(0x0605_4b50))?;
    let dir = u32_at(data, eocd + 16)? as usize;
    if u32_at(data, dir)? != 0x0201_4b50 {
        return None;
    }
    Some((
        u16_at(data, dir + 10)?,
        u32_at(data, dir + 20)? as usize,
        u32_at(data, dir + 24)? as usize,
        u32_at(data, dir + 42)? as usize,
    ))
}
