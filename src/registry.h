//
// Created by Caelum van Ispelen on 5/12/21.
//

#ifndef RIPOSTE_REGISTRY_H
#define RIPOSTE_REGISTRY_H

#include <string>
#include <vector>
#include <array>
#include <absl/container/flat_hash_map.h>
#include <absl/container/flat_hash_set.h>
#include <nlohmann/json.hpp>
#include "assets.h"
#include "yield.h"

namespace rip {
    struct Leader {
        // Leader name (Lincoln etc.)
        std::string name;

        // Leader traits - used in AI.
        // Range is [0, 10].
        float aggressive;
        float nukemonger;
        float submissive;
        float paranoia;
        float expansiveness;
        float religious;

        NLOHMANN_DEFINE_TYPE_INTRUSIVE(Leader, name, aggressive, nukemonger, submissive, paranoia, expansiveness, religious);
    };

    struct CivKind : public Asset {
        // Unique string ID
        std::string id;
        // Display name ("Greece", "United States")
        std::string name;
        // Adjective ("Greek", "American")
        std::string adjective;
        // Color for borders, etc.
        std::array<uint8_t, 3> color;
        // List of possible leaders for the civ
        std::vector<Leader> leaders;
        // A pool of city names to use
        std::vector<std::string> cities;
        // List of starting tech names
        std::vector<std::string> startingTechs;

        NLOHMANN_DEFINE_TYPE_INTRUSIVE(CivKind, id, name, adjective, color, leaders, cities, startingTechs);
    };

    class ParseException : public std::exception {
        std::string message;

    public:
        ParseException(std::string message) : message(std::move(message)) {}

        const char *what() const noexcept override {
            return message.c_str();
        }
    };

    struct CombatBonus {
        int whenInCityBonus = 0;
        int againstUnitCategoryBonus = 0;
        int againstUnitBonus = 0;
        bool onlyOnAttack = false;
        bool onlyOnDefense = false;
        std::string unit;
        std::string unitCategory;

        friend void from_json(const nlohmann::json &json, CombatBonus &bonus) {
            auto type = json.at("type").get<std::string>();
            int *targetBonus = nullptr;
            if (type == "whenInCity") {
                targetBonus = &bonus.whenInCityBonus;
            } else if (type == "againstUnit") {
                targetBonus = &bonus.againstUnitBonus;
                bonus.unit = json.at("unit").get<std::string>();
            } else if (type == "againstUnitCategory") {
                targetBonus = &bonus.againstUnitCategoryBonus;
                bonus.unitCategory = json.at("category").get<std::string>();
            } else {
                throw ParseException("unrecognized combat bonus '" + type + "'");
            }

            *targetBonus = json.at("bonusPercent").get<int>();

            if (json.contains("onlyOnAttack")) {
                bonus.onlyOnAttack = json["onlyOnAttack"].get<bool>();
            }
            if (json.contains("onlyOnDefense")) {
                bonus.onlyOnDefense = json["onlyOnDefense"].get<bool>();
            }
        }
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
        // Specialized combat bonuses.
        std::vector<CombatBonus> combatBonuses;
        // Category of the unit - melee, mounted, gunpowder, etc.
        std::string category;
        // Whether the unit can only travel on water
        bool ship = false;
        // If the unit has the carry_units capability,
        // this is the number of units it can hold.
        int carryUnitCapacity = 0;
        // If the unit has the bombard_city_defenses capability,
        // this is the max damage per turn.
        int maxBombardPerTurn = 0;

        // Whether the unit deals collateral damage.
        bool doesCollateralDamage = false;
        // Maximum number of targets for collateral damage.
        int maxCollateralTargets = 0;

        // The civs that are able to build this unit.
        // If empty, defaults to all civs.
        absl::flat_hash_set<std::string> onlyForCivs;

        friend void from_json(const nlohmann::json &json, UnitKind &unit) {
            json.at("id").get_to(unit.id);
            json.at("name").get_to(unit.name);
            json.at("strength").get_to(unit.strength);
            json.at("movement").get_to(unit.movement);
            if (json.contains("capabilities")) {
                json.at("capabilities").get_to(unit.capabilities);
            }
            json.at("cost").get_to(unit.cost);
            json.at("techs").get_to(unit.techs);
            if (json.contains("resources")) {
                json.at("resources").get_to(unit.resources);
            }

            if (json.contains("combatBonuses")) {
                json.at("combatBonuses").get_to(unit.combatBonuses);
            }

            json.at("category").get_to(unit.category);

            if (json.contains("ship")) {
                json.at("ship").get_to(unit.ship);
            }

            if (json.contains("carryUnitCapacity")) {
                json.at("carryUnitCapacity").get_to(unit.carryUnitCapacity);
            }

            if (json.contains("maxBombardPerTurn")) {
                json.at("maxBombardPerTurn").get_to(unit.maxBombardPerTurn);
            }

            if (json.contains("doesCollateralDamage")) {
                json.at("doesCollateralDamage").get_to(unit.doesCollateralDamage);
            }
            if (json.contains("maxCollateralTargets")) {
                json.at("maxCollateralTargets").get_to(unit.maxCollateralTargets);
            }

            if (json.contains("onlyForCivs")) {
                std::vector<std::string> civs;
                json.at("onlyForCivs").get_to(civs);
                for (auto &c : civs) {
                    unit.onlyForCivs.insert(std::move(c));
                }
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

        uint32_t healthBonus = 0;
        uint32_t happyBonus = 0;

        friend void from_json(const nlohmann::json &nlohmann_json_j, Resource &nlohmann_json_t) {
            nlohmann_json_j.at("id").get_to(nlohmann_json_t.id);
            nlohmann_json_j.at("name").get_to(nlohmann_json_t.name);
            nlohmann_json_j.at("revealedBy").get_to(nlohmann_json_t.revealedBy);
            nlohmann_json_j.at("yieldBonus").get_to(nlohmann_json_t.yieldBonus);
            nlohmann_json_j.at("improvement").get_to(nlohmann_json_t.improvement);
            nlohmann_json_j.at("improvedBonus").get_to(nlohmann_json_t.improvedBonus);
            nlohmann_json_j.at("scarcity").get_to(nlohmann_json_t.scarcity);
            if (nlohmann_json_j.contains("healthBonus")) {
                nlohmann_json_j.at("healthBonus").get_to(nlohmann_json_t.healthBonus);
            }
            if (nlohmann_json_j.contains("happyBonus")) {
                nlohmann_json_j.at("happyBonus").get_to(nlohmann_json_t.happyBonus);
            }
        }
    };

    // An effect given by a building.
    struct BuildingEffect {
        int bonusHammers = 0;
        int bonusHammerPercent = 0;
        int bonusCommerce = 0;
        int bonusCommercePercent = 0;
        int bonusFood = 0;
        int bonusFoodPercent = 0;


        // Difference with bonusCommerce:
        // gold applies only to gold production after the
        // research slider is applied.
        int bonusGold = 0;
        int bonusGoldPercent = 0;
        int bonusBeakers = 0;
        int bonusBeakerPercent = 0;

        int bonusCulture = 0;
        int bonusCulturePercent = 0;

        int defenseBonusPercent = 0;

        int minusMaintenancePercent = 0;

        bool hasGranaryFoodStore = false;

        int oceanFoodBonus = 0;

        int happiness = 0;

        void operator+=(const BuildingEffect &o) {

            bonusHammers += o.bonusHammers;
            bonusHammerPercent += o.bonusHammerPercent;
            bonusCommerce += o.bonusCommerce;
            bonusCommercePercent += o.bonusCommercePercent;
            bonusFood += o.bonusFood;
            bonusFoodPercent += o.bonusFoodPercent;
            bonusGold += o.bonusGold;
            bonusGoldPercent += o.bonusGoldPercent;
            bonusBeakers += o.bonusBeakers;
            bonusBeakerPercent += o.bonusBeakerPercent;
            bonusCulture += o.bonusCulture;
            bonusCulturePercent += o.bonusCulturePercent;
            defenseBonusPercent += o.defenseBonusPercent;
            hasGranaryFoodStore |= o.hasGranaryFoodStore;
            oceanFoodBonus += o.oceanFoodBonus;
            minusMaintenancePercent += o.minusMaintenancePercent;
            happiness += o.happiness;

        }

        friend void from_json(const nlohmann::json &json, BuildingEffect &e) {
            auto type = json.at("type").get<std::string>();

            if (type == "granaryFoodStore") {
                e.hasGranaryFoodStore = true;
            } else {
                auto amount = json.at("amount").get<int>();
                int *target = nullptr;
                if (type == "bonusHammers") {
                    target = &e.bonusHammers;
                } else if (type == "bonusHammerPercent") {
                    target = &e.bonusHammerPercent;
                } else if (type == "bonusCommerce") {
                    target = &e.bonusCommerce;
                } else if (type == "bonusCommercePercent") {
                    target = &e.bonusCommercePercent;
                } else if (type == "bonusFood") {
                    target = &e.bonusFood;
                } else if (type == "bonusFoodPercent") {
                    target = &e.bonusFoodPercent;
                } else if (type == "bonusGold") {
                    target = &e.bonusGold;
                } else if (type == "bonusGoldPercent") {
                    target = &e.bonusGoldPercent;
                } else if (type == "bonusBeakers") {
                    target = &e.bonusBeakers;
                } else if (type == "bonusBeakerPercent") {
                    target = &e.bonusBeakerPercent;
                } else if (type == "bonusCulture") {
                    target = &e.bonusCulture;
                } else if (type == "bonusCulturePercent") {
                    target = &e.bonusCulturePercent;
                } else if (type == "defenseBonusPercent") {
                    target = &e.defenseBonusPercent;
                } else if (type == "oceanFoodBonus") {
                    target = &e.oceanFoodBonus;
                } else if (type == "minusMaintenancePercent") {
                    target = &e.minusMaintenancePercent;
                } else if (type == "happiness") {
                    target = &e.happiness;
                } else {
                    throw ParseException("unknown building effect type '" + type + "'");
                }

                *target = amount;
            }
        }
    };

    struct Building : public Asset {
        // Name displayed in the UI
        std::string name;
        // Cost in hammers
        int cost;
        // Any buildings required in a city before it can build this building
        std::vector<std::string> prerequisites;
        // Techs required to build
        std::vector<std::string> techs;
        // Whether the building can only be built in coastal cities
        bool onlyCoastal = false;
        // Effects of the building when built
        std::vector<BuildingEffect> effects;

        // Same as UnitKind.onlyForCivs
        absl::flat_hash_set<std::string> onlyForCivs;

        friend void from_json(const nlohmann::json &json, Building &b) {
            json.at("name").get_to(b.name);
            json.at("cost").get_to(b.cost);
            if (json.contains("prerequisites")) {
                json.at("prerequisites").get_to(b.prerequisites);
            }
            json.at("techs").get_to(b.techs);
            if (json.contains("onlyCoastal")) {
                json.at("onlyCoastal").get_to(b.onlyCoastal);
            }
            json.at("effects").get_to(b.effects);

            if (json.contains("onlyForCivs")) {
                std::vector<std::string> civs;
                json.at("onlyForCivs").get_to(civs);
                for (auto &c : civs) {
                    b.onlyForCivs.insert(std::move(c));
                }
            }
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
        std::vector<std::shared_ptr<Building>> buildings;

    public:
        const std::vector<std::shared_ptr<CivKind>> &getCivs() const;

        const std::vector<std::shared_ptr<Building>> &getBuildings() const;

        void addCiv(std::shared_ptr<CivKind> c) {
            civs.push_back(std::move(c));
        }

        void addUnit(std::shared_ptr<UnitKind> u) {
            units.push_back(std::move(u));
        }

        void addResource(std::shared_ptr<Resource> r) {
            resources.emplace(r->id, std::move(r));
        }

        void addBuilding(std::shared_ptr<Building> b) {
            buildings.push_back(std::move(b));
        }

        const std::shared_ptr<Resource> &getResource(const std::string &name) const {
            return resources.at(name);
        }

        const std::vector<std::shared_ptr<UnitKind>> &getUnits() const;

        const std::shared_ptr<UnitKind> &getUnit(const std::string &id) const;

        const absl::flat_hash_map<std::string, std::shared_ptr<Resource>> &getResources() const;

        const std::shared_ptr<Building> &getBuilding(const std::string &name) const;

        const std::shared_ptr<CivKind> &getCiv(const std::string &id) const;
    };

    class CivLoader : public AssetLoader {
        std::shared_ptr<Registry> registry;

    public:
        CivLoader(std::shared_ptr<Registry> registry) : registry(std::move(registry)) {}

        std::shared_ptr<Asset> loadAsset(const std::string &id, const std::string &data) override;
    };

    class UnitLoader : public AssetLoader {
        std::shared_ptr<Registry> registry;

    public:
        UnitLoader(std::shared_ptr<Registry> registry) : registry(std::move(registry)) {}

        std::shared_ptr<Asset> loadAsset(const std::string &id, const std::string &data) override;
    };

    class ResourceLoader : public AssetLoader {
        std::shared_ptr<Registry> registry;

    public:
        ResourceLoader(std::shared_ptr<Registry> registry) : registry(std::move(registry)) {}

        std::shared_ptr<Asset> loadAsset(const std::string &id, const std::string &data) override;
    };

    class BuildingLoader : public AssetLoader {
        std::shared_ptr<Registry> registry;

    public:
        BuildingLoader(std::shared_ptr<Registry> registry) : registry(std::move(registry)) {}

        std::shared_ptr<Asset> loadAsset(const std::string &id, const std::string &data) override;
    };
}

#endif //RIPOSTE_REGISTRY_H
