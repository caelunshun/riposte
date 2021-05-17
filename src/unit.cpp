//
// Created by Caelum van Ispelen on 5/13/21.
//

#include "unit.h"
#include "game.h"
#include "ripmath.h"
#include <nuklear.h>

namespace rip {
     FoundCityCapability::FoundCityCapability(UnitId unitID) : Capability(unitID) {}

     bool FoundCityCapability::foundCity(Game &game) {
         const auto &unit = game.getUnit(unitID);
         auto &player = game.getPlayer(unit.getOwner());
         if (game.getCityAtLocation(unit.getPos())) {
             return false;
         } else {
             player.createCity(unit.getPos(), game);
             game.deferKillUnit(unitID);
             return true;
         }
     }

     void FoundCityCapability::paintMainUI(Game &game, nk_context *nk) {
         nk_layout_row_push(nk, 100);
         if (nk_button_label(nk, "Found City")) {
             foundCity(game);
         }
     }

    void Unit::resetMovement() {
        movementLeft = kind->movement;
    }

    Unit::Unit(std::shared_ptr<UnitKind> kind, glm::uvec2 pos, PlayerId owner) : kind(std::move(kind)), pos(pos),
                                                                                 owner(owner) {
        health = 1;
        movementLeft = this->kind->movement;
    }

    void Unit::setID(UnitId id) {
        this->id = id;

        for (const auto &capabilityName : kind->capabilities) {
            if (capabilityName == "found_city") {
                capabilities.push_back(std::make_unique<FoundCityCapability>(id));
            } else {
                throw std::string("missing capability: " + capabilityName);
            }
        }
    }

    const UnitKind &Unit::getKind() const {
        return *kind;
    }

    glm::uvec2 Unit::getPos() const {
        return pos;
    }

    UnitId Unit::getID() const {
        return id;
    }

    PlayerId Unit::getOwner() const {
        return owner;
    }

    double Unit::getCombatStrength() const {
        return health * kind->strength;
    }

    int Unit::getMovementLeft() const {
        return movementLeft;
    }

    bool Unit::canMove(glm::uvec2 target, const Game &game) const {
        if (target == pos) {
            return false;
        }

        if (!game.containsTile(target)) {
            return false;
        }

        if (movementLeft == 0) {
            return false;
        }

        if (game.getTile(target).getTerrain() == Terrain::Ocean) {
            return false;
        }

        return true;
    }

    void Unit::moveTo(glm::uvec2 target, Game &game) {
        if (!canMove(target, game)) return;

        moveTime = 0;
        moveFrom = pos;

        movementLeft -= game.getTile(target).getMovementCost();
        if (movementLeft < 0) movementLeft = 0;
        pos = target;

        // Unit has moved; update visibility
        game.getPlayer(owner).recomputeVisibility(game);
    }

    bool Unit::hasPath() const {
        return currentPath.has_value();
    }

    void Unit::setPath(Path path) {
        currentPath = std::move(path);
    }

    void Unit::moveAlongCurrentPath(Game &game) {
        if (currentPath.has_value()) {
            while (currentPath->getNumPoints() > 0 && movementLeft != 0) {
                auto point = currentPath->popNextPoint();
                moveTo(*point, game);
            }

            if (currentPath->getNumPoints() == 0) {
                // Path is over.
                currentPath = std::optional<Path>();
            }
        }
    }

    const Path &Unit::getPath() const {
        return *currentPath;
    }

    void Unit::onTurnEnd() {
        resetMovement();
        const auto regen = 0.2;
        health = std::clamp(health + regen, 0.0, 1.0);
    }

    std::vector<std::unique_ptr<Capability>> &Unit::getCapabilities() {
        return capabilities;
    }
}
