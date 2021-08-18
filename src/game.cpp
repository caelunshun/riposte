//
// Created by Caelum van Ispelen on 5/11/21.
//

#include "server.h"
#include "game.h"
#include "culture.h"
#include "city.h"
#include "player.h"
#include "unit.h"
#include "trade.h"
#include "registry.h"
#include "tile.h"
#include "stack.h"
#include "event.h"
#include "script.h"

#include <absl/container/flat_hash_map.h>

namespace rip {
    struct Game::_impl {
        std::vector<Tile> theMap;
        uint32_t mapWidth;
        uint32_t mapHeight;

        slot_map<City> cities;
        slot_map<Player> players;
        slot_map<Unit> units;
        slot_map<Stack> stacks;

        absl::flat_hash_map<glm::uvec2, std::vector<StackId>, PosHash> stacksByPos;

        // The human player.
        PlayerId thePlayer;

        Cursor cursor;
        View view;

        std::shared_ptr<Registry> registry;

        std::vector<UnitId> unitKillQueue;

        float dt = 0;
        float lastFrameTime = 0;

        int turn = 0;

        bool cheatMode = false;

        std::vector<std::optional<CityId>> workedTiles;

        CultureMap cultureMap;

        TradeRoutes tradeRoutes;

        std::vector<std::unique_ptr<Event>> events;

        bool inTurnEnd = false;

        std::shared_ptr<ScriptEngine> scriptEngine;

        Server *server;

        std::shared_ptr<TechTree> techTree;

        _impl(uint32_t mapWidth, uint32_t mapHeight, std::shared_ptr<Registry> registry, std::shared_ptr<TechTree> techTree)
        : theMap(static_cast<size_t>(mapWidth) * mapHeight),
        workedTiles(static_cast<size_t>(mapWidth) * mapHeight),
        mapWidth(mapWidth),
        mapHeight(mapHeight),
        registry(std::move(registry)),
        cultureMap(mapWidth, mapHeight),
        cursor(),
        techTree(techTree) {}
    };

    void Game::advanceTurn() {
        impl->inTurnEnd = true;
        for (auto &unit : impl->units) {
            unit.onTurnEnd(*this);
        }

        impl->tradeRoutes.updateResources(*this);

        for (auto &city : impl->cities) {
            city.onTurnEnd(*this);
        }

        for (auto &player : impl->players) {
            player.onTurnEnd(*this);
        }

        impl->cultureMap.onTurnEnd(*this);

        ++(impl->turn);
        impl->inTurnEnd = false;
    }

    std::optional<UnitId> Game::getNextUnitToMove() {
        for (auto &unit : impl->units) {
            if (unit.getMovementLeft() != 0 && unit.getOwner() == impl->thePlayer && !unit.isFortified()) {
                if (unit.hasPath()) {
                    unit.moveAlongCurrentPath(*this, false);
                }
                if (unit.getMovementLeft() != 0 && !unit.hasPath()) {
                    return std::make_optional<UnitId>(unit.getID());
                }
            }
        }

        return std::optional<UnitId>();
    }

    Game::Game(uint32_t mapWidth, uint32_t mapHeight, std::shared_ptr<Registry> registry, std::shared_ptr<TechTree> techTree)
    : impl(std::make_unique<Game::_impl>(mapWidth, mapHeight, registry, techTree)) {

    }

    uint32_t Game::getMapWidth() const {
        return impl->mapWidth;
    }

    uint32_t Game::getMapHeight() const {
        return impl->mapHeight;
    }

    bool Game::containsTile(glm::uvec2 pos) const {
        return (pos.x >= 0 && pos.y >= 0 && pos.x < getMapWidth() && pos.y < getMapHeight());
    }

    Tile &Game::getTile(glm::uvec2 pos) {
        return impl->theMap[pos.x + pos.y * getMapWidth()];
    }

    const Tile &Game::getTile(glm::uvec2 pos) const {
        return impl->theMap[pos.x + pos.y * getMapWidth()];
    }

    void Game::setTile(glm::uvec2 pos, Tile tile) {
        impl->theMap[pos.x + pos.y * getMapWidth()] = std::move(tile);
    }

    Cursor &Game::getCursor() {
        return impl->cursor;
    }

    const Cursor &Game::getCursor() const {
        return impl->cursor;
    }

    View &Game::getView() {
        return impl->view;
    }

    const View &Game::getView() const {
        return impl->view;
    }

    float Game::getDeltaTime() const {
        return impl->dt;
    }

    glm::vec2 Game::getMapOrigin() const {
        return getView().getMapCenter() - (getCursor().getWindowSize() / 2.0f);
    }

    glm::vec2 Game::getScreenOffset(glm::uvec2 tile) const {
        return glm::vec2(tile) * 100.0f - getMapOrigin();
    }

    glm::uvec2 Game::getPosFromScreenOffset(glm::vec2 offset) const {
        auto centered = offset - getCursor().getWindowSize() / 2.0f;
        centered /= getView().getZoomFactor();
        auto translated = centered + getView().getMapCenter();
        auto scaled = translated / 100.0f;
        return glm::uvec2(static_cast<uint32_t>(floor(scaled.x)), static_cast<uint32_t>(floor(scaled.y)));
    }

    void Game::tick() {
        for (const auto unitID : impl->unitKillQueue) {
            killUnit(unitID);
        }
        impl->unitKillQueue.clear();
    }

    const slot_map<City> &Game::getCities() const {
        return impl->cities;
    }

    slot_map<City> &Game::getCities() {
        return impl->cities;
    }

    CityId Game::addCity(City city) {
        auto id = getCities().insert(std::move(city));
        getCity(id).setID(id);
        getCity(id).onCreated(*this);
        getServer().markCityDirty(id);
        return id;
    }

    City *Game::getCityAtLocation(glm::uvec2 location) {
        for (auto &city : getCities()) {
            if (city.getPos() == location) {
                return &city;
            }
        }
        return nullptr;
    }

    const City *Game::getCityAtLocation(glm::uvec2 location) const {
        for (const auto &city : getCities()) {
            if (city.getPos() == location) {
                return &city;
            }
        }
        return nullptr;
    }

    City &Game::getCity(CityId id) {
        return getCities()[id];
    }

    const City &Game::getCity(CityId id) const {
        return getCities()[id];
    }

    Player &Game::getPlayer(PlayerId id) {
        return getPlayers()[id];
    }

    const Player &Game::getPlayer(PlayerId id) const {
        return impl->players[id];
    }

    Player &Game::getThePlayer() {
        return getPlayer(impl->thePlayer);
    }

    const Player &Game::getThePlayer() const {
        return getPlayer(impl->thePlayer);
    }

    PlayerId Game::getThePlayerID() const {
        return impl->thePlayer;
    }

    size_t Game::getNumPlayers() const {
        return impl->players.size();
    }

    void Game::setThePlayerID(PlayerId id) {
        impl->thePlayer = id;
    }

    PlayerId Game::addPlayer(Player player) {
        auto id = impl->players.insert(std::move(player));
        getPlayer(id).setID(id);
        return id;
    }

    slot_map<Player> &Game::getPlayers() {
        return impl->players;
    }

    const Registry &Game::getRegistry() const {
        return *impl->registry;
    }

    UnitId Game::addUnit(Unit unit) {
        auto id = impl->units.insert(std::move(unit));
        auto &u = getUnit(id);
        u.setID(id);
        onUnitMoved(id, {}, u.getPos());
        getServer().markUnitDirty(id);
        return id;
    }

    const Unit &Game::getUnit(UnitId id) const {
        return getUnits()[id];
    }

    Unit &Game::getUnit(UnitId id) {
        return getUnits()[id];
    }

    void Game::killUnit(UnitId id) {
        if (impl->units.contains(id)) {
            auto stackID = getUnit(id).getStack(*this);
            auto &stack = getStack(stackID);
            stack.removeUnit(id);
            if (stack.getUnits().empty()) {
                deleteStack(stackID);
            }
            impl->units.erase(id);
        }

        getServer().broadcastUnitDeath(id);
    }

    void Game::deferKillUnit(UnitId id) {
        impl->unitKillQueue.push_back(id);
    }

    slot_map<Unit> &Game::getUnits() {
        return impl->units;
    }

    const slot_map<Unit> &Game::getUnits() const {
        return impl->units;
    }

    void Game::setTurn(int turn) {
        impl->turn = turn;
    }

    int Game::getTurn() const {
        return impl->turn;
    }

    void Game::toggleCheatMode() {
        impl->cheatMode = !(impl->cheatMode);
    }

    bool Game::isCheatMode() const {
        return impl->cheatMode;
    }

    std::optional<CityId> Game::isTileWorked(glm::uvec2 pos) const {
        return impl->workedTiles[pos.x + pos.y * getMapWidth()];
    }

    void Game::setTileWorked(glm::uvec2 pos, bool worked, CityId worker) {
        std::optional<CityId> val;
        if (worked) val = worker;
        impl->workedTiles[pos.x + pos.y * getMapWidth()] = val;
        getServer().markTileDirty(pos);
    }

    CultureMap &Game::getCultureMap() {
        return impl->cultureMap;
    }

    const CultureMap &Game::getCultureMap() const {
        return impl->cultureMap;
    }

    TradeRoutes &Game::getTradeRoutes() {
        return impl->tradeRoutes;
    }

    const TradeRoutes &Game::getTradeRoutes() const {
        return impl->tradeRoutes;
    }

    void Game::onUnitMoved(UnitId unitID, std::optional<glm::uvec2> oldPos, glm::uvec2 newPos) {
        const auto &unit = getUnit(unitID);

        if (oldPos.has_value()) {
            auto oldStackID = *getStackByKey(unit.getOwner(), *oldPos);
            auto &oldStack = getStack(oldStackID);
            oldStack.removeUnit(unitID);

            if (oldStack.getUnits().empty()) {
                deleteStack(oldStackID);
            }
        }

        auto &newStack = getStack(createStack(unit.getOwner(), newPos));
        newStack.addUnit(unitID);
    }

    StackId Game::createStack(PlayerId owner, glm::uvec2 pos) {
        const auto existing = getStackByKey(owner, pos);
        if (existing.has_value()) {
            return *existing;
        }

        auto id = impl->stacks.insert(Stack(owner, pos));
        if (!impl->stacksByPos.contains(pos)) {
            impl->stacksByPos[pos] = {};
        }
        impl->stacksByPos[pos].push_back(id);
        return id;
    }

    void Game::deleteStack(StackId id) {
        // Remove the stack from the stacksByPos index.
        auto &stack = getStack(id);
        auto &inPos = impl->stacksByPos[stack.getPos()];
        inPos.erase(std::find(inPos.begin(), inPos.end(), id));
        if (inPos.empty()) {
            impl->stacksByPos.erase(stack.getPos());
        }

        impl->stacks.erase(id);
    }

    std::optional<StackId> Game::getStackByKey(PlayerId owner, glm::uvec2 pos) const {
        const auto &inPos = getStacksAtPos(pos);
        for (const auto id : inPos) {
            if (getStack(id).getOwner() == owner) {
                assert(impl->stacks.id_is_valid(id));
                return id;
            }
        }
        return {};
    }

    static auto emptyStackVec = std::vector<StackId>();

    const std::vector<StackId> &Game::getStacksAtPos(glm::uvec2 pos) const {
        if (impl->stacksByPos.contains(pos)) {
            return impl->stacksByPos[pos];
        } else {
            return emptyStackVec;
        }
    }

    const Stack &Game::getStack(StackId id) const {
        assert(impl->stacks.id_is_valid(id));
        return impl->stacks[id];
    }

    Stack &Game::getStack(StackId id) {
        assert(impl->stacks.id_is_valid(id));
        return impl->stacks[id];
    }

    slot_map<Stack> &Game::getStacks() {
        return impl->stacks;
    }

    void Game::addEvent(std::unique_ptr<Event> event) {
        getEvents().push_back(std::move(event));
    }

    std::vector<std::unique_ptr<Event>> &Game::getEvents() {
        return impl->events;
    }

    void Game::setScriptEngine(std::shared_ptr<ScriptEngine> engine) {
        impl->scriptEngine = engine;
    }

    ScriptEngine &Game::getScriptEngine() {
        return *impl->scriptEngine;
    }

    Server &Game::getServer() {
        return *impl->server;
    }

    void Game::setServer(Server *server) {
        impl->server = server;
    }

    const TechTree &Game::getTechTree() const {
        return *impl->techTree;
    }

    // EVENTS

    void Game::onWarDeclared(Player &declarer, Player &declared) {

    }

    void Game::onDialogueOpened(Player &with) {

    }

    Game::~Game() = default;

    Game::Game(Game &&other) = default;
}
