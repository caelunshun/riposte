#include "lobby.h"

using rip::proto::LobbySlot;

int main(int argc, char **argv) {
    auto networkCtx = std::make_shared<rip::NetworkingContext>();

    rip::LobbyServer lobbyServer(networkCtx);

    rip::proto::UUID hostUUID;
    hostUUID.set_uuid("temp");

    LobbySlot initialSlot;
    initialSlot.mutable_owneruuid()->CopyFrom(hostUUID);
    lobbyServer.addSlot(std::move(initialSlot));

    auto hostConnection = networkCtx->connectStdio();
    lobbyServer.addConnection(std::move(hostConnection), hostUUID, true);

    lobbyServer.run();

    return 0;
}
