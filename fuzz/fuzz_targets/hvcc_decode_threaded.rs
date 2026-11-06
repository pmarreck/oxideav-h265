//! [`hvcc_decode`](hvcc_decode.rs) with the decoder given four threads, so
//! hostile input also runs through picture-parallel decoding (frame
//! threads that wait on the rows of the pictures they reference) and the
//! wavefront rows. Input layout and contract are `hvcc_decode`'s: every
//! path returns `Ok` / `Err`, with no panic, no abort, no unbounded
//! allocation, and no thread left waiting (run it with
//! `-ignore_timeouts=0` so a hang counts as a failure).

#![no_main]

use libfuzzer_sys::fuzz_target;
use oxideav_core::{CodecParameters, Error, ExecutionContext, Packet, TimeBase};

fuzz_target!(|data: &[u8]| {
    if data.len() < 2 {
        return;
    }
    let hvcc_len = usize::from(u16::from_be_bytes([data[0], data[1]]));
    let Some(hvcc) = data.get(2..2 + hvcc_len) else {
        return;
    };
    let payload = &data[2 + hvcc_len..];

    let mut params = CodecParameters::video("h265".into());
    params.extradata = hvcc.to_vec();
    let Ok(mut dec) = oxideav_h265::make_decoder(&params) else {
        return;
    };
    dec.set_execution_context(&ExecutionContext { threads: 4 });
    let _ = dec.send_packet(&Packet::new(0, TimeBase::new(1, 1), payload.to_vec()));
    let _ = dec.flush();
    while let Ok(_) | Err(Error::NeedMore) = dec.receive_frame() {}
});
