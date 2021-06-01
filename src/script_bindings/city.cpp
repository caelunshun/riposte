//
// Created by Caelum van Ispelen on 6/1/21.
//

#include <sol/sol.hpp>
#include "../city.h"
#include "../game.h"

namespace rip {
    void bindCity(sol::state &lua, std::shared_ptr<Game*> game) {
        auto build_type = lua.new_usertype<BuildTask>("BuildTask");
        build_type["getCost"] = &BuildTask::getCost;
        build_type["getProgress"] = &BuildTask::getProgress;
        build_type["isFinished"] = &BuildTask::isFinished;
        build_type["getOverflow"] = &BuildTask::getOverflow;
        build_type["spendHammers"] = &BuildTask::spendHammers;
        build_type["getName"] = &BuildTask::getName;

        auto yield_type = lua.new_usertype<Yield>("Yield");
        yield_type["hammers"] = &Yield::hammers;
        yield_type["commerce"] = &Yield::commerce;
        yield_type["food"] = &Yield::food;

        auto city_type = lua.new_usertype<City>("City");
        city_type["getPos"] = &City::getPos;
        city_type["getName"] = &City::getName;
        city_type["getOwner"] = [=] (City &self) {
            return &(*game)->getPlayer(self.getOwner());
        };
        city_type["isCapital"] = &City::isCapital;
        city_type["getCulture"] = [=] (City &self) {
            return self.getCulture().getCultureForPlayer(self.getOwner());
        };
        city_type["getCulturePerTurn"] = &City::getCulturePerTurn;
        city_type["getCultureLevel"] = [=] (City &self) {
            return self.getCultureLevel().value;
        };
        city_type["setName"] = &City::setName;
        city_type["getPopulation"] = &City::getPopulation;
        city_type["isCoastal"] = &City::isCoastal;
        city_type["hasBuildTask"] = &City::hasBuildTask;
        city_type["getBuildTask"] = &City::getBuildTask;
        city_type["estimateTurnsForCompletion"] = [=] (City &self, const BuildTask &task) {
            return self.estimateTurnsForCompletion(task, **game);
        };
        city_type["getBuildings"] = &City::getBuildings;
        city_type["hasBuilding"] = &City::hasBuilding;
        city_type["computeYield"] = [=] (City &self) {
            return self.computeYield(**game);
        };
        city_type["getWorkedTiles"] = &City::getWorkedTiles;
        city_type["updateWorkedTiles"] = [=] (City &self) {
            self.updateWorkedTiles(**game);
        };
        city_type["addManualWorkedTile"] = &City::addManualWorkedTile;
        city_type["removeManualWorkedTile"] = &City::removeManualWorkedTile;
        city_type["getManualWorkedTiles"] = &City::getManualWorkedTiles;
        city_type["canWorkTile"] = [=] (City &self, glm::uvec2 pos) {
            return self.canWorkTile(pos, **game);
        };
        city_type["getStoredFood"] = &City::getStoredFood;
        city_type["getFoodNeededForGrowth"] = &City::getFoodNeededForGrowth;
        city_type["getConsumedFood"] = &City::getConsumedFood;
    }
}
