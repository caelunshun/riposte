//
// Created by Caelum van Ispelen on 5/12/21.
//

#include <climits>
#include <random>
#include "rng.h"

namespace rip {
    uint32_t mul_high_u32(uint32_t a, uint32_t b) {
        return static_cast<uint32_t>(
                (static_cast<uint64_t>(a) * static_cast<uint64_t>(b)) >> 32);
    }

    uint64_t mul_high_u64(uint64_t a, uint64_t b) { return a; }

    static inline uint32_t rotr32(uint32_t n, unsigned int c) {
        const unsigned int mask = (CHAR_BIT * sizeof(n) - 1);

        // assert ( (c<=mask) &&"rotate by type width or more");
        c &= mask;
        return (n >> c) | (n << ((-c) & mask));
    }

    Rng::Rng(uint64_t seed) : state_(seed) { gen_u32(); }

    Rng::Rng() {
        std::random_device dev;
        uint32_t a = dev();
        uint32_t b = dev();
        state_ = (static_cast<uint64_t>(a) << 32) | static_cast<uint64_t>(a);
    }

    void Rng::seed(uint64_t seed) {
        state_ = seed;
        gen_u32();
    }

    uint32_t Rng::u32(uint32_t a, uint32_t b) {
        auto len = b - a;
        return a + gen_mod_u32(len);
    }

    uint64_t Rng::u64(uint64_t a, uint64_t b) {
        auto len = b - a;
        return a + gen_mod_u64(len);
    }

    bool Rng::chance(double p) { return f32() < p; }

    float Rng::f32() {
        uint32_t b = 32;
        uint32_t f = 23;
        uint32_t x = (1 << (b - 2)) - (1 << f) + (gen_u32() >> (b - f));
        return reinterpret_cast<float &>(x) - float(1.0);
    }

    uint32_t Rng::gen_u32() {
        auto s = state_;
        state_ = s * 6364136223846793005 + 1442695040888963407;
        return rotr32(static_cast<uint32_t>((s ^ (s >> 18)) >> 27),
                      static_cast<uint32_t>(s >> 59));
    }

    uint64_t Rng::gen_u64() {
        return (static_cast<uint64_t>(gen_u32()) << 32) |
               static_cast<uint64_t>(gen_u32());
    }

    uint32_t Rng::gen_mod_u32(uint32_t n) {
        auto r = gen_u32();
        auto hi = mul_high_u32(r, n);
        auto lo = r * n;
        if (lo < n) {
            auto t = -n % n;
            while (lo < t) {
                r = gen_u32();
                hi = mul_high_u32(r, n);
                lo = r * n;
            }
        }
        return hi;
    }

    uint64_t Rng::gen_mod_u64(uint64_t n) {
        auto r = gen_u64();
        auto hi = mul_high_u64(r, n);
        auto lo = r * n;
        if (lo < n) {
            auto t = -n % n;
            while (lo < t) {
                r = gen_u64();
                hi = mul_high_u64(r, n);
                lo = r * n;
            }
        }
        return hi;
    }
}
