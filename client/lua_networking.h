//
// Created by Caelum van Ispelen on 7/30/21.
//

#ifndef RIPOSTE_LUA_NETWORKING_H
#define RIPOSTE_LUA_NETWORKING_H

#include <sol/sol.hpp>

namespace rip {
    void registerNetworkBindings(std::shared_ptr<sol::state> &lua);
}

#endif //RIPOSTE_LUA_NETWORKING_H
