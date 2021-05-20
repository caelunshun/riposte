//
// Created by Caelum van Ispelen on 5/19/21.
//

#ifndef RIPOSTE_TECH_H
#define RIPOSTE_TECH_H

#include <string>
#include <vector>
#include <memory>
#include <absl/container/flat_hash_map.h>
#include <absl/container/flat_hash_set.h>
#include "registry.h"

namespace rip {
    struct JSONTech;

    struct Tech {
        // Tech name to be displayed in the UI.
        std::string name;
        // Cost in beakers.
        int cost;

        // The set of units unlocked by researching this tech.
        // Note that units may depend on multiple techs.
        std::vector<std::shared_ptr<UnitKind>> unlocksUnits;
        // The set of improvements unlocked by researching this tech.
        std::vector<std::string> unlocksImprovements;

        // The set of techs that need to be unlocked to research this one.
        std::vector<std::shared_ptr<Tech>> prerequisites;
        std::vector<std::shared_ptr<Tech>> leadsTo;

        Tech(const std::string &name, int cost, const std::vector<std::string> &unlocksImprovements);

        int estimateResearchTurns(int beakersPerTurn) const;
    };

    // Stores the entire tech tree.
    class TechTree {
        absl::flat_hash_map<std::string, std::shared_ptr<Tech>> techs;

    public:
        TechTree(const Assets &assets, const Registry &registry);

        Tech &getTech(const std::string &name);

        const absl::flat_hash_map<std::string, std::shared_ptr<Tech>> &getTechs() const;
    };

    // Stores the techs unlocked by a player.
    class PlayerTechs {
        std::shared_ptr<TechTree> techTree;
        std::vector<std::shared_ptr<Tech>> unlockedTechs;
        absl::flat_hash_set<std::string> unlockedTechNames;

    public:
        explicit PlayerTechs(std::shared_ptr<TechTree> techTree);

        const std::vector<std::shared_ptr<Tech>> &getUnlockedTechs() const;

        std::vector<std::shared_ptr<Tech>> getPossibleResearches() const;

        void unlockTech(std::shared_ptr<Tech> tech);

        bool arePrerequisitesMet(const Tech &tech) const;

        bool isTechUnlocked(const std::string &name) const;

        bool isUnitUnlocked(const UnitKind &kind) const;

        bool isImprovementUnlocked(const std::string &name) const;
    };

    class TechLoader : public AssetLoader {
    public:
        std::shared_ptr<Asset> loadAsset(const std::string &data) override;
    };
}

#endif //RIPOSTE_TECH_H
