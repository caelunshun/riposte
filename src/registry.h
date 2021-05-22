//
// Created by Caelum van Ispelen on 5/12/21.
//

#ifndef RIPOSTE_REGISTRY_H
#define RIPOSTE_REGISTRY_H

#include <string>
#include <vector>
#include <array>
#include <absl/container/flat_hash_map.h>
#include <nlohmann/json.hpp>
#include "assets.h"
#include "yield.h"

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
        // How many hammers it costs to build this unit.
        int cost;
        // Techs that need to be unlocked before building this unit.
        std::vector<std::string> techs;
        // Resources required to build the unit.
        std::vector<std::string> resources;

        friend void from_json(const nlohmann::json &nlohmann_json_j, UnitKind &nlohmann_json_t) {
            nlohmann_json_j.at("id").get_to(nlohmann_json_t.id);
            nlohmann_json_j.at("name").get_to(nlohmann_json_t.name);
            nlohmann_json_j.at("strength").get_to(nlohmann_json_t.strength);
            nlohmann_json_j.at("movement").get_to(nlohmann_json_t.movement);
            if (nlohmann_json_j.contains("capabilities")) {
                nlohmann_json_j.at("capabilities").get_to(nlohmann_json_t.capabilities);
            }
            nlohmann_json_j.at("cost").get_to(nlohmann_json_t.cost);
            nlohmann_json_j.at("techs").get_to(nlohmann_json_t.techs);
            if (nlohmann_json_j.contains("resources")) {
                nlohmann_json_j.at("resources").get_to(nlohmann_json_t.resources);
            }
        }
    };

    struct Resource : public Asset {
        std::string id;
        std::string name;

        // Tech (name) required to reveal the resource
        std::string revealedBy;

        // Bonus added to tiles with this resource (when revealed)
        Yield yieldBonus;

        // Improvement required to harvest resource
        std::string improvement;
        // Extra yield when the resource is improved.
        // Added on top of yieldBonus.
        Yield improvedBonus;

        // Determines how frequently the resource is generated.
        // Units are in resources/1000 tiles.
        float scarcity;

        friend void from_json(const nlohmann::json &nlohmann_json_j, Resource &nlohmann_json_t) {
            nlohmann_json_j.at("id").get_to(nlohmann_json_t.id);
            nlohmann_json_j.at("name").get_to(nlohmann_json_t.name);
            nlohmann_json_j.at("revealedBy").get_to(nlohmann_json_t.revealedBy);
            nlohmann_json_j.at("yieldBonus").get_to(nlohmann_json_t.yieldBonus);
            nlohmann_json_j.at("improvement").get_to(nlohmann_json_t.improvement);
            nlohmann_json_j.at("improvedBonus").get_to(nlohmann_json_t.improvedBonus);
            nlohmann_json_j.at("scarcity").get_to(nlohmann_json_t.scarcity);
        }
    };

    struct ResourceHash {
        size_t operator()(const std::shared_ptr<Resource> &resource)const
        {
            return std::hash<std::string>()(resource->name);
        }

        bool operator()(const std::shared_ptr<Resource> &a, const std::shared_ptr<Resource> &b)const
        {
            return a->name == b->name;
        }
    };

    /**
     * A registry of civilization, unit, etc. __kinds__.
     */
    class Registry {
        std::vector<std::shared_ptr<CivKind>> civs;
        std::vector<std::shared_ptr<UnitKind>> units;
        absl::flat_hash_map<std::string, std::shared_ptr<Resource>> resources;

    public:
        const std::vector<std::shared_ptr<CivKind>> &getCivs() const;

        void addCiv(std::shared_ptr<CivKind> c) {
            civs.push_back(std::move(c));
        }

        void addUnit(std::shared_ptr<UnitKind> u) {
            units.push_back(std::move(u));
        }

        void addResource(std::shared_ptr<Resource> r) {
            resources.emplace(r->name, std::move(r));
        }

        const std::shared_ptr<Resource> &getResource(const std::string &name) const {
            return resources.at(name);
        }

        const std::vector<std::shared_ptr<UnitKind>> &getUnits() const;

        const absl::flat_hash_map<std::string, std::shared_ptr<Resource>> &getResources() const;
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

    class ResourceLoader : public AssetLoader {
        std::shared_ptr<Registry> registry;

    public:
        ResourceLoader(std::shared_ptr<Registry> registry) : registry(std::move(registry)) {}

        std::shared_ptr<Asset> loadAsset(const std::string &data) override;
    };
}

#endif //RIPOSTE_REGISTRY_H
