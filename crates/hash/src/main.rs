use std::env;

fn generate_random_blob(len: usize) -> Vec<u8> {
    let mut blob = vec![0; len];
    blob.iter_mut().for_each(|x| *x = rand::random());
    blob
}

trait Hasher {
    fn output_size(&self) -> usize;
    fn hash(&self, data: &[u8], out: &mut [u8]);
}

struct XorHash;
impl Hasher for XorHash {
    fn output_size(&self) -> usize {
        8
    }

    fn hash(&self, data: &[u8], out: &mut [u8]) {
        // scan through the data, xoring 8 bytes at a time
        let mut hash = 0;
        let mut i = 0;
        while i + 8 <= data.len() {
            hash ^= u64::from_le_bytes(data[i..i + 8].try_into().unwrap());
            i += 8;
        }
        // handle the remaining bytes
        if i < data.len() {
            let mut last = [0; 8];
            last[..data.len() - i].copy_from_slice(&data[i..]);
            hash ^= u64::from_le_bytes(last);
        }
        // write the hash to the output buffer
        out.copy_from_slice(&hash.to_le_bytes());
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct UmashFingerprint {
    params: umash::Params,
}
#[cfg(not(target_arch = "wasm32"))]
impl Hasher for UmashFingerprint {
    fn output_size(&self) -> usize {
        16
    }

    fn hash(&self, data: &[u8], out: &mut [u8]) {
        let mut hasher = self.params.fingerprinter(0);
        hasher.write(data);
        let hash = hasher.digest().hash;
        let hash_bytes = unsafe { std::slice::from_raw_parts(hash.as_ptr() as *const u8, 16) };
        out.copy_from_slice(hash_bytes);
    }
}

#[cfg(not(target_arch = "wasm32"))]
struct UmashComponent {
    params: umash::Params,
    component: umash::UmashComponent,
}
#[cfg(not(target_arch = "wasm32"))]
impl Hasher for UmashComponent {
    fn output_size(&self) -> usize {
        8
    }

    fn hash(&self, data: &[u8], out: &mut [u8]) {
        let mut hasher = self.params.component_hasher(0, self.component);
        hasher.write(data);
        let hash = hasher.digest();
        out.copy_from_slice(&hash.to_le_bytes());
    }
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
        let buf = generate_random_blob(self.buf_size);
        let mut out = vec![0; hasher.output_size()];
        for _ in 0..self.count {
            hasher.hash(&buf, &mut out);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 4 {
        println!("usage: hash <hasher> <buf_size> <count>");
        return Ok(());
    }
    let hasher = &args[1];
    let buf_size = args[2].parse::<usize>()?;
    let count = args[3].parse::<usize>()?;

    let runner = Runner::new(buf_size, count);

    match hasher.as_str() {
        "xor" => runner.run(XorHash),

        #[cfg(not(target_arch = "wasm32"))]
        "umash-fingerprint" => runner.run(UmashFingerprint {
            params: umash::Params::new(),
        }),
        #[cfg(not(target_arch = "wasm32"))]
        "umash-primary" => runner.run(UmashComponent {
            params: umash::Params::new(),
            component: umash::UmashComponent::Hash,
        }),
        #[cfg(not(target_arch = "wasm32"))]
        "umash-secondary" => runner.run(UmashComponent {
            params: umash::Params::new(),
            component: umash::UmashComponent::Secondary,
        }),

        "blake3" => runner.run(Blake3Hasher),
        "polymur" => runner.run(PolymurHasher {
            inner: polymur_hash::PolymurHash::new(rand::random()),
        }),

        _ => {
            return Err(format!("unknown hasher: {}", hasher).into());
        }
    };

    Ok(())
}
