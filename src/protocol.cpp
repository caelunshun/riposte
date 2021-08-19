//
// Created by Caelum van Ispelen on 8/17/21.
//

#include "protocol.h"
#include "player.h"
#include "unit.h"
#include "city.h"
#include "worker.h"
#include "culture.h"
#include "yield.h"
#include "tile.h"
#include "ship.h"

namespace rip {
    void setPlayerInfo(const Player &player, PlayerInfo &playerInfo) {
        playerInfo.set_username(player.getUsername());
        playerInfo.set_civid(player.getCiv().id);
        playerInfo.set_leadername(player.getLeader().name);
        playerInfo.set_score(player.getScore());
        playerInfo.set_id(player.getID().encode());
        playerInfo.set_isadmin(false); // TODO: permissions
    }

    UpdateGlobalData getUpdateGlobalDataPacket(Game &game, PlayerId thePlayerID) {
        UpdateGlobalData packet;
        packet.set_turn(game.getTurn());
        packet.set_era(static_cast<::Era>(static_cast<int>(game.getPlayer(thePlayerID).getEra())));

        for (const auto &player : game.getPlayers()) {
            auto *protoPlayer = packet.add_players();
            setPlayerInfo(player, *protoPlayer);
        }

        packet.set_playerid(thePlayerID.encode());

        return packet;
    }

    void writeYield(const Yield &yield, ::Yield &protoYield) {
        protoYield.set_commerce(yield.commerce);
        protoYield.set_food(yield.food);
        protoYield.set_hammers(yield.hammers);
    }

    void writeCultureValues(const Culture &culture, CultureValues &proto) {
        for (const auto &entry : culture.getValues()) {
            proto.add_amounts(entry.amount);
            proto.add_playerids(entry.owner.encode());
        }
    }

    void setTile(const Game &game, PlayerId player, glm::uvec2 pos, const Tile &tile, ::Tile &protoTile) {
        protoTile.set_terrain(static_cast<::Terrain>(static_cast<int>(tile.getTerrain())));
        protoTile.set_forested(tile.isForested());
        protoTile.set_hilled(tile.isHilled());

        const auto yield = tile.getYield(game, pos, player);
        writeYield(yield, *protoTile.mutable_yield());

        const auto owner = game.getCultureMap().getTileOwner(pos);
        if (owner.has_value()) {
            protoTile.set_ownerid(owner->encode());
        }
        protoTile.set_hasowner(owner.has_value());
        protoTile.set_isworked(game.isTileWorked(pos).has_value());

        for (const auto &improvement : tile.getImprovements()) {
            auto *protoImprovement = protoTile.add_improvements();
            protoImprovement->set_id(improvement->getName());

            auto *cottage = dynamic_cast<Cottage*>(&*improvement);
            if (cottage) {
                protoImprovement->set_cottagelevel(cottage->getLevelName());
            }
        }

        if (tile.hasResource()) {
            protoTile.set_resourceid((*tile.getResource())->id);
        }

        writeCultureValues(
                game.getCultureMap().getTileCulture(pos),
                *protoTile.mutable_culturevalues()
                );
    }

    UpdateMap getUpdateMapPacket(Game &game, PlayerId playerID) {
        UpdateMap packet;
        packet.set_width(game.getMapWidth());
        packet.set_height(game.getMapHeight());

        const auto &player = game.getPlayer(playerID);
        for (int y = 0; y < game.getMapHeight(); y++) {
            for (int x = 0; x < game.getMapWidth(); x++) {
                glm::uvec2 pos(x, y);
                const auto &tile = game.getTile(pos);
                auto protoTile = packet.add_tiles();
                setTile(game, playerID, pos, tile, *protoTile);
            }
        }

        return packet;
    }

    UpdateVisibility getUpdateVisibilityPacket(Game &game, PlayerId playerID) {
        UpdateVisibility packet;

        const auto &player = game.getPlayer(playerID);
        for (int y = 0; y < game.getMapHeight(); y++) {
            for (int x = 0; x < game.getMapWidth(); x++) {
                glm::uvec2 pos(x, y);
                packet.add_visibility(static_cast<::Visibility>(static_cast<int>(player.getVisibilityMap()[pos])));
            }
        }

        return packet;
    }

    UpdateTile getUpdateTilePacket(Game &game, glm::uvec2 pos, PlayerId player) {
        auto &tile = game.getTile(pos);

        UpdateTile packet;
        setTile(game, player, pos, tile, *packet.mutable_tile());

        packet.set_x(pos.x);
        packet.set_y(pos.y);

        return packet;
    }

    void writePath(const Path &path, ::Path &protoPath) {
        for (const auto pos : path.getPoints()) {
            protoPath.add_positions(pos.x);
            protoPath.add_positions(pos.y);
        }
    }

    void writeWorkerTask(const rip::WorkerTask &task, ::WorkerTask &protoTask) {
        protoTask.set_name(task.getName());
        protoTask.set_turnsleft(task.getRemainingTurns());
        protoTask.set_presentparticiple(task.getPresentParticiple());

        auto *buildImprovement = dynamic_cast<const BuildImprovementTask*>(&task);
        if (buildImprovement) {
            protoTask.mutable_kind()->mutable_buildimprovement()->set_improvementid(buildImprovement->getImprovement().getName());
        }
    }

    UpdateUnit getUpdateUnitPacket(Game &game, Unit &unit) {
        UpdateUnit packet;

        auto *protoPos = packet.mutable_pos();
        protoPos->set_x(unit.getPos().x);
        protoPos->set_y(unit.getPos().y);

        packet.set_kindid(unit.getKind().id);
        packet.set_ownerid(unit.getOwner().encode());
        packet.set_health(unit.getHealth());
        packet.set_movementleft(unit.getMovementLeft());
        packet.set_strength(unit.getCombatStrength());
        packet.set_isfortified(unit.isFortified());
        packet.set_usedattack(unit.hasUsedAttack());

        if (unit.hasPath()) {
            writePath(unit.getPath(), *packet.mutable_followingpath());
        }

        for (const auto &capability : unit.getCapabilities()) {
            auto &protoCap = *packet.add_capabilities();
            const auto *foundCity = dynamic_cast<const FoundCityCapability*>(&*capability);
            const auto *worker = dynamic_cast<const WorkerCapability*>(&*capability);
            const auto *carryUnits = dynamic_cast<const CarryUnitsCapability*>(&*capability);
            const auto *bombardCity = dynamic_cast<const BombardCityCapability*>(&*capability);
            if (foundCity) {
                protoCap.mutable_foundcity();
            } else if (worker) {
                auto *protoWorker = protoCap.mutable_worker();
                if (worker->getTask()) {
                    writeWorkerTask(*worker->getTask(), *protoWorker->mutable_currenttask());
                }
                for (const auto &possibleTask : worker->getPossibleTasks(game)) {
                    writeWorkerTask(*possibleTask, *protoWorker->add_possibletasks());
                }
            } else if (carryUnits) {
                auto *protoCarryUnits = protoCap.mutable_carryunits();
                for (const auto unitID : carryUnits->getCarryingUnits()) {
                    protoCarryUnits->add_carryingunitids(unitID.encode());
                }
            } else if (bombardCity) {
                protoCap.mutable_bombardcity();
            }
        }

        packet.set_id(unit.getID().encode());

        packet.set_fortifiedforever(unit.fortified);
        packet.set_skippingturn(unit.skippingTurn);
        packet.set_fortifieduntilheal(unit.fortifiedUntilHeal);

        return packet;
    }

    void writeBuildTask(const BuildTask &task, ::BuildTask &protoTask) {
        protoTask.set_progress(task.getProgress());
        protoTask.set_cost(task.getCost());

        auto *kind = protoTask.mutable_kind();

        const auto *building = dynamic_cast<const BuildingBuildTask*>(&task);
        const auto *unit = dynamic_cast<const UnitBuildTask*>(&task);

        if (building) {
            kind->mutable_building()->set_buildingname(building->getBuilding()->name);
        } else if (unit) {
            kind->mutable_unit()->set_unitkindid(unit->getUnitKind()->id);
        }
    }

    UpdateCity getUpdateCityPacket(Game &game, City &city) {
        UpdateCity packet;

        packet.mutable_pos()->set_x(city.getPos().x);
        packet.mutable_pos()->set_y(city.getPos().y);

        packet.set_name(city.getName());
        packet.set_ownerid(city.getOwner().encode());

        if (city.hasBuildTask()) {
            writeBuildTask(*city.getBuildTask(), *packet.mutable_buildtask());
        }

        writeYield(city.computeYield(game), *packet.mutable_yield());
        packet.set_culture(city.getCulture().getCultureForPlayer(city.getOwner()));
        // packet.set_cultureneeded(city.getCultureNeeded()); TODO
        packet.set_id(city.getID().encode());

        for (const auto &building : city.getBuildings()) {
            packet.add_buildingnames(building->name);
        }

        packet.set_population(city.getPopulation());
        packet.set_storedfood(city.getStoredFood());
        packet.set_foodneededforgrowth(city.getFoodNeededForGrowth());
        packet.set_consumedfood(city.getConsumedFood());
        packet.set_iscapital(city.isCapital());

        for (const auto workedPos : city.getWorkedTiles()) {
            auto *pos = packet.add_workedtiles();
            pos->set_x(workedPos.x);
            pos->set_y(workedPos.y);
        }

        for (const auto &entry : city.getHappinessSources()) {
            packet.add_happinesssources()->CopyFrom(entry);
        }
        for (const auto &entry : city.getUnhappinessSources()) {
            packet.add_unhappinesssources()->CopyFrom(entry);
        }
        for (const auto &entry : city.getHealthSources()) {
            packet.add_healthsources()->CopyFrom(entry);
        }
        for (const auto &entry : city.getSicknessSources()) {
            packet.add_sicknesssources()->CopyFrom(entry);
        }

        packet.set_culturedefensebonus(city.getCultureDefenseBonus());

        for (const auto &resource : city.getResources()) {
            packet.add_resources(resource->id);
        }

        writeCultureValues(city.getCulture(), *packet.mutable_culturevalues());

        for (const auto tilePos : city.getManualWorkedTiles()) {
            auto *p = packet.add_manualworkedtiles();
            p->set_x(tilePos.x);
            p->set_y(tilePos.y);
        }

        return packet;
    }

    UpdatePlayer getUpdatePlayerPacket(Game &game, Player &player) {
        UpdatePlayer packet;

        packet.set_id(player.getID().encode());
        packet.set_username(player.getUsername());

        packet.set_baserevenue(player.getBaseRevenue());
        packet.set_beakerrevenue(player.getBeakerRevenue());
        packet.set_goldrevenue(player.getGoldRevenue());
        packet.set_expenses(player.getExpenses());
        packet.set_netgold(player.getNetGold());
        packet.set_gold(player.getGold());
        packet.set_beakerpercent(player.getSciencePercent());

        if (player.getResearchingTech().has_value()) {
            auto *tech = packet.mutable_researchingtech();
            tech->set_techid(player.getResearchingTech()->tech->name);
            tech->set_progress(player.getResearchingTech()->beakersAccumulated);
        }

        for (const auto &tech : player.getTechs().getUnlockedTechs()) {
            packet.add_unlockedtechids(tech->name);
        }

        for (const auto &otherPlayer : game.getPlayers()) {
            if (player.isAtWarWith(otherPlayer.getID())) {
                packet.add_atwarwithids(otherPlayer.getID().encode());
            }
        }

        packet.set_era(static_cast<::Era>(static_cast<int>(player.getEra())));
        packet.set_hasai(player.hasAI());
        packet.mutable_visibility()->CopyFrom(getUpdateVisibilityPacket(game, player.getID()));

        for (const auto cityID : player.getCities()) {
            packet.add_cityids(cityID.encode());
        }

        packet.set_score(player.getScore());

        packet.set_civid(player.getCiv().id);
        packet.set_leadername(player.getLeader().name);

        return packet;
    }

    Culture getCultureFromProto(const CultureValues &proto, const IdConverter &playerIDs) {
        Culture c;

        for (int i = 0; i < proto.amounts_size(); i++) {
            const auto amount = proto.amounts()[i];
            const auto owner = playerIDs.get(proto.playerids()[i]);
            c.addCultureForPlayer(owner, amount);
        }

        return c;
    }
}
