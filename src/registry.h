//
// Created by Caelum van Ispelen on 5/12/21.
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

    struct UnitKind : public Asset {
        // Unique string ID
        std::string id;
        // Display name
        std::string name;
        // Combat strength
        double strength;
        // How many tiles we can move per turn
        int movement;
        // Capabilities (e.g. found city, do work)
        std::vector<std::string> capabilities;

        NLOHMANN_DEFINE_TYPE_INTRUSIVE(UnitKind, id, name, strength, capabilities);
    };

    /**
     * A registry of civilization, unit, etc. __kinds__.
     */
    class Registry {
        std::vector<std::shared_ptr<CivKind>> civs;
        std::vector<std::shared_ptr<UnitKind>> units;

    public:
        const std::vector<std::shared_ptr<CivKind>> &getCivs() const;

        void addCiv(std::shared_ptr<CivKind> c) {
            civs.push_back(std::move(c));
        }

        void addUnit(std::shared_ptr<UnitKind> u) {
            units.push_back(std::move(u));
        }

        const std::vector<std::shared_ptr<UnitKind>> &getUnits() const;
    };

    class CivLoader : public AssetLoader {
        std::shared_ptr<Registry> registry;

    public:
        CivLoader(std::shared_ptr<Registry> registry) : registry(std::move(registry)) {}

        std::shared_ptr<Asset> loadAsset(const std::string &data) override;
    };

    class UnitLoader : public AssetLoader {
        std::shared_ptr<Registry> registry;

    public:
        UnitLoader(std::shared_ptr<Registry> registry) : registry(std::move(registry)) {}

        std::shared_ptr<Asset> loadAsset(const std::string &data) override;
    };
}

#endif //RIPOSTE_REGISTRY_H
