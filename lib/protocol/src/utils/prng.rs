pub struct Xoshiro256ss([u64; 4]);

impl Xoshiro256ss {
    fn rol64(x: u64, k: u64) -> u64 {
        (x << k) | (x >> (64_u64.overflowing_sub(k).0))
    }

    pub fn next(&mut self) -> u64 {
        let result = Self::rol64(self.0[1].overflowing_mul(5).0, 7).overflowing_mul(9).0;
        let t = self.0[1] << 17;

        self.0[2] ^= self.0[0];
        self.0[3] ^= self.0[1];
        self.0[1] ^= self.0[2];
        self.0[0] ^= self.0[3];

        self.0[2] ^= t;
        self.0[3] = Self::rol64(self.0[3], 45);

        result
    }

    fn jump_inner(&mut self, jump_const: [u64; 4]) {
        let mut s0: u64 = 0;
        let mut s1: u64 = 0;
        let mut s2: u64 = 0;
        let mut s3: u64 = 0;
        for j in jump_const {
            for b in 0..u64::BITS {
                if (j & 1_u64 << b) == 1 {
                    s0 ^= self.0[0];
                    s1 ^= self.0[1];
                    s2 ^= self.0[2];
                    s3 ^= self.0[3];
                }
                self.next();
            }
        }

        self.0[0] = s0;
        self.0[1] = s1;
        self.0[2] = s2;
        self.0[3] = s3;
    }

    // suited for parallel execution
    #[allow(unused)]
    pub fn jump(&mut self) {
        const JUMP: [u64; 4] = [0x180ec6d33cfd0aba, 0xd5a61266f0c9392c, 0xa9582618e03fc9aa, 0x39abdc4529b1661c];
        self.jump_inner(JUMP)
    }

    // suited for distributed parallel execution
    #[allow(unused)]
    pub fn long_jump(&mut self) {
        const LONG_JUMP: [u64; 4] = [0x76e15d3efefdcbbf, 0xc5004e441c522fb3, 0x77710069854ee241, 0x39109bb02acbe635];
        self.jump_inner(LONG_JUMP)
    }
}

pub struct Splitmix64(u64);

impl Splitmix64 {
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn splitmix64(&mut self) -> u64 {
        self.0 = self.0.overflowing_add(0x9e3779b97f4a7c15).0;
        let mut z = self.0;
        z = (z ^ (z >> 30)).overflowing_mul(0xbf58476d1ce4e5b9).0;
        z = (z ^ (z >> 27)).overflowing_mul(0x94d049bb133111eb).0;
        z ^ (z >> 31)
    }

    pub fn xorshift256ss(&mut self) -> Xoshiro256ss {
        Xoshiro256ss([
            self.splitmix64(),
            self.splitmix64(),
            self.splitmix64(),
            self.splitmix64(),
        ])
    }
}

