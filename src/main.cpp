#include "lobby.h"
#include "assets.h"
#include "registry.h"

using rip::proto::LobbySlot;

int main(int argc, char **argv) {
    const auto *hostUUIDstr = std::getenv("RIPOSTE_HOST_UUID");
    if (!hostUUIDstr) {
        std::cerr << "RIPOSTE_HOST_UUID must be set" << std::endl;
        return 1;
    }

    auto assets = std::make_shared<rip::Assets>();
    auto registry = std::make_shared<rip::Registry>();
    assets->addLoader("civ", std::make_unique<rip::CivLoader>(registry));
    assets->loadAssetsDir("assets", true);

    auto networkCtx = std::make_shared<rip::NetworkingContext>();

    auto lobbyServer = std::make_shared<rip::LobbyServer>(networkCtx, registry);

    rip::proto::UUID hostUUID;
    hostUUID.set_uuid(hostUUIDstr);

    LobbySlot initialSlot;
    initialSlot.mutable_owneruuid()->CopyFrom(hostUUID);
    lobbyServer->addSlot(std::move(initialSlot));

    auto hostConnection = networkCtx->connectStdio();
    lobbyServer->addConnection(std::move(hostConnection), hostUUID, true);

    lobbyServer->run();

    return 0;
}
