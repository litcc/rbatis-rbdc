use crate::types::date::decode_date_buf;
use crate::types::time::decode_time;
use crate::types::{Decode, Encode};
use crate::value::{MySqlValue, MySqlValueFormat};
use rbdc::datetime::DateTime;
use rbdc::Error;

impl Encode for DateTime {
    fn encode(self, buf: &mut Vec<u8>) -> Result<usize, Error> {
        let datetime = self.0;
        let datetime_size = date_time_size_hint(
            datetime.hour(),
            datetime.minute(),
            datetime.sec(),
            datetime.nano(),
        );
        buf.push(datetime_size as u8);
        let date = rbdc::date::Date(fastdate::Date {
            day: datetime.day(),
            mon: datetime.mon(),
            year: datetime.year(),
        });
        let mut size = date.encode(buf)?;
        buf.remove(buf.len() - (size + 1));
        if datetime_size > 4 {
            let time = fastdate::Time {
                nano: datetime.nano(),
                sec: datetime.sec(),
                minute: datetime.minute(),
                hour: datetime.hour(),
            };
            let before_len = buf.len();
            let encode_len = time.encode(buf)?;
            if encode_len > 6 {
                for _ in 0..6 {
                    buf.remove(before_len);
                }
            }
            let after_len = buf.len();
            size += after_len - before_len;
        }
        Ok(1 + size)
    }
}

impl Decode for DateTime {
    fn decode(value: MySqlValue) -> Result<Self, Error> {
        Ok(match value.format() {
            MySqlValueFormat::Text => Self({
                let s = value.as_str()?;
                fastdate::DateTime::from_str_default(s, value.option.offset_sec)
                    .map_err(|e| Error::from(e.to_string()))?
            }),
            MySqlValueFormat::Binary => {
                let buf = value.as_bytes()?;
                let len = buf[0];
                let date = decode_date_buf(&buf[1..])?;
                let time = if len > 4 {
                    decode_time(&buf[5..])
                } else {
                    fastdate::Time {
                        nano: 0,
                        sec: 0,
                        minute: 0,
                        hour: 0,
                    }
                };
                let v = fastdate::DateTime::from((date, time, value.option.offset_sec));
                Self(v)
            }
        })
    }
}

fn date_time_size_hint(hour: u8, min: u8, sec: u8, nano: u32) -> usize {
    // to save space the packet can be compressed:
    match (hour, min, sec, nano) {
        // if hour, minutes, seconds and micro_seconds are all 0,
        // length is 4 and no other field is sent
        (0, 0, 0, 0) => 4,

        // if micro_seconds is 0, length is 7
        // and micro_seconds is not sent
        (_, _, _, 0) => 7,

        // otherwise length is 11
        (_, _, _, _) => 11,
    }
}
