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

namespace rip {
    class Game;

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

        virtual void onUnitMoved(Game &game) {}

        virtual UnitUIStatus paintMainUI(Game &game, nk_context *nk) {
            return UnitUIStatus::None;
        }
    };

    // Capability attached to settlers.
    class FoundCityCapability : public Capability {
    public:
        explicit FoundCityCapability(UnitId unitID);

        UnitUIStatus paintMainUI(Game &game, nk_context *nk) override;

        // Founds a city. If successful, the unit is killed.
        // Don't use it anymore.
        bool foundCity(Game &game);
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

        // Capabilities attached to the unit.
        std::vector<std::unique_ptr<Capability>> capabilities;

        void resetMovement();

    public:
        // Used by the renderer to animate the unit's position between two tiles.
        float moveTime = -1;
        glm::uvec2 moveFrom = glm::uvec2(0);

        Unit(std::shared_ptr<UnitKind> kind, glm::uvec2 pos, PlayerId owner);

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

        bool canFight() const;
        bool shouldDie() const;

        void setMovementLeft(int movement);

        // Determines whether the unit can move to the given target
        // this turn.
        bool canMove(glm::uvec2 target, const Game &game) const;
        // Attempts to move the unit to a target position.
        // Does nothing if canMove(target) is false.
        void moveTo(glm::uvec2 target, Game &game);

        bool hasPath() const;
        const Path &getPath() const;
        void setPath(Path path);
        void moveAlongCurrentPath(Game &game);

        void onTurnEnd(Game &game);

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
