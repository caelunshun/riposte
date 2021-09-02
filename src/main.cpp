#include "lobby.h"
#include "assets.h"
#include "registry.h"
#include "mapgen.h"
#include "server.h"

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
    assets->addLoader("tech", std::make_unique<rip::TechLoader>());
    assets->addLoader("unit", std::make_unique<rip::UnitLoader>(registry));
    assets->addLoader("resource", std::make_unique<rip::ResourceLoader>(registry));
    assets->addLoader("building", std::make_unique<rip::BuildingLoader>(registry));
    assets->loadAssetsDir("assets", true);
    auto techTree = std::make_shared<rip::TechTree>(*assets, *registry);

    auto networkCtx = std::make_shared<rip::NetworkingContext>();

    auto lobbyServer = std::make_shared<rip::LobbyServer>(networkCtx, registry);

    rip::proto::UUID hostUUID;
    hostUUID.set_uuid(hostUUIDstr);

    LobbySlot initialSlot;
    initialSlot.mutable_owneruuid()->CopyFrom(hostUUID);
    lobbyServer->addSlot(std::move(initialSlot));

    auto hostConnection = networkCtx->connectStdio();
    lobbyServer->addConnection(std::move(hostConnection), hostUUID, true);

    const auto lobbyResult = lobbyServer->run();

    if (lobbyResult == rip::LobbyResult::Exit) {
        return 0;
    }

    // TEMP
    rip::mapgen::MapgenSettings settings;
    settings.set_mapwidth(80);
    settings.set_mapheight(48);
    settings.mutable_continents()->set_numcontinents(rip::mapgen::NumContinents::Two);

    rip::Server server(networkCtx, "Test Game", "singleplayer");

    // Start the game.
    rip::MapGenerator mapgen;
    auto mapgenResult = mapgen.generate(lobbyServer->getSlots(), settings, registry, techTree, &server);
    auto playerIDMapping = mapgenResult.second;

    server.game = std::make_unique<rip::Game>(std::move(mapgenResult.first));

    for (const auto &lobbySlot : lobbyServer->getSlots()) {
        if (!lobbySlot.isai() && playerIDMapping.find(lobbySlot.id()) != playerIDMapping.end()) {
            rip::PlayerId player = playerIDMapping[lobbySlot.id()];
            rip::ConnectionHandle handle = std::move(lobbyServer->getConnectionForSlot(lobbySlot.id()));
            server.addConnection(std::move(handle), player, lobbySlot.isadmin());
        }
    }

    server.run({});

    return 0;
}
