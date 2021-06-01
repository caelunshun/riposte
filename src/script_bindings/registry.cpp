//
// Created by Caelum van Ispelen on 6/1/21.
//

#include <sol/sol.hpp>
#include "../registry.h"

namespace rip {
    void bindRegistry(sol::state &lua) {
        auto leader_type = lua.new_usertype<Leader>("Leader");
        leader_type["name"] = &Leader::name;
        leader_type["aggressive"] = &Leader::aggressive;
        leader_type["nukemonger"] = &Leader::nukemonger;
        leader_type["submissive"] = &Leader::submissive;
        leader_type["paranoia"] = &Leader::paranoia;
        leader_type["religious"] = &Leader::religious;

        auto civ_type = lua.new_usertype<CivKind>("CivKind");
        civ_type["id"] = &CivKind::id;
        civ_type["name"] = &CivKind::name;
        civ_type["adjective"] = &CivKind::adjective;
        civ_type["color"] = &CivKind::color;
        civ_type["leaders"] = &CivKind::leaders;
        civ_type["cities"] = &CivKind::cities;
        civ_type["startingTechs"] = &CivKind::startingTechs;

        auto combat_type = lua.new_usertype<CombatBonus>("CombatBonus");
        combat_type["whenInCityBonus"] = &CombatBonus::whenInCityBonus;
        combat_type["againstUnitCategoryBonus"] = &CombatBonus::againstUnitCategoryBonus;
        combat_type["againstUnitBonus"] = &CombatBonus::againstUnitBonus;
        combat_type["onlyOnAttack"] = &CombatBonus::onlyOnAttack;
        combat_type["onlyOnDefense"] = &CombatBonus::onlyOnDefense;
        combat_type["unit"] = &CombatBonus::unit;
        combat_type["unitCategory"] = &CombatBonus::unitCategory;

        auto unit_kind_type = lua.new_usertype<UnitKind>("UnitKind");
        unit_kind_type["id"] = &UnitKind::id;
        unit_kind_type["name"] = &UnitKind::name;
        unit_kind_type["strength"] = &UnitKind::strength;
        unit_kind_type["movement"] = &UnitKind::movement;
        unit_kind_type["capabilities"] = &UnitKind::capabilities;
        unit_kind_type["cost"] = &UnitKind::cost;
        unit_kind_type["techs"] = &UnitKind::techs;
        unit_kind_type["resources"] = &UnitKind::resources;
        unit_kind_type["combatBonuses"] = &UnitKind::combatBonuses;
        unit_kind_type["category"] = &UnitKind::category;

        auto building_type = lua.new_usertype<Building>("Building");
        building_type["name"] = &Building::name;
        building_type["cost"] = &Building::cost;
        building_type["prerequisites"] = &Building::prerequisites;
        building_type["techs"] = &Building::techs;
        building_type["onlyCoastal"] = &Building::onlyCoastal;
    }
}
