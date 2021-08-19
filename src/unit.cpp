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
#include "protocol.h"
#include "saveload.h"

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

    BombardCityCapability::BombardCityCapability(UnitId unitID) : Capability(unitID) {

    }

    void BombardCityCapability::bombardCity(Game &game, City &city) {
         auto &unit = game.getUnit(unitID);
         if (unit.getMovementLeft() == 0) return;
         unit.setMovementLeft(0);

         if (game.getPlayer(city.getOwner()).isAtWarWith(unit.getOwner())) {
             city.bombardCultureDefenses(game, unit.getKind().maxBombardPerTurn);
             game.getServer().markUnitDirty(unitID);
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

    Unit::Unit(const UpdateUnit &packet, const IdConverter &playerIDs, const IdConverter &unitIDs, const Registry &registry, UnitId id) {
         id = id;
        pos = glm::uvec2(packet.pos().x(), packet.pos().y());

        kind = registry.getUnit(packet.kindid());
        owner = playerIDs.get(packet.ownerid());

        health = packet.health();
        movementLeft = packet.movementleft();
        fortified = packet.fortifiedforever();
        skippingTurn = packet.skippingturn();
        fortifiedUntilHeal = packet.fortifieduntilheal();
        usedAttack = packet.usedattack();

        for (const auto &protoCap : packet.capabilities()) {
            if (protoCap.has_worker()) {
                auto workerCap = std::make_unique<WorkerCapability>(id);
                if (protoCap.worker().has_currenttask()) {
                    const auto &task = protoCap.worker().currenttask();
                    if (task.kind().has_buildimprovement()) {
                        std::unique_ptr<Improvement> improvement;
                        const auto &id = task.kind().buildimprovement().improvementid();
                        if (id == "Cottage") {
                            improvement = std::make_unique<Cottage>(pos);
                        } else if (id == "Mine") {
                            improvement = std::make_unique<Mine>(pos);
                        } else if (id == "Pasture") {
                            improvement = std::make_unique<Pasture>(pos);
                        } else if (id == "Road") {
                            improvement = std::make_unique<Road>(pos);
                        } else if (id == "Farm") {
                            improvement = std::make_unique<Farm>(pos);
                        }
                        workerCap->setTask(std::make_unique<BuildImprovementTask>(
                                task.turnsleft(),
                                pos,
                                std::move(improvement)
                                ));
                    }
                }
                capabilities.emplace_back(std::move(workerCap));
            } else if (protoCap.has_foundcity()) {
                capabilities.push_back(std::make_unique<FoundCityCapability>(id));
            } else if (protoCap.has_bombardcity()) {
                capabilities.push_back(std::make_unique<BombardCityCapability>(id));
            } else if (protoCap.has_carryunits()) {
                auto cap = std::make_unique<CarryUnitsCapability>(id, kind->carryUnitCapacity);
                for (const auto unitID : protoCap.carryunits().carryingunitids()) {
                    cap->addCarryingUnit(unitIDs.get(unitID));
                }
                capabilities.emplace_back(std::move(cap));
            }
        }
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
            } else if (capabilityName == "bombard_city_defenses") {
                capabilities.push_back(std::make_unique<BombardCityCapability>(id));
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

        const auto strongestDefender = game.getStrongestDefender(*this, target);

        if (!canFight() && strongestDefender.has_value()) {
            return false;
        }

        if (hasUsedAttack() && strongestDefender.has_value()
            && game.getUnit(*strongestDefender).canFight()) {
            return false;
        }

        return true;
    }

    void Unit::moveTo(glm::uvec2 target, Game &game, bool allowCombat) {
        if (!canMove(target, game)) return;

        auto oldPos = pos;

        // Check for attacks.
        auto otherUnit = game.getStrongestDefender(*this, target);
        if (otherUnit.has_value()) {
            if (!allowCombat && game.getUnit(*otherUnit).canFight()) return;
            Combat combat(getID(), *otherUnit, game);
            combat.finish(game);
            if (game.getUnit(*otherUnit).canFight()) {
                useAttack();
            }
            return;
        }

        // Check for city captures.
        auto *city = game.getCityAtLocation(target);
        if (city && city->getOwner() != owner) {
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
                if (!allowCombat && game.getStrongestDefender(*this, *point).has_value()) {
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

        game.getServer().markUnitDirty(id);
    }

    std::vector<std::unique_ptr<Capability>> &Unit::getCapabilities() {
        return capabilities;
    }

    void Unit::setMovementLeft(float movement) {
        movementLeft = movement;
        if (movementLeft < 0) movementLeft = 0;
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
         if (!game.containsTile(target)) return;

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

        game.getServer().markUnitDirty(id);
    }

    double Unit::getModifiedDefendingStrength(const Unit &attacker, const Game &game) const {
         int percentBonus = 0;

         // Tile defense bonus
         const auto &tile = game.getTile(pos);
         percentBonus += tile.getDefensiveBonus();

         // City defense bonuses
         const auto *city = game.getCityAtLocation(pos);
         if (city) {
             percentBonus += city->getCultureDefenseBonus();
             percentBonus += city->getBuildingEffects().defenseBonusPercent;
         }

         // Subtract opponent bonuses
         for (const auto &bonus : attacker.getKind().combatBonuses) {
             if (bonus.onlyOnDefense) continue;
             if (bonus.unit == kind->id) {
                 percentBonus -= bonus.againstUnitBonus;
             }
             if (bonus.unitCategory == kind->category) {
                 percentBonus -= bonus.againstUnitCategoryBonus;
             }
             if (game.getCityAtLocation(attacker.getPos())) {
                 percentBonus -= bonus.whenInCityBonus;
             }
         }

         // Add our bonuses
         for (const auto &bonus : kind->combatBonuses) {
             if (bonus.onlyOnAttack) continue;
             if (bonus.unit == attacker.getKind().id) {
                 percentBonus += bonus.againstUnitBonus;
             }
             if (bonus.unitCategory == attacker.getKind().category) {
                 percentBonus += bonus.againstUnitCategoryBonus;
             }
             percentBonus += bonus.whenInCityBonus;
         }

         double result = health * kind->strength;

         if (percentBonus >= 0) {
             result *= 1 + (static_cast<double>(percentBonus) / 100);
         } else {
             result /= 1 + (abs(static_cast<double>(percentBonus)) / 100);
         }

         return result;
    }

    double Unit::getModifiedAttackingStrength(const Game &game) const {
         return health * kind->strength;
    }

    bool Unit::hasUsedAttack() const {
        return usedAttack;
    }

    void Unit::useAttack() {
        usedAttack = true;
    }
}
