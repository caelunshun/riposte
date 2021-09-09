//
// Created by Caelum van Ispelen on 6/20/21.
//

#ifndef RIPOSTE_SERVER_H
#define RIPOSTE_SERVER_H

#include <riposte.pb.h>
#include <absl/container/flat_hash_set.h>
#include <readerwriterqueue/readerwriterqueue.h>

#include "game.h"
#include "tech.h"
#include "saveload.h"
#include "network.h"

using namespace moodycamel;
using namespace rip::proto;

namespace rip {
    class Server;
    class BuildTask;

    // Represents a connection to the server from a client.
    class Connection {
        // The bridge used to transfer packets.
        ConnectionHandle handle;
        // ID of the player using this connection.
        PlayerId playerID;

        uint32_t currentRequestID = 0;

        bool isAdmin;

        Server *server;

    public:
        // Whether the player has ended their current turn.
        bool endedTurn = false;

        Connection(ConnectionHandle handle, PlayerId playerID, bool isAdmin, Server *server);

        template<typename T>
        void send(T &packet) {
            std::string data;
            packet.SerializeToString(&data);
            FnCallback callback = [](const RipResult &res) {};
            handle.sendMessage(data, callback);
        }

        void requestMoreData();

        void sendGameStarted(Game &game);
        void sendUpdateTile(Game &game, glm::uvec2 pos);
        void sendUpdateVisibility(Game &game);
        void sendGlobalData(Game &game);
        void sendTradeNetworks(Game &game);

        void handleComputePath(Game &game, const ComputePath &packet);
        void handleMoveUnits(Game &game, const MoveUnits &packet);
        void handleGetBuildTasks(Game &game, const GetBuildTasks &packet);
        void handleSetBuildTask(Game &game, const SetCityBuildTask &packet);
        void handleSetResearch(Game &game, const SetResearch &packet);
        void handleGetPossibleTechs(Game &game, const GetPossibleTechs &packet);
        void handleSetEconomySettings(Game &game, const SetEconomySettings &packet);
        void handleDoUnitAction(Game &game, const DoUnitAction &packet);
        void handleSetWorkerTask(Game &game, const SetWorkerTask &packet);
        void handleDeclareWar(Game &game, const DeclareWar &packet);
        void handleDeclarePeace(Game &game, const DeclarePeace &packet);
        void handleConfigureWorkedTiles(Game &game, const ConfigureWorkedTiles &packet);
        void handleBombardCity(Game &game, const BombardCity &packet);
        void handleSaveGame();

        void handlePacket(Game *game, AnyClient &packet);

        PlayerId getPlayerID() const;
        bool getIsAdmin() const;
    };

    // The Riposte game server.
    //
    // Wraps a Game and handles connections by sending/handling packets.
    class Server {
        std::vector<std::shared_ptr<Connection>> connections;

        absl::flat_hash_set<UnitId> dirtyUnits;
        absl::flat_hash_set<CityId> dirtyCities;
        absl::flat_hash_set<PlayerId> playersWithDirtyVisibility;
        absl::flat_hash_set<glm::uvec2, PosHash> dirtyTiles;
        absl::flat_hash_set<PlayerId> dirtyPlayers;

        // fields used for saving
        std::string gameName;
        std::string gameCategory;

        std::shared_ptr<NetworkingContext> networkCtx;

    public:
        // Must be initialized by the Server creator.
        // Not initialized by the constructor because
        // Game requires a Server pointer.
        std::unique_ptr<Game> game;
        std::shared_ptr<Registry> registry;
        std::shared_ptr<TechTree> techTree;

        Server(std::shared_ptr<NetworkingContext> networkCtx, std::string gameName, std::string gameCategory);

        void startGame();

        void addConnection(ConnectionHandle handle, PlayerId playerID, bool isAdmin);

        void broadcastUnitDeath(UnitId unitID);

        void flushDirtyItems();

        void markUnitDirty(UnitId unit);
        void markCityDirty(CityId city);
        void markPlayerVisibilityDirty(PlayerId player);
        void markTileDirty(glm::uvec2 pos);
        void markPlayerDirty(PlayerId player);

        void broadcastCombatEvent(
                UnitId attackerID,
                UnitId defenderID,
                UnitId winnerID,
                const std::vector<CombatRound> &rounds,
                int numCollateralTargets
                );
        void broadcastCityCaptured(CityId id, PlayerId capturer);
        void broadcastWarDeclared(PlayerId declarer, PlayerId declared);
        void broadcastPeaceDeclared(PlayerId declarer, PlayerId declared);
        void broadcastBordersExpanded(CityId cityID);

        void sendBuildTaskFinished(CityId cityID, const BuildTask *task);
        void sendBuildTaskFailed(CityId cityID, const BuildTask &task);

        void run(std::shared_ptr<ReaderWriterQueue<ConnectionHandle>> newConnections);

        void saveGame();
    };
}

#endif //RIPOSTE_SERVER_H
