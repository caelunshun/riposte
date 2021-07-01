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
#include <thread>

#define PACKET(packet, anyservername, id) AnyServer _anyServer; _anyServer.set_requestid(id); _anyServer.mutable_##anyservername()->CopyFrom(packet);
#define SEND(packet, anyservername, id)  { PACKET(packet, anyservername, id); send(_anyServer); }
#define BROADCAST(packet, anyservername, id) { PACKET(packet, anyservername, id); for (auto &connection : connections) { connection.send(_anyServer); }}

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
        packet.set_era(static_cast<::Era>(static_cast<int>(game.getEra())));

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

        if (unit.hasPath()) {
            writePath(unit.getPath(), *packet.mutable_followingpath());
        }

        for (const auto &capability : unit.getCapabilities()) {
            auto &protoCap = *packet.add_capabilities();
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
                    protoCarryUnits->add_carryingunitids(unitID.encode());
                }
            }
        }

        packet.set_id(unit.getID().encode());

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

        return packet;
    }

    void Connection::sendUpdateTile(Game &game, glm::uvec2 pos) {
        auto packet = getUpdateTilePacket(game, pos, playerID);
        SEND(packet, updatetile, 0);
    }

    void Connection::sendUpdateVisibility(Game &game) {
        auto packet = getUpdateVisibilityPacket(game, playerID);
        SEND(packet, updatevisibility, 0);
    }

    void Connection::sendPlayerData(Game &game) {
        auto packet = getUpdatePlayerPacket(game, game.getPlayer(playerID));
        SEND(packet, updateplayer, 0);
    }

    PlayerId Connection::getPlayerID() const {
        return rip::PlayerId();
    }

    void Connection::sendGlobalData(Game &game) {
        SEND(getUpdateGlobalDataPacket(game, playerID), updateglobaldata, 0);
    }

    void Connection::sendGameData(Game &game) {
        sendGlobalData(game);
        sendPlayerData(game);
        sendUpdateVisibility(game);
        SEND(getUpdateMapPacket(game, playerID), updatemap, 0);

        for (auto &unit : game.getUnits()) {
            SEND(getUpdateUnitPacket(game, unit), updateunit, 0);
        }
        for (auto &city : game.getCities()) {
            SEND(getUpdateCityPacket(game, city), updatecity, 0);
        }
    }

    void Connection::handleClientInfo(Game &game, const ClientInfo &packet) {
        game.getPlayer(playerID).setUsername(packet.username());
    }

    void Connection::handleComputePath(Game &game, const ComputePath &packet) {
        auto &unitKind = game.getRegistry().getUnit(packet.unitkindid());
        auto path = computeShortestPath(game, glm::uvec2(packet.from().x(), packet.from().y()), glm::uvec2(packet.to().x(), packet.to().y()),
                                        game.getPlayer(playerID).getVisibilityMap(), *unitKind);

        PathComputed response;
        if (path.has_value()) {
            writePath(*path, *response.mutable_path());
        }
        SEND(response, pathcomputed, currentRequestID);
    }

    void Connection::handleMoveUnits(Game &game, const MoveUnits &packet) {
        bool success = false;

        std::deque<glm::uvec2> path(packet.pathtofollow().positions_size() / 2);
        for (int i = 0; i < packet.pathtofollow().positions_size() / 2; i++) {
            path[i] = glm::uvec2(packet.pathtofollow().positions(i * 2), packet.pathtofollow().positions(i * 2 + 1));
        }

        while (!path.empty()) {
            auto targetPos = path[0];
            path.pop_front();

            bool possible = true;
            bool skip = false;
            for (const auto unitID : packet.unitids()) {
                auto &unit = game.getUnit(UnitId(unitID));
                if (unit.getPos() == targetPos) {
                    skip = true;
                    break;
                }
                if (!unit.canMove(targetPos, game)) {
                    possible = false;
                    break;
                }
            }

            if (skip) continue;

            if (!possible) break;

            success = true;
            for (const auto unitID : packet.unitids()) {
                auto &unit = game.getUnit(UnitId(unitID));
                unit.moveTo(targetPos, game, true);
            }
        }

        game.getServer().flushDirtyItems();

        ConfirmMoveUnits response;
        response.set_success(success);
        SEND(response, confirmmoveunits, currentRequestID);
    }

    void Connection::handleGetBuildTasks(Game &game, const GetBuildTasks &packet) {
        PossibleCityBuildTasks response;

        auto &city = game.getCity(CityId(packet.cityid()));
        for (const auto &task : city.getPossibleBuildTasks(game)) {
            writeBuildTask(*task, *response.add_tasks());
        }

        SEND(response, possiblecitybuildtasks, currentRequestID);
    }

    void Connection::handleSetBuildTask(Game &game, const SetCityBuildTask &packet) {
        std::unique_ptr<BuildTask> task;

        if (packet.task().has_unit()) {
            task = std::make_unique<UnitBuildTask>(
                    game.getRegistry().getUnit(packet.task().unit().unitkindid())
                    );
        } else {
            task = std::make_unique<BuildingBuildTask>(
                    game.getRegistry().getBuilding(packet.task().building().buildingname())
                    );
        }

        CityId cityID(packet.cityid());
        auto &city = game.getCity(cityID);
        city.setBuildTask(std::move(task));

        game.getServer().markCityDirty(cityID);
    }

    void Connection::handleSetResearch(Game &game, const SetResearch &packet) {
        auto tech = game.getTechTree().getTechs().at(packet.techid());
        game.getPlayer(playerID).setResearchingTech(tech);
        game.getServer().markPlayerDirty(playerID);
    }

    void Connection::handleGetPossibleTechs(Game &game, const GetPossibleTechs &packet) {
        PossibleTechs response;

        for (const auto &tech : game.getPlayer(playerID).getTechs().getPossibleResearches()) {
            response.add_techs(tech->name);
        }

        SEND(response, possibletechs, currentRequestID);
    }

    void Connection::handleSetEconomySettings(Game &game, const SetEconomySettings &packet) {
        auto &player = game.getPlayer(playerID);
        player.setSciencePercent(packet.beakerpercent(), game);
        game.getServer().markPlayerDirty(playerID);
    }

    void Connection::handleDoUnitAction(Game &game, const DoUnitAction &packet) {
        UnitId id(packet.unitid());
        auto &unit = game.getUnit(id);

        switch (packet.action()) {
            case UnitAction::Kill:
                game.killUnit(UnitId(packet.unitid()));

                break;
            case UnitAction::Fortify:
                unit.fortify();
                break;
            case UnitAction::SkipTurn:
                unit.skipTurn();
                break;
            case UnitAction::FortifyUntilHealed:
                unit.fortifyUntilHealed();
                break;
            case UnitAction::FoundCity:
                auto *cap = unit.getCapability<FoundCityCapability>();
                if (cap) {
                    cap->foundCity(game);
                }
                break;
        }

        game.getServer().markUnitDirty(unit.getID());
    }

    void Connection::handleSetWorkerTask(Game &game, const SetWorkerTask &packet) {
        auto &worker = game.getUnit(UnitId(packet.workerid()));
        auto *workerCap = worker.getCapability<WorkerCapability>();
        if (!workerCap) return;

        std::unique_ptr<Improvement> improvement;
        auto &kind = packet.task().kind().buildimprovement();
        if (kind.improvementid() == "Cottage") {
            improvement = std::make_unique<Cottage>(worker.getPos());
        } else if (kind.improvementid() == "Road") {
            improvement = std::make_unique<Road>(worker.getPos());
        } else if (kind.improvementid() == "Farm") {
            improvement = std::make_unique<Farm>(worker.getPos());
        } else if (kind.improvementid() == "Pasture") {
            improvement = std::make_unique<Pasture>(worker.getPos());
        } else if (kind.improvementid() == "Mine") {
            improvement = std::make_unique<Mine>(worker.getPos());
        } else {
            std::cout << "[server-err] invalid improvement ID " << kind.improvementid() << std::endl;
            return;
        }

        workerCap->setTask(std::make_unique<BuildImprovementTask>(improvement->getNumBuildTurns(), worker.getPos(), std::move(improvement)));

        game.getServer().markUnitDirty(worker.getID());
    }

    void Connection::handleDeclareWar(Game &game, const DeclareWar &packet) {
        game.getPlayer(playerID).declareWarOn(PlayerId(packet.onplayerid()), game);
    }

    void Connection::handlePacket(Game &game, AnyClient &packet) {
        currentRequestID = packet.requestid();
        if (packet.has_clientinfo()) {
            handleClientInfo(game, packet.clientinfo());
        } else if (packet.has_computepath()) {
            handleComputePath(game, packet.computepath());
        } else if (packet.has_moveunits()) {
            handleMoveUnits(game, packet.moveunits());
        } else if (packet.has_endturn()) {
            endedTurn = true;
        } else if (packet.has_getbuildtasks()) {
            handleGetBuildTasks(game, packet.getbuildtasks());
        } else if (packet.has_setcitybuildtask()) {
            handleSetBuildTask(game, packet.setcitybuildtask());
        } else if (packet.has_setresearch()) {
            handleSetResearch(game, packet.setresearch());
        } else if (packet.has_getpossibletechs()) {
            handleGetPossibleTechs(game, packet.getpossibletechs());
        } else if (packet.has_seteconomysettings()) {
            handleSetEconomySettings(game, packet.seteconomysettings());
        } else if (packet.has_dounitaction()) {
            handleDoUnitAction(game, packet.dounitaction());
        } else if (packet.has_setworkertask()) {
            handleSetWorkerTask(game, packet.setworkertask());
        } else if (packet.has_declarewar()) {
            handleDeclareWar(game, packet.declarewar());
        }
    }

    void Connection::update(Game &game) {
        while (true) {
            auto packetData = bridge->pollReceivedPacket();
            if (!packetData.has_value()) break;

            // Parse the packet.
            AnyClient packet;
            if (!packet.ParseFromString(*packetData)) {
                std::cout << "received malformed packet!" << std::endl;
                continue;
            }

            handlePacket(game, packet);
        }
    }

    Server::Server(std::shared_ptr<Registry> registry, std::shared_ptr<TechTree> techTree)
        : game(MapGenerator().generate(64, 64, registry, techTree, this)) {
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

            bool haveAllTurnsEnded = true;
            for (auto &connection : connections) {
                if (!connection.endedTurn) {
                    haveAllTurnsEnded = false;
                    break;
                }
            }

            if (haveAllTurnsEnded) {
                game.advanceTurn();
                for (auto &connection : connections) {
                    connection.endedTurn = false;
                    connection.sendGlobalData(game);
                }
            }

            game.tick();
            flushDirtyItems();

            std::this_thread::sleep_for(std::chrono::milliseconds(15));
        }
    }

    void Server::broadcastUnitDeath(UnitId unitID) {
        DeleteUnit packet;
        packet.set_unitid(unitID.encode());
        BROADCAST(packet, deleteunit, 0);
    }

    void Server::flushDirtyItems() {
        for (const auto unitID : dirtyUnits) {
            if (game.getUnits().contains(unitID)) {
                auto packet = getUpdateUnitPacket(game, game.getUnit(unitID));
                BROADCAST(packet, updateunit, 0);
            }
        }

        for (const auto cityID : dirtyCities) {
            if (game.getCities().contains(cityID)) {
                auto packet = getUpdateCityPacket(game, game.getCity(cityID));
                BROADCAST(packet, updatecity, 0);
            }
        }

        for (const auto playerID : playersWithDirtyVisibility) {
            for (auto &conn : connections) {
                if (conn.getPlayerID() == playerID) {
                    conn.sendUpdateVisibility(game);
                    break;
                }
            }
        }

        for (const auto tile : dirtyTiles) {
            for (auto &conn : connections) {
                conn.sendUpdateTile(game, tile);
            }
        }

        for (const auto playerID : dirtyPlayers) {
            auto &player = game.getPlayer(playerID);
            for (auto &conn : connections) {
                if (conn.getPlayerID() == playerID) {
                    conn.sendPlayerData(game);
                    break;
                }
            }
        }

        dirtyUnits.clear();
        dirtyCities.clear();
        playersWithDirtyVisibility.clear();
        dirtyTiles.clear();
        dirtyPlayers.clear();
    }

    void Server::markUnitDirty(UnitId unit) {
        dirtyUnits.insert(unit);
    }

    void Server::markCityDirty(CityId city) {
        dirtyCities.insert(city);
    }

    void Server::markPlayerVisibilityDirty(PlayerId player) {
        playersWithDirtyVisibility.insert(player);
    }

    void Server::markTileDirty(glm::uvec2 pos) {
        dirtyTiles.insert(pos);
    }

    void Server::markPlayerDirty(PlayerId player) {
        dirtyPlayers.insert(player);
    }
}
