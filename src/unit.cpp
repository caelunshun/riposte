//
// Created by Caelum van Ispelen on 5/13/21.
//

#include "unit.h"
#include "game.h"
#include "ripmath.h"
#include "tile.h"
#include "worker.h"
#include "combat.h"
#include "stack.h"
#include "city.h"
#include "ship.h"
#include <nuklear.h>
#include <iostream>
#include "server.h"

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

     UnitUIStatus FoundCityCapability::paintMainUI(Game &game, Hud &hud, nk_context *nk) {
         nk_layout_row_push(nk, 100);
         if (nk_button_label(nk, "Found City")) {
             foundCity(game);
             return UnitUIStatus::Deselect;
         }
         return UnitUIStatus::None;
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
            } else if (capabilityName == "do_work") {
                capabilities.push_back(std::make_unique<WorkerCapability>(id));
            } else if (capabilityName == "carry_units") {
                capabilities.push_back(std::make_unique<CarryUnitsCapability>(id, kind->carryUnitCapacity));
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

    float Unit::getMovementLeft() const {
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

        const auto terrain = game.getTile(target).getTerrain();
        if (!kind->ship && terrain == Terrain::Ocean ) {
            return false;
        }
        if (kind->ship && terrain != Terrain::Ocean && !game.getCityAtLocation(target)) {
            return false;
        }

        if (!canFight() && wouldAttackPos(game, target)) {
            return false;
        }

        return true;
    }

    bool Unit::wouldAttack(const Game &game, const Unit &other) const {
        return
            other.getID() != id
            && !other.shouldDie()
            && owner != other.getOwner()
            && game.getPlayer(owner).isAtWarWith(other.getOwner());
    }

    std::optional<UnitId> Unit::wouldAttackPos(const Game &game, glm::uvec2 target) const {
        for (const auto stackID : game.getStacksAtPos(target)) {
            const auto &stack = game.getStack(stackID);
            for (const auto otherUnitID : stack.getUnits()) {
                auto &otherUnit = game.getUnit(otherUnitID);
                if (wouldAttack(game, otherUnit)) {
                    return otherUnitID;
                }
            }
        }
        return {};
    }

    void Unit::moveTo(glm::uvec2 target, Game &game, bool allowCombat) {
        if (!canMove(target, game)) return;

        auto oldPos = pos;

        // Check for attacks.
        auto otherUnit = wouldAttackPos(game, target);
        if (otherUnit.has_value()) {
            if (!allowCombat && game.getUnit(*otherUnit).canFight()) return;
            Combat combat(getID(), *otherUnit, game);
            game.addCombat(combat);
            game.getUnit(*otherUnit).setInCombat(true);
            this->setInCombat(true);
            std::cout << "attack" << std::endl;
            return;
        }

        // Check for city captures.
        auto *city = game.getCityAtLocation(target);
        if (city) {
            if (canFight()) {
                if (game.getPlayer(city->getOwner()).isAtWarWith(getOwner())) {
                    city->transferControlTo(game, getOwner());
                }
            } else {
                return;
            }
        }

        moveTime = 0;
        moveFrom = pos;

        movementLeft -= game.getTile(target).getMovementCost();
        if (movementLeft <= 0.1) movementLeft = 0;

        teleportTo(target, game);
    }

    bool Unit::hasPath() const {
        return currentPath.has_value();
    }

    void Unit::setPath(Path path) {
        currentPath = std::move(path);
    }

    void Unit::moveAlongCurrentPath(Game &game, bool allowCombat) {
        if (currentPath.has_value()) {
            while (currentPath->getNumPoints() > 0 && movementLeft != 0) {
                auto point = currentPath->popNextPoint();
                if (!allowCombat && wouldAttackPos(game, *point)) {
                    currentPath = {};
                    return;
                }
                moveTo(*point, game, allowCombat);
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

    void Unit::onTurnEnd(Game &game) {
        if (movementLeft > 0) {
            const auto regen = 0.2;
            health = std::clamp(health + regen, 0.0, 1.0);
        }

        resetMovement();

        for (auto &capability : getCapabilities()) {
            capability->onTurnEnd(game);
        }

        skippingTurn = false;
        if (health == 1.0) {
            fortifiedUntilHeal = false;
        }
    }

    std::vector<std::unique_ptr<Capability>> &Unit::getCapabilities() {
        return capabilities;
    }

    void Unit::setMovementLeft(int movement) {
        movementLeft = movement;
    }

    double Unit::getHealth() const {
        return health;
    }

    void Unit::setHealth(double health) {
        this->health = health;
        if (health < 0) {
            this-> health = 0;
        }
    }

    bool Unit::canFight() const {
        return kind->strength > 0;
    }

    bool Unit::shouldDie() const {
        return health < 0.1;
    }

    bool Unit::isInCombat() const {
        return inCombat;
    }

    void Unit::setInCombat(bool inCombat) {
        this->inCombat = inCombat;
    }

    StackId Unit::getStack(const Game &game) const {
        return *game.getStackByKey(owner, pos);
    }

    void Unit::fortify() {
        fortified = true;
    }

    bool Unit::isFortified() const {
        return fortified || fortifiedUntilHeal || skippingTurn;
    }

    void Unit::fortifyUntilHealed() {
        fortifiedUntilHeal = true;
    }

    void Unit::skipTurn() {
        skippingTurn = true;
    }

    void Unit::teleportTo(glm::uvec2 target, Game &game) {
         const auto oldPos = pos;
        pos = target;

        // Unit has moved; update visibility
        game.getPlayer(owner).recomputeVisibility(game);

        fortified = false;
        skippingTurn = false;
        fortifiedUntilHeal = false;

        game.onUnitMoved(id, oldPos, target);

        // Update capabilities
        for (auto &capability : getCapabilities()) {
            capability->onUnitMoved(game, oldPos);
        }

        inCombat = false;

        game.getServer().broadcastUnitUpdate(*this);
    }
}
