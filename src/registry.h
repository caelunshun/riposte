//
// Created by caelum on 5/12/21.
//

#ifndef RIPOSTE_REGISTRY_H
#define RIPOSTE_REGISTRY_H

#include <string>
#include <vector>
#include <array>
#include <nlohmann/json.hpp>
#include "assets.h"

namespace rip {
    struct CivKind : public Asset {
        // Unique string ID
        std::string id;
        // Display name ("Greece", "United States")
        std::string name;
        // Adjective ("Greek", "American")
        std::string adjective;
        // Color for borders, etc.
        std::array<uint8_t, 3> color;
        // Leader name
        std::string leader;
        // A pool of city names to use
        std::vector<std::string> cities;

        NLOHMANN_DEFINE_TYPE_INTRUSIVE(CivKind, id, name, adjective, color, leader, cities);
    };

    /**
     * A registry of civilization, unit, etc. __kinds__.
     */
    class Registry {
        std::vector<std::shared_ptr<CivKind>> civs;

    public:
        const std::vector<std::shared_ptr<CivKind>> &getCivs() const;

        void addCiv(std::shared_ptr<CivKind> c) {

            civs.push_back(std::move(c));
        }
    };

    class CivLoader : public AssetLoader {
        std::shared_ptr<Registry> registry;

    public:
        CivLoader(std::shared_ptr<Registry> registry) : registry(std::move(registry)) {}

        std::shared_ptr<Asset> loadAsset(const std::string &data) override;
    };
}

#endif //RIPOSTE_REGISTRY_H
