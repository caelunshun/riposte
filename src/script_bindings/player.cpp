//
// Created by Caelum van Ispelen on 6/1/21.
//

#include <sol/sol.hpp>
#include "../player.h"

namespace rip {
    void bindPlayer(sol::state &lua, std::shared_ptr<Game*> game) {
        auto player_type = lua.new_usertype<Player>("Player");
        player_type["getLeader"] = &Player::getLeader;
        player_type["getName"] = &Player::getUsername;
        player_type["hasAI"] = &Player::hasAI;
        player_type["getCiv"] = &Player::getCiv;
        player_type["declareWarOn"] = [=] (Player &self, Player &opponent) {
            self.declareWarOn(opponent.getID(), **game);
        };
        player_type["isAtWarWith"] = [=] (Player &self, Player &opponent) {
            return self.isAtWarWith(opponent.getID());
        };
        player_type["isDead"] = &Player::isDead;
        player_type["getBaseRevenue"] = &Player::getBaseRevenue;
        player_type["getGoldRevenue"] = &Player::getGoldRevenue;
        player_type["getBeakerExpenses"] = &Player::getExpenses;
        player_type["getNetGold"] = &Player::getNetGold;
        player_type["getGold"] = &Player::getGold;
        player_type["getSciencePercent"] = &Player::getSciencePercent;
        player_type["setSciencePercent"] = [=] (Player &self, int sciencePercent) {
            self.setSciencePercent(sciencePercent, **game);
        };
        player_type["getScore"] = &Player::getScore;
        player_type["recomputeScore"] = [=] (Player &self) {
            self.recomputeScore(**game);
        };
        player_type["die"] = [=] (Player &self) {
            self.die(**game);
        };
    }
}
