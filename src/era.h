//
// Created by Caelum van Ispelen on 5/22/21.
//

#ifndef RIPOSTE_ERA_H
#define RIPOSTE_ERA_H

#include <nlohmann/json.hpp>

namespace rip {
    enum Era {
        Ancient,
        Classical,
        Medieval,
        Renaissance,
        Industrial,
        Modern,
        Future,
    };

    const char *eraID(Era era);

    Era eraFromID(const std::string &id);

NLOHMANN_JSON_SERIALIZE_ENUM(Era, {
    {Ancient, "Ancient"},
    {Classical, "Classical"},
    {Medieval, "Medieval"},
    {Renaissance, "Renaissance"},
    {Industrial, "Industrial"},
    {Modern, "Modern"},
    {Future, "Future"}
});
}

#endif //RIPOSTE_ERA_H
