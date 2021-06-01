//
// Created by Caelum van Ispelen on 6/1/21.
//

#include <sol/sol.hpp>
#include "../game.h"
#include "../player.h"
#include "../city.h"

namespace rip {
    void bindGame(sol::state &lua) {
        auto cursor_type = lua.new_usertype<Cursor>("Cursor");
        cursor_type["getWindowSize"] = &Cursor::getWindowSize;
        cursor_type["getPos"] = &Cursor::getPos;

        auto game_type = lua.new_usertype<Game>("Game");
        game_type["getThePlayer"] = [=](Game &game) {
            return &game.getThePlayer();
        };
        game_type["getCursor"] = [=](Game &game) {
            return &game.getCursor();
        };
        game_type["getCityAtLocation"] = [=](Game &game, glm::uvec2 pos) {
            auto *city = game.getCityAtLocation(pos);
            return city;
        };
        game_type["getScreenOffset"] = &Game::getScreenOffset;
        game_type["getPosFromScreenOffset"] = &Game::getPosFromScreenOffset;
    }
}
