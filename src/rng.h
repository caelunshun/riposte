//
// Created by Caelum van Ispelen on 5/12/21.
//

#ifndef RIPOSTE_RNG_H
#define RIPOSTE_RNG_H

#include <cstdint>
#include <iterator>
#include <array>
#include <string>
#include <vector>

namespace rip {
    /**
        * A random number generator based on PCG XSH RR 64/32.
        * Fast and simple but not cryptographic.
        */
    class Rng {
    public:
        /**
         * Creates an Rng with the given seed.
         */
        explicit Rng(uint64_t seed);

        /**
         * Creates an Rng seeded from the OS entropy source.
         * This is not deterministic across multiple runs or
         * Rng creations.
         */
        Rng();

        /**
         * Sets the RNG's seed to a new value.
         */
        void seed(uint64_t seed);

        /**
         * Generates a random 32-bit integer in the range [a, b).
         */
        uint32_t u32(uint32_t a, uint32_t b);

        /**
         * Generates a random 64-bit integer in the range [a, b).
         */
        uint64_t u64(uint64_t a, uint64_t b);

        /**
         * Generates a boolean with probability p of being true.
         */
        bool chance(double p);

        /**
         * Generates a random 32-bit float in the range [0.0, 1.0).
         */
        float f32();

        /**
         * Randomly shuffles an iterator.
         */
        template <class It> void shuffle(It first, It last) {
            typename std::iterator_traits<It>::difference_type i, n;
            n = last - first;
            for (i = n - 1; i > 0; i--) {
                std::swap(first[i], first[gen_mod_u32(i + 1)]);
            }
        }

        /**
         * Chooses a random value from an array.
         */
         template <class T, size_t Size> T choose(const std::array<T, Size> &array) {
             auto index = static_cast<size_t>(u32(0, Size));
             return array[index];
         }

    private:
        uint64_t state_;

        uint32_t gen_u32();

        uint64_t gen_u64();

        uint32_t gen_mod_u32(uint32_t n);

        uint64_t gen_mod_u64(uint64_t n);
    };

    // Picks random values from an array in a way that implements the gambler's
    // fallacy. That is, values are less likely to be chosen twice in a row...
    template<class T>
    class FairPicker {
        Rng rng;
        std::vector<T> options;
        std::vector<float> weights;

        void updateWeightsForChoice(int choiceIndex) {
            // Increase all other weights. Decrease this weight.
            weights[choiceIndex] = 0;
            for (int i = 0; i < weights.size(); i++) {
                if (i != choiceIndex) {
                    weights[i] += 1;
                }
            }
        }

    public:
        FairPicker() = default;

        explicit FairPicker(Rng rng) : rng(rng) {}

        void addChoice(T choice) {
            options.push_back(std::move(choice));
            weights.push_back(1);
        }

        T pickNext() {
            float weightSum = 0;
            for (const auto weight : weights) weightSum += weight;

            float choice = rng.f32() * weightSum;

            assert(options.size() == weights.size());
            float cursor = 0;
            for (int i = 0; i < options.size(); i++) {
                if (choice < cursor + weights[i]) {
                    updateWeightsForChoice(i);
                    return options[i];
                }
                cursor += weights[i];
            }

            throw std::string("zero options");
        }
    };
}

#endif //RIPOSTE_RNG_H
