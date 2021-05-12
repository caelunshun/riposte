//
// Created by Caelum van Ispelen on 5/12/21.
//

#ifndef RIPOSTE_CITY_H
#define RIPOSTE_CITY_H

#include <memory>
#include <glm/vec2.hpp>

namespace rip {
    class City {
        class impl;
        std::unique_ptr<impl> _impl;

    public:
        City(glm::uvec2 pos, std::string name);
        ~City();
        City(City &&other);

        glm::uvec2 getPos() const;
        const std::string &getName() const;

        void setName(std::string name);
    };
}

#endif //RIPOSTE_CITY_H
