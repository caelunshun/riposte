//
// Created by Caelum van Ispelen on 5/19/21.
//

#include "tech.h"

namespace rip {
    int Tech::estimateResearchTurns(int beakersPerTurn) const {
        return (cost + beakersPerTurn - 1) / beakersPerTurn;
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
}
