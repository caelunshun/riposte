//
// Created by Caelum van Ispelen on 6/20/21.
//

#ifndef RIPOSTE_SERVER_H
#define RIPOSTE_SERVER_H

#include <riposte.pb.h>
#include <absl/container/flat_hash_set.h>
#include <readerwriterqueue/readerwriterqueue.h>

#include "bridge.h"
#include "game.h"
#include "tech.h"
#include "saveload.h"

using namespace moodycamel;

namespace rip {
    class Server;

    // Represents a connection to the server from a client.
    class Connection {
        // The bridge used to transfer packets.
        std::unique_ptr<Bridge> bridge;
        // ID of the player using this connection.
        PlayerId playerID;

        uint32_t currentRequestID = 0;

        bool isAdmin;

        Server *server;

        LobbyPlayer lobbyPlayerInfo;

        std::string username;

    public:
        // Whether the player has ended their current turn.
        bool endedTurn = false;

        Connection(std::unique_ptr<Bridge> bridge, PlayerId playerID, bool isAdmin, Server *server);

        template<typename T>
        void send(T &packet) {
            std::string data;
            packet.SerializeToString(&data);
            bridge->sendPacket(std::move(data));
        }

        void sendGameData(Game &game);
        void sendUpdateTile(Game &game, glm::uvec2 pos);
        void sendUpdateVisibility(Game &game);
        void sendPlayerData(Game &game);
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
        void handleConfigureWorkedTiles(Game &game, const ConfigureWorkedTiles &packet);
        void handleBombardCity(Game &game, const BombardCity &packet);
        void handleSaveGame();

        void handleClientInfo(const ClientInfo &packet);
        void handleGameOptions(const GameOptions &packet);
        void handleAdminStartGame(const AdminStartGame &packet);
        void handleSetLeader(const SetLeader &packet);

        void handlePacket(Game *game, AnyClient &packet);

        void update(Game *game);

        PlayerId getPlayerID() const;
        const LobbyPlayer &getLobbyPlayerInfo() const;
        const std::string &getUsername() const;
        bool getIsAdmin() const;
    };

    // The Riposte game server.
    //
    // Wraps a Game and handles connections by sending/handling packets.
    class Server {
        std::vector<Connection> connections;

        slot_map<uint16_t> playerIDAllocator;

        absl::flat_hash_set<UnitId> dirtyUnits;
        absl::flat_hash_set<CityId> dirtyCities;
        absl::flat_hash_set<PlayerId> playersWithDirtyVisibility;
        absl::flat_hash_set<glm::uvec2, PosHash> dirtyTiles;
        absl::flat_hash_set<PlayerId> dirtyPlayers;

        GameOptions gameOptions;

        // fields used for saving
        std::string gameName;
        std::string gameCategory;

        std::optional<SaveFile> saveFileToLoadFrom;

    public:
        bool inLobby = true;

        // NB: may be null if we're still in the lobby phase.
        std::unique_ptr<Game> game;
        std::shared_ptr<Registry> registry;
        std::shared_ptr<TechTree> techTree;

        Server(std::shared_ptr<Registry> registry, std::shared_ptr<TechTree> techTree, std::string gameCategory);
        void setGameOptions(GameOptions gameOptions);
        void setSaveFileToLoadFrom(SaveFile f);

        void startGame();

        void addConnection(std::unique_ptr<Bridge> bridge, bool isAdmin);

        void updateServerInfo();

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
        void broadcastBordersExpanded(CityId cityID);

        void run(std::shared_ptr<ReaderWriterQueue<std::unique_ptr<Bridge>>> newConnections);

        bool hasPlayerWithCiv(const std::string &civID) const;

        void saveGame();
    };
}

#endif //RIPOSTE_SERVER_H
