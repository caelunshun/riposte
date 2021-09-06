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
#include "trade.h"
#include "protocol.h"
#include <fstream>

#define PACKET(packet, anyservername, id) AnyServer _anyServer; _anyServer.set_requestid(id); _anyServer.mutable_##anyservername()->CopyFrom(packet);
#define SEND(packet, anyservername, id)  { PACKET(packet, anyservername, id); send(_anyServer); }
#define BROADCAST(packet, anyservername, id) { PACKET(packet, anyservername, id); for (auto &connection : connections) { connection->send(_anyServer); }}
#define SENDTOCONN(packet, anyservername, id, playerID) { PACKET(packet, anyservername, id); for (auto &connection : connections) { if (connection->getPlayerID() == playerID) { connection->send(_anyServer); } } }

namespace rip {
    Connection::Connection(ConnectionHandle handle, PlayerId playerID, bool isAdmin, Server *server)
        : handle(std::move(handle)), playerID(playerID), isAdmin(isAdmin), server(server) {

        // Start receiving packets
        requestMoreData();
    }

    void Connection::sendUpdateTile(Game &game, glm::uvec2 pos) {
        auto packet = getUpdateTilePacket(game, pos, playerID);
        SEND(packet, updatetile, 0);
    }

    void Connection::sendUpdateVisibility(Game &game) {
        auto packet = getUpdateVisibilityPacket(game, playerID);
        SEND(packet, updatevisibility, 0);
    }

    PlayerId Connection::getPlayerID() const {
        return playerID;
    }

    void Connection::sendGlobalData(Game &game) {
        SEND(getUpdateGlobalDataPacket(game, playerID), updateglobaldata, 0);
    }

    void Connection::sendGameStarted(Game &game) {
        GameStarted gameStarted;
        auto *gameData = gameStarted.mutable_gamedata();

        for (auto &player : game.getPlayers()) {
            gameData->add_players()->CopyFrom(getUpdatePlayerPacket(game, player));
        }
        for (auto &city : game.getCities()) {
            gameData->add_cities()->CopyFrom(getUpdateCityPacket(game, city));
        }
        for (auto &unit : game.getUnits()) {
            gameData->add_units()->CopyFrom(getUpdateUnitPacket(game, unit));
        }

        gameData->mutable_globaldata()->CopyFrom(getUpdateGlobalDataPacket(game, playerID));
        gameData->mutable_map()->CopyFrom(getUpdateMapPacket(game, playerID));
        gameData->mutable_visibility()->CopyFrom(getUpdateVisibilityPacket(game, playerID));

        ServerLobbyPacket packet;
        packet.mutable_gamestarted()->CopyFrom(gameStarted);
        send(packet);
    }

    void Connection::sendTradeNetworks(Game &game) {
        UpdateTradeNetworks packet;
        for (const auto &route : game.getTradeRoutes().getTradeRoutes()) {
            auto *network = packet.add_networks();
            for (const auto pos : route.getTiles()) {
                auto *p = network->add_positions();
                p->set_x(pos.x);
                p->set_y(pos.y);
            }
            for (const auto cityID : route.getVisitedCities()) {
                network->add_visitedcityids(cityID.encode());
            }
            network->set_id(route.id.encode());
        }
        SEND(packet, updatetradenetworks, 0);
    }

    void Connection::handleComputePath(Game &game, const ComputePath &packet) {
        auto &unitKind = game.getRegistry().getUnit(packet.unitkindid());
        auto path = computeShortestPath(game, glm::uvec2(packet.from().x(), packet.from().y()), glm::uvec2(packet.to().x(), packet.to().y()),
                                        game.getPlayer(playerID).getVisibilityMap(), *unitKind, playerID);

        PathComputed response;
        if (path.has_value()) {
            writePath(*path, *response.mutable_path());
        }
        SEND(response, pathcomputed, currentRequestID);
    }

    void Connection::handleMoveUnits(Game &game, const MoveUnits &packet) {
        bool success = true;

        glm::uvec2 targetPos(packet.targetpos().x(), packet.targetpos().y());

        // First, check if we can move all units.
        for (const auto unitID : packet.unitids()) {
            const auto &unit = game.getUnit(UnitId(unitID));
            if (!unit.canMove(targetPos, game)) {
                success = false;
                break;
            }
        }

        if (success) {
            // Now actually move the units.
            for (const auto unitID : packet.unitids()) {
                auto &unit = game.getUnit(UnitId(unitID));
                unit.moveTo(targetPos, game, true);
            }
        }

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
            std::cerr << "[server-err] invalid improvement ID " << kind.improvementid() << std::endl;
            return;
        }

        workerCap->setTask(std::make_unique<BuildImprovementTask>(improvement->getNumBuildTurns(), worker.getPos(), std::move(improvement)));

        game.getServer().markUnitDirty(worker.getID());
    }

    void Connection::handleDeclareWar(Game &game, const DeclareWar &packet) {
        game.getPlayer(playerID).declareWarOn(PlayerId(packet.onplayerid()), game);
    }

    void Connection::handleConfigureWorkedTiles(Game &game, const ConfigureWorkedTiles &packet) {
        auto &city = game.getCity(CityId(packet.cityid()));
        glm::uvec2 tilePos(packet.tilepos().x(), packet.tilepos().y());
        if (packet.shouldmanuallywork()) {
            city.addManualWorkedTile(game, tilePos);
        } else {
            city.removeManualWorkedTile(tilePos);
        }
        city.updateWorkedTiles(game);
        game.getServer().markCityDirty(city.getID());
    }

    void Connection::handleBombardCity(Game &game, const BombardCity &packet) {
        auto &siegeUnit = game.getUnit(UnitId(packet.siegeunitid()));
        auto &targetCity = game.getCity(CityId(packet.targetcityid()));

        auto *cap = siegeUnit.getCapability<BombardCityCapability>();
        if (cap) {
            cap->bombardCity(game, targetCity);
        }
    }

    void Connection::handleSaveGame() {
        if (isAdmin) {
            server->saveGame();
        }
    }

    void Connection::requestMoreData() {
        FnCallback callback = [&](const RipResult &res) {
            if (rip_result_is_success(&res)) {
                const auto bytes = rip_result_get_bytes(&res);
                AnyClient packet;
                packet.ParseFromArray((void*) bytes.ptr, bytes.len);
                handlePacket(&*server->game, packet);

                requestMoreData();
            }
        };
        handle.recvMessage(callback);
    }

    void Connection::handlePacket(Game *gamePtr, AnyClient &packet) {
        currentRequestID = packet.requestid();
        auto &game = *gamePtr;
        if (packet.has_computepath()) {
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
        } else if (packet.has_configureworkedtiles()) {
            handleConfigureWorkedTiles(game, packet.configureworkedtiles());
        } else if (packet.has_bombardcity()) {
            handleBombardCity(game, packet.bombardcity());
        } else if (packet.has_savegame()) {
            handleSaveGame();
        }
    }

    bool Connection::getIsAdmin() const {
        return isAdmin;
    }

    Server::Server(std::shared_ptr<NetworkingContext> networkCtx, std::string gameName, std::string gameCategory)
        : networkCtx(networkCtx),
        gameName(std::move(gameName)), gameCategory(std::move(gameCategory)) {
    }

    void Server::addConnection(ConnectionHandle handle, PlayerId playerID, bool isAdmin) {
        connections.emplace_back(std::make_shared<Connection>(std::move(handle), playerID, isAdmin, this));
    }

    void Server::run(std::shared_ptr<ReaderWriterQueue<ConnectionHandle>> newConnections) {
        startGame();

        while (!connections.empty()) {
            networkCtx->waitAndInvokeCallbacks();

            bool haveAllTurnsEnded = true;
            for (auto &connection : connections) {
                if (!connection->endedTurn) {
                    haveAllTurnsEnded = false;
                    break;
                }
            }

            if (haveAllTurnsEnded) {
                game->advanceTurn();
                // saveGame();
                for (auto &connection : connections) {
                    connection->endedTurn = false;
                    connection->sendGlobalData(*game);
                    connection->sendTradeNetworks(*game);
                }
            }

            game->tick();
            flushDirtyItems();
        }
    }

    void Server::startGame() {
        for (auto &conn : connections) {
            conn->sendGameStarted(*game);
        }
    }

    void Server::broadcastUnitDeath(UnitId unitID) {
        DeleteUnit packet;
        packet.set_unitid(unitID.encode());
        BROADCAST(packet, deleteunit, 0);
    }

    void Server::broadcastCityCaptured(CityId id, PlayerId capturer) {
        CityCaptured packet;
        packet.set_cityid(id.encode());
        packet.set_capturerid(capturer.encode());
        BROADCAST(packet, citycaptured, 0);
    }

    void Server::broadcastWarDeclared(PlayerId declarer, PlayerId declared) {
        WarDeclared packet;
        packet.set_declarerid(declarer.encode());
        packet.set_declaredid(declared.encode());
        BROADCAST(packet, wardeclared, 0);
    }

    void Server::flushDirtyItems() {
        auto &game = *this->game;

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
                if (conn->getPlayerID() == playerID) {
                    conn->sendUpdateVisibility(game);
                    break;
                }
            }
        }

        for (const auto tile : dirtyTiles) {
            for (auto &conn : connections) {
                conn->sendUpdateTile(game, tile);
            }
        }

        for (const auto playerID : dirtyPlayers) {
            auto &player = game.getPlayer(playerID);
            auto packet = getUpdatePlayerPacket(game, player);
            BROADCAST(packet, updateplayer, 0);
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

    void Server::broadcastCombatEvent(UnitId attackerID, UnitId defenderID, UnitId winnerID,
                                      const std::vector<CombatRound> &rounds,
                                      int numCollateralTargets) {
        CombatEvent packet;
        packet.set_attackerid(attackerID.encode());
        packet.set_defenderid(defenderID.encode());
        for (const auto &round : rounds) {
            packet.add_rounds()->CopyFrom(round);
        }

        packet.set_numcollateraltargets(numCollateralTargets);

        packet.set_attackerwon(winnerID == attackerID);

        AnyServer wrappedPacket;
        wrappedPacket.mutable_combatevent()->CopyFrom(packet);

        // A combat event is sent to a client only if one of its units is involved in the combat.
        const auto &attacker = game->getUnit(attackerID);
        const auto &defender = game->getUnit(defenderID);
        for (auto &conn : connections) {
            if (attacker.getOwner() == conn->getPlayerID() || defender.getOwner() == conn->getPlayerID()) {
                conn->send(wrappedPacket);
            }
        }
    }

    void Server::saveGame() {
        if (!game) return;

        const auto path = getSavePath(gameCategory, gameName, game->getTurn());
        std::ofstream f(path);

        const auto data = serializeGameToSave(*game, gameName);
        f << data;

        f.close();

        GameSaved packet;
        BROADCAST(packet, gamesaved, 0);

        std::cerr << "Saved game to " << path << std::endl;
    }

    void Server::broadcastBordersExpanded(CityId cityID) {
        BordersExpanded packet;
        packet.set_cityid(cityID.encode());
        BROADCAST(packet, bordersexpanded, 0);
    }

    void Server::sendBuildTaskFinished(CityId cityID, const BuildTask *task) {
        if (game->getPlayer(game->getCity(cityID).getOwner()).hasAI()) return;

        // ensure UpdateCity is sent first if the city was just created
        flushDirtyItems();

        BuildTaskFinished packet;
        packet.set_cityid(cityID.encode());

        if (task) {
            writeBuildTask(*task, *packet.mutable_task());
        }

        SENDTOCONN(packet, buildtaskfinished, 0, game->getCity(cityID).getOwner());
    }

    void Server::sendBuildTaskFailed(CityId cityID, const BuildTask &task) {
        BuildTaskFailed packet;
        packet.set_cityid(cityID.encode());
        writeBuildTask(task, *packet.mutable_task());
        SENDTOCONN(packet, buildtaskfailed, 0, game->getCity(cityID).getOwner());
    }
}
