//! Test suite for the Web and headless browsers.

extern crate wasm_bindgen_test;
use wasm_bindgen_test::*;
use web_sys::console;

wasm_bindgen_test::wasm_bindgen_test_configure!(run_in_browser);

fn generate_random_blob(len: usize) -> Vec<u8> {
    let mut blob = vec![0; len];
    blob.iter_mut().for_each(|x| *x = rand::random());
    blob
}

trait Hasher {
    fn output_size(&self) -> usize;
    fn hash(&self, data: &[u8], out: &mut [u8]);
}

struct Blake3Hasher;
impl Hasher for Blake3Hasher {
    fn output_size(&self) -> usize {
        32
    }

    fn hash(&self, data: &[u8], out: &mut [u8]) {
        let mut hasher = blake3::Hasher::new();
        let hash = hasher.update(data).finalize();
        out.copy_from_slice(hash.as_bytes());
    }
}

struct PolymurHasher {
    inner: polymur_hash::PolymurHash,
}
impl Hasher for PolymurHasher {
    fn output_size(&self) -> usize {
        8
    }

    fn hash(&self, data: &[u8], out: &mut [u8]) {
        let hash = self.inner.hash(data);
        out.copy_from_slice(&hash.to_le_bytes());
    }
}

struct Runner {
    buf_size: usize,
    count: usize,
}

impl Runner {
    fn new(buf_size: usize, count: usize) -> Self {
        Self { buf_size, count }
    }

    fn run<T: Hasher>(&self, hasher: T) {
        let buf = generate_random_blob(self.buf_size * 4);
        let mut out = vec![0; hasher.output_size()];
        let mut offset = 0;
        for _ in 0..self.count {
            let data = &buf[offset..offset + self.buf_size];
            hasher.hash(data, &mut out);
            offset += 1;
            if offset + self.buf_size > buf.len() {
                offset = 0;
            }
        }
    }
}

#[wasm_bindgen_test]
fn polymur() {
    let runner = Runner::new(4096, 10000);

    console::time_with_label("polymur");
    runner.run(PolymurHasher {
        inner: polymur_hash::PolymurHash::new(rand::random()),
    });
    console::time_end_with_label("polymur");
}

#[wasm_bindgen_test]
fn blake() {
    let runner = Runner::new(4096, 10000);

    console::time_with_label("blake");
    runner.run(Blake3Hasher);
    console::time_end_with_label("blake");
}
