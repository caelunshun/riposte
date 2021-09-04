//
// Created by Caelum van Ispelen on 8/17/21.
//

#ifndef RIPOSTE_PROTOCOL_H
#define RIPOSTE_PROTOCOL_H

#include <riposte.pb.h>

#include "game.h"
#include "saveload.h"

using namespace rip::proto;

namespace rip {
    class WorkerTask;
    class CultureValue;
    class Culture;

    UpdateGlobalData getUpdateGlobalDataPacket(Game &game, PlayerId thePlayerID);
    void writeYield(const Yield &yield, proto::Yield &protoYield);
    void writeCultureValues(const Culture &culture, proto::CultureValues &proto);
    void setTile(const Game &game, PlayerId player, glm::uvec2 pos, const Tile &tile, proto::Tile &protoTile);
    UpdateMap getUpdateMapPacket(Game &game, PlayerId playerID);
    UpdateVisibility getUpdateVisibilityPacket(Game &game, PlayerId playerID);
    UpdateTile getUpdateTilePacket(Game &game, glm::uvec2 pos, PlayerId player);
    void writePath(const Path &path, proto::Path &protoPath);
    void writeWorkerTask(const rip::WorkerTask &task, proto::WorkerTask &protoTask);
    UpdateUnit getUpdateUnitPacket(Game &game, Unit &unit);
    void writeBuildTask(const BuildTask &task, proto::BuildTask &protoTask);
    UpdateCity getUpdateCityPacket(Game &game, City &city);
    UpdatePlayer getUpdatePlayerPacket(Game &game, Player &player);

    Culture getCultureFromProto(const CultureValues &proto, const IdConverter &playerIDs);
}

#endif //RIPOSTE_PROTOCOL_H
