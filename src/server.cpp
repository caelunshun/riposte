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
#define BROADCAST(packet, anyservername, id) { PACKET(packet, anyservername, id); for (auto &connection : connections) { connection.send(_anyServer); }}

namespace rip {
    Connection::Connection(std::unique_ptr<Bridge> bridge, PlayerId playerID, bool isAdmin, Server *server)
        : bridge(std::move(bridge)), playerID(playerID), isAdmin(isAdmin), server(server) {
        // choose random initial civ and leader
        Rng rng;
        while (true) {
            auto civ = server->registry->getCivs()[rng.u32(0, server->registry->getCivs().size())];
            if (!server->hasPlayerWithCiv(civ->id)) {
                lobbyPlayerInfo.set_civid(civ->id);
                lobbyPlayerInfo.set_leadername(civ->leaders[rng.u32(0, civ->leaders.size())].name);
                break;
            }
        }
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
        return playerID;
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

    void Connection::handleGameOptions(const GameOptions &packet) {
        if (isAdmin) {
            server->setGameOptions(packet);
        }
    }

    void Connection::handleAdminStartGame(const AdminStartGame &packet) {
        if (isAdmin) {
            server->startGame();
        }
    }

    void Connection::handleSetLeader(const SetLeader &packet) {
        lobbyPlayerInfo.set_civid(packet.civid());
        lobbyPlayerInfo.set_leadername(packet.leader());
    }

    void Connection::handleClientInfo(const ClientInfo &packet) {
        username = packet.username();
        server->updateServerInfo();
    }

    void Connection::handlePacket(Game *gamePtr, AnyClient &packet) {
        currentRequestID = packet.requestid();
        if (gamePtr && !server->inLobby) {
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
        } else {
            // game hasn't started; we're in the lobby phase
            if (packet.has_gameoptions()) {
                handleGameOptions(packet.gameoptions());
            } else if (packet.has_adminstartgame()) {
                handleAdminStartGame(packet.adminstartgame());
            } else if (packet.has_setleader()) {
                handleSetLeader(packet.setleader());
            } else if (packet.has_clientinfo()) {
                handleClientInfo(packet.clientinfo());
            }
        }
    }

    void Connection::update(Game *game) {
        while (true) {
            auto packetData = bridge->pollReceivedPacket();
            if (!packetData.has_value()) break;

            // Parse the packet.
            AnyClient packet;
            if (!packet.ParseFromString(*packetData)) {
                std::cout << "received malformed packet!" << std::endl;
                continue;
            }

            try {
                handlePacket(game, packet);
            } catch (std::exception &e) {
                std::cout << "ERROR while handling packet: " << e.what() << std::endl;
            }
        }
    }

    const LobbyPlayer &Connection::getLobbyPlayerInfo() const {
        return lobbyPlayerInfo;
    }

    const std::string &Connection::getUsername() const {
        return username;
    }

    bool Connection::getIsAdmin() const {
        return isAdmin;
    }

    Server::Server(std::shared_ptr<Registry> registry, std::shared_ptr<TechTree> techTree, std::string gameCategory)
        : registry(registry), techTree(techTree), gameCategory(std::move(gameCategory)) {
        gameOptions.set_mapwidth(32);
        gameOptions.set_mapheight(32);
        gameOptions.set_numhumanplayers(1);
        gameOptions.set_numaiplayers(1);

        gameName = "Beta Game - #" + std::to_string(Rng().u32(0, 10000));
    }

    void Server::addConnection(std::unique_ptr<Bridge> bridge, bool isAdmin) {
        connections.emplace_back(std::move(bridge), playerIDAllocator.insert(0), isAdmin, this);
    }

    void Server::updateServerInfo() {
        ServerInfo packet;
        
        // Add human players
        for (auto &conn : connections) {
            auto *player = packet.add_players();
            player->set_civid(conn.getLobbyPlayerInfo().civid());
            player->set_leadername(conn.getLobbyPlayerInfo().leadername());
            player->set_username(conn.getUsername());
            player->set_ishuman(true);
            player->set_playerid(conn.getPlayerID().encode());
            player->set_exists(true);
            player->set_isadmin(conn.getIsAdmin());
        }

        // Add empty slots
        int neededPlayers = gameOptions.numhumanplayers() - connections.size();
        for (int i = 0; i < neededPlayers; i++) {
            auto *player = packet.add_players();
            player->set_exists(false);
            player->set_ishuman(true);
        }

        for (auto &conn : connections) {
            packet.set_theplayerid(conn.getPlayerID().encode());
            AnyServer anyServer;
            anyServer.mutable_serverinfo()->CopyFrom(packet);
            conn.send(anyServer);
        }
    }

    void Server::run(std::shared_ptr<ReaderWriterQueue<std::unique_ptr<Bridge>>> newConnections) {
        while (!connections.empty()) {
            for (auto &connection : connections) {
                connection.update(game.get());
            }

            if (!inLobby) {
                bool haveAllTurnsEnded = true;
                for (auto &connection : connections) {
                    if (!connection.endedTurn) {
                        haveAllTurnsEnded = false;
                        break;
                    }
                }

                if (haveAllTurnsEnded) {
                    game->advanceTurn();
                    for (auto &connection : connections) {
                        connection.endedTurn = false;
                        connection.sendGlobalData(*game);
                        connection.sendTradeNetworks(*game);
                    }
                }

                game->tick();
                flushDirtyItems();
            } else {
                std::unique_ptr<Bridge> newBridge;
                while (newConnections->try_dequeue(newBridge)) {
                    addConnection(std::move(newBridge), false);
                }
            }

            std::this_thread::sleep_for(std::chrono::milliseconds(15));
        }
    }

    void Server::setGameOptions(GameOptions gameOptions) {
        this->gameOptions = std::move(gameOptions);
    }

    void Server::startGame() {
        const int minMapWidth = 16, minMapHeight = 16;

        if (gameOptions.mapwidth() < minMapWidth) {
            gameOptions.set_mapwidth(minMapWidth);
        }
        if (gameOptions.mapheight() < minMapHeight) {
            gameOptions.set_mapheight(minMapHeight);
        }

        if (gameOptions.numhumanplayers() < 1) {
            gameOptions.set_numhumanplayers(1);
        }

        if (!game) {
            // Generate a new game.
            auto theGame = MapGenerator().generate(gameOptions, registry, techTree, this);
            game = std::make_unique<Game>(std::move(theGame));
        }

        inLobby = false;

        StartGame startGame;
        BROADCAST(startGame, startgame, 0);

        for (auto &conn : connections) {
            conn.sendGameData(*game);
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
            if (attacker.getOwner() == conn.getPlayerID() || defender.getOwner() == conn.getPlayerID()) {
                conn.send(wrappedPacket);
            }
        }
    }

    bool Server::hasPlayerWithCiv(const std::string &civID) const {
        for (const auto &conn : connections) {
            if (conn.getLobbyPlayerInfo().civid() == civID) {
                return true;
            }
        }
        return false;
    }

    void Server::setSaveFileToLoadFrom(SaveFile f) {
        saveFileToLoadFrom = std::move(f);

        auto loadedGame = loadGameFromSave(this, *saveFileToLoadFrom, registry, techTree);
        game = std::make_unique<Game>(std::move(loadedGame.game));
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

        std::cout << "Saved game to " << path << std::endl;
    }

    void Server::broadcastBordersExpanded(CityId cityID) {
        BordersExpanded packet;
        packet.set_cityid(cityID.encode());
        BROADCAST(packet, bordersexpanded, 0);
    }
}
