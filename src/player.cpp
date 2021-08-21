//
// Created by Caelum van Ispelen on 5/12/21.
//

#include <unordered_set>
#include "player.h"
#include "game.h"
#include "city.h"
#include "tile.h"
#include "unit.h"
#include "event.h"
#include "server.h"
#include "saveload.h"
#include <riposte.pb.h>

namespace rip {
    Player::Player(std::string username, std::shared_ptr<CivKind> civ, Leader leader, uint32_t mapWidth, uint32_t mapHeight, const std::shared_ptr<TechTree> &techTree)
        : username(std::move(username)), visibilityMap(mapWidth, mapHeight), civ(civ), leader(std::move(leader)),
        techs(techTree) {
        for (const auto &startingTechName : civ->startingTechs) {
            techs.unlockTech(techTree->getTechs().at(startingTechName));
        }
    }

    Player::Player(const UpdatePlayer &packet, const Registry &registry, const std::shared_ptr<TechTree> &techTree,
                   const IdConverter &cityIDs,
                   const IdConverter &playerIDs,
                   uint32_t mapWidth, uint32_t mapHeight)
        : username(packet.username()), gold(packet.gold()), visibilityMap(mapWidth, mapHeight), techs(techTree) {
        if (packet.has_researchingtech()) {
            researchingTech = ResearchingTech(techTree->getTechs().at(packet.researchingtech().techid()));
            researchingTech->beakersAccumulated = packet.researchingtech().progress();
        }

        if (packet.capitalcityid() != 0) {
            capital = cityIDs.get(packet.capitalcityid());
        }

        for (const auto &techID : packet.unlockedtechids()) {
            techs.unlockTech(techTree->getTechs().at(techID));
        }

        sciencePercent = packet.beakerpercent();

        for (const auto opponentID : packet.atwarwithids()) {
            atWarWith.insert(playerIDs.get(opponentID));
        }

        civ = registry.getCiv(packet.civid());
        for (auto &leader : civ->leaders) {
            if (leader.name == packet.leadername()) {
                this->leader = leader;
                break;
            }
        }

        era = static_cast<Era>(static_cast<int>(packet.era()));
        if (packet.hasai()) {
            id = PlayerId(packet.id());
            enableAI();
        }

        for (const auto cityID : packet.cityids()) {
            cities.push_back(cityIDs.get(cityID));
        }

        score = packet.score();

        for (int x = 0; x < mapWidth; x++) {
            for (int y = 0; y < mapHeight; y++) {
                visibilityMap[glm::uvec2(x, y)] =
                        static_cast<Visibility>(static_cast<int>(
                                packet.visibility().visibility(x + y * mapWidth)));
            }
        }
    }

    void Player::onLoaded(Game &game) {
        cities = {};
        for (const auto &city : game.getCities()) {
            if (city.getOwner() == id) {
                cities.push_back(city.getID());
            }
        }

        recomputeRevenue(game);
        recomputeExpenses(game);
    }

    void Player::setID(PlayerId id) {
        this->id = id;
    }

    void Player::setCapital(CityId capital) {
        this->capital = capital;
    }

    void Player::enableAI() {
        ai = std::make_optional<AI>(id);
    }

    bool Player::hasAI() const {
        return ai.has_value();
    }

    PlayerId Player::getID() const {
        return id;
    }

    const std::string &Player::getUsername() const {
        return username;
    }

    const std::vector<CityId> &Player::getCities() const {
        return cities;
    }

    const VisibilityMap &Player::getVisibilityMap() const {
        return visibilityMap;
    }

    const CivKind &Player::getCiv() const {
        return *civ;
    }

    const Leader &Player::getLeader() const {
        return leader;
    }

    void Player::registerCity(CityId id) {
        cities.push_back(id);
    }

    void Player::removeCity(CityId id, Game &game) {
        cities.erase(std::remove(cities.begin(), cities.end(), id), cities.end());
        if (capital == id) {
            capital = {};
            if (!cities.empty()) {
                CityId biggestCity;
                int biggestPop;
                for (const auto cityID : getCities()) {
                    const auto pop = game.getCity(cityID).getPopulation();
                    if (pop > biggestPop) {
                        biggestCity = cityID;
                        biggestPop = pop;
                    }
                }
                capital = biggestCity;
                game.getCity(biggestCity).setCapital(game, true);
                game.getServer().markCityDirty(biggestCity);
            } else {
                die(game);
            }
        }
    }

    std::string Player::getNextCityName(const Game &game) {
        std::unordered_set<std::string> usedNames;
        for (const auto &city : game.getCities()) {
            usedNames.emplace(city.getName());
        }

        int numNews = 0;
        while (true) {
            for (const auto &name : civ->cities) {
                std::string prefix;
                for (int i = 0; i < numNews; i++) {
                    prefix += "New ";
                }
                auto prefixedName = prefix + name;
                if (usedNames.find(prefixedName) == usedNames.end()) {
                    return prefixedName;
                }
            }
            ++numNews;
        }
    }

    CityId Player::createCity(glm::uvec2 pos, Game &game) {
        auto name = getNextCityName(game);
        City city(pos, std::move(name), id);
        auto cityID = game.addCity(std::move(city));
        registerCity(cityID);

        if (cities.size() == 1) {
            game.getCity(cityID).setCapital(game, true);
        }

        game.getCity(cityID).updateWorkedTiles(game);

        auto &tile = game.getTile(pos);
        tile.setForested(false);

        recomputeVisibility(game);
        recomputeRevenue(game);
        recomputeExpenses(game);
        recomputeScore(game);

        return cityID;
    }

    CityId Player::getCapital() const {
        return capital;
    }

    void Player::recomputeVisibility(Game &game) {
        // Change Visible => Fogged
        for (int x = 0; x < game.getMapWidth(); x++) {
            for (int y = 0; y < game.getMapHeight(); y++) {
                glm::uvec2 pos(x, y);
                if (visibilityMap[pos] == Visibility::Visible) {
                    visibilityMap[pos] = Visibility::Fogged;
                }
            }
        }

        std::vector<glm::uvec2> sightPositions;

        // Cultural borders
        for (int x = 0; x < game.getMapWidth(); x++) {
            for (int y = 0; y < game.getMapHeight(); y++) {
                glm::uvec2 pos(x, y);
                if (game.getCultureMap().getTileOwner(pos) == id) {
                    sightPositions.push_back(pos);
                }
            }
        }

        // Units
        for (const auto &unit : game.getUnits()) {
            if (unit.getOwner() == id) {
                sightPositions.push_back(unit.getPos());
            }
        }

        for (const auto sightPos : sightPositions) {
            int sightDistance = 1;
            const auto &tile = game.getTile(sightPos);
            if (tile.isHilled() && !tile.isForested()) sightDistance = 2;

            for (int dx = -sightDistance; dx <= sightDistance; dx++) {
                for (int dy = -sightDistance; dy <= sightDistance; dy++) {
                    auto p = glm::ivec2(sightPos) + glm::ivec2(dx, dy);
                    if (p.x < 0 || p.y < 0 || p.x >= game.getMapWidth() || p.y >= game.getMapHeight()) {
                        continue;
                    }
                    glm::uvec2 pos(p);
                    visibilityMap[pos] = Visibility::Visible;
                }
            }
        }

        game.getServer().markPlayerVisibilityDirty(id);
    }

    bool Player::isDead() const {
        return dead;
    }

    void Player::onTurnEnd(Game &game) {
        if (isDead()) return;
        recomputeRevenue(game);
        recomputeExpenses(game);
        doEconomyTurn(game);
        if (ai.has_value()) {
            ai->doTurn(game);
        }
        recomputeScore(game);
        game.getServer().markPlayerDirty(id);
    }

    const PlayerTechs &Player::getTechs() const {
        return techs;
    }

    void Player::setUsername(std::string username) {
        this->username = std::move(username);
    }

    void Player::recomputeRevenue(Game &game) {
        baseRevenue = 0;
        beakerRevenue = 0;
        goldRevenue = 0;
        for (const auto cityID : cities) {
            auto &city = game.getCity(cityID);

            const auto cityBaseCommerce = city.getGoldProduced(game);

            // Split base revenue into gold and beaker revenue
            // based on beakerPercent.

            int cityBeakers = percentOf(cityBaseCommerce, sciencePercent);
            cityBeakers += city.getBuildingEffects().bonusBeakers;
            cityBeakers += percentOf(cityBeakers, city.getBuildingEffects().bonusBeakerPercent);

            int cityGold = cityBaseCommerce - percentOf(cityBaseCommerce, sciencePercent);
            cityGold += city.getBuildingEffects().bonusGold;
            cityGold += percentOf(cityGold, city.getBuildingEffects().bonusGoldPercent);

            baseRevenue += cityBaseCommerce;
            beakerRevenue += cityBeakers;
            goldRevenue += cityGold;
        }
    }

    void Player::recomputeExpenses(Game &game) {
        // City maintenence expenses
        expenses = 0;
        for (const auto cityID : cities) {
            const auto &city = game.getCity(cityID);
            expenses += city.getMaintenanceCost(game);
        }

        // Unit upkeep. Each unit costs 1 GPT.
        // Units outside of our borders contribute an additional
        // 1/2 GPT each.
        // We get a handicap of 10 free units.
        const int handicap = 10;
        int unitExpensesDoubled = -handicap * 2;
        for (const auto &unit : game.getUnits()) {
            if (unit.getOwner() != id) continue;

            unitExpensesDoubled += 2;
            if (game.getCultureMap().getTileOwner(unit.getPos()) != id) {
                unitExpensesDoubled += 1;
            }
        }

        // heheh
        if (ai.has_value()) {
            unitExpensesDoubled = 0;
        }

        if (unitExpensesDoubled > 0) {
            expenses += unitExpensesDoubled / 2;
        }

        // Apply inflation. Inflation is computed as
        // a percent of all other expenses, where
        // that percent increases linearily starting at turn 100.
        double inflationPercent = game.getTurn() < 100 ? 0 : (game.getTurn() - 100.0) * 1.0 / 400.0;
        int inflation = static_cast<int>(expenses * inflationPercent);
        expenses += inflation;

        game.getServer().markPlayerDirty(id);
    }

    int Player::getBaseRevenue() const {
        return baseRevenue;
    }

    int Player::getGoldRevenue() const {
        return goldRevenue;
    }

    int Player::getBeakerRevenue() const {
        return beakerRevenue;
    }

    int Player::getNetGold() const {
        return getGoldRevenue() - getExpenses();
    }

    int Player::getExpenses() const {
        return expenses;
    }

    int Player::getGold() const {
        return gold;
    }

    void Player::doEconomyTurn(Game &game) {
        if (researchingTech.has_value()) {
            researchingTech->beakersAccumulated += getBeakerRevenue();
            updateResearch(game);
        }

        // Lower beaker percent if needed.
        while (gold + getNetGold() < 0 && sciencePercent >= 10) {
            sciencePercent -= 10;
            recomputeRevenue(game);
            // TODO: what happens after sciencePercent==0 and gold==0?
        }

        gold += getNetGold();
    }

    void Player::updateResearch(Game &game) {
        if (researchingTech.has_value() && researchingTech->isFinished()) {
            techs.unlockTech(researchingTech->tech);

            if (researchingTech->tech->era > era) {
                era = researchingTech->tech->era;
            }

            researchingTech = {};
        }
    }

    const std::optional<ResearchingTech> &Player::getResearchingTech() const {
        return researchingTech;
    }

    void Player::setResearchingTech(const std::shared_ptr<Tech> &tech) {
        researchingTech = std::make_optional(tech);
    }

    bool ResearchingTech::isFinished() const {
        return beakersAccumulated >= tech->cost;
    }

    ResearchingTech::ResearchingTech(std::shared_ptr<Tech> tech) : tech(std::move(tech)) {}

    int ResearchingTech::estimateCompletionTurns(int beakersPerTurn) const {
        if (beakersPerTurn == 0) return tech->cost + 1;
        return (tech->cost - beakersAccumulated + beakersPerTurn - 1) / beakersPerTurn;
    }

    int Player::getSciencePercent() const {
        return sciencePercent;
    }

    void Player::setSciencePercent(int percent, Game &game) {
        sciencePercent = percent;
        if (sciencePercent > 100) {
            sciencePercent = 100;
        } else if (sciencePercent < 0) {
            sciencePercent = 0;
        }
        recomputeRevenue(game);
    }

    int Player::getTotalPopulation(const Game &game) {
        int sum = 0;
        for (const auto cityID : cities) sum += game.getCity(cityID).getPopulation();
        return sum;
    }

    void Player::recomputeScore(Game &game) {
        score = 0;

        // population
        score += static_cast<int>(5000.0 * (getTotalPopulation(game) / 400.0));

        // techs
        score += static_cast<int>(2000.0 * (getTechs().getUnlockedTechs().size() / 200.0));
    }

    int Player::getScore() const {
        return score;
    }

    bool Player::isAtWarWith(PlayerId player) const {
        return atWarWith.contains(player);
    }

    void Player::declareWarOn(PlayerId player, Game &game) {
        if (player == id) return;
        if (atWarWith.insert(player).second) {
            onWarDeclared(player, game);
            auto &other = game.getPlayer(player);
            other.onWarDeclared(id, game);

            game.onWarDeclared(*this, other);
            game.addEvent(std::make_unique<WarDeclaredEvent>(leader.name, other.leader.name));

            game.getServer().broadcastWarDeclared(id, player);
            game.getServer().markPlayerDirty(id);
            game.getServer().markPlayerDirty(player);
        }
    }

    void Player::onWarDeclared(PlayerId withPlayer, Game &game) {
        atWarWith.insert(withPlayer);
        expelUnitsInTerritoryOf(withPlayer, game);
    }

    void Player::expelUnitsInTerritoryOf(PlayerId player, Game &game) {
        for (auto &unit : game.getUnits()) {
            if (unit.getOwner() != id) continue;

            if (game.getCultureMap().getTileOwner(unit.getPos()) == player) {
                auto capitalPos = game.getCity(capital).getPos();
                unit.teleportTo(capitalPos, game);
            }
        }
    }

    void Player::die(Game &game) {
        // Kill all units. They can't live without cities
        // to support them.
        for (const auto &unit : game.getUnits()) {
            if (unit.getOwner() == id) {
                game.deferKillUnit(unit.getID());
            }
        }

        game.addEvent(std::make_unique<PlayerKilledEvent>(civ->name));

        dead = true;

        game.getServer().markPlayerDirty(id);
    }

    Era Player::getEra() const {
        return era;
    }
}
