//
// Created by Caelum van Ispelen on 5/20/21.
//

#ifndef RIPOSTE_YIELD_H
#define RIPOSTE_YIELD_H

#include <nlohmann/json.hpp>

namespace rip {
    struct Yield {
        int hammers = 0;
        int commerce = 0;
        int food = 0;

        Yield() = default;
        Yield(int hammers, int commerce, int food);

        Yield operator+(const Yield &other) const;

        void operator+=(const Yield &other);

        friend void from_json(const nlohmann::json &nlohmann_json_j, Yield &nlohmann_json_t) {
            if (nlohmann_json_j.contains("hammers"))
                nlohmann_json_j.at("hammers").get_to(nlohmann_json_t.hammers);
            if (nlohmann_json_j.contains("commerce"))
                nlohmann_json_j.at("commerce").get_to(nlohmann_json_t.commerce);
            if (nlohmann_json_j.contains("food"))
                nlohmann_json_j.at("food").get_to(nlohmann_json_t.food);
        }
    };
}

#endif //RIPOSTE_YIELD_H
