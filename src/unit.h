//
// Created by Caelum van Ispelen on 5/13/21.
//

#ifndef RIPOSTE_UNIT_H
#define RIPOSTE_UNIT_H

#include <glm/vec2.hpp>
#include "registry.h"
#include "ids.h"
#include "path.h"

struct nk_context;

class UpdateUnit;

namespace rip {
    class Game;
    class Hud;
    class City;
    class IdConverter;

    // Value returned by Capability::paintMainUI which determines whether the unit
    // should remain selected.
    enum UnitUIStatus {
        // Keep the unit selected.
        None,
        // Deselect the unit.
        Deselect,
    };

    // An instantiated capability attached to a unit.
    //
    // This object is created when a unit is created whose kind has the needed
    // capability.
    class Capability {
    protected:
        UnitId unitID;

        explicit Capability(UnitId unitID) : unitID(unitID) {}
    public:
        virtual void onTurnEnd(Game &game) {}

        virtual void onUnitMoved(Game &game, glm::uvec2 oldPos) {}

        virtual UnitUIStatus paintMainUI(Game &game, Hud &hud, nk_context *nk) {
            return UnitUIStatus::None;
        }
    };

    // Capability attached to settlers.
    class FoundCityCapability : public Capability {
    public:
        explicit FoundCityCapability(UnitId unitID);

        UnitUIStatus paintMainUI(Game &game, Hud &hud, nk_context *nk) override;

        // Founds a city. If successful, the unit is killed.
        // Don't use it anymore.
        bool foundCity(Game &game);
    };

    // Capability attached to siege weapons.
    class BombardCityCapability : public Capability {
    public:
        explicit BombardCityCapability(UnitId unitID);

        void bombardCity(Game &game, City &city);
    };

    /**
     * A unit on the map.
     */
    class Unit {
        // What kind the unit is - determines name, strength, movement, etc
        std::shared_ptr<UnitKind> kind;
        // The position of the unit on the map
        glm::uvec2 pos;
        // The unit's ID in the Game class
        UnitId id;
        // The player that owns the unit
        PlayerId owner;
        // The unit's current health, between 0 and 1 inclusive.
        // The actual combat strength is the unit's strength times its health.
        double health;
        // How many tiles the unit has left to move on this turn.
        // Resets to kind.movement at the start of every turn.
        float movementLeft;
        // The path the unit is currently following.
        std::optional<Path> currentPath;
        bool inCombat = false;

        bool usedAttack = false;

        // Capabilities attached to the unit.
        std::vector<std::unique_ptr<Capability>> capabilities;

        void resetMovement();

    public:
        bool fortified = false;
        bool skippingTurn = false;
        bool fortifiedUntilHeal = false;

        // Used by the renderer to animate the unit's position between two tiles.
        float moveTime = -1;
        glm::uvec2 moveFrom = glm::uvec2(0);

        Unit(std::shared_ptr<UnitKind> kind, glm::uvec2 pos, PlayerId owner);

        Unit(const UpdateUnit &packet, const IdConverter &playerIDs, const IdConverter &unitIDs, const Registry &registry, UnitId id);

        void setID(UnitId id);


        const UnitKind &getKind() const;
        glm::uvec2 getPos() const;
        UnitId getID() const;
        PlayerId getOwner() const;
        double getCombatStrength() const;
        float getMovementLeft() const;
        double getHealth() const;
        void setHealth(double health);
        std::vector<std::unique_ptr<Capability>> &getCapabilities();
        StackId getStack(const Game &game) const;

        bool canFight() const;
        bool shouldDie() const;

        void setMovementLeft(float movement);

        // Determines whether the unit can move to the given target
        // this turn.
        bool canMove(glm::uvec2 target, const Game &game) const;
        // Attempts to move the unit to a target position.
        // Does nothing if canMove(target) is false.
        void moveTo(glm::uvec2 target, Game &game, bool allowCombat);

        bool hasPath() const;
        const Path &getPath() const;
        void setPath(Path path);
        void moveAlongCurrentPath(Game &game, bool allowCombat);

        bool isInCombat() const;
        void setInCombat(bool inCombat);

        void onTurnEnd(Game &game);

        void fortify();
        bool isFortified() const;
        void fortifyUntilHealed();
        void skipTurn();

        void teleportTo(glm::uvec2 newPos, Game &game);

        double getModifiedDefendingStrength(const Unit &attacker, const Game &game) const;
        double getModifiedAttackingStrength(const Game &game) const;

        bool hasUsedAttack() const;
        void useAttack();

        template<class T>
        T *getCapability() {
            for (auto &cap : capabilities) {
                T *downcasted = dynamic_cast<T*>(&*cap);
                if (downcasted) {
                    return downcasted;
                }
            }
            return nullptr;
        }

        template<class T>
        bool hasCapability() {
            return getCapability<T>() != nullptr;
        }
    };
}

#endif //RIPOSTE_UNIT_H
