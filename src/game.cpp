//
// Created by Caelum van Ispelen on 5/11/21.
//

#include "game.h"
#include "culture.h"
#include "city.h"
#include "player.h"
#include "unit.h"
#include "trade.h"
#include "registry.h"
#include "tile.h"
#include "combat.h"
#include "stack.h"

#include <absl/container/flat_hash_map.h>

namespace rip {
    struct Game::_impl {
        std::vector<Tile> theMap;
        uint32_t mapWidth;
        uint32_t mapHeight;

        rea::versioned_slot_map<City> cities;
        rea::versioned_slot_map<Player> players;
        rea::versioned_slot_map<Unit> units;
        rea::versioned_slot_map<Stack> stacks;

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

        std::vector<bool> workedTiles;

        CultureMap cultureMap;

        TradeRoutes tradeRoutes;

        std::vector<Combat> ongoingCombats;

        _impl(uint32_t mapWidth, uint32_t mapHeight, std::shared_ptr<Registry> registry)
        : theMap(static_cast<size_t>(mapWidth) * mapHeight),
        workedTiles(static_cast<size_t>(mapWidth) * mapHeight),
        mapWidth(mapWidth),
        mapHeight(mapHeight),
        registry(std::move(registry)),
        cultureMap(mapWidth, mapHeight),
        cursor() {}
    };

    void Game::advanceTurn() {
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

        for (auto &combat : impl->ongoingCombats) {
            combat.finish(*this);
        }
        impl->ongoingCombats.clear();

        ++(impl->turn);
    }

    std::optional<UnitId> Game::getNextUnitToMove() {
        for (auto &unit : impl->units) {
            if (unit.getMovementLeft() != 0 && unit.getOwner() == impl->thePlayer && !unit.isFortified()) {
                if (unit.hasPath()) {
                    unit.moveAlongCurrentPath(*this);
                } else {
                    return std::make_optional<UnitId>(unit.getID());
                }
            }
        }

        return std::optional<UnitId>();
    }

    Game::Game(uint32_t mapWidth, uint32_t mapHeight, std::shared_ptr<Registry> registry)
    : impl(std::make_unique<Game::_impl>(mapWidth, mapHeight, registry)) {

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
        auto translated = offset + getMapOrigin();
        auto scaled = translated / 100.0f;
        return glm::uvec2(static_cast<uint32_t>(floor(scaled.x)), static_cast<uint32_t>(floor(scaled.y)));
    }

    void Game::tick(GLFWwindow *window, bool hudHasFocus) {
        impl->dt = glfwGetTime() - impl->lastFrameTime;
        impl->lastFrameTime = glfwGetTime();

        getCursor().tick(window);
        getView().tick(impl->dt, getCursor(), hudHasFocus);

        for (const auto unitID : impl->unitKillQueue) {
            killUnit(unitID);
        }
        impl->unitKillQueue.clear();

        for (int i = impl->ongoingCombats.size() - 1; i >= 0; i--) {
            auto &combat = impl->ongoingCombats[i];

            if (!getUnits().id_is_valid(combat.getAttacker()) || !getUnits().id_is_valid(combat.getDefender())) {
                impl->ongoingCombats.erase(impl->ongoingCombats.begin() + i);
                continue;
            }

            combat.advance(*this, getDeltaTime());
            if (combat.isFinished()) {
                combat.finish(*this);
                impl->ongoingCombats.erase(impl->ongoingCombats.begin() + i);
            }
        }
    }

    const rea::versioned_slot_map<City> &Game::getCities() const {
        return impl->cities;
    }

    rea::versioned_slot_map<City> &Game::getCities() {
        return impl->cities;
    }

    CityId Game::addCity(City city) {
        auto id = getCities().insert(std::move(city)).second;
        getCity(id).setID(id);
        getCity(id).onCreated(*this);
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
        for (auto &city : getCities()) {
            if (city.getPos() == location) {
                return &city;
            }
        }
        return nullptr;
    }

    City &Game::getCity(CityId id) {
        return getCities().id_value(id);
    }

    const City &Game::getCity(CityId id) const {
        return getCities().id_value(id);
    }

    Player &Game::getPlayer(PlayerId id) {
        return getPlayers().id_value(id);
    }

    const Player &Game::getPlayer(PlayerId id) const {
        return impl->players.id_value(id);
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
        return impl->players.insert(std::move(player)).second;
    }

    rea::versioned_slot_map<Player> &Game::getPlayers() {
        return impl->players;
    }

    const Registry &Game::getRegistry() const {
        return *impl->registry;
    }

    UnitId Game::addUnit(Unit unit) {
        auto id = impl->units.insert(std::move(unit)).second;
        auto &u = getUnit(id);
        u.setID(id);
        onUnitMoved(id, {}, u.getPos());
        return id;
    }

    const Unit &Game::getUnit(UnitId id) const {
        return getUnits().id_value(id);
    }

    Unit &Game::getUnit(UnitId id) {
        return getUnits().id_value(id);
    }

    void Game::killUnit(UnitId id) {
        if (impl->units.id_is_valid(id)) {
            auto stackID = getUnit(id).getStack(*this);
            auto &stack = getStack(stackID);
            stack.removeUnit(id);
            if (stack.getUnits().empty()) {
                deleteStack(stackID);
            }
            impl->units.erase(id);
        }
    }

    void Game::deferKillUnit(UnitId id) {
        impl->unitKillQueue.push_back(id);
    }

    rea::versioned_slot_map<Unit> &Game::getUnits() {
        return impl->units;
    }

    const rea::versioned_slot_map<Unit> &Game::getUnits() const {
        return impl->units;
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

    bool Game::isTileWorked(glm::uvec2 pos) const {
        return impl->workedTiles[pos.x + pos.y * getMapWidth()];
    }

    void Game::setTileWorked(glm::uvec2 pos, bool worked) {
        impl->workedTiles[pos.x + pos.y * getMapWidth()] = worked;
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

    Era Game::getEra() const {
        const auto turn = impl->turn;
        if (turn < 50) {
            return Era::Ancient;
        } else if (turn < 150) {
            return Era::Classical;
        } else if (turn < 250) {
            return Era::Medieval;
        } else if (turn < 300) {
            return Era::Renaissance;
        } else if (turn < 400) {
            return Era::Industrial;
        } else if (turn < 450) {
            return Era::Modern;
        } else {
            return Era::Future;
        }
    }

    void Game::addCombat(Combat &combat) {
        impl->ongoingCombats.push_back(combat);
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

        auto id = impl->stacks.insert(Stack(owner, pos)).second;
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
        return impl->stacks.id_value(id);
    }

    Stack &Game::getStack(StackId id) {
        assert(impl->stacks.id_is_valid(id));
        return impl->stacks.id_value(id);
    }

    rea::versioned_slot_map<Stack> &Game::getStacks() {
        return impl->stacks;
    }

    Game::~Game() = default;

    Game::Game(Game &&other) = default;
}
