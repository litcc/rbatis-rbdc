use bytes::{Buf, Bytes};

use crate::io::MySqlBufExt;
use crate::protocol::Row;
use crate::result_set::MySqlColumn;
use rbdc::io::Decode;
use rbdc::Error;

#[derive(Debug)]
pub struct TextRow(pub Row);

impl<'de> Decode<'de, &'de [MySqlColumn]> for TextRow {
    fn decode_with(mut buf: Bytes, columns: &'de [MySqlColumn]) -> Result<Self, Error> {
        let storage = buf.clone();
        let offset = buf.len();

        let mut values = Vec::with_capacity(columns.len());

        for _ in columns {
            if buf[0] == 0xfb {
                // NULL is sent as 0xfb
                values.push(None);
                buf.advance(1);
            } else {
                let size = buf.get_uint_lenenc() as usize;
                let offset = offset - buf.len();

                // bounds check to avoid panic when server returns non-MySQL-compliant payloads (e.g., ClickHouse MySQL compatibility layer)
                if size > buf.len() {
                    return Err(rbdc::Error::protocol(format!(
                        "text row column length {} exceeds remaining {} bytes (first byte: 0x{:02x})",
                        size,
                        buf.len(),
                        storage[offset] // original first byte of this value in storage snapshot
                    )));
                }

                values.push(Some(offset..(offset + size)));

                buf.advance(size);
            }
        }
        Ok(TextRow(Row::from((values, storage.to_vec()))))
    }
}
