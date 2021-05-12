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

        impl(glm::uvec2 pos, std::string name) : pos(pos), name(std::move(name)) {}
    };

    City::City(glm::uvec2 pos, std::string name) {
        _impl = std::make_unique<impl>(pos, std::move(name));
    }

    City::~City() = default;

    City::City(City &&other) = default;

    glm::uvec2 City::getPos() const {
        return _impl->pos;
    }

    const std::string &City::getName() const {
        return _impl->name;
    }

    void City::setName(std::string name) {
        _impl->name = std::move(name);
    }
}
