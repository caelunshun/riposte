//
// Created by Caelum van Ispelen on 5/12/21.
//

#include "city.h"
#include <string>
#include <utility>

namespace rip {
    class City::impl {
    public:
        glm::uvec2 pos;
        std::string name;
        PlayerId owner;

        impl(glm::uvec2 pos, std::string name, PlayerId owner) : pos(pos), name(std::move(name)), owner(owner) {}
    };

    City::City(glm::uvec2 pos, std::string name, PlayerId owner) {
        _impl = std::make_unique<impl>(pos, std::move(name), owner);
    }

    City::~City() = default;

    City::City(City &&other) = default;

    glm::uvec2 City::getPos() const {
        return _impl->pos;
    }

    const std::string &City::getName() const {
        return _impl->name;
    }

    PlayerId City::getOwner() const {
        return _impl->owner;
    }

    void City::setName(std::string name) {
        _impl->name = std::move(name);
    }
}
