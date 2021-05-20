//
// Created by Caelum van Ispelen on 5/19/21.
//

#include "tech.h"
#include <nlohmann/json.hpp>

namespace rip {
    struct JSONTech: public Asset {
        std::string name;
        int cost = 0;
        std::vector<std::string> prerequisites;
        std::vector<std::string> unlocksImprovements;

        friend void from_json(const nlohmann::json &nlohmann_json_j, JSONTech &nlohmann_json_t) {
            nlohmann_json_j.at("name").get_to(nlohmann_json_t.name);
            nlohmann_json_j.at("cost").get_to(nlohmann_json_t.cost);
            if (nlohmann_json_j.contains("prerequisites")) {
                nlohmann_json_j.at("prerequisites").get_to(nlohmann_json_t.prerequisites);
            }
            if (nlohmann_json_j.contains("unlocksImprovements")) {
                nlohmann_json_j.at("unlocksImprovements").get_to(nlohmann_json_t.unlocksImprovements);
            }
        }
    };

    int Tech::estimateResearchTurns(int beakersPerTurn) const {
        if (beakersPerTurn == 0) return cost + 1;
        return (cost + beakersPerTurn - 1) / beakersPerTurn;
    }

    Tech::Tech(const std::string &name, int cost, const std::vector<std::string> &unlocksImprovements) : name(name), cost(cost),
                                                                          unlocksImprovements(unlocksImprovements) {}

    TechTree::TechTree(const Assets &assets, const Registry &registry) {
        // Step 1: add all techs.
        auto jsonTechs = assets.getAll<JSONTech>();
        for (const auto &jsonTech : jsonTechs) {
            techs.emplace(jsonTech->name, std::make_shared<Tech>(jsonTech->name, jsonTech->cost, jsonTech->unlocksImprovements));
        }

        // Step 2: resolve dependencies (leadsTo and prerequisites)
        for (int i = 0; i < jsonTechs.size(); i++) {
            const auto &jsonTech = jsonTechs.at(i);
            auto &tech = techs[jsonTech->name];

            for (const auto &prerequisiteName : jsonTech->prerequisites) {
                auto &prerequisite = techs[prerequisiteName];
                tech->prerequisites.push_back(prerequisite);
                prerequisite->leadsTo.push_back(tech);
            }
        }

        // Step 3: resolve unit kinds
        for (const auto &unit : registry.getUnits()) {
            for (const auto &techName : unit->techs) {
                auto &tech = techs[techName];
                tech->unlocksUnits.push_back(unit);
            }
        }
    }

    Tech &TechTree::getTech(const std::string &name) {
        return *techs[name];
    }

    const absl::flat_hash_map<std::string, std::shared_ptr<Tech>> &TechTree::getTechs() const {
        return techs;
    }

    PlayerTechs::PlayerTechs(std::shared_ptr<TechTree> techTree) : techTree(std::move(techTree)) {}

    const std::vector<std::shared_ptr<Tech>> &PlayerTechs::getUnlockedTechs() const {
        return unlockedTechs;
    }

    std::vector<std::shared_ptr<Tech>> PlayerTechs::getPossibleResearches() const {
        std::vector<std::shared_ptr<Tech>> results;
        for (const auto &entry : techTree->getTechs()) {
            const auto &tech = entry.second;
            if (!isTechUnlocked(tech->name) && arePrerequisitesMet(*tech)) {
                results.push_back(tech);
            }
        }
        return results;
    }

    bool PlayerTechs::arePrerequisitesMet(const Tech &tech) const {
        for (const auto &prereq : tech.prerequisites) {
            if (!isTechUnlocked(prereq->name)) {
                return false;
            }
        }
        return true;
    }

    void PlayerTechs::unlockTech(std::shared_ptr<Tech> tech) {
        unlockedTechNames.insert(tech->name);
        unlockedTechs.push_back(std::move(tech));
    }

    bool PlayerTechs::isTechUnlocked(const std::string &name) const {
        return unlockedTechNames.contains(name);
    }

    bool PlayerTechs::isUnitUnlocked(const UnitKind &kind) const {
        for (const auto &techName : kind.techs) {
            if (!isTechUnlocked(techName)) return false;
        }
        return true;
    }

    bool PlayerTechs::isImprovementUnlocked(const std::string &name) const {
        for (const auto &tech : unlockedTechs) {
            const auto &unlocks = tech->unlocksImprovements;
            if (std::find(unlocks.begin(), unlocks.end(), name) != unlocks.end()) {
                return true;
            }
        }
        return false;
    }

    std::shared_ptr<Asset> TechLoader::loadAsset(const std::string &data) {
        auto tech = nlohmann::json::parse(data).get<JSONTech>();
        return std::make_unique<JSONTech>(std::move(tech));
    }
}
