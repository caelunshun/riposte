//
// Created by Caelum van Ispelen on 6/20/21.
//

#include "server.h"
#include "mapgen.h"
#include "player.h"
#include "tile.h"
#include "unit.h"
#include "culture.h"
#include "worker.h"
#include "city.h"
#include "ship.h"
#include <riposte.pb.h>
#include <thread>

#define SEND(packet, anyservername)  { AnyServer _anyServer; _anyServer.mutable_##anyservername()->CopyFrom(packet); send(std::move(_anyServer)); }

namespace rip {
    void setPlayerInfo(const Player &player, PlayerInfo &playerInfo) {
        playerInfo.set_username(player.getUsername());
        playerInfo.set_civid(player.getCiv().id);
        playerInfo.set_leadername(player.getLeader().name);
        playerInfo.set_score(player.getScore());
        playerInfo.set_id(player.getID().first);
        playerInfo.set_isadmin(false); // TODO: permissions
    }

    UpdateGlobalData getUpdateGlobalDataPacket(Game &game) {
        UpdateGlobalData packet;
        packet.set_turn(game.getTurn());
        packet.set_era(static_cast<::Era>(static_cast<int>(game.getEra())));

        for (const auto &player : game.getPlayers()) {
            auto *protoPlayer = packet.add_players();
            setPlayerInfo(player, *protoPlayer);
        }

        return packet;
    }

    void writeYield(const Yield &yield, ::Yield &protoYield) {
        protoYield.set_commerce(yield.commerce);
        protoYield.set_food(yield.food);
        protoYield.set_hammers(yield.hammers);
    }

    void setTile(const Game &game, PlayerId player, glm::uvec2 pos, const Tile &tile, ::Tile &protoTile) {
        protoTile.set_terrain(static_cast<::Terrain>(static_cast<int>(tile.getTerrain())));
        protoTile.set_forested(tile.isForested());
        protoTile.set_hilled(tile.isHilled());

        const auto yield = tile.getYield(game, pos, player);
        writeYield(yield, *protoTile.mutable_yield());

        const auto owner = game.getCultureMap().getTileOwner(pos);
        if (owner.has_value()) {
            protoTile.set_ownerid(owner->second);
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

                packet.add_visibility(static_cast<::Visibility>(static_cast<int>(player.getVisibilityMap()[pos])));
            }
        }

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
        packet.set_ownerid(unit.getOwner().first);
        packet.set_health(unit.getHealth());
        packet.set_movementleft(unit.getMovementLeft());

        if (unit.hasPath()) {
            writePath(unit.getPath(), *packet.mutable_followingpath());
        }

        for (const auto &capability : unit.getCapabilities()) {
            ::Capability protoCap;
            const auto *foundCity = dynamic_cast<const FoundCityCapability*>(&*capability);
            const auto *worker = dynamic_cast<const WorkerCapability*>(&*capability);
            const auto *carryUnits = dynamic_cast<const CarryUnitsCapability*>(&*capability);
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
                    protoCarryUnits->add_carryingunitids(unitID.first);
                }
            }
        }

        packet.set_id(unit.getID().first);

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
        packet.set_ownerid(city.getOwner().first);

        if (city.hasBuildTask()) {
            writeBuildTask(*city.getBuildTask(), *packet.mutable_buildtask());
        }

        writeYield(city.computeYield(game), *packet.mutable_yield());
        packet.set_culture(city.getCulture().getCultureForPlayer(city.getOwner()));
        // packet.set_cultureneeded(city.getCultureNeeded()); TODO
        packet.set_id(city.getID().first);

        for (const auto &building : city.getBuildings()) {
            packet.add_buildingnames(building->name);
        }

        packet.set_population(city.getPopulation());
        packet.set_storedfood(city.getStoredFood());
        packet.set_foodneededforgrowth(city.getFoodNeededForGrowth());
        packet.set_consumedfood(city.getConsumedFood());

        return packet;
    }

    void Connection::sendGameData(Game &game) {
        SEND(getUpdateGlobalDataPacket(game), updateglobaldata);
        SEND(getUpdateMapPacket(game, playerID), updatemap);

        for (auto &unit : game.getUnits()) {
            SEND(getUpdateUnitPacket(game, unit), updateunit);
        }
        for (auto &city : game.getCities()) {
            SEND(getUpdateCityPacket(game, city), updatecity);
        }
    }

    void Connection::update(Game &game) {

    }

    Server::Server(std::shared_ptr<Registry> registry, std::shared_ptr<TechTree> techTree)
        : game(MapGenerator().generate(64, 64, registry, techTree)) {
    }

    void Server::addConnection(std::unique_ptr<Bridge> bridge) {
        connections.emplace_back(std::move(bridge), game.getThePlayerID()); // TODO: multiplayer
        connections[connections.size() - 1].sendGameData(game);
    }

    void Server::run() {
        while (!connections.empty()) {
            for (auto &connection : connections) {
                connection.update(game);
            }

            std::this_thread::sleep_for(std::chrono::milliseconds(15));
        }
    }
}
