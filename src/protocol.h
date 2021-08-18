//
// Created by Caelum van Ispelen on 8/17/21.
//

#ifndef RIPOSTE_PROTOCOL_H
#define RIPOSTE_PROTOCOL_H

#include <riposte.pb.h>

#include "game.h"
#include "saveload.h"

namespace rip {
    class WorkerTask;
    class CultureValue;
    class Culture;

    void setPlayerInfo(const Player &player, PlayerInfo &playerInfo);
    UpdateGlobalData getUpdateGlobalDataPacket(Game &game, PlayerId thePlayerID);
    void writeYield(const Yield &yield, ::Yield &protoYield);
    void writeCultureValues(const Culture &culture, ::CultureValues &proto);
    void setTile(const Game &game, PlayerId player, glm::uvec2 pos, const Tile &tile, ::Tile &protoTile);
    UpdateMap getUpdateMapPacket(Game &game, PlayerId playerID);
    UpdateVisibility getUpdateVisibilityPacket(Game &game, PlayerId playerID);
    UpdateTile getUpdateTilePacket(Game &game, glm::uvec2 pos, PlayerId player);
    void writePath(const Path &path, ::Path &protoPath);
    void writeWorkerTask(const rip::WorkerTask &task, ::WorkerTask &protoTask);
    UpdateUnit getUpdateUnitPacket(Game &game, Unit &unit);
    void writeBuildTask(const BuildTask &task, ::BuildTask &protoTask);
    UpdateCity getUpdateCityPacket(Game &game, City &city);
    UpdatePlayer getUpdatePlayerPacket(Game &game, Player &player);

    Culture getCultureFromProto(const CultureValues &proto, const IdConverter &playerIDs);
}

#endif //RIPOSTE_PROTOCOL_H
