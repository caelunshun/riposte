//
// Created by Caelum van Ispelen on 6/1/21.
//

#include <sol/sol.hpp>
#include "../unit.h"
#include "../game.h"

namespace rip {
    void bindUnit(sol::state &lua, std::shared_ptr<Game*> game) {
        auto unit_type = lua.new_usertype<Unit>("Unit");
        unit_type["getKind"] = &Unit::getKind;
        unit_type["getPos"] = &Unit::getPos;
        unit_type["getOwner"] = [=] (Unit &self) {
            return &(*game)->getPlayer(self.getOwner());
        };
        unit_type["getCombatStrength"] = &Unit::getCombatStrength;
        unit_type["getMovementLeft"] = &Unit::getMovementLeft;
        unit_type["getHealth"] = &Unit::getHealth;
        unit_type["setHealth"] = &Unit::setHealth;
        unit_type["canFight"] = &Unit::canFight;
        unit_type["shouldDie"] = &Unit::shouldDie;
        unit_type["setMovementLeft"] = &Unit::setMovementLeft;
        unit_type["canMove"] = [=] (Unit &self, glm::uvec2 target) {
            return self.canMove(target, **game);
        };
        unit_type["moveTo"] = [=] (Unit &self, glm::uvec2 target, bool allowCombat) {
            self.moveTo(target, **game, allowCombat);
        };
        unit_type["wouldAttack"] = [=] (Unit &self, Unit &other) {
            return self.wouldAttack(**game, other);
        };
        unit_type["hasPath"] = &Unit::hasPath;
        unit_type["setPath"] = &Unit::setPath;
        unit_type["moveAlongCurrentPath"] = &Unit::moveAlongCurrentPath;
        unit_type["isInCombat"] = &Unit::isInCombat;
        unit_type["fortify"] = &Unit::fortify;
        unit_type["isFortified"] = &Unit::isFortified;
        unit_type["fortifyUntilHealed"] = &Unit::fortifyUntilHealed;
        unit_type["skipTurn"] = &Unit::skipTurn;
        unit_type["teleportTo"] = [=] (Unit &self, glm::uvec2 pos) {
            self.teleportTo(pos, **game);
        };

    }
}
